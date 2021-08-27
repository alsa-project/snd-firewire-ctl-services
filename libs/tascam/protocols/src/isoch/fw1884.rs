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

impl MachineStateOperation for Fw1884Protocol {
    const BOOL_ITEMS: &'static [MachineItem] = &[
        MachineItem::Rec(0),
        MachineItem::Rec(1),
        MachineItem::Rec(2),
        MachineItem::Rec(3),
        MachineItem::Rec(4),
        MachineItem::Rec(5),
        MachineItem::Rec(6),
        MachineItem::Rec(7),
        MachineItem::Select(0),
        MachineItem::Select(1),
        MachineItem::Select(2),
        MachineItem::Select(3),
        MachineItem::Select(4),
        MachineItem::Select(5),
        MachineItem::Select(6),
        MachineItem::Select(7),
        MachineItem::Solo(0),
        MachineItem::Solo(1),
        MachineItem::Solo(2),
        MachineItem::Solo(3),
        MachineItem::Solo(4),
        MachineItem::Solo(5),
        MachineItem::Solo(6),
        MachineItem::Solo(7),
        MachineItem::Mute(0),
        MachineItem::Mute(1),
        MachineItem::Mute(2),
        MachineItem::Mute(3),
        MachineItem::Mute(4),
        MachineItem::Mute(5),
        MachineItem::Mute(6),
        MachineItem::Mute(7),
        MachineItem::Func(0),
        MachineItem::Func(1),
        MachineItem::Func(2),
        MachineItem::Func(3),
        MachineItem::Func(4),
        MachineItem::Func(5),
        MachineItem::Func(6),
        MachineItem::Func(7),
        MachineItem::Func(8),
        MachineItem::Func(9),
        MachineItem::Pfl,
        MachineItem::Read,
        MachineItem::Wrt,
        MachineItem::Tch,
        MachineItem::Latch,
        MachineItem::Shuttle,
        MachineItem::Computer,
        MachineItem::Clock,
        MachineItem::Up,
        MachineItem::Left,
        MachineItem::Down,
        MachineItem::Right,
        MachineItem::NudgeLeft,
        MachineItem::NudgeRight,
        MachineItem::LocateLeft,
        MachineItem::LocateRight,
        MachineItem::Set,
        MachineItem::In,
        MachineItem::Out,
        MachineItem::Flip,
        MachineItem::Pan,
        MachineItem::Aux(0),
        MachineItem::Aux(1),
        MachineItem::Aux(2),
        MachineItem::Aux(3),
        MachineItem::Aux(4),
        MachineItem::Aux(5),
        MachineItem::Aux(6),
        MachineItem::Aux(7),
        MachineItem::High,
        MachineItem::HighMid,
        MachineItem::LowMid,
        MachineItem::Low,
        MachineItem::Recall,
        MachineItem::Panel,
        MachineItem::Save,
        MachineItem::Revert,
        MachineItem::AllSafe,
        MachineItem::ClrSolo,
        MachineItem::Markers,
        MachineItem::Loop,
        MachineItem::Cut,
        MachineItem::Del,
        MachineItem::Copy,
        MachineItem::Paste,
        MachineItem::Alt,
        MachineItem::Cmd,
        MachineItem::Undo,
        MachineItem::Shift,
        MachineItem::Ctrl,
    ];

    const U16_ITEMS: &'static [MachineItem] = &[
        MachineItem::Master,
        MachineItem::Rotary(0),
        MachineItem::Rotary(1),
        MachineItem::Rotary(2),
        MachineItem::Rotary(3),
        MachineItem::Rotary(4),
        MachineItem::Rotary(5),
        MachineItem::Rotary(6),
        MachineItem::Rotary(7),
        MachineItem::Input(0),
        MachineItem::Input(1),
        MachineItem::Input(2),
        MachineItem::Input(3),
        MachineItem::Input(4),
        MachineItem::Input(5),
        MachineItem::Input(6),
        MachineItem::Input(7),
        MachineItem::Wheel,
        MachineItem::Gain,
        MachineItem::Freq,
        MachineItem::Q,
    ];

    const HAS_TRANSPORT: bool = true;
    const HAS_BANK: bool = true;
}
