// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol specific to Lexicon I-ONIX FW810s.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by Lexicon for I-ONIX FW810s.

use glib::Error;
use hinawa::{FwNode, FwReq};

use crate::{
    tcat::{extension::*, *},
    *,
};

const BASE_OFFSET: usize = 0x00200000;

const MIXER_BUS_SRC_OFFSET: usize = 0x0000;
const MIXER_MAIN_SRC_OFFSET: usize = 0x02d0;
const MIXER_REVERB_SRC_OFFSET: usize = 0x0360;
const METER_OFFSET: usize = 0x0500;
const EFFECT_OFFSET: usize = 0x4000;

/// The structure for protocol implementation specific to Lexicon I-ONIX FW810s.
pub struct IonixProtocol;

fn lexicon_read(
    req: &mut FwReq,
    node: &mut FwNode,
    offset: usize,
    frame: &mut [u8],
    timeout_ms: u32,
) -> Result<(), Error> {
    GeneralProtocol::read(req, node, BASE_OFFSET + offset, frame, timeout_ms)
}

fn lexicon_write(
    req: &mut FwReq,
    node: &mut FwNode,
    offset: usize,
    frame: &mut [u8],
    timeout_ms: u32,
) -> Result<(), Error> {
    GeneralProtocol::write(req, node, BASE_OFFSET + offset, frame, timeout_ms)
}

const DATA_PREFIX: [u8; 5] = [0x06, 0x00, 0x1b, 0x01, 0x41];

#[allow(dead_code)]
const SYSEX_MSG_PREFIX: u8 = 0xf0;
#[allow(dead_code)]
const SYSEX_MSG_SUFFIX: u8 = 0xf7;

impl IonixProtocol {
    // NOTE: states of all effect are available with structured data by read block request with 512 bytes.
    pub fn write_effect_data(
        req: &mut FwReq,
        node: &mut FwNode,
        data: &[u8],
        timeout_ms: u32,
    ) -> Result<(), Error> {
        // NOTE: The data has prefix.
        let mut msgs = DATA_PREFIX.to_vec();
        msgs.extend_from_slice(&data);

        // NOTE: Append checksum calculated by XOR for all the data.
        let checksum = msgs.iter().fold(0u8, |val, &msg| val | msg);
        msgs.push(checksum);

        // NOTE: Construct MIDI system exclusive message.
        msgs.insert(0, 0xf0);
        msgs.push(0xf7);

        // NOTE: One quadlet deliver one byte of message.
        let mut raw = Vec::<u8>::new();
        msgs.iter()
            .for_each(|&msg| raw.extend_from_slice(&(msg as u32).to_be_bytes()));

        lexicon_write(req, node, EFFECT_OFFSET, &mut raw, timeout_ms)
    }
}

#[derive(Default, Debug)]
/// The structure to represent hardware meter.
pub struct IonixMeter {
    pub analog_inputs: [i32; IonixProtocol::ANALOG_INPUT_COUNT],
    pub spdif_inputs: [i32; IonixProtocol::SPDIF_INPUT_COUNT],
    pub stream_inputs: [i32; IonixProtocol::STREAM_INPUT_COUNT],
    pub bus_outputs: [i32; IonixProtocol::MIXER_BUS_COUNT],
    pub main_outputs: [i32; IonixProtocol::MIXER_MAIN_COUNT],
}

/// The structure to represent entry of hardware meter.
#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
struct IonixMeterEntry {
    /// The level of audio signal from the source to the destination.
    level: i16,
    /// The source of audio signal.
    src: SrcBlk,
    /// The destination of audio signal.
    dst: DstBlk,
}

impl From<u32> for IonixMeterEntry {
    fn from(val: u32) -> Self {
        IonixMeterEntry {
            level: ((val & 0xffff0000) >> 16) as i16,
            src: SrcBlk::from(((val & 0x0000ff00) >> 8) as u8),
            dst: DstBlk::from(((val & 0x000000ff) >> 0) as u8),
        }
    }
}

impl From<IonixMeterEntry> for u32 {
    fn from(entry: IonixMeterEntry) -> Self {
        ((entry.level as u32) << 16)
            | ((u8::from(entry.src) as u32) << 8)
            | (u8::from(entry.dst) as u32)
    }
}

impl IonixProtocol {
    // NOTE: 90 entries are valid at all of supported sampling rate.
    const ENTRY_COUNT: usize = 90;

    pub const ANALOG_INPUT_COUNT: usize = 8;
    pub const SPDIF_INPUT_COUNT: usize = 2;
    pub const STREAM_INPUT_COUNT: usize = 10;

    pub fn read_meters(
        req: &mut FwReq,
        node: &mut FwNode,
        meters: &mut IonixMeter,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut raw = vec![0; Self::ENTRY_COUNT * 4];
        lexicon_read(req, node, METER_OFFSET, &mut raw, timeout_ms)?;
        let mut entries = vec![IonixMeterEntry::default(); Self::ENTRY_COUNT];
        entries.parse_quadlet_block(&raw);

        entries
            .iter()
            .filter(|entry| entry.src.id == SrcBlkId::Avs0 && entry.dst.id == DstBlkId::MixerTx0)
            .take(meters.stream_inputs.len())
            .enumerate()
            .for_each(|(i, entry)| meters.stream_inputs[i] = entry.level as i32);

        entries
            .iter()
            .filter(|entry| entry.src.id == SrcBlkId::Ins0 && entry.dst.id == DstBlkId::MixerTx0)
            .take(meters.analog_inputs.len())
            .enumerate()
            .for_each(|(i, entry)| meters.analog_inputs[i] = entry.level as i32);

        entries
            .iter()
            .filter(|entry| entry.src.id == SrcBlkId::Aes && entry.dst.id == DstBlkId::MixerTx0)
            .take(meters.spdif_inputs.len())
            .enumerate()
            .for_each(|(i, entry)| meters.spdif_inputs[i] = entry.level as i32);

        entries
            .iter()
            .filter(|entry| entry.src.id == SrcBlkId::Mixer && entry.dst.id == DstBlkId::Ins0)
            .take(meters.bus_outputs.len())
            .enumerate()
            .for_each(|(i, entry)| meters.bus_outputs[i] = entry.level as i32);

        entries
            .iter()
            .filter(|entry| {
                entry.src.id == SrcBlkId::Mixer
                    && entry.dst.id == DstBlkId::Ins1
                    && entry.dst.ch < 2
            })
            .take(meters.main_outputs.len())
            .enumerate()
            .for_each(|(i, entry)| meters.main_outputs[i] = entry.level as i32);

        Ok(())
    }
}

/// The enumeration to represent source of mixer.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum MixerSrc {
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

impl IonixProtocol {
    /// The number of channels in bus mixer.
    pub const MIXER_BUS_COUNT: usize = 8;

    /// The number of channels in main mixer.
    pub const MIXER_MAIN_COUNT: usize = 2;

    /// The number of channels in reverb mixer.
    pub const MIXER_REVERB_COUNT: usize = 2;

    pub fn read_mixer_bus_src_gain(
        req: &mut FwReq,
        node: &mut FwNode,
        dst: usize,
        src: MixerSrc,
        timeout_ms: u32,
    ) -> Result<u32, Error> {
        assert!(dst < Self::MIXER_BUS_COUNT);

        let mut raw = [0; 4];
        let offset = MIXER_SRC_SIZE * dst + usize::from(src);
        lexicon_read(
            req,
            node,
            MIXER_BUS_SRC_OFFSET + offset,
            &mut raw,
            timeout_ms,
        )
        .map(|_| u32::from_be_bytes(raw))
    }

    pub fn write_mixer_bus_src_gain(
        req: &mut FwReq,
        node: &mut FwNode,
        dst: usize,
        src: MixerSrc,
        gain: u32,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        assert!(dst < Self::MIXER_BUS_COUNT);

        let mut raw = [0; 4];
        raw.copy_from_slice(&gain.to_be_bytes());
        let offset = MIXER_SRC_SIZE * dst + usize::from(src);
        lexicon_write(
            req,
            node,
            MIXER_BUS_SRC_OFFSET + offset,
            &mut raw,
            timeout_ms,
        )
    }

    pub fn read_mixer_main_src_gain(
        req: &mut FwReq,
        node: &mut FwNode,
        dst: usize,
        src: MixerSrc,
        timeout_ms: u32,
    ) -> Result<u32, Error> {
        assert!(dst < Self::MIXER_MAIN_COUNT);

        let mut raw = [0; 4];
        let offset = MIXER_SRC_SIZE * dst + usize::from(src);
        lexicon_read(
            req,
            node,
            MIXER_MAIN_SRC_OFFSET + offset,
            &mut raw,
            timeout_ms,
        )
        .map(|_| u32::from_be_bytes(raw))
    }

    pub fn write_mixer_main_src_gain(
        req: &mut FwReq,
        node: &mut FwNode,
        dst: usize,
        src: MixerSrc,
        gain: u32,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        assert!(dst < Self::MIXER_MAIN_COUNT);

        let mut raw = [0; 4];
        raw.copy_from_slice(&gain.to_be_bytes());
        let offset = MIXER_SRC_SIZE * dst + usize::from(src);
        lexicon_write(
            req,
            node,
            MIXER_MAIN_SRC_OFFSET + offset,
            &mut raw,
            timeout_ms,
        )
    }

    pub fn read_mixer_reverb_src_gain(
        req: &mut FwReq,
        node: &mut FwNode,
        dst: usize,
        src: MixerSrc,
        timeout_ms: u32,
    ) -> Result<u32, Error> {
        assert!(dst < Self::MIXER_REVERB_COUNT);

        let mut raw = [0; 4];
        let offset = MIXER_SRC_SIZE * dst + usize::from(src);
        lexicon_read(
            req,
            node,
            MIXER_REVERB_SRC_OFFSET + offset,
            &mut raw,
            timeout_ms,
        )
        .map(|_| u32::from_be_bytes(raw))
    }

    pub fn write_mixer_reverb_src_gain(
        req: &mut FwReq,
        node: &mut FwNode,
        dst: usize,
        src: MixerSrc,
        gain: u32,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        assert!(dst < Self::MIXER_REVERB_COUNT);

        let mut raw = [0; 4];
        raw.copy_from_slice(&gain.to_be_bytes());
        let offset = MIXER_SRC_SIZE * dst + usize::from(src);
        lexicon_write(
            req,
            node,
            MIXER_REVERB_SRC_OFFSET + offset,
            &mut raw,
            timeout_ms,
        )
    }
}
