// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol implementation for FE-8
//!
//! The module includes protocol implementation defined by Tascam for FE-8.

use crate::{asynch::*, *};

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
