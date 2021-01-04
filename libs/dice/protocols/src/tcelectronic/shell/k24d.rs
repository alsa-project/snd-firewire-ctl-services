// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Protocol defined by TC Electronic for Konnekt 24d.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by TC Electronic for Konnekt 24d.

use super::*;
use crate::tcelectronic::{*, ch_strip::*, reverb::*, standalone::*};

/// The structure to represent segments in memory space of Konnekt 24d.
#[derive(Default, Debug)]
pub struct K24dSegments{
    /// Segment for configuration. 0x0028..0x0073 (76 quads).
    pub config: TcKonnektSegment<K24dConfig>,
    /// Segment for state of mixer. 0x0074..0x01cf (87 quads).
    pub mixer_state: TcKonnektSegment<K24dMixerState>,
    /// Segment for state of reverb effect. 0x01d0..0x0213. (17 quads)
    pub reverb_state: TcKonnektSegment<K24dReverbState>,
    /// Segment for states of channel strip effect. 0x0214..0x0337 (73 quads).
    pub ch_strip_state: TcKonnektSegment<K24dChStripStates>,
    /// Segment for mixer meter. 0x105c..0x10b7 (23 quads).
    pub mixer_meter: TcKonnektSegment<K24dMixerMeter>,
    /// Segment for state of hardware. 0x100c..0x1027 (7 quads).
    pub hw_state: TcKonnektSegment<K24dHwState>,
    /// Segment for meter of reverb effect. 0x10b8..0x010cf (6 quads).
    pub reverb_meter: TcKonnektSegment<K24dReverbMeter>,
    /// Segment for meters of channel strip effect. 0x10d0..0x110b (15 quads).
    pub ch_strip_meter: TcKonnektSegment<K24dChStripMeters>,
}

#[derive(Default, Debug)]
pub struct K24dConfig{
    pub opt: ShellOptIfaceConfig,
    pub coax_out_src: ShellCoaxOutPairSrc,
    pub out_23_src: ShellPhysOutSrc,
    pub standalone_src: ShellStandaloneClkSrc,
    pub standalone_rate: TcKonnektStandaloneClkRate,
}

impl AsRef<ShellOptIfaceConfig> for K24dConfig {
    fn as_ref(&self) -> &ShellOptIfaceConfig {
        &self.opt
    }
}

impl AsMut<ShellOptIfaceConfig> for K24dConfig {
    fn as_mut(&mut self) -> &mut ShellOptIfaceConfig {
        &mut self.opt
    }
}

impl AsRef<ShellCoaxOutPairSrc> for K24dConfig {
    fn as_ref(&self) -> &ShellCoaxOutPairSrc {
        &self.coax_out_src
    }
}

impl AsMut<ShellCoaxOutPairSrc> for K24dConfig {
    fn as_mut(&mut self) -> &mut ShellCoaxOutPairSrc {
        &mut self.coax_out_src
    }
}

impl AsRef<ShellStandaloneClkSrc> for K24dConfig {
    fn as_ref(&self) -> &ShellStandaloneClkSrc {
        &self.standalone_src
    }
}

impl AsMut<ShellStandaloneClkSrc> for K24dConfig {
    fn as_mut(&mut self) -> &mut ShellStandaloneClkSrc {
        &mut self.standalone_src
    }
}

impl<'a> ShellStandaloneClkSpec<'a> for K24dConfig {
    const STANDALONE_CLOCK_SOURCES: &'a [ShellStandaloneClkSrc] = &[
        ShellStandaloneClkSrc::Optical,
        ShellStandaloneClkSrc::Coaxial,
        ShellStandaloneClkSrc::Internal,
    ];
}

impl AsRef<TcKonnektStandaloneClkRate> for K24dConfig {
    fn as_ref(&self) -> &TcKonnektStandaloneClkRate {
        &self.standalone_rate
    }
}

impl AsMut<TcKonnektStandaloneClkRate> for K24dConfig {
    fn as_mut(&mut self) -> &mut TcKonnektStandaloneClkRate {
        &mut self.standalone_rate
    }
}

impl TcKonnektSegmentData for K24dConfig {
    fn build(&self, raw: &mut [u8]) {
        self.opt.build(&mut raw[..12]);
        self.coax_out_src.0.build_quadlet(&mut raw[12..16]);
        self.out_23_src.build_quadlet(&mut raw[16..20]);
        self.standalone_src.build_quadlet(&mut raw[20..24]);
        self.standalone_rate.build_quadlet(&mut raw[24..28]);
    }

    fn parse(&mut self, raw: &[u8]) {
        self.opt.parse(&raw[..12]);
        self.coax_out_src.0.parse_quadlet(&raw[12..16]);
        self.out_23_src.parse_quadlet(&raw[16..20]);
        self.standalone_src.parse_quadlet(&raw[20..24]);
        self.standalone_rate.parse_quadlet(&raw[24..28]);
    }
}

impl TcKonnektSegmentSpec for TcKonnektSegment<K24dConfig> {
    const OFFSET: usize = 0x0028;
    const SIZE: usize = 76;
}

impl TcKonnektNotifiedSegmentSpec for TcKonnektSegment<K24dConfig> {
    const NOTIFY_FLAG: u32 = SHELL_CONFIG_NOTIFY_FLAG;
}

/// The structure to represent state of mixer.
#[derive(Debug)]
pub struct K24dMixerState{
    /// The common structure for state of mixer.
    pub mixer: ShellMixerState,
    /// The parameter of return from reverb effect.
    pub reverb_return: ShellReverbReturn,
    /// Whether to use channel strip effect as plugin. It results in output of channel strip effect
    /// on tx stream.
    pub use_ch_strip_as_plugin: bool,
    /// Whether to use reverb effect at middle sampling rate (88.2/96.0 kHz).
    pub use_reverb_at_mid_rate: bool,
    /// Whether to use mixer function.
    pub enabled: bool,
}

impl Default for K24dMixerState {
    fn default() -> Self {
        K24dMixerState{
            mixer: Self::create_mixer_state(),
            reverb_return: Default::default(),
            use_ch_strip_as_plugin: Default::default(),
            use_reverb_at_mid_rate: Default::default(),
            enabled: Default::default(),
        }
    }
}

impl AsRef<ShellMixerState> for K24dMixerState {
    fn as_ref(&self) -> &ShellMixerState {
        &self.mixer
    }
}

impl AsMut<ShellMixerState> for K24dMixerState {
    fn as_mut(&mut self) -> &mut ShellMixerState {
        &mut self.mixer
    }
}

impl AsRef<ShellReverbReturn> for K24dMixerState {
    fn as_ref(&self) -> &ShellReverbReturn {
        &self.reverb_return
    }
}

impl AsMut<ShellReverbReturn> for K24dMixerState {
    fn as_mut(&mut self) -> &mut ShellReverbReturn {
        &mut self.reverb_return
    }
}

impl ShellMixerConvert for K24dMixerState {
    const MONITOR_SRC_MAP: [Option<ShellMixerMonitorSrcType>;SHELL_MIXER_MONITOR_SRC_COUNT] = [
        Some(ShellMixerMonitorSrcType::Stream),
        None,
        None,
        None,
        Some(ShellMixerMonitorSrcType::Analog),
        Some(ShellMixerMonitorSrcType::Analog),
        Some(ShellMixerMonitorSrcType::AdatSpdif),
        Some(ShellMixerMonitorSrcType::Adat),
        Some(ShellMixerMonitorSrcType::Adat),
        Some(ShellMixerMonitorSrcType::AdatSpdif),
    ];
}

impl TcKonnektSegmentData for K24dMixerState {
    fn build(&self, raw: &mut [u8]) {
        ShellMixerConvert::build(self, raw);

        self.reverb_return.build(&mut raw[316..328]);
        self.use_ch_strip_as_plugin.build_quadlet(&mut raw[328..332]);
        self.use_reverb_at_mid_rate.build_quadlet(&mut raw[332..336]);
        self.enabled.build_quadlet(&mut raw[340..344]);
    }

    fn parse(&mut self, raw: &[u8]) {
        ShellMixerConvert::parse(self, raw);

        self.reverb_return.parse(&raw[316..328]);
        self.use_ch_strip_as_plugin.parse_quadlet(&raw[328..332]);
        self.use_reverb_at_mid_rate.parse_quadlet(&raw[332..336]);
        self.enabled.parse_quadlet(&raw[340..344]);
    }
}

impl TcKonnektSegmentSpec for TcKonnektSegment<K24dMixerState> {
    const OFFSET: usize = 0x0074;
    const SIZE: usize = ShellMixerState::SIZE + 32;
}

impl TcKonnektNotifiedSegmentSpec for TcKonnektSegment<K24dMixerState> {
    const NOTIFY_FLAG: u32 = SHELL_MIXER_NOTIFY_FLAG;
}

#[derive(Default, Debug)]
pub struct K24dReverbState(ReverbState);

impl AsRef<ReverbState> for K24dReverbState {
    fn as_ref(&self) -> &ReverbState {
        &self.0
    }
}

impl AsMut<ReverbState> for K24dReverbState {
    fn as_mut(&mut self) -> &mut ReverbState {
        &mut self.0
    }
}
impl TcKonnektSegmentData for K24dReverbState {
    fn build(&self, raw: &mut [u8]) {
        self.0.build(raw)
    }

    fn parse(&mut self, raw: &[u8]) {
        self.0.parse(raw)
    }
}

impl TcKonnektSegmentSpec for TcKonnektSegment<K24dReverbState> {
    const OFFSET: usize = 0x01d0;
    const SIZE: usize = ReverbState::SIZE;
}

impl TcKonnektNotifiedSegmentSpec for TcKonnektSegment<K24dReverbState> {
    const NOTIFY_FLAG: u32 = SHELL_REVERB_NOTIFY_FLAG;
}

#[derive(Default, Debug)]
pub struct K24dChStripStates([ChStripState;SHELL_CH_STRIP_COUNT]);

impl AsRef<[ChStripState]> for K24dChStripStates {
    fn as_ref(&self) -> &[ChStripState] {
        &self.0
    }
}

impl AsMut<[ChStripState]> for K24dChStripStates {
    fn as_mut(&mut self) -> &mut [ChStripState] {
        &mut self.0
    }
}

impl TcKonnektSegmentData for K24dChStripStates {
    fn build(&self, raw: &mut [u8]) {
        self.0.build(raw)
    }

    fn parse(&mut self, raw: &[u8]) {
        self.0.parse(raw)
    }
}

impl TcKonnektSegmentSpec for TcKonnektSegment<K24dChStripStates> {
    const OFFSET: usize = 0x0214;
    const SIZE: usize = ChStripState::SIZE * SHELL_CH_STRIP_COUNT + 4;
}

impl TcKonnektNotifiedSegmentSpec for TcKonnektSegment<K24dChStripStates> {
    const NOTIFY_FLAG: u32 = SHELL_CH_STRIP_NOTIFY_FLAG;
}

#[derive(Default, Debug)]
pub struct K24dHwState(ShellHwState);

impl AsRef<[ShellAnalogJackState]> for K24dHwState {
    fn as_ref(&self) -> &[ShellAnalogJackState] {
        &self.0.analog_jack_states
    }
}

impl AsMut<[ShellAnalogJackState]> for K24dHwState {
    fn as_mut(&mut self) -> &mut [ShellAnalogJackState] {
        &mut self.0.analog_jack_states
    }
}

impl AsRef<FireWireLedState> for K24dHwState {
    fn as_ref(&self) -> &FireWireLedState {
        &self.0.firewire_led
    }
}

impl AsMut<FireWireLedState> for K24dHwState {
    fn as_mut(&mut self) -> &mut FireWireLedState {
        &mut self.0.firewire_led
    }
}

impl TcKonnektSegmentData for K24dHwState {
    fn build(&self, raw: &mut [u8]) {
        self.0.build(raw);
    }

    fn parse(&mut self, raw: &[u8]) {
        self.0.parse(raw);
    }
}

impl TcKonnektSegmentSpec for TcKonnektSegment<K24dHwState> {
    const OFFSET: usize = 0x100c;
    const SIZE: usize = ShellHwState::SIZE;
}

impl TcKonnektNotifiedSegmentSpec for TcKonnektSegment<K24dHwState> {
    const NOTIFY_FLAG: u32 = SHELL_HW_STATE_NOTIFY_FLAG;
}

const K24D_METER_ANALOG_INPUT_COUNT: usize = 2;
const K24D_METER_DIGITAL_INPUT_COUNT: usize = 2;

#[derive(Debug)]
pub struct K24dMixerMeter(ShellMixerMeter);

impl AsRef<ShellMixerMeter> for K24dMixerMeter {
    fn as_ref(&self) -> &ShellMixerMeter {
        &self.0
    }
}

impl AsMut<ShellMixerMeter> for K24dMixerMeter {
    fn as_mut(&mut self) -> &mut ShellMixerMeter {
        &mut self.0
    }
}

impl Default for K24dMixerMeter {
    fn default() -> Self {
        K24dMixerMeter(Self::create_meter_state())
    }
}

impl ShellMixerMeterConvert for K24dMixerMeter {
    const ANALOG_INPUT_COUNT: usize = K24D_METER_ANALOG_INPUT_COUNT;
    const DIGITAL_INPUT_COUNT: usize = K24D_METER_DIGITAL_INPUT_COUNT;
}

impl TcKonnektSegmentData for K24dMixerMeter {
    fn build(&self, raw: &mut [u8]) {
        ShellMixerMeterConvert::build(self, raw)
    }

    fn parse(&mut self, raw: &[u8]) {
        ShellMixerMeterConvert::parse(self, raw)
    }
}

impl TcKonnektSegmentSpec for TcKonnektSegment<K24dMixerMeter> {
    const OFFSET: usize = 0x105c;
    const SIZE: usize = ShellMixerMeter::SIZE;
}

#[derive(Default, Debug)]
pub struct K24dReverbMeter(ReverbMeter);

impl AsRef<ReverbMeter> for K24dReverbMeter {
    fn as_ref(&self) -> &ReverbMeter {
        &self.0
    }
}

impl AsMut<ReverbMeter> for K24dReverbMeter {
    fn as_mut(&mut self) -> &mut ReverbMeter {
        &mut self.0
    }
}

impl TcKonnektSegmentData for K24dReverbMeter {
    fn build(&self, raw: &mut [u8]) {
        self.0.build(raw)
    }

    fn parse(&mut self, raw: &[u8]) {
        self.0.parse(raw)
    }
}

impl TcKonnektSegmentSpec for TcKonnektSegment<K24dReverbMeter> {
    const OFFSET: usize = 0x10b8;
    const SIZE: usize = ReverbMeter::SIZE;
}

#[derive(Default, Debug)]
pub struct K24dChStripMeters([ChStripMeter;SHELL_CH_STRIP_COUNT]);

impl AsRef<[ChStripMeter]> for K24dChStripMeters {
    fn as_ref(&self) -> &[ChStripMeter] {
        &self.0
    }
}

impl AsMut<[ChStripMeter]> for K24dChStripMeters {
    fn as_mut(&mut self) -> &mut [ChStripMeter] {
        &mut self.0
    }
}

impl TcKonnektSegmentData for K24dChStripMeters {
    fn build(&self, raw: &mut [u8]) {
        self.0.build(raw)
    }

    fn parse(&mut self, raw: &[u8]) {
        self.0.parse(raw)
    }
}

impl TcKonnektSegmentSpec for TcKonnektSegment<K24dChStripMeters> {
    const OFFSET: usize = 0x10d0;
    const SIZE: usize = ChStripMeter::SIZE * SHELL_CH_STRIP_COUNT + 4;
}
