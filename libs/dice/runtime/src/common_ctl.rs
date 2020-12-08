// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::{Error, FileError};

use alsactl::{ElemId, ElemIfaceType};
use alsactl::{ElemValue, ElemValueExt, ElemValueExtManual};

use hinawa::{SndDice, SndUnitExt};
use hinawa::FwReq;

use core::card_cntr::*;
use core::elem_value_accessor::ElemValueAccessor;

use dice_protocols::tcat::{*, global_section::*};

#[derive(Default)]
pub struct CommonCtl {
    rates: Vec<ClockRate>,
    srcs: Vec<ClockSource>,
    curr_rate_idx: u32,
    curr_src_idx: u32,
}

const CLK_RATE_NAME: & str = "clock-rate";
const CLK_SRC_NAME: &str = "clock-source";
const NICKNAME: &str = "nickname";

impl CommonCtl {
    pub fn load(&mut self, card_cntr: &mut CardCntr, caps: &ClockCaps, src_labels: &ClockSourceLabels)
        -> Result<(), Error>
    {
        self.rates = caps.get_rate_entries();
        self.srcs = caps.get_src_entries(src_labels);

        let labels = self.rates.iter()
            .map(|r| r.to_string())
            .collect::<Vec<_>>();

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, CLK_RATE_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let labels = self.srcs.iter()
            .map(|s| s.get_label(&src_labels, false).unwrap())
            .collect::<Vec<_>>();

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, CLK_SRC_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, NICKNAME, 0);
        let _ = card_cntr.add_bytes_elems(&elem_id, 1, NICKNAME_MAX_SIZE, None, true)?;

        Ok(())
    }

    fn cache_clock_config(&mut self, config: &ClockConfig)
        -> Result<(), Error>
    {
        self.rates.iter().position(|&r| r == config.rate)
            .ok_or_else(|| {
                let msg = format!("Unexpected value read for clock rate: {}", config.rate);
                Error::new(FileError::Io, &msg)
            })
            .map(|pos| self.curr_rate_idx = pos as u32)?;
        self.srcs.iter().position(|&s| s == config.src)
            .ok_or_else(|| {
                let msg = format!("Unexpected value read for clock source: {}", config.src);
                Error::new(FileError::Io, &msg)
            })
            .map(|pos| self.curr_src_idx = pos as u32)
    }

    pub fn read(&mut self, unit: &SndDice, proto: &FwReq, sections: &GeneralSections,
                elem_id: &ElemId, elem_value: &ElemValue, timeout_ms: u32)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            CLK_RATE_NAME => {
                let config = proto.read_clock_config(&unit.get_node(), sections, timeout_ms)?;
                self.cache_clock_config(&config)?;
                ElemValueAccessor::<u32>::set_val(elem_value, || Ok(self.curr_rate_idx))
                .map(|_| true)
            }
            CLK_SRC_NAME => {
                let config = proto.read_clock_config(&unit.get_node(), sections, timeout_ms)?;
                self.cache_clock_config(&config)?;
                ElemValueAccessor::<u32>::set_val(elem_value, || Ok(self.curr_src_idx))
                .map(|_| true)
            }
            NICKNAME => {
                proto.read_nickname(&unit.get_node(), sections, timeout_ms)
                    .map(|name| {
                        let mut vals = vec![0;NICKNAME_MAX_SIZE];
                        let raw = name.as_bytes();
                        vals[..raw.len()].copy_from_slice(&raw);
                        elem_value.set_bytes(&vals);
                        true
                    })
            }
            _ => Ok(false),
        }
    }

    fn update_clock_config(&mut self, config: &mut ClockConfig, rate: Option<u32>, src: Option<u32>)
        -> Result<(), Error>
    {
        if let Some(pos) = rate {
            self.rates.iter()
                .nth(pos as usize)
                .ok_or_else(|| {
                    let msg = format!("Invalid value for index of rate: {} greater than {}",
                                      pos, self.rates.len());
                    Error::new(FileError::Inval, &msg)
                })
                .map(|&r| config.rate = r)?;
        }
        if let Some(pos) = src {
            self.srcs.iter()
                .nth(pos as usize)
                .ok_or_else(|| {
                    let msg = format!("Invalid value for index of source: {} greater than {}",
                                      pos, self.srcs.len());
                    Error::new(FileError::Inval, &msg)
                })
                .map(|&s| config.src = s)?;
        }
        Ok(())
    }

    pub fn write(&mut self, unit: &SndDice, proto: &FwReq, sections: &GeneralSections,
                 elem_id: &ElemId, _: &ElemValue, new: &ElemValue, timeout_ms: u32)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            CLK_RATE_NAME => {
                ElemValueAccessor::<u32>::get_val(new, |val| {
                    unit.lock()?;
                    let res = proto.read_clock_config(&unit.get_node(), sections, timeout_ms)
                        .and_then(|mut config| {
                            self.update_clock_config(&mut config, Some(val as u32), None)?;
                            proto.write_clock_config(&unit.get_node(), sections, config,
                                                     timeout_ms)?;
                            self.curr_rate_idx = val;
                            Ok(())
                        });
                    let _ = unit.unlock();
                    res
                })
                .map(|_| true)
            }
            CLK_SRC_NAME => {
                ElemValueAccessor::<u32>::get_val(new, |val| {
                    unit.lock()?;
                    let res = proto.read_clock_config(&unit.get_node(), sections, timeout_ms)
                        .and_then(|mut config| {
                            self.update_clock_config(&mut config, None, Some(val as u32))?;
                            proto.write_clock_config(&unit.get_node(), sections, config,
                                                     timeout_ms)?;
                            self.curr_src_idx = val;
                            Ok(())
                        });
                    let _ = unit.unlock();
                    res
                })
                .map(|_| true)
            }
            NICKNAME => {
                let mut vals = vec![0;NICKNAME_MAX_SIZE];
                new.get_bytes(&mut vals);
                std::str::from_utf8(&vals)
                    .map_err(|e| {
                        let msg = format!("Invalid bytes for string: {}", e);
                        Error::new(FileError::Inval, &msg)
                    })
                    .and_then(|text| {
                        text.find('\0')
                            .ok_or(Error::new(FileError::Inval, "Unterminated string found"))
                            .and_then(|pos| {
                                proto.write_nickname(&unit.get_node(), sections, &text[..pos], timeout_ms)
                            })
                    })
                    .map(|_| true)
            }
            _ => Ok(false),
        }
    }
}
