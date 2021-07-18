// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol implementation for Behringer Firepower series.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by Behringer for Firepower series.

use super::*;

/// The protocol implementation for media and sampling clock of FCA 610.
#[derive(Default)]
pub struct Fca610ClkProtocol;

impl MediaClockFrequencyOperation for Fca610ClkProtocol {
    const FREQ_LIST: &'static [u32] = &[44100, 48000, 88200, 96000];
}
