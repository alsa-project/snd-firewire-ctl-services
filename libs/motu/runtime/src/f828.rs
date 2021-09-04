// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto
use glib::Error;

use hinawa::FwReq;
use hinawa::{SndMotu, SndUnitExt};

use alsactl::{ElemId, ElemIfaceType, ElemValue};

use core::card_cntr::{CardCntr, CtlModel};
use core::elem_value_accessor::ElemValueAccessor;

use motu_protocols::version_1::*;

use super::v1_ctls::*;

const TIMEOUT_MS: u32 = 100;

#[derive(Default)]
pub struct F828 {
    req: FwReq,
    proto: F828Protocol,
    clk_ctls: V1ClkCtl,
    monitor_input_ctl: V1MonitorInputCtl,
    specific_ctls: SpecificCtl,
}

impl CtlModel<SndMotu> for F828 {
    fn load(&mut self, _: &mut SndMotu, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.clk_ctls.load(&self.proto, card_cntr)?;
        self.monitor_input_ctl.load(&self.proto, card_cntr)?;
        self.specific_ctls.load(&self.proto, card_cntr)?;
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
            .read(unit, &mut self.req, &self.proto, elem_id, elem_value, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .monitor_input_ctl
            .read(unit, &self.proto, elem_id, elem_value, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .specific_ctls
            .read(unit, &self.proto, elem_id, elem_value, TIMEOUT_MS)?
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
        old: &ElemValue,
        new: &ElemValue,
    ) -> Result<bool, Error> {
        if self
            .clk_ctls
            .write(unit, &mut self.req, &self.proto, elem_id, old, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .monitor_input_ctl
            .write(unit, &self.proto, elem_id, old, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .specific_ctls
            .write(unit, &self.proto, elem_id, old, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

fn opt_iface_mode_to_label(mode: &V1OptIfaceMode) -> String {
    match mode {
        V1OptIfaceMode::Adat => "ADAT",
        V1OptIfaceMode::Spdif => "SPDIF",
    }
    .to_string()
}

#[derive(Default)]
pub struct SpecificCtl;

impl SpecificCtl {
    const OPT_IN_IFACE_MODE_NAME: &'static str = "optical-iface-in-mode";
    const OPT_OUT_IFACE_MODE_NAME: &'static str = "optical-iface-out-mode";

    pub fn load(&mut self, _: &F828Protocol, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let labels: Vec<String> = F828Protocol::OPT_IFACE_MODES
            .iter()
            .map(|l| opt_iface_mode_to_label(&l))
            .collect();

        let elem_id =
            ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::OPT_IN_IFACE_MODE_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let elem_id =
            ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::OPT_OUT_IFACE_MODE_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        Ok(())
    }

    pub fn read(
        &mut self,
        unit: &SndMotu,
        proto: &F828Protocol,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            Self::OPT_IN_IFACE_MODE_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    proto
                        .get_optical_input_iface_mode(&mut unit.get_node(), timeout_ms)
                        .map(|val| val as u32)
                })?;
                Ok(true)
            }
            Self::OPT_OUT_IFACE_MODE_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    proto
                        .get_optical_output_iface_mode(&mut unit.get_node(), timeout_ms)
                        .map(|val| val as u32)
                })?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub fn write(
        &mut self,
        unit: &SndMotu,
        proto: &F828Protocol,
        elem_id: &ElemId,
        _: &ElemValue,
        new: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            Self::OPT_IN_IFACE_MODE_NAME => {
                ElemValueAccessor::<u32>::get_val(new, |val| {
                    unit.lock()?;
                    let res = proto.set_optical_input_iface_mode(&mut unit.get_node(), val as usize, timeout_ms);
                    unit.unlock()?;
                    res
                })?;
                Ok(true)
            }
            Self::OPT_OUT_IFACE_MODE_NAME => {
                ElemValueAccessor::<u32>::get_val(new, |val| {
                    unit.lock()?;
                    let res = proto.set_optical_output_iface_mode(&mut unit.get_node(), val as usize, timeout_ms);
                    unit.unlock()?;
                    res
                })?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
