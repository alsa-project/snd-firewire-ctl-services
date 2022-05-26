// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Data of reverb effect in protocol defined by TC Electronic for Konnekt series.
//!
//! The module includes structure, trait and its implementation for data of reverb effect in
//! protocol defined by TC Electronic for Konnekt series. It's called as `Fabrik R`.

use super::*;

/// The enumeration to represent algorithm of reverb effect.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ReverbAlgorithm {
    Live1,
    Hall,
    Plate,
    Club,
    ConcertHall,
    Cathedral,
    Church,
    Room,
    SmallRoom,
    Box,
    Ambient,
    Live2,
    Live3,
    Spring,
    Reserved(u32),
}

impl ReverbAlgorithm {
    const LIVE_1: u32 = 0;
    const HALL: u32 = 1;
    const PLATE: u32 = 2;
    const CLUB: u32 = 3;
    const CONCERT_HALL: u32 = 4;
    const CATHEDRAL: u32 = 5;
    const CHURCH: u32 = 6;
    const ROOM: u32 = 7;
    const SMALL_ROOM: u32 = 8;
    const BOX: u32 = 9;
    const AMBIENT: u32 = 10;
    const LIVE_2: u32 = 11;
    const LIVE_3: u32 = 12;
    const SPRING: u32 = 13;
}

impl Default for ReverbAlgorithm {
    fn default() -> Self {
        ReverbAlgorithm::Reserved(u32::MAX)
    }
}

impl From<u32> for ReverbAlgorithm {
    fn from(val: u32) -> Self {
        match val {
            Self::LIVE_1 => Self::Live1,
            Self::HALL => Self::Hall,
            Self::PLATE => Self::Plate,
            Self::CLUB => Self::Club,
            Self::CONCERT_HALL => Self::ConcertHall,
            Self::CATHEDRAL => Self::Cathedral,
            Self::CHURCH => Self::Church,
            Self::ROOM => Self::Room,
            Self::SMALL_ROOM => Self::SmallRoom,
            Self::BOX => Self::Box,
            Self::AMBIENT => Self::Ambient,
            Self::LIVE_2 => Self::Live2,
            Self::LIVE_3 => Self::Live3,
            Self::SPRING => Self::Spring,
            _ => Self::Reserved(val),
        }
    }
}

impl From<ReverbAlgorithm> for u32 {
    fn from(algorithm: ReverbAlgorithm) -> Self {
        match algorithm {
            ReverbAlgorithm::Live1 => ReverbAlgorithm::LIVE_1,
            ReverbAlgorithm::Hall => ReverbAlgorithm::HALL,
            ReverbAlgorithm::Plate => ReverbAlgorithm::PLATE,
            ReverbAlgorithm::Club => ReverbAlgorithm::CLUB,
            ReverbAlgorithm::ConcertHall => ReverbAlgorithm::CONCERT_HALL,
            ReverbAlgorithm::Cathedral => ReverbAlgorithm::CATHEDRAL,
            ReverbAlgorithm::Church => ReverbAlgorithm::CHURCH,
            ReverbAlgorithm::Room => ReverbAlgorithm::ROOM,
            ReverbAlgorithm::SmallRoom => ReverbAlgorithm::SMALL_ROOM,
            ReverbAlgorithm::Box => ReverbAlgorithm::BOX,
            ReverbAlgorithm::Ambient => ReverbAlgorithm::AMBIENT,
            ReverbAlgorithm::Live2 => ReverbAlgorithm::LIVE_2,
            ReverbAlgorithm::Live3 => ReverbAlgorithm::LIVE_3,
            ReverbAlgorithm::Spring => ReverbAlgorithm::SPRING,
            ReverbAlgorithm::Reserved(val) => val,
        }
    }
}

/// The structure to represent state of reverb effect.
#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
pub struct ReverbState {
    /// The level of input. -24..0 (-24.0..0.0 dB).
    pub input_level: i32,
    pub bypass: bool,
    pub kill_wet: bool,
    pub kill_dry: bool,
    /// The level of output. -24..12 (-24.0..12.0 dB).
    pub output_level: i32,
    /// The decay of time. 1..290.
    pub time_decay: i32,
    /// The pre decay of time. 1..100.
    pub time_pre_decay: i32,
    // blank
    /// The color at low frequency. -50..50.
    pub color_low: i32,
    /// The color at high frequency. -50..50.
    pub color_high: i32,
    /// The factor of color at high frequency. -25..25.
    pub color_high_factor: i32,
    /// The rate of modulation. -25..25.
    pub mod_rate: i32,
    /// The depth of modulation. -25..25.
    pub mod_depth: i32,
    /// The level of early reflection. -48..0.
    pub level_early: i32,
    /// The level of reverb. -48..0.
    pub level_reverb: i32,
    /// The level of dry. -48..0.
    pub level_dry: i32,
    /// The algorithm of reverb.
    pub algorithm: ReverbAlgorithm,
}

impl ReverbState {
    pub const SIZE: usize = 68;

    pub fn build(&self, raw: &mut [u8]) {
        assert_eq!(
            raw.len(),
            Self::SIZE,
            "Programming error for the length of raw data"
        );

        self.input_level.build_quadlet(&mut raw[..4]);
        self.bypass.build_quadlet(&mut raw[4..8]);
        self.kill_wet.build_quadlet(&mut raw[8..12]);
        self.kill_dry.build_quadlet(&mut raw[12..16]);
        self.output_level.build_quadlet(&mut raw[16..20]);
        self.time_decay.build_quadlet(&mut raw[20..24]);
        self.time_pre_decay.build_quadlet(&mut raw[24..28]);
        // blank
        self.color_low.build_quadlet(&mut raw[32..36]);
        self.color_high.build_quadlet(&mut raw[36..40]);
        self.color_high_factor.build_quadlet(&mut raw[40..44]);
        self.mod_rate.build_quadlet(&mut raw[44..48]);
        self.mod_depth.build_quadlet(&mut raw[48..52]);
        self.level_early.build_quadlet(&mut raw[52..56]);
        self.level_dry.build_quadlet(&mut raw[60..64]);
        self.algorithm.build_quadlet(&mut raw[64..]);
    }

    pub fn parse(&mut self, raw: &[u8]) {
        assert_eq!(
            raw.len(),
            Self::SIZE,
            "Programming error for the length of raw data"
        );

        self.input_level.parse_quadlet(&raw[..4]);
        self.bypass.parse_quadlet(&raw[4..8]);
        self.kill_wet.parse_quadlet(&raw[8..12]);
        self.kill_dry.parse_quadlet(&raw[12..16]);
        self.output_level.parse_quadlet(&raw[16..20]);
        self.time_decay.parse_quadlet(&raw[20..24]);
        self.time_pre_decay.parse_quadlet(&raw[24..28]);
        // blank
        self.color_low.parse_quadlet(&raw[32..36]);
        self.color_high.parse_quadlet(&raw[36..40]);
        self.color_high_factor.parse_quadlet(&raw[40..44]);
        self.mod_rate.parse_quadlet(&raw[44..48]);
        self.mod_depth.parse_quadlet(&raw[48..52]);
        self.level_early.parse_quadlet(&raw[52..56]);
        self.level_dry.parse_quadlet(&raw[60..64]);
        self.algorithm.parse_quadlet(&raw[64..]);
    }
}

/// The structure to represent meter of reverb effect.
#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
pub struct ReverbMeter {
    /// The meter of left and right outputs. -1000..500 (-24.0..12.0 dB)
    pub outputs: [i32; 2],
    /// The meter of left and right inputs. -1000..0 (-24.0..0.0 dB)
    pub inputs: [i32; 2],
}

impl ReverbMeter {
    pub const SIZE: usize = 24;
}

impl ReverbMeter {
    pub fn build(&self, raw: &mut [u8]) {
        assert_eq!(
            raw.len(),
            Self::SIZE,
            "Programming error for the length of raw data"
        );

        self.outputs.build_quadlet_block(&mut raw[..8]);
        self.inputs.build_quadlet_block(&mut raw[8..16]);
    }

    pub fn parse(&mut self, raw: &[u8]) {
        assert_eq!(
            raw.len(),
            Self::SIZE,
            "Programming error for the length of raw data"
        );

        self.outputs.parse_quadlet_block(&raw[..8]);
        self.inputs.parse_quadlet_block(&raw[8..16]);
    }
}
