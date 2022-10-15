// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol specific to Lexicon I-ONIX FW810s.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by Lexicon for I-ONIX FW810s.

use {
    super::{
        tcat::{extension::*, *},
        *,
    },
    std::ops::Range,
};

const BASE_OFFSET: usize = 0x00200000;

const MIXER_BUS_SRC_OFFSET: usize = 0x0000;
const MIXER_MAIN_SRC_OFFSET: usize = 0x02d0;
const MIXER_REVERB_SRC_OFFSET: usize = 0x0360;
const METER_OFFSET: usize = 0x0500;
const EFFECT_OFFSET: usize = 0x4000;

const METER_SIZE: usize = 0x200;

/// Protocol implementation specific to Lexicon I-ONIX FW810s.
#[derive(Default, Debug)]
pub struct IonixProtocol;

impl TcatOperation for IonixProtocol {}

impl TcatGlobalSectionSpecification for IonixProtocol {}

impl LexiconOperation for IonixProtocol {}

/// Serialize and deserialize parameters.
pub trait LexiconParametersSerdes<T> {
    /// Name of parameters.
    const NAME: &'static str;

    /// List of offset ranges for parameters.
    const OFFSET_RANGES: &'static [Range<usize>];

    /// Serialize parameters.
    fn serialize_params(params: &T, raw: &mut [u8]) -> Result<(), String>;

    /// Deserialize parameters.
    fn deserialize_params(params: &mut T, raw: &[u8]) -> Result<(), String>;
}

/// Common operation for Lexicon I-ONIX F810s.
pub trait LexiconOperation: TcatOperation {
    /// Read parameters from specific address range.
    fn read_parameters(
        req: &FwReq,
        node: &FwNode,
        offset: usize,
        raw: &mut [u8],
        timeout_ms: u32,
    ) -> Result<(), Error> {
        Self::read(req, node, BASE_OFFSET + offset, raw, timeout_ms)
    }

    /// Write parameters to specific address range.
    fn write_parameters(
        req: &FwReq,
        node: &FwNode,
        offset: usize,
        raw: &mut [u8],
        timeout_ms: u32,
    ) -> Result<(), Error> {
        Self::write(req, node, BASE_OFFSET + offset, raw, timeout_ms)
    }
}

fn compute_params_size(ranges: &[Range<usize>]) -> usize {
    ranges
        .iter()
        .fold(0usize, |size, range| size + range.end - range.start)
}

fn generate_err(name: &str, cause: &str, raw: &[u8]) -> Error {
    let msg = format!("parms: {}, cause: {}, raw: {:02x?}", name, cause, raw);
    Error::new(GeneralProtocolError::VendorDependent, &msg)
}

/// Operation to cache parameters.
pub trait LexiconParametersOperation<T>: LexiconOperation + LexiconParametersSerdes<T> {
    /// Cache parameters.
    fn cache_whole_params(
        req: &FwReq,
        node: &FwNode,
        params: &mut T,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let size = compute_params_size(Self::OFFSET_RANGES);
        let mut raw = vec![0u8; size];

        let mut pos = 0;

        Self::OFFSET_RANGES.iter().try_for_each(|range| {
            let size = range.end - range.start;
            Self::read_parameters(
                req,
                node,
                range.start,
                &mut raw[pos..(pos + size)],
                timeout_ms,
            )
            .map(|_| pos += size)
        })?;

        Self::deserialize_params(params, &raw)
            .map_err(|cause| generate_err(Self::NAME, &cause, &raw))
    }
}

impl<O: LexiconOperation + LexiconParametersSerdes<T>, T> LexiconParametersOperation<T> for O {}

/// Operation for parameters to update state of hardware.
pub trait LexiconMutableParametersOperation<T>:
    LexiconOperation + LexiconParametersSerdes<T>
{
    /// Update the hardware partially for any change of parameter.
    fn update_partial_parameters(
        req: &FwReq,
        node: &FwNode,
        params: &T,
        prev: &mut T,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let size = compute_params_size(Self::OFFSET_RANGES);

        let mut new = vec![0u8; size];
        let mut old = vec![0u8; size];
        Self::serialize_params(params, &mut new)
            .map_err(|cause| generate_err(Self::NAME, &cause, &new))?;
        Self::serialize_params(prev, &mut old)
            .map_err(|cause| generate_err(Self::NAME, &cause, &old))?;

        let mut pos = 0;

        Self::OFFSET_RANGES.iter().try_for_each(|range| {
            let size = range.end - range.start;

            if new[pos..(pos + size)] != old[pos..(pos + size)] {
                (0..size).step_by(4).try_for_each(|offset| {
                    let p = pos + offset;
                    if new[p..(p + 4)] != old[p..(p + 4)] {
                        Self::write_parameters(
                            req,
                            node,
                            range.start + offset,
                            &mut new[p..(p + 4)],
                            timeout_ms,
                        )
                    } else {
                        Ok(())
                    }
                })
            } else {
                Ok(())
            }
            .map(|_| pos += size)
        })?;

        Self::deserialize_params(prev, &new).map_err(|cause| generate_err(Self::NAME, &cause, &new))
    }
}

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

/// Hardware meter.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct IonixMeter {
    pub analog_inputs: [i32; IonixProtocol::ANALOG_INPUT_COUNT],
    pub spdif_inputs: [i32; IonixProtocol::SPDIF_INPUT_COUNT],
    pub stream_inputs: [i32; IonixProtocol::STREAM_INPUT_COUNT],
    pub bus_outputs: [i32; IonixProtocol::MIXER_BUS_COUNT],
    pub main_outputs: [i32; IonixProtocol::MIXER_MAIN_COUNT],
}

/// Entry of hardware meter.
#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
struct IonixMeterEntry {
    /// The level of audio signal from the source to the destination.
    level: i16,
    /// The source of audio signal.
    src: SrcBlk,
    /// The destination of audio signal.
    dst: DstBlk,
}

fn serialize_meter_entry(entry: &IonixMeterEntry, raw: &mut [u8]) -> Result<(), String> {
    assert!(raw.len() >= 4);

    let val = ((entry.level as u32) << 16)
        | ((u8::from(entry.src) as u32) << 8)
        | (u8::from(entry.dst) as u32);
    val.build_quadlet(&mut raw[..4]);

    Ok(())
}

fn deserialize_meter_entry(entry: &mut IonixMeterEntry, raw: &[u8]) -> Result<(), String> {
    assert!(raw.len() >= 4);

    let mut val = 0u32;
    val.parse_quadlet(&raw[..4]);

    entry.level = ((val & 0xffff0000) >> 16) as i16;
    entry.src = SrcBlk::from(((val & 0x0000ff00) >> 8) as u8);
    entry.dst = DstBlk::from(((val & 0x000000ff) >> 0) as u8);

    Ok(())
}

// NOTE: Router entries in address region for hardware metering.
//
// analog-input-0/1                 -> ins0:0/1
// analog-input-2/3                 -> ins0:2/3
// analog-input-4/5                 -> ins0:8/9
// analog-input-6/8                 -> ins0:10/11
// coaxial-input-2/3                -> aes:2/3
//
// avs0:0/1    (stream-input-0/1)   -> mixer-tx0:0/1   (mixer-input-0/1)
// avs0:2/3    (stream-input-2/3)   -> mixer-tx0:2/3   (mixer-input-2/3)
// avs0:4/5    (stream-input-4/5)   -> mixer-tx0:4/5   (mixer-input-4/5)
// avs0:6/7    (stream-input-6/7)   -> mixer-tx0:6/7   (mixer-input-6/7)
// aes:2/3                          -> mixer-tx0:8/9   (mixer-input-8/9)
// ins0:0/1                         -> mixer-tx0:10/11 (mixer-input-10/11)
// ins0:2/3                         -> mixer-tx0:12/13 (mixer-input-12/13)
// ins0:8/9                         -> mixer-tx0:14/15 (mixer-input-14/15)
// ins0:10/11                       -> mixer-tx1:0/1   (mixer-input-16/17)
//
// ins0:0/1                         -> avs0:0/1        (stream-output-0/1)
// ins0:2/3                         -> avs0:2/3        (stream-output-2/3)
// ins0:8/9                         -> avs0:4/5        (stream-output-4/5)
// ins0:10/11                       -> avs0:6/7        (stream-output-6/7)
// aes:2/3                          -> avs0:8/9        (stream-output-8/9)
//
// mixer:0/1   (mixer-output-0/1)   -> ins0:4/5
// mixer:2/3   (mixer-output-2/3)   -> ins0:6/7
// mixer:4/5   (mixer-output-4/5)   -> ins0:12/13
// mixer:6/7   (mixer-output-6/7)   -> ins0:14/15
// mixer:8/9   (mixer-output-8/9)   -> (unused?)
// mixer:10/11 (mixer-output-10/11) -> ins1:0/1
// mixer:12/13 (mixer-output-12/13) -> ins1:2/3 (unused?)
//
// ins0:4/5                         -> analog-output-0/1
// ins0:6/7                         -> analog-output-2/3
// ins0:12/13                       -> analog-output-4/5
// ins0:14/15                       -> analog-output-6/7
// avs0:8/9                         -> coaxial-output-0/1
// ins1:0/1                         -> main-output-0/1 (headphone-output-0/1)

impl<O: LexiconOperation> LexiconParametersSerdes<IonixMeter> for O {
    const NAME: &'static str = "meter";

    const OFFSET_RANGES: &'static [Range<usize>] = &[Range {
        start: METER_OFFSET,
        end: METER_OFFSET + METER_SIZE,
    }];

    fn serialize_params(params: &IonixMeter, raw: &mut [u8]) -> Result<(), String> {
        raw.fill_with(Default::default);

        let mut entry = IonixMeterEntry::default();
        let mut pos = 0;

        [
            (
                &params.stream_inputs[..8],
                DstBlkId::MixerTx0,
                0,
                SrcBlkId::Avs0,
                0,
            ),
            (
                &params.stream_inputs[8..10],
                DstBlkId::Aes,
                2,
                SrcBlkId::Avs0,
                8,
            ),
            (
                &params.spdif_inputs[..],
                DstBlkId::MixerTx0,
                8,
                SrcBlkId::Aes,
                2,
            ),
            (
                &params.analog_inputs[..4],
                DstBlkId::MixerTx0,
                10,
                SrcBlkId::Ins0,
                0,
            ),
            (
                &params.analog_inputs[4..6],
                DstBlkId::MixerTx0,
                14,
                SrcBlkId::Ins0,
                8,
            ),
            (
                &params.analog_inputs[6..8],
                DstBlkId::MixerTx1,
                0,
                SrcBlkId::Ins0,
                10,
            ),
            (
                &params.bus_outputs[..4],
                DstBlkId::Ins0,
                4,
                SrcBlkId::Mixer,
                0,
            ),
            (
                &params.bus_outputs[4..],
                DstBlkId::Ins0,
                12,
                SrcBlkId::Mixer,
                4,
            ),
            (
                &params.main_outputs[..],
                DstBlkId::Ins1,
                0,
                SrcBlkId::Mixer,
                10,
            ),
        ]
        .iter()
        .try_for_each(
            |(levels, dst_blk_id, dst_blk_ch_offset, src_blk_id, src_blk_ch_offset)| {
                levels.iter().enumerate().try_for_each(|(i, &level)| {
                    entry.level = level as i16;
                    entry.dst.id = *dst_blk_id;
                    entry.dst.ch = (i + dst_blk_ch_offset) as u8;
                    entry.src.id = *src_blk_id;
                    entry.src.ch = (i + src_blk_ch_offset) as u8;
                    serialize_meter_entry(&entry, &mut raw[pos..(pos + 4)]).map(|_| pos += 4)
                })
            },
        )
    }

    fn deserialize_params(params: &mut IonixMeter, raw: &[u8]) -> Result<(), String> {
        let mut entry = IonixMeterEntry::default();

        (0..raw.len()).step_by(4).try_for_each(|pos| {
            deserialize_meter_entry(&mut entry, &raw[pos..(pos + 4)]).map(|_| {
                match (entry.dst.id, entry.dst.ch, entry.src.id, entry.src.ch) {
                    (DstBlkId::MixerTx0, 0..=7, SrcBlkId::Avs0, 0..=7) => {
                        let pos = entry.src.ch as usize;
                        params.stream_inputs[pos] = entry.level as i32;
                    }
                    (DstBlkId::Aes, 2..=3, SrcBlkId::Avs0, 8..=9) => {
                        let pos = entry.src.ch as usize;
                        params.stream_inputs[pos] = entry.level as i32;
                    }
                    (DstBlkId::MixerTx0, 8..=9, SrcBlkId::Aes, 2..=3) => {
                        let pos = (entry.src.ch - 2) as usize;
                        params.spdif_inputs[pos] = entry.level as i32;
                    }
                    (DstBlkId::MixerTx0, 10..=13, SrcBlkId::Ins0, 0..=3) => {
                        let pos = entry.src.ch as usize;
                        params.analog_inputs[pos] = entry.level as i32;
                    }
                    (DstBlkId::MixerTx0, 14..=15, SrcBlkId::Ins0, 8..=9) => {
                        let pos = 4 + (entry.src.ch - 8) as usize;
                        params.analog_inputs[pos] = entry.level as i32;
                    }
                    (DstBlkId::MixerTx1, 0..=2, SrcBlkId::Ins0, 10..=11) => {
                        let pos = 6 + (entry.src.ch - 10) as usize;
                        params.analog_inputs[pos] = entry.level as i32;
                    }
                    (DstBlkId::Ins0, 4..=7, SrcBlkId::Mixer, 0..=3) => {
                        let pos = entry.src.ch as usize;
                        params.bus_outputs[pos] = entry.level as i32;
                    }
                    (DstBlkId::Ins0, 12..=15, SrcBlkId::Mixer, 4..=7) => {
                        let pos = 4 + (entry.src.ch - 4) as usize;
                        params.bus_outputs[pos] = entry.level as i32;
                    }
                    (DstBlkId::Ins1, 0..=2, SrcBlkId::Mixer, 10..=11) => {
                        let pos = (entry.src.ch - 10) as usize;
                        params.main_outputs[pos] = entry.level as i32;
                    }
                    _ => (),
                }
            })
        })
    }
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

/// Source of mixer.
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
