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

#[derive(Default)]
pub struct V3ClkCtl;

impl V3ClkCtl {
    const RATE_NAME: &'static str = "sampling-rate";
    const SRC_NAME: &'static str = "clock-source";

    pub fn load<O>(&mut self, _: &O, card_cntr: &mut CardCntr) -> Result<(), Error>
    where
        O: V3ClkProtocol,
    {
        let labels: Vec<String> = O::CLK_RATES
            .iter()
            .map(|e| clk_rate_to_string(&e.0))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::RATE_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let labels: Vec<String> = O::CLK_SRCS.iter().map(|e| clk_src_to_label(&e.0)).collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::SRC_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        Ok(())
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
        O: V3ClkProtocol,
    {
        match elem_id.get_name().as_str() {
            Self::RATE_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    proto.get_clk_rate(req, &mut unit.get_node(), timeout_ms).map(|val| val as u32)
                })?;
                Ok(true)
            }
            Self::SRC_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let mut node = unit.get_node();
                    let val = proto.get_clk_src(req, &mut node, timeout_ms)?;
                    if O::HAS_LCD {
                        let label = clk_src_to_label(&O::CLK_SRCS[val].0);
                        let _ = proto.update_clk_display(req, &mut node, &label, timeout_ms);
                    }
                    Ok(val as u32)
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
        _: &ElemValue,
        new: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error>
    where
        O: V3ClkProtocol,
    {
        match elem_id.get_name().as_str() {
            Self::RATE_NAME => {
                ElemValueAccessor::<u32>::get_val(new, |val| {
                    unit.lock()?;
                    let res = proto.set_clk_rate(req, &mut unit.get_node(), val as usize, timeout_ms);
                    let _ = unit.unlock();
                    res
                })?;
                Ok(true)
            }
            Self::SRC_NAME => {
                ElemValueAccessor::<u32>::get_val(new, |val| {
                    let prev_src = proto.get_clk_src(req, &mut unit.get_node(), timeout_ms)?;
                    unit.lock()?;
                    let mut node = unit.get_node();
                    let mut res = proto.set_clk_src(req, &mut node, val as usize, timeout_ms);
                    if res.is_ok() && O::HAS_LCD {
                        let label = clk_src_to_label(&O::CLK_SRCS[val as usize].0);
                        res = proto.update_clk_display(req, &mut node, &label, timeout_ms);
                        if res.is_err() {
                            let _ = proto.set_clk_src(req, &mut node, prev_src, timeout_ms);
                        }
                    }
                    let _ = unit.unlock();
                    res
                })?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

#[derive(Default)]
pub struct V3PortAssignCtl(pub Vec<ElemId>);

impl V3PortAssignCtl {
    const MAIN_ASSIGN_NAME: &'static str = "main-assign";
    const RETURN_ASSIGN_NAME: &'static str = "return-assign";

    pub fn load<O>(&mut self, _: &O, card_cntr: &mut CardCntr) -> Result<(), Error>
    where
        O: V3PortAssignProtocol,
    {
        let labels: Vec<String> = O::ASSIGN_PORTS.iter().map(|e| e.0.to_string()).collect();

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::MAIN_ASSIGN_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, true)
            .map(|elem_id_list| self.0.extend_from_slice(&elem_id_list))?;

        let elem_id =
            alsactl::ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::RETURN_ASSIGN_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, true)
            .map(|elem_id_list| self.0.extend_from_slice(&elem_id_list))?;

        Ok(())
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
        O: V3PortAssignProtocol,
    {
        match elem_id.get_name().as_str() {
            Self::MAIN_ASSIGN_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    proto
                        .get_main_assign(req, &mut unit.get_node(), timeout_ms)
                        .map(|val| val as u32)
                })?;
                Ok(true)
            }
            Self::RETURN_ASSIGN_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    proto
                        .get_return_assign(req, &mut unit.get_node(), timeout_ms)
                        .map(|val| val as u32)
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
        _: &ElemValue,
        new: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error>
    where
        O: V3PortAssignProtocol,
    {
        match elem_id.get_name().as_str() {
            Self::MAIN_ASSIGN_NAME => {
                ElemValueAccessor::<u32>::get_val(new, |val| {
                    proto.set_main_assign(req, &mut unit.get_node(), val as usize, timeout_ms)
                })?;
                Ok(true)
            }
            Self::RETURN_ASSIGN_NAME => {
                ElemValueAccessor::<u32>::get_val(new, |val| {
                    proto.set_return_assign(req, &mut unit.get_node(), val as usize, timeout_ms)
                })?;
                Ok(true)
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
        proto: &O,
        is_out: bool,
        is_b: bool,
        timeout_ms: u32,
    ) -> Result<u32, Error>
    where
        O: V3OptIfaceProtocol,
    {
        proto
            .get_opt_iface_mode(req, &mut unit.get_node(), is_out, is_b, timeout_ms)
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
        proto: &O,
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
        proto.set_opt_iface_mode(req, &mut unit.get_node(), is_out, is_b, enabled, no_adat, timeout_ms)
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
