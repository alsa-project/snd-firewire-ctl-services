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

        s.comp.input_gain.build_quadlet(&mut r[..4]);
        serialize_src_type(&s.src_type, &mut r[4..8])?;
        s.comp.full_band_enabled.build_quadlet(&mut r[8..12]);
        s.deesser.ratio.build_quadlet(&mut r[12..16]);
        s.deesser.bypass.build_quadlet(&mut r[16..20]);
        s.eq[0].enabled.build_quadlet(&mut r[20..24]);
        s.eq[0].bandwidth.build_quadlet(&mut r[24..28]);
        s.eq[0].gain.build_quadlet(&mut r[28..32]);
        // blank
        s.eq[0].freq.build_quadlet(&mut r[36..40]);
        s.eq[1].enabled.build_quadlet(&mut r[40..44]);
        s.eq[1].bandwidth.build_quadlet(&mut r[44..48]);
        s.eq[1].gain.build_quadlet(&mut r[48..52]);
        // blank
        s.eq[1].freq.build_quadlet(&mut r[56..60]);
        s.eq[2].enabled.build_quadlet(&mut r[60..64]);
        s.eq[2].bandwidth.build_quadlet(&mut r[64..68]);
        s.eq[2].gain.build_quadlet(&mut r[68..72]);
        // blank
        s.eq[2].freq.build_quadlet(&mut r[76..80]);
        s.eq[3].enabled.build_quadlet(&mut r[80..84]);
        s.eq[3].bandwidth.build_quadlet(&mut r[84..88]);
        s.eq[3].gain.build_quadlet(&mut r[88..92]);
        // blank
        s.eq[3].freq.build_quadlet(&mut r[96..100]);
        s.eq_bypass.build_quadlet(&mut r[100..104]);
        s.comp.ctl[0].build_quadlet(&mut r[104..108]);
        s.comp.level[0].build_quadlet(&mut r[108..112]);
        s.comp.ctl[1].build_quadlet(&mut r[112..116]);
        s.comp.level[1].build_quadlet(&mut r[116..120]);
        s.comp.ctl[2].build_quadlet(&mut r[120..124]);
        s.comp.level[2].build_quadlet(&mut r[124..128]);
        s.limitter_bypass.build_quadlet(&mut r[128..132]);
        s.comp.make_up_gain.build_quadlet(&mut r[132..136]);
        s.limitter.threshold.build_quadlet(&mut r[136..140]);
        s.bypass.build_quadlet(&mut r[140..]);

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

        s.comp.input_gain.parse_quadlet(&r[..4]);
        deserialize_src_type(&mut s.src_type, &r[4..8])?;
        s.comp.full_band_enabled.parse_quadlet(&r[8..12]);
        s.deesser.ratio.parse_quadlet(&r[12..16]);
        s.deesser.bypass.parse_quadlet(&r[16..20]);
        s.eq[0].enabled.parse_quadlet(&r[20..24]);
        s.eq[0].bandwidth.parse_quadlet(&r[24..28]);
        s.eq[0].gain.parse_quadlet(&r[28..32]);
        // blank
        s.eq[0].freq.parse_quadlet(&r[36..40]);
        s.eq[1].enabled.parse_quadlet(&r[40..44]);
        s.eq[1].bandwidth.parse_quadlet(&r[44..48]);
        s.eq[1].gain.parse_quadlet(&r[48..52]);
        // blank
        s.eq[1].freq.parse_quadlet(&r[56..60]);
        s.eq[2].enabled.parse_quadlet(&r[60..64]);
        s.eq[2].bandwidth.parse_quadlet(&r[64..68]);
        s.eq[2].gain.parse_quadlet(&r[68..72]);
        // blank
        s.eq[2].freq.parse_quadlet(&r[76..80]);
        s.eq[3].enabled.parse_quadlet(&r[80..84]);
        s.eq[3].bandwidth.parse_quadlet(&r[84..88]);
        s.eq[3].gain.parse_quadlet(&r[88..92]);
        // blank
        s.eq[3].freq.parse_quadlet(&r[96..100]);
        s.eq_bypass.parse_quadlet(&r[100..104]);
        s.comp.ctl[0].parse_quadlet(&r[104..108]);
        s.comp.level[0].parse_quadlet(&r[108..112]);
        s.comp.ctl[1].parse_quadlet(&r[112..116]);
        s.comp.level[1].parse_quadlet(&r[116..120]);
        s.comp.ctl[2].parse_quadlet(&r[120..124]);
        s.comp.level[2].parse_quadlet(&r[124..128]);
        s.limitter_bypass.parse_quadlet(&r[128..132]);
        s.comp.make_up_gain.parse_quadlet(&r[132..136]);
        s.limitter.threshold.parse_quadlet(&r[136..140]);
        s.bypass.parse_quadlet(&r[140..]);

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

        m.input.build_quadlet(&mut r[..4]);
        m.limit.build_quadlet(&mut r[4..8]);
        m.output.build_quadlet(&mut r[8..12]);
        m.gains[0].build_quadlet(&mut r[20..24]);
        m.gains[1].build_quadlet(&mut r[16..20]);
        m.gains[2].build_quadlet(&mut r[12..16]);

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

        m.input.parse_quadlet(&r[..4]);
        m.limit.parse_quadlet(&r[4..8]);
        m.output.parse_quadlet(&r[8..12]);
        m.gains[0].parse_quadlet(&r[20..24]);
        m.gains[1].parse_quadlet(&r[16..20]);
        m.gains[2].parse_quadlet(&r[12..16]);

        Ok(())
    })
}
