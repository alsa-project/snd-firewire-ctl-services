// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol implementation for FW-1884.
//!
//! The module includes protocol implementation defined by Tascam for FW-1884.
//!
//! The protocol implementation for Tascam FW-1884 was written with firmware version below:
//!
//! ```text
//! $ cargo run --bin tascam-hardware-information /dev/fw1
//!
//! Hardware information:
//!   Register: 0x00000014
//!   FPGA:     0x00010031
//!   ARM:      0x000100b7
//!   Hardware: 0x00030000
//! ```

use crate::{isoch::*, *};

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

/// The structure for state of control surface in FW-1884.
#[derive(Default, Debug)]
pub struct Fw1884SurfaceState {
    common: SurfaceCommonState,
    isoch: SurfaceIsochState,
    led_state: LedState,
}

impl SurfaceImageOperation<Fw1884SurfaceState> for Fw1884Protocol {
    fn initialize_surface_state(state: &mut Fw1884SurfaceState) {
        Self::initialize_surface_common_state(&mut state.common);
        Self::initialize_surface_isoch_state(&mut state.isoch);
    }

    fn decode_surface_image(
        state: &Fw1884SurfaceState,
        image: &[u32],
        index: u32,
        before: u32,
        after: u32,
    ) -> Vec<(MachineItem, ItemValue)> {
        let mut machine_values = Vec::new();

        Self::decode_surface_image_common(
            &mut machine_values,
            &state.common,
            image,
            index,
            before,
            after,
        );

        Self::decode_surface_image_isoch(&mut machine_values, &state.isoch, index, before, after);

        machine_values
    }

    fn feedback_to_surface(
        state: &mut Fw1884SurfaceState,
        machine_value: &(MachineItem, ItemValue),
        req: &mut FwReq,
        node: &mut FwNode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        Self::feedback_to_surface_common(&mut state.common, machine_value);
        Self::feedback_to_surface_isoch(&mut state.isoch, machine_value);

        if let ItemValue::Bool(value) = machine_value.1 {
            if let Some(pos) = Self::find_normal_led_pos(&machine_value.0) {
                operate_led_cached(&mut state.led_state, req, node, pos, value, timeout_ms)?;
            }
        } else if let ItemValue::U16(value) = machine_value.1 {
            if MachineItem::Bank.eq(&machine_value.0) {
                Self::operate_bank_leds(&mut state.led_state, req, node, value, timeout_ms)?;
            }
        }

        Ok(())
    }

    fn finalize_surface(
        state: &mut Fw1884SurfaceState,
        req: &mut FwReq,
        node: &mut FwNode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        clear_leds(&mut state.led_state, req, node, timeout_ms)
    }
}

impl SurfaceImageCommonOperation for Fw1884Protocol {
    const STATEFUL_ITEMS: &'static [(SurfaceBoolValue, MachineItem)] = &[
        (SurfaceBoolValue(7, 0x00200000), MachineItem::Aux(3)),
        (SurfaceBoolValue(7, 0x00100000), MachineItem::Aux(1)),
        (SurfaceBoolValue(7, 0x00040000), MachineItem::Aux(2)),
        (SurfaceBoolValue(7, 0x00020000), MachineItem::Aux(0)),
        (SurfaceBoolValue(7, 0x00000800), MachineItem::Aux(7)),
        (SurfaceBoolValue(7, 0x00000400), MachineItem::Aux(5)),
        (SurfaceBoolValue(7, 0x00000200), MachineItem::Aux(6)),
        (SurfaceBoolValue(7, 0x00000100), MachineItem::Aux(4)),
        (SurfaceBoolValue(6, 0x80000000), MachineItem::Solo(7)),
        (SurfaceBoolValue(6, 0x40000000), MachineItem::Solo(6)),
        (SurfaceBoolValue(6, 0x20000000), MachineItem::Solo(5)),
        (SurfaceBoolValue(6, 0x10000000), MachineItem::Solo(4)),
        (SurfaceBoolValue(6, 0x08000000), MachineItem::Solo(3)),
        (SurfaceBoolValue(6, 0x04000000), MachineItem::Solo(2)),
        (SurfaceBoolValue(6, 0x02000000), MachineItem::Solo(1)),
        (SurfaceBoolValue(6, 0x01000000), MachineItem::Solo(0)),
        (SurfaceBoolValue(6, 0x00800000), MachineItem::Select(7)),
        (SurfaceBoolValue(6, 0x00400000), MachineItem::Select(6)),
        (SurfaceBoolValue(6, 0x00200000), MachineItem::Select(5)),
        (SurfaceBoolValue(6, 0x00100000), MachineItem::Select(4)),
        (SurfaceBoolValue(6, 0x00080000), MachineItem::Select(3)),
        (SurfaceBoolValue(6, 0x00040000), MachineItem::Select(2)),
        (SurfaceBoolValue(6, 0x00020000), MachineItem::Select(1)),
        (SurfaceBoolValue(6, 0x00010000), MachineItem::Select(0)),
        (SurfaceBoolValue(7, 0x00000080), MachineItem::Mute(7)),
        (SurfaceBoolValue(7, 0x00000040), MachineItem::Mute(6)),
        (SurfaceBoolValue(7, 0x00000020), MachineItem::Mute(5)),
        (SurfaceBoolValue(7, 0x00000010), MachineItem::Mute(4)),
        (SurfaceBoolValue(7, 0x00000008), MachineItem::Mute(3)),
        (SurfaceBoolValue(7, 0x00000004), MachineItem::Mute(2)),
        (SurfaceBoolValue(7, 0x00000002), MachineItem::Mute(1)),
        (SurfaceBoolValue(7, 0x00000001), MachineItem::Mute(0)),
        (SurfaceBoolValue(9, 0x00800000), MachineItem::Shuttle),
        (SurfaceBoolValue(9, 0x00000800), MachineItem::Low),
        (SurfaceBoolValue(9, 0x00000400), MachineItem::LowMid),
        (SurfaceBoolValue(9, 0x00000200), MachineItem::HighMid),
        (SurfaceBoolValue(9, 0x00000100), MachineItem::High),
    ];

    const STATELESS_ITEMS: &'static [(SurfaceBoolValue, MachineItem)] = &[
        (SurfaceBoolValue(7, 0x20000000), MachineItem::Copy),
        (SurfaceBoolValue(7, 0x10000000), MachineItem::Cut),
        (SurfaceBoolValue(7, 0x01000000), MachineItem::Panel),
        (SurfaceBoolValue(7, 0x00080000), MachineItem::Pan),
        (SurfaceBoolValue(7, 0x00010000), MachineItem::Flip),
        (SurfaceBoolValue(8, 0x10000000), MachineItem::Clock),
        (SurfaceBoolValue(8, 0x02000000), MachineItem::Computer),
        (SurfaceBoolValue(8, 0x01000000), MachineItem::Pfl),
        (SurfaceBoolValue(8, 0x00000040), MachineItem::Ctrl),
        (SurfaceBoolValue(8, 0x00000020), MachineItem::Undo),
        (SurfaceBoolValue(8, 0x00000010), MachineItem::Paste),
        (SurfaceBoolValue(8, 0x00000008), MachineItem::Del),
        (SurfaceBoolValue(9, 0x04000000), MachineItem::Out),
        (SurfaceBoolValue(9, 0x02000000), MachineItem::In),
        (SurfaceBoolValue(9, 0x01000000), MachineItem::Set),
        (SurfaceBoolValue(9, 0x00400000), MachineItem::LocateRight),
        (SurfaceBoolValue(9, 0x00200000), MachineItem::LocateLeft),
        (SurfaceBoolValue(9, 0x00040000), MachineItem::NudgeRight),
        (SurfaceBoolValue(9, 0x00020000), MachineItem::NudgeLeft),
        (SurfaceBoolValue(9, 0x00010000), MachineItem::Recall),
        (SurfaceBoolValue(9, 0x00008000), MachineItem::Right),
        (SurfaceBoolValue(9, 0x00004000), MachineItem::Down),
        (SurfaceBoolValue(9, 0x00002000), MachineItem::Left),
        (SurfaceBoolValue(9, 0x00001000), MachineItem::Up),
        (SurfaceBoolValue(9, 0x00000080), MachineItem::Latch),
        (SurfaceBoolValue(9, 0x00000040), MachineItem::Tch),
        (SurfaceBoolValue(9, 0x00000020), MachineItem::Wrt),
        (SurfaceBoolValue(9, 0x00000010), MachineItem::Read),
        (SurfaceBoolValue(9, 0x00000008), MachineItem::Func(9)),
        (SurfaceBoolValue(9, 0x00000004), MachineItem::Func(8)),
        (SurfaceBoolValue(9, 0x00000002), MachineItem::Func(7)),
        (SurfaceBoolValue(9, 0x00000001), MachineItem::Func(6)),
        (SurfaceBoolValue(9, 0x80000000), MachineItem::Record),
        (SurfaceBoolValue(9, 0x40000000), MachineItem::Play),
        (SurfaceBoolValue(9, 0x20000000), MachineItem::Stop),
        (SurfaceBoolValue(9, 0x10000000), MachineItem::Fwd),
        (SurfaceBoolValue(9, 0x08000000), MachineItem::Rew),
    ];

    const ROTARIES: &'static [(SurfaceU16Value, MachineItem)] = &[
        (SurfaceU16Value(10, 0x0000ffff, 0), MachineItem::Rotary(0)),
        (SurfaceU16Value(10, 0xffff0000, 16), MachineItem::Rotary(1)),
        (SurfaceU16Value(11, 0x0000ffff, 0), MachineItem::Rotary(2)),
        (SurfaceU16Value(11, 0xffff0000, 16), MachineItem::Rotary(3)),
        (SurfaceU16Value(12, 0x0000ffff, 0), MachineItem::Rotary(4)),
        (SurfaceU16Value(12, 0xffff0000, 16), MachineItem::Rotary(5)),
        (SurfaceU16Value(13, 0x0000ffff, 0), MachineItem::Rotary(6)),
        (SurfaceU16Value(13, 0xffff0000, 16), MachineItem::Rotary(7)),
        (SurfaceU16Value(14, 0x0000ffff, 0), MachineItem::Gain),
        (SurfaceU16Value(14, 0xffff0000, 16), MachineItem::Freq),
        (SurfaceU16Value(15, 0x0000ffff, 0), MachineItem::Q),
        (SurfaceU16Value(15, 0xffff0000, 16), MachineItem::Wheel),
    ];

    const FADERS: &'static [(SurfaceBoolValue, SurfaceU16Value, MachineItem)] = &[
        (
            SurfaceBoolValue(5, 0x00010000),
            SurfaceU16Value(0, 0x0000ffff, 0),
            MachineItem::Input(0),
        ),
        (
            SurfaceBoolValue(5, 0x00020000),
            SurfaceU16Value(0, 0x0ffff000, 16),
            MachineItem::Input(1),
        ),
        (
            SurfaceBoolValue(5, 0x00040000),
            SurfaceU16Value(1, 0x0000ffff, 0),
            MachineItem::Input(2),
        ),
        (
            SurfaceBoolValue(5, 0x00080000),
            SurfaceU16Value(1, 0xffff0000, 16),
            MachineItem::Input(3),
        ),
        (
            SurfaceBoolValue(5, 0x00100000),
            SurfaceU16Value(2, 0x0000ffff, 0),
            MachineItem::Input(4),
        ),
        (
            SurfaceBoolValue(5, 0x00200000),
            SurfaceU16Value(2, 0xffff0000, 16),
            MachineItem::Input(5),
        ),
        (
            SurfaceBoolValue(5, 0x00400000),
            SurfaceU16Value(3, 0x0000ffff, 0),
            MachineItem::Input(6),
        ),
        (
            SurfaceBoolValue(5, 0x00800000),
            SurfaceU16Value(3, 0xffff0000, 16),
            MachineItem::Input(7),
        ),
        (
            SurfaceBoolValue(5, 0x01000000),
            SurfaceU16Value(4, 0x0000ffff, 0),
            MachineItem::Master,
        ),
    ];
}

impl SurfaceImageIsochOperation for Fw1884Protocol {
    const SHIFT_ITEM: SurfaceBoolValue = SurfaceBoolValue(7, 0x80000000);

    const SHIFTED_ITEMS: &'static [(SurfaceBoolValue, [MachineItem; 2])] = &[
        (
            SurfaceBoolValue(7, 0x40000000),
            [MachineItem::Alt, MachineItem::Cmd],
        ),
        (
            SurfaceBoolValue(7, 0x08000000),
            [MachineItem::Markers, MachineItem::Func(4)],
        ),
        (
            SurfaceBoolValue(7, 0x04000000),
            [MachineItem::AllSafe, MachineItem::Func(2)],
        ),
        (
            SurfaceBoolValue(7, 0x02000000),
            [MachineItem::Save, MachineItem::Func(0)],
        ),
        (
            SurfaceBoolValue(8, 0x00000004),
            [MachineItem::Loop, MachineItem::Func(5)],
        ),
        (
            SurfaceBoolValue(8, 0x00000002),
            [MachineItem::ClrSolo, MachineItem::Func(3)],
        ),
        (
            SurfaceBoolValue(8, 0x00000001),
            [MachineItem::Revert, MachineItem::Func(1)],
        ),
    ];

    const BANK_CURSORS: [SurfaceBoolValue; 2] = [
        SurfaceBoolValue(9, 0x00080000),
        SurfaceBoolValue(9, 0x00100000),
    ];
}

impl FireWireLedOperation for Fw1884Protocol {
    const POSITIONS: &'static [u16] = &[0x8e];
}

impl SurfaceNormalLedOperation for Fw1884Protocol {
    const NORMAL_LEDS: &'static [(&'static [MachineItem], &'static [u16])] = &[
        (&[MachineItem::Ol(0)], &[3]),
        (&[MachineItem::Ol(1)], &[22]),
        (&[MachineItem::Ol(2)], &[35]),
        (&[MachineItem::Ol(3)], &[54]),
        (&[MachineItem::Ol(4)], &[67, 131]),
        (&[MachineItem::Ol(5)], &[86, 150, 163]),
        (&[MachineItem::Ol(6)], &[99, 182, 195]),
        (&[MachineItem::Ol(7)], &[118, 214, 227]),
        (&[MachineItem::Rec(0)], &[5]),
        (&[MachineItem::Rec(1)], &[24]),
        (&[MachineItem::Rec(2)], &[37]),
        (&[MachineItem::Rec(3)], &[56]),
        (&[MachineItem::Rec(4)], &[69, 133]),
        (&[MachineItem::Rec(5)], &[88, 152, 165]),
        (&[MachineItem::Rec(6)], &[101, 184, 197]),
        (&[MachineItem::Rec(7)], &[120, 216, 229]),
        (&[MachineItem::Func(6)], &[11]),
        (&[MachineItem::Func(7)], &[30, 43]),
        (&[MachineItem::Func(8)], &[62, 75]),
        (&[MachineItem::Func(9)], &[94, 107]),
        (&[MachineItem::Select(0)], &[0]),
        (&[MachineItem::Select(1)], &[19, 32]),
        (&[MachineItem::Select(2)], &[51, 64]),
        (&[MachineItem::Select(3)], &[83, 96]),
        (&[MachineItem::Select(4)], &[115, 128]),
        (&[MachineItem::Select(5)], &[147, 160]),
        (&[MachineItem::Select(6)], &[179, 192]),
        (&[MachineItem::Select(7)], &[211, 224]),
        (&[MachineItem::Solo(0)], &[1]),
        (&[MachineItem::Solo(1)], &[20, 33]),
        (&[MachineItem::Solo(2)], &[52, 65]),
        (&[MachineItem::Solo(3)], &[84, 97]),
        (&[MachineItem::Solo(4)], &[116, 129]),
        (&[MachineItem::Solo(5)], &[148, 161]),
        (&[MachineItem::Solo(6)], &[180, 193]),
        (&[MachineItem::Solo(7)], &[212, 225]),
        (&[MachineItem::Mute(0)], &[2]),
        (&[MachineItem::Mute(1)], &[21, 34]),
        (&[MachineItem::Mute(2)], &[53, 66]),
        (&[MachineItem::Mute(3)], &[85, 98]),
        (&[MachineItem::Mute(4)], &[117, 130]),
        (&[MachineItem::Mute(5)], &[149, 162]),
        (&[MachineItem::Mute(6)], &[181, 194]),
        (&[MachineItem::Mute(7)], &[213, 226]),
        (&[MachineItem::Aux(0)], &[155, 168]),
        (&[MachineItem::Aux(1)], &[89, 102]),
        (&[MachineItem::Aux(2)], &[156, 169]),
        (&[MachineItem::Aux(3)], &[91, 104]),
        (&[MachineItem::Aux(4)], &[188, 200]),
        (&[MachineItem::Aux(5)], &[92, 105]),
        (&[MachineItem::Aux(6)], &[187, 201]),
        (&[MachineItem::Aux(7)], &[6]),
        (&[MachineItem::Save, MachineItem::Func(0)], &[122, 135]),
        (&[MachineItem::Revert, MachineItem::Func(1)], &[7]),
        (&[MachineItem::AllSafe, MachineItem::Func(2)], &[123, 136]),
        (&[MachineItem::ClrSolo, MachineItem::Func(3)], &[8]),
        (&[MachineItem::Markers, MachineItem::Func(4)], &[124, 137]),
        (&[MachineItem::Loop, MachineItem::Func(5)], &[9]),
        (&[MachineItem::Del], &[25, 38]),
        (&[MachineItem::Paste], &[26, 39]),
        (&[MachineItem::Undo], &[27, 40]),
        (&[MachineItem::Ctrl], &[28, 41]),
        (&[MachineItem::Cut], &[57, 70]),
        (&[MachineItem::Copy], &[58, 71]),
        (&[MachineItem::Alt], &[59, 72]),
        (&[MachineItem::Shift], &[60, 73]),
        (&[MachineItem::Pan], &[90, 103]),
        (&[MachineItem::Read], &[126, 139]),
        (&[MachineItem::Flip], &[154, 167]),
        (&[MachineItem::Wrt], &[158, 171]),
        (&[MachineItem::Tch], &[190, 203]),
        (&[MachineItem::Latch], &[222, 235]),
        (&[MachineItem::High], &[12]),
        (&[MachineItem::HighMid], &[31, 44]),
        (&[MachineItem::LowMid], &[63, 76]),
        (&[MachineItem::Low], &[95, 108]),
        (&[MachineItem::Shuttle], &[77]),
        (&[MachineItem::Rew], &[13]),
        (&[MachineItem::Play], &[17]),
        (&[MachineItem::Fwd], &[45]),
        (&[MachineItem::Record], &[146]),
        (&[MachineItem::Stop], &[242]),
    ];
}

impl SurfaceBankLedOperation for Fw1884Protocol {
    const BANK_LEDS: [&'static [u16]; 4] = [&[127, 140], &[159, 172], &[191, 204], &[223, 236]];
}
