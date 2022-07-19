// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

#![doc = include_str!("../README.md")]

#[allow(dead_code)]
mod uapi;

mod containers;
mod items;
mod range_utils;

pub use {containers::*, items::*, range_utils::*};

use uapi::*;

/// The cause of error to decode given array as TLV data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TlvDecodeErrorCtx {
    /// Insufficient length of available elements in array.
    Length(
        /// The lengh of available elements in array.
        usize,
        /// The length expected at least.
        usize,
    ),
    /// Invalid value in type field.
    ValueType(
        /// The value in type field.
        u32,
        /// The allowed types.
        &'static [u32],
    ),
    /// Invalid value in length field.
    ValueLength(
        /// The value in length field.
        usize,
        /// The actual length of available elements in array.
        usize,
    ),
    /// Invalid data in value field.
    ValueContent(Box<TlvDecodeError>),
}

/// The structure to store decode context and position for error reporting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TlvDecodeError {
    /// The context at which decode error appears.
    pub ctx: TlvDecodeErrorCtx,
    /// The offset at which decode error appears. It's mostly offset from the beginning of
    /// Type-Length-Value array. Each entry in DbRange is an exception that the offset is
    /// from the beginning of the entry.
    pub offset: usize,
}

impl TlvDecodeError {
    pub fn new(ctx: TlvDecodeErrorCtx, offset: usize) -> Self {
        Self { ctx, offset }
    }

    pub fn delve_into_lowest_error(&self, mut offset: usize) -> (&TlvDecodeError, usize) {
        offset += self.offset;
        match &self.ctx {
            TlvDecodeErrorCtx::ValueContent(e) => e.delve_into_lowest_error(offset),
            _ => (self, offset),
        }
    }
}

impl std::fmt::Display for TlvDecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (err, offset) = self.delve_into_lowest_error(0);
        match err.ctx {
            TlvDecodeErrorCtx::Length(len, at_least) => {
                write!(
                    f,
                    "Insufficient length of array {}, should be greater than {}, at {}",
                    len, at_least, offset
                )
            }
            TlvDecodeErrorCtx::ValueType(val_type, allowed_types) => {
                let mut allowed = format!("{}", allowed_types[0]);
                allowed_types[1..]
                    .iter()
                    .for_each(|allowed_type| allowed = format!("{}, {}", allowed, allowed_type));
                write!(
                    f,
                    "Invalid value {} in type field, expected {}, at {}",
                    val_type, allowed, offset
                )
            }
            TlvDecodeErrorCtx::ValueLength(val_len, array_len) => {
                write!(
                    f,
                    "Invalid value {} in length field, actual {}, at {}",
                    val_len, array_len, offset
                )
            }
            _ => unreachable!(),
        }
    }
}

impl std::error::Error for TlvDecodeError {}

/// The trait for common methods to data of TLV (Type-Length-Value) in ALSA control interface.
/// The TryFrom supertrait should be implemented to parse the array of u32 elements and it
/// can return TlvDecodeError at failure. The trait boundary to Vec::<u32>::From(&Self)
/// should be implemented as well to build the array of u32 element.
pub trait TlvData<'a>: std::convert::TryFrom<&'a [u32]>
where
    for<'b> Vec<u32>: From<&'b Self>,
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
pub enum TlvItem {
    Container(Container),
    DbRange(DbRange),
    DbScale(DbScale),
    DbInterval(DbInterval),
    Chmap(Chmap),
    Unknown(Vec<u32>),
}

impl<'a> std::convert::TryFrom<&'a [u32]> for TlvItem {
    type Error = TlvDecodeError;

    fn try_from(raw: &[u32]) -> Result<Self, Self::Error> {
        match raw[0] {
            SNDRV_CTL_TLVT_CONTAINER => {
                Container::try_from(raw).map(|data| TlvItem::Container(data))
            }
            SNDRV_CTL_TLVT_DB_RANGE => DbRange::try_from(raw).map(|data| TlvItem::DbRange(data)),
            SNDRV_CTL_TLVT_DB_SCALE => DbScale::try_from(raw).map(|data| TlvItem::DbScale(data)),
            SNDRV_CTL_TLVT_DB_LINEAR | SNDRV_CTL_TLVT_DB_MINMAX | SNDRV_CTL_TLVT_DB_MINMAX_MUTE => {
                DbInterval::try_from(raw).map(|data| TlvItem::DbInterval(data))
            }
            SNDRV_CTL_TLVT_CHMAP_FIXED | SNDRV_CTL_TLVT_CHMAP_VAR | SNDRV_CTL_TLVT_CHMAP_PAIRED => {
                Chmap::try_from(raw).map(|data| TlvItem::Chmap(data))
            }
            _ => Ok(TlvItem::Unknown(raw.to_owned())),
        }
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
            TlvItem::Unknown(d) => d.to_owned(),
        }
    }
}

impl From<TlvItem> for Vec<u32> {
    fn from(item: TlvItem) -> Self {
        (&item).into()
    }
}
