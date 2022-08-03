// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol implementation specific to PreSonus FireStudio series.
//!
//! The module includes structure, enumeration, trait and its implementation for protocol defined
//! by PreSonus for FireStudio series.

pub mod fstudio;

pub mod fstudiomobile;
pub mod fstudioproject;

use super::*;
