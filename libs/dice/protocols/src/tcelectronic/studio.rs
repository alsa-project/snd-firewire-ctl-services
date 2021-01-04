// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Protocol defined by TC Electronic for Studio Konnekt 48.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by TC Electronic for Studio Konnekt 48.

use super::{*, ch_strip::*, reverb::*};

/// The structure to represent segments in memory space of Studio Konnekt 48.
#[derive(Default, Debug)]
pub struct StudioSegments{
    /// Segment for state of reverb effect. 0x0594..0x05d7. (17 quads)
    pub reverb_state: TcKonnektSegment<StudioReverbState>,
    /// Segment for states of channel strip effect. 0x05d8..0x081f (146 quads).
    pub ch_strip_state: TcKonnektSegment<StudioChStripStates>,
    /// Segment for meter of reverb effect. 0x2164..0x217b (6 quads).
    pub reverb_meter: TcKonnektSegment<StudioReverbMeter>,
    /// Segment for meters of channel strip effect. 0x217c..0x21b7 (30 quads).
    pub ch_strip_meter: TcKonnektSegment<StudioChStripMeters>,
}

const STUDIO_REVERB_NOTIFY_CHANGE: u32 = 0x00200000;
const STUDIO_CH_STRIP_NOTIFY_01_CHANGE: u32 = 0x00400000;
const STUDIO_CH_STRIP_NOTIFY_23_CHANGE: u32 = 0x00800000;

const STUDIO_CH_STRIP_COUNT: usize = 4;

#[derive(Default, Debug)]
pub struct StudioReverbState(ReverbState);

impl AsRef<ReverbState> for StudioReverbState {
    fn as_ref(&self) -> &ReverbState {
        &self.0
    }
}

impl AsMut<ReverbState> for StudioReverbState {
    fn as_mut(&mut self) -> &mut ReverbState {
        &mut self.0
    }
}

impl TcKonnektSegmentData for StudioReverbState {
    fn build(&self, raw: &mut [u8]) {
        self.0.build(raw)
    }

    fn parse(&mut self, raw: &[u8]) {
        self.0.parse(raw)
    }
}

impl TcKonnektSegmentSpec for TcKonnektSegment<StudioReverbState> {
    const OFFSET: usize = 0x0594;
    const SIZE: usize = ReverbState::SIZE;
}

impl TcKonnektNotifiedSegmentSpec for TcKonnektSegment<StudioReverbState> {
    const NOTIFY_FLAG: u32 = STUDIO_REVERB_NOTIFY_CHANGE;
}

#[derive(Default, Debug)]
pub struct StudioChStripStates([ChStripState;STUDIO_CH_STRIP_COUNT]);

impl AsRef<[ChStripState]> for StudioChStripStates {
    fn as_ref(&self) -> &[ChStripState] {
        &self.0
    }
}

impl AsMut<[ChStripState]> for StudioChStripStates {
    fn as_mut(&mut self) -> &mut [ChStripState] {
        &mut self.0
    }
}

impl TcKonnektSegmentData for StudioChStripStates {
    fn build(&self, raw: &mut [u8]) {
        self.0.build(raw)
    }

    fn parse(&mut self, raw: &[u8]) {
        self.0.parse(raw)
    }
}

impl TcKonnektSegmentSpec for TcKonnektSegment<StudioChStripStates> {
    const OFFSET: usize = 0x05d8;
    const SIZE: usize = ChStripState::SIZE * STUDIO_CH_STRIP_COUNT + 8;
}

impl TcKonnektNotifiedSegmentSpec for TcKonnektSegment<StudioChStripStates> {
    const NOTIFY_FLAG: u32 = STUDIO_CH_STRIP_NOTIFY_01_CHANGE | STUDIO_CH_STRIP_NOTIFY_23_CHANGE;
}

#[derive(Default, Debug)]
pub struct StudioReverbMeter(ReverbMeter);

impl AsRef<ReverbMeter> for StudioReverbMeter {
    fn as_ref(&self) -> &ReverbMeter {
        &self.0
    }
}

impl AsMut<ReverbMeter> for StudioReverbMeter {
    fn as_mut(&mut self) -> &mut ReverbMeter {
        &mut self.0
    }
}

impl TcKonnektSegmentData for StudioReverbMeter {
    fn build(&self, raw: &mut [u8]) {
        self.0.build(raw)
    }

    fn parse(&mut self, raw: &[u8]) {
        self.0.parse(raw)
    }
}

impl TcKonnektSegmentSpec for TcKonnektSegment<StudioReverbMeter> {
    const OFFSET: usize = 0x2164;
    const SIZE: usize = ReverbMeter::SIZE;
}

#[derive(Default, Debug)]
pub struct StudioChStripMeters([ChStripMeter;STUDIO_CH_STRIP_COUNT]);

impl AsRef<[ChStripMeter]> for StudioChStripMeters {
    fn as_ref(&self) -> &[ChStripMeter] {
        &self.0
    }
}

impl AsMut<[ChStripMeter]> for StudioChStripMeters {
    fn as_mut(&mut self) -> &mut [ChStripMeter] {
        &mut self.0
    }
}

impl TcKonnektSegmentData for StudioChStripMeters {
    fn build(&self, raw: &mut [u8]) {
        self.0.build(raw)
    }

    fn parse(&mut self, raw: &[u8]) {
        self.0.parse(raw)
    }
}

impl TcKonnektSegmentSpec for TcKonnektSegment<StudioChStripMeters> {
    const OFFSET: usize = 0x217c;
    const SIZE: usize = ChStripMeter::SIZE * STUDIO_CH_STRIP_COUNT + 8;
}
