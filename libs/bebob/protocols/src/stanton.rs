// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol implementation for Stanton Magnetics Final Scratch 2 ScratchAmp.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by Stanton Magnetics for Final Scratch 2 ScratchAmp.
//!
//! DM1000E is used for Stanton Scratchamp.
//!
//! ## Diagram of internal signal flow
//!
//! ```text
//! analog-input-1/2 -----------------> stream-output-1/2
//! analog-input-3/4 -----------------> stream-output-3/4
//! analog-input-5/6 -----------------> stream-output-5/6
//!
//! stream-input-1/2 -----------------> analog-output-1/2
//! stream-input-3/4 -----------------> analog-output-3/4
//! stream-input-5/6 -----------------> headphone-1/2
//! ```
//!
//! The protocol implementation for Stanton Scratchamp was written with firmware version below:
//!
//! ```sh
//! $ cargo run --bin bco-bootloader-info -- /dev/fw1
//! protocol:
//!   version: 1
//! bootloader:
//!   timestamp: 2004-10-06T10:07:36+0000
//!   version: 0.0.0
//! hardware:
//!   GUID: 0x1000000000126000
//!   model ID: 0x000001
//!   revision: 0.0.3
//! software:
//!   timestamp: 2004-01-01T12:00:00+0000
//!   ID: 0x00000000
//!   revision: 0.0.0
//! image:
//!   base address: 0x0
//!   maximum size: 0x120000
//! ```

use super::*;

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

/// The protocol implementation for physical output of Scratchamp.
#[derive(Default)]
pub struct ScratchampOutputProtocol;

impl AvcLevelOperation for ScratchampOutputProtocol {
    const ENTRIES: &'static [(u8, AudioCh)] = &[
        (0x01, AudioCh::Each(0)), // analog-output-1
        (0x01, AudioCh::Each(1)), // analog-output-2
        (0x02, AudioCh::Each(0)), // analog-output-3
        (0x02, AudioCh::Each(1)), // analog-output-4
    ];
}

/// The protocol implementation for headphone output of Scratchamp.
#[derive(Default)]
pub struct ScratchampHeadphoneProtocol;

impl AvcLevelOperation for ScratchampHeadphoneProtocol {
    const ENTRIES: &'static [(u8, AudioCh)] = &[
        (0x03, AudioCh::Each(0)), // headphone-1
        (0x03, AudioCh::Each(1)), // headphone-2
    ];
}
