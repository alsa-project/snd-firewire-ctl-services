// SPDX-License-Identifier: LGPL-3.0-or-later
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
#[derive(Default, Debug)]
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
#[derive(Default, Debug)]
pub struct FireboxPhysOutputProtocol;

impl AvcAudioFeatureSpecification for FireboxPhysOutputProtocol {
    const ENTRIES: &'static [(u8, AudioCh)] = &[
        (0x01, AudioCh::Each(0)),
        (0x01, AudioCh::Each(1)),
        (0x02, AudioCh::Each(0)),
        (0x02, AudioCh::Each(1)),
        (0x03, AudioCh::Each(0)),
        (0x03, AudioCh::Each(1)),
    ];
}

impl AvcLevelOperation for FireboxPhysOutputProtocol {}

impl AvcMuteOperation for FireboxPhysOutputProtocol {}

impl AvcSelectorOperation for FireboxPhysOutputProtocol {
    // NOTE: "analog-output-1/2", "analog-output-3/4", "analog-output-5/6", "digital-output-1/2"
    const FUNC_BLOCK_ID_LIST: &'static [u8] = &[0x01, 0x02, 0x03, 0x05];
    // NOTE: "stream-input", "mixer-output-1/2"
    const INPUT_PLUG_ID_LIST: &'static [u8] = &[0x00, 0x01];
}

/// The protocol implementation of headphone.
#[derive(Default, Debug)]
pub struct FireboxHeadphoneProtocol;

impl AvcAudioFeatureSpecification for FireboxHeadphoneProtocol {
    const ENTRIES: &'static [(u8, AudioCh)] = &[(0x04, AudioCh::Each(0)), (0x04, AudioCh::Each(1))];
}

impl AvcLevelOperation for FireboxHeadphoneProtocol {}

impl AvcMuteOperation for FireboxHeadphoneProtocol {}

impl AvcSelectorOperation for FireboxHeadphoneProtocol {
    // NOTE: "headphone-1/2"
    const FUNC_BLOCK_ID_LIST: &'static [u8] = &[0x04];
    // NOTE: "stream-input-1/2", "stream-input-3/4", "stream-input-5/6", "stream-input-7/8",
    //       "mixer-output-1/2"
    const INPUT_PLUG_ID_LIST: &'static [u8] = &[0x00, 0x01, 0x02, 0x03, 0x04];
}

/// The protocol implementation of physical source for mixer.
#[derive(Default, Debug)]
pub struct FireboxMixerPhysSourceProtocol;

impl AvcAudioFeatureSpecification for FireboxMixerPhysSourceProtocol {
    const ENTRIES: &'static [(u8, AudioCh)] = &[
        (0x05, AudioCh::Each(0)),
        (0x05, AudioCh::Each(1)),
        (0x06, AudioCh::Each(0)),
        (0x06, AudioCh::Each(1)),
        (0x07, AudioCh::Each(0)),
        (0x07, AudioCh::Each(1)),
    ];
}

impl AvcLevelOperation for FireboxMixerPhysSourceProtocol {}

impl AvcLrBalanceOperation for FireboxMixerPhysSourceProtocol {}

impl AvcMuteOperation for FireboxMixerPhysSourceProtocol {}

/// The protocol implementation of stream source for mixer.
#[derive(Default, Debug)]
pub struct FireboxMixerStreamSourceProtocol;

impl AvcAudioFeatureSpecification for FireboxMixerStreamSourceProtocol {
    const ENTRIES: &'static [(u8, AudioCh)] = &[(0x08, AudioCh::Each(0))];
}

impl AvcLevelOperation for FireboxMixerStreamSourceProtocol {}

impl AvcMuteOperation for FireboxMixerStreamSourceProtocol {}

impl AvcSelectorOperation for FireboxMixerStreamSourceProtocol {
    // NOTE: "stream-source-1/2"
    const FUNC_BLOCK_ID_LIST: &'static [u8] = &[0x06];
    // NOTE: "stream-input-1/2", "stream-input-3/4", "stream-input-5/6", "stream-input-7/8",
    const INPUT_PLUG_ID_LIST: &'static [u8] = &[0x00, 0x01, 0x02, 0x03];
}
/// The protocol implementation of mixer output.
#[derive(Default, Debug)]
pub struct FireboxMixerOutputProtocol;

impl AvcAudioFeatureSpecification for FireboxMixerOutputProtocol {
    const ENTRIES: &'static [(u8, AudioCh)] = &[(0x09, AudioCh::Each(0)), (0x09, AudioCh::Each(1))];
}

impl AvcLevelOperation for FireboxMixerOutputProtocol {}

impl AvcLrBalanceOperation for FireboxMixerOutputProtocol {}

impl AvcMuteOperation for FireboxMixerOutputProtocol {}

/// The parameters of analog inputs.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct FireboxAnalogInputParameters {
    /// Boost signal for analog input 1-4.
    pub boosts: [bool; FireboxAnalogInputProtocol::CH_COUNT],
}

/// The protocol implementation of analog inputs.
#[derive(Default, Debug)]
pub struct FireboxAnalogInputProtocol;

impl FireboxAnalogInputProtocol {
    const BOOST_OFF: i16 = VolumeData::VALUE_MIN;
    const BOOST_ON: i16 = VolumeData::VALUE_ZERO;

    const FUNC_BLOCK_ID_LIST: &'static [u8] = &[0x0a, 0x0b];
    const INPUT_PLUG_ID_LIST: &'static [u8] = &[0x00, 0x01];

    const CH_COUNT: usize = 4;

    /// Cache state of hardware to the parameters.
    pub fn cache(
        avc: &BebobAvc,
        params: &mut FireboxAnalogInputParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        params
            .boosts
            .iter_mut()
            .enumerate()
            .try_for_each(|(i, boost)| {
                let func_block_id = Self::FUNC_BLOCK_ID_LIST[i / 2];
                let input_plug_id = Self::INPUT_PLUG_ID_LIST[i % 2];

                let mut op = AudioFeature::new(
                    func_block_id,
                    CtlAttr::Current,
                    AudioCh::Each(input_plug_id),
                    FeatureCtl::Volume(VolumeData::new(1)),
                );
                avc.status(&AUDIO_SUBUNIT_0_ADDR, &mut op, timeout_ms)
                    .map(|_| {
                        if let FeatureCtl::Volume(data) = op.ctl {
                            *boost = data.0[0] == Self::BOOST_ON;
                        }
                    })
            })
    }

    /// Update hardware when detecting changes in the parameters.
    pub fn update(
        avc: &BebobAvc,
        params: &FireboxAnalogInputParameters,
        prev: &mut FireboxAnalogInputParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        prev.boosts
            .iter_mut()
            .zip(params.boosts.iter())
            .enumerate()
            .filter(|(_, (o, n))| !o.eq(n))
            .try_for_each(|(i, (old, &new))| {
                let func_block_id = Self::FUNC_BLOCK_ID_LIST[i / 2];
                let input_plug_id = Self::INPUT_PLUG_ID_LIST[i % 2];

                let mut op = AudioFeature::new(
                    func_block_id,
                    CtlAttr::Current,
                    AudioCh::Each(input_plug_id),
                    FeatureCtl::Volume(VolumeData(vec![if new {
                        Self::BOOST_ON
                    } else {
                        Self::BOOST_OFF
                    }])),
                );
                avc.control(&AUDIO_SUBUNIT_0_ADDR, &mut op, timeout_ms)
                    .map(|_| *old = new)
            })
    }
}
