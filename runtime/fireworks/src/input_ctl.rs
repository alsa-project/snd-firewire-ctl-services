// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use {
    super::*,
    protocols::{hw_info::*, phys_input::*, *},
};

#[derive(Default)]
pub struct InputCtl {
    params: EfwPhysInputParameters,
}

const IN_NOMINAL_NAME: &str = "input-nominal";

impl InputCtl {
    const IN_NOMINAL_LABELS: [&'static str; 2] = ["+4dBu", "-10dBV"];
    const IN_NOMINAL_LEVELS: [NominalSignalLevel; 2] = [
        NominalSignalLevel::Professional,
        NominalSignalLevel::Consumer,
    ];

    fn cache(&mut self, hw_info: &HwInfo, unit: &mut SndEfw, timeout_ms: u32) -> Result<(), Error> {
        if hw_info
            .caps
            .iter()
            .find(|cap| HwCap::NominalInput.eq(cap))
            .is_some()
        {
            let count = hw_info
                .phys_inputs
                .iter()
                .fold(0, |accm, entry| accm + entry.group_count);

            self.params.nominals = vec![Default::default(); count];

            let has_fpga = hw_info
                .caps
                .iter()
                .find(|&cap| *cap == HwCap::Fpga)
                .is_some();
            if has_fpga {
                // FPGA models return invalid state of nominal level.
                self.params
                    .nominals
                    .iter()
                    .enumerate()
                    .try_for_each(|(ch, &level)| unit.set_nominal(ch, level, timeout_ms))?;
            } else {
                self.params
                    .nominals
                    .iter_mut()
                    .enumerate()
                    .try_for_each(|(ch, level)| {
                        unit.get_nominal(ch, timeout_ms).map(|l| *level = l)
                    })?;
            }
        }

        Ok(())
    }

    pub fn load(
        &mut self,
        unit: &mut SndEfw,
        hwinfo: &HwInfo,
        card_cntr: &mut CardCntr,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        self.cache(hwinfo, unit, timeout_ms)?;

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
                self.params.nominals.len(),
                &Self::IN_NOMINAL_LABELS,
                None,
                true,
            )?;
        }

        Ok(())
    }

    pub fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            IN_NOMINAL_NAME => {
                let vals: Vec<u32> = self
                    .params
                    .nominals
                    .iter()
                    .map(|level| {
                        Self::IN_NOMINAL_LEVELS
                            .iter()
                            .position(|l| level.eq(l))
                            .map(|pos| pos as u32)
                            .unwrap()
                    })
                    .collect();
                elem_value.set_enum(&vals);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub fn write(
        &mut self,
        unit: &mut SndEfw,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            IN_NOMINAL_NAME => {
                self.params
                    .nominals
                    .iter_mut()
                    .zip(elem_value.enumerated())
                    .enumerate()
                    .try_for_each(|(ch, (o, &val))| {
                        let pos = val as usize;
                        let level = Self::IN_NOMINAL_LEVELS
                            .iter()
                            .nth(pos)
                            .ok_or_else(|| {
                                let label =
                                    format!("Invalid value for nominal level of input: {}", pos);
                                Error::new(FileError::Inval, &label)
                            })
                            .copied()?;
                        unit.set_nominal(ch, level, timeout_ms).map(|_| *o = level)
                    })?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
