// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Global section in general protocol defined by TCAT for ASICs of DICE.
//!
//! The module includes structure, enumeration, and trait and its implementation for global section
//! in general protocol defined by TCAT for ASICs of DICE.
use super::{*, utils::*};

use std::convert::TryFrom;

/// The enumeration for nominal sampling rate.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ClockRate {
    R32000,
    R44100,
    R48000,
    R88200,
    R96000,
    R176400,
    R192000,
    AnyLow,
    AnyMid,
    AnyHigh,
    None,
    Reserved(u8),
}

impl ClockRate {
    const R32000_VAL: u8 = 0x00;
    const R44100_VAL: u8 = 0x01;
    const R48000_VAL: u8 = 0x02;
    const R88200_VAL: u8 = 0x03;
    const R96000_VAL: u8 = 0x04;
    const R176400_VAL: u8 = 0x05;
    const R192000_VAL: u8 = 0x06;
    const ANY_LOW_VAL: u8 = 0x07;
    const ANY_MID_VAL: u8 = 0x08;
    const ANY_HIGH_VAL: u8 = 0x09;
    const NONE_VAL: u8 = 0x0a;

    pub fn is_supported(&self, caps: &ClockCaps) -> bool {
        caps.rate_bits & (1 << u8::from(*self)) > 0
    }
}

impl Default for ClockRate {
    fn default() -> Self {
        ClockRate::Reserved(0xff)
    }
}

impl From<u8> for ClockRate {
    fn from(val: u8) -> Self {
        match val {
            Self::R32000_VAL => Self::R32000,
            Self::R44100_VAL => Self::R44100,
            Self::R48000_VAL => Self::R48000,
            Self::R88200_VAL => Self::R88200,
            Self::R96000_VAL => Self::R96000,
            Self::R176400_VAL => Self::R176400,
            Self::R192000_VAL => Self::R192000,
            Self::ANY_LOW_VAL => Self::AnyLow,
            Self::ANY_MID_VAL => Self::AnyMid,
            Self::ANY_HIGH_VAL => Self::AnyHigh,
            Self::NONE_VAL => Self::None,
            _ => Self::Reserved(val),
        }
    }
}

impl From<ClockRate> for u8 {
    fn from(rate: ClockRate) -> u8 {
        match rate {
            ClockRate::R32000 => ClockRate::R32000_VAL,
            ClockRate::R44100 => ClockRate::R44100_VAL,
            ClockRate::R48000 => ClockRate::R48000_VAL,
            ClockRate::R88200 => ClockRate::R88200_VAL,
            ClockRate::R96000 => ClockRate::R96000_VAL,
            ClockRate::R176400 => ClockRate::R176400_VAL,
            ClockRate::R192000 => ClockRate::R192000_VAL,
            ClockRate::AnyLow => ClockRate::ANY_LOW_VAL,
            ClockRate::AnyMid => ClockRate::ANY_MID_VAL,
            ClockRate::AnyHigh => ClockRate::ANY_HIGH_VAL,
            ClockRate::None => ClockRate::NONE_VAL,
            ClockRate::Reserved(val) => val,
        }
    }
}

impl TryFrom<u32> for ClockRate {
    type Error = Error;

    fn try_from(val: u32) -> Result<ClockRate, Self::Error> {
        match val {
            32000 => Ok(Self::R32000),
            44100 => Ok(Self::R44100),
            48000 => Ok(Self::R48000),
            88200 => Ok(Self::R88200),
            96000 => Ok(Self::R96000),
            176400 => Ok(Self::R176400),
            192000 => Ok(Self::R192000),
            _ => {
                let msg = format!("Fail to convert from nominal rate: {}", val);
                Err(Error::new(GeneralProtocolError::Global, &msg))
            },
        }
    }
}

impl TryFrom<ClockRate> for u32 {
    type Error = Error;

    fn try_from(rate: ClockRate) -> Result<u32, Self::Error> {
        match rate {
            ClockRate::R32000 => Ok(32000),
            ClockRate::R44100 => Ok(44100),
            ClockRate::R48000 => Ok(48000),
            ClockRate::R88200 => Ok(88200),
            ClockRate::R96000 => Ok(96000),
            ClockRate::R176400 => Ok(176400),
            ClockRate::R192000 => Ok(192000),
            _ => {
                let msg = format!("Fail to convert to nominal rate: {}", rate);
                Err(Error::new(GeneralProtocolError::Global, &msg))
            },
        }
    }
}

impl std::fmt::Display for ClockRate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            Self::R32000 => "32000".to_string(),
            Self::R44100 => "44100".to_string(),
            Self::R48000 => "48000".to_string(),
            Self::R88200 => "88200".to_string(),
            Self::R96000 => "96000".to_string(),
            Self::R176400 => "176400".to_string(),
            Self::R192000 => "192000".to_string(),
            Self::AnyLow => "Any-low".to_string(),
            Self::AnyMid => "Any-mid".to_string(),
            Self::AnyHigh => "Any-high".to_string(),
            Self::None => "None".to_string(),
            Self::Reserved(val) => format!("Reserved({})", val),
        };
        write!(f, "{}", label)
    }
}

/// The enumeration for nominal sampling rate.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ClockSource {
    Aes1,
    Aes2,
    Aes3,
    Aes4,
    AesAny,
    Adat,
    Tdif,
    WordClock,
    Arx1,
    Arx2,
    Arx3,
    Arx4,
    Internal,
    Reserved(u8),
}

impl Default for ClockSource {
    fn default() -> Self {
        ClockSource::Reserved(0xff)
    }
}

impl ClockSource {
    const AES1_VAL: u8 = 0x00;
    const AES2_VAL: u8 = 0x01;
    const AES3_VAL: u8 = 0x02;
    const AES4_VAL: u8 = 0x03;
    const AES_ANY_VAL: u8 = 0x04;
    const ADAT_VAL: u8 = 0x05;
    const TDIF_VAL: u8 = 0x06;
    const WORD_CLOCK_VAL: u8 = 0x07;
    const ARX1_VAL: u8 = 0x08;
    const ARX2_VAL: u8 = 0x09;
    const ARX3_VAL: u8 = 0x0a;
    const ARX4_VAL: u8 = 0x0b;
    const INTERNAL_VAL: u8 = 0x0c;

    pub fn is_supported(&self, caps: &ClockCaps, labels: &ClockSourceLabels) -> bool {
        let idx = u8::from(*self) as usize;
        (caps.src_bits & (1u16 << idx) > 0) &&
            labels.entries.iter().nth(idx).filter(|&l| l != "Unused" && l != "unused").is_some()
    }

    pub fn get_label<'a>(&self, labels: &'a ClockSourceLabels, is_ext: bool) -> Option<&'a str> {
        if is_ext && *self == ClockSource::Arx1 {
            Some("Stream")
        } else {
            let idx = u8::from(*self) as usize;
            labels.entries.iter()
                .nth(idx)
                .map(|l| l.as_str())
        }
    }

    fn parse(src: ClockSource, flags: u16) -> bool {
        ExtSourceStates::SRCS.iter()
            .enumerate()
            .find(|&(_, &s)| s == src)
            .map(|(i, _)| flags & (1 << i) > 0)
            .unwrap_or(false)
    }

    pub fn is_locked(&self, states: &ExtSourceStates) -> bool {
        Self::parse(*self, states.locked_bits)
    }

    pub fn is_slipped(&self, states: &ExtSourceStates) -> bool {
        Self::parse(*self, states.slipped_bits)
    }
}

impl From<u8> for ClockSource {
    fn from(val: u8) -> Self {
        match val {
            Self::AES1_VAL => Self::Aes1,
            Self::AES2_VAL => Self::Aes2,
            Self::AES3_VAL => Self::Aes3,
            Self::AES4_VAL => Self::Aes4,
            Self::AES_ANY_VAL => Self::AesAny,
            Self::ADAT_VAL => Self::Adat,
            Self::TDIF_VAL => Self::Tdif,
            Self::WORD_CLOCK_VAL => Self::WordClock,
            Self::ARX1_VAL => Self::Arx1,
            Self::ARX2_VAL => Self::Arx2,
            Self::ARX3_VAL => Self::Arx3,
            Self::ARX4_VAL => Self::Arx4,
            Self::INTERNAL_VAL => Self::Internal,
            _ => Self::Reserved(val),
        }
    }
}

impl From<ClockSource> for u8 {
    fn from(src: ClockSource) -> u8 {
        match src {
            ClockSource::Aes1 => ClockSource::AES1_VAL,
            ClockSource::Aes2 => ClockSource::AES2_VAL,
            ClockSource::Aes3 => ClockSource::AES3_VAL,
            ClockSource::Aes4 => ClockSource::AES4_VAL,
            ClockSource::AesAny => ClockSource::AES_ANY_VAL,
            ClockSource::Adat => ClockSource::ADAT_VAL,
            ClockSource::Tdif => ClockSource::TDIF_VAL,
            ClockSource::WordClock => ClockSource::WORD_CLOCK_VAL,
            ClockSource::Arx1 => ClockSource::ARX1_VAL,
            ClockSource::Arx2 => ClockSource::ARX2_VAL,
            ClockSource::Arx3 => ClockSource::ARX3_VAL,
            ClockSource::Arx4 => ClockSource::ARX4_VAL,
            ClockSource::Internal => ClockSource::INTERNAL_VAL,
            ClockSource::Reserved(val) => val,
        }
    }
}

impl std::fmt::Display for ClockSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            Self::Aes1 => "AES1".to_string(),
            Self::Aes2 => "AES2".to_string(),
            Self::Aes3 => "AES3".to_string(),
            Self::Aes4 => "AES4".to_string(),
            Self::AesAny => "AES-ANY".to_string(),
            Self::Adat => "ADAT".to_string(),
            Self::Tdif => "TDIF".to_string(),
            Self::WordClock => "Word-Clock".to_string(),
            Self::Arx1 => "ARX1".to_string(),
            Self::Arx2 => "ARX2".to_string(),
            Self::Arx3 => "ARX3".to_string(),
            Self::Arx4 => "ARX4".to_string(),
            Self::Internal=> "Internal".to_string(),
            Self::Reserved(val) => format!("Reserved({})", val),
        };
        write!(f, "{}", label)
    }
}

/// The structure to represent configuration of clock.
#[derive(Default, Debug, Clone, Copy, Eq, PartialEq)]
pub struct ClockConfig{
    pub rate: ClockRate,
    pub src: ClockSource,
}

impl ClockConfig {
    const SRC_MASK: u32 = 0x000000ff;
    const SRC_SHIFT: usize = 0;
    const RATE_MASK: u32 = 0x0000ff00;
    const RATE_SHIFT: usize = 8;
}

impl From<u32> for ClockConfig {
    fn from(val: u32) -> Self {
        let src_val = ((val & Self::SRC_MASK) >> Self::SRC_SHIFT) as u8;
        let rate_val = ((val & Self::RATE_MASK) >> Self::RATE_SHIFT) as u8;
        let src = ClockSource::from(src_val);
        let rate = ClockRate::from(rate_val);
        ClockConfig{rate, src}
    }
}

impl From<ClockConfig> for u32 {
    fn from(cfg: ClockConfig) -> u32 {
        let src_val = u8::from(cfg.src) as u32;
        let rate_val = u8::from(cfg.rate) as u32;
        ((rate_val << ClockConfig::RATE_SHIFT) & ClockConfig::RATE_MASK)
            | ((src_val << ClockConfig::SRC_SHIFT) & ClockConfig::SRC_MASK)
    }
}

/// The structure to represent status of sampling clock.
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct ClockStatus{
    pub src_is_locked: bool,
    pub rate: ClockRate,
}

impl ClockStatus {
    const SRC_LOCKED: u32 = 0x00000001;
    const RATE_MASK: u32 = 0x0000ff00;
    const RATE_SHIFT: usize = 8;
}

impl From<u32> for ClockStatus {
    fn from(val: u32) -> Self {
        let src_is_locked = (val & Self::SRC_LOCKED) > 0;
        let rate_val = ((val & Self::RATE_MASK) >> Self::RATE_SHIFT) as u8;
        let rate = ClockRate::from(rate_val);
        ClockStatus{src_is_locked, rate}
    }
}

/// The structure to represent states of available clock source.
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct ExtSourceStates{
    locked_bits: u16,
    slipped_bits: u16,
}

impl ExtSourceStates {
    const LOCKED_BITS_MASK: u32 = 0x0000ffff;
    const LOCKED_BITS_SHIFT: usize = 0;
    const SLIPPED_BITS_MASK: u32 = 0xffff0000;
    const SLIPPED_BITS_SHIFT: usize = 16;

    const SRCS: [ClockSource;11] = [
        ClockSource::Aes1,
        ClockSource::Aes2,
        ClockSource::Aes3,
        ClockSource::Aes4,
        ClockSource::Adat,
        ClockSource::Tdif,
        ClockSource::Arx1,
        ClockSource::Arx2,
        ClockSource::Arx3,
        ClockSource::Arx4,
        ClockSource::WordClock,
    ];

    pub fn get_entries(caps: &ClockCaps, labels: &ClockSourceLabels) -> Vec<ClockSource> {
        Self::SRCS.iter()
            .filter(|&s| s.is_supported(caps, labels) || *s == ClockSource::Arx1)
            .copied()
            .collect()
    }
}

impl From<u32> for ExtSourceStates {
    fn from(val: u32) -> Self {
        let locked_bits = ((val & Self::LOCKED_BITS_MASK) >> Self::LOCKED_BITS_SHIFT) as u16;
        let slipped_bits = ((val & Self::SLIPPED_BITS_MASK) >> Self::SLIPPED_BITS_SHIFT) as u16;
        ExtSourceStates{locked_bits, slipped_bits}
    }
}

/// The structure to represent capabilities of clock configurations.
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct ClockCaps{
    pub rate_bits: u16,
    pub src_bits: u16,
}

impl ClockCaps {
    const RATE_MASK: u32 = 0x0000ffff;
    const RATE_SHIFT: usize = 0;
    const SRC_MASK: u32 = 0xffff0000;
    const SRC_SHIFT: usize = 16;

    const RATES: [ClockRate;11] = [
        ClockRate::R32000,
        ClockRate::R44100,
        ClockRate::R48000,
        ClockRate::R88200,
        ClockRate::R96000,
        ClockRate::R176400,
        ClockRate::R192000,
        ClockRate::AnyLow,
        ClockRate::AnyMid,
        ClockRate::AnyHigh,
        ClockRate::None,
    ];

    const SRCS: [ClockSource;13] = [
        ClockSource::Aes1,
        ClockSource::Aes2,
        ClockSource::Aes3,
        ClockSource::Aes4,
        ClockSource::AesAny,
        ClockSource::Adat,
        ClockSource::Tdif,
        ClockSource::WordClock,
        ClockSource::Arx1,
        ClockSource::Arx2,
        ClockSource::Arx3,
        ClockSource::Arx4,
        ClockSource::Internal,
    ];

    pub fn new(rates: &[ClockRate], srcs: &[ClockSource]) -> Self {
        let rate_bits = rates.iter()
            .fold(0, |val, &r| val | (1 << u8::from(r) as u16));
        let src_bits = srcs.iter()
            .fold(0, |val, &s| val | (1 << u8::from(s) as u16));
        ClockCaps{rate_bits, src_bits}
    }

    pub fn get_rate_entries(&self) -> Vec<ClockRate> {
        Self::RATES.iter()
            .filter(|&r| r.is_supported(self))
            .copied()
            .collect()
    }

    pub fn get_src_entries(&self, labels: &ClockSourceLabels) -> Vec<ClockSource> {
        Self::SRCS.iter()
            .filter(|s| s.is_supported(self, labels))
            .copied()
            .collect()
    }
}

impl From<u32> for ClockCaps {
    fn from(val: u32) -> Self {
        let rate_bits = ((val & Self::RATE_MASK) >> Self::RATE_SHIFT) as u16;
        let src_bits = ((val & Self::SRC_MASK) >> Self::SRC_SHIFT) as u16;
        ClockCaps{rate_bits, src_bits}
    }
}

/// The structure with labels for source of sampling clock.
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct ClockSourceLabels{
    entries: Vec<String>,
}

/// The maximum size of nickname in bytes.
pub const NICKNAME_MAX_SIZE: usize = 64;

/// The trait for global protocol.
pub trait GlobalSectionProtocol<T> : GeneralProtocol<T>
    where T: AsRef<FwNode>,
{
    const OWNER_OFFSET: usize = 0x00;
    const LATEST_NOTIFICATION_OFFSET: usize = 0x04;
    const NICKNAME_OFFSET: usize = 0x0c;
    const CLK_SELECT_OFFSET: usize = 0x4c;
    const ENABLED_OFFSET: usize = 0x50;
    const STATUS_OFFSET: usize = 0x54;
    const CLK_SRC_STATES_OFFSET: usize = 0x58;
    const CURRENT_RATE_OFFSET: usize = 0x5c;
    const VERSION_OFFSET: usize = 0x60;
    const CLK_CAPS_OFFSET: usize = 0x64;
    const CLK_NAMES_OFFSET: usize = 0x68;

    const CLK_NAMES_MAX_SIZE: usize = 256;

    fn read_owner_addr(&self, node: &T, sections: &GeneralSections, timeout_ms: u32)
        -> Result<u64, Error>
    {
        let mut data = [0;8];
        self.read(node, sections.global.offset + Self::OWNER_OFFSET, &mut data, timeout_ms)
            .map_err(|e| Error::new(GeneralProtocolError::Global, &e.to_string()))
            .map(|_| {
                let mut quadlet = [0;4];
                quadlet.copy_from_slice(&data[..4]);
                let mut addr = (u32::from_be_bytes(quadlet) as u64) << 32;

                quadlet.copy_from_slice(&data[4..8]);
                addr |= u32::from_be_bytes(quadlet) as u64;

                addr
            })
    }

    fn read_latest_notification(&self, node: &T, sections: &GeneralSections, timeout_ms: u32)
        -> Result<u32, Error>
    {
        let mut data = [0;4];
        self.read(node, sections.global.offset + Self::LATEST_NOTIFICATION_OFFSET, &mut data, timeout_ms)
            .map_err(|e| Error::new(GeneralProtocolError::Global, &e.to_string()))
            .map(|_| u32::from_be_bytes(data))
    }

    fn write_nickname<N>(&self, node: &T, sections: &GeneralSections, name: N, timeout_ms: u32)
        -> Result<(), Error>
        where N: AsRef<str>
    {
        let mut data = build_label(name, NICKNAME_MAX_SIZE);
        self.write(node, sections.global.offset + Self::NICKNAME_OFFSET, &mut data, timeout_ms)
            .map_err(|e| Error::new(GeneralProtocolError::Global, &e.to_string()))
    }

    fn read_nickname(&self, node: &T, sections: &GeneralSections, timeout_ms: u32)
        -> Result<String, Error>
    {
        let mut data = vec![0;NICKNAME_MAX_SIZE];
        self.read(node, sections.global.offset + Self::NICKNAME_OFFSET, &mut data, timeout_ms)
            .map_err(|e| Error::new(GeneralProtocolError::Global, &e.to_string()))
            .and_then(|_| {
                parse_label(&data[..])
                    .map_err(|e| {
                        let msg = format!("Fail to parse data for string: {}", e);
                        Error::new(GeneralProtocolError::Global, &msg)
                    })
            })
    }

    fn write_clock_config(&self, node: &T, sections: &GeneralSections, config: ClockConfig,
                          timeout_ms: u32)
        -> Result<(), Error>
    {
        let mut data = [0;4];
        let val = u32::from(config);
        data.copy_from_slice(&val.to_be_bytes());
        self.write(node, sections.global.offset + Self::CLK_SELECT_OFFSET, &mut data, timeout_ms)
            .map_err(|e| Error::new(GeneralProtocolError::Global, &e.to_string()))
    }

    fn read_clock_config(&self, node: &T, sections: &GeneralSections, timeout_ms: u32)
        -> Result<ClockConfig, Error>
    {
        let mut data = [0;4];
        self.read(node, sections.global.offset + Self::CLK_SELECT_OFFSET, &mut data, timeout_ms)
            .map_err(|e| Error::new(GeneralProtocolError::Global, &e.to_string()))
            .map(|_| ClockConfig::from(u32::from_be_bytes(data)))
    }

    fn read_enabled(&self, node: &T, sections: &GeneralSections, timeout_ms: u32)
        -> Result<bool, Error>
    {
        let mut data = [0;4];
        self.read(node, sections.global.offset + Self::ENABLED_OFFSET, &mut data, timeout_ms)
            .map_err(|e| Error::new(GeneralProtocolError::Global, &e.to_string()))
            .map(|_| u32::from_be_bytes(data) > 0)
    }

    fn read_clock_status(&self, node: &T, sections: &GeneralSections, timeout_ms: u32)
        -> Result<ClockStatus, Error>
    {
        let mut data = [0;4];
        self.read(node, sections.global.offset + Self::STATUS_OFFSET, &mut data, timeout_ms)
            .map_err(|e| Error::new(GeneralProtocolError::Global, &e.to_string()))
            .map(|_| ClockStatus::from(u32::from_be_bytes(data)))
    }

    fn read_clock_source_states(&self, node: &T, sections: &GeneralSections, timeout_ms: u32)
        -> Result<ExtSourceStates, Error>
    {
        let mut data = [0;4];
        self.read(node, sections.global.offset + Self::CLK_SRC_STATES_OFFSET, &mut data, timeout_ms)
            .map_err(|e| Error::new(GeneralProtocolError::Global, &e.to_string()))
            .map(|_| ExtSourceStates::from(u32::from_be_bytes(data)))
    }

    fn read_current_rate(&self, node: &T, sections: &GeneralSections, timeout_ms: u32)
        -> Result<u32, Error>
    {
        let mut data = [0;4];
        self.read(node, sections.global.offset + Self::CURRENT_RATE_OFFSET, &mut data, timeout_ms)
            .map_err(|e| Error::new(GeneralProtocolError::Global, &e.to_string()))
            .map(|_| u32::from_be_bytes(data))
    }

    fn read_version(&self, node: &T, sections: &GeneralSections, timeout_ms: u32)
        -> Result<u32, Error>
    {
        let mut data = [0;4];
        self.read(node, sections.global.offset + Self::VERSION_OFFSET, &mut data, timeout_ms)
            .map_err(|e| Error::new(GeneralProtocolError::Global, &e.to_string()))
            .map(|_| u32::from_be_bytes(data))
    }

    fn read_clock_caps(&self, node: &T, sections: &GeneralSections, timeout_ms: u32)
        -> Result<ClockCaps, Error>
    {
        if sections.global.size > Self::CLK_CAPS_OFFSET {
            let mut data = [0;4];
            self.read(node, sections.global.offset + Self::CLK_CAPS_OFFSET, &mut data, timeout_ms)
                .map_err(|e| Error::new(GeneralProtocolError::Global, &e.to_string()))
                .map(|_| ClockCaps::from(u32::from_be_bytes(data)))
        } else {
            let caps = ClockCaps::new(&[ClockRate::R44100, ClockRate::R48000], &[ClockSource::Internal]);
            Ok(caps)
        }
    }

    fn read_clock_source_labels(&self, node: &T, sections: &GeneralSections, timeout_ms: u32)
        -> Result<ClockSourceLabels, Error>
    {
        if sections.global.size > Self::CLK_NAMES_MAX_SIZE {
            let mut data = vec![0;Self::CLK_NAMES_MAX_SIZE];
            self.read(node, sections.global.offset + Self::CLK_NAMES_OFFSET, &mut data, timeout_ms)
                .map_err(|e| Error::new(GeneralProtocolError::Global, &e.to_string()))
                .and_then(|_| {
                    parse_labels(&data[..])
                        .map_err(|e| {
                            let msg = format!("Fail to parse data for strings: {}", e);
                            Error::new(GeneralProtocolError::Global, &msg)
                        })
                        .map(|entries| ClockSourceLabels{entries})
                })
        } else {
            let mut entries = vec!["".to_string();12];
            entries.push("Internal".to_string());
            Ok(ClockSourceLabels{entries})
        }
    }
}

impl<O: AsRef<FwReq>, T: AsRef<FwNode>> GlobalSectionProtocol<T> for O {}
