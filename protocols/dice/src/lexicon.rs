// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol specific to Lexicon I-ONIX FW810s.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by Lexicon for I-ONIX FW810s.
//!
//! ## Diagram of internal signal flow
//!
//! ```text
//!
//! analog-input-0/1        -> stream-output-0/1
//! analog-input-2/3        -> stream-output-2/3
//! analog-input-4/5        -> stream-output-4/5
//! analog-input-6/8        -> stream-output-6/7
//! coaxial-input-0/1       -> stream-output-8/9
//!
//! stream-input-0/1        -> mixer-input-0/1
//! stream-input-2/3        -> mixer-input-2/3
//! stream-input-4/5        -> mixer-input-4/5
//! stream-input-6/7        -> mixer-input-6/7
//! analog-input-0/1        -> mixer-input-8/9
//! analog-input-2/3        -> mixer-input-10/11
//! analog-input-4/5        -> mixer-input-12/13
//! analog-input-6/8        -> mixer-input-14/15
//! coaxial-input-0/1       -> mixer-input-16/17
//!
//! mixer-output-0/1        -> analog-output-0/1
//! mixer-output-2/3        -> analog-output-2/3
//! mixer-output-4/5        -> analog-output-4/5
//! mixer-output-6/7        -> analog-output-6/7
//! mixer-output-8/9        -> (unused)
//! mixer-output-10/11      -> coaxial-output-0/1
//! mixer-output-12/13      -> main-output-0/1 (headphone-output-0/1)
//! ```

use {
    super::{
        tcat::{
            extension::{router_entry::*, *},
            *,
        },
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

const MIXER_BUS_SRC_SIZE: usize = 0x02d0;
const MIXER_MAIN_SRC_SIZE: usize = 0x0090;
const MIXER_REVERB_SRC_SIZE: usize = 0x090;
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
    let msg = format!("params: {}, cause: {}, raw: {:02x?}", name, cause, raw);
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

impl IonixProtocol {
    /// The number of analog inputs.
    pub const ANALOG_INPUT_COUNT: usize = 8;

    /// The number of S/PDIF inputs.
    pub const SPDIF_INPUT_COUNT: usize = 2;

    /// The number of stream inputs.
    pub const STREAM_INPUT_COUNT: usize = 10;

    /// The number of analog inputs for mixer.
    pub const MIXER_ANALOG_INPUT_COUNT: usize = 8;

    /// The number of S/PDIF inputs for mixer.
    pub const MIXER_SPDIF_INPUT_COUNT: usize = 2;

    /// The number of stream inputs for mixer.
    pub const MIXER_STREAM_INPUT_COUNT: usize = 8;

    /// The number of channels in bus mixer.
    pub const MIXER_BUS_COUNT: usize = 8;

    /// The number of channels in main mixer.
    pub const MIXER_MAIN_COUNT: usize = 2;

    /// The number of channels in reverb mixer.
    pub const MIXER_REVERB_COUNT: usize = 2;
}

/// Hardware meter.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct IonixMeter {
    /// Detected level of analog inputs.
    pub analog_inputs: [u16; IonixProtocol::ANALOG_INPUT_COUNT],
    /// Detected level of S/PDIF inputs.
    pub spdif_inputs: [u16; IonixProtocol::SPDIF_INPUT_COUNT],
    /// Detected level of stream inputs.
    pub stream_inputs: [u16; IonixProtocol::STREAM_INPUT_COUNT],
    /// Detected level of mixer bus outputs.
    pub bus_outputs: [u16; IonixProtocol::MIXER_BUS_COUNT],
    /// Detected level of main outputs.
    pub main_outputs: [u16; IonixProtocol::MIXER_MAIN_COUNT],
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

        let mut entry = RouterEntry::default();
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
                    entry.peak = level;
                    entry.dst.id = *dst_blk_id;
                    entry.dst.ch = (i + dst_blk_ch_offset) as u8;
                    entry.src.id = *src_blk_id;
                    entry.src.ch = (i + src_blk_ch_offset) as u8;
                    serialize_router_entry(&entry, &mut raw[pos..(pos + 4)]).map(|_| pos += 4)
                })
            },
        )
    }

    fn deserialize_params(params: &mut IonixMeter, raw: &[u8]) -> Result<(), String> {
        let mut entry = RouterEntry::default();

        (0..raw.len()).step_by(4).try_for_each(|pos| {
            deserialize_router_entry(&mut entry, &raw[pos..(pos + 4)]).map(|_| {
                match (entry.dst.id, entry.dst.ch, entry.src.id, entry.src.ch) {
                    (DstBlkId::MixerTx0, 0..=7, SrcBlkId::Avs0, 0..=7) => {
                        let pos = entry.src.ch as usize;
                        params.stream_inputs[pos] = entry.peak;
                    }
                    (DstBlkId::Aes, 2..=3, SrcBlkId::Avs0, 8..=9) => {
                        let pos = entry.src.ch as usize;
                        params.stream_inputs[pos] = entry.peak;
                    }
                    (DstBlkId::MixerTx0, 8..=9, SrcBlkId::Aes, 2..=3) => {
                        let pos = (entry.src.ch - 2) as usize;
                        params.spdif_inputs[pos] = entry.peak;
                    }
                    (DstBlkId::MixerTx0, 10..=13, SrcBlkId::Ins0, 0..=3) => {
                        let pos = entry.src.ch as usize;
                        params.analog_inputs[pos] = entry.peak;
                    }
                    (DstBlkId::MixerTx0, 14..=15, SrcBlkId::Ins0, 8..=9) => {
                        let pos = 4 + (entry.src.ch - 8) as usize;
                        params.analog_inputs[pos] = entry.peak;
                    }
                    (DstBlkId::MixerTx1, 0..=1, SrcBlkId::Ins0, 10..=11) => {
                        let pos = 6 + (entry.src.ch - 10) as usize;
                        params.analog_inputs[pos] = entry.peak;
                    }
                    (DstBlkId::Ins0, 4..=7, SrcBlkId::Mixer, 0..=3) => {
                        let pos = entry.src.ch as usize;
                        params.bus_outputs[pos] = entry.peak;
                    }
                    (DstBlkId::Ins0, 12..=15, SrcBlkId::Mixer, 4..=7) => {
                        let pos = 4 + (entry.src.ch - 4) as usize;
                        params.bus_outputs[pos] = entry.peak;
                    }
                    (DstBlkId::Ins1, 0..=1, SrcBlkId::Mixer, 10..=11) => {
                        let pos = (entry.src.ch - 10) as usize;
                        params.main_outputs[pos] = entry.peak;
                    }
                    _ => (),
                }
            })
        })
    }
}

/// Gains of sources for mixer.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct IonixMixerSources {
    /// Gains of 8 analog inputs.
    pub stream_inputs: [i16; IonixProtocol::MIXER_STREAM_INPUT_COUNT],
    /// Gains of 2 S/PDIF inputs.
    pub spdif_inputs: [i16; IonixProtocol::MIXER_SPDIF_INPUT_COUNT],
    /// Gains of 8 analog inputs.
    pub analog_inputs: [i16; IonixProtocol::MIXER_ANALOG_INPUT_COUNT],
}

/// Parameters of mixer.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct IonixMixerParameters {
    /// Sources for bus mixers.
    pub bus_sources: [IonixMixerSources; IonixProtocol::MIXER_BUS_COUNT],
    /// Sources for main mixers.
    pub main_sources: [IonixMixerSources; IonixProtocol::MIXER_MAIN_COUNT],
    /// Sources for reverb effects.
    pub reverb_sources: [IonixMixerSources; IonixProtocol::MIXER_REVERB_COUNT],
}

impl<O: LexiconOperation> LexiconParametersSerdes<IonixMixerParameters> for O {
    const NAME: &'static str = "meter";

    const OFFSET_RANGES: &'static [Range<usize>] = &[
        Range {
            start: MIXER_BUS_SRC_OFFSET,
            end: MIXER_BUS_SRC_OFFSET + MIXER_BUS_SRC_SIZE,
        },
        Range {
            start: MIXER_MAIN_SRC_OFFSET,
            end: MIXER_MAIN_SRC_OFFSET + MIXER_MAIN_SRC_SIZE,
        },
        Range {
            start: MIXER_REVERB_SRC_OFFSET,
            end: MIXER_REVERB_SRC_OFFSET + MIXER_REVERB_SRC_SIZE,
        },
    ];

    fn serialize_params(params: &IonixMixerParameters, raw: &mut [u8]) -> Result<(), String> {
        [
            (&params.bus_sources[..], 0x0000),
            (&params.main_sources[..], 0x02d0),
            (&params.reverb_sources[..], 0x0360),
        ]
        .iter()
        .for_each(|(srcs, offset)| {
            srcs.iter().enumerate().for_each(|(i, src)| {
                src.stream_inputs
                    .iter()
                    .chain(src.spdif_inputs.iter())
                    .chain(src.analog_inputs.iter())
                    .enumerate()
                    .for_each(|(j, gain)| {
                        let pos = *offset + i * 0x48 + j * 4;
                        serialize_i16(gain, &mut raw[pos..(pos + 4)]);
                    });
            });
        });

        Ok(())
    }

    fn deserialize_params(params: &mut IonixMixerParameters, raw: &[u8]) -> Result<(), String> {
        [
            (&mut params.bus_sources[..], 0x0000),
            (&mut params.main_sources[..], 0x02d0),
            (&mut params.reverb_sources[..], 0x0360),
        ]
        .iter_mut()
        .for_each(|(srcs, offset)| {
            srcs.iter_mut().enumerate().for_each(|(i, src)| {
                src.stream_inputs
                    .iter_mut()
                    .chain(src.spdif_inputs.iter_mut())
                    .chain(src.analog_inputs.iter_mut())
                    .enumerate()
                    .for_each(|(j, gain)| {
                        let pos = *offset + i * 0x48 + j * 4;
                        deserialize_i16(gain, &raw[pos..(pos + 4)]);
                    });
            });
        });

        Ok(())
    }
}

impl<O: LexiconOperation + LexiconParametersSerdes<IonixMixerParameters>>
    LexiconMutableParametersOperation<IonixMixerParameters> for O
{
}

/// Serialize and deserialize data of system exclusive message.
pub trait IonixSysExDataSerdes<T> {
    /// Name of system exclusive message.
    const NAME: &'static str;

    /// Serialize for data of system exclusive message.
    fn serialize_sysex_data(params: &T) -> Result<Vec<Vec<u8>>, String>;

    /// Deserialize for data of system exclusive message.
    fn deserialize_sysex_data<U: AsRef<[u8]>>(params: &mut T, raw: &[U]) -> Result<(), String>;
}

fn serialize_effect_frame(data: &[u8]) -> Vec<u8> {
    /// Prefix of data for system exclusive message.
    const DATA_PREFIX: [u8; 5] = [0x06, 0x00, 0x1b, 0x01, 0x41];

    /// Prefix of system exclusive message.
    const SYSEX_MSG_PREFIX: u8 = 0xf0;

    /// Suffix of system exclusive message.
    const SYSEX_MSG_SUFFIX: u8 = 0xf7;

    // NOTE: The data has prefix.
    let mut sysex_data = DATA_PREFIX.to_vec();
    sysex_data.extend_from_slice(data);

    // NOTE: Append checksum calculated by XOR for all the data.
    let checksum = sysex_data.iter().fold(0u8, |val, &msg| val | msg);
    sysex_data.push(checksum);

    // NOTE: Construct MIDI system exclusive message.
    let mut sysex = vec![SYSEX_MSG_PREFIX];
    sysex.append(&mut sysex_data);
    sysex.push(SYSEX_MSG_SUFFIX);

    // NOTE: One quadlet deliver one byte of message.
    let mut raw = Vec::new();
    sysex
        .iter()
        .for_each(|&msg| raw.extend_from_slice(&(msg as u32).to_be_bytes()));

    raw
}

fn generate_effect_err(name: &str, cause: &str) -> Error {
    let msg = format!("params: {}, cause: {}", name, cause);
    Error::new(GeneralProtocolError::VendorDependent, &msg)
}

/// Operation for effect.
pub trait LexiconEffectOperation<T>: LexiconOperation + IonixSysExDataSerdes<T> {
    /// Update parameters for effect.
    fn update_effect_params(
        req: &FwReq,
        node: &FwNode,
        params: &T,
        prev: &mut T,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let new = Self::serialize_sysex_data(params)
            .map_err(|cause| generate_effect_err(Self::NAME, &cause))?;
        let old = Self::serialize_sysex_data(prev)
            .map_err(|cause| generate_effect_err(Self::NAME, &cause))?;

        new.iter()
            .zip(old.iter())
            .filter(|(n, o)| !n.eq(o))
            .try_for_each(|(data, _)| {
                let mut raw = serialize_effect_frame(data);
                Self::write(req, node, EFFECT_OFFSET, &mut raw, timeout_ms)
            })?;

        Self::deserialize_sysex_data(prev, &new)
            .map_err(|cause| generate_effect_err(Self::NAME, &cause))
    }
}

impl<O: LexiconOperation + IonixSysExDataSerdes<T>, T> LexiconEffectOperation<T> for O {}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn mixer_params_serdes() {
        let mut params = IonixMixerParameters::default();
        params
            .bus_sources
            .iter_mut()
            .chain(params.main_sources.iter_mut())
            .chain(params.reverb_sources.iter_mut())
            .flat_map(|srcs| {
                srcs.stream_inputs
                    .iter_mut()
                    .chain(srcs.spdif_inputs.iter_mut())
                    .chain(srcs.analog_inputs.iter_mut())
            })
            .enumerate()
            .for_each(|(i, gain)| *gain = i as i16);

        let size = compute_params_size(
            <IonixProtocol as LexiconParametersSerdes<IonixMixerParameters>>::OFFSET_RANGES,
        );
        let mut raw = vec![0u8; size];
        IonixProtocol::serialize_params(&params, &mut raw).unwrap();

        let mut p = IonixMixerParameters::default();
        IonixProtocol::deserialize_params(&mut p, &raw).unwrap();

        assert_eq!(params.bus_sources, p.bus_sources, "{:02x?}", raw);
        assert_eq!(params.main_sources, p.main_sources);
        assert_eq!(params.reverb_sources, p.reverb_sources);
    }
}
