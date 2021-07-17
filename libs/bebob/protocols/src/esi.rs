// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol implementation for ESI Quatafire series.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by Ego Systems (ESI) for Quatafire series.

use super::*;

/// The protocol implementation for media and sampling clock of Quatafire 610.
#[derive(Default)]
pub struct Quatafire610ClkProtocol;

impl MediaClockFrequencyOperation for Quatafire610ClkProtocol {
    const FREQ_LIST: &'static [u32] = &[44100, 48000, 88200, 96000, 192000];
}
