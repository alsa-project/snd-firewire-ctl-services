// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use {
    super::{shell_ctl::*, *},
    protocols::tcelectronic::shell::{k8::*, *},
};

#[derive(Default, Debug)]
pub struct K8Model {
    req: FwReq,
    sections: GeneralSections,
    common_ctl: CommonCtl<K8Protocol>,
    knob_ctl: KnobCtl,
    config_ctl: ConfigCtl,
    mixer_state_ctl: MixerStateCtl,
    mixer_meter_ctl: MixerMeterCtl,
    hw_state_ctl: HwStateCtl,
}

const TIMEOUT_MS: u32 = 20;

impl CtlModel<(SndDice, FwNode)> for K8Model {
    fn cache(&mut self, unit: &mut (SndDice, FwNode)) -> Result<(), Error> {
        K8Protocol::read_general_sections(&self.req, &unit.1, &mut self.sections, TIMEOUT_MS)?;

        self.common_ctl
            .cache_whole_params(&self.req, &unit.1, &mut self.sections, TIMEOUT_MS)?;

        self.knob_ctl.cache(&self.req, &unit.1, TIMEOUT_MS)?;
        self.config_ctl.cache(&self.req, &unit.1, TIMEOUT_MS)?;
        self.mixer_state_ctl.cache(&self.req, &unit.1, TIMEOUT_MS)?;
        self.mixer_meter_ctl.cache(&self.req, &unit.1, TIMEOUT_MS)?;
        self.hw_state_ctl.cache(&self.req, &unit.1, TIMEOUT_MS)?;

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

    fn read(
        &mut self,
        _: &mut (SndDice, FwNode),
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
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
        unit: &mut (SndDice, FwNode),
        elem_id: &ElemId,
        _: &ElemValue,
        new: &ElemValue,
    ) -> Result<bool, Error> {
        if self.common_ctl.write(
            &unit.0,
            &self.req,
            &unit.1,
            &mut self.sections,
            elem_id,
            new,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self
            .knob_ctl
            .write(&self.req, &unit.1, elem_id, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .config_ctl
            .write(&self.req, &unit.1, elem_id, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .mixer_state_ctl
            .write(&self.req, &unit.1, elem_id, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .hw_state_ctl
            .write(&self.req, &unit.1, elem_id, new, TIMEOUT_MS)?
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
        unit: &mut (SndDice, FwNode),
        &msg: &u32,
    ) -> Result<(), Error> {
        self.common_ctl.parse_notification(
            &self.req,
            &unit.1,
            &mut self.sections,
            msg,
            TIMEOUT_MS,
        )?;
        self.knob_ctl
            .parse_notification(&self.req, &unit.1, msg, TIMEOUT_MS)?;
        self.config_ctl
            .parse_notification(&self.req, &unit.1, msg, TIMEOUT_MS)?;
        self.mixer_state_ctl
            .parse_notification(&self.req, &unit.1, msg, TIMEOUT_MS)?;
        self.hw_state_ctl
            .parse_notification(&self.req, &unit.1, msg, TIMEOUT_MS)?;
        Ok(())
    }
}

impl MeasureModel<(SndDice, FwNode)> for K8Model {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.common_ctl.measured_elem_id_list);
        elem_id_list.extend_from_slice(&self.mixer_meter_ctl.1);
    }

    fn measure_states(&mut self, unit: &mut (SndDice, FwNode)) -> Result<(), Error> {
        self.common_ctl
            .cache_partial_params(&self.req, &unit.1, &mut self.sections, TIMEOUT_MS)?;
        self.mixer_meter_ctl.cache(&self.req, &unit.1, TIMEOUT_MS)?;
        Ok(())
    }

    fn measure_elem(
        &mut self,
        _: &(SndDice, FwNode),
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.common_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_meter_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[derive(Default, Debug)]
struct KnobCtl(K8KnobSegment, Vec<ElemId>);

impl ShellKnob0CtlOperation<K8Knob, K8Protocol> for KnobCtl {
    fn segment(&self) -> &K8KnobSegment {
        &self.0
    }

    fn segment_mut(&mut self) -> &mut K8KnobSegment {
        &mut self.0
    }

    fn knob0_target(params: &K8Knob) -> &ShellKnob0Target {
        &params.knob0_target
    }

    fn knob0_target_mut(params: &mut K8Knob) -> &mut ShellKnob0Target {
        &mut params.knob0_target
    }
}

impl ShellKnob1CtlOperation<K8Knob, K8Protocol> for KnobCtl {
    fn segment(&self) -> &K8KnobSegment {
        &self.0
    }

    fn segment_mut(&mut self) -> &mut K8KnobSegment {
        &mut self.0
    }

    fn knob1_target(params: &K8Knob) -> &ShellKnob1Target {
        &params.knob1_target
    }

    fn knob1_target_mut(params: &mut K8Knob) -> &mut ShellKnob1Target {
        &mut params.knob1_target
    }
}

impl KnobCtl {
    fn cache(&mut self, req: &FwReq, node: &FwNode, timeout_ms: u32) -> Result<(), Error> {
        let res = K8Protocol::cache_whole_segment(req, node, &mut self.0, timeout_ms);
        debug!(params = ?self.0.data, ?res);
        res
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.load_knob0_target(card_cntr)
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;
        self.load_knob1_target(card_cntr)
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        Ok(())
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        if self.read_knob0_target(elem_id, elem_value)? {
            Ok(true)
        } else if self.read_knob1_target(elem_id, elem_value)? {
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
        if self.write_knob0_target(req, node, elem_id, elem_value, timeout_ms)? {
            Ok(true)
        } else if self.write_knob1_target(req, node, elem_id, elem_value, timeout_ms)? {
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

impl ShellCoaxIfaceCtlOperation<K8Config, K8Protocol> for ConfigCtl {
    fn segment(&self) -> &K8ConfigSegment {
        &self.0
    }

    fn segment_mut(&mut self) -> &mut K8ConfigSegment {
        &mut self.0
    }

    fn coax_out_src(params: &K8Config) -> &ShellCoaxOutPairSrc {
        &params.coax_out_src
    }

    fn coax_out_src_mut(params: &mut K8Config) -> &mut ShellCoaxOutPairSrc {
        &mut params.coax_out_src
    }
}

impl StandaloneCtlOperation<K8Config, K8Protocol> for ConfigCtl {
    fn segment(&self) -> &K8ConfigSegment {
        &self.0
    }

    fn segment_mut(&mut self) -> &mut K8ConfigSegment {
        &mut self.0
    }

    fn standalone_rate(params: &K8Config) -> &TcKonnektStandaloneClockRate {
        &params.standalone_rate
    }

    fn standalone_rate_mut(params: &mut K8Config) -> &mut TcKonnektStandaloneClockRate {
        &mut params.standalone_rate
    }
}

impl ShellStandaloneCtlOperation<K8Config, K8Protocol> for ConfigCtl {
    fn standalone_src(params: &K8Config) -> &ShellStandaloneClockSource {
        &params.standalone_src
    }

    fn standalone_src_mut(params: &mut K8Config) -> &mut ShellStandaloneClockSource {
        &mut params.standalone_src
    }
}

impl ConfigCtl {
    fn cache(&mut self, req: &FwReq, node: &FwNode, timeout_ms: u32) -> Result<(), Error> {
        let res = K8Protocol::cache_whole_segment(req, node, &mut self.0, timeout_ms);
        debug!(params = ?self.0.data, ?res);
        res
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.load_coax_out_src(card_cntr)?;
        self.load_standalone(card_cntr)?;

        Ok(())
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        if self.read_coax_out_src(elem_id, elem_value)? {
            Ok(true)
        } else if self.read_standalone(elem_id, elem_value)? {
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
        if self.write_coax_out_src(req, node, elem_id, elem_value, timeout_ms)? {
            Ok(true)
        } else if self.write_standalone(req, node, elem_id, elem_value, timeout_ms)? {
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

impl ShellMixerStateCtlOperation<K8MixerState, K8MixerMeter, K8Protocol> for MixerStateCtl {
    fn segment(&self) -> &K8MixerStateSegment {
        &self.0
    }

    fn segment_mut(&mut self) -> &mut K8MixerStateSegment {
        &mut self.0
    }

    fn state(params: &K8MixerState) -> &ShellMixerState {
        &params.mixer
    }

    fn state_mut(params: &mut K8MixerState) -> &mut ShellMixerState {
        &mut params.mixer
    }

    fn enabled(&self) -> bool {
        self.0.data.enabled
    }
}

const MIXER_ENABLE_NAME: &str = "mixer-enable";

impl MixerStateCtl {
    fn cache(&mut self, req: &FwReq, node: &FwNode, timeout_ms: u32) -> Result<(), Error> {
        let res = K8Protocol::cache_whole_segment(req, node, &mut self.0, timeout_ms);
        debug!(params = ?self.0.data, ?res);
        res
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.load_mixer(card_cntr)
            .map(|(notified_elem_id_list, _)| {
                self.1 = notified_elem_id_list;
            })?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIXER_ENABLE_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        Ok(())
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        if self.read_mixer(elem_id, elem_value)? {
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
        if self.write_mixer(req, node, elem_id, elem_value, timeout_ms)? {
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
struct MixerMeterCtl(K8MixerMeterSegment, Vec<ElemId>);

impl ShellMixerMeterCtlOperation<K8MixerMeter, K8Protocol> for MixerMeterCtl {
    fn meter(&self) -> &ShellMixerMeter {
        &self.0.data.0
    }

    fn segment(&self) -> &TcKonnektSegment<K8MixerMeter> {
        &self.0
    }

    fn segment_mut(&mut self) -> &mut TcKonnektSegment<K8MixerMeter> {
        &mut self.0
    }
}

#[derive(Default, Debug)]
struct HwStateCtl(K8HwStateSegment, Vec<ElemId>);

impl FirewireLedCtlOperation<K8HwState, K8Protocol> for HwStateCtl {
    fn segment(&self) -> &K8HwStateSegment {
        &self.0
    }

    fn segment_mut(&mut self) -> &mut K8HwStateSegment {
        &mut self.0
    }

    fn firewire_led(params: &K8HwState) -> &FireWireLedState {
        &params.hw_state.firewire_led
    }

    fn firewire_led_mut(params: &mut K8HwState) -> &mut FireWireLedState {
        &mut params.hw_state.firewire_led
    }
}

impl ShellHwStateCtlOperation<K8HwState, K8Protocol> for HwStateCtl {
    fn hw_state(&self) -> &ShellHwState {
        &self.0.data.hw_state
    }

    fn hw_state_mut(&mut self) -> &mut ShellHwState {
        &mut self.0.data.hw_state
    }
}

const AUX_IN_ENABLED_NAME: &str = "aux-input-enable";

impl HwStateCtl {
    fn cache(&mut self, req: &FwReq, node: &FwNode, timeout_ms: u32) -> Result<(), Error> {
        let res = K8Protocol::cache_whole_segment(req, node, &mut self.0, timeout_ms);
        debug!(params = ?self.0.data, ?res);
        res
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.load_hw_state(card_cntr)
            .map(|mut notified_elem_id_list| self.1.append(&mut notified_elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, AUX_IN_ENABLED_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, 1, 1, false)
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        Ok(())
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        if self.read_hw_state(elem_id, elem_value)? {
            Ok(true)
        } else {
            match elem_id.name().as_str() {
                AUX_IN_ENABLED_NAME => {
                    elem_value.set_bool(&[self.0.data.aux_input_enabled]);
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
        self.write_hw_state(req, node, elem_id, elem_value, timeout_ms)
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
