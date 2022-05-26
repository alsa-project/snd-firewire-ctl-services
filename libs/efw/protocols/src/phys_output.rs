// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol about physical output.
//!
//! The module includes protocol about physical output defined by Echo Audio Digital Corporation for
//! Fireworks board module.

use glib::Error;

use super::{EfwProtocol, NominalSignalLevel};

const CATEGORY_PHYS_OUTPUT: u32 = 4;

const CMD_SET_VOL: u32 = 0;
const CMD_GET_VOL: u32 = 1;
const CMD_SET_MUTE: u32 = 2;
const CMD_GET_MUTE: u32 = 3;
const CMD_SET_NOMINAL: u32 = 8;
const CMD_GET_NOMINAL: u32 = 9;

/// Protocol about physical output for Fireworks board module.
pub trait PhysOutputProtocol: EfwProtocol {
    /// Set volume of output.  The value of vol is unsigned fixed-point number of 8.24 format; i.e. Q24.
    /// (0x00000000..0x02000000, -144.0..+6.0 dB)
    fn set_vol(&mut self, ch: usize, vol: i32, timeout_ms: u32) -> Result<(), Error> {
        let args = [ch as u32, vol as u32];
        let mut params = [0; 2];
        self.transaction_sync(
            CATEGORY_PHYS_OUTPUT,
            CMD_SET_VOL,
            Some(&args),
            Some(&mut params),
            timeout_ms,
        )
    }

    /// Get volume of output. The value of vol is unsigned fixed-point number of 8.24 format; i.e. Q24.
    /// (0x00000000..0x02000000, -144.0..+6.0 dB)
    fn get_vol(&mut self, ch: usize, timeout_ms: u32) -> Result<i32, Error> {
        let args = [ch as u32, 0];
        let mut params = [0; 2];
        self.transaction_sync(
            CATEGORY_PHYS_OUTPUT,
            CMD_GET_VOL,
            Some(&args),
            Some(&mut params),
            timeout_ms,
        )
        .map(|_| params[1] as i32)
    }

    fn set_mute(&mut self, ch: usize, mute: bool, timeout_ms: u32) -> Result<(), Error> {
        let args = [ch as u32, mute as u32];
        let mut params = [0; 2];
        self.transaction_sync(
            CATEGORY_PHYS_OUTPUT,
            CMD_SET_MUTE,
            Some(&args),
            Some(&mut params),
            timeout_ms,
        )
    }

    fn get_mute(&mut self, ch: usize, timeout_ms: u32) -> Result<bool, Error> {
        let args = [ch as u32, 0];
        let mut params = [0; 2];
        self.transaction_sync(
            CATEGORY_PHYS_OUTPUT,
            CMD_GET_MUTE,
            Some(&args),
            Some(&mut params),
            timeout_ms,
        )
        .map(|_| params[1] > 0)
    }

    fn set_nominal(
        &mut self,
        ch: usize,
        level: NominalSignalLevel,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let args = [ch as u32, u32::from(level)];
        let mut params = [0; 2];
        self.transaction_sync(
            CATEGORY_PHYS_OUTPUT,
            CMD_SET_NOMINAL,
            Some(&args),
            Some(&mut params),
            timeout_ms,
        )
    }

    fn get_nominal(&mut self, ch: usize, timeout_ms: u32) -> Result<NominalSignalLevel, Error> {
        let args = [ch as u32, 0];
        let mut params = [0; 2];
        self.transaction_sync(
            CATEGORY_PHYS_OUTPUT,
            CMD_GET_NOMINAL,
            Some(&args),
            Some(&mut params),
            timeout_ms,
        )
        .map(|_| NominalSignalLevel::from(params[1]))
    }
}

impl<O: EfwProtocol> PhysOutputProtocol for O {}
