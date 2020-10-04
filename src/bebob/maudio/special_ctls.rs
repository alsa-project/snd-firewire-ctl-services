// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use hinawa::SndUnitExt;

use alsactl::{ElemValueExt, ElemValueExtManual};

use crate::card_cntr;

use crate::ta1394::{AvcAddr, Ta1394Avc};
use crate::ta1394::{AvcOp, AvcControl};
use crate::ta1394::general::{InputPlugSignalFormat, OutputPlugSignalFormat, VendorDependent};
use crate::ta1394::amdtp::{AmdtpEventType, AmdtpFdf, FMT_IS_AMDTP};

use crate::bebob::BebobAvc;
use crate::bebob::model::{CLK_RATE_NAME, IN_METER_NAME, OUT_METER_NAME};

use super::common_proto::{FCP_TIMEOUT_MS, CommonProto};

pub struct ClkCtl{
    supported_clk_rates: Vec<u32>,
    pub notified_elem_list: Vec<alsactl::ElemId>,
}

impl<'a> ClkCtl {
    pub fn new(is_fw1814: bool) -> Self {
        let mut supported_clk_rates = Vec::new();
        supported_clk_rates.extend_from_slice(&[32000, 44100, 48000, 88200, 96000]);
        if is_fw1814 {
            supported_clk_rates.extend_from_slice(&[176400, 192000]);
        }
        ClkCtl{
            supported_clk_rates,
            notified_elem_list: Vec::new(),
        }
    }

    pub fn load(&mut self, card_cntr: &mut card_cntr::CardCntr)
        -> Result<(), Error>
    {
        let labels = self.supported_clk_rates.iter().map(|l| l.to_string()).collect::<Vec<String>>();

        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Card, 0, 0, CLK_RATE_NAME, 0);
        let mut elem_id_list = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;
        self.notified_elem_list.append(&mut elem_id_list);

        Ok(())
    }

    pub fn read(&mut self, avc: &BebobAvc, elem_id: &alsactl::ElemId, elem_value: &mut alsactl::ElemValue)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            CLK_RATE_NAME => {
                let mut op = InputPlugSignalFormat::new(0);
                avc.status(&AvcAddr::Unit, &mut op, FCP_TIMEOUT_MS)?;

                let fdf = AmdtpFdf::from(op.fdf.as_ref());
                match self.supported_clk_rates.iter().position(|r| *r == fdf.freq) {
                    Some(p) => {
                        elem_value.set_enum(&[p as u32]);
                        Ok(true)
                    }
                    None => Ok(false),
                }
            }
            _ => Ok(false),
        }
    }

    pub fn write(&mut self, unit: &hinawa::SndUnit, avc: &BebobAvc, elem_id: &alsactl::ElemId,
                 _: &alsactl::ElemValue, new: &alsactl::ElemValue)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            CLK_RATE_NAME => {
                let mut vals = [0];
                new.get_enum(&mut vals);

                let freq = self.supported_clk_rates[vals[0] as usize];
                let fdf = AmdtpFdf::new(AmdtpEventType::Am824, false, freq);

                unit.lock()?;
                let mut op = OutputPlugSignalFormat{
                    plug_id: 0,
                    fmt: FMT_IS_AMDTP,
                    fdf: fdf.into(),
                };
                let mut res = avc.control(&AvcAddr::Unit, &mut op, FCP_TIMEOUT_MS * 2);
                if res.is_ok() {
                    let mut op = InputPlugSignalFormat{
                        plug_id: 0,
                        fmt: FMT_IS_AMDTP,
                        fdf: fdf.into(),
                    };
                    res = avc.control(&AvcAddr::Unit, &mut op, FCP_TIMEOUT_MS * 2)
                }
                unit.unlock()?;
                res.and(Ok(true))
            }
            _ => Ok(false),
        }
    }
}

struct LedSwitch{
    state: bool,
    op: VendorDependent,
}

impl LedSwitch {
    // NOTE: Unknown OUI.
    const OUI: [u8;3] = [0x03, 0x00, 0x01];

    pub fn new(state: bool) -> Self {
        LedSwitch{
            state,
            op: VendorDependent{
                company_id: Self::OUI,
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
        self.op.data.extend_from_slice(&[self.state as u8, 0xff]);
        AvcControl::build_operands(&mut self.op, addr, operands)
    }

    fn parse_operands(&mut self, addr: &AvcAddr, operands: &[u8]) -> Result<(), Error> {
        AvcControl::parse_operands(&mut self.op, addr, operands)
    }
}

pub struct MeterCtl{
    pub measure_elems: Vec<alsactl::ElemId>,
    meters: [u8; Self::METER_FRAME_SIZE],
    switch: bool,
    rotaries: [i32;3],
    sync_status: bool,
}

impl<'a> MeterCtl {
    const ROTARY0_NAME: &'a str = "rotary0";
    const ROTARY1_NAME: &'a str = "rotary1";
    const ROTARY2_NAME: &'a str = "rotary2";
    const SWITCH_NAME: &'a str = "switch";
    const SYNC_STATUS_NAME: &'a str = "Sync Status";
    const HP_OUT_METER_NAME: &'a str = "headhpone-meters";

    const IN_METER_LABELS: &'a [&'a str] = &[
        "analog-in-1", "analog-in-2", "analog-in-3", "analog-in-4",
        "analog-in-5", "analog-in-6", "analog-in-7", "analog-in-8",
        "spdif-in-1", "spdif-in-2",
        "adat-in-1", "adat-in-2", "adat-in-3", "adat-in-4",
        "adat-in-5", "adat-in-6", "adat-in-7", "adat-in-8",
    ];

    const OUT_METER_LABELS: &'a [&'a str] = &[
        "analog-out-1", "analog-out-2", "analog-out-3", "analog-out-4",
        "spdif-out-1", "spdif-out-2",
        "adat-out-1", "adat-out-2", "adat-out-3", "adat-out-4",
        "adat-out-5", "adat-out-6", "adat-out-7", "adat-out-8",
    ];

    const HP_OUT_METER_LABELS: &'a [&'a str] = &[
        "headphone-out-1", "headphone-out-2", "headphone-out-3", "headphone-out-4",
    ];

    const VAL_MIN: i32 = 0;
    const VAL_MAX: i32 = i16::MAX as i32;
    const VAL_STEP: i32 = 256;
    const VAL_TLV: &'a [i32; 4] = &[5, 8, -12800, 0];

    const METER_FRAME_SIZE: usize = 84;

    pub fn new() -> Self {
        MeterCtl{
            measure_elems: Vec::new(),
            meters: [0;Self::METER_FRAME_SIZE],
            switch: false,
            rotaries: [0;3],
            sync_status: false,
        }
    }

    fn add_meter_elem(&mut self, card_cntr: &mut card_cntr::CardCntr, name: &str, labels: &[&str])
        -> Result<(), Error>
    {
        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Mixer, 0, 0, name, 0);
        let mut elem_id_list = card_cntr.add_int_elems(&elem_id, 1, Self::VAL_MIN, Self::VAL_MAX, Self::VAL_STEP,
                                                       labels.len(), Some(Self::VAL_TLV), false)?;
        self.measure_elems.append(&mut elem_id_list);
        Ok(())
    }

    pub fn load(&mut self, unit: &hinawa::SndUnit, req: &hinawa::FwReq, avc: &BebobAvc,
                card_cntr: &mut card_cntr::CardCntr)
        -> Result<(), Error>
    {
        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Card, 0, 0, Self::ROTARY0_NAME, 0);
        let mut elem_id_list = card_cntr.add_int_elems(&elem_id, 1, Self::VAL_MIN, Self::VAL_MAX, Self::VAL_STEP,
                                                       1, None, false)?;
        self.measure_elems.append(&mut elem_id_list);

        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Card, 0, 0, Self::ROTARY1_NAME, 0);
        let mut elem_id_list = card_cntr.add_int_elems(&elem_id, 1, Self::VAL_MIN, Self::VAL_MAX, Self::VAL_STEP,
                                                       1, None, false)?;
        self.measure_elems.append(&mut elem_id_list);

        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Card, 0, 0, Self::ROTARY2_NAME, 0);
        let mut elem_id_list = card_cntr.add_int_elems(&elem_id, 1, Self::VAL_MIN, Self::VAL_MAX, Self::VAL_STEP,
                                                       1, None, false)?;
        self.measure_elems.append(&mut elem_id_list);

        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Card, 0, 0, Self::SWITCH_NAME, 0);
        let mut elem_id_list = card_cntr.add_bool_elems(&elem_id, 1, 1, false)?;
        self.measure_elems.append(&mut elem_id_list);

        self.add_meter_elem(card_cntr, IN_METER_NAME, Self::IN_METER_LABELS)?;
        self.add_meter_elem(card_cntr, OUT_METER_NAME, Self::OUT_METER_LABELS)?;
        self.add_meter_elem(card_cntr, Self::HP_OUT_METER_NAME, Self::HP_OUT_METER_LABELS)?;

        self.measure_states(unit, req, avc)?;

        Ok(())
    }

    pub fn measure_states(&mut self, unit: &hinawa::SndUnit, req: &hinawa::FwReq, avc: &BebobAvc)
        -> Result<(), Error>
    {
        let mut frames = [0;Self::METER_FRAME_SIZE];
        req.read_meters(unit, &mut frames)?;

        if self.meters[0] == 0x01 && frames[0] == 0x00 {
            let mut op = LedSwitch::new(!self.switch);
            avc.control(&AvcAddr::Unit, &mut op, FCP_TIMEOUT_MS)?;
            self.switch = !self.switch;
        }

        let meters = &self.meters;
        self.rotaries.iter_mut().enumerate().for_each(|(i, v)| {
            if meters[i + 1] ^ frames[i + 1] > 0 {
                let delta = match meters[i + 1] {
                    0x01 => Self::VAL_STEP,
                    0x02 => -Self::VAL_STEP,
                    _ => 0,
                };
                if *v + delta < Self::VAL_MIN {
                    *v = Self::VAL_MIN;
                } else if *v + delta > Self::VAL_MAX {
                    *v = Self::VAL_MAX;
                } else {
                    *v += delta;
                }
            }
        });

        if self.meters[83] != frames[83] {
            self.sync_status = frames[83] != 0;
        }

        self.meters.copy_from_slice(&frames);
        Ok(())
    }

    fn parse_meters(&self, elem_value: &mut alsactl::ElemValue, offset: usize, labels: &[&str]) {
        let vals = (0..labels.len()).map(|i| {
            let pos = (offset + i) * std::mem::size_of::<i16>();
            let mut doublet = [0;std::mem::size_of::<i16>()];
            doublet.copy_from_slice(&self.meters[pos..(pos + std::mem::size_of::<i16>())]);
            i16::from_be_bytes(doublet) as i32
        }).collect::<Vec<i32>>();
        elem_value.set_int(&vals);
    }

    pub fn measure_elem(&mut self, elem_id: &alsactl::ElemId, elem_value: &mut alsactl::ElemValue)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::ROTARY0_NAME => {
                elem_value.set_int(&[self.rotaries[0]]);
                Ok(true)
            }
            Self::ROTARY1_NAME => {
                elem_value.set_int(&[self.rotaries[1]]);
                Ok(true)
            }
            Self::ROTARY2_NAME => {
                elem_value.set_int(&[self.rotaries[2]]);
                Ok(true)
            }
            Self::SWITCH_NAME => {
                elem_value.set_bool(&[self.switch]);
                Ok(true)
            }
            Self::SYNC_STATUS_NAME => {
                elem_value.set_bool(&[self.sync_status]);
                Ok(true)
            }
            IN_METER_NAME => {
                self.parse_meters(elem_value, 0, Self::IN_METER_LABELS);
                Ok(true)
            }
            OUT_METER_NAME => {
                let offset = Self::IN_METER_LABELS.len();
                self.parse_meters(elem_value, offset, Self::OUT_METER_LABELS);
                Ok(true)
            }
            Self::HP_OUT_METER_NAME => {
                let offset = Self::IN_METER_LABELS.len() + Self::OUT_METER_LABELS.len();
                self.parse_meters(elem_value, offset, Self::HP_OUT_METER_LABELS);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

pub struct StateCache{
    cache: [u8;Self::CACHE_SIZE],
}

impl StateCache {
    const CACHE_SIZE: usize = 160;

    pub fn new() -> Self {
        StateCache{
            cache: [0;Self::CACHE_SIZE],
        }
    }

    pub fn upload(&mut self, unit: &hinawa::SndUnit, req: &hinawa::FwReq) -> Result<(), Error> {
        (0..(Self::CACHE_SIZE / 4)).try_for_each(|pos| {
            let offset = pos * 4;
            req.write_quadlet(unit, offset, &mut self.cache)
        })
    }
}

trait SpecialProto : CommonProto {
    fn write_quadlet(&self, unit: &hinawa::SndUnit, offset: usize, cache: &mut [u8]) -> Result<(), Error> {
        self.transaction_sync(&unit.get_node(), hinawa::FwTcode::WriteQuadletRequest,
                              Self::BASE_ADDR + offset as u64, 4, &mut cache[offset..(offset + 4)], Self::TIMEOUT)
    }
}

impl SpecialProto for hinawa::FwReq {}

pub trait StateCacheAccessor {
    fn get_u32(&self, pos: usize) -> u32;
    fn set_u32(&mut self, pos: usize, val: u32);
    fn get_i16(&self, pos: usize) -> i16;
    fn set_i16(&mut self, pos: usize, val: i16);
}

impl StateCacheAccessor for StateCache {
    fn get_u32(&self, pos: usize) -> u32 {
        let mut quadlet = [0;std::mem::size_of::<u32>()];
        quadlet.copy_from_slice(&self.cache[pos..(pos + std::mem::size_of::<u32>())]);
        u32::from_be_bytes(quadlet)
    }

    fn set_u32(&mut self, pos: usize, val: u32) {
        let quadlet = val.to_be_bytes();
        self.cache[pos..(pos + std::mem::size_of::<u32>())].copy_from_slice(&quadlet);
    }

    fn get_i16(&self, pos: usize) -> i16 {
        let mut doublet = [0;std::mem::size_of::<i16>()];
        doublet.copy_from_slice(&self.cache[pos..(pos + std::mem::size_of::<i16>())]);
        i16::from_be_bytes(doublet)
    }

    fn set_i16(&mut self, pos: usize, val: i16) {
        let doublet = val.to_be_bytes();
        self.cache[pos..(pos + std::mem::size_of::<i16>())].copy_from_slice(&doublet);
    }
}

const STREAM_SRC_PAIR_LABELS: &[&str] = &["stream-1/2", "stream-3/4"];
const ANALOG_SRC_PAIR_LABELS: &[&str] = &["analog-1/2", "analog-3/4", "analog-5/6", "analog-7/8"];
const SPDIF_SRC_PAIR_LABELS: &[&str] = &["spdif-1/2"];
const ADAT_SRC_PAIR_LABELS: &[&str] = &["adat-1/2", "adat-3/4", "adat-5/6", "adat-7/8"];

const MIXER_DST_PAIR_LABELS: &[&str] = &["mixer-1/2", "mixer-3/4"];

const MIXER_PHYS_SRC_POS: usize = 0x90;
const MIXER_STREAM_SRC_POS: usize = 0x94;

const MIXER_ANALOG_SRC_TO_DST_01_SHIFT: [usize;2] = [0, 4];
const MIXER_SPDIF_SRC_TO_DST_01_SHIFT: [usize;2] = [16, 20];
const MIXER_ADAT_SRC_TO_DST_01_SHIFT: [usize;2] = [8, 12];
const MIXER_STREAM_SRC_01_TO_DST_SHIFT: [usize;2] = [0, 2];

const MIXER_SRC_NAME: &str = "mixer-source";

pub trait MixerCtl : StateCacheAccessor {
    fn load(&mut self, card_cntr: &mut card_cntr::CardCntr) -> Result<(), Error>;
    fn read(&self, elem_id: &alsactl::ElemId, elem_value: &mut alsactl::ElemValue) -> Result<bool, Error>;
    fn write(&mut self, unit: &hinawa::SndUnit, req: &hinawa::FwReq, elem_id: &alsactl::ElemId,
             old: &alsactl::ElemValue, new: &alsactl::ElemValue)
        -> Result<bool, Error>;
}

trait MixerSrcOperation {
    fn parse_stream_src_flags(&self, target: usize, table: &[usize;2]) -> Vec<bool>;
    fn parse_phys_src_flags(&self, count: usize, target: usize, table: &[usize;2]) -> Vec<bool>;
    fn build_stream_src_flags(&self, vals: &[bool], target: usize, table: &[usize;2]) -> Self;
    fn build_phys_src_flags(&self, vals: &[bool], target: usize, table: &[usize;2]) -> Self;
}

impl MixerSrcOperation for u32 {
    fn parse_stream_src_flags(&self, target: usize, table: &[usize;2]) -> Vec<bool> {
        table.iter().map(|shift| {
            let flag = (1 << (shift + target)) as u32;
            (flag & *self) > 0
        }).collect::<Vec<bool>>()
    }

    fn parse_phys_src_flags(&self, count: usize, target: usize, table: &[usize;2]) -> Vec<bool> {
        (0..count).map(|i| {
            let flag = (1 << (table[target] + i)) as u32;
            (flag & *self) > 0
        }).collect::<Vec<bool>>()
    }

    fn build_stream_src_flags(&self, vals: &[bool], target: usize, table: &[usize;2]) -> Self {
        vals.iter().zip(table.iter()).fold(*self, |mut flags, (v, shift)| {
            let flag = (1 << shift + target) as u32;
            flags &= !flag;
            if *v {
                flags |= flag;
            }
            flags
        })
    }

    fn build_phys_src_flags(&self, vals: &[bool], target: usize, table: &[usize;2]) -> Self {
        let shift = table[target];
        vals.iter().enumerate().fold(*self, |mut flags, (i, v)| {
            let flag = (1 << (shift + i)) as u32;
            flags &= !flag;
            if *v {
                flags |= flag;
            }
            flags
        })
    }
}

impl MixerCtl for StateCache {
    fn load(&mut self, card_cntr: &mut card_cntr::CardCntr) -> Result<(), Error> {
        // Source of mixers.
        let mut flags = self.get_u32(MIXER_STREAM_SRC_POS);
        flags = flags.build_stream_src_flags(&[true, false], 0, &MIXER_STREAM_SRC_01_TO_DST_SHIFT);
        flags = flags.build_stream_src_flags(&[false, true], 1, &MIXER_STREAM_SRC_01_TO_DST_SHIFT);
        self.set_u32(MIXER_STREAM_SRC_POS, flags);

        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Mixer, 0, 0, MIXER_SRC_NAME, 0);
        let in_count = STREAM_SRC_PAIR_LABELS.len() + ANALOG_SRC_PAIR_LABELS.len()
                     + SPDIF_SRC_PAIR_LABELS.len() + ADAT_SRC_PAIR_LABELS.len();
        let _ = card_cntr.add_bool_elems(&elem_id, MIXER_DST_PAIR_LABELS.len(), in_count, true)?;

        Ok(())
    }

    fn read(&self, elem_id: &alsactl::ElemId, elem_value: &mut alsactl::ElemValue) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            MIXER_SRC_NAME => {
                let target = elem_id.get_index() as usize;

                let flags = self.get_u32(MIXER_STREAM_SRC_POS);
                let mut vals = flags.parse_stream_src_flags(target, &MIXER_STREAM_SRC_01_TO_DST_SHIFT);

                let flags = self.get_u32(MIXER_PHYS_SRC_POS);
                vals.append(&mut flags.parse_phys_src_flags(ANALOG_SRC_PAIR_LABELS.len(), target,
                                                            &MIXER_ANALOG_SRC_TO_DST_01_SHIFT));
                vals.append(&mut flags.parse_phys_src_flags(SPDIF_SRC_PAIR_LABELS.len(), target,
                                                            &MIXER_SPDIF_SRC_TO_DST_01_SHIFT));
                vals.append(&mut flags.parse_phys_src_flags(ADAT_SRC_PAIR_LABELS.len(), target,
                                                            &MIXER_ADAT_SRC_TO_DST_01_SHIFT));

                elem_value.set_bool(&vals);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write(&mut self, unit: &hinawa::SndUnit, req: &hinawa::FwReq, elem_id: &alsactl::ElemId,
             _: &alsactl::ElemValue, new: &alsactl::ElemValue)
        -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            MIXER_SRC_NAME => {
                let index = elem_id.get_index() as usize;
                let in_count = STREAM_SRC_PAIR_LABELS.len() + ANALOG_SRC_PAIR_LABELS.len()
                             + SPDIF_SRC_PAIR_LABELS.len() + ADAT_SRC_PAIR_LABELS.len();
                let mut vals = vec![false;in_count];
                new.get_bool(&mut vals);

                let prev_flags = self.get_u32(MIXER_STREAM_SRC_POS);
                let mut curr_flags = prev_flags;
                curr_flags = curr_flags.build_stream_src_flags(&vals[..STREAM_SRC_PAIR_LABELS.len()], index,
                                                               &MIXER_STREAM_SRC_01_TO_DST_SHIFT);
                if prev_flags != curr_flags {
                    self.set_u32(MIXER_STREAM_SRC_POS, curr_flags);
                    req.write_quadlet(unit, MIXER_STREAM_SRC_POS, &mut self.cache)?;
                }

                let prev_flags = self.get_u32(MIXER_PHYS_SRC_POS);
                let mut curr_flags = prev_flags;
                let mut skip = STREAM_SRC_PAIR_LABELS.len();
                curr_flags = curr_flags.build_phys_src_flags(&vals[skip..(skip + ANALOG_SRC_PAIR_LABELS.len())], index,
                                                             &MIXER_ANALOG_SRC_TO_DST_01_SHIFT);
                skip += ANALOG_SRC_PAIR_LABELS.len();
                curr_flags = curr_flags.build_phys_src_flags(&vals[skip..(skip + SPDIF_SRC_PAIR_LABELS.len())], index,
                                                             &MIXER_SPDIF_SRC_TO_DST_01_SHIFT);
                skip += SPDIF_SRC_PAIR_LABELS.len();
                curr_flags = curr_flags.build_phys_src_flags(&vals[skip..(skip + ADAT_SRC_PAIR_LABELS.len())], index,
                                                             &MIXER_ADAT_SRC_TO_DST_01_SHIFT);
                if prev_flags != curr_flags {
                    self.set_u32(MIXER_PHYS_SRC_POS, curr_flags);
                    req.write_quadlet(unit, MIXER_PHYS_SRC_POS, &mut self.cache)?;
                }

                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

const STREAM_IN_LABELS: &[&str] = &[
    "stream-1", "stream-2", "stream-3", "stream-4",
];
const ANALOG_IN_LABELS: &[&str] = &[
    "analog-1", "analog-2", "analog-3", "analog-4",
    "analog-5", "analog-6", "analog-7", "analog-8",
];
const SPDIF_IN_LABELS: &[&str] = &[
    "spdif-1", "spdif-2",
];
const ADAT_IN_LABELS: &[&str] = &[
    "adat-1", "adat-2", "adat-3", "adat-4",
    "adat-5", "adat-6", "adat-7", "adat-8",
];

const STREAM_IN_GAIN_POS: usize = 0x0000;       // 0x0000 - 0x0008
const ANALOG_IN_GAIN_POS: usize = 0x0010;       // 0x0010 - 0x0020
const SPDIF_IN_GAIN_POS: usize = 0x0020;        // 0x0020 - 0x0024
const ADAT_IN_GAIN_POS: usize = 0x0024;         // 0x0024 - 0x0034

const ANALOG_IN_PAN_POS: usize = 0x0034;    // 0x0040 - 0x0050
const SPDIF_IN_PAN_POS: usize = 0x0050;     // 0x0050 - 0x0054
const ADAT_IN_PAN_POS: usize = 0x0054;      // 0x0054 - 0x0064.

const GAIN_SIZE: usize = std::mem::size_of::<i16>();
const PAN_SIZE: usize = std::mem::size_of::<i16>();

const STREAM_IN_GAIN_NAME: &str = "stream-in-gain";
const ANALOG_IN_GAIN_NAME: &str = "analog-in-gain";
const SPDIF_IN_GAIN_NAME: &str = "spdif-in-gain";
const ADAT_IN_GAIN_NAME: &str = "adat-in-gain";

const ANALOG_IN_PAN_NAME: &str = "analog-in-balance";
const SPDIF_IN_PAN_NAME: &str = "spdif-in-balance";
const ADAT_IN_PAN_NAME: &str = "adat-in-balance";

const GAIN_MIN: i32 = i16::MIN as i32;
const GAIN_MAX: i32 = 0;
const GAIN_STEP: i32 = 256;
const GAIN_TLV: &[i32;4] = &[5, 8, -12800, 0];

const PAN_MIN: i32 = i16::MIN as i32;
const PAN_MAX: i32 = i16::MAX as i32;
const PAN_STEP: i32 = 256;

pub trait InputCtl<'a> : StateCacheAccessor {
    fn load(&mut self, card_cntr: &mut card_cntr::CardCntr) -> Result<(), Error>;
    fn read(&self, elem_id: &alsactl::ElemId, elem_value: &mut alsactl::ElemValue) -> Result<bool, Error>;
    fn write(&mut self, unit: &hinawa::SndUnit, req: &hinawa::FwReq, elem_id: &alsactl::ElemId,
             old: &alsactl::ElemValue, new: &alsactl::ElemValue)
        -> Result<bool, Error>;
}

fn add_input_gain_elem(card_cntr: &mut card_cntr::CardCntr, name: &str, value_count: usize)
    -> Result<Vec<alsactl::ElemId>, Error>
{
    let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Mixer, 0, 0, name, 0);
    card_cntr.add_int_elems(&elem_id, 1, GAIN_MIN, GAIN_MAX, GAIN_STEP, value_count, Some(GAIN_TLV), true)
}

fn add_input_pan_elem(card_cntr: &mut card_cntr::CardCntr, name: &str, value_count: usize)
    -> Result<Vec<alsactl::ElemId>, Error>
{
    let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Mixer, 0, 0, name, 0);
    card_cntr.add_int_elems(&elem_id, 1, PAN_MIN, PAN_MAX, PAN_STEP, value_count, None, true)
}

impl StateCache {
    fn read_input_gain(&self, value_count: usize, pos: usize, elem_value: &mut alsactl::ElemValue) {
        let vals = (0..value_count).map(|i| {
            let p = pos + i * GAIN_SIZE;
            self.get_i16(p) as i32
        }).collect::<Vec<i32>>();
        elem_value.set_int(&vals);
    }

    fn read_input_pan(&self, value_count: usize, pos: usize, elem_value: &mut alsactl::ElemValue) {
        let vals = (0..value_count).map(|i| {
            let p = pos + i * PAN_SIZE;
            self.get_i16(p) as i32
        }).collect::<Vec<i32>>();
        elem_value.set_int(&vals);
    }

    fn write_input_gain(&mut self, unit: &hinawa::SndUnit, req: &hinawa::FwReq, value_count: usize, pos: usize,
                        old: &alsactl::ElemValue, new: &alsactl::ElemValue) -> Result<(), Error> {
        let mut vals = vec![0;value_count * 2];
        new.get_int(&mut vals[..value_count]);
        old.get_int(&mut vals[value_count..]);
        vals[..value_count].iter().zip(vals[value_count..].iter()).enumerate()
            .filter(|(_, (n, o))| *n != *o)
            .try_for_each(|(i, (v, _))| {
                let mut p = pos + i * GAIN_SIZE;
                self.set_i16(p, *v as i16);
                p -= p % std::mem::size_of::<u32>();
                req.write_quadlet(unit, p, &mut self.cache)
            })
    }

    fn write_input_pan(&mut self, unit: &hinawa::SndUnit, req: &hinawa::FwReq, value_count: usize, pos: usize,
                       old: &alsactl::ElemValue, new: &alsactl::ElemValue) -> Result<(), Error> {
        let mut vals = vec![0;value_count * 2];
        new.get_int(&mut vals[..value_count]);
        old.get_int(&mut vals[value_count..]);
        vals[..value_count].iter().zip(vals[value_count..].iter()).enumerate()
            .filter(|(_, (n, o))| *n != *o)
            .try_for_each(|(i, (v, _))| {
                let mut p = pos + i * PAN_SIZE;
                self.set_i16(p, *v as i16);
                p -= p % std::mem::size_of::<u32>();
                req.write_quadlet(unit, p, &mut self.cache)
            })
    }
}

impl<'a> InputCtl<'a> for StateCache {
    fn load(&mut self, card_cntr: &mut card_cntr::CardCntr) -> Result<(), Error> {
        // Gain of inputs from stream.
        (0..STREAM_IN_LABELS.len()).for_each(|i| {
            let pos = STREAM_IN_GAIN_POS + i * GAIN_SIZE;
            self.set_i16(pos, GAIN_MAX as i16);
        });

        // Gain of inputs from physical ports.
        (0..ANALOG_IN_LABELS.len()).for_each(|i| {
            let pos = ANALOG_IN_GAIN_POS + i * GAIN_SIZE;
            self.set_i16(pos, GAIN_MAX as i16);
        });
        (0..SPDIF_IN_LABELS.len()).for_each(|i| {
            let pos = SPDIF_IN_GAIN_POS + i * GAIN_SIZE;
            self.set_i16(pos, GAIN_MAX as i16);
        });
        (0..ADAT_IN_LABELS.len()).for_each(|i| {
            let pos = ADAT_IN_GAIN_POS + i * GAIN_SIZE;
            self.set_i16(pos, GAIN_MAX as i16);
        });

        // L/R balance of inputs from analog 1-8, S/PDIF 1/2, ADAT 1-8.
        (0..(ANALOG_IN_LABELS.len() / 2)).for_each(|i| {
            let mut pos = ANALOG_IN_PAN_POS + i * PAN_SIZE * 2;
            self.set_i16(pos, PAN_MIN as i16);
            pos += PAN_SIZE;
            self.set_i16(pos, PAN_MAX as i16);
        });
        (0..(SPDIF_IN_LABELS.len() / 2)).for_each(|i| {
            let mut pos = SPDIF_IN_PAN_POS+ i * PAN_SIZE * 2;
            self.set_i16(pos, PAN_MIN as i16);
            pos += PAN_SIZE;
            self.set_i16(pos, PAN_MAX as i16);
        });
        (0..(ADAT_IN_LABELS.len() / 2)).for_each(|i| {
            let mut pos = ADAT_IN_PAN_POS + i * PAN_SIZE * 2;
            self.set_i16(pos, PAN_MIN as i16);
            pos += PAN_SIZE;
            self.set_i16(pos, PAN_MAX as i16);
        });

        let _ = add_input_gain_elem(card_cntr, STREAM_IN_GAIN_NAME, STREAM_IN_LABELS.len())?;
        let _ = add_input_gain_elem(card_cntr, ANALOG_IN_GAIN_NAME, ANALOG_IN_LABELS.len())?;
        let _ = add_input_gain_elem(card_cntr, SPDIF_IN_GAIN_NAME, SPDIF_IN_LABELS.len())?;
        let _ = add_input_gain_elem(card_cntr, ADAT_IN_GAIN_NAME, ADAT_IN_LABELS.len())?;

        let _ = add_input_pan_elem(card_cntr, ANALOG_IN_PAN_NAME, ANALOG_IN_LABELS.len())?;
        let _ = add_input_pan_elem(card_cntr, SPDIF_IN_PAN_NAME, SPDIF_IN_LABELS.len())?;
        let _ = add_input_pan_elem(card_cntr, ADAT_IN_PAN_NAME, ADAT_IN_LABELS.len())?;

        Ok(())
    }

    fn read(&self, elem_id: &alsactl::ElemId, elem_value: &mut alsactl::ElemValue) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            STREAM_IN_GAIN_NAME => {
                self.read_input_gain(STREAM_IN_LABELS.len(), STREAM_IN_GAIN_POS, elem_value);
                Ok(true)
            }
            ANALOG_IN_GAIN_NAME => {
                self.read_input_gain(ANALOG_IN_LABELS.len(), ANALOG_IN_GAIN_POS, elem_value);
                Ok(true)
            }
            SPDIF_IN_GAIN_NAME => {
                self.read_input_gain(SPDIF_IN_LABELS.len(), SPDIF_IN_GAIN_POS, elem_value);
                Ok(true)
            }
            ADAT_IN_GAIN_NAME => {
                self.read_input_gain(ADAT_IN_LABELS.len(), ADAT_IN_GAIN_POS, elem_value);
                Ok(true)
            }
            ANALOG_IN_PAN_NAME => {
                self.read_input_pan(ANALOG_IN_LABELS.len(), ANALOG_IN_PAN_POS, elem_value);
                Ok(true)
            }
            SPDIF_IN_PAN_NAME => {
                self.read_input_pan(SPDIF_IN_LABELS.len(), SPDIF_IN_PAN_POS, elem_value);
                Ok(true)
            }
            ADAT_IN_PAN_NAME => {
                self.read_input_pan(ADAT_IN_LABELS.len(), ADAT_IN_PAN_POS, elem_value);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write(&mut self, unit: &hinawa::SndUnit, req: &hinawa::FwReq, elem_id: &alsactl::ElemId,
             old: &alsactl::ElemValue, new: &alsactl::ElemValue)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            STREAM_IN_GAIN_NAME => {
                self.write_input_gain(unit, req, STREAM_IN_LABELS.len(), STREAM_IN_GAIN_POS, old, new)?;
                Ok(true)
            }
            ANALOG_IN_GAIN_NAME => {
                self.write_input_gain(unit, req, ANALOG_IN_LABELS.len(), ANALOG_IN_GAIN_POS, old, new)?;
                Ok(true)
            }
            SPDIF_IN_GAIN_NAME => {
                self.write_input_gain(unit, req, ANALOG_IN_LABELS.len(), SPDIF_IN_GAIN_POS, old, new)?;
                Ok(true)
            }
            ADAT_IN_GAIN_NAME => {
                self.write_input_gain(unit, req, ANALOG_IN_LABELS.len(), ADAT_IN_GAIN_POS, old, new)?;
                Ok(true)
            }
            ANALOG_IN_PAN_NAME => {
                self.write_input_pan(unit, req, ANALOG_IN_LABELS.len(), ANALOG_IN_PAN_POS, old, new)?;
                Ok(true)
            }
            SPDIF_IN_PAN_NAME => {
                self.write_input_pan(unit, req, SPDIF_IN_LABELS.len(), SPDIF_IN_PAN_POS, old, new)?;
                Ok(true)
            }
            ADAT_IN_PAN_NAME => {
                self.write_input_pan(unit, req, ADAT_IN_LABELS.len(), ADAT_IN_PAN_POS, old, new)?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
