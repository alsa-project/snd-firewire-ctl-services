// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::{Error, FileError};

use alsa_ctl_tlv_codec::items::DbInterval;

use core::card_cntr;
use core::elem_value_accessor::ElemValueAccessor;

use efw_protocols::transactions::{HwCap, PhysGroupType, NominalLevel, EfwPhysOutput, HwInfo};

pub struct OutputCtl {
    phys_outputs: usize,
}

impl<'a> OutputCtl {
    const OUT_VOL_NAME: &'a str = "output-volume";
    const OUT_MUTE_NAME: &'a str = "output-mute";
    const OUT_NOMINAL_NAME: &'a str = "output-nominal";

    // The fixed point number of 8.24 format.
    const COEF_MIN: i32 = 0x00000000;
    const COEF_MAX: i32 = 0x02000000;
    const COEF_STEP: i32 = 0x00000001;
    const COEF_TLV: DbInterval = DbInterval{min: -14400, max: 600, linear: false, mute_avail: false};

    const OUT_NOMINAL_LABELS: &'a [&'a str] = &["+4dBu", "-10dBV"];
    const OUT_NOMINAL_LEVELS: &'a [NominalLevel] = &[NominalLevel::PlusFour, NominalLevel::MinusTen];

    pub fn new() -> Self {
        OutputCtl { phys_outputs: 0 }
    }

    pub fn load(&mut self, hwinfo: &HwInfo, card_cntr: &mut card_cntr::CardCntr)
        -> Result<(), Error>
    {
        self.phys_outputs = hwinfo.phys_outputs.iter().fold(0, |accm, entry| {
            if entry.group_type != PhysGroupType::AnalogMirror {
                accm + entry.group_count
            } else {
                accm
            }
        });

        let elem_id = alsactl::ElemId::new_by_name(
            alsactl::ElemIfaceType::Mixer, 0, 0, Self::OUT_VOL_NAME, 0);
        let _ = card_cntr.add_int_elems(&elem_id, 1,
            Self::COEF_MIN, Self::COEF_MAX, Self::COEF_STEP,
            self.phys_outputs, Some(&Into::<Vec<u32>>::into(Self::COEF_TLV)), true)?;

        let elem_id = alsactl::ElemId::new_by_name(
            alsactl::ElemIfaceType::Mixer, 0, 0, Self::OUT_MUTE_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, self.phys_outputs, true)?;

        if hwinfo.caps.iter().find(|&cap| *cap == HwCap::NominalOutput).is_some() {
            let elem_id = alsactl::ElemId::new_by_name(
                alsactl::ElemIfaceType::Mixer, 0, 0, Self::OUT_NOMINAL_NAME, 0);
            let _ = card_cntr.add_enum_elems(&elem_id, 1,
                self.phys_outputs, Self::OUT_NOMINAL_LABELS, None, true)?;
        }

        Ok(())
    }

    pub fn read(
        &mut self,
        unit: &hinawa::SndEfw,
        elem_id: &alsactl::ElemId,
        elem_value: &mut alsactl::ElemValue,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            Self::OUT_VOL_NAME => {
                ElemValueAccessor::<i32>::set_vals(elem_value, self.phys_outputs, |idx| {
                    let val = EfwPhysOutput::get_vol(unit, idx)?;
                    Ok(val)
                })?;
                Ok(true)
            }
            Self::OUT_MUTE_NAME => {
                ElemValueAccessor::<bool>::set_vals(elem_value, self.phys_outputs, |idx| {
                    let val = EfwPhysOutput::get_mute(unit, idx)?;
                    Ok(val)
                })?;
                Ok(true)
            }
            Self::OUT_NOMINAL_NAME => {
                ElemValueAccessor::<u32>::set_vals(elem_value, self.phys_outputs, |idx| {
                    let level = EfwPhysOutput::get_nominal(unit, idx)?;
                    if let Some(pos) = Self::OUT_NOMINAL_LEVELS.iter().position(|&l| l == level) {
                        Ok(pos as u32)
                    } else {
                        unreachable!();
                    }
                })?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub fn write(
        &mut self,
        unit: &hinawa::SndEfw,
        elem_id: &alsactl::ElemId,
        old: &alsactl::ElemValue,
        new: &alsactl::ElemValue,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            Self::OUT_VOL_NAME => {
                ElemValueAccessor::<i32>::get_vals(new, old, self.phys_outputs, |idx, val| {
                    EfwPhysOutput::set_vol(unit, idx, val)
                })?;
                Ok(true)
            }
            Self::OUT_MUTE_NAME => {
                ElemValueAccessor::<bool>::get_vals(new, old, self.phys_outputs, |idx, val| {
                    EfwPhysOutput::set_mute(unit, idx, val)
                })?;
                Ok(true)
            }
            Self::OUT_NOMINAL_NAME => {
                ElemValueAccessor::<u32>::get_vals(new, old, self.phys_outputs, |idx, val| {
                    if let Some(&level) = Self::OUT_NOMINAL_LEVELS.iter().nth(val as usize) {
                        EfwPhysOutput::set_nominal(unit, idx, level)
                    } else {
                        let label = "Invalid value for nominal level of output";
                        Err(Error::new(FileError::Io, &label))
                    }
                })?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
