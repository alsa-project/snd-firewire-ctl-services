// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol implementation for M-Audio ProFire Lightbridge.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by M-Audio ProFire Lightbridge

use crate::*;

/// The protocol implementation for media and sampling clock of ProFire Lightbridge.
#[derive(Default)]
pub struct PflClkProtocol;

impl MediaClockFrequencyOperation for PflClkProtocol {
    const FREQ_LIST: &'static [u32] = &[44100, 48000, 88200, 96000];
}
