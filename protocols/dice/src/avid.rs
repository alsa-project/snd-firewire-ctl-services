// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Hardware specification and application protocol specific to Avid Mbox 3 Pro
//!
//! The module includes structure, enumeration, and trait and its implementation for hardware
//! specification and application protocol specific to Avid Mbox 3 Pro.

use super::{
    tcat::{
        extension::{appl_section::*, *},
        tcd22xx_spec::*,
        *,
    },
    *,
};

const USE_CASE_OFFSET: usize = 0x00;
const MASTER_KNOB_ASSIGN_OFFSET: usize = 0x0c;
const BUTTON_LED_STATE_OFFSET: usize = 0x10;
const DIM_LED_USAGE_OFFSET: usize = 0x1c;
const BUTTON_HOLD_DURATION_OFFSET: usize = 0x20;
const HPF_ENABLE_OFFSET: usize = 0x24; // High pass filter for analog inputs.
const OUT_0_TRIM_OFFSET: usize = 0x28;
const OUT_1_TRIM_OFFSET: usize = 0x2c;
const OUT_2_TRIM_OFFSET: usize = 0x30;
const OUT_3_TRIM_OFFSET: usize = 0x34;
const OUT_4_TRIM_OFFSET: usize = 0x38;
const OUT_5_TRIM_OFFSET: usize = 0x3c;
const REVERB_TYPE_OFFSET: usize = 0x40;
const REVERB_VOLUME_OFFSET: usize = 0x44;
const REVERB_DURATION_OFFSET: usize = 0x48;
const REVERB_FEEDBACK_OFFSET: usize = 0x4c;

/// Protocol implementation of Avid Mbox 3 Pro.
#[derive(Default, Debug)]
pub struct Mbox3Protocol;

impl TcatOperation for Mbox3Protocol {}

impl TcatGlobalSectionSpecification for Mbox3Protocol {}

impl Tcd22xxSpecOperation for Mbox3Protocol {
    const INPUTS: &'static [Input] = &[
        Input {
            id: SrcBlkId::Ins0,
            offset: 0,
            count: 6,
            label: None,
        },
        Input {
            id: SrcBlkId::Ins1,
            offset: 0,
            count: 2,
            label: Some("Reverb"),
        },
        Input {
            id: SrcBlkId::Aes,
            offset: 0,
            count: 2,
            label: None,
        },
    ];
    const OUTPUTS: &'static [Output] = &[
        Output {
            id: DstBlkId::Ins0,
            offset: 0,
            count: 6,
            label: None,
        },
        Output {
            id: DstBlkId::Ins1,
            offset: 0,
            count: 4,
            label: Some("Headphone"),
        },
        Output {
            id: DstBlkId::Ins1,
            offset: 4,
            count: 2,
            label: Some("Reverb"),
        },
        Output {
            id: DstBlkId::Aes,
            offset: 0,
            count: 2,
            label: None,
        },
        Output {
            id: DstBlkId::Reserved(0x08),
            offset: 0,
            count: 2,
            label: Some("ControlRoom"),
        },
    ];
    const FIXED: &'static [SrcBlk] = &[
        SrcBlk {
            id: SrcBlkId::Ins0,
            ch: 0,
        },
        SrcBlk {
            id: SrcBlkId::Ins0,
            ch: 1,
        },
        SrcBlk {
            id: SrcBlkId::Ins0,
            ch: 2,
        },
        SrcBlk {
            id: SrcBlkId::Ins0,
            ch: 3,
        },
    ];
}

/// Usecase of standalone mode.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum StandaloneUseCase {
    Preamp,
    AdDa,
    Mixer,
}

impl Default for StandaloneUseCase {
    fn default() -> Self {
        Self::Preamp
    }
}

impl StandaloneUseCase {
    const MIXER: u32 = 0;
    const AD_DA: u32 = 1;
    const PREAMP: u32 = 2;
}

impl From<u32> for StandaloneUseCase {
    fn from(val: u32) -> Self {
        match val {
            StandaloneUseCase::MIXER => StandaloneUseCase::Mixer,
            StandaloneUseCase::AD_DA => StandaloneUseCase::AdDa,
            _ => StandaloneUseCase::Preamp,
        }
    }
}

impl From<StandaloneUseCase> for u32 {
    fn from(use_case: StandaloneUseCase) -> u32 {
        match use_case {
            StandaloneUseCase::Mixer => StandaloneUseCase::MIXER,
            StandaloneUseCase::AdDa => StandaloneUseCase::AD_DA,
            StandaloneUseCase::Preamp => StandaloneUseCase::PREAMP,
        }
    }
}

impl Mbox3Protocol {
    pub fn read_standalone_use_case(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        timeout_ms: u32,
    ) -> Result<StandaloneUseCase, Error> {
        let mut data = [0; 4];
        ApplSectionProtocol::read_appl_data(
            req,
            node,
            sections,
            USE_CASE_OFFSET,
            &mut data,
            timeout_ms,
        )
        .map(|_| u32::from_be_bytes(data).into())
    }

    pub fn write_standalone_use_case(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        use_case: StandaloneUseCase,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut data = u32::from(use_case).to_be_bytes().clone();
        ApplSectionProtocol::write_appl_data(
            req,
            node,
            sections,
            USE_CASE_OFFSET,
            &mut data,
            timeout_ms,
        )
    }
}

/// Assignment map for master knob.
pub type MasterKnobAssigns = [bool; 6];

/// LED state of mute button.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum MuteLedState {
    Off,
    Blink,
    On,
}

impl Default for MuteLedState {
    fn default() -> Self {
        Self::Off
    }
}

impl MuteLedState {
    const LED_MASK: u32 = 0x00000003;
    const LED_BLINK_MASK: u32 = 0x00000001;
}

impl From<u32> for MuteLedState {
    fn from(val: u32) -> Self {
        if val & Self::LED_MASK == 0 {
            Self::Off
        } else if val & Self::LED_BLINK_MASK > 0 {
            Self::Blink
        } else {
            Self::On
        }
    }
}

impl From<&MuteLedState> for u32 {
    fn from(state: &MuteLedState) -> Self {
        match state {
            MuteLedState::Off => 0,
            MuteLedState::Blink => MuteLedState::LED_BLINK_MASK,
            MuteLedState::On => MuteLedState::LED_MASK & !MuteLedState::LED_BLINK_MASK,
        }
    }
}

/// LED state of mono button.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum MonoLedState {
    Off,
    On,
}

impl Default for MonoLedState {
    fn default() -> Self {
        Self::Off
    }
}

impl MonoLedState {
    const LED_MASK: u32 = 0x0000000c;
    const LED_SHIFT: usize = 2;
}

impl From<u32> for MonoLedState {
    fn from(val: u32) -> Self {
        if (val & Self::LED_MASK) >> Self::LED_SHIFT > 0 {
            MonoLedState::On
        } else {
            MonoLedState::Off
        }
    }
}

impl From<&MonoLedState> for u32 {
    fn from(state: &MonoLedState) -> Self {
        match state {
            MonoLedState::On => MonoLedState::LED_MASK,
            MonoLedState::Off => 0,
        }
    }
}

/// LED state of spkr button.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SpkrLedState {
    Off,
    Green,
    GreenBlink,
    Red,
    RedBlink,
    Orange,
    OrangeBlink,
}

impl Default for SpkrLedState {
    fn default() -> Self {
        Self::Off
    }
}

impl SpkrLedState {
    const COLOR_MASK: u32 = 0x00000060;
    const COLOR_SHIFT: usize = 5;
    const BLINK_MASK: u32 = 0x00000010;
    const BLINK_SHIFT: u32 = 4;

    const NONE: u32 = 0;
    const GREEN: u32 = 1;
    const RED: u32 = 2;
    const ORANGE: u32 = 3;
}

impl From<u32> for SpkrLedState {
    fn from(val: u32) -> Self {
        let color = (val & Self::COLOR_MASK) >> Self::COLOR_SHIFT;
        let blink = (val & Self::BLINK_MASK) >> Self::BLINK_SHIFT > 0;

        match color {
            Self::NONE => SpkrLedState::Off,
            Self::GREEN => {
                if blink {
                    SpkrLedState::GreenBlink
                } else {
                    SpkrLedState::Green
                }
            }
            Self::RED => {
                if blink {
                    SpkrLedState::RedBlink
                } else {
                    SpkrLedState::Red
                }
            }
            Self::ORANGE => {
                if blink {
                    SpkrLedState::OrangeBlink
                } else {
                    SpkrLedState::Orange
                }
            }
            _ => SpkrLedState::Off,
        }
    }
}

impl From<&SpkrLedState> for u32 {
    fn from(state: &SpkrLedState) -> Self {
        let (color, blink) = match state {
            SpkrLedState::GreenBlink => (SpkrLedState::GREEN, true),
            SpkrLedState::Green => (SpkrLedState::GREEN, false),
            SpkrLedState::RedBlink => (SpkrLedState::RED, true),
            SpkrLedState::Red => (SpkrLedState::RED, false),
            SpkrLedState::OrangeBlink => (SpkrLedState::RED, true),
            SpkrLedState::Orange => (SpkrLedState::ORANGE, false),
            SpkrLedState::Off => (SpkrLedState::NONE, false),
        };

        let mut val = color << SpkrLedState::COLOR_SHIFT;
        if blink {
            val |= SpkrLedState::BLINK_MASK;
        }

        val
    }
}

/// Status of buttons.
#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
pub struct ButtonLedState {
    pub mute: MuteLedState,
    pub mono: MonoLedState,
    pub spkr: SpkrLedState,
}

impl ButtonLedState {
    const SIZE: usize = 8;
}

impl From<[u8; ButtonLedState::SIZE]> for ButtonLedState {
    fn from(raw: [u8; ButtonLedState::SIZE]) -> Self {
        let mut quadlet = [0; 4];
        quadlet.copy_from_slice(&raw[..4]);
        let val = u32::from_be_bytes(quadlet);
        let mute = MuteLedState::from(val);

        quadlet.copy_from_slice(&raw[4..8]);
        let val = u32::from_be_bytes(quadlet);
        let mono = MonoLedState::from(val);
        let spkr = SpkrLedState::from(val);

        ButtonLedState { mute, mono, spkr }
    }
}

impl From<&ButtonLedState> for [u8; ButtonLedState::SIZE] {
    fn from(state: &ButtonLedState) -> Self {
        let mut data = [0; ButtonLedState::SIZE];

        let val = u32::from(&state.mute);
        data[..4].copy_from_slice(&val.to_be_bytes());

        let val = u32::from(&state.mono) | u32::from(&state.spkr);
        data[4..].copy_from_slice(&val.to_be_bytes());

        data
    }
}

const OUT_TRIM_OFFSETS: [usize; 6] = [
    OUT_0_TRIM_OFFSET,
    OUT_1_TRIM_OFFSET,
    OUT_2_TRIM_OFFSET,
    OUT_3_TRIM_OFFSET,
    OUT_4_TRIM_OFFSET,
    OUT_5_TRIM_OFFSET,
];

impl Mbox3Protocol {
    pub fn read_master_knob_assign(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        assigns: &mut MasterKnobAssigns,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut data = [0; 4];
        ApplSectionProtocol::read_appl_data(
            req,
            node,
            sections,
            MASTER_KNOB_ASSIGN_OFFSET,
            &mut data,
            timeout_ms,
        )
        .map(|_| {
            let val = u32::from_be_bytes(data);
            assigns
                .iter_mut()
                .enumerate()
                .for_each(|(i, assigned)| *assigned = val & (1 << i) > 0);
            ()
        })
    }

    pub fn write_master_knob_assign(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        assigns: &MasterKnobAssigns,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut data = [0; 4];
        let val: u32 = assigns
            .iter()
            .enumerate()
            .filter(|&(_, &assigned)| assigned)
            .fold(0, |val, (i, _)| val | (1 << i));
        data.copy_from_slice(&val.to_be_bytes());
        ApplSectionProtocol::write_appl_data(
            req,
            node,
            sections,
            MASTER_KNOB_ASSIGN_OFFSET,
            &mut data,
            timeout_ms,
        )
    }

    pub fn read_button_led_state(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        timeout_ms: u32,
    ) -> Result<ButtonLedState, Error> {
        let mut data = [0; ButtonLedState::SIZE];
        ApplSectionProtocol::read_appl_data(
            req,
            node,
            sections,
            BUTTON_LED_STATE_OFFSET,
            &mut data,
            timeout_ms,
        )
        .map(|_| ButtonLedState::from(data))
    }

    pub fn write_button_led_state(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        state: &ButtonLedState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut data = Into::<[u8; ButtonLedState::SIZE]>::into(state);
        ApplSectionProtocol::write_appl_data(
            req,
            node,
            sections,
            BUTTON_LED_STATE_OFFSET,
            &mut data,
            timeout_ms,
        )
    }

    pub fn read_dim_led_usage(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        let mut data = [0; 4];
        ApplSectionProtocol::read_appl_data(
            req,
            node,
            sections,
            DIM_LED_USAGE_OFFSET,
            &mut data,
            timeout_ms,
        )
        .map(|_| u32::from_be_bytes(data) > 0)
    }

    pub fn write_dim_led_usage(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        usage: bool,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut data = (usage as u32).to_be_bytes().clone();
        ApplSectionProtocol::write_appl_data(
            req,
            node,
            sections,
            DIM_LED_USAGE_OFFSET,
            &mut data,
            timeout_ms,
        )
    }

    pub fn read_hold_duration(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        timeout_ms: u32,
    ) -> Result<u8, Error> {
        let mut data = [0; 4];
        ApplSectionProtocol::read_appl_data(
            req,
            node,
            sections,
            BUTTON_HOLD_DURATION_OFFSET,
            &mut data,
            timeout_ms,
        )
        .map(|_| u32::from_be_bytes(data) as u8)
    }

    pub fn write_hold_duration(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        duration: u8,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut data = (duration as u32).to_be_bytes().clone();
        ApplSectionProtocol::write_appl_data(
            req,
            node,
            sections,
            BUTTON_HOLD_DURATION_OFFSET,
            &mut data,
            timeout_ms,
        )
    }

    pub fn read_hpf_enable(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        inputs: &mut [bool; 4],
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut data = [0; 4];
        ApplSectionProtocol::read_appl_data(
            req,
            node,
            sections,
            HPF_ENABLE_OFFSET,
            &mut data,
            timeout_ms,
        )
        .map(|_| {
            let val = u32::from_be_bytes(data);
            inputs
                .iter_mut()
                .enumerate()
                .for_each(|(i, v)| *v = val & (1 << i) > 0);
        })
    }

    pub fn write_hpf_enable(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        inputs: [bool; 4],
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let val = inputs
            .iter()
            .enumerate()
            .filter(|(_, &v)| v)
            .fold(0 as u32, |val, (i, _)| val | (1 << i));
        ApplSectionProtocol::write_appl_data(
            req,
            node,
            sections,
            HPF_ENABLE_OFFSET,
            &mut val.to_be_bytes().clone(),
            timeout_ms,
        )
    }

    pub fn read_output_trim(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        idx: usize,
        timeout_ms: u32,
    ) -> Result<u8, Error> {
        let &offset = OUT_TRIM_OFFSETS.iter().nth(idx).ok_or_else(|| {
            let msg = format!(
                "Invalid index of output: {} greater than {}",
                idx,
                OUT_TRIM_OFFSETS.len()
            );
            Error::new(FileError::Inval, &msg)
        })?;
        let mut data = [0; 4];
        ApplSectionProtocol::read_appl_data(req, node, sections, offset, &mut data, timeout_ms)
            .map(|_| u32::from_be_bytes(data) as u8)
    }

    pub fn write_output_trim(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        idx: usize,
        trim: u8,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let &offset = OUT_TRIM_OFFSETS.iter().nth(idx).ok_or_else(|| {
            let msg = format!(
                "Invalid index of output: {} greater than {}",
                idx,
                OUT_TRIM_OFFSETS.len()
            );
            Error::new(FileError::Inval, &msg)
        })?;
        let mut data = [0; 4];
        data.copy_from_slice(&(trim as u32).to_be_bytes());
        ApplSectionProtocol::write_appl_data(req, node, sections, offset, &mut data, timeout_ms)
    }
}

/// Type of reverb DSP.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ReverbType {
    Room1,
    Room2,
    Room3,
    Hall1,
    Hall2,
    Plate,
    Delay,
    Echo,
}

impl Default for ReverbType {
    fn default() -> Self {
        Self::Room1
    }
}

impl ReverbType {
    const ROOM_1: u32 = 0x01;
    const ROOM_2: u32 = 0x04;
    const ROOM_3: u32 = 0x05;
    const HALL_1: u32 = 0x06;
    const HALL_2: u32 = 0x08;
    const PLATE: u32 = 0x0b;
    const DELAY: u32 = 0x13;
    const ECHO: u32 = 0x14;
}

impl From<u32> for ReverbType {
    fn from(val: u32) -> Self {
        match val {
            ReverbType::ROOM_1 => ReverbType::Room1,
            ReverbType::ROOM_2 => ReverbType::Room2,
            ReverbType::ROOM_3 => ReverbType::Room3,
            ReverbType::HALL_1 => ReverbType::Hall1,
            ReverbType::HALL_2 => ReverbType::Hall2,
            ReverbType::PLATE => ReverbType::Plate,
            ReverbType::DELAY => ReverbType::Delay,
            _ => ReverbType::Echo,
        }
    }
}

impl From<ReverbType> for u32 {
    fn from(reverb_type: ReverbType) -> Self {
        match reverb_type {
            ReverbType::Room1 => ReverbType::ROOM_1,
            ReverbType::Room2 => ReverbType::ROOM_2,
            ReverbType::Room3 => ReverbType::ROOM_3,
            ReverbType::Hall1 => ReverbType::HALL_1,
            ReverbType::Hall2 => ReverbType::HALL_2,
            ReverbType::Plate => ReverbType::PLATE,
            ReverbType::Delay => ReverbType::DELAY,
            ReverbType::Echo => ReverbType::ECHO,
        }
    }
}

impl Mbox3Protocol {
    pub fn read_reverb_type(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        timeout_ms: u32,
    ) -> Result<ReverbType, Error> {
        let mut data = [0; 4];
        ApplSectionProtocol::read_appl_data(
            req,
            node,
            sections,
            REVERB_TYPE_OFFSET,
            &mut data,
            timeout_ms,
        )
        .map(|_| ReverbType::from(u32::from_be_bytes(data)))
    }

    pub fn write_reverb_type(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        reverb_type: ReverbType,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut data = u32::from(reverb_type).to_be_bytes().clone();
        ApplSectionProtocol::write_appl_data(
            req,
            node,
            sections,
            REVERB_TYPE_OFFSET,
            &mut data,
            timeout_ms,
        )
    }

    pub fn read_reverb_volume(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        timeout_ms: u32,
    ) -> Result<u8, Error> {
        let mut data = [0; 4];
        ApplSectionProtocol::read_appl_data(
            req,
            node,
            sections,
            REVERB_VOLUME_OFFSET,
            &mut data,
            timeout_ms,
        )
        .map(|_| u32::from_be_bytes(data) as u8)
    }

    pub fn write_reverb_volume(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        volume: u8,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut data = (volume as u32).to_be_bytes().clone();
        ApplSectionProtocol::write_appl_data(
            req,
            node,
            sections,
            REVERB_VOLUME_OFFSET,
            &mut data,
            timeout_ms,
        )
    }

    pub fn read_reverb_duration(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        timeout_ms: u32,
    ) -> Result<u8, Error> {
        let mut data = [0; 4];
        ApplSectionProtocol::read_appl_data(
            req,
            node,
            sections,
            REVERB_DURATION_OFFSET,
            &mut data,
            timeout_ms,
        )
        .map(|_| u32::from_be_bytes(data) as u8)
    }

    pub fn write_reverb_duration(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        duration: u8,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut data = (duration as u32).to_be_bytes().clone();
        ApplSectionProtocol::write_appl_data(
            req,
            node,
            sections,
            REVERB_DURATION_OFFSET,
            &mut data,
            timeout_ms,
        )
    }

    pub fn read_reverb_feedback(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        timeout_ms: u32,
    ) -> Result<u8, Error> {
        let mut data = [0; 4];
        ApplSectionProtocol::read_appl_data(
            req,
            node,
            sections,
            REVERB_FEEDBACK_OFFSET,
            &mut data,
            timeout_ms,
        )
        .map(|_| u32::from_be_bytes(data) as u8)
    }

    pub fn write_reverb_feedback(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        feedback: u8,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut data = (feedback as u32).to_be_bytes().clone();
        ApplSectionProtocol::write_appl_data(
            req,
            node,
            sections,
            REVERB_FEEDBACK_OFFSET,
            &mut data,
            timeout_ms,
        )
    }
}

impl Mbox3Protocol {
    const PHANTOM_BUTTON_PUSHED: u32 = 0x10000000;
    const SPKR_BUTTON_PUSHED: u32 = 0x04000000;
    const SPKR_BUTTON_HELD: u32 = 0x02000000;
    const MONO_BUTTON_PUSHED: u32 = 0x00800000;
    const MUTE_BUTTON_PUSHED: u32 = 0x00400000;
    const MUTE_BUTTON_HELD: u32 = 0x00200000;

    pub fn has_phantom_button_pushed(msg: u32) -> bool {
        msg & Self::PHANTOM_BUTTON_PUSHED > 0
    }

    pub fn has_spkr_button_pushed(msg: u32) -> bool {
        msg & Self::SPKR_BUTTON_PUSHED > 0
    }

    pub fn has_spkr_button_held(msg: u32) -> bool {
        msg & Self::SPKR_BUTTON_HELD > 0
    }

    pub fn has_mono_button_pushed(msg: u32) -> bool {
        msg & Self::MONO_BUTTON_PUSHED > 0
    }

    pub fn has_mute_button_pushed(msg: u32) -> bool {
        msg & Self::MUTE_BUTTON_PUSHED > 0
    }

    pub fn has_mute_button_held(msg: u32) -> bool {
        msg & Self::MUTE_BUTTON_HELD > 0
    }
}
