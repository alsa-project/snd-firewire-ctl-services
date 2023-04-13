// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use super::{common_ctls::*, register_dsp_ctls::*, register_dsp_runtime::*, v2_ctls::*};

const TIMEOUT_MS: u32 = 100;

#[derive(Default)]
pub struct TravelerModel {
    req: FwReq,
    clk_ctls: V2LcdClkCtl<TravelerProtocol>,
    opt_iface_ctl: V2OptIfaceCtl<TravelerProtocol>,
    phone_assign_ctl: RegisterDspPhoneAssignCtl<TravelerProtocol>,
    word_clk_ctl: WordClockCtl<TravelerProtocol>,
    mixer_return_ctl: RegisterDspMixerReturnCtl<TravelerProtocol>,
    params: SndMotuRegisterDspParameter,
    mixer_output_ctl: RegisterDspMixerOutputCtl<TravelerProtocol>,
    mixer_source_ctl: RegisterDspMixerMonauralSourceCtl<TravelerProtocol>,
    output_ctl: RegisterDspOutputCtl<TravelerProtocol>,
    line_input_ctl: RegisterDspLineInputCtl<TravelerProtocol>,
    mic_input_ctl: MicInputCtl,
    meter_ctl: RegisterDspMeterCtl<TravelerProtocol>,
    meter_output_target_ctl: RegisterDspMeterOutputTargetCtl<TravelerProtocol>,
}

impl CtlModel<(SndMotu, FwNode)> for TravelerModel {
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
        self.mic_input_ctl.cache(&mut self.req, node, TIMEOUT_MS)?;
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
        self.mic_input_ctl.load(card_cntr)?;
        self.meter_ctl.load(card_cntr)?;
        self.meter_output_target_ctl.load(card_cntr)?;

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
        } else if self.mic_input_ctl.read(elem_id, elem_value)? {
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
        } else if self
            .mic_input_ctl
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

impl NotifyModel<(SndMotu, FwNode), u32> for TravelerModel {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.phone_assign_ctl.0.elem_id_list);
        elem_id_list.extend_from_slice(&self.word_clk_ctl.elem_id_list);
        elem_id_list.extend_from_slice(&self.opt_iface_ctl.elem_id_list);
        elem_id_list.extend_from_slice(&self.mic_input_ctl.elem_id_list);
    }

    fn parse_notification(
        &mut self,
        (_, node): &mut (SndMotu, FwNode),
        msg: &u32,
    ) -> Result<(), Error> {
        if *msg & TravelerProtocol::NOTIFY_MIC_PARAM_MASK > 0 {
            self.mic_input_ctl.cache(&mut self.req, node, TIMEOUT_MS)?;
        }
        if *msg & TravelerProtocol::NOTIFY_PORT_CHANGE > 0 {
            self.phone_assign_ctl
                .0
                .cache(&mut self.req, node, TIMEOUT_MS)?;
            self.word_clk_ctl.cache(&mut self.req, node, TIMEOUT_MS)?;
        }
        if *msg & TravelerProtocol::NOTIFY_FORMAT_CHANGE > 0 {
            self.opt_iface_ctl.cache(&mut self.req, node, TIMEOUT_MS)?;
        }
        Ok(())
    }
}

impl NotifyModel<(SndMotu, FwNode), bool> for TravelerModel {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.phone_assign_ctl.0.elem_id_list);
        elem_id_list.extend_from_slice(&self.mixer_output_ctl.elem_id_list);
        elem_id_list.extend_from_slice(&self.mixer_source_ctl.elem_id_list);
        elem_id_list.extend_from_slice(&self.output_ctl.elem_id_list);
        elem_id_list.extend_from_slice(&self.line_input_ctl.elem_id_list);
    }

    fn parse_notification(
        &mut self,
        unit: &mut (SndMotu, FwNode),
        is_locked: &bool,
    ) -> Result<(), Error> {
        if *is_locked {
            unit.0.read_parameter(&mut self.params).map(|_| {
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

impl NotifyModel<(SndMotu, FwNode), Vec<RegisterDspEvent>> for TravelerModel {
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

impl MeasureModel<(SndMotu, FwNode)> for TravelerModel {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.meter_ctl.elem_id_list);
    }

    fn measure_states(&mut self, (unit, _): &mut (SndMotu, FwNode)) -> Result<(), Error> {
        self.meter_ctl.read_dsp_meter(unit)
    }
}

#[derive(Default, Debug)]
struct MicInputCtl {
    elem_id_list: Vec<ElemId>,
    params: TravelerMicInputState,
}

const MIC_GAIN_NAME: &str = "mic-gain-name";
const MIC_PAD_NAME: &str = "mic-pad-name";

impl MicInputCtl {
    const GAIN_TLV: DbInterval = DbInterval {
        min: -6400,
        max: 0,
        linear: true,
        mute_avail: false,
    };

    fn cache(&mut self, req: &mut FwReq, node: &mut FwNode, timeout_ms: u32) -> Result<(), Error> {
        let res = TravelerProtocol::cache_wholly(req, node, &mut self.params, timeout_ms);
        debug!(params = ?self.params, ?res);
        res
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIC_GAIN_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                TravelerProtocol::MIC_GAIN_MIN as i32,
                TravelerProtocol::MIC_GAIN_MAX as i32,
                TravelerProtocol::MIC_GAIN_STEP as i32,
                TravelerProtocol::MIC_INPUT_COUNT,
                Some(&Vec::<u32>::from(&Self::GAIN_TLV)),
                true,
            )
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIC_PAD_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, 1, TravelerProtocol::MIC_INPUT_COUNT, true)
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        Ok(())
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            MIC_GAIN_NAME => {
                let vals: Vec<i32> = self.params.gain.iter().map(|&val| val as i32).collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            MIC_PAD_NAME => {
                elem_value.set_bool(&self.params.pad);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            MIC_GAIN_NAME => {
                let mut params = self.params.clone();
                params
                    .gain
                    .iter_mut()
                    .zip(elem_value.int())
                    .for_each(|(gain, &val)| *gain = val as u8);
                let res = TravelerProtocol::update_wholly(req, node, &params, timeout_ms)
                    .map(|_| self.params = params);
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            MIC_PAD_NAME => {
                let mut params = self.params.clone();
                let vals = &elem_value.boolean()[..TravelerProtocol::MIC_INPUT_COUNT];
                params.pad.copy_from_slice(vals);
                let res = TravelerProtocol::update_wholly(req, node, &params, timeout_ms)
                    .map(|_| self.params = params);
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            _ => Ok(false),
        }
    }
}
