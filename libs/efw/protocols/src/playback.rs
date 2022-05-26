// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol about stream playback.
//!
//! The module includes protocol about stream playback defined by Echo Audio Digital Corporation for
//! Fireworks board module.

use super::*;

const CATEGORY_PLAYBACK: u32 = 6;

const CMD_SET_VOL: u32 = 0;
const CMD_GET_VOL: u32 = 1;
const CMD_SET_MUTE: u32 = 2;
const CMD_GET_MUTE: u32 = 3;
const CMD_SET_SOLO: u32 = 4;
const CMD_GET_SOLO: u32 = 5;

/// Protocol about stream playback for Fireworks board module.
pub trait PlaybackProtocol: EfwProtocol {
    /// Set volume of stream playback. The value of vol is unsigned fixed-point number of 8.24
    /// format; i.e. Q24. (0x00000000..0x02000000, -144.0..+6.0 dB)
    fn set_playback_vol(&mut self, ch: usize, vol: i32, timeout_ms: u32) -> Result<(), Error> {
        let args = [ch as u32, vol as u32];
        let mut params = vec![0; 2];
        self.transaction(
            CATEGORY_PLAYBACK,
            CMD_SET_VOL,
            &args,
            &mut params,
            timeout_ms,
        )
    }

    /// Get volume of stream playback. The value of vol is unsigned fixed-point number of 8.24
    /// format; e.g. Q24. (0x00000000..0x02000000, -144.0..+6.0 dB)
    fn get_playback_vol(&mut self, ch: usize, timeout_ms: u32) -> Result<i32, Error> {
        let args = [ch as u32, 0];
        let mut params = vec![0; 2];
        self.transaction(
            CATEGORY_PLAYBACK,
            CMD_GET_VOL,
            &args,
            &mut params,
            timeout_ms,
        )
        .map(|_| params[1] as i32)
    }

    fn set_playback_mute(&mut self, ch: usize, mute: bool, timeout_ms: u32) -> Result<(), Error> {
        let args = [ch as u32, mute as u32];
        let mut params = vec![0; 2];
        self.transaction(
            CATEGORY_PLAYBACK,
            CMD_SET_MUTE,
            &args,
            &mut params,
            timeout_ms,
        )
    }

    fn get_playback_mute(&mut self, ch: usize, timeout_ms: u32) -> Result<bool, Error> {
        let args = [ch as u32, 0];
        let mut params = vec![0; 2];
        self.transaction(
            CATEGORY_PLAYBACK,
            CMD_GET_MUTE,
            &args,
            &mut params,
            timeout_ms,
        )
        .map(|_| params[1] > 0)
    }

    fn set_playback_solo(&mut self, ch: usize, solo: bool, timeout_ms: u32) -> Result<(), Error> {
        let args = [ch as u32, solo as u32];
        let mut params = vec![0; 2];
        self.transaction(
            CATEGORY_PLAYBACK,
            CMD_SET_SOLO,
            &args,
            &mut params,
            timeout_ms,
        )
    }

    fn get_playback_solo(&mut self, ch: usize, timeout_ms: u32) -> Result<bool, Error> {
        let args = [ch as u32, 0];
        let mut params = vec![0; 2];
        self.transaction(
            CATEGORY_PLAYBACK,
            CMD_GET_SOLO,
            &args,
            &mut params,
            timeout_ms,
        )
        .map(|_| params[1] > 0)
    }
}

impl<O: EfwProtocol> PlaybackProtocol for O {}
