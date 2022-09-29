// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Global section in general protocol defined by TCAT for ASICs of DICE.
//!
//! The module includes structure, enumeration, and trait and its implementation for global section
//! in general protocol defined by TCAT for ASICs of DICE.
use super::{utils::*, *};

/// Nominal sampling rate.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ClockRate {
    /// 32.0 kHz.
    R32000,
    /// 44.1 kHz.
    R44100,
    /// 48.0 kHz.
    R48000,
    /// 88.2 kHz.
    R88200,
    /// 96.0 kHz.
    R96000,
    /// 176.4 kHz.
    R176400,
    /// 192.0 kHz.
    R192000,
    /// Smaller than 48.0 kHz.
    AnyLow,
    /// Between 48.0 and 96.0 kHz.
    AnyMid,
    /// Larger than 96.0 kHz.
    AnyHigh,
    /// Not available.
    None,
    /// Unspecified
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
            }
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

/// Nominal sampling rate.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ClockSource {
    /// IEC 60958 receiver 0.
    Aes1,
    /// IEC 60958 receiver 1.
    Aes2,
    /// IEC 60958 receiver 2.
    Aes3,
    /// IEC 60958 receiver 3.
    Aes4,
    /// Any IEC 60958 receiver.
    AesAny,
    /// ADAT receiver.
    Adat,
    /// TDIF receiver.
    Tdif,
    /// Word clock.
    WordClock,
    /// Audio Video System Receiver 0.
    Arx1,
    /// Audio Video System Receiver 1.
    Arx2,
    /// Audio Video System Receiver 2.
    Arx3,
    /// Audio Video System Receiver 3.
    Arx4,
    /// Internal oscillator.
    Internal,
    /// Unspecified.
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
        (caps.src_bits & (1u16 << idx) > 0)
            && labels
                .entries
                .iter()
                .nth(idx)
                .filter(|&l| l != "Unused" && l != "unused")
                .is_some()
    }

    pub fn get_label<'a>(&self, labels: &'a ClockSourceLabels, is_ext: bool) -> Option<&'a str> {
        if is_ext && *self == ClockSource::Arx1 {
            Some("Stream")
        } else {
            let idx = u8::from(*self) as usize;
            labels.entries.iter().nth(idx).map(|l| l.as_str())
        }
    }

    fn parse(src: ClockSource, flags: u16) -> bool {
        ExtSourceStates::SRCS
            .iter()
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
            Self::Internal => "Internal".to_string(),
            Self::Reserved(val) => format!("Reserved({})", val),
        };
        write!(f, "{}", label)
    }
}

/// Configuration of clock.
#[derive(Default, Debug, Clone, Copy, Eq, PartialEq)]
pub struct ClockConfig {
    /// For frequency of media clock.
    pub rate: ClockRate,
    /// For signal source of sampling clock.
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
        ClockConfig { rate, src }
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

/// Status of sampling clock.
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct ClockStatus {
    /// Whether the current clock source is locked.
    pub src_is_locked: bool,
    /// The detected frequency of media clock.
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
        ClockStatus {
            src_is_locked,
            rate,
        }
    }
}

/// States of available clock source.
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct ExtSourceStates {
    locked_bits: u16,
    slipped_bits: u16,
}

impl ExtSourceStates {
    const LOCKED_BITS_MASK: u32 = 0x0000ffff;
    const LOCKED_BITS_SHIFT: usize = 0;
    const SLIPPED_BITS_MASK: u32 = 0xffff0000;
    const SLIPPED_BITS_SHIFT: usize = 16;

    const SRCS: [ClockSource; 11] = [
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
        Self::SRCS
            .iter()
            .filter(|&s| s.is_supported(caps, labels) || *s == ClockSource::Arx1)
            .copied()
            .collect()
    }
}

impl From<u32> for ExtSourceStates {
    fn from(val: u32) -> Self {
        let locked_bits = ((val & Self::LOCKED_BITS_MASK) >> Self::LOCKED_BITS_SHIFT) as u16;
        let slipped_bits = ((val & Self::SLIPPED_BITS_MASK) >> Self::SLIPPED_BITS_SHIFT) as u16;
        ExtSourceStates {
            locked_bits,
            slipped_bits,
        }
    }
}

/// Capabilities of clock configurations.
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct ClockCaps {
    pub rate_bits: u16,
    pub src_bits: u16,
}

impl ClockCaps {
    const RATE_MASK: u32 = 0x0000ffff;
    const RATE_SHIFT: usize = 0;
    const SRC_MASK: u32 = 0xffff0000;
    const SRC_SHIFT: usize = 16;

    const RATES: [ClockRate; 11] = [
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

    const SRCS: [ClockSource; 13] = [
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
        let rate_bits = rates
            .iter()
            .fold(0, |val, &r| val | (1 << u8::from(r) as u16));
        let src_bits = srcs
            .iter()
            .fold(0, |val, &s| val | (1 << u8::from(s) as u16));
        ClockCaps {
            rate_bits,
            src_bits,
        }
    }

    pub fn get_rate_entries(&self) -> Vec<ClockRate> {
        Self::RATES
            .iter()
            .filter(|&r| r.is_supported(self))
            .copied()
            .collect()
    }

    pub fn get_src_entries(&self, labels: &ClockSourceLabels) -> Vec<ClockSource> {
        Self::SRCS
            .iter()
            .filter(|s| s.is_supported(self, labels))
            .copied()
            .collect()
    }
}

impl From<u32> for ClockCaps {
    fn from(val: u32) -> Self {
        let rate_bits = ((val & Self::RATE_MASK) >> Self::RATE_SHIFT) as u16;
        let src_bits = ((val & Self::SRC_MASK) >> Self::SRC_SHIFT) as u16;
        ClockCaps {
            rate_bits,
            src_bits,
        }
    }
}

/// Labels for source of sampling clock.
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct ClockSourceLabels {
    pub entries: Vec<String>,
}

/// The states of external signal sources of sampling clock.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct ExternalSourceStates {
    /// The list of external sources. Internal clock source is always excluded.
    pub sources: Vec<ClockSource>,
    /// Whether to lock the corresponding source or not. Any change is notified.
    pub locked: Vec<bool>,
    /// Whether to detect slipped for the sorresponding source since the last read operation. Any
    /// change is not notified.
    pub slipped: Vec<bool>,
}

/// Parameters in global section.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct GlobalParameters {
    /// The address offset of owner node to which Any notification is sent.
    pub owner: u64,
    /// The latest notification sent to the owner node.
    pub latest_notification: u32,
    /// Nickname of the node. When the same units are in the same bus, it is available to
    /// distinguish them.
    pub nickname: String,
    /// The configuration of media clock and sampling clock.
    pub clock_config: ClockConfig,
    /// Whether to enable packet streaming or not.
    pub enable: bool,
    /// The status of sampling clock.
    pub clock_status: ClockStatus,
    /// The states of external clock sources. Any change is notified by NOTIFY_EXT_STATUS.
    pub external_source_states: ExternalSourceStates,
    /// Detected rate of sampling clock in Hz.
    pub current_rate: u32,
    /// The version of protocol.
    pub version: u32,
    /// The list of available rates for media clock.
    pub avail_rates: Vec<ClockRate>,
    /// The list of available sources for sampling clock.
    pub avail_sources: Vec<ClockSource>,
    /// The list of external signal names for source of sampling clock.
    pub clock_source_labels: Vec<(ClockSource, String)>,
}

/// The specification for parameters in global section.
pub trait TcatGlobalSectionSpecification {
    /// Some models report invalid list for signal source of sampling clock.
    const AVAILABLE_CLOCK_SOURCE_OVERRIDE: Option<&'static [ClockSource]> = None;

    /// Some models report list of labels for signal source of sampling clock with unexpected
    /// position.
    const CLOCK_SOURCE_LABEL_TABLE: &'static [ClockSource] = &[
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
}

const CLOCK_SOURCE_STREAM_LABEL_TABLE: [(ClockSource, &'static str); 4] = [
    (ClockSource::Arx1, "Stream-1"),
    (ClockSource::Arx2, "Stream-2"),
    (ClockSource::Arx3, "Stream-3"),
    (ClockSource::Arx4, "Stream-4"),
];

impl<O: TcatOperation + TcatGlobalSectionSpecification> TcatSectionSerdes<GlobalParameters> for O {
    const MIN_SIZE: usize = 96;

    const ERROR_TYPE: GeneralProtocolError = GeneralProtocolError::Global;

    fn serialize(params: &GlobalParameters, raw: &mut [u8]) -> Result<(), String> {
        // NOTE: The owner field is changed by ALSA dice driver.

        raw[12..76].fill(0x00);
        let data = build_label(&params.nickname, NICKNAME_MAX_SIZE);
        raw[12..(12 + data.len())].copy_from_slice(&data);

        params.clock_config.build_quadlet(&mut raw[76..80]);

        // NOTE: The enable field is changed by ALSA dice driver.

        Ok(())
    }

    fn deserialize(params: &mut GlobalParameters, raw: &[u8]) -> Result<(), String> {
        let mut val = 0u32;

        // NOTE: the global section was extended in later version of protocol. Evaluate the
        // extended fields at first.
        let (version, avail_rates, avail_srcs, src_labels) = if raw.len() > 96 {
            let mut labels: Vec<(ClockSource, String)> = parse_labels(&raw[104..360])
                .map_err(|err| format!("Fail to parse clock source labels: {}", err))
                .map(|labels| {
                    Self::CLOCK_SOURCE_LABEL_TABLE
                        .iter()
                        .cloned()
                        .zip(labels.into_iter())
                        .collect()
                })?;

            val.parse_quadlet(&raw[100..104]);
            let rate_bits = (val & 0x0000ffff) as u16;
            let src_bits = ((val & 0xffff0000) >> 16) as u16;

            let avail_rates = ClockCaps::RATES
                .iter()
                .enumerate()
                .filter(|(i, _)| rate_bits & (1 << i) > 0)
                .map(|(_, &r)| r)
                .collect();

            // NOTE: The labels for stream sources are always "unused", while they are available
            // for external source states. Let us generate corresponding labels here for
            // applications if available.
            labels
                .iter_mut()
                .filter(|(src, _)| {
                    ClockCaps::SRCS
                        .iter()
                        .enumerate()
                        .find(|(i, s)| src.eq(s) && (src_bits & (1 << i)) > 0)
                        .is_some()
                })
                .for_each(|(src, label)| {
                    CLOCK_SOURCE_STREAM_LABEL_TABLE
                        .iter()
                        .find(|(s, _)| src.eq(&s))
                        .map(|(_, l)| *label = l.to_string());
                });

            // NOTE: Some devices report wrong bits for available clock sources. Let us use
            // hard-coded list alternatively.
            let avail_srcs = if let Some(table) = Self::AVAILABLE_CLOCK_SOURCE_OVERRIDE {
                table.to_vec()
            } else {
                ClockCaps::SRCS
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| src_bits & (1 << i) > 0)
                    .filter(|(_, src)| {
                        // NOTE: The stream sources are always detectable, thus no need to be
                        // selectable.
                        CLOCK_SOURCE_STREAM_LABEL_TABLE
                            .iter()
                            .find(|(s, _)| src.eq(&s))
                            .is_none()
                    })
                    .filter(|(_, src)| {
                        labels
                            .iter()
                            .filter(|(_, label)| label.to_lowercase() != "unused")
                            .find(|(s, _)| s.eq(&src))
                            .is_some()
                    })
                    .map(|(_, &src)| src)
                    .collect()
            };

            labels.retain(|(src, label)| {
                label.to_lowercase() != "unused"
                    && (CLOCK_SOURCE_STREAM_LABEL_TABLE
                        .iter()
                        .find(|(s, _)| src.eq(&s))
                        .is_some()
                        || avail_srcs.iter().find(|s| src.eq(s)).is_some())
            });

            val.parse_quadlet(&raw[96..100]);
            let version = val;

            (version, avail_rates, avail_srcs, labels)
        } else {
            let src_labels = vec![
                (ClockSource::Arx1, "Stream-1".to_string()),
                (ClockSource::Internal, "internal".to_string()),
            ];
            let avail_rates = vec![ClockRate::R44100, ClockRate::R48000];
            let avail_srcs = vec![ClockSource::Internal];
            let version = 0;

            (version, avail_rates, avail_srcs, src_labels)
        };

        val.parse_quadlet(&raw[..4]);
        params.owner = (val as u64) << 32;
        val.parse_quadlet(&raw[4..8]);
        params.owner |= val as u64;

        params.latest_notification.parse_quadlet(&raw[8..12]);

        params.nickname =
            parse_label(&raw[12..76]).map_err(|err| format!("Fail to parse nickname: {}", err))?;

        val.parse_quadlet(&raw[76..80]);
        params.clock_config = ClockConfig::from(val);

        params.enable.parse_quadlet(&raw[80..84]);

        val.parse_quadlet(&raw[84..88]);
        params.clock_status = ClockStatus::from(val);

        val.parse_quadlet(&raw[88..92]);
        let locked_bits = (val & 0x0000ffff) as u16;
        let slipped_bits = ((val & 0xffff0000) >> 16) as u16;

        let srcs: Vec<ClockSource> = ExtSourceStates::SRCS
            .iter()
            .filter(|src| src_labels.iter().find(|(s, _)| src.eq(&s)).is_some())
            .copied()
            .collect();
        let locked: Vec<bool> = ExtSourceStates::SRCS
            .iter()
            .enumerate()
            .filter(|(_, src)| srcs.iter().find(|s| src.eq(s)).is_some())
            .map(|(i, _)| locked_bits & (1 << i) > 0)
            .collect();
        let slipped: Vec<bool> = ExtSourceStates::SRCS
            .iter()
            .enumerate()
            .filter(|(_, src)| srcs.iter().find(|s| src.eq(s)).is_some())
            .map(|(i, _)| slipped_bits & (1 << i) > 0)
            .collect();

        params.external_source_states.sources = srcs;
        params.external_source_states.locked = locked;
        params.external_source_states.slipped = slipped;

        params.current_rate.parse_quadlet(&raw[92..96]);

        params.version = version;
        params.avail_rates = avail_rates;
        params.avail_sources = avail_srcs;
        params.clock_source_labels = src_labels;

        Ok(())
    }
}

impl<O: TcatOperation + TcatSectionSerdes<GlobalParameters>> TcatSectionOperation<GlobalParameters>
    for O
{
}

impl<O: TcatOperation + TcatSectionSerdes<GlobalParameters>>
    TcatMutableSectionOperation<GlobalParameters> for O
{
}

impl<O: TcatSectionOperation<GlobalParameters>> TcatNotifiedSectionOperation<GlobalParameters>
    for O
{
    const NOTIFY_FLAG: u32 = NOTIFY_LOCK_CHG | NOTIFY_CLOCK_ACCEPTED | NOTIFY_EXT_STATUS;
}

impl<O: TcatSectionOperation<GlobalParameters>> TcatFluctuatedSectionOperation<GlobalParameters>
    for O
{
    const FLUCTUATED_OFFSETS: &'static [usize] = &[
        88, // The slipped bits in GLOBAL_EXTENDED_STATUS are fluctuated without any notification.
    ];
}

/// The maximum size of nickname in bytes.
pub const NICKNAME_MAX_SIZE: usize = 64;

/// Protocol implementaion of global section.
pub struct GlobalSectionProtocol;

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

impl GlobalSectionProtocol {
    const CLK_NAMES_MAX_SIZE: usize = 256;

    pub fn read_owner_addr(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &GeneralSections,
        timeout_ms: u32,
    ) -> Result<u64, Error> {
        let mut data = [0; 8];
        GeneralProtocol::read(
            req,
            node,
            sections.global.offset + OWNER_OFFSET,
            &mut data,
            timeout_ms,
        )
        .map_err(|e| Error::new(GeneralProtocolError::Global, &e.to_string()))
        .map(|_| {
            let mut quadlet = [0; 4];
            quadlet.copy_from_slice(&data[..4]);
            let mut addr = (u32::from_be_bytes(quadlet) as u64) << 32;

            quadlet.copy_from_slice(&data[4..8]);
            addr |= u32::from_be_bytes(quadlet) as u64;

            addr
        })
    }

    pub fn read_latest_notification(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &GeneralSections,
        timeout_ms: u32,
    ) -> Result<u32, Error> {
        let mut data = [0; 4];
        GeneralProtocol::read(
            req,
            node,
            sections.global.offset + LATEST_NOTIFICATION_OFFSET,
            &mut data,
            timeout_ms,
        )
        .map_err(|e| Error::new(GeneralProtocolError::Global, &e.to_string()))
        .map(|_| u32::from_be_bytes(data))
    }

    pub fn write_nickname<N>(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &GeneralSections,
        name: N,
        timeout_ms: u32,
    ) -> Result<(), Error>
    where
        N: AsRef<str>,
    {
        let mut data = build_label(name, NICKNAME_MAX_SIZE);
        GeneralProtocol::write(
            req,
            node,
            sections.global.offset + NICKNAME_OFFSET,
            &mut data,
            timeout_ms,
        )
        .map_err(|e| Error::new(GeneralProtocolError::Global, &e.to_string()))
    }

    pub fn read_nickname(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &GeneralSections,
        timeout_ms: u32,
    ) -> Result<String, Error> {
        let mut data = vec![0; NICKNAME_MAX_SIZE];
        GeneralProtocol::read(
            req,
            node,
            sections.global.offset + NICKNAME_OFFSET,
            &mut data,
            timeout_ms,
        )
        .map_err(|e| Error::new(GeneralProtocolError::Global, &e.to_string()))
        .and_then(|_| {
            parse_label(&data[..]).map_err(|e| {
                let msg = format!("Fail to parse data for string: {}", e);
                Error::new(GeneralProtocolError::Global, &msg)
            })
        })
    }

    pub fn write_clock_config(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &GeneralSections,
        config: ClockConfig,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut data = [0; 4];
        let val = u32::from(config);
        data.copy_from_slice(&val.to_be_bytes());
        GeneralProtocol::write(
            req,
            node,
            sections.global.offset + CLK_SELECT_OFFSET,
            &mut data,
            timeout_ms,
        )
        .map_err(|e| Error::new(GeneralProtocolError::Global, &e.to_string()))
    }

    pub fn read_clock_config(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &GeneralSections,
        timeout_ms: u32,
    ) -> Result<ClockConfig, Error> {
        let mut data = [0; 4];
        GeneralProtocol::read(
            req,
            node,
            sections.global.offset + CLK_SELECT_OFFSET,
            &mut data,
            timeout_ms,
        )
        .map_err(|e| Error::new(GeneralProtocolError::Global, &e.to_string()))
        .map(|_| ClockConfig::from(u32::from_be_bytes(data)))
    }

    pub fn read_enabled(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &GeneralSections,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        let mut data = [0; 4];
        GeneralProtocol::read(
            req,
            node,
            sections.global.offset + ENABLED_OFFSET,
            &mut data,
            timeout_ms,
        )
        .map_err(|e| Error::new(GeneralProtocolError::Global, &e.to_string()))
        .map(|_| u32::from_be_bytes(data) > 0)
    }

    pub fn read_clock_status(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &GeneralSections,
        timeout_ms: u32,
    ) -> Result<ClockStatus, Error> {
        let mut data = [0; 4];
        GeneralProtocol::read(
            req,
            node,
            sections.global.offset + STATUS_OFFSET,
            &mut data,
            timeout_ms,
        )
        .map_err(|e| Error::new(GeneralProtocolError::Global, &e.to_string()))
        .map(|_| ClockStatus::from(u32::from_be_bytes(data)))
    }

    pub fn read_clock_source_states(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &GeneralSections,
        timeout_ms: u32,
    ) -> Result<ExtSourceStates, Error> {
        let mut data = [0; 4];
        GeneralProtocol::read(
            req,
            node,
            sections.global.offset + CLK_SRC_STATES_OFFSET,
            &mut data,
            timeout_ms,
        )
        .map_err(|e| Error::new(GeneralProtocolError::Global, &e.to_string()))
        .map(|_| ExtSourceStates::from(u32::from_be_bytes(data)))
    }

    pub fn read_current_rate(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &GeneralSections,
        timeout_ms: u32,
    ) -> Result<u32, Error> {
        let mut data = [0; 4];
        GeneralProtocol::read(
            req,
            node,
            sections.global.offset + CURRENT_RATE_OFFSET,
            &mut data,
            timeout_ms,
        )
        .map_err(|e| Error::new(GeneralProtocolError::Global, &e.to_string()))
        .map(|_| u32::from_be_bytes(data))
    }

    pub fn read_version(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &GeneralSections,
        timeout_ms: u32,
    ) -> Result<u32, Error> {
        let mut data = [0; 4];
        GeneralProtocol::read(
            req,
            node,
            sections.global.offset + VERSION_OFFSET,
            &mut data,
            timeout_ms,
        )
        .map_err(|e| Error::new(GeneralProtocolError::Global, &e.to_string()))
        .map(|_| u32::from_be_bytes(data))
    }

    pub fn read_clock_caps(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &GeneralSections,
        timeout_ms: u32,
    ) -> Result<ClockCaps, Error> {
        if sections.global.size > CLK_CAPS_OFFSET {
            let mut data = [0; 4];
            GeneralProtocol::read(
                req,
                node,
                sections.global.offset + CLK_CAPS_OFFSET,
                &mut data,
                timeout_ms,
            )
            .map_err(|e| Error::new(GeneralProtocolError::Global, &e.to_string()))
            .map(|_| ClockCaps::from(u32::from_be_bytes(data)))
        } else {
            let caps = ClockCaps::new(
                &[ClockRate::R44100, ClockRate::R48000],
                &[ClockSource::Internal],
            );
            Ok(caps)
        }
    }

    pub fn read_clock_source_labels(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &GeneralSections,
        timeout_ms: u32,
    ) -> Result<ClockSourceLabels, Error> {
        if sections.global.size > Self::CLK_NAMES_MAX_SIZE {
            let mut data = vec![0; Self::CLK_NAMES_MAX_SIZE];
            GeneralProtocol::read(
                req,
                node,
                sections.global.offset + CLK_NAMES_OFFSET,
                &mut data,
                timeout_ms,
            )
            .map_err(|e| Error::new(GeneralProtocolError::Global, &e.to_string()))
            .and_then(|_| {
                parse_labels(&data[..])
                    .map_err(|e| {
                        let msg = format!("Fail to parse data for strings: {}", e);
                        Error::new(GeneralProtocolError::Global, &msg)
                    })
                    .map(|entries| ClockSourceLabels { entries })
            })
        } else {
            let mut entries = vec!["".to_string(); 12];
            entries.push("Internal".to_string());
            Ok(ClockSourceLabels { entries })
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    struct Protocol;

    impl TcatOperation for Protocol {}

    impl TcatGlobalSectionSpecification for Protocol {}

    #[test]
    fn global_params_serdes() {
        let mut raw = [
            0xff, 0xc1, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x6b, 0x73,
            0x65, 0x44, 0x4b, 0x70, 0x6f, 0x74, 0x65, 0x6e, 0x6e, 0x6f, 0x00, 0x36, 0x74, 0x6b,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x0c, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x02, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xbb, 0x80, 0x01, 0x00,
            0x04, 0x00, 0x13, 0x00, 0x00, 0x7e, 0x73, 0x75, 0x6e, 0x55, 0x55, 0x5c, 0x64, 0x65,
            0x65, 0x73, 0x75, 0x6e, 0x6e, 0x55, 0x5c, 0x64, 0x64, 0x65, 0x73, 0x75, 0x75, 0x6e,
            0x55, 0x5c, 0x5c, 0x64, 0x65, 0x73, 0x73, 0x75, 0x6e, 0x55, 0x55, 0x5c, 0x64, 0x65,
            0x65, 0x73, 0x75, 0x6e, 0x6e, 0x55, 0x5c, 0x64, 0x64, 0x65, 0x73, 0x75, 0x75, 0x6e,
            0x55, 0x5c, 0x5c, 0x64, 0x65, 0x73, 0x73, 0x75, 0x6e, 0x55, 0x55, 0x5c, 0x64, 0x65,
            0x65, 0x73, 0x75, 0x6e, 0x6e, 0x55, 0x5c, 0x64, 0x64, 0x65, 0x73, 0x75, 0x75, 0x6e,
            0x55, 0x5c, 0x5c, 0x64, 0x65, 0x73, 0x45, 0x54, 0x4e, 0x49, 0x4c, 0x41, 0x4e, 0x52,
            0x00, 0x00, 0x5c, 0x5c, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        let mut params = GlobalParameters::default();
        Protocol::deserialize(&mut params, &raw).unwrap();

        assert_eq!(params.owner, 0xffc1000100000000);
        assert_eq!(params.latest_notification, 0x00000010);
        assert_eq!(params.nickname, "DesktopKonnekt6");
        assert_eq!(
            params.clock_config,
            ClockConfig {
                rate: ClockRate::R48000,
                src: ClockSource::Internal
            }
        );
        assert_eq!(params.enable, false);
        assert_eq!(
            params.clock_status,
            ClockStatus {
                src_is_locked: true,
                rate: ClockRate::R48000
            }
        );
        assert_eq!(
            params.external_source_states,
            ExternalSourceStates {
                sources: vec![ClockSource::Arx1, ClockSource::Arx2],
                locked: vec![false, false],
                slipped: vec![false, false],
            }
        );
        assert_eq!(params.current_rate, 48000);
        assert_eq!(params.version, 0x01000400);
        assert_eq!(
            params.avail_rates,
            vec![
                ClockRate::R44100,
                ClockRate::R48000,
                ClockRate::R88200,
                ClockRate::R96000,
                ClockRate::R176400,
                ClockRate::R192000
            ]
        );
        assert_eq!(params.avail_sources, vec![ClockSource::Internal]);
        assert_eq!(
            params.clock_source_labels,
            vec![
                (ClockSource::Arx1, "Stream-1".to_string()),
                (ClockSource::Arx2, "Stream-2".to_string()),
                (ClockSource::Internal, "INTERNAL".to_string()),
            ]
        );

        let mut p = params.clone();
        p.nickname = "tcat-procotol-general".to_string();
        p.clock_config = ClockConfig {
            rate: ClockRate::R192000,
            src: ClockSource::Adat,
        };
        Protocol::serialize(&p, &mut raw).unwrap();
        assert_eq!(
            raw[12..76],
            [
                0x74, 0x61, 0x63, 0x74, 0x6f, 0x72, 0x70, 0x2d, 0x6f, 0x74, 0x6f, 0x63, 0x65, 0x67,
                0x2d, 0x6c, 0x61, 0x72, 0x65, 0x6e, 0x00, 0x00, 0x00, 0x6c, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            ]
        );
        assert_eq!(raw[76..80], [00, 00, 06, 05]);
    }

    #[test]
    fn global_old_params_serdes() {
        let mut raw = [
            0xff, 0xc1, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x20, 0x73, 0x65,
            0x6c, 0x41, 0x4d, 0x20, 0x73, 0x69, 0x69, 0x74, 0x6c, 0x75, 0x00, 0x78, 0x69, 0x4d,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x0c, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x02, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xbb, 0x80,
        ];
        let mut params = GlobalParameters::default();
        Protocol::deserialize(&mut params, &raw).unwrap();

        assert_eq!(params.owner, 0xffc1000100000000);
        assert_eq!(params.latest_notification, 0x00000020);
        assert_eq!(params.nickname, "Alesis MultiMix");
        assert_eq!(
            params.clock_config,
            ClockConfig {
                rate: ClockRate::R48000,
                src: ClockSource::Internal
            }
        );
        assert_eq!(params.enable, false);
        assert_eq!(
            params.clock_status,
            ClockStatus {
                src_is_locked: true,
                rate: ClockRate::R48000
            }
        );
        assert_eq!(
            params.external_source_states,
            ExternalSourceStates {
                sources: vec![ClockSource::Arx1],
                locked: vec![false],
                slipped: vec![false],
            }
        );
        assert_eq!(params.current_rate, 48000);
        assert_eq!(params.version, 0);
        assert_eq!(
            params.avail_rates,
            vec![ClockRate::R44100, ClockRate::R48000,]
        );
        assert_eq!(params.avail_sources, vec![ClockSource::Internal]);
        assert_eq!(
            params.clock_source_labels,
            vec![
                (ClockSource::Arx1, "Stream-1".to_string()),
                (ClockSource::Internal, "internal".to_string()),
            ]
        );

        let mut p = params.clone();
        p.nickname = "tcat-procotol-general".to_string();
        p.clock_config = ClockConfig {
            rate: ClockRate::R192000,
            src: ClockSource::Adat,
        };
        Protocol::serialize(&p, &mut raw).unwrap();
        assert_eq!(
            raw[12..76],
            [
                0x74, 0x61, 0x63, 0x74, 0x6f, 0x72, 0x70, 0x2d, 0x6f, 0x74, 0x6f, 0x63, 0x65, 0x67,
                0x2d, 0x6c, 0x61, 0x72, 0x65, 0x6e, 0x00, 0x00, 0x00, 0x6c, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            ]
        );
        assert_eq!(raw[76..80], [00, 00, 06, 05]);
    }
}
