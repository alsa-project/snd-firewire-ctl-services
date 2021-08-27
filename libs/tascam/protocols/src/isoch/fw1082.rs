// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol implementation for FW-1082.
//!
//! The module includes protocol implementation defined by Tascam for FW-1082.

use super::*;

/// The protocol implementation of FW-1082.
#[derive(Default)]
pub struct Fw1082Protocol;

impl IsochMeterOperation for Fw1082Protocol {
    const INPUT_COUNT: usize = 10;
    const OUTPUT_COUNT: usize = 4;
    const HAS_SOLO: bool = true;
}

impl IsochCommonOperation for Fw1082Protocol {
    const SAMPLING_CLOCK_SOURCES: &'static [ClkSrc] = &[ClkSrc::Internal, ClkSrc::Spdif];
}
