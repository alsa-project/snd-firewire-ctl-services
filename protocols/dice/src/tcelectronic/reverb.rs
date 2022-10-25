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

    serialize_i32(&state.input_level, &mut raw[..4]);
    serialize_bool(&state.bypass, &mut raw[4..8]);
    serialize_bool(&state.kill_wet, &mut raw[8..12]);
    serialize_bool(&state.kill_dry, &mut raw[12..16]);
    serialize_i32(&state.output_level, &mut raw[16..20]);
    serialize_i32(&state.time_decay, &mut raw[20..24]);
    serialize_i32(&state.time_pre_decay, &mut raw[24..28]);
    // blank
    serialize_i32(&state.color_low, &mut raw[32..36]);
    serialize_i32(&state.color_high, &mut raw[36..40]);
    serialize_i32(&state.color_high_factor, &mut raw[40..44]);
    serialize_i32(&state.mod_rate, &mut raw[44..48]);
    serialize_i32(&state.mod_depth, &mut raw[48..52]);
    serialize_i32(&state.level_early, &mut raw[52..56]);
    serialize_i32(&state.level_dry, &mut raw[60..64]);
    serialize_algorithm(&state.algorithm, &mut raw[64..])?;

    Ok(())
}

pub(crate) fn deserialize_reverb_state(state: &mut ReverbState, raw: &[u8]) -> Result<(), String> {
    assert!(raw.len() >= ReverbState::SIZE);

    deserialize_i32(&mut state.input_level, &raw[..4]);
    deserialize_bool(&mut state.bypass, &raw[4..8]);
    deserialize_bool(&mut state.kill_wet, &raw[8..12]);
    deserialize_bool(&mut state.kill_dry, &raw[12..16]);
    deserialize_i32(&mut state.output_level, &raw[16..20]);
    deserialize_i32(&mut state.time_decay, &raw[20..24]);
    deserialize_i32(&mut state.time_pre_decay, &raw[24..28]);
    // blank
    deserialize_i32(&mut state.color_low, &raw[32..36]);
    deserialize_i32(&mut state.color_high, &raw[36..40]);
    deserialize_i32(&mut state.color_high_factor, &raw[40..44]);
    deserialize_i32(&mut state.mod_rate, &raw[44..48]);
    deserialize_i32(&mut state.mod_depth, &raw[48..52]);
    deserialize_i32(&mut state.level_early, &raw[52..56]);
    deserialize_i32(&mut state.level_dry, &raw[60..64]);
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

    serialize_i32(&meter.outputs[0], &mut raw[..4]);
    serialize_i32(&meter.outputs[1], &mut raw[4..8]);
    serialize_i32(&meter.inputs[0], &mut raw[8..12]);
    serialize_i32(&meter.inputs[1], &mut raw[12..16]);

    Ok(())
}

pub(crate) fn deserialize_reverb_meter(meter: &mut ReverbMeter, raw: &[u8]) -> Result<(), String> {
    assert!(raw.len() >= ReverbMeter::SIZE);

    deserialize_i32(&mut meter.outputs[0], &raw[..4]);
    deserialize_i32(&mut meter.outputs[1], &raw[4..8]);
    deserialize_i32(&mut meter.inputs[0], &raw[8..12]);
    deserialize_i32(&mut meter.inputs[1], &raw[12..16]);

    Ok(())
}
