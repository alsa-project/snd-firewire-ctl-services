// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol implementation for Terratec Aureon 7.1 FW.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by Terratec for Aureon 7.1 FW.

use crate::*;

/// The protocol implementation for media and sampling clock.
#[derive(Default)]
pub struct AureonClkProtocol;

impl MediaClockFrequencyOperation for AureonClkProtocol {
    const FREQ_LIST: &'static [u32] = &[32000, 44100, 48000, 88200, 96000, 192000];
}

impl SamplingClockSourceOperation for AureonClkProtocol {
    const DST: SignalAddr = SignalAddr::Subunit(SignalSubunitAddr{
        subunit: MUSIC_SUBUNIT_0,
        plug_id: 0x03,
    });

    const SRC_LIST: &'static [SignalAddr] = &[
        // Internal.
        SignalAddr::Subunit(SignalSubunitAddr{subunit: MUSIC_SUBUNIT_0, plug_id: 0x03}),
    ];
}
