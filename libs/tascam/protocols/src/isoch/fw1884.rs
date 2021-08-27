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

impl IsochCommonOperation for Fw1884Protocol {
    const SAMPLING_CLOCK_SOURCES: &'static [ClkSrc] = &[
        ClkSrc::Internal,
        ClkSrc::Wordclock,
        ClkSrc::Spdif,
        ClkSrc::Adat,
    ];
}

impl IsochOpticalOperation for Fw1884Protocol {
    const OPTICAL_OUTPUT_SOURCES: &'static [(OpticalOutputSource, u32, u32)] = &[
        (
            OpticalOutputSource::StreamInputPairs,
            0x00000080,
            0x0000c000,
        ),
        (
            OpticalOutputSource::CoaxialOutputPair0,
            0x00000004,
            0x00080400,
        ),
        (
            OpticalOutputSource::AnalogInputPair0,
            0x00000088,
            0x00048800,
        ),
        (
            OpticalOutputSource::AnalogOutputPairs,
            0x00000008,
            0x00840800,
        ),
    ];
}
