// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::{Error, FileError};

use core::card_cntr;
use core::elem_value_accessor::ElemValueAccessor;

use efw_protocols::transactions::{HwCap, EfwPhysInput, NominalLevel, HwInfo};

pub struct InputCtl {
    phys_inputs: usize,
    cache: Option<Vec::<NominalLevel>>,
}

impl<'a> InputCtl {
    const IN_NOMINAL_NAME: &'a str = "input-nominal";

    const IN_NOMINAL_LABELS: &'a [&'a str] = &["+4dBu", "-10dBV"];
    const IN_NOMINAL_LEVELS: &'a [NominalLevel] = &[NominalLevel::PlusFour, NominalLevel::MinusTen];

    pub fn new() -> Self {
        InputCtl { phys_inputs: 0, cache: None}
    }

    pub fn load(&mut self, unit: &hinawa::SndEfw, hwinfo: &HwInfo, card_cntr: &mut card_cntr::CardCntr)
        -> Result<(), Error>
    {
        self.phys_inputs = hwinfo.phys_inputs.iter().fold(0, |accm, entry| accm + entry.group_count);

        if hwinfo.caps.iter().find(|&cap| *cap == HwCap::NominalInput).is_some() {
            let elem_id = alsactl::ElemId::new_by_name(
                alsactl::ElemIfaceType::Mixer, 0, 0, Self::IN_NOMINAL_NAME, 0);
            let _ = card_cntr.add_enum_elems(&elem_id, 1,
                self.phys_inputs, Self::IN_NOMINAL_LABELS, None, true)?;
        }

        // FPGA models return invalid state of nominal level. Here, initialize them and cache the
        // state instead.
        let has_fpga = hwinfo.caps.iter().find(|&cap| *cap == HwCap::Fpga).is_some();
        if has_fpga {
            let cache = vec![NominalLevel::PlusFour;self.phys_inputs];
            cache.iter().enumerate()
                .try_for_each( |(i, &level)| {
                    EfwPhysInput::set_nominal(unit, i, level)?;
                    Ok(())
                })?;
            self.cache = Some(cache);
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
                ElemValueAccessor::<u32>::set_vals(elem_value, self.phys_inputs, |idx| {
                    if let Some(cache) = &self.cache {
                        // For models with FPGA.
                        Ok(u32::from(cache[idx]))
                    } else {
                        // For models with DSP.
                        let level = EfwPhysInput::get_nominal(unit, idx)?;
                        if let Some(pos) = Self::IN_NOMINAL_LEVELS.iter().position(|&l| l == level) {
                            Ok(pos as u32)
                        } else {
                            unreachable!();
                        }
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
            Self::IN_NOMINAL_NAME => {
                ElemValueAccessor::<u32>::get_vals(new, old, self.phys_inputs, |idx, val| {
                    if let Some(&level) = Self::IN_NOMINAL_LEVELS.iter().nth(val as usize) {
                        EfwPhysInput::set_nominal(unit, idx, level)?;
                        if let Some(cache) = &mut self.cache {
                            // For FPGA models.
                            cache[idx] = level;
                        }
                        Ok(())
                    } else {
                        let label = "Invalid value for nominal level of input";
                        Err(Error::new(FileError::Inval, &label))
                    }
                })?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
