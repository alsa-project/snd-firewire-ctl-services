// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::{Error, FileError};

use hinawa::SndEfwExtManual;

enum Category {
    Info,
}

impl From<Category> for u32 {
    fn from(cat: Category) -> Self {
        match cat {
            Category::Info => 0x00,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum HwCap {
    ChangeableRespAddr,
    MirrorOutput,
    SpdifCoax,
    AesebuXlr,
    Dsp,
    Fpga,
    PhantomPowering,
    OutputMapping,
    InputGain,
    SpdifOpt,
    AdatOpt,
    NominalInput,
    NominalOutput,
    SoftClip,
    RobotGuitar,
    GuitarCharging,
    Unknown(usize),
    // For my purpose.
    InputMapping,
}

impl From<usize> for HwCap {
    fn from(val: usize) -> Self {
        match val {
            0 => HwCap::ChangeableRespAddr,
            1 => HwCap::MirrorOutput,
            2 => HwCap::SpdifCoax,
            3 => HwCap::AesebuXlr,
            4 => HwCap::Dsp,
            5 => HwCap::Fpga,
            6 => HwCap::PhantomPowering,
            7 => HwCap::OutputMapping,
            8 => HwCap::InputGain,
            9 => HwCap::SpdifOpt,
            10 => HwCap::AdatOpt,
            11 => HwCap::NominalInput,
            12 => HwCap::NominalOutput,
            13 => HwCap::SoftClip,
            14 => HwCap::RobotGuitar,
            15 => HwCap::GuitarCharging,
            _ => HwCap::Unknown(val),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ClkSrc {
    Internal,
    WordClock,
    Spdif,
    Adat,
    Adat2,
    Continuous,
    Unknown(usize),
}

impl From<usize> for ClkSrc {
    fn from(val: usize) -> Self {
        match val {
            0 => ClkSrc::Internal,
            2 => ClkSrc::WordClock,
            3 => ClkSrc::Spdif,
            4 => ClkSrc::Adat,
            5 => ClkSrc::Adat2,
            6 => ClkSrc::Continuous,
            _ => ClkSrc::Unknown(val),
        }
    }
}

impl From<ClkSrc> for usize {
    fn from(src: ClkSrc) -> Self {
        match src {
            ClkSrc::Internal => 0,
            ClkSrc::WordClock => 2,
            ClkSrc::Spdif => 3,
            ClkSrc::Adat => 4,
            ClkSrc::Adat2 => 5,
            ClkSrc::Continuous => 6,
            ClkSrc::Unknown(val) => val,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum PhysGroupType {
    Analog,
    Spdif,
    Adat,
    SpdifOrAdat,
    AnalogMirror,
    Headphones,
    I2s,
    Guitar,
    PiezoGuitar,
    GuitarString,
    Unknown(usize),
}

impl From<usize> for PhysGroupType {
    fn from(val: usize) -> Self {
        match val {
            0 => PhysGroupType::Analog,
            1 => PhysGroupType::Spdif,
            2 => PhysGroupType::Adat,
            3 => PhysGroupType::SpdifOrAdat,
            4 => PhysGroupType::AnalogMirror,
            5 => PhysGroupType::Headphones,
            6 => PhysGroupType::I2s,
            7 => PhysGroupType::Guitar,
            8 => PhysGroupType::PiezoGuitar,
            9 => PhysGroupType::GuitarString,
            _ => PhysGroupType::Unknown(val),
        }
    }
}

#[derive(Debug)]
pub struct PhysGroupEntry {
    pub group_type: PhysGroupType,
    pub group_count: usize,
}

#[derive(Debug)]
pub struct HwInfo {
    pub caps: Vec<HwCap>,
    pub guid: u64,
    pub hw_type: u32,
    pub hw_version: u32,
    pub vendor_name: String,
    pub model_name: String,
    pub clk_srcs: Vec<ClkSrc>,
    pub rx_channels: [usize; 3],
    pub tx_channels: [usize; 3],
    pub phys_outputs: Vec<PhysGroupEntry>,
    pub phys_inputs: Vec<PhysGroupEntry>,
    pub midi_outputs: usize,
    pub midi_inputs: usize,
    pub clk_rates: Vec<u32>,
    pub dsp_version: u32,
    pub arm_version: u32,
    pub mixer_playbacks: usize,
    pub mixer_captures: usize,
    pub fpga_version: u32,
}

impl HwInfo {
    const SIZE: usize = 65;

    fn new(data: &[u32;Self::SIZE]) -> Result<Self, Error> {
        let model_name = Self::parse_text(&data[13..21])?;
        let info = HwInfo {
            caps: Self::parse_caps(data[0], &model_name),
            guid: ((data[1] as u64) << 32) | (data[2] as u64),
            hw_type: data[3],
            hw_version: data[4],
            vendor_name: Self::parse_text(&data[5..13])?,
            model_name: model_name,
            clk_srcs: Self::parse_supported_clk_srcs(data[21]),
            rx_channels: [
                data[22] as usize,
                data[45] as usize,
                data[47] as usize,
            ],
            tx_channels: [
                data[23] as usize,
                data[46] as usize,
                data[48] as usize,
            ],
            phys_outputs: Self::parse_phys_groups(&data[26..31]),
            phys_inputs: Self::parse_phys_groups(&data[31..36]),
            midi_outputs: data[36] as usize,
            midi_inputs: data[37] as usize,
            clk_rates: Self::parse_supported_clk_rates(data[38], data[39]),
            dsp_version: data[40],
            arm_version: data[41],
            mixer_playbacks: data[42] as usize,
            mixer_captures: data[43] as usize,
            fpga_version: data[44],
        };
        Ok(info)
    }

    fn parse_caps(flags: u32, model_name: &str) -> Vec<HwCap> {
        let mut caps = Vec::new();
        (0..16).for_each(|i| {
            if (1 << i) & flags > 0 {
                caps.push(HwCap::from(i))
            }
        });
        if model_name == "Onyx 1200F" {
            caps.push(HwCap::InputMapping);
        }
        caps
    }

    fn parse_text(quads: &[u32]) -> Result<String, Error> {
        let mut literal = Vec::new();
        quads.iter().for_each(|quad| {
            literal.extend_from_slice(&quad.to_be_bytes());
        });
        if let Ok(text) = std::str::from_utf8(&literal) {
            if let Some(pos) = text.find('\0') {
                return Ok(text[0..pos].to_string());
            }
        }
        Err(Error::new(FileError::Io, "Fail to parse string."))
    }

    fn parse_supported_clk_srcs(flags: u32) -> Vec<ClkSrc> {
        let mut srcs = Vec::new();
        (0..6).for_each(|i| {
            if (1 << i) & flags > 0 {
                srcs.push(ClkSrc::from(i));
            }
        });
        srcs
    }

    fn parse_supported_clk_rates(max: u32, min: u32) -> Vec<u32> {
        let mut rates = Vec::new();
        [32000, 44100, 48000, 88200, 96000, 176400, 192000].iter().for_each(|rate| {
            if *rate >= min && *rate <= max {
                rates.push(*rate);
            }
        });
        rates
    }

    fn parse_phys_groups(quads: &[u32]) -> Vec<PhysGroupEntry> {
        let mut groups = Vec::new();
        let mut bytes = Vec::new();
        let count = quads[0] as usize;
        quads[1..].iter().for_each(|quad| {
            bytes.extend_from_slice(&quad.to_be_bytes());
        });
        (0..count).for_each(|i| {
            let group_type = PhysGroupType::from(bytes[i * 2] as usize);
            let group_count = bytes[i * 2 + 1] as usize;
            groups.push(PhysGroupEntry {
                group_type,
                group_count,
            });
        });
        groups
    }
}

pub struct EfwInfo {}

impl EfwInfo {
    const CMD_HWINFO: u32 = 0;

    pub fn get_hwinfo(unit: &hinawa::SndEfw) -> Result<HwInfo, Error> {
        let mut data = [0; HwInfo::SIZE];
        let _ = unit.transaction(u32::from(Category::Info), Self::CMD_HWINFO,
                                 &[], &mut data)?;
        HwInfo::new(&data)
    }
}
