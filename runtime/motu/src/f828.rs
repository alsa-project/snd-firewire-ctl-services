// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

use super::{v1_ctls::*, v1_runtime::*};

const TIMEOUT_MS: u32 = 100;

#[derive(Default)]
pub struct F828 {
    req: FwReq,
    clk_ctls: V1ClkCtl<F828Protocol>,
    monitor_input_ctl: V1MonitorInputCtl<F828Protocol>,
    specific_ctls: SpecificCtl,
}

impl CtlModel<(SndMotu, FwNode)> for F828 {
    fn cache(&mut self, (_, node): &mut (SndMotu, FwNode)) -> Result<(), Error> {
        self.clk_ctls.cache(&mut self.req, node, TIMEOUT_MS)?;
        self.monitor_input_ctl
            .cache(&mut self.req, node, TIMEOUT_MS)?;
        self.specific_ctls.cache(&mut self.req, node, TIMEOUT_MS)?;
        Ok(())
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.clk_ctls.load(card_cntr)?;
        self.monitor_input_ctl.load(card_cntr)?;
        self.specific_ctls.load(card_cntr)?;
        Ok(())
    }

    fn read(
        &mut self,
        _: &mut (SndMotu, FwNode),
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.clk_ctls.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.monitor_input_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.specific_ctls.read(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(
        &mut self,
        (unit, node): &mut (SndMotu, FwNode),
        elem_id: &ElemId,
        _: &ElemValue,
        elem_value: &ElemValue,
    ) -> Result<bool, Error> {
        if self
            .clk_ctls
            .write(unit, &mut self.req, node, elem_id, elem_value, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self.monitor_input_ctl.write(
            &mut self.req,
            node,
            elem_id,
            elem_value,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self.specific_ctls.write(
            unit,
            &mut self.req,
            node,
            elem_id,
            elem_value,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl NotifyModel<(SndMotu, FwNode), u32> for F828 {
    fn get_notified_elem_list(&mut self, _: &mut Vec<ElemId>) {}

    fn parse_notification(&mut self, _: &mut (SndMotu, FwNode), _: &u32) -> Result<(), Error> {
        Ok(())
    }
}

fn opt_iface_mode_to_str(mode: &V1OptIfaceMode) -> &'static str {
    match mode {
        V1OptIfaceMode::Adat => "ADAT",
        V1OptIfaceMode::Spdif => "SPDIF",
    }
}

#[derive(Default, Debug)]
struct SpecificCtl {
    elem_id_list: Vec<ElemId>,
    params: F828OpticalIfaceParameters,
}

const OPT_IN_IFACE_MODE_NAME: &str = "optical-iface-in-mode";
const OPT_OUT_IFACE_MODE_NAME: &str = "optical-iface-out-mode";

impl SpecificCtl {
    fn cache(&mut self, req: &mut FwReq, node: &mut FwNode, timeout_ms: u32) -> Result<(), Error> {
        let res = F828Protocol::cache_wholly(req, node, &mut self.params, timeout_ms);
        debug!(params = ?self.params, ?res);
        res
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let labels: Vec<&str> = F828Protocol::OPT_IFACE_MODES
            .iter()
            .map(|l| opt_iface_mode_to_str(&l))
            .collect();

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, OPT_IN_IFACE_MODE_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, true)
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, OPT_OUT_IFACE_MODE_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, true)
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        Ok(())
    }

    fn read(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            OPT_IN_IFACE_MODE_NAME => {
                let pos = F828Protocol::OPT_IFACE_MODES
                    .iter()
                    .position(|m| self.params.input_mode.eq(m))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            OPT_OUT_IFACE_MODE_NAME => {
                let pos = F828Protocol::OPT_IFACE_MODES
                    .iter()
                    .position(|m| self.params.output_mode.eq(m))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write(
        &mut self,
        unit: &mut SndMotu,
        req: &mut FwReq,
        node: &mut FwNode,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            OPT_IN_IFACE_MODE_NAME => {
                let mut params = self.params.clone();
                let pos = elem_value.enumerated()[0] as usize;
                F828Protocol::OPT_IFACE_MODES
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| {
                        let msg = format!("Invalid argument for optical input interface: {}", pos);
                        Error::new(FileError::Inval, &msg)
                    })
                    .map(|&mode| params.input_mode = mode)?;
                unit.lock()?;
                let res = F828Protocol::update_wholly(req, node, &params, timeout_ms)
                    .map(|_| self.params = params);
                unit.unlock()?;
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            OPT_OUT_IFACE_MODE_NAME => {
                let mut params = self.params.clone();
                let pos = elem_value.enumerated()[0] as usize;
                F828Protocol::OPT_IFACE_MODES
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| {
                        let msg = format!("Invalid argument for optical input interface: {}", pos);
                        Error::new(FileError::Inval, &msg)
                    })
                    .map(|&mode| params.output_mode = mode)?;
                unit.lock()?;
                let res = F828Protocol::update_wholly(req, node, &params, timeout_ms)
                    .map(|_| self.params = params);
                unit.unlock()?;
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            _ => Ok(false),
        }
    }
}
