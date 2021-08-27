// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol implementation for FW-1804.
//!
//! The module includes protocol implementation defined by Tascam for FW-1804.

use super::*;

/// The protocol implementation of FW-1804.
#[derive(Default)]
pub struct Fw1804Protocol;

impl IsochMeterOperation for Fw1804Protocol {
    const INPUT_COUNT: usize = 18;
    const OUTPUT_COUNT: usize = 18;
    const HAS_SOLO: bool = false;
}

impl IsochCommonOperation for Fw1804Protocol {
    const SAMPLING_CLOCK_SOURCES: &'static [ClkSrc] = &[
        ClkSrc::Internal,
        ClkSrc::Wordclock,
        ClkSrc::Spdif,
        ClkSrc::Adat,
    ];
}
