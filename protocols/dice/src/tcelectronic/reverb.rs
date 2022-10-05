// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Data of reverb effect in protocol defined by TC Electronic for Konnekt series.
//!
//! The module includes structure, trait and its implementation for data of reverb effect in
//! protocol defined by TC Electronic for Konnekt series. It's called as `Fabrik R`.

use super::*;

/// Algorithm of reverb effect.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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
}

impl Default for ReverbAlgorithm {
    fn default() -> Self {
        Self::Live1
    }
}

const REVERB_ALGORITHMS: &[ReverbAlgorithm] = &[
    ReverbAlgorithm::Live1,
    ReverbAlgorithm::Hall,
    ReverbAlgorithm::Plate,
    ReverbAlgorithm::Club,
    ReverbAlgorithm::ConcertHall,
    ReverbAlgorithm::Cathedral,
    ReverbAlgorithm::Church,
    ReverbAlgorithm::Room,
    ReverbAlgorithm::SmallRoom,
    ReverbAlgorithm::Box,
    ReverbAlgorithm::Ambient,
    ReverbAlgorithm::Live2,
    ReverbAlgorithm::Live3,
    ReverbAlgorithm::Spring,
];

const REVERB_ALGORITHM_LABEL: &str = "reverb algorithm";

fn serialize_algorithm(algo: &ReverbAlgorithm, raw: &mut [u8]) -> Result<(), String> {
    serialize_position(REVERB_ALGORITHMS, algo, raw, REVERB_ALGORITHM_LABEL)
}

fn deserialize_algorithm(algo: &mut ReverbAlgorithm, raw: &[u8]) -> Result<(), String> {
    deserialize_position(REVERB_ALGORITHMS, algo, raw, REVERB_ALGORITHM_LABEL)
}

/// State of reverb effect.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
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
    pub(crate) const SIZE: usize = 68;
}

pub(crate) fn serialize_reverb_state(state: &ReverbState, raw: &mut [u8]) -> Result<(), String> {
    assert!(raw.len() >= ReverbState::SIZE);

    state.input_level.build_quadlet(&mut raw[..4]);
    state.bypass.build_quadlet(&mut raw[4..8]);
    state.kill_wet.build_quadlet(&mut raw[8..12]);
    state.kill_dry.build_quadlet(&mut raw[12..16]);
    state.output_level.build_quadlet(&mut raw[16..20]);
    state.time_decay.build_quadlet(&mut raw[20..24]);
    state.time_pre_decay.build_quadlet(&mut raw[24..28]);
    // blank
    state.color_low.build_quadlet(&mut raw[32..36]);
    state.color_high.build_quadlet(&mut raw[36..40]);
    state.color_high_factor.build_quadlet(&mut raw[40..44]);
    state.mod_rate.build_quadlet(&mut raw[44..48]);
    state.mod_depth.build_quadlet(&mut raw[48..52]);
    state.level_early.build_quadlet(&mut raw[52..56]);
    state.level_dry.build_quadlet(&mut raw[60..64]);
    serialize_algorithm(&state.algorithm, &mut raw[64..])?;

    Ok(())
}

pub(crate) fn deserialize_reverb_state(state: &mut ReverbState, raw: &[u8]) -> Result<(), String> {
    assert!(raw.len() >= ReverbState::SIZE);

    state.input_level.parse_quadlet(&raw[..4]);
    state.bypass.parse_quadlet(&raw[4..8]);
    state.kill_wet.parse_quadlet(&raw[8..12]);
    state.kill_dry.parse_quadlet(&raw[12..16]);
    state.output_level.parse_quadlet(&raw[16..20]);
    state.time_decay.parse_quadlet(&raw[20..24]);
    state.time_pre_decay.parse_quadlet(&raw[24..28]);
    // blank
    state.color_low.parse_quadlet(&raw[32..36]);
    state.color_high.parse_quadlet(&raw[36..40]);
    state.color_high_factor.parse_quadlet(&raw[40..44]);
    state.mod_rate.parse_quadlet(&raw[44..48]);
    state.mod_depth.parse_quadlet(&raw[48..52]);
    state.level_early.parse_quadlet(&raw[52..56]);
    state.level_dry.parse_quadlet(&raw[60..64]);
    deserialize_algorithm(&mut state.algorithm, &raw[64..])?;

    Ok(())
}

/// Meter of reverb effect.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct ReverbMeter {
    /// The meter of left and right outputs. -1000..500 (-24.0..12.0 dB)
    pub outputs: [i32; 2],
    /// The meter of left and right inputs. -1000..0 (-24.0..0.0 dB)
    pub inputs: [i32; 2],
}

impl ReverbMeter {
    pub(crate) const SIZE: usize = 24;
}

pub(crate) fn serialize_reverb_meter(meter: &ReverbMeter, raw: &mut [u8]) -> Result<(), String> {
    assert!(raw.len() >= ReverbMeter::SIZE);

    meter.outputs.build_quadlet_block(&mut raw[..8]);
    meter.inputs.build_quadlet_block(&mut raw[8..16]);

    Ok(())
}

pub(crate) fn deserialize_reverb_meter(meter: &mut ReverbMeter, raw: &[u8]) -> Result<(), String> {
    assert!(raw.len() >= ReverbMeter::SIZE);

    meter.outputs.parse_quadlet_block(&raw[..8]);
    meter.inputs.parse_quadlet_block(&raw[8..16]);

    Ok(())
}
