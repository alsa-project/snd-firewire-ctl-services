// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol implementation for Firepod/FP10.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by PreSonus for Firepod/FP10.

use crate::*;

/// The protocol implementation for media and sampling clock of Firepod/FP10.
#[derive(Default)]
pub struct Fp10ClkProtocol;

impl MediaClockFrequencyOperation for Fp10ClkProtocol {
    const FREQ_LIST: &'static [u32] = &[44100, 48000, 88200, 96000];
}

impl SamplingClockSourceOperation for Fp10ClkProtocol {
    const DST: SignalAddr = SignalAddr::Subunit(SignalSubunitAddr{
        subunit: MUSIC_SUBUNIT_0,
        plug_id: 0x07,
    });

    const SRC_LIST: &'static [SignalAddr] = &[
        // Internal.
        SignalAddr::Subunit(SignalSubunitAddr{subunit: MUSIC_SUBUNIT_0, plug_id: 0x07}),
        // S/PDIF
        SignalAddr::Unit(SignalUnitAddr::Ext(0x01)),
    ];
}
