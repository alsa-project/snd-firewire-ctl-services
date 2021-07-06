// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::{Error, FileError};

use core::card_cntr;
use core::elem_value_accessor::ElemValueAccessor;

use hinawa::SndUnitExt;

use efw_protocols::transactions::EfwHwCtl;
use efw_protocols::ClkSrc;
use efw_protocols::hw_info::*;

fn clk_src_to_string(src: &ClkSrc) -> String {
    match src {
        ClkSrc::Internal => "Internal",
        ClkSrc::WordClock => "WordClock",
        ClkSrc::Spdif => "S/PDIF",
        ClkSrc::Adat => "ADAT",
        ClkSrc::Adat2 => "ADAT2",
        ClkSrc::Continuous => "Continuous",
        ClkSrc::Reserved(_) => "Reserved",
    }.to_string()
}

pub struct ClkCtl {
    srcs: Vec<ClkSrc>,
    rates: Vec<u32>,
}

impl<'a> ClkCtl {
    const SRC_NAME: &'a str = "clock-source";
    const RATE_NAME: &'a str = "clock-rate";

    pub fn new() -> Self {
        ClkCtl {
            srcs: Vec::new(),
            rates: Vec::new(),
        }
    }

    pub fn load(
        &mut self,
        hwinfo: &HwInfo,
        card_cntr: &mut card_cntr::CardCntr,
    ) -> Result<(), Error> {
        self.srcs.extend_from_slice(&hwinfo.clk_srcs);
        self.rates.extend_from_slice(&hwinfo.clk_rates);

        let labels = self.srcs.iter()
            .map(|src| clk_src_to_string(src))
            .collect::<Vec<String>>();

        let elem_id =
            alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Card, 0, 0, Self::SRC_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let labels = hwinfo.clk_rates.iter()
            .map(|rate| rate.to_string())
            .collect::<Vec<String>>();

        let elem_id =
            alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Card, 0, 0, Self::RATE_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        Ok(())
    }

    pub fn read(
        &mut self,
        unit: &hinawa::SndEfw,
        elem_id: &alsactl::ElemId,
        elem_value: &mut alsactl::ElemValue,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            Self::SRC_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let (src, _) = EfwHwCtl::get_clock(unit)?;
                    if let Some(pos) = self.srcs.iter().position(|s| *s == src) {
                        Ok(pos as u32)
                    } else {
                        let label = "Unexpected value for source of clock";
                        Err(Error::new(FileError::Io, &label))
                    }
                })?;
                Ok(true)
            }
            Self::RATE_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let (_, rate) = EfwHwCtl::get_clock(unit)?;
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
        unit: &hinawa::SndEfw,
        elem_id: &alsactl::ElemId,
        _: &alsactl::ElemValue,
        new: &alsactl::ElemValue,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            Self::SRC_NAME => {
                ElemValueAccessor::<u32>::get_val(new, |val| {
                    if let Some(&src) = self.srcs.iter().nth(val as usize) {
                        unit.lock()?;
                        let res = EfwHwCtl::set_clock(unit, Some(src), None);
                        let _ = unit.unlock();
                        res
                    } else {
                        let label = "Invalid value for source of clock";
                        Err(Error::new(FileError::Io, &label))
                    }
                })?;
                Ok(true)
            }
            Self::RATE_NAME => {
                ElemValueAccessor::<u32>::get_val(new, |val| {
                    if let Some(&rate) = self.rates.iter().nth(val as usize) {
                        unit.lock()?;
                        let res = EfwHwCtl::set_clock(unit, None, Some(rate));
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
