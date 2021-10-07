// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol for hardware mixer function operated by command.
//!
//! The module includes structure, enumeration, and trait for hardware mixer function operated by
//! command.

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
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum EqualizerParameter {
    Enable(bool),
    HpfEnable(bool),
    HpfSlope(RollOffLevel),
    HpfFreq(i32),
    LpfEnable(bool),
    LpfSlope(RollOffLevel),
    LpfFreq(i32),
    LfEnable(bool),
    LfType(FilterType5),
    LfFreq(i32),
    LfGain(i32),
    LfWidth(i32),
    LmfEnable(bool),
    LmfType(FilterType4),
    LmfFreq(i32),
    LmfGain(i32),
    LmfWidth(i32),
    MfEnable(bool),
    MfType(FilterType4),
    MfFreq(i32),
    MfGain(i32),
    MfWidth(i32),
    HmfEnable(bool),
    HmfType(FilterType4),
    HmfFreq(i32),
    HmfGain(i32),
    HmfWidth(i32),
    HfEnable(bool),
    HfType(FilterType5),
    HfFreq(i32),
    HfGain(i32),
    HfWidth(i32),
}

/// The DSP command specific to dynamics effects.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum DynamicsParameter {
    Enable(bool),
    CompEnable(bool),
    CompDetectMode(LevelDetectMode),
    CompThreshold(i32),
    CompRatio(i32),
    CompAttach(i32),
    CompRelease(i32),
    CompTrim(i32),
    LevelerEnable(bool),
    LevelerMode(LevelerMode),
    LevelerMakeup(i32),
    LevelerReduce(i32),
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
    assert_eq!(raw.len(), 4);

    let mut quadlet = [0; 4];
    quadlet.copy_from_slice(raw);

    i32::from_le_bytes(quadlet)
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

/// The DSP command specific to master output.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum MonitorCmd {
    Volume(i32),
    ReturnAssign(usize),
    Reserved(Vec<u8>, Vec<u8>),
}

impl MonitorCmd {
    fn parse(identifier: &[u8], vals: &[u8]) -> Self {
        assert_eq!(identifier.len(), 4);
        assert!(vals.len() > 0);

        match (identifier[3], identifier[2], identifier[1]) {
            (0x00, 0x00, 0x00) => MonitorCmd::Volume(to_i32(vals)),
            // TODO: model dependent, I guess.
            // (0, 0, 1) => u8
            // (0, 0, 2) => u8
            // (0, 0, 3) => u8
            // (0, 0, 4) => u8
            // (0, 0, 5) => i32
            // (0, 0, 6) => i32
            // (0, 0, 7) => i32
            (0x00, 0x00, 0x08) => MonitorCmd::ReturnAssign(to_usize(vals)),
            _ => MonitorCmd::Reserved(identifier.to_vec(), vals.to_vec()),
        }
    }

    fn build(&self, raw: &mut Vec<u8>) {
        match self {
            MonitorCmd::ReturnAssign(target) =>         append_u8(raw, 0x00, 0x00, 0x08, 0, *target as u8),
            MonitorCmd::Volume(val) =>                  append_i32(raw, 0x00, 0x00, 0x00, 0, *val),
            MonitorCmd::Reserved(identifier, vals) =>   append_data(raw, identifier, vals),
        }
    }
}

/// The DSP command specific to input.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum InputCmd {
    Phase(usize, bool),
    Pair(usize, bool),
    Trim(usize, i32),
    Swap(usize, bool),
    StereoMode(usize, InputStereoPairMode),
    Width(usize, i32),
    Equalizer(usize, EqualizerParameter),
    Dynamics(usize, DynamicsParameter),
    ReverbSend(usize, i32),
    ReverbLrBalance(usize, i32),
    Pad(usize, bool),
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
            (0x01, 0x00, 0x00) => InputCmd::Phase(ch,  to_bool(vals)),
            (0x01, 0x00, 0x01) => InputCmd::Pair(ch, to_bool(vals)),
            (0x01, 0x00, 0x02) => InputCmd::Trim(ch, to_i32(vals)),
            (0x01, 0x00, 0x03) => InputCmd::Swap(ch, to_bool(vals)),
            (0x01, 0x00, 0x04) => InputCmd::StereoMode(ch, InputStereoPairMode::from(vals[0])),
            (0x01, 0x00, 0x05) => InputCmd::Width(ch, to_i32(vals)),
            (0x01, 0x00, 0x06) => InputCmd::Limitter(ch, to_bool(vals)),
            (0x01, 0x00, 0x07) => InputCmd::Lookahead(ch, to_bool(vals)),
            (0x01, 0x00, 0x08) => InputCmd::Softclip(ch, to_bool(vals)),
            (0x01, 0x00, 0x09) => InputCmd::Pad(ch, to_bool(vals)),
            (0x01, 0x00, 0x0b) => InputCmd::Phantom(ch, to_bool(vals)),

            (0x01, 0x01, 0x00) => InputCmd::Equalizer(ch, EqualizerParameter::Enable(to_bool(vals))),

            (0x01, 0x02, 0x00) => InputCmd::Equalizer(ch, EqualizerParameter::HpfEnable(to_bool(vals))),
            (0x01, 0x02, 0x01) => InputCmd::Equalizer(ch, EqualizerParameter::HpfSlope(RollOffLevel::from(vals[0]))),
            (0x01, 0x02, 0x02) => InputCmd::Equalizer(ch, EqualizerParameter::HpfFreq(to_i32(vals))),

            (0x01, 0x03, 0x00) => InputCmd::Equalizer(ch, EqualizerParameter::LfEnable(to_bool(vals))),
            (0x01, 0x03, 0x01) => InputCmd::Equalizer(ch, EqualizerParameter::LfType(FilterType5::from(vals[0]))),
            (0x01, 0x03, 0x02) => InputCmd::Equalizer(ch, EqualizerParameter::LfFreq(to_i32(vals))),
            (0x01, 0x03, 0x03) => InputCmd::Equalizer(ch, EqualizerParameter::LfGain(to_i32(vals))),
            (0x01, 0x03, 0x04) => InputCmd::Equalizer(ch, EqualizerParameter::LfWidth(to_i32(vals))),

            (0x01, 0x04, 0x00) => InputCmd::Equalizer(ch, EqualizerParameter::LmfEnable(to_bool(vals))),
            (0x01, 0x04, 0x01) => InputCmd::Equalizer(ch, EqualizerParameter::LmfType(FilterType4::from(vals[0]))),
            (0x01, 0x04, 0x02) => InputCmd::Equalizer(ch, EqualizerParameter::LmfFreq(to_i32(vals))),
            (0x01, 0x04, 0x03) => InputCmd::Equalizer(ch, EqualizerParameter::LmfGain(to_i32(vals))),
            (0x01, 0x04, 0x04) => InputCmd::Equalizer(ch, EqualizerParameter::LmfWidth(to_i32(vals))),

            (0x01, 0x05, 0x00) => InputCmd::Equalizer(ch, EqualizerParameter::MfEnable(to_bool(vals))),
            (0x01, 0x05, 0x01) => InputCmd::Equalizer(ch, EqualizerParameter::MfType(FilterType4::from(vals[0]))),
            (0x01, 0x05, 0x02) => InputCmd::Equalizer(ch, EqualizerParameter::MfFreq(to_i32(vals))),
            (0x01, 0x05, 0x03) => InputCmd::Equalizer(ch, EqualizerParameter::MfGain(to_i32(vals))),
            (0x01, 0x05, 0x04) => InputCmd::Equalizer(ch, EqualizerParameter::MfWidth(to_i32(vals))),

            (0x01, 0x06, 0x00) => InputCmd::Equalizer(ch, EqualizerParameter::HmfEnable(to_bool(vals))),
            (0x01, 0x06, 0x01) => InputCmd::Equalizer(ch, EqualizerParameter::HmfType(FilterType4::from(vals[0]))),
            (0x01, 0x06, 0x02) => InputCmd::Equalizer(ch, EqualizerParameter::HmfFreq(to_i32(vals))),
            (0x01, 0x06, 0x03) => InputCmd::Equalizer(ch, EqualizerParameter::HmfGain(to_i32(vals))),
            (0x01, 0x06, 0x04) => InputCmd::Equalizer(ch, EqualizerParameter::HmfWidth(to_i32(vals))),

            (0x01, 0x07, 0x00) => InputCmd::Equalizer(ch, EqualizerParameter::HfEnable(to_bool(vals))),
            (0x01, 0x07, 0x01) => InputCmd::Equalizer(ch, EqualizerParameter::HfType(FilterType5::from(vals[0]))),
            (0x01, 0x07, 0x02) => InputCmd::Equalizer(ch, EqualizerParameter::HfFreq(to_i32(vals))),
            (0x01, 0x07, 0x03) => InputCmd::Equalizer(ch, EqualizerParameter::HfGain(to_i32(vals))),
            (0x01, 0x07, 0x04) => InputCmd::Equalizer(ch, EqualizerParameter::HfWidth(to_i32(vals))),

            (0x01, 0x08, 0x00) => InputCmd::Equalizer(ch, EqualizerParameter::LpfEnable(to_bool(vals))),
            (0x01, 0x08, 0x01) => InputCmd::Equalizer(ch, EqualizerParameter::LpfSlope(RollOffLevel::from(vals[0]))),
            (0x01, 0x08, 0x02) => InputCmd::Equalizer(ch, EqualizerParameter::LpfFreq(to_i32(vals))),

            (0x01, 0x09, 0x00) => InputCmd::Dynamics(ch, DynamicsParameter::Enable(to_bool(vals))),

            (0x01, 0x0a, 0x00) => InputCmd::Dynamics(ch, DynamicsParameter::CompEnable(to_bool(vals))),
            (0x01, 0x0a, 0x01) => InputCmd::Dynamics(ch, DynamicsParameter::CompThreshold(to_i32(vals))),
            (0x01, 0x0a, 0x02) => InputCmd::Dynamics(ch, DynamicsParameter::CompRatio(to_i32(vals))),
            (0x01, 0x0a, 0x03) => InputCmd::Dynamics(ch, DynamicsParameter::CompAttach(to_i32(vals))),
            (0x01, 0x0a, 0x04) => InputCmd::Dynamics(ch, DynamicsParameter::CompRelease(to_i32(vals))),
            (0x01, 0x0a, 0x05) => InputCmd::Dynamics(ch, DynamicsParameter::CompTrim(to_i32(vals))),
            (0x01, 0x0a, 0x06) => InputCmd::Dynamics(ch, DynamicsParameter::CompDetectMode(LevelDetectMode::from(vals[0]))),

            (0x01, 0x0b, 0x00) => InputCmd::Dynamics(ch, DynamicsParameter::LevelerEnable(to_bool(vals))),
            (0x01, 0x0b, 0x01) => InputCmd::Dynamics(ch, DynamicsParameter::LevelerMode(LevelerMode::from(vals[0]))),
            (0x01, 0x0b, 0x02) => InputCmd::Dynamics(ch, DynamicsParameter::LevelerMakeup(to_i32(vals))),
            (0x01, 0x0b, 0x03) => InputCmd::Dynamics(ch, DynamicsParameter::LevelerReduce(to_i32(vals))),

            (0x01, 0x0c, 0x00) => InputCmd::ReverbSend(ch, to_i32(vals)),
            (0x01, 0x0c, 0x02) => InputCmd::ReverbLrBalance(ch, to_i32(vals)),

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
            InputCmd::Phase(ch, enabled) =>                                         append_u8(raw, 0x01, 0x00, 0x00, *ch, *enabled),
            InputCmd::Pair(ch, enabled) =>                                          append_u8(raw, 0x01, 0x00, 0x01, *ch, *enabled),
            InputCmd::Trim(ch, val) =>                                              append_i32(raw, 0x01, 0x00, 0x02, *ch, *val),
            InputCmd::Swap(ch, enabled) =>                                          append_u8(raw, 0x01, 0x00, 0x03, *ch, *enabled),
            InputCmd::StereoMode(ch, pair_mode) =>                                  append_u8(raw, 0x01, 0x00, 0x04, *ch, *pair_mode),
            InputCmd::Width(ch, val) =>                                             append_i32(raw, 0x01, 0x00, 0x05, *ch, *val),
            InputCmd::Limitter(ch, enabled) =>                                      append_u8(raw, 0x01, 0x00, 0x06, *ch, *enabled),
            InputCmd::Lookahead(ch, enabled) =>                                     append_u8(raw, 0x01, 0x00, 0x07, *ch, *enabled),
            InputCmd::Softclip(ch, enabled) =>                                      append_u8(raw, 0x01, 0x00, 0x08, *ch, *enabled),
            InputCmd::Pad(ch, enabled) =>                                           append_u8(raw, 0x01, 0x00, 0x09, *ch, *enabled),
            InputCmd::Phantom(ch, enabled) =>                                       append_u8(raw, 0x01, 0x00, 0x0b, *ch, *enabled),

            InputCmd::Equalizer(ch, EqualizerParameter::Enable(enabled)) =>         append_u8(raw, 0x01, 0x01, 0x00, *ch, *enabled),

            InputCmd::Equalizer(ch, EqualizerParameter::HpfEnable(enabled)) =>      append_u8(raw, 0x01, 0x02, 0x00, *ch, *enabled),
            InputCmd::Equalizer(ch, EqualizerParameter::HpfSlope(level)) =>         append_u8(raw, 0x01, 0x02, 0x01, *ch, *level),
            InputCmd::Equalizer(ch, EqualizerParameter::HpfFreq(val)) =>            append_i32(raw, 0x01, 0x02, 0x02, *ch, *val),

            InputCmd::Equalizer(ch, EqualizerParameter::LfEnable(enabled)) =>       append_u8(raw, 0x01, 0x03, 0x00, *ch, *enabled),
            InputCmd::Equalizer(ch, EqualizerParameter::LfType(filter_type)) =>     append_u8(raw, 0x01, 0x03, 0x01, *ch, *filter_type),
            InputCmd::Equalizer(ch, EqualizerParameter::LfFreq(val)) =>             append_i32(raw, 0x01, 0x03, 0x02, *ch, *val),
            InputCmd::Equalizer(ch, EqualizerParameter::LfGain(val)) =>             append_i32(raw, 0x01, 0x03, 0x03, *ch, *val),
            InputCmd::Equalizer(ch, EqualizerParameter::LfWidth(val)) =>            append_i32(raw, 0x01, 0x03, 0x04, *ch, *val),

            InputCmd::Equalizer(ch, EqualizerParameter::LmfEnable(enabled)) =>      append_u8(raw, 0x01, 0x04, 0x00, *ch, *enabled),
            InputCmd::Equalizer(ch, EqualizerParameter::LmfType(filter_type)) =>    append_u8(raw, 0x01, 0x04, 0x01, *ch, *filter_type),
            InputCmd::Equalizer(ch, EqualizerParameter::LmfFreq(val)) =>            append_i32(raw, 0x01, 0x04, 0x02, *ch, *val),
            InputCmd::Equalizer(ch, EqualizerParameter::LmfGain(val)) =>            append_i32(raw, 0x01, 0x04, 0x03, *ch, *val),
            InputCmd::Equalizer(ch, EqualizerParameter::LmfWidth(val)) =>           append_i32(raw, 0x01, 0x04, 0x04, *ch, *val),

            InputCmd::Equalizer(ch, EqualizerParameter::MfEnable(enabled)) =>       append_u8(raw, 0x01, 0x05, 0x00, *ch, *enabled),
            InputCmd::Equalizer(ch, EqualizerParameter::MfType(filter_type)) =>     append_u8(raw, 0x01, 0x05, 0x01, *ch, *filter_type),
            InputCmd::Equalizer(ch, EqualizerParameter::MfFreq(val)) =>             append_i32(raw, 0x01, 0x05, 0x02, *ch, *val),
            InputCmd::Equalizer(ch, EqualizerParameter::MfGain(val)) =>             append_i32(raw, 0x01, 0x05, 0x03, *ch, *val),
            InputCmd::Equalizer(ch, EqualizerParameter::MfWidth(val)) =>            append_i32(raw, 0x01, 0x05, 0x04, *ch, *val),

            InputCmd::Equalizer(ch, EqualizerParameter::HmfEnable(enabled)) =>      append_u8(raw, 0x01, 0x06, 0x00, *ch, *enabled),
            InputCmd::Equalizer(ch, EqualizerParameter::HmfType(filter_type)) =>    append_u8(raw, 0x01, 0x06, 0x01, *ch, *filter_type),
            InputCmd::Equalizer(ch, EqualizerParameter::HmfFreq(val)) =>            append_i32(raw, 0x01, 0x06, 0x02, *ch, *val),
            InputCmd::Equalizer(ch, EqualizerParameter::HmfGain(val)) =>            append_i32(raw, 0x01, 0x06, 0x03, *ch, *val),
            InputCmd::Equalizer(ch, EqualizerParameter::HmfWidth(val)) =>           append_i32(raw, 0x01, 0x06, 0x04, *ch, *val),

            InputCmd::Equalizer(ch, EqualizerParameter::HfEnable(enabled)) =>       append_u8(raw, 0x01, 0x07, 0x00, *ch, *enabled),
            InputCmd::Equalizer(ch, EqualizerParameter::HfType(filter_type)) =>     append_u8(raw, 0x01, 0x07, 0x01, *ch, *filter_type),
            InputCmd::Equalizer(ch, EqualizerParameter::HfFreq(val)) =>             append_i32(raw, 0x01, 0x07, 0x02, *ch, *val),
            InputCmd::Equalizer(ch, EqualizerParameter::HfGain(val)) =>             append_i32(raw, 0x01, 0x07, 0x03, *ch, *val),
            InputCmd::Equalizer(ch, EqualizerParameter::HfWidth(val)) =>            append_i32(raw, 0x01, 0x07, 0x04, *ch, *val),

            InputCmd::Equalizer(ch, EqualizerParameter::LpfEnable(enabled)) =>      append_u8(raw, 0x01, 0x08, 0x00, *ch, *enabled),
            InputCmd::Equalizer(ch, EqualizerParameter::LpfSlope(level)) =>         append_u8(raw, 0x01, 0x08, 0x01, *ch, *level),
            InputCmd::Equalizer(ch, EqualizerParameter::LpfFreq(val)) =>            append_i32(raw, 0x01, 0x08, 0x02, *ch, *val),

            InputCmd::Dynamics(ch, DynamicsParameter::Enable(enabled)) =>           append_u8(raw, 0x01, 0x09, 0x00, *ch, *enabled),

            InputCmd::Dynamics(ch, DynamicsParameter::CompEnable(enabled)) =>       append_u8(raw, 0x01, 0x0a, 0x00, *ch, *enabled),
            InputCmd::Dynamics(ch, DynamicsParameter::CompThreshold(val)) =>        append_i32(raw, 0x01, 0x0a, 0x01, *ch, *val),
            InputCmd::Dynamics(ch, DynamicsParameter::CompRatio(val)) =>            append_i32(raw, 0x01, 0x0a, 0x02, *ch, *val),
            InputCmd::Dynamics(ch, DynamicsParameter::CompAttach(val)) =>           append_i32(raw, 0x01, 0x0a, 0x03, *ch, *val),
            InputCmd::Dynamics(ch, DynamicsParameter::CompRelease(val)) =>          append_i32(raw, 0x01, 0x0a, 0x04, *ch, *val),
            InputCmd::Dynamics(ch, DynamicsParameter::CompTrim(val)) =>             append_i32(raw, 0x01, 0x0a, 0x05, *ch, *val),
            InputCmd::Dynamics(ch, DynamicsParameter::CompDetectMode(mode)) =>      append_u8(raw, 0x01, 0x0a, 0x06, *ch, *mode),

            InputCmd::Dynamics(ch, DynamicsParameter::LevelerEnable(enabled)) =>    append_u8(raw, 0x01, 0x0b, 0x00, *ch, *enabled),
            InputCmd::Dynamics(ch, DynamicsParameter::LevelerMode(mode)) =>         append_u8(raw, 0x01, 0x0b, 0x01, *ch, *mode),
            InputCmd::Dynamics(ch, DynamicsParameter::LevelerMakeup(val)) =>        append_i32(raw, 0x01, 0x0b, 0x02, *ch, *val),
            InputCmd::Dynamics(ch, DynamicsParameter::LevelerReduce(val)) =>        append_i32(raw, 0x01, 0x0b, 0x03, *ch, *val),

            InputCmd::ReverbSend(ch, val) =>                                        append_i32(raw, 0x01, 0x0c, 0x00, *ch, *val),
            InputCmd::ReverbLrBalance(ch, val) =>                                   append_i32(raw, 0x01, 0x0c, 0x02, *ch, *val),

            InputCmd::Reserved(identifier, vals) =>                                 append_data(raw, identifier, vals),
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
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum MixerCmd {
    OutputAssign(usize, usize),
    OutputMute(usize, bool),
    OutputVolume(usize, i32),
    ReverbSend(usize, i32),
    ReverbReturn(usize, i32),
    SourceMute(usize, usize, bool),
    SourceSolo(usize, usize, bool),
    SourceMonauralLrBalance(usize, usize, i32),
    SourceGain(usize, usize, i32),
    SourceStereoMode(usize, usize, SourceStereoPairMode),
    SourceStereoLrBalance(usize, usize, i32),
    SourceStereoWidth(usize, usize, i32),
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
            (0x02, 0x00, 0x02) => MixerCmd::OutputVolume(ch, to_i32(vals)),

            (0x02, 0x01, 0x00) => MixerCmd::ReverbSend(ch, to_i32(vals)),
            (0x02, 0x01, 0x01) => MixerCmd::ReverbReturn(ch, to_i32(vals)),

            (0x02,    _, 0x00) => MixerCmd::SourceMute(ch, mixer_src_ch - 2, to_bool(vals)),
            (0x02,    _, 0x01) => MixerCmd::SourceSolo(ch, mixer_src_ch - 2, to_bool(vals)),
            (0x02,    _, 0x02) => MixerCmd::SourceMonauralLrBalance(ch, mixer_src_ch - 2, to_i32(vals)),
            (0x02,    _, 0x03) => MixerCmd::SourceGain(ch, mixer_src_ch - 2, to_i32(vals)),
            (0x02,    _, 0x04) => MixerCmd::SourceStereoMode(ch, mixer_src_ch - 2, SourceStereoPairMode::from(vals[0])),
            (0x02,    _, 0x05) => MixerCmd::SourceStereoLrBalance(ch, mixer_src_ch - 2, to_i32(vals)),
            (0x02,    _, 0x06) => MixerCmd::SourceStereoWidth(ch, mixer_src_ch - 2, to_i32(vals)),
            _ => MixerCmd::Reserved(identifier.to_vec(), vals.to_vec()),
        }
    }

    fn build(&self, raw: &mut Vec<u8>) {
        match self {
            MixerCmd::OutputAssign(ch, target) =>                       append_u8(raw, 0x02, 0x00, 0x00, *ch, *target as u8),
            MixerCmd::OutputMute(ch, enabled) =>                        append_u8(raw, 0x02, 0x00, 0x01, *ch, *enabled),
            MixerCmd::OutputVolume(ch, val) =>                          append_i32(raw, 0x02, 0x00, 0x02, *ch, *val),

            MixerCmd::ReverbSend(ch, val) =>                            append_i32(raw, 0x02, 0x01, 0x00, *ch, *val),
            MixerCmd::ReverbReturn(ch, val) =>                          append_i32(raw, 0x02, 0x01, 0x01, *ch, *val),

            MixerCmd::SourceMute(ch, mixer_src_ch, enabled) =>          append_u8(raw, 0x02, (*mixer_src_ch + 2) as u8, 0x00, *ch, *enabled),
            MixerCmd::SourceSolo(ch, mixer_src_ch, enabled) =>          append_u8(raw, 0x02, (*mixer_src_ch + 2) as u8, 0x01, *ch, *enabled),
            MixerCmd::SourceMonauralLrBalance(ch, mixer_src_ch, val) => append_i32(raw, 0x02, (*mixer_src_ch + 2) as u8, 0x02, *ch, *val),
            MixerCmd::SourceGain(ch, mixer_src_ch, val) =>              append_i32(raw, 0x02, (*mixer_src_ch + 2) as u8, 0x03, *ch, *val),
            MixerCmd::SourceStereoMode(ch, mixer_src_ch, pair_mode) =>  append_u8(raw, 0x02, (*mixer_src_ch + 2) as u8, 0x04, *ch, *pair_mode),
            MixerCmd::SourceStereoLrBalance(ch, mixer_src_ch, val) =>   append_i32(raw, 0x02, (*mixer_src_ch + 2) as u8, 0x05, *ch, *val),
            MixerCmd::SourceStereoWidth(ch, mixer_src_ch, val) =>       append_i32(raw, 0x02, (*mixer_src_ch + 2) as u8, 0x06, *ch, *val),

            MixerCmd::Reserved(identifier, vals) =>                     append_data(raw, identifier, vals),
        }
    }
}

/// The DSP command specific to input.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum OutputCmd {
    Equalizer(usize, EqualizerParameter),
    Dynamics(usize, DynamicsParameter),
    ReverbSend(usize, i32),
    ReverbReturn(usize, i32),
    MasterMonitor(usize, bool),
    MasterTalkback(usize, bool),
    MasterListenback(usize, bool),
    Reserved(Vec<u8>, Vec<u8>),
}

impl OutputCmd {
    fn parse(identifier: &[u8], vals: &[u8]) -> Self {
        let ch = identifier[0] as usize;

        match (identifier[3], identifier[2], identifier[1]) {
            (0x03, 0x00, 0x00) => OutputCmd::Equalizer(ch, EqualizerParameter::Enable(to_bool(vals))),

            (0x03, 0x01, 0x00) => OutputCmd::Equalizer(ch, EqualizerParameter::HpfEnable(to_bool(vals))),
            (0x03, 0x01, 0x01) => OutputCmd::Equalizer(ch, EqualizerParameter::HpfSlope(RollOffLevel::from(vals[0]))),
            (0x03, 0x01, 0x02) => OutputCmd::Equalizer(ch, EqualizerParameter::HpfFreq(to_i32(vals))),

            (0x03, 0x02, 0x00) => OutputCmd::Equalizer(ch, EqualizerParameter::LfEnable(to_bool(vals))),
            (0x03, 0x02, 0x01) => OutputCmd::Equalizer(ch, EqualizerParameter::LfType(FilterType5::from(vals[0]))),
            (0x03, 0x02, 0x02) => OutputCmd::Equalizer(ch, EqualizerParameter::LfFreq(to_i32(vals))),
            (0x03, 0x02, 0x03) => OutputCmd::Equalizer(ch, EqualizerParameter::LfGain(to_i32(vals))),
            (0x03, 0x02, 0x04) => OutputCmd::Equalizer(ch, EqualizerParameter::LfWidth(to_i32(vals))),

            (0x03, 0x03, 0x00) => OutputCmd::Equalizer(ch, EqualizerParameter::LmfEnable(to_bool(vals))),
            (0x03, 0x03, 0x01) => OutputCmd::Equalizer(ch, EqualizerParameter::LmfType(FilterType4::from(vals[0]))),
            (0x03, 0x03, 0x02) => OutputCmd::Equalizer(ch, EqualizerParameter::LmfFreq(to_i32(vals))),
            (0x03, 0x03, 0x03) => OutputCmd::Equalizer(ch, EqualizerParameter::LmfGain(to_i32(vals))),
            (0x03, 0x03, 0x04) => OutputCmd::Equalizer(ch, EqualizerParameter::LmfWidth(to_i32(vals))),

            (0x03, 0x04, 0x00) => OutputCmd::Equalizer(ch, EqualizerParameter::MfEnable(to_bool(vals))),
            (0x03, 0x04, 0x01) => OutputCmd::Equalizer(ch, EqualizerParameter::MfType(FilterType4::from(vals[0]))),
            (0x03, 0x04, 0x02) => OutputCmd::Equalizer(ch, EqualizerParameter::MfFreq(to_i32(vals))),
            (0x03, 0x04, 0x03) => OutputCmd::Equalizer(ch, EqualizerParameter::MfGain(to_i32(vals))),
            (0x03, 0x04, 0x04) => OutputCmd::Equalizer(ch, EqualizerParameter::MfWidth(to_i32(vals))),

            (0x03, 0x05, 0x00) => OutputCmd::Equalizer(ch, EqualizerParameter::HmfEnable(to_bool(vals))),
            (0x03, 0x05, 0x01) => OutputCmd::Equalizer(ch, EqualizerParameter::HmfType(FilterType4::from(vals[0]))),
            (0x03, 0x05, 0x02) => OutputCmd::Equalizer(ch, EqualizerParameter::HmfFreq(to_i32(vals))),
            (0x03, 0x05, 0x03) => OutputCmd::Equalizer(ch, EqualizerParameter::HmfGain(to_i32(vals))),
            (0x03, 0x05, 0x04) => OutputCmd::Equalizer(ch, EqualizerParameter::HmfWidth(to_i32(vals))),

            (0x03, 0x06, 0x00) => OutputCmd::Equalizer(ch, EqualizerParameter::HfEnable(to_bool(vals))),
            (0x03, 0x06, 0x01) => OutputCmd::Equalizer(ch, EqualizerParameter::HfType(FilterType5::from(vals[0]))),
            (0x03, 0x06, 0x02) => OutputCmd::Equalizer(ch, EqualizerParameter::HfFreq(to_i32(vals))),
            (0x03, 0x06, 0x03) => OutputCmd::Equalizer(ch, EqualizerParameter::HfGain(to_i32(vals))),
            (0x03, 0x06, 0x04) => OutputCmd::Equalizer(ch, EqualizerParameter::HfWidth(to_i32(vals))),

            (0x03, 0x07, 0x00) => OutputCmd::Equalizer(ch, EqualizerParameter::LpfEnable(to_bool(vals))),
            (0x03, 0x07, 0x01) => OutputCmd::Equalizer(ch, EqualizerParameter::LpfSlope(RollOffLevel::from(vals[0]))),
            (0x03, 0x07, 0x02) => OutputCmd::Equalizer(ch, EqualizerParameter::LpfFreq(to_i32(vals))),

            (0x03, 0x08, 0x00) => OutputCmd::Dynamics(ch, DynamicsParameter::Enable(to_bool(vals))),

            (0x03, 0x09, 0x00) => OutputCmd::Dynamics(ch, DynamicsParameter::CompEnable(to_bool(vals))),
            (0x03, 0x09, 0x01) => OutputCmd::Dynamics(ch, DynamicsParameter::CompThreshold(to_i32(vals))),
            (0x03, 0x09, 0x02) => OutputCmd::Dynamics(ch, DynamicsParameter::CompRatio(to_i32(vals))),
            (0x03, 0x09, 0x03) => OutputCmd::Dynamics(ch, DynamicsParameter::CompAttach(to_i32(vals))),
            (0x03, 0x09, 0x04) => OutputCmd::Dynamics(ch, DynamicsParameter::CompRelease(to_i32(vals))),
            (0x03, 0x09, 0x05) => OutputCmd::Dynamics(ch, DynamicsParameter::CompTrim(to_i32(vals))),
            (0x03, 0x09, 0x06) => OutputCmd::Dynamics(ch, DynamicsParameter::CompDetectMode(LevelDetectMode::from(vals[0]))),

            (0x03, 0x0a, 0x00) => OutputCmd::Dynamics(ch, DynamicsParameter::LevelerEnable(to_bool(vals))),
            (0x03, 0x0a, 0x01) => OutputCmd::Dynamics(ch, DynamicsParameter::LevelerMode(LevelerMode::from(vals[0]))),
            (0x03, 0x0a, 0x02) => OutputCmd::Dynamics(ch, DynamicsParameter::LevelerMakeup(to_i32(vals))),
            (0x03, 0x0a, 0x03) => OutputCmd::Dynamics(ch, DynamicsParameter::LevelerReduce(to_i32(vals))),

            (0x03, 0x0b, 0x00) => OutputCmd::ReverbSend(ch, to_i32(vals)),
            (0x03, 0x0b, 0x01) => OutputCmd::ReverbReturn(ch, to_i32(vals)),

            (0x03, 0x0c, 0x00) => OutputCmd::MasterMonitor(ch, to_bool(vals)),
            (0x03, 0x0c, 0x01) => OutputCmd::MasterTalkback(ch, to_bool(vals)),
            (0x03, 0x0c, 0x02) => OutputCmd::MasterListenback(ch, to_bool(vals)),

            _ => OutputCmd::Reserved(identifier.to_vec(), vals.to_vec()),
        }
    }

    fn build(&self, raw: &mut Vec<u8>) {
        match self {
            OutputCmd::Equalizer(ch, EqualizerParameter::Enable(enabled)) =>        append_u8(raw, 0x03, 0x00, 0x00, *ch, *enabled),

            OutputCmd::Equalizer(ch, EqualizerParameter::HpfEnable(enabled)) =>     append_u8(raw, 0x03, 0x01, 0x00, *ch, *enabled),
            OutputCmd::Equalizer(ch, EqualizerParameter::HpfSlope(level)) =>        append_u8(raw, 0x03, 0x01, 0x01, *ch, *level),
            OutputCmd::Equalizer(ch, EqualizerParameter::HpfFreq(val)) =>           append_i32(raw, 0x03, 0x01, 0x02, *ch, *val),

            OutputCmd::Equalizer(ch, EqualizerParameter::LfEnable(enabled)) =>      append_u8(raw, 0x03, 0x02, 0x00, *ch, *enabled),
            OutputCmd::Equalizer(ch, EqualizerParameter::LfType(filter_type)) =>    append_u8(raw, 0x03, 0x02, 0x01, *ch, *filter_type),
            OutputCmd::Equalizer(ch, EqualizerParameter::LfFreq(val)) =>            append_i32(raw, 0x03, 0x02, 0x02, *ch, *val),
            OutputCmd::Equalizer(ch, EqualizerParameter::LfGain(val)) =>            append_i32(raw, 0x03, 0x02, 0x03, *ch, *val),
            OutputCmd::Equalizer(ch, EqualizerParameter::LfWidth(val)) =>           append_i32(raw, 0x03, 0x02, 0x04, *ch, *val),

            OutputCmd::Equalizer(ch, EqualizerParameter::LmfEnable(enabled)) =>     append_u8(raw, 0x03, 0x03, 0x00, *ch, *enabled),
            OutputCmd::Equalizer(ch, EqualizerParameter::LmfType(filter_type)) =>   append_u8(raw, 0x03, 0x03, 0x01, *ch, *filter_type),
            OutputCmd::Equalizer(ch, EqualizerParameter::LmfFreq(val)) =>           append_i32(raw, 0x03, 0x03, 0x02, *ch, *val),
            OutputCmd::Equalizer(ch, EqualizerParameter::LmfGain(val)) =>           append_i32(raw, 0x03, 0x03, 0x03, *ch, *val),
            OutputCmd::Equalizer(ch, EqualizerParameter::LmfWidth(val)) =>          append_i32(raw, 0x03, 0x03, 0x04, *ch, *val),

            OutputCmd::Equalizer(ch, EqualizerParameter::MfEnable(enabled)) =>      append_u8(raw, 0x03, 0x04, 0x00, *ch, *enabled),
            OutputCmd::Equalizer(ch, EqualizerParameter::MfType(filter_type)) =>    append_u8(raw, 0x03, 0x04, 0x01, *ch, *filter_type),
            OutputCmd::Equalizer(ch, EqualizerParameter::MfFreq(val)) =>            append_i32(raw, 0x03, 0x04, 0x02, *ch, *val),
            OutputCmd::Equalizer(ch, EqualizerParameter::MfGain(val)) =>            append_i32(raw, 0x03, 0x04, 0x03, *ch, *val),
            OutputCmd::Equalizer(ch, EqualizerParameter::MfWidth(val)) =>           append_i32(raw, 0x03, 0x04, 0x04, *ch, *val),

            OutputCmd::Equalizer(ch, EqualizerParameter::HmfEnable(enabled)) =>     append_u8(raw, 0x03, 0x05, 0x00, *ch, *enabled),
            OutputCmd::Equalizer(ch, EqualizerParameter::HmfType(filter_type)) =>   append_u8(raw, 0x03, 0x05, 0x01, *ch, *filter_type),
            OutputCmd::Equalizer(ch, EqualizerParameter::HmfFreq(val)) =>           append_i32(raw, 0x03, 0x05, 0x02, *ch, *val),
            OutputCmd::Equalizer(ch, EqualizerParameter::HmfGain(val)) =>           append_i32(raw, 0x03, 0x05, 0x03, *ch, *val),
            OutputCmd::Equalizer(ch, EqualizerParameter::HmfWidth(val)) =>          append_i32(raw, 0x03, 0x05, 0x04, *ch, *val),

            OutputCmd::Equalizer(ch, EqualizerParameter::HfEnable(enabled)) =>      append_u8(raw, 0x03, 0x06, 0x00, *ch, *enabled),
            OutputCmd::Equalizer(ch, EqualizerParameter::HfType(filter_type)) =>    append_u8(raw, 0x03, 0x06, 0x01, *ch, *filter_type),
            OutputCmd::Equalizer(ch, EqualizerParameter::HfFreq(val)) =>            append_i32(raw, 0x03, 0x06, 0x02, *ch, *val),
            OutputCmd::Equalizer(ch, EqualizerParameter::HfGain(val)) =>            append_i32(raw, 0x03, 0x06, 0x03, *ch, *val),
            OutputCmd::Equalizer(ch, EqualizerParameter::HfWidth(val)) =>           append_i32(raw, 0x03, 0x06, 0x04, *ch, *val),

            OutputCmd::Equalizer(ch, EqualizerParameter::LpfEnable(enabled)) =>     append_u8(raw, 0x03, 0x07, 0x00, *ch, *enabled),
            OutputCmd::Equalizer(ch, EqualizerParameter::LpfSlope(level)) =>        append_u8(raw, 0x03, 0x07, 0x01, *ch, *level),
            OutputCmd::Equalizer(ch, EqualizerParameter::LpfFreq(val)) =>           append_i32(raw, 0x03, 0x07, 0x02, *ch, *val),

            OutputCmd::Dynamics(ch, DynamicsParameter::Enable(enabled)) =>          append_u8(raw, 0x03, 0x08, 0x00, *ch, *enabled),

            OutputCmd::Dynamics(ch, DynamicsParameter::CompEnable(enabled)) =>      append_u8(raw, 0x03, 0x09, 0x00, *ch, *enabled),
            OutputCmd::Dynamics(ch, DynamicsParameter::CompThreshold(val)) =>       append_i32(raw, 0x03, 0x09, 0x01, *ch, *val),
            OutputCmd::Dynamics(ch, DynamicsParameter::CompRatio(val)) =>           append_i32(raw, 0x03, 0x09, 0x02, *ch, *val),
            OutputCmd::Dynamics(ch, DynamicsParameter::CompAttach(val)) =>          append_i32(raw, 0x03, 0x09, 0x03, *ch, *val),
            OutputCmd::Dynamics(ch, DynamicsParameter::CompRelease(val)) =>         append_i32(raw, 0x03, 0x09, 0x04, *ch, *val),
            OutputCmd::Dynamics(ch, DynamicsParameter::CompTrim(val)) =>            append_i32(raw, 0x03, 0x09, 0x05, *ch, *val),
            OutputCmd::Dynamics(ch, DynamicsParameter::CompDetectMode(mode)) =>     append_u8(raw, 0x03, 0x09, 0x06, *ch, *mode),

            OutputCmd::Dynamics(ch, DynamicsParameter::LevelerEnable(enabled)) =>   append_u8(raw, 0x03, 0x0a, 0x00, *ch, *enabled),
            OutputCmd::Dynamics(ch, DynamicsParameter::LevelerMode(mode)) =>        append_u8(raw, 0x03, 0x0a, 0x01, *ch, *mode),
            OutputCmd::Dynamics(ch, DynamicsParameter::LevelerMakeup(val)) =>       append_i32(raw, 0x03, 0x0a, 0x02, *ch, *val),
            OutputCmd::Dynamics(ch, DynamicsParameter::LevelerReduce(val)) =>       append_i32(raw, 0x03, 0x0a, 0x03, *ch, *val),

            OutputCmd::ReverbSend(ch, val) =>                                       append_i32(raw, 0x03, 0x0b, 0x00, *ch, *val),
            OutputCmd::ReverbReturn(ch, val) =>                                     append_i32(raw, 0x03, 0x0b, 0x01, *ch, *val),

            OutputCmd::MasterMonitor(ch, val) =>                                    append_u8(raw, 0x03, 0x0c, 0x00, *ch, *val),
            OutputCmd::MasterTalkback(ch, enabled) =>                               append_u8(raw, 0x03, 0x0c, 0x01, *ch, *enabled),
            OutputCmd::MasterListenback(ch, enabled) =>                             append_u8(raw, 0x03, 0x0c, 0x02, *ch, *enabled),

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
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ReverbCmd {
    Enable(bool),
    Split(SplitPoint),
    PreDelay(i32),
    ShelfFilterFreq(i32),
    ShelfFilterAttenuation(i32),
    DecayTime(i32),
    TimeLow(i32),
    TimeMiddle(i32),
    TimeHigh(i32),
    CrossoverLow(i32),
    CrossoverHigh(i32),
    Width(i32),
    ReflectionMode(RoomShape),
    ReflectionSize(i32),
    ReflectionLevel(i32),
    Reserved(Vec<u8>, Vec<u8>),
}

impl ReverbCmd {
    fn parse(identifier: &[u8], vals: &[u8]) -> Self {
        assert_eq!(identifier.len(), 4);
        assert!(vals.len() > 0);

        match (identifier[3], identifier[2], identifier[1]) {
            (0x04, 0x00, 0x00) => ReverbCmd::Enable(to_bool(vals)),
            (0x04, 0x00, 0x01) => ReverbCmd::Split(SplitPoint::from(vals[0])),
            (0x04, 0x00, 0x02) => ReverbCmd::PreDelay(to_i32(vals)),
            (0x04, 0x00, 0x03) => ReverbCmd::ShelfFilterFreq(to_i32(vals)),
            (0x04, 0x00, 0x04) => ReverbCmd::ShelfFilterAttenuation(to_i32(vals)),
            (0x04, 0x00, 0x05) => ReverbCmd::DecayTime(to_i32(vals)),
            (0x04, 0x00, 0x06) => ReverbCmd::TimeLow(to_i32(vals)),
            (0x04, 0x00, 0x07) => ReverbCmd::TimeMiddle(to_i32(vals)),
            (0x04, 0x00, 0x08) => ReverbCmd::TimeHigh(to_i32(vals)),
            (0x04, 0x00, 0x09) => ReverbCmd::CrossoverLow(to_i32(vals)),
            (0x04, 0x00, 0x0a) => ReverbCmd::CrossoverHigh(to_i32(vals)),
            (0x04, 0x00, 0x0b) => ReverbCmd::Width(to_i32(vals)),
            (0x04, 0x00, 0x0c) => ReverbCmd::ReflectionMode(RoomShape::from(vals[0])),
            (0x04, 0x00, 0x0d) => ReverbCmd::ReflectionSize(to_i32(vals)),
            (0x04, 0x00, 0x0e) => ReverbCmd::ReflectionLevel(to_i32(vals)),
            _ => ReverbCmd::Reserved(identifier.to_vec(), vals.to_vec()),
        }
    }

    fn build(&self, raw: &mut Vec<u8>) {
        match self {
            ReverbCmd::Enable(enabled) =>               append_u8(raw, 0x04, 0x00, 0x00, 0, *enabled),
            ReverbCmd::Split(point) =>                  append_u8(raw, 0x04, 0x00, 0x01, 0, *point),
            ReverbCmd::PreDelay(val) =>                 append_i32(raw, 0x04, 0x00, 0x02, 0, *val),
            ReverbCmd::ShelfFilterFreq(val) =>          append_i32(raw, 0x04, 0x00, 0x03, 0, *val),
            ReverbCmd::ShelfFilterAttenuation(val) =>   append_i32(raw, 0x04, 0x00, 0x04, 0, *val),
            ReverbCmd::DecayTime(val) =>                append_i32(raw, 0x04, 0x00, 0x05, 0, *val),
            ReverbCmd::TimeLow(val) =>                  append_i32(raw, 0x04, 0x00, 0x06, 0, *val),
            ReverbCmd::TimeMiddle(val) =>               append_i32(raw, 0x04, 0x00, 0x07, 0, *val),
            ReverbCmd::TimeHigh(val) =>                 append_i32(raw, 0x04, 0x00, 0x08, 0, *val),
            ReverbCmd::CrossoverLow(val) =>             append_i32(raw, 0x04, 0x00, 0x09, 0, *val),
            ReverbCmd::CrossoverHigh(val) =>            append_i32(raw, 0x04, 0x00, 0x0a, 0, *val),
            ReverbCmd::Width(val) =>                    append_i32(raw, 0x04, 0x00, 0x0b, 0, *val),
            ReverbCmd::ReflectionSize(val) =>           append_i32(raw, 0x04, 0x00, 0x0d, 0, *val),
            ReverbCmd::ReflectionMode(shape) =>         append_u8(raw, 0x04, 0x00, 0x0c, 0, *shape),
            ReverbCmd::ReflectionLevel(val) =>          append_i32(raw, 0x04, 0x00, 0x0e, 0, *val),
            ReverbCmd::Reserved(identifier, vals) =>    append_data(raw, identifier, vals),
        }
    }
}

/// The DSP command specific to usage of resource.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ResourceCmd {
    Usage(u32, u8),
    Reserved(Vec<u8>),
}

impl ResourceCmd {
    fn parse(raw: &[u8]) -> Self {
        let mut quadlet = [0; 4];
        quadlet.copy_from_slice(&raw[1..5]);
        ResourceCmd::Usage(u32::from_be_bytes(quadlet), raw[5])
    }

    fn build(&self, raw: &mut Vec<u8>) {
        match self {
            Self::Usage(usage, flag) => append_resource(raw, *usage, *flag),
            Self::Reserved(data) => raw.extend_from_slice(data),
        }
    }
}

/// The DSP command.
#[derive(Debug, Clone, Eq, PartialEq)]
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
                    (0..count)
                        .for_each(|i| {
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
                    (0..count)
                        .for_each(|i| {
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

fn append_u8<T>(raw: &mut Vec<u8>, first_level: u8, second_level: u8, third_level: u8, ch: usize, val: T)
    where u8: From<T>
{
    raw.push(CMD_BYTE_SINGLE);
    raw.push(u8::from(val));
    raw.push(ch as u8);
    raw.push(third_level);
    raw.push(second_level);
    raw.push(first_level);
}

fn append_i32(raw: &mut Vec<u8>, first_level: u8, second_level: u8, third_level: u8, ch: usize, val: i32) {
    raw.push(CMD_QUADLET_SINGLE);
    raw.push(ch as u8);
    raw.push(third_level);
    raw.push(second_level);
    raw.push(first_level);
    raw.extend_from_slice(&val.to_le_bytes());
}

fn append_resource(raw: &mut Vec<u8>, usage: u32, flag: u8) {
    raw.push(CMD_RESOURCE);
    raw.extend_from_slice(&usage.to_be_bytes());
    raw.push(flag);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_u8_cmds() {
        [
            DspCmd::Monitor(MonitorCmd::ReturnAssign(0x69)),
            DspCmd::Input(InputCmd::Phase(0x59, true)),
            DspCmd::Input(InputCmd::Pair(0x0, false)),
            DspCmd::Input(InputCmd::Swap(0x24, false)),
            DspCmd::Input(InputCmd::StereoMode(0x35, InputStereoPairMode::MonauralStereo)),
            DspCmd::Input(InputCmd::Limitter(0xad, true)),
            DspCmd::Input(InputCmd::Lookahead(0xdd, true)),
            DspCmd::Input(InputCmd::Softclip(0xfc, false)),
            DspCmd::Input(InputCmd::Pad(0x91, true)),
            DspCmd::Input(InputCmd::Phantom(0x13, false)),
            DspCmd::Input(InputCmd::Equalizer(0x14, EqualizerParameter::Enable(false))),
            DspCmd::Input(InputCmd::Equalizer(0x23, EqualizerParameter::HpfEnable(true))),
            DspCmd::Input(InputCmd::Equalizer(0x32, EqualizerParameter::HpfSlope(RollOffLevel::L30))),
            DspCmd::Input(InputCmd::Equalizer(0x41, EqualizerParameter::LfEnable(false))),
            DspCmd::Input(InputCmd::Equalizer(0x59, EqualizerParameter::LfType(FilterType5::Shelf))),
            DspCmd::Input(InputCmd::Equalizer(0x68, EqualizerParameter::LmfEnable(true))),
            DspCmd::Input(InputCmd::Equalizer(0x77, EqualizerParameter::LmfType(FilterType4::T4))),
            DspCmd::Input(InputCmd::Equalizer(0x86, EqualizerParameter::MfEnable(false))),
            DspCmd::Input(InputCmd::Equalizer(0x95, EqualizerParameter::MfType(FilterType4::T3))),
            DspCmd::Input(InputCmd::Equalizer(0xaf, EqualizerParameter::HmfEnable(true))),
            DspCmd::Input(InputCmd::Equalizer(0xbe, EqualizerParameter::HmfType(FilterType4::T2))),
            DspCmd::Input(InputCmd::Equalizer(0xcd, EqualizerParameter::HfEnable(false))),
            DspCmd::Input(InputCmd::Equalizer(0xdc, EqualizerParameter::HfType(FilterType5::T1))),
            DspCmd::Input(InputCmd::Equalizer(0xeb, EqualizerParameter::LpfEnable(true))),
            DspCmd::Input(InputCmd::Equalizer(0xfa, EqualizerParameter::LpfSlope(RollOffLevel::L24))),
            DspCmd::Input(InputCmd::Dynamics(0xf0, DynamicsParameter::Enable(false))),
            DspCmd::Input(InputCmd::Dynamics(0xe1, DynamicsParameter::CompEnable(true))),
            DspCmd::Input(InputCmd::Dynamics(0xd2, DynamicsParameter::CompDetectMode(LevelDetectMode::Rms))),
            DspCmd::Input(InputCmd::Dynamics(0xc3, DynamicsParameter::LevelerEnable(false))),
            DspCmd::Input(InputCmd::Dynamics(0xb4, DynamicsParameter::LevelerMode(LevelerMode::Limit))),
            DspCmd::Mixer(MixerCmd::OutputAssign(0xa5, 0x91)),
            DspCmd::Mixer(MixerCmd::OutputMute(0x96, true)),
            DspCmd::Mixer(MixerCmd::SourceMute(0x87, 0x13, false)),
            DspCmd::Mixer(MixerCmd::SourceSolo(0x78, 0x31, true)),
            DspCmd::Mixer(MixerCmd::SourceStereoMode(0x69, 0x11, SourceStereoPairMode::LrBalance)),
            DspCmd::Output(OutputCmd::Equalizer(0x5a, EqualizerParameter::Enable(false))),
            DspCmd::Output(OutputCmd::Equalizer(0x4b, EqualizerParameter::HpfEnable(true))),
            DspCmd::Output(OutputCmd::Equalizer(0x3c, EqualizerParameter::HpfSlope(RollOffLevel::L6))),
            DspCmd::Output(OutputCmd::Equalizer(0x2d, EqualizerParameter::LfEnable(false))),
            DspCmd::Output(OutputCmd::Equalizer(0x1e, EqualizerParameter::LfType(FilterType5::Shelf))),
            DspCmd::Output(OutputCmd::Equalizer(0x0f, EqualizerParameter::LmfEnable(true))),
            DspCmd::Output(OutputCmd::Equalizer(0xf1, EqualizerParameter::LmfType(FilterType4::T4))),
            DspCmd::Output(OutputCmd::Equalizer(0xe2, EqualizerParameter::MfEnable(false))),
            DspCmd::Output(OutputCmd::Equalizer(0xd3, EqualizerParameter::MfType(FilterType4::T3))),
            DspCmd::Output(OutputCmd::Equalizer(0xc4, EqualizerParameter::HmfEnable(true))),
            DspCmd::Output(OutputCmd::Equalizer(0xb5, EqualizerParameter::HmfType(FilterType4::T2))),
            DspCmd::Output(OutputCmd::Equalizer(0xa6, EqualizerParameter::HfEnable(false))),
            DspCmd::Output(OutputCmd::Equalizer(0x97, EqualizerParameter::HfType(FilterType5::T1))),
            DspCmd::Output(OutputCmd::Equalizer(0x88, EqualizerParameter::LpfEnable(true))),
            DspCmd::Output(OutputCmd::Equalizer(0x79, EqualizerParameter::LpfSlope(RollOffLevel::L18))),
            DspCmd::Output(OutputCmd::Dynamics(0xff, DynamicsParameter::Enable(false))),
            DspCmd::Output(OutputCmd::Dynamics(0xee, DynamicsParameter::CompEnable(true))),
            DspCmd::Output(OutputCmd::Dynamics(0xdd, DynamicsParameter::CompDetectMode(LevelDetectMode::Peak))),
            DspCmd::Output(OutputCmd::Dynamics(0xcc, DynamicsParameter::LevelerEnable(false))),
            DspCmd::Output(OutputCmd::Dynamics(0xbb, DynamicsParameter::LevelerMode(LevelerMode::Compress))),
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
            DspCmd::Monitor(MonitorCmd::Volume(0x00)),
            DspCmd::Input(InputCmd::Trim(0xe4, 0x01)),
            DspCmd::Input(InputCmd::Width(0xd3, 0x02)),
            DspCmd::Input(InputCmd::Equalizer(0xc2, EqualizerParameter::HpfFreq(0x01010101))),
            DspCmd::Input(InputCmd::Equalizer(0xb1, EqualizerParameter::LfFreq(0x02020202))),
            DspCmd::Input(InputCmd::Equalizer(0xa0, EqualizerParameter::LfGain(0x03030303))),
            DspCmd::Input(InputCmd::Equalizer(0x9f, EqualizerParameter::LfWidth(0x04040404))),
            DspCmd::Input(InputCmd::Equalizer(0x8e, EqualizerParameter::LmfFreq(0x05050505))),
            DspCmd::Input(InputCmd::Equalizer(0x7d, EqualizerParameter::LmfGain(0x06060606))),
            DspCmd::Input(InputCmd::Equalizer(0x6c, EqualizerParameter::LmfWidth(0x07070707))),
            DspCmd::Input(InputCmd::Equalizer(0x5b, EqualizerParameter::MfFreq(0x08080808))),
            DspCmd::Input(InputCmd::Equalizer(0x4a, EqualizerParameter::MfGain(0x09090909))),
            DspCmd::Input(InputCmd::Equalizer(0x39, EqualizerParameter::MfWidth(0x0a0a0a0a))),
            DspCmd::Input(InputCmd::Equalizer(0x28, EqualizerParameter::HmfFreq(0x0b0b0b0b))),
            DspCmd::Input(InputCmd::Equalizer(0x17, EqualizerParameter::HmfGain(0x0c0c0c0c))),
            DspCmd::Input(InputCmd::Equalizer(0x06, EqualizerParameter::HmfWidth(0x0d0d0d0d))),
            DspCmd::Input(InputCmd::Equalizer(0xf5, EqualizerParameter::HfFreq(0x0e0e0e0e))),
            DspCmd::Input(InputCmd::Equalizer(0xe4, EqualizerParameter::HfGain(0x0f0f0f0f))),
            DspCmd::Input(InputCmd::Equalizer(0xd3, EqualizerParameter::HfWidth(01234567))),
            DspCmd::Input(InputCmd::Equalizer(0xc2, EqualizerParameter::LpfFreq(0x2345678))),
            DspCmd::Input(InputCmd::Dynamics(0xb1, DynamicsParameter::CompThreshold(0x3456789))),
            DspCmd::Input(InputCmd::Dynamics(0xa0, DynamicsParameter::CompRatio(0x456789a))),
            DspCmd::Input(InputCmd::Dynamics(0x9f, DynamicsParameter::CompAttach(0x56789ab))),
            DspCmd::Input(InputCmd::Dynamics(0x8e, DynamicsParameter::CompRelease(0x6789abc))),
            DspCmd::Input(InputCmd::Dynamics(0x7d, DynamicsParameter::CompTrim(0x789abcde))),
            DspCmd::Input(InputCmd::Dynamics(0x6c, DynamicsParameter::LevelerMakeup(0x09abcdef))),
            DspCmd::Input(InputCmd::Dynamics(0x5b, DynamicsParameter::LevelerReduce(0x1c92835a))),
            DspCmd::Input(InputCmd::ReverbSend(0x33, 0x35792468)),
            DspCmd::Input(InputCmd::ReverbLrBalance(0xcc, 0x24689753)),
            DspCmd::Mixer(MixerCmd::OutputVolume(0x4a, 0x7789abcd)),
            DspCmd::Mixer(MixerCmd::ReverbSend(0x39, 0x66789abc)),
            DspCmd::Mixer(MixerCmd::ReverbReturn(0x28, 0x11234567)),
            DspCmd::Mixer(MixerCmd::SourceMonauralLrBalance(0x17, 0xc8, 0x76543210)),
            DspCmd::Mixer(MixerCmd::SourceGain(0x06, 0x11, 0x65432109)),
            DspCmd::Mixer(MixerCmd::SourceStereoLrBalance(0xe5, 0x13, 0x54321987)),
            DspCmd::Mixer(MixerCmd::SourceStereoWidth(0xd4, 0x1a, 0x43210987)),
            DspCmd::Output(OutputCmd::Equalizer(0xa8, EqualizerParameter::HpfFreq(0x77792f78))),
            DspCmd::Output(OutputCmd::Equalizer(0x39, EqualizerParameter::LfFreq(0x20fc256f))),
            DspCmd::Output(OutputCmd::Equalizer(0x11, EqualizerParameter::LfGain(0x34649fb4))),
            DspCmd::Output(OutputCmd::Equalizer(0x5a, EqualizerParameter::LfWidth(0x6620a2de))),
            DspCmd::Output(OutputCmd::Equalizer(0x5b, EqualizerParameter::LmfFreq(0x1e10a3f8))),
            DspCmd::Output(OutputCmd::Equalizer(0x98, EqualizerParameter::LmfGain(0x6d0b5422))),
            DspCmd::Output(OutputCmd::Equalizer(0x74, EqualizerParameter::LmfWidth(0x72b8ce7c))),
            DspCmd::Output(OutputCmd::Equalizer(0xbc, EqualizerParameter::MfFreq(0x50110b27))),
            DspCmd::Output(OutputCmd::Equalizer(0x32, EqualizerParameter::MfGain(0x2155f212))),
            DspCmd::Output(OutputCmd::Equalizer(0x20, EqualizerParameter::MfWidth(0x31d83f53))),
            DspCmd::Output(OutputCmd::Equalizer(0xf7, EqualizerParameter::HmfFreq(0x2c79c6f3))),
            DspCmd::Output(OutputCmd::Equalizer(0xc0, EqualizerParameter::HmfGain(0x12d6c247))),
            DspCmd::Output(OutputCmd::Equalizer(0xf5, EqualizerParameter::HmfWidth(0x53a26fe4))),
            DspCmd::Output(OutputCmd::Equalizer(0xc0, EqualizerParameter::HfFreq(0x0b1f0cb3))),
            DspCmd::Output(OutputCmd::Equalizer(0x01, EqualizerParameter::HfGain(0x2b6de491))),
            DspCmd::Output(OutputCmd::Equalizer(0xdb, EqualizerParameter::HfWidth(0x0e7a2c75))),
            DspCmd::Output(OutputCmd::Equalizer(0x29, EqualizerParameter::LpfFreq(0x1cbdda81))),
            DspCmd::Output(OutputCmd::Dynamics(0x45, DynamicsParameter::CompThreshold(0x2469b8dd))),
            DspCmd::Output(OutputCmd::Dynamics(0x5e, DynamicsParameter::CompRatio(0x71136c4f))),
            DspCmd::Output(OutputCmd::Dynamics(0x1b, DynamicsParameter::CompAttach(0x0ea8d07d))),
            DspCmd::Output(OutputCmd::Dynamics(0x49, DynamicsParameter::CompRelease(0x28cff071))),
            DspCmd::Output(OutputCmd::Dynamics(0xba, DynamicsParameter::CompTrim(0x7cfab69f))),
            DspCmd::Output(OutputCmd::Dynamics(0x7f, DynamicsParameter::LevelerMakeup(0x100e66ba))),
            DspCmd::Output(OutputCmd::Dynamics(0xf2, DynamicsParameter::LevelerReduce(0x3a6bd56a))),
            DspCmd::Output(OutputCmd::ReverbSend(0x99, 0x19287465)),
            DspCmd::Output(OutputCmd::ReverbReturn(0x88, 0x59187342)),
            DspCmd::Reverb(ReverbCmd::PreDelay(0x556e2bc1)),
            DspCmd::Reverb(ReverbCmd::ShelfFilterFreq(0x4f760819)),
            DspCmd::Reverb(ReverbCmd::ShelfFilterAttenuation(0x29f2c867)),
            DspCmd::Reverb(ReverbCmd::DecayTime(0x5c5b8924)),
            DspCmd::Reverb(ReverbCmd::TimeLow(0x704980ae)),
            DspCmd::Reverb(ReverbCmd::TimeMiddle(0x741fdbf1)),
            DspCmd::Reverb(ReverbCmd::TimeHigh(0x4c24fcd4)),
            DspCmd::Reverb(ReverbCmd::CrossoverLow(0x11a9d331)),
            DspCmd::Reverb(ReverbCmd::CrossoverHigh(0x76a9aa46)),
            DspCmd::Reverb(ReverbCmd::Width(0x1d9d06c7)),
            DspCmd::Reverb(ReverbCmd::ReflectionSize(0x5e847d68)),
            DspCmd::Reverb(ReverbCmd::ReflectionLevel(0x235868ad)),
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
    fn test_resource() {
        let cmd = DspCmd::Resource(ResourceCmd::Usage(0x98765432, 0x17));
        let mut raw = Vec::new();
        cmd.build(&mut raw);
        let mut c = Vec::new();
        assert_eq!(DspCmd::parse(&raw, &mut c), CMD_RESOURCE_LENGTH);
        assert_eq!(c[0], cmd);
    }
}
