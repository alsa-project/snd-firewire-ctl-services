// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol about physical input.
//!
//! The module includes protocol about physical input defined by Echo Audio Digital Corporation for
//! Fireworks board module.

use super::*;

const CATEGORY_PHYS_INPUT: u32 = 5;

const CMD_SET_NOMINAL: u32 = 8;
const CMD_GET_NOMINAL: u32 = 9;

/// Protocol about physical input for Fireworks board module.
pub trait PhysInputProtocol: EfwProtocol {
    fn set_nominal(
        &mut self,
        ch: usize,
        level: NominalSignalLevel,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let args = [ch as u32, u32::from(level)];
        let mut params = [0; 2];
        self.transaction_sync(
            CATEGORY_PHYS_INPUT,
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
            CATEGORY_PHYS_INPUT,
            CMD_GET_NOMINAL,
            Some(&args),
            Some(&mut params),
            timeout_ms,
        )
        .map(|_| NominalSignalLevel::from(params[1]))
    }
}

impl<O: EfwProtocol> PhysInputProtocol for O {}
