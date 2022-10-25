// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Data of channel strip effect in protocol defined by TC Electronic for Konnekt series.
//!
//! The module includes structure, trait and its implementation for data of channel strip effect in
//! protocol defined by TC Electronic for Konnekt series. It's called as `Fabrik C`.

use super::*;

/// Type of source.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ChStripSrcType {
    FemaleVocal,
    MaleVocal,
    Guitar,
    Piano,
    Speak,
    Choir,
    Horns,
    Bass,
    Kick,
    Snare,
    MixRock,
    MixSoft,
    Percussion,
    Kit,
    MixAcoustic,
    MixPurist,
    House,
    Trance,
    Chill,
    HipHop,
    DrumAndBass,
    ElectroTechno,
}

impl Default for ChStripSrcType {
    fn default() -> Self {
        ChStripSrcType::FemaleVocal
    }
}

const CH_STRIP_SRC_TYPES: &[ChStripSrcType] = &[
    ChStripSrcType::FemaleVocal,
    ChStripSrcType::MaleVocal,
    ChStripSrcType::Guitar,
    ChStripSrcType::Piano,
    ChStripSrcType::Speak,
    ChStripSrcType::Choir,
    ChStripSrcType::Horns,
    ChStripSrcType::Bass,
    ChStripSrcType::Kick,
    ChStripSrcType::Snare,
    ChStripSrcType::MixRock,
    ChStripSrcType::MixSoft,
    ChStripSrcType::Percussion,
    ChStripSrcType::Kit,
    ChStripSrcType::MixAcoustic,
    ChStripSrcType::MixPurist,
    ChStripSrcType::House,
    ChStripSrcType::Trance,
    ChStripSrcType::Chill,
    ChStripSrcType::HipHop,
    ChStripSrcType::DrumAndBass,
    ChStripSrcType::ElectroTechno,
];

const CH_STRIP_SRC_TYPE_LABEL: &str = "channel strip source type";

fn serialize_src_type(src_type: &ChStripSrcType, raw: &mut [u8]) -> Result<(), String> {
    serialize_position(CH_STRIP_SRC_TYPES, src_type, raw, CH_STRIP_SRC_TYPE_LABEL)
}

fn deserialize_src_type(src_type: &mut ChStripSrcType, raw: &[u8]) -> Result<(), String> {
    deserialize_position(CH_STRIP_SRC_TYPES, src_type, raw, CH_STRIP_SRC_TYPE_LABEL)
}

/// State of compressor part.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct CompState {
    /// The gain of input. 0..360 (-18.0..18.0 dB).
    pub input_gain: u32,
    /// The gain of output. 0..360 (-18.0..18.0 dB).
    pub make_up_gain: u32,
    /// Whether three bands are available or not.
    pub full_band_enabled: bool,
    /// The amount to control for low/mid/high frequencies. 0..200 (-100.0..+100.0 %)
    pub ctl: [u32; 3],
    /// The level of low/mid/high frequencies. 0..48 (-18.0..+6.0 dB)
    pub level: [u32; 3],
}

/// State of deesser part.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct DeesserState {
    /// The ratio to deesser. 0..10 (0..100 %)
    pub ratio: u32,
    /// Whether to bypass deesser effect.
    pub bypass: bool,
}

/// State of equalizer part.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct EqState {
    /// Whether to enable equalizer.
    pub enabled: bool,
    /// The bandwidth. 0..39
    pub bandwidth: u32,
    /// The gain. 0..240 (-12.0..+12.0 dB)
    pub gain: u32,
    // blank
    /// The frequency. 0..240 (20.0..40.0 Hz)
    pub freq: u32,
}

/// State of limitter part.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct LimitterState {
    /// The threshold to limit. 0..72 (-18.0..+18.0)
    pub threshold: u32,
}

/// State entry of channel strip effect.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct ChStripState {
    pub src_type: ChStripSrcType,
    /// Compressor for low/mid/high frequencies.
    pub comp: CompState,
    /// Deesser.
    pub deesser: DeesserState,
    /// Equalizers for low/mid-low/mid-high/high frequencies.
    pub eq: [EqState; 4],
    /// Whether to bypass equalizer or not.
    pub eq_bypass: bool,
    /// Limitter.
    pub limitter: LimitterState,
    /// Whether to bypass limitter or not.
    pub limitter_bypass: bool,
    /// Whether to bypass whole parts or not.
    pub bypass: bool,
}

impl ChStripState {
    pub(crate) const SIZE: usize = 144;
}

pub(crate) fn calculate_ch_strip_state_segment_size(count: usize) -> usize {
    (((count + 1) / 2) * 4) + count + ChStripState::SIZE
}

fn calculate_ch_strip_state_segment_pos(idx: usize) -> usize {
    (((idx + 1) / 2) * 4) + idx * ChStripState::SIZE
}

pub(crate) fn serialize_ch_strip_states(
    states: &[ChStripState],
    raw: &mut [u8],
) -> Result<(), String> {
    assert!(raw.len() >= calculate_ch_strip_state_segment_size(states.len()));

    states.iter().enumerate().try_for_each(|(i, s)| {
        let pos = calculate_ch_strip_state_segment_pos(i);
        let r = &mut raw[pos..(pos + ChStripState::SIZE)];

        serialize_u32(&s.comp.input_gain, &mut r[..4]);
        serialize_src_type(&s.src_type, &mut r[4..8])?;
        serialize_bool(&s.comp.full_band_enabled, &mut r[8..12]);
        serialize_u32(&s.deesser.ratio, &mut r[12..16]);
        serialize_bool(&s.deesser.bypass, &mut r[16..20]);
        serialize_bool(&s.eq[0].enabled, &mut r[20..24]);
        serialize_u32(&s.eq[0].bandwidth, &mut r[24..28]);
        serialize_u32(&s.eq[0].gain, &mut r[28..32]);
        // blank
        serialize_u32(&s.eq[0].freq, &mut r[36..40]);
        serialize_bool(&s.eq[1].enabled, &mut r[40..44]);
        serialize_u32(&s.eq[1].bandwidth, &mut r[44..48]);
        serialize_u32(&s.eq[1].gain, &mut r[48..52]);
        // blank
        serialize_u32(&s.eq[1].freq, &mut r[56..60]);
        serialize_bool(&s.eq[2].enabled, &mut r[60..64]);
        serialize_u32(&s.eq[2].bandwidth, &mut r[64..68]);
        serialize_u32(&s.eq[2].gain, &mut r[68..72]);
        // blank
        serialize_u32(&s.eq[2].freq, &mut r[76..80]);
        serialize_bool(&s.eq[3].enabled, &mut r[80..84]);
        serialize_u32(&s.eq[3].bandwidth, &mut r[84..88]);
        serialize_u32(&s.eq[3].gain, &mut r[88..92]);
        // blank
        serialize_u32(&s.eq[3].freq, &mut r[96..100]);
        serialize_bool(&s.eq_bypass, &mut r[100..104]);
        serialize_u32(&s.comp.ctl[0], &mut r[104..108]);
        serialize_u32(&s.comp.level[0], &mut r[108..112]);
        serialize_u32(&s.comp.ctl[1], &mut r[112..116]);
        serialize_u32(&s.comp.level[1], &mut r[116..120]);
        serialize_u32(&s.comp.ctl[2], &mut r[120..124]);
        serialize_u32(&s.comp.level[2], &mut r[124..128]);
        serialize_bool(&s.limitter_bypass, &mut r[128..132]);
        serialize_u32(&s.comp.make_up_gain, &mut r[132..136]);
        serialize_u32(&s.limitter.threshold, &mut r[136..140]);
        serialize_bool(&s.bypass, &mut r[140..]);

        Ok(())
    })
}

pub(crate) fn deserialize_ch_strip_states(
    states: &mut [ChStripState],
    raw: &[u8],
) -> Result<(), String> {
    assert!(raw.len() >= calculate_ch_strip_state_segment_size(states.len()));

    states.iter_mut().enumerate().try_for_each(|(i, s)| {
        let pos = calculate_ch_strip_state_segment_pos(i);
        let r = &raw[pos..(pos + ChStripState::SIZE)];

        deserialize_u32(&mut s.comp.input_gain, &r[..4]);
        deserialize_src_type(&mut s.src_type, &r[4..8])?;
        deserialize_bool(&mut s.comp.full_band_enabled, &r[8..12]);
        deserialize_u32(&mut s.deesser.ratio, &r[12..16]);
        deserialize_bool(&mut s.deesser.bypass, &r[16..20]);
        deserialize_bool(&mut s.eq[0].enabled, &r[20..24]);
        deserialize_u32(&mut s.eq[0].bandwidth, &r[24..28]);
        deserialize_u32(&mut s.eq[0].gain, &r[28..32]);
        // blank
        deserialize_u32(&mut s.eq[0].freq, &r[36..40]);
        deserialize_bool(&mut s.eq[1].enabled, &r[40..44]);
        deserialize_u32(&mut s.eq[1].bandwidth, &r[44..48]);
        deserialize_u32(&mut s.eq[1].gain, &r[48..52]);
        // blank
        deserialize_u32(&mut s.eq[1].freq, &r[56..60]);
        deserialize_bool(&mut s.eq[2].enabled, &r[60..64]);
        deserialize_u32(&mut s.eq[2].bandwidth, &r[64..68]);
        deserialize_u32(&mut s.eq[2].gain, &r[68..72]);
        // blank
        deserialize_u32(&mut s.eq[2].freq, &r[76..80]);
        deserialize_bool(&mut s.eq[3].enabled, &r[80..84]);
        deserialize_u32(&mut s.eq[3].bandwidth, &r[84..88]);
        deserialize_u32(&mut s.eq[3].gain, &r[88..92]);
        // blank
        deserialize_u32(&mut s.eq[3].freq, &r[96..100]);
        deserialize_bool(&mut s.eq_bypass, &r[100..104]);
        deserialize_u32(&mut s.comp.ctl[0], &r[104..108]);
        deserialize_u32(&mut s.comp.level[0], &r[108..112]);
        deserialize_u32(&mut s.comp.ctl[1], &r[112..116]);
        deserialize_u32(&mut s.comp.level[1], &r[116..120]);
        deserialize_u32(&mut s.comp.ctl[2], &r[120..124]);
        deserialize_u32(&mut s.comp.level[2], &r[124..128]);
        deserialize_bool(&mut s.limitter_bypass, &r[128..132]);
        deserialize_u32(&mut s.comp.make_up_gain, &r[132..136]);
        deserialize_u32(&mut s.limitter.threshold, &r[136..140]);
        deserialize_bool(&mut s.bypass, &r[140..]);

        Ok(())
    })
}

/// Meter entry of channel strip effect.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct ChStripMeter {
    /// Input meter. -72..0 (-72.0..0.0 dB)
    pub input: i32,
    /// Limit meter. -12..0 (-12.0..0.0 dB)
    pub limit: i32,
    /// Output meter. -72..0 (-72.0..0.0 dB)
    pub output: i32,
    /// Gain meter at low/middle/high frequency. -24..18 (-24.0..18.0 dB)
    pub gains: [i32; 3],
}

impl ChStripMeter {
    pub(crate) const SIZE: usize = 28;
}

pub(crate) fn calculate_ch_strip_meter_segment_size(count: usize) -> usize {
    (((count + 1) / 2) * 4) + count * ChStripMeter::SIZE
}

fn calculate_ch_strip_meter_segment_pos(idx: usize) -> usize {
    (((idx + 1) / 2) * 4) + idx * ChStripMeter::SIZE
}

pub(crate) fn serialize_ch_strip_meters(
    meters: &[ChStripMeter],
    raw: &mut [u8],
) -> Result<(), String> {
    assert!(raw.len() >= calculate_ch_strip_meter_segment_size(meters.len()));

    meters.iter().enumerate().try_for_each(|(i, m)| {
        let pos = calculate_ch_strip_meter_segment_pos(i);
        let r = &mut raw[pos..(pos + ChStripMeter::SIZE)];

        serialize_i32(&m.input, &mut r[..4]);
        serialize_i32(&m.limit, &mut r[4..8]);
        serialize_i32(&m.output, &mut r[8..12]);
        serialize_i32(&m.gains[0], &mut r[20..24]);
        serialize_i32(&m.gains[1], &mut r[16..20]);
        serialize_i32(&m.gains[2], &mut r[12..16]);

        Ok(())
    })
}

pub(crate) fn deserialize_ch_strip_meters(
    meters: &mut [ChStripMeter],
    raw: &[u8],
) -> Result<(), String> {
    assert!(raw.len() >= calculate_ch_strip_meter_segment_size(meters.len()));

    meters.iter_mut().enumerate().try_for_each(|(i, m)| {
        let pos = calculate_ch_strip_meter_segment_pos(i);
        let r = &raw[pos..(pos + ChStripMeter::SIZE)];

        deserialize_i32(&mut m.input, &r[..4]);
        deserialize_i32(&mut m.limit, &r[4..8]);
        deserialize_i32(&mut m.output, &r[8..12]);
        deserialize_i32(&mut m.gains[0], &r[20..24]);
        deserialize_i32(&mut m.gains[1], &r[16..20]);
        deserialize_i32(&mut m.gains[2], &r[12..16]);

        Ok(())
    })
}
