// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol implementation for PreSonus FireStudio.

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

const PARAMS_OFFSET: usize = 0x0f68;
const SRC_OFFSET: usize = 0x10ac;
const MAIN_OFFSET: usize = 0x10f4;
const HP01_OFFSET: usize = 0x10f8;
const HP23_OFFSET: usize = 0x10fc;
const HP45_OFFSET: usize = 0x1100;
const BNC_TERMINATE_OFFSET: usize = 0x1118;
const LINK_OFFSET: usize = 0x1150;
const METER_OFFSET: usize = 0x13e8;

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

fn presonus_read(
    req: &mut FwReq,
    node: &mut FwNode,
    offset: usize,
    raw: &mut [u8],
    timeout_ms: u32,
) -> Result<(), Error> {
    GeneralProtocol::read(req, node, OFFSET + offset, raw, timeout_ms)
}

fn presonus_write(
    req: &mut FwReq,
    node: &mut FwNode,
    offset: usize,
    raw: &mut [u8],
    timeout_ms: u32,
) -> Result<(), Error> {
    GeneralProtocol::write(req, node, OFFSET + offset, raw, timeout_ms)
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

impl FStudioProtocol {
    pub fn read_meter(
        req: &mut FwReq,
        node: &mut FwNode,
        meter: &mut FStudioMeter,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut raw = vec![0; FStudioMeter::SIZE];
        presonus_read(req, node, METER_OFFSET, &mut raw, timeout_ms).map(|_| {
            let mut quadlet = [0; 4];
            (0..(FStudioMeter::SIZE / 4)).for_each(|i| {
                let pos = i * 4;
                quadlet.copy_from_slice(&raw[pos..(pos + 4)]);
                let val = u32::from_be_bytes(quadlet);
                raw[pos..(pos + 4)].copy_from_slice(&val.to_le_bytes());
            });
            meter.analog_inputs.copy_from_slice(&raw[8..16]);
            meter.stream_inputs.copy_from_slice(&raw[16..34]);
            meter.mixer_outputs.copy_from_slice(&raw[40..58]);
        })
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
        // (volume, unused, mute) * 18 outputs.
        Range {
            start: PARAMS_OFFSET,
            end: PARAMS_OFFSET + 4 * 3 * 18,
        },
        // source * 18 outputs.
        Range {
            start: SRC_OFFSET,
            end: SRC_OFFSET + 72,
        },
        // Assignment to main and headphone outputs.
        Range {
            start: MAIN_OFFSET,
            end: MAIN_OFFSET + 16,
        },
        // BNC terminate.
        Range {
            start: BNC_TERMINATE_OFFSET,
            end: BNC_TERMINATE_OFFSET + 4,
        },
        // link bit flags for 18 outputs.
        Range {
            start: LINK_OFFSET,
            end: LINK_OFFSET + 4,
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

/// State of outputs for FireStudio.
#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
pub struct OutputState {
    pub vols: [u8; 18],
    pub mutes: [bool; 18],
    pub srcs: [OutputSrc; 18],
    pub links: [bool; 9],
}

impl FStudioProtocol {
    pub fn read_output_states(
        req: &mut FwReq,
        node: &mut FwNode,
        states: &mut OutputState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut raw = vec![0; 4 * states.vols.len() * 3];
        presonus_read(req, node, PARAMS_OFFSET, &mut raw, timeout_ms)?;
        let mut quadlet = [0; 4];
        let quads: Vec<u32> = (0..(states.vols.len() * 3))
            .map(|i| {
                let pos = i * 4;
                quadlet.copy_from_slice(&raw[pos..(pos + 4)]);
                u32::from_be_bytes(quadlet)
            })
            .collect();
        states.vols.iter_mut().enumerate().for_each(|(i, vol)| {
            let pos = i * 3;
            *vol = quads[pos] as u8;
        });
        states.mutes.iter_mut().enumerate().for_each(|(i, mute)| {
            let pos = 2 + i * 3;
            *mute = quads[pos] > 0;
        });

        let mut raw = vec![0; 4 * states.srcs.len()];
        presonus_read(req, node, SRC_OFFSET, &mut raw, timeout_ms)?;
        states.srcs.iter_mut().enumerate().for_each(|(i, src)| {
            let pos = i * 4;
            let _ = deserialize_output_source(src, &raw[pos..(pos + 4)]);
        });

        let mut raw = [0; 4];
        presonus_read(req, node, LINK_OFFSET, &mut raw, timeout_ms)?;
        let val = u32::from_be_bytes(raw);
        states
            .links
            .iter_mut()
            .enumerate()
            .for_each(|(i, link)| *link = val & (1 << i) > 0);

        Ok(())
    }

    pub fn write_output_vols(
        req: &mut FwReq,
        node: &mut FwNode,
        states: &mut OutputState,
        vols: &[u8],
        timeout_ms: u32,
    ) -> Result<(), Error> {
        assert_eq!(vols.len(), states.vols.len());

        let mut raw = [0; 4];
        states
            .vols
            .iter_mut()
            .zip(vols)
            .enumerate()
            .filter(|(_, (old, new))| !new.eq(old))
            .try_for_each(|(i, (old, new))| {
                let pos = i * 3 * 4;
                raw.copy_from_slice(&(*new as u32).to_be_bytes());
                presonus_write(req, node, PARAMS_OFFSET + pos, &mut raw, timeout_ms)
                    .map(|_| *old = *new)
            })
    }

    pub fn write_output_mute(
        req: &mut FwReq,
        node: &mut FwNode,
        states: &mut OutputState,
        mutes: &[bool],
        timeout_ms: u32,
    ) -> Result<(), Error> {
        assert_eq!(mutes.len(), states.mutes.len());

        let mut raw = [0; 4];
        states
            .mutes
            .iter_mut()
            .zip(mutes)
            .enumerate()
            .filter(|(_, (old, new))| !new.eq(old))
            .try_for_each(|(i, (old, new))| {
                let pos = (2 + i * 3) * 4;
                raw.copy_from_slice(&(*new as u32).to_be_bytes());
                presonus_write(req, node, PARAMS_OFFSET + pos, &mut raw, timeout_ms)
                    .map(|_| *old = *new)
            })
    }

    pub fn write_output_src(
        req: &mut FwReq,
        node: &mut FwNode,
        states: &mut OutputState,
        srcs: &[OutputSrc],
        timeout_ms: u32,
    ) -> Result<(), Error> {
        assert_eq!(srcs.len(), states.srcs.len());

        let mut raw = [0; 4];
        states
            .srcs
            .iter_mut()
            .zip(srcs)
            .enumerate()
            .filter(|(_, (old, new))| !new.eq(old))
            .try_for_each(|(i, (old, new))| {
                let pos = i * 4;
                let _ = serialize_output_source(new, &mut raw);
                presonus_write(req, node, SRC_OFFSET + pos, &mut raw, timeout_ms)
                    .map(|_| *old = *new)
            })
    }

    pub fn write_output_link(
        req: &mut FwReq,
        node: &mut FwNode,
        states: &mut OutputState,
        links: &[bool],
        timeout_ms: u32,
    ) -> Result<(), Error> {
        assert_eq!(links.len(), states.links.len());

        let val: u32 = links
            .iter()
            .enumerate()
            .filter(|(_, &link)| link)
            .fold(0u32, |val, (i, _)| val | (1 << i));

        let mut raw = [0; 4];
        val.build_quadlet(&mut raw);
        presonus_write(req, node, LINK_OFFSET, &mut raw, timeout_ms)
            .map(|_| states.links.copy_from_slice(links))
    }

    pub fn read_bnc_terminate(
        req: &mut FwReq,
        node: &mut FwNode,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        let mut raw = [0; 4];
        presonus_read(req, node, BNC_TERMINATE_OFFSET, &mut raw, timeout_ms)
            .map(|_| u32::from_be_bytes(raw) > 0)
    }

    pub fn write_bnc_terminalte(
        req: &mut FwReq,
        node: &mut FwNode,
        terminate: bool,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut raw = [0; 4];
        terminate.build_quadlet(&mut raw);
        presonus_write(req, node, BNC_TERMINATE_OFFSET, &mut raw, timeout_ms)
    }
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

impl FStudioProtocol {
    fn read_assign_target(
        req: &mut FwReq,
        node: &mut FwNode,
        offset: usize,
        timeout_ms: u32,
    ) -> Result<AssignTarget, Error> {
        let mut raw = [0; 4];
        presonus_read(req, node, offset, &mut raw, timeout_ms).map(|_| {
            let mut target = AssignTarget::default();
            let _ = deserialize_assign_target(&mut target, &raw);
            target
        })
    }

    fn write_assign_target(
        req: &mut FwReq,
        node: &mut FwNode,
        offset: usize,
        target: AssignTarget,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut raw = [0; 4];
        let _ = serialize_assign_target(&target, &mut raw);
        presonus_write(req, node, offset, &mut raw, timeout_ms)
    }

    pub fn read_main_assign_target(
        req: &mut FwReq,
        node: &mut FwNode,
        timeout_ms: u32,
    ) -> Result<AssignTarget, Error> {
        Self::read_assign_target(req, node, MAIN_OFFSET, timeout_ms)
    }

    pub fn read_hp_assign_target(
        req: &mut FwReq,
        node: &mut FwNode,
        hp: usize,
        timeout_ms: u32,
    ) -> Result<AssignTarget, Error> {
        let offset = match hp {
            0 => HP01_OFFSET,
            1 => HP23_OFFSET,
            2 => HP45_OFFSET,
            _ => unreachable!(),
        };
        Self::read_assign_target(req, node, offset, timeout_ms)
    }

    pub fn write_main_assign_target(
        req: &mut FwReq,
        node: &mut FwNode,
        target: AssignTarget,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        Self::write_assign_target(req, node, MAIN_OFFSET, target, timeout_ms)
    }

    pub fn write_hp_assign_target(
        req: &mut FwReq,
        node: &mut FwNode,
        hp: usize,
        target: AssignTarget,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let offset = match hp {
            0 => HP01_OFFSET,
            1 => HP23_OFFSET,
            2 => HP45_OFFSET,
            _ => unreachable!(),
        };
        Self::write_assign_target(req, node, offset, target, timeout_ms)
    }
}

/// Mode of mixer expansion.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ExpansionMode {
    Stream10_17,
    AdatB0_7,
}

impl Default for ExpansionMode {
    fn default() -> Self {
        Self::Stream10_17
    }
}

impl From<u32> for ExpansionMode {
    fn from(val: u32) -> Self {
        match val {
            0 => Self::Stream10_17,
            _ => Self::AdatB0_7,
        }
    }
}

impl From<ExpansionMode> for u32 {
    fn from(mode: ExpansionMode) -> Self {
        match mode {
            ExpansionMode::Stream10_17 => 0,
            ExpansionMode::AdatB0_7 => 1,
        }
    }
}

/// Parameters of mixer sources.
#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
pub struct SrcParams {
    pub gains: [u8; 18],
    pub pans: [u8; 18],
    pub mutes: [bool; 18],
}

/// Parameters of mixer outputs.
#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
pub struct OutParams {
    pub vols: [u8; MIXER_COUNT],
    pub mutes: [bool; MIXER_COUNT],
}

/// The number of mixers.
pub const MIXER_COUNT: usize = 9;

const PHYS_SRC_PARAMS_OFFSET: usize = 0x0038;
const STREAM_SRC_PARAMS_OFFSET: usize = 0x07d0;
const OUT_PARAMS_OFFSET: usize = 0x1040;
const EXPANSION_MODE_OFFSET: usize = 0x1128;
const SRC_LINK_OFFSET: usize = 0x112c;

impl FStudioProtocol {
    pub fn read_mixer_src_params(
        req: &mut FwReq,
        node: &mut FwNode,
        params: &mut SrcParams,
        offset: usize,
        ch: usize,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        assert!(ch < MIXER_COUNT);

        let quad_count = 3 * (8 + 8 + 2);
        let mut raw = vec![0; quad_count * 4];
        let pos = ch * quad_count * 4;
        presonus_read(req, node, offset + pos, &mut raw, timeout_ms).map(|_| {
            let mut quads = vec![0u32; quad_count];
            quads.parse_quadlet_block(&raw);

            params.gains.iter_mut().enumerate().for_each(|(i, gain)| {
                let pos = i * 3;
                *gain = quads[pos] as u8;
            });

            params.pans.iter_mut().enumerate().for_each(|(i, pan)| {
                let pos = 1 + i * 3;
                *pan = quads[pos] as u8;
            });

            params.mutes.iter_mut().enumerate().for_each(|(i, mute)| {
                let pos = 2 + i * 3;
                *mute = quads[pos] > 0;
            });
        })
    }

    pub fn read_mixer_phys_src_params(
        req: &mut FwReq,
        node: &mut FwNode,
        params: &mut SrcParams,
        ch: usize,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        Self::read_mixer_src_params(req, node, params, PHYS_SRC_PARAMS_OFFSET, ch, timeout_ms)
    }

    pub fn read_mixer_stream_src_params(
        req: &mut FwReq,
        node: &mut FwNode,
        params: &mut SrcParams,
        ch: usize,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        Self::read_mixer_src_params(req, node, params, STREAM_SRC_PARAMS_OFFSET, ch, timeout_ms)
    }

    pub fn write_mixer_src_gains(
        req: &mut FwReq,
        node: &mut FwNode,
        params: &mut SrcParams,
        offset: usize,
        ch: usize,
        gains: &[u8],
        timeout_ms: u32,
    ) -> Result<(), Error> {
        assert!(ch < MIXER_COUNT);
        assert_eq!(params.gains.len(), gains.len());

        let mut raw = [0; 4];
        params
            .gains
            .iter_mut()
            .zip(gains)
            .enumerate()
            .filter(|(_, (old, new))| !old.eq(new))
            .try_for_each(|(i, (old, new))| {
                new.build_quadlet(&mut raw);
                let mut pos = ch * 3 * (8 + 8 + 2) * 4;
                pos += i * 3 * 4;
                presonus_write(req, node, offset + pos, &mut raw, timeout_ms).map(|_| *old = *new)
            })
    }

    pub fn write_mixer_src_pans(
        req: &mut FwReq,
        node: &mut FwNode,
        params: &mut SrcParams,
        offset: usize,
        ch: usize,
        pans: &[u8],
        timeout_ms: u32,
    ) -> Result<(), Error> {
        assert!(ch < MIXER_COUNT);
        assert_eq!(params.pans.len(), pans.len());

        let mut raw = [0; 4];
        params
            .pans
            .iter_mut()
            .zip(pans)
            .enumerate()
            .filter(|(_, (old, new))| !old.eq(new))
            .try_for_each(|(i, (old, new))| {
                (*new as u32).build_quadlet(&mut raw);
                let mut pos = ch * 3 * (8 + 8 + 2) * 4;
                pos += (i * 3 + 1) * 4;
                presonus_write(req, node, offset + pos, &mut raw, timeout_ms).map(|_| *old = *new)
            })
    }

    pub fn write_mixer_src_mutes(
        req: &mut FwReq,
        node: &mut FwNode,
        params: &mut SrcParams,
        offset: usize,
        ch: usize,
        mutes: &[bool],
        timeout_ms: u32,
    ) -> Result<(), Error> {
        assert!(ch < MIXER_COUNT);
        assert_eq!(params.mutes.len(), mutes.len());

        let mut raw = [0; 4];
        params
            .mutes
            .iter_mut()
            .zip(mutes)
            .enumerate()
            .filter(|(_, (old, new))| !old.eq(new))
            .try_for_each(|(i, (old, new))| {
                (*new as u32).build_quadlet(&mut raw);
                let mut pos = ch * 3 * (8 + 8 + 2) * 4;
                pos += (i * 3 + 2) * 4;
                presonus_write(req, node, offset + pos, &mut raw, timeout_ms).map(|_| *old = *new)
            })
    }
    pub fn write_mixer_phys_src_gains(
        req: &mut FwReq,
        node: &mut FwNode,
        params: &mut SrcParams,
        ch: usize,
        gains: &[u8],
        timeout_ms: u32,
    ) -> Result<(), Error> {
        Self::write_mixer_src_gains(
            req,
            node,
            params,
            PHYS_SRC_PARAMS_OFFSET,
            ch,
            gains,
            timeout_ms,
        )
    }

    pub fn write_mixer_phys_src_pans(
        req: &mut FwReq,
        node: &mut FwNode,
        params: &mut SrcParams,
        ch: usize,
        pans: &[u8],
        timeout_ms: u32,
    ) -> Result<(), Error> {
        Self::write_mixer_src_pans(
            req,
            node,
            params,
            PHYS_SRC_PARAMS_OFFSET,
            ch,
            pans,
            timeout_ms,
        )
    }

    pub fn write_mixer_phys_src_mutes(
        req: &mut FwReq,
        node: &mut FwNode,
        params: &mut SrcParams,
        ch: usize,
        mutes: &[bool],
        timeout_ms: u32,
    ) -> Result<(), Error> {
        Self::write_mixer_src_mutes(
            req,
            node,
            params,
            PHYS_SRC_PARAMS_OFFSET,
            ch,
            mutes,
            timeout_ms,
        )
    }

    pub fn write_mixer_stream_src_gains(
        req: &mut FwReq,
        node: &mut FwNode,
        params: &mut SrcParams,
        ch: usize,
        gains: &[u8],
        timeout_ms: u32,
    ) -> Result<(), Error> {
        Self::write_mixer_src_gains(
            req,
            node,
            params,
            STREAM_SRC_PARAMS_OFFSET,
            ch,
            gains,
            timeout_ms,
        )
    }

    pub fn write_mixer_stream_src_pans(
        req: &mut FwReq,
        node: &mut FwNode,
        params: &mut SrcParams,
        ch: usize,
        pans: &[u8],
        timeout_ms: u32,
    ) -> Result<(), Error> {
        Self::write_mixer_src_pans(
            req,
            node,
            params,
            STREAM_SRC_PARAMS_OFFSET,
            ch,
            pans,
            timeout_ms,
        )
    }

    pub fn write_mixer_stream_src_mutes(
        req: &mut FwReq,
        node: &mut FwNode,
        params: &mut SrcParams,
        ch: usize,
        mutes: &[bool],
        timeout_ms: u32,
    ) -> Result<(), Error> {
        Self::write_mixer_src_mutes(
            req,
            node,
            params,
            STREAM_SRC_PARAMS_OFFSET,
            ch,
            mutes,
            timeout_ms,
        )
    }

    pub fn read_mixer_out_params(
        req: &mut FwReq,
        node: &mut FwNode,
        params: &mut OutParams,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut raw = vec![0; 3 * MIXER_COUNT * 4];
        presonus_read(req, node, OUT_PARAMS_OFFSET, &mut raw, timeout_ms).map(|_| {
            let mut quads = vec![0u32; 3 * MIXER_COUNT];
            quads.parse_quadlet_block(&raw);

            params.vols.iter_mut().enumerate().for_each(|(i, vol)| {
                let pos = i * 3;
                *vol = quads[pos] as u8;
            });

            params.mutes.iter_mut().enumerate().for_each(|(i, mute)| {
                let pos = i * 3 + 2;
                *mute = quads[pos] > 0;
            });
        })
    }

    pub fn write_mixer_out_vol(
        req: &mut FwReq,
        node: &mut FwNode,
        params: &mut OutParams,
        vols: &[u8],
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut raw = [0; 4];
        params
            .vols
            .iter_mut()
            .zip(vols)
            .enumerate()
            .filter(|(_, (o, n))| !o.eq(n))
            .try_for_each(|(i, (o, n))| {
                (*n as u32).build_quadlet(&mut raw);
                let offset = 3 * i * 4;
                presonus_write(req, node, OUT_PARAMS_OFFSET + offset, &mut raw, timeout_ms)
                    .map(|_| *o = *n)
            })
    }

    pub fn write_mixer_out_mute(
        req: &mut FwReq,
        node: &mut FwNode,
        params: &mut OutParams,
        mutes: &[bool],
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut raw = [0; 4];
        params
            .mutes
            .iter_mut()
            .zip(mutes)
            .enumerate()
            .filter(|(_, (o, n))| !o.eq(n))
            .try_for_each(|(i, (o, n))| {
                (*n as u32).build_quadlet(&mut raw);
                let offset = (3 * i + 2) * 4;
                presonus_write(req, node, OUT_PARAMS_OFFSET + offset, &mut raw, timeout_ms)
                    .map(|_| *o = *n)
            })
    }

    pub fn read_mixer_expansion_mode(
        req: &mut FwReq,
        node: &mut FwNode,
        timeout_ms: u32,
    ) -> Result<ExpansionMode, Error> {
        let mut raw = [0; 4];
        presonus_read(req, node, EXPANSION_MODE_OFFSET, &mut raw, timeout_ms)
            .map(|_| ExpansionMode::from(u32::from_be_bytes(raw)))
    }

    pub fn write_mixer_expansion_mode(
        req: &mut FwReq,
        node: &mut FwNode,
        mode: ExpansionMode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut raw = [0; 4];
        mode.build_quadlet(&mut raw);
        presonus_write(req, node, EXPANSION_MODE_OFFSET, &mut raw, timeout_ms)
    }

    pub fn read_mixer_src_links(
        req: &mut FwReq,
        node: &mut FwNode,
        links: &mut [bool],
        ch: usize,
        shift: usize,
        mask: u32,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        assert!(ch < MIXER_COUNT);
        assert_eq!(links.len(), 9);

        let mut raw = [0; 4];
        let offset = ch * 4;
        presonus_read(req, node, SRC_LINK_OFFSET + offset, &mut raw, timeout_ms).map(|_| {
            let val = u32::from_be_bytes(raw) & mask;
            links
                .iter_mut()
                .enumerate()
                .for_each(|(i, link)| *link = val & (1 << (i + shift)) > 0);
        })
    }

    pub fn write_mixer_src_links(
        req: &mut FwReq,
        node: &mut FwNode,
        links: &[bool],
        ch: usize,
        shift: usize,
        mask: u32,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut raw = [0; 4];
        let offset = ch * 4;
        presonus_read(req, node, SRC_LINK_OFFSET + offset, &mut raw, timeout_ms)?;

        let mut val = u32::from_be_bytes(raw) & !mask;
        links
            .iter()
            .enumerate()
            .filter(|(_, &link)| link)
            .for_each(|(i, _)| val |= 1 << (i + shift));

        val.build_quadlet(&mut raw);
        presonus_write(req, node, SRC_LINK_OFFSET + offset, &mut raw, timeout_ms)
    }

    pub fn read_mixer_phys_src_links(
        req: &mut FwReq,
        node: &mut FwNode,
        links: &mut [bool],
        ch: usize,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        Self::read_mixer_src_links(req, node, links, ch, 0, 0x0000ffff, timeout_ms)
    }

    pub fn read_mixer_stream_src_links(
        req: &mut FwReq,
        node: &mut FwNode,
        links: &mut [bool],
        ch: usize,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        Self::read_mixer_src_links(req, node, links, ch, 16, 0xffff0000, timeout_ms)
    }

    pub fn write_mixer_phys_src_links(
        req: &mut FwReq,
        node: &mut FwNode,
        links: &[bool],
        ch: usize,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        Self::write_mixer_src_links(req, node, links, ch, 0, 0x0000ffff, timeout_ms)
    }

    pub fn write_mixer_stream_src_links(
        req: &mut FwReq,
        node: &mut FwNode,
        links: &[bool],
        ch: usize,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        Self::write_mixer_src_links(req, node, links, ch, 16, 0xffff0000, timeout_ms)
    }
}

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
}
