// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use {
    super::{shell_ctl::*, *},
    protocols::tcelectronic::shell::{klive::*, *},
};

#[derive(Default)]
pub struct KliveModel {
    req: FwReq,
    sections: GeneralSections,
    common_ctl: CommonCtl,
    knob_ctl: KnobCtl,
    config_ctl: ConfigCtl,
    mixer_ctl: MixerCtl,
    hw_state_ctl: HwStateCtl,
    reverb_ctl: ReverbCtl,
    ch_strip_ctl: ChStripCtl,
}

const TIMEOUT_MS: u32 = 20;

impl KliveModel {
    pub fn cache(&mut self, unit: &mut (SndDice, FwNode)) -> Result<(), Error> {
        KliveProtocol::read_general_sections(&self.req, &unit.1, &mut self.sections, TIMEOUT_MS)?;

        self.common_ctl
            .whole_cache(&self.req, &unit.1, &mut self.sections, TIMEOUT_MS)?;

        self.ch_strip_ctl.cache(&self.req, &unit.1, TIMEOUT_MS)?;
        self.reverb_ctl.cache(&self.req, &unit.1, TIMEOUT_MS)?;

        Ok(())
    }
}

impl CtlModel<(SndDice, FwNode)> for KliveModel {
    fn load(
        &mut self,
        unit: &mut (SndDice, FwNode),
        card_cntr: &mut CardCntr,
    ) -> Result<(), Error> {
        self.common_ctl.load(card_cntr, &self.sections).map(
            |(measured_elem_id_list, notified_elem_id_list)| {
                self.common_ctl.0 = measured_elem_id_list;
                self.common_ctl.1 = notified_elem_id_list;
            },
        )?;

        self.knob_ctl
            .load(card_cntr, unit, &mut self.req, TIMEOUT_MS)?;
        self.config_ctl
            .load(card_cntr, unit, &mut self.req, TIMEOUT_MS)?;
        self.mixer_ctl
            .load(card_cntr, unit, &mut self.req, TIMEOUT_MS)?;
        self.hw_state_ctl
            .load(card_cntr, unit, &mut self.req, TIMEOUT_MS)?;
        self.reverb_ctl
            .load(card_cntr)
            .map(|(notified_elem_id_list, measured_elem_id_list)| {
                self.reverb_ctl.2 = notified_elem_id_list;
                self.reverb_ctl.3 = measured_elem_id_list;
            })?;
        self.ch_strip_ctl.load(card_cntr).map(
            |(notified_elem_id_list, measured_elem_id_list)| {
                self.ch_strip_ctl.2 = notified_elem_id_list;
                self.ch_strip_ctl.3 = measured_elem_id_list;
            },
        )?;

        Ok(())
    }

    fn read(
        &mut self,
        _: &mut (SndDice, FwNode),
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.common_ctl.read(&self.sections, elem_id, elem_value)? {
            Ok(true)
        } else if self.knob_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.config_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.hw_state_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.reverb_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.ch_strip_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(
        &mut self,
        unit: &mut (SndDice, FwNode),
        elem_id: &ElemId,
        old: &ElemValue,
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
            .write(unit, &mut self.req, elem_id, old, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .config_ctl
            .write(unit, &mut self.req, elem_id, old, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .mixer_ctl
            .write(unit, &mut self.req, elem_id, old, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .reverb_ctl
            .write(&self.req, &unit.1, elem_id, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .ch_strip_ctl
            .write(&self.req, &unit.1, elem_id, old, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .hw_state_ctl
            .write(&self.req, &unit.1, elem_id, old, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl NotifyModel<(SndDice, FwNode), u32> for KliveModel {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.common_ctl.1);
        elem_id_list.extend_from_slice(&self.knob_ctl.1);
        elem_id_list.extend_from_slice(&self.config_ctl.1);
        elem_id_list.extend_from_slice(&self.mixer_ctl.2);
        elem_id_list.extend_from_slice(&self.hw_state_ctl.1);
        elem_id_list.extend_from_slice(&self.reverb_ctl.2);
        elem_id_list.extend_from_slice(&self.ch_strip_ctl.2);
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
            .parse_notification(unit, &mut self.req, msg, TIMEOUT_MS)?;
        self.config_ctl
            .parse_notification(unit, &mut self.req, msg, TIMEOUT_MS)?;
        self.mixer_ctl
            .parse_notification(unit, &mut self.req, msg, TIMEOUT_MS)?;
        self.hw_state_ctl
            .parse_notification(unit, &mut self.req, msg, TIMEOUT_MS)?;
        self.reverb_ctl
            .parse_notification(&self.req, &unit.1, msg, TIMEOUT_MS)?;
        self.ch_strip_ctl
            .parse_notification(&self.req, &unit.1, msg, TIMEOUT_MS)?;
        Ok(())
    }

    fn read_notified_elem(
        &mut self,
        _: &(SndDice, FwNode),
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.common_ctl.read(&self.sections, elem_id, elem_value)? {
            Ok(true)
        } else if self.knob_ctl.read_notified_elem(elem_id, elem_value)? {
            Ok(true)
        } else if self.config_ctl.read_notified_elem(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_ctl.read_notified_elem(elem_id, elem_value)? {
            Ok(true)
        } else if self.hw_state_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.reverb_ctl.read_notified_elem(elem_id, elem_value)? {
            Ok(true)
        } else if self.ch_strip_ctl.read_notified_elem(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl MeasureModel<(SndDice, FwNode)> for KliveModel {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.common_ctl.0);
        elem_id_list.extend_from_slice(&self.mixer_ctl.3);
        elem_id_list.extend_from_slice(&self.reverb_ctl.3);
        elem_id_list.extend_from_slice(&self.ch_strip_ctl.3);
    }

    fn measure_states(&mut self, unit: &mut (SndDice, FwNode)) -> Result<(), Error> {
        self.common_ctl
            .measure(&self.req, &unit.1, &mut self.sections, TIMEOUT_MS)?;
        self.mixer_ctl
            .measure_states(unit, &mut self.req, TIMEOUT_MS)?;
        self.reverb_ctl
            .measure_states(&self.req, &unit.1, TIMEOUT_MS)?;
        self.ch_strip_ctl
            .measure_states(&self.req, &unit.1, TIMEOUT_MS)?;
        Ok(())
    }

    fn measure_elem(
        &mut self,
        _: &(SndDice, FwNode),
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.common_ctl.read(&self.sections, elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_ctl.read_measured_elem(elem_id, elem_value)? {
            Ok(true)
        } else if self.reverb_ctl.read_measured_elem(elem_id, elem_value)? {
            Ok(true)
        } else if self.ch_strip_ctl.read_measured_elem(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[derive(Default, Debug)]
struct CommonCtl(Vec<ElemId>, Vec<ElemId>);

impl CommonCtlOperation<KliveProtocol> for CommonCtl {}

#[derive(Default)]
struct KnobCtl(KliveKnobSegment, Vec<ElemId>);

impl ShellKnobCtlOperation<KliveKnob, KliveProtocol> for KnobCtl {
    const TARGETS: [&'static str; 4] = ["Analog-1", "Analog-2", "Analog-3/4", "Configurable"];

    fn segment_mut(&mut self) -> &mut KliveKnobSegment {
        &mut self.0
    }

    fn knob_target(&self) -> &ShellKnobTarget {
        &self.0.data.target
    }

    fn knob_target_mut(&mut self) -> &mut ShellKnobTarget {
        &mut self.0.data.target
    }
}

impl ShellKnob2CtlOperation<KliveKnob, KliveProtocol> for KnobCtl {
    const TARGETS: &'static [&'static str] = &[
        "Digital-1/2",
        "Digital-3/4",
        "Digital-5/6",
        "Digital-7/8",
        "Stream",
        "Reverb-1/2",
        "Mixer-1/2",
        "Tune-pitch-tone",
        "Midi-send",
    ];

    fn segment_mut(&mut self) -> &mut KliveKnobSegment {
        &mut self.0
    }

    fn knob2_target(&self) -> &ShellKnob2Target {
        &self.0.data.knob2_target
    }

    fn knob2_target_mut(&mut self) -> &mut ShellKnob2Target {
        &mut self.0.data.knob2_target
    }
}

impl ProgramCtlOperation<KliveKnob, KliveProtocol> for KnobCtl {
    fn segment_mut(&mut self) -> &mut KliveKnobSegment {
        &mut self.0
    }

    fn prog(&self) -> &TcKonnektLoadedProgram {
        &self.0.data.prog
    }

    fn prog_mut(&mut self) -> &mut TcKonnektLoadedProgram {
        &mut self.0.data.prog
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

    fn load(
        &mut self,
        card_cntr: &mut CardCntr,
        unit: &mut (SndDice, FwNode),
        req: &mut FwReq,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        KliveProtocol::read_segment(req, &mut unit.1, &mut self.0, timeout_ms)?;

        self.load_knob_target(card_cntr)
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;
        self.load_knob2_target(card_cntr)
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
        if self.read_knob_target(elem_id, elem_value)? {
            Ok(true)
        } else if self.read_knob2_target(elem_id, elem_value)? {
            Ok(true)
        } else if self.read_prog(elem_id, elem_value)? {
            Ok(true)
        } else {
            match elem_id.name().as_str() {
                OUTPUT_IMPEDANCE_NAME => ElemValueAccessor::<u32>::set_vals(elem_value, 2, |idx| {
                    let pos = Self::OUTPUT_IMPEDANCES
                        .iter()
                        .position(|&i| i == self.0.data.out_impedance[idx])
                        .unwrap();
                    Ok(pos as u32)
                })
                .map(|_| true),
                _ => Ok(false),
            }
        }
    }

    fn write(
        &mut self,
        unit: &mut (SndDice, FwNode),
        req: &mut FwReq,
        elem_id: &ElemId,
        old: &ElemValue,
        new: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        if self.write_knob_target(unit, req, elem_id, new, timeout_ms)? {
            Ok(true)
        } else if self.write_knob2_target(unit, req, elem_id, new, timeout_ms)? {
            Ok(true)
        } else if self.write_prog(unit, req, elem_id, new, timeout_ms)? {
            Ok(true)
        } else {
            match elem_id.name().as_str() {
                OUTPUT_IMPEDANCE_NAME => {
                    ElemValueAccessor::<u32>::get_vals(new, old, 2, |idx, val| {
                        Self::OUTPUT_IMPEDANCES
                            .iter()
                            .nth(val as usize)
                            .ok_or_else(|| {
                                let msg = format!("Invalid index of output impedance: {}", val);
                                Error::new(FileError::Inval, &msg)
                            })
                            .map(|&i| self.0.data.out_impedance[idx] = i)
                    })?;
                    KliveProtocol::write_segment(req, &mut unit.1, &mut self.0, timeout_ms)
                        .map(|_| true)
                }
                _ => Ok(false),
            }
        }
    }

    fn parse_notification(
        &mut self,
        unit: &mut (SndDice, FwNode),
        req: &mut FwReq,
        msg: u32,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        if self.0.has_segment_change(msg) {
            KliveProtocol::read_segment(req, &mut unit.1, &mut self.0, timeout_ms)
        } else {
            Ok(())
        }
    }

    fn read_notified_elem(
        &mut self,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.read_knob_target(elem_id, elem_value)? {
            Ok(true)
        } else if self.read_knob2_target(elem_id, elem_value)? {
            Ok(true)
        } else if self.read_prog(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[derive(Default)]
struct ConfigCtl(KliveConfigSegment, Vec<ElemId>);

impl ShellMixerStreamSrcCtlOperation<KliveConfig, KliveProtocol> for ConfigCtl {
    fn segment_mut(&mut self) -> &mut KliveConfigSegment {
        &mut self.0
    }

    fn mixer_stream_src(&self) -> &ShellMixerStreamSrcPair {
        &self.0.data.mixer_stream_src_pair
    }

    fn mixer_stream_src_mut(&mut self) -> &mut ShellMixerStreamSrcPair {
        &mut self.0.data.mixer_stream_src_pair
    }
}

impl ShellCoaxIfaceCtlOperation<KliveConfig, KliveProtocol> for ConfigCtl {
    fn segment_mut(&mut self) -> &mut KliveConfigSegment {
        &mut self.0
    }

    fn coax_out_src(&self) -> &ShellCoaxOutPairSrc {
        &self.0.data.coax_out_src
    }

    fn coax_out_src_mut(&mut self) -> &mut ShellCoaxOutPairSrc {
        &mut self.0.data.coax_out_src
    }
}

impl ShellOptIfaceCtl<KliveConfig, KliveProtocol> for ConfigCtl {
    fn segment_mut(&mut self) -> &mut KliveConfigSegment {
        &mut self.0
    }

    fn opt_iface_config(&self) -> &ShellOptIfaceConfig {
        &self.0.data.opt
    }

    fn opt_iface_config_mut(&mut self) -> &mut ShellOptIfaceConfig {
        &mut self.0.data.opt
    }
}

impl StandaloneCtlOperation<KliveConfig, KliveProtocol> for ConfigCtl {
    fn segment_mut(&mut self) -> &mut KliveConfigSegment {
        &mut self.0
    }

    fn standalone_rate(&self) -> &TcKonnektStandaloneClkRate {
        &self.0.data.standalone_rate
    }

    fn standalone_rate_mut(&mut self) -> &mut TcKonnektStandaloneClkRate {
        &mut self.0.data.standalone_rate
    }
}

impl ShellStandaloneCtlOperation<KliveConfig, KliveProtocol> for ConfigCtl {
    fn standalone_src(&self) -> &ShellStandaloneClkSrc {
        &self.0.data.standalone_src
    }

    fn standalone_src_mut(&mut self) -> &mut ShellStandaloneClkSrc {
        &mut self.0.data.standalone_src
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
    fn load(
        &mut self,
        card_cntr: &mut CardCntr,
        unit: &mut (SndDice, FwNode),
        req: &mut FwReq,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        KliveProtocol::read_segment(req, &mut unit.1, &mut self.0, timeout_ms)?;

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
            Ok(false)
        }
    }

    fn write(
        &mut self,
        unit: &mut (SndDice, FwNode),
        req: &mut FwReq,
        elem_id: &ElemId,
        _: &ElemValue,
        new: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        if self.write_mixer_stream_src(unit, req, elem_id, new, timeout_ms)? {
            Ok(true)
        } else if self.write_coax_out_src(unit, req, elem_id, new, timeout_ms)? {
            Ok(true)
        } else if self.write_opt_iface_config(unit, req, elem_id, new, timeout_ms)? {
            Ok(true)
        } else if self.write_standalone(unit, req, elem_id, new, timeout_ms)? {
            Ok(true)
        } else if self.write_midi_sender(req, &unit.1, elem_id, new, timeout_ms)? {
            Ok(true)
        } else {
            match elem_id.name().as_str() {
                OUT_01_SRC_NAME => {
                    ElemValueAccessor::<u32>::get_val(new, |val| {
                        PHYS_OUT_SRCS
                            .iter()
                            .nth(val as usize)
                            .ok_or_else(|| {
                                let msg = format!("Invalid index of output source: {}", val);
                                Error::new(FileError::Inval, &msg)
                            })
                            .map(|&s| self.0.data.out_01_src = s)
                    })?;
                    KliveProtocol::write_segment(req, &mut unit.1, &mut self.0, timeout_ms)
                        .map(|_| true)
                }
                OUT_23_SRC_NAME => {
                    ElemValueAccessor::<u32>::get_val(new, |val| {
                        PHYS_OUT_SRCS
                            .iter()
                            .nth(val as usize)
                            .ok_or_else(|| {
                                let msg = format!("Invalid index of output source: {}", val);
                                Error::new(FileError::Inval, &msg)
                            })
                            .map(|&s| self.0.data.out_23_src = s)
                    })?;
                    KliveProtocol::write_segment(req, &mut unit.1, &mut self.0, timeout_ms)
                        .map(|_| true)
                }
                _ => Ok(false),
            }
        }
    }

    fn parse_notification(
        &mut self,
        unit: &mut (SndDice, FwNode),
        req: &mut FwReq,
        msg: u32,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        if self.0.has_segment_change(msg) {
            KliveProtocol::read_segment(req, &mut unit.1, &mut self.0, timeout_ms)
        } else {
            Ok(())
        }
    }

    fn read_notified_elem(
        &mut self,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.read_mixer_stream_src(elem_id, elem_value)? {
            Ok(true)
        } else if self.read_coax_out_src(elem_id, elem_value)? {
            Ok(true)
        } else if self.read_opt_iface_config(elem_id, elem_value)? {
            Ok(true)
        } else if self.read_midi_sender(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[derive(Default)]
struct MixerCtl(
    KliveMixerStateSegment,
    KliveMixerMeterSegment,
    Vec<ElemId>,
    Vec<ElemId>,
);

impl ShellMixerCtlOperation<KliveMixerState, KliveMixerMeter, KliveProtocol> for MixerCtl {
    fn state_segment(&self) -> &KliveMixerStateSegment {
        &self.0
    }

    fn state_segment_mut(&mut self) -> &mut KliveMixerStateSegment {
        &mut self.0
    }

    fn meter_segment_mut(&mut self) -> &mut KliveMixerMeterSegment {
        &mut self.1
    }

    fn state(&self) -> &ShellMixerState {
        &self.0.data.mixer
    }

    fn state_mut(&mut self) -> &mut ShellMixerState {
        &mut self.0.data.mixer
    }

    fn meter(&self) -> &ShellMixerMeter {
        &self.1.data.0
    }

    fn meter_mut(&mut self) -> &mut ShellMixerMeter {
        &mut self.1.data.0
    }

    fn enabled(&self) -> bool {
        self.0.data.enabled
    }
}

impl ShellReverbReturnCtlOperation<KliveMixerState, KliveProtocol> for MixerCtl {
    fn segment_mut(&mut self) -> &mut KliveMixerStateSegment {
        &mut self.0
    }

    fn reverb_return(&self) -> &ShellReverbReturn {
        &self.0.data.reverb_return
    }

    fn reverb_return_mut(&mut self) -> &mut ShellReverbReturn {
        &mut self.0.data.reverb_return
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
impl MixerCtl {
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

    fn load(
        &mut self,
        card_cntr: &mut CardCntr,
        unit: &mut (SndDice, FwNode),
        req: &mut FwReq,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        KliveProtocol::read_segment(req, &mut unit.1, &mut self.0, timeout_ms)?;
        KliveProtocol::read_segment(req, &mut unit.1, &mut self.1, timeout_ms)?;

        self.load_mixer(card_cntr)
            .map(|(notified_elem_id_list, measured_elem_id_list)| {
                self.2 = notified_elem_id_list;
                self.3 = measured_elem_id_list;
            })?;

        self.load_reverb_return(card_cntr)
            .map(|mut notified_elem_id_list| self.2.append(&mut notified_elem_id_list))?;

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
        if self.read_mixer(elem_id, elem_value)? {
            Ok(true)
        } else if self.read_reverb_return(elem_id, elem_value)? {
            Ok(true)
        } else {
            match elem_id.name().as_str() {
                MIXER_ENABLE_NAME => {
                    ElemValueAccessor::<bool>::set_val(elem_value, || Ok(self.0.data.enabled))
                        .map(|_| true)
                }
                USE_CH_STRIP_AS_PLUGIN_NAME => {
                    ElemValueAccessor::<bool>::set_val(elem_value, || {
                        Ok(self.0.data.use_ch_strip_as_plugin)
                    })
                    .map(|_| true)
                }
                CH_STRIP_SRC_NAME => ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let pos = Self::CH_STRIP_SRCS
                        .iter()
                        .position(|s| self.0.data.ch_strip_src.eq(s))
                        .unwrap();
                    Ok(pos as u32)
                })
                .map(|_| true),
                CH_STRIP_MODE_NAME => ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let pos = Self::CH_STRIP_MODES
                        .iter()
                        .position(|s| self.0.data.ch_strip_mode.eq(s))
                        .unwrap();
                    Ok(pos as u32)
                })
                .map(|_| true),
                USE_REVERB_AT_MID_RATE => ElemValueAccessor::<bool>::set_val(elem_value, || {
                    Ok(self.0.data.use_reverb_at_mid_rate)
                })
                .map(|_| true),
                _ => Ok(false),
            }
        }
    }

    fn write(
        &mut self,
        unit: &mut (SndDice, FwNode),
        req: &mut FwReq,
        elem_id: &ElemId,
        old: &ElemValue,
        new: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        if self.write_mixer(unit, req, elem_id, old, new, timeout_ms)? {
            Ok(true)
        } else if self.write_reverb_return(unit, req, elem_id, new, timeout_ms)? {
            Ok(true)
        } else {
            match elem_id.name().as_str() {
                MIXER_ENABLE_NAME => {
                    ElemValueAccessor::<bool>::get_val(new, |val| {
                        self.0.data.enabled = val;
                        Ok(())
                    })?;
                    KliveProtocol::write_segment(req, &mut unit.1, &mut self.0, timeout_ms)
                        .map(|_| true)
                }
                USE_CH_STRIP_AS_PLUGIN_NAME => {
                    ElemValueAccessor::<bool>::get_val(new, |val| {
                        self.0.data.use_ch_strip_as_plugin = val;
                        Ok(())
                    })?;
                    KliveProtocol::write_segment(req, &mut unit.1, &mut self.0, timeout_ms)
                        .map(|_| true)
                }
                CH_STRIP_SRC_NAME => {
                    ElemValueAccessor::<u32>::get_val(new, |val| {
                        Self::CH_STRIP_SRCS
                            .iter()
                            .nth(val as usize)
                            .ok_or_else(|| {
                                let msg =
                                    format!("Invalid value for index of ch strip src: {}", val);
                                Error::new(FileError::Inval, &msg)
                            })
                            .map(|&s| self.0.data.ch_strip_src = s)
                    })?;
                    KliveProtocol::write_segment(req, &mut unit.1, &mut self.0, timeout_ms)
                        .map(|_| true)
                }
                CH_STRIP_MODE_NAME => {
                    ElemValueAccessor::<u32>::get_val(new, |val| {
                        Self::CH_STRIP_MODES
                            .iter()
                            .nth(val as usize)
                            .ok_or_else(|| {
                                let msg =
                                    format!("Invalid value for index of ch strip mode: {}", val);
                                Error::new(FileError::Inval, &msg)
                            })
                            .map(|&m| self.0.data.ch_strip_mode = m)
                    })?;
                    KliveProtocol::write_segment(req, &mut unit.1, &mut self.0, timeout_ms)
                        .map(|_| true)
                }
                USE_REVERB_AT_MID_RATE => {
                    ElemValueAccessor::<bool>::get_val(new, |val| {
                        self.0.data.use_reverb_at_mid_rate = val;
                        Ok(())
                    })?;
                    KliveProtocol::write_segment(req, &mut unit.1, &mut self.0, timeout_ms)
                        .map(|_| true)
                }
                _ => Ok(false),
            }
        }
    }

    fn parse_notification(
        &mut self,
        unit: &mut (SndDice, FwNode),
        req: &mut FwReq,
        msg: u32,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        if self.0.has_segment_change(msg) {
            KliveProtocol::read_segment(req, &mut unit.1, &mut self.0, timeout_ms)
        } else {
            Ok(())
        }
    }

    fn read_notified_elem(
        &mut self,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.read_mixer_notified_elem(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn measure_states(
        &mut self,
        unit: &mut (SndDice, FwNode),
        req: &mut FwReq,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        KliveProtocol::read_segment(req, &mut unit.1, &mut self.1, timeout_ms)
    }

    fn read_measured_elem(
        &mut self,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.read_mixer_measured_elem(elem_id, elem_value)? {
            Ok(true)
        } else if self.read_reverb_return_notified_elem(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[derive(Default)]
struct HwStateCtl(KliveHwStateSegment, Vec<ElemId>);

impl FirewireLedCtlOperation<KliveHwState, KliveProtocol> for HwStateCtl {
    fn segment(&self) -> &KliveHwStateSegment {
        &self.0
    }

    fn segment_mut(&mut self) -> &mut KliveHwStateSegment {
        &mut self.0
    }

    fn firewire_led(params: &KliveHwState) -> &FireWireLedState {
        &params.0.firewire_led
    }

    fn firewire_led_mut(params: &mut KliveHwState) -> &mut FireWireLedState {
        &mut params.0.firewire_led
    }
}

impl ShellHwStateCtlOperation<KliveHwState, KliveProtocol> for HwStateCtl {
    fn hw_state(&self) -> &ShellHwState {
        &self.0.data.0
    }

    fn hw_state_mut(&mut self) -> &mut ShellHwState {
        &mut self.0.data.0
    }
}

impl HwStateCtl {
    fn load(
        &mut self,
        card_cntr: &mut CardCntr,
        unit: &mut (SndDice, FwNode),
        req: &mut FwReq,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        KliveProtocol::read_segment(req, &mut unit.1, &mut self.0, timeout_ms)?;

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
        _: &ElemValue,
        new: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        if self.write_hw_state(req, node, elem_id, new, timeout_ms)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn parse_notification(
        &mut self,
        unit: &mut (SndDice, FwNode),
        req: &mut FwReq,
        msg: u32,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        if self.0.has_segment_change(msg) {
            KliveProtocol::read_segment(req, &mut unit.1, &mut self.0, timeout_ms)
        } else {
            Ok(())
        }
    }
}

#[derive(Default, Debug)]
struct ReverbCtl(
    KliveReverbStateSegment,
    KliveReverbMeterSegment,
    Vec<ElemId>,
    Vec<ElemId>,
);

impl ReverbCtlOperation<KliveReverbState, KliveReverbMeter, KliveProtocol> for ReverbCtl {
    fn state_segment(&self) -> &KliveReverbStateSegment {
        &self.0
    }

    fn state_segment_mut(&mut self) -> &mut KliveReverbStateSegment {
        &mut self.0
    }

    fn meter_segment(&self) -> &KliveReverbMeterSegment {
        &self.1
    }

    fn meter_segment_mut(&mut self) -> &mut KliveReverbMeterSegment {
        &mut self.1
    }

    fn state(params: &KliveReverbState) -> &ReverbState {
        &params.0
    }

    fn state_mut(params: &mut KliveReverbState) -> &mut ReverbState {
        &mut params.0
    }

    fn meter(params: &KliveReverbMeter) -> &ReverbMeter {
        &params.0
    }
}

#[derive(Default)]
struct ChStripCtl(
    KliveChStripStatesSegment,
    KliveChStripMetersSegment,
    Vec<ElemId>,
    Vec<ElemId>,
);

impl ChStripCtlOperation<KliveChStripStates, KliveChStripMeters, KliveProtocol> for ChStripCtl {
    fn states_segment(&self) -> &KliveChStripStatesSegment {
        &self.0
    }

    fn states_segment_mut(&mut self) -> &mut KliveChStripStatesSegment {
        &mut self.0
    }

    fn meters_segment_mut(&mut self) -> &mut KliveChStripMetersSegment {
        &mut self.1
    }

    fn states(params: &KliveChStripStates) -> &[ChStripState] {
        &params.0
    }

    fn states_mut(params: &mut KliveChStripStates) -> &mut [ChStripState] {
        &mut params.0
    }

    fn meters(&self) -> &[ChStripMeter] {
        &self.1.data.0
    }
}
