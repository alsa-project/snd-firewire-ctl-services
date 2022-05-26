// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol implementation for M-Audio FireWire series.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by M-Audio FireWire series.

pub mod normal;
pub mod pfl;
pub mod special;

const MAUDIO_OUI: [u8; 3] = [0x00, 0x0d, 0x6c];

use super::*;
