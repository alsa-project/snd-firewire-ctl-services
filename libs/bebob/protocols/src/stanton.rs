// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol implementation for Stanton Magnetics Final Scratch 2 ScratchAmp.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by Stanton Magnetics for Final Scratch 2 ScratchAmp.

use super::*;

/// The protocol implementation for media and sampling clock of Scratchamp.
#[derive(Default)]
pub struct ScratchampClkProtocol;

impl MediaClockFrequencyOperation for ScratchampClkProtocol {
    const FREQ_LIST: &'static [u32] = &[44100, 48000, 88200, 96000];
}
