// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Mixer protocol specific to Lexicon I-ONIX FW810s.
//!
//! The module includes structure, enumeration, and trait and its implementation for mixer protocol
//! defined by Lexicon for I-ONIX FW810s.

use super::*;

/// The number of channels in bus mixer.
pub const MIXER_BUS_CHANNEL_COUNT: usize = 8;

/// The number of channels in main mixer.
pub const MIXER_MAIN_CHANNEL_COUNT: usize = 2;

/// The number of channels in reverb mixer.
pub const MIXER_REVERB_CHANNEL_COUNT: usize = 2;

/// The enumeration to represent source of mixer.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum MixerSrc{
    Stream(usize),
    Spdif(usize),
    Analog(usize),
}

// Followed by gains for 8 channels of stream input.
const STREAM_SRC_OFFSET: usize = 0x00;

// Followed by gains for 2 channels of spdif input.
const SPDIF_SRC_OFFSET: usize = 0x20;

// Followed by gains for 8 channels of analog input.
const ANALOG_SRC_OFFSET: usize = 0x28;

// Including gains for the above channels.
const MIXER_SRC_SIZE: usize = STREAM_SRC_OFFSET + SPDIF_SRC_OFFSET + ANALOG_SRC_OFFSET;

impl From<MixerSrc> for usize {
    fn from(src: MixerSrc) -> Self {
        match src {
            MixerSrc::Stream(ch) => STREAM_SRC_OFFSET + 4 * ch,
            MixerSrc::Spdif(ch) => SPDIF_SRC_OFFSET + 4 * ch,
            MixerSrc::Analog(ch) => ANALOG_SRC_OFFSET + 4 * ch,
        }
    }
}

pub trait IonixMixerProtocol: IonixProtocol {
    const MIXER_BUS_SRC_OFFSET: usize = 0x0000;
    const MIXER_MAIN_SRC_OFFSET: usize = 0x02d0;
    const MIXER_REVERB_SRC_OFFSET: usize = 0x0360;

    fn read_mixer_bus_src_gain(
        &self,
        node: &mut FwNode,
        dst: usize,
        src: MixerSrc,
        timeout_ms: u32
    ) -> Result<u32, Error> {
        assert!(dst < MIXER_BUS_CHANNEL_COUNT);

        let mut raw = [0;4];
        let offset = MIXER_SRC_SIZE * dst + usize::from(src);
        IonixProtocol::read(self, node, Self::MIXER_BUS_SRC_OFFSET + offset, &mut raw, timeout_ms)
            .map(|_| u32::from_be_bytes(raw))
    }

    fn write_mixer_bus_src_gain(
        &self,
        node: &mut FwNode,
        dst: usize,
        src: MixerSrc,
        gain: u32,
        timeout_ms: u32
    ) -> Result<(), Error> {
        assert!(dst < MIXER_BUS_CHANNEL_COUNT);

        let mut raw = [0;4];
        raw.copy_from_slice(&gain.to_be_bytes());
        let offset = MIXER_SRC_SIZE * dst + usize::from(src);
        IonixProtocol::write(self, node, Self::MIXER_BUS_SRC_OFFSET + offset, &mut raw, timeout_ms)
    }

    fn read_mixer_main_src_gain(
        &self,
        node: &mut FwNode,
        dst: usize,
        src: MixerSrc,
        timeout_ms: u32
    ) -> Result<u32, Error> {
        assert!(dst < MIXER_MAIN_CHANNEL_COUNT);

        let mut raw = [0;4];
        let offset = MIXER_SRC_SIZE * dst + usize::from(src);
        IonixProtocol::read(self, node, Self::MIXER_MAIN_SRC_OFFSET + offset, &mut raw, timeout_ms)
            .map(|_| u32::from_be_bytes(raw))
    }

    fn write_mixer_main_src_gain(
        &self,
        node: &mut FwNode,
        dst: usize,
        src: MixerSrc,
        gain: u32,
        timeout_ms: u32
    ) -> Result<(), Error> {
        assert!(dst < MIXER_MAIN_CHANNEL_COUNT);

        let mut raw = [0;4];
        raw.copy_from_slice(&gain.to_be_bytes());
        let offset = MIXER_SRC_SIZE * dst + usize::from(src);
        IonixProtocol::write(self, node, Self::MIXER_MAIN_SRC_OFFSET + offset, &mut raw, timeout_ms)
    }

    fn read_mixer_reverb_src_gain(
        &self,
        node: &mut FwNode,
        dst: usize,
        src: MixerSrc,
        timeout_ms: u32
    ) -> Result<u32, Error> {
        assert!(dst < MIXER_REVERB_CHANNEL_COUNT);

        let mut raw = [0;4];
        let offset = MIXER_SRC_SIZE * dst + usize::from(src);
        IonixProtocol::read(self, node, Self::MIXER_REVERB_SRC_OFFSET + offset, &mut raw, timeout_ms)
            .map(|_| u32::from_be_bytes(raw))
    }

    fn write_mixer_reverb_src_gain(
        &self,
        node: &mut FwNode,
        dst: usize,
        src: MixerSrc,
        gain: u32,
        timeout_ms: u32
    ) -> Result<(), Error> {
        assert!(dst < MIXER_REVERB_CHANNEL_COUNT);

        let mut raw = [0;4];
        raw.copy_from_slice(&gain.to_be_bytes());
        let offset = MIXER_SRC_SIZE * dst + usize::from(src);
        IonixProtocol::write(self, node, Self::MIXER_REVERB_SRC_OFFSET + offset, &mut raw, timeout_ms)
    }
}

impl<O> IonixMixerProtocol for O
    where O: IonixProtocol,
{}
