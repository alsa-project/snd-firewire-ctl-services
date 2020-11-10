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

#[cfg(test)]
mod test {
    use std::convert::TryFrom;
    use super::DbScale;

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
}
