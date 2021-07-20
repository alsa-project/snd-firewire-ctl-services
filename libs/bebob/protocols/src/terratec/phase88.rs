// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol implementation for Terratec Phase 88 FW.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by Terratec for Phase 88 FW.
//!
//! ## Diagram of internal signal flow
//!
//! ```text
//!                        analog-input-1/2 ------+------------> stream-output-1/2
//!                        analog-input-3/4 ------|------------> stream-output-3/4
//!                        analog-input-5/6 ------|-|-+--------> stream-output-5/6
//! line-input-7/8 ---or-> analog-input-7/8 ------|-|-|-+------> stream-output-7/8
//! mic-input-7/8  ---+                           | | | |
//!                        digital-input-1/2 -----|-|-|-|-+----> stream-output-9/10
//!                                               | | | | |
//!                                               v v v v v
//!                                             ++=========++
//!                                             ||         ||
//!                        stream-source-1/2 -> || 12 x  2 ||
//!                        (one source only)    ||  mixer  ||
//!                            ^ ^ ^ ^ ^        ||         ||
//!                            | | | | |        ++---------++
//!                            | | | | |              v
//!                            | | | | |       mixer-output-1/2
//!                            | | | | |              v
//!                            | | | | |     (one destination only)
//!                            | | | | |          | | | | |
//! stream-input-1/2 ----------+-|-|-|-|---------or-|-|-|-|----> analog-output-1/2
//! stream-input-3/4 ------------+-|-|-|-----------or-|-|-|----> analog-output-3/4
//! stream-input-5/6 --------------+-|-|-------------or-|-|----> analog-output-5/6
//! stream-input-7/8 ----------------+-|---------------or-|----> analog-output-7/8
//! stream-input-9/10 -----------------+-----------------or----> digital-output-1/2
//! ```

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

/// The protocol implementation of source of physical input to mixer.
#[derive(Default)]
pub struct Phase88MixerPhysSourceProtocol;

impl AvcLevelOperation for Phase88MixerPhysSourceProtocol {
    const ENTRIES: &'static [(u8, AudioCh)] = &[
        (0x02, AudioCh::Each(0)), // analog-input-1
        (0x02, AudioCh::Each(1)), // analog-input-2
        (0x03, AudioCh::Each(0)), // analog-input-3
        (0x03, AudioCh::Each(1)), // analog-input-4
        (0x04, AudioCh::Each(0)), // analog-input-5
        (0x04, AudioCh::Each(1)), // analog-input-6
        (0x05, AudioCh::Each(0)), // analog-input-7
        (0x05, AudioCh::Each(1)), // analog-input-8
        (0x06, AudioCh::Each(0)), // digital-input-1
        (0x06, AudioCh::Each(1)), // digital-input-2
    ];
}

impl AvcMuteOperation for Phase88MixerPhysSourceProtocol {}

/// The protocol implementation of source of stream input to mixer.
#[derive(Default)]
pub struct Phase88MixerStreamSourceProtocol;

impl AvcLevelOperation for Phase88MixerStreamSourceProtocol {
    const ENTRIES: &'static [(u8, AudioCh)] = &[
        (0x07, AudioCh::Each(0)), // stream-source-1
        (0x07, AudioCh::Each(1)), // stream-source-2
    ];
}

impl AvcMuteOperation for Phase88MixerStreamSourceProtocol {}

/// The protocol implementation of mixer output.
#[derive(Default)]
pub struct Phase88MixerOutputProtocol;

impl AvcLevelOperation for Phase88MixerOutputProtocol {
    const ENTRIES: &'static [(u8, AudioCh)] = &[
        (0x00, AudioCh::Each(0)),
        (0x01, AudioCh::Each(1)),
    ];
}

impl AvcMuteOperation for Phase88MixerOutputProtocol {}
