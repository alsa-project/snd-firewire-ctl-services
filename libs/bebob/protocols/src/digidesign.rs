// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol implementation for Digidesign Mbox 2 Pro.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by Digidesign for Mbox 2 pro.
//!
//! ## Diagram of internal signal flow
//!
//! ```text
//! digital-input-1/2 -------------------------> stream-output-1/2
//! analog-input-1/2 ------------+-------------> stream-output-3/4
//! analog-input-3/4 ------------|-+-----------> stream-output-5/6
//!                              | |
//!                              v v
//!                          ++=======++
//!                     +--> || 6 x 2 ||-------> monitor-output-1/2
//!                     |    || mixer ||
//!                     |    ++=======++
//! stream-input-1/2 ---+----------------------> digital-output-1/2
//! stream-input-3/4 --------------------------> analog-output-1/2
//! stream-input-5/6 --------------------------> analog-output-3/4
//! ```
//!
//! None of the above audio signals is configurable by software.

use super::*;

/// The protocol implementation of operation for media clock and sampling clock.
#[derive(Default)]
pub struct Mbox2proClkProtocol;

impl MediaClockFrequencyOperation for Mbox2proClkProtocol {
    const FREQ_LIST: &'static [u32] = &[44100, 48000, 88200, 96000];
}

impl SamplingClockSourceOperation for Mbox2proClkProtocol {
    const DST: SignalAddr = SignalAddr::Subunit(SignalSubunitAddr{
        subunit: MUSIC_SUBUNIT_0,
        plug_id: 0x01,
    });

    const SRC_LIST: &'static [SignalAddr] = &[
        // Internal.
        SignalAddr::Subunit(SignalSubunitAddr{subunit: MUSIC_SUBUNIT_0, plug_id: 0x01}),
        // Internal with S/PDIF output.
        SignalAddr::Subunit(SignalSubunitAddr{subunit: MUSIC_SUBUNIT_0, plug_id: 0x07}),
        // S/PDIF input in coaxial interface.
        SignalAddr::Unit(SignalUnitAddr::Ext(0x03)),
        // Word clock input in BNC interface.
        SignalAddr::Unit(SignalUnitAddr::Ext(0x04)),
        // Word clock input or S/PDIF input.
        SignalAddr::Unit(SignalUnitAddr::Ext(0x05)),
    ];
}
