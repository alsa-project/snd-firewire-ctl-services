// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol defined by Griffin for FireWave.
//!
//! The module includes protocol implementation defined by Griffin Technologies for FireWave.

use super::*;

/// The protocol implementation for FireWave.
#[derive(Default, Debug)]
pub struct FirewaveProtocol;

impl OxfwAudioFbSpecification for FirewaveProtocol {
    const VOLUME_FB_ID: u8 = 0x02;
    const MUTE_FB_ID: u8 = 0x01;
    const CHANNEL_MAP: &'static [usize] = &[0, 1, 4, 5, 2, 3];
}

impl FirewaveProtocol {
    pub const VOLUME_MIN: i16 = VolumeData::VALUE_NEG_INFINITY;
    pub const VOLUME_MAX: i16 = VolumeData::VALUE_ZERO;
    pub const VOLUME_STEP: i16 = 1;

    pub const PLAYBACK_COUNT: usize = Self::CHANNEL_MAP.len();

    const VOL_FB_ID: u8 = 0x02;
    const MUTE_FB_ID: u8 = 0x01;

    pub fn read_volume(
        avc: &mut OxfwAvc,
        idx: usize,
        volume: &mut i16,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        if idx >= Self::CHANNEL_MAP.len() {
            let msg = format!("Invalid index for audio channel: {}", idx);
            Err(Error::new(FileError::Inval, &msg))
        } else {
            let mut op = AudioFeature::new(
                Self::VOL_FB_ID,
                CtlAttr::Current,
                AudioCh::Each(Self::CHANNEL_MAP[idx] as u8),
                FeatureCtl::Volume(VolumeData::new(1)),
            );
            avc.status(&AUDIO_SUBUNIT_0_ADDR, &mut op, timeout_ms)
                .map(|_| {
                    if let FeatureCtl::Volume(data) = op.ctl {
                        *volume = data.0[0]
                    }
                })
        }
    }

    pub fn write_volume(
        avc: &mut OxfwAvc,
        idx: usize,
        volume: i16,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        if idx >= Self::CHANNEL_MAP.len() {
            let msg = format!("Invalid index for audio channel: {}", idx);
            Err(Error::new(FileError::Inval, &msg))
        } else {
            let mut op = AudioFeature::new(
                Self::VOL_FB_ID,
                CtlAttr::Current,
                AudioCh::Each(Self::CHANNEL_MAP[idx] as u8),
                FeatureCtl::Volume(VolumeData(vec![volume])),
            );
            avc.control(&AUDIO_SUBUNIT_0_ADDR, &mut op, timeout_ms)
        }
    }

    pub fn read_mute(avc: &mut OxfwAvc, mute: &mut bool, timeout_ms: u32) -> Result<(), Error> {
        let mut op = AudioFeature::new(
            Self::MUTE_FB_ID,
            CtlAttr::Current,
            AudioCh::Master,
            FeatureCtl::Mute(vec![false]),
        );
        avc.status(&AUDIO_SUBUNIT_0_ADDR, &mut op, timeout_ms)
            .map(|_| {
                if let FeatureCtl::Mute(val) = op.ctl {
                    *mute = val[0]
                }
            })
    }

    pub fn write_mute(avc: &mut OxfwAvc, mute: bool, timeout_ms: u32) -> Result<(), Error> {
        let mut op = AudioFeature::new(
            Self::MUTE_FB_ID,
            CtlAttr::Current,
            AudioCh::Master,
            FeatureCtl::Mute(vec![mute]),
        );
        avc.control(&AUDIO_SUBUNIT_0_ADDR, &mut op, timeout_ms)
    }
}
