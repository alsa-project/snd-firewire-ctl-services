// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use {
    super::*,
    protocols::{hw_info::*, phys_input::*, *},
};

#[derive(Default)]
pub struct InputCtl {
    phys_inputs: usize,
    cache: Option<Vec<NominalSignalLevel>>,
}

const IN_NOMINAL_NAME: &str = "input-nominal";

impl InputCtl {
    const IN_NOMINAL_LABELS: [&'static str; 2] = ["+4dBu", "-10dBV"];
    const IN_NOMINAL_LEVELS: [NominalSignalLevel; 2] = [
        NominalSignalLevel::Professional,
        NominalSignalLevel::Consumer,
    ];

    pub fn load(
        &mut self,
        unit: &mut SndEfw,
        hwinfo: &HwInfo,
        card_cntr: &mut CardCntr,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        self.phys_inputs = hwinfo
            .phys_inputs
            .iter()
            .fold(0, |accm, entry| accm + entry.group_count);

        if hwinfo
            .caps
            .iter()
            .find(|&cap| *cap == HwCap::NominalInput)
            .is_some()
        {
            let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, IN_NOMINAL_NAME, 0);
            let _ = card_cntr.add_enum_elems(
                &elem_id,
                1,
                self.phys_inputs,
                &Self::IN_NOMINAL_LABELS,
                None,
                true,
            )?;

            // FPGA models return invalid state of nominal level.
            let has_fpga = hwinfo
                .caps
                .iter()
                .find(|&cap| *cap == HwCap::Fpga)
                .is_some();
            if has_fpga {
                let cache = vec![NominalSignalLevel::Professional; self.phys_inputs];
                cache
                    .iter()
                    .enumerate()
                    .try_for_each(|(i, &level)| unit.set_nominal(i, level, timeout_ms))?;
                self.cache = Some(cache);
            }
        }

        Ok(())
    }

    pub fn read(
        &mut self,
        unit: &mut SndEfw,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            IN_NOMINAL_NAME => {
                ElemValueAccessor::<u32>::set_vals(elem_value, self.phys_inputs, |idx| {
                    let level = if let Some(cache) = &self.cache {
                        Ok(cache[idx])
                    } else {
                        unit.get_nominal(idx, timeout_ms)
                    }?;
                    let pos = Self::IN_NOMINAL_LEVELS
                        .iter()
                        .position(|l| level.eq(l))
                        .unwrap();
                    Ok(pos as u32)
                })?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub fn write(
        &mut self,
        unit: &mut SndEfw,
        elem_id: &ElemId,
        old: &ElemValue,
        new: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            IN_NOMINAL_NAME => {
                ElemValueAccessor::<u32>::get_vals(new, old, self.phys_inputs, |idx, val| {
                    if let Some(&level) = Self::IN_NOMINAL_LEVELS.iter().nth(val as usize) {
                        unit.set_nominal(idx, level, timeout_ms)?;
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
