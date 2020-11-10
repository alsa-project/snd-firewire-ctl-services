// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! The crate is designed to process data represented by TLV (Type-Length-Value) style in
//! ALSA control interface. The crate produces encoder and decoder for the u32 array of data
//! as well as structures and enumerations to represent the data.
//!
//! The data of TLV style is used for several purposes. As of Linux kernel 5.10, it includes
//! information about dB representation as well as information about channel mapping in ALSA
//! PCM substream.

#[allow(dead_code)]
mod uapi;

pub mod items;
pub mod containers;

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
/// can return InvalidTlvDataError at failure. The Into supertrait should be implemented as well
/// to build the array of u32 element.
pub trait TlvData<'a> : std::convert::TryFrom<&'a [u32]> + Into<Vec<u32>> {
    /// Return the value of type field. It should come from UAPI of Linux kernel.
    fn value_type(&self) -> u32;

    /// Return the length of value field. It should be in byte unit and multiples of 4 as result.
    fn value_length(&self) -> usize;

    /// Generate vector with u32 element for raw data.
    fn value(&self) -> Vec<u32>;
}
