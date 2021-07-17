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

impl LevelOperation for Quatafire610PhysInputProtocol {
    const FUNC_BLOCK_ID_LIST: &'static [u8] = &[0x01, 0x02, 0x03];
    const CH_ID_LIST: &'static [[u8; 2]] = &[[0x00, 0x01], [0x00, 0x01], [0x00, 0x01]];
}

impl LrBalanceOperation for Quatafire610PhysInputProtocol {}

/// The protocol implementation for physical output of Quatafire 610.
#[derive(Default)]
pub struct Quatafire610PhysOutputProtocol;

impl LevelOperation for Quatafire610PhysOutputProtocol {
    const FUNC_BLOCK_ID_LIST: &'static [u8] = &[0x04, 0x04, 0x04, 0x04];
    const CH_ID_LIST: &'static [[u8; 2]] =
        &[[0x00, 0x01], [0x02, 0x03], [0x04, 0x05], [0x06, 0x07]];
}
