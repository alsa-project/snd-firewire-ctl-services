// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Protocol defined by TC Electronic for Konnekt 8.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by TC Electronic for Konnekt 8.

use super::*;
use crate::tcelectronic::{*, standalone::*};

/// The structure to represent segments in memory space of Konnekt 8.
#[derive(Default, Debug)]
pub struct K8Segments{
    /// Segment for configuration. 0x0028..0x0073 (19 quads).
    pub config: TcKonnektSegment<K8Config>,
    /// Segment for state of mixer. 0x0074..0x01cf (87 quads).
    pub mixer_state: TcKonnektSegment<K8MixerState>,
    /// Segment for mixer meter. 0x105c..0x10b7 (23 quads).
    pub mixer_meter: TcKonnektSegment<K8MixerMeter>,
    /// Segment tor state of hardware. 0x100c..0x1027 (7 quads).
    pub hw_state: TcKonnektSegment<K8HwState>,
}

/// The structure to represent configuration.
#[derive(Default, Debug)]
pub struct K8Config{
    pub coax_out_src: ShellCoaxOutPairSrc,
    pub standalone_src: ShellStandaloneClkSrc,
    pub standalone_rate: TcKonnektStandaloneClkRate,
}

impl AsRef<ShellCoaxOutPairSrc> for K8Config {
    fn as_ref(&self) -> &ShellCoaxOutPairSrc {
        &self.coax_out_src
    }
}

impl AsMut<ShellCoaxOutPairSrc> for K8Config {
    fn as_mut(&mut self) -> &mut ShellCoaxOutPairSrc {
        &mut self.coax_out_src
    }
}

impl AsRef<ShellStandaloneClkSrc> for K8Config {
    fn as_ref(&self) -> &ShellStandaloneClkSrc {
        &self.standalone_src
    }
}

impl AsMut<ShellStandaloneClkSrc> for K8Config {
    fn as_mut(&mut self) -> &mut ShellStandaloneClkSrc {
        &mut self.standalone_src
    }
}

impl<'a> ShellStandaloneClkSpec<'a> for K8Config {
    const STANDALONE_CLOCK_SOURCES: &'a [ShellStandaloneClkSrc] = &[
        ShellStandaloneClkSrc::Coaxial,
        ShellStandaloneClkSrc::Internal,
    ];
}

impl AsRef<TcKonnektStandaloneClkRate> for K8Config {
    fn as_ref(&self) -> &TcKonnektStandaloneClkRate {
        &self.standalone_rate
    }
}

impl AsMut<TcKonnektStandaloneClkRate> for K8Config {
    fn as_mut(&mut self) -> &mut TcKonnektStandaloneClkRate {
        &mut self.standalone_rate
    }
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
pub struct K8MixerState{
    /// The common structure for state of mixer.
    pub mixer: ShellMixerState,
    /// Whether to use mixer function.
    pub enabled: bool,
}

impl AsRef<ShellMixerState> for K8MixerState {
    fn as_ref(&self) -> &ShellMixerState {
        &self.mixer
    }
}

impl AsMut<ShellMixerState> for K8MixerState {
    fn as_mut(&mut self) -> &mut ShellMixerState {
        &mut self.mixer
    }
}

impl Default for K8MixerState {
    fn default() -> Self {
        K8MixerState{
            mixer: Self::create_mixer_state(),
            enabled: Default::default(),
        }
    }
}

impl ShellMixerConvert for K8MixerState {
    const MONITOR_SRC_MAP: [Option<ShellMixerMonitorSrcType>;SHELL_MIXER_MONITOR_SRC_COUNT] = [
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
}

impl TcKonnektSegmentData for K8MixerState {
    fn build(&self, raw: &mut [u8]) {
        ShellMixerConvert::build(self, raw);

        self.enabled.build_quadlet(&mut raw[340..344]);
    }

    fn parse(&mut self, raw: &[u8]) {
        ShellMixerConvert::parse(self, raw);

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
pub struct K8HwState{
    pub hw_state: ShellHwState,
    pub aux_input_enabled: bool,
}

impl AsRef<[ShellAnalogJackState]> for K8HwState {
    fn as_ref(&self) -> &[ShellAnalogJackState] {
        &self.hw_state.analog_jack_states
    }
}

impl AsMut<[ShellAnalogJackState]> for K8HwState {
    fn as_mut(&mut self) -> &mut [ShellAnalogJackState] {
        &mut self.hw_state.analog_jack_states
    }
}

impl AsRef<FireWireLedState> for K8HwState {
    fn as_ref(&self) -> &FireWireLedState {
        &self.hw_state.firewire_led
    }
}

impl AsMut<FireWireLedState> for K8HwState {
    fn as_mut(&mut self) -> &mut FireWireLedState {
        &mut self.hw_state.firewire_led
    }
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
pub struct K8MixerMeter(ShellMixerMeter);

impl AsRef<ShellMixerMeter> for K8MixerMeter {
    fn as_ref(&self) -> &ShellMixerMeter {
        &self.0
    }
}

impl AsMut<ShellMixerMeter> for K8MixerMeter {
    fn as_mut(&mut self) -> &mut ShellMixerMeter {
        &mut self.0
    }
}

impl Default for K8MixerMeter {
    fn default() -> Self {
        K8MixerMeter(Self::create_meter_state())
    }
}

impl ShellMixerMeterConvert for K8MixerMeter {
    const ANALOG_INPUT_COUNT: usize = K8_METER_ANALOG_INPUT_COUNT;
    const DIGITAL_INPUT_COUNT: usize = K8_METER_DIGITAL_INPUT_COUNT;
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
