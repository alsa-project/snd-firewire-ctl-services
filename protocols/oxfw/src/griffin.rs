// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol defined by Griffin for FireWave.
//!
//! The module includes protocol implementation defined by Griffin Technologies for FireWave.

use super::*;

/// The protocol implementation for FireWave.
#[derive(Default, Debug)]
pub struct FirewaveProtocol;

impl OxfwAudioFbSpecification for FirewaveProtocol {
    const VOLUME_FB_ID: u8 = 0x02;
    const MUTE_FB_ID: u8 = 0x01;
    const CHANNEL_MAP: &'static [usize] = &[0, 1, 4, 5, 2, 3];
}
