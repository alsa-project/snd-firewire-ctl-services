// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2024 Takashi Sakamoto

//! Protocol specific to Weiss Engineering models.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by Weiss Engineering.

pub mod avc;
pub mod normal;

use super::tcat::{global_section::*, *};
