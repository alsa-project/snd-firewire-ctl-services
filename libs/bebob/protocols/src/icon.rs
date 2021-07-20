// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol implementation for models of Icon Digital International.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by Icon for FireWire models.
//!

//! ## Diagram of internal signal flow for Firexon
//!
//! ```text
//! analog-input-1/2 --------+------------------------------> stream-output-1/2
//! analog-input-3/4 --------|-+----------------------------> stream-output-3/4
//! digital-input-1/2 -------|-|-+--------------------------> stream-output-5/6
//!                          | | |
//!                          v v v
//!                      ++=========++
//!                      ||  6 x 2  ||
//!                      || monitor ||
//!                      ++=========++
//!                            |
//!                            v              ++=======++
//!                    monitor-output-1/2 --> || 4 x 2 ||
//!                                           || mixer ||
//!                            +------------> ||       ||
//!                            |              ++=======++
//!                            |                   |
//!                            |            mixer-output-1/2
//!                            |                  | |
//! stream-input-1/2 ----------+                  | +-------> analog-output-1/2
//!                                               v
//! stream-input-3/4 ---------------------(one source only)-> analog-output-3/4
//!                                               ^
//! stream-input-5/6 -----------------------------+---------> digital-output-1/2
//! ```

use ta1394::*;

use crate::*;

/// The protocol implementation of clock operation.
#[derive(Default)]
pub struct FirexonClkProtocol;

impl MediaClockFrequencyOperation for FirexonClkProtocol {
    const FREQ_LIST: &'static [u32] = &[44100, 48000, 88200, 96000, 176400, 192000];
}

impl SamplingClockSourceOperation for FirexonClkProtocol {
    const DST: SignalAddr = SignalAddr::Subunit(SignalSubunitAddr{
        subunit: MUSIC_SUBUNIT_0,
        plug_id: 0x05,
    });

    const SRC_LIST: &'static [SignalAddr] = &[
        // Internal.
        SignalAddr::Subunit(SignalSubunitAddr{subunit: MUSIC_SUBUNIT_0, plug_id: 0x05}),
        // S/PDIF in coaxial interface.
        SignalAddr::Unit(SignalUnitAddr::Ext(0x03)),
    ];
}

/// The protocol implementation of physical output.
pub struct FirexonPhysOutputProtocol;

impl AvcLevelOperation for FirexonPhysOutputProtocol {
    const ENTRIES: &'static [(u8, AudioCh)] = &[
        (0x06, AudioCh::Each(0)), // analog-output-1
        (0x06, AudioCh::Each(1)), // analog-output-2
        (0x07, AudioCh::Each(0)), // analog-output-3
        (0x07, AudioCh::Each(1)), // analog-output-4
    ];
}

impl AvcLrBalanceOperation for FirexonPhysOutputProtocol {}

impl AvcMuteOperation for FirexonPhysOutputProtocol {}

impl AvcSelectorOperation for FirexonPhysOutputProtocol {
    // NOTE: "analog-output-3/4" (not "analog-output-1/2")
    const FUNC_BLOCK_ID_LIST: &'static [u8] = &[0x01];
    // NOTE: "mixer-output-1/2", "stream-input-3/4", "stream-input-5/6"
    const INPUT_PLUG_ID_LIST: &'static [u8] = &[0x00, 0x01, 0x02];
}

/// The protocol implementation of source to monitor mixer for physical inputs
pub struct FirexonMonitorSourceProtocol;

impl AvcLevelOperation for FirexonMonitorSourceProtocol{
    const ENTRIES: &'static [(u8, AudioCh)] = &[
        (0x01, AudioCh::Each(0)), // analog-input-1
        (0x01, AudioCh::Each(1)), // analog-input-2
        (0x02, AudioCh::Each(0)), // analog-input-3
        (0x02, AudioCh::Each(1)), // analog-input-4
        (0x03, AudioCh::Each(0)), // digital-input-5
        (0x03, AudioCh::Each(1)), // digital-input-6
    ];
}

impl AvcLrBalanceOperation for FirexonMonitorSourceProtocol {}

impl AvcMuteOperation for FirexonMonitorSourceProtocol {}

/// The protocol implementation of source to mixer for stream input and output of the monitor mixer.
pub struct FirexonMixerSourceProtocol;

impl AvcLevelOperation for FirexonMixerSourceProtocol {
    const ENTRIES: &'static [(u8, AudioCh)] = &[
        (0x04, AudioCh::All), // stream-input-1/2
        (0x05, AudioCh::All), // monitor-output-1/2
    ];
}
