// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! A set of minimum items in TLV (Type-Length-Value) of ALSA control interface.

use super::*;
use super::uapi::*;

/// The data to represent dB scale in TLV (Type-Length-Value) of ALSA control interface.
///
/// It has `SNDRV_CTL_TLVT_DB_SCALE` (=1) in its type field and has two elements in value field.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct DbScale{
    /// The minimum value by dB representation, in 0.1 dB unit. This corresponds to the minimum
    /// value in the state of control element.
    pub min: i32,
    /// The step by dB representation, in 0.1 dB unit. This corresponds to one increase of the value
    /// in the state of control element.
    pub step: u16,
    /// If true, the value less than or equals to [`CTL_VALUE_MUTE`] (=-9999999) is available to
    /// mute the control element explicitly.
    pub mute_avail: bool,
}

/// When information about dB includes mute_avail, the value is available to mute the control
/// element. It's relevant to `SNDRV_CTL_TLVD_DB_GAIN_MUTE` macro in UAPI of Linux kernel.
pub const CTL_VALUE_MUTE: i32 = SNDRV_CTL_TLVD_DB_GAIN_MUTE;

/// The value of dB should be represented in 0.1 dB unit in data of TLV and crate structures.
pub const DB_VALUE_MULTIPLIER: i32= 100;

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

impl std::convert::TryFrom<&[u32]> for DbScale {
    type Error = InvalidTlvDataError;

    fn try_from(raw: &[u32]) -> Result<Self, Self::Error> {
        if raw.len() != 4 || raw[1] != 4 * Self::VALUE_COUNT as u32 {
            Err(InvalidTlvDataError::new("Invalid length of data for DbScale"))
        } else if raw[0] != SNDRV_CTL_TLVT_DB_SCALE {
            Err(InvalidTlvDataError::new("Invalid type of data for DbScale"))
        } else {
            let data = DbScale{
                min: raw[2] as i32,
                step: (raw[3] & SNDRV_CTL_TLVD_DB_SCALE_MASK) as u16,
                mute_avail: raw[3] & SNDRV_CTL_TLVD_DB_SCALE_MUTE > 0,
            };
            Ok(data)
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

/// The data to represent dB interval in TLV (Type-Length-Value) of ALSA control interface.
///
/// It has three variants below;
///  * SNDRV_CTL_TLVT_DB_LINEAR(=2)
///  * SNDRV_CTL_TLVT_DB_MINMAX(=4)
///  * SNDRV_CTL_TLVT_DB_MINMAX_MUTE(=5)
///
///  All of them have two elements in value field.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct DbInterval{
    /// The minimum value by dB representation, in 0.1 dB unit. This corresponds to the minimum
    /// value in the state of control element.
    pub min: i32,
    /// The maximum value by dB representation, 0.1 dB unit. This corresponds to the maximum value
    /// in the state of control element.
    pub max: i32,
    /// If true, the value in the state of control element increases linearly, thus need calculation
    /// to convert to the value by dB representation. The calculation shall be:
    ///
    /// 20 * log10( current_value / ( maximum_value - minimum_value ) ) (* 100 in 0.1dB unit)
    ///
    /// Else, the value in the state of control element corresponds to dB representation itself.
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

impl std::convert::TryFrom<&[u32]> for DbInterval {
    type Error = InvalidTlvDataError;

    fn try_from(raw: &[u32]) -> Result<Self, Self::Error> {
        if raw.len() != 4 || raw[1] != 4 * Self::VALUE_COUNT as u32 {
            Err(InvalidTlvDataError::new("Invalid length of data for DbInterval"))
        } else if raw[0] != SNDRV_CTL_TLVT_DB_LINEAR &&
                  raw[0] != SNDRV_CTL_TLVT_DB_MINMAX &&
                  raw[0] != SNDRV_CTL_TLVT_DB_MINMAX_MUTE {
            Err(InvalidTlvDataError::new("Invalid type of data for DbInterval"))
        } else {
            let mut data = DbInterval{
                min: raw[2] as i32,
                max: raw[3] as i32,
                linear: false,
                mute_avail: false,
            };
            if raw[0] == SNDRV_CTL_TLVT_DB_LINEAR {
                data.linear = true;
                data.mute_avail = true;
            } else if raw[0] == SNDRV_CTL_TLVT_DB_MINMAX_MUTE {
                data.mute_avail = true;
            }
            Ok(data)
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

#[cfg(test)]
mod test {
    use std::convert::TryFrom;
    use super::{DbScale, DbInterval};

    #[test]
    fn test_dbitem() {
        let raw = [1u32, 8, -10i32 as u32, 0x00000010];
        let item = DbScale::try_from(raw.as_ref()).unwrap();
        assert_eq!(item.min, -10);
        assert_eq!(item.step, 16);
        assert_eq!(item.mute_avail, false);
        assert_eq!(&Into::<Vec<u32>>::into(item)[..], &raw[..]);
    }

    #[test]
    fn test_dbitem_mute_avail() {
        let raw = [1u32, 8, 10, 0x00010010];
        let item = DbScale::try_from(raw.as_ref()).unwrap();
        assert_eq!(item.min, 10);
        assert_eq!(item.step, 16);
        assert_eq!(item.mute_avail, true);
        assert_eq!(&Into::<Vec<u32>>::into(item)[..], &raw[..]);
    }

    #[test]
    fn test_dbinterval() {
        let raw = [4u32, 8, -100i32 as u32, 100];
        let item = DbInterval::try_from(&raw[..]).unwrap();
        assert_eq!(item.min, -100);
        assert_eq!(item.max, 100);
        assert_eq!(item.linear, false);
        assert_eq!(item.mute_avail, false);
        assert_eq!(&Into::<Vec<u32>>::into(item)[..], &raw[..]);
    }

    #[test]
    fn test_dbinterval_mute() {
        let raw = [5u32, 8, -100i32 as u32, 100];
        let item = DbInterval::try_from(&raw[..]).unwrap();
        assert_eq!(item.min, -100);
        assert_eq!(item.max, 100);
        assert_eq!(item.linear, false);
        assert_eq!(item.mute_avail, true);
        assert_eq!(&Into::<Vec<u32>>::into(item)[..], &raw[..]);
    }

    #[test]
    fn test_dbinterval_linear() {
        let raw = [2u32, 8, -100i32 as u32, 100];
        let item = DbInterval::try_from(&raw[..]).unwrap();
        assert_eq!(item.min, -100);
        assert_eq!(item.max, 100);
        assert_eq!(item.linear, true);
        assert_eq!(item.mute_avail, true);
        assert_eq!(&Into::<Vec<u32>>::into(item)[..], &raw[..]);
    }
}
