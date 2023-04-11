// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol for hardware mixer function operated by command.
//!
//! The module includes structure, enumeration, and trait for hardware mixer function operated by
//! command.
//!
//! The hardware transfers asynchronous packet to registered address when changing its state by
//! user operation. The packet includes some command to express the change of status. The
//! hardware also accepts the same command in asynchronous packet arrived at specific address.

use {
    super::*,
    hinawa::{prelude::FwRespExt, FwResp},
};

const DSP_CMD_OFFSET: u64 = 0xffff00010000;
const DSP_MSG_DST_HIGH_OFFSET: u32 = 0x0b38;
const DSP_MSG_DST_LOW_OFFSET: u32 = 0x0b3c;

const MAXIMUM_DSP_FRAME_SIZE: usize = 248;

const CMD_RESOURCE: u8 = 0x23;
const CMD_BYTE_MULTIPLE: u8 = 0x49;
const CMD_QUADLET_MULTIPLE: u8 = 0x46;
const CMD_DRAIN: u8 = 0x62;
const CMD_END: u8 = 0x65;
const CMD_BYTE_SINGLE: u8 = 0x69;
const CMD_QUADLET_SINGLE: u8 = 0x66;

const CMD_RESOURCE_LENGTH: usize = 6;
const CMD_BYTE_SINGLE_LENGTH: usize = 6;
const CMD_QUADLET_SINGLE_LENGTH: usize = 9;

const MSG_DST_OFFSET_BEGIN: u64 = 0xffffe0000000;
const MSG_DST_OFFSET_END: u64 = MSG_DST_OFFSET_BEGIN + 0x10000000;

/// The mode of stereo-paired channels.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum InputStereoPairMode {
    /// Adjustable left/right balance.
    LeftRight,
    /// Adjustable monaural/stereo balance.
    MonauralStereo,
    Reserved(u8),
}

impl Default for InputStereoPairMode {
    fn default() -> Self {
        InputStereoPairMode::LeftRight
    }
}

impl From<u8> for InputStereoPairMode {
    fn from(val: u8) -> Self {
        match val {
            0 => Self::LeftRight,
            1 => Self::MonauralStereo,
            _ => Self::Reserved(val),
        }
    }
}

impl From<InputStereoPairMode> for u8 {
    fn from(mode: InputStereoPairMode) -> Self {
        match mode {
            InputStereoPairMode::LeftRight => 0,
            InputStereoPairMode::MonauralStereo => 1,
            InputStereoPairMode::Reserved(val) => val,
        }
    }
}

/// The level to decline audio signal.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum RollOffLevel {
    /// 6 dB per octave.
    L6,
    /// 12 dB per octave.
    L12,
    /// 18 dB per octave.
    L18,
    /// 24 dB per octave.
    L24,
    /// 30 dB per octave.
    L30,
    /// 36 dB per octave.
    L36,
    Reserved(u8),
}

impl Default for RollOffLevel {
    fn default() -> Self {
        Self::L6
    }
}

impl From<u8> for RollOffLevel {
    fn from(val: u8) -> Self {
        match val {
            0 => Self::L6,
            1 => Self::L12,
            2 => Self::L18,
            3 => Self::L24,
            4 => Self::L30,
            5 => Self::L36,
            _ => Self::Reserved(val),
        }
    }
}

impl From<RollOffLevel> for u8 {
    fn from(level: RollOffLevel) -> Self {
        match level {
            RollOffLevel::L6 => 0,
            RollOffLevel::L12 => 1,
            RollOffLevel::L18 => 2,
            RollOffLevel::L24 => 3,
            RollOffLevel::L30 => 4,
            RollOffLevel::L36 => 5,
            RollOffLevel::Reserved(val) => val,
        }
    }
}

/// The type of filter for equalizer (5 options).
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum FilterType5 {
    T1,
    T2,
    T3,
    T4,
    Shelf,
    Reserved(u8),
}

impl Default for FilterType5 {
    fn default() -> Self {
        Self::T1
    }
}

impl From<u8> for FilterType5 {
    fn from(val: u8) -> Self {
        match val {
            0 => Self::T1,
            1 => Self::T2,
            2 => Self::T3,
            3 => Self::T4,
            4 => Self::Shelf,
            _ => Self::Reserved(val),
        }
    }
}

impl From<FilterType5> for u8 {
    fn from(filter_type: FilterType5) -> Self {
        match filter_type {
            FilterType5::T1 => 0,
            FilterType5::T2 => 1,
            FilterType5::T3 => 2,
            FilterType5::T4 => 3,
            FilterType5::Shelf => 4,
            FilterType5::Reserved(val) => val,
        }
    }
}

/// The type of filter for equalizer (5 options).
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum FilterType4 {
    T1,
    T2,
    T3,
    T4,
    Reserved(u8),
}

impl Default for FilterType4 {
    fn default() -> Self {
        Self::T1
    }
}

impl From<u8> for FilterType4 {
    fn from(val: u8) -> Self {
        match val {
            0 => Self::T1,
            1 => Self::T2,
            2 => Self::T3,
            3 => Self::T4,
            _ => Self::Reserved(val),
        }
    }
}

impl From<FilterType4> for u8 {
    fn from(filter_type: FilterType4) -> Self {
        match filter_type {
            FilterType4::T1 => 0,
            FilterType4::T2 => 1,
            FilterType4::T3 => 2,
            FilterType4::T4 => 3,
            FilterType4::Reserved(val) => val,
        }
    }
}

/// The way to decide loudness level of input signal.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum LevelDetectMode {
    /// According to the peak of signal.
    Peak,
    /// According to the Root Mean Square of signal.
    Rms,
    Reserved(u8),
}

impl Default for LevelDetectMode {
    fn default() -> Self {
        Self::Peak
    }
}

impl From<u8> for LevelDetectMode {
    fn from(val: u8) -> Self {
        match val {
            0 => Self::Peak,
            1 => Self::Rms,
            _ => Self::Reserved(val),
        }
    }
}

impl From<LevelDetectMode> for u8 {
    fn from(mode: LevelDetectMode) -> Self {
        match mode {
            LevelDetectMode::Peak => 0,
            LevelDetectMode::Rms => 1,
            LevelDetectMode::Reserved(val) => val,
        }
    }
}

/// The mode of leveler.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum LevelerMode {
    Compress,
    Limit,
    Reserved(u8),
}

impl Default for LevelerMode {
    fn default() -> Self {
        LevelerMode::Compress
    }
}

impl From<u8> for LevelerMode {
    fn from(val: u8) -> Self {
        match val {
            0 => Self::Compress,
            1 => Self::Limit,
            _ => Self::Reserved(val),
        }
    }
}

impl From<LevelerMode> for u8 {
    fn from(mode: LevelerMode) -> Self {
        match mode {
            LevelerMode::Compress => 0,
            LevelerMode::Limit => 1,
            LevelerMode::Reserved(val) => val,
        }
    }
}

/// The DSP command specific to equalizer effects.
#[derive(Debug, Clone, PartialEq)]
pub enum EqualizerParameter {
    Enable(bool),
    HpfEnable(bool),
    HpfSlope(RollOffLevel),
    HpfFreq(u32),
    LpfEnable(bool),
    LpfSlope(RollOffLevel),
    LpfFreq(u32),
    LfEnable(bool),
    LfType(FilterType5),
    LfFreq(u32),
    LfGain(f32),
    LfWidth(f32),
    LmfEnable(bool),
    LmfType(FilterType4),
    LmfFreq(u32),
    LmfGain(f32),
    LmfWidth(f32),
    MfEnable(bool),
    MfType(FilterType4),
    MfFreq(u32),
    MfGain(f32),
    MfWidth(f32),
    HmfEnable(bool),
    HmfType(FilterType4),
    HmfFreq(u32),
    HmfGain(f32),
    HmfWidth(f32),
    HfEnable(bool),
    HfType(FilterType5),
    HfFreq(u32),
    HfGain(f32),
    HfWidth(f32),
}

/// The DSP command specific to dynamics effects.
#[derive(Debug, Clone, PartialEq)]
pub enum DynamicsParameter {
    Enable(bool),
    CompEnable(bool),
    CompDetectMode(LevelDetectMode),
    CompThreshold(i32),
    CompRatio(f32),
    CompAttack(u32),
    CompRelease(u32),
    CompGain(f32),
    LevelerEnable(bool),
    LevelerMode(LevelerMode),
    LevelerMakeup(u32),
    LevelerReduce(u32),
}

fn to_bool(raw: &[u8]) -> bool {
    assert_eq!(raw.len(), 1);

    raw[0] > 0
}

fn to_usize(raw: &[u8]) -> usize {
    assert_eq!(raw.len(), 1);

    raw[0] as usize
}

fn to_i32(raw: &[u8]) -> i32 {
    to_f32(raw) as i32
}

fn to_f32(raw: &[u8]) -> f32 {
    assert_eq!(raw.len(), 4);

    let mut quadlet = [0; 4];
    quadlet.copy_from_slice(raw);

    f32::from_le_bytes(quadlet)
}

fn to_u32(raw: &[u8]) -> u32 {
    to_f32(raw) as u32
}

fn append_data(raw: &mut Vec<u8>, identifier: &[u8], vals: &[u8]) {
    if vals.len() == 4 {
        raw.push(CMD_QUADLET_SINGLE);
        raw.extend_from_slice(identifier);
        raw.extend_from_slice(vals);
    } else {
        raw.push(CMD_BYTE_SINGLE);
        raw.extend_from_slice(vals);
        raw.extend_from_slice(identifier);
    }
}

/// Target of focus.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum FocusTarget {
    Output(usize),
    Input(usize),
    Reserved(usize, usize),
}

impl Default for FocusTarget {
    fn default() -> Self {
        Self::Output(0)
    }
}

impl From<&[u8]> for FocusTarget {
    fn from(raw: &[u8]) -> Self {
        match raw[3] {
            0x01 => Self::Input(raw[0] as usize),
            0x03 => Self::Output(raw[0] as usize),
            _ => Self::Reserved(raw[3] as usize, raw[0] as usize),
        }
    }
}

impl From<&FocusTarget> for Vec<u8> {
    fn from(target: &FocusTarget) -> Self {
        match target {
            FocusTarget::Input(ch) => vec![*ch as u8, 0x00, 0x00, 0x01],
            FocusTarget::Output(ch) => vec![*ch as u8, 0x00, 0x00, 0x03],
            FocusTarget::Reserved(dir, ch) => vec![*ch as u8, 0x00, 0x00, *dir as u8],
        }
    }
}

/// The DSP command specific to master output.
#[derive(Debug, Clone, PartialEq)]
pub enum MonitorCmd {
    Volume(f32),
    TalkbackEnable(bool),
    ListenbackEnable(bool),
    TalkbackVolume(f32),
    ListenbackVolume(f32),
    Focus(FocusTarget),
    ReturnAssign(usize),
    Reserved(Vec<u8>, Vec<u8>),
}

impl MonitorCmd {
    fn parse(identifier: &[u8], vals: &[u8]) -> Self {
        assert_eq!(identifier.len(), 4);
        assert!(vals.len() > 0);

        match (identifier[3], identifier[2], identifier[1]) {
            (0x00, 0x00, 0x00) => MonitorCmd::Volume(to_f32(vals)),
            (0x00, 0x00, 0x01) => MonitorCmd::TalkbackEnable(to_bool(vals)),
            (0x00, 0x00, 0x02) => MonitorCmd::ListenbackEnable(to_bool(vals)),
            // TODO: model dependent, I guess.
            // (0, 0, 3) => u8
            // (0, 0, 4) => u8
            (0x00, 0x00, 0x05) => MonitorCmd::TalkbackVolume(to_f32(vals)),
            (0x00, 0x00, 0x06) => MonitorCmd::ListenbackVolume(to_f32(vals)),
            (0x00, 0x00, 0x07) => MonitorCmd::Focus(FocusTarget::from(vals)),
            (0x00, 0x00, 0x08) => MonitorCmd::ReturnAssign(to_usize(vals)),
            _ => MonitorCmd::Reserved(identifier.to_vec(), vals.to_vec()),
        }
    }

    fn build(&self, raw: &mut Vec<u8>) {
        match self {
            MonitorCmd::Volume(val) => append_f32(raw, 0x00, 0x00, 0x00, 0, *val),
            MonitorCmd::TalkbackEnable(val) => append_u8(raw, 0x00, 0x00, 0x01, 0, *val as u8),
            MonitorCmd::ListenbackEnable(val) => append_u8(raw, 0x00, 0x00, 0x02, 0, *val as u8),
            MonitorCmd::TalkbackVolume(val) => append_f32(raw, 0x00, 0x00, 0x05, 0, *val),
            MonitorCmd::ListenbackVolume(val) => append_f32(raw, 0x00, 0x00, 0x06, 0, *val),
            MonitorCmd::Focus(target) => {
                append_data(raw, &[0x00, 0x07, 0x00, 0x00], &Vec::from(target))
            }
            MonitorCmd::ReturnAssign(target) => append_u8(raw, 0x00, 0x00, 0x08, 0, *target as u8),
            MonitorCmd::Reserved(identifier, vals) => append_data(raw, identifier, vals),
        }
    }
}

/// The DSP command specific to input.
#[derive(Debug, Clone, PartialEq)]
pub enum InputCmd {
    Phase(usize, bool),
    Pair(usize, bool),
    Gain(usize, i32),
    Swap(usize, bool),
    StereoMode(usize, InputStereoPairMode),
    Width(usize, f32),
    Equalizer(usize, EqualizerParameter),
    Dynamics(usize, DynamicsParameter),
    ReverbSend(usize, f32),
    ReverbLrBalance(usize, f32),
    Pad(usize, bool),
    NominalLevel(usize, NominalSignalLevel),
    Phantom(usize, bool),
    Limitter(usize, bool),
    Lookahead(usize, bool),
    Softclip(usize, bool),
    Reserved(Vec<u8>, Vec<u8>),
}

impl InputCmd {
    fn parse(identifier: &[u8], vals: &[u8]) -> Self {
        assert_eq!(identifier.len(), 4);
        assert!(vals.len() > 0);

        let ch = identifier[0] as usize;

        match (identifier[3], identifier[2], identifier[1]) {
            (0x01, 0x00, 0x00) => InputCmd::Phase(ch, to_bool(vals)),
            (0x01, 0x00, 0x01) => InputCmd::Pair(ch, to_bool(vals)),
            (0x01, 0x00, 0x02) => InputCmd::Gain(ch, to_i32(vals)),
            (0x01, 0x00, 0x03) => InputCmd::Swap(ch, to_bool(vals)),
            (0x01, 0x00, 0x04) => InputCmd::StereoMode(ch, InputStereoPairMode::from(vals[0])),
            (0x01, 0x00, 0x05) => InputCmd::Width(ch, to_f32(vals)),
            (0x01, 0x00, 0x06) => InputCmd::Limitter(ch, to_bool(vals)),
            (0x01, 0x00, 0x07) => InputCmd::Lookahead(ch, to_bool(vals)),
            (0x01, 0x00, 0x08) => InputCmd::Softclip(ch, to_bool(vals)),
            (0x01, 0x00, 0x09) => InputCmd::Pad(ch, to_bool(vals)),
            (0x01, 0x00, 0x0a) => {
                let level = if to_bool(vals) {
                    NominalSignalLevel::Professional
                } else {
                    NominalSignalLevel::Consumer
                };
                InputCmd::NominalLevel(ch, level)
            }
            (0x01, 0x00, 0x0b) => InputCmd::Phantom(ch, to_bool(vals)),

            (0x01, 0x01, 0x00) => {
                InputCmd::Equalizer(ch, EqualizerParameter::Enable(to_bool(vals)))
            }

            (0x01, 0x02, 0x00) => {
                InputCmd::Equalizer(ch, EqualizerParameter::HpfEnable(to_bool(vals)))
            }
            (0x01, 0x02, 0x01) => InputCmd::Equalizer(
                ch,
                EqualizerParameter::HpfSlope(RollOffLevel::from(vals[0])),
            ),
            (0x01, 0x02, 0x02) => {
                InputCmd::Equalizer(ch, EqualizerParameter::HpfFreq(to_u32(vals)))
            }

            (0x01, 0x03, 0x00) => {
                InputCmd::Equalizer(ch, EqualizerParameter::LfEnable(to_bool(vals)))
            }
            (0x01, 0x03, 0x01) => {
                InputCmd::Equalizer(ch, EqualizerParameter::LfType(FilterType5::from(vals[0])))
            }
            (0x01, 0x03, 0x02) => InputCmd::Equalizer(ch, EqualizerParameter::LfFreq(to_u32(vals))),
            (0x01, 0x03, 0x03) => InputCmd::Equalizer(ch, EqualizerParameter::LfGain(to_f32(vals))),
            (0x01, 0x03, 0x04) => {
                InputCmd::Equalizer(ch, EqualizerParameter::LfWidth(to_f32(vals)))
            }

            (0x01, 0x04, 0x00) => {
                InputCmd::Equalizer(ch, EqualizerParameter::LmfEnable(to_bool(vals)))
            }
            (0x01, 0x04, 0x01) => {
                InputCmd::Equalizer(ch, EqualizerParameter::LmfType(FilterType4::from(vals[0])))
            }
            (0x01, 0x04, 0x02) => {
                InputCmd::Equalizer(ch, EqualizerParameter::LmfFreq(to_u32(vals)))
            }
            (0x01, 0x04, 0x03) => {
                InputCmd::Equalizer(ch, EqualizerParameter::LmfGain(to_f32(vals)))
            }
            (0x01, 0x04, 0x04) => {
                InputCmd::Equalizer(ch, EqualizerParameter::LmfWidth(to_f32(vals)))
            }

            (0x01, 0x05, 0x00) => {
                InputCmd::Equalizer(ch, EqualizerParameter::MfEnable(to_bool(vals)))
            }
            (0x01, 0x05, 0x01) => {
                InputCmd::Equalizer(ch, EqualizerParameter::MfType(FilterType4::from(vals[0])))
            }
            (0x01, 0x05, 0x02) => InputCmd::Equalizer(ch, EqualizerParameter::MfFreq(to_u32(vals))),
            (0x01, 0x05, 0x03) => InputCmd::Equalizer(ch, EqualizerParameter::MfGain(to_f32(vals))),
            (0x01, 0x05, 0x04) => {
                InputCmd::Equalizer(ch, EqualizerParameter::MfWidth(to_f32(vals)))
            }

            (0x01, 0x06, 0x00) => {
                InputCmd::Equalizer(ch, EqualizerParameter::HmfEnable(to_bool(vals)))
            }
            (0x01, 0x06, 0x01) => {
                InputCmd::Equalizer(ch, EqualizerParameter::HmfType(FilterType4::from(vals[0])))
            }
            (0x01, 0x06, 0x02) => {
                InputCmd::Equalizer(ch, EqualizerParameter::HmfFreq(to_u32(vals)))
            }
            (0x01, 0x06, 0x03) => {
                InputCmd::Equalizer(ch, EqualizerParameter::HmfGain(to_f32(vals)))
            }
            (0x01, 0x06, 0x04) => {
                InputCmd::Equalizer(ch, EqualizerParameter::HmfWidth(to_f32(vals)))
            }

            (0x01, 0x07, 0x00) => {
                InputCmd::Equalizer(ch, EqualizerParameter::HfEnable(to_bool(vals)))
            }
            (0x01, 0x07, 0x01) => {
                InputCmd::Equalizer(ch, EqualizerParameter::HfType(FilterType5::from(vals[0])))
            }
            (0x01, 0x07, 0x02) => InputCmd::Equalizer(ch, EqualizerParameter::HfFreq(to_u32(vals))),
            (0x01, 0x07, 0x03) => InputCmd::Equalizer(ch, EqualizerParameter::HfGain(to_f32(vals))),
            (0x01, 0x07, 0x04) => {
                InputCmd::Equalizer(ch, EqualizerParameter::HfWidth(to_f32(vals)))
            }

            (0x01, 0x08, 0x00) => {
                InputCmd::Equalizer(ch, EqualizerParameter::LpfEnable(to_bool(vals)))
            }
            (0x01, 0x08, 0x01) => InputCmd::Equalizer(
                ch,
                EqualizerParameter::LpfSlope(RollOffLevel::from(vals[0])),
            ),
            (0x01, 0x08, 0x02) => {
                InputCmd::Equalizer(ch, EqualizerParameter::LpfFreq(to_u32(vals)))
            }

            (0x01, 0x09, 0x00) => InputCmd::Dynamics(ch, DynamicsParameter::Enable(to_bool(vals))),

            (0x01, 0x0a, 0x00) => {
                InputCmd::Dynamics(ch, DynamicsParameter::CompEnable(to_bool(vals)))
            }
            (0x01, 0x0a, 0x01) => {
                InputCmd::Dynamics(ch, DynamicsParameter::CompThreshold(to_i32(vals)))
            }
            (0x01, 0x0a, 0x02) => {
                InputCmd::Dynamics(ch, DynamicsParameter::CompRatio(to_f32(vals)))
            }
            (0x01, 0x0a, 0x03) => {
                InputCmd::Dynamics(ch, DynamicsParameter::CompAttack(to_u32(vals)))
            }
            (0x01, 0x0a, 0x04) => {
                InputCmd::Dynamics(ch, DynamicsParameter::CompRelease(to_u32(vals)))
            }
            (0x01, 0x0a, 0x05) => InputCmd::Dynamics(ch, DynamicsParameter::CompGain(to_f32(vals))),
            (0x01, 0x0a, 0x06) => InputCmd::Dynamics(
                ch,
                DynamicsParameter::CompDetectMode(LevelDetectMode::from(vals[0])),
            ),

            (0x01, 0x0b, 0x00) => {
                InputCmd::Dynamics(ch, DynamicsParameter::LevelerEnable(to_bool(vals)))
            }
            (0x01, 0x0b, 0x01) => InputCmd::Dynamics(
                ch,
                DynamicsParameter::LevelerMode(LevelerMode::from(vals[0])),
            ),
            (0x01, 0x0b, 0x02) => {
                InputCmd::Dynamics(ch, DynamicsParameter::LevelerMakeup(to_u32(vals)))
            }
            (0x01, 0x0b, 0x03) => {
                InputCmd::Dynamics(ch, DynamicsParameter::LevelerReduce(to_u32(vals)))
            }

            (0x01, 0x0c, 0x00) => InputCmd::ReverbSend(ch, to_f32(vals)),
            (0x01, 0x0c, 0x02) => InputCmd::ReverbLrBalance(ch, to_f32(vals)),

            // TODO: model dependent, I guess.
            // (0x01, 0xfe, 0x00) => u8
            // (0x01, 0xfe, 0x01) => i32
            // (0x01, 0xfe, 0x02) => i32
            // (0x01, 0xfe, 0x03) => u8
            _ => InputCmd::Reserved(identifier.to_vec(), vals.to_vec()),
        }
    }

    fn build(&self, raw: &mut Vec<u8>) {
        match self {
            InputCmd::Phase(ch, enabled) => append_u8(raw, 0x01, 0x00, 0x00, *ch, *enabled),
            InputCmd::Pair(ch, enabled) => append_u8(raw, 0x01, 0x00, 0x01, *ch, *enabled),
            InputCmd::Gain(ch, val) => append_i32(raw, 0x01, 0x00, 0x02, *ch, *val),
            InputCmd::Swap(ch, enabled) => append_u8(raw, 0x01, 0x00, 0x03, *ch, *enabled),
            InputCmd::StereoMode(ch, pair_mode) => {
                append_u8(raw, 0x01, 0x00, 0x04, *ch, *pair_mode)
            }
            InputCmd::Width(ch, val) => append_f32(raw, 0x01, 0x00, 0x05, *ch, *val),
            InputCmd::Limitter(ch, enabled) => append_u8(raw, 0x01, 0x00, 0x06, *ch, *enabled),
            InputCmd::Lookahead(ch, enabled) => append_u8(raw, 0x01, 0x00, 0x07, *ch, *enabled),
            InputCmd::Softclip(ch, enabled) => append_u8(raw, 0x01, 0x00, 0x08, *ch, *enabled),
            InputCmd::Pad(ch, enabled) => append_u8(raw, 0x01, 0x00, 0x09, *ch, *enabled),
            InputCmd::NominalLevel(ch, level) => {
                let val = if *level == NominalSignalLevel::Professional {
                    0x01
                } else {
                    0x00
                };
                append_u8(raw, 0x01, 0x00, 0x0a, *ch, val)
            }

            InputCmd::Phantom(ch, enabled) => append_u8(raw, 0x01, 0x00, 0x0b, *ch, *enabled),

            InputCmd::Equalizer(ch, EqualizerParameter::Enable(enabled)) => {
                append_u8(raw, 0x01, 0x01, 0x00, *ch, *enabled)
            }

            InputCmd::Equalizer(ch, EqualizerParameter::HpfEnable(enabled)) => {
                append_u8(raw, 0x01, 0x02, 0x00, *ch, *enabled)
            }
            InputCmd::Equalizer(ch, EqualizerParameter::HpfSlope(level)) => {
                append_u8(raw, 0x01, 0x02, 0x01, *ch, *level)
            }
            InputCmd::Equalizer(ch, EqualizerParameter::HpfFreq(val)) => {
                append_u32(raw, 0x01, 0x02, 0x02, *ch, *val)
            }

            InputCmd::Equalizer(ch, EqualizerParameter::LfEnable(enabled)) => {
                append_u8(raw, 0x01, 0x03, 0x00, *ch, *enabled)
            }
            InputCmd::Equalizer(ch, EqualizerParameter::LfType(filter_type)) => {
                append_u8(raw, 0x01, 0x03, 0x01, *ch, *filter_type)
            }
            InputCmd::Equalizer(ch, EqualizerParameter::LfFreq(val)) => {
                append_u32(raw, 0x01, 0x03, 0x02, *ch, *val)
            }
            InputCmd::Equalizer(ch, EqualizerParameter::LfGain(val)) => {
                append_f32(raw, 0x01, 0x03, 0x03, *ch, *val)
            }
            InputCmd::Equalizer(ch, EqualizerParameter::LfWidth(val)) => {
                append_f32(raw, 0x01, 0x03, 0x04, *ch, *val)
            }

            InputCmd::Equalizer(ch, EqualizerParameter::LmfEnable(enabled)) => {
                append_u8(raw, 0x01, 0x04, 0x00, *ch, *enabled)
            }
            InputCmd::Equalizer(ch, EqualizerParameter::LmfType(filter_type)) => {
                append_u8(raw, 0x01, 0x04, 0x01, *ch, *filter_type)
            }
            InputCmd::Equalizer(ch, EqualizerParameter::LmfFreq(val)) => {
                append_u32(raw, 0x01, 0x04, 0x02, *ch, *val)
            }
            InputCmd::Equalizer(ch, EqualizerParameter::LmfGain(val)) => {
                append_f32(raw, 0x01, 0x04, 0x03, *ch, *val)
            }
            InputCmd::Equalizer(ch, EqualizerParameter::LmfWidth(val)) => {
                append_f32(raw, 0x01, 0x04, 0x04, *ch, *val)
            }

            InputCmd::Equalizer(ch, EqualizerParameter::MfEnable(enabled)) => {
                append_u8(raw, 0x01, 0x05, 0x00, *ch, *enabled)
            }
            InputCmd::Equalizer(ch, EqualizerParameter::MfType(filter_type)) => {
                append_u8(raw, 0x01, 0x05, 0x01, *ch, *filter_type)
            }
            InputCmd::Equalizer(ch, EqualizerParameter::MfFreq(val)) => {
                append_u32(raw, 0x01, 0x05, 0x02, *ch, *val)
            }
            InputCmd::Equalizer(ch, EqualizerParameter::MfGain(val)) => {
                append_f32(raw, 0x01, 0x05, 0x03, *ch, *val)
            }
            InputCmd::Equalizer(ch, EqualizerParameter::MfWidth(val)) => {
                append_f32(raw, 0x01, 0x05, 0x04, *ch, *val)
            }

            InputCmd::Equalizer(ch, EqualizerParameter::HmfEnable(enabled)) => {
                append_u8(raw, 0x01, 0x06, 0x00, *ch, *enabled)
            }
            InputCmd::Equalizer(ch, EqualizerParameter::HmfType(filter_type)) => {
                append_u8(raw, 0x01, 0x06, 0x01, *ch, *filter_type)
            }
            InputCmd::Equalizer(ch, EqualizerParameter::HmfFreq(val)) => {
                append_u32(raw, 0x01, 0x06, 0x02, *ch, *val)
            }
            InputCmd::Equalizer(ch, EqualizerParameter::HmfGain(val)) => {
                append_f32(raw, 0x01, 0x06, 0x03, *ch, *val)
            }
            InputCmd::Equalizer(ch, EqualizerParameter::HmfWidth(val)) => {
                append_f32(raw, 0x01, 0x06, 0x04, *ch, *val)
            }

            InputCmd::Equalizer(ch, EqualizerParameter::HfEnable(enabled)) => {
                append_u8(raw, 0x01, 0x07, 0x00, *ch, *enabled)
            }
            InputCmd::Equalizer(ch, EqualizerParameter::HfType(filter_type)) => {
                append_u8(raw, 0x01, 0x07, 0x01, *ch, *filter_type)
            }
            InputCmd::Equalizer(ch, EqualizerParameter::HfFreq(val)) => {
                append_u32(raw, 0x01, 0x07, 0x02, *ch, *val)
            }
            InputCmd::Equalizer(ch, EqualizerParameter::HfGain(val)) => {
                append_f32(raw, 0x01, 0x07, 0x03, *ch, *val)
            }
            InputCmd::Equalizer(ch, EqualizerParameter::HfWidth(val)) => {
                append_f32(raw, 0x01, 0x07, 0x04, *ch, *val)
            }

            InputCmd::Equalizer(ch, EqualizerParameter::LpfEnable(enabled)) => {
                append_u8(raw, 0x01, 0x08, 0x00, *ch, *enabled)
            }
            InputCmd::Equalizer(ch, EqualizerParameter::LpfSlope(level)) => {
                append_u8(raw, 0x01, 0x08, 0x01, *ch, *level)
            }
            InputCmd::Equalizer(ch, EqualizerParameter::LpfFreq(val)) => {
                append_u32(raw, 0x01, 0x08, 0x02, *ch, *val)
            }

            InputCmd::Dynamics(ch, DynamicsParameter::Enable(enabled)) => {
                append_u8(raw, 0x01, 0x09, 0x00, *ch, *enabled)
            }

            InputCmd::Dynamics(ch, DynamicsParameter::CompEnable(enabled)) => {
                append_u8(raw, 0x01, 0x0a, 0x00, *ch, *enabled)
            }
            InputCmd::Dynamics(ch, DynamicsParameter::CompThreshold(val)) => {
                append_i32(raw, 0x01, 0x0a, 0x01, *ch, *val)
            }
            InputCmd::Dynamics(ch, DynamicsParameter::CompRatio(val)) => {
                append_f32(raw, 0x01, 0x0a, 0x02, *ch, *val)
            }
            InputCmd::Dynamics(ch, DynamicsParameter::CompAttack(val)) => {
                append_u32(raw, 0x01, 0x0a, 0x03, *ch, *val)
            }
            InputCmd::Dynamics(ch, DynamicsParameter::CompRelease(val)) => {
                append_u32(raw, 0x01, 0x0a, 0x04, *ch, *val)
            }
            InputCmd::Dynamics(ch, DynamicsParameter::CompGain(val)) => {
                append_f32(raw, 0x01, 0x0a, 0x05, *ch, *val)
            }
            InputCmd::Dynamics(ch, DynamicsParameter::CompDetectMode(mode)) => {
                append_u8(raw, 0x01, 0x0a, 0x06, *ch, *mode)
            }

            InputCmd::Dynamics(ch, DynamicsParameter::LevelerEnable(enabled)) => {
                append_u8(raw, 0x01, 0x0b, 0x00, *ch, *enabled)
            }
            InputCmd::Dynamics(ch, DynamicsParameter::LevelerMode(mode)) => {
                append_u8(raw, 0x01, 0x0b, 0x01, *ch, *mode)
            }
            InputCmd::Dynamics(ch, DynamicsParameter::LevelerMakeup(val)) => {
                append_u32(raw, 0x01, 0x0b, 0x02, *ch, *val)
            }
            InputCmd::Dynamics(ch, DynamicsParameter::LevelerReduce(val)) => {
                append_u32(raw, 0x01, 0x0b, 0x03, *ch, *val)
            }

            InputCmd::ReverbSend(ch, val) => append_f32(raw, 0x01, 0x0c, 0x00, *ch, *val),
            InputCmd::ReverbLrBalance(ch, val) => append_f32(raw, 0x01, 0x0c, 0x02, *ch, *val),

            InputCmd::Reserved(identifier, vals) => append_data(raw, identifier, vals),
        }
    }
}

/// The mode of stereo pair for source of mixer.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SourceStereoPairMode {
    Width,
    LrBalance,
    Reserved(u8),
}

impl Default for SourceStereoPairMode {
    fn default() -> Self {
        Self::Width
    }
}

impl From<u8> for SourceStereoPairMode {
    fn from(val: u8) -> Self {
        match val {
            0 => Self::Width,
            1 => Self::LrBalance,
            _ => Self::Reserved(val),
        }
    }
}

impl From<SourceStereoPairMode> for u8 {
    fn from(mode: SourceStereoPairMode) -> Self {
        match mode {
            SourceStereoPairMode::Width => 0,
            SourceStereoPairMode::LrBalance => 1,
            SourceStereoPairMode::Reserved(val) => val,
        }
    }
}

/// The DSP command specific to mixer.
#[derive(Debug, Clone, PartialEq)]
pub enum MixerCmd {
    OutputAssign(usize, usize),
    OutputMute(usize, bool),
    OutputVolume(usize, f32),
    ReverbSend(usize, f32),
    ReverbReturn(usize, f32),
    SourceMute(usize, usize, bool),
    SourceSolo(usize, usize, bool),
    SourceMonauralLrBalance(usize, usize, f32),
    SourceGain(usize, usize, f32),
    SourceStereoMode(usize, usize, SourceStereoPairMode),
    SourceStereoLrBalance(usize, usize, f32),
    SourceStereoWidth(usize, usize, f32),
    Reserved(Vec<u8>, Vec<u8>),
}

impl MixerCmd {
    fn parse(identifier: &[u8], vals: &[u8]) -> Self {
        assert_eq!(identifier.len(), 4);
        assert!(vals.len() > 0);

        let ch = identifier[0] as usize;
        let mixer_src_ch = identifier[2] as usize;

        match (identifier[3], identifier[2], identifier[1]) {
            (0x02, 0x00, 0x00) => MixerCmd::OutputAssign(ch, to_usize(vals)),
            (0x02, 0x00, 0x01) => MixerCmd::OutputMute(ch, to_bool(vals)),
            (0x02, 0x00, 0x02) => MixerCmd::OutputVolume(ch, to_f32(vals)),

            (0x02, 0x01, 0x00) => MixerCmd::ReverbSend(ch, to_f32(vals)),
            (0x02, 0x01, 0x01) => MixerCmd::ReverbReturn(ch, to_f32(vals)),

            (0x02, _, 0x00) => MixerCmd::SourceMute(ch, mixer_src_ch - 2, to_bool(vals)),
            (0x02, _, 0x01) => MixerCmd::SourceSolo(ch, mixer_src_ch - 2, to_bool(vals)),
            (0x02, _, 0x02) => {
                MixerCmd::SourceMonauralLrBalance(ch, mixer_src_ch - 2, to_f32(vals))
            }
            (0x02, _, 0x03) => MixerCmd::SourceGain(ch, mixer_src_ch - 2, to_f32(vals)),
            (0x02, _, 0x04) => MixerCmd::SourceStereoMode(
                ch,
                mixer_src_ch - 2,
                SourceStereoPairMode::from(vals[0]),
            ),
            (0x02, _, 0x05) => MixerCmd::SourceStereoLrBalance(ch, mixer_src_ch - 2, to_f32(vals)),
            (0x02, _, 0x06) => MixerCmd::SourceStereoWidth(ch, mixer_src_ch - 2, to_f32(vals)),
            _ => MixerCmd::Reserved(identifier.to_vec(), vals.to_vec()),
        }
    }

    fn build(&self, raw: &mut Vec<u8>) {
        match self {
            MixerCmd::OutputAssign(ch, target) => {
                append_u8(raw, 0x02, 0x00, 0x00, *ch, *target as u8)
            }
            MixerCmd::OutputMute(ch, enabled) => append_u8(raw, 0x02, 0x00, 0x01, *ch, *enabled),
            MixerCmd::OutputVolume(ch, val) => append_f32(raw, 0x02, 0x00, 0x02, *ch, *val),

            MixerCmd::ReverbSend(ch, val) => append_f32(raw, 0x02, 0x01, 0x00, *ch, *val),
            MixerCmd::ReverbReturn(ch, val) => append_f32(raw, 0x02, 0x01, 0x01, *ch, *val),

            MixerCmd::SourceMute(ch, mixer_src_ch, enabled) => {
                append_u8(raw, 0x02, (*mixer_src_ch + 2) as u8, 0x00, *ch, *enabled)
            }
            MixerCmd::SourceSolo(ch, mixer_src_ch, enabled) => {
                append_u8(raw, 0x02, (*mixer_src_ch + 2) as u8, 0x01, *ch, *enabled)
            }
            MixerCmd::SourceMonauralLrBalance(ch, mixer_src_ch, val) => {
                append_f32(raw, 0x02, (*mixer_src_ch + 2) as u8, 0x02, *ch, *val)
            }
            MixerCmd::SourceGain(ch, mixer_src_ch, val) => {
                append_f32(raw, 0x02, (*mixer_src_ch + 2) as u8, 0x03, *ch, *val)
            }
            MixerCmd::SourceStereoMode(ch, mixer_src_ch, pair_mode) => {
                append_u8(raw, 0x02, (*mixer_src_ch + 2) as u8, 0x04, *ch, *pair_mode)
            }
            MixerCmd::SourceStereoLrBalance(ch, mixer_src_ch, val) => {
                append_f32(raw, 0x02, (*mixer_src_ch + 2) as u8, 0x05, *ch, *val)
            }
            MixerCmd::SourceStereoWidth(ch, mixer_src_ch, val) => {
                append_f32(raw, 0x02, (*mixer_src_ch + 2) as u8, 0x06, *ch, *val)
            }

            MixerCmd::Reserved(identifier, vals) => append_data(raw, identifier, vals),
        }
    }
}

/// The DSP command specific to input.
#[derive(Debug, Clone, PartialEq)]
pub enum OutputCmd {
    Equalizer(usize, EqualizerParameter),
    Dynamics(usize, DynamicsParameter),
    ReverbSend(usize, f32),
    ReverbReturn(usize, f32),
    MasterMonitor(usize, bool),
    MasterTalkback(usize, bool),
    MasterListenback(usize, bool),
    Reserved(Vec<u8>, Vec<u8>),
}

impl OutputCmd {
    fn parse(identifier: &[u8], vals: &[u8]) -> Self {
        let ch = identifier[0] as usize;

        match (identifier[3], identifier[2], identifier[1]) {
            (0x03, 0x00, 0x00) => {
                OutputCmd::Equalizer(ch, EqualizerParameter::Enable(to_bool(vals)))
            }

            (0x03, 0x01, 0x00) => {
                OutputCmd::Equalizer(ch, EqualizerParameter::HpfEnable(to_bool(vals)))
            }
            (0x03, 0x01, 0x01) => OutputCmd::Equalizer(
                ch,
                EqualizerParameter::HpfSlope(RollOffLevel::from(vals[0])),
            ),
            (0x03, 0x01, 0x02) => {
                OutputCmd::Equalizer(ch, EqualizerParameter::HpfFreq(to_u32(vals)))
            }

            (0x03, 0x02, 0x00) => {
                OutputCmd::Equalizer(ch, EqualizerParameter::LfEnable(to_bool(vals)))
            }
            (0x03, 0x02, 0x01) => {
                OutputCmd::Equalizer(ch, EqualizerParameter::LfType(FilterType5::from(vals[0])))
            }
            (0x03, 0x02, 0x02) => {
                OutputCmd::Equalizer(ch, EqualizerParameter::LfFreq(to_u32(vals)))
            }
            (0x03, 0x02, 0x03) => {
                OutputCmd::Equalizer(ch, EqualizerParameter::LfGain(to_f32(vals)))
            }
            (0x03, 0x02, 0x04) => {
                OutputCmd::Equalizer(ch, EqualizerParameter::LfWidth(to_f32(vals)))
            }

            (0x03, 0x03, 0x00) => {
                OutputCmd::Equalizer(ch, EqualizerParameter::LmfEnable(to_bool(vals)))
            }
            (0x03, 0x03, 0x01) => {
                OutputCmd::Equalizer(ch, EqualizerParameter::LmfType(FilterType4::from(vals[0])))
            }
            (0x03, 0x03, 0x02) => {
                OutputCmd::Equalizer(ch, EqualizerParameter::LmfFreq(to_u32(vals)))
            }
            (0x03, 0x03, 0x03) => {
                OutputCmd::Equalizer(ch, EqualizerParameter::LmfGain(to_f32(vals)))
            }
            (0x03, 0x03, 0x04) => {
                OutputCmd::Equalizer(ch, EqualizerParameter::LmfWidth(to_f32(vals)))
            }

            (0x03, 0x04, 0x00) => {
                OutputCmd::Equalizer(ch, EqualizerParameter::MfEnable(to_bool(vals)))
            }
            (0x03, 0x04, 0x01) => {
                OutputCmd::Equalizer(ch, EqualizerParameter::MfType(FilterType4::from(vals[0])))
            }
            (0x03, 0x04, 0x02) => {
                OutputCmd::Equalizer(ch, EqualizerParameter::MfFreq(to_u32(vals)))
            }
            (0x03, 0x04, 0x03) => {
                OutputCmd::Equalizer(ch, EqualizerParameter::MfGain(to_f32(vals)))
            }
            (0x03, 0x04, 0x04) => {
                OutputCmd::Equalizer(ch, EqualizerParameter::MfWidth(to_f32(vals)))
            }

            (0x03, 0x05, 0x00) => {
                OutputCmd::Equalizer(ch, EqualizerParameter::HmfEnable(to_bool(vals)))
            }
            (0x03, 0x05, 0x01) => {
                OutputCmd::Equalizer(ch, EqualizerParameter::HmfType(FilterType4::from(vals[0])))
            }
            (0x03, 0x05, 0x02) => {
                OutputCmd::Equalizer(ch, EqualizerParameter::HmfFreq(to_u32(vals)))
            }
            (0x03, 0x05, 0x03) => {
                OutputCmd::Equalizer(ch, EqualizerParameter::HmfGain(to_f32(vals)))
            }
            (0x03, 0x05, 0x04) => {
                OutputCmd::Equalizer(ch, EqualizerParameter::HmfWidth(to_f32(vals)))
            }

            (0x03, 0x06, 0x00) => {
                OutputCmd::Equalizer(ch, EqualizerParameter::HfEnable(to_bool(vals)))
            }
            (0x03, 0x06, 0x01) => {
                OutputCmd::Equalizer(ch, EqualizerParameter::HfType(FilterType5::from(vals[0])))
            }
            (0x03, 0x06, 0x02) => {
                OutputCmd::Equalizer(ch, EqualizerParameter::HfFreq(to_u32(vals)))
            }
            (0x03, 0x06, 0x03) => {
                OutputCmd::Equalizer(ch, EqualizerParameter::HfGain(to_f32(vals)))
            }
            (0x03, 0x06, 0x04) => {
                OutputCmd::Equalizer(ch, EqualizerParameter::HfWidth(to_f32(vals)))
            }

            (0x03, 0x07, 0x00) => {
                OutputCmd::Equalizer(ch, EqualizerParameter::LpfEnable(to_bool(vals)))
            }
            (0x03, 0x07, 0x01) => OutputCmd::Equalizer(
                ch,
                EqualizerParameter::LpfSlope(RollOffLevel::from(vals[0])),
            ),
            (0x03, 0x07, 0x02) => {
                OutputCmd::Equalizer(ch, EqualizerParameter::LpfFreq(to_u32(vals)))
            }

            (0x03, 0x08, 0x00) => OutputCmd::Dynamics(ch, DynamicsParameter::Enable(to_bool(vals))),

            (0x03, 0x09, 0x00) => {
                OutputCmd::Dynamics(ch, DynamicsParameter::CompEnable(to_bool(vals)))
            }
            (0x03, 0x09, 0x01) => {
                OutputCmd::Dynamics(ch, DynamicsParameter::CompThreshold(to_i32(vals)))
            }
            (0x03, 0x09, 0x02) => {
                OutputCmd::Dynamics(ch, DynamicsParameter::CompRatio(to_f32(vals)))
            }
            (0x03, 0x09, 0x03) => {
                OutputCmd::Dynamics(ch, DynamicsParameter::CompAttack(to_u32(vals)))
            }
            (0x03, 0x09, 0x04) => {
                OutputCmd::Dynamics(ch, DynamicsParameter::CompRelease(to_u32(vals)))
            }
            (0x03, 0x09, 0x05) => {
                OutputCmd::Dynamics(ch, DynamicsParameter::CompGain(to_f32(vals)))
            }
            (0x03, 0x09, 0x06) => OutputCmd::Dynamics(
                ch,
                DynamicsParameter::CompDetectMode(LevelDetectMode::from(vals[0])),
            ),

            (0x03, 0x0a, 0x00) => {
                OutputCmd::Dynamics(ch, DynamicsParameter::LevelerEnable(to_bool(vals)))
            }
            (0x03, 0x0a, 0x01) => OutputCmd::Dynamics(
                ch,
                DynamicsParameter::LevelerMode(LevelerMode::from(vals[0])),
            ),
            (0x03, 0x0a, 0x02) => {
                OutputCmd::Dynamics(ch, DynamicsParameter::LevelerMakeup(to_u32(vals)))
            }
            (0x03, 0x0a, 0x03) => {
                OutputCmd::Dynamics(ch, DynamicsParameter::LevelerReduce(to_u32(vals)))
            }

            (0x03, 0x0b, 0x00) => OutputCmd::ReverbSend(ch, to_f32(vals)),
            (0x03, 0x0b, 0x01) => OutputCmd::ReverbReturn(ch, to_f32(vals)),

            (0x03, 0x0c, 0x00) => OutputCmd::MasterMonitor(ch, to_bool(vals)),
            (0x03, 0x0c, 0x01) => OutputCmd::MasterTalkback(ch, to_bool(vals)),
            (0x03, 0x0c, 0x02) => OutputCmd::MasterListenback(ch, to_bool(vals)),

            _ => OutputCmd::Reserved(identifier.to_vec(), vals.to_vec()),
        }
    }

    fn build(&self, raw: &mut Vec<u8>) {
        match self {
            OutputCmd::Equalizer(ch, EqualizerParameter::Enable(enabled)) => {
                append_u8(raw, 0x03, 0x00, 0x00, *ch, *enabled)
            }

            OutputCmd::Equalizer(ch, EqualizerParameter::HpfEnable(enabled)) => {
                append_u8(raw, 0x03, 0x01, 0x00, *ch, *enabled)
            }
            OutputCmd::Equalizer(ch, EqualizerParameter::HpfSlope(level)) => {
                append_u8(raw, 0x03, 0x01, 0x01, *ch, *level)
            }
            OutputCmd::Equalizer(ch, EqualizerParameter::HpfFreq(val)) => {
                append_u32(raw, 0x03, 0x01, 0x02, *ch, *val)
            }

            OutputCmd::Equalizer(ch, EqualizerParameter::LfEnable(enabled)) => {
                append_u8(raw, 0x03, 0x02, 0x00, *ch, *enabled)
            }
            OutputCmd::Equalizer(ch, EqualizerParameter::LfType(filter_type)) => {
                append_u8(raw, 0x03, 0x02, 0x01, *ch, *filter_type)
            }
            OutputCmd::Equalizer(ch, EqualizerParameter::LfFreq(val)) => {
                append_u32(raw, 0x03, 0x02, 0x02, *ch, *val)
            }
            OutputCmd::Equalizer(ch, EqualizerParameter::LfGain(val)) => {
                append_f32(raw, 0x03, 0x02, 0x03, *ch, *val)
            }
            OutputCmd::Equalizer(ch, EqualizerParameter::LfWidth(val)) => {
                append_f32(raw, 0x03, 0x02, 0x04, *ch, *val)
            }

            OutputCmd::Equalizer(ch, EqualizerParameter::LmfEnable(enabled)) => {
                append_u8(raw, 0x03, 0x03, 0x00, *ch, *enabled)
            }
            OutputCmd::Equalizer(ch, EqualizerParameter::LmfType(filter_type)) => {
                append_u8(raw, 0x03, 0x03, 0x01, *ch, *filter_type)
            }
            OutputCmd::Equalizer(ch, EqualizerParameter::LmfFreq(val)) => {
                append_u32(raw, 0x03, 0x03, 0x02, *ch, *val)
            }
            OutputCmd::Equalizer(ch, EqualizerParameter::LmfGain(val)) => {
                append_f32(raw, 0x03, 0x03, 0x03, *ch, *val)
            }
            OutputCmd::Equalizer(ch, EqualizerParameter::LmfWidth(val)) => {
                append_f32(raw, 0x03, 0x03, 0x04, *ch, *val)
            }

            OutputCmd::Equalizer(ch, EqualizerParameter::MfEnable(enabled)) => {
                append_u8(raw, 0x03, 0x04, 0x00, *ch, *enabled)
            }
            OutputCmd::Equalizer(ch, EqualizerParameter::MfType(filter_type)) => {
                append_u8(raw, 0x03, 0x04, 0x01, *ch, *filter_type)
            }
            OutputCmd::Equalizer(ch, EqualizerParameter::MfFreq(val)) => {
                append_u32(raw, 0x03, 0x04, 0x02, *ch, *val)
            }
            OutputCmd::Equalizer(ch, EqualizerParameter::MfGain(val)) => {
                append_f32(raw, 0x03, 0x04, 0x03, *ch, *val)
            }
            OutputCmd::Equalizer(ch, EqualizerParameter::MfWidth(val)) => {
                append_f32(raw, 0x03, 0x04, 0x04, *ch, *val)
            }

            OutputCmd::Equalizer(ch, EqualizerParameter::HmfEnable(enabled)) => {
                append_u8(raw, 0x03, 0x05, 0x00, *ch, *enabled)
            }
            OutputCmd::Equalizer(ch, EqualizerParameter::HmfType(filter_type)) => {
                append_u8(raw, 0x03, 0x05, 0x01, *ch, *filter_type)
            }
            OutputCmd::Equalizer(ch, EqualizerParameter::HmfFreq(val)) => {
                append_u32(raw, 0x03, 0x05, 0x02, *ch, *val)
            }
            OutputCmd::Equalizer(ch, EqualizerParameter::HmfGain(val)) => {
                append_f32(raw, 0x03, 0x05, 0x03, *ch, *val)
            }
            OutputCmd::Equalizer(ch, EqualizerParameter::HmfWidth(val)) => {
                append_f32(raw, 0x03, 0x05, 0x04, *ch, *val)
            }

            OutputCmd::Equalizer(ch, EqualizerParameter::HfEnable(enabled)) => {
                append_u8(raw, 0x03, 0x06, 0x00, *ch, *enabled)
            }
            OutputCmd::Equalizer(ch, EqualizerParameter::HfType(filter_type)) => {
                append_u8(raw, 0x03, 0x06, 0x01, *ch, *filter_type)
            }
            OutputCmd::Equalizer(ch, EqualizerParameter::HfFreq(val)) => {
                append_u32(raw, 0x03, 0x06, 0x02, *ch, *val)
            }
            OutputCmd::Equalizer(ch, EqualizerParameter::HfGain(val)) => {
                append_f32(raw, 0x03, 0x06, 0x03, *ch, *val)
            }
            OutputCmd::Equalizer(ch, EqualizerParameter::HfWidth(val)) => {
                append_f32(raw, 0x03, 0x06, 0x04, *ch, *val)
            }

            OutputCmd::Equalizer(ch, EqualizerParameter::LpfEnable(enabled)) => {
                append_u8(raw, 0x03, 0x07, 0x00, *ch, *enabled)
            }
            OutputCmd::Equalizer(ch, EqualizerParameter::LpfSlope(level)) => {
                append_u8(raw, 0x03, 0x07, 0x01, *ch, *level)
            }
            OutputCmd::Equalizer(ch, EqualizerParameter::LpfFreq(val)) => {
                append_u32(raw, 0x03, 0x07, 0x02, *ch, *val)
            }

            OutputCmd::Dynamics(ch, DynamicsParameter::Enable(enabled)) => {
                append_u8(raw, 0x03, 0x08, 0x00, *ch, *enabled)
            }

            OutputCmd::Dynamics(ch, DynamicsParameter::CompEnable(enabled)) => {
                append_u8(raw, 0x03, 0x09, 0x00, *ch, *enabled)
            }
            OutputCmd::Dynamics(ch, DynamicsParameter::CompThreshold(val)) => {
                append_i32(raw, 0x03, 0x09, 0x01, *ch, *val)
            }
            OutputCmd::Dynamics(ch, DynamicsParameter::CompRatio(val)) => {
                append_f32(raw, 0x03, 0x09, 0x02, *ch, *val)
            }
            OutputCmd::Dynamics(ch, DynamicsParameter::CompAttack(val)) => {
                append_u32(raw, 0x03, 0x09, 0x03, *ch, *val)
            }
            OutputCmd::Dynamics(ch, DynamicsParameter::CompRelease(val)) => {
                append_u32(raw, 0x03, 0x09, 0x04, *ch, *val)
            }
            OutputCmd::Dynamics(ch, DynamicsParameter::CompGain(val)) => {
                append_f32(raw, 0x03, 0x09, 0x05, *ch, *val)
            }
            OutputCmd::Dynamics(ch, DynamicsParameter::CompDetectMode(mode)) => {
                append_u8(raw, 0x03, 0x09, 0x06, *ch, *mode)
            }

            OutputCmd::Dynamics(ch, DynamicsParameter::LevelerEnable(enabled)) => {
                append_u8(raw, 0x03, 0x0a, 0x00, *ch, *enabled)
            }
            OutputCmd::Dynamics(ch, DynamicsParameter::LevelerMode(mode)) => {
                append_u8(raw, 0x03, 0x0a, 0x01, *ch, *mode)
            }
            OutputCmd::Dynamics(ch, DynamicsParameter::LevelerMakeup(val)) => {
                append_u32(raw, 0x03, 0x0a, 0x02, *ch, *val)
            }
            OutputCmd::Dynamics(ch, DynamicsParameter::LevelerReduce(val)) => {
                append_u32(raw, 0x03, 0x0a, 0x03, *ch, *val)
            }

            OutputCmd::ReverbSend(ch, val) => append_f32(raw, 0x03, 0x0b, 0x00, *ch, *val),
            OutputCmd::ReverbReturn(ch, val) => append_f32(raw, 0x03, 0x0b, 0x01, *ch, *val),

            OutputCmd::MasterMonitor(ch, val) => append_u8(raw, 0x03, 0x0c, 0x00, *ch, *val),
            OutputCmd::MasterTalkback(ch, enabled) => {
                append_u8(raw, 0x03, 0x0c, 0x01, *ch, *enabled)
            }
            OutputCmd::MasterListenback(ch, enabled) => {
                append_u8(raw, 0x03, 0x0c, 0x02, *ch, *enabled)
            }

            OutputCmd::Reserved(identifier, vals) => append_data(raw, identifier, vals),
        }
    }
}

/// The mode of early reflection.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum RoomShape {
    A,
    B,
    C,
    D,
    E,
    Reserved(u8),
}

impl Default for RoomShape {
    fn default() -> Self {
        Self::A
    }
}

impl From<u8> for RoomShape {
    fn from(val: u8) -> Self {
        match val {
            0 => Self::A,
            1 => Self::B,
            2 => Self::C,
            3 => Self::D,
            4 => Self::E,
            _ => Self::Reserved(val),
        }
    }
}

impl From<RoomShape> for u8 {
    fn from(shape: RoomShape) -> Self {
        match shape {
            RoomShape::A => 0,
            RoomShape::B => 1,
            RoomShape::C => 2,
            RoomShape::D => 3,
            RoomShape::E => 4,
            RoomShape::Reserved(val) => val,
        }
    }
}

/// The point of split.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SplitPoint {
    Output,
    Mixer,
    Reserved(u8),
}

impl Default for SplitPoint {
    fn default() -> Self {
        Self::Output
    }
}

impl From<u8> for SplitPoint {
    fn from(val: u8) -> Self {
        match val {
            0 => Self::Output,
            1 => Self::Mixer,
            _ => Self::Reserved(val),
        }
    }
}

impl From<SplitPoint> for u8 {
    fn from(point: SplitPoint) -> Self {
        match point {
            SplitPoint::Output => 0,
            SplitPoint::Mixer => 1,
            SplitPoint::Reserved(val) => val,
        }
    }
}

/// The DSP command specific to reverb effect.
#[derive(Debug, Clone, PartialEq)]
pub enum ReverbCmd {
    Enable(bool),
    Split(SplitPoint),
    PreDelay(u32),
    ShelfFilterFreq(u32),
    ShelfFilterAttenuation(i32),
    DecayTime(u32),
    LowFreqTime(u32),
    MiddleFreqTime(u32),
    HighFreqTime(u32),
    LowFreqCrossover(u32),
    HighFreqCrossover(u32),
    Width(f32),
    ReflectionMode(RoomShape),
    ReflectionSize(u32),
    ReflectionLevel(f32),
    Reserved(Vec<u8>, Vec<u8>),
}

impl ReverbCmd {
    fn parse(identifier: &[u8], vals: &[u8]) -> Self {
        assert_eq!(identifier.len(), 4);
        assert!(vals.len() > 0);

        match (identifier[3], identifier[2], identifier[1]) {
            (0x04, 0x00, 0x00) => ReverbCmd::Enable(to_bool(vals)),
            (0x04, 0x00, 0x01) => ReverbCmd::Split(SplitPoint::from(vals[0])),
            (0x04, 0x00, 0x02) => ReverbCmd::PreDelay(to_u32(vals)),
            (0x04, 0x00, 0x03) => ReverbCmd::ShelfFilterFreq(to_u32(vals)),
            (0x04, 0x00, 0x04) => ReverbCmd::ShelfFilterAttenuation(to_i32(vals)),
            (0x04, 0x00, 0x05) => ReverbCmd::DecayTime(to_u32(vals)),
            (0x04, 0x00, 0x06) => ReverbCmd::LowFreqTime(to_u32(vals)),
            (0x04, 0x00, 0x07) => ReverbCmd::MiddleFreqTime(to_u32(vals)),
            (0x04, 0x00, 0x08) => ReverbCmd::HighFreqTime(to_u32(vals)),
            (0x04, 0x00, 0x09) => ReverbCmd::LowFreqCrossover(to_u32(vals)),
            (0x04, 0x00, 0x0a) => ReverbCmd::HighFreqCrossover(to_u32(vals)),
            (0x04, 0x00, 0x0b) => ReverbCmd::Width(to_f32(vals)),
            (0x04, 0x00, 0x0c) => ReverbCmd::ReflectionMode(RoomShape::from(vals[0])),
            (0x04, 0x00, 0x0d) => ReverbCmd::ReflectionSize(to_u32(vals)),
            (0x04, 0x00, 0x0e) => ReverbCmd::ReflectionLevel(to_f32(vals)),
            _ => ReverbCmd::Reserved(identifier.to_vec(), vals.to_vec()),
        }
    }

    fn build(&self, raw: &mut Vec<u8>) {
        match self {
            ReverbCmd::Enable(enabled) => append_u8(raw, 0x04, 0x00, 0x00, 0, *enabled),
            ReverbCmd::Split(point) => append_u8(raw, 0x04, 0x00, 0x01, 0, *point),
            ReverbCmd::PreDelay(val) => append_u32(raw, 0x04, 0x00, 0x02, 0, *val),
            ReverbCmd::ShelfFilterFreq(val) => append_u32(raw, 0x04, 0x00, 0x03, 0, *val),
            ReverbCmd::ShelfFilterAttenuation(val) => append_i32(raw, 0x04, 0x00, 0x04, 0, *val),
            ReverbCmd::DecayTime(val) => append_u32(raw, 0x04, 0x00, 0x05, 0, *val),
            ReverbCmd::LowFreqTime(val) => append_u32(raw, 0x04, 0x00, 0x06, 0, *val),
            ReverbCmd::MiddleFreqTime(val) => append_u32(raw, 0x04, 0x00, 0x07, 0, *val),
            ReverbCmd::HighFreqTime(val) => append_u32(raw, 0x04, 0x00, 0x08, 0, *val),
            ReverbCmd::LowFreqCrossover(val) => append_u32(raw, 0x04, 0x00, 0x09, 0, *val),
            ReverbCmd::HighFreqCrossover(val) => append_u32(raw, 0x04, 0x00, 0x0a, 0, *val),
            ReverbCmd::Width(val) => append_f32(raw, 0x04, 0x00, 0x0b, 0, *val),
            ReverbCmd::ReflectionMode(shape) => append_u8(raw, 0x04, 0x00, 0x0c, 0, *shape),
            ReverbCmd::ReflectionSize(val) => append_u32(raw, 0x04, 0x00, 0x0d, 0, *val),
            ReverbCmd::ReflectionLevel(val) => append_f32(raw, 0x04, 0x00, 0x0e, 0, *val),
            ReverbCmd::Reserved(identifier, vals) => append_data(raw, identifier, vals),
        }
    }
}

/// The DSP command specific to usage of resource.
#[derive(Debug, Clone, PartialEq)]
pub enum ResourceCmd {
    Usage(f32, u8),
    Reserved(Vec<u8>),
}

impl ResourceCmd {
    pub const USAGE_MIN: f32 = 0.0;
    pub const USAGE_MAX: f32 = 100.0;

    fn parse(raw: &[u8]) -> Self {
        let mut quadlet = [0; 4];
        quadlet.copy_from_slice(&raw[1..5]);
        ResourceCmd::Usage(f32::from_le_bytes(quadlet), raw[5])
    }

    fn build(&self, raw: &mut Vec<u8>) {
        match self {
            Self::Usage(usage, flag) => append_resource(raw, *usage, *flag),
            Self::Reserved(data) => raw.extend_from_slice(data),
        }
    }
}

/// The DSP command.
#[derive(Debug, Clone, PartialEq)]
pub enum DspCmd {
    Monitor(MonitorCmd),
    Input(InputCmd),
    Mixer(MixerCmd),
    Output(OutputCmd),
    Reverb(ReverbCmd),
    Resource(ResourceCmd),
    Reserved(Vec<u8>),
}

impl DspCmd {
    // MEMO: Eight types of command are used in transaction frame from/to the target device. Each
    // type is expressed in the first byte of command:
    //
    // 0x00: Type 0: padding bytes start
    // 0x23: Type 1: command with DSP resource.
    // 0x46: Type 2: command with multiple quadlet coefficients
    // 0x49: Type 3: command with multiple byte coefficients
    // 0x62: Type 4: command for draining previous commands in frame
    // 0x65: Type 5: end of command if appears
    // 0x66: Type 6: command with single quadlet coefficient.
    // 0x69: Type 7: command with single byte coefficient.
    //
    // The layout of each type of command which has own content is described below:
    //
    // Type 1 command:
    //
    // command[0]: 0x23
    // command[1..5]: current usage as quadlet data aligned to big-endianness
    // command[5]: 0x11: identifier
    //
    // Type 2 command:
    //
    // command[0]: 0x46
    // command[1]: the number of coefficients
    // command[2..6]: identifier
    // command[6..]: the list of coefficients aligned to big-endianness
    //
    // Type 3 command:
    //
    // command[0]: 0x49
    // command[1]: the number of coefficients
    // command[2..6]: identifier
    // command[6..]: the list of coefficients
    //
    // Type 6 command:
    //
    // command[0]: 0x66
    // command[1..5]: identifier
    // command[5..9]: quadlet coefficient aligned to big-endianness
    //
    // Type 7 command:
    //
    // command[0]: 0x69
    // command[1]: byte coefficient
    // command[2..6]: identifier
    //
    // The last field of identifier expresses the target of command at first level:
    //
    // 0x00: monitor
    // 0x01: input
    // 0x02: mixer
    // 0x03: output
    // 0x04: reverb
    //
    // The rest fields of identifier has unique purpose depending on the first level. For example,
    // in input command, the identifier has below fields:
    //
    // identifier[0]: channel number
    // identifier[1]: third level; e.g. 0x01 the type of filter for low frequency filter.
    // identifier[2]: second level; e.g. 0x03 for low frequency filter.
    // identifier[3]: 0x01: first level
    //
    pub fn parse(raw: &[u8], cmds: &mut Vec<DspCmd>) -> usize {
        match raw[0] {
            CMD_RESOURCE => {
                let r = &raw[..CMD_RESOURCE_LENGTH];
                let cmd = DspCmd::Resource(ResourceCmd::parse(r));
                cmds.push(cmd);

                CMD_RESOURCE_LENGTH
            }
            CMD_BYTE_MULTIPLE => {
                let count = raw[1] as usize;
                let length = 6 + count;

                let mut identifier = [0; 4];
                identifier.copy_from_slice(&raw[2..6]);
                let first_level = identifier[3];

                if first_level <= 0x04 {
                    (0..count).for_each(|i| {
                        identifier[0] = i as u8;
                        let vals = &raw[(6 + i)..(6 + i + 1)];
                        let cmd = match first_level {
                            0x00 => DspCmd::Monitor(MonitorCmd::parse(&identifier, vals)),
                            0x01 => DspCmd::Input(InputCmd::parse(&identifier, vals)),
                            0x02 => DspCmd::Mixer(MixerCmd::parse(&identifier, vals)),
                            0x03 => DspCmd::Output(OutputCmd::parse(&identifier, vals)),
                            0x04 => DspCmd::Reverb(ReverbCmd::parse(&identifier, vals)),
                            _ => unreachable!(),
                        };
                        cmds.push(cmd);
                    });
                } else {
                    let cmd = DspCmd::Reserved(raw[..length].to_vec());
                    cmds.push(cmd);
                }

                length
            }
            CMD_QUADLET_MULTIPLE => {
                let count = raw[1] as usize;
                let length = 6 + count * 4;

                let mut identifier = [0; 4];
                identifier.copy_from_slice(&raw[2..6]);
                let first_level = identifier[3];

                if first_level <= 0x04 {
                    (0..count).for_each(|i| {
                        identifier[0] = i as u8;
                        let vals = &raw[(6 + i * 4)..(6 + i * 4 + 4)];
                        let cmd = match first_level {
                            0x00 => DspCmd::Monitor(MonitorCmd::parse(&identifier, vals)),
                            0x01 => DspCmd::Input(InputCmd::parse(&identifier, vals)),
                            0x02 => DspCmd::Mixer(MixerCmd::parse(&identifier, vals)),
                            0x03 => DspCmd::Output(OutputCmd::parse(&identifier, vals)),
                            0x04 => DspCmd::Reverb(ReverbCmd::parse(&identifier, vals)),
                            _ => unreachable!(),
                        };
                        cmds.push(cmd);
                    });
                } else {
                    let cmd = DspCmd::Reserved(raw[..length].to_vec());
                    cmds.push(cmd);
                }

                6 + count * 4
            }
            CMD_DRAIN => 1,
            CMD_END => raw.len(),
            CMD_BYTE_SINGLE => {
                let identifier = &raw[2..6];
                let vals = &raw[1..2];

                let first_level = identifier[3];
                let r = &raw[..CMD_BYTE_SINGLE_LENGTH];

                let cmd = match first_level {
                    0x00 => DspCmd::Monitor(MonitorCmd::parse(identifier, vals)),
                    0x01 => DspCmd::Input(InputCmd::parse(identifier, vals)),
                    0x02 => DspCmd::Mixer(MixerCmd::parse(identifier, vals)),
                    0x03 => DspCmd::Output(OutputCmd::parse(identifier, vals)),
                    0x04 => DspCmd::Reverb(ReverbCmd::parse(identifier, vals)),
                    _ => DspCmd::Reserved(r.to_vec()),
                };
                cmds.push(cmd);

                CMD_BYTE_SINGLE_LENGTH
            }
            CMD_QUADLET_SINGLE => {
                let identifier = &raw[1..5];
                let vals = &raw[5..9];

                let first_level = identifier[3];
                let r = &raw[..CMD_QUADLET_SINGLE_LENGTH];

                let cmd = match first_level {
                    0x00 => DspCmd::Monitor(MonitorCmd::parse(identifier, vals)),
                    0x01 => DspCmd::Input(InputCmd::parse(identifier, vals)),
                    0x02 => DspCmd::Mixer(MixerCmd::parse(identifier, vals)),
                    0x03 => DspCmd::Output(OutputCmd::parse(identifier, vals)),
                    0x04 => DspCmd::Reverb(ReverbCmd::parse(identifier, vals)),
                    _ => DspCmd::Reserved(r.to_vec()),
                };
                cmds.push(cmd);

                CMD_QUADLET_SINGLE_LENGTH
            }
            _ => 0,
        }
    }

    pub fn build(&self, raw: &mut Vec<u8>) {
        match self {
            DspCmd::Monitor(cmd) => cmd.build(raw),
            DspCmd::Input(cmd) => cmd.build(raw),
            DspCmd::Mixer(cmd) => cmd.build(raw),
            DspCmd::Output(cmd) => cmd.build(raw),
            DspCmd::Reverb(cmd) => cmd.build(raw),
            DspCmd::Resource(cmd) => cmd.build(raw),
            DspCmd::Reserved(data) => raw.extend_from_slice(data),
        }
    }
}

fn append_u8<T>(
    raw: &mut Vec<u8>,
    first_level: u8,
    second_level: u8,
    third_level: u8,
    ch: usize,
    val: T,
) where
    u8: From<T>,
{
    raw.push(CMD_BYTE_SINGLE);
    raw.push(u8::from(val));
    raw.push(ch as u8);
    raw.push(third_level);
    raw.push(second_level);
    raw.push(first_level);
}

fn append_i32(
    raw: &mut Vec<u8>,
    first_level: u8,
    second_level: u8,
    third_level: u8,
    ch: usize,
    val: i32,
) {
    append_f32(raw, first_level, second_level, third_level, ch, val as f32)
}

fn append_f32(
    raw: &mut Vec<u8>,
    first_level: u8,
    second_level: u8,
    third_level: u8,
    ch: usize,
    val: f32,
) {
    raw.push(CMD_QUADLET_SINGLE);
    raw.push(ch as u8);
    raw.push(third_level);
    raw.push(second_level);
    raw.push(first_level);
    raw.extend_from_slice(&val.to_le_bytes());
}

fn append_u32(
    raw: &mut Vec<u8>,
    first_level: u8,
    second_level: u8,
    third_level: u8,
    ch: usize,
    val: u32,
) {
    append_f32(raw, first_level, second_level, third_level, ch, val as f32)
}

fn append_resource(raw: &mut Vec<u8>, usage: f32, flag: u8) {
    raw.push(CMD_RESOURCE);
    raw.extend_from_slice(&usage.to_le_bytes());
    raw.push(flag);
}

// MEMO: The transaction frame can be truncated according to maximum length of frame (248 bytes).
// When truncated, the rest of frame is delivered by subsequent transaction.
//
// The sequence number is independent of the sequence number in message from the peer.
//
fn send_message(
    req: &mut FwReq,
    node: &mut FwNode,
    tag: u8,
    sequence_number: &mut u8,
    mut msg: &[u8],
    timeout_ms: u32,
) -> Result<(), Error> {
    while msg.len() > 0 {
        let length = std::cmp::min(msg.len(), MAXIMUM_DSP_FRAME_SIZE - 2);
        let mut frame = Vec::with_capacity(2 + length);
        frame.push(tag);
        frame.push(*sequence_number);
        frame.extend_from_slice(&msg[..length]);

        // The length of frame should be aligned to quadlet unit. If it's not, the unit becomes
        // not to transfer any messages voluntarily.
        while frame.len() % 4 > 0 {
            frame.push(0x00);
        }

        req.transaction_sync(
            node,
            FwTcode::WriteBlockRequest,
            DSP_CMD_OFFSET,
            frame.len(),
            &mut frame,
            timeout_ms,
        )?;

        *sequence_number += 1;
        *sequence_number %= 0xff;

        msg = &msg[length..];
    }

    Ok(())
}

/// The trait for operation of command DSP.
pub trait CommandDspOperation {
    fn send_commands(
        req: &mut FwReq,
        node: &mut FwNode,
        sequence_number: &mut u8,
        cmds: &[DspCmd],
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut frame = Vec::new();
        cmds.iter().for_each(|cmd| cmd.build(&mut frame));
        send_message(req, node, 0x02, sequence_number, &mut frame, timeout_ms)
    }

    fn register_message_destination_address(
        resp: &mut FwResp,
        req: &mut FwReq,
        node: &mut FwNode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        if !resp.is_reserved() {
            resp.reserve_within_region(
                node,
                MSG_DST_OFFSET_BEGIN,
                MSG_DST_OFFSET_END,
                8 + MAXIMUM_DSP_FRAME_SIZE as u32,
            )?;
        }

        let local_node_id = node.local_node_id() as u64;
        let addr = (local_node_id << 48) | resp.offset();

        let high = (addr >> 32) as u32;
        write_quad(req, node, DSP_MSG_DST_HIGH_OFFSET, high, timeout_ms)?;

        let low = (addr & 0xffffffff) as u32;
        write_quad(req, node, DSP_MSG_DST_LOW_OFFSET, low, timeout_ms)?;

        Ok(())
    }

    fn begin_messaging(
        req: &mut FwReq,
        node: &mut FwNode,
        sequence_number: &mut u8,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let frame = [0x00, 0x00];
        send_message(req, node, 0x01, sequence_number, &frame, timeout_ms)?;

        let frame = [0x00, 0x00];
        send_message(req, node, 0x02, sequence_number, &frame, timeout_ms)?;

        Ok(())
    }

    fn cancel_messaging(
        req: &mut FwReq,
        node: &mut FwNode,
        sequence_number: &mut u8,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let frame = [0x00, 0x00];
        send_message(req, node, 0x00, sequence_number, &frame, timeout_ms)
    }

    fn release_message_destination_address(
        resp: &mut FwResp,
        req: &mut FwReq,
        node: &mut FwNode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        write_quad(req, node, DSP_MSG_DST_HIGH_OFFSET, 0, timeout_ms)?;
        write_quad(req, node, DSP_MSG_DST_LOW_OFFSET, 0, timeout_ms)?;

        if resp.is_reserved() {
            resp.release();
        }

        Ok(())
    }
}

/// State of message parser.
#[derive(Debug)]
pub struct CommandDspMessageHandler {
    state: ParserState,
    cache: Vec<u8>,
    seq_num: u8,
}

#[derive(Debug, Eq, PartialEq)]
enum ParserState {
    Initialized,
    Prepared,
    InTruncatedMessage,
}

impl Default for CommandDspMessageHandler {
    fn default() -> Self {
        Self {
            state: ParserState::Initialized,
            cache: Vec::with_capacity(MAXIMUM_DSP_FRAME_SIZE + 6),
            seq_num: 0,
        }
    }
}

fn remove_padding(cache: &mut Vec<u8>) {
    let mut buf = &cache[..];
    let mut count = 0;

    while buf.len() > 4 {
        let length = match buf[0] {
            CMD_RESOURCE => CMD_RESOURCE_LENGTH,
            CMD_QUADLET_MULTIPLE => 6 + 4 * buf[1] as usize,
            CMD_BYTE_MULTIPLE => 6 + buf[1] as usize,
            CMD_DRAIN => 1,
            CMD_END => 0,
            CMD_QUADLET_SINGLE => CMD_QUADLET_SINGLE_LENGTH,
            CMD_BYTE_SINGLE => CMD_BYTE_SINGLE_LENGTH,
            _ => 0,
        };
        if length == 0 {
            break;
        }

        count += length;
        buf = &buf[length..];
    }

    let _ = cache.drain(count..);
}

fn increment_seq_num(seq_num: u8) -> u8 {
    if seq_num == u8::MAX {
        0
    } else {
        seq_num + 1
    }
}

impl CommandDspMessageHandler {
    // MEMO: After initiating messaging function by sending command with 0x02 in its first byte, the
    // target device start transferring messages immediately. There are two types of messages:
    //
    // Type 1: active sensing message
    // Type 2: commands
    //
    // In both, the fransaction frame has two bytes prefixes which consists of:
    //
    // 0: 0x00/0x01/0x02. Unknown purpose.
    // 1: sequence number, incremented within 1 byte.
    //
    // When message is split to several transactions due to maximum length of frame (248 bytes),
    // Type 1 message is not delivered between subsequent transactions.
    //
    pub fn cache_dsp_messages(&mut self, frame: &[u8]) {
        let seq_num = frame[1];

        if self.state == ParserState::Initialized {
            self.seq_num = seq_num;
            self.state = ParserState::Prepared;
        }

        if self.seq_num == seq_num {
            self.seq_num = increment_seq_num(seq_num);

            if self.state == ParserState::Prepared {
                // Check the type of first command in the message.
                if frame.len() > 4 && frame[2] != 0x00 {
                    self.cache.extend_from_slice(&frame[2..]);

                    if frame.len() == MAXIMUM_DSP_FRAME_SIZE {
                        self.state = ParserState::InTruncatedMessage;
                    } else {
                        remove_padding(&mut self.cache);
                    }
                }
            } else if self.state == ParserState::InTruncatedMessage {
                self.cache.extend_from_slice(&frame[2..]);

                if frame.len() < MAXIMUM_DSP_FRAME_SIZE {
                    remove_padding(&mut self.cache);
                    self.state = ParserState::Prepared;
                }
            }
        } else {
            self.cache.clear();
            self.state = ParserState::Prepared;
        }
    }

    pub fn has_dsp_message(&self) -> bool {
        self.cache.len() > 0 && (self.state == ParserState::Prepared)
    }

    pub fn decode_messages(&mut self) -> Vec<DspCmd> {
        let mut cmds = Vec::new();

        while self.cache.len() > 0 {
            let consumed = DspCmd::parse(&self.cache, &mut cmds);
            if consumed == 0 {
                break;
            }

            let _ = self.cache.drain(..consumed);
        }

        cmds
    }
}

/// The trait for parameter operations.
pub trait MotuCommandDspParametersOperation<T> {
    /// Build DSP commands for parameters.
    fn build_commands(params: &T) -> Vec<DspCmd>;
    /// Parse DSP command for parameters.
    fn parse_command(params: &mut T, command: &DspCmd) -> bool;
}

/// The trait for DSP image operations.
pub trait MotuCommandDspImageOperation<T, U> {
    /// Parse image transferred in the series of isochronous packets.
    fn parse_image(params: &mut T, image: &U);
}

/// The trait for operation to update parameters.
pub trait MotuCommandDspUpdatableParamsOperation<T> {
    /// Update the part of parameters.
    fn update_partially(
        req: &mut FwReq,
        node: &mut FwNode,
        sequence_number: &mut u8,
        params: &mut T,
        updates: T,
        timeout_ms: u32,
    ) -> Result<(), Error>;
}

impl<O, T> MotuCommandDspUpdatableParamsOperation<T> for O
where
    O: CommandDspOperation + MotuCommandDspParametersOperation<T>,
{
    fn update_partially(
        req: &mut FwReq,
        node: &mut FwNode,
        sequence_number: &mut u8,
        params: &mut T,
        updates: T,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut new_cmds = O::build_commands(&updates);
        let old_cmds = O::build_commands(params);
        new_cmds.retain(|cmd| old_cmds.iter().find(|c| c.eq(&cmd)).is_none());
        Self::send_commands(req, node, sequence_number, &new_cmds, timeout_ms)
            .map(|_| *params = updates)
    }
}

/// State of reverb function.
#[derive(Default, Debug, Copy, Clone, PartialEq)]
pub struct CommandDspReverbState {
    /// Whether to enable reverb effect.
    pub enable: bool,
    /// The split point to prevent feedback loop.
    pub split_point: SplitPoint,
    /// The time before first reflection.
    pub pre_delay: u32,
    /// The frequency of low pass in shelf filter.
    pub shelf_filter_freq: u32,
    /// The attenuation of low pass in shelf filter.
    pub shelf_filter_attenuation: i32,
    /// The time length of decay.
    pub decay_time: u32,
    /// The percentages against decay at low/middle/high frequencies.
    pub freq_time: [u32; 3],
    /// The point to cross over between low and middle, between middle and high,
    pub freq_crossover: [u32; 2],
    /// The width of stereo channels.
    pub width: f32,
    /// The mode of reflection.
    pub reflection_mode: RoomShape,
    /// The size of reflection.
    pub reflection_size: u32,
    /// The level of reflection.
    pub reflection_level: f32,
}

/// The specification of reverb.
pub trait MotuCommandDspReverbSpecification {
    /// The minimum value of decay time.
    const DECAY_TIME_MIN: u32 = 100;
    /// The maximum value of decay time.
    const DECAY_TIME_MAX: u32 = 60000;
    /// The step value of decay time.
    const DECAY_TIME_STEP: u32 = 1;

    /// The minimum value of pre decay time.
    const PRE_DELAY_MIN: u32 = 0;
    /// The maximum value of pre decay time.
    const PRE_DELAY_MAX: u32 = 100;
    /// The step value of pre decay time.
    const PRE_DELAY_STEP: u32 = 1;

    /// The minimum value of shelf filter.
    const SHELF_FILTER_FREQ_MIN: u32 = 1000;
    /// The maximum value of shelf filter.
    const SHELF_FILTER_FREQ_MAX: u32 = 20000;
    /// The step value of shelf filter.
    const SHELF_FILTER_FREQ_STEP: u32 = 1;

    /// The minimum value of shelf filter attenuation.
    const SHELF_FILTER_ATTR_MIN: i32 = -40;
    /// The maximum value of shelf filter attenuation.
    const SHELF_FILTER_ATTR_MAX: i32 = 0;
    /// The step value of shelf filter attenuation.
    const SHELF_FILTER_ATTR_STEP: i32 = 0;

    /// The number of frequency times.
    const FREQ_TIME_COUNT: usize = 3;
    /// The minimum value of frequency time.
    const FREQ_TIME_MIN: u32 = 0;
    /// The maximum value of frequency time.
    const FREQ_TIME_MAX: u32 = 100;
    /// The step value of frequency time.
    const FREQ_TIME_STEP: u32 = 1;

    /// The number of frequency crossovers.
    const FREQ_CROSSOVER_COUNT: usize = 2;
    /// The minimum value of frequency crossover.
    const FREQ_CROSSOVER_MIN: u32 = 100;
    /// The maximum value of frequency crossover.
    const FREQ_CROSSOVER_MAX: u32 = 20000;
    /// The step value of frequency crossover.
    const FREQ_CROSSOVER_STEP: u32 = 1;

    /// The minimum value of width.
    const WIDTH_MIN: f32 = -1.0;
    /// The maximum value of width.
    const WIDTH_MAX: f32 = 1.0;

    /// The minimum value of reflection size.
    const REFLECTION_SIZE_MIN: u32 = 50;
    /// The maximum value of reflection size.
    const REFLECTION_SIZE_MAX: u32 = 400;
    /// The step value of reflection size.
    const REFLECTION_SIZE_STEP: u32 = 1;

    /// The minimum value of reflection level.
    const REFLECTION_LEVEL_MIN: f32 = 0.0;
    /// The maximum value of reflection level.
    const REFLECTION_LEVEL_MAX: f32 = 1.0;
}

impl<O> MotuCommandDspParametersOperation<CommandDspReverbState> for O
where
    O: MotuCommandDspReverbSpecification,
{
    fn build_commands(params: &CommandDspReverbState) -> Vec<DspCmd> {
        vec![
            DspCmd::Reverb(ReverbCmd::Enable(params.enable)),
            DspCmd::Reverb(ReverbCmd::Split(params.split_point)),
            DspCmd::Reverb(ReverbCmd::PreDelay(params.pre_delay)),
            DspCmd::Reverb(ReverbCmd::ShelfFilterFreq(params.shelf_filter_freq)),
            DspCmd::Reverb(ReverbCmd::ShelfFilterAttenuation(
                params.shelf_filter_attenuation,
            )),
            DspCmd::Reverb(ReverbCmd::DecayTime(params.decay_time)),
            DspCmd::Reverb(ReverbCmd::LowFreqTime(params.freq_time[0])),
            DspCmd::Reverb(ReverbCmd::MiddleFreqTime(params.freq_time[1])),
            DspCmd::Reverb(ReverbCmd::HighFreqTime(params.freq_time[2])),
            DspCmd::Reverb(ReverbCmd::LowFreqCrossover(params.freq_crossover[0])),
            DspCmd::Reverb(ReverbCmd::HighFreqCrossover(params.freq_crossover[1])),
            DspCmd::Reverb(ReverbCmd::Width(params.width)),
            DspCmd::Reverb(ReverbCmd::ReflectionMode(params.reflection_mode)),
            DspCmd::Reverb(ReverbCmd::ReflectionSize(params.reflection_size)),
            DspCmd::Reverb(ReverbCmd::ReflectionLevel(params.reflection_level)),
        ]
    }

    fn parse_command(params: &mut CommandDspReverbState, command: &DspCmd) -> bool {
        if let DspCmd::Reverb(cmd) = command {
            match cmd {
                ReverbCmd::Enable(val) => params.enable = *val,
                ReverbCmd::Split(val) => params.split_point = *val,
                ReverbCmd::PreDelay(val) => params.pre_delay = *val,
                ReverbCmd::ShelfFilterFreq(val) => params.shelf_filter_freq = *val,
                ReverbCmd::ShelfFilterAttenuation(val) => params.shelf_filter_attenuation = *val,
                ReverbCmd::DecayTime(val) => params.decay_time = *val,
                ReverbCmd::LowFreqTime(val) => params.freq_time[0] = *val,
                ReverbCmd::MiddleFreqTime(val) => params.freq_time[1] = *val,
                ReverbCmd::HighFreqTime(val) => params.freq_time[2] = *val,
                ReverbCmd::LowFreqCrossover(val) => params.freq_crossover[0] = *val,
                ReverbCmd::HighFreqCrossover(val) => params.freq_crossover[1] = *val,
                ReverbCmd::Width(val) => params.width = *val,
                ReverbCmd::ReflectionMode(val) => params.reflection_mode = *val,
                ReverbCmd::ReflectionSize(val) => params.reflection_size = *val,
                ReverbCmd::ReflectionLevel(val) => params.reflection_level = *val,
                _ => (),
            };
            true
        } else {
            false
        }
    }
}

/// State of monitor function.
#[derive(Default, Debug, Copy, Clone, PartialEq)]
pub struct CommandDspMonitorState {
    /// The volume adjusted by main (master) knob. -inf (mute), -80.0 dB to 0.0 dB.
    pub main_volume: f32,
    /// Whether to enable talkback or not.
    pub talkback_enable: bool,
    /// Whether to listenback or not.
    pub listenback_enable: bool,
    /// The volume of talkback.
    pub talkback_volume: f32,
    /// The volume of listenback.
    pub listenback_volume: f32,
    /// Input or output to focus on.
    pub focus: FocusTarget,
    /// The target to focus on.
    pub assign_target: TargetPort,
}

/// The trait for specification of monitor.
pub trait MotuCommandDspMonitorSpecification {
    /// The targets of mixer return.
    const RETURN_ASSIGN_TARGETS: &'static [TargetPort];

    /// The minimum value of volume for monitor output.
    const VOLUME_MIN: f32 = 0.0;
    /// The maximum value of volume for monitor output.
    const VOLUME_MAX: f32 = 1.0;
}

impl<O> MotuCommandDspParametersOperation<CommandDspMonitorState> for O
where
    O: MotuCommandDspMonitorSpecification,
{
    fn build_commands(params: &CommandDspMonitorState) -> Vec<DspCmd> {
        let pos = Self::RETURN_ASSIGN_TARGETS
            .iter()
            .position(|p| params.assign_target.eq(p))
            .unwrap_or_default();

        vec![
            DspCmd::Monitor(MonitorCmd::Volume(params.main_volume)),
            DspCmd::Monitor(MonitorCmd::TalkbackEnable(params.talkback_enable)),
            DspCmd::Monitor(MonitorCmd::ListenbackEnable(params.listenback_enable)),
            DspCmd::Monitor(MonitorCmd::TalkbackVolume(params.talkback_volume)),
            DspCmd::Monitor(MonitorCmd::ListenbackVolume(params.listenback_volume)),
            DspCmd::Monitor(MonitorCmd::Focus(params.focus)),
            DspCmd::Monitor(MonitorCmd::ReturnAssign(pos)),
        ]
    }

    fn parse_command(params: &mut CommandDspMonitorState, command: &DspCmd) -> bool {
        if let DspCmd::Monitor(cmd) = command {
            match cmd {
                MonitorCmd::Volume(val) => params.main_volume = *val,
                MonitorCmd::TalkbackEnable(val) => params.talkback_enable = *val,
                MonitorCmd::ListenbackEnable(val) => params.listenback_enable = *val,
                MonitorCmd::TalkbackVolume(val) => params.talkback_volume = *val,
                MonitorCmd::ListenbackVolume(val) => params.listenback_volume = *val,
                MonitorCmd::Focus(val) => params.focus = *val,
                MonitorCmd::ReturnAssign(val) => {
                    params.assign_target = Self::RETURN_ASSIGN_TARGETS
                        .iter()
                        .nth(*val as usize)
                        .map(|&p| p)
                        .unwrap_or_default();
                }
                _ => (),
            };
            true
        } else {
            false
        }
    }
}

/// State of entry of mixer function.
#[derive(Default, Debug, Clone, PartialEq)]
pub struct CommandDspMixerSourceState {
    /// Whether to mute the source of mixer.
    pub mute: Vec<bool>,
    /// Whether to mute the other sources of mixer.
    pub solo: Vec<bool>,
    /// The gain for source of mixer.
    pub gain: Vec<f32>,
    /// The left and right balance for source of mixer.
    pub pan: Vec<f32>,
    /// The mode of stereo pair.
    pub stereo_mode: Vec<SourceStereoPairMode>,
    /// The left and right balance for source of mixer when paired.
    pub stereo_balance: Vec<f32>,
    /// The left and right width for source of mixer when paired.
    pub stereo_width: Vec<f32>,
}

const MIXER_COUNT: usize = 8;

/// State of mixer function.
#[derive(Default, Debug, Clone, PartialEq)]
pub struct CommandDspMixerState {
    /// The destination of mixer outputs.
    pub output_assign: [TargetPort; MIXER_COUNT],
    /// Whether to mute mixer outputs.
    pub output_mute: [bool; MIXER_COUNT],
    /// The volume of mixer outputs.
    pub output_volume: [f32; MIXER_COUNT],
    /// The volume to send to reverb effect.
    pub reverb_send: [f32; MIXER_COUNT],
    /// The gain to return from reverb effect.
    pub reverb_return: [f32; MIXER_COUNT],
    /// The parameters of mixer sources.
    pub source: [CommandDspMixerSourceState; MIXER_COUNT],
}

/// The trait for specification of mixer.
pub trait MotuCommandDspMixerSpecification {
    /// The sources of mixer inputs.
    const SOURCE_PORTS: &'static [TargetPort];
    /// The destination of mixer outputs.
    const OUTPUT_PORTS: &'static [TargetPort];

    /// The number of mixers.
    const MIXER_COUNT: usize = MIXER_COUNT;

    /// The minimum value of volume for mixer output.
    const OUTPUT_VOLUME_MIN: f32 = 0.0;
    /// The maximum value of volume for mixer output.
    const OUTPUT_VOLUME_MAX: f32 = 1.0;

    /// The minimum value of gain for mixer source.
    const SOURCE_GAIN_MIN: f32 = 0.0;
    /// The maximum value of gain for mixer source.
    const SOURCE_GAIN_MAX: f32 = 1.0;

    /// The minimum value of left and right balance for mixer source.
    const SOURCE_PAN_MIN: f32 = -1.0;
    /// The maximum value of left and right balance for mixer source.
    const SOURCE_PAN_MAX: f32 = 1.0;

    fn create_mixer_state() -> CommandDspMixerState {
        let mut state = CommandDspMixerState::default();

        state.source.iter_mut().for_each(|src| {
            src.mute = vec![Default::default(); Self::SOURCE_PORTS.len()];
            src.solo = vec![Default::default(); Self::SOURCE_PORTS.len()];
            src.gain = vec![Default::default(); Self::SOURCE_PORTS.len()];
            src.pan = vec![Default::default(); Self::SOURCE_PORTS.len()];
            src.stereo_mode = vec![Default::default(); Self::SOURCE_PORTS.len()];
            src.stereo_balance = vec![Default::default(); Self::SOURCE_PORTS.len()];
            src.stereo_width = vec![Default::default(); Self::SOURCE_PORTS.len()];
        });

        state
    }
}

impl<O> MotuCommandDspParametersOperation<CommandDspMixerState> for O
where
    O: MotuCommandDspMixerSpecification,
{
    fn build_commands(params: &CommandDspMixerState) -> Vec<DspCmd> {
        let mut cmds = Vec::new();

        (0..MIXER_COUNT).for_each(|mixer| {
            let pos = Self::OUTPUT_PORTS
                .iter()
                .position(|p| params.output_assign[mixer].eq(p))
                .unwrap_or_default();
            cmds.push(DspCmd::Mixer(MixerCmd::OutputAssign(mixer, pos)));
            cmds.push(DspCmd::Mixer(MixerCmd::OutputMute(
                mixer,
                params.output_mute[mixer],
            )));
            cmds.push(DspCmd::Mixer(MixerCmd::OutputVolume(
                mixer,
                params.output_volume[mixer],
            )));
            cmds.push(DspCmd::Mixer(MixerCmd::ReverbSend(
                mixer,
                params.reverb_send[mixer],
            )));
            cmds.push(DspCmd::Mixer(MixerCmd::ReverbReturn(
                mixer,
                params.reverb_return[mixer],
            )));

            let src = &params.source[mixer];
            (0..Self::SOURCE_PORTS.len()).for_each(|ch| {
                cmds.push(DspCmd::Mixer(MixerCmd::SourceMute(mixer, ch, src.mute[ch])));
                cmds.push(DspCmd::Mixer(MixerCmd::SourceSolo(mixer, ch, src.solo[ch])));
                cmds.push(DspCmd::Mixer(MixerCmd::SourceGain(mixer, ch, src.gain[ch])));
                cmds.push(DspCmd::Mixer(MixerCmd::SourceMonauralLrBalance(
                    mixer,
                    ch,
                    src.pan[ch],
                )));
                cmds.push(DspCmd::Mixer(MixerCmd::SourceStereoMode(
                    mixer,
                    ch,
                    src.stereo_mode[ch],
                )));
                cmds.push(DspCmd::Mixer(MixerCmd::SourceStereoLrBalance(
                    mixer,
                    ch,
                    src.stereo_balance[ch],
                )));
                cmds.push(DspCmd::Mixer(MixerCmd::SourceStereoWidth(
                    mixer,
                    ch,
                    src.stereo_width[ch],
                )));
            });
        });

        cmds
    }

    fn parse_command(params: &mut CommandDspMixerState, command: &DspCmd) -> bool {
        if let DspCmd::Mixer(cmd) = command {
            match cmd {
                MixerCmd::OutputAssign(mixer, val) => {
                    params.output_assign[*mixer] = Self::OUTPUT_PORTS
                        .iter()
                        .nth(*val)
                        .map(|&p| p)
                        .unwrap_or_else(|| Self::OUTPUT_PORTS[0]);
                }
                MixerCmd::OutputMute(mixer, val) => params.output_mute[*mixer] = *val,
                MixerCmd::OutputVolume(mixer, val) => params.output_volume[*mixer] = *val,
                MixerCmd::ReverbSend(mixer, val) => params.reverb_send[*mixer] = *val,
                MixerCmd::ReverbReturn(mixer, val) => params.reverb_return[*mixer] = *val,
                MixerCmd::SourceMute(mixer, src, val) => params.source[*mixer].mute[*src] = *val,
                MixerCmd::SourceSolo(mixer, src, val) => params.source[*mixer].solo[*src] = *val,
                MixerCmd::SourceGain(mixer, src, val) => params.source[*mixer].gain[*src] = *val,
                MixerCmd::SourceMonauralLrBalance(mixer, src, val) => {
                    params.source[*mixer].pan[*src] = *val
                }
                MixerCmd::SourceStereoMode(mixer, src, val) => {
                    params.source[*mixer].stereo_mode[*src] = *val
                }
                MixerCmd::SourceStereoLrBalance(mixer, src, val) => {
                    params.source[*mixer].stereo_balance[*src] = *val
                }
                MixerCmd::SourceStereoWidth(mixer, src, val) => {
                    params.source[*mixer].stereo_width[*src] = *val
                }
                _ => (),
            };
            true
        } else {
            false
        }
    }
}

/// State of equalizer.
#[derive(Default, Debug, Clone, PartialEq)]
pub struct CommandDspEqualizerState {
    /// Whether to enable whole equalizer.
    pub enable: Vec<bool>,

    /// Whether to enable high pass filter.
    pub hpf_enable: Vec<bool>,
    /// The type of slope for high pass filter.
    pub hpf_slope: Vec<RollOffLevel>,
    /// The frequency of high pass filter.
    pub hpf_freq: Vec<u32>,

    /// Whether to enable low pass filter.
    pub lpf_enable: Vec<bool>,
    /// The type of slope for loa pass filter.
    pub lpf_slope: Vec<RollOffLevel>,
    /// The frequency of low pass filter.
    pub lpf_freq: Vec<u32>,

    /// Whether to enable equalizer at low frequency.
    pub lf_enable: Vec<bool>,
    /// The type of equalizer at low frequency.
    pub lf_type: Vec<FilterType5>,
    /// The center frequency of equalizer at low frequency.
    pub lf_freq: Vec<u32>,
    /// The gain of equalizer at low frequency.
    pub lf_gain: Vec<f32>,
    /// The width of equalizer at low frequency.
    pub lf_width: Vec<f32>,

    /// Whether to enable equalizer at low-middle frequency
    pub lmf_enable: Vec<bool>,
    /// The type of equalizer at low-middle frequency
    pub lmf_type: Vec<FilterType4>,
    /// The center frequency of equalizer at low-middle frequency
    pub lmf_freq: Vec<u32>,
    /// The gain of equalizer at low-middle frequency
    pub lmf_gain: Vec<f32>,
    /// The width of equalizer at low-middle frequency
    pub lmf_width: Vec<f32>,

    /// Whether to enable equalizer at middle frequency.
    pub mf_enable: Vec<bool>,
    /// The type of equalizer at middle frequency.
    pub mf_type: Vec<FilterType4>,
    /// The center frequency of equalizer at middle frequency.
    pub mf_freq: Vec<u32>,
    /// The gain of equalizer at middle frequency.
    pub mf_gain: Vec<f32>,
    /// The width of equalizer at middle frequency.
    pub mf_width: Vec<f32>,

    /// Whether to enable equalizer at high-middle frequency.
    pub hmf_enable: Vec<bool>,
    /// The type of equalizer at high-middle frequency.
    pub hmf_type: Vec<FilterType4>,
    /// The center frequency of equalizer at high-middle frequency.
    pub hmf_freq: Vec<u32>,
    /// The gain of equalizer at high-middle frequency.
    pub hmf_gain: Vec<f32>,
    /// The width of equalizer at high-middle frequency.
    pub hmf_width: Vec<f32>,

    /// Whether to enable equalizer at high frequency.
    pub hf_enable: Vec<bool>,
    /// The type of equalizer at high frequency.
    pub hf_type: Vec<FilterType5>,
    /// The center frequency of equalizer at high frequency.
    pub hf_freq: Vec<u32>,
    /// The gain of equalizer at high frequency.
    pub hf_gain: Vec<f32>,
    /// The width of equalizer at high frequency.
    pub hf_width: Vec<f32>,
}

/// The trait for specification of equalizer effect.
pub trait MotuCommandDspEqualizerSpecification {
    /// The minimum value of frequency.
    const EQUALIZER_FREQ_MIN: u32 = 20;
    /// The maximum value of frequency.
    const EQUALIZER_FREQ_MAX: u32 = 20000;
    /// The step value of frequency.
    const EQUALIZER_FREQ_STEP: u32 = 1;

    /// The minimum value of gain.
    const EQUALIZER_GAIN_MIN: f32 = -20.0;
    /// The maximum value of gain.
    const EQUALIZER_GAIN_MAX: f32 = 20.0;

    /// The minimum value of width.
    const EQUALIZER_WIDTH_MIN: f32 = 0.01;
    /// The maximum value of width.
    const EQUALIZER_WIDTH_MAX: f32 = 3.0;

    fn create_equalizer_parameters(
        state: &CommandDspEqualizerState,
        ch: usize,
    ) -> Vec<EqualizerParameter> {
        let mut params = Vec::new();

        params.push(EqualizerParameter::Enable(state.enable[ch]));

        params.push(EqualizerParameter::HpfEnable(state.hpf_enable[ch]));
        params.push(EqualizerParameter::HpfSlope(state.hpf_slope[ch]));
        params.push(EqualizerParameter::HpfFreq(state.hpf_freq[ch]));

        params.push(EqualizerParameter::LpfEnable(state.lpf_enable[ch]));
        params.push(EqualizerParameter::LpfSlope(state.lpf_slope[ch]));
        params.push(EqualizerParameter::LpfFreq(state.lpf_freq[ch]));

        params.push(EqualizerParameter::LfEnable(state.lf_enable[ch]));
        params.push(EqualizerParameter::LfType(state.lf_type[ch]));
        params.push(EqualizerParameter::LfFreq(state.lf_freq[ch]));
        params.push(EqualizerParameter::LfGain(state.lf_gain[ch]));
        params.push(EqualizerParameter::LfWidth(state.lf_width[ch]));

        params.push(EqualizerParameter::LmfEnable(state.lmf_enable[ch]));
        params.push(EqualizerParameter::LmfType(state.lmf_type[ch]));
        params.push(EqualizerParameter::LmfFreq(state.lmf_freq[ch]));
        params.push(EqualizerParameter::LmfGain(state.lmf_gain[ch]));
        params.push(EqualizerParameter::LmfWidth(state.lmf_width[ch]));

        params.push(EqualizerParameter::MfEnable(state.mf_enable[ch]));
        params.push(EqualizerParameter::MfType(state.mf_type[ch]));
        params.push(EqualizerParameter::MfFreq(state.mf_freq[ch]));
        params.push(EqualizerParameter::MfGain(state.mf_gain[ch]));
        params.push(EqualizerParameter::MfWidth(state.mf_width[ch]));

        params.push(EqualizerParameter::HmfEnable(state.hmf_enable[ch]));
        params.push(EqualizerParameter::HmfType(state.hmf_type[ch]));
        params.push(EqualizerParameter::HmfFreq(state.hmf_freq[ch]));
        params.push(EqualizerParameter::HmfGain(state.hmf_gain[ch]));
        params.push(EqualizerParameter::HmfWidth(state.hmf_width[ch]));

        params.push(EqualizerParameter::HfEnable(state.hf_enable[ch]));
        params.push(EqualizerParameter::HfType(state.hf_type[ch]));
        params.push(EqualizerParameter::HfFreq(state.hf_freq[ch]));
        params.push(EqualizerParameter::HfGain(state.hf_gain[ch]));
        params.push(EqualizerParameter::HfWidth(state.hf_width[ch]));

        params
    }

    fn parse_equalizer_parameter(
        state: &mut CommandDspEqualizerState,
        param: &EqualizerParameter,
        ch: usize,
    ) {
        match param {
            EqualizerParameter::Enable(val) => state.enable[ch] = *val,

            EqualizerParameter::HpfEnable(val) => state.hpf_enable[ch] = *val,
            EqualizerParameter::HpfSlope(val) => state.hpf_slope[ch] = *val,
            EqualizerParameter::HpfFreq(val) => state.hpf_freq[ch] = *val,

            EqualizerParameter::LpfEnable(val) => state.lpf_enable[ch] = *val,
            EqualizerParameter::LpfSlope(val) => state.lpf_slope[ch] = *val,
            EqualizerParameter::LpfFreq(val) => state.lpf_freq[ch] = *val,

            EqualizerParameter::LfEnable(val) => state.lf_enable[ch] = *val,
            EqualizerParameter::LfType(val) => state.lf_type[ch] = *val,
            EqualizerParameter::LfFreq(val) => state.lf_freq[ch] = *val,
            EqualizerParameter::LfGain(val) => state.lf_gain[ch] = *val,
            EqualizerParameter::LfWidth(val) => state.lf_width[ch] = *val,

            EqualizerParameter::LmfEnable(val) => state.lmf_enable[ch] = *val,
            EqualizerParameter::LmfType(val) => state.lmf_type[ch] = *val,
            EqualizerParameter::LmfFreq(val) => state.lmf_freq[ch] = *val,
            EqualizerParameter::LmfGain(val) => state.lmf_gain[ch] = *val,
            EqualizerParameter::LmfWidth(val) => state.lmf_width[ch] = *val,

            EqualizerParameter::MfEnable(val) => state.mf_enable[ch] = *val,
            EqualizerParameter::MfType(val) => state.mf_type[ch] = *val,
            EqualizerParameter::MfFreq(val) => state.mf_freq[ch] = *val,
            EqualizerParameter::MfGain(val) => state.mf_gain[ch] = *val,
            EqualizerParameter::MfWidth(val) => state.mf_width[ch] = *val,

            EqualizerParameter::HmfEnable(val) => state.hmf_enable[ch] = *val,
            EqualizerParameter::HmfType(val) => state.hmf_type[ch] = *val,
            EqualizerParameter::HmfFreq(val) => state.hmf_freq[ch] = *val,
            EqualizerParameter::HmfGain(val) => state.hmf_gain[ch] = *val,
            EqualizerParameter::HmfWidth(val) => state.hmf_width[ch] = *val,

            EqualizerParameter::HfEnable(val) => state.hf_enable[ch] = *val,
            EqualizerParameter::HfType(val) => state.hf_type[ch] = *val,
            EqualizerParameter::HfFreq(val) => state.hf_freq[ch] = *val,
            EqualizerParameter::HfGain(val) => state.hf_gain[ch] = *val,
            EqualizerParameter::HfWidth(val) => state.hf_width[ch] = *val,
        }
    }
}

fn create_equalizer_parameters(
    state: &CommandDspEqualizerState,
    ch: usize,
) -> Vec<EqualizerParameter> {
    let mut params = Vec::new();

    params.push(EqualizerParameter::Enable(state.enable[ch]));

    params.push(EqualizerParameter::HpfEnable(state.hpf_enable[ch]));
    params.push(EqualizerParameter::HpfSlope(state.hpf_slope[ch]));
    params.push(EqualizerParameter::HpfFreq(state.hpf_freq[ch]));

    params.push(EqualizerParameter::LpfEnable(state.lpf_enable[ch]));
    params.push(EqualizerParameter::LpfSlope(state.lpf_slope[ch]));
    params.push(EqualizerParameter::LpfFreq(state.lpf_freq[ch]));

    params.push(EqualizerParameter::LfEnable(state.lf_enable[ch]));
    params.push(EqualizerParameter::LfType(state.lf_type[ch]));
    params.push(EqualizerParameter::LfFreq(state.lf_freq[ch]));
    params.push(EqualizerParameter::LfGain(state.lf_gain[ch]));
    params.push(EqualizerParameter::LfWidth(state.lf_width[ch]));

    params.push(EqualizerParameter::LmfEnable(state.lmf_enable[ch]));
    params.push(EqualizerParameter::LmfType(state.lmf_type[ch]));
    params.push(EqualizerParameter::LmfFreq(state.lmf_freq[ch]));
    params.push(EqualizerParameter::LmfGain(state.lmf_gain[ch]));
    params.push(EqualizerParameter::LmfWidth(state.lmf_width[ch]));

    params.push(EqualizerParameter::MfEnable(state.mf_enable[ch]));
    params.push(EqualizerParameter::MfType(state.mf_type[ch]));
    params.push(EqualizerParameter::MfFreq(state.mf_freq[ch]));
    params.push(EqualizerParameter::MfGain(state.mf_gain[ch]));
    params.push(EqualizerParameter::MfWidth(state.mf_width[ch]));

    params.push(EqualizerParameter::HmfEnable(state.hmf_enable[ch]));
    params.push(EqualizerParameter::HmfType(state.hmf_type[ch]));
    params.push(EqualizerParameter::HmfFreq(state.hmf_freq[ch]));
    params.push(EqualizerParameter::HmfGain(state.hmf_gain[ch]));
    params.push(EqualizerParameter::HmfWidth(state.hmf_width[ch]));

    params.push(EqualizerParameter::HfEnable(state.hf_enable[ch]));
    params.push(EqualizerParameter::HfType(state.hf_type[ch]));
    params.push(EqualizerParameter::HfFreq(state.hf_freq[ch]));
    params.push(EqualizerParameter::HfGain(state.hf_gain[ch]));
    params.push(EqualizerParameter::HfWidth(state.hf_width[ch]));

    params
}

fn parse_equalizer_parameter(
    state: &mut CommandDspEqualizerState,
    param: &EqualizerParameter,
    ch: usize,
) {
    match param {
        EqualizerParameter::Enable(val) => state.enable[ch] = *val,

        EqualizerParameter::HpfEnable(val) => state.hpf_enable[ch] = *val,
        EqualizerParameter::HpfSlope(val) => state.hpf_slope[ch] = *val,
        EqualizerParameter::HpfFreq(val) => state.hpf_freq[ch] = *val,

        EqualizerParameter::LpfEnable(val) => state.lpf_enable[ch] = *val,
        EqualizerParameter::LpfSlope(val) => state.lpf_slope[ch] = *val,
        EqualizerParameter::LpfFreq(val) => state.lpf_freq[ch] = *val,

        EqualizerParameter::LfEnable(val) => state.lf_enable[ch] = *val,
        EqualizerParameter::LfType(val) => state.lf_type[ch] = *val,
        EqualizerParameter::LfFreq(val) => state.lf_freq[ch] = *val,
        EqualizerParameter::LfGain(val) => state.lf_gain[ch] = *val,
        EqualizerParameter::LfWidth(val) => state.lf_width[ch] = *val,

        EqualizerParameter::LmfEnable(val) => state.lmf_enable[ch] = *val,
        EqualizerParameter::LmfType(val) => state.lmf_type[ch] = *val,
        EqualizerParameter::LmfFreq(val) => state.lmf_freq[ch] = *val,
        EqualizerParameter::LmfGain(val) => state.lmf_gain[ch] = *val,
        EqualizerParameter::LmfWidth(val) => state.lmf_width[ch] = *val,

        EqualizerParameter::MfEnable(val) => state.mf_enable[ch] = *val,
        EqualizerParameter::MfType(val) => state.mf_type[ch] = *val,
        EqualizerParameter::MfFreq(val) => state.mf_freq[ch] = *val,
        EqualizerParameter::MfGain(val) => state.mf_gain[ch] = *val,
        EqualizerParameter::MfWidth(val) => state.mf_width[ch] = *val,

        EqualizerParameter::HmfEnable(val) => state.hmf_enable[ch] = *val,
        EqualizerParameter::HmfType(val) => state.hmf_type[ch] = *val,
        EqualizerParameter::HmfFreq(val) => state.hmf_freq[ch] = *val,
        EqualizerParameter::HmfGain(val) => state.hmf_gain[ch] = *val,
        EqualizerParameter::HmfWidth(val) => state.hmf_width[ch] = *val,

        EqualizerParameter::HfEnable(val) => state.hf_enable[ch] = *val,
        EqualizerParameter::HfType(val) => state.hf_type[ch] = *val,
        EqualizerParameter::HfFreq(val) => state.hf_freq[ch] = *val,
        EqualizerParameter::HfGain(val) => state.hf_gain[ch] = *val,
        EqualizerParameter::HfWidth(val) => state.hf_width[ch] = *val,
    }
}

/// State of dynamics.
#[derive(Default, Debug, Clone, PartialEq)]
pub struct CommandDspDynamicsState {
    /// whether to enable dynamics effects.
    pub enable: Vec<bool>,

    /// Whether to enable compressors.
    pub comp_enable: Vec<bool>,
    /// The mode to detect level in compressors.
    pub comp_detect_mode: Vec<LevelDetectMode>,
    /// The threshold of compressors.
    pub comp_threshold: Vec<i32>,
    /// The ratio of compressors.
    pub comp_ratio: Vec<f32>,
    /// The attack of compressors.
    pub comp_attack: Vec<u32>,
    /// The release of compressors.
    pub comp_release: Vec<u32>,
    /// The gain of compressors.
    pub comp_gain: Vec<f32>,

    /// Whether to enable levelers.
    pub leveler_enable: Vec<bool>,
    /// The mode of levelers.
    pub leveler_mode: Vec<LevelerMode>,
    /// The markup of levelers.
    pub leveler_makeup: Vec<u32>,
    /// The reduce of levelers.
    pub leveler_reduce: Vec<u32>,
}

/// The trait for specification of dynamics effect.
pub trait MotuCommandDspDynamicsSpecification {
    /// The minimum value of threshold in compressor.
    const COMP_THRESHOLD_MIN: i32 = -48;
    /// The maximum value of threshold in compressor.
    const COMP_THRESHOLD_MAX: i32 = 0;
    /// The step value of threshold in compressor.
    const COMP_THRESHOLD_STEP: i32 = 1;

    /// The minimum value of ration in compressor.
    const COMP_RATIO_MIN: f32 = 1.0;
    /// The maximum value of ration in compressor.
    const COMP_RATIO_MAX: f32 = 10.0;

    /// The minimum value of attack in compressor.
    const COMP_ATTACK_MIN: i32 = 10;
    /// The maximum value of attack in compressor.
    const COMP_ATTACK_MAX: i32 = 100;
    /// The step value of attack in compressor.
    const COMP_ATTACK_STEP: i32 = 1;

    /// The minimum value of release in compressor.
    const COMP_RELEASE_MIN: i32 = 10;
    /// The maximum value of release in compressor.
    const COMP_RELEASE_MAX: i32 = 100;
    /// The step value of release in compressor.
    const COMP_RELEASE_STEP: i32 = 1;

    /// The minimum value of gain in compressor.
    const COMP_GAIN_MIN: f32 = -6.0;
    /// The maximum value of gain in compressor.
    const COMP_GAIN_MAX: f32 = 0.0;

    /// The minimum value of percentge in leveler.
    const LEVELER_PERCENTAGE_MIN: u32 = 0;
    /// The maximum value of percentge in leveler.
    const LEVELER_PERCENTAGE_MAX: u32 = 100;
    /// The step value of percentge in leveler.
    const LEVELER_PERCENTAGE_STEP: u32 = 1;

    fn create_dynamics_parameters(
        state: &CommandDspDynamicsState,
        ch: usize,
    ) -> Vec<DynamicsParameter> {
        let mut params = Vec::new();

        params.push(DynamicsParameter::Enable(state.enable[ch]));

        params.push(DynamicsParameter::CompEnable(state.comp_enable[ch]));
        params.push(DynamicsParameter::CompDetectMode(
            state.comp_detect_mode[ch],
        ));
        params.push(DynamicsParameter::CompThreshold(state.comp_threshold[ch]));
        params.push(DynamicsParameter::CompRatio(state.comp_ratio[ch]));
        params.push(DynamicsParameter::CompAttack(state.comp_attack[ch]));
        params.push(DynamicsParameter::CompRelease(state.comp_release[ch]));
        params.push(DynamicsParameter::CompGain(state.comp_gain[ch]));

        params.push(DynamicsParameter::LevelerEnable(state.leveler_enable[ch]));
        params.push(DynamicsParameter::LevelerMode(state.leveler_mode[ch]));
        params.push(DynamicsParameter::LevelerMakeup(state.leveler_makeup[ch]));
        params.push(DynamicsParameter::LevelerReduce(state.leveler_reduce[ch]));

        params
    }

    fn parse_dynamics_parameter(
        state: &mut CommandDspDynamicsState,
        param: &DynamicsParameter,
        ch: usize,
    ) {
        match param {
            DynamicsParameter::Enable(val) => state.enable[ch] = *val,

            DynamicsParameter::CompEnable(val) => state.comp_enable[ch] = *val,
            DynamicsParameter::CompDetectMode(val) => state.comp_detect_mode[ch] = *val,
            DynamicsParameter::CompThreshold(val) => state.comp_threshold[ch] = *val,
            DynamicsParameter::CompRatio(val) => state.comp_ratio[ch] = *val,
            DynamicsParameter::CompAttack(val) => state.comp_attack[ch] = *val,
            DynamicsParameter::CompRelease(val) => state.comp_release[ch] = *val,
            DynamicsParameter::CompGain(val) => state.comp_gain[ch] = *val,

            DynamicsParameter::LevelerEnable(val) => state.leveler_enable[ch] = *val,
            DynamicsParameter::LevelerMode(val) => state.leveler_mode[ch] = *val,
            DynamicsParameter::LevelerMakeup(val) => state.leveler_makeup[ch] = *val,
            DynamicsParameter::LevelerReduce(val) => state.leveler_reduce[ch] = *val,
        }
    }
}

fn create_dynamics_parameters(
    state: &CommandDspDynamicsState,
    ch: usize,
) -> Vec<DynamicsParameter> {
    let mut params = Vec::new();

    params.push(DynamicsParameter::Enable(state.enable[ch]));

    params.push(DynamicsParameter::CompEnable(state.comp_enable[ch]));
    params.push(DynamicsParameter::CompDetectMode(
        state.comp_detect_mode[ch],
    ));
    params.push(DynamicsParameter::CompThreshold(state.comp_threshold[ch]));
    params.push(DynamicsParameter::CompRatio(state.comp_ratio[ch]));
    params.push(DynamicsParameter::CompAttack(state.comp_attack[ch]));
    params.push(DynamicsParameter::CompRelease(state.comp_release[ch]));
    params.push(DynamicsParameter::CompGain(state.comp_gain[ch]));

    params.push(DynamicsParameter::LevelerEnable(state.leveler_enable[ch]));
    params.push(DynamicsParameter::LevelerMode(state.leveler_mode[ch]));
    params.push(DynamicsParameter::LevelerMakeup(state.leveler_makeup[ch]));
    params.push(DynamicsParameter::LevelerReduce(state.leveler_reduce[ch]));

    params
}

fn parse_dynamics_parameter(
    state: &mut CommandDspDynamicsState,
    param: &DynamicsParameter,
    ch: usize,
) {
    match param {
        DynamicsParameter::Enable(val) => state.enable[ch] = *val,

        DynamicsParameter::CompEnable(val) => state.comp_enable[ch] = *val,
        DynamicsParameter::CompDetectMode(val) => state.comp_detect_mode[ch] = *val,
        DynamicsParameter::CompThreshold(val) => state.comp_threshold[ch] = *val,
        DynamicsParameter::CompRatio(val) => state.comp_ratio[ch] = *val,
        DynamicsParameter::CompAttack(val) => state.comp_attack[ch] = *val,
        DynamicsParameter::CompRelease(val) => state.comp_release[ch] = *val,
        DynamicsParameter::CompGain(val) => state.comp_gain[ch] = *val,

        DynamicsParameter::LevelerEnable(val) => state.leveler_enable[ch] = *val,
        DynamicsParameter::LevelerMode(val) => state.leveler_mode[ch] = *val,
        DynamicsParameter::LevelerMakeup(val) => state.leveler_makeup[ch] = *val,
        DynamicsParameter::LevelerReduce(val) => state.leveler_reduce[ch] = *val,
    }
}

/// State of input function.
#[derive(Default, Debug, Clone, PartialEq)]
pub struct CommandDspInputState {
    /// Whether to invert phase of input signals.
    pub phase: Vec<bool>,
    /// Whether to enable stereo pair.
    pub pair: Vec<bool>,
    /// The gain of inputs.
    pub gain: Vec<i32>,
    /// Whether to swap a pair of channels.
    pub swap: Vec<bool>,
    /// The mode of stereo pair.
    pub stereo_mode: Vec<InputStereoPairMode>,
    /// The left and right width of stereo pair.
    pub width: Vec<f32>,

    /// The volume to send to reverb effect.
    pub reverb_send: Vec<f32>,
    /// The left and right balance to send to reverb effect.
    pub reverb_balance: Vec<f32>,

    /// The parameters of qeualizers for each input.
    pub equalizer: CommandDspEqualizerState,
    /// The parameters of dynamics for each input.
    pub dynamics: CommandDspDynamicsState,

    /// Whether to attenuate inputs.
    pub pad: Vec<bool>,
    /// The nominal level of inputs.
    pub nominal_level: Vec<NominalSignalLevel>,
    /// Whether to enable phantom powering for inputs.
    pub phantom: Vec<bool>,
    /// Whether to enable limitter.
    pub limitter: Vec<bool>,
    /// Whether to increase head room.
    pub lookahead: Vec<bool>,
    /// Whether to enable soft clipping.
    pub soft_clip: Vec<bool>,
}

impl AsRef<CommandDspEqualizerState> for CommandDspInputState {
    fn as_ref(&self) -> &CommandDspEqualizerState {
        &self.equalizer
    }
}

impl AsMut<CommandDspEqualizerState> for CommandDspInputState {
    fn as_mut(&mut self) -> &mut CommandDspEqualizerState {
        &mut self.equalizer
    }
}

impl AsRef<CommandDspDynamicsState> for CommandDspInputState {
    fn as_ref(&self) -> &CommandDspDynamicsState {
        &self.dynamics
    }
}

impl AsMut<CommandDspDynamicsState> for CommandDspInputState {
    fn as_mut(&mut self) -> &mut CommandDspDynamicsState {
        &mut self.dynamics
    }
}

/// The trait for specification of inputs.
pub trait MotuCommandDspInputSpecification {
    /// The input ports.
    const INPUT_PORTS: &'static [TargetPort];
    /// The number of microphone inputs.
    const MIC_COUNT: usize;
    /// The number of line inputs.
    const LINE_INPUT_COUNT: usize;

    /// The minimum value of gain.
    const INPUT_GAIN_MIN: i32 = -96;
    /// The maximum value of gain.
    const INPUT_GAIN_MAX: i32 = 22;
    /// The step value of gain.
    const INPUT_GAIN_STEP: i32 = 1;

    /// The minimum value of width.
    const INPUT_WIDTH_MIN: f32 = 0.0;
    /// The maximum value of width.
    const INPUT_WIDTH_MAX: f32 = 1.0;

    /// The minimum value of gain to send to reverb.
    const INPUT_REVERB_GAIN_MIN: f32 = 0.0;
    /// The maximum value of gain to send to reverb.
    const INPUT_REVERB_GAIN_MAX: f32 = 1.0;

    /// The minimum value of left and right balance to send to reverb.
    const INPUT_REVERB_BALANCE_MIN: f32 = -1.0;
    /// The maximum value of left and right balance to send to reverb.
    const INPUT_REVERB_BALANCE_MAX: f32 = 1.0;

    /// Instantiate input parameters.
    fn create_input_state() -> CommandDspInputState {
        CommandDspInputState {
            phase: vec![Default::default(); Self::INPUT_PORTS.len()],
            pair: vec![Default::default(); Self::INPUT_PORTS.len()],
            gain: vec![Default::default(); Self::INPUT_PORTS.len()],
            swap: vec![Default::default(); Self::INPUT_PORTS.len()],
            stereo_mode: vec![Default::default(); Self::INPUT_PORTS.len()],
            width: vec![Default::default(); Self::INPUT_PORTS.len()],
            reverb_send: vec![Default::default(); Self::INPUT_PORTS.len()],
            reverb_balance: vec![Default::default(); Self::INPUT_PORTS.len()],
            equalizer: CommandDspEqualizerState {
                enable: vec![Default::default(); Self::INPUT_PORTS.len()],

                hpf_enable: vec![Default::default(); Self::INPUT_PORTS.len()],
                hpf_slope: vec![Default::default(); Self::INPUT_PORTS.len()],
                hpf_freq: vec![Default::default(); Self::INPUT_PORTS.len()],

                lpf_enable: vec![Default::default(); Self::INPUT_PORTS.len()],
                lpf_slope: vec![Default::default(); Self::INPUT_PORTS.len()],
                lpf_freq: vec![Default::default(); Self::INPUT_PORTS.len()],

                lf_enable: vec![Default::default(); Self::INPUT_PORTS.len()],
                lf_type: vec![Default::default(); Self::INPUT_PORTS.len()],
                lf_freq: vec![Default::default(); Self::INPUT_PORTS.len()],
                lf_gain: vec![Default::default(); Self::INPUT_PORTS.len()],
                lf_width: vec![Default::default(); Self::INPUT_PORTS.len()],

                lmf_enable: vec![Default::default(); Self::INPUT_PORTS.len()],
                lmf_type: vec![Default::default(); Self::INPUT_PORTS.len()],
                lmf_freq: vec![Default::default(); Self::INPUT_PORTS.len()],
                lmf_gain: vec![Default::default(); Self::INPUT_PORTS.len()],
                lmf_width: vec![Default::default(); Self::INPUT_PORTS.len()],

                mf_enable: vec![Default::default(); Self::INPUT_PORTS.len()],
                mf_type: vec![Default::default(); Self::INPUT_PORTS.len()],
                mf_freq: vec![Default::default(); Self::INPUT_PORTS.len()],
                mf_gain: vec![Default::default(); Self::INPUT_PORTS.len()],
                mf_width: vec![Default::default(); Self::INPUT_PORTS.len()],

                hmf_enable: vec![Default::default(); Self::INPUT_PORTS.len()],
                hmf_type: vec![Default::default(); Self::INPUT_PORTS.len()],
                hmf_freq: vec![Default::default(); Self::INPUT_PORTS.len()],
                hmf_gain: vec![Default::default(); Self::INPUT_PORTS.len()],
                hmf_width: vec![Default::default(); Self::INPUT_PORTS.len()],

                hf_enable: vec![Default::default(); Self::INPUT_PORTS.len()],
                hf_type: vec![Default::default(); Self::INPUT_PORTS.len()],
                hf_freq: vec![Default::default(); Self::INPUT_PORTS.len()],
                hf_gain: vec![Default::default(); Self::INPUT_PORTS.len()],
                hf_width: vec![Default::default(); Self::INPUT_PORTS.len()],
            },
            dynamics: CommandDspDynamicsState {
                enable: vec![Default::default(); Self::INPUT_PORTS.len()],

                comp_enable: vec![Default::default(); Self::INPUT_PORTS.len()],
                comp_detect_mode: vec![Default::default(); Self::INPUT_PORTS.len()],
                comp_threshold: vec![Default::default(); Self::INPUT_PORTS.len()],
                comp_ratio: vec![Default::default(); Self::INPUT_PORTS.len()],
                comp_attack: vec![Default::default(); Self::INPUT_PORTS.len()],
                comp_release: vec![Default::default(); Self::INPUT_PORTS.len()],
                comp_gain: vec![Default::default(); Self::INPUT_PORTS.len()],

                leveler_enable: vec![Default::default(); Self::INPUT_PORTS.len()],
                leveler_mode: vec![Default::default(); Self::INPUT_PORTS.len()],
                leveler_makeup: vec![Default::default(); Self::INPUT_PORTS.len()],
                leveler_reduce: vec![Default::default(); Self::INPUT_PORTS.len()],
            },
            pad: vec![Default::default(); Self::MIC_COUNT],
            phantom: vec![Default::default(); Self::MIC_COUNT],
            limitter: vec![Default::default(); Self::MIC_COUNT],
            lookahead: vec![Default::default(); Self::MIC_COUNT],
            soft_clip: vec![Default::default(); Self::MIC_COUNT],
            nominal_level: vec![Default::default(); Self::LINE_INPUT_COUNT],
        }
    }
}

impl<O> MotuCommandDspParametersOperation<CommandDspInputState> for O
where
    O: MotuCommandDspInputSpecification
        + MotuCommandDspEqualizerSpecification
        + MotuCommandDspDynamicsSpecification,
{
    fn build_commands(params: &CommandDspInputState) -> Vec<DspCmd> {
        let mut cmds = Vec::new();

        (0..Self::INPUT_PORTS.len()).for_each(|ch| {
            cmds.push(DspCmd::Input(InputCmd::Phase(ch, params.phase[ch])));
            cmds.push(DspCmd::Input(InputCmd::Pair(ch, params.pair[ch])));
            cmds.push(DspCmd::Input(InputCmd::Gain(ch, params.gain[ch])));
            cmds.push(DspCmd::Input(InputCmd::Swap(ch, params.swap[ch])));
            cmds.push(DspCmd::Input(InputCmd::StereoMode(
                ch,
                params.stereo_mode[ch],
            )));
            cmds.push(DspCmd::Input(InputCmd::Width(ch, params.width[ch])));

            O::create_equalizer_parameters(&params.equalizer, ch)
                .into_iter()
                .for_each(|param| cmds.push(DspCmd::Input(InputCmd::Equalizer(ch, param))));

            O::create_dynamics_parameters(&params.dynamics, ch)
                .into_iter()
                .for_each(|param| cmds.push(DspCmd::Input(InputCmd::Dynamics(ch, param))));

            cmds.push(DspCmd::Input(InputCmd::ReverbSend(
                ch,
                params.reverb_send[ch],
            )));
            cmds.push(DspCmd::Input(InputCmd::ReverbLrBalance(
                ch,
                params.reverb_balance[ch],
            )));
        });

        (0..Self::MIC_COUNT).for_each(|ch| {
            cmds.push(DspCmd::Input(InputCmd::Pad(ch, params.pad[ch])));
            cmds.push(DspCmd::Input(InputCmd::Phantom(ch, params.phantom[ch])));
            cmds.push(DspCmd::Input(InputCmd::Limitter(ch, params.limitter[ch])));
            cmds.push(DspCmd::Input(InputCmd::Lookahead(ch, params.lookahead[ch])));
            cmds.push(DspCmd::Input(InputCmd::Softclip(ch, params.soft_clip[ch])));
        });

        (0..Self::LINE_INPUT_COUNT).for_each(|ch| {
            cmds.push(DspCmd::Input(InputCmd::NominalLevel(
                Self::MIC_COUNT + ch,
                params.nominal_level[ch],
            )));
        });

        cmds
    }

    fn parse_command(params: &mut CommandDspInputState, command: &DspCmd) -> bool {
        if let DspCmd::Input(cmd) = command {
            match cmd {
                InputCmd::Phase(ch, val) => params.phase[*ch] = *val,
                InputCmd::Pair(ch, val) => params.pair[*ch] = *val,
                InputCmd::Gain(ch, val) => params.gain[*ch] = *val,
                InputCmd::Swap(ch, val) => params.swap[*ch] = *val,
                InputCmd::StereoMode(ch, val) => params.stereo_mode[*ch] = *val,
                InputCmd::Width(ch, val) => params.width[*ch] = *val,
                InputCmd::Equalizer(ch, param) => {
                    O::parse_equalizer_parameter(&mut params.equalizer, param, *ch)
                }
                InputCmd::Dynamics(ch, param) => {
                    O::parse_dynamics_parameter(&mut params.dynamics, param, *ch)
                }
                InputCmd::ReverbSend(ch, val) => params.reverb_send[*ch] = *val,
                InputCmd::ReverbLrBalance(ch, val) => params.reverb_balance[*ch] = *val,
                InputCmd::NominalLevel(ch, val) => {
                    if *ch >= params.pad.len()
                        && *ch < params.pad.len() + params.nominal_level.len()
                    {
                        params.nominal_level[*ch - params.pad.len()] = *val;
                    }
                }
                _ => (),
            };
            true
        } else {
            false
        }
    }
}

/// State of input function.
#[derive(Default, Debug, Clone, PartialEq)]
pub struct CommandDspOutputState {
    /// The parameters of equalizers for each output.
    pub equalizer: CommandDspEqualizerState,
    /// The parameters of dynamics for each output.
    pub dynamics: CommandDspDynamicsState,

    /// The gain to send to reverb effect.
    pub reverb_send: Vec<f32>,
    /// The volume to return from reverb effect.
    pub reverb_return: Vec<f32>,

    /// Whether to monitor output in master.
    pub master_monitor: Vec<bool>,
    /// Whether to take talkback in master.
    pub master_talkback: Vec<bool>,
    /// Whether to take listenback in master.
    pub master_listenback: Vec<bool>,
}

impl AsRef<CommandDspEqualizerState> for CommandDspOutputState {
    fn as_ref(&self) -> &CommandDspEqualizerState {
        &self.equalizer
    }
}

impl AsMut<CommandDspEqualizerState> for CommandDspOutputState {
    fn as_mut(&mut self) -> &mut CommandDspEqualizerState {
        &mut self.equalizer
    }
}

impl AsRef<CommandDspDynamicsState> for CommandDspOutputState {
    fn as_ref(&self) -> &CommandDspDynamicsState {
        &self.dynamics
    }
}

impl AsMut<CommandDspDynamicsState> for CommandDspOutputState {
    fn as_mut(&mut self) -> &mut CommandDspDynamicsState {
        &mut self.dynamics
    }
}

/// The trait for specification of output.
pub trait MotuCommandDspOutputSpecification {
    /// The destination port of outputs.
    const OUTPUT_PORTS: &'static [TargetPort];

    /// The minimum value of gain for outputs.
    const OUTPUT_GAIN_MIN: f32 = 0.0;
    /// The maximum value of gain for outputs.
    const OUTPUT_GAIN_MAX: f32 = 1.0;

    /// The minimum value of volume for outputs.
    const OUTPUT_VOLUME_MIN: f32 = 0.0;
    /// The maximum value of volume for outputs.
    const OUTPUT_VOLUME_MAX: f32 = 1.0;

    fn create_output_state() -> CommandDspOutputState {
        CommandDspOutputState {
            equalizer: CommandDspEqualizerState {
                enable: vec![Default::default(); Self::OUTPUT_PORTS.len()],

                hpf_enable: vec![Default::default(); Self::OUTPUT_PORTS.len()],
                hpf_slope: vec![Default::default(); Self::OUTPUT_PORTS.len()],
                hpf_freq: vec![Default::default(); Self::OUTPUT_PORTS.len()],

                lpf_enable: vec![Default::default(); Self::OUTPUT_PORTS.len()],
                lpf_slope: vec![Default::default(); Self::OUTPUT_PORTS.len()],
                lpf_freq: vec![Default::default(); Self::OUTPUT_PORTS.len()],

                lf_enable: vec![Default::default(); Self::OUTPUT_PORTS.len()],
                lf_type: vec![Default::default(); Self::OUTPUT_PORTS.len()],
                lf_freq: vec![Default::default(); Self::OUTPUT_PORTS.len()],
                lf_gain: vec![Default::default(); Self::OUTPUT_PORTS.len()],
                lf_width: vec![Default::default(); Self::OUTPUT_PORTS.len()],

                lmf_enable: vec![Default::default(); Self::OUTPUT_PORTS.len()],
                lmf_type: vec![Default::default(); Self::OUTPUT_PORTS.len()],
                lmf_freq: vec![Default::default(); Self::OUTPUT_PORTS.len()],
                lmf_gain: vec![Default::default(); Self::OUTPUT_PORTS.len()],
                lmf_width: vec![Default::default(); Self::OUTPUT_PORTS.len()],

                mf_enable: vec![Default::default(); Self::OUTPUT_PORTS.len()],
                mf_type: vec![Default::default(); Self::OUTPUT_PORTS.len()],
                mf_freq: vec![Default::default(); Self::OUTPUT_PORTS.len()],
                mf_gain: vec![Default::default(); Self::OUTPUT_PORTS.len()],
                mf_width: vec![Default::default(); Self::OUTPUT_PORTS.len()],

                hmf_enable: vec![Default::default(); Self::OUTPUT_PORTS.len()],
                hmf_type: vec![Default::default(); Self::OUTPUT_PORTS.len()],
                hmf_freq: vec![Default::default(); Self::OUTPUT_PORTS.len()],
                hmf_gain: vec![Default::default(); Self::OUTPUT_PORTS.len()],
                hmf_width: vec![Default::default(); Self::OUTPUT_PORTS.len()],

                hf_enable: vec![Default::default(); Self::OUTPUT_PORTS.len()],
                hf_type: vec![Default::default(); Self::OUTPUT_PORTS.len()],
                hf_freq: vec![Default::default(); Self::OUTPUT_PORTS.len()],
                hf_gain: vec![Default::default(); Self::OUTPUT_PORTS.len()],
                hf_width: vec![Default::default(); Self::OUTPUT_PORTS.len()],
            },
            dynamics: CommandDspDynamicsState {
                enable: vec![Default::default(); Self::OUTPUT_PORTS.len()],

                comp_enable: vec![Default::default(); Self::OUTPUT_PORTS.len()],
                comp_detect_mode: vec![Default::default(); Self::OUTPUT_PORTS.len()],
                comp_threshold: vec![Default::default(); Self::OUTPUT_PORTS.len()],
                comp_ratio: vec![Default::default(); Self::OUTPUT_PORTS.len()],
                comp_attack: vec![Default::default(); Self::OUTPUT_PORTS.len()],
                comp_release: vec![Default::default(); Self::OUTPUT_PORTS.len()],
                comp_gain: vec![Default::default(); Self::OUTPUT_PORTS.len()],

                leveler_enable: vec![Default::default(); Self::OUTPUT_PORTS.len()],
                leveler_mode: vec![Default::default(); Self::OUTPUT_PORTS.len()],
                leveler_makeup: vec![Default::default(); Self::OUTPUT_PORTS.len()],
                leveler_reduce: vec![Default::default(); Self::OUTPUT_PORTS.len()],
            },
            reverb_send: vec![Default::default(); Self::OUTPUT_PORTS.len()],
            reverb_return: vec![Default::default(); Self::OUTPUT_PORTS.len()],
            master_monitor: vec![Default::default(); Self::OUTPUT_PORTS.len()],
            master_talkback: vec![Default::default(); Self::OUTPUT_PORTS.len()],
            master_listenback: vec![Default::default(); Self::OUTPUT_PORTS.len()],
        }
    }
}

impl<O> MotuCommandDspParametersOperation<CommandDspOutputState> for O
where
    O: MotuCommandDspOutputSpecification
        + MotuCommandDspEqualizerSpecification
        + MotuCommandDspDynamicsSpecification,
{
    fn build_commands(params: &CommandDspOutputState) -> Vec<DspCmd> {
        let mut cmds = Vec::new();

        (0..Self::OUTPUT_PORTS.len()).for_each(|ch| {
            O::create_equalizer_parameters(&params.equalizer, ch)
                .into_iter()
                .for_each(|param| cmds.push(DspCmd::Output(OutputCmd::Equalizer(ch, param))));

            O::create_dynamics_parameters(&params.dynamics, ch)
                .into_iter()
                .for_each(|param| cmds.push(DspCmd::Output(OutputCmd::Dynamics(ch, param))));

            cmds.push(DspCmd::Output(OutputCmd::ReverbSend(
                ch,
                params.reverb_send[ch],
            )));
            cmds.push(DspCmd::Output(OutputCmd::ReverbReturn(
                ch,
                params.reverb_return[ch],
            )));

            cmds.push(DspCmd::Output(OutputCmd::MasterMonitor(
                ch,
                params.master_monitor[ch],
            )));
            cmds.push(DspCmd::Output(OutputCmd::MasterTalkback(
                ch,
                params.master_talkback[ch],
            )));
            cmds.push(DspCmd::Output(OutputCmd::MasterListenback(
                ch,
                params.master_listenback[ch],
            )));
        });

        cmds
    }

    fn parse_command(params: &mut CommandDspOutputState, command: &DspCmd) -> bool {
        if let DspCmd::Output(cmd) = command {
            match cmd {
                OutputCmd::Equalizer(ch, param) => {
                    O::parse_equalizer_parameter(&mut params.equalizer, param, *ch)
                }
                OutputCmd::Dynamics(ch, param) => {
                    O::parse_dynamics_parameter(&mut params.dynamics, param, *ch)
                }
                OutputCmd::ReverbSend(ch, val) => params.reverb_send[*ch] = *val,
                OutputCmd::ReverbReturn(ch, val) => params.reverb_return[*ch] = *val,
                OutputCmd::MasterMonitor(ch, val) => params.master_monitor[*ch] = *val,
                OutputCmd::MasterTalkback(ch, val) => params.master_talkback[*ch] = *val,
                OutputCmd::MasterListenback(ch, val) => params.master_listenback[*ch] = *val,
                _ => (),
            };
            true
        } else {
            false
        }
    }
}

fn create_output_commands(state: &CommandDspOutputState, output_count: usize) -> Vec<DspCmd> {
    let mut cmds = Vec::new();

    (0..output_count).for_each(|ch| {
        create_equalizer_parameters(&state.equalizer, ch)
            .into_iter()
            .for_each(|param| cmds.push(DspCmd::Output(OutputCmd::Equalizer(ch, param))));

        create_dynamics_parameters(&state.dynamics, ch)
            .into_iter()
            .for_each(|param| cmds.push(DspCmd::Output(OutputCmd::Dynamics(ch, param))));

        cmds.push(DspCmd::Output(OutputCmd::ReverbSend(
            ch,
            state.reverb_send[ch],
        )));
        cmds.push(DspCmd::Output(OutputCmd::ReverbReturn(
            ch,
            state.reverb_return[ch],
        )));

        cmds.push(DspCmd::Output(OutputCmd::MasterMonitor(
            ch,
            state.master_monitor[ch],
        )));
        cmds.push(DspCmd::Output(OutputCmd::MasterTalkback(
            ch,
            state.master_talkback[ch],
        )));
        cmds.push(DspCmd::Output(OutputCmd::MasterListenback(
            ch,
            state.master_listenback[ch],
        )));
    });

    cmds
}

fn parse_output_command(state: &mut CommandDspOutputState, cmd: &OutputCmd) {
    match cmd {
        OutputCmd::Equalizer(ch, param) => {
            parse_equalizer_parameter(&mut state.equalizer, param, *ch)
        }
        OutputCmd::Dynamics(ch, param) => parse_dynamics_parameter(&mut state.dynamics, param, *ch),
        OutputCmd::ReverbSend(ch, val) => state.reverb_send[*ch] = *val,
        OutputCmd::ReverbReturn(ch, val) => state.reverb_return[*ch] = *val,
        OutputCmd::MasterMonitor(ch, val) => state.master_monitor[*ch] = *val,
        OutputCmd::MasterTalkback(ch, val) => state.master_talkback[*ch] = *val,
        OutputCmd::MasterListenback(ch, val) => state.master_listenback[*ch] = *val,
        _ => (),
    }
}

/// The trait for operation of input function.
pub trait CommandDspOutputOperation: CommandDspOperation {
    const OUTPUT_PORTS: &'static [TargetPort];

    const GAIN_MIN: f32 = 0.0;
    const GAIN_MAX: f32 = 1.0;

    const VOLUME_MIN: f32 = 0.0;
    const VOLUME_MAX: f32 = 1.0;

    fn create_output_state() -> CommandDspOutputState {
        CommandDspOutputState {
            equalizer: CommandDspEqualizerState {
                enable: vec![Default::default(); Self::OUTPUT_PORTS.len()],

                hpf_enable: vec![Default::default(); Self::OUTPUT_PORTS.len()],
                hpf_slope: vec![Default::default(); Self::OUTPUT_PORTS.len()],
                hpf_freq: vec![Default::default(); Self::OUTPUT_PORTS.len()],

                lpf_enable: vec![Default::default(); Self::OUTPUT_PORTS.len()],
                lpf_slope: vec![Default::default(); Self::OUTPUT_PORTS.len()],
                lpf_freq: vec![Default::default(); Self::OUTPUT_PORTS.len()],

                lf_enable: vec![Default::default(); Self::OUTPUT_PORTS.len()],
                lf_type: vec![Default::default(); Self::OUTPUT_PORTS.len()],
                lf_freq: vec![Default::default(); Self::OUTPUT_PORTS.len()],
                lf_gain: vec![Default::default(); Self::OUTPUT_PORTS.len()],
                lf_width: vec![Default::default(); Self::OUTPUT_PORTS.len()],

                lmf_enable: vec![Default::default(); Self::OUTPUT_PORTS.len()],
                lmf_type: vec![Default::default(); Self::OUTPUT_PORTS.len()],
                lmf_freq: vec![Default::default(); Self::OUTPUT_PORTS.len()],
                lmf_gain: vec![Default::default(); Self::OUTPUT_PORTS.len()],
                lmf_width: vec![Default::default(); Self::OUTPUT_PORTS.len()],

                mf_enable: vec![Default::default(); Self::OUTPUT_PORTS.len()],
                mf_type: vec![Default::default(); Self::OUTPUT_PORTS.len()],
                mf_freq: vec![Default::default(); Self::OUTPUT_PORTS.len()],
                mf_gain: vec![Default::default(); Self::OUTPUT_PORTS.len()],
                mf_width: vec![Default::default(); Self::OUTPUT_PORTS.len()],

                hmf_enable: vec![Default::default(); Self::OUTPUT_PORTS.len()],
                hmf_type: vec![Default::default(); Self::OUTPUT_PORTS.len()],
                hmf_freq: vec![Default::default(); Self::OUTPUT_PORTS.len()],
                hmf_gain: vec![Default::default(); Self::OUTPUT_PORTS.len()],
                hmf_width: vec![Default::default(); Self::OUTPUT_PORTS.len()],

                hf_enable: vec![Default::default(); Self::OUTPUT_PORTS.len()],
                hf_type: vec![Default::default(); Self::OUTPUT_PORTS.len()],
                hf_freq: vec![Default::default(); Self::OUTPUT_PORTS.len()],
                hf_gain: vec![Default::default(); Self::OUTPUT_PORTS.len()],
                hf_width: vec![Default::default(); Self::OUTPUT_PORTS.len()],
            },
            dynamics: CommandDspDynamicsState {
                enable: vec![Default::default(); Self::OUTPUT_PORTS.len()],

                comp_enable: vec![Default::default(); Self::OUTPUT_PORTS.len()],
                comp_detect_mode: vec![Default::default(); Self::OUTPUT_PORTS.len()],
                comp_threshold: vec![Default::default(); Self::OUTPUT_PORTS.len()],
                comp_ratio: vec![Default::default(); Self::OUTPUT_PORTS.len()],
                comp_attack: vec![Default::default(); Self::OUTPUT_PORTS.len()],
                comp_release: vec![Default::default(); Self::OUTPUT_PORTS.len()],
                comp_gain: vec![Default::default(); Self::OUTPUT_PORTS.len()],

                leveler_enable: vec![Default::default(); Self::OUTPUT_PORTS.len()],
                leveler_mode: vec![Default::default(); Self::OUTPUT_PORTS.len()],
                leveler_makeup: vec![Default::default(); Self::OUTPUT_PORTS.len()],
                leveler_reduce: vec![Default::default(); Self::OUTPUT_PORTS.len()],
            },
            reverb_send: vec![Default::default(); Self::OUTPUT_PORTS.len()],
            reverb_return: vec![Default::default(); Self::OUTPUT_PORTS.len()],
            master_monitor: vec![Default::default(); Self::OUTPUT_PORTS.len()],
            master_talkback: vec![Default::default(); Self::OUTPUT_PORTS.len()],
            master_listenback: vec![Default::default(); Self::OUTPUT_PORTS.len()],
        }
    }

    fn parse_output_commands(state: &mut CommandDspOutputState, cmds: &[DspCmd]) {
        cmds.iter().for_each(|cmd| {
            if let DspCmd::Output(c) = cmd {
                parse_output_command(state, c);
            }
        });
    }

    fn write_output_state(
        req: &mut FwReq,
        node: &mut FwNode,
        sequence_number: &mut u8,
        state: CommandDspOutputState,
        old: &mut CommandDspOutputState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut new_cmds = create_output_commands(&state, Self::OUTPUT_PORTS.len());
        let old_cmds = create_output_commands(old, Self::OUTPUT_PORTS.len());
        new_cmds.retain(|cmd| old_cmds.iter().find(|c| c.eq(&cmd)).is_none());
        Self::send_commands(req, node, sequence_number, &new_cmds, timeout_ms).map(|_| *old = state)
    }
}

/// Information of Meter.
#[derive(Default, Debug)]
pub struct CommandDspMeterState {
    pub inputs: Vec<f32>,
    pub outputs: Vec<f32>,
    // TODO: other fields.
}

/// The trait for meter operation.
pub trait CommandDspMeterOperation {
    const INPUT_PORTS: &'static [(TargetPort, usize)];
    const OUTPUT_PORTS: &'static [(TargetPort, usize)];

    const LEVEL_MIN: f32 = 0.0;
    const LEVEL_MAX: f32 = 1.0;

    fn create_meter_state() -> CommandDspMeterState {
        CommandDspMeterState {
            inputs: vec![0.0; Self::INPUT_PORTS.len()],
            outputs: vec![0.0; Self::OUTPUT_PORTS.len()],
        }
    }

    fn parse_dsp_meter(state: &mut CommandDspMeterState, meter: &[f32]) {
        state
            .inputs
            .iter_mut()
            .zip(Self::INPUT_PORTS)
            .for_each(|(m, &(_, pos))| *m = meter[pos]);
        state
            .outputs
            .iter_mut()
            .zip(Self::OUTPUT_PORTS)
            .for_each(|(m, &(_, pos))| *m = meter[pos]);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_u8_cmds() {
        [
            DspCmd::Monitor(MonitorCmd::ReturnAssign(0x69)),
            DspCmd::Monitor(MonitorCmd::TalkbackEnable(true)),
            DspCmd::Monitor(MonitorCmd::ListenbackEnable(true)),
            DspCmd::Input(InputCmd::Phase(0x59, true)),
            DspCmd::Input(InputCmd::Pair(0x0, false)),
            DspCmd::Input(InputCmd::Swap(0x24, false)),
            DspCmd::Input(InputCmd::StereoMode(
                0x35,
                InputStereoPairMode::MonauralStereo,
            )),
            DspCmd::Input(InputCmd::Limitter(0xad, true)),
            DspCmd::Input(InputCmd::Lookahead(0xdd, true)),
            DspCmd::Input(InputCmd::Softclip(0xfc, false)),
            DspCmd::Input(InputCmd::Pad(0x91, true)),
            DspCmd::Input(InputCmd::NominalLevel(
                0x91,
                NominalSignalLevel::Professional,
            )),
            DspCmd::Input(InputCmd::Phantom(0x13, false)),
            DspCmd::Input(InputCmd::Equalizer(0x14, EqualizerParameter::Enable(false))),
            DspCmd::Input(InputCmd::Equalizer(
                0x23,
                EqualizerParameter::HpfEnable(true),
            )),
            DspCmd::Input(InputCmd::Equalizer(
                0x32,
                EqualizerParameter::HpfSlope(RollOffLevel::L30),
            )),
            DspCmd::Input(InputCmd::Equalizer(
                0x41,
                EqualizerParameter::LfEnable(false),
            )),
            DspCmd::Input(InputCmd::Equalizer(
                0x59,
                EqualizerParameter::LfType(FilterType5::Shelf),
            )),
            DspCmd::Input(InputCmd::Equalizer(
                0x68,
                EqualizerParameter::LmfEnable(true),
            )),
            DspCmd::Input(InputCmd::Equalizer(
                0x77,
                EqualizerParameter::LmfType(FilterType4::T4),
            )),
            DspCmd::Input(InputCmd::Equalizer(
                0x86,
                EqualizerParameter::MfEnable(false),
            )),
            DspCmd::Input(InputCmd::Equalizer(
                0x95,
                EqualizerParameter::MfType(FilterType4::T3),
            )),
            DspCmd::Input(InputCmd::Equalizer(
                0xaf,
                EqualizerParameter::HmfEnable(true),
            )),
            DspCmd::Input(InputCmd::Equalizer(
                0xbe,
                EqualizerParameter::HmfType(FilterType4::T2),
            )),
            DspCmd::Input(InputCmd::Equalizer(
                0xcd,
                EqualizerParameter::HfEnable(false),
            )),
            DspCmd::Input(InputCmd::Equalizer(
                0xdc,
                EqualizerParameter::HfType(FilterType5::T1),
            )),
            DspCmd::Input(InputCmd::Equalizer(
                0xeb,
                EqualizerParameter::LpfEnable(true),
            )),
            DspCmd::Input(InputCmd::Equalizer(
                0xfa,
                EqualizerParameter::LpfSlope(RollOffLevel::L24),
            )),
            DspCmd::Input(InputCmd::Dynamics(0xf0, DynamicsParameter::Enable(false))),
            DspCmd::Input(InputCmd::Dynamics(
                0xe1,
                DynamicsParameter::CompEnable(true),
            )),
            DspCmd::Input(InputCmd::Dynamics(
                0xd2,
                DynamicsParameter::CompDetectMode(LevelDetectMode::Rms),
            )),
            DspCmd::Input(InputCmd::Dynamics(
                0xc3,
                DynamicsParameter::LevelerEnable(false),
            )),
            DspCmd::Input(InputCmd::Dynamics(
                0xb4,
                DynamicsParameter::LevelerMode(LevelerMode::Limit),
            )),
            DspCmd::Mixer(MixerCmd::OutputAssign(0xa5, 0x91)),
            DspCmd::Mixer(MixerCmd::OutputMute(0x96, true)),
            DspCmd::Mixer(MixerCmd::SourceMute(0x87, 0x13, false)),
            DspCmd::Mixer(MixerCmd::SourceSolo(0x78, 0x31, true)),
            DspCmd::Mixer(MixerCmd::SourceStereoMode(
                0x69,
                0x11,
                SourceStereoPairMode::LrBalance,
            )),
            DspCmd::Output(OutputCmd::Equalizer(
                0x5a,
                EqualizerParameter::Enable(false),
            )),
            DspCmd::Output(OutputCmd::Equalizer(
                0x4b,
                EqualizerParameter::HpfEnable(true),
            )),
            DspCmd::Output(OutputCmd::Equalizer(
                0x3c,
                EqualizerParameter::HpfSlope(RollOffLevel::L6),
            )),
            DspCmd::Output(OutputCmd::Equalizer(
                0x2d,
                EqualizerParameter::LfEnable(false),
            )),
            DspCmd::Output(OutputCmd::Equalizer(
                0x1e,
                EqualizerParameter::LfType(FilterType5::Shelf),
            )),
            DspCmd::Output(OutputCmd::Equalizer(
                0x0f,
                EqualizerParameter::LmfEnable(true),
            )),
            DspCmd::Output(OutputCmd::Equalizer(
                0xf1,
                EqualizerParameter::LmfType(FilterType4::T4),
            )),
            DspCmd::Output(OutputCmd::Equalizer(
                0xe2,
                EqualizerParameter::MfEnable(false),
            )),
            DspCmd::Output(OutputCmd::Equalizer(
                0xd3,
                EqualizerParameter::MfType(FilterType4::T3),
            )),
            DspCmd::Output(OutputCmd::Equalizer(
                0xc4,
                EqualizerParameter::HmfEnable(true),
            )),
            DspCmd::Output(OutputCmd::Equalizer(
                0xb5,
                EqualizerParameter::HmfType(FilterType4::T2),
            )),
            DspCmd::Output(OutputCmd::Equalizer(
                0xa6,
                EqualizerParameter::HfEnable(false),
            )),
            DspCmd::Output(OutputCmd::Equalizer(
                0x97,
                EqualizerParameter::HfType(FilterType5::T1),
            )),
            DspCmd::Output(OutputCmd::Equalizer(
                0x88,
                EqualizerParameter::LpfEnable(true),
            )),
            DspCmd::Output(OutputCmd::Equalizer(
                0x79,
                EqualizerParameter::LpfSlope(RollOffLevel::L18),
            )),
            DspCmd::Output(OutputCmd::Dynamics(0xff, DynamicsParameter::Enable(false))),
            DspCmd::Output(OutputCmd::Dynamics(
                0xee,
                DynamicsParameter::CompEnable(true),
            )),
            DspCmd::Output(OutputCmd::Dynamics(
                0xdd,
                DynamicsParameter::CompDetectMode(LevelDetectMode::Peak),
            )),
            DspCmd::Output(OutputCmd::Dynamics(
                0xcc,
                DynamicsParameter::LevelerEnable(false),
            )),
            DspCmd::Output(OutputCmd::Dynamics(
                0xbb,
                DynamicsParameter::LevelerMode(LevelerMode::Compress),
            )),
            DspCmd::Output(OutputCmd::MasterMonitor(0x97, true)),
            DspCmd::Output(OutputCmd::MasterTalkback(0xec, false)),
            DspCmd::Output(OutputCmd::MasterListenback(0xd5, true)),
            DspCmd::Reverb(ReverbCmd::Enable(false)),
            DspCmd::Reverb(ReverbCmd::Split(SplitPoint::Mixer)),
            DspCmd::Reverb(ReverbCmd::ReflectionMode(RoomShape::D)),
            DspCmd::Reserved(vec![0x69, 0xed, 0xba, 0x98, 0xec, 0x75]),
        ]
        .iter()
        .for_each(|cmd| {
            let mut raw = Vec::new();
            cmd.build(&mut raw);
            let mut c = Vec::new();
            assert_eq!(DspCmd::parse(&raw, &mut c), CMD_BYTE_SINGLE_LENGTH);
            assert_eq!(&c[0], cmd);
        });
    }

    #[test]
    fn test_i32_cmds() {
        [
            DspCmd::Monitor(MonitorCmd::Focus(FocusTarget::Output(11))),
            DspCmd::Input(InputCmd::Gain(0xe4, 0x01)),
            DspCmd::Input(InputCmd::Dynamics(
                0xb1,
                DynamicsParameter::CompThreshold(97531),
            )),
            DspCmd::Output(OutputCmd::Dynamics(
                0x45,
                DynamicsParameter::CompThreshold(86420),
            )),
            DspCmd::Reverb(ReverbCmd::ShelfFilterAttenuation(98765)),
        ]
        .iter()
        .for_each(|cmd| {
            let mut raw = Vec::new();
            cmd.build(&mut raw);
            let mut c = Vec::new();
            assert_eq!(DspCmd::parse(&raw, &mut c), CMD_QUADLET_SINGLE_LENGTH);
            assert_eq!(&c[0], cmd);
        });
    }

    #[test]
    fn test_u32_cmds() {
        [
            DspCmd::Input(InputCmd::Equalizer(0xc2, EqualizerParameter::HpfFreq(10))),
            DspCmd::Input(InputCmd::Equalizer(0xb1, EqualizerParameter::LfFreq(20))),
            DspCmd::Input(InputCmd::Equalizer(0x8e, EqualizerParameter::LmfFreq(30))),
            DspCmd::Input(InputCmd::Equalizer(0x5b, EqualizerParameter::MfFreq(40))),
            DspCmd::Input(InputCmd::Equalizer(0x28, EqualizerParameter::HmfFreq(50))),
            DspCmd::Input(InputCmd::Equalizer(0xf5, EqualizerParameter::HfFreq(60))),
            DspCmd::Input(InputCmd::Equalizer(0xc2, EqualizerParameter::LpfFreq(70))),
            DspCmd::Input(InputCmd::Dynamics(0x9f, DynamicsParameter::CompAttack(100))),
            DspCmd::Input(InputCmd::Dynamics(
                0x8e,
                DynamicsParameter::CompRelease(200),
            )),
            DspCmd::Output(OutputCmd::Dynamics(
                0x7f,
                DynamicsParameter::LevelerMakeup(1000),
            )),
            DspCmd::Output(OutputCmd::Dynamics(
                0xf2,
                DynamicsParameter::LevelerReduce(2000),
            )),
            DspCmd::Output(OutputCmd::Equalizer(0xa8, EqualizerParameter::HpfFreq(103))),
            DspCmd::Output(OutputCmd::Equalizer(0x39, EqualizerParameter::LfFreq(105))),
            DspCmd::Output(OutputCmd::Equalizer(0x5b, EqualizerParameter::LmfFreq(107))),
            DspCmd::Output(OutputCmd::Equalizer(0xbc, EqualizerParameter::MfFreq(109))),
            DspCmd::Output(OutputCmd::Equalizer(0xf7, EqualizerParameter::HmfFreq(111))),
            DspCmd::Output(OutputCmd::Equalizer(0xc0, EqualizerParameter::HfFreq(113))),
            DspCmd::Output(OutputCmd::Equalizer(0x29, EqualizerParameter::LpfFreq(115))),
            DspCmd::Output(OutputCmd::Dynamics(
                0x1b,
                DynamicsParameter::CompAttack(111),
            )),
            DspCmd::Output(OutputCmd::Dynamics(
                0x49,
                DynamicsParameter::CompRelease(113),
            )),
            DspCmd::Input(InputCmd::Dynamics(
                0x6c,
                DynamicsParameter::LevelerMakeup(1111),
            )),
            DspCmd::Input(InputCmd::Dynamics(
                0x5b,
                DynamicsParameter::LevelerReduce(1113),
            )),
            DspCmd::Reverb(ReverbCmd::PreDelay(11111)),
            DspCmd::Reverb(ReverbCmd::ShelfFilterFreq(111113)),
            DspCmd::Reverb(ReverbCmd::DecayTime(111115)),
            DspCmd::Reverb(ReverbCmd::LowFreqTime(111117)),
            DspCmd::Reverb(ReverbCmd::MiddleFreqTime(111119)),
            DspCmd::Reverb(ReverbCmd::HighFreqTime(111121)),
            DspCmd::Reverb(ReverbCmd::LowFreqCrossover(111123)),
            DspCmd::Reverb(ReverbCmd::HighFreqCrossover(111125)),
            DspCmd::Reverb(ReverbCmd::ReflectionSize(111127)),
            DspCmd::Reserved(vec![0x66, 0x00, 0x01, 0x02, 0x80, 0x04, 0x05, 0x06, 0x07]),
        ]
        .iter()
        .for_each(|cmd| {
            let mut raw = Vec::new();
            cmd.build(&mut raw);
            let mut c = Vec::new();
            assert_eq!(DspCmd::parse(&raw, &mut c), CMD_QUADLET_SINGLE_LENGTH);
            assert_eq!(&c[0], cmd);
        });
    }

    #[test]
    fn test_f32_cmds() {
        [
            DspCmd::Monitor(MonitorCmd::Volume(9.012345678)),
            DspCmd::Monitor(MonitorCmd::ListenbackVolume(9.234567891)),
            DspCmd::Monitor(MonitorCmd::TalkbackVolume(9.345678912)),
            DspCmd::Input(InputCmd::Width(0xd3, 0.0987654321)),
            DspCmd::Input(InputCmd::Equalizer(
                0xa0,
                EqualizerParameter::LfGain(0.123456789),
            )),
            DspCmd::Input(InputCmd::Equalizer(
                0x9f,
                EqualizerParameter::LfWidth(0.987654321),
            )),
            DspCmd::Input(InputCmd::Equalizer(
                0x7d,
                EqualizerParameter::LmfGain(0.234567891),
            )),
            DspCmd::Input(InputCmd::Equalizer(
                0x6c,
                EqualizerParameter::LmfWidth(0.876543219),
            )),
            DspCmd::Input(InputCmd::Equalizer(
                0x4a,
                EqualizerParameter::MfGain(0.345678912),
            )),
            DspCmd::Input(InputCmd::Equalizer(
                0x39,
                EqualizerParameter::MfWidth(0.765432198),
            )),
            DspCmd::Input(InputCmd::Equalizer(
                0x17,
                EqualizerParameter::HmfGain(0.456789123),
            )),
            DspCmd::Input(InputCmd::Equalizer(
                0x06,
                EqualizerParameter::HmfWidth(0.654321987),
            )),
            DspCmd::Input(InputCmd::Equalizer(
                0xe4,
                EqualizerParameter::HfGain(0.567891234),
            )),
            DspCmd::Input(InputCmd::Equalizer(
                0xd3,
                EqualizerParameter::HfWidth(0.543219876),
            )),
            DspCmd::Input(InputCmd::Dynamics(
                0xa0,
                DynamicsParameter::CompRatio(0.678912345),
            )),
            DspCmd::Input(InputCmd::Dynamics(
                0x7d,
                DynamicsParameter::CompGain(0.432198765),
            )),
            DspCmd::Input(InputCmd::ReverbSend(0x33, 0.789123456)),
            DspCmd::Input(InputCmd::ReverbLrBalance(0xcc, 0.891234567)),
            DspCmd::Mixer(MixerCmd::OutputVolume(0x4a, 1.2345678)),
            DspCmd::Mixer(MixerCmd::ReverbSend(0x39, 1.3456789)),
            DspCmd::Mixer(MixerCmd::ReverbReturn(0x28, 1.456789)),
            DspCmd::Mixer(MixerCmd::SourceMonauralLrBalance(0x17, 0xc8, 1.987654)),
            DspCmd::Mixer(MixerCmd::SourceGain(0x06, 0x11, 1.876543)),
            DspCmd::Mixer(MixerCmd::SourceStereoLrBalance(0xe5, 0x13, 1.7654321)),
            DspCmd::Mixer(MixerCmd::SourceStereoWidth(0xd4, 0x1a, 1.654321)),
            DspCmd::Output(OutputCmd::Equalizer(
                0x11,
                EqualizerParameter::LfGain(2.123456789),
            )),
            DspCmd::Output(OutputCmd::Equalizer(
                0x5a,
                EqualizerParameter::LfWidth(2.987654321),
            )),
            DspCmd::Output(OutputCmd::Equalizer(
                0x98,
                EqualizerParameter::LmfGain(2.234567891),
            )),
            DspCmd::Output(OutputCmd::Equalizer(
                0x74,
                EqualizerParameter::LmfWidth(2.876543219),
            )),
            DspCmd::Output(OutputCmd::Equalizer(
                0x32,
                EqualizerParameter::MfGain(2.345678912),
            )),
            DspCmd::Output(OutputCmd::Equalizer(
                0x20,
                EqualizerParameter::MfWidth(2.765432198),
            )),
            DspCmd::Output(OutputCmd::Equalizer(
                0xc0,
                EqualizerParameter::HmfGain(2.456789123),
            )),
            DspCmd::Output(OutputCmd::Equalizer(
                0xf5,
                EqualizerParameter::HmfWidth(2.654321987),
            )),
            DspCmd::Output(OutputCmd::Equalizer(
                0x01,
                EqualizerParameter::HfGain(2.567891234),
            )),
            DspCmd::Output(OutputCmd::Equalizer(
                0xdb,
                EqualizerParameter::HfWidth(2.543219876),
            )),
            DspCmd::Output(OutputCmd::Dynamics(
                0x5e,
                DynamicsParameter::CompRatio(2.678912345),
            )),
            DspCmd::Output(OutputCmd::Dynamics(
                0xba,
                DynamicsParameter::CompGain(2.432198765),
            )),
            DspCmd::Output(OutputCmd::ReverbSend(0x99, 2.78912345)),
            DspCmd::Output(OutputCmd::ReverbReturn(0x88, 2.321987654)),
            DspCmd::Reverb(ReverbCmd::Width(123.456)),
            DspCmd::Reverb(ReverbCmd::ReflectionLevel(234.561)),
        ]
        .iter()
        .for_each(|cmd| {
            let mut raw = Vec::new();
            cmd.build(&mut raw);
            let mut c = Vec::new();
            assert_eq!(DspCmd::parse(&raw, &mut c), CMD_QUADLET_SINGLE_LENGTH);
            assert_eq!(&c[0], cmd);
        });
    }

    #[test]
    fn test_resource() {
        let cmd = DspCmd::Resource(ResourceCmd::Usage(99.99999, 0x17));
        let mut raw = Vec::new();
        cmd.build(&mut raw);
        let mut c = Vec::new();
        assert_eq!(DspCmd::parse(&raw, &mut c), CMD_RESOURCE_LENGTH);
        assert_eq!(c[0], cmd);
    }

    #[test]
    fn message_decode_test() {
        let raw = [
            0x66, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x3f, 0x69, 0x00, 0x00, 0x0a, 0x00,
            0x00, 0x69, 0x00, 0x00, 0x0b, 0x00, 0x00, 0x66, 0x00, 0x07, 0x00, 0xff, 0x00, 0x00,
            0x00, 0x01, 0x62, 0x46, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x3f, 0x49,
            0x07, 0x00, 0x02, 0x0c, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46, 0x02,
            0x00, 0x05, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x65, 0x46,
            0x00, 0xa0, 0x8c, 0x46, 0x00, 0xa0, 0x8c,
        ];
        let mut handler = CommandDspMessageHandler::default();
        handler.cache.extend_from_slice(&raw);
        let cmds = handler.decode_messages();
        assert_eq!(cmds[0], DspCmd::Monitor(MonitorCmd::Volume(1.0)));
        assert_eq!(
            cmds[1],
            DspCmd::Monitor(MonitorCmd::Reserved(
                vec![0x00, 0x0a, 0x00, 0x00],
                vec![0x00]
            ))
        );
        assert_eq!(
            cmds[2],
            DspCmd::Monitor(MonitorCmd::Reserved(
                vec![0x00, 0x0b, 0x00, 0x00],
                vec![0x00]
            ))
        );
        assert_eq!(
            cmds[3],
            DspCmd::Reserved(vec![0x66, 0x00, 0x07, 0x00, 0xff, 0x00, 0x00, 0x00, 0x01])
        );
        assert_eq!(cmds[4], DspCmd::Monitor(MonitorCmd::Volume(1.0)));
        assert_eq!(
            cmds[5],
            DspCmd::Output(OutputCmd::MasterListenback(0, false))
        );
        assert_eq!(
            cmds[6],
            DspCmd::Output(OutputCmd::MasterListenback(1, false))
        );
        assert_eq!(
            cmds[7],
            DspCmd::Output(OutputCmd::MasterListenback(2, false))
        );
        assert_eq!(
            cmds[8],
            DspCmd::Output(OutputCmd::MasterListenback(3, false))
        );
        assert_eq!(
            cmds[9],
            DspCmd::Output(OutputCmd::MasterListenback(4, false))
        );
        assert_eq!(
            cmds[10],
            DspCmd::Output(OutputCmd::MasterListenback(5, false))
        );
        assert_eq!(
            cmds[11],
            DspCmd::Output(OutputCmd::MasterListenback(6, false))
        );
        assert_eq!(cmds[12], DspCmd::Input(InputCmd::Width(0, 0.0)));
        assert_eq!(cmds[13], DspCmd::Input(InputCmd::Width(1, 0.0)));
        assert_eq!(cmds.len(), 14);
    }
}
