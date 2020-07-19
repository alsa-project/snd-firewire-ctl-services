// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use crate::card_cntr;

use alsactl::{ElemValueExt, ElemValueExtManual};

use super::transactions::{HwCap, EfwPhysInput, NominalLevel, HwInfo};

pub struct InputCtl {
    phys_inputs: usize,
}

impl<'a> InputCtl {
    const IN_NOMINAL_NAME: &'a str = "input-nominal";

    const IN_NOMINAL_LABELS: &'a [&'a str] = &["+4dBu", "-10dBV"];
    const IN_NOMINAL_LEVELS: &'a [NominalLevel] = &[NominalLevel::PlusFour, NominalLevel::MinusTen];

    pub fn new() -> Self {
        InputCtl { phys_inputs: 0 }
    }

    pub fn load(&mut self, hwinfo: &HwInfo, card_cntr: &mut card_cntr::CardCntr)
        -> Result<(), Error>
    {
        self.phys_inputs = hwinfo.phys_inputs.iter().fold(0, |accm, entry| accm + entry.group_count);

        if hwinfo.caps.iter().find(|&cap| *cap == HwCap::NominalInput).is_some() {
            let elem_id = alsactl::ElemId::new_by_name(
                alsactl::ElemIfaceType::Mixer, 0, 0, Self::IN_NOMINAL_NAME, 0);
            let _ = card_cntr.add_enum_elems(&elem_id, 1,
                self.phys_inputs, Self::IN_NOMINAL_LABELS, None, true)?;
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
            Self::IN_NOMINAL_NAME => {
                let mut vals = vec![0; self.phys_inputs];
                vals.iter_mut().enumerate().try_for_each(|(i, val)| {
                    let level = EfwPhysInput::get_nominal(unit, i)?;
                    if let Some(pos) = Self::IN_NOMINAL_LEVELS.iter().position(|&l| l == level) {
                        *val = pos as u32;
                    }
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
            Self::IN_NOMINAL_NAME => {
                let mut vals = vec![0; self.phys_inputs * 2];
                new.get_enum(&mut vals[..self.phys_inputs]);
                old.get_enum(&mut vals[self.phys_inputs..]);
                (0..self.phys_inputs).try_for_each(|i| {
                    if vals[i] != vals[self.phys_inputs + i] {
                        if let Some(&level) = Self::IN_NOMINAL_LEVELS.iter().nth(vals[i] as usize) {
                            EfwPhysInput::set_nominal(unit, i, level)?;
                        }
                    }
                    Ok(())
                })?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
