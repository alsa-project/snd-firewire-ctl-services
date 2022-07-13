// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

#![doc = include_str!("../README.md")]

#[allow(dead_code)]
mod uapi;

mod items;
mod containers;
mod range_utils;

pub use {
    containers::*,
    items::*,
    range_utils::*,
};

use uapi::*;

/// The error type at failure to convert from array of u32 elements to TLV data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidTlvDataError{
    msg: &'static str,
}

impl InvalidTlvDataError {
    pub fn new(msg: &'static str) -> Self {
        InvalidTlvDataError{msg}
    }
}

impl std::fmt::Display for InvalidTlvDataError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl std::error::Error for InvalidTlvDataError {}

/// The trait for common methods to data of TLV (Type-Length-Value) in ALSA control interface.
/// The TryFrom supertrait should be implemented to parse the array of u32 elements and it
/// can return InvalidTlvDataError at failure. The trait boundary to Vec::<u32>::From(&Self)
/// should be implemented as well to build the array of u32 element.
pub trait TlvData<'a> : std::convert::TryFrom<&'a [u32]>
    where for<'b> Vec<u32>: From<&'b Self>,
{
    /// Return the value of type field. It should come from UAPI of Linux kernel.
    fn value_type(&self) -> u32;

    /// Return the length of value field. It should be in byte unit and multiples of 4 as result.
    fn value_length(&self) -> usize;

    /// Generate vector with u32 element for raw data.
    fn value(&self) -> Vec<u32>;
}

/// Available items as data of TLV (Type-Length-Value) style in ALSA control interface.
///
/// When decoding from data of TLV, use implementation of `TryFrom<&[u32]>` trait.  Data assigned
/// to each enumeration implements `TlvData`, `TryFrom<&[u32]`, and `Vec::<u32>::from(&Self)` trait.
/// When decoding to data of TLV, use implementation of `Vec::<u32>::from(&Self)` for the data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TlvItem{
    Container(Container),
    DbRange(DbRange),
    DbScale(DbScale),
    DbInterval(DbInterval),
    Chmap(Chmap),
}

impl<'a> std::convert::TryFrom<&'a [u32]> for TlvItem {
    type Error = InvalidTlvDataError;

    fn try_from(raw: &[u32]) -> Result<Self, Self::Error> {
        let entry = match raw[0] {
            SNDRV_CTL_TLVT_CONTAINER => {
                let data = Container::try_from(raw)?;
                TlvItem::Container(data)
            }
            SNDRV_CTL_TLVT_DB_RANGE => {
                let data = DbRange::try_from(raw)?;
                TlvItem::DbRange(data)
            }
            SNDRV_CTL_TLVT_DB_SCALE => {
                let data = DbScale::try_from(raw)?;
                TlvItem::DbScale(data)
            }
            SNDRV_CTL_TLVT_DB_LINEAR |
            SNDRV_CTL_TLVT_DB_MINMAX |
            SNDRV_CTL_TLVT_DB_MINMAX_MUTE => {
                let data = DbInterval::try_from(raw)?;
                TlvItem::DbInterval(data)
            }
            SNDRV_CTL_TLVT_CHMAP_FIXED |
            SNDRV_CTL_TLVT_CHMAP_VAR |
            SNDRV_CTL_TLVT_CHMAP_PAIRED => {
                let data = Chmap::try_from(raw)?;
                TlvItem::Chmap(data)
            }
            _ => {
                return Err(InvalidTlvDataError::new("Invalid type of data for TlvItem"));
            }
        };
        Ok(entry)
    }
}

impl From<&TlvItem> for Vec<u32> {
    fn from(item: &TlvItem) -> Self {
        match item {
            TlvItem::Container(d) => d.into(),
            TlvItem::DbRange(d) => d.into(),
            TlvItem::DbScale(d) => d.into(),
            TlvItem::DbInterval(d) => d.into(),
            TlvItem::Chmap(d) => d.into(),
        }
    }
}

impl From<TlvItem> for Vec<u32> {
    fn from(item: TlvItem) -> Self {
        (&item).into()
    }
}
