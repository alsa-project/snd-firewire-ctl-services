// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Protocol defined by TC Electronic for Konnekt 24d, Konnekt 8, Konnekt Live, and Impact Twin.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by TC Electronic for Konnekt 24d, Konnekt 8, Konnekt Live, and Impact Twin.

pub mod k8;
pub mod k24d;
pub mod klive;
pub mod itwin;

use super::fw_led::*;

use crate::*;

const SHELL_REVERB_NOTIFY_FLAG: u32 = 0x00080000;
const SHELL_CH_STRIP_NOTIFY_FLAG: u32 = 0x00100000;

const SHELL_CH_STRIP_COUNT: usize = 2;

/// The enumeration to represent state of jack sense for analog input.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ShellAnalogJackState {
    FrontSelected,
    FrontInserted,
    FrontInsertedAttenuated,
    RearSelected,
    RearInserted,
}

impl Default for ShellAnalogJackState {
    fn default() -> Self {
        Self::FrontSelected
    }
}

impl ShellAnalogJackState {
    const FRONT_SELECTED: u32 = 0x00;
    const FRONT_INSERTED: u32 = 0x05;
    const FRONT_INSERTED_ATTENUATED: u32 = 0x06;
    const REAR_SELECTED: u32 = 0x07;
    const REAR_INSERTED: u32 = 0x08;
}

impl From<u32> for ShellAnalogJackState {
    fn from(val: u32) -> Self {
        match val & 0xff {
            Self::FRONT_INSERTED => Self::FrontInserted,
            Self::FRONT_INSERTED_ATTENUATED => Self::FrontInsertedAttenuated,
            Self::REAR_SELECTED => Self::RearSelected,
            Self::REAR_INSERTED => Self::RearInserted,
            _ => Self::FrontSelected,
        }
    }
}

impl From<ShellAnalogJackState> for u32 {
    fn from(state: ShellAnalogJackState) -> Self {
        match state {
            ShellAnalogJackState::FrontSelected => ShellAnalogJackState::FRONT_SELECTED,
            ShellAnalogJackState::FrontInserted => ShellAnalogJackState::FRONT_INSERTED,
            ShellAnalogJackState::FrontInsertedAttenuated => ShellAnalogJackState::FRONT_INSERTED_ATTENUATED,
            ShellAnalogJackState::RearSelected => ShellAnalogJackState::REAR_SELECTED,
            ShellAnalogJackState::RearInserted => ShellAnalogJackState::REAR_INSERTED,
        }
    }
}

/// The number of analog inputs which has jack sense.
pub const SHELL_ANALOG_JACK_STATE_COUNT: usize = 2;

/// The structure to represent hardware state.
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct ShellHwState{
    pub analog_jack_states: [ShellAnalogJackState;SHELL_ANALOG_JACK_STATE_COUNT],
    pub firewire_led: FireWireLedState,
}

impl ShellHwState {
    pub const SIZE: usize = 28;

    pub fn build(&self, raw: &mut [u8]) {
        assert_eq!(raw.len(), Self::SIZE, "Programming error...");

        self.analog_jack_states.build_quadlet_block(&mut raw[..8]);
        self.firewire_led.build_quadlet(&mut raw[20..24]);
    }

    pub fn parse(&mut self, raw: &[u8]) {
        assert_eq!(raw.len(), Self::SIZE, "Programming error...");

        self.analog_jack_states.parse_quadlet_block(&raw[..8]);
        self.firewire_led.parse_quadlet(&raw[20..24]);
    }
}
