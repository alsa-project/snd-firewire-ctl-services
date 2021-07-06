// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol defined by Echo Audio Digital Corporation for Fireworks board module.
//!
//! The crate includes protocols defined by Echo Audio Digital Corporation for Fireworks board
//! module.

pub mod transactions;

use hinawa::SndEfwExtManual;

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
