// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Protocol specific to Alesis iO FireWire series.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by Alesis for iO FireWire series.

pub mod meter;
pub mod mixer;
pub mod output;

use glib::Error;
use hinawa::FwNode;

use super::{*, tcat::*};

/// The trait to represent protocol defined by Alesis for iO FireWire series.
pub trait AlesisIoProtocol: GeneralProtocol {
    const BASE_OFFSET: usize = 0x00200000;

    fn read_block(
        &self,
        node: &mut FwNode,
        offset: usize,
        frame: &mut [u8],
        timeout_ms: u32
    ) -> Result<(), Error> {
        self.read(node, Self::BASE_OFFSET + offset, frame, timeout_ms)
    }

    fn write_block(
        &self,
        node: &mut FwNode,
        offset: usize,
        frame: &mut [u8],
        timeout_ms: u32
    ) -> Result<(), Error> {
        self.write(node, Self::BASE_OFFSET + offset, frame, timeout_ms)
    }

    fn read_flags(
        &self,
        node: &mut FwNode,
        offset: usize,
        flags: &mut [bool],
        timeout_ms: u32
    ) -> Result<(), Error> {
        let mut raw = [0;4];
        self.read_block(node, offset, &mut raw, timeout_ms)
            .map(|_| {
                let mut val = 0u32;
                val.parse_quadlet(&raw[..]);
                flags.iter_mut()
                    .enumerate()
                    .for_each(|(i, flag)| {
                        *flag = val & (1 << i) > 0;
                    });
            })
    }

    fn write_flags(
        &self,
        node: &mut FwNode,
        offset: usize,
        flags: &[bool],
        timeout_ms: u32
    ) -> Result<(), Error> {
        let val = flags.iter()
            .enumerate()
            .filter(|(_, &flag)| flag)
            .fold(0 as u32, |val, (i, _)| val | (1 << i));
        let mut raw = [0;4];
        val.build_quadlet(&mut raw[..]);
        self.write_block(node, offset, &mut raw, timeout_ms)
    }
}

impl<O: GeneralProtocol> AlesisIoProtocol for O {}
