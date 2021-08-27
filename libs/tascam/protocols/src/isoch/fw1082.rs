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

impl IsochConsoleOperation for Fw1082Protocol {}

impl MachineStateOperation for Fw1082Protocol {
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
        MachineItem::Shuttle,
        MachineItem::Up,
        MachineItem::Left,
        MachineItem::Down,
        MachineItem::Right,
        MachineItem::LocateLeft,
        MachineItem::LocateRight,
        MachineItem::Set,
        MachineItem::In,
        MachineItem::Out,
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
        MachineItem::Shift,
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
        MachineItem::Pan,
        MachineItem::EncoderMode,
    ];

    const HAS_TRANSPORT: bool = true;
    const HAS_BANK: bool = true;
}

/// The structure for state of control surface in FW-1082.
#[derive(Default, Debug)]
pub struct Fw1082SurfaceState;

impl SurfaceImageOperation<Fw1082SurfaceState> for Fw1082Protocol {
    fn initialize_surface_state(_: &mut Fw1082SurfaceState) {}

    fn decode_surface_image(
        _: &Fw1082SurfaceState,
        _: &[u32],
        _: u32,
        _: u32,
        _: u32,
    ) -> Vec<(MachineItem, ItemValue)> {
        Vec::new()
    }

    fn feedback_to_surface(
        _: &mut Fw1082SurfaceState,
        _: &(MachineItem, ItemValue),
        _: &mut FwReq,
        _: &mut FwNode,
        _: u32,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn finalize_surface(
        _: &mut Fw1082SurfaceState,
        _: &mut FwReq,
        _: &mut FwNode,
        _: u32,
    ) -> Result<(), Error> {
        Ok(())
    }
}
