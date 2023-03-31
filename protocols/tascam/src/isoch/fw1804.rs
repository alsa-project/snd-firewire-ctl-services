// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol implementation for FW-1804.
//!
//! The module includes protocol implementation defined by Tascam for FW-1804.
//!
//! The protocol implementation for Tascam FW-1804 was written with firmware version below:
//!
//! ```text
//! $ cargo run --bin tascam-hardware-information /dev/fw1
//!
//! Hardware information:
//!   Register: 0x00020014
//!   FPGA:     0x00010031
//!   ARM:      0x000100b7
//!   Hardware: 0x00060000
//! ```

use super::*;

/// The protocol implementation of FW-1804.
#[derive(Default)]
pub struct Fw1804Protocol;

impl TascamIsochClockSpecification for Fw1804Protocol {
    const SAMPLING_CLOCK_SOURCES: &'static [ClkSrc] = &[
        ClkSrc::Internal,
        ClkSrc::Wordclock,
        ClkSrc::Spdif,
        ClkSrc::Adat,
    ];
}

impl TascamIsochInputDetectionSpecification for Fw1804Protocol {}

impl TascamIsochCoaxialOutputSpecification for Fw1804Protocol {}

impl TascamIsochOpticalIfaceSpecification for Fw1804Protocol {
    const OPTICAL_OUTPUT_SOURCES: &'static [(OpticalOutputSource, u32, u32)] = &[
        (
            OpticalOutputSource::StreamInputPairs,
            0x00000000,
            0x0000c000,
        ),
        (
            OpticalOutputSource::CoaxialOutputPair0,
            0x00000004,
            0x00080400,
        ),
        (
            OpticalOutputSource::AnalogInputPair0,
            0x00000008,
            0x00048800,
        ),
    ];
}

impl IsochMeterOperation for Fw1804Protocol {
    const INPUT_COUNT: usize = 18;
    const OUTPUT_COUNT: usize = 18;
    const HAS_SOLO: bool = false;
}

impl IsochRackOperation for Fw1804Protocol {}

impl FireWireLedOperation for Fw1804Protocol {
    const POSITIONS: &'static [u16] = &[0x8e];
}
