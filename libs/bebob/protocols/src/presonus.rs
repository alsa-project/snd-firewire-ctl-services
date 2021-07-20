// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol implementation for some PreSonus models.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by PreSonus for some FireWire models.

const PRESONUS_OUI: [u8; 3] = [0x00, 0x0a, 0x92];

pub mod inspire1394;
pub mod fp10;
