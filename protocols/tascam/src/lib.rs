// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

#![doc = include_str!("../README.md")]

//! ## Process events for control surface
//!
//! Tascam FW-1884, FW-1082, and FE-8 have control surface, in which the hardware make no
//! superficial change to the surface according to user operation. Instead, the surface notifies
//! the operation to system.
//!
//! In FE-8, the image consists of 32 quadlets. Asynchronous notification is simply sent for one
//! of quadlet to which user operation effects.
//!
//! In FW-1884 and FW-1082, the state of surface is expressed as an image which consists of 64
//! quadlets. One of the quadlets is multiplexed to data block in isochronous packet as well as
//! PCM frame in order.
//!
//! FW-1884 and FW-1082 has shift button to divert some buttons. Furthermore, FW-1082 has some
//! rotaries and buttons which change their role according to encoder mode. The module includes
//! two stuffs to abstract the above design; surface state and machine state. The former is used
//! used to parse surface imageg and detect event and operate LED. The latter is used to monitor
//! current state of each surface item by handling the event, and generate normalized events. It's
//! task of runtime implementation to prepare converter between the machine event and application
//! specific message such as ALSA Sequencer and Open Sound Control.
//!
//! The relationship between the constrol surface, surface state, machine state, and message
//! converter is illustrated in below diagram:
//!
//! ```text
//!                       ++====================================================++
//!                       ||                  Service runtime                   ||
//!                       ||                                                    ||
//! ++==========++  surface image    +---------+   machine event   +---------+  ||
//! ||          || ----------------> |         | ----------------> |         |  ||
//! || surface  ||        ||         | surface |                   | machine |  ||
//! || hardware ||  LED operation    |  state  |   machine event   |  state  |  ||
//! ||          || <---------------- |         | <---------------- |         |  ||
//! ++==========++        ||         +---------+                   +---------+  ||
//!                       ||                                         ^    |     ||
//!                       ||                                      machine event ||
//!                       ||                                         |    v     ||
//!                       ||                                       +---------+  ||
//!                       ||                                       |         |  ||
//!                       ||                                       | message |  ||
//!                       ||                                       |converter|  ||
//!                       ||                                       |         |  ||
//!                       ||                                       +---------+  ||
//!                       ||                                         ^    |     ||
//!                       ||                                   specific message ||
//!                       ||                                         |    |     ||
//!                       ++=========================================|====|=====++
//!                                                                  |    |
//!                                     Inter process communication  |    |
//!                                     (ALSA Sequencer, OSC, etc.)  |    v
//!                                                             ++=============++
//!                                                             || application ||
//!                                                             ++=============++
//! ```

pub mod asynch;
pub mod isoch;

pub mod config_rom;

use {
    glib::{Error, FileError},
    hinawa::{prelude::*, *},
};

const BASE_OFFSET: u64 = 0xffff00000000;
const HW_INFO_REGISTER_OFFSET: u64 = 0x00;
const HW_INFO_FPGA_OFFSET: u64 = 0x04;
const HW_INFO_ARM_OFFSET: u64 = 0x08;
const HW_INFO_HW_OFFSET: u64 = 0x0c;
const LED_OFFSET: u64 = 0x0404;

fn read_quadlet(
    req: &mut FwReq,
    node: &mut FwNode,
    offset: u64,
    frames: &mut [u8],
    timeout_ms: u32,
) -> Result<(), Error> {
    req.transaction(
        node,
        FwTcode::ReadQuadletRequest,
        BASE_OFFSET + offset,
        4,
        frames,
        timeout_ms,
    )
}

fn write_quadlet(
    req: &mut FwReq,
    node: &FwNode,
    offset: u64,
    frames: &mut [u8],
    timeout_ms: u32,
) -> Result<(), Error> {
    req.transaction(
        node,
        FwTcode::WriteQuadletRequest,
        BASE_OFFSET + offset,
        4,
        frames,
        timeout_ms,
    )
}

/// Information of hardware.
#[derive(Debug, Default, Copy, Clone)]
pub struct HardwareInformation {
    pub register: u32,
    pub fpga: u32,
    pub arm: u32,
    pub hardware: u32,
}

/// The protocol implementaion commonly available to Tascam FireWire models.
#[derive(Debug, Default)]
pub struct HardwareInformationProtocol;

/// The trait for oepration of hardware information.
impl HardwareInformationProtocol {
    pub fn read_hardware_information(
        req: &mut FwReq,
        node: &mut FwNode,
        info: &mut HardwareInformation,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut quads = [0; 4];
        read_quadlet(req, node, HW_INFO_REGISTER_OFFSET, &mut quads, timeout_ms)
            .map(|_| info.register = u32::from_be_bytes(quads))?;
        read_quadlet(req, node, HW_INFO_FPGA_OFFSET, &mut quads, timeout_ms)
            .map(|_| info.fpga = u32::from_be_bytes(quads))?;
        read_quadlet(req, node, HW_INFO_ARM_OFFSET, &mut quads, timeout_ms)
            .map(|_| info.arm = u32::from_be_bytes(quads))?;
        read_quadlet(req, node, HW_INFO_HW_OFFSET, &mut quads, timeout_ms)
            .map(|_| info.hardware = u32::from_be_bytes(quads))?;
        Ok(())
    }
}

/// The specification of hardware image.
pub trait TascamHardwareImageSpecification {
    const IMAGE_QUADLET_COUNT: usize;

    fn create_hardware_image() -> Vec<u32> {
        vec![0; Self::IMAGE_QUADLET_COUNT]
    }
}

/// Items of surface.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MachineItem {
    // Channel section.
    Master,
    Ol(usize),
    Rec(usize),
    Signal(usize),
    Rotary(usize),
    Select(usize),
    Solo(usize),
    Mute(usize),
    Input(usize),
    Func(usize),
    Pfl,

    // Global section.
    Read,
    Wrt,
    Tch,
    Latch,
    Wheel,
    Shuttle,
    Computer,
    Clock,
    Up,
    Left,
    Down,
    Right,
    NudgeLeft,
    NudgeRight,
    LocateLeft,
    LocateRight,
    Set,
    In,
    Out,

    // Encoder section.
    Flip,
    Pan, // has bool value in FW-1884, has u16 value in FW-1082.
    Aux(usize),
    EncoderMode, // FW-1082 only.

    // Equalizer section.
    High,
    HighMid,
    LowMid,
    Low,
    Recall,
    Gain,
    Freq,
    Q,

    // Bank section.
    Bank,

    // Transport section.
    Rew,
    Fwd,
    Stop,
    Play,
    Record,

    // Shortcut section.
    Panel,
    Save,
    Revert,
    AllSafe,
    ClrSolo,
    Markers,
    Loop,
    Cut,
    Del,
    Copy,
    Paste,
    Alt,
    Cmd,
    Undo,
    Shift,
    Ctrl,
}

impl Default for MachineItem {
    fn default() -> Self {
        Self::Master
    }
}

impl std::fmt::Display for MachineItem {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Master => write!(f, "master"),
            Self::Ol(ch) => write!(f, "ol {}", ch),
            Self::Rec(ch) => write!(f, "rec {}", ch),
            Self::Signal(ch) => write!(f, "signal {}", ch),
            Self::Rotary(ch) => write!(f, "rotary {}", ch),
            Self::Select(ch) => write!(f, "select {}", ch),
            Self::Solo(ch) => write!(f, "solo {}", ch),
            Self::Mute(ch) => write!(f, "mute {}", ch),
            Self::Input(ch) => write!(f, "input {}", ch),
            Self::Func(ch) => write!(f, "func {}", ch),
            Self::Pfl => write!(f, "pfl"),
            Self::Read => write!(f, "read"),
            Self::Wrt => write!(f, "wrt"),
            Self::Tch => write!(f, "tch"),
            Self::Latch => write!(f, "latch"),
            Self::Wheel => write!(f, "wheel"),
            Self::Shuttle => write!(f, "Shuttle"),
            Self::Computer => write!(f, "computer"),
            Self::Clock => write!(f, "clock"),
            Self::Up => write!(f, "up"),
            Self::Left => write!(f, "left"),
            Self::Down => write!(f, "down"),
            Self::Right => write!(f, "right"),
            Self::NudgeLeft => write!(f, "nudge left"),
            Self::NudgeRight => write!(f, "nudge right"),
            Self::LocateLeft => write!(f, "locate left"),
            Self::LocateRight => write!(f, "locate right"),
            Self::Set => write!(f, "set"),
            Self::In => write!(f, "in"),
            Self::Out => write!(f, "out"),
            Self::Flip => write!(f, "flip"),
            Self::Pan => write!(f, "pan"),
            Self::Aux(ch) => write!(f, "aux {}", ch),
            Self::EncoderMode => write!(f, "encoder model"),
            Self::High => write!(f, "high"),
            Self::HighMid => write!(f, "high-mid"),
            Self::LowMid => write!(f, "low-mid"),
            Self::Low => write!(f, "low"),
            Self::Recall => write!(f, "recall"),
            Self::Gain => write!(f, "gain"),
            Self::Freq => write!(f, "freq"),
            Self::Q => write!(f, "q"),
            Self::Bank => write!(f, "bank"),
            Self::Rew => write!(f, "rew"),
            Self::Fwd => write!(f, "fwd"),
            Self::Stop => write!(f, "stop"),
            Self::Play => write!(f, "play"),
            Self::Record => write!(f, "record"),
            Self::Panel => write!(f, "panel"),
            Self::Save => write!(f, "save"),
            Self::Revert => write!(f, "revert"),
            Self::AllSafe => write!(f, "all safe"),
            Self::ClrSolo => write!(f, "clr solo"),
            Self::Markers => write!(f, "markers"),
            Self::Loop => write!(f, "loop"),
            Self::Cut => write!(f, "cut"),
            Self::Del => write!(f, "del"),
            Self::Copy => write!(f, "copy"),
            Self::Paste => write!(f, "paste"),
            Self::Alt => write!(f, "alt"),
            Self::Cmd => write!(f, "cmd"),
            Self::Undo => write!(f, "undo"),
            Self::Shift => write!(f, "shift"),
            Self::Ctrl => write!(f, "ctrl"),
        }
    }
}

/// The state machine of control surface.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct MachineState {
    /// The boolean value of each item.
    bool_items: Vec<bool>,
    /// The u16 value of each item.
    u16_items: Vec<u16>,
    /// Between 0-3.
    bank: u16,
    /// One of Rew, Fwd, Stop, Play, and Record.
    transport: MachineItem,
}

/// The event of state machine.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ItemValue {
    Bool(bool),
    U16(u16),
}

const BANK_MIN: u16 = 0;
const BANK_MAX: u16 = 3;

/// The trait for operation of state machine.
pub trait MachineStateOperation {
    const BOOL_ITEMS: &'static [MachineItem];
    const U16_ITEMS: &'static [MachineItem];
    const HAS_TRANSPORT: bool;
    const HAS_BANK: bool;

    const BANK_MIN: u16 = BANK_MIN;
    const BANK_MAX: u16 = BANK_MAX;

    const U16_ITEM_MIN: u16 = 0;
    const U16_ITEM_MAX: u16 = 0xffffu16;

    const TRANSPORT_ITEMS: [MachineItem; 5] = [
        MachineItem::Rew,
        MachineItem::Fwd,
        MachineItem::Stop,
        MachineItem::Play,
        MachineItem::Record,
    ];

    const EQ_BAND_ITEMS: [MachineItem; 4] = [
        MachineItem::High,
        MachineItem::HighMid,
        MachineItem::LowMid,
        MachineItem::Low,
    ];

    fn create_machine_state() -> MachineState {
        MachineState {
            bool_items: vec![false; Self::BOOL_ITEMS.len()],
            u16_items: vec![0; Self::U16_ITEMS.len()],
            bank: 0,
            transport: MachineItem::Stop,
        }
    }

    fn get_machine_current_values(state: &MachineState) -> Vec<(MachineItem, ItemValue)> {
        let mut machine_values = Vec::new();

        Self::BOOL_ITEMS
            .iter()
            .zip(&state.bool_items)
            .for_each(|(&item, &value)| machine_values.push((item, ItemValue::Bool(value))));

        Self::U16_ITEMS
            .iter()
            .zip(&state.u16_items)
            .for_each(|(&item, &value)| machine_values.push((item, ItemValue::U16(value))));

        if Self::HAS_BANK {
            machine_values.push((MachineItem::Bank, ItemValue::U16(state.bank)));
        }

        if Self::HAS_TRANSPORT {
            Self::TRANSPORT_ITEMS.iter().for_each(|&item| {
                machine_values.push((item, ItemValue::Bool(item.eq(&state.transport))));
            });
        }

        machine_values
    }

    fn change_machine_value(
        state: &mut MachineState,
        input: &(MachineItem, ItemValue),
    ) -> Vec<(MachineItem, ItemValue)> {
        let mut outputs = Vec::new();

        if let ItemValue::Bool(value) = input.1 {
            // Normal items.
            let _ = Self::BOOL_ITEMS
                .iter()
                .zip(&mut state.bool_items)
                .find(|(i, v)| input.0.eq(i) && !value.eq(v))
                .map(|(_, v)| {
                    *v = value;
                    outputs.push((input.0, ItemValue::Bool(*v)));
                });

            // One of transport items should be enabled.
            if Self::HAS_TRANSPORT
                && Self::TRANSPORT_ITEMS
                    .iter()
                    .find(|i| input.0.eq(i))
                    .is_some()
            {
                if input.0 != state.transport {
                    outputs.push((state.transport, ItemValue::Bool(false)));
                    outputs.push((input.0, ItemValue::Bool(true)));
                    state.transport = input.0;
                }
            }

            // None of, or one of equalizer band items should be enabled.
            if Self::EQ_BAND_ITEMS.iter().find(|i| input.0.eq(i)).is_some() {
                if value {
                    Self::BOOL_ITEMS
                        .iter()
                        .zip(&mut state.bool_items)
                        .filter(|(i, v)| {
                            !input.0.eq(i)
                                && **v
                                && Self::EQ_BAND_ITEMS.iter().find(|item| item.eq(i)).is_some()
                        })
                        .for_each(|(i, v)| {
                            *v = false;
                            outputs.push((*i, ItemValue::Bool(*v)));
                        });
                }
            }
        } else if let ItemValue::U16(value) = input.1 {
            let _ = Self::U16_ITEMS
                .iter()
                .zip(&mut state.u16_items)
                .find(|(i, v)| input.0.eq(i) && !value.eq(v))
                .map(|(_, v)| {
                    *v = value;
                    outputs.push((input.0, ItemValue::U16(*v)));
                });

            if Self::HAS_BANK && input.0 == MachineItem::Bank {
                if state.bank != value && value <= Self::BANK_MAX {
                    state.bank = value;
                    outputs.push((MachineItem::Bank, ItemValue::U16(state.bank)));
                }
            }
        }

        outputs
    }
}

/// The trait to operate LED in surface.
pub trait TascamSurfaceLedOperation<T> {
    fn operate_leds(
        state: &mut T,
        machine_value: &(MachineItem, ItemValue),
        req: &mut FwReq,
        node: &mut FwNode,
        timeout_ms: u32,
    ) -> Result<(), Error>;

    fn clear_leds(
        state: &mut T,
        req: &mut FwReq,
        node: &mut FwNode,
        timeout_ms: u32,
    ) -> Result<(), Error>;
}

/// The trait to operate local state of surface.
pub trait TascamSurfaceStateOperation<T> {
    /// Initialize the state.
    fn init(state: &mut T);

    /// Peek machine value from image and event.
    fn peek(
        state: &T,
        image: &[u32],
        index: u32,
        before: u32,
        after: u32,
    ) -> Vec<(MachineItem, ItemValue)>;

    /// Ack the machine value returned from peek method.
    fn ack(state: &mut T, machine_value: &(MachineItem, ItemValue));
}

/// Common state of surface.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct TascamSurfaceCommonState {
    stateful_items: Vec<bool>,
    enabled_leds: LedState,
}

/// The trait to express specification of normal LEDs.
pub trait TascamSurfaceLedNormalSpecification {
    const NORMAL_LEDS: &'static [(&'static [MachineItem], &'static [u16])];
}

impl<O> TascamSurfaceLedOperation<TascamSurfaceCommonState> for O
where
    O: TascamSurfaceLedNormalSpecification,
{
    fn operate_leds(
        state: &mut TascamSurfaceCommonState,
        machine_value: &(MachineItem, ItemValue),
        req: &mut FwReq,
        node: &mut FwNode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        if let ItemValue::Bool(value) = machine_value.1 {
            if let Some((_, positions)) = Self::NORMAL_LEDS.iter().find(|(items, _)| {
                if items.len() == 1 {
                    machine_value.0.eq(&items[0])
                } else {
                    items.iter().find(|i| machine_value.0.eq(i)).is_some()
                }
            }) {
                operate_led_cached(
                    &mut state.enabled_leds,
                    req,
                    node,
                    positions[0],
                    value,
                    timeout_ms,
                )?;
            }
        }

        Ok(())
    }

    fn clear_leds(
        state: &mut TascamSurfaceCommonState,
        req: &mut FwReq,
        node: &mut FwNode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        clear_leds(&mut state.enabled_leds, req, node, timeout_ms)
    }
}

/// The trait to express specification for common state of surface.
pub trait TascamSurfaceStateCommonSpecification {
    /// The surface items to be stateful.
    const STATEFUL_ITEMS: &'static [(SurfaceBoolValue, MachineItem)];
    /// The surface items to be stateless.
    const STATELESS_ITEMS: &'static [(SurfaceBoolValue, MachineItem)];
    /// The surface rotaries.
    const ROTARIES: &'static [(SurfaceU16Value, MachineItem)];
    /// The surface faders.
    const FADERS: &'static [(SurfaceBoolValue, SurfaceU16Value, MachineItem)];
}

impl<O> TascamSurfaceStateOperation<TascamSurfaceCommonState> for O
where
    O: TascamSurfaceStateCommonSpecification,
{
    fn init(state: &mut TascamSurfaceCommonState) {
        state.stateful_items = vec![Default::default(); Self::STATEFUL_ITEMS.len()];
    }

    fn peek(
        state: &TascamSurfaceCommonState,
        image: &[u32],
        index: u32,
        before: u32,
        after: u32,
    ) -> Vec<(MachineItem, ItemValue)> {
        let mut machine_values = Vec::new();

        Self::STATEFUL_ITEMS
            .iter()
            .zip(&state.stateful_items)
            .filter(|((bool_val, _), _)| {
                detect_stateful_bool_action(bool_val, index, before, after)
            })
            .for_each(|((_, item), &s)| machine_values.push((*item, ItemValue::Bool(!s))));

        Self::STATELESS_ITEMS
            .iter()
            .filter(|(bool_val, _)| detect_bool_action(bool_val, index, before, after))
            .for_each(|(bool_val, item)| {
                let value = detect_bool_value(bool_val, before);
                machine_values.push((*item, ItemValue::Bool(value)));
            });

        Self::ROTARIES
            .iter()
            .filter(|(u16_val, _)| detect_u16_action(u16_val, index, before, after))
            .for_each(|(u16_val, item)| {
                let value = detect_u16_value(u16_val, after);
                machine_values.push((*item, ItemValue::U16(value)));
            });

        Self::FADERS
            .iter()
            .filter(|(bool_val, _, _)| detect_bool_action(bool_val, index, before, after))
            .for_each(|(_, u16_val, item)| {
                let value = detect_u16_value_in_image(u16_val, image);
                machine_values.push((*item, ItemValue::U16(value)));
            });

        machine_values
    }

    fn ack(state: &mut TascamSurfaceCommonState, machine_value: &(MachineItem, ItemValue)) {
        if let ItemValue::Bool(val) = machine_value.1 {
            Self::STATEFUL_ITEMS
                .iter()
                .zip(&mut state.stateful_items)
                .find(|((_, item), _)| machine_value.0.eq(item))
                .map(|((_, _), s)| *s = val);
        }
    }
}

/// Boolean value in surface image.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct SurfaceBoolValue(usize, u32); // index, mask.

/// U16 value in surface image.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct SurfaceU16Value(usize, u32, usize); // index, mask, shift

fn detect_stateful_bool_action(
    bool_val: &SurfaceBoolValue,
    index: u32,
    before: u32,
    after: u32,
) -> bool {
    bool_val.0 == index as usize && (before ^ after) & bool_val.1 > 0 && before & bool_val.1 > 0
}

fn detect_bool_action(bool_val: &SurfaceBoolValue, index: u32, before: u32, after: u32) -> bool {
    bool_val.0 == index as usize && (before ^ after) & bool_val.1 > 0
}

fn detect_bool_value(bool_val: &SurfaceBoolValue, before: u32) -> bool {
    before & bool_val.1 > 0
}

fn detect_u16_action(u16_val: &SurfaceU16Value, index: u32, before: u32, after: u32) -> bool {
    u16_val.0 == index as usize && (before ^ after) & u16_val.1 > 0
}

fn detect_u16_value(u16_val: &SurfaceU16Value, after: u32) -> u16 {
    ((after & u16_val.1) >> u16_val.2) as u16
}

fn detect_u16_value_in_image(u16_val: &SurfaceU16Value, image: &[u32]) -> u16 {
    ((image[u16_val.0] & u16_val.1) >> u16_val.2) as u16
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
struct LedState(Vec<u16>);

fn operate_led(
    req: &mut FwReq,
    node: &mut FwNode,
    pos: u16,
    enable: bool,
    timeout_ms: u32,
) -> Result<(), Error> {
    let mut frame = [0; 4];
    frame[0..2].copy_from_slice(&(enable as u16).to_be_bytes());
    frame[2..4].copy_from_slice(&pos.to_be_bytes());
    write_quadlet(req, node, LED_OFFSET, &mut frame, timeout_ms)
}

fn operate_led_cached(
    state: &mut LedState,
    req: &mut FwReq,
    node: &mut FwNode,
    pos: u16,
    enable: bool,
    timeout_ms: u32,
) -> Result<(), Error> {
    operate_led(req, node, pos, enable, timeout_ms).map(|_| {
        if !enable {
            state.0.retain(|&p| p != pos);
        } else if state.0.iter().find(|&p| *p == pos).is_none() {
            state.0.push(pos);
        }
    })
}

fn clear_leds(
    state: &mut LedState,
    req: &mut FwReq,
    node: &mut FwNode,
    timeout_ms: u32,
) -> Result<(), Error> {
    let cache = state.0.to_vec();
    cache
        .iter()
        .try_for_each(|&pos| operate_led_cached(state, req, node, pos, false, timeout_ms))
}

/// The trait for operation of FireWire LED.
pub trait FireWireLedOperation {
    const POSITIONS: &'static [u16];

    fn operate_firewire_led(
        req: &mut FwReq,
        node: &mut FwNode,
        enable: bool,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        operate_led(req, node, Self::POSITIONS[0], enable, timeout_ms)
    }
}
