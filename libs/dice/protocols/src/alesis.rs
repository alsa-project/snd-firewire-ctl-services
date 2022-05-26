// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Protocol specific to Alesis iO FireWire series.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by Alesis for iO FireWire series.

pub mod meter;
pub mod mixer;
pub mod output;

use {
    self::{meter::*, mixer::*, output::*},
    super::{tcat::*, *},
};

/// The structure for protocol implementation specific to iO 14 FireWire.
#[derive(Default)]
pub struct Io14fwProtocol;

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

/// The structure for protocol implementation specific to iO 26 FireWire.
#[derive(Default)]
pub struct Io26fwProtocol;

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
