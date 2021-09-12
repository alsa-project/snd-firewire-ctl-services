// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Protocol defined by TC Electronic for Konnekt 8.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by TC Electronic for Konnekt 8.

use super::*;
use crate::tcelectronic::{standalone::*, *};

/// The structure for protocol implementation of Konnekt 8.
#[derive(Default)]
pub struct K8Protocol;

/// Segment for knob. 0x0000..0x0027 (36 quads).
pub type K8KnobSegment = TcKonnektSegment<K8Knob>;
impl SegmentOperation<K8Knob> for K8Protocol {}

/// Segment for configuration. 0x0028..0x0073 (19 quads).
pub type K8ConfigSegment = TcKonnektSegment<K8Config>;
impl SegmentOperation<K8Config> for K8Protocol {}

/// Segment for state of mixer. 0x0074..0x01cf (87 quads).
pub type K8MixerStateSegment = TcKonnektSegment<K8MixerState>;
impl SegmentOperation<K8MixerState> for K8Protocol {}

/// Segment for mixer meter. 0x105c..0x10b7 (23 quads).
pub type K8MixerMeterSegment = TcKonnektSegment<K8MixerMeter>;
impl SegmentOperation<K8MixerMeter> for K8Protocol {}

/// Segment tor state of hardware. 0x100c..0x1027 (7 quads).
pub type K8HwStateSegment = TcKonnektSegment<K8HwState>;
impl SegmentOperation<K8HwState> for K8Protocol {}

/// The structure to represent state of knob.
#[derive(Default, Debug)]
pub struct K8Knob {
    pub target: ShellKnobTarget,
    pub knob2_target: ShellKnob2Target,
}

impl ShellKnobTargetSpec for K8Knob {
    const HAS_SPDIF: bool = true;
    const HAS_EFFECTS: bool = false;
}

impl ShellKnob2TargetSpec for K8Knob {
    const KNOB2_TARGET_COUNT: usize = 2;
}

impl TcKonnektSegmentData for K8Knob {
    fn build(&self, raw: &mut [u8]) {
        self.target.0.build_quadlet(&mut raw[..4]);
        self.knob2_target.0.build_quadlet(&mut raw[4..8]);
    }

    fn parse(&mut self, raw: &[u8]) {
        self.target.0.parse_quadlet(&raw[..4]);
        self.knob2_target.0.parse_quadlet(&raw[4..8]);
    }
}

impl TcKonnektSegmentSpec for TcKonnektSegment<K8Knob> {
    const OFFSET: usize = 0x0004;
    const SIZE: usize = SHELL_KNOB_SIZE;
}

impl TcKonnektNotifiedSegmentSpec for TcKonnektSegment<K8Knob> {
    const NOTIFY_FLAG: u32 = SHELL_KNOB_NOTIFY_FLAG;
}

/// The structure to represent configuration.
#[derive(Default, Debug)]
pub struct K8Config {
    pub coax_out_src: ShellCoaxOutPairSrc,
    pub standalone_src: ShellStandaloneClkSrc,
    pub standalone_rate: TcKonnektStandaloneClkRate,
}

impl ShellStandaloneClkSpec for K8Config {
    const STANDALONE_CLOCK_SOURCES: &'static [ShellStandaloneClkSrc] = &[
        ShellStandaloneClkSrc::Coaxial,
        ShellStandaloneClkSrc::Internal,
    ];
}

impl TcKonnektSegmentData for K8Config {
    fn build(&self, raw: &mut [u8]) {
        self.coax_out_src.0.build_quadlet(&mut raw[12..16]);
        self.standalone_src.build_quadlet(&mut raw[20..24]);
        self.standalone_rate.build_quadlet(&mut raw[24..28]);
    }

    fn parse(&mut self, raw: &[u8]) {
        self.coax_out_src.0.parse_quadlet(&raw[12..16]);
        self.standalone_src.parse_quadlet(&raw[20..24]);
        self.standalone_rate.parse_quadlet(&raw[24..28]);
    }
}

impl TcKonnektSegmentSpec for TcKonnektSegment<K8Config> {
    const OFFSET: usize = 0x0028;
    const SIZE: usize = 76;
}

impl TcKonnektNotifiedSegmentSpec for TcKonnektSegment<K8Config> {
    const NOTIFY_FLAG: u32 = SHELL_CONFIG_NOTIFY_FLAG;
}

/// The structureto represent state of mixer.
#[derive(Debug)]
pub struct K8MixerState {
    /// The common structure for state of mixer.
    pub mixer: ShellMixerState,
    /// Whether to use mixer function.
    pub enabled: bool,
}

impl Default for K8MixerState {
    fn default() -> Self {
        K8MixerState {
            mixer: Self::create_mixer_state(),
            enabled: Default::default(),
        }
    }
}

impl ShellMixerStateConvert for K8MixerState {
    const MONITOR_SRC_MAP: [Option<ShellMixerMonitorSrcType>; SHELL_MIXER_MONITOR_SRC_COUNT] = [
        Some(ShellMixerMonitorSrcType::Stream),
        None,
        None,
        None,
        Some(ShellMixerMonitorSrcType::Analog),
        None,
        None,
        None,
        None,
        Some(ShellMixerMonitorSrcType::Spdif),
    ];

    fn state(&self) -> &ShellMixerState {
        &self.mixer
    }

    fn state_mut(&mut self) -> &mut ShellMixerState {
        &mut self.mixer
    }
}

impl TcKonnektSegmentData for K8MixerState {
    fn build(&self, raw: &mut [u8]) {
        ShellMixerStateConvert::build(self, raw);

        self.enabled.build_quadlet(&mut raw[340..344]);
    }

    fn parse(&mut self, raw: &[u8]) {
        ShellMixerStateConvert::parse(self, raw);

        self.enabled.parse_quadlet(&raw[340..344]);
    }
}

impl TcKonnektSegmentSpec for TcKonnektSegment<K8MixerState> {
    const OFFSET: usize = 0x0074;
    const SIZE: usize = ShellMixerState::SIZE + 32;
}

impl TcKonnektNotifiedSegmentSpec for TcKonnektSegment<K8MixerState> {
    const NOTIFY_FLAG: u32 = SHELL_MIXER_NOTIFY_FLAG;
}

#[derive(Default, Debug)]
pub struct K8HwState {
    pub hw_state: ShellHwState,
    pub aux_input_enabled: bool,
}

impl TcKonnektSegmentData for K8HwState {
    fn build(&self, raw: &mut [u8]) {
        self.hw_state.build(raw);
        self.aux_input_enabled.build_quadlet(&mut raw[8..12]);
    }

    fn parse(&mut self, raw: &[u8]) {
        self.hw_state.parse(raw);
        self.aux_input_enabled.parse_quadlet(&raw[8..12]);
    }
}

impl TcKonnektSegmentSpec for TcKonnektSegment<K8HwState> {
    const OFFSET: usize = 0x100c;
    const SIZE: usize = ShellHwState::SIZE;
}

impl TcKonnektNotifiedSegmentSpec for TcKonnektSegment<K8HwState> {
    const NOTIFY_FLAG: u32 = SHELL_HW_STATE_NOTIFY_FLAG;
}

const K8_METER_ANALOG_INPUT_COUNT: usize = 2;
const K8_METER_DIGITAL_INPUT_COUNT: usize = 2;

#[derive(Debug)]
pub struct K8MixerMeter(pub ShellMixerMeter);

impl Default for K8MixerMeter {
    fn default() -> Self {
        K8MixerMeter(Self::create_meter_state())
    }
}

impl ShellMixerMeterConvert for K8MixerMeter {
    const ANALOG_INPUT_COUNT: usize = K8_METER_ANALOG_INPUT_COUNT;
    const DIGITAL_INPUT_COUNT: usize = K8_METER_DIGITAL_INPUT_COUNT;

    fn meter(&self) -> &ShellMixerMeter {
        &self.0
    }

    fn meter_mut(&mut self) -> &mut ShellMixerMeter {
        &mut self.0
    }
}

impl TcKonnektSegmentData for K8MixerMeter {
    fn build(&self, raw: &mut [u8]) {
        ShellMixerMeterConvert::build(self, raw)
    }

    fn parse(&mut self, raw: &[u8]) {
        ShellMixerMeterConvert::parse(self, raw)
    }
}

impl TcKonnektSegmentSpec for TcKonnektSegment<K8MixerMeter> {
    const OFFSET: usize = 0x105c;
    const SIZE: usize = ShellMixerMeter::SIZE;
}
