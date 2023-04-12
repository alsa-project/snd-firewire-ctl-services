// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use {
    super::{shell_ctl::*, *},
    protocols::tcelectronic::shell::{itwin::*, *},
};

#[derive(Default, Debug)]
pub struct ItwinModel {
    req: FwReq,
    sections: GeneralSections,
    common_ctl: CommonCtl<ItwinProtocol>,
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

impl CtlModel<(SndDice, FwNode)> for ItwinModel {
    fn cache(&mut self, unit: &mut (SndDice, FwNode)) -> Result<(), Error> {
        ItwinProtocol::read_general_sections(&self.req, &unit.1, &mut self.sections, TIMEOUT_MS)?;

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

impl NotifyModel<(SndDice, FwNode), u32> for ItwinModel {
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

impl MeasureModel<(SndDice, FwNode)> for ItwinModel {
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
struct KnobCtl(ItwinKnobSegment, Vec<ElemId>);

impl ShellKnob0CtlOperation<ItwinKnob, ItwinProtocol> for KnobCtl {
    fn segment(&self) -> &ItwinKnobSegment {
        &self.0
    }

    fn segment_mut(&mut self) -> &mut ItwinKnobSegment {
        &mut self.0
    }

    fn knob0_target(params: &ItwinKnob) -> &ShellKnob0Target {
        &params.target
    }

    fn knob0_target_mut(params: &mut ItwinKnob) -> &mut ShellKnob0Target {
        &mut params.target
    }
}

const CLK_RECOVERY_NAME: &str = "clock-recovery";

impl KnobCtl {
    fn cache(&mut self, req: &FwReq, node: &FwNode, timeout_ms: u32) -> Result<(), Error> {
        let res = ItwinProtocol::cache_whole_segment(req, node, &mut self.0, timeout_ms);
        debug!(params = ?self.0.data, ?res);
        res
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.load_knob0_target(card_cntr)
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, CLK_RECOVERY_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        Ok(())
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        if self.read_knob0_target(elem_id, elem_value)? {
            Ok(true)
        } else {
            match elem_id.name().as_str() {
                CLK_RECOVERY_NAME => {
                    let params = &self.0.data;
                    elem_value.set_bool(&[params.clock_recovery]);
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
        } else {
            match elem_id.name().as_str() {
                CLK_RECOVERY_NAME => {
                    let mut params = self.0.data.clone();
                    params.clock_recovery = elem_value.boolean()[0];
                    let res = ItwinProtocol::update_partial_segment(
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
        if ItwinProtocol::is_notified_segment(&self.0, msg) {
            let res = ItwinProtocol::cache_whole_segment(req, node, &mut self.0, timeout_ms);
            debug!(params = ?self.0.data, ?res);
            res
        } else {
            Ok(())
        }
    }
}

#[derive(Default, Debug)]
struct ConfigCtl(ItwinConfigSegment, Vec<ElemId>);

impl ShellMixerStreamSrcCtlOperation<ItwinConfig, ItwinProtocol> for ConfigCtl {
    fn segment(&self) -> &ItwinConfigSegment {
        &self.0
    }

    fn segment_mut(&mut self) -> &mut ItwinConfigSegment {
        &mut self.0
    }

    fn mixer_stream_src(params: &ItwinConfig) -> &ShellMixerStreamSourcePair {
        &params.mixer_stream_src_pair
    }

    fn mixer_stream_src_mut(params: &mut ItwinConfig) -> &mut ShellMixerStreamSourcePair {
        &mut params.mixer_stream_src_pair
    }
}

impl StandaloneCtlOperation<ItwinConfig, ItwinProtocol> for ConfigCtl {
    fn segment(&self) -> &ItwinConfigSegment {
        &self.0
    }

    fn segment_mut(&mut self) -> &mut ItwinConfigSegment {
        &mut self.0
    }

    fn standalone_rate(params: &ItwinConfig) -> &TcKonnektStandaloneClockRate {
        &params.standalone_rate
    }

    fn standalone_rate_mut(params: &mut ItwinConfig) -> &mut TcKonnektStandaloneClockRate {
        &mut params.standalone_rate
    }
}

impl ShellStandaloneCtlOperation<ItwinConfig, ItwinProtocol> for ConfigCtl {
    fn standalone_src(params: &ItwinConfig) -> &ShellStandaloneClockSource {
        &params.standalone_src
    }

    fn standalone_src_mut(params: &mut ItwinConfig) -> &mut ShellStandaloneClockSource {
        &mut params.standalone_src
    }
}

const OUT_SRC_NAME: &str = "output-source";

fn itwin_phys_out_src_to_string(src: &ItwinOutputPairSrc) -> &'static str {
    match src {
        ItwinOutputPairSrc::MixerOut01 => "Mixer-out-1/2",
        ItwinOutputPairSrc::Analog01 => "Analog-1/2",
        ItwinOutputPairSrc::Analog23 => "Analog-3/4",
        ItwinOutputPairSrc::Spdif01 => "S/PDIF-1/2",
        ItwinOutputPairSrc::Adat01 => "ADAT-1/2",
        ItwinOutputPairSrc::Adat23 => "ADAT-3/4",
        ItwinOutputPairSrc::Adat45 => "ADAT-5/6",
        ItwinOutputPairSrc::Adat67 => "ADAT-7/8",
        ItwinOutputPairSrc::Stream01 => "Stream-1/2",
        ItwinOutputPairSrc::Stream23 => "Stream-3/4",
        ItwinOutputPairSrc::Stream45 => "Stream-5/6",
        ItwinOutputPairSrc::Stream67 => "Stream-7/8",
        ItwinOutputPairSrc::Stream89 => "Stream-9/10",
        ItwinOutputPairSrc::Stream1011 => "Stream-11/12",
        ItwinOutputPairSrc::Stream1213 => "Stream-13/14",
        ItwinOutputPairSrc::MixerSend01 => "Mixer-send-1/2",
    }
}

impl ConfigCtl {
    const OUT_SRCS: [ItwinOutputPairSrc; 16] = [
        ItwinOutputPairSrc::MixerOut01,
        ItwinOutputPairSrc::Analog01,
        ItwinOutputPairSrc::Analog23,
        ItwinOutputPairSrc::Spdif01,
        ItwinOutputPairSrc::Adat01,
        ItwinOutputPairSrc::Adat23,
        ItwinOutputPairSrc::Adat45,
        ItwinOutputPairSrc::Adat67,
        ItwinOutputPairSrc::Stream01,
        ItwinOutputPairSrc::Stream23,
        ItwinOutputPairSrc::Stream45,
        ItwinOutputPairSrc::Stream67,
        ItwinOutputPairSrc::Stream89,
        ItwinOutputPairSrc::Stream1011,
        ItwinOutputPairSrc::Stream1213,
        ItwinOutputPairSrc::MixerSend01,
    ];

    fn cache(&mut self, req: &FwReq, node: &FwNode, timeout_ms: u32) -> Result<(), Error> {
        let res = ItwinProtocol::cache_whole_segment(req, node, &mut self.0, timeout_ms);
        debug!(params = ?self.0.data, ?res);
        res
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.load_mixer_stream_src(card_cntr)?;
        self.load_standalone(card_cntr)?;

        let labels: Vec<&str> = Self::OUT_SRCS
            .iter()
            .map(|s| itwin_phys_out_src_to_string(s))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, OUT_SRC_NAME, 0);
        let _ = card_cntr.add_enum_elems(
            &elem_id,
            1,
            ITWIN_PHYS_OUT_PAIR_COUNT,
            &labels,
            None,
            true,
        )?;

        Ok(())
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        if self.read_mixer_stream_src(elem_id, elem_value)? {
            Ok(true)
        } else if self.read_standalone(elem_id, elem_value)? {
            Ok(true)
        } else {
            match elem_id.name().as_str() {
                OUT_SRC_NAME => {
                    let params = &self.0.data;
                    let vals: Vec<u32> = params
                        .output_pair_src
                        .iter()
                        .map(|src| {
                            let pos = Self::OUT_SRCS.iter().position(|s| src.eq(s)).unwrap();
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
        if self.write_mixer_stream_src(req, node, elem_id, elem_value, timeout_ms)? {
            Ok(true)
        } else if self.write_standalone(req, node, elem_id, elem_value, timeout_ms)? {
            Ok(true)
        } else {
            match elem_id.name().as_str() {
                OUT_SRC_NAME => {
                    let mut params = self.0.data.clone();
                    params
                        .output_pair_src
                        .iter_mut()
                        .zip(elem_value.enumerated())
                        .try_for_each(|(src, &val)| {
                            let pos = val as usize;
                            Self::OUT_SRCS
                                .iter()
                                .nth(pos)
                                .ok_or_else(|| {
                                    let msg = format!("Invalid index of output source: {}", pos);
                                    Error::new(FileError::Inval, &msg)
                                })
                                .map(|&s| *src = s)
                        })?;
                    let res = ItwinProtocol::update_partial_segment(
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
        if ItwinProtocol::is_notified_segment(&self.0, msg) {
            let res = ItwinProtocol::cache_whole_segment(req, node, &mut self.0, timeout_ms);
            debug!(params = ?self.0.data, ?res);
            res
        } else {
            Ok(())
        }
    }
}

#[derive(Default, Debug)]
struct MixerStateCtl(ItwinMixerStateSegment, Vec<ElemId>);

impl ShellMixerStateCtlOperation<ItwinMixerState, ItwinMixerMeter, ItwinProtocol>
    for MixerStateCtl
{
    fn segment(&self) -> &ItwinMixerStateSegment {
        &self.0
    }

    fn segment_mut(&mut self) -> &mut ItwinMixerStateSegment {
        &mut self.0
    }

    fn state(params: &ItwinMixerState) -> &ShellMixerState {
        &params.mixer
    }

    fn state_mut(params: &mut ItwinMixerState) -> &mut ShellMixerState {
        &mut params.mixer
    }

    fn enabled(&self) -> bool {
        self.0.data.enabled
    }
}

const MIXER_ENABLE_NAME: &str = "mixer-enable";

impl MixerStateCtl {
    fn cache(&mut self, req: &FwReq, node: &FwNode, timeout_ms: u32) -> Result<(), Error> {
        let res = ItwinProtocol::cache_whole_segment(req, node, &mut self.0, timeout_ms);
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
                    let res = ItwinProtocol::update_partial_segment(
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
        if ItwinProtocol::is_notified_segment(&self.0, msg) {
            let res = ItwinProtocol::cache_whole_segment(req, node, &mut self.0, timeout_ms);
            debug!(params = ?self.0.data, ?res);
            res
        } else {
            Ok(())
        }
    }
}

#[derive(Default, Debug)]
struct MixerMeterCtl(ItwinMixerMeterSegment, Vec<ElemId>);

impl ShellMixerMeterCtlOperation<ItwinMixerMeter, ItwinProtocol> for MixerMeterCtl {
    fn meter(&self) -> &ShellMixerMeter {
        &self.0.data.0
    }

    fn segment(&self) -> &TcKonnektSegment<ItwinMixerMeter> {
        &self.0
    }

    fn segment_mut(&mut self) -> &mut TcKonnektSegment<ItwinMixerMeter> {
        &mut self.0
    }
}

#[derive(Default, Debug)]
struct HwStateCtl(ItwinHwStateSegment, Vec<ElemId>);

impl FirewireLedCtlOperation<ItwinHwState, ItwinProtocol> for HwStateCtl {
    fn segment(&self) -> &ItwinHwStateSegment {
        &self.0
    }

    fn segment_mut(&mut self) -> &mut ItwinHwStateSegment {
        &mut self.0
    }

    fn firewire_led(params: &ItwinHwState) -> &FireWireLedState {
        &params.hw_state.firewire_led
    }

    fn firewire_led_mut(params: &mut ItwinHwState) -> &mut FireWireLedState {
        &mut params.hw_state.firewire_led
    }
}

impl ShellHwStateCtlOperation<ItwinHwState, ItwinProtocol> for HwStateCtl {
    fn hw_state(&self) -> &ShellHwState {
        &self.0.data.hw_state
    }

    fn hw_state_mut(&mut self) -> &mut ShellHwState {
        &mut self.0.data.hw_state
    }
}

const LISTENING_MODE_NAME: &str = "listening-mode";

fn listening_mode_to_str(mode: &ListeningMode) -> &'static str {
    match mode {
        ListeningMode::Monaural => "Monaural",
        ListeningMode::Stereo => "Stereo",
        ListeningMode::Side => "Side",
    }
}

impl HwStateCtl {
    const LISTENING_MODES: [ListeningMode; 3] = [
        ListeningMode::Monaural,
        ListeningMode::Stereo,
        ListeningMode::Side,
    ];

    fn cache(&mut self, req: &FwReq, node: &FwNode, timeout_ms: u32) -> Result<(), Error> {
        let res = ItwinProtocol::cache_whole_segment(req, node, &mut self.0, timeout_ms);
        debug!(params = ?self.0.data, ?res);
        res
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.load_hw_state(card_cntr)
            .map(|mut notified_elem_id_list| self.1.append(&mut notified_elem_id_list))?;

        let labels: Vec<&str> = Self::LISTENING_MODES
            .iter()
            .map(|m| listening_mode_to_str(m))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, LISTENING_MODE_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, true)
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        Ok(())
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        if self.read_hw_state(elem_id, elem_value)? {
            Ok(true)
        } else {
            match elem_id.name().as_str() {
                LISTENING_MODE_NAME => {
                    let params = &self.0.data;
                    let pos = Self::LISTENING_MODES
                        .iter()
                        .position(|m| params.listening_mode.eq(m))
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
        if self.write_hw_state(req, node, elem_id, elem_value, timeout_ms)? {
            Ok(true)
        } else {
            match elem_id.name().as_str() {
                LISTENING_MODE_NAME => {
                    let mut params = self.0.data.clone();
                    let pos = elem_value.enumerated()[0] as usize;
                    Self::LISTENING_MODES
                        .iter()
                        .nth(pos)
                        .ok_or_else(|| {
                            let msg = format!("Invalid index of listening mode: {}", pos);
                            Error::new(FileError::Inval, &msg)
                        })
                        .map(|&m| params.listening_mode = m)?;
                    let res = ItwinProtocol::update_partial_segment(
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
        if ItwinProtocol::is_notified_segment(&self.0, msg) {
            let res = ItwinProtocol::cache_whole_segment(req, node, &mut self.0, timeout_ms);
            debug!(params = ?self.0.data, ?res);
            res
        } else {
            Ok(())
        }
    }
}

#[derive(Default, Debug)]
struct ReverbStateCtl(ItwinReverbStateSegment, Vec<ElemId>);

impl ReverbStateCtlOpreation<ItwinReverbState, ItwinReverbMeter, ItwinProtocol> for ReverbStateCtl {
    fn segment(&self) -> &ItwinReverbStateSegment {
        &self.0
    }

    fn segment_mut(&mut self) -> &mut ItwinReverbStateSegment {
        &mut self.0
    }

    fn state(params: &ItwinReverbState) -> &ReverbState {
        &params.0
    }

    fn state_mut(params: &mut ItwinReverbState) -> &mut ReverbState {
        &mut params.0
    }
}

#[derive(Default, Debug)]
struct ReverbMeterCtl(ItwinReverbMeterSegment, Vec<ElemId>);

impl ReverbMeterCtlOperation<ItwinReverbMeter, ItwinProtocol> for ReverbMeterCtl {
    fn meter(&self) -> &ReverbMeter {
        &self.0.data.0
    }

    fn segment(&self) -> &TcKonnektSegment<ItwinReverbMeter> {
        &self.0
    }

    fn segment_mut(&mut self) -> &mut TcKonnektSegment<ItwinReverbMeter> {
        &mut self.0
    }
}

#[derive(Default, Debug)]
struct ChStripStateCtl(ItwinChStripStatesSegment, Vec<ElemId>);

impl ChStripStateCtlOperation<ItwinChStripStates, ItwinProtocol> for ChStripStateCtl {
    fn segment(&self) -> &ItwinChStripStatesSegment {
        &self.0
    }

    fn segment_mut(&mut self) -> &mut ItwinChStripStatesSegment {
        &mut self.0
    }

    fn states(params: &ItwinChStripStates) -> &[ChStripState] {
        &params.0
    }

    fn states_mut(params: &mut ItwinChStripStates) -> &mut [ChStripState] {
        &mut params.0
    }
}

#[derive(Default, Debug)]
struct ChStripMeterCtl(ItwinChStripMetersSegment, Vec<ElemId>);

impl ChStripMeterCtlOperation<ItwinChStripMeters, ItwinProtocol> for ChStripMeterCtl {
    fn meters(&self) -> &[ChStripMeter] {
        &self.0.data.0
    }

    fn segment(&self) -> &TcKonnektSegment<ItwinChStripMeters> {
        &self.0
    }

    fn segment_mut(&mut self) -> &mut TcKonnektSegment<ItwinChStripMeters> {
        &mut self.0
    }
}
