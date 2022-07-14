// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use {
    super::*,
    alsa_ctl_tlv_codec::DbInterval,
    efw_protocols::{hw_info::*, phys_output::*, *},
};

#[derive(Default)]
pub struct OutputCtl {
    phys_outputs: usize,
}

const OUT_VOL_NAME: &str = "output-volume";
const OUT_MUTE_NAME: &str = "output-mute";
const OUT_NOMINAL_NAME: &str = "output-nominal";

impl OutputCtl {
    // The fixed point number of 8.24 format.
    const COEF_MIN: i32 = 0x00000000;
    const COEF_MAX: i32 = 0x02000000;
    const COEF_STEP: i32 = 0x00000001;
    const COEF_TLV: DbInterval = DbInterval {
        min: -14400,
        max: 600,
        linear: false,
        mute_avail: false,
    };

    const OUT_NOMINAL_LABELS: [&'static str; 2] = ["+4dBu", "-10dBV"];
    const OUT_NOMINAL_LEVELS: [NominalSignalLevel; 2] = [
        NominalSignalLevel::Professional,
        NominalSignalLevel::Consumer,
    ];

    pub fn load(&mut self, hwinfo: &HwInfo, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.phys_outputs = hwinfo.phys_outputs.iter().fold(0, |accm, entry| {
            if entry.group_type != PhysGroupType::AnalogMirror {
                accm + entry.group_count
            } else {
                accm
            }
        });

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, OUT_VOL_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            1,
            Self::COEF_MIN,
            Self::COEF_MAX,
            Self::COEF_STEP,
            self.phys_outputs,
            Some(&Into::<Vec<u32>>::into(Self::COEF_TLV)),
            true,
        )?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, OUT_MUTE_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, self.phys_outputs, true)?;

        if hwinfo
            .caps
            .iter()
            .find(|&cap| *cap == HwCap::NominalOutput)
            .is_some()
        {
            let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, OUT_NOMINAL_NAME, 0);
            let _ = card_cntr.add_enum_elems(
                &elem_id,
                1,
                self.phys_outputs,
                &Self::OUT_NOMINAL_LABELS,
                None,
                true,
            )?;
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
            OUT_VOL_NAME => {
                ElemValueAccessor::<i32>::set_vals(elem_value, self.phys_outputs, |idx| {
                    unit.get_vol(idx, timeout_ms)
                })?;
                Ok(true)
            }
            OUT_MUTE_NAME => {
                ElemValueAccessor::<bool>::set_vals(elem_value, self.phys_outputs, |idx| {
                    unit.get_mute(idx, timeout_ms)
                })?;
                Ok(true)
            }
            OUT_NOMINAL_NAME => {
                ElemValueAccessor::<u32>::set_vals(elem_value, self.phys_outputs, |idx| {
                    let level = unit.get_nominal(idx, timeout_ms)?;
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
        unit: &mut SndEfw,
        elem_id: &ElemId,
        old: &ElemValue,
        new: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            OUT_VOL_NAME => {
                ElemValueAccessor::<i32>::get_vals(new, old, self.phys_outputs, |idx, val| {
                    unit.set_vol(idx, val, timeout_ms)
                })?;
                Ok(true)
            }
            OUT_MUTE_NAME => {
                ElemValueAccessor::<bool>::get_vals(new, old, self.phys_outputs, |idx, val| {
                    unit.set_mute(idx, val, timeout_ms)
                })?;
                Ok(true)
            }
            OUT_NOMINAL_NAME => {
                ElemValueAccessor::<u32>::get_vals(new, old, self.phys_outputs, |idx, val| {
                    if let Some(&level) = Self::OUT_NOMINAL_LEVELS.iter().nth(val as usize) {
                        unit.set_nominal(idx, level, timeout_ms)
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
