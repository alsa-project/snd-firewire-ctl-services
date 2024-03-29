// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use {
    super::{shell_ctl::*, *},
    protocols::tcelectronic::shell::itwin::*,
};

#[derive(Default, Debug)]
pub struct ItwinModel {
    req: FwReq,
    sections: GeneralSections,
    common_ctl: CommonCtl<ItwinProtocol>,
    knob_ctl: KnobCtl,
    config_ctl: ConfigCtl,
    mixer_state_ctl: MixerStateCtl,
    mixer_meter_ctl: MixerMeterCtl<ItwinProtocol, ItwinMixerMeter>,
    hw_state_ctl: HwStateCtl,
    reverb_state_ctl: ReverbStateCtl<ItwinProtocol, ItwinReverbState>,
    reverb_meter_ctl: ReverbMeterCtl<ItwinProtocol, ItwinReverbMeter>,
    ch_strip_state_ctl: ChStripStateCtl<ItwinProtocol, ItwinChStripStates>,
    ch_strip_meter_ctl: ChStripMeterCtl<ItwinProtocol, ItwinChStripMeters>,
}

const TIMEOUT_MS: u32 = 20;

impl CtlModel<(SndDice, FwNode)> for ItwinModel {
    fn cache(&mut self, (_, node): &mut (SndDice, FwNode)) -> Result<(), Error> {
        ItwinProtocol::read_general_sections(&mut self.req, node, &mut self.sections, TIMEOUT_MS)?;

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
            &self.req,
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

impl MeasureModel<(SndDice, FwNode)> for ItwinModel {
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
struct KnobCtl(ItwinKnobSegment, Vec<ElemId>);

const CLK_RECOVERY_NAME: &str = "clock-recovery";

impl KnobCtl {
    fn cache(&mut self, req: &FwReq, node: &FwNode, timeout_ms: u32) -> Result<(), Error> {
        let res = ItwinProtocol::cache_whole_segment(req, node, &mut self.0, timeout_ms);
        debug!(params = ?self.0.data, ?res);
        res
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        load_knob0_target::<ItwinProtocol, ItwinKnob>(card_cntr)
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, CLK_RECOVERY_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        Ok(())
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            CLK_RECOVERY_NAME => {
                let params = &self.0.data;
                elem_value.set_bool(&[params.clock_recovery]);
                Ok(true)
            }
            _ => read_knob0_target::<ItwinProtocol, ItwinKnob>(&self.0, elem_id, elem_value),
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
            _ => write_knob0_target::<ItwinProtocol, ItwinKnob>(
                &mut self.0,
                req,
                node,
                elem_id,
                elem_value,
                timeout_ms,
            ),
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
        load_mixer_stream_src::<ItwinProtocol, ItwinConfig>(card_cntr)?;
        load_standalone::<ItwinProtocol, ItwinConfig>(card_cntr)?;

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
        if read_mixer_stream_src::<ItwinProtocol, ItwinConfig>(&self.0, elem_id, elem_value)? {
            Ok(true)
        } else if read_standalone::<ItwinProtocol, ItwinConfig>(&self.0, elem_id, elem_value)? {
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
        if write_mixer_stream_src::<ItwinProtocol, ItwinConfig>(
            &mut self.0,
            req,
            node,
            elem_id,
            elem_value,
            timeout_ms,
        )? {
            Ok(true)
        } else if write_standalone::<ItwinProtocol, ItwinConfig>(
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

const MIXER_ENABLE_NAME: &str = "mixer-enable";

impl MixerStateCtl {
    fn cache(&mut self, req: &FwReq, node: &FwNode, timeout_ms: u32) -> Result<(), Error> {
        let res = ItwinProtocol::cache_whole_segment(req, node, &mut self.0, timeout_ms);
        debug!(params = ?self.0.data, ?res);
        res
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        load_mixer::<ItwinProtocol, ItwinMixerState>(&self.0, card_cntr)
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIXER_ENABLE_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        Ok(())
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        if read_mixer::<ItwinProtocol, ItwinMixerState>(&self.0, elem_id, elem_value)? {
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
        if write_mixer::<ItwinProtocol, ItwinMixerState>(
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
struct HwStateCtl(ItwinHwStateSegment, Vec<ElemId>);

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
        load_hw_state::<ItwinProtocol, ItwinHwState>(card_cntr)
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

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
            _ => read_hw_state::<ItwinProtocol, ItwinHwState>(&self.0, elem_id, elem_value),
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
            _ => write_hw_state::<ItwinProtocol, ItwinHwState>(
                &mut self.0,
                req,
                node,
                elem_id,
                elem_value,
                timeout_ms,
            ),
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
