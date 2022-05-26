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
pub trait PhysInputProtocol: EfwProtocolExtManual {
    fn set_nominal(
        &mut self,
        ch: usize,
        level: NominalSignalLevel,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let args = [ch as u32, u32::from(level)];
        self.transaction(
            CATEGORY_PHYS_INPUT,
            CMD_SET_NOMINAL,
            &args,
            &mut vec![0; 2],
            timeout_ms,
        )
    }

    fn get_nominal(&mut self, ch: usize, timeout_ms: u32) -> Result<NominalSignalLevel, Error> {
        let args = [ch as u32, 0];
        let mut params = vec![0; 2];
        self.transaction(
            CATEGORY_PHYS_INPUT,
            CMD_GET_NOMINAL,
            &args,
            &mut params,
            timeout_ms,
        )
        .map(|_| NominalSignalLevel::from(params[1]))
    }
}

impl<O: EfwProtocolExtManual> PhysInputProtocol for O {}
