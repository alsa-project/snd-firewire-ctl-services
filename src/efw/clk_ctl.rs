// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use crate::card_cntr;

use hinawa::SndUnitExt;

use alsactl::{ElemValueExt, ElemValueExtManual};

use super::transactions::{ClkSrc, HwInfo, EfwHwCtl};

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
            .map(|src| src.to_string())
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
                let (src, _) = EfwHwCtl::get_clock(unit)?;
                if let Some(pos) = self.srcs.iter().position(|s| *s == src) {
                    elem_value.set_enum(&[pos as u32]);
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            Self::RATE_NAME => {
                let (_, rate) = EfwHwCtl::get_clock(unit)?;
                if let Some(pos) = self.rates.iter().position(|r| *r == rate) {
                    elem_value.set_enum(&[pos as u32]);
                    Ok(true)
                } else {
                    Ok(false)
                }
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
                let mut vals = [0];
                new.get_enum(&mut vals);
                if let Some(&src) = self.srcs.iter().nth(vals[0] as usize) {
                    unit.lock()?;
                    let res = EfwHwCtl::set_clock(unit, Some(src), None);
                    let _ = unit.unlock();
                    match res {
                        Err(err) => Err(err),
                        Ok(()) => Ok(true),
                    }
                } else {
                    Ok(false)
                }
            }
            Self::RATE_NAME => {
                let mut vals = [0];
                new.get_enum(&mut vals);
                if let Some(&rate) = self.rates.iter().nth(vals[0] as usize) {
                    unit.lock()?;
                    let res = EfwHwCtl::set_clock(unit, None, Some(rate));
                    let _ = unit.unlock();
                    match res {
                        Err(err) => Err(err),
                        Ok(()) => Ok(true),
                    }
                } else {
                    Ok(false)
                }
            }
            _ => Ok(false),
        }
    }
}

impl ToString for ClkSrc {
    fn to_string(&self) -> String {
        match self {
            ClkSrc::Internal => "Internal",
            ClkSrc::WordClock => "WordClock",
            ClkSrc::Spdif => "S/PDIF",
            ClkSrc::Adat => "ADAT",
            ClkSrc::Adat2 => "ADAT2",
            ClkSrc::Continuous => "Continuous",
            ClkSrc::Unknown(_) => "Unknown",
        }.to_string()
    }
}
