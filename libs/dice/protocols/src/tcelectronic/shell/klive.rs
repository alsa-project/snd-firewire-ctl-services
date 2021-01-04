// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Protocol defined by TC Electronic for Konnekt Live.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by TC Electronic for Konnekt Live.

/// The structure to represent segments in memory space of Konnekt Live.
#[derive(Default, Debug)]
pub struct KliveSegments;
