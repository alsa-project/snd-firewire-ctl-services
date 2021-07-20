// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol implementation for M-Audio ProFire Lightbridge.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by M-Audio ProFire Lightbridge
//!
//! ## Diagram of internal signal flow
//!
//! ```text
//! adat-input-1/2 ----> stream-output-1/2
//! adat-input-3/4 ----> stream-output-3/4
//! adat-input-5/6 ----> stream-output-5/6
//! adat-input-7/8 ----> stream-output-7/8
//! adat-input-9/10 ---> stream-output-9/10
//! adat-input-11/12 --> stream-output-11/12
//! adat-input-13/14 --> stream-output-13/14
//! adat-input-15/16 --> stream-output-15/16
//! adat-input-17/18 --> stream-output-17/18
//! adat-input-19/20 --> stream-output-19/20
//! adat-input-21/22 --> stream-output-21/22
//! adat-input-23/24 --> stream-output-23/24
//! adat-input-25/26 --> stream-output-25/26
//! adat-input-27/28 --> stream-output-27/28
//! adat-input-29/30 --> stream-output-29/30
//! adat-input-31/32 --> stream-output-31/32
//! spdif-input-1/2 ---> stream-output-33/34
//!
//! stream-input-1/2 ----> adat-output-1/2
//! stream-input-3/4 ----> adat-output-3/4
//! stream-input-5/6 ----> adat-output-5/6
//! stream-input-7/8 ----> adat-output-7/8
//! stream-input-9/10 ---> adat-output-9/10
//! stream-input-11/12 --> adat-output-11/12
//! stream-input-13/14 --> adat-output-13/14
//! stream-input-15/16 --> adat-output-15/16
//! stream-input-17/18 --> adat-output-17/18
//! stream-input-19/20 --> adat-output-19/20
//! stream-input-21/22 --> adat-output-21/22
//! stream-input-23/24 --> adat-output-23/24
//! stream-input-25/26 --> adat-output-25/26
//! stream-input-27/28 --> adat-output-27/28
//! stream-input-29/30 --> adat-output-29/30
//! stream-input-31/32 --> adat-output-31/32
//! stream-input-33/34 --> spdif-output-1/2
//! stream-input-35/36 --> analog-output-1/2
//! ```

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
