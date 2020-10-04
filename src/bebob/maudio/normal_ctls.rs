// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use alsactl::{ElemValueExt, ElemValueExtManual};

use crate::card_cntr;

use crate::ta1394::{AvcAddr, Ta1394Avc};
use crate::ta1394::{AvcOp, AvcControl};
use crate::ta1394::general::VendorDependent;

use crate::bebob::BebobAvc;
use crate::bebob::model::{IN_METER_NAME, OUT_METER_NAME};

use super::common_proto::CommonProto;

impl CommonProto for hinawa::FwReq {}

pub const FCP_TIMEOUT_MS: u32 = 100;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SwitchState {
    Off,
    A,
    B,
}

impl From<SwitchState> for u8 {
    fn from(state: SwitchState) -> Self {
        match state {
            SwitchState::Off => 0x00,
            SwitchState::A => 0x01,
            SwitchState::B => 0x02,
        }
    }
}

impl From<u8> for SwitchState {
    fn from(val: u8) -> Self {
        match val {
            0x01 => SwitchState::A,
            0x02 => SwitchState::B,
            _ => SwitchState::Off,
        }
    }
}

struct LedSwitch{
    state: SwitchState,
    op: VendorDependent,
}

impl LedSwitch {
    pub fn new(company_id: &[u8;3], state: SwitchState) -> Self {
        LedSwitch{
            state,
            op: VendorDependent{
                company_id: *company_id,
                data: Vec::new(),
            },
        }
    }
}

impl AvcOp for LedSwitch {
    const OPCODE: u8 = VendorDependent::OPCODE;
}

impl AvcControl for LedSwitch {
    fn build_operands(&mut self, addr: &AvcAddr, operands: &mut Vec<u8>) -> Result<(), Error> {
        self.op.data.extend_from_slice(&[0x02, 0x00, 0x01, self.state.into(), 0xff, 0xff]);
        AvcControl::build_operands(&mut self.op, addr, operands)
    }

    fn parse_operands(&mut self, addr: &AvcAddr, operands: &[u8]) -> Result<(), Error> {
        AvcControl::parse_operands(&mut self.op, addr, operands)
    }
}

pub struct MeterCtl<'a> {
    pub measure_elems: Vec<alsactl::ElemId>,

    in_meter_labels: &'a [&'a str],
    stream_meter_labels: &'a [&'a str],
    out_meter_labels: &'a [&'a str],

    cache: Vec<u8>,
    switch: Option<SwitchState>,
    rotary0: Option<i32>,
    rotary1: Option<i32>,
    sync_status: Option<bool>,
}

impl<'a> MeterCtl<'a> {
    const HP_OUT_LABELS: &'a [&'a str] = &["headphone-1", "headphone-2"];
    const AUX_OUT_LABELS: &'a [&'a str] = &["aux-1", "aux-2"];

    const STREAM_METER_NAME: &'a str = "stream-meters";
    const HP_OUT_METER_NAME: &'a str = "headphone-meters";
    const AUX_OUT_METER_NAME: &'a str = "aux-meters";

    const SWITCH_NAME: &'a str = "Switch";
    const ROTARY0_NAME: &'a str = "Rotary0";
    const ROTARY1_NAME: &'a str = "Rotary1";
    const SYNC_STATUS_NAME: &'a str = "Sync status";

    const SWITCH_LABELS: &'a [&'a str] = &["Off", "A", "B"];

    const ROTARY_MIN: i32 = i16::MIN as i32;
    const ROTARY_MAX: i32 = 0;
    const ROTARY_STEP: i32 = 256;

    const METER_MIN: i32 = 0;
    const METER_MAX: i32 = i32::MAX;
    const METER_STEP: i32 = 256;
    const METER_TLV: &'a [i32] = &[5, 8, -14400, 0];

    pub fn new(in_meter_labels: &'a [&'a str], stream_meter_labels: &'a [&'a str], out_meter_labels: &'a [&'a str],
               has_switch: bool, rotary_count: usize, has_sync_status: bool)
        -> Self
    {
        let mut len = in_meter_labels.len() + out_meter_labels.len();
        if stream_meter_labels.len() > 0 {
            len += stream_meter_labels.len();
        } else {
            // Plus headphone-1 and -2, aux-1 and -2.
            len += 4;
        }
        if has_switch || rotary_count > 0 || has_sync_status {
            len += 1;
        }
        len *= std::mem::size_of::<i32>();
        let cache = vec![0;len];

        let switch = if has_switch { Some(SwitchState::Off) } else { None };
        let rotary0 = if rotary_count == 1 { Some(0) } else { None };
        let rotary1 = if rotary_count == 2 { Some(0) } else { None };
        let sync_status = if has_sync_status { Some(false) } else { None };

        MeterCtl {
            in_meter_labels,
            stream_meter_labels,
            out_meter_labels,
            cache: cache,
            switch,
            rotary0,
            rotary1,
            sync_status,
            measure_elems: Vec::new(),
        }
    }

    fn add_meter_elem(&mut self, card_cntr: &mut card_cntr::CardCntr, name: &str, labels: &[&str])
        -> Result<(), Error>
    {
        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Mixer, 0, 0, name, 0);
        let mut elem_id_list = card_cntr.add_int_elems(&elem_id, 1, Self::METER_MIN, Self::METER_MAX, Self::METER_STEP,
                                                       labels.len(), Some(Self::METER_TLV), false)?;
        self.measure_elems.append(&mut elem_id_list);
        Ok(())
    }

    pub fn load(&mut self, unit: &hinawa::SndUnit, avc: &BebobAvc, req: &hinawa::FwReq,
                card_cntr: &mut card_cntr::CardCntr)
        -> Result<(), Error>
    {
        self.measure_states(unit, avc, req)?;

        // For metering.
        self.add_meter_elem(card_cntr, IN_METER_NAME, self.in_meter_labels)?;
        self.add_meter_elem(card_cntr, OUT_METER_NAME, self.out_meter_labels)?;

        if self.stream_meter_labels.len() > 0 {
            self.add_meter_elem(card_cntr, Self::STREAM_METER_NAME, self.stream_meter_labels)?;
        } else {
            self.add_meter_elem(card_cntr, Self::HP_OUT_METER_NAME, Self::HP_OUT_LABELS)?;
            self.add_meter_elem(card_cntr, Self::AUX_OUT_METER_NAME, Self::AUX_OUT_LABELS)?;
        }

        // For switch button.
        if self.switch.is_some() {
            let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Card,
                                                       0, 0, Self::SWITCH_NAME, 0);
            let mut elem_id_list = card_cntr.add_enum_elems(&elem_id, 1, 1, Self::SWITCH_LABELS, None, true)?;
            self.measure_elems.append(&mut elem_id_list);
        }

        // For rotary knob.
        if self.rotary0.is_some() {
            let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Card,
                                                       0, 0, Self::ROTARY0_NAME, 0);
            let mut elem_id_list = card_cntr.add_int_elems(&elem_id, 1,
                                                Self::ROTARY_MIN, Self::ROTARY_MAX, Self::ROTARY_STEP,
                                                1, None, false)?;
            self.measure_elems.append(&mut elem_id_list);
        }

        if self.rotary1.is_some() {
            let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Card,
                                                       0, 0, Self::ROTARY1_NAME, 0);
            let mut elem_id_list = card_cntr.add_int_elems(&elem_id, 1,
                                                Self::ROTARY_MIN, Self::ROTARY_MAX, Self::ROTARY_STEP,
                                                1, None, false)?;
            self.measure_elems.append(&mut elem_id_list);
        }

        // For sync status.
        if self.sync_status.is_some() {
            let elem_id = alsactl::ElemId::new_by_name( alsactl::ElemIfaceType::Card,
                                                        0, 0, Self::SYNC_STATUS_NAME, 0);
            let mut elem_id_list = card_cntr.add_bool_elems(&elem_id, 1, 1, false)?;
            self.measure_elems.append(&mut elem_id_list);
        }

        Ok(())
    }

    pub fn read(&mut self, elem_id: &alsactl::ElemId, elem_value: &mut alsactl::ElemValue)
        -> Result<bool, Error>
    {
        self.measure_elem(elem_id, elem_value)
    }

    pub fn write(&mut self, avc: &BebobAvc, elem_id: &alsactl::ElemId, _: &alsactl::ElemValue, new: &alsactl::ElemValue)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::SWITCH_NAME => {
                if let Some(s) = &mut self.switch {
                    let mut val = [0];
                    new.get_enum(&mut val);
                    let switch = SwitchState::from(val[0] as u8);
                    let mut op = LedSwitch::new(&avc.company_id, switch);
                    avc.control(&AvcAddr::Unit, &mut op, FCP_TIMEOUT_MS)?;
                    *s = switch;
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            _ => Ok(false),
        }
    }

    pub fn measure_states(&mut self, unit: &hinawa::SndUnit, avc: &BebobAvc, req: &hinawa::FwReq)
        -> Result<(), Error>
    {
        let mut frames = vec![0;self.cache.len()];
        req.read_meters(unit, &mut frames)?;

        if let Some(s) = &mut self.switch {
            let pos = self.cache.len() - 4;

            if (self.cache[pos] ^ frames[pos]) & 0xf0 > 0 {
                if self.cache[pos] & 0xf0 > 0 {
                    let switch = match s {
                        SwitchState::Off => SwitchState::A,
                        SwitchState::A => SwitchState::B,
                        SwitchState::B => SwitchState::Off,
                    };
                    let mut op = LedSwitch::new(&avc.company_id, switch);
                    avc.control(&AvcAddr::Unit, &mut op, FCP_TIMEOUT_MS)?;
                    *s = switch;
                }
            }
        }

        if self.rotary0.is_some() || self.rotary1.is_some() {
            let pos = self.cache.len() - 3;
            let prev = self.cache[pos];
            let curr = frames[pos];

            if let Some(rotary0) = &mut self.rotary0 {
                if (prev ^ curr) & 0x0f > 0 {
                    if prev & 0x0f == 0x01 {
                        if *rotary0 <= Self::ROTARY_MAX - Self::ROTARY_STEP {
                            *rotary0 += Self::ROTARY_STEP;
                        }
                    } else if prev & 0x0f == 0x02 {
                        if *rotary0 >= Self::ROTARY_MIN + Self::ROTARY_STEP {
                            *rotary0 -= Self::ROTARY_STEP;
                        }
                    }
                }
            }

            if let Some(rotary1) = &mut self.rotary1 {
                if (prev ^ curr) & 0xf0 > 0 {
                    if prev & 0xf0 == 0x10 {
                        if *rotary1 <= Self::ROTARY_MAX - Self::ROTARY_STEP {
                            *rotary1 += Self::ROTARY_STEP;
                        }
                    } else if prev & 0xf0 == 0x20 {
                        if *rotary1 >= Self::ROTARY_MIN + Self::ROTARY_STEP {
                            *rotary1 -= Self::ROTARY_STEP;
                        }
                    }
                }
            }
        }

        if let Some(sync_status) = &mut self.sync_status {
            let pos = self.cache.len() - 1;
            if self.cache[pos] != frames[pos] {
                *sync_status = frames[pos] > 0;
            }
        }

        self.cache.copy_from_slice(&frames);

        Ok(())
    }

    fn parse_meters(&self, elem_value: &mut alsactl::ElemValue, offset: usize, labels: &[&str]) {
        let vals = (0..labels.len()).map(|i| {
            let pos = (offset + i) * std::mem::size_of::<i32>();
            let mut quadlet = [0;std::mem::size_of::<i32>()];
            quadlet.copy_from_slice(&self.cache[pos..(pos + std::mem::size_of::<i32>())]);
            i32::from_be_bytes(quadlet)
        }).collect::<Vec<i32>>();
        elem_value.set_int(&vals);
    }

    pub fn measure_elem(&mut self, elem_id: &alsactl::ElemId, elem_value: &mut alsactl::ElemValue)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            IN_METER_NAME => {
                self.parse_meters(elem_value, 0, self.in_meter_labels);
                Ok(true)
            }
            Self::STREAM_METER_NAME => {
                let offset = self.in_meter_labels.len();
                self.parse_meters(elem_value, offset, self.stream_meter_labels);
                Ok(true)
            }
            OUT_METER_NAME => {
                let offset = self.in_meter_labels.len() + self.stream_meter_labels.len();
                self.parse_meters(elem_value, offset, self.out_meter_labels);
                Ok(true)
            }
            Self::HP_OUT_METER_NAME => {
                let offset = self.in_meter_labels.len() + self.stream_meter_labels.len() +
                             self.out_meter_labels.len();
                self.parse_meters(elem_value, offset, Self::HP_OUT_LABELS);
                Ok(true)
            }
            Self::AUX_OUT_METER_NAME => {
                let offset = self.in_meter_labels.len() + self.stream_meter_labels.len() +
                             self.out_meter_labels.len() + 2;
                self.parse_meters(elem_value, offset, Self::AUX_OUT_LABELS);
                Ok(true)
            }
            Self::SWITCH_NAME => {
                if let Some(s) = &self.switch {
                    elem_value.set_enum(&[u8::from(*s) as u32]);
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            Self::ROTARY0_NAME => {
                if let Some(rotary0) = &self.rotary0 {
                    elem_value.set_int(&[*rotary0 as i32]);
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            Self::ROTARY1_NAME => {
                if let Some(rotary1) = &self.rotary1 {
                    elem_value.set_int(&[*rotary1 as i32]);
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            Self::SYNC_STATUS_NAME => {
                if let Some(sync_status) = &self.sync_status {
                    elem_value.set_bool(&[*sync_status]);
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            _ => Ok(false),
        }
    }

}
