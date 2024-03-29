// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use {
    super::{shell_ctl::*, *},
    protocols::tcelectronic::shell::k8::*,
};

#[derive(Default, Debug)]
pub struct K8Model {
    req: FwReq,
    sections: GeneralSections,
    common_ctl: CommonCtl<K8Protocol>,
    knob_ctl: KnobCtl,
    config_ctl: ConfigCtl,
    mixer_state_ctl: MixerStateCtl,
    mixer_meter_ctl: MixerMeterCtl<K8Protocol, K8MixerMeter>,
    hw_state_ctl: HwStateCtl,
}

const TIMEOUT_MS: u32 = 20;

impl CtlModel<(SndDice, FwNode)> for K8Model {
    fn cache(&mut self, (_, node): &mut (SndDice, FwNode)) -> Result<(), Error> {
        K8Protocol::read_general_sections(&mut self.req, node, &mut self.sections, TIMEOUT_MS)?;

        self.common_ctl
            .cache_whole_params(&mut self.req, node, &mut self.sections, TIMEOUT_MS)?;

        self.knob_ctl.cache(&mut self.req, node, TIMEOUT_MS)?;
        self.config_ctl.cache(&mut self.req, node, TIMEOUT_MS)?;
        self.mixer_state_ctl
            .cache(&mut self.req, node, TIMEOUT_MS)?;
        self.mixer_meter_ctl
            .cache(&mut self.req, node, TIMEOUT_MS)?;
        self.hw_state_ctl.cache(&mut self.req, node, TIMEOUT_MS)?;

        Ok(())
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.common_ctl.load(card_cntr)?;

        self.knob_ctl.load(card_cntr)?;
        self.config_ctl.load(card_cntr)?;
        self.mixer_state_ctl.load(card_cntr)?;
        self.mixer_meter_ctl.load(card_cntr)?;
        self.hw_state_ctl.load(card_cntr)?;

        Ok(())
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        if self.common_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.knob_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.config_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_state_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_meter_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.hw_state_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(
        &mut self,
        (unit, node): &mut (SndDice, FwNode),
        elem_id: &ElemId,
        elem_value: &ElemValue,
    ) -> Result<bool, Error> {
        if self.common_ctl.write(
            unit,
            &mut self.req,
            node,
            &mut self.sections,
            elem_id,
            elem_value,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self
            .knob_ctl
            .write(&mut self.req, node, elem_id, elem_value, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .config_ctl
            .write(&mut self.req, node, elem_id, elem_value, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self.mixer_state_ctl.write(
            &mut self.req,
            node,
            elem_id,
            elem_value,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self
            .hw_state_ctl
            .write(&mut self.req, node, elem_id, elem_value, TIMEOUT_MS)?
        {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl NotifyModel<(SndDice, FwNode), u32> for K8Model {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.common_ctl.notified_elem_id_list);
        elem_id_list.extend_from_slice(&self.knob_ctl.1);
        elem_id_list.extend_from_slice(&self.config_ctl.1);
        elem_id_list.extend_from_slice(&self.mixer_state_ctl.1);
        elem_id_list.extend_from_slice(&self.hw_state_ctl.1);
    }

    fn parse_notification(
        &mut self,
        (_, node): &mut (SndDice, FwNode),
        &msg: &u32,
    ) -> Result<(), Error> {
        self.common_ctl
            .parse_notification(&self.req, node, &mut self.sections, msg, TIMEOUT_MS)?;
        self.knob_ctl
            .parse_notification(&self.req, node, msg, TIMEOUT_MS)?;
        self.config_ctl
            .parse_notification(&self.req, node, msg, TIMEOUT_MS)?;
        self.mixer_state_ctl
            .parse_notification(&self.req, node, msg, TIMEOUT_MS)?;
        self.hw_state_ctl
            .parse_notification(&self.req, node, msg, TIMEOUT_MS)?;
        Ok(())
    }
}

impl MeasureModel<(SndDice, FwNode)> for K8Model {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.common_ctl.measured_elem_id_list);
        elem_id_list.extend_from_slice(&self.mixer_meter_ctl.elem_id_list);
    }

    fn measure_states(&mut self, (_, node): &mut (SndDice, FwNode)) -> Result<(), Error> {
        self.common_ctl
            .cache_partial_params(&self.req, node, &mut self.sections, TIMEOUT_MS)?;
        self.mixer_meter_ctl.cache(&self.req, node, TIMEOUT_MS)?;
        Ok(())
    }
}

#[derive(Default, Debug)]
struct KnobCtl(K8KnobSegment, Vec<ElemId>);

impl KnobCtl {
    fn cache(&mut self, req: &FwReq, node: &FwNode, timeout_ms: u32) -> Result<(), Error> {
        let res = K8Protocol::cache_whole_segment(req, node, &mut self.0, timeout_ms);
        debug!(params = ?self.0.data, ?res);
        res
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        load_knob0_target::<K8Protocol, K8Knob>(card_cntr)
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;
        load_knob1_target::<K8Protocol, K8Knob>(card_cntr)
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        Ok(())
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        if read_knob0_target::<K8Protocol, K8Knob>(&self.0, elem_id, elem_value)? {
            Ok(true)
        } else if read_knob1_target::<K8Protocol, K8Knob>(&self.0, elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(
        &mut self,
        req: &FwReq,
        node: &FwNode,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        if write_knob0_target::<K8Protocol, K8Knob>(
            &mut self.0,
            req,
            node,
            elem_id,
            elem_value,
            timeout_ms,
        )? {
            Ok(true)
        } else if write_knob1_target::<K8Protocol, K8Knob>(
            &mut self.0,
            req,
            node,
            elem_id,
            elem_value,
            timeout_ms,
        )? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn parse_notification(
        &mut self,
        req: &FwReq,
        node: &FwNode,
        msg: u32,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        if K8Protocol::is_notified_segment(&self.0, msg) {
            let res = K8Protocol::cache_whole_segment(req, node, &mut self.0, timeout_ms);
            debug!(params = ?self.0.data, ?res);
            res
        } else {
            Ok(())
        }
    }
}

#[derive(Default, Debug)]
struct ConfigCtl(K8ConfigSegment, Vec<ElemId>);

impl ConfigCtl {
    fn cache(&mut self, req: &FwReq, node: &FwNode, timeout_ms: u32) -> Result<(), Error> {
        let res = K8Protocol::cache_whole_segment(req, node, &mut self.0, timeout_ms);
        debug!(params = ?self.0.data, ?res);
        res
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        load_coax_out_src::<K8Protocol, K8Config>(card_cntr)?;
        load_standalone::<K8Protocol, K8Config>(card_cntr)?;

        Ok(())
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        if read_coax_out_src::<K8Protocol, K8Config>(&self.0, elem_id, elem_value)? {
            Ok(true)
        } else if read_standalone::<K8Protocol, K8Config>(&self.0, elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(
        &mut self,
        req: &FwReq,
        node: &FwNode,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        if write_coax_out_src::<K8Protocol, K8Config>(
            &mut self.0,
            req,
            node,
            elem_id,
            elem_value,
            timeout_ms,
        )? {
            Ok(true)
        } else if write_standalone::<K8Protocol, K8Config>(
            &mut self.0,
            req,
            node,
            elem_id,
            elem_value,
            timeout_ms,
        )? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn parse_notification(
        &mut self,
        req: &FwReq,
        node: &FwNode,
        msg: u32,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        if K8Protocol::is_notified_segment(&self.0, msg) {
            let res = K8Protocol::cache_whole_segment(req, node, &mut self.0, timeout_ms);
            debug!(params = ?self.0.data, ?res);
            res
        } else {
            Ok(())
        }
    }
}

#[derive(Default, Debug)]
struct MixerStateCtl(K8MixerStateSegment, Vec<ElemId>);

const MIXER_ENABLE_NAME: &str = "mixer-enable";

impl MixerStateCtl {
    fn cache(&mut self, req: &FwReq, node: &FwNode, timeout_ms: u32) -> Result<(), Error> {
        let res = K8Protocol::cache_whole_segment(req, node, &mut self.0, timeout_ms);
        debug!(params = ?self.0.data, ?res);
        res
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        load_mixer::<K8Protocol, K8MixerState>(&self.0, card_cntr)
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIXER_ENABLE_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        Ok(())
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        if read_mixer::<K8Protocol, K8MixerState>(&self.0, elem_id, elem_value)? {
            Ok(true)
        } else {
            match elem_id.name().as_str() {
                MIXER_ENABLE_NAME => {
                    elem_value.set_bool(&[self.0.data.enabled]);
                    Ok(true)
                }
                _ => Ok(false),
            }
        }
    }

    fn write(
        &mut self,
        req: &FwReq,
        node: &FwNode,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        if write_mixer::<K8Protocol, K8MixerState>(
            &mut self.0,
            req,
            node,
            elem_id,
            elem_value,
            timeout_ms,
        )? {
            Ok(true)
        } else {
            match elem_id.name().as_str() {
                MIXER_ENABLE_NAME => {
                    let mut params = self.0.data.clone();
                    params.enabled = elem_value.boolean()[0];
                    let res = K8Protocol::update_partial_segment(
                        req,
                        node,
                        &params,
                        &mut self.0,
                        timeout_ms,
                    );
                    debug!(params = ?self.0.data, ?res);
                    res.map(|_| true)
                }
                _ => Ok(false),
            }
        }
    }

    fn parse_notification(
        &mut self,
        req: &FwReq,
        node: &FwNode,
        msg: u32,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        if K8Protocol::is_notified_segment(&self.0, msg) {
            let res = K8Protocol::cache_whole_segment(req, node, &mut self.0, timeout_ms);
            debug!(params = ?self.0.data, ?res);
            res
        } else {
            Ok(())
        }
    }
}

#[derive(Default, Debug)]
struct HwStateCtl(K8HwStateSegment, Vec<ElemId>);

const AUX_IN_ENABLED_NAME: &str = "aux-input-enable";

impl HwStateCtl {
    fn cache(&mut self, req: &FwReq, node: &FwNode, timeout_ms: u32) -> Result<(), Error> {
        let res = K8Protocol::cache_whole_segment(req, node, &mut self.0, timeout_ms);
        debug!(params = ?self.0.data, ?res);
        res
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        load_hw_state::<K8Protocol, K8HwState>(card_cntr)
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, AUX_IN_ENABLED_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, 1, 1, false)
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        Ok(())
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            AUX_IN_ENABLED_NAME => {
                elem_value.set_bool(&[self.0.data.aux_input_enabled]);
                Ok(true)
            }
            _ => read_hw_state::<K8Protocol, K8HwState>(&self.0, elem_id, elem_value),
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
        write_hw_state::<K8Protocol, K8HwState>(
            &mut self.0,
            req,
            node,
            elem_id,
            elem_value,
            timeout_ms,
        )
    }

    fn parse_notification(
        &mut self,
        req: &FwReq,
        node: &FwNode,
        msg: u32,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        if K8Protocol::is_notified_segment(&self.0, msg) {
            let res = K8Protocol::cache_whole_segment(req, node, &mut self.0, timeout_ms);
            debug!(params = ?self.0.data, ?res);
            res
        } else {
            Ok(())
        }
    }
}
