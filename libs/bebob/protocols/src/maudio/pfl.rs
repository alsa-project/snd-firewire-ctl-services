// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol implementation for M-Audio ProFire Lightbridge.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by M-Audio ProFire Lightbridge

use crate::*;

use ta1394::ccm::{SignalAddr, SignalSubunitAddr, SignalUnitAddr};
use ta1394::MUSIC_SUBUNIT_0;

/// The protocol implementation for media and sampling clock of ProFire Lightbridge.
#[derive(Default)]
pub struct PflClkProtocol;

impl MediaClockFrequencyOperation for PflClkProtocol {
    const FREQ_LIST: &'static [u32] = &[44100, 48000, 88200, 96000];
}

impl SamplingClockSourceOperation for PflClkProtocol {
    const DST: SignalAddr = SignalAddr::Subunit(SignalSubunitAddr {
        subunit: MUSIC_SUBUNIT_0,
        plug_id: 0x07,
    });

    const SRC_LIST: &'static [SignalAddr] = &[
        // Internal
        SignalAddr::Subunit(SignalSubunitAddr {
            subunit: MUSIC_SUBUNIT_0,
            plug_id: 0x08,
        }),
        // S/PDIF
        SignalAddr::Unit(SignalUnitAddr::Ext(0x01)),
        // Optical iface 1
        SignalAddr::Unit(SignalUnitAddr::Ext(0x02)),
        // Optical iface 2
        SignalAddr::Unit(SignalUnitAddr::Ext(0x03)),
        // Optical iface 3
        SignalAddr::Unit(SignalUnitAddr::Ext(0x04)),
        // Optical iface 4
        SignalAddr::Unit(SignalUnitAddr::Ext(0x05)),
        // Word clock
        SignalAddr::Unit(SignalUnitAddr::Ext(0x06)),
    ];
}
