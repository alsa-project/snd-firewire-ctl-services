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
use dice_protocols::tcelectronic::reverb::*;
use dice_protocols::tcelectronic::standalone::*;
use dice_protocols::tcelectronic::{
    shell::{itwin::*, *},
    *,
};

use super::{ch_strip_ctl::*, fw_led_ctl::*, reverb_ctl::*, shell_ctl::*, standalone_ctl::*};
use crate::common_ctl::*;

#[derive(Default)]
pub struct ItwinModel {
    req: FwReq,
    sections: GeneralSections,
    ctl: CommonCtl,
    knob_ctl: KnobCtl,
    config_ctl: ConfigCtl,
    mixer_ctl: MixerCtl,
    hw_state_ctl: HwStateCtl,
    reverb_ctl: ReverbCtl,
    ch_strip_ctl: ChStripCtl,
}

const TIMEOUT_MS: u32 = 20;

impl CtlModel<SndDice> for ItwinModel {
    fn load(&mut self, unit: &mut SndDice, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let mut node = unit.get_node();

        self.sections = GeneralProtocol::read_general_sections(
            &mut self.req,
            &mut node,
            TIMEOUT_MS
        )?;
        let caps = GlobalSectionProtocol::read_clock_caps(
            &mut self.req,
            &mut node,
            &self.sections,
            TIMEOUT_MS
        )?;
        let src_labels = GlobalSectionProtocol::read_clock_source_labels(
            &mut self.req,
            &mut node,
            &self.sections,
            TIMEOUT_MS
        )?;
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

impl NotifyModel<SndDice, u32> for ItwinModel {
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

impl MeasureModel<SndDice> for ItwinModel {
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
        } else if self.ch_strip_ctl.read_measured_elem(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_ctl.read_measured_elem(elem_id, elem_value)? {
            Ok(true)
        } else if self.reverb_ctl.read_measured_elem(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[derive(Default)]
struct KnobCtl(ItwinKnobSegment, Vec<ElemId>);

impl ShellKnobCtlOperation<ItwinKnob, ItwinProtocol> for KnobCtl {
    const TARGETS: [&'static str; 4] = [
        "Channel-strip-1",
        "Channel-strip-2",
        "Reverb-1/2",
        "Mixer-1/2",
    ];

    fn segment_mut(&mut self) -> &mut ItwinKnobSegment {
        &mut self.0
    }

    fn knob_target(&self) -> &ShellKnobTarget {
        &self.0.data.target
    }

    fn knob_target_mut(&mut self) -> &mut ShellKnobTarget {
        &mut self.0.data.target
    }
}

const CLK_RECOVERY_NAME: &str = "clock-recovery";

impl KnobCtl {
    fn load(
        &mut self,
        card_cntr: &mut CardCntr,
        unit: &mut SndDice,
        req: &mut FwReq,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        ItwinProtocol::read_segment(req, &mut unit.get_node(), &mut self.0, timeout_ms)?;

        self.load_knob_target(card_cntr)
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, CLK_RECOVERY_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        Ok(())
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        if self.read_knob_target(elem_id, elem_value)? {
            Ok(true)
        } else {
            match elem_id.get_name().as_str() {
                CLK_RECOVERY_NAME => ElemValueAccessor::<bool>::set_val(elem_value, || {
                    Ok(self.0.data.clock_recovery)
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
        if self.write_knob_target(unit, req, elem_id, new, timeout_ms)? {
            Ok(true)
        } else {
            match elem_id.get_name().as_str() {
                CLK_RECOVERY_NAME => {
                    ElemValueAccessor::<bool>::get_val(new, |val| {
                        self.0.data.clock_recovery = val;
                        Ok(())
                    })?;
                    ItwinProtocol::write_segment(req, &mut unit.get_node(), &mut self.0, timeout_ms)
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
            ItwinProtocol::read_segment(req, &mut unit.get_node(), &mut self.0, timeout_ms)
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
        } else {
            Ok(false)
        }
    }
}

#[derive(Default)]
struct ConfigCtl(ItwinConfigSegment, Vec<ElemId>);

impl ShellMixerStreamSrcCtlOperation<ItwinConfig, ItwinProtocol> for ConfigCtl {
    fn segment_mut(&mut self) -> &mut ItwinConfigSegment {
        &mut self.0
    }

    fn mixer_stream_src(&self) -> &ShellMixerStreamSrcPair {
        &self.0.data.mixer_stream_src_pair
    }

    fn mixer_stream_src_mut(&mut self) -> &mut ShellMixerStreamSrcPair {
        &mut self.0.data.mixer_stream_src_pair
    }
}

impl StandaloneCtlOperation<ItwinConfig, ItwinProtocol> for ConfigCtl {
    fn segment_mut(&mut self) -> &mut ItwinConfigSegment {
        &mut self.0
    }

    fn standalone_rate(&self) -> &TcKonnektStandaloneClkRate {
        &self.0.data.standalone_rate
    }

    fn standalone_rate_mut(&mut self) -> &mut TcKonnektStandaloneClkRate {
        &mut self.0.data.standalone_rate
    }
}

impl ShellStandaloneCtlOperation<ItwinConfig, ItwinProtocol> for ConfigCtl {
    fn standalone_src(&self) -> &ShellStandaloneClkSrc {
        &self.0.data.standalone_src
    }

    fn standalone_src_mut(&mut self) -> &mut ShellStandaloneClkSrc {
        &mut self.0.data.standalone_src
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

    fn load(
        &mut self,
        card_cntr: &mut CardCntr,
        unit: &mut SndDice,
        req: &mut FwReq,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        ItwinProtocol::read_segment(req, &mut unit.get_node(), &mut self.0, timeout_ms)?;

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
            match elem_id.get_name().as_str() {
                OUT_SRC_NAME => ElemValueAccessor::<u32>::set_vals(
                    elem_value,
                    ITWIN_PHYS_OUT_PAIR_COUNT,
                    |idx| {
                        let pos = Self::OUT_SRCS
                            .iter()
                            .position(|s| self.0.data.output_pair_src[idx].eq(s))
                            .unwrap();
                        Ok(pos as u32)
                    },
                )
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
        if self.write_mixer_stream_src(unit, req, elem_id, new, timeout_ms)? {
            Ok(true)
        } else if self.write_standalone(unit, req, elem_id, new, timeout_ms)? {
            Ok(true)
        } else {
            match elem_id.get_name().as_str() {
                OUT_SRC_NAME => {
                    ElemValueAccessor::<u32>::get_vals(
                        new,
                        old,
                        ITWIN_PHYS_OUT_PAIR_COUNT,
                        |idx, val| {
                            Self::OUT_SRCS
                                .iter()
                                .nth(val as usize)
                                .ok_or_else(|| {
                                    let msg = format!("Invalid index of output source: {}", val);
                                    Error::new(FileError::Inval, &msg)
                                })
                                .map(|&s| self.0.data.output_pair_src[idx] = s)
                        },
                    )?;
                    ItwinProtocol::write_segment(req, &mut unit.get_node(), &mut self.0, timeout_ms)
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
            ItwinProtocol::read_segment(req, &mut unit.get_node(), &mut self.0, timeout_ms)
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
        } else {
            Ok(false)
        }
    }
}

#[derive(Default)]
struct MixerCtl(
    ItwinMixerStateSegment,
    ItwinMixerMeterSegment,
    Vec<ElemId>,
    Vec<ElemId>,
);

impl ShellMixerCtlOperation<ItwinMixerState, ItwinMixerMeter, ItwinProtocol> for MixerCtl {
    fn state_segment(&self) -> &ItwinMixerStateSegment {
        &self.0
    }

    fn state_segment_mut(&mut self) -> &mut ItwinMixerStateSegment {
        &mut self.0
    }

    fn meter_segment_mut(&mut self) -> &mut ItwinMixerMeterSegment {
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

const MIXER_ENABLE_NAME: &str = "mixer-enable";

impl MixerCtl {
    fn load(
        &mut self,
        card_cntr: &mut CardCntr,
        unit: &mut SndDice,
        req: &mut FwReq,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        ItwinProtocol::read_segment(req, &mut unit.get_node(), &mut self.0, timeout_ms)?;
        ItwinProtocol::read_segment(req, &mut unit.get_node(), &mut self.1, timeout_ms)?;

        self.load_mixer(card_cntr)
            .map(|(notified_elem_id_list, measured_elem_id_list)| {
                self.2 = notified_elem_id_list;
                self.3 = measured_elem_id_list;
            })?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIXER_ENABLE_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        Ok(())
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        if self.read_mixer(elem_id, elem_value)? {
            Ok(true)
        } else {
            match elem_id.get_name().as_str() {
                MIXER_ENABLE_NAME => {
                    ElemValueAccessor::<bool>::set_val(elem_value, || Ok(self.0.data.enabled))
                        .map(|_| true)
                }
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
        } else {
            match elem_id.get_name().as_str() {
                MIXER_ENABLE_NAME => {
                    ElemValueAccessor::<bool>::get_val(new, |val| {
                        self.0.data.enabled = val;
                        Ok(())
                    })?;
                    ItwinProtocol::write_segment(req, &mut unit.get_node(), &mut self.0, timeout_ms)
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
            ItwinProtocol::read_segment(req, &mut unit.get_node(), &mut self.0, timeout_ms)
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
        unit: &mut SndDice,
        req: &mut FwReq,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        ItwinProtocol::read_segment(req, &mut unit.get_node(), &mut self.1, timeout_ms)
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
struct HwStateCtl(ItwinHwStateSegment, Vec<ElemId>);

impl FirewireLedCtlOperation<ItwinHwState, ItwinProtocol> for HwStateCtl {
    fn segment_mut(&mut self) -> &mut ItwinHwStateSegment {
        &mut self.0
    }

    fn firewire_led(&self) -> &FireWireLedState {
        &self.0.data.hw_state.firewire_led
    }

    fn firewire_led_mut(&mut self) -> &mut FireWireLedState {
        &mut self.0.data.hw_state.firewire_led
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

    fn load(
        &mut self,
        card_cntr: &mut CardCntr,
        unit: &mut SndDice,
        req: &mut FwReq,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        ItwinProtocol::read_segment(req, &mut unit.get_node(), &mut self.0, timeout_ms)?;

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
            match elem_id.get_name().as_str() {
                LISTENING_MODE_NAME => ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let pos = Self::LISTENING_MODES
                        .iter()
                        .position(|m| self.0.data.listening_mode.eq(m))
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
        if self.write_hw_state(unit, req, elem_id, new, timeout_ms)? {
            Ok(true)
        } else {
            match elem_id.get_name().as_str() {
                LISTENING_MODE_NAME => {
                    ElemValueAccessor::<u32>::get_val(new, |val| {
                        Self::LISTENING_MODES
                            .iter()
                            .nth(val as usize)
                            .ok_or_else(|| {
                                let msg = format!("Invalid index of listening mode: {}", val);
                                Error::new(FileError::Inval, &msg)
                            })
                            .map(|&m| self.0.data.listening_mode = m)
                    })?;
                    ItwinProtocol::write_segment(req, &mut unit.get_node(), &mut self.0, timeout_ms)
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
            ItwinProtocol::read_segment(req, &mut unit.get_node(), &mut self.0, timeout_ms)
        } else {
            Ok(())
        }
    }
}

#[derive(Default)]
struct ReverbCtl(
    ItwinReverbStateSegment,
    ItwinReverbMeterSegment,
    Vec<ElemId>,
    Vec<ElemId>,
);

impl ReverbCtlOperation<ItwinReverbState, ItwinReverbMeter, ItwinProtocol> for ReverbCtl {
    fn state_segment(&self) -> &ItwinReverbStateSegment {
        &self.0
    }

    fn state_segment_mut(&mut self) -> &mut ItwinReverbStateSegment {
        &mut self.0
    }

    fn meter_segment_mut(&mut self) -> &mut ItwinReverbMeterSegment {
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
    ItwinChStripStatesSegment,
    ItwinChStripMetersSegment,
    Vec<ElemId>,
    Vec<ElemId>,
);

impl ChStripCtlOperation<ItwinChStripStates, ItwinChStripMeters, ItwinProtocol> for ChStripCtl {
    fn states_segment(&self) -> &ItwinChStripStatesSegment {
        &self.0
    }

    fn states_segment_mut(&mut self) -> &mut ItwinChStripStatesSegment {
        &mut self.0
    }

    fn meters_segment_mut(&mut self) -> &mut ItwinChStripMetersSegment {
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
