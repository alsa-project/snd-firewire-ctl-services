// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Protocol defined by TC Electronic for Konnekt 24d.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by TC Electronic for Konnekt 24d.

use super::*;
use crate::tcelectronic::{*, ch_strip::*};

/// The structure to represent segments in memory space of Konnekt 24d.
#[derive(Default, Debug)]
pub struct K24dSegments{
    /// Segment for states of channel strip effect. 0x0214..0x0337 (73 quads).
    pub ch_strip_state: TcKonnektSegment<K24dChStripStates>,
    /// Segment for meters of channel strip effect. 0x10d0..0x110b (15 quads).
    pub ch_strip_meter: TcKonnektSegment<K24dChStripMeters>,
}

#[derive(Default, Debug)]
pub struct K24dChStripStates([ChStripState;SHELL_CH_STRIP_COUNT]);

impl AsRef<[ChStripState]> for K24dChStripStates {
    fn as_ref(&self) -> &[ChStripState] {
        &self.0
    }
}

impl AsMut<[ChStripState]> for K24dChStripStates {
    fn as_mut(&mut self) -> &mut [ChStripState] {
        &mut self.0
    }
}

impl TcKonnektSegmentData for K24dChStripStates {
    fn build(&self, raw: &mut [u8]) {
        self.0.build(raw)
    }

    fn parse(&mut self, raw: &[u8]) {
        self.0.parse(raw)
    }
}

impl TcKonnektSegmentSpec for TcKonnektSegment<K24dChStripStates> {
    const OFFSET: usize = 0x0214;
    const SIZE: usize = ChStripState::SIZE * SHELL_CH_STRIP_COUNT + 4;
}

impl TcKonnektNotifiedSegmentSpec for TcKonnektSegment<K24dChStripStates> {
    const NOTIFY_FLAG: u32 = SHELL_CH_STRIP_NOTIFY_FLAG;
}

#[derive(Default, Debug)]
pub struct K24dChStripMeters([ChStripMeter;SHELL_CH_STRIP_COUNT]);

impl AsRef<[ChStripMeter]> for K24dChStripMeters {
    fn as_ref(&self) -> &[ChStripMeter] {
        &self.0
    }
}

impl AsMut<[ChStripMeter]> for K24dChStripMeters {
    fn as_mut(&mut self) -> &mut [ChStripMeter] {
        &mut self.0
    }
}

impl TcKonnektSegmentData for K24dChStripMeters {
    fn build(&self, raw: &mut [u8]) {
        self.0.build(raw)
    }

    fn parse(&mut self, raw: &[u8]) {
        self.0.parse(raw)
    }
}

impl TcKonnektSegmentSpec for TcKonnektSegment<K24dChStripMeters> {
    const OFFSET: usize = 0x10d0;
    const SIZE: usize = ChStripMeter::SIZE * SHELL_CH_STRIP_COUNT + 4;
}
