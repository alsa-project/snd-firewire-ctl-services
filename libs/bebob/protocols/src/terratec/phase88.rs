// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol implementation for Terratec Phase 88 FW.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by Terratec for Phase 88 FW.

use crate::*;

/// The protocol implementation for media and sampling clock.
#[derive(Default)]
pub struct Phase88ClkProtocol;

impl MediaClockFrequencyOperation for Phase88ClkProtocol {
    const FREQ_LIST: &'static [u32] = &[32000, 44100, 48000, 88200, 96000];
}

const CLK_SRC_EXT_FB_ID: u8 = 0x08;
const CLK_SRC_EXT_WORD_FB_ID: u8 = 0x09;

impl SamplingClockSourceOperation for Phase88ClkProtocol {
    const DST: SignalAddr = SignalAddr::Subunit(SignalSubunitAddr{
        subunit: MUSIC_SUBUNIT_0,
        plug_id: 0x03,
    });

    const SRC_LIST: &'static [SignalAddr] = &[
        // Internal.
        SignalAddr::Subunit(SignalSubunitAddr{subunit: MUSIC_SUBUNIT_0, plug_id: 0x05}),
        // S/PDIF input in optical interface.
        SignalAddr::Unit(SignalUnitAddr::Ext(0x04)),
        // Word clock input in BNC interface.
        SignalAddr::Unit(SignalUnitAddr::Ext(0x07)),
    ];

    fn read_clk_src(avc: &BebobAvc, timeout_ms: u32) -> Result<usize, Error> {
        let mut op = AudioSelector::new(CLK_SRC_EXT_FB_ID, CtlAttr::Current, 0xff);
        avc.status(&AUDIO_SUBUNIT_0_ADDR, &mut op, timeout_ms)?;
        if op.input_plug_id == 0x00 {
            // Internal.
            Ok(0)
        } else {
            let mut op = AudioSelector::new(CLK_SRC_EXT_WORD_FB_ID, CtlAttr::Current, 0xff);
            avc.status(&AUDIO_SUBUNIT_0_ADDR, &mut op, timeout_ms)?;
            if op.input_plug_id == 0x00 {
                // S/PDIF.
                Ok(1)
            } else {
                // Word clock.
                Ok(2)
            }
        }
    }

    fn write_clk_src(avc: &BebobAvc, idx: usize, timeout_ms: u32) -> Result<(), Error> {
        let (is_ext, ext_is_word) = match idx {
            0 => (0u8, 0u8),
            1 => (0u8, 1u8),
            2 => (1u8, 1u8),
            _ => {
                let msg = format!("Invalid index of source of clock: {}", idx);
                Err(Error::new(FileError::Inval, &msg))?
            }
        };

        let mut op = AudioSelector::new(CLK_SRC_EXT_WORD_FB_ID, CtlAttr::Current, ext_is_word);
        avc.status(&AUDIO_SUBUNIT_0_ADDR, &mut op, timeout_ms)?;

        let mut op = AudioSelector::new(CLK_SRC_EXT_FB_ID, CtlAttr::Current, is_ext);
        avc.status(&AUDIO_SUBUNIT_0_ADDR, &mut op, timeout_ms)?;

        Ok(())
    }
}
