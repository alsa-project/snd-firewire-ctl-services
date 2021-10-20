// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto
use glib::Error;

use hinawa::FwReq;
use hinawa::{SndMotu, SndUnitExt};

use alsactl::{ElemId, ElemIfaceType, ElemValue};

use core::card_cntr::*;
use core::elem_value_accessor::ElemValueAccessor;

use motu_protocols::version_1::*;

use super::v1_ctls::*;

const TIMEOUT_MS: u32 = 100;

#[derive(Default)]
pub struct F828 {
    req: FwReq,
    clk_ctls: ClkCtl,
    monitor_input_ctl: MonitorInputCtl,
    specific_ctls: SpecificCtl,
}

#[derive(Default)]
struct ClkCtl;

impl V1ClkCtlOperation<F828Protocol> for ClkCtl {}

#[derive(Default)]
struct MonitorInputCtl;

impl V1MonitorInputCtlOperation<F828Protocol> for MonitorInputCtl {}

impl CtlModel<SndMotu> for F828 {
    fn load(&mut self, _: &mut SndMotu, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.clk_ctls.load(card_cntr)?;
        self.monitor_input_ctl.load(card_cntr)?;
        self.specific_ctls.load(card_cntr)?;
        Ok(())
    }

    fn read(
        &mut self,
        unit: &mut SndMotu,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self
            .clk_ctls
            .read(unit, &mut self.req, elem_id, elem_value, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .monitor_input_ctl
            .read(unit, &mut self.req, elem_id, elem_value, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .specific_ctls
            .read(unit, &mut self.req, elem_id, elem_value, TIMEOUT_MS)?
        {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(
        &mut self,
        unit: &mut SndMotu,
        elem_id: &ElemId,
        _: &ElemValue,
        new: &ElemValue,
    ) -> Result<bool, Error> {
        if self
            .clk_ctls
            .write(unit, &mut self.req, elem_id, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .monitor_input_ctl
            .write(unit, &mut self.req, elem_id, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .specific_ctls
            .write(unit, &mut self.req, elem_id, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl NotifyModel<SndMotu, u32> for F828 {
    fn get_notified_elem_list(&mut self, _: &mut Vec<ElemId>) {
    }

    fn parse_notification(&mut self, _: &mut SndMotu, _: &u32) -> Result<(), Error> {
        Ok(())
    }

    fn read_notified_elem(
        &mut self,
        _: &SndMotu,
        _: &ElemId,
        _: &mut ElemValue
    ) -> Result<bool, Error> {
        Ok(false)
    }
}

fn opt_iface_mode_to_str(mode: &V1OptIfaceMode) -> &'static str {
    match mode {
        V1OptIfaceMode::Adat => "ADAT",
        V1OptIfaceMode::Spdif => "SPDIF",
    }
}

#[derive(Default)]
struct SpecificCtl;

const OPT_IN_IFACE_MODE_NAME: &str = "optical-iface-in-mode";
const OPT_OUT_IFACE_MODE_NAME: &str = "optical-iface-out-mode";

impl SpecificCtl {
    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let labels: Vec<&str> = F828Protocol::OPT_IFACE_MODES
            .iter()
            .map(|l| opt_iface_mode_to_str(&l))
            .collect();

        let elem_id =
            ElemId::new_by_name(ElemIfaceType::Card, 0, 0, OPT_IN_IFACE_MODE_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let elem_id =
            ElemId::new_by_name(ElemIfaceType::Card, 0, 0, OPT_OUT_IFACE_MODE_NAME, 0);
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
            OPT_IN_IFACE_MODE_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    F828Protocol::get_optical_input_iface_mode(
                        req,
                        &mut unit.get_node(),
                        timeout_ms
                    )
                        .map(|val| val as u32)
                })
                .map(|_| true)
            }
            OPT_OUT_IFACE_MODE_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    F828Protocol::get_optical_output_iface_mode(
                        req,
                        &mut unit.get_node(),
                        timeout_ms
                    )
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
            OPT_IN_IFACE_MODE_NAME => {
                ElemValueAccessor::<u32>::get_val(elem_value, |val| {
                    unit.lock()?;
                    let res = F828Protocol::set_optical_input_iface_mode(
                        req,
                        &mut unit.get_node(),
                        val as usize,
                        timeout_ms
                    );
                    unit.unlock()?;
                    res
                })
                .map(|_| true)
            }
            OPT_OUT_IFACE_MODE_NAME => {
                ElemValueAccessor::<u32>::get_val(elem_value, |val| {
                    unit.lock()?;
                    let res = F828Protocol::set_optical_output_iface_mode(
                        req,
                        &mut unit.get_node(),
                        val as usize,
                        timeout_ms
                    );
                    unit.unlock()?;
                    res
                })
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }
}
