// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol implementation for Apogee Electronics FireWire models.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by Apogee Electronics for FireWire models.

pub mod ensemble;

const APOGEE_OUI: [u8; 3] = [0x00, 0x03, 0xdb];
