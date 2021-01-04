// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Data of channel strip effect in protocol defined by TC Electronic for Konnekt series.
//!
//! The module includes structure, trait and its implementation for data of channel strip effect in
//! protocol defined by TC Electronic for Konnekt series. It's called as `Fabrik C`.

use crate::*;

/// The enumeration to represent type of source.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
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
    Reserved(u32),
}

impl ChStripSrcType {
    const FEMALE_VOCAL: u32 = 0;
    const MALE_VOCAL: u32 = 1;
    const GUITAR: u32 = 2;
    const PIANO: u32 = 3;
    const SPEAK: u32 = 4;
    const CHOIR: u32 = 5;
    const HORNS: u32 = 6;
    const BASS: u32 = 7;
    const KICK: u32 = 8;
    const SNARE: u32 = 9;
    const MIX_ROCK: u32 = 10;
    const MIX_SOFT: u32 = 11;
    const PERCUSSION: u32 = 12;
    const KIT: u32 = 13;
    const MIX_ACOUSTIC: u32 = 14;
    const MIX_PURIST: u32 = 15;
    const HOUSE: u32 = 16;
    const TRANCE: u32 = 17;
    const CHILL: u32 = 18;
    const HIP_HOP: u32 = 19;
    const DRUM_AND_BASS: u32 = 20;
    const ELECTRO_TECHNO: u32 = 21;
}

impl Default for ChStripSrcType {
    fn default() -> Self {
        ChStripSrcType::FemaleVocal
    }
}

impl From<u32> for ChStripSrcType {
    fn from(val: u32) -> Self {
        match val {
            Self::FEMALE_VOCAL => Self::FemaleVocal,
            Self::MALE_VOCAL => Self::MaleVocal,
            Self::GUITAR => Self::Guitar,
            Self::PIANO => Self::Piano,
            Self::SPEAK => Self::Speak,
            Self::CHOIR => Self::Choir,
            Self::HORNS => Self::Horns,
            Self::BASS => Self::Bass,
            Self::KICK => Self::Kick,
            Self::SNARE => Self::Snare,
            Self::MIX_ROCK => Self::MixRock,
            Self::MIX_SOFT => Self::MixSoft,
            Self::PERCUSSION => Self::Percussion,
            Self::KIT => Self::Kit,
            Self::MIX_ACOUSTIC => Self::MixAcoustic,
            Self::MIX_PURIST => Self::MixPurist,
            Self::HOUSE => Self::House,
            Self::TRANCE => Self::Trance,
            Self::CHILL => Self::Chill,
            Self::HIP_HOP => Self::HipHop,
            Self::DRUM_AND_BASS => Self::DrumAndBass,
            Self::ELECTRO_TECHNO => Self::ElectroTechno,
            _ => Self::Reserved(val),
        }
    }
}

impl From<ChStripSrcType> for u32 {
    fn from(src_type: ChStripSrcType) -> Self {
        match src_type {
            ChStripSrcType::FemaleVocal => ChStripSrcType::FEMALE_VOCAL,
            ChStripSrcType::MaleVocal => ChStripSrcType::MALE_VOCAL,
            ChStripSrcType::Guitar => ChStripSrcType::GUITAR,
            ChStripSrcType::Piano => ChStripSrcType::PIANO,
            ChStripSrcType::Speak => ChStripSrcType::SPEAK,
            ChStripSrcType::Choir => ChStripSrcType::CHOIR,
            ChStripSrcType::Horns => ChStripSrcType::HORNS,
            ChStripSrcType::Bass => ChStripSrcType::BASS,
            ChStripSrcType::Kick => ChStripSrcType::KICK,
            ChStripSrcType::Snare => ChStripSrcType::SNARE,
            ChStripSrcType::MixRock => ChStripSrcType::MIX_ROCK,
            ChStripSrcType::MixSoft => ChStripSrcType::MIX_SOFT,
            ChStripSrcType::Percussion => ChStripSrcType::PERCUSSION,
            ChStripSrcType::Kit => ChStripSrcType::KIT,
            ChStripSrcType::MixAcoustic => ChStripSrcType::MIX_ACOUSTIC,
            ChStripSrcType::MixPurist => ChStripSrcType::MIX_PURIST,
            ChStripSrcType::House => ChStripSrcType::HOUSE,
            ChStripSrcType::Trance => ChStripSrcType::TRANCE,
            ChStripSrcType::Chill => ChStripSrcType::CHILL,
            ChStripSrcType::HipHop => ChStripSrcType::HIP_HOP,
            ChStripSrcType::DrumAndBass => ChStripSrcType::DRUM_AND_BASS,
            ChStripSrcType::ElectroTechno => ChStripSrcType::ELECTRO_TECHNO,
            ChStripSrcType::Reserved(val) => val,
        }
    }
}

/// The structure to represent state of compressor part.
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct CompState{
    /// The gain of input. 0..360 (-18.0..18.0 dB).
    pub input_gain: u32,
    /// The gain of output. 0..360 (-18.0..18.0 dB).
    pub make_up_gain: u32,
    /// Whether three bands are available or not.
    pub full_band_enabled: bool,
    /// The amount to control for low/mid/high frequencies. 0..200 (-100.0..+100.0 %)
    pub ctl: [u32;3],
    /// The level of low/mid/high frequencies. 0..48 (-18.0..+6.0 dB)
    pub level: [u32;3],
}

/// The structure to represent state of deesser part.
#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
pub struct DeesserState{
    /// The ratio to deesser. 0..10 (0..100 %)
    pub ratio: u32,
    pub bypass: bool,
}

/// The structure to represent state of equalizer part.
#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
pub struct EqState{
    pub enabled: bool,
    // The bandwidth. 0..39
    pub bandwidth: u32,
    // The gain. 0..240 (-12.0..+12.0 dB)
    pub gain: u32,
    // blank
    // The frequency. 0..240 (20.0..40.0 Hz)
    pub freq: u32,
}

/// The structure to represent state of limitter part.
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct LimitterState{
    /// The threshold to limit. 0..72 (-18.0..+18.0)
    pub threshold: u32,
}

/// The structure to represent state entry of channel strip effect.
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct ChStripState{
    pub src_type: ChStripSrcType,
    /// Compressor for low/mid/high frequencies.
    pub comp: CompState,
    /// Deesser.
    pub deesser: DeesserState,
    /// Equalizers for low/mid-low/mid-high/high frequencies.
    pub eq: [EqState;4],
    /// Whether to bypass equalizer or not.
    pub eq_bypass: bool,
    pub limitter: LimitterState,
    /// Whether to bypass limitter or not.
    pub limitter_bypass: bool,
    /// Whether to bypass whole parts or not.
    pub bypass: bool,
}

impl ChStripState {
    pub const SIZE: usize = 144;
}

pub fn calculate_ch_strip_state_segment_size(count: usize) -> usize {
    (((count + 1) / 2) * 4) + count + ChStripState::SIZE
}

fn calculate_ch_strip_state_segment_pos(idx: usize) -> usize {
    (((idx + 1) / 2) * 4) + idx * ChStripState::SIZE
}

/// The trait to represent conversion between data and raw.
pub trait ChStripStatesConvert {
    fn build(&self, raw: &mut [u8]);
    fn parse(&mut self, raw: &[u8]);
}

impl ChStripStatesConvert for [ChStripState] {
    fn build(&self, raw: &mut [u8]) {
        assert_eq!(raw.len(), calculate_ch_strip_state_segment_pos(self.len()),
                   "Programming error for the length of raw data");
        self.iter()
            .enumerate()
            .for_each(|(i, s)| {
                let pos = calculate_ch_strip_state_segment_pos(i);
                let r = &mut raw[pos..(pos + ChStripState::SIZE)];

                s.comp.input_gain.build_quadlet(&mut r[..4]);
                s.src_type.build_quadlet(&mut r[4..8]);
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
            });
    }

    fn parse(&mut self, raw: &[u8]) {
        assert_eq!(raw.len(), calculate_ch_strip_state_segment_pos(self.len()),
                   "Programming error for the length of raw data");
        self.iter_mut()
            .enumerate()
            .for_each(|(i, s)| {
                let pos = calculate_ch_strip_state_segment_pos(i);
                let r = &raw[pos..(pos + ChStripState::SIZE)];

                s.comp.input_gain.parse_quadlet(&r[..4]);
                s.src_type.parse_quadlet(&r[4..8]);
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
            });
    }
}

/// The structure to represent meter entry of channel strip effect.
#[derive(Default, Debug, Clone)]
pub struct ChStripMeter{
    /// Input meter. -72..0 (-72.0..0.0 dB)
    pub input: i32,
    /// Limit meter. -12..0 (-12.0..0.0 dB)
    pub limit: i32,
    /// Output meter. -72..0 (-72.0..0.0 dB)
    pub output: i32,
    /// Gain meter at low/middle/high frequency. -24..18 (-24.0..18.0 dB)
    pub gains: [i32;3],
}

impl ChStripMeter {
    pub const SIZE: usize = 28;
}

pub fn calculate_ch_strip_meter_segment_size(count: usize) -> usize {
    (((count + 1) / 2) * 4) + count * ChStripMeter::SIZE
}

fn calculate_ch_strip_meter_segment_pos(idx: usize) -> usize {
    (((idx + 1) / 2) * 4) + idx * ChStripMeter::SIZE
}

/// The trait to represent conversion between data and raw.
pub trait ChStripMetersConvert {
    fn build(&self, raw: &mut [u8]);
    fn parse(&mut self, raw: &[u8]);
}

impl ChStripMetersConvert for [ChStripMeter] {
    fn build(&self, raw: &mut [u8]) {
        assert_eq!(raw.len(), calculate_ch_strip_meter_segment_size(self.len()),
                   "Programming error for the length of raw data");
        self.iter()
            .enumerate()
            .for_each(|(i, m)| {
                let pos = calculate_ch_strip_meter_segment_pos(i);
                let r = &mut raw[pos..(pos + ChStripMeter::SIZE)];

                m.input.build_quadlet(&mut r[..4]);
                m.limit.build_quadlet(&mut r[4..8]);
                m.output.build_quadlet(&mut r[8..12]);
                m.gains.build_quadlet_block(&mut r[12..24]);
            });
    }

    fn parse(&mut self, raw: &[u8]) {
        assert_eq!(raw.len(), calculate_ch_strip_meter_segment_size(self.len()),
                   "Programming error for the length of raw data");
        self.iter_mut()
            .enumerate()
            .for_each(|(i, m)| {
                let pos = calculate_ch_strip_meter_segment_pos(i);
                let r = &raw[pos..(pos + ChStripMeter::SIZE)];

                m.input.parse_quadlet(&r[..4]);
                m.limit.parse_quadlet(&r[4..8]);
                m.output.parse_quadlet(&r[8..12]);
                m.gains.parse_quadlet_block(&r[12..24]);
            });
    }
}
