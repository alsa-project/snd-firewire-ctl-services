// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol defined by Echo Audio Digital Corporation for Fireworks board module.
//!
//! The crate includes protocols defined by Echo Audio Digital Corporation for Fireworks board
//! module.

pub mod flash;
pub mod hw_info;
pub mod transactions;

use hinawa::SndEfwExtManual;

/// The enumeration to express source of sampling clock.
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

/// The trait to represent protocol for Echo Audio Fireworks board module.
pub trait EfwProtocol {
    fn transaction_sync(
        &mut self,
        category: u32,
        command: u32,
        args: Option<&[u32]>,
        params: Option<&mut [u32]>,
        timeout_ms: u32,
    ) -> Result<(), glib::Error>;
}

impl<O: SndEfwExtManual> EfwProtocol for O {
    fn transaction_sync(
        &mut self,
        category: u32,
        command: u32,
        args: Option<&[u32]>,
        params: Option<&mut [u32]>,
        timeout_ms: u32,
    ) -> Result<(), glib::Error> {
        O::transaction_sync(self, category, command, args, params, timeout_ms).map(|_| ())
    }
}
