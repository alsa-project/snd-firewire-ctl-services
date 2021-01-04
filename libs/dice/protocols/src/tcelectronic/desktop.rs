// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Protocol defined by TC Electronic for Desktop Konnekt 6.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by TC Electronic for Desktop Konnekt 6.

use crate::*;

use crate::tcelectronic::*;

/// The structure to represent segments in memory space of Desktop Konnekt 6.
#[derive(Default, Debug)]
pub struct DesktopSegments{
    /// Segment for meter. 0x20e4..0x213f (23 quads).
    pub meter: TcKonnektSegment<DesktopMeter>,
}

#[derive(Default, Debug)]
pub struct DesktopMeter{
    pub analog_inputs: [i32;2],
    pub mixer_outputs: [i32;2],
    pub stream_inputs: [i32;2],
}

impl DesktopMeter {
    const SIZE: usize = 92;
}

impl TcKonnektSegmentData for DesktopMeter {
    fn build(&self, raw: &mut [u8]) {
        self.analog_inputs.build_quadlet_block(&mut raw[..8]);
        self.mixer_outputs.build_quadlet_block(&mut raw[40..48]);
        self.stream_inputs.build_quadlet_block(&mut raw[48..56]);
    }

    fn parse(&mut self, raw: &[u8]) {
        self.analog_inputs.parse_quadlet_block(&raw[..8]);
        self.mixer_outputs.parse_quadlet_block(&raw[40..48]);
        self.stream_inputs.parse_quadlet_block(&raw[48..56]);
    }
}

impl TcKonnektSegmentSpec for TcKonnektSegment<DesktopMeter> {
    const OFFSET: usize = 0x20e4;
    const SIZE: usize = DesktopMeter::SIZE;
}
