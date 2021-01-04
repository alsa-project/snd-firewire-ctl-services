// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Protocol defined by TC Electronic for Konnekt Live.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by TC Electronic for Konnekt Live.

use super::*;
use crate::tcelectronic::{*, ch_strip::*, reverb::*};

/// The structure to represent segments in memory space of Konnekt Live.
#[derive(Default, Debug)]
pub struct KliveSegments{
    /// Segment for state of mixer. 0x00ac..0x0217 (91 quads).
    pub mixer_state: TcKonnektSegment<KliveMixerState>,
    /// Segment for state of reverb effect. 0x0218..0x025b. (17 quads)
    pub reverb_state: TcKonnektSegment<KliveReverbState>,
    /// Segment for states of channel strip effect. 0x025c..0x037f (73 quads).
    pub ch_strip_state: TcKonnektSegment<KliveChStripStates>,
    /// Segment for mixer meter. 0x1068..0x10c3 (23 quads).
    pub mixer_meter: TcKonnektSegment<KliveMixerMeter>,
    /// Segment for state of hardware. 0x1008..0x1023 (7 quads).
    pub hw_state: TcKonnektSegment<KliveHwState>,
    /// Segment for meter of reverb effect. 0x10c4..0x010db (6 quads).
    pub reverb_meter: TcKonnektSegment<KliveReverbMeter>,
    /// Segment for meters of channel strip effect. 0x10dc..0x1117 (15 quads).
    pub ch_strip_meter: TcKonnektSegment<KliveChStripMeters>,
}

/// The source of channel strip effect.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ChStripSrc {
    Stream01,
    Analog01,
    Analog23,
    Digital01,
    Digital23,
    Digital45,
    Digital67,
    MixerOutput,
    None,
}

impl Default for ChStripSrc {
    fn default() -> Self {
        ChStripSrc::None
    }
}

impl From<u32> for ChStripSrc {
    fn from(val: u32) -> Self {
        match val {
            0 => Self::Stream01,
            4 => Self::Analog01,
            5 => Self::Analog23,
            6 => Self::Digital01,
            7 => Self::Digital23,
            8 => Self::Digital45,
            9 => Self::Digital67,
            10 => Self::MixerOutput,
            _ => Self::None,
        }
    }
}

impl From<ChStripSrc> for u32 {
    fn from(src: ChStripSrc) -> Self {
        match src {
            ChStripSrc::Stream01 => 0,
            ChStripSrc::Analog01 => 4,
            ChStripSrc::Analog23 => 5,
            ChStripSrc::Digital01 => 6,
            ChStripSrc::Digital23 => 7,
            ChStripSrc::Digital45 => 8,
            ChStripSrc::Digital67 => 9,
            ChStripSrc::MixerOutput => 10,
            ChStripSrc::None => 11,
        }
    }
}

/// The type of channel strip effect.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ChStripMode {
    FabrikC,
    RIAA1964,
    RIAA1987,
}

impl Default for ChStripMode {
    fn default() -> Self {
        ChStripMode::FabrikC
    }
}

impl From<u32> for ChStripMode {
    fn from(val: u32) -> Self {
        match val {
            2 => Self::RIAA1987,
            1 => Self::RIAA1964,
            _ => Self::FabrikC,
        }
    }
}

impl From<ChStripMode> for u32 {
    fn from(effect: ChStripMode) -> Self {
        match effect {
            ChStripMode::FabrikC => 0,
            ChStripMode::RIAA1964 => 1,
            ChStripMode::RIAA1987 => 2,
        }
    }
}

/// The structureto represent state of mixer.
#[derive(Debug)]
pub struct KliveMixerState{
    /// The common structure for state of mixer.
    pub mixer: ShellMixerState,
    /// The parameter of return from reverb effect.
    pub reverb_return: ShellReverbReturn,
    /// Whether to use channel strip effect as plugin. It results in output of channel strip effect
    /// on tx stream.
    pub use_ch_strip_as_plugin: bool,
    /// The source of channel strip effect.
    pub ch_strip_src: ChStripSrc,
    /// The type of channel effect. Fabrik-C, RIAA-1964, and RIAA-1987.
    pub ch_strip_mode: ChStripMode,
    /// Whether to use channel strip effect at middle sampling rate (88.2/96.0 kHz).
    pub use_reverb_at_mid_rate: bool,
    /// Whether to use mixer function.
    pub enabled: bool,
}

impl Default for KliveMixerState {
    fn default() -> Self {
        KliveMixerState{
            mixer: Self::create_mixer_state(),
            reverb_return: Default::default(),
            use_ch_strip_as_plugin: Default::default(),
            ch_strip_src: Default::default(),
            ch_strip_mode: Default::default(),
            use_reverb_at_mid_rate: Default::default(),
            enabled: Default::default(),
        }
    }
}

impl AsRef<ShellMixerState> for KliveMixerState {
    fn as_ref(&self) -> &ShellMixerState {
        &self.mixer
    }
}

impl AsMut<ShellMixerState> for KliveMixerState {
    fn as_mut(&mut self) -> &mut ShellMixerState {
        &mut self.mixer
    }
}

impl AsRef<ShellReverbReturn> for KliveMixerState {
    fn as_ref(&self) -> &ShellReverbReturn {
        &self.reverb_return
    }
}

impl AsMut<ShellReverbReturn> for KliveMixerState {
    fn as_mut(&mut self) -> &mut ShellReverbReturn {
        &mut self.reverb_return
    }
}

impl ShellMixerConvert for KliveMixerState {
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

impl TcKonnektSegmentData for KliveMixerState {
    fn build(&self, raw: &mut [u8]) {
        ShellMixerConvert::build(self, raw);

        self.reverb_return.build(&mut raw[316..328]);
        self.use_ch_strip_as_plugin.build_quadlet(&mut raw[328..332]);
        self.ch_strip_src.build_quadlet(&mut raw[332..336]);
        self.ch_strip_mode.build_quadlet(&mut raw[336..340]);
        self.use_reverb_at_mid_rate.build_quadlet(&mut raw[340..344]);
        self.enabled.build_quadlet(&mut raw[344..348]);
    }

    fn parse(&mut self, raw: &[u8]) {
        ShellMixerConvert::parse(self, raw);

        self.reverb_return.parse(&raw[316..328]);
        self.use_ch_strip_as_plugin.parse_quadlet(&raw[328..332]);
        self.ch_strip_src.parse_quadlet(&raw[332..336]);
        self.ch_strip_mode.parse_quadlet(&raw[336..340]);
        self.use_reverb_at_mid_rate.parse_quadlet(&raw[340..344]);
        self.enabled.parse_quadlet(&raw[344..348]);
    }
}

impl TcKonnektSegmentSpec for TcKonnektSegment<KliveMixerState> {
    const OFFSET: usize = 0x00ac;
    const SIZE: usize = ShellMixerState::SIZE + 48;
}

impl TcKonnektNotifiedSegmentSpec for TcKonnektSegment<KliveMixerState> {
    const NOTIFY_FLAG: u32 = SHELL_MIXER_NOTIFY_FLAG;
}

#[derive(Default, Debug)]
pub struct KliveReverbState(ReverbState);

impl AsRef<ReverbState> for KliveReverbState {
    fn as_ref(&self) -> &ReverbState {
        &self.0
    }
}

impl AsMut<ReverbState> for KliveReverbState {
    fn as_mut(&mut self) -> &mut ReverbState {
        &mut self.0
    }
}

impl TcKonnektSegmentData for KliveReverbState {
    fn build(&self, raw: &mut [u8]) {
        self.0.build(raw)
    }

    fn parse(&mut self, raw: &[u8]) {
        self.0.parse(raw)
    }
}

impl TcKonnektSegmentSpec for TcKonnektSegment<KliveReverbState> {
    const OFFSET: usize = 0x0218;
    const SIZE: usize = ReverbState::SIZE;
}

impl TcKonnektNotifiedSegmentSpec for TcKonnektSegment<KliveReverbState> {
    const NOTIFY_FLAG: u32 = SHELL_REVERB_NOTIFY_FLAG;
}

#[derive(Default, Debug)]
pub struct KliveChStripStates([ChStripState;SHELL_CH_STRIP_COUNT]);

impl AsRef<[ChStripState]> for KliveChStripStates {
    fn as_ref(&self) -> &[ChStripState] {
        &self.0
    }
}

impl AsMut<[ChStripState]> for KliveChStripStates {
    fn as_mut(&mut self) -> &mut [ChStripState] {
        &mut self.0
    }
}

impl TcKonnektSegmentData for KliveChStripStates {
    fn build(&self, raw: &mut [u8]) {
        self.0.build(raw)
    }

    fn parse(&mut self, raw: &[u8]) {
        self.0.parse(raw)
    }
}

impl TcKonnektSegmentSpec for TcKonnektSegment<KliveChStripStates> {
    const OFFSET: usize = 0x025c;
    const SIZE: usize = ChStripState::SIZE * SHELL_CH_STRIP_COUNT + 4;
}

impl TcKonnektNotifiedSegmentSpec for TcKonnektSegment<KliveChStripStates> {
    const NOTIFY_FLAG: u32 = SHELL_CH_STRIP_NOTIFY_FLAG;
}

#[derive(Default, Debug)]
pub struct KliveHwState(ShellHwState);

impl AsRef<[ShellAnalogJackState]> for KliveHwState {
    fn as_ref(&self) -> &[ShellAnalogJackState] {
        &self.0.analog_jack_states
    }
}

impl AsMut<[ShellAnalogJackState]> for KliveHwState {
    fn as_mut(&mut self) -> &mut [ShellAnalogJackState] {
        &mut self.0.analog_jack_states
    }
}

impl AsRef<FireWireLedState> for KliveHwState {
    fn as_ref(&self) -> &FireWireLedState {
        &self.0.firewire_led
    }
}

impl AsMut<FireWireLedState> for KliveHwState {
    fn as_mut(&mut self) -> &mut FireWireLedState {
        &mut self.0.firewire_led
    }
}

impl TcKonnektSegmentData for KliveHwState {
    fn build(&self, raw: &mut [u8]) {
        self.0.build(raw)
    }

    fn parse(&mut self, raw: &[u8]) {
        self.0.parse(raw)
    }
}

impl TcKonnektSegmentSpec for TcKonnektSegment<KliveHwState> {
    const OFFSET: usize = 0x1008;
    const SIZE: usize = ShellHwState::SIZE;
}

impl TcKonnektNotifiedSegmentSpec for TcKonnektSegment<KliveHwState> {
    const NOTIFY_FLAG: u32 = SHELL_HW_STATE_NOTIFY_FLAG;
}

const KLIVE_METER_ANALOG_INPUT_COUNT: usize = 4;
const KLIVE_METER_DIGITAL_INPUT_COUNT: usize = 8;

#[derive(Debug)]
pub struct KliveMixerMeter(ShellMixerMeter);

impl AsRef<ShellMixerMeter> for KliveMixerMeter {
    fn as_ref(&self) -> &ShellMixerMeter {
        &self.0
    }
}

impl AsMut<ShellMixerMeter> for KliveMixerMeter {
    fn as_mut(&mut self) -> &mut ShellMixerMeter {
        &mut self.0
    }
}

impl Default for KliveMixerMeter {
    fn default() -> Self {
        KliveMixerMeter(Self::create_meter_state())
    }
}

impl ShellMixerMeterConvert for KliveMixerMeter {
    const ANALOG_INPUT_COUNT: usize = KLIVE_METER_ANALOG_INPUT_COUNT;
    const DIGITAL_INPUT_COUNT: usize = KLIVE_METER_DIGITAL_INPUT_COUNT;
}

impl TcKonnektSegmentData for KliveMixerMeter {
    fn build(&self, raw: &mut [u8]) {
        ShellMixerMeterConvert::build(self, raw)
    }

    fn parse(&mut self, raw: &[u8]) {
        ShellMixerMeterConvert::parse(self, raw)
    }
}

impl TcKonnektSegmentSpec for TcKonnektSegment<KliveMixerMeter> {
    const OFFSET: usize = 0x1068;
    const SIZE: usize = ShellMixerMeter::SIZE;
}

#[derive(Default, Debug)]
pub struct KliveReverbMeter(ReverbMeter);

impl AsRef<ReverbMeter> for KliveReverbMeter {
    fn as_ref(&self) -> &ReverbMeter {
        &self.0
    }
}

impl AsMut<ReverbMeter> for KliveReverbMeter {
    fn as_mut(&mut self) -> &mut ReverbMeter {
        &mut self.0
    }
}

impl TcKonnektSegmentData for KliveReverbMeter {
    fn build(&self, raw: &mut [u8]) {
        self.0.build(raw)
    }

    fn parse(&mut self, raw: &[u8]) {
        self.0.parse(raw)
    }
}

impl TcKonnektSegmentSpec for TcKonnektSegment<KliveReverbMeter> {
    const OFFSET: usize = 0x10c4;
    const SIZE: usize = ReverbMeter::SIZE;
}

#[derive(Default, Debug)]
pub struct KliveChStripMeters([ChStripMeter;SHELL_CH_STRIP_COUNT]);

impl AsRef<[ChStripMeter]> for KliveChStripMeters {
    fn as_ref(&self) -> &[ChStripMeter] {
        &self.0
    }
}

impl AsMut<[ChStripMeter]> for KliveChStripMeters {
    fn as_mut(&mut self) -> &mut [ChStripMeter] {
        &mut self.0
    }
}

impl TcKonnektSegmentData for KliveChStripMeters {
    fn build(&self, raw: &mut [u8]) {
        self.0.build(raw)
    }

    fn parse(&mut self, raw: &[u8]) {
        self.0.parse(raw)
    }
}

impl TcKonnektSegmentSpec for TcKonnektSegment<KliveChStripMeters> {
    const OFFSET: usize = 0x10dc;
    const SIZE: usize = ChStripMeter::SIZE * SHELL_CH_STRIP_COUNT + 4;
}
