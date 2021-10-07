// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto
use glib::{Error, FileError};

use hinawa::FwReq;
use hinawa::{SndMotu, SndUnitExt};

use alsactl::{ElemId, ElemIfaceType, ElemValue};

use core::card_cntr::CardCntr;
use core::elem_value_accessor::ElemValueAccessor;

use motu_protocols::version_3::*;

use super::*;

fn clk_src_to_str(src: &V3ClkSrc) -> &'static str {
    match src {
        V3ClkSrc::Internal => "Internal",
        V3ClkSrc::SpdifCoax => "S/PDIF-on-coax",
        V3ClkSrc::WordClk => "Word-clk-on-BNC",
        V3ClkSrc::SignalOptA => "Signal-on-opt-A",
        V3ClkSrc::SignalOptB => "Signal-on-opt-B",
    }
}

const RATE_NAME: &str = "sampling-rate";
const SRC_NAME: &str = "clock-source";

pub trait V3ClkCtlOperation<T: V3ClkOperation> {
    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let labels: Vec<&str> = T::CLK_RATES
            .iter()
            .map(|e| clk_rate_to_str(&e.0))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, RATE_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let labels: Vec<&str> = T::CLK_SRCS.iter().map(|e| clk_src_to_str(&e.0)).collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, SRC_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        Ok(())
    }

    fn read(
        &mut self,
        unit: &mut SndMotu,
        req: &mut FwReq,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            RATE_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    T::get_clk_rate(req, &mut unit.get_node(), timeout_ms).map(|val| val as u32)
                })
                .map(|_| true)
            }
            SRC_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let mut node = unit.get_node();
                    let val = T::get_clk_src(req, &mut node, timeout_ms)?;
                    if T::HAS_LCD {
                        let label = clk_src_to_str(&T::CLK_SRCS[val].0);
                        let _ = T::update_clk_display(req, &mut node, &label, timeout_ms);
                    }
                    Ok(val as u32)
                })
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }

    fn write(
        &mut self,
        unit: &mut SndMotu,
        req: &mut FwReq,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            RATE_NAME => {
                ElemValueAccessor::<u32>::get_val(elem_value, |val| {
                    unit.lock()?;
                    let res = T::set_clk_rate(req, &mut unit.get_node(), val as usize, timeout_ms);
                    let _ = unit.unlock();
                    res
                })
                .map(|_| true)
            }
            SRC_NAME => {
                ElemValueAccessor::<u32>::get_val(elem_value, |val| {
                    let prev_src = T::get_clk_src(req, &mut unit.get_node(), timeout_ms)?;
                    unit.lock()?;
                    let mut node = unit.get_node();
                    let mut res = T::set_clk_src(req, &mut node, val as usize, timeout_ms);
                    if res.is_ok() && T::HAS_LCD {
                        let label = clk_src_to_str(&T::CLK_SRCS[val as usize].0);
                        res = T::update_clk_display(req, &mut node, &label, timeout_ms);
                        if res.is_err() {
                            let _ = T::set_clk_src(req, &mut node, prev_src, timeout_ms);
                        }
                    }
                    let _ = unit.unlock();
                    res
                })
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

const MAIN_ASSIGN_NAME: &str = "main-assign";
const RETURN_ASSIGN_NAME: &str = "return-assign";

pub trait V3PortAssignCtlOperation<T: V3PortAssignOperation> {
    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<Vec<ElemId>, Error> {
        let mut notified_elem_id_list = Vec::new();

        let labels: Vec<&str> = T::ASSIGN_PORTS
            .iter()
            .map(|p| target_port_to_str(&p.0))
            .collect();

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MAIN_ASSIGN_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, true)
            .map(|elem_id_list| notified_elem_id_list.extend_from_slice(&elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, RETURN_ASSIGN_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, true)
            .map(|elem_id_list| notified_elem_id_list.extend_from_slice(&elem_id_list))?;

        Ok(notified_elem_id_list)
    }

    fn read(
        &mut self,
        unit: &mut SndMotu,
        req: &mut FwReq,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            MAIN_ASSIGN_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    T::get_main_assign(req, &mut unit.get_node(), timeout_ms)
                        .map(|val| val as u32)
                })
                .map(|_| true)
            }
            RETURN_ASSIGN_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    T::get_return_assign(req, &mut unit.get_node(), timeout_ms)
                        .map(|val| val as u32)
                })
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }

    fn write(
        &mut self,
        unit: &mut SndMotu,
        req: &mut FwReq,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            MAIN_ASSIGN_NAME => {
                ElemValueAccessor::<u32>::get_val(elem_value, |val| {
                    T::set_main_assign(req, &mut unit.get_node(), val as usize, timeout_ms)
                })
                .map(|_| true)
            }
            RETURN_ASSIGN_NAME => {
                ElemValueAccessor::<u32>::get_val(elem_value, |val| {
                    T::set_return_assign(req, &mut unit.get_node(), val as usize, timeout_ms)
                })
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

fn opt_iface_mode_to_str(mode: &V3OptIfaceMode) -> &'static str {
    match mode {
        V3OptIfaceMode::Disabled => "Disabled",
        V3OptIfaceMode::Adat => "ADAT",
        V3OptIfaceMode::Spdif => "S/PDIF",
    }
}

const OPT_IFACE_IN_MODE_NAME: &str = "optical-iface-in-mode";
const OPT_IFACE_OUT_MODE_NAME: &str = "optical-iface-out-mode";

pub trait V3OptIfaceCtlOperation<T: V3OptIfaceOperation> {
    const MODES: [V3OptIfaceMode; 3] = [
        V3OptIfaceMode::Disabled,
        V3OptIfaceMode::Adat,
        V3OptIfaceMode::Spdif,
    ];
    const TARGETS: [V3OptIfaceTarget; 2] = [V3OptIfaceTarget::A, V3OptIfaceTarget::B];

    fn load(&self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let labels: Vec<&str> = Self::MODES.iter()
            .map(|m| opt_iface_mode_to_str(m))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, OPT_IFACE_IN_MODE_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, Self::TARGETS.len(), &labels, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, OPT_IFACE_OUT_MODE_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, Self::TARGETS.len(), &labels, None, true)?;

        Ok(())
    }

    fn read(
        &self,
        unit: &mut SndMotu,
        req: &mut FwReq,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            OPT_IFACE_IN_MODE_NAME => {
                ElemValueAccessor::<u32>::set_vals(elem_value, Self::TARGETS.len(), |idx| {
                    T::get_opt_input_iface_mode(
                        req,
                        &mut unit.get_node(),
                        Self::TARGETS[idx],
                        timeout_ms
                    )
                        .map(|mode| {
                            Self::MODES.iter().position(|m| m.eq(&mode)).unwrap() as u32
                        })
                })
                .map(|_| true)
            }
            OPT_IFACE_OUT_MODE_NAME => {
                ElemValueAccessor::<u32>::set_vals(elem_value, Self::TARGETS.len(), |idx| {
                    T::get_opt_output_iface_mode(
                        req,
                        &mut unit.get_node(),
                        Self::TARGETS[idx],
                        timeout_ms
                    )
                        .map(|mode| {
                            Self::MODES.iter().position(|m| m.eq(&mode)).unwrap() as u32
                        })
                })
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }

    fn write(
        &self,
        unit: &mut SndMotu,
        req: &mut FwReq,
        elem_id: &ElemId,
        old: &ElemValue,
        new: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            OPT_IFACE_IN_MODE_NAME => {
                unit.lock()?;
                let res = ElemValueAccessor::<u32>::get_vals(new, old, Self::TARGETS.len(), |idx, val| {
                    let &mode = Self::MODES.iter()
                        .nth(val as usize)
                        .ok_or_else(|| {
                            let msg = format!("Invalid index for mode of opt interface: {}", val);
                            Error::new(FileError::Inval, &msg)
                        })?;
                    T::set_opt_input_iface_mode(
                        req,
                        &mut unit.get_node(),
                        Self::TARGETS[idx],
                        mode,
                        timeout_ms,
                    )
                });
                let _ = unit.unlock();
                res.and(Ok(true))
            }
            OPT_IFACE_OUT_MODE_NAME => {
                unit.lock()?;
                let res = ElemValueAccessor::<u32>::get_vals(new, old, Self::TARGETS.len(), |idx, val| {
                    let &mode = Self::MODES.iter()
                        .nth(val as usize)
                        .ok_or_else(|| {
                            let msg = format!("Invalid index for mode of opt interface: {}", val);
                            Error::new(FileError::Inval, &msg)
                        })?;
                    T::set_opt_input_iface_mode(
                        req,
                        &mut unit.get_node(),
                        Self::TARGETS[idx],
                        mode,
                        timeout_ms,
                    )
                });
                let _ = unit.unlock();
                res.and(Ok(true))
            }
            _ => Ok(false),
        }
    }
}
