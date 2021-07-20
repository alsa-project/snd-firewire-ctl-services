// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol implementation for Inspire 1394.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by PreSonus for Inspire 1394.

use crate::*;

/// The protocol implementation of clock operation.
#[derive(Default)]
pub struct Inspire1394ClkProtocol;

impl MediaClockFrequencyOperation for Inspire1394ClkProtocol {
    const FREQ_LIST: &'static [u32] = &[44100, 48000, 88200, 96000];
}

impl SamplingClockSourceOperation for Inspire1394ClkProtocol {
    const DST: SignalAddr = SignalAddr::Subunit(SignalSubunitAddr{
        subunit: MUSIC_SUBUNIT_0,
        plug_id: 0x03,
    });

    const SRC_LIST: &'static [SignalAddr] = &[
        // Internal.
        SignalAddr::Subunit(SignalSubunitAddr{subunit: MUSIC_SUBUNIT_0, plug_id: 0x02}),
    ];
}
