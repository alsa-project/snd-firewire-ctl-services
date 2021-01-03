// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! The crate is designed to process data represented by TLV (Type-Length-Value) style in
//! ALSA control interface. The crate produces encoder and decoder for the u32 array of data
//! of TLV style as well as structures and enumerations to represent the content.
//!
//! The data of TLV style is used for several purposes. As of Linux kernel 5.10, it includes
//! information about dB representation as well as information about channel mapping in ALSA
//! PCM substream. The definitions are under `include/uapi/sound/tlv.h` of source code of
//! Linux kernel.
//!
//! ## Structures and enumerations
//!
//! Linux kernel has the series of macro to build u32 array for data of TLV, instead of definitions
//! of structure. This is convenient to embed binary data to object file, however not friendly to
//! developers and users. The crate has some structures and enumerations to represent the data of
//! TLV. The relationship between structures and macros is listed below:
//!
//! * `DbScale`
//!     * `SNDRV_CTL_TLVT_DB_SCALE`
//! * `DbInterval`
//!     * `SNDRV_CTL_TLVT_DB_LINEAR`
//!     * `SNDRV_CTL_TLVT_DB_MINMAX`
//!     * `SNDRV_CTL_TLVT_DB_MINMAX_MUTE`
//! * `Chmap` / `ChmapMode` / `ChmapEntry` / `ChmapPos` / `ChmapGenericPos`
//!     * `SNDRV_CTL_TLVT_CHMAP_FIXED`
//!     * `SNDRV_CTL_TLVT_CHMAP_VAR`
//!     * `SNDRV_CTL_TLVT_CHMAP_PAIRED`
//! * `DbRange` / `DbRangeEntry` / `DbRangeEntryData`
//!     * `SNDRV_CTL_TLVT_DB_RANGE`
//! * `Container`
//!     * `SNDRV_CTL_TLVT_CONTAINER`
//!
//! The crate has `TlvItem` enumeration to dispatch data of TLV for the above structures.
//!
//! ## Usage
//!
//! ```rust
//! use alsa_ctl_tlv_codec::TlvItem;
//! use std::convert::TryFrom;
//!
//! // Prepare raw data of TLV as array of u32 elements.
//! let raw = [2 as u32, 8, -100i32 as u32, 0]; // This is for SNDRV_CTL_TLVT_DB_LINEAR.
//!
//! match TlvItem::try_from(&raw[..]) {
//!     Ok(data) => {
//!         let raw_generated: Vec<u32> = match data {
//!           TlvItem::Container(d) => d.into(),
//!           TlvItem::DbRange(d) => d.into(),
//!           TlvItem::DbScale(d) => d.into(),
//!           TlvItem::DbInterval(d) => d.into(),
//!           TlvItem::Chmap(d) => d.into(),
//!         };
//!
//!         assert_eq!(&raw[..], &raw_generated[..]);
//!     }
//!     Err(err) => println!("{}", err),
//! }
//!
//! ```
//!
//! `TlvItem` enumeration is a good start to use the crate. It implements `TryFrom<&[u32]>` to
//! decode raw data of TLV which is array of u32 elements. The type of data is retrieved by a shape
//! of Rust enumeration items. Each item has associated value. Both of enumeration itself and the
//! structure of associated value has trait boundary to `Vec::<u32>: From(&Self)` to generate raw
//! data of TLV.
//!
//! The associated value can be instantiated directly, then raw data can be generated:
//!
//! ```rust
//! use alsa_ctl_tlv_codec::items;
//!
//! let scale = items::DbScale{
//!     min: -100,
//!     step: 10,
//!     mute_avail: true,
//! };
//!
//! let raw_generated: Vec<u32> = scale.into();
//!
//! let raw_expected = [1 as u32, 8, -100i32 as u32, 10 | 0x00010000];
//!
//! assert_eq!(&raw_generated[..], &raw_expected[..]);
//! ```
//!
//! Some of the associated value are container type, which aggregates the other items. In this
//! case, `TlvItem` is used for the aggregation of `Container`.
//!
//! ```rust
//! use alsa_ctl_tlv_codec::{*, items::*, containers::*};
//!
//! let cntr = Container{
//!     entries: vec![
//!         TlvItem::Chmap(Chmap{
//!             mode: ChmapMode::Fixed,
//!             entries: vec![
//!                 ChmapEntry{pos: ChmapPos::Generic(ChmapGenericPos::FrontLeft), ..Default::default()},
//!                 ChmapEntry{pos: ChmapPos::Generic(ChmapGenericPos::FrontRight), ..Default::default()},
//!                 ChmapEntry{pos: ChmapPos::Generic(ChmapGenericPos::LowFrequencyEffect), ..Default::default()},
//!             ],
//!         }),
//!         TlvItem::Chmap(Chmap{
//!             mode: ChmapMode::ArbitraryExchangeable,
//!             entries: vec![
//!                 ChmapEntry{pos: ChmapPos::Generic(ChmapGenericPos::FrontLeft), ..Default::default()},
//!                 ChmapEntry{pos: ChmapPos::Generic(ChmapGenericPos::FrontRight), ..Default::default()},
//!             ],
//!         }),
//!         TlvItem::Chmap(Chmap{
//!             mode: ChmapMode::PairedExchangeable,
//!             entries: vec![
//!                 ChmapEntry{pos: ChmapPos::Generic(ChmapGenericPos::FrontLeft), ..Default::default()},
//!                 ChmapEntry{pos: ChmapPos::Generic(ChmapGenericPos::FrontRight), ..Default::default()},
//!                 ChmapEntry{pos: ChmapPos::Generic(ChmapGenericPos::RearLeft), ..Default::default()},
//!                 ChmapEntry{pos: ChmapPos::Generic(ChmapGenericPos::RearRight), ..Default::default()},
//!             ],
//!         }),
//!     ],
//! };
//!
//! let raw_generated: Vec<u32> = cntr.into();
//!
//! let raw_expected = [0 as u32, 60,
//!                     0x101, 12, 3, 4, 8,
//!                     0x102, 8, 3, 4,
//!                     0x103, 16, 3, 4, 5, 6];
//!
//! assert_eq!(&raw_generated[..], &raw_expected[..]);
//!
//! ```
//!
//! ## Utilities
//!
//! Some programs are available under `src/bin` directory.
//!
//! ### src/bin/tlv-decode.rs
//!
//! This program decodes raw data of TLV from stdin, or numeric literals as arguments of command line,
//! then print parsed structure.
//!
//! Without any command line argument, it prints help message and exit.
//!
//! ```sh
//! $ cargo run --bin tlv-decode
//! Usage:
//!   tlv-decode MODE DATA | "-"
//! 
//!   where:
//!     MODE:           The mode to process after parsing DATA:
//!                         "structure":    prints data structures.
//!                         "macro":        prints C macro representation
//!                         "literal":      prints space-separated decimal array.
//!                         "raw":          prints binary with host endian.
//!     DATA:           space-separated DECIMAL and HEXADECIMAL array for the data of TLV.
//!     "-":            use binary from STDIN to interpret DATA according to host endian.
//!     DECIMAL:        decimal number. It can be signed if needed.
//!     HEXADECIMAL:    hexadecimal number. It should have '0x' as prefix.
//! ```
//!
//! For data of TLV from arguments in command line:
//!
//! ```sh
//! $ cargo run --bin tlv-decode -- structure 5 8 0xfffffe00 128
//! ...
//! DbInterval(DbInterval { min: -512, max: 128, linear: false, mute_avail: true })
//! ```
//!
//! For data of TLV from STDIN, in the case that the machine architecture is little endian:
//!
//! ```sh
//! $ echo -en "\x05\x00\x00\x00\x08\x00\x00\x00\x00\xfe\xff\xff\x80\x00\x00\x00" | \
//!     cargo run --bin tlv-decode -- structure -
//! ...
//! DbInterval(DbInterval { min: -512, max: 128, linear: false, mute_avail: true })
//! ```
//!
//! The data of TLV can be printed in C language macro representation:
//!
//! ```sh
//! $ echo -en "\x05\x00\x00\x00\x08\x00\x00\x00\x00\xfe\xff\xff\x80\x00\x00\x00" | \
//!     cargo run --bin tlv-decode -- macro -
//! ...
//! SNDRV_CTL_TLVD_ITEM ( SNDRV_CTL_TLVT_DB_MINMAX_MUTE, 0xfffffe00, 0x80 ) 
//! ```
//!
//! The data of TLV can be printed as both of u32 numeric literal array and u8 binary:
//!
//! ```sh
//! $ echo -en "\x05\x00\x00\x00\x08\x00\x00\x00\x00\xfe\xff\xff\x80\x00\x00\x00" | \
//!     cargo run --bin tlv-decode -- literal -
//! ...
//! 5 8 4294966784 128 
//!
//! $ echo -en "\x05\x00\x00\x00\x08\x00\x00\x00\x00\xfe\xff\xff\x80\x00\x00\x00" | \
//!     cargo run --bin tlv-decode -- raw -
//! ...
//! ```
//!
//! ### src/bin/db-calculate.rs
//!
//! This program calculates between dB value and raw value for control element, based on data of
//! TLV from STDIN or command line argument. It uses double precision floating point number for
//! dB calculation internally. For linear type of dB calculation, it uses exponentiation and logarithm.
//!
//! Without any command line argument, it prints help message and exit.
//!
//! ```sh
//! $ cargo run --bin db-calculate
//! Usage:
//!   db-calculate "db" DECIMAL-FLOATING-POINT VALUE-RANGE DATA | "-"
//!   db-calculate "value" DECIMAL | HEXADECIMAL VALUE-RANGE DATA | "-"
//!
//!   where:
//!     "db":                   Use this program for db calculation.
//!     "value":                Use this program for value calculation.
//!     DECIMAL-FLOATING-POINT: decimal floating point number. It can be signed if needed.
//!     DECIMAL:                decimal number. It can be signed if needed.
//!     HEXADECIMAL:            hexadecimal number. It should have '0x' as prefix.
//!     VALUE-RANGE:            space-separated triplet of MIN, MAX, and STEP comes from information of
//!                             control element. All of them are DECIMAL or HEXADECIMAL.
//!     DATA:                   space-separated DECIMAL and HEXADECIMAL array for the data of TLV.
//!     "-":                    use STDIN to interpret DATA according to host endian.
//!
//!    When data of TLV has information to support mute, "-9999999" for value and "-inf" for db are
//!    available.
//! ```
//!
//! For calculation from dB to value based on data of TLV from STDIN, in the case that the machine
//! architecture is little endian:
//!
//! ```sh
//! $ echo -en "\x05\x00\x00\x00\x08\x00\x00\x00\x00\xfe\xff\xff\x80\x00\x00\x00" | \
//!     cargo run --bin db-calculate db 1.0    128 512 1    -
//!   ...
//!   495
//! ```
//!
//! For calculation to dB from value based on data of TLV from arguments of command line:
//!
//! ```sh
//! $ cargo run --bin db-calculate value 495    128 512 1    5 8 0xfffffe00 0
//!   ...
//!   0.996666666666667
//! ```
//!
//! The calculation has no validated numerics.

#[allow(dead_code)]
mod uapi;

pub mod items;
pub mod containers;
pub mod range_utils;

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

use uapi::*;
use items::*;
use containers::*;

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
