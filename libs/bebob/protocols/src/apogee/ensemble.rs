// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol implementation for Apogee Electronics Ensemble FireWire.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by Apogee Electronics Ensemble FireWire.
//!
//! ## Diagram of internal signal flow for FireWire 410
//!
//! ```text
//!                                ++==========++
//! analog-inputs (8 channels) --> ||  36x36   ||
//! spdif-inputs (2 channels) ---> ||  capture || --> stream-outputs (18 channels)
//! adat-inputs (8 channels) ----> ||  router  ||
//!                                ++==========++
//!
//!                                ++==========++
//! analog-inputs (8 channels) --> ||  36x4    ||
//! spdif-inputs (2 channels) ---> ||          || --> mixer-outputs (4 channels)
//! adat-inputs (8 channels) ----> ||  mixer   ||
//! stream-inputs (18 channels) -> ||          ||
//!                                ++==========++
//!
//!                                ++==========++
//! analog-inputs (8 channels) --> ||  40x18   ||
//! spdif-inputs (2 channels) ---> ||          || --> analog-outputs (8 channels)
//! adat-inputs (8 channels) ----> || playback || --> spdif-outputs (2 channels)
//! stream-inputs (18 channels) -> ||          || --> adat-outputs (8 channels)
//! mixer-outputs (4 channels) --> ||  router  ||
//!                                ++==========++
//!
//! (source) ----------------------------> spdif-output-1/2
//!                           ^
//!                           |
//!                 ++================++
//!                 || rate converter || (optional)
//!                 ++================++
//!                           |
//!                           v
//! spdif-input-1/2 ------------------------> (destination)
//! 
//! analog-input-1/2 ------------------------> (destination)
//! analog-input-3/4 ------------------------> (destination)
//! analog-input-5/6 ------------------------> (destination)
//! analog-input-7/8 ------------------------> (destination)
//! spdif-input-1/2 -------------------------> (destination)
//!                           ^
//!                           |
//!                ++==================++
//!                || format converter || (optoinal)
//!                ++==================++
//!                           |
//!                           v
//! (source) ------------------------------> spdif-output-1/2
//! ```

use crate::*;

/// The protocol implementation for media and sampling clock of Ensemble FireWire.
#[derive(Default)]
pub struct EnsembleClkProtocol;

impl MediaClockFrequencyOperation for EnsembleClkProtocol {
    const FREQ_LIST: &'static [u32] = &[44100, 48000, 88200, 96000, 176400, 192000];
}

impl SamplingClockSourceOperation for EnsembleClkProtocol {
    const DST: SignalAddr = SignalAddr::Subunit(SignalSubunitAddr {
        subunit: MUSIC_SUBUNIT_0,
        plug_id: 7,
    });

    const SRC_LIST: &'static [SignalAddr] = &[
        // Internal
        SignalAddr::Subunit(SignalSubunitAddr {
            subunit: MUSIC_SUBUNIT_0,
            plug_id: 7,
        }),
        // S/PDIF-coax
        SignalAddr::Unit(SignalUnitAddr::Ext(4)),
        // Optical
        SignalAddr::Unit(SignalUnitAddr::Ext(5)),
        // Word clock
        SignalAddr::Unit(SignalUnitAddr::Ext(6)),
    ];
}
