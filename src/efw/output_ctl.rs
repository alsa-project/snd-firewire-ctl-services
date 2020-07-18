// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use crate::card_cntr;

use alsactl::{ElemValueExt, ElemValueExtManual};

use super::transactions::{HwCap, PhysGroupType, NominalLevel, EfwPhysOutput, HwInfo};

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
    const COEF_TLV: &'a [i32] = &[5, 8, -14400, 6];

    const OUT_NOMINAL_LABELS: &'a [&'a str] = &["+4dBu", "0", "-10dBV"];

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
            self.phys_outputs, Some(Self::COEF_TLV), true)?;

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
                let mut vals = vec![0; self.phys_outputs];
                vals.iter_mut().enumerate().try_for_each(|(i, val)| {
                    *val = EfwPhysOutput::get_vol(unit, i)?;
                    Ok(())
                })?;
                elem_value.set_int(&vals);
                Ok(true)
            }
            Self::OUT_MUTE_NAME => {
                let mut vals = vec![false; self.phys_outputs];
                vals.iter_mut().enumerate().try_for_each(|(i, val)| {
                    *val = EfwPhysOutput::get_mute(unit, i)?;
                    Ok(())
                })?;
                elem_value.set_bool(&vals);
                Ok(true)
            }
            Self::OUT_NOMINAL_NAME => {
                let mut vals = vec![0; self.phys_outputs];
                vals.iter_mut().enumerate().try_for_each(|(i, val)| {
                    *val = match EfwPhysOutput::get_nominal(unit, i)? {
                        NominalLevel::MinusTen => 2,
                        NominalLevel::Medium => 1,
                        NominalLevel::PlusFour => 0,
                    };
                    Ok(())
                })?;
                elem_value.set_enum(&vals);
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
                let mut vals = vec![0; self.phys_outputs * 2];
                new.get_int(&mut vals[..self.phys_outputs]);
                old.get_int(&mut vals[self.phys_outputs..]);
                (0..self.phys_outputs).try_for_each(|i| {
                    if vals[i] != vals[self.phys_outputs + i] {
                        EfwPhysOutput::set_vol(unit, i, vals[i])?;
                    }
                    Ok(())
                })?;
                Ok(true)
            }
            Self::OUT_MUTE_NAME => {
                let mut vals = vec![false; self.phys_outputs * 2];
                new.get_bool(&mut vals[..self.phys_outputs]);
                old.get_bool(&mut vals[self.phys_outputs..]);
                (0..self.phys_outputs).try_for_each(|i| {
                    if vals[i] != vals[self.phys_outputs + i] {
                        EfwPhysOutput::set_mute(unit, i, vals[i])?;
                    }
                    Ok(())
                })?;
                Ok(true)
            }
            Self::OUT_NOMINAL_NAME => {
                let mut vals = vec![0; self.phys_outputs * 2];
                new.get_enum(&mut vals[..self.phys_outputs]);
                old.get_enum(&mut vals[self.phys_outputs..]);
                (0..self.phys_outputs).try_for_each(|i| {
                    if vals[i] != vals[self.phys_outputs + i] {
                        let level = match vals[i] {
                            2 => NominalLevel::MinusTen,
                            1 => NominalLevel::Medium,
                            _ => NominalLevel::PlusFour,
                        };
                        EfwPhysOutput::set_nominal(unit, i, level)?;
                    }
                    Ok(())
                })?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
