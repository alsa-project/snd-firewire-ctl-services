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

use super::*;

/// The protocol implementation of clock operation.
#[derive(Default)]
pub struct FireboxClkProtocol;

impl MediaClockFrequencyOperation for FireboxClkProtocol {
    const FREQ_LIST: &'static [u32] = &[44100, 48000, 88200, 96000];
}

impl SamplingClockSourceOperation for FireboxClkProtocol {
    const DST: SignalAddr = SignalAddr::Subunit(SignalSubunitAddr {
        subunit: MUSIC_SUBUNIT_0,
        plug_id: 0x05,
    });

    const SRC_LIST: &'static [SignalAddr] = &[
        // Internal.
        SignalAddr::Subunit(SignalSubunitAddr {
            subunit: MUSIC_SUBUNIT_0,
            plug_id: 0x06,
        }),
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

impl AvcSelectorOperation for FireboxPhysOutputProtocol {
    // NOTE: "analog-output-1/2", "analog-output-3/4", "analog-output-5/6", "digital-output-1/2"
    const FUNC_BLOCK_ID_LIST: &'static [u8] = &[0x01, 0x02, 0x03, 0x05];
    // NOTE: "stream-input", "mixer-output-1/2"
    const INPUT_PLUG_ID_LIST: &'static [u8] = &[0x00, 0x01];
}

/// The protocol implementation of headphone.
pub struct FireboxHeadphoneProtocol;

impl AvcLevelOperation for FireboxHeadphoneProtocol {
    const ENTRIES: &'static [(u8, AudioCh)] = &[(0x04, AudioCh::Each(0)), (0x04, AudioCh::Each(1))];
}

impl AvcMuteOperation for FireboxHeadphoneProtocol {}

impl AvcSelectorOperation for FireboxHeadphoneProtocol {
    // NOTE: "headphone-1/2"
    const FUNC_BLOCK_ID_LIST: &'static [u8] = &[0x04];
    // NOTE: "stream-input-1/2", "stream-input-3/4", "stream-input-5/6", "stream-input-7/8",
    //       "mixer-output-1/2"
    const INPUT_PLUG_ID_LIST: &'static [u8] = &[0x00, 0x01, 0x02, 0x03, 0x04];
}

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
    const ENTRIES: &'static [(u8, AudioCh)] = &[(0x08, AudioCh::Each(0))];
}

impl AvcMuteOperation for FireboxMixerStreamSourceProtocol {}

impl AvcSelectorOperation for FireboxMixerStreamSourceProtocol {
    // NOTE: "stream-source-1/2"
    const FUNC_BLOCK_ID_LIST: &'static [u8] = &[0x06];
    // NOTE: "stream-input-1/2", "stream-input-3/4", "stream-input-5/6", "stream-input-7/8",
    const INPUT_PLUG_ID_LIST: &'static [u8] = &[0x00, 0x01, 0x02, 0x03];
}
/// The protocol implementation of mixer output.
pub struct FireboxMixerOutputProtocol;

impl AvcLevelOperation for FireboxMixerOutputProtocol {
    const ENTRIES: &'static [(u8, AudioCh)] = &[(0x09, AudioCh::Each(0)), (0x09, AudioCh::Each(1))];
}

impl AvcLrBalanceOperation for FireboxMixerOutputProtocol {}

impl AvcMuteOperation for FireboxMixerOutputProtocol {}

/// The protocol implementation of analog input.
pub struct FireboxAnalogInputProtocol;

const BOOST_OFF: i16 = 0x7ffe;
const BOOST_ON: i16 = 0x0000;

impl AvcSelectorOperation for FireboxAnalogInputProtocol {
    // NOTE: "analog-input-1", "analog-input-2", "analog-input-3", "analog-input-4"
    const FUNC_BLOCK_ID_LIST: &'static [u8] = &[0x0a, 0x0a, 0x0b, 0x0b];
    // NOTE: off(=0x7ffe)/on(=0x0000) for volume control of audio function block.
    const INPUT_PLUG_ID_LIST: &'static [u8] = &[0x00, 0x01];

    fn read_selector(avc: &BebobAvc, idx: usize, timeout_ms: u32) -> Result<usize, Error> {
        let func_block_id = Self::FUNC_BLOCK_ID_LIST
            .iter()
            .nth(idx)
            .ok_or_else(|| {
                let msg = format!("Invalid argument for index of selector: {}", idx);
                Error::new(FileError::Inval, &msg)
            })
            .map(|func_block_id| *func_block_id)?;
        let ch_id = idx % 2;

        let mut op = AudioFeature::new(
            func_block_id,
            CtlAttr::Current,
            AudioCh::Each(ch_id as u8),
            FeatureCtl::Volume(vec![-1]),
        );
        avc.status(&AUDIO_SUBUNIT_0_ADDR, &mut op, timeout_ms)?;

        if let FeatureCtl::Volume(data) = op.ctl {
            let val = if data[0] == BOOST_OFF { 0 } else { 1 };
            Ok(val)
        } else {
            unreachable!();
        }
    }

    fn write_selector(
        avc: &BebobAvc,
        idx: usize,
        val: usize,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let func_block_id = Self::FUNC_BLOCK_ID_LIST
            .iter()
            .nth(idx)
            .ok_or_else(|| {
                let msg = format!("Invalid argument for index of selector: {}", idx);
                Error::new(FileError::Inval, &msg)
            })
            .map(|func_block_id| *func_block_id)?;
        let ch_id = idx & 2;

        let mut op = AudioFeature::new(
            func_block_id,
            CtlAttr::Current,
            AudioCh::Each(ch_id as u8),
            FeatureCtl::Volume(vec![if val == 0 { BOOST_OFF } else { BOOST_ON }]),
        );
        avc.control(&AUDIO_SUBUNIT_0_ADDR, &mut op, timeout_ms)
    }
}
