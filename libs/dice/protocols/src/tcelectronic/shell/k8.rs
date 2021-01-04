// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Protocol defined by TC Electronic for Konnekt 8.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by TC Electronic for Konnekt 8.

use super::*;
use crate::tcelectronic::*;

/// The structure to represent segments in memory space of Konnekt 8.
#[derive(Default, Debug)]
pub struct K8Segments{
    /// Segment tor state of hardware. 0x100c..0x1027 (7 quads).
    pub hw_state: TcKonnektSegment<K8HwState>,
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
