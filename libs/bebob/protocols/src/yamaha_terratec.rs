// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol implementation for Yamaha Go and Terratec Phase 24 FW series.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by Yamaha and Terratec for Go and 24 FW series.
//!
//! ## Diagram of internal signal flow
//!
//! ```text
//! analog-input-1/2  --+----------------------------------------+-----> stream-output-1/2
//! digital-input-1/2 --|-+--------------------------------------|-+---> stream-output-3/4
//!                     | |                                      | |
//!                     | |      ++=======++                     | |
//!                     +-|----> ||       ||                     | |
//!                       +----> ||       ||                     | |
//!                              || 6 x 2 ||                     | |
//! stream-input-1/2 ---+------> || mixer ||--> mixer-output-1/2 | |
//! stream-input-3/4 ---|-+----> ||       ||           |         | |
//! stream-input-5/6 ---|-|-+--> ||       ||           |         | |
//!                     | | |    ++=======++           |         | |
//!                     +-|-|--------------------------|---------|-|--->
//!                     | +-|--------------------------|---------|-|--->
//!                     | | +--------------------------|---------|-|---> analog-output-1/2
//!                     | | |                          +---------|-|---> (one source only)
//!                     | | |                          |         +-|--->
//!                     | | |                          |         | +--->
//!                     | | |                          |         | |
//!                     +-|-|--------------------------|---------|-|--->
//!                     | +-|--------------------------|---------|-|--->
//!                     | | +--------------------------|---------|-|---> analog-output-3/4
//!                     | | |                          +---------|-|---> (one source only)
//!                     | | |                          |         +-|--->
//!                     | | |                          |         | +--->
//!                     | | |                          |         | |
//!                     +-|-|--------------------------|---------|-|--->
//!                       +-|--------------------------|---------|-|--->
//!                         +--------------------------|---------|-|---> digital-output-1/2
//!                                                    +---------|-|---> (one source only)
//!                                                              +-|--->
//!                                                                +--->
//! ```

use crate::*;

/// The protocol implementation of media and sampling clock for Yamaha Go 44/46 and PHASE 24/X24 FW;
pub struct GoPhase24ClkProtocol;

impl MediaClockFrequencyOperation for GoPhase24ClkProtocol {
    const FREQ_LIST: &'static [u32] = &[44100, 48000, 88200, 96000, 192000];
}

const CLK_SRC_FB_ID: u8 = 0x04;

impl SamplingClockSourceOperation for GoPhase24ClkProtocol {
    // NOTE: these destination and source can not be connected actually.
    const DST: SignalAddr = SignalAddr::Subunit(SignalSubunitAddr{
        subunit: MUSIC_SUBUNIT_0,
        plug_id: 0x04,
    });

    const SRC_LIST: &'static [SignalAddr] = &[
        // Internal.
        SignalAddr::Subunit(SignalSubunitAddr{
            subunit: MUSIC_SUBUNIT_0,
            plug_id: 0x03,
        }),
        // S/PDIF input.
        SignalAddr::Unit(SignalUnitAddr::Ext(0x01)),
    ];

    fn read_clk_src(avc: &BebobAvc, timeout_ms: u32) -> Result<usize, Error> {
        let mut op = AudioSelector::new(CLK_SRC_FB_ID, CtlAttr::Current, 0xff);
        avc.status(&AUDIO_SUBUNIT_0_ADDR, &mut op, timeout_ms)
            .map(|_| op.input_plug_id as usize)
    }

    fn write_clk_src(avc: &BebobAvc, val: usize, timeout_ms: u32) -> Result<(), Error> {
        let mut op = AudioSelector::new(CLK_SRC_FB_ID, CtlAttr::Current, val as u8);
        avc.control(&AUDIO_SUBUNIT_0_ADDR, &mut op, timeout_ms)
    }
}

/// The protocol implementation of physical output for optical models.
pub struct GoPhase24OptPhysOutputProtocol;

impl AvcLevelOperation for GoPhase24OptPhysOutputProtocol {
    const ENTRIES: &'static [(u8, AudioCh)] = &[
        (0x01, AudioCh::Each(0)), // analog-output-1
        (0x01, AudioCh::Each(1)), // analog-output-2
        (0x01, AudioCh::Each(2)), // analog-output-3
        (0x01, AudioCh::Each(3)), // analog-output-4
    ];
}

impl AvcMuteOperation for GoPhase24OptPhysOutputProtocol {}

/// The protocol implementation of mixer source gain.
pub struct GoPhase24MixerSourceProtocol;

impl AvcLevelOperation for GoPhase24MixerSourceProtocol {
    const ENTRIES: &'static [(u8, AudioCh)] = &[
        (0x06, AudioCh::Each(0)), // analog-input-1/2
        (0x06, AudioCh::Each(1)), // analog-input-1/2
        (0x07, AudioCh::Each(0)), // digital-input-1/2
        (0x07, AudioCh::Each(1)), // digital-input-1/2
        (0x03, AudioCh::Each(0)), // stream-input-1/2
        (0x03, AudioCh::Each(1)), // stream-input-1/2
        (0x04, AudioCh::Each(0)), // stream-input-3/4
        (0x04, AudioCh::Each(1)), // stream-input-3/4
        (0x05, AudioCh::Each(0)), // stream-input-5/6
        (0x05, AudioCh::Each(1)), // stream-input-5/6
    ];
}

impl AvcMuteOperation for GoPhase24MixerSourceProtocol {}

/// The protocol implementation of mixer output volume for coaxial models.
pub struct GoPhase24CoaxMixerOutputProtocol;

impl AvcLevelOperation for GoPhase24CoaxMixerOutputProtocol {
    const ENTRIES: &'static [(u8, AudioCh)] = &[
        (0x01, AudioCh::Each(0)),
        (0x01, AudioCh::Each(1)),
    ];
}

impl AvcMuteOperation for GoPhase24CoaxMixerOutputProtocol {}

/// The protocol implementation of mixer output volume for optical models.
pub struct GoPhase24OptMixerOutputProtocol;

impl AvcLevelOperation for GoPhase24OptMixerOutputProtocol {
    const ENTRIES: &'static [(u8, AudioCh)] = &[
        (0x02, AudioCh::Each(0)),
        (0x02, AudioCh::Each(1)),
    ];
}

impl AvcMuteOperation for GoPhase24OptMixerOutputProtocol {}
