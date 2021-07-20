// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol implementation for Firepod/FP10.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by PreSonus for Firepod/FP10.
//!
//! ## Diagram of internal signal flow
//!
//! ```text
//! analog-input-1/2 -----+-----------> stream-output-1/2
//! analog-input-3/4 -----|-+---------> stream-output-3/4
//! analog-input-5/6 -----|-|-+-------> stream-output-5/6
//! analog-input-7/8 -----|-|-|-+-----> stream-output-7/8
//! digital-input-1/2 ----|-|-|-|-----> stream-output-9/10
//!                       v v v v
//!                     ++=======++
//!                     || 10x2  ||
//!                     || mixer ||
//!                     ++=======++
//!                        ^   |
//! stream-input-1/2 ------+   +------> analog-output-1/2
//! stream-input-3/4 -----------------> analog-output-3/4
//! stream-input-5/6 -----------------> analog-output-5/6
//! stream-input-7/8 -----------------> analog-output-7/8
//! stream-input-9/10 ----------------> digital-output-1/2
//! ```

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

/// The protocol implementation for physical output.
pub struct Fp10PhysOutputProtocol;

impl AvcLevelOperation for Fp10PhysOutputProtocol {
    const ENTRIES: &'static [(u8, AudioCh)] = &[
        (0x01, AudioCh::Each(0)),
        (0x01, AudioCh::Each(1)),
        (0x02, AudioCh::Each(0)),
        (0x02, AudioCh::Each(1)),
        (0x03, AudioCh::Each(0)),
        (0x03, AudioCh::Each(1)),
        (0x04, AudioCh::Each(0)),
        (0x04, AudioCh::Each(1)),
    ];
}
