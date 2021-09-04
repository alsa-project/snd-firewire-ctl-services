// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto
use glib::Error;

use hinawa::FwReq;
use hinawa::{SndMotu, SndUnitExt};

use alsactl::{ElemId, ElemIfaceType, ElemValue};

use core::card_cntr::CardCntr;
use core::elem_value_accessor::ElemValueAccessor;

use motu_protocols::version_1::*;

use super::model::clk_rate_to_string;

fn clk_src_to_label(src: &V1ClkSrc) -> String {
    match src {
        V1ClkSrc::Internal => "Internal",
        V1ClkSrc::Spdif => "S/PDIF",
        V1ClkSrc::WordClk => "Word-on-BNC",
        V1ClkSrc::AdatOpt => "Adat-on-opt",
        V1ClkSrc::AdatDsub => "Adat-on-Dsub",
        V1ClkSrc::AesebuXlr => "AES/EBU-on-XLR",
    }
    .to_string()
}

#[derive(Default)]
pub struct V1ClkCtl;

impl V1ClkCtl {
    const RATE_NAME: &'static str = "sampling- rate";
    const SRC_NAME: &'static str = "clock-source";

    pub fn load<O>(&mut self, _: &O, card_cntr: &mut CardCntr) -> Result<(), Error>
    where
        O: V1ClkProtocol,
    {
        let labels: Vec<String> = O::CLK_RATE_LABELS
            .iter()
            .map(|l| clk_rate_to_string(&l))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::RATE_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let labels: Vec<String> = O::CLK_SRC_LABELS
            .iter()
            .map(|l| clk_src_to_label(&l))
            .collect();
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
        O: V1ClkProtocol,
    {
        match elem_id.get_name().as_str() {
            Self::RATE_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    proto.get_clk_rate(req, &mut unit.get_node(), timeout_ms).map(|idx| idx as u32)
                })?;
                Ok(true)
            }
            Self::SRC_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    proto.get_clk_src(req, &mut unit.get_node(), timeout_ms).map(|idx| idx as u32)
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
        O: V1ClkProtocol,
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
                    unit.lock()?;
                    let res = proto.set_clk_src(req, &mut unit.get_node(), val as usize, timeout_ms);
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
pub struct V1MonitorInputCtl;

impl V1MonitorInputCtl {
    const MONITOR_INPUT_NAME: &'static str = "monitor-input";

    pub fn load<O>(&mut self, _: &O, card_cntr: &mut CardCntr) -> Result<(), Error>
    where
        O: V1MonitorInputProtocol,
    {
        let labels: Vec<String> = O::MONITOR_INPUT_MODES
            .iter()
            .map(|e| e.to_string())
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::MONITOR_INPUT_NAME, 0);
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
        O: V1MonitorInputProtocol,
    {
        match elem_id.get_name().as_str() {
            Self::MONITOR_INPUT_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    proto
                        .get_monitor_input(&mut unit.get_node(), timeout_ms)
                        .map(|idx| idx as u32)
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
        O: V1MonitorInputProtocol,
    {
        match elem_id.get_name().as_str() {
            Self::MONITOR_INPUT_NAME => {
                ElemValueAccessor::<u32>::get_val(new, |val| {
                    proto.set_monitor_input(&mut unit.get_node(), val as usize, timeout_ms)
                })?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
