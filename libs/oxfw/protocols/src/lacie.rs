// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol defined by La Cie for FireWire Speakers.
//!
//! The module includes protocol implementation defined by La Cie for FireWire Speackers.

use super::*;

/// The protocol implementation for FireWire Speackers.
#[derive(Default, Debug)]
pub struct FwSpeakersProtocol;

impl FwSpeakersProtocol {
    pub const VOLUME_MIN: i16 = i16::MIN;
    pub const VOLUME_MAX: i16 = 0;
    pub const VOLUME_STEP: i16 = 1;

    const FB_ID: u8 = 0x01;

    pub fn read_volume(avc: &mut FwFcp, volume: &mut i16, timeout_ms: u32) -> Result<(), Error> {
        let mut op = AudioFeature::new(
            Self::FB_ID,
            CtlAttr::Current,
            AudioCh::All,
            FeatureCtl::Volume(vec![-1]),
        );
        avc.status(&AUDIO_SUBUNIT_0_ADDR, &mut op, timeout_ms)
            .map(|_| {
                if let FeatureCtl::Volume(data) = op.ctl {
                    *volume = data[0]
                }
            })
    }

    pub fn write_volume(avc: &mut FwFcp, volume: i16, timeout_ms: u32) -> Result<(), Error> {
        let mut op = AudioFeature::new(
            Self::FB_ID,
            CtlAttr::Current,
            AudioCh::All,
            FeatureCtl::Volume(vec![volume]),
        );
        avc.control(&AUDIO_SUBUNIT_0_ADDR, &mut op, timeout_ms)
    }

    pub fn read_mute(avc: &mut FwFcp, mute: &mut bool, timeout_ms: u32) -> Result<(), Error> {
        let mut op = AudioFeature::new(
            Self::FB_ID,
            CtlAttr::Current,
            AudioCh::All,
            FeatureCtl::Mute(vec![false]),
        );
        avc.status(&AUDIO_SUBUNIT_0_ADDR, &mut op, timeout_ms)
            .map(|_| {
                if let FeatureCtl::Mute(data) = op.ctl {
                    *mute = data[0]
                }
            })
    }

    pub fn write_mute(avc: &mut FwFcp, mute: bool, timeout_ms: u32) -> Result<(), Error> {
        let mut op = AudioFeature::new(
            Self::FB_ID,
            CtlAttr::Current,
            AudioCh::All,
            FeatureCtl::Mute(vec![mute]),
        );
        avc.control(&AUDIO_SUBUNIT_0_ADDR, &mut op, timeout_ms)
    }
}
