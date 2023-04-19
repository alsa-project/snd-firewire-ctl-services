// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Protocol specific to Alesis iO FireWire series.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by Alesis for iO FireWire series.
//!
//! ## Diagram of internal signal flow for iO 14 FireWire.
//!
//! ```text
//!
//! analog-input-1/2 ----------------------> stream-output-A-1/2
//! analog-input-3/4 ----------------------> stream-output-A-3/4
//! spdif-input-1/2 -----------------------> stream-output-A-5/6
//! adat-input-1/2 ------------------------> stream-output-B-1/2
//! adat-input-3/4 ------------------------> stream-output-B-3/4
//! adat-input-5/6 ------------------------> stream-output-B-5/6
//! adat-input-7/8 ------------------------> stream-output-B-7/8
//!
//!                        ++=========++
//! analog-input-1/2 ----> ||         ||
//! analog-input-3/4 ----> || 14 x 8  || --> monitor-output-1/2
//! adat-input-1/2 ------> ||         || --> monitor-output-3/4
//! adat-input-3/4 ------> || monitor || --> monitor-output-5/6
//! adat-input-5/6 ------> ||         || --> monitor-output-7/8
//! adat-input-7/8 ------> ||  mixer  ||
//! spdif-input-1/2 -----> ||         ||
//!                        ++=========++
//!
//!                        ++=========++
//! monitor-output-1/2 --> ||  4 x 2  || --> mixer-output-1/2
//! stream-input-1/2 ----> ||  mixer  ||
//!                        ++=========++
//!
//!                        ++=========++
//! monitor-output-3/4 --> ||  4 x 2  || --> mixer-output-3/4
//! stream-input-3/4 ----> ||  mixer  ||
//!                        ++=========++
//!
//!                        ++=========++
//! monitor-output-5/6 --> ||  4 x 2  || --> mixer-output-5/6
//! stream-input-5/6 ----> ||  mixer  ||
//!                        ++=========++
//!
//!                        ++=========++
//! monitor-output-7/8 --> ||  4 x 2  || --> mixer-output-7/8
//! stream-input-7/8 ----> ||  mixer  ||
//!                        ++=========++
//!
//! mixer-output-1/2 -----------+----------> analog-output-1/2
//!                             +----------> headphone-output-1/2
//! mixer-output-3/4 ----------------------> analog-output-3/4
//! mixer-output-5/6 ----------------------> analog-output-5/6
//! mixer-output-7/8 ----------------------> analog-output-7/8
//!
//! mixer-output-1/2 -------(one of)------> headphone-output-3/4
//! mixer-output-3/4 -----------+
//! mixer-output-5/6 -----------+
//! mixer-output-7/8 -----------+
//!
//! mixer-output-1/2 -------(one of)-------> spdif-output-1/2
//! mixer-output-3/4 -----------+
//! mixer-output-5/6 -----------+
//! mixer-output-7/8 -----------+
//! ```
//!
//! ## Diagram of internal signal flow for iO 26 FireWire.
//!
//! ```text
//!
//! analog-input-1/2 ----------------------> stream-output-A-1/2
//! analog-input-3/4 ----------------------> stream-output-A-3/4
//! analog-input-5/6 ----------------------> stream-output-A-5/6
//! analog-input-7/8 ----------------------> stream-output-A-7/8
//! spdif-input-1/2 -----------------------> stream-output-A-9/10
//! adat-input-1/2 ------------------------> stream-output-B-1/2
//! adat-input-3/4 ------------------------> stream-output-B-3/4
//! adat-input-5/6 ------------------------> stream-output-B-5/6
//! adat-input-7/8 ------------------------> stream-output-B-7/8
//! adat-input-9/10 -----------------------> stream-output-B-9/10
//! adat-input-11/12 ----------------------> stream-output-B-11/12
//! adat-input-13/14 ----------------------> stream-output-B-13/14
//! adat-input-15/16 ----------------------> stream-output-B-15/16
//!
//!                        ++=========++
//! analog-input-1/2 ----> ||         ||
//! analog-input-3/4 ----> ||         ||
//! analog-input-5/6 ----> ||         ||
//! analog-input-7/8 ----> || 24 x 8  ||
//! adat-input-1/2 ------> ||         || --> monitor-output-1/2
//! adat-input-3/4 ------> || monitor || --> monitor-output-3/4
//! adat-input-5/6 ------> ||         || --> monitor-output-5/6
//! adat-input-7/8 ------> ||  mixer  || --> monitor-output-7/8
//! adat-input-9/10 -----> ||         ||
//! adat-input-11/12 ----> ||         ||
//! adat-input-13/14 ----> ||         ||
//! adat-input-15/16 -or-> ||         ||
//! spdif-input-1/2 --+    ||         ||
//!                        ++=========++
//!
//!                        ++=========++
//! monitor-output-1/2 --> ||  4 x 2  || --> mixer-output-1/2
//! stream-input-1/2 ----> ||  mixer  ||
//!                        ++=========++
//!
//!                        ++=========++
//! monitor-output-3/4 --> ||  4 x 2  || --> mixer-output-3/4
//! stream-input-3/4 ----> ||  mixer  ||
//!                        ++=========++
//!
//!                        ++=========++
//! monitor-output-5/6 --> ||  4 x 2  || --> mixer-output-5/6
//! stream-input-5/6 ----> ||  mixer  ||
//!                        ++=========++
//!
//!                        ++=========++
//! monitor-output-7/8 --> ||  4 x 2  || --> mixer-output-7/8
//! stream-input-7/8 ----> ||  mixer  ||
//!                        ++=========++
//!
//! mixer-output-1/2 -----------+----------> analog-output-1/2
//!                             +----------> headphone-output-1/2
//! mixer-output-3/4 ----------------------> analog-output-3/4
//! mixer-output-5/6 ----------------------> analog-output-5/6
//! mixer-output-7/8 ----------------------> analog-output-7/8
//!
//! mixer-output-1/2 -------(one of)------> headphone-output-3/4
//! mixer-output-3/4 -----------+
//! mixer-output-5/6 -----------+
//! mixer-output-7/8 -----------+
//!
//! mixer-output-1/2 -------(one of)-------> spdif-output-1/2
//! mixer-output-3/4 -----------+
//! mixer-output-5/6 -----------+
//! mixer-output-7/8 -----------+
//! ```

use {
    super::{tcat::*, *},
    std::ops::Range,
};

/// Protocol implementation specific to iO 14 FireWire.
#[derive(Default, Debug)]
pub struct Io14fwProtocol;

impl TcatOperation for Io14fwProtocol {}

impl TcatGlobalSectionSpecification for Io14fwProtocol {}

impl AlesisOperation for Io14fwProtocol {}

impl IofwMeterSpecification for Io14fwProtocol {
    const ANALOG_INPUT_COUNT: usize = 4;
    const DIGITAL_B_INPUT_COUNT: usize = 2;
}

impl IofwOutputSpecification for Io14fwProtocol {
    const ANALOG_OUTPUT_COUNT: usize = 4;
    const HAS_OPT_IFACE_B: bool = false;
}

impl IofwMixerSpecification for Io14fwProtocol {
    const ANALOG_INPUT_PAIR_COUNT: usize = 2;
    const DIGITAL_B_INPUT_PAIR_COUNT: usize = 1;
}

/// Protocol implementation specific to iO 26 FireWire.
#[derive(Default, Debug)]
pub struct Io26fwProtocol;

impl TcatOperation for Io26fwProtocol {}

impl TcatGlobalSectionSpecification for Io26fwProtocol {}

impl AlesisOperation for Io26fwProtocol {}

impl IofwMeterSpecification for Io26fwProtocol {
    const ANALOG_INPUT_COUNT: usize = 8;
    const DIGITAL_B_INPUT_COUNT: usize = 8;
}

impl IofwOutputSpecification for Io26fwProtocol {
    const ANALOG_OUTPUT_COUNT: usize = 8;
    const HAS_OPT_IFACE_B: bool = true;
}

impl IofwMixerSpecification for Io26fwProtocol {
    const ANALOG_INPUT_PAIR_COUNT: usize = 4;
    const DIGITAL_B_INPUT_PAIR_COUNT: usize = 4;
}

const BASE_OFFSET: usize = 0x00200000;

const MIXER_PARAMS_OFFSET: usize = 0x0038;
// const MIXER_PAIR_SOURCE_GAIN_OFFSET: usize = 0x0038;
const MIXER_OUTPUT_VOLUME_OFFSET: usize = 0x0438;
// const MIXER_PAIR_SOURCE_MUTE_OFFSET: usize = 0x0458;
// const MIXER_OUTPUT_MUTE_OFFSET: usize = 0x0468;
// const MIXER_PAIR_SOURCE_SOLO_OFFSET: usize = 0x046c;
// const MIXER_PAIR_SOURCE_LINK_OFFSET: usize = 0x047c;

const METER_OFFSET: usize = 0x04c0;
// NOTE: 0: mixer 0/1, 1: mixer 2/3, 2: mixer 4/5, 3: mixer 6/7, 4: meter.
// const UI_SELECT_OFFSET: usize = 0x0560;
const OUT_LEVEL_OFFSET: usize = 0x0564;
// const MIXER_DIGITAL_B_67_SRC_OFFSET: usize = 0x0568;
// const SPDIF_OUT_SRC_OFFSET: usize = 0x056c;
// const HP34_SRC_OFFSET: usize = 0x0570;

const KNOB_PARAMS_OFFSET: usize = 0x0574;
// const MIXER_BLEND_KNOB_OFFSET: usize = 0x0574;
// const MIXER_MASTER_KNOB_OFFSET: usize = 0x0578;

const METER_SIZE: usize = 160;

const MIXER_PARAMS_SIZE: usize = 0x454;
// const MIXER_PAIR_SOURCE_GAIN_SIZE: usize = 4 * 4 * (8 + 8 + 8 + 8);
// const MIXER_OUTPUT_VOLUME_SIZE: usize = 4 * 8;
// const MIXER_PAIR_SOURCE_MUTE_SIZE: usize = 4 * 4;
// const MIXER_OUTPUT_MUTE_SIZE: usize = 4;
// const MIXER_PAIR_SOURCE_SOLO_SIZE: usize = 4 * 4;
// const MIXER_PAIR_SOURCE_LINK_SIZE: usize = 4 * 4;
const KNOB_PARAMS_SIZE: usize = 8;
// const MIXER_BLEND_KNOB_SIZE: usize = 4;
// const MIXER_MASTER_KNOB_SIZE: usize = 4;

/// Serialize and deserialize for parameters of iO FireWire series.
pub trait AlesisParametersSerdes<T> {
    /// The name of parameters
    const NAME: &'static str;

    /// The range of offset for parameters.
    const OFFSET_RANGES: &'static [Range<usize>];

    /// Serialize parameters to raw layout of data.
    fn serialize_params(params: &T, raw: &mut [u8]) -> Result<(), String>;

    /// Deserialize parameters from raw layout of data.
    fn deserialize_params(params: &mut T, raw: &[u8]) -> Result<(), String>;
}

/// Specification for hardware meter.
pub trait IofwMeterSpecification {
    /// The number of analog inputs.
    const ANALOG_INPUT_COUNT: usize;

    /// The number of digital B inputs.
    const DIGITAL_B_INPUT_COUNT: usize;

    /// The number of stream inputs.
    const STREAM_INPUT_COUNT: usize = 8;

    /// The number of digital A inputs.
    const DIGITAL_A_INPUT_COUNT: usize = 8;

    /// The number of mixer outputs.
    const MIXER_OUTPUT_COUNT: usize = 8;

    /// The minimum value of detected signal level.
    const LEVEL_MIN: i32 = 0;

    /// The maximum value of detected signal level.
    const LEVEL_MAX: i32 = i16::MAX as i32;

    /// Instantiate state of meters.
    fn create_meter_params() -> IofwMeterParams {
        IofwMeterParams {
            analog_inputs: vec![0; Self::ANALOG_INPUT_COUNT],
            stream_inputs: [0; 8],
            digital_a_inputs: [0; 8],
            digital_b_inputs: vec![0; Self::DIGITAL_B_INPUT_COUNT],
            mixer_outputs: [0; 8],
        }
    }
}

/// Specification of outputs.
pub trait IofwOutputSpecification {
    /// The number of analog outputs.
    const ANALOG_OUTPUT_COUNT: usize;

    /// Whether optical interface B is available or not.
    const HAS_OPT_IFACE_B: bool;

    /// Instantiate output parameters.
    fn create_output_params() -> IofwOutputParams {
        IofwOutputParams {
            nominal_levels: vec![Default::default(); Self::ANALOG_OUTPUT_COUNT],
            digital_67_src: Default::default(),
            spdif_out_src: Default::default(),
            headphone2_3_out_src: Default::default(),
        }
    }
}

impl<O> AlesisMutableParametersOperation<IofwOutputParams> for O where
    O: AlesisOperation + IofwOutputSpecification + AlesisParametersSerdes<IofwOutputParams>
{
}

/// Specification of mixers.
pub trait IofwMixerSpecification {
    /// The number of analog input pairs.
    const ANALOG_INPUT_PAIR_COUNT: usize;

    /// The number of digital input B pairs.
    const DIGITAL_B_INPUT_PAIR_COUNT: usize;

    /// The number of stream input pairs.
    const STREAM_INPUT_PAIR_COUNT: usize = 4;

    /// The number of digital input A pairs.
    const DIGITAL_A_INPUT_PAIR_COUNT: usize = 4;

    /// The number of mixer output pairs.
    const MIXER_OUTPUT_PAIR_COUNT: usize = 4;

    /// The minimum value of gain.
    const GAIN_MIN: i32 = 0;

    /// The maximum value of gain.
    const GAIN_MAX: i32 = 0x007fffff;

    /// The minimum value of volume, as well as minimum value of knob.
    const VOLUME_MIN: u32 = 0;

    /// The maximum value of volume, as well as maximum value of knob.
    const VOLUME_MAX: u32 = 0x100;

    /// Instantiate mixer parameters.
    fn create_mixer_params() -> IofwMixerParams {
        IofwMixerParams {
            mixer_pairs: [
                IofwMixerPair {
                    monitor_pair: IofwMonitorPair {
                        analog_input_pairs: vec![Default::default(); Self::ANALOG_INPUT_PAIR_COUNT],
                        digital_a_input_pairs: [Default::default(); 4],
                        digital_b_input_pairs: vec![
                            Default::default();
                            Self::DIGITAL_B_INPUT_PAIR_COUNT
                        ],
                        output_volumes: [Default::default(); 2],
                        output_mutes: [Default::default(); 2],
                    },
                    stream_inputs_to_left: [Default::default(); 8],
                    stream_inputs_to_right: [Default::default(); 8],
                },
                IofwMixerPair {
                    monitor_pair: IofwMonitorPair {
                        analog_input_pairs: vec![Default::default(); Self::ANALOG_INPUT_PAIR_COUNT],
                        digital_a_input_pairs: [Default::default(); 4],
                        digital_b_input_pairs: vec![
                            Default::default();
                            Self::DIGITAL_B_INPUT_PAIR_COUNT
                        ],
                        output_volumes: [Default::default(); 2],
                        output_mutes: [Default::default(); 2],
                    },
                    stream_inputs_to_left: [Default::default(); 8],
                    stream_inputs_to_right: [Default::default(); 8],
                },
                IofwMixerPair {
                    monitor_pair: IofwMonitorPair {
                        analog_input_pairs: vec![Default::default(); Self::ANALOG_INPUT_PAIR_COUNT],
                        digital_a_input_pairs: [Default::default(); 4],
                        digital_b_input_pairs: vec![
                            Default::default();
                            Self::DIGITAL_B_INPUT_PAIR_COUNT
                        ],
                        output_volumes: [Default::default(); 2],
                        output_mutes: [Default::default(); 2],
                    },
                    stream_inputs_to_left: [Default::default(); 8],
                    stream_inputs_to_right: [Default::default(); 8],
                },
                IofwMixerPair {
                    monitor_pair: IofwMonitorPair {
                        analog_input_pairs: vec![Default::default(); Self::ANALOG_INPUT_PAIR_COUNT],
                        digital_a_input_pairs: [Default::default(); 4],
                        digital_b_input_pairs: vec![
                            Default::default();
                            Self::DIGITAL_B_INPUT_PAIR_COUNT
                        ],
                        output_volumes: [Default::default(); 2],
                        output_mutes: [Default::default(); 2],
                    },
                    stream_inputs_to_left: [Default::default(); 8],
                    stream_inputs_to_right: [Default::default(); 8],
                },
            ],
            master_knob: Default::default(),
            blend_knob: Default::default(),
        }
    }
}

impl<O> AlesisMutableParametersOperation<IofwMixerParams> for O where
    O: AlesisOperation + IofwMixerSpecification + AlesisParametersSerdes<IofwMixerParams>
{
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

/// Operation for Alesis iO FireWire series.
pub trait AlesisOperation: TcatOperation {
    /// Read from specific range of address.
    fn read_params(
        req: &FwReq,
        node: &FwNode,
        offset: usize,
        raw: &mut [u8],
        timeout_ms: u32,
    ) -> Result<(), Error> {
        Self::read(req, node, BASE_OFFSET + offset, raw, timeout_ms)
    }

    /// Write to specific range of address.
    fn write_params(
        req: &FwReq,
        node: &FwNode,
        offset: usize,
        raw: &mut [u8],
        timeout_ms: u32,
    ) -> Result<(), Error> {
        Self::write(req, node, BASE_OFFSET + offset, raw, timeout_ms)
    }
}

/// Operation for parameters to cache state of hardware.
pub trait AlesisParametersOperation<T>: AlesisOperation + AlesisParametersSerdes<T> {
    /// Cache whole segment and deserialize for parameters.
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
            Self::read_params(
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

impl<O: AlesisOperation + AlesisParametersSerdes<T>, T> AlesisParametersOperation<T> for O {}

/// Operation for parameters to update state of hardware.
pub trait AlesisMutableParametersOperation<T>: AlesisOperation + AlesisParametersSerdes<T> {
    /// Update the hardware partially for any change of parameter.
    fn update_partial_params(
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
                        Self::write_params(
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

/// Operation for parameters to include fluctuated values.
pub trait AlesisFluctuatedParametersOperation<T>:
    AlesisOperation + AlesisParametersSerdes<T>
{
    /// The set of address offsets in which any value is changed apart from software operation.
    const FLUCTUATED_OFFSET_RANGES: &'static [Range<usize>];

    /// Cache part of offset ranges for fluctuated values, then deserialize for parameters.
    fn cache_partial_params(
        req: &FwReq,
        node: &FwNode,
        params: &mut T,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let size = compute_params_size(Self::OFFSET_RANGES);

        let mut raw = vec![0u8; size];
        Self::serialize_params(params, &mut raw)
            .map_err(|cause| generate_err(Self::NAME, &cause, &raw))?;

        Self::FLUCTUATED_OFFSET_RANGES
            .iter()
            .try_for_each(|range| {
                let mut pos = 0;
                for r in Self::OFFSET_RANGES {
                    if !r.contains(&range.start) {
                        pos += r.end - r.start;
                    } else {
                        pos += range.start - r.start;
                        break;
                    }
                }
                assert!(
                    pos < size,
                    "Programming error. The offset range should be found."
                );

                let end = pos + range.end - range.start;
                Self::read_params(req, node, range.start, &mut raw[pos..end], timeout_ms)
            })
            .and_then(|_| {
                Self::deserialize_params(params, &raw)
                    .map_err(|cause| generate_err(Self::NAME, &cause, &raw))
            })
    }
}

/// For hardware meters, between 0..0x7fff (-90.0..0.0 dB).
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct IofwMeterParams {
    /// Detected levels for analog inputs.
    pub analog_inputs: Vec<i16>,
    /// Detected levels for stream inputs.
    pub stream_inputs: [i16; 8],
    /// Detected levels for digital A inputs.
    pub digital_a_inputs: [i16; 8],
    /// Detected levels for digital B inputs.
    pub digital_b_inputs: Vec<i16>,
    /// Detected levels for mixer outputs.
    pub mixer_outputs: [i16; 8],
}

impl<O: IofwMeterSpecification> AlesisParametersSerdes<IofwMeterParams> for O {
    const NAME: &'static str = "meter";

    const OFFSET_RANGES: &'static [Range<usize>] = &[Range {
        start: METER_OFFSET,
        end: METER_OFFSET + METER_SIZE,
    }];

    fn serialize_params(params: &IofwMeterParams, raw: &mut [u8]) -> Result<(), String> {
        [
            (&params.analog_inputs[..], 0),
            (&params.stream_inputs[..], 32),
            (&params.digital_a_inputs[..], 64),
            (&params.mixer_outputs[..], 128),
        ]
        .iter()
        .for_each(|(levels, offset)| {
            levels.iter().enumerate().for_each(|(i, &level)| {
                let pos = *offset + i * 4;
                let val = (level as i32) << 8;
                raw[pos..(pos + 4)].copy_from_slice(&val.to_be_bytes());
            });
        });

        params
            .digital_b_inputs
            .iter()
            .rev()
            .enumerate()
            .for_each(|(i, &level)| {
                let pos = 96 + (7 - i) * 4;
                let val = (level as i32) << 8;
                raw[pos..(pos + 4)].copy_from_slice(&val.to_be_bytes());
            });

        Ok(())
    }

    fn deserialize_params(params: &mut IofwMeterParams, raw: &[u8]) -> Result<(), String> {
        [
            (&mut params.analog_inputs[..], 0),
            (&mut params.stream_inputs[..], 32),
            (&mut params.digital_a_inputs[..], 64),
            (&mut params.mixer_outputs[..], 128),
        ]
        .iter_mut()
        .for_each(|(levels, offset)| {
            levels.iter_mut().enumerate().for_each(|(i, level)| {
                let pos = *offset + i * 4;
                let mut val = 0i32;
                deserialize_i32(&mut val, &raw[pos..(pos + 4)]);
                *level = ((val & 0x00ffff00) >> 8) as i16;
            });
        });

        params
            .digital_b_inputs
            .iter_mut()
            .rev()
            .enumerate()
            .for_each(|(i, level)| {
                let pos = 96 + (7 - i) * 4;
                let mut val = 0i32;
                deserialize_i32(&mut val, &raw[pos..(pos + 4)]);
                *level = ((val & 0x00ffff00) >> 8) as i16;
            });

        Ok(())
    }
}

/// Parameters of output.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct IofwOutputParams {
    /// Nominal signal level of outputs.
    pub nominal_levels: Vec<NominalSignalLevel>,
    /// Source of digital output 7/8.
    pub digital_67_src: DigitalB67Src,
    /// Source of S/PDIF output 1/2.
    pub spdif_out_src: MixerOutPair,
    /// Source of headphone output 3/4.
    pub headphone2_3_out_src: MixerOutPair,
}

/// Nominal level of signal.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum NominalSignalLevel {
    /// -10dBV.
    Consumer,
    /// +4dBu.
    Professional,
}

impl Default for NominalSignalLevel {
    fn default() -> Self {
        NominalSignalLevel::Consumer
    }
}

fn serialize_nominal_signal_levels(
    levels: &[NominalSignalLevel],
    raw: &mut [u8],
) -> Result<(), String> {
    assert!(raw.len() >= 4);

    let val = levels
        .iter()
        .enumerate()
        .filter(|(_, &level)| level == NominalSignalLevel::Professional)
        .fold(0u32, |val, (i, _)| val | (1 << i));

    serialize_u32(&val, raw);

    Ok(())
}

fn deserialize_nominal_signal_levels(
    levels: &mut [NominalSignalLevel],
    raw: &[u8],
) -> Result<(), String> {
    assert!(raw.len() >= 4);

    let mut val = 0u32;
    deserialize_u32(&mut val, raw);

    levels.iter_mut().enumerate().for_each(|(i, level)| {
        *level = if val & (1 << i) > 0 {
            NominalSignalLevel::Professional
        } else {
            NominalSignalLevel::Consumer
        };
    });

    Ok(())
}

/// Source of 6/7 channels of digital B input.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DigitalB67Src {
    /// S/PDIF input 1/2.
    Spdif12,
    /// ADAT input B 7/8.
    Adat67,
}

impl Default for DigitalB67Src {
    fn default() -> Self {
        Self::Spdif12
    }
}

fn serialize_digital_b67_src(src: &DigitalB67Src, raw: &mut [u8]) -> Result<(), String> {
    assert!(raw.len() >= 4);

    let val = match src {
        DigitalB67Src::Spdif12 => 0,
        DigitalB67Src::Adat67 => 1,
    };
    serialize_u32(&val, raw);

    Ok(())
}

fn deserialize_digital_b67_src(src: &mut DigitalB67Src, raw: &[u8]) -> Result<(), String> {
    assert!(raw.len() >= 4);

    let mut val = 0u32;
    deserialize_u32(&mut val, raw);

    *src = match val {
        0 => DigitalB67Src::Spdif12,
        1 => DigitalB67Src::Adat67,
        _ => Err(format!("Digital B 7/8 source not found for value: {}", val))?,
    };

    Ok(())
}

/// Pair of mixer output.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MixerOutPair {
    /// Mixer output 1/2.
    Mixer01,
    /// Mixer output 3/4.
    Mixer23,
    /// Mixer output 5/6.
    Mixer45,
    /// Mixer output 7/8.
    Mixer67,
}

impl Default for MixerOutPair {
    fn default() -> Self {
        Self::Mixer01
    }
}

fn serialize_mixer_out_pair(pair: &MixerOutPair, raw: &mut [u8]) -> Result<(), String> {
    assert!(raw.len() >= 4);

    let val = match pair {
        MixerOutPair::Mixer01 => 0,
        MixerOutPair::Mixer23 => 1,
        MixerOutPair::Mixer45 => 2,
        MixerOutPair::Mixer67 => 3,
    };
    serialize_u32(&val, raw);

    Ok(())
}

fn deserialize_mixer_out_pair(pair: &mut MixerOutPair, raw: &[u8]) -> Result<(), String> {
    assert!(raw.len() >= 4);

    let mut val = 0u32;
    deserialize_u32(&mut val, raw);

    *pair = match val {
        0 => MixerOutPair::Mixer01,
        1 => MixerOutPair::Mixer23,
        2 => MixerOutPair::Mixer45,
        3 => MixerOutPair::Mixer67,
        _ => Err(format!("Mixer output pair not found for value: {}", val))?,
    };

    Ok(())
}

impl<O: IofwOutputSpecification> AlesisParametersSerdes<IofwOutputParams> for O {
    const NAME: &'static str = "output-params";

    const OFFSET_RANGES: &'static [Range<usize>] = &[Range {
        start: OUT_LEVEL_OFFSET,
        end: OUT_LEVEL_OFFSET + 16,
    }];

    fn serialize_params(params: &IofwOutputParams, raw: &mut [u8]) -> Result<(), String> {
        serialize_nominal_signal_levels(&params.nominal_levels, &mut raw[..4])?;
        serialize_digital_b67_src(&params.digital_67_src, &mut raw[4..8])?;
        serialize_mixer_out_pair(&params.spdif_out_src, &mut raw[8..12])?;
        serialize_mixer_out_pair(&params.headphone2_3_out_src, &mut raw[12..16])?;
        Ok(())
    }

    fn deserialize_params(params: &mut IofwOutputParams, raw: &[u8]) -> Result<(), String> {
        deserialize_nominal_signal_levels(&mut params.nominal_levels, &raw[..4])?;
        deserialize_digital_b67_src(&mut params.digital_67_src, &raw[4..8])?;
        deserialize_mixer_out_pair(&mut params.spdif_out_src, &raw[8..12])?;
        deserialize_mixer_out_pair(&mut params.headphone2_3_out_src, &raw[12..16])?;
        Ok(())
    }
}

/// Parameters for pair of sources of paired mixer.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct IofwMonitorPairSourcePair {
    /// Gain from left and right channels to left channel of monitor, between 0x7fffff and 0 (-60
    /// and 0 dB).
    pub gain_to_left: [i32; 2],
    /// Gain from left and right channels to right channel of monitor, between 0x7fffff and 0 (-60
    /// and 0 dB).
    pub gain_to_right: [i32; 2],
    /// Whether to mute left and right channels.
    pub mutes: [bool; 2],
    /// Whether to mute the other channels.
    pub solos: [bool; 2],
    /// Whether to link left and right channels.
    pub link: bool,
}

/// Parameters of source pairs for monitor. The function to control volume of monitor outputs
/// seems not to be available in all of models with the latest firmware, while vendor's GUI
/// application still operates them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IofwMonitorPair {
    /// Source pairs of analog inputs.
    pub analog_input_pairs: Vec<IofwMonitorPairSourcePair>,
    /// Source pairs of digital A inputs.
    pub digital_a_input_pairs: [IofwMonitorPairSourcePair; 4],
    /// Source pairs of digital B inputs.
    pub digital_b_input_pairs: Vec<IofwMonitorPairSourcePair>,
    /// Volume of left and right outputs, between 0 and 0x100 (-60 and 0 dB).
    pub output_volumes: [u32; 2],
    /// Mute of left and right outputs.
    pub output_mutes: [bool; 2],
}

/// Parameters of source pairs for mixer. The function to control gain of stream inputs seems not
/// to be available in iO 26 FireWire with the latest version of firmware.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IofwMixerPair {
    /// Source pairs for monitor.
    pub monitor_pair: IofwMonitorPair,
    /// Stream inputs to left channel of mixer.
    pub stream_inputs_to_left: [i32; 8],
    /// Stream inputs to right channel of mixer.
    pub stream_inputs_to_right: [i32; 8],
}

/// Parametes of source pairs for mixer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IofwMixerParams {
    /// The setting of each mixer.
    pub mixer_pairs: [IofwMixerPair; 4],
    /// The value of master knob, between 0 and 0x100.
    pub blend_knob: u32,
    /// The value of master knob, between 0 and 0x100.
    pub master_knob: u32,
}

impl<O: IofwMixerSpecification> AlesisParametersSerdes<IofwMixerParams> for O {
    const NAME: &'static str = "mixer";

    const OFFSET_RANGES: &'static [Range<usize>] = &[
        Range {
            start: MIXER_PARAMS_OFFSET,
            end: MIXER_PARAMS_OFFSET + MIXER_PARAMS_SIZE,
        },
        Range {
            start: KNOB_PARAMS_OFFSET,
            end: KNOB_PARAMS_OFFSET + KNOB_PARAMS_SIZE,
        },
    ];

    fn serialize_params(params: &IofwMixerParams, raw: &mut [u8]) -> Result<(), String> {
        params.mixer_pairs.iter().enumerate().for_each(|(i, srcs)| {
            let mut mutes_val = 0u32;
            let mut solos_val = 0u32;
            let mut links_val = 0u32;

            let digital_b_pos = 2 * (4 - Self::DIGITAL_B_INPUT_PAIR_COUNT);
            [
                (&srcs.monitor_pair.analog_input_pairs[..], 0),
                (&srcs.monitor_pair.digital_a_input_pairs[..], 16),
                (
                    &srcs.monitor_pair.digital_b_input_pairs[..],
                    24 + digital_b_pos,
                ),
            ]
            .iter()
            .for_each(|(pairs, offset)| {
                pairs
                    .iter()
                    .flat_map(|pair| pair.gain_to_left.iter())
                    .enumerate()
                    .for_each(|(j, gain)| {
                        let mixer_index = i * 2;
                        let pos = 4 * (mixer_index * (8 + 8 + 8 + 8) + *offset + j);
                        serialize_i32(gain, &mut raw[pos..(pos + 4)]);
                    });

                pairs
                    .iter()
                    .flat_map(|pair| pair.gain_to_right.iter())
                    .enumerate()
                    .for_each(|(j, gain)| {
                        let mixer_index = i * 2 + 1;
                        let pos = 4 * (mixer_index * (8 + 8 + 8 + 8) + *offset + j);
                        serialize_i32(gain, &mut raw[pos..(pos + 4)]);
                    });

                pairs
                    .iter()
                    .flat_map(|pair| pair.mutes.iter())
                    .enumerate()
                    .filter(|(_, &mute)| mute)
                    .for_each(|(j, _)| mutes_val |= 1 << (*offset + j));

                pairs
                    .iter()
                    .flat_map(|pair| pair.solos.iter())
                    .enumerate()
                    .filter(|(_, &solo)| solo)
                    .for_each(|(j, _)| solos_val |= 1 << (*offset + j));

                pairs
                    .iter()
                    .enumerate()
                    .filter(|(_, &pair)| pair.link)
                    .for_each(|(j, _)| links_val |= 1 << (*offset / 2 + j));
            });

            let pos = 0x0420 + 4 * i;
            serialize_u32(&mutes_val, &mut raw[pos..(pos + 4)]);

            let pos = 0x0434 + 4 * i;
            serialize_u32(&solos_val, &mut raw[pos..(pos + 4)]);

            let pos = 0x0444 + 4 * i;
            serialize_u32(&links_val, &mut raw[pos..(pos + 4)]);

            [
                &srcs.stream_inputs_to_left[..],
                &srcs.stream_inputs_to_right[..],
            ]
            .iter()
            .enumerate()
            .for_each(|(j, gains)| {
                gains.iter().enumerate().for_each(|(k, gain)| {
                    let mixer_index = i * 2 + j;
                    let pos = 4 * (mixer_index * (8 + 8 + 8 + 8) + 8 + k);
                    serialize_i32(gain, &mut raw[pos..(pos + 4)]);
                });
            });
        });

        params
            .mixer_pairs
            .iter()
            .flat_map(|srcs| srcs.monitor_pair.output_volumes.iter())
            .enumerate()
            .for_each(|(i, vol)| {
                let pos = 0x400 + 4 * i;
                serialize_u32(vol, &mut raw[pos..(pos + 4)]);
            });

        let mut val = 0u32;
        params
            .mixer_pairs
            .iter()
            .flat_map(|srcs| srcs.monitor_pair.output_mutes.iter())
            .enumerate()
            .filter(|(_, &mute)| mute)
            .for_each(|(i, _)| val |= 1 << i);
        serialize_u32(&val, &mut raw[0x430..0x434]);

        serialize_u32(&params.blend_knob, &mut raw[0x454..0x458]);
        serialize_u32(&params.master_knob, &mut raw[0x458..0x45c]);

        Ok(())
    }

    fn deserialize_params(params: &mut IofwMixerParams, raw: &[u8]) -> Result<(), String> {
        params
            .mixer_pairs
            .iter_mut()
            .enumerate()
            .for_each(|(i, srcs)| {
                let digital_b_pos = 2 * (4 - Self::DIGITAL_B_INPUT_PAIR_COUNT);

                let pos = 0x0420 + 4 * i;
                let mut mutes_val = 0u32;
                deserialize_u32(&mut mutes_val, &raw[pos..(pos + 4)]);

                let pos = 0x0434 + 4 * i;
                let mut solos_val = 0u32;
                deserialize_u32(&mut solos_val, &raw[pos..(pos + 4)]);

                let pos = 0x0444 + 4 * i;
                let mut links_val = 0u32;
                deserialize_u32(&mut links_val, &raw[pos..(pos + 4)]);

                [
                    (&mut srcs.monitor_pair.analog_input_pairs[..], 0),
                    (&mut srcs.monitor_pair.digital_a_input_pairs[..], 16),
                    (
                        &mut srcs.monitor_pair.digital_b_input_pairs[..],
                        24 + digital_b_pos,
                    ),
                ]
                .iter_mut()
                .for_each(|(pairs, offset)| {
                    pairs
                        .iter_mut()
                        .flat_map(|pair| pair.gain_to_left.iter_mut())
                        .enumerate()
                        .for_each(|(j, gain)| {
                            let mixer_index = i * 2;
                            let pos = 4 * (mixer_index * (8 + 8 + 8 + 8) + *offset + j);
                            deserialize_i32(gain, &raw[pos..(pos + 4)]);
                        });

                    pairs
                        .iter_mut()
                        .flat_map(|pair| pair.gain_to_right.iter_mut())
                        .enumerate()
                        .for_each(|(j, gain)| {
                            let mixer_index = i * 2 + 1;
                            let pos = 4 * (mixer_index * (8 + 8 + 8 + 8) + *offset + j);
                            deserialize_i32(gain, &raw[pos..(pos + 4)]);
                        });

                    pairs
                        .iter_mut()
                        .flat_map(|pair| pair.mutes.iter_mut())
                        .enumerate()
                        .for_each(|(j, mute)| *mute = mutes_val & (1 << (*offset + j)) > 0);

                    pairs
                        .iter_mut()
                        .flat_map(|pair| pair.solos.iter_mut())
                        .enumerate()
                        .for_each(|(j, solo)| *solo = solos_val & (1 << (*offset + j)) > 0);

                    pairs
                        .iter_mut()
                        .enumerate()
                        .for_each(|(j, pair)| pair.link = links_val & (1 << (*offset / 2 + j)) > 0);
                });

                [
                    &mut srcs.stream_inputs_to_left[..],
                    &mut srcs.stream_inputs_to_right[..],
                ]
                .iter_mut()
                .enumerate()
                .for_each(|(j, gains)| {
                    gains.iter_mut().enumerate().for_each(|(k, gain)| {
                        let mixer_index = i * 2 + j;
                        let pos = 4 * (mixer_index * (8 + 8 + 8 + 8) + 8 + k);
                        deserialize_i32(gain, &raw[pos..(pos + 4)]);
                    });
                });
            });

        params
            .mixer_pairs
            .iter_mut()
            .flat_map(|srcs| srcs.monitor_pair.output_volumes.iter_mut())
            .enumerate()
            .for_each(|(i, vol)| {
                let pos = 0x400 + 4 * i;
                deserialize_u32(vol, &raw[pos..(pos + 4)]);
            });

        let mut val = 0u32;
        deserialize_u32(&mut val, &raw[0x430..0x434]);
        params
            .mixer_pairs
            .iter_mut()
            .flat_map(|srcs| srcs.monitor_pair.output_mutes.iter_mut())
            .enumerate()
            .for_each(|(i, mute)| *mute = val & (1 << i) > 0);

        deserialize_u32(&mut params.blend_knob, &raw[0x454..0x458]);
        deserialize_u32(&mut params.master_knob, &raw[0x458..0x45c]);

        Ok(())
    }
}

impl<O: AlesisOperation + AlesisParametersSerdes<IofwMixerParams>>
    AlesisFluctuatedParametersOperation<IofwMixerParams> for O
{
    const FLUCTUATED_OFFSET_RANGES: &'static [Range<usize>] = &[
        // NOTE: Mix blend knob operates output volume of mixer 1/2.
        Range {
            start: MIXER_OUTPUT_VOLUME_OFFSET,
            end: MIXER_OUTPUT_VOLUME_OFFSET + 8,
        },
        Range {
            start: KNOB_PARAMS_OFFSET,
            end: KNOB_PARAMS_OFFSET + KNOB_PARAMS_SIZE,
        },
    ];
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn io14_meter_params_serdes() {
        let mut params = <Io14fwProtocol as IofwMeterSpecification>::create_meter_params();
        params
            .analog_inputs
            .iter_mut()
            .chain(params.stream_inputs.iter_mut())
            .chain(params.digital_a_inputs.iter_mut())
            .chain(params.digital_b_inputs.iter_mut())
            .chain(params.mixer_outputs.iter_mut())
            .enumerate()
            .for_each(|(i, level)| *level = i as i16);

        let size = compute_params_size(
            <Io14fwProtocol as AlesisParametersSerdes<IofwMeterParams>>::OFFSET_RANGES,
        );
        let mut raw = vec![0u8; size];
        Io14fwProtocol::serialize_params(&params, &mut raw).unwrap();

        let mut target = <Io14fwProtocol as IofwMeterSpecification>::create_meter_params();
        Io14fwProtocol::deserialize_params(&mut target, &raw).unwrap();

        assert_eq!(params, target);
    }

    #[test]
    fn io26_meter_params_serdes() {
        let mut params = <Io26fwProtocol as IofwMeterSpecification>::create_meter_params();
        params
            .analog_inputs
            .iter_mut()
            .chain(params.stream_inputs.iter_mut())
            .chain(params.digital_a_inputs.iter_mut())
            .chain(params.digital_b_inputs.iter_mut())
            .chain(params.mixer_outputs.iter_mut())
            .enumerate()
            .for_each(|(i, level)| *level = i as i16);

        let size = compute_params_size(
            <Io14fwProtocol as AlesisParametersSerdes<IofwMeterParams>>::OFFSET_RANGES,
        );
        let mut raw = vec![0u8; size];
        Io26fwProtocol::serialize_params(&params, &mut raw).unwrap();

        let mut target = <Io26fwProtocol as IofwMeterSpecification>::create_meter_params();
        Io26fwProtocol::deserialize_params(&mut target, &raw).unwrap();

        assert_eq!(params, target);
    }

    #[test]
    fn io14_output_params_serdes() {
        let mut params = <Io14fwProtocol as IofwOutputSpecification>::create_output_params();
        params
            .nominal_levels
            .iter_mut()
            .enumerate()
            .filter(|(i, _)| i % 2 > 0)
            .for_each(|(_, level)| *level = NominalSignalLevel::Professional);
        params.digital_67_src = DigitalB67Src::Spdif12;
        params.spdif_out_src = MixerOutPair::Mixer45;
        params.headphone2_3_out_src = MixerOutPair::Mixer67;

        let size = compute_params_size(
            <Io14fwProtocol as AlesisParametersSerdes<IofwOutputParams>>::OFFSET_RANGES,
        );
        let mut raw = vec![0u8; size];
        Io14fwProtocol::serialize_params(&params, &mut raw).unwrap();

        let mut target = <Io14fwProtocol as IofwOutputSpecification>::create_output_params();
        Io14fwProtocol::deserialize_params(&mut target, &raw).unwrap();

        assert_eq!(params, target);
    }

    #[test]
    fn io26_output_params_serdes() {
        let mut params = <Io26fwProtocol as IofwOutputSpecification>::create_output_params();
        params
            .nominal_levels
            .iter_mut()
            .enumerate()
            .filter(|(i, _)| i % 2 > 0)
            .for_each(|(_, level)| *level = NominalSignalLevel::Professional);
        params.digital_67_src = DigitalB67Src::Adat67;
        params.spdif_out_src = MixerOutPair::Mixer01;
        params.headphone2_3_out_src = MixerOutPair::Mixer23;

        let size = compute_params_size(
            <Io14fwProtocol as AlesisParametersSerdes<IofwOutputParams>>::OFFSET_RANGES,
        );
        let mut raw = vec![0u8; size];
        Io26fwProtocol::serialize_params(&params, &mut raw).unwrap();

        let mut target = <Io26fwProtocol as IofwOutputSpecification>::create_output_params();
        Io26fwProtocol::deserialize_params(&mut target, &raw).unwrap();

        assert_eq!(params, target);
    }

    #[test]
    fn io14fw_mixer_params_serdes() {
        let mut params = Io14fwProtocol::create_mixer_params();
        params.mixer_pairs.iter_mut().for_each(|mixer_pair| {
            mixer_pair
                .monitor_pair
                .analog_input_pairs
                .iter_mut()
                .chain(mixer_pair.monitor_pair.digital_a_input_pairs.iter_mut())
                .chain(mixer_pair.monitor_pair.digital_b_input_pairs.iter_mut())
                .flat_map(|pair| pair.gain_to_left.iter_mut())
                .enumerate()
                .for_each(|(i, gain)| *gain = 2 * i as i32);
            mixer_pair
                .monitor_pair
                .analog_input_pairs
                .iter_mut()
                .chain(mixer_pair.monitor_pair.digital_a_input_pairs.iter_mut())
                .chain(mixer_pair.monitor_pair.digital_b_input_pairs.iter_mut())
                .flat_map(|pair| pair.gain_to_right.iter_mut())
                .enumerate()
                .for_each(|(i, gain)| *gain = 1 + 2 * i as i32);
            [
                &mut mixer_pair.stream_inputs_to_left[..],
                &mut mixer_pair.stream_inputs_to_right[..],
            ]
            .iter_mut()
            .enumerate()
            .for_each(|(i, gains)| {
                gains
                    .iter_mut()
                    .enumerate()
                    .for_each(|(j, gain)| *gain = (i * 8 + j) as i32);
            });
            mixer_pair
                .monitor_pair
                .analog_input_pairs
                .iter_mut()
                .chain(mixer_pair.monitor_pair.digital_a_input_pairs.iter_mut())
                .chain(mixer_pair.monitor_pair.digital_b_input_pairs.iter_mut())
                .flat_map(|pair| pair.mutes.iter_mut())
                .enumerate()
                .for_each(|(i, mute)| *mute = i % 2 > 0);
            mixer_pair
                .monitor_pair
                .analog_input_pairs
                .iter_mut()
                .chain(mixer_pair.monitor_pair.digital_a_input_pairs.iter_mut())
                .chain(mixer_pair.monitor_pair.digital_b_input_pairs.iter_mut())
                .flat_map(|pair| pair.solos.iter_mut())
                .enumerate()
                .for_each(|(i, solo)| *solo = i % 2 > 0);
            mixer_pair
                .monitor_pair
                .analog_input_pairs
                .iter_mut()
                .chain(mixer_pair.monitor_pair.digital_a_input_pairs.iter_mut())
                .chain(mixer_pair.monitor_pair.digital_b_input_pairs.iter_mut())
                .enumerate()
                .for_each(|(i, pair)| pair.link = i % 2 > 0);
        });
        params
            .mixer_pairs
            .iter_mut()
            .flat_map(|pair| pair.monitor_pair.output_volumes.iter_mut())
            .enumerate()
            .for_each(|(i, vol)| *vol = 3 * i as u32);
        params
            .mixer_pairs
            .iter_mut()
            .flat_map(|pair| pair.monitor_pair.output_mutes.iter_mut())
            .enumerate()
            .for_each(|(i, mute)| *mute = i % 2 > 0);
        params.master_knob = 111;
        params.blend_knob = 111;

        let size = compute_params_size(
            <Io14fwProtocol as AlesisParametersSerdes<IofwMixerParams>>::OFFSET_RANGES,
        );
        let mut raw = vec![0u8; size];
        Io14fwProtocol::serialize_params(&params, &mut raw).unwrap();

        let mut p = Io14fwProtocol::create_mixer_params();
        Io14fwProtocol::deserialize_params(&mut p, &raw).unwrap();

        assert_eq!(params, p);
    }

    #[test]
    fn io26fw_mixer_params_serdes() {
        let mut params = Io26fwProtocol::create_mixer_params();
        params.mixer_pairs.iter_mut().for_each(|mixer_pair| {
            mixer_pair
                .monitor_pair
                .analog_input_pairs
                .iter_mut()
                .chain(mixer_pair.monitor_pair.digital_a_input_pairs.iter_mut())
                .chain(mixer_pair.monitor_pair.digital_b_input_pairs.iter_mut())
                .flat_map(|pair| pair.gain_to_left.iter_mut())
                .enumerate()
                .for_each(|(i, gain)| *gain = 2 * i as i32);
            mixer_pair
                .monitor_pair
                .analog_input_pairs
                .iter_mut()
                .chain(mixer_pair.monitor_pair.digital_a_input_pairs.iter_mut())
                .chain(mixer_pair.monitor_pair.digital_b_input_pairs.iter_mut())
                .flat_map(|pair| pair.gain_to_right.iter_mut())
                .enumerate()
                .for_each(|(i, gain)| *gain = 1 + 2 * i as i32);
            [
                &mut mixer_pair.stream_inputs_to_left[..],
                &mut mixer_pair.stream_inputs_to_right[..],
            ]
            .iter_mut()
            .enumerate()
            .for_each(|(i, gains)| {
                gains
                    .iter_mut()
                    .enumerate()
                    .for_each(|(j, gain)| *gain = (i * 8 + j) as i32);
            });
            mixer_pair
                .monitor_pair
                .analog_input_pairs
                .iter_mut()
                .chain(mixer_pair.monitor_pair.digital_a_input_pairs.iter_mut())
                .chain(mixer_pair.monitor_pair.digital_b_input_pairs.iter_mut())
                .flat_map(|pair| pair.mutes.iter_mut())
                .enumerate()
                .for_each(|(i, mute)| *mute = i % 2 > 0);
            mixer_pair
                .monitor_pair
                .analog_input_pairs
                .iter_mut()
                .chain(mixer_pair.monitor_pair.digital_a_input_pairs.iter_mut())
                .chain(mixer_pair.monitor_pair.digital_b_input_pairs.iter_mut())
                .flat_map(|pair| pair.solos.iter_mut())
                .enumerate()
                .for_each(|(i, solo)| *solo = i % 2 > 0);
            mixer_pair
                .monitor_pair
                .analog_input_pairs
                .iter_mut()
                .chain(mixer_pair.monitor_pair.digital_a_input_pairs.iter_mut())
                .chain(mixer_pair.monitor_pair.digital_b_input_pairs.iter_mut())
                .enumerate()
                .for_each(|(i, pair)| pair.link = i % 2 > 0);
        });
        params
            .mixer_pairs
            .iter_mut()
            .flat_map(|pair| pair.monitor_pair.output_volumes.iter_mut())
            .enumerate()
            .for_each(|(i, vol)| *vol = 3 * i as u32);
        params
            .mixer_pairs
            .iter_mut()
            .flat_map(|pair| pair.monitor_pair.output_mutes.iter_mut())
            .enumerate()
            .for_each(|(i, mute)| *mute = i % 2 > 0);
        params.master_knob = 111;
        params.blend_knob = 111;
        let size = compute_params_size(
            <Io26fwProtocol as AlesisParametersSerdes<IofwMixerParams>>::OFFSET_RANGES,
        );
        let mut raw = vec![0u8; size];
        Io26fwProtocol::serialize_params(&params, &mut raw).unwrap();

        let mut p = Io26fwProtocol::create_mixer_params();
        Io26fwProtocol::deserialize_params(&mut p, &raw).unwrap();

        assert_eq!(params, p);
    }
}
