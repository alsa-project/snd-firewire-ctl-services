// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol implementation for Terratec Aureon 7.1 FW.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by Terratec for Aureon 7.1 FW.
//!
//! ## Diagram of internal signal flow
//!
//! ```text
//! analog-input-1/2 ----------+
//! analog-input-3/4 ----------|-+
//! analog-input-5/6 ----------|-|-+
//! digital-input-1/2 ---------|-|-|-+
//!                            | | | |
//!                       (one source only)
//!                       monitor-source-1/2 -------------------> stream-output-1/2
//!                                |
//!                                v
//!                          ++=========++
//! stream-input-1/2 --------||         || -> mixer-output-7/8 -> analog-output-7/8
//! stream-input-3/4 --------|| 10 x  8 || -> mixer-output-5/6 -> analog-output-5/6
//! stream-input-5/6 --------||  mixer  || -> mixer-output-2/3 -> analog-output-3/4
//! stream-input-7/8 --------||         || -> mixer-output-1/2 -> analog-output-1/2
//!        |                 +==========++           |
//!        +----------------------------------------or----------> digital-output-1/2
//! ```

use crate::*;

/// The protocol implementation for media and sampling clock.
#[derive(Default)]
pub struct AureonClkProtocol;

impl MediaClockFrequencyOperation for AureonClkProtocol {
    const FREQ_LIST: &'static [u32] = &[32000, 44100, 48000, 88200, 96000, 192000];
}

impl SamplingClockSourceOperation for AureonClkProtocol {
    const DST: SignalAddr = SignalAddr::Subunit(SignalSubunitAddr{
        subunit: MUSIC_SUBUNIT_0,
        plug_id: 0x03,
    });

    const SRC_LIST: &'static [SignalAddr] = &[
        // Internal.
        SignalAddr::Subunit(SignalSubunitAddr{subunit: MUSIC_SUBUNIT_0, plug_id: 0x03}),
    ];
}

/// The protocol implementation of mixer output.
#[derive(Default)]
pub struct AureonMixerOutputProtocol;

impl AvcLevelOperation for AureonMixerOutputProtocol {
    const ENTRIES: &'static [(u8, AudioCh)] = &[
        (0x01, AudioCh::Each(0)), // mixer-output-1
        (0x01, AudioCh::Each(1)), // mixer-output-2
        (0x01, AudioCh::Each(2)), // mixer-output-3
        (0x01, AudioCh::Each(3)), // mixer-output-4
        (0x01, AudioCh::Each(4)), // mixer-output-5
        (0x01, AudioCh::Each(5)), // mixer-output-6
        (0x01, AudioCh::Each(6)), // mixer-output-7
        (0x01, AudioCh::Each(7)), // mixer-output-8
    ];
}

impl AvcMuteOperation for AureonMixerOutputProtocol {}

/// The protocol implementation of analog input.
#[derive(Default)]
pub struct AureonPhysInputProtocol;

impl AvcLevelOperation for AureonPhysInputProtocol {
    const ENTRIES: &'static [(u8, AudioCh)] = &[
        (0x02, AudioCh::All), // analog-input-1/2
        (0x03, AudioCh::All), // analog-input-3/4
    ];
}

/// The protocol implementation of monitor output.
#[derive(Default)]
pub struct AureonMonitorSourceProtocol;

impl AvcSelectorOperation for AureonMonitorSourceProtocol {
    const FUNC_BLOCK_ID_LIST: &'static [u8] = &[0x01];
    // NOTE: "analog-input-1/2", "analog-input-3/4", "analog-input-5/6", "digital-input-1/2"
    const INPUT_PLUG_ID_LIST: &'static [u8] = &[0x00, 0x01, 0x02, 0x03];
}

/// The protocol implementation of monitor source.
#[derive(Default)]
pub struct AureonMonitorOutputProtocol;

impl AvcLevelOperation for AureonMonitorOutputProtocol {
    const ENTRIES: &'static [(u8, AudioCh)] = &[
        (0x04, AudioCh::All), // monitor-output-1/2
    ];
}

impl AvcMuteOperation for AureonMonitorOutputProtocol {}

/// The protocol implementation of spdif output.
#[derive(Default)]
pub struct AureonSpdifOutputProtocol;

impl AvcSelectorOperation for AureonSpdifOutputProtocol {
    const FUNC_BLOCK_ID_LIST: &'static [u8] = &[0x02];
    // NOTE: "mixer-output-1/2", "stream-input-9/10"
    const INPUT_PLUG_ID_LIST: &'static [u8] = &[0x00, 0x01];
}
