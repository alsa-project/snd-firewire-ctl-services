// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use {
    glib::{Error, FileError},
    core::{card_cntr::*, elem_value_accessor::*},
    hinawa::{SndEfw, SndUnitExt},
    alsactl::{ElemId, ElemIfaceType, ElemValue},
    efw_protocols::{ClkSrc, hw_info::*, hw_ctl::*},
};

fn clk_src_to_str(src: &ClkSrc) -> &'static str {
    match src {
        ClkSrc::Internal => "Internal",
        ClkSrc::WordClock => "WordClock",
        ClkSrc::Spdif => "S/PDIF",
        ClkSrc::Adat => "ADAT",
        ClkSrc::Adat2 => "ADAT2",
        ClkSrc::Continuous => "Continuous",
        ClkSrc::Reserved(_) => "Reserved",
    }
}

#[derive(Default)]
pub struct ClkCtl {
    srcs: Vec<ClkSrc>,
    rates: Vec<u32>,
}

const SRC_NAME: &str = "clock-source";
const RATE_NAME: &str = "clock-rate";

impl ClkCtl {
    pub fn load(
        &mut self,
        hwinfo: &HwInfo,
        card_cntr: &mut CardCntr,
    ) -> Result<(), Error> {
        self.srcs.extend_from_slice(&hwinfo.clk_srcs);
        self.rates.extend_from_slice(&hwinfo.clk_rates);

        let labels: Vec<&str> = self.srcs.iter().map(|src| clk_src_to_str(src)).collect();

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, SRC_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let labels: Vec<String> = hwinfo.clk_rates.iter().map(|rate| rate.to_string()).collect();

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, RATE_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        Ok(())
    }

    pub fn read(
        &mut self,
        unit: &mut SndEfw,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            SRC_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let (src, _) = unit.get_clock(timeout_ms)?;
                    if let Some(pos) = self.srcs.iter().position(|s| *s == src) {
                        Ok(pos as u32)
                    } else {
                        let label = "Unexpected value for source of clock";
                        Err(Error::new(FileError::Io, &label))
                    }
                })?;
                Ok(true)
            }
            RATE_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let (_, rate) = unit.get_clock(timeout_ms)?;
                    if let Some(pos) = self.rates.iter().position(|r| *r == rate) {
                        Ok(pos as u32)
                    } else {
                        let label = "Unexpected value for rate of clock";
                        Err(Error::new(FileError::Io, &label))
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
        _: &ElemValue,
        new: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            SRC_NAME => {
                ElemValueAccessor::<u32>::get_val(new, |val| {
                    if let Some(&src) = self.srcs.iter().nth(val as usize) {
                        unit.lock()?;
                        let res = unit.set_clock(Some(src), None, timeout_ms);
                        let _ = unit.unlock();
                        res
                    } else {
                        let label = "Invalid value for source of clock";
                        Err(Error::new(FileError::Io, &label))
                    }
                })?;
                Ok(true)
            }
            RATE_NAME => {
                ElemValueAccessor::<u32>::get_val(new, |val| {
                    if let Some(&rate) = self.rates.iter().nth(val as usize) {
                        unit.lock()?;
                        let res = unit.set_clock(None, Some(rate), timeout_ms);
                        let _ = unit.unlock();
                        res
                    } else {
                        let label = "Invalid value for rate of clock";
                        Err(Error::new(FileError::Io, &label))
                    }
                })?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
