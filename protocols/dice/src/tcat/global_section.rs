// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Global section in general protocol defined by TCAT for ASICs of DICE.
//!
//! The module includes structure, enumeration, and trait and its implementation for global section
//! in general protocol defined by TCAT for ASICs of DICE.
use super::*;

/// Nominal sampling rate.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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
}

impl Default for ClockRate {
    fn default() -> Self {
        ClockRate::Reserved(0xff)
    }
}

pub(crate) fn serialize_clock_rate(rate: &ClockRate, val: &mut u8) -> Result<(), String> {
    *val = match *rate {
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
    };

    Ok(())
}

pub(crate) fn deserialize_clock_rate(rate: &mut ClockRate, val: &u8) -> Result<(), String> {
    *rate = match *val {
        ClockRate::R32000_VAL => ClockRate::R32000,
        ClockRate::R44100_VAL => ClockRate::R44100,
        ClockRate::R48000_VAL => ClockRate::R48000,
        ClockRate::R88200_VAL => ClockRate::R88200,
        ClockRate::R96000_VAL => ClockRate::R96000,
        ClockRate::R176400_VAL => ClockRate::R176400,
        ClockRate::R192000_VAL => ClockRate::R192000,
        ClockRate::ANY_LOW_VAL => ClockRate::AnyLow,
        ClockRate::ANY_MID_VAL => ClockRate::AnyMid,
        ClockRate::ANY_HIGH_VAL => ClockRate::AnyHigh,
        ClockRate::NONE_VAL => ClockRate::None,
        _ => ClockRate::Reserved(*val),
    };

    Ok(())
}

/// Nominal sampling rate.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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
}

pub(crate) fn serialize_clock_source(src: &ClockSource, val: &mut u8) -> Result<(), String> {
    *val = match *src {
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
        ClockSource::Reserved(src_val) => src_val,
    };

    Ok(())
}

pub(crate) fn deserialize_clock_source(src: &mut ClockSource, val: &u8) -> Result<(), String> {
    *src = match *val {
        ClockSource::AES1_VAL => ClockSource::Aes1,
        ClockSource::AES2_VAL => ClockSource::Aes2,
        ClockSource::AES3_VAL => ClockSource::Aes3,
        ClockSource::AES4_VAL => ClockSource::Aes4,
        ClockSource::AES_ANY_VAL => ClockSource::AesAny,
        ClockSource::ADAT_VAL => ClockSource::Adat,
        ClockSource::TDIF_VAL => ClockSource::Tdif,
        ClockSource::WORD_CLOCK_VAL => ClockSource::WordClock,
        ClockSource::ARX1_VAL => ClockSource::Arx1,
        ClockSource::ARX2_VAL => ClockSource::Arx2,
        ClockSource::ARX3_VAL => ClockSource::Arx3,
        ClockSource::ARX4_VAL => ClockSource::Arx4,
        ClockSource::INTERNAL_VAL => ClockSource::Internal,
        _ => ClockSource::Reserved(*val),
    };
    Ok(())
}

/// Configuration of clock.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct ClockConfig {
    /// For frequency of media clock.
    pub rate: ClockRate,
    /// For signal source of sampling clock.
    pub src: ClockSource,
}

impl ClockConfig {
    const SIZE: usize = 4;

    const SRC_MASK: u32 = 0x000000ff;
    const SRC_SHIFT: usize = 0;
    const RATE_MASK: u32 = 0x0000ff00;
    const RATE_SHIFT: usize = 8;
}

fn serialize_clock_config(config: &ClockConfig, raw: &mut [u8]) -> Result<(), String> {
    assert!(raw.len() >= ClockConfig::SIZE);

    let mut src_val = 0u8;
    serialize_clock_source(&config.src, &mut src_val)?;

    let mut rate_val = 0u8;
    serialize_clock_rate(&config.rate, &mut rate_val)?;

    let val = (((rate_val as u32) << ClockConfig::RATE_SHIFT) & ClockConfig::RATE_MASK)
        | (((src_val as u32) << ClockConfig::SRC_SHIFT) & ClockConfig::SRC_MASK);

    serialize_u32(&val, raw);

    Ok(())
}

fn deserialize_clock_config(config: &mut ClockConfig, raw: &[u8]) -> Result<(), String> {
    assert!(raw.len() >= ClockConfig::SIZE);

    let mut val = 0u32;
    deserialize_u32(&mut val, raw);

    let src_val = ((val & ClockConfig::SRC_MASK) >> ClockConfig::SRC_SHIFT) as u8;
    let rate_val = ((val & ClockConfig::RATE_MASK) >> ClockConfig::RATE_SHIFT) as u8;

    deserialize_clock_source(&mut config.src, &src_val)?;
    deserialize_clock_rate(&mut config.rate, &rate_val)?;

    Ok(())
}

/// Status of sampling clock.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct ClockStatus {
    /// Whether the current clock source is locked.
    pub src_is_locked: bool,
    /// The detected frequency of media clock.
    pub rate: ClockRate,
}

impl ClockStatus {
    const SIZE: usize = 4;

    const SRC_LOCKED: u32 = 0x00000001;
    const RATE_MASK: u32 = 0x0000ff00;
    const RATE_SHIFT: usize = 8;
}

fn deserialize_clock_status(status: &mut ClockStatus, raw: &[u8]) -> Result<(), String> {
    assert!(raw.len() >= ClockStatus::SIZE);

    let mut val = 0u32;
    deserialize_u32(&mut val, raw);

    status.src_is_locked = (val & ClockStatus::SRC_LOCKED) > 0;

    let rate_val = ((val & ClockStatus::RATE_MASK) >> ClockStatus::RATE_SHIFT) as u8;
    deserialize_clock_rate(&mut status.rate, &rate_val)?;

    Ok(())
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

const CLOCK_CAPS_RATE_TABLE: [ClockRate; 11] = [
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

const CLOCK_CAPS_SRC_TABLE: [ClockSource; 13] = [
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

const EXTERNAL_CLOCK_SOURCE_TABLE: [ClockSource; 11] = [
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

impl<O: TcatOperation + TcatGlobalSectionSpecification> TcatSectionSerdes<GlobalParameters> for O {
    const MIN_SIZE: usize = 96;

    const ERROR_TYPE: GeneralProtocolError = GeneralProtocolError::Global;

    fn serialize(params: &GlobalParameters, raw: &mut [u8]) -> Result<(), String> {
        // NOTE: The owner field is changed by ALSA dice driver.

        raw[12..76].fill(0x00);
        serialize_label(&params.nickname, &mut raw[12..76])?;

        serialize_clock_config(&params.clock_config, &mut raw[76..80])?;

        Ok(())
    }

    fn deserialize(params: &mut GlobalParameters, raw: &[u8]) -> Result<(), String> {
        let mut val = 0u32;

        // NOTE: the global section was extended in later version of protocol. Evaluate the
        // extended fields at first.
        let (version, avail_rates, avail_srcs, src_labels) = if raw.len() > 96 {
            let mut labels = Vec::new();
            deserialize_labels(&mut labels, &raw[104..360])?;

            let mut labels: Vec<(ClockSource, String)> = Self::CLOCK_SOURCE_LABEL_TABLE
                .iter()
                .cloned()
                .zip(labels.into_iter())
                .collect();

            deserialize_u32(&mut val, &raw[100..104]);
            let rate_bits = (val & 0x0000ffff) as u16;
            let src_bits = ((val & 0xffff0000) >> 16) as u16;

            let avail_rates = CLOCK_CAPS_RATE_TABLE
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
                    CLOCK_CAPS_SRC_TABLE
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
                CLOCK_CAPS_SRC_TABLE
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

            deserialize_u32(&mut val, &raw[96..100]);
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

        deserialize_u32(&mut val, &raw[..4]);
        params.owner = (val as u64) << 32;
        deserialize_u32(&mut val, &raw[4..8]);
        params.owner |= val as u64;

        deserialize_u32(&mut params.latest_notification, &raw[8..12]);

        deserialize_label(&mut params.nickname, &raw[12..76])?;

        deserialize_clock_config(&mut params.clock_config, &raw[76..80])?;

        deserialize_bool(&mut params.enable, &raw[80..84]);

        deserialize_clock_status(&mut params.clock_status, &raw[84..88])?;

        deserialize_u32(&mut val, &raw[88..92]);
        let locked_bits = (val & 0x0000ffff) as u16;
        let slipped_bits = ((val & 0xffff0000) >> 16) as u16;

        let srcs: Vec<ClockSource> = EXTERNAL_CLOCK_SOURCE_TABLE
            .iter()
            .filter(|src| src_labels.iter().find(|(s, _)| src.eq(&s)).is_some())
            .copied()
            .collect();
        let locked: Vec<bool> = EXTERNAL_CLOCK_SOURCE_TABLE
            .iter()
            .enumerate()
            .filter(|(_, src)| srcs.iter().find(|s| src.eq(s)).is_some())
            .map(|(i, _)| locked_bits & (1 << i) > 0)
            .collect();
        let slipped: Vec<bool> = EXTERNAL_CLOCK_SOURCE_TABLE
            .iter()
            .enumerate()
            .filter(|(_, src)| srcs.iter().find(|s| src.eq(s)).is_some())
            .map(|(i, _)| slipped_bits & (1 << i) > 0)
            .collect();

        params.external_source_states.sources = srcs;
        params.external_source_states.locked = locked;
        params.external_source_states.slipped = slipped;

        deserialize_u32(&mut params.current_rate, &raw[92..96]);

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
