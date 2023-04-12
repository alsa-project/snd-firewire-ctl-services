// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use {
    super::{shell_ctl::*, *},
    protocols::tcelectronic::shell::{k24d::*, *},
};

#[derive(Default, Debug)]
pub struct K24dModel {
    req: FwReq,
    sections: GeneralSections,
    common_ctl: CommonCtl<K24dProtocol>,
    knob_ctl: KnobCtl,
    config_ctl: ConfigCtl,
    mixer_state_ctl: MixerStateCtl,
    mixer_meter_ctl: MixerMeterCtl,
    hw_state_ctl: HwStateCtl,
    reverb_state_ctl: ReverbStateCtl,
    reverb_meter_ctl: ReverbMeterCtl,
    ch_strip_state_ctl: ChStripStateCtl,
    ch_strip_meter_ctl: ChStripMeterCtl,
}

const TIMEOUT_MS: u32 = 20;

impl CtlModel<(SndDice, FwNode)> for K24dModel {
    fn cache(&mut self, unit: &mut (SndDice, FwNode)) -> Result<(), Error> {
        K24dProtocol::read_general_sections(&self.req, &unit.1, &mut self.sections, TIMEOUT_MS)?;

        self.common_ctl
            .cache_whole_params(&self.req, &unit.1, &mut self.sections, TIMEOUT_MS)?;

        self.knob_ctl.cache(&self.req, &unit.1, TIMEOUT_MS)?;
        self.config_ctl.cache(&self.req, &unit.1, TIMEOUT_MS)?;
        self.mixer_state_ctl.cache(&self.req, &unit.1, TIMEOUT_MS)?;
        self.mixer_meter_ctl.cache(&self.req, &unit.1, TIMEOUT_MS)?;
        self.hw_state_ctl.cache(&self.req, &unit.1, TIMEOUT_MS)?;
        self.reverb_state_ctl
            .cache(&self.req, &unit.1, TIMEOUT_MS)?;
        self.reverb_meter_ctl
            .cache(&self.req, &unit.1, TIMEOUT_MS)?;
        self.ch_strip_state_ctl
            .cache(&self.req, &unit.1, TIMEOUT_MS)?;
        self.ch_strip_meter_ctl
            .cache(&self.req, &unit.1, TIMEOUT_MS)?;

        Ok(())
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.common_ctl.load(card_cntr)?;

        self.knob_ctl.load(card_cntr)?;
        self.config_ctl.load(card_cntr)?;
        self.mixer_state_ctl.load(card_cntr)?;
        self.mixer_meter_ctl.load(card_cntr)?;
        self.hw_state_ctl.load(card_cntr)?;
        self.reverb_state_ctl
            .load(card_cntr)
            .map(|notified_elem_id_list| self.reverb_state_ctl.1 = notified_elem_id_list)?;
        self.reverb_meter_ctl
            .load(card_cntr)
            .map(|measured_elem_id_list| self.reverb_meter_ctl.1 = measured_elem_id_list)?;
        self.ch_strip_state_ctl
            .load(card_cntr)
            .map(|notified_elem_id_list| self.ch_strip_state_ctl.1 = notified_elem_id_list)?;
        self.ch_strip_meter_ctl
            .load(card_cntr)
            .map(|measured_elem_id_list| {
                self.ch_strip_meter_ctl.1 = measured_elem_id_list;
            })?;

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
        } else if self.reverb_state_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.reverb_meter_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.ch_strip_state_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.ch_strip_meter_ctl.read(elem_id, elem_value)? {
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
        } else if self
            .reverb_state_ctl
            .write(&self.req, &unit.1, elem_id, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .ch_strip_state_ctl
            .write(&self.req, &unit.1, elem_id, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl NotifyModel<(SndDice, FwNode), u32> for K24dModel {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.common_ctl.notified_elem_id_list);
        elem_id_list.extend_from_slice(&self.knob_ctl.1);
        elem_id_list.extend_from_slice(&self.config_ctl.1);
        elem_id_list.extend_from_slice(&self.mixer_state_ctl.1);
        elem_id_list.extend_from_slice(&self.hw_state_ctl.1);
        elem_id_list.extend_from_slice(&self.reverb_state_ctl.1);
        elem_id_list.extend_from_slice(&self.ch_strip_state_ctl.1);
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
        self.reverb_state_ctl
            .parse_notification(&self.req, &unit.1, msg, TIMEOUT_MS)?;
        self.ch_strip_state_ctl
            .parse_notification(&self.req, &unit.1, msg, TIMEOUT_MS)?;
        Ok(())
    }
}

impl MeasureModel<(SndDice, FwNode)> for K24dModel {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.common_ctl.measured_elem_id_list);
        elem_id_list.extend_from_slice(&self.mixer_meter_ctl.1);
        elem_id_list.extend_from_slice(&self.reverb_meter_ctl.1);
        elem_id_list.extend_from_slice(&self.ch_strip_meter_ctl.1);
    }

    fn measure_states(&mut self, unit: &mut (SndDice, FwNode)) -> Result<(), Error> {
        self.common_ctl
            .cache_partial_params(&self.req, &unit.1, &mut self.sections, TIMEOUT_MS)?;
        self.mixer_meter_ctl.cache(&self.req, &unit.1, TIMEOUT_MS)?;
        if !self.reverb_state_ctl.is_bypassed() {
            self.reverb_meter_ctl
                .cache(&self.req, &unit.1, TIMEOUT_MS)?;
        }
        if !self.ch_strip_state_ctl.are_bypassed() {
            self.ch_strip_meter_ctl
                .cache(&self.req, &unit.1, TIMEOUT_MS)?;
        }
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
        } else if self.reverb_meter_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.ch_strip_meter_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[derive(Default, Debug)]
struct KnobCtl(K24dKnobSegment, Vec<ElemId>);

impl ShellKnob0CtlOperation<K24dKnob, K24dProtocol> for KnobCtl {
    fn segment(&self) -> &K24dKnobSegment {
        &self.0
    }

    fn segment_mut(&mut self) -> &mut K24dKnobSegment {
        &mut self.0
    }

    fn knob0_target(params: &K24dKnob) -> &ShellKnob0Target {
        &params.knob0_target
    }

    fn knob0_target_mut(params: &mut K24dKnob) -> &mut ShellKnob0Target {
        &mut params.knob0_target
    }
}

impl ShellKnob1CtlOperation<K24dKnob, K24dProtocol> for KnobCtl {
    fn segment(&self) -> &K24dKnobSegment {
        &self.0
    }

    fn segment_mut(&mut self) -> &mut K24dKnobSegment {
        &mut self.0
    }

    fn knob1_target(params: &K24dKnob) -> &ShellKnob1Target {
        &params.knob1_target
    }

    fn knob1_target_mut(params: &mut K24dKnob) -> &mut ShellKnob1Target {
        &mut params.knob1_target
    }
}

impl ProgramCtlOperation<K24dKnob, K24dProtocol> for KnobCtl {
    fn segment(&self) -> &K24dKnobSegment {
        &self.0
    }

    fn segment_mut(&mut self) -> &mut K24dKnobSegment {
        &mut self.0
    }

    fn prog(params: &K24dKnob) -> &TcKonnektLoadedProgram {
        &params.prog
    }

    fn prog_mut(params: &mut K24dKnob) -> &mut TcKonnektLoadedProgram {
        &mut params.prog
    }
}

impl KnobCtl {
    fn cache(&mut self, req: &FwReq, node: &FwNode, timeout_ms: u32) -> Result<(), Error> {
        let res = K24dProtocol::cache_whole_segment(req, node, &mut self.0, timeout_ms);
        debug!(params = ?self.0.data, ?res);
        res
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.load_knob0_target(card_cntr)
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        self.load_knob1_target(card_cntr)
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        self.load_prog(card_cntr)
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        Ok(())
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        if self.read_knob0_target(elem_id, elem_value)? {
            Ok(true)
        } else if self.read_knob1_target(elem_id, elem_value)? {
            Ok(true)
        } else if self.read_prog(elem_id, elem_value)? {
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
        } else if self.write_prog(req, node, elem_id, elem_value, timeout_ms)? {
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
        if K24dProtocol::is_notified_segment(&self.0, msg) {
            let res = K24dProtocol::cache_whole_segment(req, node, &mut self.0, timeout_ms);
            debug!(params = ?self.0.data, ?res);
            res
        } else {
            Ok(())
        }
    }
}

#[derive(Default, Debug)]
struct ConfigCtl(K24dConfigSegment, Vec<ElemId>);

impl ShellCoaxIfaceCtlOperation<K24dConfig, K24dProtocol> for ConfigCtl {
    fn segment(&self) -> &K24dConfigSegment {
        &self.0
    }

    fn segment_mut(&mut self) -> &mut K24dConfigSegment {
        &mut self.0
    }

    fn coax_out_src(params: &K24dConfig) -> &ShellCoaxOutPairSrc {
        &params.coax_out_src
    }

    fn coax_out_src_mut(params: &mut K24dConfig) -> &mut ShellCoaxOutPairSrc {
        &mut params.coax_out_src
    }
}

impl ShellOptIfaceCtl<K24dConfig, K24dProtocol> for ConfigCtl {
    fn segment(&self) -> &K24dConfigSegment {
        &self.0
    }

    fn segment_mut(&mut self) -> &mut K24dConfigSegment {
        &mut self.0
    }

    fn opt_iface_config(params: &K24dConfig) -> &ShellOptIfaceConfig {
        &params.opt
    }

    fn opt_iface_config_mut(params: &mut K24dConfig) -> &mut ShellOptIfaceConfig {
        &mut params.opt
    }
}

impl StandaloneCtlOperation<K24dConfig, K24dProtocol> for ConfigCtl {
    fn segment(&self) -> &K24dConfigSegment {
        &self.0
    }

    fn segment_mut(&mut self) -> &mut K24dConfigSegment {
        &mut self.0
    }

    fn standalone_rate(params: &K24dConfig) -> &TcKonnektStandaloneClockRate {
        &params.standalone_rate
    }

    fn standalone_rate_mut(params: &mut K24dConfig) -> &mut TcKonnektStandaloneClockRate {
        &mut params.standalone_rate
    }
}

impl ShellStandaloneCtlOperation<K24dConfig, K24dProtocol> for ConfigCtl {
    fn standalone_src(params: &K24dConfig) -> &ShellStandaloneClockSource {
        &params.standalone_src
    }

    fn standalone_src_mut(params: &mut K24dConfig) -> &mut ShellStandaloneClockSource {
        &mut params.standalone_src
    }
}

const OUT_23_SRC_NAME: &str = "output-3/4-source";

impl ConfigCtl {
    fn cache(&mut self, req: &FwReq, node: &FwNode, timeout_ms: u32) -> Result<(), Error> {
        let res = K24dProtocol::cache_whole_segment(req, node, &mut self.0, timeout_ms);
        debug!(params = ?self.0.data, ?res);
        res
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.load_coax_out_src(card_cntr)?;
        self.load_opt_iface_config(card_cntr)?;
        self.load_standalone(card_cntr)?;

        let labels: Vec<&str> = PHYS_OUT_SRCS
            .iter()
            .map(|s| phys_out_src_to_str(s))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, OUT_23_SRC_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        Ok(())
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        if self.read_coax_out_src(elem_id, elem_value)? {
            Ok(true)
        } else if self.read_opt_iface_config(elem_id, elem_value)? {
            Ok(true)
        } else if self.read_standalone(elem_id, elem_value)? {
            Ok(true)
        } else {
            match elem_id.name().as_str() {
                OUT_23_SRC_NAME => {
                    let pos = PHYS_OUT_SRCS
                        .iter()
                        .position(|s| self.0.data.out_23_src.eq(s))
                        .unwrap();
                    elem_value.set_enum(&[pos as u32]);
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
        if self.write_coax_out_src(req, node, elem_id, elem_value, timeout_ms)? {
            Ok(true)
        } else if self.write_opt_iface_config(req, node, elem_id, elem_value, timeout_ms)? {
            Ok(true)
        } else if self.write_standalone(req, node, elem_id, elem_value, timeout_ms)? {
            Ok(true)
        } else {
            match elem_id.name().as_str() {
                OUT_23_SRC_NAME => {
                    let mut params = self.0.data.clone();
                    let pos = elem_value.enumerated()[0] as usize;
                    PHYS_OUT_SRCS
                        .iter()
                        .nth(pos)
                        .ok_or_else(|| {
                            let msg = format!("Invalid index of output source: {}", pos);
                            Error::new(FileError::Inval, &msg)
                        })
                        .map(|&s| params.out_23_src = s)?;
                    let res = K24dProtocol::update_partial_segment(
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
        if K24dProtocol::is_notified_segment(&self.0, msg) {
            let res = K24dProtocol::cache_whole_segment(req, node, &mut self.0, timeout_ms);
            debug!(params = ?self.0.data, ?res);
            res
        } else {
            Ok(())
        }
    }
}

#[derive(Default, Debug)]
struct MixerStateCtl(K24dMixerStateSegment, Vec<ElemId>);

impl ShellMixerStateCtlOperation<K24dMixerState, K24dMixerMeter, K24dProtocol> for MixerStateCtl {
    fn segment(&self) -> &K24dMixerStateSegment {
        &self.0
    }

    fn segment_mut(&mut self) -> &mut K24dMixerStateSegment {
        &mut self.0
    }

    fn state(params: &K24dMixerState) -> &ShellMixerState {
        &params.mixer
    }

    fn state_mut(params: &mut K24dMixerState) -> &mut ShellMixerState {
        &mut params.mixer
    }

    fn enabled(&self) -> bool {
        self.0.data.enabled
    }
}

impl ShellReverbReturnCtlOperation<K24dMixerState, K24dProtocol> for MixerStateCtl {
    fn segment(&self) -> &K24dMixerStateSegment {
        &self.0
    }

    fn segment_mut(&mut self) -> &mut K24dMixerStateSegment {
        &mut self.0
    }

    fn reverb_return(params: &K24dMixerState) -> &ShellReverbReturn {
        &params.reverb_return
    }

    fn reverb_return_mut(params: &mut K24dMixerState) -> &mut ShellReverbReturn {
        &mut params.reverb_return
    }
}

const MIXER_ENABLE_NAME: &str = "mixer-enable";
const USE_CH_STRIP_AS_PLUGIN_NAME: &str = "use-channel-strip-as-plugin";
const USE_REVERB_AT_MID_RATE: &str = "use-reverb-at-mid-rate";

impl MixerStateCtl {
    fn cache(&mut self, req: &FwReq, node: &FwNode, timeout_ms: u32) -> Result<(), Error> {
        let res = K24dProtocol::cache_whole_segment(req, node, &mut self.0, timeout_ms);
        debug!(params = ?self.0.data, ?res);
        res
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.load_mixer(card_cntr)
            .map(|(notified_elem_id_list, _)| {
                self.1 = notified_elem_id_list;
            })?;

        self.load_reverb_return(card_cntr)
            .map(|mut notified_elem_id_list| self.1.append(&mut notified_elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIXER_ENABLE_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        let elem_id =
            ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, USE_CH_STRIP_AS_PLUGIN_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, USE_REVERB_AT_MID_RATE, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        Ok(())
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        if self.read_mixer(elem_id, elem_value)? {
            Ok(true)
        } else if self.read_reverb_return(elem_id, elem_value)? {
            Ok(true)
        } else {
            match elem_id.name().as_str() {
                MIXER_ENABLE_NAME => {
                    elem_value.set_bool(&[self.0.data.enabled]);
                    Ok(true)
                }
                USE_CH_STRIP_AS_PLUGIN_NAME => {
                    elem_value.set_bool(&[self.0.data.use_ch_strip_as_plugin]);
                    Ok(true)
                }
                USE_REVERB_AT_MID_RATE => {
                    elem_value.set_bool(&[self.0.data.use_reverb_at_mid_rate]);
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
        } else if self.write_reverb_return(req, node, elem_id, elem_value, timeout_ms)? {
            Ok(true)
        } else {
            match elem_id.name().as_str() {
                MIXER_ENABLE_NAME => {
                    let mut params = self.0.data.clone();
                    params.enabled = elem_value.boolean()[0];
                    let res = K24dProtocol::update_partial_segment(
                        req,
                        node,
                        &params,
                        &mut self.0,
                        timeout_ms,
                    );
                    debug!(params = ?self.0.data, ?res);
                    res.map(|_| true)
                }
                USE_CH_STRIP_AS_PLUGIN_NAME => {
                    let mut params = self.0.data.clone();
                    params.use_ch_strip_as_plugin = elem_value.boolean()[0];
                    let res = K24dProtocol::update_partial_segment(
                        req,
                        node,
                        &params,
                        &mut self.0,
                        timeout_ms,
                    );
                    debug!(params = ?self.0.data, ?res);
                    res.map(|_| true)
                }
                USE_REVERB_AT_MID_RATE => {
                    let mut params = self.0.data.clone();
                    params.use_reverb_at_mid_rate = elem_value.boolean()[0];
                    let res = K24dProtocol::update_partial_segment(
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
        if K24dProtocol::is_notified_segment(&self.0, msg) {
            let res = K24dProtocol::cache_whole_segment(req, node, &mut self.0, timeout_ms);
            debug!(params = ?self.0.data, ?res);
            res
        } else {
            Ok(())
        }
    }
}

#[derive(Default, Debug)]
struct MixerMeterCtl(K24dMixerMeterSegment, Vec<ElemId>);

impl ShellMixerMeterCtlOperation<K24dMixerMeter, K24dProtocol> for MixerMeterCtl {
    fn meter(&self) -> &ShellMixerMeter {
        &self.0.data.0
    }

    fn segment(&self) -> &TcKonnektSegment<K24dMixerMeter> {
        &self.0
    }

    fn segment_mut(&mut self) -> &mut TcKonnektSegment<K24dMixerMeter> {
        &mut self.0
    }
}

#[derive(Default, Debug)]
struct HwStateCtl(K24dHwStateSegment, Vec<ElemId>);

impl FirewireLedCtlOperation<K24dHwState, K24dProtocol> for HwStateCtl {
    fn segment(&self) -> &K24dHwStateSegment {
        &self.0
    }

    fn segment_mut(&mut self) -> &mut K24dHwStateSegment {
        &mut self.0
    }

    fn firewire_led(params: &K24dHwState) -> &FireWireLedState {
        &params.0.firewire_led
    }

    fn firewire_led_mut(params: &mut K24dHwState) -> &mut FireWireLedState {
        &mut params.0.firewire_led
    }
}

impl ShellHwStateCtlOperation<K24dHwState, K24dProtocol> for HwStateCtl {
    fn hw_state(&self) -> &ShellHwState {
        &self.0.data.0
    }

    fn hw_state_mut(&mut self) -> &mut ShellHwState {
        &mut self.0.data.0
    }
}

impl HwStateCtl {
    fn cache(&mut self, req: &FwReq, node: &FwNode, timeout_ms: u32) -> Result<(), Error> {
        let res = K24dProtocol::cache_whole_segment(req, node, &mut self.0, timeout_ms);
        debug!(params = ?self.0.data, ?res);
        res
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.load_hw_state(card_cntr)
            .map(|mut notified_elem_id_list| self.1.append(&mut notified_elem_id_list))?;

        Ok(())
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        if self.read_hw_state(elem_id, elem_value)? {
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
        if self.write_hw_state(req, node, elem_id, elem_value, timeout_ms)? {
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
        if K24dProtocol::is_notified_segment(&self.0, msg) {
            let res = K24dProtocol::cache_whole_segment(req, node, &mut self.0, timeout_ms);
            debug!(params = ?self.0.data, ?res);
            res
        } else {
            Ok(())
        }
    }
}

#[derive(Default, Debug)]
struct ReverbStateCtl(K24dReverbStateSegment, Vec<ElemId>);

impl ReverbStateCtlOpreation<K24dReverbState, K24dReverbMeter, K24dProtocol> for ReverbStateCtl {
    fn segment(&self) -> &K24dReverbStateSegment {
        &self.0
    }

    fn segment_mut(&mut self) -> &mut K24dReverbStateSegment {
        &mut self.0
    }

    fn state(params: &K24dReverbState) -> &ReverbState {
        &params.0
    }

    fn state_mut(params: &mut K24dReverbState) -> &mut ReverbState {
        &mut params.0
    }
}

#[derive(Default, Debug)]
struct ReverbMeterCtl(K24dReverbMeterSegment, Vec<ElemId>);

impl ReverbMeterCtlOperation<K24dReverbMeter, K24dProtocol> for ReverbMeterCtl {
    fn meter(&self) -> &ReverbMeter {
        &self.0.data.0
    }

    fn segment(&self) -> &TcKonnektSegment<K24dReverbMeter> {
        &self.0
    }

    fn segment_mut(&mut self) -> &mut TcKonnektSegment<K24dReverbMeter> {
        &mut self.0
    }
}

#[derive(Default, Debug)]
struct ChStripStateCtl(K24dChStripStatesSegment, Vec<ElemId>);

impl ChStripStateCtlOperation<K24dChStripStates, K24dProtocol> for ChStripStateCtl {
    fn segment(&self) -> &K24dChStripStatesSegment {
        &self.0
    }

    fn segment_mut(&mut self) -> &mut K24dChStripStatesSegment {
        &mut self.0
    }

    fn states(params: &K24dChStripStates) -> &[ChStripState] {
        &params.0
    }

    fn states_mut(params: &mut K24dChStripStates) -> &mut [ChStripState] {
        &mut params.0
    }
}

#[derive(Default, Debug)]
struct ChStripMeterCtl(K24dChStripMetersSegment, Vec<ElemId>);

impl ChStripMeterCtlOperation<K24dChStripMeters, K24dProtocol> for ChStripMeterCtl {
    fn meters(&self) -> &[ChStripMeter] {
        &self.0.data.0
    }

    fn segment(&self) -> &TcKonnektSegment<K24dChStripMeters> {
        &self.0
    }

    fn segment_mut(&mut self) -> &mut TcKonnektSegment<K24dChStripMeters> {
        &mut self.0
    }
}
