// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto
use glib::Error;

use hinawa::FwReq;
use hinawa::{SndMotu, SndUnitExt};

use alsactl::{ElemId, ElemIfaceType, ElemValue};

use core::card_cntr::CardCntr;
use core::elem_value_accessor::ElemValueAccessor;

use motu_protocols::version_3::*;

use super::model::clk_rate_to_string;

fn clk_src_to_label(src: &V3ClkSrc) -> String {
    match src {
        V3ClkSrc::Internal => "Internal",
        V3ClkSrc::SpdifCoax => "S/PDIF-on-coax",
        V3ClkSrc::WordClk => "Word-clk-on-BNC",
        V3ClkSrc::SignalOptA => "Signal-on-opt-A",
        V3ClkSrc::SignalOptB => "Signal-on-opt-B",
    }
    .to_string()
}

const RATE_NAME: &str = "sampling-rate";
const SRC_NAME: &str = "clock-source";

pub trait V3ClkCtlOperation<T: V3ClkProtocol> {
    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let labels: Vec<String> = T::CLK_RATES
            .iter()
            .map(|e| clk_rate_to_string(&e.0))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, RATE_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let labels: Vec<String> = T::CLK_SRCS.iter().map(|e| clk_src_to_label(&e.0)).collect();
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
                        let label = clk_src_to_label(&T::CLK_SRCS[val].0);
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
                        let label = clk_src_to_label(&T::CLK_SRCS[val as usize].0);
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

pub trait V3PortAssignCtlOperation<T: V3PortAssignProtocol> {
    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<Vec<ElemId>, Error> {
        let mut notified_elem_id_list = Vec::new();

        let labels: Vec<String> = T::ASSIGN_PORTS.iter().map(|e| e.0.to_string()).collect();

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MAIN_ASSIGN_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, true)
            .map(|elem_id_list| notified_elem_id_list.extend_from_slice(&elem_id_list))?;

        let elem_id =
            alsactl::ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, RETURN_ASSIGN_NAME, 0);
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

#[derive(Default)]
pub struct V3OptIfaceCtl;

impl V3OptIfaceCtl {
    const OPT_IFACE_IN_MODE_NAME: &'static str = "optical-iface-in-mode";
    const OPT_IFACE_OUT_MODE_NAME: &'static str = "optical-iface-out-mode";

    const OPT_IFACE_MODE_LABELS: &'static [&'static str] = &["None", "ADAT", "S/PDIF"];

    pub fn load<O>(&mut self, _: &O, card_cntr: &mut CardCntr) -> Result<(), Error>
    where
        O: V3OptIfaceProtocol,
    {
        let elem_id =
            ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::OPT_IFACE_IN_MODE_NAME, 0);
        let _ =
            card_cntr.add_enum_elems(&elem_id, 1, 2, Self::OPT_IFACE_MODE_LABELS, None, true)?;

        let elem_id =
            ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::OPT_IFACE_OUT_MODE_NAME, 0);
        let _ =
            card_cntr.add_enum_elems(&elem_id, 1, 2, Self::OPT_IFACE_MODE_LABELS, None, true)?;

        Ok(())
    }

    fn get_opt_iface_mode<O>(
        &mut self,
        unit: &mut SndMotu,
        req: &mut FwReq,
        _: &O,
        is_out: bool,
        is_b: bool,
        timeout_ms: u32,
    ) -> Result<u32, Error>
    where
        O: V3OptIfaceProtocol,
    {
        O::get_opt_iface_mode(req, &mut unit.get_node(), is_out, is_b, timeout_ms)
            .map(|(enabled, no_adat)| {
                if enabled {
                    0
                } else {
                    if no_adat {
                        2
                    } else {
                        1
                    }
                }
            })
    }

    fn set_opt_iface_mode<O>(
        &mut self,
        unit: &mut SndMotu,
        req: &mut FwReq,
        _: &O,
        is_out: bool,
        is_b: bool,
        mode: u32,
        timeout_ms: u32,
    ) -> Result<(), Error>
    where
        O: V3OptIfaceProtocol,
    {
        let (enabled, no_adat) = match mode {
            0 => (false, false),
            1 => (true, false),
            2 => (true, true),
            _ => unreachable!(),
        };
        O::set_opt_iface_mode(req, &mut unit.get_node(), is_out, is_b, enabled, no_adat, timeout_ms)
    }

    pub fn read<O>(
        &mut self,
        unit: &mut SndMotu,
        req: &mut FwReq,
        proto: &O,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error>
    where
        O: V3OptIfaceProtocol,
    {
        match elem_id.get_name().as_str() {
            Self::OPT_IFACE_IN_MODE_NAME => {
                ElemValueAccessor::<u32>::set_vals(elem_value, 2, |idx| {
                    self.get_opt_iface_mode(unit, req, proto, false, idx > 0, timeout_ms)
                })?;
                Ok(true)
            }
            Self::OPT_IFACE_OUT_MODE_NAME => {
                ElemValueAccessor::<u32>::set_vals(elem_value, 2, |idx| {
                    self.get_opt_iface_mode(unit, req, proto, true, idx > 0, timeout_ms)
                })?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub fn write<O>(
        &mut self,
        unit: &mut SndMotu,
        req: &mut FwReq,
        proto: &O,
        elem_id: &ElemId,
        old: &ElemValue,
        new: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error>
    where
        O: V3OptIfaceProtocol,
    {
        match elem_id.get_name().as_str() {
            Self::OPT_IFACE_IN_MODE_NAME => {
                unit.lock()?;
                let res = ElemValueAccessor::<u32>::get_vals(new, old, 2, |idx, val| {
                    self.set_opt_iface_mode(unit, req, proto, false, idx > 0, val, timeout_ms)
                });
                let _ = unit.unlock();
                res.and(Ok(true))
            }
            Self::OPT_IFACE_OUT_MODE_NAME => {
                unit.lock()?;
                let res = ElemValueAccessor::<u32>::get_vals(new, old, 2, |idx, val| {
                    self.set_opt_iface_mode(unit, req, proto, true, idx > 0, val, timeout_ms)
                });
                let _ = unit.unlock();
                res.and(Ok(true))
            }
            _ => Ok(false),
        }
    }
}
