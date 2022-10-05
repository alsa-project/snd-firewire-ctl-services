// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Protocol defined by TC Electronic for Impact Twin.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by TC Electronic for Impact Twin.

use super::*;

/// Protocol implementation of Impact Twin.
#[derive(Default, Debug)]
pub struct ItwinProtocol;

impl TcatOperation for ItwinProtocol {}

impl TcatGlobalSectionSpecification for ItwinProtocol {}

/// Segment for knob. 0x0000..0x0027 (36 quads).
pub type ItwinKnobSegment = TcKonnektSegment<ItwinKnob>;

/// Segment for configuration. 0x0028..0x00cf (168 quads).
pub type ItwinConfigSegment = TcKonnektSegment<ItwinConfig>;

/// Segment for state of mixer. 0x00d0..0x0243 (93 quads).
pub type ItwinMixerStateSegment = TcKonnektSegment<ItwinMixerState>;

/// Segment for state of reverb effect. 0x0244..0x0287. (17 quads)
pub type ItwinReverbStateSegment = TcKonnektSegment<ItwinReverbState>;

/// Segment for states of channel strip effect. 0x028c..0x03ab (72 quads).
pub type ItwinChStripStatesSegment = TcKonnektSegment<ItwinChStripStates>;

// NOTE: Segment for tuner. 0x03ac..0x03c8 (8 quads).

/// Segment for mixer meter. 0x106c..0x10c7 (23 quads).
pub type ItwinMixerMeterSegment = TcKonnektSegment<ItwinMixerMeter>;

/// Segment for state of hardware. 0x1008..0x1023 (7 quads).
pub type ItwinHwStateSegment = TcKonnektSegment<ItwinHwState>;

/// Segment for meter of reverb effect. 0x10c8..0x010df (6 quads).
pub type ItwinReverbMeterSegment = TcKonnektSegment<ItwinReverbMeter>;

/// Segment for meters of channel strip effect. 0x10e0..0x111b (15 quads).
pub type ItwinChStripMetersSegment = TcKonnektSegment<ItwinChStripMeters>;

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

segment_default!(ItwinProtocol, ItwinKnob);
segment_default!(ItwinProtocol, ItwinConfig);
segment_default!(ItwinProtocol, ItwinMixerState);
segment_default!(ItwinProtocol, ItwinReverbState);
segment_default!(ItwinProtocol, ItwinChStripStates);
segment_default!(ItwinProtocol, ItwinMixerMeter);
segment_default!(ItwinProtocol, ItwinHwState);
segment_default!(ItwinProtocol, ItwinReverbMeter);
segment_default!(ItwinProtocol, ItwinChStripMeters);

/// State of knob.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ItwinKnob {
    pub target: ShellKnob0Target,
    pub clock_recovery: bool,
}

impl Default for ItwinKnob {
    fn default() -> Self {
        Self {
            target: <ItwinProtocol as ShellKnob0TargetSpecification>::KNOB0_TARGETS[0],
            clock_recovery: Default::default(),
        }
    }
}

impl ShellKnob0TargetSpecification for ItwinProtocol {
    const KNOB0_TARGETS: &'static [ShellKnob0Target] = &[
        ShellKnob0Target::ChannelStrip0,
        ShellKnob0Target::ChannelStrip1,
        ShellKnob0Target::Reverb,
        ShellKnob0Target::Mixer,
    ];
}

impl TcKonnektSegmentSerdes<ItwinKnob> for ItwinProtocol {
    const NAME: &'static str = "knob";
    const OFFSET: usize = 0x0004;
    const SIZE: usize = SHELL_KNOB_SEGMENT_SIZE;

    fn serialize(params: &ItwinKnob, raw: &mut [u8]) -> Result<(), String> {
        serialize_knob0_target::<ItwinProtocol>(&params.target, &mut raw[..4])?;
        params.clock_recovery.build_quadlet(&mut raw[8..12]);
        Ok(())
    }

    fn deserialize(params: &mut ItwinKnob, raw: &[u8]) -> Result<(), String> {
        deserialize_knob0_target::<ItwinProtocol>(&mut params.target, &raw[..4])?;
        params.clock_recovery.parse_quadlet(&raw[8..12]);
        Ok(())
    }
}

impl TcKonnektMutableSegmentOperation<ItwinKnob> for ItwinProtocol {}

impl TcKonnektNotifiedSegmentOperation<ItwinKnob> for ItwinProtocol {
    const NOTIFY_FLAG: u32 = SHELL_KNOB_NOTIFY_FLAG;
}

/// The number of pair for physical output.
pub const ITWIN_PHYS_OUT_PAIR_COUNT: usize = 7;

/// Source of stream for mixer.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ItwinOutputPairSrc {
    MixerOut01,
    Analog01,
    Analog23,
    Spdif01,
    Adat01,
    Adat23,
    Adat45,
    Adat67,
    Stream01,
    Stream23,
    Stream45,
    Stream67,
    Stream89,
    Stream1011,
    Stream1213,
    MixerSend01,
}

impl Default for ItwinOutputPairSrc {
    fn default() -> Self {
        Self::MixerOut01
    }
}

impl From<ItwinOutputPairSrc> for u32 {
    fn from(src: ItwinOutputPairSrc) -> Self {
        match src {
            ItwinOutputPairSrc::MixerSend01 => 15,
            ItwinOutputPairSrc::Stream1213 => 14,
            ItwinOutputPairSrc::Stream1011 => 13,
            ItwinOutputPairSrc::Stream89 => 12,
            ItwinOutputPairSrc::Stream67 => 11,
            ItwinOutputPairSrc::Stream45 => 10,
            ItwinOutputPairSrc::Stream23 => 9,
            ItwinOutputPairSrc::Stream01 => 8,
            ItwinOutputPairSrc::Adat67 => 7,
            ItwinOutputPairSrc::Adat45 => 6,
            ItwinOutputPairSrc::Adat23 => 5,
            ItwinOutputPairSrc::Adat01 => 4,
            ItwinOutputPairSrc::Spdif01 => 3,
            ItwinOutputPairSrc::Analog23 => 2,
            ItwinOutputPairSrc::Analog01 => 1,
            ItwinOutputPairSrc::MixerOut01 => 0,
        }
    }
}

impl From<u32> for ItwinOutputPairSrc {
    fn from(val: u32) -> Self {
        match val {
            15 => ItwinOutputPairSrc::MixerSend01,
            14 => ItwinOutputPairSrc::Stream1213,
            13 => ItwinOutputPairSrc::Stream1011,
            12 => ItwinOutputPairSrc::Stream89,
            11 => ItwinOutputPairSrc::Stream67,
            10 => ItwinOutputPairSrc::Stream45,
            9 => ItwinOutputPairSrc::Stream23,
            8 => ItwinOutputPairSrc::Stream01,
            7 => ItwinOutputPairSrc::Adat67,
            6 => ItwinOutputPairSrc::Adat45,
            5 => ItwinOutputPairSrc::Adat23,
            4 => ItwinOutputPairSrc::Adat01,
            3 => ItwinOutputPairSrc::Spdif01,
            2 => ItwinOutputPairSrc::Analog23,
            1 => ItwinOutputPairSrc::Analog01,
            _ => ItwinOutputPairSrc::MixerOut01,
        }
    }
}

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct ItwinConfig {
    pub mixer_stream_src_pair: ShellMixerStreamSourcePair,
    pub standalone_src: ShellStandaloneClockSource,
    pub standalone_rate: TcKonnektStandaloneClockRate,
    pub output_pair_src: [ItwinOutputPairSrc; ITWIN_PHYS_OUT_PAIR_COUNT],
}

impl ShellMixerStreamSourcePairSpecification for ItwinProtocol {
    const MIXER_STREAM_SOURCE_PAIRS: &'static [ShellMixerStreamSourcePair] = &[
        ShellMixerStreamSourcePair::Stream0_1,
        ShellMixerStreamSourcePair::Stream2_3,
        ShellMixerStreamSourcePair::Stream4_5,
        ShellMixerStreamSourcePair::Stream6_7,
        ShellMixerStreamSourcePair::Stream8_9,
        ShellMixerStreamSourcePair::Stream10_11,
        ShellMixerStreamSourcePair::Stream12_13,
    ];
}

impl ShellStandaloneClockSpecification for ItwinProtocol {
    const STANDALONE_CLOCK_SOURCES: &'static [ShellStandaloneClockSource] = &[
        ShellStandaloneClockSource::Optical,
        ShellStandaloneClockSource::Coaxial,
        ShellStandaloneClockSource::Internal,
    ];
}

impl TcKonnektSegmentSerdes<ItwinConfig> for ItwinProtocol {
    const NAME: &'static str = "configuration";
    const OFFSET: usize = 0x0028;
    const SIZE: usize = 168;

    fn serialize(params: &ItwinConfig, raw: &mut [u8]) -> Result<(), String> {
        serialize_mixer_stream_source_pair::<ItwinProtocol>(
            &params.mixer_stream_src_pair,
            &mut raw[24..28],
        )?;
        serialize_standalone_clock_source::<ItwinProtocol>(
            &params.standalone_src,
            &mut raw[28..32],
        )?;
        serialize_standalone_clock_rate(&params.standalone_rate, &mut raw[32..36])?;
        params
            .output_pair_src
            .build_quadlet_block(&mut raw[120..148]);
        Ok(())
    }

    fn deserialize(params: &mut ItwinConfig, raw: &[u8]) -> Result<(), String> {
        deserialize_mixer_stream_source_pair::<ItwinProtocol>(
            &mut params.mixer_stream_src_pair,
            &raw[24..28],
        )?;
        deserialize_standalone_clock_source::<ItwinProtocol>(
            &mut params.standalone_src,
            &raw[28..32],
        )?;
        deserialize_standalone_clock_rate(&mut params.standalone_rate, &raw[32..36])?;
        params.output_pair_src.parse_quadlet_block(&raw[120..148]);
        Ok(())
    }
}

impl TcKonnektMutableSegmentOperation<ItwinConfig> for ItwinProtocol {}

impl TcKonnektNotifiedSegmentOperation<ItwinConfig> for ItwinProtocol {
    const NOTIFY_FLAG: u32 = SHELL_CONFIG_NOTIFY_FLAG;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItwinMixerState {
    pub mixer: ShellMixerState,
    /// The balance between analog and stream inputs to mix. 0..1000.
    pub stream_mix_balance: u32,
    pub enabled: bool,
}

impl Default for ItwinMixerState {
    fn default() -> Self {
        ItwinMixerState {
            mixer: Self::create_mixer_state(),
            enabled: Default::default(),
            stream_mix_balance: Default::default(),
        }
    }
}

impl ShellMixerStateConvert for ItwinMixerState {
    const MONITOR_SRC_MAP: [Option<ShellMixerMonitorSrcType>; SHELL_MIXER_MONITOR_SRC_COUNT] = [
        Some(ShellMixerMonitorSrcType::Stream),
        None,
        None,
        Some(ShellMixerMonitorSrcType::Spdif),
        Some(ShellMixerMonitorSrcType::Analog),
        Some(ShellMixerMonitorSrcType::Analog),
        Some(ShellMixerMonitorSrcType::AdatSpdif),
        Some(ShellMixerMonitorSrcType::Adat),
        Some(ShellMixerMonitorSrcType::Adat),
        Some(ShellMixerMonitorSrcType::Adat),
    ];

    fn state(&self) -> &ShellMixerState {
        &self.mixer
    }

    fn state_mut(&mut self) -> &mut ShellMixerState {
        &mut self.mixer
    }
}

impl TcKonnektSegmentSerdes<ItwinMixerState> for ItwinProtocol {
    const NAME: &'static str = "mixer-state";
    const OFFSET: usize = 0x00d0;
    const SIZE: usize = ShellMixerState::SIZE + 56;

    fn serialize(params: &ItwinMixerState, raw: &mut [u8]) -> Result<(), String> {
        ShellMixerStateConvert::build(params, raw);

        params.stream_mix_balance.build_quadlet(&mut raw[348..352]);
        params.enabled.build_quadlet(&mut raw[352..356]);
        Ok(())
    }

    fn deserialize(params: &mut ItwinMixerState, raw: &[u8]) -> Result<(), String> {
        ShellMixerStateConvert::parse(params, raw);

        params.stream_mix_balance.parse_quadlet(&raw[348..352]);
        params.enabled.parse_quadlet(&raw[352..356]);
        Ok(())
    }
}

impl TcKonnektMutableSegmentOperation<ItwinMixerState> for ItwinProtocol {}

impl TcKonnektNotifiedSegmentOperation<ItwinMixerState> for ItwinProtocol {
    const NOTIFY_FLAG: u32 = SHELL_MIXER_NOTIFY_FLAG;
}

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct ItwinReverbState(pub ReverbState);

impl TcKonnektSegmentSerdes<ItwinReverbState> for ItwinProtocol {
    const NAME: &'static str = "reverb-state";
    const OFFSET: usize = 0x0244;
    const SIZE: usize = ReverbState::SIZE;

    fn serialize(params: &ItwinReverbState, raw: &mut [u8]) -> Result<(), String> {
        params.0.build(raw);
        Ok(())
    }

    fn deserialize(params: &mut ItwinReverbState, raw: &[u8]) -> Result<(), String> {
        params.0.parse(raw);
        Ok(())
    }
}

impl TcKonnektMutableSegmentOperation<ItwinReverbState> for ItwinProtocol {}

impl TcKonnektNotifiedSegmentOperation<ItwinReverbState> for ItwinProtocol {
    const NOTIFY_FLAG: u32 = SHELL_REVERB_NOTIFY_FLAG;
}

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct ItwinChStripStates(pub [ChStripState; SHELL_CH_STRIP_COUNT]);

impl TcKonnektSegmentSerdes<ItwinChStripStates> for ItwinProtocol {
    const NAME: &'static str = "channel-strip-state";
    const OFFSET: usize = 0x028c;
    const SIZE: usize = ChStripState::SIZE * SHELL_CH_STRIP_COUNT + 4;

    fn serialize(params: &ItwinChStripStates, raw: &mut [u8]) -> Result<(), String> {
        params.0.build(raw);
        Ok(())
    }

    fn deserialize(params: &mut ItwinChStripStates, raw: &[u8]) -> Result<(), String> {
        params.0.parse(raw);
        Ok(())
    }
}

impl TcKonnektMutableSegmentOperation<ItwinChStripStates> for ItwinProtocol {}

impl TcKonnektNotifiedSegmentOperation<ItwinChStripStates> for ItwinProtocol {
    const NOTIFY_FLAG: u32 = SHELL_CH_STRIP_NOTIFY_FLAG;
}

/// The mode to listen for analog outputs.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ListeningMode {
    Monaural,
    Stereo,
    Side,
}

impl ListeningMode {
    const MONAURAL: u32 = 0x00;
    const STEREO: u32 = 0x01;
    const SIDE: u32 = 0x02;
}

impl Default for ListeningMode {
    fn default() -> Self {
        Self::Monaural
    }
}

impl From<u32> for ListeningMode {
    fn from(val: u32) -> Self {
        match val & 0x03 {
            Self::STEREO => Self::Stereo,
            Self::SIDE => Self::Side,
            _ => Self::Monaural,
        }
    }
}

impl From<ListeningMode> for u32 {
    fn from(mode: ListeningMode) -> u32 {
        match mode {
            ListeningMode::Monaural => ListeningMode::MONAURAL,
            ListeningMode::Stereo => ListeningMode::STEREO,
            ListeningMode::Side => ListeningMode::SIDE,
        }
    }
}

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct ItwinHwState {
    pub hw_state: ShellHwState,
    pub listening_mode: ListeningMode,
}

impl TcKonnektSegmentSerdes<ItwinHwState> for ItwinProtocol {
    const NAME: &'static str = "hardware-state";
    const OFFSET: usize = 0x1008;
    const SIZE: usize = ShellHwState::SIZE;

    fn serialize(params: &ItwinHwState, raw: &mut [u8]) -> Result<(), String> {
        params.hw_state.build(raw);
        params.listening_mode.build_quadlet(&mut raw[8..12]);
        Ok(())
    }

    fn deserialize(params: &mut ItwinHwState, raw: &[u8]) -> Result<(), String> {
        params.hw_state.parse(raw);
        params.listening_mode.parse_quadlet(&raw[8..12]);
        Ok(())
    }
}

impl TcKonnektMutableSegmentOperation<ItwinHwState> for ItwinProtocol {}

impl TcKonnektNotifiedSegmentOperation<ItwinHwState> for ItwinProtocol {
    const NOTIFY_FLAG: u32 = SHELL_HW_STATE_NOTIFY_FLAG;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItwinMixerMeter(pub ShellMixerMeter);

impl Default for ItwinMixerMeter {
    fn default() -> Self {
        ItwinMixerMeter(Self::create_meter_state())
    }
}

impl ShellMixerMeterConvert for ItwinMixerMeter {
    const ANALOG_INPUT_COUNT: usize = 4;
    const DIGITAL_INPUT_COUNT: usize = 8;

    fn meter(&self) -> &ShellMixerMeter {
        &self.0
    }

    fn meter_mut(&mut self) -> &mut ShellMixerMeter {
        &mut self.0
    }
}

impl TcKonnektSegmentSerdes<ItwinMixerMeter> for ItwinProtocol {
    const NAME: &'static str = "mixer-meter";
    const OFFSET: usize = 0x106c;
    const SIZE: usize = ShellMixerMeter::SIZE;

    fn serialize(params: &ItwinMixerMeter, raw: &mut [u8]) -> Result<(), String> {
        ShellMixerMeterConvert::build(params, raw);
        Ok(())
    }

    fn deserialize(params: &mut ItwinMixerMeter, raw: &[u8]) -> Result<(), String> {
        ShellMixerMeterConvert::parse(params, raw);
        Ok(())
    }
}

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct ItwinReverbMeter(pub ReverbMeter);

impl TcKonnektSegmentSerdes<ItwinReverbMeter> for ItwinProtocol {
    const NAME: &'static str = "reverb-meter";
    const OFFSET: usize = 0x10c8;
    const SIZE: usize = ReverbMeter::SIZE;

    fn serialize(params: &ItwinReverbMeter, raw: &mut [u8]) -> Result<(), String> {
        params.0.build(raw);
        Ok(())
    }

    fn deserialize(params: &mut ItwinReverbMeter, raw: &[u8]) -> Result<(), String> {
        params.0.parse(raw);
        Ok(())
    }
}

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct ItwinChStripMeters(pub [ChStripMeter; SHELL_CH_STRIP_COUNT]);

impl TcKonnektSegmentSerdes<ItwinChStripMeters> for ItwinProtocol {
    const NAME: &'static str = "channel-strip-meter";
    const OFFSET: usize = 0x10e0;
    const SIZE: usize = ChStripMeter::SIZE * SHELL_CH_STRIP_COUNT + 4;

    fn serialize(params: &ItwinChStripMeters, raw: &mut [u8]) -> Result<(), String> {
        params.0.build(raw);
        Ok(())
    }

    fn deserialize(params: &mut ItwinChStripMeters, raw: &[u8]) -> Result<(), String> {
        params.0.parse(raw);
        Ok(())
    }
}
