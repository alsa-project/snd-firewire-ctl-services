// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::{Error, FileError};

use alsactl::{ElemId, ElemIfaceType, ElemValue};

use hinawa::FwReq;
use hinawa::{SndDice, SndUnitExt};

use core::card_cntr::*;
use core::elem_value_accessor::*;

use dice_protocols::tcat::{global_section::*, *};
use dice_protocols::tcelectronic::ch_strip::*;
use dice_protocols::tcelectronic::fw_led::*;
use dice_protocols::tcelectronic::prog::*;
use dice_protocols::tcelectronic::reverb::*;
use dice_protocols::tcelectronic::standalone::*;
use dice_protocols::tcelectronic::{
    shell::{k24d::*, *},
    *,
};

use super::prog_ctl::*;
use super::{ch_strip_ctl::*, fw_led_ctl::*, reverb_ctl::*, shell_ctl::*, standalone_ctl::*};
use crate::common_ctl::*;

#[derive(Default)]
pub struct K24dModel {
    req: FwReq,
    sections: GeneralSections,
    ctl: CommonCtl,
    knob_ctl: KnobCtl,
    config_ctl: ConfigCtl,
    mixer_ctl: MixerCtl,
    hw_state_ctl: HwStateCtl,
    ch_strip_ctl: ChStripCtl,
    reverb_ctl: ReverbCtl,
}

const TIMEOUT_MS: u32 = 20;

impl CtlModel<SndDice> for K24dModel {
    fn load(&mut self, unit: &mut SndDice, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let mut node = unit.get_node();

        self.sections = self.req.read_general_sections(&mut node, TIMEOUT_MS)?;
        let caps = self
            .req
            .read_clock_caps(&mut node, &self.sections, TIMEOUT_MS)?;
        let src_labels =
            self.req
                .read_clock_source_labels(&mut node, &self.sections, TIMEOUT_MS)?;
        self.ctl.load(card_cntr, &caps, &src_labels)?;

        self.knob_ctl
            .load(card_cntr, unit, &mut self.req, TIMEOUT_MS)?;
        self.config_ctl
            .load(card_cntr, unit, &mut self.req, TIMEOUT_MS)?;
        self.mixer_ctl
            .load(card_cntr, unit, &mut self.req, TIMEOUT_MS)?;
        self.hw_state_ctl
            .load(card_cntr, unit, &mut self.req, TIMEOUT_MS)?;
        self.reverb_ctl
            .load(card_cntr, unit, &mut self.req, TIMEOUT_MS)
            .map(|(notified_elem_id_list, measured_elem_id_list)| {
                self.reverb_ctl.2 = notified_elem_id_list;
                self.reverb_ctl.3 = measured_elem_id_list;
            })?;
        self.ch_strip_ctl
            .load(card_cntr, unit, &mut self.req, TIMEOUT_MS)
            .map(|(notified_elem_id_list, measured_elem_id_list)| {
                self.ch_strip_ctl.2 = notified_elem_id_list;
                self.ch_strip_ctl.3 = measured_elem_id_list;
            })?;

        Ok(())
    }

    fn read(
        &mut self,
        unit: &mut SndDice,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.ctl.read(
            unit,
            &mut self.req,
            &self.sections,
            elem_id,
            elem_value,
            TIMEOUT_MS,
        )? {
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
        unit: &mut SndDice,
        elem_id: &ElemId,
        old: &ElemValue,
        new: &ElemValue,
    ) -> Result<bool, Error> {
        if self.ctl.write(
            unit,
            &mut self.req,
            &self.sections,
            elem_id,
            old,
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
            .hw_state_ctl
            .write(unit, &mut self.req, elem_id, old, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .reverb_ctl
            .write(unit, &mut self.req, elem_id, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .ch_strip_ctl
            .write(unit, &mut self.req, elem_id, old, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl NotifyModel<SndDice, u32> for K24dModel {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.ctl.notified_elem_list);
        elem_id_list.extend_from_slice(&self.knob_ctl.1);
        elem_id_list.extend_from_slice(&self.config_ctl.1);
        elem_id_list.extend_from_slice(&self.mixer_ctl.2);
        elem_id_list.extend_from_slice(&self.hw_state_ctl.1);
        elem_id_list.extend_from_slice(&self.reverb_ctl.2);
        elem_id_list.extend_from_slice(&self.ch_strip_ctl.2);
    }

    fn parse_notification(&mut self, unit: &mut SndDice, msg: &u32) -> Result<(), Error> {
        self.ctl
            .parse_notification(unit, &mut self.req, &self.sections, *msg, TIMEOUT_MS)?;
        self.knob_ctl
            .parse_notification(unit, &mut self.req, *msg, TIMEOUT_MS)?;
        self.config_ctl
            .parse_notification(unit, &mut self.req, *msg, TIMEOUT_MS)?;
        self.mixer_ctl
            .parse_notification(unit, &mut self.req, *msg, TIMEOUT_MS)?;
        self.hw_state_ctl
            .parse_notification(unit, &mut self.req, *msg, TIMEOUT_MS)?;
        self.reverb_ctl
            .parse_notification(unit, &mut self.req, *msg, TIMEOUT_MS)?;
        self.ch_strip_ctl
            .parse_notification(unit, &mut self.req, *msg, TIMEOUT_MS)?;
        Ok(())
    }

    fn read_notified_elem(
        &mut self,
        _: &SndDice,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.ctl.read_notified_elem(elem_id, elem_value)? {
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

impl MeasureModel<SndDice> for K24dModel {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.ctl.measured_elem_list);
        elem_id_list.extend_from_slice(&self.mixer_ctl.3);
        elem_id_list.extend_from_slice(&self.reverb_ctl.3);
        elem_id_list.extend_from_slice(&self.ch_strip_ctl.3);
    }

    fn measure_states(&mut self, unit: &mut SndDice) -> Result<(), Error> {
        self.ctl
            .measure_states(unit, &mut self.req, &self.sections, TIMEOUT_MS)?;
        self.mixer_ctl
            .measure_states(unit, &mut self.req, TIMEOUT_MS)?;
        self.reverb_ctl
            .measure_states(unit, &mut self.req, TIMEOUT_MS)?;
        self.ch_strip_ctl
            .measure_states(unit, &mut self.req, TIMEOUT_MS)?;
        Ok(())
    }

    fn measure_elem(
        &mut self,
        _: &SndDice,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.ctl.measure_elem(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_ctl.read_measured_elem(elem_id, elem_value)? {
            Ok(true)
        } else if self.ch_strip_ctl.read_measured_elem(elem_id, elem_value)? {
            Ok(true)
        } else if self.reverb_ctl.read_measured_elem(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[derive(Default)]
struct KnobCtl(K24dKnobSegment, Vec<ElemId>);

impl ShellKnobCtlOperation<K24dKnob, K24dProtocol> for KnobCtl {
    const TARGETS: [&'static str; 4] = ["Analog-1", "Analog-2", "Analog-3/4", "Configurable"];

    fn segment_mut(&mut self) -> &mut K24dKnobSegment {
        &mut self.0
    }

    fn knob_target(&self) -> &ShellKnobTarget {
        &self.0.data.target
    }

    fn knob_target_mut(&mut self) -> &mut ShellKnobTarget {
        &mut self.0.data.target
    }
}

impl ShellKnob2CtlOperation<K24dKnob, K24dProtocol> for KnobCtl {
    const TARGETS: &'static [&'static str] = &[
        "Digital-1/2",
        "Digital-3/4",
        "Digital-5/6",
        "Digital-7/8",
        "Stream",
        "Reverb-1/2",
        "Mixer-1/2",
        "Tune-pitch-tone",
    ];

    fn segment_mut(&mut self) -> &mut K24dKnobSegment {
        &mut self.0
    }

    fn knob2_target(&self) -> &ShellKnob2Target {
        &self.0.data.knob2_target
    }

    fn knob2_target_mut(&mut self) -> &mut ShellKnob2Target {
        &mut self.0.data.knob2_target
    }
}

impl ProgramCtlOperation<K24dKnob, K24dProtocol> for KnobCtl {
    fn segment_mut(&mut self) -> &mut K24dKnobSegment {
        &mut self.0
    }

    fn prog(&self) -> &TcKonnektLoadedProgram {
        &self.0.data.prog
    }

    fn prog_mut(&mut self) -> &mut TcKonnektLoadedProgram {
        &mut self.0.data.prog
    }
}

impl KnobCtl {
    fn load(
        &mut self,
        card_cntr: &mut CardCntr,
        unit: &mut SndDice,
        req: &mut FwReq,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        K24dProtocol::read_segment(req, &mut unit.get_node(), &mut self.0, timeout_ms)?;

        self.load_knob_target(card_cntr)
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        self.load_knob2_target(card_cntr)
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        self.load_prog(card_cntr)
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

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
            Ok(false)
        }
    }

    fn write(
        &mut self,
        unit: &mut SndDice,
        req: &mut FwReq,
        elem_id: &ElemId,
        _: &ElemValue,
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
            Ok(false)
        }
    }

    fn parse_notification(
        &mut self,
        unit: &mut SndDice,
        req: &mut FwReq,
        msg: u32,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        if self.0.has_segment_change(msg) {
            K24dProtocol::read_segment(req, &mut unit.get_node(), &mut self.0, timeout_ms)
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
struct ConfigCtl(K24dConfigSegment, Vec<ElemId>);

impl ShellCoaxIfaceCtlOperation<K24dConfig, K24dProtocol> for ConfigCtl {
    fn segment_mut(&mut self) -> &mut K24dConfigSegment {
        &mut self.0
    }

    fn coax_out_src(&self) -> &ShellCoaxOutPairSrc {
        &self.0.data.coax_out_src
    }

    fn coax_out_src_mut(&mut self) -> &mut ShellCoaxOutPairSrc {
        &mut self.0.data.coax_out_src
    }
}

impl ShellOptIfaceCtl<K24dConfig, K24dProtocol> for ConfigCtl {
    fn segment_mut(&mut self) -> &mut K24dConfigSegment {
        &mut self.0
    }

    fn opt_iface_config(&self) -> &ShellOptIfaceConfig {
        &self.0.data.opt
    }

    fn opt_iface_config_mut(&mut self) -> &mut ShellOptIfaceConfig {
        &mut self.0.data.opt
    }
}

impl StandaloneCtlOperation<K24dConfig, K24dProtocol> for ConfigCtl {
    fn segment_mut(&mut self) -> &mut K24dConfigSegment {
        &mut self.0
    }

    fn standalone_rate(&self) -> &TcKonnektStandaloneClkRate {
        &self.0.data.standalone_rate
    }

    fn standalone_rate_mut(&mut self) -> &mut TcKonnektStandaloneClkRate {
        &mut self.0.data.standalone_rate
    }
}

impl ShellStandaloneCtlOperation<K24dConfig, K24dProtocol> for ConfigCtl {
    fn standalone_src(&self) -> &ShellStandaloneClkSrc {
        &self.0.data.standalone_src
    }

    fn standalone_src_mut(&mut self) -> &mut ShellStandaloneClkSrc {
        &mut self.0.data.standalone_src
    }
}

const OUT_23_SRC_NAME: &str = "output-3/4-source";

impl ConfigCtl {
    fn load(
        &mut self,
        card_cntr: &mut CardCntr,
        unit: &mut SndDice,
        req: &mut FwReq,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        K24dProtocol::read_segment(req, &mut unit.get_node(), &mut self.0, timeout_ms)?;

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
            match elem_id.get_name().as_str() {
                OUT_23_SRC_NAME => ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let pos = PHYS_OUT_SRCS
                        .iter()
                        .position(|s| self.0.data.out_23_src.eq(s))
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
        unit: &mut SndDice,
        req: &mut FwReq,
        elem_id: &ElemId,
        _: &ElemValue,
        new: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        if self.write_coax_out_src(unit, req, elem_id, new, timeout_ms)? {
            Ok(true)
        } else if self.write_opt_iface_config(unit, req, elem_id, new, timeout_ms)? {
            Ok(true)
        } else if self.write_standalone(unit, req, elem_id, new, timeout_ms)? {
            Ok(true)
        } else {
            match elem_id.get_name().as_str() {
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
                    K24dProtocol::write_segment(req, &mut unit.get_node(), &mut self.0, timeout_ms)
                        .map(|_| true)
                }
                _ => Ok(false),
            }
        }
    }

    fn parse_notification(
        &mut self,
        unit: &mut SndDice,
        req: &mut FwReq,
        msg: u32,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        if self.0.has_segment_change(msg) {
            K24dProtocol::read_segment(req, &mut unit.get_node(), &mut self.0, timeout_ms)
        } else {
            Ok(())
        }
    }

    fn read_notified_elem(
        &mut self,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.read_coax_out_src(elem_id, elem_value)? {
            Ok(true)
        } else if self.read_opt_iface_config(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[derive(Default)]
struct MixerCtl(
    K24dMixerStateSegment,
    K24dMixerMeterSegment,
    Vec<ElemId>,
    Vec<ElemId>,
);

impl ShellMixerCtlOperation<K24dMixerState, K24dMixerMeter, K24dProtocol> for MixerCtl {
    fn state_segment(&self) -> &K24dMixerStateSegment {
        &self.0
    }

    fn state_segment_mut(&mut self) -> &mut K24dMixerStateSegment {
        &mut self.0
    }

    fn meter_segment_mut(&mut self) -> &mut K24dMixerMeterSegment {
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

impl ShellReverbReturnCtlOperation<K24dMixerState, K24dProtocol> for MixerCtl {
    fn segment_mut(&mut self) -> &mut K24dMixerStateSegment {
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
const USE_REVERB_AT_MID_RATE: &str = "use-reverb-at-mid-rate";

impl MixerCtl {
    fn load(
        &mut self,
        card_cntr: &mut CardCntr,
        unit: &mut SndDice,
        req: &mut FwReq,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        K24dProtocol::read_segment(req, &mut unit.get_node(), &mut self.0, timeout_ms)?;
        K24dProtocol::read_segment(req, &mut unit.get_node(), &mut self.1, timeout_ms)?;

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
            match elem_id.get_name().as_str() {
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
        unit: &mut SndDice,
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
            match elem_id.get_name().as_str() {
                MIXER_ENABLE_NAME => {
                    ElemValueAccessor::<bool>::get_val(new, |val| {
                        self.0.data.enabled = val;
                        Ok(())
                    })?;
                    K24dProtocol::write_segment(req, &mut unit.get_node(), &mut self.0, timeout_ms)
                        .map(|_| true)
                }
                USE_CH_STRIP_AS_PLUGIN_NAME => {
                    ElemValueAccessor::<bool>::get_val(new, |val| {
                        self.0.data.use_ch_strip_as_plugin = val;
                        Ok(())
                    })?;
                    K24dProtocol::write_segment(req, &mut unit.get_node(), &mut self.0, timeout_ms)
                        .map(|_| true)
                }
                USE_REVERB_AT_MID_RATE => {
                    ElemValueAccessor::<bool>::get_val(new, |val| {
                        self.0.data.use_reverb_at_mid_rate = val;
                        Ok(())
                    })?;
                    K24dProtocol::write_segment(req, &mut unit.get_node(), &mut self.0, timeout_ms)
                        .map(|_| true)
                }
                _ => Ok(false),
            }
        }
    }

    fn parse_notification(
        &mut self,
        unit: &mut SndDice,
        req: &mut FwReq,
        msg: u32,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        if self.0.has_segment_change(msg) {
            K24dProtocol::read_segment(req, &mut unit.get_node(), &mut self.0, timeout_ms)
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
        } else if self.read_reverb_return_notified_elem(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn measure_states(
        &mut self,
        unit: &mut SndDice,
        req: &mut FwReq,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        K24dProtocol::read_segment(req, &mut unit.get_node(), &mut self.1, timeout_ms)
    }

    fn read_measured_elem(
        &mut self,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.read_mixer_measured_elem(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[derive(Default)]
struct HwStateCtl(K24dHwStateSegment, Vec<ElemId>);

impl FirewireLedCtlOperation<K24dHwState, K24dProtocol> for HwStateCtl {
    fn segment_mut(&mut self) -> &mut K24dHwStateSegment {
        &mut self.0
    }

    fn firewire_led(&self) -> &FireWireLedState {
        &self.0.data.0.firewire_led
    }

    fn firewire_led_mut(&mut self) -> &mut FireWireLedState {
        &mut self.0.data.0.firewire_led
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
    fn load(
        &mut self,
        card_cntr: &mut CardCntr,
        unit: &mut SndDice,
        req: &mut FwReq,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        K24dProtocol::read_segment(req, &mut unit.get_node(), &mut self.0, timeout_ms)?;

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
        unit: &mut SndDice,
        req: &mut FwReq,
        elem_id: &ElemId,
        _: &ElemValue,
        new: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        if self.write_hw_state(unit, req, elem_id, new, timeout_ms)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn parse_notification(
        &mut self,
        unit: &mut SndDice,
        req: &mut FwReq,
        msg: u32,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        if self.0.has_segment_change(msg) {
            K24dProtocol::read_segment(req, &mut unit.get_node(), &mut self.0, timeout_ms)
        } else {
            Ok(())
        }
    }
}

#[derive(Default)]
struct ReverbCtl(
    K24dReverbStateSegment,
    K24dReverbMeterSegment,
    Vec<ElemId>,
    Vec<ElemId>,
);

impl ReverbCtlOperation<K24dReverbState, K24dReverbMeter, K24dProtocol> for ReverbCtl {
    fn state_segment(&self) -> &K24dReverbStateSegment {
        &self.0
    }

    fn state_segment_mut(&mut self) -> &mut K24dReverbStateSegment {
        &mut self.0
    }

    fn meter_segment_mut(&mut self) -> &mut K24dReverbMeterSegment {
        &mut self.1
    }

    fn state(&self) -> &ReverbState {
        &self.0.data.0
    }

    fn state_mut(&mut self) -> &mut ReverbState {
        &mut self.0.data.0
    }

    fn meter(&self) -> &ReverbMeter {
        &self.1.data.0
    }
}

#[derive(Default)]
struct ChStripCtl(
    K24dChStripStatesSegment,
    K24dChStripMetersSegment,
    Vec<ElemId>,
    Vec<ElemId>,
);

impl ChStripCtlOperation<K24dChStripStates, K24dChStripMeters, K24dProtocol> for ChStripCtl {
    fn states_segment(&self) -> &K24dChStripStatesSegment {
        &self.0
    }

    fn states_segment_mut(&mut self) -> &mut K24dChStripStatesSegment {
        &mut self.0
    }

    fn meters_segment_mut(&mut self) -> &mut K24dChStripMetersSegment {
        &mut self.1
    }

    fn states(&self) -> &[ChStripState] {
        &self.0.data.0
    }

    fn states_mut(&mut self) -> &mut [ChStripState] {
        &mut self.0.data.0
    }

    fn meters(&self) -> &[ChStripMeter] {
        &self.1.data.0
    }
}
