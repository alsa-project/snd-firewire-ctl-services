// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol implementation for some Terratec models.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by Terratec for some FireWire models.

pub mod aureon;
pub mod phase88;

use super::*;
