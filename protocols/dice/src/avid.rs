// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Hardware specification and application protocol specific to Avid Mbox 3 Pro
//!
//! The module includes structure, enumeration, and trait and its implementation for hardware
//! specification and application protocol specific to Avid Mbox 3 Pro.
//!
//! ## Diagram of internal signal flow
//!
//! ```text
//!
//! XLR input 1 -----------------+-------+-------> analog-input-1/2
//! Phone input 1 ---------------+       |
//!                                      |
//! XLR input 2 -----------------+-------+
//! Phone input 2 ---------------+
//!
//! XLR input 3 -----------------+-------+-------> analog-input-3/4
//! Phone input 3 ---------------+       |
//!                                      |
//! XLR input 4 -----------------+-------+
//! Phone input 4 ---------------+
//!
//! RCA input 5/6 ---------------or--------------> analog-input-5/6
//! Mini Phone ------------------+
//! coaxial input 1/2 ---------------------------> spdif-input-1/2
//!
//!                          ++=============++
//! analog-input-1/2 ------> ||             || --> analog-output-1/2
//! analog-input-3/4 ------> ||             || --> analog-output-3/4
//! analog-input-5/6 ------> ||             || --> analog-output-5/6
//! spdif-input-1/2 -------> ||             || --> spdif-output-1/2
//!                          ||             ||
//!                          ||             || --> headphone-output-1/2
//!                          ||             || --> headphone-output-3/4
//!                          ||   34 x 42   ||
//! reberb-output-1/2 -----> ||             || --> reverb-input-1/2
//!                          ||   router    ||
//! stream-input-1/2 ------> ||             || --> stream-output-1/2
//! stream-input-3/4 ------> ||   up to     || --> stream-output-3/4
//! stream-input-5/6 ------> ||             || --> stream-output-5/6
//! stream-input-7/8 ------> || 128 entries || --> stream-output-7/8
//!                          ||             ||
//! mixer-output-1/2 ------> ||             || --> mixer-input-1/2
//! mixer-output-3/4 ------> ||             || --> mixer-input-3/4
//! mixer-output-5/6 ------> ||             || --> mixer-input-5/6
//! mixer-output-7/8 ------> ||             || --> mixer-input-7/8
//! mixer-output-9/10 -----> ||             || --> mixer-input-9/10
//! mixer-output-11/12 ----> ||             || --> mixer-input-11/12
//! mixer-output-13/14 ----> ||             || --> mixer-input-13/14
//! mixer-output-15/16 ----> ||             || --> mixer-input-15/16
//!                          ||             || --> mixer-input-17/18
//!                          ||             ||
//!                          ||             || --> control-room-output-1/2
//!                          ++=============++
//!
//!                          ++=============++
//! reverb-input-1/2 ------> ||    reverb   || --> reverb-output-1/2
//!                          ||    effect   ||
//!                          ++=============++
//!
//!                          ++=============++
//! mixer-input-1/2 -------> ||             || --> mixer-output-1/2
//! mixer-input-3/4 -------> ||             || --> mixer-output-3/4
//! mixer-input-5/6 -------> ||             || --> mixer-output-5/6
//! mixer-input-7/8 -------> ||   18 x 16   || --> mixer-output-7/8
//! mixer-input-9/10 ------> ||             || --> mixer-output-9/10
//! mixer-input-11/11 -----> ||    mixer    || --> mixer-output-11/12
//! mixer-input-13/14 -----> ||             || --> mixer-output-13/14
//! mixer-input-15/16 -----> ||             || --> mixer-output-15/16
//! mixer-input-17/18 -----> ||             ||
//!                          ++=============++
//!
//! analog-output-1/2 ---------------------------> Phone output 1/2
//! analog-output-3/4 ---------------------------> Phone output 3/4
//! analog-output-5/6 ---------------------------> Phone output 5/6
//!
//! headphone-output-1/2 ------------------------> Headphone output 1/2
//! headphone-output-3/4 ------------------------> Headphone output 3/4
//!
//! spdif-output-1/2 ----------------------------> Coaxial output 1/2
//!
//! ```

use super::{
    tcat::{
        extension::{appl_section::*, *},
        tcd22xx_spec::*,
        *,
    },
    *,
};

// const USE_CASE_OFFSET: usize = 0x00;
// (0x04 is unclear)
// const MASTER_KNOB_VALUE_OFFSET: usize = 0x08;
// const MASTER_KNOB_ASSIGN_OFFSET: usize = 0x0c;
// const MUTE_BUTTON_LED_OFFSET: usize = 0x10;
// const MONO_BUTTON_LED_OFFSET: usize = 0x14;  // 0x08 enables playback routing to analog input 3/4.
// const SPKR_BUTTON_LED_OFFSET: usize = 0x14;
// const ALL_LEDS_BRIGHTNESS_OFFSET: usize = 0x18;   // 0x00 - 0x7f.
// const FORCE_LEDS_BRIGHTNESS_OFFSET: usize = 0x1c;   // 0x00 - 0x7f.
// const BUTTON_HOLD_DURATION_OFFSET: usize = 0x20;
// const HPF_ENABLE_OFFSET: usize = 0x24; // High pass filter for analog inputs.
// const OUT_0_TRIM_OFFSET: usize = 0x28;
// const OUT_1_TRIM_OFFSET: usize = 0x2c;
// const OUT_2_TRIM_OFFSET: usize = 0x30;
// const OUT_3_TRIM_OFFSET: usize = 0x34;
// const OUT_4_TRIM_OFFSET: usize = 0x38;
// const OUT_5_TRIM_OFFSET: usize = 0x3c;
// const REVERB_TYPE_OFFSET: usize = 0x40;
// const REVERB_VOLUME_OFFSET: usize = 0x44;
// const REVERB_DURATION_OFFSET: usize = 0x48;
// const REVERB_FEEDBACK_OFFSET: usize = 0x4c;

/// Protocol implementation of Avid Mbox 3 Pro.
#[derive(Default, Debug)]
pub struct Mbox3Protocol;

impl TcatOperation for Mbox3Protocol {}

impl TcatGlobalSectionSpecification for Mbox3Protocol {}

impl TcatExtensionOperation for Mbox3Protocol {}

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

impl ApplSectionParamsSerdes<Mbox3SpecificParams> for Mbox3Protocol {
    const APPL_PARAMS_OFFSET: usize = 0;

    const APPL_PARAMS_SIZE: usize = MIN_SIZE;

    fn serialize_appl_params(params: &Mbox3SpecificParams, raw: &mut [u8]) -> Result<(), String> {
        serialize(params, raw)
    }

    fn deserialize_appl_params(params: &mut Mbox3SpecificParams, raw: &[u8]) -> Result<(), String> {
        deserialize(params, raw)
    }
}

impl TcatApplSectionParamsOperation<Mbox3SpecificParams> for Mbox3Protocol {}

impl TcatApplSectionMutableParamsOperation<Mbox3SpecificParams> for Mbox3Protocol {}

impl TcatApplSectionNotifiedParamsOperation<Mbox3SpecificParams> for Mbox3Protocol {
    fn cache_appl_notified_params(
        req: &FwReq,
        node: &FwNode,
        sections: &ExtensionSections,
        params: &mut Mbox3SpecificParams,
        msg: u32,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        if msg & (PHANTOM_POWERING_CHANGED | MASTER_KNOB_CHANGED) > 0 {
            Self::cache_appl_whole_params(req, node, sections, params, timeout_ms)?;
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
            Self::update_appl_partial_params(req, node, sections, &p, params, timeout_ms)?;
        }

        Ok(())
    }
}

impl Mbox3Protocol {
    /// Cache state of hardware for whole parameters.
    pub fn cache_whole_params(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        params: &mut Mbox3SpecificParams,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        Self::cache_appl_whole_params(req, node, sections, params, timeout_ms)
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
        Self::update_appl_partial_params(req, node, sections, params, prev, timeout_ms)
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
        Self::cache_appl_notified_params(req, node, sections, params, msg, timeout_ms)
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
