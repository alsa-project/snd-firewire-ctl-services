// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use glib::Error;

use hinawa::{FwReq, FwTcode};
use hinawa::{SndUnit, SndUnitExt};

use alsactl::{ElemId, ElemIfaceType, ElemValue, ElemValueExtManual};

use core::card_cntr::*;

use super::common_proto::CommonProto;

pub struct StateCache{
    pub cache: [u8;Self::CACHE_SIZE],
}

impl StateCache {
    const CACHE_SIZE: usize = 160;

    pub fn new() -> Self {
        StateCache{
            cache: [0;Self::CACHE_SIZE],
        }
    }

    pub fn upload(&mut self, unit: &SndUnit, req: &FwReq) -> Result<(), Error> {
        (0..(Self::CACHE_SIZE / 4)).try_for_each(|pos| {
            let offset = pos * 4;
            req.write_quadlet(unit, offset, &mut self.cache)
        })
    }
}

trait SpecialProto : CommonProto {
    fn write_quadlet(&self, unit: &SndUnit, offset: usize, cache: &mut [u8]) -> Result<(), Error> {
        self.transaction_sync(&unit.get_node(), FwTcode::WriteQuadletRequest,
                              Self::BASE_ADDR + offset as u64, 4, &mut cache[offset..(offset + 4)], Self::TIMEOUT)
    }
}

impl CommonProto for FwReq {}

impl SpecialProto for FwReq {}

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
    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error>;
    fn read(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error>;
    fn write(&mut self, unit: &SndUnit, req: &FwReq, elem_id: &ElemId,
             old: &ElemValue, new: &ElemValue)
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
    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        // Source of mixers.
        let mut flags = self.get_u32(MIXER_STREAM_SRC_POS);
        flags = flags.build_stream_src_flags(&[true, false], 0, &MIXER_STREAM_SRC_01_TO_DST_SHIFT);
        flags = flags.build_stream_src_flags(&[false, true], 1, &MIXER_STREAM_SRC_01_TO_DST_SHIFT);
        self.set_u32(MIXER_STREAM_SRC_POS, flags);

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIXER_SRC_NAME, 0);
        let in_count = STREAM_SRC_PAIR_LABELS.len() + ANALOG_SRC_PAIR_LABELS.len()
                     + SPDIF_SRC_PAIR_LABELS.len() + ADAT_SRC_PAIR_LABELS.len();
        let _ = card_cntr.add_bool_elems(&elem_id, MIXER_DST_PAIR_LABELS.len(), in_count, true)?;

        Ok(())
    }

    fn read(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
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

    fn write(&mut self, unit: &SndUnit, req: &FwReq, elem_id: &ElemId,
             _: &ElemValue, new: &ElemValue)
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
