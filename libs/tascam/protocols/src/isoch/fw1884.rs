// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol implementation for FW-1884.
//!
//! The module includes protocol implementation defined by Tascam for FW-1884.

use super::*;

/// The protocol implementation of FW-1884.
#[derive(Default)]
pub struct Fw1884Protocol;

impl IsochMeterOperation for Fw1884Protocol {
    const INPUT_COUNT: usize = 18;
    const OUTPUT_COUNT: usize = 18;
    const HAS_SOLO: bool = true;
}
