// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use super::{common_ctls::*, register_dsp_ctls::*, register_dsp_runtime::*, v2_ctls::*};

const TIMEOUT_MS: u32 = 100;

#[derive(Default)]
pub struct F828mk2Model {
    req: FwReq,
    clk_ctls: V2LcdClkCtl<F828mk2Protocol>,
    opt_iface_ctl: V2OptIfaceCtl<F828mk2Protocol>,
    phone_assign_ctl: RegisterDspPhoneAssignCtl<F828mk2Protocol>,
    word_clk_ctl: WordClockCtl<F828mk2Protocol>,
    mixer_return_ctl: RegisterDspMixerReturnCtl<F828mk2Protocol>,
    params: SndMotuRegisterDspParameter,
    mixer_output_ctl: RegisterDspMixerOutputCtl<F828mk2Protocol>,
    mixer_source_ctl: RegisterDspMixerMonauralSourceCtl<F828mk2Protocol>,
    output_ctl: RegisterDspOutputCtl<F828mk2Protocol>,
    line_input_ctl: RegisterDspLineInputCtl<F828mk2Protocol>,
    meter_ctl: RegisterDspMeterCtl<F828mk2Protocol>,
    meter_output_target_ctl: RegisterDspMeterOutputTargetCtl<F828mk2Protocol>,
}

impl CtlModel<(SndMotu, FwNode)> for F828mk2Model {
    fn cache(&mut self, (unit, node): &mut (SndMotu, FwNode)) -> Result<(), Error> {
        unit.read_parameter(&mut self.params)?;
        self.phone_assign_ctl.parse_dsp_parameter(&self.params);
        self.mixer_output_ctl.parse_dsp_parameter(&self.params);
        self.mixer_source_ctl.parse_dsp_parameter(&self.params);
        self.output_ctl.parse_dsp_parameter(&self.params);
        self.line_input_ctl.parse_dsp_parameter(&self.params);

        self.meter_ctl.read_dsp_meter(unit)?;

        self.clk_ctls.cache(&mut self.req, node, TIMEOUT_MS)?;
        self.word_clk_ctl.cache(&mut self.req, node, TIMEOUT_MS)?;
        self.opt_iface_ctl.cache(&mut self.req, node, TIMEOUT_MS)?;
        self.phone_assign_ctl
            .0
            .cache(&mut self.req, node, TIMEOUT_MS)?;
        self.mixer_return_ctl
            .cache(&mut self.req, node, TIMEOUT_MS)?;
        self.mixer_output_ctl
            .cache(&mut self.req, node, TIMEOUT_MS)?;
        self.mixer_source_ctl
            .cache(&mut self.req, node, TIMEOUT_MS)?;
        self.output_ctl.cache(&mut self.req, node, TIMEOUT_MS)?;
        self.line_input_ctl.cache(&mut self.req, node, TIMEOUT_MS)?;
        self.meter_output_target_ctl
            .cache(&mut self.req, node, TIMEOUT_MS)?;

        Ok(())
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.clk_ctls.load(card_cntr)?;
        self.opt_iface_ctl.load(card_cntr)?;
        self.phone_assign_ctl.0.load(card_cntr)?;
        self.word_clk_ctl.load(card_cntr)?;
        self.mixer_return_ctl.load(card_cntr)?;
        self.mixer_output_ctl.load(card_cntr)?;
        self.mixer_source_ctl.load(card_cntr)?;
        self.output_ctl.load(card_cntr)?;
        self.line_input_ctl.load(card_cntr)?;
        self.meter_ctl.load(card_cntr)?;
        self.meter_output_target_ctl.load(card_cntr)?;

        Ok(())
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        if self.clk_ctls.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.opt_iface_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.phone_assign_ctl.0.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.word_clk_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_return_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_output_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_source_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.output_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.line_input_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.meter_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.meter_output_target_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(
        &mut self,
        (unit, node): &mut (SndMotu, FwNode),
        elem_id: &ElemId,
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
        } else if self
            .word_clk_ctl
            .write(&mut self.req, node, elem_id, elem_value, TIMEOUT_MS)?
        {
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
        } else if self
            .line_input_ctl
            .write(&mut self.req, node, elem_id, elem_value, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self.meter_output_target_ctl.write(
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

impl NotifyModel<(SndMotu, FwNode), u32> for F828mk2Model {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.word_clk_ctl.elem_id_list);
    }

    fn parse_notification(
        &mut self,
        (_, node): &mut (SndMotu, FwNode),
        msg: &u32,
    ) -> Result<(), Error> {
        if *msg & F828mk2Protocol::NOTIFY_PORT_CHANGE > 0 {
            self.word_clk_ctl.cache(&mut self.req, node, TIMEOUT_MS)?;
        }
        // TODO: what kind of event is preferable for NOTIFY_FOOTSWITCH_MASK?
        Ok(())
    }
}

impl NotifyModel<(SndMotu, FwNode), bool> for F828mk2Model {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.phone_assign_ctl.0.elem_id_list);
        elem_id_list.extend_from_slice(&self.mixer_output_ctl.elem_id_list);
        elem_id_list.extend_from_slice(&self.mixer_source_ctl.elem_id_list);
        elem_id_list.extend_from_slice(&self.output_ctl.elem_id_list);
        elem_id_list.extend_from_slice(&self.line_input_ctl.elem_id_list);
    }

    fn parse_notification(
        &mut self,
        (unit, _): &mut (SndMotu, FwNode),
        is_locked: &bool,
    ) -> Result<(), Error> {
        if *is_locked {
            unit.read_parameter(&mut self.params).map(|_| {
                self.phone_assign_ctl.parse_dsp_parameter(&self.params);
                self.mixer_output_ctl.parse_dsp_parameter(&self.params);
                self.mixer_source_ctl.parse_dsp_parameter(&self.params);
                self.output_ctl.parse_dsp_parameter(&self.params);
                self.line_input_ctl.parse_dsp_parameter(&self.params);
            })
        } else {
            Ok(())
        }
    }
}

impl NotifyModel<(SndMotu, FwNode), Vec<RegisterDspEvent>> for F828mk2Model {
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
                || self.output_ctl.parse_dsp_event(event)
                || self.line_input_ctl.parse_dsp_event(event);
        });
        Ok(())
    }
}

impl MeasureModel<(SndMotu, FwNode)> for F828mk2Model {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.meter_ctl.elem_id_list);
    }

    fn measure_states(&mut self, (unit, _): &mut (SndMotu, FwNode)) -> Result<(), Error> {
        self.meter_ctl.read_dsp_meter(unit)
    }
}
