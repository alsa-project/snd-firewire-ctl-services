// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol implementation for Firebox.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by PreSonus for Firebox.

use ta1394::*;

use crate::*;

/// The protocol implementation of clock operation.
#[derive(Default)]
pub struct FireboxClkProtocol;

impl MediaClockFrequencyOperation for FireboxClkProtocol {
    const FREQ_LIST: &'static [u32] = &[44100, 48000, 88200, 96000];
}

impl SamplingClockSourceOperation for FireboxClkProtocol {
    const DST: SignalAddr = SignalAddr::Subunit(SignalSubunitAddr{
        subunit: MUSIC_SUBUNIT_0,
        plug_id: 0x05,
    });

    const SRC_LIST: &'static [SignalAddr] = &[
        // Internal.
        SignalAddr::Subunit(SignalSubunitAddr{subunit: MUSIC_SUBUNIT_0, plug_id: 0x06}),
        // S/PDIF in coaxial interface.
        SignalAddr::Unit(SignalUnitAddr::Ext(0x03)),
    ];
}
