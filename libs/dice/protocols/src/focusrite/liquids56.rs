// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol specific to Focusrite Liquid Saffire 56.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by Focusrite for Liquid Saffire 56.

use crate::tcat::extension::*;
use crate::tcat::tcd22xx_spec::*;
use crate::*;

use super::*;

/// The structure to represent state of TCD22xx on Liquid Saffire 56.
#[derive(Debug)]
pub struct LiquidS56State{
    tcd22xx: Tcd22xxState,
    out_grp: OutGroupState,
}

impl Default for LiquidS56State {
    fn default() -> Self {
        LiquidS56State{
            tcd22xx: Tcd22xxState::default(),
            out_grp: Self::create_out_group_state(),
        }
    }
}

impl Tcd22xxSpec for LiquidS56State {
    const INPUTS: &'static [Input] = &[
        Input{id: SrcBlkId::Ins0, offset: 0, count: 2, label: None},
        Input{id: SrcBlkId::Ins1, offset: 0, count: 6, label: None},
        Input{id: SrcBlkId::Adat, offset: 0, count: 8, label: None},
        Input{id: SrcBlkId::Aes, offset: 0, count: 2, label: Some("S/PDIF-coax")},
        // NOTE: share the same optical interface.
        Input{id: SrcBlkId::Adat, offset: 8, count: 8, label: None},
        Input{id: SrcBlkId::Aes, offset: 6, count: 2, label: Some("S/PDIF-opt")},
    ];
    const OUTPUTS: &'static [Output] = &[
        Output{id: DstBlkId::Ins0, offset: 0, count: 2, label: None},
        Output{id: DstBlkId::Ins1, offset: 0, count: 8, label: None},
        Output{id: DstBlkId::Adat, offset: 0, count: 8, label: None},
        Output{id: DstBlkId::Aes, offset: 0, count: 2, label: Some("S/PDIF-coax")},
        // NOTE: share the same optical interface.
        Output{id: DstBlkId::Adat, offset: 8, count: 8, label: None},
        Output{id: DstBlkId::Aes, offset: 6, count: 2, label: Some("S/PDIF-opt")},
    ];
    // NOTE: The 8 entries are selected by unique protocol from the first 26 entries in router
    // section are used to display hardware metering.
    const FIXED: &'static [SrcBlk] = &[
        SrcBlk{id: SrcBlkId::Ins1, ch: 0},
        SrcBlk{id: SrcBlkId::Ins1, ch: 1},
        SrcBlk{id: SrcBlkId::Ins1, ch: 2},
        SrcBlk{id: SrcBlkId::Ins1, ch: 3},
        SrcBlk{id: SrcBlkId::Ins1, ch: 4},
        SrcBlk{id: SrcBlkId::Ins1, ch: 5},
        SrcBlk{id: SrcBlkId::Ins1, ch: 6},
        SrcBlk{id: SrcBlkId::Ins1, ch: 7},
        SrcBlk{id: SrcBlkId::Aes, ch: 0},
        SrcBlk{id: SrcBlkId::Aes, ch: 1},
        SrcBlk{id: SrcBlkId::Adat, ch: 0},
        SrcBlk{id: SrcBlkId::Adat, ch: 1},
        SrcBlk{id: SrcBlkId::Adat, ch: 2},
        SrcBlk{id: SrcBlkId::Adat, ch: 3},
        SrcBlk{id: SrcBlkId::Adat, ch: 4},
        SrcBlk{id: SrcBlkId::Adat, ch: 5},
        SrcBlk{id: SrcBlkId::Adat, ch: 6},
        SrcBlk{id: SrcBlkId::Adat, ch: 7},
        SrcBlk{id: SrcBlkId::Adat, ch: 8},
        SrcBlk{id: SrcBlkId::Adat, ch: 9},
        SrcBlk{id: SrcBlkId::Adat, ch: 10},
        SrcBlk{id: SrcBlkId::Adat, ch: 11},
        SrcBlk{id: SrcBlkId::Adat, ch: 12},
        SrcBlk{id: SrcBlkId::Adat, ch: 13},
        SrcBlk{id: SrcBlkId::Adat, ch: 14},
        SrcBlk{id: SrcBlkId::Adat, ch: 15},
    ];
}

impl AsMut<Tcd22xxState> for LiquidS56State {
    fn as_mut(&mut self) -> &mut Tcd22xxState {
        &mut self.tcd22xx
    }
}

impl AsRef<Tcd22xxState> for LiquidS56State {
    fn as_ref(&self) -> &Tcd22xxState {
        &self.tcd22xx
    }
}

const SW_NOTICE_OFFSET: usize = 0x02c8;

const SRC_SW_NOTICE: u32 = 0x00000001;
const DIM_MUTE_SW_NOTICE: u32 = 0x00000003;
const IO_FLAG_SW_NOTICE: u32 = 0x00000004;
const MIC_AMP_1_HARMONICS_SW_NOTICE: u32 = 0x00000006;
const MIC_AMP_2_HARMONICS_SW_NOTICE: u32 = 0x00000007;
const MIC_AMP_1_EMULATION_SW_NOTICE: u32 = 0x00000008;
const MIC_AMP_2_EMULATION_SW_NOTICE: u32 = 0x00000009;
const MIC_AMP_POLARITY_SW_NOTICE: u32 = 0x0000000a;
const INPUT_LEVEL_SW_NOTICE: u32 = 0x0000000b;

impl OutGroupSpec for LiquidS56State {
    const ENTRY_COUNT: usize = 10;
    const HAS_VOL_HWCTL: bool = true;
    const OUT_CTL_OFFSET: usize = 0x000c;
    const SW_NOTICE_OFFSET: usize = SW_NOTICE_OFFSET;

    const SRC_NOTICE: u32 = SRC_SW_NOTICE;
    const DIM_MUTE_NOTICE: u32 = DIM_MUTE_SW_NOTICE;
}

impl AsMut<OutGroupState> for LiquidS56State {
    fn as_mut(&mut self) -> &mut OutGroupState {
        &mut self.out_grp
    }
}

impl AsRef<OutGroupState> for LiquidS56State {
    fn as_ref(&self) -> &OutGroupState {
        &self.out_grp
    }
}

/// The enumeration to represent type of signal for optical output interface.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum OptOutIfaceMode {
    Adat,
    Spdif,
    AesEbu,
}

/// The enumeration to represent emulation type of mic pre amp.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum MicAmpEmulationType{
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

impl From<u32> for MicAmpEmulationType {
    fn from(val: u32) -> Self {
        match val {
            0x00 => Self::Flat,
            0x01 => Self::Trany1h,
            0x02 => Self::Silver2,
            0x03 => Self::FfRed1h,
            0x04 => Self::Savillerow,
            0x05 => Self::Dunk,
            0x06 => Self::ClassA2a,
            0x07 => Self::OldTube,
            0x08 => Self::Deutsch72,
            0x09 => Self::Stellar1b,
            0x0a => Self::NewAge,
            _ => Self::Reserved(val),
        }
    }
}

impl From<MicAmpEmulationType> for u32 {
    fn from(emulation_type: MicAmpEmulationType) -> Self {
        match emulation_type {
            MicAmpEmulationType::Flat => 0x00,
            MicAmpEmulationType::Trany1h => 0x01,
            MicAmpEmulationType::Silver2 => 0x02,
            MicAmpEmulationType::FfRed1h => 0x03,
            MicAmpEmulationType::Savillerow => 0x04,
            MicAmpEmulationType::Dunk => 0x05,
            MicAmpEmulationType::ClassA2a => 0x06,
            MicAmpEmulationType::OldTube => 0x07,
            MicAmpEmulationType::Deutsch72 => 0x08,
            MicAmpEmulationType::Stellar1b => 0x09,
            MicAmpEmulationType::NewAge => 0x0a,
            MicAmpEmulationType::Reserved(val) => val,
        }
    }
}

/// The enumeration to represent level of analog input.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum AnalogInputLevel {
    Line,
    Mic,
    /// Available for Analog input 3 and 4 only.
    Inst,
    Reserved(u8),
}

impl From<u8> for AnalogInputLevel {
    fn from(val: u8) -> Self {
        match val {
            0 => Self::Line,
            1 => Self::Mic,
            2 => Self::Inst,
            _ => Self::Reserved(val),
        }
    }
}

impl From<AnalogInputLevel> for u8 {
    fn from(level: AnalogInputLevel) -> Self {
        match level {
            AnalogInputLevel::Line => 0,
            AnalogInputLevel::Mic => 1,
            AnalogInputLevel::Inst => 2,
            AnalogInputLevel::Reserved(val) => val,
        }
    }
}

/// The enumeration to represent target of meter display.
#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
pub struct LedState{
    pub adat1: bool,
    pub adat2: bool,
    pub spdif: bool,
    pub midi_in: bool,
}

impl LedState {
    fn parse(&mut self, raw: &[u8]) {
        assert_eq!(raw.len() ,4);

        let mut val = 0u32;
        val.parse_quadlet(&raw);

        if val & 0x00000001 > 0 {
            self.adat1 = true;
        }
        if val & 0x00000002 > 0 {
            self.adat2 = true;
        }
        if val & 0x00000004 > 0 {
            self.spdif = true;
        }
        if val & 0x00000008 > 0 {
            self.midi_in = true;
        }
    }

    fn build(&self, raw: &mut [u8]) {
        assert_eq!(raw.len() ,4);

        let mut val = 0u32;
        if self.adat1 {
            val |= 0x00000001;
        }
        if self.adat2 {
            val |= 0x00000002;
        }
        if self.spdif {
            val |= 0x00000004;
        }
        if self.midi_in {
            val |= 0x00000008;
        }
        val.build_quadlet(raw);
    }
}

/// The trait to represent protocol specific to Saffire Pro 26.
pub trait LiquidS56Protocol<T> : ApplSectionProtocol<T>
    where T: AsRef<FwNode>,
{
    const ANALOG_OUT_0_1_PAD_OFFSET: usize = 0x0040;
    const IO_FLAGS_OFFSET: usize = 0x005c;
    const EMULATION_TYPE_OFFSET: usize = 0x0278;
    const HARMONICS_OFFSET: usize = 0x0280;
    const POLARITY_OFFSET: usize = 0x0288;
    const METER_DISPLAY_TARGET_OFFSET: usize = 0x029c;
    const ANALOG_INPUT_LEVEL_OFFSET: usize = 0x02b4;
    const LED_OFFSET: usize = 0x02bc;

    fn write_sw_notice(&self, node: &T, sections: &ExtensionSections, notice: u32, timeout_ms: u32)
        -> Result<(), Error>
    {
        let mut raw = [0;4];
        notice.build_quadlet(&mut raw);
        self.write_appl_data(node, sections, SW_NOTICE_OFFSET, &mut raw, timeout_ms)
    }

    fn read_analog_out_0_1_pad_offset(&self, node: &T, sections: &ExtensionSections, timeout_ms: u32)
        ->Result<bool, Error>
    {
        let mut raw = [0;4];
        self.read_appl_data(node, sections, Self::ANALOG_OUT_0_1_PAD_OFFSET, &mut raw, timeout_ms)
            .map(|_| u32::from_be_bytes(raw) > 0)
    }

    fn write_analog_out_0_1_pad_offset(&self, node: &T, sections: &ExtensionSections, enable: bool,
                                       timeout_ms: u32)
        -> Result<(), Error>
    {
        let mut raw = [0;4];
        enable.build_quadlet(&mut raw);
        self.write_appl_data(node, sections, Self::ANALOG_OUT_0_1_PAD_OFFSET, &mut raw, timeout_ms)?;
        self.write_sw_notice(node, sections, DIM_MUTE_SW_NOTICE, timeout_ms)
    }

    fn read_opt_out_iface_mode(&self, node: &T, sections: &ExtensionSections, timeout_ms: u32)
        -> Result<OptOutIfaceMode, Error>
    {
        let mut raw = [0;4];
        self.read_appl_data(node, sections, Self::IO_FLAGS_OFFSET, &mut raw, timeout_ms)
            .map(|_| {
                let val = u32::from_be_bytes(raw);
                if val & 0x00000001 > 0 {
                    OptOutIfaceMode::Spdif
                } else if val & 0x00000002 > 0 {
                    OptOutIfaceMode::AesEbu
                } else {
                    OptOutIfaceMode::Adat
                }
            })
    }

    fn write_opt_out_iface_mode(&self, node: &T, sections: &ExtensionSections, mode: OptOutIfaceMode,
                                timeout_ms: u32)
        -> Result<(), Error>
    {
        let mut raw = [0;4];
        self.read_appl_data(node, sections, Self::IO_FLAGS_OFFSET, &mut raw, timeout_ms)?;

        let mut val = u32::from_be_bytes(raw);
        val &= !0x00000003;

        if mode == OptOutIfaceMode::Spdif {
            val |= 0x00000001;
        } else if mode == OptOutIfaceMode::AesEbu {
            val |= 0x00000002;
        }
        val.build_quadlet(&mut raw);
        self.write_appl_data(node, sections, Self::IO_FLAGS_OFFSET, &mut raw, timeout_ms)?;
        self.write_sw_notice(node, sections, IO_FLAG_SW_NOTICE, timeout_ms)
    }

    fn read_mic_amp_transformer(&self, node: &T, sections: &ExtensionSections, ch: usize,
                                timeout_ms: u32)
        -> Result<bool, Error>
    {
        let mut raw = [0;4];
        self.read_appl_data(node, sections, Self::IO_FLAGS_OFFSET, &mut raw, timeout_ms)
            .map(|_| u32::from_be_bytes(raw) & (1 << (ch + 4)) > 0)
    }

    fn write_mic_amp_transformer(&self, node: &T, sections: &ExtensionSections, ch: usize,
                                 state: bool, timeout_ms: u32)
        -> Result<(), Error>
    {
        let mut raw = [0;4];
        self.read_appl_data(node, sections, Self::IO_FLAGS_OFFSET, &mut raw, timeout_ms)?;

        let mut val = u32::from_be_bytes(raw);
        val &= !0x00000018;
        if state {
            val |= 1 << (ch + 4);
        } else {
            val &= !(1 << (ch + 4));
        }
        val.build_quadlet(&mut raw);
        self.write_appl_data(node, sections, Self::IO_FLAGS_OFFSET, &mut raw, timeout_ms)?;
        let sw_notice = if ch == 0 {
            MIC_AMP_1_EMULATION_SW_NOTICE
        } else {
            MIC_AMP_2_EMULATION_SW_NOTICE
        };
        self.write_sw_notice(node, sections, sw_notice, timeout_ms)
    }

    fn read_mic_amp_emulation_type(&self, node: &T, sections: &ExtensionSections, ch: usize,
                                   timeout_ms: u32)
        -> Result<MicAmpEmulationType, Error>
    {
        assert!(ch < 2);

        let mut raw = [0;4];
        let offset = Self::EMULATION_TYPE_OFFSET + ch * 4;
        self.read_appl_data(node, sections, offset, &mut raw, timeout_ms)
            .map(|_| MicAmpEmulationType::from(u32::from_be_bytes(raw)))
    }

    fn write_mic_amp_emulation_type(&self, node: &T, sections: &ExtensionSections, ch: usize,
                                    emulation_type: MicAmpEmulationType, timeout_ms: u32)
        -> Result<(), Error>
    {
        assert!(ch < 2);

        let mut raw = [0;4];
        emulation_type.build_quadlet(&mut raw);
        let offset = Self::EMULATION_TYPE_OFFSET + ch * 4;
        self.write_appl_data(node, sections, offset, &mut raw, timeout_ms)?;

        let sw_notice = if ch == 0 {
            MIC_AMP_1_EMULATION_SW_NOTICE
        } else {
            MIC_AMP_2_EMULATION_SW_NOTICE
        };
        self.write_sw_notice(node, sections, sw_notice, timeout_ms)
    }

    /// The return value is between 0x00 to 0x15.
    fn read_mic_amp_harmonics(&self, node: &T, sections: &ExtensionSections, ch: usize,
                                 timeout_ms: u32)
        -> Result<u8, Error>
    {
        assert!(ch < 2);

        let mut raw = [0;4];
        let offset = Self::HARMONICS_OFFSET + ch * 4;
        self.read_appl_data(node, sections, offset, &mut raw, timeout_ms)
            .map(|_| u32::from_be_bytes(raw) as u8)
    }

    fn write_mic_amp_harmonics(&self, node: &T, sections: &ExtensionSections, ch: usize,
                                  harmonics: u8, timeout_ms: u32)
        -> Result<(), Error>
    {
        assert!(ch < 2);

        let mut raw = [0;4];
        (harmonics as u32).build_quadlet(&mut raw);
        let offset = Self::HARMONICS_OFFSET + ch * 4;
        self.write_appl_data(node, sections, offset, &mut raw, timeout_ms)?;

        let sw_notice = if ch == 0 {
            MIC_AMP_1_HARMONICS_SW_NOTICE
        } else {
            MIC_AMP_2_HARMONICS_SW_NOTICE
        };
        self.write_sw_notice(node, sections, sw_notice, timeout_ms)
    }

    fn read_mic_amp_polarity(&self, node: &T, sections: &ExtensionSections, ch: usize,
                                timeout_ms: u32)
        -> Result<bool, Error>
    {
        assert!(ch < 2);

        let mut raw = [0;4];
        let offset = Self::POLARITY_OFFSET + ch * 4;
        self.read_appl_data(node, sections, offset, &mut raw, timeout_ms)
            .map(|_| u32::from_be_bytes(raw) > 0)
    }

    fn write_mic_amp_polarity(&self, node: &T, sections: &ExtensionSections, ch: usize,
                                 inverted: bool, timeout_ms: u32)
        -> Result<(), Error>
    {
        assert!(ch < 2);

        let mut raw = [0;4];
        inverted.build_quadlet(&mut raw);
        let offset = Self::POLARITY_OFFSET + ch * 4;
        self.write_appl_data(node, sections, offset, &mut raw, timeout_ms)?;
        self.write_sw_notice(node, sections, MIC_AMP_POLARITY_SW_NOTICE, timeout_ms)
    }

    fn read_analog_input_level(&self, node: &T, sections: &ExtensionSections,
                               levels: &mut [AnalogInputLevel;8], timeout_ms: u32)
        -> Result<(), Error>
    {
        let mut raw = [0;8];
        self.read_appl_data(node, sections, Self::ANALOG_INPUT_LEVEL_OFFSET, &mut raw, timeout_ms)
            .map(|_| {
                let mut quads = [0u32;2];
                quads.parse_quadlet_block(&raw);
                levels[..4].iter_mut()
                    .zip(quads[0].to_ne_bytes().iter())
                    .for_each(|(level, &val)| *level = AnalogInputLevel::from(val));
                levels[4..].iter_mut()
                    .zip(quads[1].to_ne_bytes().iter())
                    .for_each(|(level, &val)| *level = AnalogInputLevel::from(val));
            })
    }

    fn write_analog_input_level(&self, node: &T, sections: &ExtensionSections, levels: &[AnalogInputLevel;8],
                                timeout_ms: u32)
        -> Result<(), Error>
    {
        let mut quads = [0u32;2];
        levels[..4].iter()
            .enumerate()
            .for_each(|(i, &level)| quads[0] |= (u8::from(level) as u32) << (i * 8));
        levels[4..].iter()
            .enumerate()
            .for_each(|(i, &level)| quads[1] |= (u8::from(level) as u32) << (i * 8));
        let mut raw = [0;8];
        quads.build_quadlet_block(&mut raw);
        self.write_appl_data(node, sections, Self::ANALOG_INPUT_LEVEL_OFFSET, &mut raw, timeout_ms)?;
        self.write_sw_notice(node, sections, INPUT_LEVEL_SW_NOTICE, timeout_ms)
    }

    fn read_led_state(&self, node: &T, sections: &ExtensionSections, state: &mut LedState, timeout_ms: u32)
        -> Result<(), Error>
    {
        let mut raw = [0;4];
        self.read_appl_data(node, sections, Self::LED_OFFSET, &mut raw, timeout_ms)
            .map(|_| state.parse(&raw))
    }

    fn write_led_state(&self, node: &T, sections: &ExtensionSections, state: &LedState, timeout_ms: u32)
        -> Result<(), Error>
    {
        let mut raw = [0;4];
        state.build(&mut raw);
        self.write_appl_data(node, sections, Self::LED_OFFSET, &mut raw, timeout_ms)
    }

    /// The target of meter display represent index of router entry.
    fn read_meter_display_targets(&self, node: &T, sections: &ExtensionSections, targets: &mut [usize;8],
                                  timeout_ms: u32)
        -> Result<(), Error>
    {
        let mut raw = [0;8];
        self.read_appl_data(node, sections, Self::METER_DISPLAY_TARGET_OFFSET, &mut raw, timeout_ms)
            .map(|_| {
                let mut quads = [0u32;2];
                quads[0].parse_quadlet(&raw[..4]);
                quads[1].parse_quadlet(&raw[4..]);
                targets[..4].iter_mut()
                    .zip(quads[0].to_ne_bytes().iter())
                    .for_each(|(target, &val)| *target = val as usize);
                targets[4..].iter_mut()
                    .zip(quads[1].to_ne_bytes().iter())
                    .for_each(|(target, &val)| *target = val as usize);
            })
    }

    fn write_meter_display_targets(&self, node: &T, sections: &ExtensionSections, targets: &[usize;8],
                                   timeout_ms: u32)
        -> Result<(), Error>
    {
        let mut quads = [0u32;2];
        targets[..4].iter()
            .enumerate()
            .for_each(|(i, &target)| quads[0] |= (target as u32) << (i * 8));
        targets[4..].iter()
            .enumerate()
            .for_each(|(i, &target)| quads[1] |= (target as u32) << (i * 8));
        let mut raw = [0;8];
        quads.build_quadlet_block(&mut raw);
        self.write_appl_data(node,  sections, Self::METER_DISPLAY_TARGET_OFFSET, &mut raw, timeout_ms)
    }
}

impl<O: ApplSectionProtocol<T>, T: AsRef<FwNode>> LiquidS56Protocol<T> for O {}
