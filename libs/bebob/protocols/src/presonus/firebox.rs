// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol implementation for Firebox.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by PreSonus for Firebox.
//!
//! ## Diagram of internal signal flow
//!
//! ```text
//! analog-input-1/2 ------------------------+----------------> stream-output-1/2
//! analog-input-3/4 ------------------------|-+--------------> stream-output-3/4
//! analog-input-5/6 ------------------------|-|-+------------> stream-output-5/6
//!                                          | | |
//!                                          v v v
//!                                       ++=======++
//!                                       || 8 x 2 ||
//!                  stream-source-1/2 -> || mixer ||
//!                  (one source only)    ++=======++
//!                       ^ ^ ^ ^              |
//!                       | | | |        mixer-output-1/2
//!                       | | | |           | | | | |
//! stream-input-1/2 -----+-|-|-|---+------or-|-|-|-|---------> analog-output-1/2
//! stream-input-3/4 -------+-|-|---|-+------or-|-|-|---------> analog-output-3/4
//! stream-input-5/6 ---------+-|---|-|-+------or-|-|---------> analog-output-5/6
//! stream-input-7/8 -----------+---|-|-|-+------or-|---------> digital-output-1/2
//!                                 | | | |         |
//!                                 v v v v         v
//!                                 (one source only)
//!                                         |
//!                                         +-----------------> headphone-1/2
//! ```

use ta1394::*;

use crate::*;

/// The protocol implementation of clock operation.
#[derive(Default)]
pub struct FireboxClkProtocol;

impl MediaClockFrequencyOperation for FireboxClkProtocol {
    const FREQ_LIST: &'static [u32] = &[44100, 48000, 88200, 96000];
}

impl SamplingClockSourceOperation for FireboxClkProtocol {
    const DST: SignalAddr = SignalAddr::Subunit(SignalSubunitAddr{
        subunit: MUSIC_SUBUNIT_0,
        plug_id: 0x05,
    });

    const SRC_LIST: &'static [SignalAddr] = &[
        // Internal.
        SignalAddr::Subunit(SignalSubunitAddr{subunit: MUSIC_SUBUNIT_0, plug_id: 0x06}),
        // S/PDIF in coaxial interface.
        SignalAddr::Unit(SignalUnitAddr::Ext(0x03)),
    ];
}

/// The protocol implementation of physical output.
pub struct FireboxPhysOutputProtocol;

impl AvcLevelOperation for FireboxPhysOutputProtocol {
    const ENTRIES: &'static [(u8, AudioCh)] = &[
        (0x01, AudioCh::Each(0)),
        (0x01, AudioCh::Each(1)),
        (0x02, AudioCh::Each(0)),
        (0x02, AudioCh::Each(1)),
        (0x03, AudioCh::Each(0)),
        (0x03, AudioCh::Each(1)),
    ];
}

impl AvcMuteOperation for FireboxPhysOutputProtocol {}

/// The protocol implementation of headphone.
pub struct FireboxHeadphoneProtocol;

impl AvcLevelOperation for FireboxHeadphoneProtocol {
    const ENTRIES: &'static [(u8, AudioCh)] = &[
        (0x04, AudioCh::Each(0)),
        (0x04, AudioCh::Each(1)),
    ];
}

impl AvcMuteOperation for FireboxHeadphoneProtocol {}

/// The protocol implementation of physical source for mixer.
pub struct FireboxMixerPhysSourceProtocol;

impl AvcLevelOperation for FireboxMixerPhysSourceProtocol {
    const ENTRIES: &'static [(u8, AudioCh)] = &[
        (0x05, AudioCh::Each(0)),
        (0x05, AudioCh::Each(1)),
        (0x06, AudioCh::Each(0)),
        (0x06, AudioCh::Each(1)),
        (0x07, AudioCh::Each(0)),
        (0x07, AudioCh::Each(1)),
    ];
}

impl AvcLrBalanceOperation for FireboxMixerPhysSourceProtocol {}

impl AvcMuteOperation for FireboxMixerPhysSourceProtocol {}

/// The protocol implementation of stream source for mixer.
pub struct FireboxMixerStreamSourceProtocol;

impl AvcLevelOperation for FireboxMixerStreamSourceProtocol {
    const ENTRIES: &'static [(u8, AudioCh)] = &[
        (0x08, AudioCh::Each(0)),
    ];
}

impl AvcMuteOperation for FireboxMixerStreamSourceProtocol {}

/// The protocol implementation of mixer output.
pub struct FireboxMixerOutputProtocol;

impl AvcLevelOperation for FireboxMixerOutputProtocol {
    const ENTRIES: &'static [(u8, AudioCh)] = &[
        (0x09, AudioCh::Each(0)),
        (0x09, AudioCh::Each(1)),
    ];
}

impl AvcLrBalanceOperation for FireboxMixerOutputProtocol {}

impl AvcMuteOperation for FireboxMixerOutputProtocol {}
