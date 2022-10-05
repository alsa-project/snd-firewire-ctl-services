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

/// Segment for configuration. 0x0028..0x0073 (19 quads).
pub type K8ConfigSegment = TcKonnektSegment<K8Config>;

/// Segment for state of mixer. 0x0074..0x01cf (87 quads).
pub type K8MixerStateSegment = TcKonnektSegment<K8MixerState>;

/// Segment for mixer meter. 0x105c..0x10b7 (23 quads).
pub type K8MixerMeterSegment = TcKonnektSegment<K8MixerMeter>;

/// Segment tor state of hardware. 0x100c..0x1027 (7 quads).
pub type K8HwStateSegment = TcKonnektSegment<K8HwState>;

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
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct K8Knob {
    pub knob0_target: ShellKnob0Target,
    pub knob1_target: ShellKnob1Target,
}

impl Default for K8Knob {
    fn default() -> Self {
        Self {
            knob0_target: K8Protocol::KNOB0_TARGETS[0],
            knob1_target: K8Protocol::KNOB1_TARGETS[0],
        }
    }
}

impl ShellKnob0TargetSpecification for K8Protocol {
    const KNOB0_TARGETS: &'static [ShellKnob0Target] = &[
        ShellKnob0Target::Analog0,
        ShellKnob0Target::Analog1,
        ShellKnob0Target::Spdif0_1,
        ShellKnob0Target::Configurable,
    ];
}

impl ShellKnob1TargetSpecification for K8Protocol {
    const KNOB1_TARGETS: &'static [ShellKnob1Target] =
        &[ShellKnob1Target::Stream, ShellKnob1Target::Mixer];
}

impl TcKonnektSegmentSerdes<K8Knob> for K8Protocol {
    const NAME: &'static str = "knob";
    const OFFSET: usize = 0x0004;
    const SIZE: usize = SHELL_KNOB_SEGMENT_SIZE;

    fn serialize(params: &K8Knob, raw: &mut [u8]) -> Result<(), String> {
        serialize_knob0_target::<K8Protocol>(&params.knob0_target, &mut raw[..4])?;
        serialize_knob1_target::<K8Protocol>(&params.knob1_target, &mut raw[4..8])?;
        Ok(())
    }

    fn deserialize(params: &mut K8Knob, raw: &[u8]) -> Result<(), String> {
        deserialize_knob0_target::<K8Protocol>(&mut params.knob0_target, &raw[..4])?;
        deserialize_knob1_target::<K8Protocol>(&mut params.knob1_target, &raw[4..8])?;
        Ok(())
    }
}

impl TcKonnektMutableSegmentOperation<K8Knob> for K8Protocol {}

impl TcKonnektNotifiedSegmentOperation<K8Knob> for K8Protocol {
    const NOTIFY_FLAG: u32 = SHELL_KNOB_NOTIFY_FLAG;
}

/// Configuration.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct K8Config {
    pub coax_out_src: ShellCoaxOutPairSrc,
    pub standalone_src: ShellStandaloneClockSource,
    pub standalone_rate: TcKonnektStandaloneClockRate,
}

impl ShellStandaloneClockSpecification for K8Protocol {
    const STANDALONE_CLOCK_SOURCES: &'static [ShellStandaloneClockSource] = &[
        ShellStandaloneClockSource::Coaxial,
        ShellStandaloneClockSource::Internal,
    ];
}

impl TcKonnektSegmentSerdes<K8Config> for K8Protocol {
    const NAME: &'static str = "configuration";
    const OFFSET: usize = 0x0028;
    const SIZE: usize = 76;

    fn serialize(params: &K8Config, raw: &mut [u8]) -> Result<(), String> {
        serialize_coax_out_pair_source(&params.coax_out_src, &mut raw[12..16])?;
        serialize_standalone_clock_source::<K8Protocol>(&params.standalone_src, &mut raw[20..24])?;
        serialize_standalone_clock_rate(&params.standalone_rate, &mut raw[24..28])?;
        Ok(())
    }

    fn deserialize(params: &mut K8Config, raw: &[u8]) -> Result<(), String> {
        deserialize_coax_out_pair_source(&mut params.coax_out_src, &raw[12..16])?;
        deserialize_standalone_clock_source::<K8Protocol>(
            &mut params.standalone_src,
            &raw[20..24],
        )?;
        deserialize_standalone_clock_rate(&mut params.standalone_rate, &raw[24..28])?;
        Ok(())
    }
}

impl TcKonnektMutableSegmentOperation<K8Config> for K8Protocol {}

impl TcKonnektNotifiedSegmentOperation<K8Config> for K8Protocol {
    const NOTIFY_FLAG: u32 = SHELL_CONFIG_NOTIFY_FLAG;
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
        serialize_hw_state(&params.hw_state, raw)?;
        params.aux_input_enabled.build_quadlet(&mut raw[8..12]);
        Ok(())
    }

    fn deserialize(params: &mut K8HwState, raw: &[u8]) -> Result<(), String> {
        deserialize_hw_state(&mut params.hw_state, raw)?;
        params.aux_input_enabled.parse_quadlet(&raw[8..12]);
        Ok(())
    }
}

impl TcKonnektMutableSegmentOperation<K8HwState> for K8Protocol {}

impl TcKonnektNotifiedSegmentOperation<K8HwState> for K8Protocol {
    const NOTIFY_FLAG: u32 = SHELL_HW_STATE_NOTIFY_FLAG;
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
