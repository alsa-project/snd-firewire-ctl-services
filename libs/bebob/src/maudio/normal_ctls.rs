// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use core::card_cntr;
use core::elem_value_accessor::ElemValueAccessor;

use ta1394::{AvcAddr, Ta1394Avc};
use ta1394::{AvcOp, AvcControl};
use ta1394::general::VendorDependent;
use ta1394::audio::{AUDIO_SUBUNIT_0_ADDR, CtlAttr, AudioCh, AudioProcessing, ProcessingCtl, AudioFeature, FeatureCtl, AudioSelector};

use super::super::BebobAvc;
use super::super::model::{IN_METER_NAME, OUT_METER_NAME, OUT_SRC_NAME, OUT_VOL_NAME, HP_SRC_NAME};

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
    const METER_TLV: &'a [u32] = &[5, 8, -14400i32 as u32, 0];

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
                    ElemValueAccessor::<u32>::get_val(new, |val| {
                        let switch = SwitchState::from(val as u8);
                        let mut op = LedSwitch::new(&avc.company_id, switch);
                        avc.control(&AvcAddr::Unit, &mut op, FCP_TIMEOUT_MS)?;
                        *s = switch;
                        Ok(())
                    })?;
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
        let mut quadlet = [0;4];
        ElemValueAccessor::<i32>::set_vals(elem_value, labels.len(), |idx| {
            let pos = (offset + idx) * 4;
            quadlet.copy_from_slice(&self.cache[pos..(pos + 4)]);
            Ok(i32::from_be_bytes(quadlet))
        }).unwrap();
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
                    ElemValueAccessor::<u32>::set_val(elem_value, || Ok(u8::from(*s) as u32))?;
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            Self::ROTARY0_NAME => {
                if let Some(rotary0) = &self.rotary0 {
                    ElemValueAccessor::<i32>::set_val(elem_value, || Ok(*rotary0 as i32))?;
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            Self::ROTARY1_NAME => {
                if let Some(rotary1) = &self.rotary1 {
                    ElemValueAccessor::<i32>::set_val(elem_value, || Ok(*rotary1 as i32))?;
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            Self::SYNC_STATUS_NAME => {
                if let Some(sync_status) = &self.sync_status {
                    ElemValueAccessor::<bool>::set_val(elem_value, || Ok(*sync_status))?;
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            _ => Ok(false),
        }
    }
}

fn get_fb_id_and_ch(fb_ids: &[u8], target: usize) -> (u8, u8) {
    let pos = target / 2;
    let mut val = u8::MAX;
    let mut left = 0;
    for i in 0..(pos + 1) {
        if val == fb_ids[i] {
            left += 2;
        } else {
            val = fb_ids[i];
            left = 0;
        }
    }
    (val, left + (target % 2) as u8)
}

pub struct MixerCtl<'a> {
    dst_fb_ids: &'a [u8],
    dst_labels: &'a [&'a str],

    phys_src_fb_ids: &'a [u8],
    phys_src_labels: &'a [&'a str],

    stream_src_fb_ids: &'a [u8],
    stream_src_labels: &'a [&'a str],
}

const ON: i16 = 0x0000;
const OFF: i16 = (0x8000 as u16) as i16;

impl<'a> MixerCtl<'a> {
    const MIXER_SRC_NAME: &'a str = "mixer-source";

    pub fn new(dst_fb_ids: &'a [u8], dst_labels: &'a [&'a str],
               phys_src_fb_ids: &'a [u8], phys_src_labels: &'a [&'a str],
               stream_src_fb_ids: &'a [u8], stream_src_labels: &'a [&'a str])
        -> Self
    {
        assert_eq!(dst_fb_ids.len(), dst_labels.len());
        assert_eq!(phys_src_fb_ids.len(), phys_src_labels.len());
        assert_eq!(stream_src_fb_ids.len(), stream_src_labels.len());

        MixerCtl {dst_fb_ids, dst_labels, phys_src_fb_ids, phys_src_labels, stream_src_fb_ids, stream_src_labels}
    }

    pub fn load(&mut self, avc: &BebobAvc, card_cntr: &mut card_cntr::CardCntr)
        -> Result<(), Error>
    {
        (0..self.dst_fb_ids.len()).take(self.stream_src_fb_ids.len()).try_for_each(|i| {
            let (dst_fb, dst_ch) = get_fb_id_and_ch(self.dst_fb_ids, i * 2);
            let (src_fb, src_ch) = get_fb_id_and_ch(self.stream_src_fb_ids, i * 2);
            let mut op = AudioProcessing::new(dst_fb, CtlAttr::Current, src_fb,
                                    AudioCh::Each(src_ch), AudioCh::Each(dst_ch),
                                    ProcessingCtl::Mixer(vec![ON]));
            avc.control(&AUDIO_SUBUNIT_0_ADDR, &mut op, FCP_TIMEOUT_MS)
        })?;

        let src_count = self.stream_src_labels.len() + self.phys_src_labels.len();

        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Mixer,
                                                   0, 0, Self::MIXER_SRC_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, self.dst_labels.len(), src_count, true)?;

        Ok(())
    }

    pub fn read(&mut self, avc: &BebobAvc, elem_id: &alsactl::ElemId, elem_value: &mut alsactl::ElemValue)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::MIXER_SRC_NAME => {
                let dst = elem_id.get_index() as usize;
                let (dst_fb, dst_ch) = get_fb_id_and_ch(self.dst_fb_ids, dst * 2);

                let mut src_fb_ids = Vec::new();
                src_fb_ids.extend_from_slice(&self.stream_src_fb_ids);
                src_fb_ids.extend_from_slice(&self.phys_src_fb_ids);

                ElemValueAccessor::<bool>::set_vals(elem_value, src_fb_ids.len(), |idx| {
                    let (src_fb, src_ch) = get_fb_id_and_ch(&src_fb_ids, idx * 2);
                    let mut op = AudioProcessing::new(dst_fb, CtlAttr::Current, src_fb,
                                                      AudioCh::Each(src_ch), AudioCh::Each(dst_ch),
                                                      ProcessingCtl::Mixer(vec![-1]));
                    avc.status(&AUDIO_SUBUNIT_0_ADDR, &mut op, FCP_TIMEOUT_MS)?;
                    if let ProcessingCtl::Mixer(data) = op.ctl {
                        Ok(data[0] == ON)
                    } else {
                        unreachable!();
                    }
                })?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub fn write(&mut self, avc: &BebobAvc, elem_id: &alsactl::ElemId, old: &alsactl::ElemValue,
                 new: &alsactl::ElemValue)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::MIXER_SRC_NAME => {
                let dst = elem_id.get_index() as usize;
                let (dst_fb, dst_ch) = get_fb_id_and_ch(self.dst_fb_ids, dst * 2);

                let mut src_fb_ids = Vec::new();
                src_fb_ids.extend_from_slice(&self.stream_src_fb_ids);
                src_fb_ids.extend_from_slice(&self.phys_src_fb_ids);

                ElemValueAccessor::<bool>::get_vals(new, old, src_fb_ids.len(), |idx, val| {
                    // Left channel is a representative.
                    let (src_fb, src_ch) = get_fb_id_and_ch(&src_fb_ids, idx * 2);
                    let mut op = AudioProcessing::new(dst_fb, CtlAttr::Current, src_fb,
                                            AudioCh::Each(src_ch), AudioCh::Each(dst_ch),
                                            ProcessingCtl::Mixer(vec![if val { ON } else { OFF }]));
                    avc.control(&AUDIO_SUBUNIT_0_ADDR, &mut op, FCP_TIMEOUT_MS)
                })?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

pub struct InputCtl<'a> {
    phys_fb_ids: &'a [u8],
    phys_labels: &'a [&'a str],
    stream_fb_ids: &'a [u8],
    stream_labels: &'a [&'a str],
}

const GAIN_MIN: i32 = i16::MIN as i32;
const GAIN_MAX: i32 = 0;
const GAIN_STEP: i32 = 256;
const GAIN_TLV: &[u32] = &[5, 8, -12800i32 as u32, 0];

const PAN_MIN: i32 = i16::MIN as i32;
const PAN_MAX: i32 = i16::MAX as i32;
const PAN_STEP: i32 = 256;
const PAN_TLV: &[u32] = &[5, 8, -12800i32 as u32, 12800];

impl<'a> InputCtl<'a> {
    const PHYS_GAIN_NAME: &'a str = "phys-in-gain";
    const PHYS_BALANCE_NAME: &'a str = "phys-in-balance";
    const STREAM_GAIN_NAME: &'a str = "stream-in-gain";

    pub fn new(phys_fb_ids: &'a [u8], phys_labels: &'a [&'a str],
               stream_fb_ids: &'a [u8], stream_labels: &'a [&'a str])
        -> Self
    {
        assert_eq!(phys_fb_ids.len(), phys_labels.len());
        assert_eq!(stream_fb_ids.len(), stream_labels.len());

        InputCtl {
            phys_fb_ids,
            phys_labels,
            stream_fb_ids,
            stream_labels,
        }
    }

    pub fn load(&mut self, _: &BebobAvc, card_cntr: &mut card_cntr::CardCntr) -> Result<(), Error> {
        // For gain of physical inputs.
        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Mixer,
                                                   0, 0, Self::PHYS_GAIN_NAME, 0);
        let len = 2 * self.phys_labels.len();
        let _ = card_cntr.add_int_elems(&elem_id, 1, GAIN_MIN, GAIN_MAX, GAIN_STEP, len, Some(GAIN_TLV), true)?;

        // For balance of physical inputs.
        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Mixer,
                                                   0, 0, Self::PHYS_BALANCE_NAME, 0);
        let len = 2 * self.phys_labels.len();
        let _ = card_cntr.add_int_elems(&elem_id, 1, PAN_MIN, PAN_MAX, PAN_STEP, len, Some(PAN_TLV), true)?;

        // For gain of stream inputs.
        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Mixer,
                                                   0, 0, Self::STREAM_GAIN_NAME, 0);
        let len = 2 * self.stream_labels.len();
        let _ = card_cntr.add_int_elems(&elem_id, 1, GAIN_MIN, GAIN_MAX, GAIN_STEP, len, Some(GAIN_TLV), true)?;

        // Balance of stream inputs is not available.

        Ok(())
    }

    pub fn read(&mut self, avc: &BebobAvc, elem_id: &alsactl::ElemId, elem_value: &mut alsactl::ElemValue)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::PHYS_GAIN_NAME => self.read_gain(avc, elem_value, self.phys_fb_ids),
            Self::PHYS_BALANCE_NAME => self.read_balance(avc, elem_value, self.phys_fb_ids),
            Self::STREAM_GAIN_NAME => self.read_gain(avc, elem_value, self.stream_fb_ids),
            _ => Ok(false),
        }
    }

    pub fn write(&mut self, avc: &BebobAvc, elem_id: &alsactl::ElemId, old: &alsactl::ElemValue,
                 new: &alsactl::ElemValue)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::PHYS_GAIN_NAME => self.write_gain(avc, old, new, self.phys_fb_ids),
            Self::PHYS_BALANCE_NAME => self.write_balance(avc, old, new, self.phys_fb_ids),
            Self::STREAM_GAIN_NAME => self.write_gain(avc, old, new, self.stream_fb_ids),
            _ => Ok(false),
        }
    }

    pub fn read_gain(&mut self, avc: &BebobAvc, elem_value: &mut alsactl::ElemValue, fb_ids: &[u8])
        -> Result<bool, Error>
    {
        ElemValueAccessor::<i32>::set_vals(elem_value, fb_ids.len(), |idx| {
            let (fb_id, ch) = get_fb_id_and_ch(fb_ids, idx);
            let mut op = AudioFeature::new(fb_id, CtlAttr::Current, AudioCh::Each(ch),
                                           FeatureCtl::Volume(vec![-1]));
            avc.status(&AUDIO_SUBUNIT_0_ADDR, &mut op, FCP_TIMEOUT_MS)?;
            if let FeatureCtl::Volume(data) = op.ctl {
                Ok(data[0] as i32)
            } else {
                unreachable!();
            }
        })?;
        Ok(true)
    }

    pub fn write_gain(&mut self, avc: &BebobAvc, old: &alsactl::ElemValue, new: &alsactl::ElemValue,
                      fb_ids: &[u8])
        -> Result<bool, Error>
    {
        ElemValueAccessor::<i32>::get_vals(new, old, fb_ids.len(), |idx, val| {
            let (fb_id, ch) = get_fb_id_and_ch(fb_ids, idx);
            let mut op = AudioFeature::new(fb_id, CtlAttr::Current, AudioCh::Each(ch),
                                           FeatureCtl::Volume(vec![val as i16]));
            avc.control(&AUDIO_SUBUNIT_0_ADDR, &mut op, FCP_TIMEOUT_MS)
        })?;
        Ok(true)
    }

    fn read_balance(&mut self, avc: &BebobAvc, elem_value: &mut alsactl::ElemValue, fb_ids: &[u8])
        -> Result<bool, Error>
    {
        ElemValueAccessor::<i32>::set_vals(elem_value, fb_ids.len(), |idx| {
            let (fb_id, ch) = get_fb_id_and_ch(fb_ids, idx);
            let mut op = AudioFeature::new(fb_id, CtlAttr::Current, AudioCh::Each(ch),
                                           FeatureCtl::LrBalance(-1));
            avc.status(&AUDIO_SUBUNIT_0_ADDR, &mut op, FCP_TIMEOUT_MS)?;
            if let FeatureCtl::LrBalance(val) = op.ctl {
                Ok(val as i32)
            } else {
                unreachable!();
            }
        })?;
        Ok(true)
    }

    fn write_balance(&mut self, avc: &BebobAvc, old: &alsactl::ElemValue, new: &alsactl::ElemValue,
                     fb_ids: &[u8])
        -> Result<bool, Error>
    {
        ElemValueAccessor::<i32>::get_vals(new, old, fb_ids.len(), |idx, val| {
            let (fb_id, ch) = get_fb_id_and_ch(fb_ids, idx);
            let mut op = AudioFeature::new(fb_id, CtlAttr::Current, AudioCh::Each(ch),
                                           FeatureCtl::LrBalance(val as i16));
            avc.control(&AUDIO_SUBUNIT_0_ADDR, &mut op, FCP_TIMEOUT_MS)
        })?;
        Ok(true)
    }
}

const VOL_MIN: i32 = i16::MIN as i32;
const VOL_MAX: i32 = 0;
const VOL_STEP: i32 = 256;
const VOL_TLV: &[u32] = &[5, 8, -12800i32 as u32, 0];

pub struct AuxCtl<'a> {
    out_fb_id: u8,
    src_fb_ids: Vec<u8>,
    src_labels: Vec<&'a str>,
}

impl<'a> AuxCtl<'a> {
    const AUX_SRC_GAIN_NAME: &'a str = "aux-source-gain";
    const AUX_OUT_VOLUME_NAME: &'a str = "aux-output-volume";

    pub fn new(out_fb_id: u8, phys_src_fb_ids: &'a [u8], phys_src_labels: &'a [&'a str],
               stream_src_fb_ids: &'a [u8], stream_src_labels: &'a [&'a str])
        -> Self
    {
        assert_eq!(phys_src_fb_ids.len(), phys_src_labels.len());
        assert_eq!(stream_src_fb_ids.len(), stream_src_labels.len());

        let mut src_fb_ids = phys_src_fb_ids.to_vec();
        src_fb_ids.extend(stream_src_fb_ids);
        let mut src_labels = phys_src_labels.to_vec();
        src_labels.extend(stream_src_labels);

        AuxCtl {
            out_fb_id: out_fb_id,
            src_fb_ids: src_fb_ids,
            src_labels: src_labels,
        }
    }

    pub fn load(&mut self, _: &BebobAvc, card_cntr: &mut card_cntr::CardCntr) -> Result<(), Error> {
        // For gain of sources to aux.
        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Mixer,
                                                   0, 0, Self::AUX_SRC_GAIN_NAME, 0);
        let len = 2 * self.src_labels.len();
        let _ = card_cntr.add_int_elems(&elem_id, 1, GAIN_MIN, GAIN_MAX, GAIN_STEP, len, Some(GAIN_TLV), true)?;

        // For volume of output from aux.
        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Mixer,
                                                   0, 0, Self::AUX_OUT_VOLUME_NAME, 0);
        let _ = card_cntr.add_int_elems(&elem_id, 1, VOL_MIN, VOL_MAX, VOL_STEP, 2, Some(VOL_TLV), true)?;

        Ok(())
    }

    pub fn read(&mut self, avc: &BebobAvc, elem_id: &alsactl::ElemId, elem_value: &mut alsactl::ElemValue)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::AUX_SRC_GAIN_NAME => {
                ElemValueAccessor::<i32>::set_vals(elem_value, self.src_labels.len(), |idx| {
                    let (fb_id, ch) = get_fb_id_and_ch(&self.src_fb_ids, idx);
                    let mut op = AudioFeature::new(fb_id, CtlAttr::Current, AudioCh::Each(ch as u8),
                                                   FeatureCtl::Volume(vec![-1]));
                    avc.status(&AUDIO_SUBUNIT_0_ADDR, &mut op, FCP_TIMEOUT_MS)?;
                    if let FeatureCtl::Volume(data) = op.ctl {
                        Ok(data[0] as i32)
                    } else {
                        unreachable!();
                    }
                })?;
                Ok(true)
            }
            Self::AUX_OUT_VOLUME_NAME => {
                ElemValueAccessor::<i32>::set_vals(elem_value, 2, |idx| {
                    let mut op = AudioFeature::new(self.out_fb_id, CtlAttr::Current, AudioCh::Each(idx as u8),
                                                   FeatureCtl::Volume(vec![-1]));
                    avc.status(&AUDIO_SUBUNIT_0_ADDR, &mut op, FCP_TIMEOUT_MS)?;
                    if let FeatureCtl::Volume(data) = op.ctl {
                        Ok(data[0] as i32)
                    } else {
                        unreachable!();
                    }
                })?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub fn write(&mut self, avc: &BebobAvc, elem_id: &alsactl::ElemId,
                 old: &alsactl::ElemValue, new: &alsactl::ElemValue)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::AUX_SRC_GAIN_NAME => {
                ElemValueAccessor::<i32>::get_vals(new, old, self.src_labels.len(), |idx, val| {
                    let (fb_id, ch) = get_fb_id_and_ch(&self.src_fb_ids, idx);
                    let mut op = AudioFeature::new(fb_id, CtlAttr::Current, AudioCh::Each(ch as u8),
                                                   FeatureCtl::Volume(vec![val as i16]));
                    avc.control(&AUDIO_SUBUNIT_0_ADDR, &mut op, FCP_TIMEOUT_MS)
                })?;
                Ok(true)
            }
            Self::AUX_OUT_VOLUME_NAME => {
                ElemValueAccessor::<i32>::get_vals(new, old, 2, |idx, val| {
                    let mut op = AudioFeature::new(self.out_fb_id, CtlAttr::Current, AudioCh::Each(idx as u8),
                                                   FeatureCtl::Volume(vec![val as i16]));
                    avc.control(&AUDIO_SUBUNIT_0_ADDR, &mut op, FCP_TIMEOUT_MS)
                })?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

pub struct OutputCtl<'a> {
    labels: &'a [&'a str],
    vol_fb_ids: &'a [u8],
    src_fb_ids: &'a [u8],
}

impl<'a> OutputCtl<'a> {
    const OUT_SRC_LABELS: &'a [&'a str] = &["mixer", "aux-1/2"];

    pub fn new(labels: &'a [&'a str], vol_fb_ids: &'a [u8], src_fb_ids: &'a [u8]) -> Self {
        assert_eq!(labels.len(), vol_fb_ids.len());
        assert_eq!(labels.len(), src_fb_ids.len());
        OutputCtl {
            labels,
            vol_fb_ids,
            src_fb_ids,
        }
    }

    pub fn load(&mut self, _: &BebobAvc, card_cntr: &mut card_cntr::CardCntr) -> Result<(), Error> {
        // For source of output.
        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Mixer, 0, 0, OUT_SRC_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, self.labels.len(), Self::OUT_SRC_LABELS, None, true)?;

        // For volume of output.
        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Mixer, 0, 0, OUT_VOL_NAME, 0);
        let len = 2 * self.labels.len();
        let _ = card_cntr.add_int_elems(&elem_id, 1, VOL_MIN, VOL_MAX, VOL_STEP, len, Some(VOL_TLV), true)?;

        Ok(())
    }

    pub fn read(&mut self, avc: &BebobAvc, elem_id: &alsactl::ElemId, elem_value: &mut alsactl::ElemValue)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            OUT_SRC_NAME => {
                ElemValueAccessor::<u32>::set_vals(elem_value, self.labels.len(), |idx| {
                    let mut op = AudioSelector::new(self.src_fb_ids[idx], CtlAttr::Current, 0xff);
                    avc.status(&AUDIO_SUBUNIT_0_ADDR, &mut op, FCP_TIMEOUT_MS)?;
                    Ok(op.input_plug_id as u32)
                })?;
                Ok(true)
            }
            OUT_VOL_NAME => {
                ElemValueAccessor::<i32>::set_vals(elem_value, self.labels.len(), |idx| {
                    let ch = (idx % 2) as u8;
                    let mut op = AudioFeature::new(self.vol_fb_ids[idx / 2], CtlAttr::Current, AudioCh::Each(ch),
                                                   FeatureCtl::Volume(vec![-1]));
                    avc.status(&AUDIO_SUBUNIT_0_ADDR, &mut op, FCP_TIMEOUT_MS)?;
                    if let FeatureCtl::Volume(data) = op.ctl {
                        Ok(data[0] as i32)
                    } else {
                        unreachable!();
                    }
                })?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub fn write(&mut self, avc: &BebobAvc, elem_id: &alsactl::ElemId,
                 old: &alsactl::ElemValue, new: &alsactl::ElemValue)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            OUT_SRC_NAME => {
                ElemValueAccessor::<u32>::get_vals(new, old, self.labels.len(), |idx, val| {
                    let mut op = AudioSelector::new(self.src_fb_ids[idx], CtlAttr::Current, val as u8);
                    avc.control(&AUDIO_SUBUNIT_0_ADDR, &mut op, FCP_TIMEOUT_MS)
                })?;
                Ok(true)
            }
            OUT_VOL_NAME => {
                ElemValueAccessor::<u32>::get_vals(new, old, self.labels.len(), |idx, val| {
                    let ch = (idx % 2) as u8;
                    let mut op = AudioFeature::new(self.vol_fb_ids[idx / 2], CtlAttr::Current, AudioCh::Each(ch),
                                                   FeatureCtl::Volume(vec![val as i16]));
                    avc.control(&AUDIO_SUBUNIT_0_ADDR, &mut op, FCP_TIMEOUT_MS)
                })?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

pub struct HpCtl<'a> {
    vol_fb_id: u8,
    src_fb_id: u8,
    src_labels: &'a [&'a str],
    pub measure_elems: Vec<alsactl::ElemId>,
}

impl<'a> HpCtl<'a> {
    const HP_VOL_NAME: &'a str = "headphone-volume";

    pub fn new(vol_fb_id: u8, src_fb_id: u8, src_labels: &'a [&'a str]) -> Self {
        HpCtl {
            vol_fb_id,
            src_fb_id,
            src_labels,
            measure_elems: Vec::new(),
        }
    }

    pub fn load(&mut self, _: &BebobAvc, card_cntr: &mut card_cntr::CardCntr) -> Result<(), Error> {
        // For source of headphone.
        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Mixer, 0, 0, HP_SRC_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &self.src_labels, None, true)?;

        // For volume of headphone.
        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Mixer, 0, 0, Self::HP_VOL_NAME, 0);
        let elem_id_list = card_cntr.add_int_elems(&elem_id, 1, VOL_MIN, VOL_MAX, VOL_STEP, 2, Some(VOL_TLV), true)?;
        self.measure_elems.push(elem_id_list[0].clone());

        Ok(())
    }

    pub fn read(&mut self, avc: &BebobAvc, elem_id: &alsactl::ElemId, elem_value: &mut alsactl::ElemValue)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            HP_SRC_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let mut op = AudioSelector::new(self.src_fb_id, CtlAttr::Current, 0xff);
                    avc.status(&AUDIO_SUBUNIT_0_ADDR, &mut op, FCP_TIMEOUT_MS)?;
                    Ok(op.input_plug_id as u32)
                })?;
                Ok(true)
            }
            Self::HP_VOL_NAME => {
                ElemValueAccessor::<i32>::set_vals(elem_value, 2, |idx| {
                    let mut op = AudioFeature::new(self.vol_fb_id, CtlAttr::Current, AudioCh::Each(idx as u8),
                                                   FeatureCtl::Volume(vec![-1]));
                    avc.status(&AUDIO_SUBUNIT_0_ADDR, &mut op, FCP_TIMEOUT_MS)?;
                    if let FeatureCtl::Volume(data) = op.ctl {
                        Ok(data[0] as i32)
                    } else {
                        unreachable!();
                    }
                })?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub fn write(&mut self, avc: &BebobAvc, elem_id: &alsactl::ElemId,
                 old: &alsactl::ElemValue, new: &alsactl::ElemValue)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            HP_SRC_NAME => {
                ElemValueAccessor::<u32>::get_val(new, |val| {
                    let mut op = AudioSelector::new(self.src_fb_id, CtlAttr::Current, val as u8);
                    avc.control(&AUDIO_SUBUNIT_0_ADDR, &mut op, FCP_TIMEOUT_MS)
                })?;
                Ok(true)
            }
            Self::HP_VOL_NAME => {
                ElemValueAccessor::<i32>::get_vals(new, old, 2, |idx, val| {
                    let mut op = AudioFeature::new(self.vol_fb_id, CtlAttr::Current, AudioCh::Each(idx as u8),
                                                   FeatureCtl::Volume(vec![val as i16]));
                    avc.control(&AUDIO_SUBUNIT_0_ADDR, &mut op, FCP_TIMEOUT_MS)
                })?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
