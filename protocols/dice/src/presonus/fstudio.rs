// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol implementation for PreSonus FireStudio.
//!
//! ## Diagram of internal signal flow
//!
//! ```text
//!
//! analog-input-1/2 ---------------------------> stream-output-A-1/2
//! analog-input-3/4 ---------------------------> stream-output-A-3/4
//! analog-input-5/6 ---------------------------> stream-output-A-5/6
//! analog-input-7/8 ---------------------------> stream-output-A-7/8
//! adat-input-1/2 -----------------------------> stream-output-A-9/10
//! adat-input-3/4 -----------------------------> stream-output-A-11/12
//! adat-input-5/6 -----------------------------> stream-output-A-13/14
//! adat-input-7/8 -----------------------------> stream-output-A-15/16
//! adat-input-9/10 ----------------------------> stream-output-B-1/2
//! adat-input-11/12 ---------------------------> stream-output-B-3/4
//! adat-input-13/14 ---------------------------> stream-output-B-5/6
//! adat-input-15/16 ---------------------------> stream-output-B-7/8
//! spdif-input-1/2 ----------------------------> stream-output-B-9/10
//!
//! analog-input-1/2 ---------------------------> mixer-source-1/2
//! analog-input-3/4 ---------------------------> mixer-source-3/4
//! analog-input-5/6 ---------------------------> mixer-source-5/6
//! analog-input-7/8 ---------------------------> mixer-source-7/8
//! adat-input-1/2 -----------------------------> mixer-source-9/10
//! adat-input-3/4 -----------------------------> mixer-source-11/12
//! adat-input-5/6 -----------------------------> mixer-source-13/14
//! adat-input-7/8 -----------------------------> mixer-source-15/16
//! spdif-input-1/2 ----------------------------> mixer-source-17/18
//! stream-input-A-1/2 -------------------------> mixer-source-19/20
//! stream-input-A-3/4 -------------------------> mixer-source-21/22
//! stream-input-A-5/6 -------------------------> mixer-source-23/24
//! stream-input-A-7/8 -------------------------> mixer-source-25/26
//! stream-input-B-9/10 ------------------------> mixer-source-27/28
//! adat-input-9..16 --------------or-----------> mixer-source-29..36
//! stream-input-A-9..16 ----------+
//!
//!                           ++===========++
//! mixer-source-1/2 -------> ||           ||
//! mixer-source-3/4 -------> ||           ||
//! mixer-source-5/6 -------> ||           ||
//! mixer-source-7/8 -------> ||           ||
//! mixer-source-9/10 ------> ||           ||
//! mixer-source-11/12 -----> ||           || --> mixer-output-1/2
//! mixer-source-13/14 -----> ||           || --> mixer-output-3/4
//! mixer-source-15/16 -----> ||           || --> mixer-output-5/6
//! mixer-source-17/18 -----> ||  36 x 18  || --> mixer-output-7/8
//! mixer-source-19/20 -----> ||           || --> mixer-output-9/10
//! mixer-source-21/22 -----> ||   mixer   || --> mixer-output-11/12
//! mixer-source-23/24 -----> ||           || --> mixer-output-13/14
//! mixer-source-25/26 -----> ||           || --> mixer-output-15/15
//! mixer-source-27/28 -----> ||           || --> mixer-output-17/18
//! mixer-source-29/30 -----> ||           ||
//! mixer-source-31/32 -----> ||           ||
//! mixer-source-33/34 -----> ||           ||
//! mixer-source-35/36 -----> ||           ||
//!                           ++===========++
//!
//!                           ++===========++
//! mixer-source-1/2 -------> ||           ||
//! mixer-source-3/4 -------> ||           ||
//! mixer-source-5/6 -------> ||           ||
//! mixer-source-7/8 -------> ||           ||
//! mixer-source-9/10 ------> ||           ||
//! mixer-source-11/12 -----> ||           ||
//! mixer-source-13/14 -----> ||           ||
//! mixer-source-15/16 -----> ||           ||
//! mixer-source-17/18 -----> ||           ||
//! mixer-source-19/20 -----> ||           || --> analog-output-1/2
//! mixer-source-21/22 -----> ||           || --> analog-output-3/4
//! mixer-source-23/24 -----> ||           || --> analog-output-5/6
//! mixer-source-25/27 -----> ||  54 x 18  || --> analog-output-7/8
//! mixer-source-27/28 -----> ||           || --> adat-output-1/2
//! mixer-source-29/30 -----> ||           || --> adat-output-3/4
//! mixer-source-31/32 -----> ||  router   || --> adat-output-5/6
//! mixer-source-33/34 -----> ||           || --> adat-output-7/8
//! mixer-source-35/36 -----> ||           || --> spdif-output-1/2
//! mixer-output-1/2 -------> ||           ||
//! mixer-output-3/4 -------> ||           ||
//! mixer-output-5/6 -------> ||           ||
//! mixer-output-7/8 -------> ||           ||
//! mixer-output-9/10 ------> ||           ||
//! mixer-output-11/12 -----> ||           ||
//! mixer-output-13/14 -----> ||           ||
//! mixer-output-15/15 -----> ||           ||
//! mixer-output-17/18 -----> ||           ||
//!                           ++===========++
//!
//! stream-input-B-1/2 -------------------------> adat-output-9/10
//! stream-input-B-3/4 -------------------------> adat-output-11/12
//! stream-input-B-5/6 -------------------------> adat-output-13/14
//! stream-input-B-7/8 -------------------------> adat-output-15/16
//!
//!                           ++===========++
//! analog-output-1/2 ------> ||           ||
//! analog-output-3/4 ------> ||           ||
//! analog-output-5/6 ------> ||           || --> main-output-1/2
//! analog-output-7/8 ------> ||  18 x 8   || --> headphone-output-1/2
//! adat-output-1/2 --------> ||           || --> headphone-output-3/4
//! adat-output-3/4 --------> ||  router   || --> headphone-output-5/6
//! adat-output-5/6 --------> ||           ||
//! adat-output-7/8 --------> ||           ||
//! spdif-output-1/2 -------> ||           ||
//!                           ++===========++
//!

use {
    super::{super::tcat::global_section::*, *},
    std::ops::Range,
};

/// Protocol implementation specific to FireStudio.
#[derive(Default, Debug)]
pub struct FStudioProtocol;

impl TcatOperation for FStudioProtocol {}

// MEMO: the device returns 'SPDIF\ADAT\Word Clock\Unused\Unused\Unused\Unused\Internal\\'.
impl TcatGlobalSectionSpecification for FStudioProtocol {
    const CLOCK_SOURCE_LABEL_TABLE: &'static [ClockSource] = &[
        ClockSource::Aes1,
        ClockSource::Adat,
        ClockSource::WordClock,
        ClockSource::Arx1,
        ClockSource::Arx2,
        ClockSource::Arx3,
        ClockSource::Arx4,
        ClockSource::Internal,
    ];
}

impl FStudioOperation for FStudioProtocol {}

const OFFSET: usize = 0x00700000;

const MIXER_PHYS_SRC_PARAMS_OFFSET: usize = 0x0038;
const MIXER_STREAM_SRC_PARAMS_OFFSET: usize = 0x07d0;
const MIXER_SELECTABLE_SRC_PARAMS_OFFSET: usize = 0x0c08;
const OUTPUT_PARAMS_OFFSET: usize = 0x0f68;
const MIXER_OUTPUT_PARAMS_OFFSET: usize = 0x1040;
const OUTPUT_SRC_OFFSET: usize = 0x10ac;
const OUTPUT_ASSIGN_OFFSET: usize = 0x10f4;
const OUTPUT_BNC_TERMINATE_OFFSET: usize = 0x1118;
const MIXER_EXPANSION_MODE_OFFSET: usize = 0x1128;
const MIXER_SRC_LINK_OFFSET: usize = 0x112c;
const OUTPUT_LINK_OFFSET: usize = 0x1150;
const METER_OFFSET: usize = 0x13e8;

// For volume, unused, mute of 8 analog outputs, 8 ADAT0 outputs, and 2 S/PDIF outputs.
const OUTPUT_PARAMS_SIZE: usize = 4 * 3 * (8 + 8 + 2);
// For source of 8 analog outputs, 8 ADAT0 outputs, and 2 S/PDIF outputs.
const OUTPUT_SRC_SIZE: usize = 4 * (8 + 8 + 2);
// Assignment to main and 3 headphone outputs.
const OUTPUT_ASSIGN_SIZE: usize = 4 * 4;
const OUTPUT_BNC_TERMINATE_SIZE: usize = 4;
// Link bit flags for 8 analog outputs, 8 ADAT0 outputs, and 2 S/PDIF outputs.
const OUTPUT_LINK_SIZE: usize = 4;

// For gain, pan, mute of 8 analog inputs, 8 Adat0 inputs, and 2 S/PDIF inputs in 9 stereo mixers.
const MIXER_PHYS_SRC_PARAMS_SIZE: usize = 4 * 9 * 3 * (8 + 8 + 2);
// For gain, pan, mute of 10 stream inputs in 9 stereo mixers.
const MIXER_STREAM_SRC_PARAMS_SIZE: usize = 4 * 9 * 3 * 10;
// For gain, pan, mute of 8 selectable inputs in 9 stereo mixers.
const MIXER_SELECTABLE_SRC_PARAMS_SIZE: usize = 4 * 9 * 3 * 8;
// For gain, unused, mute of output pairs from 9 stereo mixers.
const MIXER_OUTPUT_PARAMS_SIZE: usize = 4 * 9 * 3;
const MIXER_EXPANSION_MODE_SIZE: usize = 4;
// For pair link of source pairs in 9 stereo mixers.
const MIXER_SRC_LINK_SIZE: usize = 4 * 9;

/// Serialize and deserialize parameters for FireStudio.
pub trait FStudioParametersSerdes<T> {
    /// The representative name of parameters.
    const NAME: &'static str;

    /// The list of ranges for offset and size.
    const OFFSET_RANGES: &'static [Range<usize>];

    /// Serialize for raw data.
    fn serialize_params(params: &T, raw: &mut [u8]) -> Result<(), String>;

    /// Deserialize for raw data.
    fn deserialize_params(params: &mut T, raw: &[u8]) -> Result<(), String>;
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

/// Operation for parameters in FireStudio.
pub trait FStudioOperation: TcatOperation {
    fn read_parameters(
        req: &FwReq,
        node: &FwNode,
        offset: usize,
        raw: &mut [u8],
        timeout_ms: u32,
    ) -> Result<(), Error> {
        Self::read(req, node, OFFSET + offset, raw, timeout_ms)
    }

    fn write_parameters(
        req: &FwReq,
        node: &FwNode,
        offset: usize,
        raw: &mut [u8],
        timeout_ms: u32,
    ) -> Result<(), Error> {
        Self::write(req, node, OFFSET + offset, raw, timeout_ms)
    }
}

/// Operation for parameters to cache state of hardware.
pub trait FStudioParametersOperation<T>: FStudioOperation + FStudioParametersSerdes<T> {
    /// Cache state of hardware for whole parameters.
    fn cache_whole_parameters(
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

impl<O: FStudioOperation + FStudioParametersSerdes<T>, T> FStudioParametersOperation<T> for O {}

/// Operation for parameters to update state of hardware.
pub trait FStudioMutableParametersOperation<T>:
    FStudioOperation + FStudioParametersSerdes<T>
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

/// Hardware meter.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct FStudioMeter {
    /// Detected levels for analog inputs.
    pub analog_inputs: [u8; 8],
    /// Detected levels for stream inputs.
    pub stream_inputs: [u8; 18],
    /// Detected levels for mixer outputs.
    pub mixer_outputs: [u8; 18],
}

impl FStudioMeter {
    const SIZE: usize = 64;
}

impl FStudioParametersSerdes<FStudioMeter> for FStudioProtocol {
    const NAME: &'static str = "meter";

    const OFFSET_RANGES: &'static [Range<usize>] = &[Range {
        start: METER_OFFSET,
        end: METER_OFFSET + FStudioMeter::SIZE,
    }];

    fn serialize_params(params: &FStudioMeter, raw: &mut [u8]) -> Result<(), String> {
        [
            (8, &params.analog_inputs[..]),
            (16, &params.stream_inputs[..]),
            (40, &params.mixer_outputs[..]),
        ]
        .iter()
        .for_each(|(offset, meters)| {
            meters.iter().enumerate().for_each(|(i, &meter)| {
                let pos = *offset + (i / 4) * 4 + (3 - i % 4);
                raw[pos] = meter;
            });
        });

        Ok(())
    }

    fn deserialize_params(params: &mut FStudioMeter, raw: &[u8]) -> Result<(), String> {
        [
            (8, &mut params.analog_inputs[..]),
            (16, &mut params.stream_inputs[..]),
            (40, &mut params.mixer_outputs[..]),
        ]
        .iter_mut()
        .for_each(|(offset, meters)| {
            meters.iter_mut().enumerate().for_each(|(i, meter)| {
                let pos = *offset + (i / 4) * 4 + (3 - i % 4);
                *meter = raw[pos];
            });
        });

        Ok(())
    }
}

/// Source of output.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum OutputSrc {
    /// Analog input 1..8.
    Analog(usize),
    /// ADAT input 1..8 in 1st optical interface.
    Adat0(usize),
    /// S/PDIF input 1/2 in coaxial interface.
    Spdif(usize),
    /// Stream input A 1..8 and stream input 9/10.
    Stream(usize),
    /// Either stream input A 9..16, or ADAT input 9..16 in 2nd optical interface.
    StreamAdat1(usize),
    /// Outputs from stereo mixer 1..9.
    MixerOut(usize),
}

impl Default for OutputSrc {
    fn default() -> Self {
        Self::Analog(0)
    }
}

fn serialize_output_source(src: &OutputSrc, raw: &mut [u8]) -> Result<(), String> {
    assert!(raw.len() >= 4);

    let val = (match src {
        OutputSrc::Analog(val) => *val,
        OutputSrc::Adat0(val) => *val + 0x08,
        OutputSrc::Spdif(val) => *val + 0x10,
        OutputSrc::Stream(val) => *val + 0x12,
        OutputSrc::StreamAdat1(val) => *val + 0x1c,
        OutputSrc::MixerOut(val) => *val + 0x24,
    }) as u32;

    val.build_quadlet(raw);

    Ok(())
}

fn deserialize_output_source(src: &mut OutputSrc, raw: &[u8]) -> Result<(), String> {
    assert!(raw.len() >= 4);

    let mut quadlet = [0; 4];
    quadlet.copy_from_slice(&raw[..4]);
    let val = u32::from_be_bytes(quadlet) as usize;

    *src = match val {
        0x00..=0x07 => OutputSrc::Analog(val),
        0x08..=0x0f => OutputSrc::Adat0(val - 0x08),
        0x10..=0x11 => OutputSrc::Spdif(val - 0x10),
        0x12..=0x1b => OutputSrc::Stream(val - 0x12),
        0x1c..=0x23 => OutputSrc::StreamAdat1(val - 0x1c),
        0x24..=0x35 => OutputSrc::MixerOut(val - 0x24),
        _ => Err(format!("Output source not found for value {}", val))?,
    };

    Ok(())
}

/// Parameters for left and right channels of output.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct OutputPair {
    /// Volume of each channel, between 0x00 and 0xff.
    pub volumes: [u8; 2],
    /// Whether to be muted for each channel.
    pub mutes: [bool; 2],
    /// Source of both channels.
    pub src: OutputSrc,
    /// Whether to link both channels.
    pub link: bool,
}

/// Parameters for outputs.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct OutputParameters {
    /// Pair of outputs.
    pub pairs: [OutputPair; 9],
    /// Source of main output.
    pub main_assign: AssignTarget,
    /// Source of 3 headphones.
    pub headphone_assigns: [AssignTarget; 3],
    /// Whether to suppress generation of word clock signal in BNC output interface.
    pub bnc_terminate: bool,
}

impl FStudioParametersSerdes<OutputParameters> for FStudioProtocol {
    const NAME: &'static str = "output-state";

    const OFFSET_RANGES: &'static [Range<usize>] = &[
        Range {
            start: OUTPUT_PARAMS_OFFSET,
            end: OUTPUT_PARAMS_OFFSET + OUTPUT_PARAMS_SIZE,
        },
        Range {
            start: OUTPUT_SRC_OFFSET,
            end: OUTPUT_SRC_OFFSET + OUTPUT_SRC_SIZE,
        },
        Range {
            start: OUTPUT_ASSIGN_OFFSET,
            end: OUTPUT_ASSIGN_OFFSET + OUTPUT_ASSIGN_SIZE,
        },
        Range {
            start: OUTPUT_BNC_TERMINATE_OFFSET,
            end: OUTPUT_BNC_TERMINATE_OFFSET + OUTPUT_BNC_TERMINATE_SIZE,
        },
        Range {
            start: OUTPUT_LINK_OFFSET,
            end: OUTPUT_LINK_OFFSET + OUTPUT_LINK_SIZE,
        },
    ];

    fn serialize_params(params: &OutputParameters, raw: &mut [u8]) -> Result<(), String> {
        params.pairs.iter().enumerate().try_for_each(|(i, pair)| {
            pair.volumes.iter().enumerate().for_each(|(j, &vol)| {
                let pos = 4 * 3 * (i * 2 + j);
                let val = vol as u32;
                raw[pos..(pos + 4)].copy_from_slice(&val.to_be_bytes());
            });

            pair.mutes.iter().enumerate().for_each(|(j, &mute)| {
                let pos = 4 * (3 * (i * 2 + j) + 2);
                let val = mute as u32;
                raw[pos..(pos + 4)].copy_from_slice(&val.to_be_bytes());
            });

            let pos = 216 + 4 * i;
            serialize_output_source(&pair.src, &mut raw[pos..(pos + 4)])
        })?;

        serialize_assign_target(&params.main_assign, &mut raw[288..292])?;
        serialize_assign_target(&params.headphone_assigns[0], &mut raw[292..296])?;
        serialize_assign_target(&params.headphone_assigns[1], &mut raw[296..300])?;
        serialize_assign_target(&params.headphone_assigns[2], &mut raw[300..304])?;

        let val = params.bnc_terminate as u32;
        raw[304..308].copy_from_slice(&val.to_be_bytes());

        let mut val = 0u32;
        params
            .pairs
            .iter()
            .enumerate()
            .filter(|(_, pair)| pair.link)
            .for_each(|(i, _)| {
                val |= 1 << i;
            });
        raw[308..312].copy_from_slice(&val.to_be_bytes());

        Ok(())
    }

    fn deserialize_params(params: &mut OutputParameters, raw: &[u8]) -> Result<(), String> {
        let mut quadlet = [0; 4];

        params
            .pairs
            .iter_mut()
            .enumerate()
            .try_for_each(|(i, pair)| {
                pair.volumes.iter_mut().enumerate().for_each(|(j, vol)| {
                    let pos = 4 * 3 * (i * 2 + j);
                    quadlet.copy_from_slice(&raw[pos..(pos + 4)]);
                    *vol = u32::from_be_bytes(quadlet) as u8;
                });

                pair.mutes.iter_mut().enumerate().for_each(|(j, mute)| {
                    let pos = 4 * (3 * (i * 2 + j) + 2);
                    quadlet.copy_from_slice(&raw[pos..(pos + 4)]);
                    *mute = u32::from_be_bytes(quadlet) > 0;
                });

                let pos = 216 + 4 * i;
                deserialize_output_source(&mut pair.src, &raw[pos..(pos + 4)])
            })?;

        deserialize_assign_target(&mut params.main_assign, &raw[288..292])?;
        deserialize_assign_target(&mut params.headphone_assigns[0], &raw[292..296])?;
        deserialize_assign_target(&mut params.headphone_assigns[1], &raw[296..300])?;
        deserialize_assign_target(&mut params.headphone_assigns[2], &raw[300..304])?;

        quadlet.copy_from_slice(&raw[304..308]);
        params.bnc_terminate = u32::from_be_bytes(quadlet) > 0;

        quadlet.copy_from_slice(&raw[308..312]);
        let val = u32::from_be_bytes(quadlet);
        params.pairs.iter_mut().enumerate().for_each(|(i, pair)| {
            pair.link = (val & (1 << i)) > 0;
        });

        Ok(())
    }
}

impl<O: FStudioOperation + FStudioParametersSerdes<OutputParameters>>
    FStudioMutableParametersOperation<OutputParameters> for O
{
}

/// Target of output assignment.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum AssignTarget {
    /// Audio signal to analog output 1/2.
    Analog01,
    /// Audio signal to analog output 3/4.
    Analog23,
    /// Audio signal to analog output 5/6.
    Analog56,
    /// Audio signal to analog output 7/8.
    Analog78,
    /// Audio signal to ADAT output 1/2.
    AdatA01,
    /// Audio signal to ADAT output 3/4.
    AdatA23,
    /// Audio signal to ADAT output 5/6.
    AdatA45,
    /// Audio signal to ADAT output 7/8.
    AdatA67,
    /// Audio signal to S/PDIF output 1/2.
    Spdif01,
}

impl Default for AssignTarget {
    fn default() -> Self {
        Self::Analog01
    }
}

fn serialize_assign_target(target: &AssignTarget, raw: &mut [u8]) -> Result<(), String> {
    let val = match target {
        AssignTarget::Analog01 => 0x00u32,
        AssignTarget::Analog23 => 0x02,
        AssignTarget::Analog56 => 0x04,
        AssignTarget::Analog78 => 0x06,
        AssignTarget::AdatA01 => 0x08,
        AssignTarget::AdatA23 => 0x0a,
        AssignTarget::AdatA45 => 0x0c,
        AssignTarget::AdatA67 => 0x0e,
        AssignTarget::Spdif01 => 0x10,
    };

    raw[..4].copy_from_slice(&val.to_be_bytes());

    Ok(())
}

fn deserialize_assign_target(target: &mut AssignTarget, raw: &[u8]) -> Result<(), String> {
    assert!(raw.len() >= 4);

    let mut quadlet = [0; 4];
    quadlet.copy_from_slice(&raw[..4]);
    let val = u32::from_be_bytes(quadlet);

    *target = match val {
        0x00 => AssignTarget::Analog01,
        0x02 => AssignTarget::Analog23,
        0x04 => AssignTarget::Analog56,
        0x06 => AssignTarget::Analog78,
        0x08 => AssignTarget::AdatA01,
        0x0a => AssignTarget::AdatA23,
        0x0c => AssignTarget::AdatA45,
        0x0e => AssignTarget::AdatA67,
        0x10 => AssignTarget::Spdif01,
        _ => Err(format!("Assign target not found for value {}", val))?,
    };

    Ok(())
}

/// Mode of mixer expansion.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ExpansionMode {
    /// Stream input B 1..8.
    StreamB0_7,
    /// ADAT input 9..16 in 2nd optical interface.
    AdatB0_7,
}

impl Default for ExpansionMode {
    fn default() -> Self {
        Self::StreamB0_7
    }
}

fn serialize_expansion_mode(mode: &ExpansionMode, raw: &mut [u8]) -> Result<(), String> {
    assert!(raw.len() >= 4);

    let val = match mode {
        ExpansionMode::StreamB0_7 => 0u32,
        ExpansionMode::AdatB0_7 => 1,
    };

    raw.copy_from_slice(&val.to_be_bytes());

    Ok(())
}

fn deserialize_expansion_mode(mode: &mut ExpansionMode, raw: &[u8]) -> Result<(), String> {
    assert!(raw.len() >= 4);

    let mut quadlet = [0; 4];
    quadlet.copy_from_slice(&raw[..4]);

    let val = u32::from_be_bytes(quadlet);
    *mode = match val {
        0 => ExpansionMode::StreamB0_7,
        1 => ExpansionMode::AdatB0_7,
        _ => Err(format!("Expansion mode not found for value {}", val))?,
    };

    Ok(())
}

/// Parameters for channels (left and right) in a pair of source to mixer.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct MixerSourcePair {
    /// Gain for each channel, between 0x00 and 0xff.
    pub gains: [u8; 2],
    /// Left and right balance for each channel, between 0x00 and 0x7f.
    pub balances: [u8; 2],
    /// Whether to be muted for each channel.
    pub mutes: [bool; 2],
    /// Whether to link both channels.
    pub link: bool,
}

fn serialize_mixer_source_pair(pair: &MixerSourcePair, raw: &mut [u8]) -> Result<(), String> {
    assert!(raw.len() >= 24);

    let val = pair.gains[0] as u32;
    raw[..4].copy_from_slice(&val.to_be_bytes());

    let val = pair.balances[0] as u32;
    raw[4..8].copy_from_slice(&val.to_be_bytes());

    let val = pair.mutes[0] as u32;
    raw[8..12].copy_from_slice(&val.to_be_bytes());

    let val = pair.gains[1] as u32;
    raw[12..16].copy_from_slice(&val.to_be_bytes());

    let val = pair.balances[1] as u32;
    raw[16..20].copy_from_slice(&val.to_be_bytes());

    let val = pair.mutes[1] as u32;
    raw[20..24].copy_from_slice(&val.to_be_bytes());

    Ok(())
}

fn deserialize_mixer_source_pair(pair: &mut MixerSourcePair, raw: &[u8]) -> Result<(), String> {
    assert!(raw.len() >= 24);

    let mut quadlet = [0; 4];

    quadlet.copy_from_slice(&raw[..4]);
    pair.gains[0] = u32::from_be_bytes(quadlet) as u8;

    quadlet.copy_from_slice(&raw[4..8]);
    pair.balances[0] = u32::from_be_bytes(quadlet) as u8;

    quadlet.copy_from_slice(&raw[8..12]);
    pair.mutes[0] = u32::from_be_bytes(quadlet) > 0;

    quadlet.copy_from_slice(&raw[12..16]);
    pair.gains[1] = u32::from_be_bytes(quadlet) as u8;

    quadlet.copy_from_slice(&raw[16..20]);
    pair.balances[1] = u32::from_be_bytes(quadlet) as u8;

    quadlet.copy_from_slice(&raw[20..24]);
    pair.mutes[1] = u32::from_be_bytes(quadlet) > 0;

    Ok(())
}

/// Parameters for pairs of source to single mixer.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct MixerSources {
    /// Pairs of analog input 1-8.
    pub analog_pairs: [MixerSourcePair; 4],
    /// Pairs of ADAT input 1-8 in optical input interface A.
    pub adat_0_pairs: [MixerSourcePair; 4],
    /// A pair of S/PDIF input 1/2 in coaxial input interface.
    pub spdif_pairs: [MixerSourcePair; 1],
    /// Pairs of stream input 1-8, 17/18 in IEEE 1394 bus.
    pub stream_pairs: [MixerSourcePair; 5],
    /// Pairs of selectable inputs either ADAT input 9-18 in optical input interface B or
    /// stream input 9-16 in IEEE 1394 bus.
    pub selectable_pairs: [MixerSourcePair; 4],
}

/// Parameters for a pair (left and right) of outputs from mixer.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct MixerOutputPair {
    /// Volume of the channels, between 0x00 and 0xff.
    pub volume: u8,
    /// Whether to mute the channels.
    pub mute: bool,
}

fn serialize_mixer_output_pair(pair: &MixerOutputPair, raw: &mut [u8]) -> Result<(), String> {
    assert!(raw.len() >= 12);

    let val = pair.volume as u32;
    raw[..4].copy_from_slice(&val.to_be_bytes());

    let val = pair.mute as u32;
    raw[8..12].copy_from_slice(&val.to_be_bytes());

    Ok(())
}

fn deserialize_mixer_output_pair(pair: &mut MixerOutputPair, raw: &[u8]) -> Result<(), String> {
    assert!(raw.len() >= 12);

    let mut quadlet = [0; 4];

    quadlet.copy_from_slice(&raw[..4]);
    pair.volume = u32::from_be_bytes(quadlet) as u8;

    quadlet.copy_from_slice(&raw[8..12]);
    pair.mute = u32::from_be_bytes(quadlet) > 0;

    Ok(())
}

/// Parameters of stereo mixer.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct MixerParameters {
    /// For sources of 9 stereo mixers.
    pub sources: [MixerSources; 9],
    /// For outputs of 9 stereo mixers.
    pub outputs: [MixerOutputPair; 9],
    /// For selectable input pairs. When expanded, ADAT input 9-18 in optical input interface B is
    /// available instead of stream input 9-16 in IEEE 1384 bus.
    pub expansion_mode: ExpansionMode,
}

impl FStudioParametersSerdes<MixerParameters> for FStudioProtocol {
    const NAME: &'static str = "mixer-parameters";

    const OFFSET_RANGES: &'static [Range<usize>] = &[
        Range {
            start: MIXER_PHYS_SRC_PARAMS_OFFSET,
            end: MIXER_PHYS_SRC_PARAMS_OFFSET + MIXER_PHYS_SRC_PARAMS_SIZE
        },
        Range {
            start: MIXER_STREAM_SRC_PARAMS_OFFSET,
            end: MIXER_STREAM_SRC_PARAMS_OFFSET + MIXER_STREAM_SRC_PARAMS_SIZE,
        },
        Range {
            start: MIXER_SELECTABLE_SRC_PARAMS_OFFSET,
            end: MIXER_SELECTABLE_SRC_PARAMS_OFFSET + MIXER_SELECTABLE_SRC_PARAMS_SIZE,
        },
        Range {
            start: MIXER_OUTPUT_PARAMS_OFFSET,
            end: MIXER_OUTPUT_PARAMS_OFFSET + MIXER_OUTPUT_PARAMS_SIZE
        },
        Range {
            start: MIXER_EXPANSION_MODE_OFFSET,
            end: MIXER_EXPANSION_MODE_OFFSET + MIXER_EXPANSION_MODE_SIZE,
        },
        Range {
            start: MIXER_SRC_LINK_OFFSET,
            end: MIXER_SRC_LINK_OFFSET + MIXER_SRC_LINK_SIZE,
        },
    ];

    fn serialize_params(params: &MixerParameters, raw: &mut [u8]) -> Result<(), String> {
        params
            .sources
            .iter()
            .enumerate()
            .try_for_each(|(i, srcs)| {
                srcs.analog_pairs
                    .iter()
                    .chain(srcs.adat_0_pairs.iter())
                    .chain(srcs.spdif_pairs.iter())
                    .enumerate()
                    .try_for_each(|(j, pair)| {
                        let pos = 4 * (3 * (i * 18 + j * 2));
                        serialize_mixer_source_pair(pair, &mut raw[pos..(pos + 24)])
                    })?;

                srcs.stream_pairs
                    .iter()
                    .enumerate()
                    .try_for_each(|(j, pair)| {
                        let pos = 4 * (3 * (9 * 18 + i * 10 + j * 2));
                        serialize_mixer_source_pair(pair, &mut raw[pos..(pos + 24)])
                    })?;

                srcs.selectable_pairs
                    .iter()
                    .enumerate()
                    .try_for_each(|(j, pair)| {
                        let pos = 4 * (3 * (9 * 28 + i * 8 + j * 2));
                        serialize_mixer_source_pair(pair, &mut raw[pos..(pos + 24)])
                    })
            })?;

        params
            .outputs
            .iter()
            .enumerate()
            .try_for_each(|(i, pair)| {
                let pos = 4 * 3 * (9 * 36 + i);
                serialize_mixer_output_pair(pair, &mut raw[pos..(pos + 12)])
            })?;

        let pos = 4 * (3 * 9 * 36 + 3 * 9);
        serialize_expansion_mode(&params.expansion_mode, &mut raw[pos..(pos + 4)])?;

        params.sources.iter().enumerate().for_each(|(i, srcs)| {
            let mut val = 0u32;

            srcs.analog_pairs
                .iter()
                .chain(srcs.adat_0_pairs.iter())
                .chain(srcs.spdif_pairs.iter())
                .enumerate()
                .filter(|(_, pair)| pair.link)
                .for_each(|(j, _)| val |= 1 << j);

            srcs.stream_pairs
                .iter()
                .chain(srcs.selectable_pairs.iter())
                .enumerate()
                .filter(|(_, pair)| pair.link)
                .for_each(|(j, _)| val |= 1 << (16 + j));

            let pos = 4 * (3 * 9 * 36 + 3 * 9 + 1 + i);
            raw[pos..(pos + 4)].copy_from_slice(&val.to_be_bytes());
        });

        Ok(())
    }

    fn deserialize_params(params: &mut MixerParameters, raw: &[u8]) -> Result<(), String> {
        params
            .sources
            .iter_mut()
            .enumerate()
            .try_for_each(|(i, srcs)| {
                srcs.analog_pairs
                    .iter_mut()
                    .chain(srcs.adat_0_pairs.iter_mut())
                    .chain(srcs.spdif_pairs.iter_mut())
                    .enumerate()
                    .try_for_each(|(j, pair)| {
                        let pos = 4 * (3 * (i * 18 + j * 2));
                        deserialize_mixer_source_pair(pair, &raw[pos..(pos + 24)])
                    })?;

                srcs.stream_pairs
                    .iter_mut()
                    .enumerate()
                    .try_for_each(|(j, pair)| {
                        let pos = 4 * (3 * (9 * 18 + i * 10 + j * 2));
                        deserialize_mixer_source_pair(pair, &raw[pos..(pos + 24)])
                    })?;

                srcs.selectable_pairs
                    .iter_mut()
                    .enumerate()
                    .try_for_each(|(j, pair)| {
                        let pos = 4 * (3 * (9 * 28 + i * 8 + j * 2));
                        deserialize_mixer_source_pair(pair, &raw[pos..(pos + 24)])
                    })
            })?;

        params
            .outputs
            .iter_mut()
            .enumerate()
            .try_for_each(|(i, pair)| {
                let pos = 4 * 3 * (9 * 36 + i);
                deserialize_mixer_output_pair(pair, &raw[pos..(pos + 12)])
            })?;

        let pos = 4 * (3 * 9 * 36 + 3 * 9);
        deserialize_expansion_mode(&mut params.expansion_mode, &raw[pos..(pos + 4)])?;

        let mut quadlet = [0; 4];
        params.sources.iter_mut().enumerate().for_each(|(i, srcs)| {
            let pos = 4 * (3 * 9 * 36 + 3 * 9 + 1 + i);
            quadlet.copy_from_slice(&raw[pos..(pos + 4)]);
            let val = u32::from_be_bytes(quadlet);

            srcs.analog_pairs
                .iter_mut()
                .chain(srcs.adat_0_pairs.iter_mut())
                .chain(srcs.spdif_pairs.iter_mut())
                .enumerate()
                .for_each(|(j, pair)| pair.link = val & (1 << j) > 0);

            srcs.stream_pairs
                .iter_mut()
                .chain(srcs.selectable_pairs.iter_mut())
                .enumerate()
                .for_each(|(j, pair)| pair.link = val & (1 << (16 + j)) > 0);
        });

        Ok(())
    }
}

impl<O: FStudioOperation + FStudioParametersSerdes<MixerParameters>>
    FStudioMutableParametersOperation<MixerParameters> for O
{
}

/// The number of mixers.
pub const MIXER_COUNT: usize = 9;

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn meter_params_serdes() {
        let target = FStudioMeter {
            analog_inputs: [0, 1, 2, 3, 4, 5, 6, 7],
            stream_inputs: [17, 16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0],
            mixer_outputs: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17],
        };

        let size = compute_params_size(
            <FStudioProtocol as FStudioParametersSerdes<FStudioMeter>>::OFFSET_RANGES,
        );
        let mut raw = vec![0; size];
        assert!(FStudioProtocol::serialize_params(&target, &mut raw).is_ok());

        let mut params = FStudioMeter::default();
        assert!(FStudioProtocol::deserialize_params(&mut params, &raw).is_ok());

        assert_eq!(target, params);
    }

    #[test]
    fn output_params_serdes() {
        let target = OutputParameters {
            pairs: [Default::default(); 9],
            main_assign: AssignTarget::AdatA67,
            headphone_assigns: [
                AssignTarget::Analog78,
                AssignTarget::Spdif01,
                AssignTarget::AdatA45,
            ],
            bnc_terminate: true,
        };

        let size = compute_params_size(
            <FStudioProtocol as FStudioParametersSerdes<OutputParameters>>::OFFSET_RANGES,
        );
        let mut raw = vec![0; size];
        assert!(FStudioProtocol::serialize_params(&target, &mut raw).is_ok());

        let mut params = OutputParameters::default();
        assert!(FStudioProtocol::deserialize_params(&mut params, &raw).is_ok());

        assert_eq!(target, params, "{:02x?}", raw);
    }

    #[test]
    fn mixer_params_serdes() {
        let mut target = MixerParameters::default();
        target.sources.iter_mut().enumerate().for_each(|(i, srcs)| {
            srcs.analog_pairs
                .iter_mut()
                .chain(srcs.adat_0_pairs.iter_mut())
                .chain(srcs.spdif_pairs.iter_mut())
                .chain(srcs.stream_pairs.iter_mut())
                .chain(srcs.selectable_pairs.iter_mut())
                .enumerate()
                .for_each(|(j, pair)| {
                    pair.gains[0] = (i * 9 + j * 3) as u8;
                    pair.gains[1] = (i * 11 + j * 1) as u8;
                    pair.balances[0] = (i * 7 + j * 5) as u8;
                    pair.balances[1] = (i * 5 + j * 7) as u8;
                    pair.mutes[0] = (i + j * 2) % 2 > 0;
                    pair.mutes[1] = (i * 2 + j) % 2 > 0;
                    pair.link = (i + j) % 2 > 0;
                });
        });
        target.outputs.iter_mut().enumerate().for_each(|(i, pair)| {
            pair.volume = 3 * i as u8;
            pair.mute = i % 2 > 0;
        });
        target.expansion_mode = ExpansionMode::AdatB0_7;

        let size = compute_params_size(
            <FStudioProtocol as FStudioParametersSerdes<MixerParameters>>::OFFSET_RANGES,
        );
        let mut raw = vec![0; size];
        assert!(FStudioProtocol::serialize_params(&target, &mut raw).is_ok());

        let mut params = MixerParameters::default();
        assert!(FStudioProtocol::deserialize_params(&mut params, &raw).is_ok());

        assert_eq!(target, params);
    }
}
