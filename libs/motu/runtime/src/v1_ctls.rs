// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto
use glib::Error;

use hinawa::FwReq;
use hinawa::{SndMotu, SndUnitExt};

use alsactl::{ElemId, ElemIfaceType, ElemValue};

use core::card_cntr::CardCntr;
use core::elem_value_accessor::ElemValueAccessor;

use motu_protocols::version_1::*;

use super::model::*;

fn clk_src_to_str(src: &V1ClkSrc) -> &'static str {
    match src {
        V1ClkSrc::Internal => "Internal",
        V1ClkSrc::Spdif => "S/PDIF",
        V1ClkSrc::WordClk => "Word-on-BNC",
        V1ClkSrc::AdatOpt => "Adat-on-opt",
        V1ClkSrc::AdatDsub => "Adat-on-Dsub",
        V1ClkSrc::AesebuXlr => "AES/EBU-on-XLR",
    }
}

const RATE_NAME: &str = "sampling- rate";
const SRC_NAME: &str = "clock-source";

pub trait V1ClkCtlOperation<T: V1ClkOperation> {
    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let labels: Vec<&str> = T::CLK_RATE_LABELS
            .iter()
            .map(|l| clk_rate_to_str(l))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, RATE_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let labels: Vec<&str> = T::CLK_SRC_LABELS
            .iter()
            .map(|l| clk_src_to_str(l))
            .collect();
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
                    T::get_clk_rate(req, &mut unit.get_node(), timeout_ms).map(|idx| idx as u32)
                })
                .map(|_| true)
            }
            SRC_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    T::get_clk_src(req, &mut unit.get_node(), timeout_ms).map(|idx| idx as u32)
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
                    unit.lock()?;
                    let res = T::set_clk_src(req, &mut unit.get_node(), val as usize, timeout_ms);
                    let _ = unit.unlock();
                    res
                })
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

const MONITOR_INPUT_NAME: &str = "monitor-input";

pub trait V1MonitorInputCtlOperation<T: V1MonitorInputOperation> {
    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let labels: Vec<String> = T::MONITOR_INPUT_MODES
            .iter()
            .map(|e| e.to_string())
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MONITOR_INPUT_NAME, 0);
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
            MONITOR_INPUT_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    T::get_monitor_input(req, &mut unit.get_node(), timeout_ms)
                        .map(|idx| idx as u32)
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
        new: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            MONITOR_INPUT_NAME => {
                ElemValueAccessor::<u32>::get_val(new, |val| {
                    T::set_monitor_input(req, &mut unit.get_node(), val as usize, timeout_ms)
                })
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }
}
