// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use {
    super::*,
    protocols::tcat::{global_section::*, *},
};

#[derive(Default)]
pub struct CommonCtl {
    rates: Vec<ClockRate>,
    srcs: Vec<ClockSource>,
    curr_rate_idx: u32,
    curr_src_idx: u32,
    ext_srcs: Vec<ClockSource>,
    ext_src_states: ExtSourceStates,
    pub notified_elem_list: Vec<ElemId>,
    pub measured_elem_list: Vec<ElemId>,
}

const CLK_RATE_NAME: &str = "clock-rate";
const CLK_SRC_NAME: &str = "clock-source";
const NICKNAME: &str = "nickname";
const LOCKED_CLK_SRC_NAME: &str = "locked-clock-source";
const SLIPPED_CLK_SRC_NAME: &str = "slipped-clock-source";

impl CommonCtl {
    pub fn load(
        &mut self,
        card_cntr: &mut CardCntr,
        caps: &ClockCaps,
        src_labels: &ClockSourceLabels,
    ) -> Result<(), Error> {
        self.rates = caps.get_rate_entries();
        self.srcs = caps.get_src_entries(src_labels);

        let labels = self.rates.iter().map(|r| r.to_string()).collect::<Vec<_>>();

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, CLK_RATE_NAME, 0);
        let mut elem_id_list = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;
        self.notified_elem_list.append(&mut elem_id_list);

        let labels = self
            .srcs
            .iter()
            .map(|s| s.get_label(&src_labels, false).unwrap())
            .collect::<Vec<_>>();

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, CLK_SRC_NAME, 0);
        let mut elem_id_list = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;
        self.notified_elem_list.append(&mut elem_id_list);

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, NICKNAME, 0);
        let _ = card_cntr.add_bytes_elems(&elem_id, 1, NICKNAME_MAX_SIZE, None, true)?;

        self.ext_srcs = ExtSourceStates::get_entries(caps, src_labels);
        let labels = self
            .ext_srcs
            .iter()
            .map(|s| s.get_label(src_labels, true).unwrap())
            .collect::<Vec<_>>();

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, LOCKED_CLK_SRC_NAME, 0);
        let mut elem_id_list = card_cntr.add_bool_elems(&elem_id, 1, labels.len(), false)?;
        self.notified_elem_list.append(&mut elem_id_list);

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, SLIPPED_CLK_SRC_NAME, 0);
        let mut elem_id_list = card_cntr.add_bool_elems(&elem_id, 1, labels.len(), false)?;
        self.measured_elem_list.append(&mut elem_id_list);

        Ok(())
    }

    fn cache_clock_config(&mut self, config: &ClockConfig) -> Result<(), Error> {
        self.rates
            .iter()
            .position(|&r| r == config.rate)
            .ok_or_else(|| {
                let msg = format!("Unexpected value read for clock rate: {}", config.rate);
                Error::new(FileError::Io, &msg)
            })
            .map(|pos| self.curr_rate_idx = pos as u32)?;
        self.srcs
            .iter()
            .position(|&s| s == config.src)
            .ok_or_else(|| {
                let msg = format!("Unexpected value read for clock source: {}", config.src);
                Error::new(FileError::Io, &msg)
            })
            .map(|pos| self.curr_src_idx = pos as u32)
    }

    pub fn read(
        &mut self,
        unit: &mut (SndDice, FwNode),
        req: &mut FwReq,
        sections: &GeneralSections,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            CLK_RATE_NAME => {
                let config = GlobalSectionProtocol::read_clock_config(
                    req,
                    &mut unit.1,
                    sections,
                    timeout_ms,
                )?;
                self.cache_clock_config(&config)?;
                ElemValueAccessor::<u32>::set_val(elem_value, || Ok(self.curr_rate_idx))
                    .map(|_| true)
            }
            CLK_SRC_NAME => {
                let config = GlobalSectionProtocol::read_clock_config(
                    req,
                    &mut unit.1,
                    sections,
                    timeout_ms,
                )?;
                self.cache_clock_config(&config)?;
                ElemValueAccessor::<u32>::set_val(elem_value, || Ok(self.curr_src_idx))
                    .map(|_| true)
            }
            NICKNAME => {
                GlobalSectionProtocol::read_nickname(req, &mut unit.1, sections, timeout_ms).map(
                    |name| {
                        let mut vals = vec![0; NICKNAME_MAX_SIZE];
                        let raw = name.as_bytes();
                        vals[..raw.len()].copy_from_slice(&raw);
                        elem_value.set_bytes(&vals);
                        true
                    },
                )
            }
            _ => Ok(false),
        }
    }

    fn update_clock_config(
        &mut self,
        config: &mut ClockConfig,
        rate: Option<u32>,
        src: Option<u32>,
    ) -> Result<(), Error> {
        if let Some(pos) = rate {
            self.rates
                .iter()
                .nth(pos as usize)
                .ok_or_else(|| {
                    let msg = format!(
                        "Invalid value for index of rate: {} greater than {}",
                        pos,
                        self.rates.len()
                    );
                    Error::new(FileError::Inval, &msg)
                })
                .map(|&r| config.rate = r)?;
        }
        if let Some(pos) = src {
            self.srcs
                .iter()
                .nth(pos as usize)
                .ok_or_else(|| {
                    let msg = format!(
                        "Invalid value for index of source: {} greater than {}",
                        pos,
                        self.srcs.len()
                    );
                    Error::new(FileError::Inval, &msg)
                })
                .map(|&s| config.src = s)?;
        }
        Ok(())
    }

    pub fn write(
        &mut self,
        unit: &mut (SndDice, FwNode),
        req: &mut FwReq,
        sections: &GeneralSections,
        elem_id: &ElemId,
        _: &ElemValue,
        new: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            CLK_RATE_NAME => ElemValueAccessor::<u32>::get_val(new, |val| {
                unit.0.lock()?;
                let res = GlobalSectionProtocol::read_clock_config(
                    req,
                    &mut unit.1,
                    sections,
                    timeout_ms,
                )
                .and_then(|mut config| {
                    self.update_clock_config(&mut config, Some(val as u32), None)?;
                    GlobalSectionProtocol::write_clock_config(
                        req,
                        &mut unit.1,
                        sections,
                        config,
                        timeout_ms,
                    )?;
                    self.curr_rate_idx = val;
                    Ok(())
                });
                let _ = unit.0.unlock();
                res
            })
            .map(|_| true),
            CLK_SRC_NAME => ElemValueAccessor::<u32>::get_val(new, |val| {
                unit.0.lock()?;
                let res = GlobalSectionProtocol::read_clock_config(
                    req,
                    &mut unit.1,
                    sections,
                    timeout_ms,
                )
                .and_then(|mut config| {
                    self.update_clock_config(&mut config, None, Some(val as u32))?;
                    GlobalSectionProtocol::write_clock_config(
                        req,
                        &mut unit.1,
                        sections,
                        config,
                        timeout_ms,
                    )?;
                    self.curr_src_idx = val;
                    Ok(())
                });
                let _ = unit.0.unlock();
                res
            })
            .map(|_| true),
            NICKNAME => {
                let vals = &new.bytes()[..NICKNAME_MAX_SIZE];
                std::str::from_utf8(vals)
                    .map_err(|e| {
                        let msg = format!("Invalid bytes for string: {}", e);
                        Error::new(FileError::Inval, &msg)
                    })
                    .and_then(|text| {
                        text.find('\0')
                            .ok_or(Error::new(FileError::Inval, "Unterminated string found"))
                            .and_then(|pos| {
                                GlobalSectionProtocol::write_nickname(
                                    req,
                                    &mut unit.1,
                                    sections,
                                    &text[..pos],
                                    timeout_ms,
                                )
                            })
                    })
                    .map(|_| true)
            }
            _ => Ok(false),
        }
    }

    pub fn parse_notification(
        &mut self,
        unit: &mut (SndDice, FwNode),
        req: &mut FwReq,
        sections: &GeneralSections,
        msg: u32,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        if GeneralProtocol::has_clock_accepted(msg) {
            let config =
                GlobalSectionProtocol::read_clock_config(req, &mut unit.1, sections, timeout_ms)?;
            self.cache_clock_config(&config)?;
        }

        if GeneralProtocol::has_ext_status_changed(msg) {
            self.ext_src_states = GlobalSectionProtocol::read_clock_source_states(
                req,
                &mut unit.1,
                sections,
                timeout_ms,
            )?;
        }

        Ok(())
    }

    pub fn read_notified_elem(
        &mut self,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            CLK_RATE_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || Ok(self.curr_rate_idx))
                    .map(|_| true)
            }
            CLK_SRC_NAME => ElemValueAccessor::<u32>::set_val(elem_value, || Ok(self.curr_src_idx))
                .map(|_| true),
            LOCKED_CLK_SRC_NAME => {
                ElemValueAccessor::<bool>::set_vals(elem_value, self.ext_srcs.len(), |idx| {
                    Ok(self.ext_srcs[idx].is_locked(&self.ext_src_states))
                })
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }

    pub fn measure_states(
        &mut self,
        unit: &mut (SndDice, FwNode),
        req: &mut FwReq,
        sections: &GeneralSections,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        GlobalSectionProtocol::read_clock_source_states(req, &mut unit.1, sections, timeout_ms)
            .map(|states| self.ext_src_states = states)
    }

    pub fn measure_elem(
        &mut self,
        elem_id: &ElemId,
        elem_value: &ElemValue,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            SLIPPED_CLK_SRC_NAME => {
                ElemValueAccessor::<bool>::set_vals(elem_value, self.ext_srcs.len(), |idx| {
                    Ok(self.ext_srcs[idx].is_slipped(&self.ext_src_states))
                })
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }
}
