// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol implementation for Behringer Firepower series.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by Behringer for Firepower series.
//!
//! DM1500 is used for Behringer Firepower FCA610.
//!
//! ## Diagram of internal signal flow for FCA610
//!
//! ```text
//! analog-input-1/2  ----------+--------------> stream-output-1/2
//! analog-input-3/4  ----------|-+------------> stream-output-3/4
//! digital-input-1/2 ----------|-|-+----------> stream-output-5/6
//!                             | | |
//!                             v v v
//!                          ++=======++
//! stream-input-1/2 ------> || 8 x 2 ||
//!                          || mixer ||-------> analog-output-1/2
//!                          ++=======++
//! stream-input-3/4 --------------------------> analog-output-3/4
//! stream-input-5/6 --------------------------> analog-output-5/6
//! stream-input-7/8 --------------------------> digital-output-1/2
//! ```
//!
//! The protocol implementation for Behringer FCA610 was written with firmware version below:
//!
//! ```sh
//! $ cargo run --bin bco-bootloader-info -- /dev/fw1
//! protocol:
//!   version: 3
//! bootloader:
//!   timestamp: 2012-10-15T10:47:10+0000
//!   version: 0.0.0
//! hardware:
//!   GUID: 0x0002ad7300156400
//!   model ID: 0x000003
//!   revision: 0.0.0
//! software:
//!   timestamp: 2012-11-02T03:34:31+0000
//!   ID: 0x00000610
//!   revision: 0.0.8348
//! image:
//!   base address: 0x400c0080
//!   maximum size: 0x15b520
//! ```

use super::*;

/// The protocol implementation for media and sampling clock of FCA 610.
#[derive(Default)]
pub struct Fca610ClkProtocol;

impl MediaClockFrequencyOperation for Fca610ClkProtocol {
    const FREQ_LIST: &'static [u32] = &[44100, 48000, 88200, 96000];
}

impl SamplingClockSourceOperation for Fca610ClkProtocol {
    const DST: SignalAddr = SignalAddr::Subunit(SignalSubunitAddr {
        subunit: MUSIC_SUBUNIT_0,
        plug_id: 0x05,
    });

    const SRC_LIST: &'static [SignalAddr] = &[
        // Device internal clock
        SignalAddr::Unit(SignalUnitAddr::Ext(0x04)),
        // S/PDIF
        SignalAddr::Unit(SignalUnitAddr::Ext(0x03)),
        // Firewire-bus. This is the same source as Internal in former BeBoB models.
        SignalAddr::Subunit(SignalSubunitAddr {
            subunit: MUSIC_SUBUNIT_0,
            plug_id: 0x07,
        }),
    ];
}
