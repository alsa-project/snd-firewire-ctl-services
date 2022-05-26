// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol about input monitor.
//!
//! The module includes protocol about input monitor defined by Echo Audio Digital Corporation for
//! Fireworks board module.

use super::*;

const CATEGORY_MONITOR: u32 = 8;

const CMD_SET_VOL: u32 = 0;
const CMD_GET_VOL: u32 = 1;
const CMD_SET_MUTE: u32 = 2;
const CMD_GET_MUTE: u32 = 3;
const CMD_SET_SOLO: u32 = 4;
const CMD_GET_SOLO: u32 = 5;
const CMD_SET_PAN: u32 = 6;
const CMD_GET_PAN: u32 = 7;

/// Protocol about input monitor for Fireworks board module.
pub trait MonitorProtocol: EfwProtocol {
    /// Set volume of monitor. The value of vol is unsigned fixed-point number of 8.24 format; i.e. Q24.
    /// (0x00000000..0x02000000, -144.0..+6.0 dB)
    fn set_monitor_vol(
        &mut self,
        dst: usize,
        src: usize,
        vol: i32,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let args = [src as u32, dst as u32, vol as u32];
        let mut params = [0; 3];
        self.transaction_sync(
            CATEGORY_MONITOR,
            CMD_SET_VOL,
            Some(&args),
            Some(&mut params),
            timeout_ms,
        )
    }

    /// Get volume of monitor. The value of vol is unsigned fixed-point number of 8.24 format; i.e. Q24.
    /// (0x00000000..0x02000000, -144.0..+6.0 dB)
    fn get_monitor_vol(&mut self, dst: usize, src: usize, timeout_ms: u32) -> Result<i32, Error> {
        let args = [src as u32, dst as u32, 0];
        let mut params = [0; 3];
        self.transaction_sync(
            CATEGORY_MONITOR,
            CMD_GET_VOL,
            Some(&args),
            Some(&mut params),
            timeout_ms,
        )
        .map(|_| params[2] as i32)
    }

    fn set_monitor_mute(
        &mut self,
        dst: usize,
        src: usize,
        mute: bool,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let args = [src as u32, dst as u32, mute as u32];
        let mut params = [0; 3];
        self.transaction_sync(
            CATEGORY_MONITOR,
            CMD_SET_MUTE,
            Some(&args),
            Some(&mut params),
            timeout_ms,
        )
    }

    fn get_monitor_mute(&mut self, dst: usize, src: usize, timeout_ms: u32) -> Result<bool, Error> {
        let args = [src as u32, dst as u32, 0];
        let mut params = [0; 3];
        self.transaction_sync(
            CATEGORY_MONITOR,
            CMD_GET_MUTE,
            Some(&args),
            Some(&mut params),
            timeout_ms,
        )
        .map(|_| params[2] > 0)
    }

    fn set_monitor_solo(
        &mut self,
        dst: usize,
        src: usize,
        solo: bool,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let args = [src as u32, dst as u32, solo as u32];
        let mut params = [0; 3];
        self.transaction_sync(
            CATEGORY_MONITOR,
            CMD_SET_SOLO,
            Some(&args),
            Some(&mut params),
            timeout_ms,
        )
    }

    fn get_monitor_solo(&mut self, dst: usize, src: usize, timeout_ms: u32) -> Result<bool, Error> {
        let args = [src as u32, dst as u32, 0];
        let mut params = [0; 3];
        self.transaction_sync(
            CATEGORY_MONITOR,
            CMD_GET_SOLO,
            Some(&args),
            Some(&mut params),
            timeout_ms,
        )
        .map(|_| params[2] > 0)
    }

    /// Set L/R balance of monitor. (0..255, left to right)
    fn set_monitor_pan(
        &mut self,
        dst: usize,
        src: usize,
        pan: u8,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let args = [src as u32, dst as u32, pan as u32];
        let mut params = [0; 3];
        self.transaction_sync(
            CATEGORY_MONITOR,
            CMD_SET_PAN,
            Some(&args),
            Some(&mut params),
            timeout_ms,
        )
    }

    /// Get L/R balance of monitor. (0..255, left to right)
    fn get_monitor_pan(&mut self, dst: usize, src: usize, timeout_ms: u32) -> Result<u8, Error> {
        let args = [src as u32, dst as u32, 0];
        let mut params = [0; 3];
        self.transaction_sync(
            CATEGORY_MONITOR,
            CMD_GET_PAN,
            Some(&args),
            Some(&mut params),
            timeout_ms,
        )
        .map(|_| params[2] as u8)
    }
}

impl<O: EfwProtocol> MonitorProtocol for O {}
