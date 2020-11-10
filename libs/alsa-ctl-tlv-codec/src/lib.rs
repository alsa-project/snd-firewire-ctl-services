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
