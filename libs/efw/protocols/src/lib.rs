// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol defined by Echo Audio Digital Corporation for Fireworks board module.
//!
//! The module includes protocols defined by Echo Audio Digital Corporation for Fireworks board
//! module.

pub mod flash;
pub mod hw_ctl;
pub mod hw_info;
pub mod monitor;
pub mod phys_input;
pub mod phys_output;
pub mod playback;
pub mod port_conf;
pub mod robot_guitar;
pub mod transaction;
pub mod transport;

use {
    glib::{Error, FileError},
    hitaki::{prelude::EfwProtocolExtManual, EfwProtocolError},
};

/// Signal source of sampling clock.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ClkSrc {
    Internal,
    WordClock,
    Spdif,
    Adat,
    Adat2,
    Continuous,
    Reserved(usize),
}

impl Default for ClkSrc {
    fn default() -> Self {
        Self::Reserved(usize::MAX)
    }
}

impl From<ClkSrc> for usize {
    fn from(src: ClkSrc) -> Self {
        match src {
            ClkSrc::Internal => 0,
            // blank.
            ClkSrc::WordClock => 2,
            ClkSrc::Spdif => 3,
            ClkSrc::Adat => 4,
            ClkSrc::Adat2 => 5,
            ClkSrc::Continuous => 6,
            ClkSrc::Reserved(val) => val,
        }
    }
}

impl From<usize> for ClkSrc {
    fn from(val: usize) -> Self {
        match val {
            0 => ClkSrc::Internal,
            // blank.
            2 => ClkSrc::WordClock,
            3 => ClkSrc::Spdif,
            4 => ClkSrc::Adat,
            5 => ClkSrc::Adat2,
            6 => ClkSrc::Continuous,
            _ => ClkSrc::Reserved(val),
        }
    }
}

/// Nominal level of audio signal.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum NominalSignalLevel {
    /// +4 dBu.
    Professional,
    Medium,
    /// -10 dBV.
    Consumer,
}

impl From<NominalSignalLevel> for u32 {
    fn from(level: NominalSignalLevel) -> Self {
        match level {
            NominalSignalLevel::Consumer => 2,
            NominalSignalLevel::Medium => 1,
            NominalSignalLevel::Professional => 0,
        }
    }
}

impl From<u32> for NominalSignalLevel {
    fn from(val: u32) -> Self {
        match val {
            2 => NominalSignalLevel::Consumer,
            1 => NominalSignalLevel::Medium,
            _ => NominalSignalLevel::Professional,
        }
    }
}
