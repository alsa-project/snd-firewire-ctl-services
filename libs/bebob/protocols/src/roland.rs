// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol implementation for Roland Edirol FA series.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by Roland for Edirol FA series.

use super::*;

/// The protocol implementation for media and sampling clock. They are not configurable by
/// software.
#[derive(Default)]
pub struct FaClkProtocol;

impl MediaClockFrequencyOperation for FaClkProtocol {
    const FREQ_LIST: &'static [u32] = &[44100, 48000, 96000, 192000];
}

impl SamplingClockSourceOperation for FaClkProtocol {
    const DST: SignalAddr = SignalAddr::Subunit(SignalSubunitAddr {
        subunit: MUSIC_SUBUNIT_0,
        plug_id: 0x05,
    });

    const SRC_LIST: &'static [SignalAddr] = &[
        SignalAddr::Subunit(SignalSubunitAddr {
            subunit: MUSIC_SUBUNIT_0,
            plug_id: 0x05,
        }),
    ];
}
