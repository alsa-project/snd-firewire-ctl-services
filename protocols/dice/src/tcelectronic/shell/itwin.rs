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
impl SegmentOperation<ItwinKnob> for ItwinProtocol {}

/// Segment for configuration. 0x0028..0x00cf (168 quads).
pub type ItwinConfigSegment = TcKonnektSegment<ItwinConfig>;
impl SegmentOperation<ItwinConfig> for ItwinProtocol {}

/// Segment for state of mixer. 0x00d0..0x0243 (93 quads).
pub type ItwinMixerStateSegment = TcKonnektSegment<ItwinMixerState>;
impl SegmentOperation<ItwinMixerState> for ItwinProtocol {}

/// Segment for state of reverb effect. 0x0244..0x0287. (17 quads)
pub type ItwinReverbStateSegment = TcKonnektSegment<ItwinReverbState>;
impl SegmentOperation<ItwinReverbState> for ItwinProtocol {}

/// Segment for states of channel strip effect. 0x028c..0x03ab (72 quads).
pub type ItwinChStripStatesSegment = TcKonnektSegment<ItwinChStripStates>;
impl SegmentOperation<ItwinChStripStates> for ItwinProtocol {}

// NOTE: Segment for tuner. 0x03ac..0x03c8 (8 quads).

/// Segment for mixer meter. 0x106c..0x10c7 (23 quads).
pub type ItwinMixerMeterSegment = TcKonnektSegment<ItwinMixerMeter>;
impl SegmentOperation<ItwinMixerMeter> for ItwinProtocol {}

/// Segment for state of hardware. 0x1008..0x1023 (7 quads).
pub type ItwinHwStateSegment = TcKonnektSegment<ItwinHwState>;
impl SegmentOperation<ItwinHwState> for ItwinProtocol {}

/// Segment for meter of reverb effect. 0x10c8..0x010df (6 quads).
pub type ItwinReverbMeterSegment = TcKonnektSegment<ItwinReverbMeter>;
impl SegmentOperation<ItwinReverbMeter> for ItwinProtocol {}

/// Segment for meters of channel strip effect. 0x10e0..0x111b (15 quads).
pub type ItwinChStripMetersSegment = TcKonnektSegment<ItwinChStripMeters>;
impl SegmentOperation<ItwinChStripMeters> for ItwinProtocol {}

/// State of knob.
#[derive(Default, Debug)]
pub struct ItwinKnob {
    pub target: ShellKnobTarget,
    pub clock_recovery: bool,
}

impl TcKonnektSegmentData for ItwinKnob {
    fn build(&self, raw: &mut [u8]) {
        self.target.0.build_quadlet(&mut raw[..4]);
        self.clock_recovery.build_quadlet(&mut raw[8..12]);
    }

    fn parse(&mut self, raw: &[u8]) {
        self.target.0.parse_quadlet(&raw[..4]);
        self.clock_recovery.parse_quadlet(&raw[8..12]);
    }
}

impl ShellKnobTargetSpec for ItwinKnob {
    const HAS_SPDIF: bool = false;
    const HAS_EFFECTS: bool = true;
}

impl TcKonnektSegmentSpec for TcKonnektSegment<ItwinKnob> {
    const OFFSET: usize = 0x0004;
    const SIZE: usize = SHELL_KNOB_SIZE;
}

impl TcKonnektNotifiedSegmentSpec for TcKonnektSegment<ItwinKnob> {
    const NOTIFY_FLAG: u32 = SHELL_KNOB_NOTIFY_FLAG;
}

/// The number of pair for physical output.
pub const ITWIN_PHYS_OUT_PAIR_COUNT: usize = 7;

/// Source of stream for mixer.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
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

#[derive(Default, Debug)]
pub struct ItwinConfig {
    pub mixer_stream_src_pair: ShellMixerStreamSrcPair,
    pub standalone_src: ShellStandaloneClkSrc,
    pub standalone_rate: TcKonnektStandaloneClkRate,
    pub output_pair_src: [ItwinOutputPairSrc; ITWIN_PHYS_OUT_PAIR_COUNT],
}

impl ShellMixerStreamSrcPairSpec for ItwinConfig {
    const MAXIMUM_STREAM_SRC_PAIR_COUNT: usize = 7;
}

impl ShellStandaloneClkSpec for ItwinConfig {
    const STANDALONE_CLOCK_SOURCES: &'static [ShellStandaloneClkSrc] = &[
        ShellStandaloneClkSrc::Optical,
        ShellStandaloneClkSrc::Coaxial,
        ShellStandaloneClkSrc::Internal,
    ];
}

impl TcKonnektSegmentData for ItwinConfig {
    fn build(&self, raw: &mut [u8]) {
        self.mixer_stream_src_pair.build_quadlet(&mut raw[24..28]);
        self.standalone_src.build_quadlet(&mut raw[28..32]);
        self.standalone_rate.build_quadlet(&mut raw[32..36]);
        self.output_pair_src.build_quadlet_block(&mut raw[120..148]);
    }

    fn parse(&mut self, raw: &[u8]) {
        self.mixer_stream_src_pair.parse_quadlet(&raw[24..28]);
        self.standalone_src.parse_quadlet(&raw[28..32]);
        self.standalone_rate.parse_quadlet(&raw[32..36]);
        self.output_pair_src.parse_quadlet_block(&raw[120..148]);
    }
}

impl TcKonnektSegmentSpec for TcKonnektSegment<ItwinConfig> {
    const OFFSET: usize = 0x0028;
    const SIZE: usize = 168;
}

impl TcKonnektNotifiedSegmentSpec for TcKonnektSegment<ItwinConfig> {
    const NOTIFY_FLAG: u32 = SHELL_CONFIG_NOTIFY_FLAG;
}

#[derive(Debug)]
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

impl TcKonnektSegmentData for ItwinMixerState {
    fn build(&self, raw: &mut [u8]) {
        ShellMixerStateConvert::build(self, raw);

        self.stream_mix_balance.build_quadlet(&mut raw[348..352]);
        self.enabled.build_quadlet(&mut raw[352..356]);
    }

    fn parse(&mut self, raw: &[u8]) {
        ShellMixerStateConvert::parse(self, raw);

        self.stream_mix_balance.parse_quadlet(&raw[348..352]);
        self.enabled.parse_quadlet(&raw[352..356]);
    }
}

impl TcKonnektSegmentSpec for TcKonnektSegment<ItwinMixerState> {
    const OFFSET: usize = 0x00d0;
    const SIZE: usize = ShellMixerState::SIZE + 56;
}

impl TcKonnektNotifiedSegmentSpec for TcKonnektSegment<ItwinMixerState> {
    const NOTIFY_FLAG: u32 = SHELL_MIXER_NOTIFY_FLAG;
}

#[derive(Default, Debug)]
pub struct ItwinReverbState(pub ReverbState);

impl TcKonnektSegmentData for ItwinReverbState {
    fn build(&self, raw: &mut [u8]) {
        self.0.build(raw)
    }

    fn parse(&mut self, raw: &[u8]) {
        self.0.parse(raw)
    }
}

impl TcKonnektSegmentSpec for TcKonnektSegment<ItwinReverbState> {
    const OFFSET: usize = 0x0244;
    const SIZE: usize = ReverbState::SIZE;
}

impl TcKonnektNotifiedSegmentSpec for TcKonnektSegment<ItwinReverbState> {
    const NOTIFY_FLAG: u32 = SHELL_REVERB_NOTIFY_FLAG;
}

#[derive(Default, Debug)]
pub struct ItwinChStripStates(pub [ChStripState; SHELL_CH_STRIP_COUNT]);

impl TcKonnektSegmentData for ItwinChStripStates {
    fn build(&self, raw: &mut [u8]) {
        self.0.build(raw)
    }

    fn parse(&mut self, raw: &[u8]) {
        self.0.parse(raw)
    }
}

impl TcKonnektSegmentSpec for TcKonnektSegment<ItwinChStripStates> {
    const OFFSET: usize = 0x028c;
    const SIZE: usize = ChStripState::SIZE * SHELL_CH_STRIP_COUNT + 4;
}

impl TcKonnektNotifiedSegmentSpec for TcKonnektSegment<ItwinChStripStates> {
    const NOTIFY_FLAG: u32 = SHELL_CH_STRIP_NOTIFY_FLAG;
}

/// The mode to listen for analog outputs.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
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

#[derive(Default, Debug)]
pub struct ItwinHwState {
    pub hw_state: ShellHwState,
    pub listening_mode: ListeningMode,
}

impl TcKonnektSegmentData for ItwinHwState {
    fn build(&self, raw: &mut [u8]) {
        self.hw_state.build(raw);
        self.listening_mode.build_quadlet(&mut raw[8..12]);
    }

    fn parse(&mut self, raw: &[u8]) {
        self.hw_state.parse(raw);
        self.listening_mode.parse_quadlet(&raw[8..12]);
    }
}

impl TcKonnektSegmentSpec for TcKonnektSegment<ItwinHwState> {
    const OFFSET: usize = 0x1008;
    const SIZE: usize = ShellHwState::SIZE;
}

impl TcKonnektNotifiedSegmentSpec for TcKonnektSegment<ItwinHwState> {
    const NOTIFY_FLAG: u32 = SHELL_HW_STATE_NOTIFY_FLAG;
}

#[derive(Debug)]
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

impl TcKonnektSegmentData for ItwinMixerMeter {
    fn build(&self, raw: &mut [u8]) {
        ShellMixerMeterConvert::build(self, raw)
    }

    fn parse(&mut self, raw: &[u8]) {
        ShellMixerMeterConvert::parse(self, raw)
    }
}

impl TcKonnektSegmentSpec for TcKonnektSegment<ItwinMixerMeter> {
    const OFFSET: usize = 0x106c;
    const SIZE: usize = ShellMixerMeter::SIZE;
}

#[derive(Default, Debug)]
pub struct ItwinReverbMeter(pub ReverbMeter);

impl TcKonnektSegmentData for ItwinReverbMeter {
    fn build(&self, raw: &mut [u8]) {
        self.0.build(raw)
    }

    fn parse(&mut self, raw: &[u8]) {
        self.0.parse(raw)
    }
}

impl TcKonnektSegmentSpec for TcKonnektSegment<ItwinReverbMeter> {
    const OFFSET: usize = 0x10c8;
    const SIZE: usize = ReverbMeter::SIZE;
}

#[derive(Default, Debug)]
pub struct ItwinChStripMeters(pub [ChStripMeter; SHELL_CH_STRIP_COUNT]);

impl TcKonnektSegmentData for ItwinChStripMeters {
    fn build(&self, raw: &mut [u8]) {
        self.0.build(raw)
    }

    fn parse(&mut self, raw: &[u8]) {
        self.0.parse(raw)
    }
}

impl TcKonnektSegmentSpec for TcKonnektSegment<ItwinChStripMeters> {
    const OFFSET: usize = 0x10e0;
    const SIZE: usize = ChStripMeter::SIZE * SHELL_CH_STRIP_COUNT + 4;
}
