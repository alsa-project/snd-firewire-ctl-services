// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto
use glib::Error;

use hinawa::{SndMotu, SndUnitExt};

use alsactl::{ElemId, ElemIfaceType, ElemValue};

use core::card_cntr::CardCntr;
use core::elem_value_accessor::ElemValueAccessor;

use motu_protocols::version_2::*;

use super::model::clk_rate_to_string;

fn clk_src_to_label(src: &V2ClkSrc) -> String {
    match src {
        V2ClkSrc::Internal => "Internal",
        V2ClkSrc::SpdifCoax => "S/PDIF-on-coax",
        V2ClkSrc::WordClk => "Word-on-BNC",
        V2ClkSrc::SignalOpt => "Signal-on-opt",
        V2ClkSrc::AdatOpt => "Adat-on-opt",
        V2ClkSrc::AdatDsub => "Adat-on-Dsub",
        V2ClkSrc::AesebuXlr => "AES/EBU-on-XLR",
    }
    .to_string()
}

#[derive(Default)]
pub struct V2ClkCtl;

impl V2ClkCtl {
    const RATE_NAME: &'static str = "sampling- rate";
    const SRC_NAME: &'static str = "clock-source";

    pub fn load<O>(&mut self, _: &O, card_cntr: &mut CardCntr) -> Result<(), Error>
    where
        O: V2ClkProtocol,
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
        proto: &O,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error>
    where
        O: V2ClkProtocol,
    {
        match elem_id.get_name().as_str() {
            Self::RATE_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    proto.get_clk_rate(&mut unit.get_node(), timeout_ms).map(|idx| idx as u32)
                })?;
                Ok(true)
            }
            Self::SRC_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let mut node = unit.get_node();
                    proto.get_clk_src(&mut node, timeout_ms).and_then(|idx| {
                        if O::HAS_LCD {
                            let label = clk_src_to_label(&O::CLK_SRCS[idx].0);
                            proto.update_clk_display(&mut node, &label, timeout_ms)?;
                        }
                        Ok(idx as u32)
                    })
                })?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub fn write<O>(
        &mut self,
        unit: &mut SndMotu,
        proto: &O,
        elem_id: &ElemId,
        _: &ElemValue,
        new: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error>
    where
        O: V2ClkProtocol,
    {
        match elem_id.get_name().as_str() {
            Self::RATE_NAME => {
                ElemValueAccessor::<u32>::get_val(new, |val| {
                    unit.lock()?;
                    let res = proto.set_clk_rate(&mut unit.get_node(), val as usize, timeout_ms);
                    let _ = unit.unlock();
                    res
                })?;
                Ok(true)
            }
            Self::SRC_NAME => {
                ElemValueAccessor::<u32>::get_val(new, |val| {
                    let mut node = unit.get_node();
                    let prev_src = proto.get_clk_src(&mut node, timeout_ms)?;
                    unit.lock()?;
                    let mut res = proto.set_clk_src(&mut node, val as usize, timeout_ms);
                    if res.is_ok() && O::HAS_LCD {
                        let label = clk_src_to_label(&O::CLK_SRCS[val as usize].0);
                        res = proto.update_clk_display(&mut node, &label, timeout_ms);
                        if res.is_err() {
                            let _ = proto.set_clk_src(&mut node, prev_src, timeout_ms);
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
pub struct V2MainAssignCtl(pub Vec<ElemId>);

impl V2MainAssignCtl {
    const MAIN_VOL_TARGET_NAME: &'static str = "main-volume-target";

    pub fn load<O>(&mut self, _: &O, card_cntr: &mut CardCntr) -> Result<(), Error>
    where
        O: V2MainAssignProtocol,
    {
        let labels: Vec<String> = O::KNOB_TARGETS.iter().map(|e| e.0.to_string()).collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::MAIN_VOL_TARGET_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, true)
            .map(|elem_id_list| self.0.extend_from_slice(&elem_id_list))
    }

    pub fn read<O>(
        &mut self,
        unit: &mut SndMotu,
        proto: &O,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error>
    where
        O: V2MainAssignProtocol,
    {
        match elem_id.get_name().as_str() {
            Self::MAIN_VOL_TARGET_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    proto
                        .get_main_vol_assign(&mut unit.get_node(), timeout_ms)
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
        proto: &O,
        elem_id: &ElemId,
        _: &ElemValue,
        new: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error>
    where
        O: V2MainAssignProtocol,
    {
        match elem_id.get_name().as_str() {
            Self::MAIN_VOL_TARGET_NAME => {
                ElemValueAccessor::<u32>::get_val(new, |val| {
                    proto.set_main_vol_assign(&mut unit.get_node(), val as usize, timeout_ms)
                })?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

fn opt_iface_mode_to_label(mode: &V2OptIfaceMode) -> String {
    match mode {
        V2OptIfaceMode::None => "None",
        V2OptIfaceMode::Adat => "ADAT",
        V2OptIfaceMode::Spdif => "S/PDIF",
    }
    .to_string()
}

#[derive(Default)]
pub struct V2OptIfaceCtl;

impl V2OptIfaceCtl {
    const OPT_IN_IFACE_MODE_NAME: &'static str = "optical-iface-in-mode";
    const OPT_OUT_IFACE_MODE_NAME: &'static str = "optical-iface-out-mode";

    pub fn load<O>(&mut self, _: &O, card_cntr: &mut CardCntr) -> Result<(), Error>
    where
        O: V2OptIfaceProtocol,
    {
        let labels: Vec<String> = O::OPT_IFACE_MODES
            .iter()
            .map(|e| opt_iface_mode_to_label(&e.0))
            .collect();

        let elem_id =
            ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::OPT_IN_IFACE_MODE_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let elem_id =
            ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::OPT_OUT_IFACE_MODE_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        Ok(())
    }

    pub fn read<O>(
        &mut self,
        unit: &mut SndMotu,
        proto: &O,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error>
    where
        O: V2OptIfaceProtocol,
    {
        match elem_id.get_name().as_str() {
            Self::OPT_IN_IFACE_MODE_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    proto
                        .get_opt_in_iface_mode(&mut unit.get_node(), timeout_ms)
                        .map(|val| val as u32)
                })?;
                Ok(true)
            }
            Self::OPT_OUT_IFACE_MODE_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    proto
                        .get_opt_out_iface_mode(&mut unit.get_node(), timeout_ms)
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
        proto: &O,
        elem_id: &ElemId,
        _: &ElemValue,
        new: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error>
    where
        O: V2OptIfaceProtocol,
    {
        match elem_id.get_name().as_str() {
            Self::OPT_IN_IFACE_MODE_NAME => {
                ElemValueAccessor::<u32>::get_val(new, |val| {
                    unit.lock()?;
                    let res = proto.set_opt_in_iface_mode(&mut unit.get_node(), val as usize, timeout_ms);
                    unit.unlock()?;
                    res
                })?;
                Ok(true)
            }
            Self::OPT_OUT_IFACE_MODE_NAME => {
                ElemValueAccessor::<u32>::get_val(new, |val| {
                    unit.lock()?;
                    let res = proto.set_opt_out_iface_mode(&mut unit.get_node(), val as usize, timeout_ms);
                    unit.unlock()?;
                    res
                })?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
