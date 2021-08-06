// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use glib::Error;

use hinawa::{FwReq, FwTcode};
use hinawa::{SndUnit, SndUnitExt};

use alsactl::{ElemId, ElemIfaceType, ElemValue, ElemValueExt, ElemValueExtManual};

use alsa_ctl_tlv_codec::items::DbInterval;

use core::card_cntr::*;
use core::elem_value_accessor::ElemValueAccessor;

use crate::model::{OUT_SRC_NAME, OUT_VOL_NAME, HP_SRC_NAME};

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

const GAIN_SIZE: usize = std::mem::size_of::<i16>();

const GAIN_MIN: i32 = i16::MIN as i32;
const GAIN_MAX: i32 = 0;
const GAIN_STEP: i32 = 256;
const GAIN_TLV: DbInterval = DbInterval{min: -12800, max: 0, linear: false, mute_avail: false};

pub trait OutputCtl : StateCacheAccessor {
    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error>;
    fn read(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error>;
    fn write(&mut self, unit: &SndUnit, req: &FwReq, elem_id: &ElemId,
             old: &ElemValue, new: &ElemValue)
        -> Result<bool, Error>;
}

const ANALOG_OUT_LABELS: &[&str] = &["analog-1", "analog-2", "analog-3", "analog-4"];
const OUT_PAIR_LABELS: &[&str] = &["output-1/2", "output-3/4"];
const OUT_PAIR_SRC_LABELS: &[&str] = &["mixer", "aux-1/2"];

const ANALOG_OUT_VOL_POS: usize = 0x08;     // 0x08 - 0x10
const OUT_PAIR_SRC_POS: usize = 0x9c;       // 0x9c

const OUT_PAIR_SRC_TABLE: [usize;2] = [0, 1];

const VOL_SIZE: usize = std::mem::size_of::<i16>();

const VOL_MIN: i32 = i16::MIN as i32;
const VOL_MAX: i32 = 0;
const VOL_STEP: i32 = 256;
const VOL_TLV: DbInterval = DbInterval{min: -12800, max: 0, linear: false, mute_avail: false};

trait OutputSrcOperation {
    fn parse_out_src_flags(&self) -> Vec<u32>;
    fn build_out_src_flags(&self, vals: &[u32]) -> Self;
}

impl OutputSrcOperation for u32 {
    fn parse_out_src_flags(&self) -> Vec<u32> {
        OUT_PAIR_SRC_TABLE.iter().map(|shift| {
            (*self & (1 << *shift) > 0) as u32
        }).collect::<Vec<u32>>()
    }

    fn build_out_src_flags(&self, vals: &[u32]) -> Self {
        vals.iter().zip(OUT_PAIR_SRC_TABLE.iter()).fold(*self, |mut flags, (v, shift)| {
            let flag = 1 << shift;
            flags &= !flag;
            if *v > 0 {
                flags |= flag;
            }
            flags
        })
    }
}

impl OutputCtl for StateCache {
    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        // Volume of outputs to analog ports.
        (0..ANALOG_OUT_LABELS.len()).for_each(|i| {
            let pos = ANALOG_OUT_VOL_POS + i * VOL_SIZE;
            self.set_i16(pos, VOL_MAX as i16);
        });

        // Source of outputs.
        let mut flags = self.get_u32(OUT_PAIR_SRC_POS);
        flags = flags.build_out_src_flags(&[0, 0]);
        self.set_u32(OUT_PAIR_SRC_POS, flags);

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, OUT_VOL_NAME, 0);
        card_cntr.add_int_elems(&elem_id, 1, VOL_MIN, VOL_MAX, VOL_STEP,
                                ANALOG_OUT_LABELS.len(), Some(&Into::<Vec<u32>>::into(VOL_TLV)), true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, OUT_SRC_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, OUT_PAIR_LABELS.len(), OUT_PAIR_SRC_LABELS, None, true)?;

        Ok(())
    }

    fn read(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            OUT_VOL_NAME => {
                ElemValueAccessor::<i32>::set_vals(elem_value, ANALOG_OUT_LABELS.len(), |idx| {
                    let pos = ANALOG_OUT_VOL_POS + idx * VOL_SIZE;
                    Ok(self.get_i16(pos) as i32)
                }).and(Ok(true))
            }
            OUT_SRC_NAME => {
                let flags = self.get_u32(OUT_PAIR_SRC_POS);
                let vals = flags.parse_out_src_flags();
                elem_value.set_enum(&vals);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write(&mut self, unit: &SndUnit, req: &FwReq, elem_id: &ElemId,
             old: &ElemValue, new: &ElemValue)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            OUT_VOL_NAME => {
                ElemValueAccessor::<i32>::get_vals(new, old, ANALOG_OUT_LABELS.len(), |idx, val| {
                    let mut pos = ANALOG_OUT_VOL_POS + idx * VOL_SIZE;
                    self.set_i16(pos, val as i16);
                    pos -= pos % 4;
                    req.write_quadlet(unit, pos, &mut self.cache)
                })?;
                Ok(true)
            }
            OUT_SRC_NAME => {
                let mut vals = [0;OUT_PAIR_SRC_LABELS.len()];
                new.get_enum(&mut vals);
                let mut flags = self.get_u32(OUT_PAIR_SRC_POS);
                flags = flags.build_out_src_flags(&vals);
                self.set_u32(OUT_PAIR_SRC_POS, flags);
                req.write_quadlet(unit, OUT_PAIR_SRC_POS, &mut self.cache)?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

pub trait AuxCtl : StateCacheAccessor {
    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error>;
    fn read(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error>;
    fn write(&mut self, unit: &SndUnit, req: &FwReq, elem_id: &ElemId,
             old: &ElemValue, new: &ElemValue)
        -> Result<bool, Error>;
}

const AUX_OUT_LABELS: &[&str] = &["aux-1", "aux-2"];

const AUX_SRC_PAIR_TO_DST_POS: usize = 0x64;    // 0x64 - 0x8c.
const AUX_OUT_POS: usize = 0x34;                // 0x34.

const AUX_SRC_PAIR_NAME: &str = "aux-source";
const AUX_OUT_VOL_NAME: &str = "aux-out-volume";

impl AuxCtl for StateCache {
    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        // Gain of inputs to aux mixer.
        let src_count = STREAM_SRC_PAIR_LABELS.len() + ANALOG_SRC_PAIR_LABELS.len() +
                        SPDIF_SRC_PAIR_LABELS.len() + ADAT_SRC_PAIR_LABELS.len();
        (0..src_count).for_each(|i| {
            let pos = AUX_SRC_PAIR_TO_DST_POS + i * GAIN_SIZE;
            self.set_i16(pos, GAIN_MIN as i16);
        });

        // Volume of outputs from aux mixer.
        (0..AUX_OUT_LABELS.len()).for_each(|i| {
            let pos = AUX_OUT_POS + i * GAIN_SIZE;
            self.set_i16(pos, GAIN_MAX as i16);
        });

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, AUX_SRC_PAIR_NAME, 0);
        let _ = card_cntr.add_int_elems(&elem_id, 1, GAIN_MIN, GAIN_MAX, GAIN_STEP, src_count,
                                        Some(&Into::<Vec<u32>>::into(GAIN_TLV)), true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, AUX_OUT_VOL_NAME, 0);
        let _ = card_cntr.add_int_elems(&elem_id, 1, VOL_MIN, VOL_MAX, VOL_STEP,
                                        AUX_OUT_LABELS.len(),
                                        Some(&Into::<Vec<u32>>::into(VOL_TLV)), true)?;

        Ok(())
    }

    fn read(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            AUX_SRC_PAIR_NAME => {
                let src_count = STREAM_SRC_PAIR_LABELS.len() + ANALOG_SRC_PAIR_LABELS.len() +
                                SPDIF_SRC_PAIR_LABELS.len() + ADAT_SRC_PAIR_LABELS.len();
                ElemValueAccessor::<i32>::set_vals(elem_value, src_count, |idx| {
                    let pos = AUX_SRC_PAIR_TO_DST_POS + idx * GAIN_SIZE;
                    Ok(self.get_i16(pos) as i32)
                })?;
                Ok(true)
            }
            AUX_OUT_VOL_NAME => {
                ElemValueAccessor::<i32>::set_vals(elem_value, AUX_OUT_LABELS.len(), |idx| {
                    let pos = AUX_OUT_POS + idx * VOL_SIZE;
                    Ok(self.get_i16(pos) as i32)
                })?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write(&mut self, unit: &SndUnit, req: &FwReq, elem_id: &ElemId,
             old: &ElemValue, new: &ElemValue)
        -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            AUX_SRC_PAIR_NAME => {
                let src_count = STREAM_SRC_PAIR_LABELS.len() + ANALOG_SRC_PAIR_LABELS.len() +
                                SPDIF_SRC_PAIR_LABELS.len() + ADAT_SRC_PAIR_LABELS.len();
                ElemValueAccessor::<i32>::get_vals(new, old, src_count, |idx, val| {
                    let mut pos = AUX_SRC_PAIR_TO_DST_POS + idx * GAIN_SIZE;
                    self.set_i16(pos, val as i16);
                    pos -= pos % 4;
                    req.write_quadlet(unit, pos, &mut self.cache)
                })?;
                Ok(true)
            }
            AUX_OUT_VOL_NAME => {
                ElemValueAccessor::<i32>::get_vals(new, old, AUX_OUT_LABELS.len(), |idx, val| {
                    let pos = AUX_OUT_POS + idx * VOL_SIZE;
                    self.set_i16(pos, val as i16);
                    Ok(())
                })?;
                req.write_quadlet(unit, AUX_OUT_POS, &mut self.cache)?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

pub trait HpCtl : StateCacheAccessor {
    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error>;
    fn read(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error>;
    fn write(&mut self, unit: &SndUnit, req: &FwReq, elem_id: &ElemId,
             old: &ElemValue, new: &ElemValue)
        -> Result<bool, Error>;
}

const HP_OUT_LABELS: &[&str] = &["headphone-1", "headphone-2", "headphone-3", "headphone-4"];
const HP_OUT_PAIR_LABELS: &[&str] = &["headphone-1/2", "headphone-3/4"];
const HP_SRC_PAIR_LABELS: &[&str] = &["mixer-1/2", "mixer-3/4", "aux-1/2"];

const HP_OUT_VOL_POS: usize = 0x38;            // 0x38 - 0x40.
const HP_SRC_PAIR_POS: usize = 0x98;                // 0x98-0x9f

const HP_DST_PAIR_SHIFT_TABLE: [usize;2] = [0, 16];
const HP_SRC_PAIR_SHIFT_TABLE: [usize;3] = [0, 1, 2];

const HP_OUT_VOL_NAME: &str = "headphone-volume";

trait HpSrcOperation {
    fn build_hp_src_flags(&self, vals: &[u32]) -> u32;
    fn parse_hp_src_flags(&self) -> Vec<u32>;
}

impl HpSrcOperation for u32 {
    fn build_hp_src_flags(&self, vals: &[u32]) -> Self {
        vals.iter().zip(HP_DST_PAIR_SHIFT_TABLE.iter()).fold(*self, |flags, (v, dst_shift)| {
            HP_SRC_PAIR_SHIFT_TABLE.iter().enumerate().fold(flags, |mut flags, (i, src_shift)| {
                let flag = 1 << (dst_shift + src_shift);
                flags &= !flag;
                if i == *v as usize {
                    flags |= flag;
                }
                flags
            })
        })
    }

    fn parse_hp_src_flags(&self) -> Vec<u32> {
        HP_DST_PAIR_SHIFT_TABLE.iter().map(|dst_shift| {
            HP_SRC_PAIR_SHIFT_TABLE.iter().position(|src_shift| {
                let flag = 1 << (dst_shift + src_shift);
                flag & *self > 0
            }).unwrap_or(0) as u32
        }).collect::<Vec<u32>>()
    }
}

impl HpCtl for StateCache {
    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        // Source of headphone.
        let mut flags = self.get_u32(HP_SRC_PAIR_POS);
        flags = flags.build_hp_src_flags(&[0, 1]);
        self.set_u32(HP_SRC_PAIR_POS, flags);

        // Volume of headphone output.
        (0..HP_OUT_LABELS.len()).for_each(|i| {
            let pos = HP_OUT_VOL_POS + i * VOL_SIZE;
            self.set_i16(pos, VOL_MAX as i16);
        });

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, HP_OUT_VOL_NAME, 0);
        let _ = card_cntr.add_int_elems(&elem_id, 1, VOL_MIN, VOL_MAX, VOL_STEP,
                                        HP_OUT_LABELS.len(),
                                        Some(&Into::<Vec<u32>>::into(VOL_TLV)), true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, HP_SRC_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, HP_OUT_PAIR_LABELS.len(), HP_SRC_PAIR_LABELS, None, true)?;

        Ok(())
    }

    fn read(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            HP_SRC_NAME => {
                let flags = self.get_u32(HP_SRC_PAIR_POS);
                let vals = flags.parse_hp_src_flags();
                elem_value.set_enum(&vals);
                Ok(true)
            }
            HP_OUT_VOL_NAME => {
                ElemValueAccessor::<i32>::set_vals(elem_value, HP_OUT_LABELS.len(), |idx| {
                    let pos = HP_OUT_VOL_POS + idx * VOL_SIZE;
                    Ok(self.get_i16(pos) as i32)
                })?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write(&mut self, unit: &SndUnit, req: &FwReq, elem_id: &ElemId,
             old: &ElemValue, new: &ElemValue)
        -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            HP_SRC_NAME => {
                let mut vals = [0;HP_OUT_PAIR_LABELS.len()];
                new.get_enum(&mut vals);
                let prev_flags = self.get_u32(HP_SRC_PAIR_POS);
                let curr_flags = prev_flags.build_hp_src_flags(&vals);
                if curr_flags != prev_flags {
                    self.set_u32(HP_SRC_PAIR_POS, curr_flags);
                    req.write_quadlet(unit, HP_SRC_PAIR_POS, &mut self.cache)?;
                }
                Ok(true)
            }
            HP_OUT_VOL_NAME => {
                ElemValueAccessor::<i32>::get_vals(new, old, HP_OUT_LABELS.len(), |idx, val| {
                    let mut pos = HP_OUT_VOL_POS + idx * VOL_SIZE;
                    self.set_i16(pos, val as i16);
                    pos -= pos % 4;
                    req.write_quadlet(unit, pos, &mut self.cache)
                })?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
