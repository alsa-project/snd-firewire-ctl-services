// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol implementation for FE-8
//!
//! The module includes protocol implementation defined by Tascam for FE-8.
//!
//! The protocol implementation for Tascam FE-8 was written with firmware version below:
//!
//! ```text
//! $ cargo run --bin tascam-hardware-information /dev/fw1
//!
//! Hardware information:
//!   Register: 0x00010014
//!   FPGA:     0x00010031
//!   ARM:      0x000100ad
//!   Hardware: 0x00020000
//! ```

use super::*;

#[derive(Default)]
pub struct Fe8Protocol;

impl ExpanderOperation for Fe8Protocol {}

impl MachineStateOperation for Fe8Protocol {
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
    ];

    const U16_ITEMS: &'static [MachineItem] = &[
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
    ];

    const HAS_TRANSPORT: bool = false;
    const HAS_BANK: bool = false;
}

/// The structure for state of control surface in FE-8.
#[derive(Default, Debug)]
pub struct Fe8SurfaceState {
    common: SurfaceCommonState,
    led_state: LedState,
}

impl SurfaceImageOperation<Fe8SurfaceState> for Fe8Protocol {
    fn initialize_surface_state(state: &mut Fe8SurfaceState) {
        Self::initialize_surface_common_state(&mut state.common);
    }

    fn decode_surface_image(
        state: &Fe8SurfaceState,
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

        machine_values
    }

    fn feedback_to_surface(
        state: &mut Fe8SurfaceState,
        machine_value: &(MachineItem, ItemValue),
        req: &mut FwReq,
        node: &mut FwNode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        Self::feedback_to_surface_common(&mut state.common, machine_value);

        if let ItemValue::Bool(value) = machine_value.1 {
            if let Some(pos) = Self::find_normal_led_pos(&machine_value.0) {
                operate_led_cached(&mut state.led_state, req, node, pos, value, timeout_ms)?;
            }
        }

        Ok(())
    }

    fn finalize_surface(
        state: &mut Fe8SurfaceState,
        req: &mut FwReq,
        node: &mut FwNode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        clear_leds(&mut state.led_state, req, node, timeout_ms)
    }
}

impl SurfaceImageCommonOperation for Fe8Protocol {
    const STATEFUL_ITEMS: &'static [(SurfaceBoolValue, MachineItem)] = &[
        (SurfaceBoolValue(13, 0x00008000), MachineItem::Solo(7)),
        (SurfaceBoolValue(13, 0x00004000), MachineItem::Solo(6)),
        (SurfaceBoolValue(13, 0x00002000), MachineItem::Solo(5)),
        (SurfaceBoolValue(13, 0x00001000), MachineItem::Solo(4)),
        (SurfaceBoolValue(13, 0x00000800), MachineItem::Solo(3)),
        (SurfaceBoolValue(13, 0x00000400), MachineItem::Solo(2)),
        (SurfaceBoolValue(13, 0x00000200), MachineItem::Solo(1)),
        (SurfaceBoolValue(13, 0x00000100), MachineItem::Solo(0)),
        (SurfaceBoolValue(13, 0x00000008), MachineItem::Select(3)),
        (SurfaceBoolValue(13, 0x00000004), MachineItem::Select(2)),
        (SurfaceBoolValue(13, 0x00000002), MachineItem::Select(1)),
        (SurfaceBoolValue(13, 0x00000001), MachineItem::Select(0)),
        (SurfaceBoolValue(13, 0x00000080), MachineItem::Select(7)),
        (SurfaceBoolValue(13, 0x00000040), MachineItem::Select(6)),
        (SurfaceBoolValue(13, 0x00000020), MachineItem::Select(5)),
        (SurfaceBoolValue(13, 0x00000010), MachineItem::Select(4)),
        (SurfaceBoolValue(14, 0x00000008), MachineItem::Mute(3)),
        (SurfaceBoolValue(14, 0x00000004), MachineItem::Mute(2)),
        (SurfaceBoolValue(14, 0x00000002), MachineItem::Mute(1)),
        (SurfaceBoolValue(14, 0x00000001), MachineItem::Mute(0)),
        (SurfaceBoolValue(14, 0x00000080), MachineItem::Mute(7)),
        (SurfaceBoolValue(14, 0x00000040), MachineItem::Mute(6)),
        (SurfaceBoolValue(14, 0x00000020), MachineItem::Mute(5)),
        (SurfaceBoolValue(14, 0x00000010), MachineItem::Mute(4)),
    ];

    const STATELESS_ITEMS: &'static [(SurfaceBoolValue, MachineItem)] = &[];

    const ROTARIES: &'static [(SurfaceU16Value, MachineItem)] = &[
        (SurfaceU16Value(20, 0x0000ffff, 0), MachineItem::Rotary(0)),
        (SurfaceU16Value(21, 0x0000ffff, 0), MachineItem::Rotary(1)),
        (SurfaceU16Value(22, 0x0000ffff, 0), MachineItem::Rotary(2)),
        (SurfaceU16Value(23, 0x0000ffff, 0), MachineItem::Rotary(3)),
        (SurfaceU16Value(24, 0x0000ffff, 0), MachineItem::Rotary(4)),
        (SurfaceU16Value(25, 0x0000ffff, 0), MachineItem::Rotary(5)),
        (SurfaceU16Value(26, 0x0000ffff, 0), MachineItem::Rotary(6)),
        (SurfaceU16Value(27, 0x0000ffff, 0), MachineItem::Rotary(7)),
    ];

    const FADERS: &'static [(SurfaceBoolValue, SurfaceU16Value, MachineItem)] = &[
        (
            SurfaceBoolValue(11, 0x00000001),
            SurfaceU16Value(0, 0x0000ffff, 0),
            MachineItem::Input(0),
        ),
        (
            SurfaceBoolValue(11, 0x00000002),
            SurfaceU16Value(1, 0x0000ffff, 0),
            MachineItem::Input(1),
        ),
        (
            SurfaceBoolValue(11, 0x00000004),
            SurfaceU16Value(2, 0x0000ffff, 0),
            MachineItem::Input(2),
        ),
        (
            SurfaceBoolValue(11, 0x00000008),
            SurfaceU16Value(3, 0x0000ffff, 0),
            MachineItem::Input(3),
        ),
        (
            SurfaceBoolValue(11, 0x00000010),
            SurfaceU16Value(4, 0x0000ffff, 0),
            MachineItem::Input(4),
        ),
        (
            SurfaceBoolValue(11, 0x00000020),
            SurfaceU16Value(5, 0x0000ffff, 0),
            MachineItem::Input(5),
        ),
        (
            SurfaceBoolValue(11, 0x00000040),
            SurfaceU16Value(6, 0x0000ffff, 0),
            MachineItem::Input(6),
        ),
        (
            SurfaceBoolValue(11, 0x00000080),
            SurfaceU16Value(7, 0x0000ffff, 0),
            MachineItem::Input(7),
        ),
    ];
}

impl FireWireLedOperation for Fe8Protocol {
    const POSITIONS: &'static [u16] = &[0x16, 0x8e];
}

impl SurfaceNormalLedOperation for Fe8Protocol {
    const NORMAL_LEDS: &'static [(&'static [MachineItem], &'static [u16])] = &[
        (&[MachineItem::Rec(0)], &[0x05]),
        (&[MachineItem::Rec(1)], &[0x18, 0x25]),
        (&[MachineItem::Rec(2)], &[0x38, 0x45]),
        (&[MachineItem::Rec(3)], &[0x58, 0x65]),
        (&[MachineItem::Rec(4)], &[0x76, 0x82]),
        (&[MachineItem::Rec(5)], &[0x98, 0xa5]),
        (&[MachineItem::Rec(6)], &[0xb8, 0xc5]),
        (&[MachineItem::Rec(7)], &[0xd8, 0xe5]),
        (&[MachineItem::Select(0)], &[0x00]),
        (&[MachineItem::Select(1)], &[0x13, 0x20]),
        (&[MachineItem::Select(2)], &[0x33, 0x40]),
        (&[MachineItem::Select(3)], &[0x53, 0x60]),
        (&[MachineItem::Select(4)], &[0x73, 0x80]),
        (&[MachineItem::Select(5)], &[0x93, 0xa0]),
        (&[MachineItem::Select(6)], &[0xb3, 0xc0]),
        (&[MachineItem::Select(7)], &[0xd3, 0xe0]),
        (&[MachineItem::Solo(0)], &[0x01]),
        (&[MachineItem::Solo(1)], &[0x14, 0x21]),
        (&[MachineItem::Solo(2)], &[0x34, 0x41]),
        (&[MachineItem::Solo(3)], &[0x54, 0x61]),
        (&[MachineItem::Solo(4)], &[0x74, 0x81]),
        (&[MachineItem::Solo(5)], &[0x94, 0xa1]),
        (&[MachineItem::Solo(6)], &[0xb4, 0xc1]),
        (&[MachineItem::Solo(7)], &[0xd4, 0xe1]),
        (&[MachineItem::Mute(0)], &[0x02]),
        (&[MachineItem::Mute(1)], &[0x15, 0x22]),
        (&[MachineItem::Mute(2)], &[0x35, 0x42]),
        (&[MachineItem::Mute(3)], &[0x55, 0x62]),
        (&[MachineItem::Mute(4)], &[0x75, 0x82]),
        (&[MachineItem::Mute(5)], &[0x95, 0xa2]),
        (&[MachineItem::Mute(6)], &[0xb5, 0xc2]),
        (&[MachineItem::Mute(7)], &[0xd5, 0xe2]),
    ];
}
