// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol implementation for FW-1082.
//!
//! The module includes protocol implementation defined by Tascam for FW-1082.
//!
//! The protocol implementation for Tascam FW-1082 was written with firmware version below:
//!
//! ```text
//! $ cargo run --bin tascam-hardware-information /dev/fw1
//!
//! Hardware information:
//!   Register: 0x00030014
//!   FPGA:     0x00010031
//!   ARM:      0x000100b7
//!   Hardware: 0x00050000
//! ```

use super::*;

/// The protocol implementation of FW-1082.
#[derive(Default)]
pub struct Fw1082Protocol;

impl TascamIsochClockSpecification for Fw1082Protocol {
    const SAMPLING_CLOCK_SOURCES: &'static [ClkSrc] = &[ClkSrc::Internal, ClkSrc::Spdif];
}

impl TascamIsochInputDetectionSpecification for Fw1082Protocol {}

impl TascamIsochCoaxialOutputSpecification for Fw1082Protocol {}

impl IsochMeterOperation for Fw1082Protocol {
    const INPUT_COUNT: usize = 10;
    const OUTPUT_COUNT: usize = 4;
    const HAS_SOLO: bool = true;
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

/// State of control surface in FW-1082.
#[derive(Default, Debug)]
pub struct Fw1082SurfaceState {
    common: SurfaceCommonState,
    isoch: SurfaceIsochState,
    specific: SurfaceSpecificState,
    led_state: LedState,
}

impl SurfaceImageOperation<Fw1082SurfaceState> for Fw1082Protocol {
    fn initialize_surface_state(state: &mut Fw1082SurfaceState) {
        Self::initialize_surface_common_state(&mut state.common);
        Self::initialize_surface_isoch_state(&mut state.isoch);
        Self::initialize_surface_specific_state(&mut state.specific);
    }

    fn decode_surface_image(
        state: &Fw1082SurfaceState,
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

        Self::decode_surface_image_specific(
            &mut machine_values,
            &state.specific,
            index,
            before,
            after,
        );

        machine_values
    }

    fn feedback_to_surface(
        state: &mut Fw1082SurfaceState,
        machine_value: &(MachineItem, ItemValue),
        req: &mut FwReq,
        node: &mut FwNode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        Self::feedback_to_surface_common(&mut state.common, machine_value);
        Self::feedback_to_surface_isoch(&mut state.isoch, machine_value);
        Self::feedback_to_surface_specific(&mut state.specific, machine_value);

        if let ItemValue::Bool(value) = machine_value.1 {
            if let Some(pos) = Self::find_normal_led_pos(&machine_value.0) {
                operate_led_cached(&mut state.led_state, req, node, pos, value, timeout_ms)?;
            } else {
                let idx = Self::ENCODER_MODES
                    .iter()
                    .position(|(_, m)| m.eq(&state.specific.mode))
                    .unwrap();
                let led_state = &mut state.led_state;
                Self::ENCODER_ITEM_LEDS
                    .iter()
                    .zip(&mut state.specific.button_states)
                    .try_for_each(|((items, positions), s)| {
                        items
                            .iter()
                            .zip(s)
                            .find(|(item, _)| machine_value.0.eq(item))
                            .map(|(_, s)| *s = value);

                        if machine_value.0.eq(&items[idx]) {
                            operate_led_cached(
                                led_state,
                                req,
                                node,
                                positions[0],
                                value,
                                timeout_ms,
                            )
                        } else {
                            Ok(())
                        }
                    })?;
            }
        } else if let ItemValue::U16(value) = machine_value.1 {
            if machine_value.0 == MachineItem::Bank {
                Self::operate_bank_leds(&mut state.led_state, req, node, value, timeout_ms)?;
            }
            if machine_value.0 == MachineItem::EncoderMode {
                // One of encode modes should be activated.
                Self::ENCODER_MODE_LEDS
                    .iter()
                    .try_for_each(|(m, positions)| {
                        operate_led_cached(
                            &mut state.led_state,
                            req,
                            node,
                            positions[0],
                            m.eq(&state.specific.mode),
                            timeout_ms,
                        )
                    })?;

                // Recover the state of button LEDs.
                let idx = Self::ENCODER_MODES
                    .iter()
                    .position(|(_, m)| m.eq(&state.specific.mode))
                    .unwrap();
                let led_state = &mut state.led_state;
                Self::ENCODER_ITEM_LEDS
                    .iter()
                    .zip(state.specific.button_states)
                    .try_for_each(|((_, positions), s)| {
                        operate_led_cached(led_state, req, node, positions[0], s[idx], timeout_ms)
                    })?;
            }
        }

        Ok(())
    }

    fn finalize_surface(
        state: &mut Fw1082SurfaceState,
        req: &mut FwReq,
        node: &mut FwNode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        clear_leds(&mut state.led_state, req, node, timeout_ms)
    }
}

impl SurfaceImageCommonOperation for Fw1082Protocol {
    const STATEFUL_ITEMS: &'static [(SurfaceBoolValue, MachineItem)] = &[
        (SurfaceBoolValue(6, 0x00800000), MachineItem::Select(7)),
        (SurfaceBoolValue(6, 0x00400000), MachineItem::Select(6)),
        (SurfaceBoolValue(6, 0x00200000), MachineItem::Select(5)),
        (SurfaceBoolValue(6, 0x00100000), MachineItem::Select(4)),
        (SurfaceBoolValue(6, 0x00080000), MachineItem::Select(3)),
        (SurfaceBoolValue(6, 0x00040000), MachineItem::Select(2)),
        (SurfaceBoolValue(6, 0x00020000), MachineItem::Select(1)),
        (SurfaceBoolValue(6, 0x00010000), MachineItem::Select(0)),
        (SurfaceBoolValue(6, 0x80000000), MachineItem::Solo(7)),
        (SurfaceBoolValue(6, 0x40000000), MachineItem::Solo(6)),
        (SurfaceBoolValue(6, 0x20000000), MachineItem::Solo(5)),
        (SurfaceBoolValue(6, 0x10000000), MachineItem::Solo(4)),
        (SurfaceBoolValue(6, 0x08000000), MachineItem::Solo(3)),
        (SurfaceBoolValue(6, 0x04000000), MachineItem::Solo(2)),
        (SurfaceBoolValue(6, 0x02000000), MachineItem::Solo(1)),
        (SurfaceBoolValue(6, 0x01000000), MachineItem::Solo(0)),
        (SurfaceBoolValue(7, 0x00000080), MachineItem::Mute(7)),
        (SurfaceBoolValue(7, 0x00000040), MachineItem::Mute(6)),
        (SurfaceBoolValue(7, 0x00000020), MachineItem::Mute(5)),
        (SurfaceBoolValue(7, 0x00000010), MachineItem::Mute(4)),
        (SurfaceBoolValue(7, 0x00000008), MachineItem::Mute(3)),
        (SurfaceBoolValue(7, 0x00000004), MachineItem::Mute(2)),
        (SurfaceBoolValue(7, 0x00000002), MachineItem::Mute(1)),
        (SurfaceBoolValue(7, 0x00000001), MachineItem::Mute(0)),
        (SurfaceBoolValue(9, 0x00800000), MachineItem::Shuttle),
    ];

    const STATELESS_ITEMS: &'static [(SurfaceBoolValue, MachineItem)] = &[
        (SurfaceBoolValue(9, 0x80000000), MachineItem::Record),
        (SurfaceBoolValue(9, 0x40000000), MachineItem::Play),
        (SurfaceBoolValue(9, 0x20000000), MachineItem::Stop),
        (SurfaceBoolValue(9, 0x10000000), MachineItem::Fwd),
        (SurfaceBoolValue(9, 0x08000000), MachineItem::Rew),
        (SurfaceBoolValue(8, 0x10000000), MachineItem::Panel),
        (SurfaceBoolValue(9, 0x04000000), MachineItem::Out),
        (SurfaceBoolValue(9, 0x02000000), MachineItem::In),
        (SurfaceBoolValue(9, 0x01000000), MachineItem::Set),
        (SurfaceBoolValue(9, 0x00400000), MachineItem::LocateRight),
        (SurfaceBoolValue(9, 0x00200000), MachineItem::LocateLeft),
        (SurfaceBoolValue(9, 0x00010000), MachineItem::Recall),
        (SurfaceBoolValue(9, 0x00008000), MachineItem::Right),
        (SurfaceBoolValue(9, 0x00004000), MachineItem::Down),
        (SurfaceBoolValue(9, 0x00002000), MachineItem::Left),
        (SurfaceBoolValue(9, 0x00001000), MachineItem::Up),
    ];

    const ROTARIES: &'static [(SurfaceU16Value, MachineItem)] =
        &[(SurfaceU16Value(15, 0xffff0000, 16), MachineItem::Wheel)];

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

impl SurfaceImageIsochOperation for Fw1082Protocol {
    const SHIFT_ITEM: SurfaceBoolValue = SurfaceBoolValue(8, 0x01000000);

    const SHIFTED_ITEMS: &'static [(SurfaceBoolValue, [MachineItem; 2])] = &[
        (
            SurfaceBoolValue(9, 0x00000008),
            [MachineItem::Func(3), MachineItem::Func(7)],
        ),
        (
            SurfaceBoolValue(9, 0x00000004),
            [MachineItem::Func(2), MachineItem::Func(6)],
        ),
        (
            SurfaceBoolValue(9, 0x00000002),
            [MachineItem::Func(1), MachineItem::Func(5)],
        ),
        (
            SurfaceBoolValue(9, 0x00000001),
            [MachineItem::Func(0), MachineItem::Func(4)],
        ),
    ];

    const BANK_CURSORS: [SurfaceBoolValue; 2] = [
        SurfaceBoolValue(9, 0x00080000),
        SurfaceBoolValue(9, 0x00100000),
    ];
}

/// The mode of encoder items.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Fw1082EncoderMode {
    Equalizer,
    Aux0123,
    Aux4567,
}

impl Default for Fw1082EncoderMode {
    fn default() -> Self {
        Self::Equalizer
    }
}

/// State of surface specific to FW-1082.
#[derive(Default, Debug)]
struct SurfaceSpecificState {
    mode: Fw1082EncoderMode,
    button_states: [[bool; 3]; 4],
}

impl Fw1082Protocol {
    const ENCODER_MODES: [(SurfaceBoolValue, Fw1082EncoderMode); 3] = [
        (
            SurfaceBoolValue(8, 0x20000000),
            Fw1082EncoderMode::Equalizer,
        ),
        (SurfaceBoolValue(8, 0x40000000), Fw1082EncoderMode::Aux0123),
        (SurfaceBoolValue(8, 0x80000000), Fw1082EncoderMode::Aux4567),
    ];

    const ENCODER_BOOL_ITEMS: [(SurfaceBoolValue, [MachineItem; 3]); 4] = [
        (
            SurfaceBoolValue(9, 0x00000800),
            [MachineItem::Low, MachineItem::Aux(3), MachineItem::Aux(7)],
        ),
        (
            SurfaceBoolValue(9, 0x00000400),
            [
                MachineItem::LowMid,
                MachineItem::Aux(2),
                MachineItem::Aux(6),
            ],
        ),
        (
            SurfaceBoolValue(9, 0x00000200),
            [
                MachineItem::HighMid,
                MachineItem::Aux(1),
                MachineItem::Aux(5),
            ],
        ),
        (
            SurfaceBoolValue(9, 0x00000100),
            [MachineItem::High, MachineItem::Aux(0), MachineItem::Aux(4)],
        ),
    ];

    const ENCODER_U16_ITEMS: [(SurfaceU16Value, [MachineItem; 3]); 4] = [
        (
            SurfaceU16Value(14, 0x0000ffff, 0),
            [
                MachineItem::Gain,
                MachineItem::Rotary(0),
                MachineItem::Rotary(4),
            ],
        ),
        (
            SurfaceU16Value(14, 0xffff0000, 16),
            [
                MachineItem::Freq,
                MachineItem::Rotary(1),
                MachineItem::Rotary(5),
            ],
        ),
        (
            SurfaceU16Value(15, 0x0000ffff, 0),
            [
                MachineItem::Q,
                MachineItem::Rotary(2),
                MachineItem::Rotary(6),
            ],
        ),
        (
            SurfaceU16Value(10, 0x0000ffff, 0),
            [
                MachineItem::Pan,
                MachineItem::Rotary(3),
                MachineItem::Rotary(7),
            ],
        ),
    ];

    fn initialize_surface_specific_state(state: &mut SurfaceSpecificState) {
        state.mode = Fw1082EncoderMode::Equalizer;
    }

    fn decode_surface_image_specific(
        machine_values: &mut Vec<(MachineItem, ItemValue)>,
        state: &SurfaceSpecificState,
        index: u32,
        before: u32,
        after: u32,
    ) {
        let mut curr_mode = state.mode;

        // One of encoder modes should be enabled always.
        Self::ENCODER_MODES
            .iter()
            .enumerate()
            .filter(|(_, (bool_val, _))| detect_bool_action(bool_val, index, before, after))
            .for_each(|(idx, (bool_val, mode))| {
                let push_event = detect_bool_value(bool_val, before);
                if push_event {
                    curr_mode = *mode;
                    machine_values.push((MachineItem::EncoderMode, ItemValue::U16(idx as u16)));
                }
            });

        let idx = Self::ENCODER_MODES
            .iter()
            .position(|(_, m)| curr_mode.eq(m))
            .unwrap();

        Self::ENCODER_BOOL_ITEMS
            .iter()
            .zip(state.button_states)
            .filter(|((bool_val, _), _)| {
                detect_stateful_bool_action(bool_val, index, before, after)
            })
            .for_each(|((_, items), s)| {
                machine_values.push((items[idx], ItemValue::Bool(!s[idx])));
            });

        Self::ENCODER_U16_ITEMS
            .iter()
            .filter(|(u16_val, _)| detect_u16_action(u16_val, index, before, after))
            .for_each(|(u16_val, items)| {
                let value = detect_u16_value(u16_val, after);
                machine_values.push((items[idx], ItemValue::U16(value)));
            });
    }

    fn feedback_to_surface_specific(
        state: &mut SurfaceSpecificState,
        machine_value: &(MachineItem, ItemValue),
    ) {
        match machine_value.1 {
            ItemValue::Bool(value) => {
                Self::ENCODER_BOOL_ITEMS
                    .iter()
                    .zip(&mut state.button_states)
                    .for_each(|((_, items), s)| {
                        let _ = items
                            .iter()
                            .zip(s)
                            .find(|(item, _)| machine_value.0.eq(item))
                            .map(|(_, s)| *s = value);
                    });
            }
            ItemValue::U16(value) => {
                if machine_value.0.eq(&MachineItem::EncoderMode) {
                    let _ = Self::ENCODER_MODES
                        .iter()
                        .nth(value as usize)
                        .map(|(_, m)| state.mode = *m);
                }
            }
        }
    }
}

impl FireWireLedOperation for Fw1082Protocol {
    const POSITIONS: &'static [u16] = &[0x8e];
}

impl SurfaceNormalLedOperation for Fw1082Protocol {
    const NORMAL_LEDS: &'static [(&'static [MachineItem], &'static [u16])] = &[
        (&[MachineItem::Rec(0)], &[5]),
        (&[MachineItem::Rec(1)], &[24]),
        (&[MachineItem::Rec(2)], &[37]),
        (&[MachineItem::Rec(3)], &[56]),
        (&[MachineItem::Rec(4)], &[69, 133]),
        (&[MachineItem::Rec(5)], &[88, 152, 165]),
        (&[MachineItem::Rec(6)], &[101, 184, 197]),
        (&[MachineItem::Rec(7)], &[120, 216, 229]),
        (&[MachineItem::Func(0), MachineItem::Func(4)], &[11]),
        (&[MachineItem::Func(1), MachineItem::Func(5)], &[30, 43]),
        (&[MachineItem::Func(2), MachineItem::Func(6)], &[62, 75]),
        (&[MachineItem::Func(3), MachineItem::Func(7)], &[94, 107]),
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
        (&[MachineItem::Shuttle], &[77]),
        (&[MachineItem::Rew], &[13]),
        (&[MachineItem::Fwd], &[45]),
        (&[MachineItem::Stop], &[242]),
        (&[MachineItem::Play], &[17]),
        (&[MachineItem::Record], &[146]),
    ];
}

impl SurfaceBankLedOperation for Fw1082Protocol {
    const BANK_LEDS: [&'static [u16]; 4] = [&[127, 140], &[159, 172], &[191, 204], &[223, 236]];
}

impl Fw1082Protocol {
    const ENCODER_MODE_LEDS: [(Fw1082EncoderMode, &'static [u16]); 3] = [
        (Fw1082EncoderMode::Equalizer, &[157, 170]),
        (Fw1082EncoderMode::Aux0123, &[189, 202]),
        (Fw1082EncoderMode::Aux4567, &[221, 234]),
    ];

    const ENCODER_ITEM_LEDS: [([MachineItem; 3], &'static [u16]); 4] = [
        (
            [MachineItem::Low, MachineItem::Aux(3), MachineItem::Aux(7)],
            &[95, 108],
        ),
        (
            [
                MachineItem::LowMid,
                MachineItem::Aux(2),
                MachineItem::Aux(6),
            ],
            &[63, 76],
        ),
        (
            [
                MachineItem::HighMid,
                MachineItem::Aux(1),
                MachineItem::Aux(5),
            ],
            &[31, 44],
        ),
        (
            [MachineItem::High, MachineItem::Aux(0), MachineItem::Aux(4)],
            &[12],
        ),
    ];
}
