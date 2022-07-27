// SPDX-License-Identifier: MIT
// Copyright (c) 2020 Takashi Sakamoto

//! A set of minimum items in TLV (Type-Length-Value) of ALSA control interface.

use super::*;

/// The data to express dB scale in TLV (Type-Length-Value) of ALSA control interface.
///
/// It has `SNDRV_CTL_TLVT_DB_SCALE` (=1) in its type field and has two elements in value field.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct DbScale {
    /// The minimum value by dB expression, in 0.1 dB unit. This corresponds to the minimum
    /// value in the state of control element.
    pub min: i32,
    /// The step by dB expression, in 0.1 dB unit. This corresponds to one increase of the value
    /// in the state of control element.
    pub step: u16,
    /// If true, the value less than or equals to [`CTL_VALUE_MUTE`] (=-9999999) is available to
    /// mute the control element explicitly.
    pub mute_avail: bool,
}

/// When information about dB includes mute_avail, the value is available to mute the control
/// element. It's relevant to `SNDRV_CTL_TLVD_DB_GAIN_MUTE` macro in UAPI of Linux kernel.
pub const CTL_VALUE_MUTE: i32 = SNDRV_CTL_TLVD_DB_GAIN_MUTE;

/// The value of dB should be expressed in 0.1 dB unit in data of TLV and crate structures.
pub const DB_VALUE_MULTIPLIER: i32 = 100;

impl DbScale {
    const VALUE_COUNT: usize = 2;
}

impl<'a> TlvData<'a> for DbScale {
    fn value_type(&self) -> u32 {
        SNDRV_CTL_TLVT_DB_SCALE
    }

    fn value_length(&self) -> usize {
        Self::VALUE_COUNT
    }

    fn value(&self) -> Vec<u32> {
        let mut raw = Vec::new();
        raw.push(self.min as u32);
        raw.push(self.step as u32);
        if self.mute_avail {
            raw[1] |= SNDRV_CTL_TLVD_DB_SCALE_MUTE;
        }
        raw
    }
}

const TYPES_FOR_DB_SCALE: &'static [u32] = &[SNDRV_CTL_TLVT_DB_SCALE];

impl std::convert::TryFrom<&[u32]> for DbScale {
    type Error = TlvDecodeError;

    fn try_from(raw: &[u32]) -> Result<Self, Self::Error> {
        // At least, type and length field should be included.
        if raw.len() < 2 {
            Err(Self::Error::new(TlvDecodeErrorCtx::Length(raw.len(), 2), 0))
        // Check type field.
        } else if raw[0] != SNDRV_CTL_TLVT_DB_SCALE {
            Err(Self::Error::new(
                TlvDecodeErrorCtx::ValueType(raw[0], TYPES_FOR_DB_SCALE),
                0,
            ))
        } else {
            // Check length field against length of value field.
            let value_length = (raw[1] / 4) as usize;
            let value = &raw[2..];
            if value.len() < value_length {
                Err(Self::Error::new(
                    TlvDecodeErrorCtx::ValueLength(value_length, value.len()),
                    1,
                ))
            } else {
                // Decode value field.
                Ok(Self {
                    min: value[0] as i32,
                    step: (value[1] & SNDRV_CTL_TLVD_DB_SCALE_MASK) as u16,
                    mute_avail: value[1] & SNDRV_CTL_TLVD_DB_SCALE_MUTE > 0,
                })
            }
        }
    }
}

impl From<&DbScale> for Vec<u32> {
    fn from(data: &DbScale) -> Self {
        let mut raw = Vec::new();
        raw.push(data.value_type());
        raw.push(4 * data.value_length() as u32);
        raw.append(&mut data.value());
        raw
    }
}

impl From<DbScale> for Vec<u32> {
    fn from(data: DbScale) -> Self {
        (&data).into()
    }
}

/// The data to express dB interval in TLV (Type-Length-Value) of ALSA control interface.
///
/// It has three variants below;
///  * SNDRV_CTL_TLVT_DB_LINEAR(=2)
///  * SNDRV_CTL_TLVT_DB_MINMAX(=4)
///  * SNDRV_CTL_TLVT_DB_MINMAX_MUTE(=5)
///
///  All of them have two elements in value field.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct DbInterval {
    /// The minimum value by dB expression, in 0.1 dB unit. This corresponds to the minimum
    /// value in the state of control element.
    pub min: i32,
    /// The maximum value by dB expression, 0.1 dB unit. This corresponds to the maximum value
    /// in the state of control element.
    pub max: i32,
    /// If true, the value in the state of control element increases linearly, thus need calculation
    /// to convert to the value by dB expression. The calculation shall be:
    ///
    /// 20 * log10( current_value / ( maximum_value - minimum_value ) ) (* 100 in 0.1dB unit)
    ///
    /// Else, the value in the state of control element corresponds to dB expression itself.
    pub linear: bool,
    /// If true, the value less than or equals to [`CTL_VALUE_MUTE`] (=-9999999) is available to
    /// mute the control element explicitly.
    pub mute_avail: bool,
}

impl DbInterval {
    const VALUE_COUNT: usize = 2;
}

impl<'a> TlvData<'a> for DbInterval {
    fn value_type(&self) -> u32 {
        if self.linear {
            SNDRV_CTL_TLVT_DB_LINEAR
        } else if self.mute_avail {
            SNDRV_CTL_TLVT_DB_MINMAX_MUTE
        } else {
            SNDRV_CTL_TLVT_DB_MINMAX
        }
    }

    fn value_length(&self) -> usize {
        Self::VALUE_COUNT
    }

    fn value(&self) -> Vec<u32> {
        vec![self.min as u32, self.max as u32]
    }
}

const TYPES_FOR_DB_INTERVAL: &'static [u32] = &[
    SNDRV_CTL_TLVT_DB_LINEAR,
    SNDRV_CTL_TLVT_DB_MINMAX,
    SNDRV_CTL_TLVT_DB_MINMAX_MUTE,
];

impl std::convert::TryFrom<&[u32]> for DbInterval {
    type Error = TlvDecodeError;

    fn try_from(raw: &[u32]) -> Result<Self, Self::Error> {
        // At least, type and length field should be included.
        if raw.len() < 2 {
            Err(Self::Error::new(TlvDecodeErrorCtx::Length(raw.len(), 2), 0))
        } else {
            // Check length field against length of value field.
            let value_length = (raw[1] / 4) as usize;
            let value = &raw[2..];
            if value.len() < value_length || value.len() < Self::VALUE_COUNT {
                Err(Self::Error::new(
                    TlvDecodeErrorCtx::ValueLength(value_length, value.len()),
                    1,
                ))
            } else {
                // Check type field.
                match raw[0] {
                    SNDRV_CTL_TLVT_DB_LINEAR => Ok(Self {
                        min: value[0] as i32,
                        max: value[1] as i32,
                        linear: true,
                        mute_avail: true,
                    }),
                    SNDRV_CTL_TLVT_DB_MINMAX => Ok(Self {
                        min: value[0] as i32,
                        max: value[1] as i32,
                        linear: false,
                        mute_avail: false,
                    }),
                    SNDRV_CTL_TLVT_DB_MINMAX_MUTE => Ok(Self {
                        min: value[0] as i32,
                        max: value[1] as i32,
                        linear: false,
                        mute_avail: true,
                    }),
                    _ => Err(Self::Error::new(
                        TlvDecodeErrorCtx::ValueType(raw[0], TYPES_FOR_DB_INTERVAL),
                        0,
                    )),
                }
            }
        }
    }
}

impl From<&DbInterval> for Vec<u32> {
    fn from(data: &DbInterval) -> Self {
        let mut raw = Vec::new();
        raw.push(data.value_type());
        raw.push(4 * data.value_length() as u32);
        raw.append(&mut data.value());
        raw
    }
}

impl From<DbInterval> for Vec<u32> {
    fn from(data: DbInterval) -> Self {
        (&data).into()
    }
}

/// The enumeration to express generic channel position corresponding to physical port on real
/// device. They are defined as `SNDRV_CHMAP_XXX` enumeration in UAPI of Linux kernel.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ChmapGenericPos {
    Unknown,
    NotAvailable,
    Monaural,
    FrontLeft,
    FrontRight,
    RearLeft,
    RearRight,
    FrontCenter,
    LowFrequencyEffect,
    SideLeft,
    SideRight,
    RearCenter,
    FrontLeftCenter,
    FrontRightCenter,
    RearLeftCenter,
    RearRightCenter,
    FrontLeftWide,
    FrontRightWide,
    FrontLeftHigh,
    FrontCenterHigh,
    FrontRightHigh,
    TopCenter,
    TopFrontLeft,
    TopFrontRight,
    TopFrontCenter,
    TopRearLeft,
    TopRearRight,
    TopRearCenter,
    TopFrontLeftCenter,
    TopFrontRightCenter,
    TopSideLeft,
    TopSideRight,
    LeftLowFrequencyEffect,
    RightLowFrequencyEffect,
    BottomCenter,
    BottomLeftCenter,
    BottomRightCenter,
    Reserved(u16),
}

impl Default for ChmapGenericPos {
    fn default() -> Self {
        Self::Unknown
    }
}

impl std::convert::From<u16> for ChmapGenericPos {
    fn from(val: u16) -> Self {
        match val as u16 {
            SNDRV_CHMAP_UNKNOWN => Self::Unknown,
            SNDRV_CHMAP_NA => Self::NotAvailable,
            SNDRV_CHMAP_MONO => Self::Monaural,
            SNDRV_CHMAP_FL => Self::FrontLeft,
            SNDRV_CHMAP_FR => Self::FrontRight,
            SNDRV_CHMAP_RL => Self::RearLeft,
            SNDRV_CHMAP_RR => Self::RearRight,
            SNDRV_CHMAP_FC => Self::FrontCenter,
            SNDRV_CHMAP_LFE => Self::LowFrequencyEffect,
            SNDRV_CHMAP_SL => Self::SideLeft,
            SNDRV_CHMAP_SR => Self::SideRight,
            SNDRV_CHMAP_RC => Self::RearCenter,
            SNDRV_CHMAP_FLC => Self::FrontLeftCenter,
            SNDRV_CHMAP_FRC => Self::FrontRightCenter,
            SNDRV_CHMAP_RLC => Self::RearLeftCenter,
            SNDRV_CHMAP_RRC => Self::RearRightCenter,
            SNDRV_CHMAP_FLW => Self::FrontLeftWide,
            SNDRV_CHMAP_FRW => Self::FrontRightWide,
            SNDRV_CHMAP_FLH => Self::FrontLeftHigh,
            SNDRV_CHMAP_FCH => Self::FrontCenterHigh,
            SNDRV_CHMAP_FRH => Self::FrontRightHigh,
            SNDRV_CHMAP_TC => Self::TopCenter,
            SNDRV_CHMAP_TFL => Self::TopFrontLeft,
            SNDRV_CHMAP_TFR => Self::TopFrontRight,
            SNDRV_CHMAP_TFC => Self::TopFrontCenter,
            SNDRV_CHMAP_TRL => Self::TopRearLeft,
            SNDRV_CHMAP_TRR => Self::TopRearRight,
            SNDRV_CHMAP_TRC => Self::TopRearCenter,
            SNDRV_CHMAP_TFLC => Self::TopFrontLeftCenter,
            SNDRV_CHMAP_TFRC => Self::TopFrontRightCenter,
            SNDRV_CHMAP_TSL => Self::TopSideLeft,
            SNDRV_CHMAP_TSR => Self::TopSideRight,
            SNDRV_CHMAP_LLFE => Self::LeftLowFrequencyEffect,
            SNDRV_CHMAP_RLFE => Self::RightLowFrequencyEffect,
            SNDRV_CHMAP_BC => Self::BottomCenter,
            SNDRV_CHMAP_BLC => Self::BottomLeftCenter,
            SNDRV_CHMAP_BRC => Self::BottomRightCenter,
            _ => Self::Reserved(val),
        }
    }
}

impl From<ChmapGenericPos> for u16 {
    fn from(code: ChmapGenericPos) -> Self {
        match code {
            ChmapGenericPos::Unknown => SNDRV_CHMAP_UNKNOWN,
            ChmapGenericPos::NotAvailable => SNDRV_CHMAP_NA,
            ChmapGenericPos::Monaural => SNDRV_CHMAP_MONO,
            ChmapGenericPos::FrontLeft => SNDRV_CHMAP_FL,
            ChmapGenericPos::FrontRight => SNDRV_CHMAP_FR,
            ChmapGenericPos::RearLeft => SNDRV_CHMAP_RL,
            ChmapGenericPos::RearRight => SNDRV_CHMAP_RR,
            ChmapGenericPos::FrontCenter => SNDRV_CHMAP_FC,
            ChmapGenericPos::LowFrequencyEffect => SNDRV_CHMAP_LFE,
            ChmapGenericPos::SideLeft => SNDRV_CHMAP_SL,
            ChmapGenericPos::SideRight => SNDRV_CHMAP_SR,
            ChmapGenericPos::RearCenter => SNDRV_CHMAP_RC,
            ChmapGenericPos::FrontLeftCenter => SNDRV_CHMAP_FLC,
            ChmapGenericPos::FrontRightCenter => SNDRV_CHMAP_FRC,
            ChmapGenericPos::RearLeftCenter => SNDRV_CHMAP_RLC,
            ChmapGenericPos::RearRightCenter => SNDRV_CHMAP_RRC,
            ChmapGenericPos::FrontLeftWide => SNDRV_CHMAP_FLW,
            ChmapGenericPos::FrontRightWide => SNDRV_CHMAP_FRW,
            ChmapGenericPos::FrontLeftHigh => SNDRV_CHMAP_FLH,
            ChmapGenericPos::FrontCenterHigh => SNDRV_CHMAP_FCH,
            ChmapGenericPos::FrontRightHigh => SNDRV_CHMAP_FRH,
            ChmapGenericPos::TopCenter => SNDRV_CHMAP_TC,
            ChmapGenericPos::TopFrontLeft => SNDRV_CHMAP_TFL,
            ChmapGenericPos::TopFrontRight => SNDRV_CHMAP_TFR,
            ChmapGenericPos::TopFrontCenter => SNDRV_CHMAP_TFC,
            ChmapGenericPos::TopRearLeft => SNDRV_CHMAP_TRL,
            ChmapGenericPos::TopRearRight => SNDRV_CHMAP_TRR,
            ChmapGenericPos::TopRearCenter => SNDRV_CHMAP_TRC,
            ChmapGenericPos::TopFrontLeftCenter => SNDRV_CHMAP_TFLC,
            ChmapGenericPos::TopFrontRightCenter => SNDRV_CHMAP_TFRC,
            ChmapGenericPos::TopSideLeft => SNDRV_CHMAP_TSL,
            ChmapGenericPos::TopSideRight => SNDRV_CHMAP_TSR,
            ChmapGenericPos::LeftLowFrequencyEffect => SNDRV_CHMAP_LLFE,
            ChmapGenericPos::RightLowFrequencyEffect => SNDRV_CHMAP_RLFE,
            ChmapGenericPos::BottomCenter => SNDRV_CHMAP_BC,
            ChmapGenericPos::BottomLeftCenter => SNDRV_CHMAP_BLC,
            ChmapGenericPos::BottomRightCenter => SNDRV_CHMAP_BRC,
            ChmapGenericPos::Reserved(val) => val,
        }
    }
}

/// The enumeration to express channel position.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ChmapPos {
    /// The position of channel is generic. It's relevant to the series of `SNDRV_CHMAP_XXX` macro.
    Generic(ChmapGenericPos),
    /// The position of channel is specific, programmed by driver. It's relevant to
    /// `SNDRV_CHMAP_DRIVER_SPEC` macro in UAPI of Linux kernel.
    Specific(u16),
}

impl Default for ChmapPos {
    fn default() -> Self {
        Self::Generic(Default::default())
    }
}

/// The entry to express information of each channel in channel map.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct ChmapEntry {
    /// The position of channel.
    pub pos: ChmapPos,
    /// If true, phase is inverted (e.g. a microphone channel within multiple channels). It's
    /// relevant to `SNDRV_CHMAP_PHASE_INVERSE` macro in UAPI of Linux kernel.
    pub phase_inverse: bool,
}

impl std::convert::From<u32> for ChmapEntry {
    fn from(val: u32) -> Self {
        let pos_val = (val & 0x0000ffff) as u16;
        let phase_inverse = val & SNDRV_CHMAP_PHASE_INVERSE > 0;
        let driver_spec = val & SNDRV_CHMAP_DRIVER_SPEC > 0;
        let pos = if driver_spec {
            ChmapPos::Specific(pos_val)
        } else {
            ChmapPos::Generic(ChmapGenericPos::from(pos_val))
        };
        ChmapEntry { pos, phase_inverse }
    }
}

impl From<ChmapEntry> for u32 {
    fn from(entry: ChmapEntry) -> Self {
        let mut val = match entry.pos {
            ChmapPos::Generic(p) => u16::from(p) as u32,
            ChmapPos::Specific(p) => (p as u32) | SNDRV_CHMAP_DRIVER_SPEC,
        };
        if entry.phase_inverse {
            val |= SNDRV_CHMAP_PHASE_INVERSE;
        }
        val
    }
}

/// The mode for channel map.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ChmapMode {
    /// The map is fixed and no way to change. It's relevant to `SNDRV_CTL_TLVT_CHMAP_FIXED`.
    Fixed,
    /// Each entry in the map is exchangeable arbitrarily. It's relevant to
    /// `SNDRV_CTL_TLVT_CHMAP_VAR`.
    ArbitraryExchangeable,
    /// The stereo pair of entries in the map is exchangeable. It's relevant to
    /// `SNDRV_CTL_TLVT_CHMAP_PAIRED`.
    PairedExchangeable,
}

impl Default for ChmapMode {
    fn default() -> Self {
        Self::Fixed
    }
}

/// The data to express channel map of PCM substream in TLV (Type-Length-Value) of ALSA control interface.
///
/// It has three variants below;
///  * `SNDRV_CTL_TLVT_CHMAP_FIXED` (=0x101)
///  * `SNDRV_CTL_TLVT_CHMAP_VAR` (=0x102)
///  * `SNDRV_CTL_TLVT_CHMAP_PAIRED` (=0x103)
///
/// The length of value field is variable depending on the number of channels.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Chmap {
    /// The mode of map.
    pub mode: ChmapMode,
    /// The entries of map corresponding to each channel.
    pub entries: Vec<ChmapEntry>,
}

impl<'a> TlvData<'a> for Chmap {
    fn value_type(&self) -> u32 {
        match self.mode {
            ChmapMode::Fixed => SNDRV_CTL_TLVT_CHMAP_FIXED,
            ChmapMode::ArbitraryExchangeable => SNDRV_CTL_TLVT_CHMAP_VAR,
            ChmapMode::PairedExchangeable => SNDRV_CTL_TLVT_CHMAP_PAIRED,
        }
    }

    fn value_length(&self) -> usize {
        self.entries.len()
    }

    fn value(&self) -> Vec<u32> {
        let mut raw = Vec::new();
        self.entries
            .iter()
            .for_each(|&entry| raw.push(u32::from(entry)));
        raw
    }
}

const TYPES_FOR_CHMAP: &'static [u32] = &[
    SNDRV_CTL_TLVT_CHMAP_FIXED,
    SNDRV_CTL_TLVT_CHMAP_VAR,
    SNDRV_CTL_TLVT_CHMAP_PAIRED,
];

impl std::convert::TryFrom<&[u32]> for Chmap {
    type Error = TlvDecodeError;

    fn try_from(raw: &[u32]) -> Result<Self, Self::Error> {
        // At least, type and length field should be included.
        if raw.len() < 2 {
            Err(Self::Error::new(TlvDecodeErrorCtx::Length(raw.len(), 2), 0))
        } else {
            // Check type field.
            let mode = match raw[0] {
                SNDRV_CTL_TLVT_CHMAP_FIXED => Ok(ChmapMode::Fixed),
                SNDRV_CTL_TLVT_CHMAP_VAR => Ok(ChmapMode::ArbitraryExchangeable),
                SNDRV_CTL_TLVT_CHMAP_PAIRED => Ok(ChmapMode::PairedExchangeable),
                _ => Err(Self::Error::new(
                    TlvDecodeErrorCtx::ValueType(raw[0], TYPES_FOR_CHMAP),
                    0,
                )),
            }?;

            // Check length field against length of value field.
            let value_length = (raw[1] / 4) as usize;
            let value = &raw[2..];
            if value.len() < value_length {
                Err(Self::Error::new(
                    TlvDecodeErrorCtx::ValueLength(value_length, value.len()),
                    1,
                ))
            } else if mode == ChmapMode::PairedExchangeable && value.len() % 2 > 0 {
                Err(Self::Error::new(
                    TlvDecodeErrorCtx::ValueLength(value_length, value.len()),
                    1,
                ))
            } else {
                // Decode value field.
                let entries = value.iter().map(|&val| ChmapEntry::from(val)).collect();
                Ok(Self { mode, entries })
            }
        }
    }
}

impl From<&Chmap> for Vec<u32> {
    fn from(data: &Chmap) -> Self {
        let mut raw = Vec::new();
        raw.push(data.value_type());
        raw.push(4 * data.value_length() as u32);
        raw.append(&mut data.value());
        raw
    }
}

impl From<Chmap> for Vec<u32> {
    fn from(data: Chmap) -> Self {
        (&data).into()
    }
}

#[cfg(test)]
mod test {
    use super::{Chmap, ChmapEntry, ChmapGenericPos, ChmapMode, ChmapPos};
    use super::{DbInterval, DbScale};
    use std::convert::TryFrom;

    #[test]
    fn test_dbitem() {
        let raw = [1u32, 8, -10i32 as u32, 0x00000010];
        let item = DbScale::try_from(raw.as_ref()).unwrap();
        assert_eq!(item.min, -10);
        assert_eq!(item.step, 16);
        assert_eq!(item.mute_avail, false);
        assert_eq!(&Vec::<u32>::from(item)[..], &raw[..]);
    }

    #[test]
    fn test_dbitem_mute_avail() {
        let raw = [1u32, 8, 10, 0x00010010];
        let item = DbScale::try_from(raw.as_ref()).unwrap();
        assert_eq!(item.min, 10);
        assert_eq!(item.step, 16);
        assert_eq!(item.mute_avail, true);
        assert_eq!(&Vec::<u32>::from(item)[..], &raw[..]);
    }

    #[test]
    fn test_dbinterval() {
        let raw = [4u32, 8, -100i32 as u32, 100];
        let item = DbInterval::try_from(&raw[..]).unwrap();
        assert_eq!(item.min, -100);
        assert_eq!(item.max, 100);
        assert_eq!(item.linear, false);
        assert_eq!(item.mute_avail, false);
        assert_eq!(&Vec::<u32>::from(item)[..], &raw[..]);
    }

    #[test]
    fn test_dbinterval_mute() {
        let raw = [5u32, 8, -100i32 as u32, 100];
        let item = DbInterval::try_from(&raw[..]).unwrap();
        assert_eq!(item.min, -100);
        assert_eq!(item.max, 100);
        assert_eq!(item.linear, false);
        assert_eq!(item.mute_avail, true);
        assert_eq!(&Vec::<u32>::from(item)[..], &raw[..]);
    }

    #[test]
    fn test_dbinterval_linear() {
        let raw = [2u32, 8, -100i32 as u32, 100];
        let item = DbInterval::try_from(&raw[..]).unwrap();
        assert_eq!(item.min, -100);
        assert_eq!(item.max, 100);
        assert_eq!(item.linear, true);
        assert_eq!(item.mute_avail, true);
        assert_eq!(&Vec::<u32>::from(item)[..], &raw[..]);
    }

    #[test]
    fn test_chmapgenericpos() {
        (0..u16::MAX).for_each(|val| {
            let generic_pos = ChmapGenericPos::from(val);
            assert_eq!(u16::from(generic_pos), val);
        });
    }

    #[test]
    fn test_chmapentry() {
        (0..37).for_each(|val| {
            let raw = val as u32;
            let entry = ChmapEntry::try_from(raw).unwrap();
            assert_eq!(entry.phase_inverse, false);
            assert_eq!(u32::from(entry), raw);

            let raw = 0x00010000u32 | (val as u32);
            let entry = ChmapEntry::try_from(raw).unwrap();
            assert_eq!(entry.phase_inverse, true);
            assert_eq!(u32::from(entry), raw);

            let raw = 0x00020000u32 | (val as u32);
            let entry = ChmapEntry::try_from(raw).unwrap();
            assert_eq!(entry.phase_inverse, false);
            assert_eq!(u32::from(entry), raw);
        });
    }

    #[test]
    fn test_chmap_fixed() {
        let raw = [0x101u32, 8, 3, 4];
        let map = Chmap::try_from(&raw[..]).unwrap();
        assert_eq!(map.mode, ChmapMode::Fixed);
        assert_eq!(
            &map.entries[..],
            &[
                ChmapEntry {
                    pos: ChmapPos::Generic(ChmapGenericPos::FrontLeft),
                    phase_inverse: false
                },
                ChmapEntry {
                    pos: ChmapPos::Generic(ChmapGenericPos::FrontRight),
                    phase_inverse: false
                },
            ]
        );
        assert_eq!(&Vec::<u32>::from(map)[..], &raw[..]);
    }

    #[test]
    fn test_chmap_arbitrary_exchangeable() {
        let raw = [0x102u32, 12, 3, 4, 8];
        let map = Chmap::try_from(&raw[..]).unwrap();
        assert_eq!(map.mode, ChmapMode::ArbitraryExchangeable);
        assert_eq!(
            &map.entries[..],
            &[
                ChmapEntry {
                    pos: ChmapPos::Generic(ChmapGenericPos::FrontLeft),
                    phase_inverse: false
                },
                ChmapEntry {
                    pos: ChmapPos::Generic(ChmapGenericPos::FrontRight),
                    phase_inverse: false
                },
                ChmapEntry {
                    pos: ChmapPos::Generic(ChmapGenericPos::LowFrequencyEffect),
                    phase_inverse: false
                },
            ][..]
        );
        assert_eq!(&Vec::<u32>::from(map)[..], &raw[..]);
    }

    #[test]
    fn test_chmap_paired_exchangeable() {
        let raw = [0x103u32, 16, 3, 4, 5, 6];
        let map = Chmap::try_from(&raw[..]).unwrap();
        assert_eq!(map.mode, ChmapMode::PairedExchangeable);
        assert_eq!(
            &map.entries[..],
            &[
                ChmapEntry {
                    pos: ChmapPos::Generic(ChmapGenericPos::FrontLeft),
                    phase_inverse: false
                },
                ChmapEntry {
                    pos: ChmapPos::Generic(ChmapGenericPos::FrontRight),
                    phase_inverse: false
                },
                ChmapEntry {
                    pos: ChmapPos::Generic(ChmapGenericPos::RearLeft),
                    phase_inverse: false
                },
                ChmapEntry {
                    pos: ChmapPos::Generic(ChmapGenericPos::RearRight),
                    phase_inverse: false
                },
            ][..]
        );
        assert_eq!(&Vec::<u32>::from(map)[..], &raw[..]);
    }
}
