// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Protocol defined by TC Electronic for Impact Twin.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by TC Electronic for Impact Twin.

use super::*;
use crate::tcelectronic::{*, ch_strip::*, reverb::*, standalone::*};

/// The structure to represent segments in memory space of Impact Twin.
#[derive(Default, Debug)]
pub struct ItwinSegments{
    /// Segment for knob. 0x0000..0x0027 (36 quads).
    pub knob: TcKonnektSegment<ItwinKnob>,
    /// Segment for configuration. 0x0028..0x00cf (168 quads).
    pub config: TcKonnektSegment<ItwinConfig>,
    /// Segment for state of mixer. 0x00d0..0x0243 (93 quads).
    pub mixer_state: TcKonnektSegment<ItwinMixerState>,
    /// Segment for state of reverb effect. 0x0244..0x0287. (17 quads)
    pub reverb_state: TcKonnektSegment<ItwinReverbState>,
    /// Segment for states of channel strip effect. 0x0288..0x03ab (73 quads).
    pub ch_strip_state: TcKonnektSegment<ItwinChStripStates>,
    // NOTE: Segment for tuner. 0x03ac..0x03c8 (8 quads).
    /// Segment for mixer meter. 0x106c..0x10c7 (23 quads).
    pub mixer_meter: TcKonnektSegment<ItwinMixerMeter>,
    /// Segment for state of hardware. 0x1008..0x1023 (7 quads).
    pub hw_state: TcKonnektSegment<ItwinHwState>,
    /// Segment for meter of reverb effect. 0x10c8..0x010df (6 quads).
    pub reverb_meter: TcKonnektSegment<ItwinReverbMeter>,
    /// Segment for meters of channel strip effect. 0x10e0..0x111b (15 quads).
    pub ch_strip_meter: TcKonnektSegment<ItwinChStripMeters>,
}

/// The structure to represent state of knob.
#[derive(Default, Debug)]
pub struct ItwinKnob{
    pub target: ShellKnobTarget,
    pub clock_recovery: bool,
}

impl AsRef<ShellKnobTarget> for ItwinKnob {
    fn as_ref(&self) -> &ShellKnobTarget {
        &self.target
    }
}

impl AsMut<ShellKnobTarget> for ItwinKnob {
    fn as_mut(&mut self) -> &mut ShellKnobTarget {
        &mut self.target
    }
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

/// The enumeration to represent source of stream for mixer.
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
pub struct ItwinConfig{
    pub mixer_stream_src_pair: ShellMixerStreamSrcPair,
    pub standalone_src: ShellStandaloneClkSrc,
    pub standalone_rate: TcKonnektStandaloneClkRate,
    pub output_pair_src: [ItwinOutputPairSrc;ITWIN_PHYS_OUT_PAIR_COUNT],
}

impl AsRef<ShellMixerStreamSrcPair> for ItwinConfig {
    fn as_ref(&self) -> &ShellMixerStreamSrcPair {
        &self.mixer_stream_src_pair
    }
}

impl AsMut<ShellMixerStreamSrcPair> for ItwinConfig {
    fn as_mut(&mut self) -> &mut ShellMixerStreamSrcPair {
        &mut self.mixer_stream_src_pair
    }
}

impl ShellMixerStreamSrcPairSpec for ItwinConfig {
    const MAXIMUM_STREAM_SRC_PAIR_COUNT: usize = 7;
}

impl AsRef<ShellStandaloneClkSrc> for ItwinConfig {
    fn as_ref(&self) -> &ShellStandaloneClkSrc {
        &self.standalone_src
    }
}

impl AsMut<ShellStandaloneClkSrc> for ItwinConfig {
    fn as_mut(&mut self) -> &mut ShellStandaloneClkSrc {
        &mut self.standalone_src
    }
}

impl ShellStandaloneClkSpec for ItwinConfig {
    const STANDALONE_CLOCK_SOURCES: &'static [ShellStandaloneClkSrc] = &[
        ShellStandaloneClkSrc::Optical,
        ShellStandaloneClkSrc::Coaxial,
        ShellStandaloneClkSrc::Internal,
    ];
}

impl AsRef<TcKonnektStandaloneClkRate> for ItwinConfig {
    fn as_ref(&self) -> &TcKonnektStandaloneClkRate {
        &self.standalone_rate
    }
}

impl AsMut<TcKonnektStandaloneClkRate> for ItwinConfig {
    fn as_mut(&mut self) -> &mut TcKonnektStandaloneClkRate {
        &mut self.standalone_rate
    }
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
pub struct ItwinMixerState{
    pub mixer: ShellMixerState,
    /// The balance between analog and stream inputs to mix. 0..1000.
    pub stream_mix_balance: u32,
    pub enabled: bool,
}

impl AsRef<ShellMixerState> for ItwinMixerState {
    fn as_ref(&self) -> &ShellMixerState {
        &self.mixer
    }
}

impl AsMut<ShellMixerState> for ItwinMixerState {
    fn as_mut(&mut self) -> &mut ShellMixerState {
        &mut self.mixer
    }
}

impl Default for ItwinMixerState {
    fn default() -> Self {
        ItwinMixerState{
            mixer: Self::create_mixer_state(),
            enabled: Default::default(),
            stream_mix_balance: Default::default(),
        }
    }
}

impl ShellMixerConvert for ItwinMixerState {
    const MONITOR_SRC_MAP: [Option<ShellMixerMonitorSrcType>;SHELL_MIXER_MONITOR_SRC_COUNT] = [
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
}

impl TcKonnektSegmentData for ItwinMixerState {
    fn build(&self, raw: &mut [u8]) {
        ShellMixerConvert::build(self, raw);

        self.stream_mix_balance.build_quadlet(&mut raw[348..352]);
        self.enabled.build_quadlet(&mut raw[352..356]);
    }

    fn parse(&mut self, raw: &[u8]) {
        ShellMixerConvert::parse(self, raw);

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
pub struct ItwinReverbState(ReverbState);

impl AsRef<ReverbState> for ItwinReverbState {
    fn as_ref(&self) -> &ReverbState {
        &self.0
    }
}

impl AsMut<ReverbState> for ItwinReverbState {
    fn as_mut(&mut self) -> &mut ReverbState {
        &mut self.0
    }
}

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
pub struct ItwinChStripStates([ChStripState;SHELL_CH_STRIP_COUNT]);

impl AsRef<[ChStripState]> for ItwinChStripStates {
    fn as_ref(&self) -> &[ChStripState] {
        &self.0
    }
}

impl AsMut<[ChStripState]> for ItwinChStripStates {
    fn as_mut(&mut self) -> &mut [ChStripState] {
        &mut self.0
    }
}

impl TcKonnektSegmentData for ItwinChStripStates {
    fn build(&self, raw: &mut [u8]) {
        self.0.build(raw)
    }

    fn parse(&mut self, raw: &[u8]) {
        self.0.parse(raw)
    }
}

impl TcKonnektSegmentSpec for TcKonnektSegment<ItwinChStripStates> {
    const OFFSET: usize = 0x0288;
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
pub struct ItwinHwState{
    pub hw_state: ShellHwState,
    pub listening_mode: ListeningMode,
}

impl AsRef<[ShellAnalogJackState]> for ItwinHwState {
    fn as_ref(&self) -> &[ShellAnalogJackState] {
        &self.hw_state.analog_jack_states
    }
}

impl AsMut<[ShellAnalogJackState]> for ItwinHwState {
    fn as_mut(&mut self) -> &mut [ShellAnalogJackState] {
        &mut self.hw_state.analog_jack_states
    }
}

impl AsRef<FireWireLedState> for ItwinHwState {
    fn as_ref(&self) -> &FireWireLedState {
        &self.hw_state.firewire_led
    }
}

impl AsMut<FireWireLedState> for ItwinHwState {
    fn as_mut(&mut self) -> &mut FireWireLedState {
        &mut self.hw_state.firewire_led
    }
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
pub struct ItwinMixerMeter(ShellMixerMeter);

impl AsRef<ShellMixerMeter> for ItwinMixerMeter {
    fn as_ref(&self) -> &ShellMixerMeter {
        &self.0
    }
}

impl AsMut<ShellMixerMeter> for ItwinMixerMeter {
    fn as_mut(&mut self) -> &mut ShellMixerMeter {
        &mut self.0
    }
}

impl Default for ItwinMixerMeter {
    fn default() -> Self {
        ItwinMixerMeter(Self::create_meter_state())
    }
}

impl ShellMixerMeterConvert for ItwinMixerMeter {
    const ANALOG_INPUT_COUNT: usize = 4;
    const DIGITAL_INPUT_COUNT: usize = 8;
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
pub struct ItwinReverbMeter(ReverbMeter);

impl AsRef<ReverbMeter> for ItwinReverbMeter {
    fn as_ref(&self) -> &ReverbMeter {
        &self.0
    }
}

impl AsMut<ReverbMeter> for ItwinReverbMeter {
    fn as_mut(&mut self) -> &mut ReverbMeter {
        &mut self.0
    }
}

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
pub struct ItwinChStripMeters([ChStripMeter;SHELL_CH_STRIP_COUNT]);

impl AsRef<[ChStripMeter]> for ItwinChStripMeters {
    fn as_ref(&self) -> &[ChStripMeter] {
        &self.0
    }
}

impl AsMut<[ChStripMeter]> for ItwinChStripMeters {
    fn as_mut(&mut self) -> &mut [ChStripMeter] {
        &mut self.0
    }
}

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
