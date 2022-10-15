// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Protocol specific to Alesis iO FireWire series.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by Alesis for iO FireWire series.

pub mod meter;
pub mod mixer;
pub mod output;

use {
    super::{tcat::*, *},
    meter::*,
    mixer::*,
    output::*,
    std::ops::Range,
};

/// Protocol implementation specific to iO 14 FireWire.
#[derive(Default, Debug)]
pub struct Io14fwProtocol;

impl TcatOperation for Io14fwProtocol {}

impl TcatGlobalSectionSpecification for Io14fwProtocol {}

impl AlesisOperation for Io14fwProtocol {}

impl IofwMeterOperation for Io14fwProtocol {
    const ANALOG_INPUT_COUNT: usize = 4;
    const DIGITAL_B_INPUT_COUNT: usize = 2;
}

impl IofwMixerOperation for Io14fwProtocol {
    const ANALOG_INPUT_COUNT: usize = 4;
    const DIGITAL_B_INPUT_COUNT: usize = 2;
}

impl IofwOutputOperation for Io14fwProtocol {
    const ANALOG_OUTPUT_COUNT: usize = 4;
    const HAS_OPT_IFACE_B: bool = false;
}

/// Protocol implementation specific to iO 26 FireWire.
#[derive(Default, Debug)]
pub struct Io26fwProtocol;

impl TcatOperation for Io26fwProtocol {}

impl TcatGlobalSectionSpecification for Io26fwProtocol {}

impl AlesisOperation for Io26fwProtocol {}

impl IofwMeterOperation for Io26fwProtocol {
    const ANALOG_INPUT_COUNT: usize = 8;
    const DIGITAL_B_INPUT_COUNT: usize = 8;
}

impl IofwMixerOperation for Io26fwProtocol {
    const ANALOG_INPUT_COUNT: usize = 8;
    const DIGITAL_B_INPUT_COUNT: usize = 8;
}

impl IofwOutputOperation for Io26fwProtocol {
    const ANALOG_OUTPUT_COUNT: usize = 8;
    const HAS_OPT_IFACE_B: bool = true;
}

const BASE_OFFSET: usize = 0x00200000;

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

fn compute_params_size(ranges: &[Range<usize>]) -> usize {
    ranges
        .iter()
        .fold(0usize, |size, range| size + range.end - range.start)
}

fn generate_err(name: &str, cause: &str, raw: &[u8]) -> Error {
    let msg = format!("parms: {}, cause: {}, raw: {:02x?}", name, cause, raw);
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

/// Operation to cache content of segment in TC Electronic Konnekt series.
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
pub trait IofwMutableParametersOperation<T>: AlesisOperation + AlesisParametersSerdes<T> {
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

fn alesis_read_block(
    req: &mut FwReq,
    node: &mut FwNode,
    offset: usize,
    frame: &mut [u8],
    timeout_ms: u32,
) -> Result<(), Error> {
    GeneralProtocol::read(req, node, BASE_OFFSET + offset, frame, timeout_ms)
}

fn alesis_write_block(
    req: &mut FwReq,
    node: &mut FwNode,
    offset: usize,
    frame: &mut [u8],
    timeout_ms: u32,
) -> Result<(), Error> {
    GeneralProtocol::write(req, node, BASE_OFFSET + offset, frame, timeout_ms)
}

fn alesis_read_flags(
    req: &mut FwReq,
    node: &mut FwNode,
    offset: usize,
    flags: &mut [bool],
    timeout_ms: u32,
) -> Result<(), Error> {
    let mut raw = [0; 4];
    alesis_read_block(req, node, offset, &mut raw, timeout_ms).map(|_| {
        let mut val = 0u32;
        val.parse_quadlet(&raw[..]);
        flags.iter_mut().enumerate().for_each(|(i, flag)| {
            *flag = val & (1 << i) > 0;
        });
    })
}

fn alesis_write_flags(
    req: &mut FwReq,
    node: &mut FwNode,
    offset: usize,
    flags: &[bool],
    timeout_ms: u32,
) -> Result<(), Error> {
    let val = flags
        .iter()
        .enumerate()
        .filter(|(_, &flag)| flag)
        .fold(0 as u32, |val, (i, _)| val | (1 << i));
    let mut raw = [0; 4];
    val.build_quadlet(&mut raw[..]);
    alesis_write_block(req, node, offset, &mut raw, timeout_ms)
}
