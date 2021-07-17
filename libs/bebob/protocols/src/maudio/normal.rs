// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol implementation for M-Audio FireWire series.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by M-Audio normal FireWire series.

use crate::*;

/// The protocol implementation for media and sampling clock of FireWire 410.
#[derive(Default)]
pub struct Fw410ClkProtocol;

impl MediaClockFrequencyOperation for Fw410ClkProtocol {
    const FREQ_LIST: &'static [u32] = &[44100, 48000, 88200, 96000, 176400, 192000];
}

/// The protocol implementation for media and sampling clock of FireWire Solo.
#[derive(Default)]
pub struct SoloClkProtocol;

impl MediaClockFrequencyOperation for SoloClkProtocol {
    const FREQ_LIST: &'static [u32] = &[44100, 48000, 88200, 96000];
}

/// The protocol implementation for media and sampling clock of FireWire Audiophile.
#[derive(Default)]
pub struct AudiophileClkProtocol;

impl MediaClockFrequencyOperation for AudiophileClkProtocol {
    const FREQ_LIST: &'static [u32] = &[44100, 48000, 88200, 96000];
}

/// The protocol implementation for media and sampling clock of Ozonic.
#[derive(Default)]
pub struct OzonicClkProtocol;

impl MediaClockFrequencyOperation for OzonicClkProtocol {
    const FREQ_LIST: &'static [u32] = &[44100, 48000, 88200, 96000];
}
