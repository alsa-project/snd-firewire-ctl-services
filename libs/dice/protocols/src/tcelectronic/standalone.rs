// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Protocol defined by TC Electronic for Konnekt series.
//!
//! The module includes enumeration for data of standalone configuration defined by TC Electronic
//! for Konnekt series.

/// Available rate for sampling clock in standalone mode.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum TcKonnektStandaloneClkRate {
    R44100,
    R48000,
    R88200,
    R96000,
}

impl Default for TcKonnektStandaloneClkRate {
    fn default() -> Self {
        Self::R44100
    }
}

impl From<TcKonnektStandaloneClkRate> for u32 {
    fn from(clk: TcKonnektStandaloneClkRate) -> Self {
        match clk {
            TcKonnektStandaloneClkRate::R96000 => 4,
            TcKonnektStandaloneClkRate::R88200 => 3,
            TcKonnektStandaloneClkRate::R48000 => 2,
            TcKonnektStandaloneClkRate::R44100 => 1,
        }
    }
}

impl From<u32> for TcKonnektStandaloneClkRate {
    fn from(val: u32) -> Self {
        match val {
            4 => TcKonnektStandaloneClkRate::R96000,
            3 => TcKonnektStandaloneClkRate::R88200,
            2 => TcKonnektStandaloneClkRate::R48000,
            _ => TcKonnektStandaloneClkRate::R44100,
        }
    }
}
