// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol implementation for Apogee Electronics Ensemble FireWire.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by Apogee Electronics Ensemble FireWire.

use crate::*;

/// The protocol implementation for media and sampling clock of Ensemble FireWire.
#[derive(Default)]
pub struct EnsembleClkProtocol;

impl MediaClockFrequencyOperation for EnsembleClkProtocol {
    const FREQ_LIST: &'static [u32] = &[44100, 48000, 88200, 96000, 176400, 192000];
}
