// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use super::{register_dsp_ctls::*, register_dsp_runtime::*, v2_ctls::*};

const TIMEOUT_MS: u32 = 100;

#[derive(Default)]
pub struct F8pre {
    req: FwReq,
    clk_ctls: V2ClkCtl<F8preProtocol>,
    opt_iface_ctl: V2OptIfaceCtl<F8preProtocol>,
    phone_assign_ctl: RegisterDspPhoneAssignCtl<F8preProtocol>,
    mixer_return_ctl: RegisterDspMixerReturnCtl<F8preProtocol>,
    params: SndMotuRegisterDspParameter,
    mixer_output_ctl: RegisterDspMixerOutputCtl<F8preProtocol>,
    mixer_source_ctl: RegisterDspMixerMonauralSourceCtl<F8preProtocol>,
    output_ctl: RegisterDspOutputCtl<F8preProtocol>,
    meter_ctl: RegisterDspMeterCtl<F8preProtocol>,
}

impl CtlModel<(SndMotu, FwNode)> for F8pre {
    fn cache(&mut self, (unit, node): &mut (SndMotu, FwNode)) -> Result<(), Error> {
        unit.read_parameter(&mut self.params)?;
        self.phone_assign_ctl.parse_dsp_parameter(&self.params);
        self.mixer_output_ctl.parse_dsp_parameter(&self.params);
        self.mixer_source_ctl.parse_dsp_parameter(&self.params);
        self.output_ctl.parse_dsp_parameter(&self.params);

        self.meter_ctl.read_dsp_meter(unit)?;

        self.clk_ctls.cache(&mut self.req, node, TIMEOUT_MS)?;
        self.opt_iface_ctl.cache(&mut self.req, node, TIMEOUT_MS)?;
        self.phone_assign_ctl
            .0
            .cache(&mut self.req, node, TIMEOUT_MS)?;
        self.mixer_return_ctl
            .cache(&mut self.req, node, TIMEOUT_MS)?;
        self.mixer_source_ctl
            .cache(&mut self.req, node, TIMEOUT_MS)?;
        self.output_ctl.cache(&mut self.req, node, TIMEOUT_MS)?;

        Ok(())
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.clk_ctls.load(card_cntr)?;
        self.opt_iface_ctl.load(card_cntr)?;
        self.phone_assign_ctl.0.load(card_cntr)?;
        self.mixer_return_ctl.load(card_cntr)?;
        self.mixer_output_ctl.load(card_cntr)?;
        self.mixer_source_ctl.load(card_cntr)?;
        self.output_ctl.load(card_cntr)?;
        self.meter_ctl.load(card_cntr)?;

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
        } else if self.opt_iface_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.phone_assign_ctl.0.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_return_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_output_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_source_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.output_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.meter_ctl.read(elem_id, elem_value)? {
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
        } else if self.opt_iface_ctl.write(
            unit,
            &mut self.req,
            node,
            elem_id,
            elem_value,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self.phone_assign_ctl.0.write(
            &mut self.req,
            node,
            elem_id,
            elem_value,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self.mixer_return_ctl.write(
            &mut self.req,
            node,
            elem_id,
            elem_value,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self.mixer_output_ctl.write(
            &mut self.req,
            node,
            elem_id,
            elem_value,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self.mixer_source_ctl.write(
            &mut self.req,
            node,
            elem_id,
            elem_value,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self
            .output_ctl
            .write(&mut self.req, node, elem_id, elem_value, TIMEOUT_MS)?
        {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl NotifyModel<(SndMotu, FwNode), u32> for F8pre {
    fn get_notified_elem_list(&mut self, _: &mut Vec<ElemId>) {}

    fn parse_notification(&mut self, _: &mut (SndMotu, FwNode), _: &u32) -> Result<(), Error> {
        Ok(())
    }
}

impl NotifyModel<(SndMotu, FwNode), bool> for F8pre {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.mixer_output_ctl.elem_id_list);
        elem_id_list.extend_from_slice(&self.mixer_source_ctl.elem_id_list);
        elem_id_list.extend_from_slice(&self.output_ctl.elem_id_list);
    }

    fn parse_notification(
        &mut self,
        unit: &mut (SndMotu, FwNode),
        is_locked: &bool,
    ) -> Result<(), Error> {
        if *is_locked {
            unit.0.read_parameter(&mut self.params).map(|_| {
                self.mixer_output_ctl.parse_dsp_parameter(&self.params);
                self.mixer_source_ctl.parse_dsp_parameter(&self.params);
                self.output_ctl.parse_dsp_parameter(&self.params);
            })
        } else {
            Ok(())
        }
    }
}

impl NotifyModel<(SndMotu, FwNode), Vec<RegisterDspEvent>> for F8pre {
    fn get_notified_elem_list(&mut self, _: &mut Vec<ElemId>) {
        // MEMO: handled by the above implementation.
    }

    fn parse_notification(
        &mut self,
        _: &mut (SndMotu, FwNode),
        events: &Vec<RegisterDspEvent>,
    ) -> Result<(), Error> {
        events.iter().for_each(|event| {
            let _ = self.phone_assign_ctl.parse_dsp_event(event)
                || self.mixer_output_ctl.parse_dsp_event(event)
                || self.mixer_source_ctl.parse_dsp_event(event)
                || self.output_ctl.parse_dsp_event(event);
        });
        Ok(())
    }
}

impl MeasureModel<(SndMotu, FwNode)> for F8pre {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.meter_ctl.elem_id_list);
    }

    fn measure_states(&mut self, (unit, _): &mut (SndMotu, FwNode)) -> Result<(), Error> {
        self.meter_ctl.read_dsp_meter(unit)
    }

    fn measure_elem(
        &mut self,
        _: &(SndMotu, FwNode),
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.meter_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
