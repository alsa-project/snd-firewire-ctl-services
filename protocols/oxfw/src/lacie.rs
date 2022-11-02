// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol defined by La Cie for FireWire Speakers.
//!
//! The module includes protocol implementation defined by La Cie for FireWire Speackers.

use super::*;

/// The protocol implementation for FireWire Speackers.
#[derive(Default, Debug)]
pub struct FwSpeakersProtocol;

impl OxfwAudioFbSpecification for FwSpeakersProtocol {
    const VOLUME_FB_ID: u8 = 0x01;
    const MUTE_FB_ID: u8 = 0x01;
    const CHANNEL_MAP: &'static [usize] = &[0];
}
