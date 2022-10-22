// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol specific to Focusrite Liquid Saffire 56.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by Focusrite for Liquid Saffire 56.
//!
//! ## Diagram of internal signal flow for Liquid Saffire 56.
//!
//! I note that optical input interface is available exclusively for ADAT input and S/PDIF input.
//!
//! ```text
//!
//! XLR input 1 ------+---------+------------------> analog-input-1/2
//! Phone input 1-----+         |
//!                             |
//! XLR input 2 ------+---------+
//! Phone input 2 ----+
//!
//! XLR input 3 ------+---------+------------------> analog-input-3/4
//! Phone input 3-----+         |
//!                             |
//! XLR input 4 ------+---------+
//! Phone input 4 ----+
//!
//! XLR input 5 ------+---------+------------------> analog-input-5/6
//! Phone input 5-----+         |
//!                             |
//! XLR input 6 ------+---------+
//! Phone input 6 ----+
//!
//! XLR input 7 ------+---------+------------------> analog-input-7/8
//! Phone input 7-----+         |
//!                             |
//! XLR input 8 ------+---------+
//! Phone input 8 ----+
//!
//! Coaxial input 1/2 -----------------------------> spdif-input-1/2
//! Optical input A -------------------------------> adat-input-1..8
//! Optical input B ------------or-----------------> spdif-input-3/4
//!                             +------------------> adat-input-9..16
//!
//!                          ++=============++
//! analog-input-1/2 ------> ||   72 x 76   || ----> analog-output-1/2
//! analog-input-3/4 ------> ||   router    || ----> analog-output-3/4
//! analog-input-5/6 ------> ||   up to     || ----> analog-output-5/6
//! analog-input-7/8 ------> || 128 entries || ----> analog-output-7/8
//!                          ||             || ----> analog-output-9/10
//! spdif-input-1/2 -------> ||             || ----> spdif-output-1/2
//! spdif-input-3/4 -------> ||             || ----> spdif-output-3/4
//! adat-input-1/2 --------> ||             || ----> adat-input-1/2
//! adat-input-3/4 --------> ||             || ----> adat-input-3/4
//! adat-input-5/6 --------> ||             || ----> adat-input-5/6
//! adat-input-7/8 --------> ||             || ----> adat-input-7/8
//! adat-input-9/10 -------> ||             || ----> adat-input-9/10
//! adat-input-11/12 ------> ||             || ----> adat-input-11/12
//! adat-input-13/14 ------> ||             || ----> adat-input-13/14
//! adat-input-15/16 ------> ||             || ----> adat-input-15/16
//!                          ||             ||
//! stream-input-A-1/2 ----> ||             || ----> stream-output-A-1/2
//! stream-input-A-3/4 ----> ||             || ----> stream-output-A-3/4
//! stream-input-A-5/6 ----> ||             || ----> stream-output-A-5/6
//! stream-input-A-7/8 ----> ||             || ----> stream-output-A-7/8
//! stream-input-A-9/10 ---> ||             || ----> stream-output-A-9/10
//! stream-input-A-11/12 --> ||             || ----> stream-output-A-11/12
//! stream-input-A-13/14 --> ||             || ----> stream-output-A-13/14
//! stream-input-A-15/16 --> ||             || ----> stream-output-A-15/16
//!                          ||             ||
//! stream-input-B-1/2 ----> ||             || ----> stream-output-B-1/2
//! stream-input-B-3/4 ----> ||             || ----> stream-output-B-3/4
//! stream-input-B-5/6 ----> ||             || ----> stream-output-B-5/6
//! stream-input-B-7/8 ----> ||             || ----> stream-output-B-7/8
//! stream-input-B-9/10 ---> ||             || ----> stream-output-B-9/10
//! stream-input-B-11/12 --> ||             || ----> stream-output-B-11/12
//!                          ||             ||
//! mixer-output-1/2 ------> ||             || ----> mixer-input-1/2
//! mixer-output-3/4 ------> ||             || ----> mixer-input-3/4
//! mixer-output-5/6 ------> ||             || ----> mixer-input-5/6
//! mixer-output-7/8 ------> ||             || ----> mixer-input-7/8
//! mixer-output-9/10 -----> ||             || ----> mixer-input-9/10
//! mixer-output-11/12 ----> ||             || ----> mixer-input-11/12
//! mixer-output-13/14 ----> ||             || ----> mixer-input-13/14
//! mixer-output-15/16 ----> ||             || ----> mixer-input-15/16
//!                          ||             || ----> mixer-input-17/18
//!                          ++=============++
//!
//!                          ++=============++
//!                          ||             || ----> Phone output 1/2
//!                          ||             || ----> Phone output 3/4
//! analog-output-1/2 -----> ||             || ----> Phone output 5/6
//! analog-output-3/4 -----> ||   output    ||
//! analog-output-5/6 -----> ||             || --+-> Phone output 7/8
//! analog-output-7/8 -----> ||   group     ||   +-> Headphone output 1/2
//! analog-output-9/10 ----> ||             ||
//!                          ||             || --+-> Phone output 9/10
//!                          ||             ||   +-> Headphone output 3/4
//!                          ||             ||
//!                          ++=============++
//!
//! spdif-output-1/2 ------------------------------> Coaxial output 1/2
//!
//! adat-output-1..8 ------------------------------> Optical output A
//!
//! spdif-output-3/4 -------------or---------------> Optical output B
//! adat-output-1..8 -------------+
//!
//! ```

use super::{tcat::tcd22xx_spec::*, *};

// const EMULATION_TYPE_OFFSET: usize = 0x0278;
// const HARMONICS_OFFSET: usize = 0x0280;
// const POLARITY_OFFSET: usize = 0x0288;
// const METER_DISPLAY_TARGET_OFFSET: usize = 0x029c;
// const ANALOG_INPUT_LEVEL_OFFSET: usize = 0x02b4;
// const LED_OFFSET: usize = 0x02bc;

/// Protocol implementation specific to Liquid Saffire 56.
#[derive(Default, Debug)]
pub struct LiquidS56Protocol;

impl TcatOperation for LiquidS56Protocol {}

impl TcatGlobalSectionSpecification for LiquidS56Protocol {}

impl TcatExtensionOperation for LiquidS56Protocol {}

impl Tcd22xxSpecOperation for LiquidS56Protocol {
    const INPUTS: &'static [Input] = &[
        Input {
            id: SrcBlkId::Ins0,
            offset: 0,
            count: 2,
            label: None,
        },
        Input {
            id: SrcBlkId::Ins1,
            offset: 2,
            count: 6,
            label: None,
        },
        Input {
            id: SrcBlkId::Adat,
            offset: 0,
            count: 8,
            label: None,
        },
        Input {
            id: SrcBlkId::Aes,
            offset: 0,
            count: 2,
            label: Some("S/PDIF-coax"),
        },
        // NOTE: share the same optical interface.
        Input {
            id: SrcBlkId::Adat,
            offset: 8,
            count: 8,
            label: None,
        },
        Input {
            id: SrcBlkId::Aes,
            offset: 6,
            count: 2,
            label: Some("S/PDIF-opt"),
        },
    ];
    const OUTPUTS: &'static [Output] = &[
        Output {
            id: DstBlkId::Ins0,
            offset: 0,
            count: 2,
            label: None,
        },
        Output {
            id: DstBlkId::Ins1,
            offset: 0,
            count: 8,
            label: None,
        },
        Output {
            id: DstBlkId::Adat,
            offset: 0,
            count: 8,
            label: None,
        },
        Output {
            id: DstBlkId::Aes,
            offset: 0,
            count: 2,
            label: Some("S/PDIF-coax"),
        },
        // NOTE: share the same optical interface.
        Output {
            id: DstBlkId::Adat,
            offset: 8,
            count: 8,
            label: None,
        },
        Output {
            id: DstBlkId::Aes,
            offset: 6,
            count: 2,
            label: Some("S/PDIF-opt"),
        },
    ];
    // NOTE: The 8 entries are selected by unique protocol from the first 26 entries in router
    // section are used to display hardware metering.
    const FIXED: &'static [SrcBlk] = &[
        SrcBlk {
            id: SrcBlkId::Ins1,
            ch: 0,
        },
        SrcBlk {
            id: SrcBlkId::Ins1,
            ch: 1,
        },
        SrcBlk {
            id: SrcBlkId::Ins1,
            ch: 2,
        },
        SrcBlk {
            id: SrcBlkId::Ins1,
            ch: 3,
        },
        SrcBlk {
            id: SrcBlkId::Ins1,
            ch: 4,
        },
        SrcBlk {
            id: SrcBlkId::Ins1,
            ch: 5,
        },
        SrcBlk {
            id: SrcBlkId::Ins1,
            ch: 6,
        },
        SrcBlk {
            id: SrcBlkId::Ins1,
            ch: 7,
        },
        SrcBlk {
            id: SrcBlkId::Aes,
            ch: 0,
        },
        SrcBlk {
            id: SrcBlkId::Aes,
            ch: 1,
        },
        SrcBlk {
            id: SrcBlkId::Adat,
            ch: 0,
        },
        SrcBlk {
            id: SrcBlkId::Adat,
            ch: 1,
        },
        SrcBlk {
            id: SrcBlkId::Adat,
            ch: 2,
        },
        SrcBlk {
            id: SrcBlkId::Adat,
            ch: 3,
        },
        SrcBlk {
            id: SrcBlkId::Adat,
            ch: 4,
        },
        SrcBlk {
            id: SrcBlkId::Adat,
            ch: 5,
        },
        SrcBlk {
            id: SrcBlkId::Adat,
            ch: 6,
        },
        SrcBlk {
            id: SrcBlkId::Adat,
            ch: 7,
        },
        SrcBlk {
            id: SrcBlkId::Adat,
            ch: 8,
        },
        SrcBlk {
            id: SrcBlkId::Adat,
            ch: 9,
        },
        SrcBlk {
            id: SrcBlkId::Adat,
            ch: 10,
        },
        SrcBlk {
            id: SrcBlkId::Adat,
            ch: 11,
        },
        SrcBlk {
            id: SrcBlkId::Adat,
            ch: 12,
        },
        SrcBlk {
            id: SrcBlkId::Adat,
            ch: 13,
        },
        SrcBlk {
            id: SrcBlkId::Adat,
            ch: 14,
        },
        SrcBlk {
            id: SrcBlkId::Adat,
            ch: 15,
        },
    ];
}

impl SaffireproSwNoticeOperation for LiquidS56Protocol {
    const SW_NOTICE_OFFSET: usize = 0x02c8;
}

const SRC_SW_NOTICE: u32 = 0x00000001;
const DIM_MUTE_SW_NOTICE: u32 = 0x00000003;
const MIC_AMP_1_HARMONICS_SW_NOTICE: u32 = 0x00000006;
const MIC_AMP_2_HARMONICS_SW_NOTICE: u32 = 0x00000007;
const MIC_AMP_1_EMULATION_SW_NOTICE: u32 = 0x00000008;
const MIC_AMP_2_EMULATION_SW_NOTICE: u32 = 0x00000009;
const MIC_AMP_POLARITY_SW_NOTICE: u32 = 0x0000000a;
const INPUT_LEVEL_SW_NOTICE: u32 = 0x0000000b;

impl SaffireproOutGroupSpecification for LiquidS56Protocol {
    const OUT_GROUP_STATE_OFFSET: usize = 0x000c;

    const ENTRY_COUNT: usize = 10;
    const HAS_VOL_HWCTL: bool = true;

    const SRC_NOTICE: u32 = SRC_SW_NOTICE;
    const DIM_MUTE_NOTICE: u32 = DIM_MUTE_SW_NOTICE;
}

impl SaffireproIoParamsSpecification for LiquidS56Protocol {
    const AESEBU_IS_SUPPORTED: bool = true;
    const MIC_PREAMP_TRANSFORMER_IS_SUPPORTED: bool = true;
}

/// Emulation type of mic pre amp.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MicAmpEmulationType {
    Flat,
    Trany1h,
    Silver2,
    FfRed1h,
    Savillerow,
    Dunk,
    ClassA2a,
    OldTube,
    Deutsch72,
    Stellar1b,
    NewAge,
    Reserved(u32),
}

impl Default for MicAmpEmulationType {
    fn default() -> Self {
        Self::Flat
    }
}

impl MicAmpEmulationType {
    const FLAT_VALUE: u32 = 0;
    const TRANY1H_VALUE: u32 = 1;
    const SILVER2_VALUE: u32 = 2;
    const FFRED1H_VALUE: u32 = 3;
    const SAVILLEROW_VALUE: u32 = 4;
    const DUNK_VALUE: u32 = 5;
    const CLASSA2A_VALUE: u32 = 6;
    const OLDTUBE_VALUE: u32 = 7;
    const DEUTSCH72_VALUE: u32 = 8;
    const STELLAR1B_VALUE: u32 = 9;
    const NEWAGE_VALUE: u32 = 10;
}

fn serialize_mic_amp_emulation_types(
    emulation_types: &[MicAmpEmulationType; 2],
    raw: &mut [u8],
) -> Result<(), String> {
    assert!(raw.len() >= 8);

    emulation_types
        .iter()
        .enumerate()
        .for_each(|(i, emulation_type)| {
            let pos = i * 4;
            match emulation_type {
                MicAmpEmulationType::Flat => MicAmpEmulationType::FLAT_VALUE,
                MicAmpEmulationType::Trany1h => MicAmpEmulationType::TRANY1H_VALUE,
                MicAmpEmulationType::Silver2 => MicAmpEmulationType::SILVER2_VALUE,
                MicAmpEmulationType::FfRed1h => MicAmpEmulationType::FFRED1H_VALUE,
                MicAmpEmulationType::Savillerow => MicAmpEmulationType::SAVILLEROW_VALUE,
                MicAmpEmulationType::Dunk => MicAmpEmulationType::DUNK_VALUE,
                MicAmpEmulationType::ClassA2a => MicAmpEmulationType::CLASSA2A_VALUE,
                MicAmpEmulationType::OldTube => MicAmpEmulationType::OLDTUBE_VALUE,
                MicAmpEmulationType::Deutsch72 => MicAmpEmulationType::DEUTSCH72_VALUE,
                MicAmpEmulationType::Stellar1b => MicAmpEmulationType::STELLAR1B_VALUE,
                MicAmpEmulationType::NewAge => MicAmpEmulationType::NEWAGE_VALUE,
                // TODO
                MicAmpEmulationType::Reserved(_) => unreachable!(),
            }
            .build_quadlet(&mut raw[pos..(pos + 4)]);
        });

    Ok(())
}

fn deserialize_mic_amp_emulation_types(
    emulation_types: &mut [MicAmpEmulationType; 2],
    raw: &[u8],
) -> Result<(), String> {
    assert!(raw.len() >= 8);

    let mut val = 0u32;

    emulation_types
        .iter_mut()
        .enumerate()
        .try_for_each(|(i, emulation_type)| {
            let pos = i * 4;
            val.parse_quadlet(&raw[pos..(pos + 4)]);

            *emulation_type = match val {
                MicAmpEmulationType::FLAT_VALUE => MicAmpEmulationType::Flat,
                MicAmpEmulationType::TRANY1H_VALUE => MicAmpEmulationType::Trany1h,
                MicAmpEmulationType::SILVER2_VALUE => MicAmpEmulationType::Silver2,
                MicAmpEmulationType::FFRED1H_VALUE => MicAmpEmulationType::FfRed1h,
                MicAmpEmulationType::SAVILLEROW_VALUE => MicAmpEmulationType::Savillerow,
                MicAmpEmulationType::DUNK_VALUE => MicAmpEmulationType::Dunk,
                MicAmpEmulationType::CLASSA2A_VALUE => MicAmpEmulationType::ClassA2a,
                MicAmpEmulationType::OLDTUBE_VALUE => MicAmpEmulationType::OldTube,
                MicAmpEmulationType::DEUTSCH72_VALUE => MicAmpEmulationType::Deutsch72,
                MicAmpEmulationType::STELLAR1B_VALUE => MicAmpEmulationType::Stellar1b,
                MicAmpEmulationType::NEWAGE_VALUE => MicAmpEmulationType::NewAge,
                _ => Err(format!(
                    "Mic amplifier emulation type not found for value: {}",
                    val
                ))?,
            };

            Ok(())
        })
}

fn serialize_mic_amp_harmonics(harmonics: &[u8; 2], raw: &mut [u8]) -> Result<(), String> {
    assert!(raw.len() >= 8);

    harmonics.iter().enumerate().for_each(|(i, &h)| {
        let pos = i * 4;
        let val = h as u32;
        val.build_quadlet(&mut raw[pos..(pos + 4)]);
    });

    Ok(())
}

fn deserialize_mic_amp_harmonics(harmonics: &mut [u8; 2], raw: &[u8]) -> Result<(), String> {
    assert!(raw.len() >= 8);

    let mut val = 0u32;

    harmonics.iter_mut().enumerate().for_each(|(i, h)| {
        let pos = i * 4;
        val.parse_quadlet(&raw[pos..(pos + 4)]);
        *h = val as u8;
    });

    Ok(())
}

fn serialize_mic_amp_polarities(polarities: &[bool; 2], raw: &mut [u8]) -> Result<(), String> {
    assert!(raw.len() >= 8);

    polarities.iter().enumerate().for_each(|(i, &polarity)| {
        let pos = i * 4;
        (polarity as u32).build_quadlet(&mut raw[pos..(pos + 4)]);
    });

    Ok(())
}

fn deserialize_mic_amp_polarities(polarities: &mut [bool; 2], raw: &[u8]) -> Result<(), String> {
    assert!(raw.len() >= 8);

    let mut val = 0u32;

    polarities.iter_mut().enumerate().for_each(|(i, polarity)| {
        let pos = i * 4;
        val.parse_quadlet(&raw[pos..(pos + 4)]);
        *polarity = val > 0;
    });

    Ok(())
}

/// Level of analog input.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum AnalogInputLevel {
    Line,
    Mic,
    /// Available for Analog input 3 and 4 only.
    Inst,
    Reserved(u8),
}

impl Default for AnalogInputLevel {
    fn default() -> Self {
        Self::Line
    }
}

impl AnalogInputLevel {
    const LINE_VALUE: u8 = 0;
    const MIC_VALUE: u8 = 1;
    const INST_VALUE: u8 = 2;
}

fn serialize_analog_input_levels(
    levels: &[AnalogInputLevel; 8],
    raw: &mut [u8],
) -> Result<(), String> {
    assert!(raw.len() >= 8);

    (0..levels.len()).step_by(4).for_each(|pos| {
        let mut val = 0u32;
        levels[pos..(pos + 4)]
            .iter()
            .enumerate()
            .for_each(|(j, level)| {
                let v = match level {
                    AnalogInputLevel::Line => AnalogInputLevel::LINE_VALUE,
                    AnalogInputLevel::Mic => AnalogInputLevel::MIC_VALUE,
                    AnalogInputLevel::Inst => AnalogInputLevel::INST_VALUE,
                    // TODO
                    _ => unreachable!(),
                };
                val |= (v as u32) << (j * 8);
            });
        val.build_quadlet(&mut raw[pos..(pos + 4)]);
    });

    Ok(())
}

fn deserialize_analog_input_levels(
    levels: &mut [AnalogInputLevel; 8],
    raw: &[u8],
) -> Result<(), String> {
    assert!(raw.len() >= 8);

    let mut val = 0u32;

    (0..levels.len()).step_by(4).try_for_each(|pos| {
        val.parse_quadlet(&raw[pos..(pos + 4)]);

        levels[pos..(pos + 4)]
            .iter_mut()
            .enumerate()
            .try_for_each(|(j, level)| {
                let v = ((val >> (j * 8)) & 0x000000ff) as u8;
                *level = match v {
                    AnalogInputLevel::LINE_VALUE => AnalogInputLevel::Line,
                    AnalogInputLevel::MIC_VALUE => AnalogInputLevel::Mic,
                    AnalogInputLevel::INST_VALUE => AnalogInputLevel::Inst,
                    _ => Err(format!("Analog input level not found for value: {}", val))?,
                };
                Ok(())
            })
    })
}

/// Target of meter display.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct LedState {
    /// For `ADAT 1` LED.
    pub adat1: bool,
    /// For `ADAT 2` LED.
    pub adat2: bool,
    /// For SPDIF LED.
    pub spdif: bool,
    /// For `MIDI In` LED.
    pub midi_in: bool,
}

impl LedState {
    const ADAT1_FLAG: u32 = 0x00000001;
    const ADAT2_FLAG: u32 = 0x00000002;
    const SPDIF_FLAG: u32 = 0x00000004;
    const MIDI_IN_FLAG: u32 = 0x00000008;
}

fn serialize_led_state(state: &LedState, raw: &mut [u8]) -> Result<(), String> {
    assert!(raw.len() >= 4);

    let mut val = 0u32;

    [
        (&state.adat1, LedState::ADAT1_FLAG),
        (&state.adat2, LedState::ADAT2_FLAG),
        (&state.spdif, LedState::SPDIF_FLAG),
        (&state.midi_in, LedState::MIDI_IN_FLAG),
    ]
    .iter_mut()
    .filter(|(&on, _)| on)
    .for_each(|(_, flag)| val |= *flag);

    val.build_quadlet(raw);

    Ok(())
}

fn deserialize_led_state(state: &mut LedState, raw: &[u8]) -> Result<(), String> {
    assert!(raw.len() >= 4);

    let mut val = 0u32;
    val.parse_quadlet(&raw);

    [
        (&mut state.adat1, LedState::ADAT1_FLAG),
        (&mut state.adat2, LedState::ADAT2_FLAG),
        (&mut state.spdif, LedState::SPDIF_FLAG),
        (&mut state.midi_in, LedState::MIDI_IN_FLAG),
    ]
    .iter_mut()
    .for_each(|(on, flag)| **on = val & *flag > 0);

    Ok(())
}

/// The target to display meter.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MeterDisplayTarget {
    AnalogInput0,
    AnalogInput1,
    AnalogInput2,
    AnalogInput3,
    AnalogInput4,
    AnalogInput5,
    AnalogInput6,
    AnalogInput7,
    SpdifInput0,
    SpdifInput1,
    AdatInput0,
    AdatInput1,
    AdatInput2,
    AdatInput3,
    AdatInput4,
    AdatInput5,
    AdatInput6,
    AdatInput7,
    AdatInput8,
    AdatInput9,
    AdatInput10,
    AdatInput11,
    AdatInput12,
    AdatInput13,
    AdatInput14,
    AdatInput15,
}

impl Default for MeterDisplayTarget {
    fn default() -> Self {
        MeterDisplayTarget::AnalogInput0
    }
}

const METER_DISPLAY_TARGETS: &[MeterDisplayTarget] = &[
    MeterDisplayTarget::AnalogInput0,
    MeterDisplayTarget::AnalogInput1,
    MeterDisplayTarget::AnalogInput2,
    MeterDisplayTarget::AnalogInput3,
    MeterDisplayTarget::AnalogInput4,
    MeterDisplayTarget::AnalogInput5,
    MeterDisplayTarget::AnalogInput6,
    MeterDisplayTarget::AnalogInput7,
    MeterDisplayTarget::SpdifInput0,
    MeterDisplayTarget::SpdifInput1,
    MeterDisplayTarget::AdatInput0,
    MeterDisplayTarget::AdatInput1,
    MeterDisplayTarget::AdatInput2,
    MeterDisplayTarget::AdatInput3,
    MeterDisplayTarget::AdatInput4,
    MeterDisplayTarget::AdatInput5,
    MeterDisplayTarget::AdatInput6,
    MeterDisplayTarget::AdatInput7,
    MeterDisplayTarget::AdatInput8,
    MeterDisplayTarget::AdatInput9,
    MeterDisplayTarget::AdatInput10,
    MeterDisplayTarget::AdatInput11,
    MeterDisplayTarget::AdatInput12,
    MeterDisplayTarget::AdatInput13,
    MeterDisplayTarget::AdatInput14,
    MeterDisplayTarget::AdatInput15,
];

fn serialize_meter_display_targets(
    targets: &[MeterDisplayTarget; 8],
    raw: &mut [u8],
) -> Result<(), String> {
    assert!(raw.len() >= 8);

    (0..targets.len()).step_by(4).for_each(|pos| {
        let mut val = 0u32;

        targets[pos..(pos + 4)]
            .iter()
            .enumerate()
            .for_each(|(j, target)| {
                let pos = METER_DISPLAY_TARGETS
                    .iter()
                    .position(|t| target.eq(t))
                    .unwrap();
                val |= (pos as u32) << (j * 8);
            });

        val.build_quadlet(&mut raw[pos..(pos + 4)]);
    });

    Ok(())
}

fn deserialize_meter_display_targets(
    targets: &mut [MeterDisplayTarget; 8],
    raw: &[u8],
) -> Result<(), String> {
    assert!(raw.len() >= 8);

    let mut val = 0u32;

    (0..targets.len()).step_by(4).try_for_each(|pos| {
        val.parse_quadlet(&raw[pos..(pos + 4)]);

        targets[pos..(pos + 4)]
            .iter_mut()
            .enumerate()
            .try_for_each(|(j, target)| {
                let pos = ((val >> (j * 8)) & 0x000000ff) as usize;
                METER_DISPLAY_TARGETS
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| format!("Meter display target not found for position {}", pos))
                    .map(|&t| *target = t)
            })
    })
}

/// Parameters specific to liquid Saffire 56.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct LiquidS56SpecificParams {
    /// Emulation type of microphone amplifiers.
    pub mic_amp_emulation_types: [MicAmpEmulationType; 2],
    /// Harmonics of microphone amplifiers.
    pub mic_amp_harmonics: [u8; 2],
    /// Polarities of microphone amplifiers.
    pub mic_amp_polarities: [bool; 2],
    /// Nominal level of analog inputs.
    pub analog_input_levels: [AnalogInputLevel; 8],
    /// State of LEDs.
    pub led_states: LedState,
    /// Targets of meter display.
    pub meter_display_targets: [MeterDisplayTarget; 8],
}

const SPECIFIC_PARAMS_OFFSET: usize = 0x0278;
const SPECIFIC_PARAMS_SIZE: usize = 0x48;

fn serialize_specific_params(
    params: &LiquidS56SpecificParams,
    raw: &mut [u8],
) -> Result<(), String> {
    assert!(raw.len() >= SPECIFIC_PARAMS_SIZE);

    serialize_mic_amp_emulation_types(&params.mic_amp_emulation_types, &mut raw[..0x08])?;
    serialize_mic_amp_harmonics(&params.mic_amp_harmonics, &mut raw[0x08..0x10])?;
    serialize_mic_amp_polarities(&params.mic_amp_polarities, &mut raw[0x10..0x18])?;
    serialize_meter_display_targets(&params.meter_display_targets, &mut raw[0x24..0x3c])?;
    serialize_analog_input_levels(&params.analog_input_levels, &mut raw[0x3c..0x44])?;
    serialize_led_state(&params.led_states, &mut raw[0x44..0x48])?;

    Ok(())
}

fn deserialize_specific_params(
    params: &mut LiquidS56SpecificParams,
    raw: &[u8],
) -> Result<(), String> {
    assert!(raw.len() >= SPECIFIC_PARAMS_SIZE);

    deserialize_mic_amp_emulation_types(&mut params.mic_amp_emulation_types, &raw[..0x08])?;
    deserialize_mic_amp_harmonics(&mut params.mic_amp_harmonics, &raw[0x08..0x10])?;
    deserialize_mic_amp_polarities(&mut params.mic_amp_polarities, &raw[0x10..0x18])?;
    deserialize_meter_display_targets(&mut params.meter_display_targets, &raw[0x24..0x3c])?;
    deserialize_analog_input_levels(&mut params.analog_input_levels, &raw[0x3c..0x44])?;
    deserialize_led_state(&mut params.led_states, &raw[0x44..0x48])?;

    Ok(())
}

impl ApplSectionParamsSerdes<LiquidS56SpecificParams> for LiquidS56Protocol {
    const APPL_PARAMS_OFFSET: usize = SPECIFIC_PARAMS_OFFSET;

    const APPL_PARAMS_SIZE: usize = SPECIFIC_PARAMS_SIZE;

    fn serialize_appl_params(
        params: &LiquidS56SpecificParams,
        raw: &mut [u8],
    ) -> Result<(), String> {
        serialize_specific_params(params, raw)
    }

    fn deserialize_appl_params(
        params: &mut LiquidS56SpecificParams,
        raw: &[u8],
    ) -> Result<(), String> {
        deserialize_specific_params(params, raw)
    }
}

/// Protocol specific to Saffire Pro 26.
impl LiquidS56Protocol {
    pub const MIC_AMP_HARMONICS_MIN: u8 = 0;
    pub const MIC_AMP_HARMONICS_MAX: u8 = 21;
}

impl TcatApplSectionParamsOperation<LiquidS56SpecificParams> for LiquidS56Protocol {}

impl TcatApplSectionMutableParamsOperation<LiquidS56SpecificParams> for LiquidS56Protocol {
    fn update_appl_partial_params(
        req: &FwReq,
        node: &FwNode,
        sections: &ExtensionSections,
        params: &LiquidS56SpecificParams,
        prev: &mut LiquidS56SpecificParams,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut new = vec![0u8; SPECIFIC_PARAMS_SIZE];
        serialize_specific_params(params, &mut new)
            .map_err(|cause| Error::new(ProtocolExtensionError::Appl, &cause))?;

        let mut old = vec![0u8; SPECIFIC_PARAMS_SIZE];
        serialize_specific_params(prev, &mut old)
            .map_err(|cause| Error::new(ProtocolExtensionError::Appl, &cause))?;

        (0..SPECIFIC_PARAMS_SIZE).step_by(4).try_for_each(|pos| {
            if new[pos..(pos + 4)] != old[pos..(pos + 4)] {
                Self::write_extension(
                    req,
                    node,
                    &sections.application,
                    SPECIFIC_PARAMS_OFFSET + pos,
                    &mut new[pos..(pos + 4)],
                    timeout_ms,
                )
            } else {
                Ok(())
            }
        })?;

        [
            (0x00, MIC_AMP_1_EMULATION_SW_NOTICE),
            (0x04, MIC_AMP_2_EMULATION_SW_NOTICE),
            (0x08, MIC_AMP_1_HARMONICS_SW_NOTICE),
            (0x0c, MIC_AMP_2_HARMONICS_SW_NOTICE),
            (0x10, MIC_AMP_POLARITY_SW_NOTICE),
            (0x14, MIC_AMP_POLARITY_SW_NOTICE),
            (0x38, INPUT_LEVEL_SW_NOTICE),
            (0x3c, INPUT_LEVEL_SW_NOTICE),
        ]
        .iter()
        .filter(|(pos, _)| &new[*pos..(*pos + 4)] != &old[*pos..(*pos + 4)])
        .try_for_each(|(_, msg)| Self::write_sw_notice(req, node, sections, *msg, timeout_ms))?;

        deserialize_specific_params(prev, &new)
            .map_err(|cause| Error::new(ProtocolExtensionError::Appl, &cause))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn serdes_specific_params() {
        let params = LiquidS56SpecificParams {
            mic_amp_emulation_types: [MicAmpEmulationType::Savillerow, MicAmpEmulationType::Dunk],
            mic_amp_harmonics: [0x59, 0xb2],
            mic_amp_polarities: [false, true],
            analog_input_levels: [
                AnalogInputLevel::Line,
                AnalogInputLevel::Mic,
                AnalogInputLevel::Inst,
                AnalogInputLevel::Line,
                AnalogInputLevel::Mic,
                AnalogInputLevel::Inst,
                AnalogInputLevel::Line,
                AnalogInputLevel::Mic,
            ],
            led_states: LedState {
                adat1: false,
                adat2: true,
                spdif: false,
                midi_in: true,
            },
            meter_display_targets: [
                MeterDisplayTarget::AdatInput8,
                MeterDisplayTarget::AdatInput11,
                MeterDisplayTarget::AdatInput2,
                MeterDisplayTarget::AdatInput5,
                MeterDisplayTarget::AnalogInput5,
                MeterDisplayTarget::SpdifInput1,
                MeterDisplayTarget::AnalogInput0,
                MeterDisplayTarget::AnalogInput3,
            ],
        };
        let mut raw = vec![0u8; SPECIFIC_PARAMS_SIZE];
        serialize_specific_params(&params, &mut raw).unwrap();

        let mut p = LiquidS56SpecificParams::default();
        deserialize_specific_params(&mut p, &raw).unwrap();

        assert_eq!(params, p);
    }
}
