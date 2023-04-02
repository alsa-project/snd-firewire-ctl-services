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
//!
//! ## Diagram of internal signal flow
//!
//! ```text
//!
//! andlog-input-0/1 ------+---------------------+--------> stream-output-0/1
//! andlog-input-2/3 ------|--+------------------|--------> stream-output-2/3
//! andlog-input-4/5 ------|--|--+---------------|--------> stream-output-4/5
//! andlog-input-6/7 ------|--|--|--+------------|--------> stream-output-6/7
//! coaxial-input-0/1 -----|--|--|--|------------|--------> stream-output-8/9
//! optical-input-0/1 -----|--|--|--|------------|--------> stream-output-10/11
//! optical-input-2/3 -----|--|--|--|------------|--------> stream-output-12/13
//! optical-input-4/5 -----|--|--|--|------------|--------> stream-output-14/15
//! optical-input-6/7 -----|--|--|--|------------|--------> stream-output-16/17
//!                        v  v  v  v            |
//!                      ++==========++          |
//!                      || monitor  ||          |
//!                      ++==========++          |
//!                            |                 |
//!                            v                 |
//!                      ++==========++          |
//! stream-input-0/1 --> ||  mixer   || --+------|--------> analog-output-0/1
//!                      ++==========++   |      |
//!                                       v      |
//! stream-input-3/4   -------------> (one of) --|--+-----> coaxial-output-0/1
//!                                              |  |
//!                                              v  v
//! stream-input-5..11 ----------------------> (one of) --> optical-output-0..7
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

impl TascamIsochRackInputSpecification for Fw1804Protocol {}

impl TascamHardwareImageSpecification for Fw1804Protocol {
    const IMAGE_QUADLET_COUNT: usize = ISOCH_IMAGE_QUADLET_COUNT;
}

impl TascamIsochMeterSpecification for Fw1804Protocol {
    const INPUT_COUNT: usize = 18;
    const OUTPUT_COUNT: usize = 18;
    const HAS_SOLO: bool = false;
}

impl FireWireLedOperation for Fw1804Protocol {
    const POSITIONS: &'static [u16] = &[0x8e];
}
