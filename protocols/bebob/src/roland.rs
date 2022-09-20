// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol implementation for Roland Edirol FA series.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by Roland for Edirol FA series.
//!
//! DM1000 is used for Roland FA-66.
//!
//! ## Diagram of internal signal flow for FA-66
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
//! stream-input-5/6 --------------------------> digital-output-1/2
//! ```
//!
//! The protocol implementation for Roland FA-66 was written with firmware version below:
//!
//! ```sh
//! $ cargo run --bin bco-bootloader-info -- /dev/fw1
//! protocol:
//!   version: 1
//! bootloader:
//!   timestamp: 2004-11-26T04:06:23+0000
//!   version: 0.0.0
//! hardware:
//!   GUID: 0x00c3216f0040ab00
//!   model ID: 0x000002
//!   revision: 0.0.1
//! software:
//!   timestamp: 2004-11-26T02:44:31+0000
//!   ID: 0x00010049
//!   revision: 0.0.256
//! image:
//!   base address: 0x20080000
//!   maximum size: 0x180000
//! ```

use super::*;

/// The protocol implementation for media and sampling clock. They are not configurable by
/// software.
#[derive(Default, Debug)]
pub struct FaClkProtocol;

impl MediaClockFrequencyOperation for FaClkProtocol {
    const FREQ_LIST: &'static [u32] = &[44100, 48000, 96000, 192000];
}

impl SamplingClockSourceOperation for FaClkProtocol {
    const DST: SignalAddr = SignalAddr::Subunit(SignalSubunitAddr {
        subunit: MUSIC_SUBUNIT_0,
        plug_id: 0x05,
    });

    const SRC_LIST: &'static [SignalAddr] = &[SignalAddr::Subunit(SignalSubunitAddr {
        subunit: MUSIC_SUBUNIT_0,
        plug_id: 0x05,
    })];
}

// NOTE: Mute function in Feature control of audio function block has no effect.

/// The protocol implementation for physical input of FA-66. Any operation is effective when
/// enabling hardware switch with 'SOFT CTRL'.
#[derive(Default)]
pub struct Fa66MixerAnalogSourceProtocol;

impl AvcLevelOperation for Fa66MixerAnalogSourceProtocol {
    const ENTRIES: &'static [(u8, AudioCh)] = &[
        (0x01, AudioCh::Each(0)),
        (0x01, AudioCh::Each(1)),
        (0x02, AudioCh::Each(0)),
        (0x02, AudioCh::Each(1)),
        (0x03, AudioCh::Each(0)),
        (0x03, AudioCh::Each(1)),
    ];
}

impl AvcLrBalanceOperation for Fa66MixerAnalogSourceProtocol {}

/// The protocol implementation for physical input of FA-101. Any operation is effective when
/// enabling hardware switch with 'SOFT CTRL'.
#[derive(Default)]
pub struct Fa101MixerAnalogSourceProtocol;

impl AvcLevelOperation for Fa101MixerAnalogSourceProtocol {
    const ENTRIES: &'static [(u8, AudioCh)] = &[
        (0x01, AudioCh::Each(0)),
        (0x01, AudioCh::Each(1)),
        (0x02, AudioCh::Each(0)),
        (0x02, AudioCh::Each(1)),
        (0x03, AudioCh::Each(0)),
        (0x03, AudioCh::Each(1)),
        (0x04, AudioCh::Each(0)),
        (0x04, AudioCh::Each(1)),
        (0x05, AudioCh::Each(0)),
        (0x05, AudioCh::Each(1)),
    ];
}

impl AvcLrBalanceOperation for Fa101MixerAnalogSourceProtocol {}
