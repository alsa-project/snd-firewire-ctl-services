// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol implementation for Stanton Magnetics Final Scratch 2 ScratchAmp.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by Stanton Magnetics for Final Scratch 2 ScratchAmp.

use super::*;

use ta1394::ccm::{SignalAddr, SignalSubunitAddr};
use ta1394::MUSIC_SUBUNIT_0;

/// The protocol implementation for media and sampling clock of Scratchamp.
#[derive(Default)]
pub struct ScratchampClkProtocol;

impl MediaClockFrequencyOperation for ScratchampClkProtocol {
    const FREQ_LIST: &'static [u32] = &[44100, 48000, 88200, 96000];
}

impl SamplingClockSourceOperation for ScratchampClkProtocol {
    const DST: SignalAddr = SignalAddr::Subunit(SignalSubunitAddr {
        subunit: MUSIC_SUBUNIT_0,
        plug_id: 0x05,
    });

    const SRC_LIST: &'static [SignalAddr] = &[
        // Internal
        SignalAddr::Subunit(SignalSubunitAddr {
            subunit: MUSIC_SUBUNIT_0,
            plug_id: 0x05,
        }),
    ];
}
