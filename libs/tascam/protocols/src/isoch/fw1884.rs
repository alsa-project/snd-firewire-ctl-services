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

impl IsochConsoleOperation for Fw1884Protocol {}

/// The target of monitor knob.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Fw1884MonitorKnobTarget {
    /// For analog output 1/2.
    AnalogOutputPair0,
    /// For analog output 1, 2, 3, 4, 5, and 6.
    AnalogOutput3Pairs,
    /// For analog output 1, 2, 3, 4, 5, 6, 7, and 8.
    AnalogOutput4Pairs,
}

impl Default for Fw1884MonitorKnobTarget {
    fn default() -> Self {
        Self::AnalogOutputPair0
    }
}

const MONITOR_KNOB_TARGETS: [(Fw1884MonitorKnobTarget, u32, u32); 3] = [
    (
        Fw1884MonitorKnobTarget::AnalogOutputPair0,
        0x01000010,
        0x02001000,
    ),
    (
        Fw1884MonitorKnobTarget::AnalogOutput3Pairs,
        0x00000010,
        0x04001000,
    ),
    (
        Fw1884MonitorKnobTarget::AnalogOutput4Pairs,
        0x00000000,
        0x00100000,
    ),
];

impl Fw1884Protocol {
    pub fn get_monitor_knob_target(
        req: &mut FwReq,
        node: &mut FwNode,
        timeout_ms: u32,
    ) -> Result<Fw1884MonitorKnobTarget, Error> {
        read_config_flag(req, node, &MONITOR_KNOB_TARGETS, timeout_ms)
    }

    pub fn set_monitor_knob_target(
        req: &mut FwReq,
        node: &mut FwNode,
        target: Fw1884MonitorKnobTarget,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        write_config_flag(req, node, &MONITOR_KNOB_TARGETS, target, timeout_ms)
    }
}
