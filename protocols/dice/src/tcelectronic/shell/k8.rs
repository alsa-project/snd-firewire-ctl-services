// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Protocol defined by TC Electronic for Konnekt 8.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by TC Electronic for Konnekt 8.

use super::*;

/// Protocol implementation of Konnekt 8.
#[derive(Default, Debug)]
pub struct K8Protocol;

impl TcatOperation for K8Protocol {}

impl TcatGlobalSectionSpecification for K8Protocol {}

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

macro_rules! segment_default {
    ($p:ty, $t:ty) => {
        impl Default for TcKonnektSegment<$t> {
            fn default() -> Self {
                Self {
                    data: <$t>::default(),
                    raw: vec![0; <$p as TcKonnektSegmentSerdes<$t>>::SIZE],
                }
            }
        }
    };
}

segment_default!(K8Protocol, K8Knob);
segment_default!(K8Protocol, K8Config);
segment_default!(K8Protocol, K8MixerState);
segment_default!(K8Protocol, K8MixerMeter);
segment_default!(K8Protocol, K8HwState);

/// State of knob.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
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

impl TcKonnektSegmentSerdes<K8Knob> for K8Protocol {
    const NAME: &'static str = "knob";
    const OFFSET: usize = 0x0004;
    const SIZE: usize = SHELL_KNOB_SIZE;

    fn serialize(params: &K8Knob, raw: &mut [u8]) -> Result<(), String> {
        params.target.0.build_quadlet(&mut raw[..4]);
        params.knob2_target.0.build_quadlet(&mut raw[4..8]);
        Ok(())
    }

    fn deserialize(params: &mut K8Knob, raw: &[u8]) -> Result<(), String> {
        params.target.0.parse_quadlet(&raw[..4]);
        params.knob2_target.0.parse_quadlet(&raw[4..8]);
        Ok(())
    }
}

impl TcKonnektMutableSegmentOperation<K8Knob> for K8Protocol {}

impl TcKonnektNotifiedSegmentOperation<K8Knob> for K8Protocol {
    const NOTIFY_FLAG: u32 = SHELL_KNOB_NOTIFY_FLAG;
}

impl TcKonnektSegmentData for K8Knob {
    fn build(&self, raw: &mut [u8]) {
        let _ = <K8Protocol as TcKonnektSegmentSerdes<K8Knob>>::serialize(self, raw);
    }

    fn parse(&mut self, raw: &[u8]) {
        let _ = <K8Protocol as TcKonnektSegmentSerdes<K8Knob>>::deserialize(self, raw);
    }
}

impl TcKonnektSegmentSpec for TcKonnektSegment<K8Knob> {
    const OFFSET: usize = <K8Protocol as TcKonnektSegmentSerdes<K8Knob>>::OFFSET;
    const SIZE: usize = <K8Protocol as TcKonnektSegmentSerdes<K8Knob>>::SIZE;
}

impl TcKonnektNotifiedSegmentSpec for TcKonnektSegment<K8Knob> {
    const NOTIFY_FLAG: u32 = <K8Protocol as TcKonnektNotifiedSegmentOperation<K8Knob>>::NOTIFY_FLAG;
}

/// Configuration.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
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

impl TcKonnektSegmentSerdes<K8Config> for K8Protocol {
    const NAME: &'static str = "configuration";
    const OFFSET: usize = 0x0028;
    const SIZE: usize = 76;

    fn serialize(params: &K8Config, raw: &mut [u8]) -> Result<(), String> {
        params.coax_out_src.0.build_quadlet(&mut raw[12..16]);
        params.standalone_src.build_quadlet(&mut raw[20..24]);
        params.standalone_rate.build_quadlet(&mut raw[24..28]);
        Ok(())
    }

    fn deserialize(params: &mut K8Config, raw: &[u8]) -> Result<(), String> {
        params.coax_out_src.0.parse_quadlet(&raw[12..16]);
        params.standalone_src.parse_quadlet(&raw[20..24]);
        params.standalone_rate.parse_quadlet(&raw[24..28]);
        Ok(())
    }
}

impl TcKonnektMutableSegmentOperation<K8Config> for K8Protocol {}

impl TcKonnektNotifiedSegmentOperation<K8Config> for K8Protocol {
    const NOTIFY_FLAG: u32 = SHELL_CONFIG_NOTIFY_FLAG;
}

impl TcKonnektSegmentData for K8Config {
    fn build(&self, raw: &mut [u8]) {
        let _ = <K8Protocol as TcKonnektSegmentSerdes<K8Config>>::serialize(self, raw);
    }

    fn parse(&mut self, raw: &[u8]) {
        let _ = <K8Protocol as TcKonnektSegmentSerdes<K8Config>>::deserialize(self, raw);
    }
}

impl TcKonnektSegmentSpec for TcKonnektSegment<K8Config> {
    const OFFSET: usize = <K8Protocol as TcKonnektSegmentSerdes<K8Config>>::OFFSET;
    const SIZE: usize = <K8Protocol as TcKonnektSegmentSerdes<K8Config>>::SIZE;
}

impl TcKonnektNotifiedSegmentSpec for TcKonnektSegment<K8Config> {
    const NOTIFY_FLAG: u32 =
        <K8Protocol as TcKonnektNotifiedSegmentOperation<K8Config>>::NOTIFY_FLAG;
}

/// State of mixer.
#[derive(Debug, Clone, PartialEq, Eq)]
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

impl TcKonnektSegmentSerdes<K8MixerState> for K8Protocol {
    const NAME: &'static str = "mixer-state";
    const OFFSET: usize = 0x0074;
    const SIZE: usize = ShellMixerState::SIZE + 32;

    fn serialize(params: &K8MixerState, raw: &mut [u8]) -> Result<(), String> {
        ShellMixerStateConvert::build(params, raw);
        params.enabled.build_quadlet(&mut raw[340..344]);
        Ok(())
    }

    fn deserialize(params: &mut K8MixerState, raw: &[u8]) -> Result<(), String> {
        ShellMixerStateConvert::parse(params, raw);
        params.enabled.parse_quadlet(&raw[340..344]);
        Ok(())
    }
}

impl TcKonnektMutableSegmentOperation<K8MixerState> for K8Protocol {}

impl TcKonnektNotifiedSegmentOperation<K8MixerState> for K8Protocol {
    const NOTIFY_FLAG: u32 = SHELL_MIXER_NOTIFY_FLAG;
}

impl TcKonnektSegmentData for K8MixerState {
    fn build(&self, raw: &mut [u8]) {
        let _ = <K8Protocol as TcKonnektSegmentSerdes<K8MixerState>>::serialize(self, raw);
    }

    fn parse(&mut self, raw: &[u8]) {
        let _ = <K8Protocol as TcKonnektSegmentSerdes<K8MixerState>>::deserialize(self, raw);
    }
}

impl TcKonnektSegmentSpec for TcKonnektSegment<K8MixerState> {
    const OFFSET: usize = <K8Protocol as TcKonnektSegmentSerdes<K8MixerState>>::OFFSET;
    const SIZE: usize = <K8Protocol as TcKonnektSegmentSerdes<K8MixerState>>::SIZE;
}

impl TcKonnektNotifiedSegmentSpec for TcKonnektSegment<K8MixerState> {
    const NOTIFY_FLAG: u32 =
        <K8Protocol as TcKonnektNotifiedSegmentOperation<K8MixerState>>::NOTIFY_FLAG;
}

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct K8HwState {
    pub hw_state: ShellHwState,
    pub aux_input_enabled: bool,
}

impl TcKonnektSegmentSerdes<K8HwState> for K8Protocol {
    const NAME: &'static str = "hardware-state";
    const OFFSET: usize = 0x100c;
    const SIZE: usize = ShellHwState::SIZE;

    fn serialize(params: &K8HwState, raw: &mut [u8]) -> Result<(), String> {
        params.hw_state.build(raw);
        params.aux_input_enabled.build_quadlet(&mut raw[8..12]);
        Ok(())
    }

    fn deserialize(params: &mut K8HwState, raw: &[u8]) -> Result<(), String> {
        params.hw_state.parse(raw);
        params.aux_input_enabled.parse_quadlet(&raw[8..12]);
        Ok(())
    }
}

impl TcKonnektMutableSegmentOperation<K8HwState> for K8Protocol {}

impl TcKonnektNotifiedSegmentOperation<K8HwState> for K8Protocol {
    const NOTIFY_FLAG: u32 = SHELL_HW_STATE_NOTIFY_FLAG;
}

impl TcKonnektSegmentData for K8HwState {
    fn build(&self, raw: &mut [u8]) {
        let _ = <K8Protocol as TcKonnektSegmentSerdes<K8HwState>>::serialize(self, raw);
    }

    fn parse(&mut self, raw: &[u8]) {
        let _ = <K8Protocol as TcKonnektSegmentSerdes<K8HwState>>::deserialize(self, raw);
    }
}

impl TcKonnektSegmentSpec for TcKonnektSegment<K8HwState> {
    const OFFSET: usize = <K8Protocol as TcKonnektSegmentSerdes<K8HwState>>::OFFSET;
    const SIZE: usize = <K8Protocol as TcKonnektSegmentSerdes<K8HwState>>::SIZE;
}

impl TcKonnektNotifiedSegmentSpec for TcKonnektSegment<K8HwState> {
    const NOTIFY_FLAG: u32 =
        <K8Protocol as TcKonnektNotifiedSegmentOperation<K8HwState>>::NOTIFY_FLAG;
}

const K8_METER_ANALOG_INPUT_COUNT: usize = 2;
const K8_METER_DIGITAL_INPUT_COUNT: usize = 2;

#[derive(Debug, Clone, PartialEq, Eq)]
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

impl TcKonnektSegmentSerdes<K8MixerMeter> for K8Protocol {
    const NAME: &'static str = "mixer-meter";
    const OFFSET: usize = 0x100c;
    const SIZE: usize = ShellMixerMeter::SIZE;

    fn serialize(params: &K8MixerMeter, raw: &mut [u8]) -> Result<(), String> {
        ShellMixerMeterConvert::build(params, raw);
        Ok(())
    }

    fn deserialize(params: &mut K8MixerMeter, raw: &[u8]) -> Result<(), String> {
        ShellMixerMeterConvert::parse(params, raw);
        Ok(())
    }
}

impl TcKonnektSegmentData for K8MixerMeter {
    fn build(&self, raw: &mut [u8]) {
        let _ = <K8Protocol as TcKonnektSegmentSerdes<K8MixerMeter>>::serialize(self, raw);
    }

    fn parse(&mut self, raw: &[u8]) {
        let _ = <K8Protocol as TcKonnektSegmentSerdes<K8MixerMeter>>::deserialize(self, raw);
    }
}

impl TcKonnektSegmentSpec for TcKonnektSegment<K8MixerMeter> {
    const OFFSET: usize = <K8Protocol as TcKonnektSegmentSerdes<K8MixerMeter>>::OFFSET;
    const SIZE: usize = <K8Protocol as TcKonnektSegmentSerdes<K8MixerMeter>>::SIZE;
}
