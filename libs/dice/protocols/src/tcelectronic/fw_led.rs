// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Common structure for hardware state for TC Electronic Konnekt series.

/// The state of FireWire LED.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum FireWireLedState {
    Off,
    On,
    BlinkSlow,
    BlinkFast,
}

impl FireWireLedState {
    const OFF: u32 = 0x00;
    const ON: u32 = 0x01;
    const BLINK_FAST: u32 = 0x02;
    const BLINK_SLOW: u32 = 0x03;
}

impl Default for FireWireLedState {
    fn default() -> Self {
        Self::Off
    }
}

impl From<u32> for FireWireLedState {
    fn from(val: u32) -> Self {
        match val & 0x03 {
            Self::ON => Self::On,
            Self::BLINK_FAST => Self::BlinkFast,
            Self::BLINK_SLOW => Self::BlinkSlow,
            _ => Self::Off,
        }
    }
}

impl From<FireWireLedState> for u32 {
    fn from(state: FireWireLedState) -> Self {
        match state {
            FireWireLedState::On => FireWireLedState::ON,
            FireWireLedState::BlinkFast => FireWireLedState::BLINK_FAST,
            FireWireLedState::BlinkSlow => FireWireLedState::BLINK_SLOW,
            _ => FireWireLedState::OFF,
        }
    }
}
