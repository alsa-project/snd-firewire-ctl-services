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
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum StandaloneUseCase {
    /// For microphone pre-amplifier.
    Preamp,
    /// For A/D and D/A conversion.
    AdDa,
    /// For signal mixing.
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

fn serialize_standalone_use_case(
    use_case: &StandaloneUseCase,
    raw: &mut [u8],
) -> Result<(), String> {
    assert!(raw.len() >= 4);

    match use_case {
        StandaloneUseCase::Mixer => StandaloneUseCase::MIXER,
        StandaloneUseCase::AdDa => StandaloneUseCase::AD_DA,
        StandaloneUseCase::Preamp => StandaloneUseCase::PREAMP,
    }
    .build_quadlet(raw);

    Ok(())
}

fn deserialize_standalone_use_case(
    use_case: &mut StandaloneUseCase,
    raw: &[u8],
) -> Result<(), String> {
    assert!(raw.len() >= 4);

    let mut val = 0u32;
    val.parse_quadlet(raw);

    *use_case = match val {
        StandaloneUseCase::MIXER => StandaloneUseCase::Mixer,
        StandaloneUseCase::AD_DA => StandaloneUseCase::AdDa,
        StandaloneUseCase::PREAMP => StandaloneUseCase::Preamp,
        _ => Err(format!("Standalone use case not found for value {}", val))?,
    };

    Ok(())
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

const MASTER_KNOB_MASK: u32 = 0x0000003f;

fn serialize_master_knob_assigns(assigns: &[bool; 6], raw: &mut [u8]) -> Result<(), String> {
    assert!(raw.len() >= 4);

    let mut val = 0u32;
    val.parse_quadlet(raw);
    val &= !MASTER_KNOB_MASK;

    assigns
        .iter()
        .enumerate()
        .filter(|(_, &assign)| assign)
        .for_each(|(i, _)| val |= 1 << i);
    val.build_quadlet(raw);

    Ok(())
}

fn deserialize_master_knob_assigns(assigns: &mut [bool; 6], raw: &[u8]) -> Result<(), String> {
    assert!(raw.len() >= 4);

    let mut val = 0u32;
    val.parse_quadlet(raw);
    assigns
        .iter_mut()
        .enumerate()
        .for_each(|(i, assign)| *assign = val & (1 << i) > 0);

    Ok(())
}

/// LED state of mute button.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MuteLedState {
    /// Turn Off.
    Off,
    /// Blink.
    Blink,
    /// Turn on.
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

fn serialize_mute_led_state(state: &MuteLedState, raw: &mut [u8]) -> Result<(), String> {
    assert!(raw.len() >= 4);

    let mut val = 0u32;
    val.parse_quadlet(raw);
    val &= !MuteLedState::LED_MASK;

    match state {
        MuteLedState::Off => (),
        MuteLedState::Blink => val |= MuteLedState::LED_BLINK_MASK,
        MuteLedState::On => val |= MuteLedState::LED_MASK & !MuteLedState::LED_BLINK_MASK,
    }

    val.build_quadlet(raw);

    Ok(())
}

fn deserialize_mute_led_state(state: &mut MuteLedState, raw: &[u8]) -> Result<(), String> {
    assert!(raw.len() >= 4);

    let mut val = 0u32;
    val.parse_quadlet(raw);

    *state = if val & MuteLedState::LED_MASK == 0 {
        MuteLedState::Off
    } else if val & MuteLedState::LED_BLINK_MASK > 0 {
        MuteLedState::Blink
    } else {
        MuteLedState::On
    };

    Ok(())
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
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MonoLedState {
    /// Turn off.
    Off,
    /// Turn on.
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

fn serialize_mono_led_state(state: &MonoLedState, raw: &mut [u8]) -> Result<(), String> {
    assert!(raw.len() >= 4);

    let mut val = 0u32;
    val.parse_quadlet(raw);
    val &= !MonoLedState::LED_MASK;

    match state {
        MonoLedState::Off => (),
        MonoLedState::On => val |= MonoLedState::LED_MASK,
    }

    val.build_quadlet(raw);

    Ok(())
}

fn deserialize_mono_led_state(state: &mut MonoLedState, raw: &[u8]) -> Result<(), String> {
    assert!(raw.len() >= 4);

    let mut val = 0u32;
    val.parse_quadlet(raw);

    *state = if val & MonoLedState::LED_MASK > 0 {
        MonoLedState::On
    } else {
        MonoLedState::Off
    };

    Ok(())
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
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SpkrLedState {
    /// Turn off.
    Off,
    /// Green light.
    Green,
    /// Blinking green light.
    GreenBlink,
    /// Red light.
    Red,
    /// Blinking red light.
    RedBlink,
    /// Orange light.
    Orange,
    /// Blinking orange light.
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

fn serialize_spkr_led_state(state: &SpkrLedState, raw: &mut [u8]) -> Result<(), String> {
    assert!(raw.len() >= 4);

    let mut val = 0u32;
    val.parse_quadlet(raw);
    val &= !(SpkrLedState::COLOR_MASK | SpkrLedState::BLINK_MASK);

    let (color, blink) = match state {
        SpkrLedState::GreenBlink => (SpkrLedState::GREEN, true),
        SpkrLedState::Green => (SpkrLedState::GREEN, false),
        SpkrLedState::RedBlink => (SpkrLedState::RED, true),
        SpkrLedState::Red => (SpkrLedState::RED, false),
        SpkrLedState::OrangeBlink => (SpkrLedState::RED, true),
        SpkrLedState::Orange => (SpkrLedState::ORANGE, false),
        SpkrLedState::Off => (SpkrLedState::NONE, false),
    };

    val |= color << SpkrLedState::COLOR_SHIFT;
    if blink {
        val |= SpkrLedState::BLINK_MASK;
    }

    val.build_quadlet(raw);

    Ok(())
}

fn deserialize_spkr_led_state(state: &mut SpkrLedState, raw: &[u8]) -> Result<(), String> {
    assert!(raw.len() >= 4);

    let mut val = 0u32;
    val.parse_quadlet(raw);

    let color = (val & SpkrLedState::COLOR_MASK) >> SpkrLedState::COLOR_SHIFT;
    let blink = (val & SpkrLedState::BLINK_MASK) >> SpkrLedState::BLINK_SHIFT > 0;

    *state = match color {
        SpkrLedState::NONE => SpkrLedState::Off,
        SpkrLedState::GREEN => {
            if blink {
                SpkrLedState::GreenBlink
            } else {
                SpkrLedState::Green
            }
        }
        SpkrLedState::RED => {
            if blink {
                SpkrLedState::RedBlink
            } else {
                SpkrLedState::Red
            }
        }
        SpkrLedState::ORANGE => {
            if blink {
                SpkrLedState::OrangeBlink
            } else {
                SpkrLedState::Orange
            }
        }
        _ => Err(format!("Speaker LED state not found for value {}", val))?,
    };

    Ok(())
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
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct ButtonLedState {
    /// State of mute LED.
    pub mute: MuteLedState,
    /// State of mono LED.
    pub mono: MonoLedState,
    /// State of spkr LED.
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

const PHANTOM_POWERING_MASK: u32 = 0x00000001;
const INPUT_HPF_ENABLES_MASK: u32 = 0x000000f0;
const INPUT_HPF_ENABLES_SHIFT: usize = 4;

fn serialize_phantom_powering(enable: &bool, raw: &mut [u8]) -> Result<(), String> {
    assert!(raw.len() >= 4);

    let mut val = 0u32;
    val.parse_quadlet(raw);
    val &= !PHANTOM_POWERING_MASK;

    if *enable {
        val |= PHANTOM_POWERING_MASK;
    }

    val.build_quadlet(raw);

    Ok(())
}

fn deserialize_phantom_powering(enable: &mut bool, raw: &[u8]) -> Result<(), String> {
    assert!(raw.len() >= 4);

    let mut val = 0u32;
    val.parse_quadlet(raw);

    *enable = val & PHANTOM_POWERING_MASK > 0;

    Ok(())
}

fn serialize_hpf_enables(enables: &[bool; 4], raw: &mut [u8]) -> Result<(), String> {
    assert!(raw.len() >= 4);

    let mut val = 0u32;
    val.parse_quadlet(raw);
    val &= !INPUT_HPF_ENABLES_MASK;

    enables
        .iter()
        .enumerate()
        .filter(|(_, &enabled)| enabled)
        .for_each(|(i, _)| val |= 1 << (i + INPUT_HPF_ENABLES_SHIFT));

    val.build_quadlet(raw);

    Ok(())
}

fn deserialize_hpf_enables(enables: &mut [bool; 4], raw: &[u8]) -> Result<(), String> {
    assert!(raw.len() >= 4);

    let mut val = 0u32;
    val.parse_quadlet(raw);

    enables
        .iter_mut()
        .enumerate()
        .for_each(|(i, enabled)| *enabled = val & (1 << (i + INPUT_HPF_ENABLES_SHIFT)) > 0);

    Ok(())
}

const OUT_TRIM_OFFSETS: [usize; 6] = [
    OUT_0_TRIM_OFFSET,
    OUT_1_TRIM_OFFSET,
    OUT_2_TRIM_OFFSET,
    OUT_3_TRIM_OFFSET,
    OUT_4_TRIM_OFFSET,
    OUT_5_TRIM_OFFSET,
];

fn serialize_output_trims(trims: &[u8; 6], raw: &mut [u8]) -> Result<(), String> {
    assert!(raw.len() >= 24);

    trims.iter().enumerate().for_each(|(i, &trim)| {
        let pos = i * 4;
        ((u8::MAX - trim) as u32).build_quadlet(&mut raw[pos..(pos + 4)]);
    });

    Ok(())
}

fn deserialize_output_trims(trims: &mut [u8; 6], raw: &[u8]) -> Result<(), String> {
    assert!(raw.len() >= 24);

    let mut val = 0u32;
    trims.iter_mut().enumerate().for_each(|(i, trim)| {
        let pos = i * 4;
        val.parse_quadlet(&raw[pos..(pos + 4)]);
        *trim = u8::MAX - val as u8;
    });

    Ok(())
}

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
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ReverbType {
    /// Room 1.
    Room1,
    /// Room 2.
    Room2,
    /// Room 3.
    Room3,
    /// Hall 1.
    Hall1,
    /// Hall 2.
    Hall2,
    /// Plate.
    Plate,
    /// Delay.
    Delay,
    /// Echo.
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

fn serialize_reverb_type(reverb_type: &ReverbType, raw: &mut [u8]) -> Result<(), String> {
    assert!(raw.len() >= 4);

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
    .build_quadlet(raw);

    Ok(())
}

fn deserialize_reverb_type(reverb_type: &mut ReverbType, raw: &[u8]) -> Result<(), String> {
    assert!(raw.len() >= 4);

    let mut val = 0u32;
    val.parse_quadlet(raw);

    *reverb_type = match val {
        ReverbType::ROOM_1 => ReverbType::Room1,
        ReverbType::ROOM_2 => ReverbType::Room2,
        ReverbType::ROOM_3 => ReverbType::Room3,
        ReverbType::HALL_1 => ReverbType::Hall1,
        ReverbType::HALL_2 => ReverbType::Hall2,
        ReverbType::PLATE => ReverbType::Plate,
        ReverbType::DELAY => ReverbType::Delay,
        ReverbType::ECHO => ReverbType::Echo,
        _ => Err(format!("Reverb type not found for value {}", val))?,
    };

    Ok(())
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

/// Parameters specific to Mbox 3 pro.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Mbox3SpecificParams {
    /// Usecase at standalone mode.
    pub standalone_use_case: StandaloneUseCase,
    /// State of master knob.
    pub master_knob_value: u8,
    /// Assignment map for master knob.
    pub master_knob_assigns: [bool; 6],
    /// State of mute LED.
    pub mute_led: MuteLedState,
    /// State of mono LED.
    pub mono_led: MonoLedState,
    /// State of spkr LED.
    pub spkr_led: SpkrLedState,
    /// Whether to use dim LED.
    pub dim_led: bool,
    /// Duration till being activate by holding button.
    pub duration_hold: u8,
    /// Whether to supply phantom powering.
    pub phantom_powering: bool,
    /// Whether to enable High Pass Filter.
    pub hpf_enables: [bool; 4],
    /// Volume of analog outputs.
    pub output_trims: [u8; 6],
    /// The type of reverb.
    pub reverb_type: ReverbType,
    /// Volume of reverb return.
    pub reverb_volume: u8,
    /// Duration of reverb.
    pub reverb_duration: u8,
    /// Feedback of reverb.
    pub reverb_feedback: u8,
}

const MIN_SIZE: usize = 0x50;

fn serialize(params: &Mbox3SpecificParams, raw: &mut [u8]) -> Result<(), String> {
    assert!(raw.len() >= MIN_SIZE);

    serialize_standalone_use_case(&params.standalone_use_case, &mut raw[..0x04])?;
    (params.master_knob_value as u32).build_quadlet(&mut raw[0x08..0x0c]);
    serialize_master_knob_assigns(&params.master_knob_assigns, &mut raw[0x0c..0x10])?;
    serialize_mute_led_state(&params.mute_led, &mut raw[0x10..0x14])?;
    serialize_mono_led_state(&params.mono_led, &mut raw[0x14..0x18])?;
    serialize_spkr_led_state(&params.spkr_led, &mut raw[0x14..0x18])?;

    params.dim_led.build_quadlet(&mut raw[0x1c..0x20]);

    (params.duration_hold as u32).build_quadlet(&mut raw[0x20..0x24]);

    serialize_phantom_powering(&params.phantom_powering, &mut raw[0x24..0x28])?;
    serialize_hpf_enables(&params.hpf_enables, &mut raw[0x24..0x28])?;
    serialize_output_trims(&params.output_trims, &mut raw[0x28..0x40])?;
    serialize_reverb_type(&params.reverb_type, &mut raw[0x40..0x44])?;
    (params.reverb_volume as u32).build_quadlet(&mut raw[0x44..0x48]);
    (params.reverb_duration as u32).build_quadlet(&mut raw[0x48..0x4c]);
    (params.reverb_feedback as u32).build_quadlet(&mut raw[0x4c..0x50]);

    Ok(())
}

fn deserialize(params: &mut Mbox3SpecificParams, raw: &[u8]) -> Result<(), String> {
    assert!(raw.len() >= MIN_SIZE);

    let mut val = 0u32;

    deserialize_standalone_use_case(&mut params.standalone_use_case, &raw[..0x04])?;

    val.parse_quadlet(&raw[0x08..0x0c]);
    params.master_knob_value = val as u8;

    deserialize_master_knob_assigns(&mut params.master_knob_assigns, &raw[0x0c..0x10])?;
    deserialize_mute_led_state(&mut params.mute_led, &raw[0x10..0x14])?;
    deserialize_mono_led_state(&mut params.mono_led, &raw[0x14..0x18])?;
    deserialize_spkr_led_state(&mut params.spkr_led, &raw[0x14..0x18])?;

    params.dim_led.parse_quadlet(&raw[0x1c..0x20]);

    val.parse_quadlet(&raw[0x20..0x24]);
    params.duration_hold = val as u8;

    deserialize_phantom_powering(&mut params.phantom_powering, &raw[0x24..0x28])?;
    deserialize_hpf_enables(&mut params.hpf_enables, &raw[0x24..0x28])?;
    deserialize_output_trims(&mut params.output_trims, &raw[0x28..0x40])?;
    deserialize_reverb_type(&mut params.reverb_type, &raw[0x40..0x44])?;

    val.parse_quadlet(&raw[0x44..0x48]);
    params.reverb_volume = val as u8;

    val.parse_quadlet(&raw[0x48..0x4c]);
    params.reverb_duration = val as u8;

    val.parse_quadlet(&raw[0x4c..0x50]);
    params.reverb_feedback = val as u8;

    Ok(())
}

const PHANTOM_POWERING_CHANGED: u32 = 0x10000000;
const MASTER_KNOB_CHANGED: u32 = 0x08000000;
const SPKR_BUTTON_PUSHED: u32 = 0x04000000;
const SPKR_BUTTON_HELD: u32 = 0x02000000;
const MONO_BUTTON_PUSHED: u32 = 0x00800000;
const MUTE_BUTTON_PUSHED: u32 = 0x00400000;
const MUTE_BUTTON_HELD: u32 = 0x00200000;

impl Mbox3Protocol {
    /// Cache state of hardware for whole parameters.
    pub fn cache_whole_params(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        params: &mut Mbox3SpecificParams,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut raw = vec![0u8; sections.application.size];
        ApplSectionProtocol::read_appl_data(req, node, sections, 0, &mut raw, timeout_ms)?;
        deserialize(params, &raw).map_err(|cause| Error::new(ProtocolExtensionError::Appl, &cause))
    }

    /// Update state of hardware for part of parameters.
    pub fn update_partial_params(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        params: &Mbox3SpecificParams,
        prev: &mut Mbox3SpecificParams,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut new = vec![0u8; sections.application.size];
        serialize(params, &mut new)
            .map_err(|cause| Error::new(ProtocolExtensionError::Appl, &cause))?;

        let mut old = vec![0u8; sections.application.size];
        serialize(prev, &mut old)
            .map_err(|cause| Error::new(ProtocolExtensionError::Appl, &cause))?;

        (0..sections.application.size)
            .step_by(4)
            .try_for_each(|pos| {
                if new[pos..(pos + 4)] != old[pos..(pos + 4)] {
                    ApplSectionProtocol::write_appl_data(
                        req,
                        node,
                        sections,
                        pos,
                        &mut new[pos..(pos + 4)],
                        timeout_ms,
                    )
                } else {
                    Ok(())
                }
            })?;

        deserialize(prev, &new).map_err(|cause| Error::new(ProtocolExtensionError::Appl, &cause))
    }

    /// Cache state of hardware for notified parameters.
    pub fn cache_notified_params(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        msg: u32,
        params: &mut Mbox3SpecificParams,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        if msg & (PHANTOM_POWERING_CHANGED | MASTER_KNOB_CHANGED) > 0 {
            Self::cache_whole_params(req, node, sections, params, timeout_ms)?;
        }

        let mut p = params.clone();
        if msg & SPKR_BUTTON_PUSHED > 0 {
            p.spkr_led = match params.spkr_led {
                SpkrLedState::Off => SpkrLedState::Green,
                SpkrLedState::GreenBlink => SpkrLedState::Green,
                SpkrLedState::Green => SpkrLedState::Red,
                SpkrLedState::RedBlink => SpkrLedState::Red,
                SpkrLedState::Red => SpkrLedState::Orange,
                SpkrLedState::OrangeBlink => SpkrLedState::Orange,
                SpkrLedState::Orange => SpkrLedState::Off,
            };
        }

        if msg & SPKR_BUTTON_HELD > 0 {
            p.spkr_led = match params.spkr_led {
                SpkrLedState::Off => SpkrLedState::Off,
                SpkrLedState::GreenBlink => SpkrLedState::Green,
                SpkrLedState::Green => SpkrLedState::GreenBlink,
                SpkrLedState::RedBlink => SpkrLedState::Red,
                SpkrLedState::Red => SpkrLedState::RedBlink,
                SpkrLedState::OrangeBlink => SpkrLedState::Orange,
                SpkrLedState::Orange => SpkrLedState::OrangeBlink,
            };
        }

        if msg & MONO_BUTTON_PUSHED > 0 {
            p.mono_led = match params.mono_led {
                MonoLedState::Off => MonoLedState::On,
                MonoLedState::On => MonoLedState::Off,
            };
        }

        if msg & MUTE_BUTTON_PUSHED > 0 {
            p.mute_led = match params.mute_led {
                MuteLedState::Off => MuteLedState::On,
                MuteLedState::Blink => MuteLedState::On,
                MuteLedState::On => MuteLedState::Off,
            };
        }

        if msg & MUTE_BUTTON_HELD > 0 {
            p.mute_led = match params.mute_led {
                MuteLedState::Off => MuteLedState::Off,
                MuteLedState::Blink => MuteLedState::On,
                MuteLedState::On => MuteLedState::Blink,
            };
        }

        if !p.eq(params) {
            Self::update_partial_params(req, node, sections, &p, params, timeout_ms)?;
        }

        Ok(())
    }

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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn mbox3_specific_params_serdes() {
        let params = Mbox3SpecificParams {
            standalone_use_case: StandaloneUseCase::AdDa,
            master_knob_value: 0xf9,
            master_knob_assigns: [true, false, true, true, false, true],
            mute_led: MuteLedState::Blink,
            mono_led: MonoLedState::On,
            spkr_led: SpkrLedState::RedBlink,
            dim_led: false,
            duration_hold: 10,
            phantom_powering: true,
            hpf_enables: [false, false, true, false],
            output_trims: [0, 1, 2, 3, 4, 5],
            reverb_type: ReverbType::Hall2,
            reverb_volume: 0xb3,
            reverb_duration: 0xa5,
            reverb_feedback: 0x17,
        };

        let mut raw = [0; MIN_SIZE];
        serialize(&params, &mut raw).unwrap();

        let mut p = Mbox3SpecificParams::default();
        deserialize(&mut p, &raw).unwrap();

        assert_eq!(params, p);
    }
}
