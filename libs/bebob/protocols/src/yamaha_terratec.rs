// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol implementation for Yamaha Go and Terratec Phase 24 FW series.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by Yamaha and Terratec for Go and 24 FW series.

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
