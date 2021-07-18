// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol implementation for ESI Quatafire series.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by Ego Systems (ESI) for Quatafire series.

use super::*;

use ta1394::ccm::{SignalAddr, SignalSubunitAddr};
use ta1394::MUSIC_SUBUNIT_0;

/// The protocol implementation for media and sampling clock of Quatafire 610.
#[derive(Default)]
pub struct Quatafire610ClkProtocol;

impl MediaClockFrequencyOperation for Quatafire610ClkProtocol {
    const FREQ_LIST: &'static [u32] = &[44100, 48000, 88200, 96000, 192000];
}

impl SamplingClockSourceOperation for Quatafire610ClkProtocol {
    const DST: SignalAddr = SignalAddr::Subunit(SignalSubunitAddr {
        subunit: MUSIC_SUBUNIT_0,
        plug_id: 0x01,
    });

    const SRC_LIST: &'static [SignalAddr] = &[
        // Internal.
        SignalAddr::Subunit(SignalSubunitAddr {
            subunit: MUSIC_SUBUNIT_0,
            plug_id: 0x01,
        }),
    ];
}

/// The protocol implementation for physical input of Quatafire 610.
#[derive(Default)]
pub struct Quatafire610PhysInputProtocol;

impl AvcLevelOperation for Quatafire610PhysInputProtocol {
    const ENTRIES: &'static [(u8, AudioCh)] = &[
        (0x01, AudioCh::Each(0)), // analog-input-1
        (0x01, AudioCh::Each(1)), // analog-input-2
        (0x02, AudioCh::Each(0)), // analog-input-3
        (0x02, AudioCh::Each(1)), // analog-input-4
        (0x03, AudioCh::Each(0)), // analog-input-5
        (0x03, AudioCh::Each(1)), // analog-input-6
    ];
}

impl AvcLrBalanceOperation for Quatafire610PhysInputProtocol {}

/// The protocol implementation for physical output of Quatafire 610.
#[derive(Default)]
pub struct Quatafire610PhysOutputProtocol;

impl AvcLevelOperation for Quatafire610PhysOutputProtocol {
    const ENTRIES: &'static [(u8, AudioCh)] = &[
        (0x04, AudioCh::Each(0)), // analog-output-1
        (0x04, AudioCh::Each(1)), // analog-output-2
        (0x04, AudioCh::Each(2)), // analog-output-3
        (0x04, AudioCh::Each(3)), // analog-output-4
        (0x04, AudioCh::Each(4)), // analog-output-5
        (0x04, AudioCh::Each(5)), // analog-output-6
        (0x04, AudioCh::Each(6)), // analog-output-7
        (0x04, AudioCh::Each(7)), // analog-output-8
    ];
}
