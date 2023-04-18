// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use {
    super::{shell_ctl::*, *},
    protocols::tcelectronic::shell::{klive::*, *},
};

#[derive(Default, Debug)]
pub struct KliveModel {
    req: FwReq,
    sections: GeneralSections,
    common_ctl: CommonCtl<KliveProtocol>,
    knob_ctl: KnobCtl,
    config_ctl: ConfigCtl,
    mixer_state_ctl: MixerStateCtl,
    mixer_meter_ctl: MixerMeterCtl<KliveProtocol, KliveMixerMeter>,
    hw_state_ctl: HwStateCtl,
    reverb_state_ctl: ReverbStateCtl<KliveProtocol, KliveReverbState>,
    reverb_meter_ctl: ReverbMeterCtl<KliveProtocol, KliveReverbMeter>,
    ch_strip_state_ctl: ChStripStateCtl<KliveProtocol, KliveChStripStates>,
    ch_strip_meter_ctl: ChStripMeterCtl<KliveProtocol, KliveChStripMeters>,
}

const TIMEOUT_MS: u32 = 20;

impl CtlModel<(SndDice, FwNode)> for KliveModel {
    fn cache(&mut self, (_, node): &mut (SndDice, FwNode)) -> Result<(), Error> {
        KliveProtocol::read_general_sections(&mut self.req, node, &mut self.sections, TIMEOUT_MS)?;

        self.common_ctl
            .cache_whole_params(&mut self.req, node, &mut self.sections, TIMEOUT_MS)?;

        self.knob_ctl.cache(&mut self.req, node, TIMEOUT_MS)?;
        self.config_ctl.cache(&mut self.req, node, TIMEOUT_MS)?;
        self.mixer_state_ctl
            .cache(&mut self.req, node, TIMEOUT_MS)?;
        self.mixer_meter_ctl
            .cache(&mut self.req, node, TIMEOUT_MS)?;
        self.hw_state_ctl.cache(&mut self.req, node, TIMEOUT_MS)?;
        self.reverb_state_ctl
            .cache(&mut self.req, node, TIMEOUT_MS)?;
        self.reverb_meter_ctl
            .cache(&mut self.req, node, TIMEOUT_MS)?;
        self.ch_strip_state_ctl
            .cache(&mut self.req, node, TIMEOUT_MS)?;
        self.ch_strip_meter_ctl
            .cache(&mut self.req, node, TIMEOUT_MS)?;

        Ok(())
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.common_ctl.load(card_cntr)?;

        self.knob_ctl.load(card_cntr)?;
        self.config_ctl.load(card_cntr)?;
        self.mixer_state_ctl.load(card_cntr)?;
        self.mixer_meter_ctl.load(card_cntr)?;
        self.hw_state_ctl.load(card_cntr)?;
        self.reverb_state_ctl.load(card_cntr)?;
        self.reverb_meter_ctl.load(card_cntr)?;
        self.ch_strip_state_ctl.load(card_cntr)?;
        self.ch_strip_meter_ctl.load(card_cntr)?;

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
        } else if self.reverb_state_ctl.write(
            &mut self.req,
            node,
            elem_id,
            elem_value,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self.ch_strip_state_ctl.write(
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

impl NotifyModel<(SndDice, FwNode), u32> for KliveModel {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.common_ctl.notified_elem_id_list);
        elem_id_list.extend_from_slice(&self.knob_ctl.1);
        elem_id_list.extend_from_slice(&self.config_ctl.1);
        elem_id_list.extend_from_slice(&self.mixer_state_ctl.1);
        elem_id_list.extend_from_slice(&self.hw_state_ctl.1);
        elem_id_list.extend_from_slice(&self.reverb_state_ctl.elem_id_list);
        elem_id_list.extend_from_slice(&self.ch_strip_state_ctl.elem_id_list);
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
        self.reverb_state_ctl
            .parse_notification(&self.req, node, msg, TIMEOUT_MS)?;
        self.ch_strip_state_ctl
            .parse_notification(&self.req, node, msg, TIMEOUT_MS)?;
        Ok(())
    }
}

impl MeasureModel<(SndDice, FwNode)> for KliveModel {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.common_ctl.measured_elem_id_list);
        elem_id_list.extend_from_slice(&self.mixer_meter_ctl.elem_id_list);
        elem_id_list.extend_from_slice(&self.reverb_meter_ctl.elem_id_list);
        elem_id_list.extend_from_slice(&self.ch_strip_meter_ctl.elem_id_list);
    }

    fn measure_states(&mut self, (_, node): &mut (SndDice, FwNode)) -> Result<(), Error> {
        self.common_ctl
            .cache_partial_params(&self.req, node, &mut self.sections, TIMEOUT_MS)?;
        self.mixer_meter_ctl.cache(&self.req, node, TIMEOUT_MS)?;
        if !self.reverb_state_ctl.is_bypassed() {
            self.reverb_meter_ctl.cache(&self.req, node, TIMEOUT_MS)?;
        }
        if !self.ch_strip_state_ctl.are_bypassed() {
            self.ch_strip_meter_ctl.cache(&self.req, node, TIMEOUT_MS)?;
        }
        Ok(())
    }
}

#[derive(Default, Debug)]
struct KnobCtl(KliveKnobSegment, Vec<ElemId>);

impl ShellKnob0CtlOperation<KliveKnob, KliveProtocol> for KnobCtl {
    fn segment(&self) -> &KliveKnobSegment {
        &self.0
    }

    fn segment_mut(&mut self) -> &mut KliveKnobSegment {
        &mut self.0
    }

    fn knob0_target(params: &KliveKnob) -> &ShellKnob0Target {
        &params.knob0_target
    }

    fn knob0_target_mut(params: &mut KliveKnob) -> &mut ShellKnob0Target {
        &mut params.knob0_target
    }
}

impl ShellKnob1CtlOperation<KliveKnob, KliveProtocol> for KnobCtl {
    fn segment(&self) -> &KliveKnobSegment {
        &self.0
    }

    fn segment_mut(&mut self) -> &mut KliveKnobSegment {
        &mut self.0
    }

    fn knob1_target(params: &KliveKnob) -> &ShellKnob1Target {
        &params.knob1_target
    }

    fn knob1_target_mut(params: &mut KliveKnob) -> &mut ShellKnob1Target {
        &mut params.knob1_target
    }
}

impl ProgramCtlOperation<KliveKnob, KliveProtocol> for KnobCtl {
    fn segment(&self) -> &KliveKnobSegment {
        &self.0
    }

    fn segment_mut(&mut self) -> &mut KliveKnobSegment {
        &mut self.0
    }

    fn prog(params: &KliveKnob) -> &TcKonnektLoadedProgram {
        &params.prog
    }

    fn prog_mut(params: &mut KliveKnob) -> &mut TcKonnektLoadedProgram {
        &mut params.prog
    }
}

const OUTPUT_IMPEDANCE_NAME: &str = "output-impedance";

fn impedance_to_str(impedance: &OutputImpedance) -> &'static str {
    match impedance {
        OutputImpedance::Unbalance => "Unbalance",
        OutputImpedance::Balance => "Balance",
    }
}

impl KnobCtl {
    const OUTPUT_IMPEDANCES: [OutputImpedance; 2] =
        [OutputImpedance::Unbalance, OutputImpedance::Balance];

    fn cache(&mut self, req: &FwReq, node: &FwNode, timeout_ms: u32) -> Result<(), Error> {
        let res = KliveProtocol::cache_whole_segment(req, node, &mut self.0, timeout_ms);
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

        let labels: Vec<&str> = Self::OUTPUT_IMPEDANCES
            .iter()
            .map(|i| impedance_to_str(i))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, OUTPUT_IMPEDANCE_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 2, &labels, None, true)?;

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
            match elem_id.name().as_str() {
                OUTPUT_IMPEDANCE_NAME => {
                    let vals: Vec<u32> = self
                        .0
                        .data
                        .out_impedance
                        .iter()
                        .map(|impedance| {
                            let pos = Self::OUTPUT_IMPEDANCES
                                .iter()
                                .position(|i| impedance.eq(i))
                                .unwrap();
                            pos as u32
                        })
                        .collect();
                    elem_value.set_enum(&vals);
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
        if self.write_knob0_target(req, node, elem_id, elem_value, timeout_ms)? {
            Ok(true)
        } else if self.write_knob1_target(req, node, elem_id, elem_value, timeout_ms)? {
            Ok(true)
        } else if self.write_prog(req, node, elem_id, elem_value, timeout_ms)? {
            Ok(true)
        } else {
            match elem_id.name().as_str() {
                OUTPUT_IMPEDANCE_NAME => {
                    let mut params = self.0.data.clone();
                    params
                        .out_impedance
                        .iter_mut()
                        .zip(elem_value.enumerated())
                        .try_for_each(|(imp, &val)| {
                            let pos = val as usize;
                            Self::OUTPUT_IMPEDANCES
                                .iter()
                                .nth(pos)
                                .ok_or_else(|| {
                                    let msg = format!("Invalid index of output impedance: {}", pos);
                                    Error::new(FileError::Inval, &msg)
                                })
                                .map(|&i| *imp = i)
                        })?;
                    let res = KliveProtocol::update_partial_segment(
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
        if KliveProtocol::is_notified_segment(&self.0, msg) {
            let res = KliveProtocol::cache_whole_segment(req, node, &mut self.0, timeout_ms);
            debug!(params = ?self.0.data, ?res);
            res
        } else {
            Ok(())
        }
    }
}

#[derive(Default, Debug)]
struct ConfigCtl(KliveConfigSegment, Vec<ElemId>);

impl ShellMixerStreamSrcCtlOperation<KliveConfig, KliveProtocol> for ConfigCtl {
    fn segment(&self) -> &KliveConfigSegment {
        &self.0
    }

    fn segment_mut(&mut self) -> &mut KliveConfigSegment {
        &mut self.0
    }

    fn mixer_stream_src(params: &KliveConfig) -> &ShellMixerStreamSourcePair {
        &params.mixer_stream_src_pair
    }

    fn mixer_stream_src_mut(params: &mut KliveConfig) -> &mut ShellMixerStreamSourcePair {
        &mut params.mixer_stream_src_pair
    }
}

impl ShellCoaxIfaceCtlOperation<KliveConfig, KliveProtocol> for ConfigCtl {
    fn segment(&self) -> &KliveConfigSegment {
        &self.0
    }

    fn segment_mut(&mut self) -> &mut KliveConfigSegment {
        &mut self.0
    }

    fn coax_out_src(params: &KliveConfig) -> &ShellCoaxOutPairSrc {
        &params.coax_out_src
    }

    fn coax_out_src_mut(params: &mut KliveConfig) -> &mut ShellCoaxOutPairSrc {
        &mut params.coax_out_src
    }
}

impl ShellOptIfaceCtl<KliveConfig, KliveProtocol> for ConfigCtl {
    fn segment(&self) -> &KliveConfigSegment {
        &self.0
    }

    fn segment_mut(&mut self) -> &mut KliveConfigSegment {
        &mut self.0
    }

    fn opt_iface_config(params: &KliveConfig) -> &ShellOptIfaceConfig {
        &params.opt
    }

    fn opt_iface_config_mut(params: &mut KliveConfig) -> &mut ShellOptIfaceConfig {
        &mut params.opt
    }
}

impl StandaloneCtlOperation<KliveConfig, KliveProtocol> for ConfigCtl {
    fn segment(&self) -> &KliveConfigSegment {
        &self.0
    }

    fn segment_mut(&mut self) -> &mut KliveConfigSegment {
        &mut self.0
    }

    fn standalone_rate(params: &KliveConfig) -> &TcKonnektStandaloneClockRate {
        &params.standalone_rate
    }

    fn standalone_rate_mut(params: &mut KliveConfig) -> &mut TcKonnektStandaloneClockRate {
        &mut params.standalone_rate
    }
}

impl ShellStandaloneCtlOperation<KliveConfig, KliveProtocol> for ConfigCtl {
    fn standalone_src(params: &KliveConfig) -> &ShellStandaloneClockSource {
        &params.standalone_src
    }

    fn standalone_src_mut(params: &mut KliveConfig) -> &mut ShellStandaloneClockSource {
        &mut params.standalone_src
    }
}

impl MidiSendCtlOperation<KliveConfig, KliveProtocol> for ConfigCtl {
    fn segment(&self) -> &KliveConfigSegment {
        &self.0
    }

    fn segment_mut(&mut self) -> &mut KliveConfigSegment {
        &mut self.0
    }

    fn midi_sender(params: &KliveConfig) -> &TcKonnektMidiSender {
        &params.midi_sender
    }

    fn midi_sender_mut(params: &mut KliveConfig) -> &mut TcKonnektMidiSender {
        &mut params.midi_sender
    }
}

const OUT_01_SRC_NAME: &str = "output-1/2-source";
const OUT_23_SRC_NAME: &str = "output-3/4-source";

impl ConfigCtl {
    fn cache(&mut self, req: &FwReq, node: &FwNode, timeout_ms: u32) -> Result<(), Error> {
        let res = KliveProtocol::cache_whole_segment(req, node, &mut self.0, timeout_ms);
        debug!(params = ?self.0.data, ?res);
        res
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.load_mixer_stream_src(card_cntr)?;
        self.load_coax_out_src(card_cntr)?;
        self.load_opt_iface_config(card_cntr)?;
        self.load_standalone(card_cntr)?;
        self.load_midi_sender(card_cntr)?;

        let labels: Vec<&str> = PHYS_OUT_SRCS
            .iter()
            .map(|s| phys_out_src_to_str(s))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, OUT_01_SRC_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, OUT_23_SRC_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        Ok(())
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        if self.read_mixer_stream_src(elem_id, elem_value)? {
            Ok(true)
        } else if self.read_coax_out_src(elem_id, elem_value)? {
            Ok(true)
        } else if self.read_opt_iface_config(elem_id, elem_value)? {
            Ok(true)
        } else if self.read_standalone(elem_id, elem_value)? {
            Ok(true)
        } else if self.read_midi_sender(elem_id, elem_value)? {
            Ok(true)
        } else {
            match elem_id.name().as_str() {
                OUT_01_SRC_NAME => {
                    let params = &self.0.data;
                    let pos = PHYS_OUT_SRCS
                        .iter()
                        .position(|s| params.out_01_src.eq(s))
                        .unwrap();
                    elem_value.set_enum(&[pos as u32]);
                    Ok(true)
                }
                OUT_23_SRC_NAME => {
                    let params = &self.0.data;
                    let pos = PHYS_OUT_SRCS
                        .iter()
                        .position(|s| params.out_23_src.eq(s))
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
        if self.write_mixer_stream_src(req, node, elem_id, elem_value, timeout_ms)? {
            Ok(true)
        } else if self.write_coax_out_src(req, node, elem_id, elem_value, timeout_ms)? {
            Ok(true)
        } else if self.write_opt_iface_config(req, node, elem_id, elem_value, timeout_ms)? {
            Ok(true)
        } else if self.write_standalone(req, node, elem_id, elem_value, timeout_ms)? {
            Ok(true)
        } else if self.write_midi_sender(req, node, elem_id, elem_value, timeout_ms)? {
            Ok(true)
        } else {
            match elem_id.name().as_str() {
                OUT_01_SRC_NAME => {
                    let mut params = self.0.data.clone();
                    let pos = elem_value.enumerated()[0] as usize;
                    PHYS_OUT_SRCS
                        .iter()
                        .nth(pos)
                        .ok_or_else(|| {
                            let msg = format!("Invalid index of output source: {}", pos);
                            Error::new(FileError::Inval, &msg)
                        })
                        .map(|&s| params.out_01_src = s)?;
                    let res = KliveProtocol::update_partial_segment(
                        req,
                        node,
                        &params,
                        &mut self.0,
                        timeout_ms,
                    );
                    debug!(params = ?self.0.data, ?res);
                    res.map(|_| true)
                }
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
                    let res = KliveProtocol::update_partial_segment(
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
        if KliveProtocol::is_notified_segment(&self.0, msg) {
            let res = KliveProtocol::cache_whole_segment(req, node, &mut self.0, timeout_ms);
            debug!(params = ?self.0.data, ?res);
            res
        } else {
            Ok(())
        }
    }
}

#[derive(Default, Debug)]
struct MixerStateCtl(KliveMixerStateSegment, Vec<ElemId>);

impl ShellReverbReturnCtlOperation<KliveMixerState, KliveProtocol> for MixerStateCtl {
    fn segment(&self) -> &KliveMixerStateSegment {
        &self.0
    }

    fn segment_mut(&mut self) -> &mut KliveMixerStateSegment {
        &mut self.0
    }

    fn reverb_return(params: &KliveMixerState) -> &ShellReverbReturn {
        &params.reverb_return
    }

    fn reverb_return_mut(params: &mut KliveMixerState) -> &mut ShellReverbReturn {
        &mut params.reverb_return
    }
}

const MIXER_ENABLE_NAME: &str = "mixer-enable";
const USE_CH_STRIP_AS_PLUGIN_NAME: &str = "use-channel-strip-as-plugin";
const CH_STRIP_SRC_NAME: &str = "channel-strip-source";
const CH_STRIP_MODE_NAME: &str = "channel-strip-mode";
const USE_REVERB_AT_MID_RATE: &str = "use-reverb-at-mid-rate";

fn ch_strip_src_to_str(src: &ChStripSrc) -> &'static str {
    match src {
        ChStripSrc::Stream01 => "Stream-1/2",
        ChStripSrc::Analog01 => "Analog-1/2",
        ChStripSrc::Analog23 => "Analog-3/4",
        ChStripSrc::Digital01 => "Digital-1/2",
        ChStripSrc::Digital23 => "Digital-3/4",
        ChStripSrc::Digital45 => "Digital-5/6",
        ChStripSrc::Digital67 => "Digital-7/8",
        ChStripSrc::MixerOutput => "Mixer-out-1/2",
        ChStripSrc::None => "None",
    }
}

fn ch_strip_mode_to_str(mode: &ChStripMode) -> &'static str {
    match mode {
        ChStripMode::FabrikC => "FabricC",
        ChStripMode::RIAA1964 => "RIAA1964",
        ChStripMode::RIAA1987 => "RIAA1987",
    }
}
impl MixerStateCtl {
    const CH_STRIP_SRCS: [ChStripSrc; 9] = [
        ChStripSrc::Stream01,
        ChStripSrc::Analog01,
        ChStripSrc::Analog23,
        ChStripSrc::Digital01,
        ChStripSrc::Digital23,
        ChStripSrc::Digital45,
        ChStripSrc::Digital67,
        ChStripSrc::MixerOutput,
        ChStripSrc::None,
    ];
    const CH_STRIP_MODES: [ChStripMode; 3] = [
        ChStripMode::FabrikC,
        ChStripMode::RIAA1964,
        ChStripMode::RIAA1987,
    ];

    fn cache(&mut self, req: &FwReq, node: &FwNode, timeout_ms: u32) -> Result<(), Error> {
        let res = KliveProtocol::cache_whole_segment(req, node, &mut self.0, timeout_ms);
        debug!(params = ?self.0.data, ?res);
        res
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        load_mixer::<KliveProtocol, KliveMixerState>(&self.0, card_cntr)
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        self.load_reverb_return(card_cntr)
            .map(|mut notified_elem_id_list| self.1.append(&mut notified_elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIXER_ENABLE_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        let elem_id =
            ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, USE_CH_STRIP_AS_PLUGIN_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        let labels: Vec<&str> = Self::CH_STRIP_SRCS
            .iter()
            .map(|s| ch_strip_src_to_str(s))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, CH_STRIP_SRC_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let labels: Vec<&str> = Self::CH_STRIP_MODES
            .iter()
            .map(|s| ch_strip_mode_to_str(s))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, CH_STRIP_MODE_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, USE_REVERB_AT_MID_RATE, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        Ok(())
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        if read_mixer::<KliveProtocol, KliveMixerState>(&self.0, elem_id, elem_value)? {
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
                CH_STRIP_SRC_NAME => {
                    let pos = Self::CH_STRIP_SRCS
                        .iter()
                        .position(|s| self.0.data.ch_strip_src.eq(s))
                        .unwrap();
                    elem_value.set_enum(&[pos as u32]);
                    Ok(true)
                }
                CH_STRIP_MODE_NAME => {
                    let pos = Self::CH_STRIP_MODES
                        .iter()
                        .position(|s| self.0.data.ch_strip_mode.eq(s))
                        .unwrap();
                    elem_value.set_enum(&[pos as u32]);
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
        if write_mixer::<KliveProtocol, KliveMixerState>(
            &mut self.0,
            req,
            node,
            elem_id,
            elem_value,
            timeout_ms,
        )? {
            Ok(true)
        } else if self.write_reverb_return(req, node, elem_id, elem_value, timeout_ms)? {
            Ok(true)
        } else {
            match elem_id.name().as_str() {
                MIXER_ENABLE_NAME => {
                    let mut params = self.0.data.clone();
                    params.enabled = elem_value.boolean()[0];
                    let res = KliveProtocol::update_partial_segment(
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
                    let res = KliveProtocol::update_partial_segment(
                        req,
                        node,
                        &params,
                        &mut self.0,
                        timeout_ms,
                    );
                    debug!(params = ?self.0.data, ?res);
                    res.map(|_| true)
                }
                CH_STRIP_SRC_NAME => {
                    let mut params = self.0.data.clone();
                    let pos = elem_value.enumerated()[0] as usize;
                    Self::CH_STRIP_SRCS
                        .iter()
                        .nth(pos)
                        .ok_or_else(|| {
                            let msg = format!("Invalid value for index of ch strip src: {}", pos);
                            Error::new(FileError::Inval, &msg)
                        })
                        .map(|&s| params.ch_strip_src = s)?;
                    let res = KliveProtocol::update_partial_segment(
                        req,
                        node,
                        &params,
                        &mut self.0,
                        timeout_ms,
                    );
                    debug!(params = ?self.0.data, ?res);
                    res.map(|_| true)
                }
                CH_STRIP_MODE_NAME => {
                    let mut params = self.0.data.clone();
                    let pos = elem_value.enumerated()[0] as usize;
                    Self::CH_STRIP_MODES
                        .iter()
                        .nth(pos)
                        .ok_or_else(|| {
                            let msg = format!("Invalid value for index of ch strip mode: {}", pos);
                            Error::new(FileError::Inval, &msg)
                        })
                        .map(|&m| params.ch_strip_mode = m)?;
                    let res = KliveProtocol::update_partial_segment(
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
                    let res = KliveProtocol::update_partial_segment(
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
        if KliveProtocol::is_notified_segment(&self.0, msg) {
            let res = KliveProtocol::cache_whole_segment(req, node, &mut self.0, timeout_ms);
            debug!(params = ?self.0.data, ?res);
            res
        } else {
            Ok(())
        }
    }
}

#[derive(Default, Debug)]
struct HwStateCtl(KliveHwStateSegment, Vec<ElemId>);

impl HwStateCtl {
    fn cache(&mut self, req: &FwReq, node: &FwNode, timeout_ms: u32) -> Result<(), Error> {
        let res = KliveProtocol::cache_whole_segment(req, node, &mut self.0, timeout_ms);
        debug!(params = ?self.0.data, ?res);
        res
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        load_hw_state::<KliveProtocol, KliveHwState>(card_cntr)
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        Ok(())
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        read_hw_state::<KliveProtocol, KliveHwState>(&self.0, elem_id, elem_value)
    }

    fn write(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        write_hw_state::<KliveProtocol, KliveHwState>(
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
        if KliveProtocol::is_notified_segment(&self.0, msg) {
            let res = KliveProtocol::cache_whole_segment(req, node, &mut self.0, timeout_ms);
            debug!(params = ?self.0.data, ?res);
            res
        } else {
            Ok(())
        }
    }
}
