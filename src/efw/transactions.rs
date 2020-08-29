// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::{Error, FileError};

use hinawa::SndEfwExtManual;

const TIMEOUT: u32 = 200;

enum Category {
    Info,
    HwCtl,
    PhysOutput,
    PhysInput,
    Playback,
    Monitor,
    PortConf,
    Guitar,
}

impl From<Category> for u32 {
    fn from(cat: Category) -> Self {
        match cat {
            Category::Info => 0x00,
            Category::HwCtl => 0x03,
            Category::PhysOutput => 0x04,
            Category::PhysInput => 0x05,
            Category::Playback => 0x06,
            Category::Monitor => 0x08,
            Category::PortConf => 0x09,
            Category::Guitar=> 0x0a,
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

    const O400F: u32 = 0x0000400f;
    const O1200F: u32 = 0x0001200f;
    const AF2: u32 = 0x00000af2;
    const AF4: u32 = 0x00000af4;
    const AF8: u32 = 0x00000af8;
    const AFP8: u32 = 0x00000af9;
    const AF12: u32 = 0x0000af12;

    fn new(data: &[u32;Self::SIZE]) -> Result<Self, Error> {
        let info = HwInfo {
            caps: Self::parse_caps(data[0], data[3]),
            guid: ((data[1] as u64) << 32) | (data[2] as u64),
            hw_type: data[3],
            hw_version: data[4],
            vendor_name: Self::parse_text(&data[5..13])?,
            model_name: Self::parse_text(&data[13..21])?,
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

    fn parse_caps(flags: u32, hw_type: u32) -> Vec<HwCap> {
        let mut caps = Vec::new();
        (0..16).for_each(|i| {
            if (1 << i) & flags > 0 {
                caps.push(HwCap::from(i))
            }
        });
        match hw_type {
            Self::O400F => caps.push(HwCap::SpdifCoax),
            Self::O1200F => caps.push(HwCap::InputMapping),
            Self::AF2 |
            Self::AF4 => {
                caps.push(HwCap::NominalInput);
                caps.push(HwCap::NominalOutput);
                caps.push(HwCap::SpdifCoax);
            }
            Self::AF8 => {
                caps.push(HwCap::NominalInput);
                caps.push(HwCap::NominalOutput);
                caps.push(HwCap::SpdifCoax);
            }
            Self::AFP8 => {
                caps.push(HwCap::NominalInput);
                caps.push(HwCap::NominalOutput);
                // It has flags for Coaxial/Optical interface for S/PDIF signal.
            }
            Self::AF12 => {
                caps.push(HwCap::NominalInput);
                caps.push(HwCap::NominalOutput);
            }
            _ => (),
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

#[derive(Debug)]
pub struct HwMeter {
    pub detected_clk_srcs: Vec<(ClkSrc, bool)>,
    pub detected_midi_inputs: [bool; 2],
    pub detected_midi_outputs: [bool; 2],
    pub guitar_charging: bool,
    pub guitar_stereo_connect: bool,
    pub guitar_hex_signal: bool,
    pub phys_output_meters: Vec<i32>,
    pub phys_input_meters: Vec<i32>,
}

impl HwMeter {
    const METER_SIZE: usize = 110;

    pub fn new(clk_srcs: &[ClkSrc], phys_inputs: usize, phys_outputs: usize) -> Self {
        HwMeter {
            detected_clk_srcs: clk_srcs.iter().map(|&src| (src, false)).collect(),
            detected_midi_inputs: [false; 2],
            detected_midi_outputs: [false; 2],
            guitar_charging: false,
            guitar_stereo_connect: false,
            guitar_hex_signal: false,
            phys_output_meters: vec![0; phys_outputs],
            phys_input_meters: vec![0; phys_inputs],
        }
    }

    pub fn parse(&mut self, data: &[u32;Self::METER_SIZE]) -> Result<(), Error> {
        let flags = data[0];
        self.detected_clk_srcs.iter_mut()
            .for_each(|(src, detected)| *detected = (1 << usize::from(*src)) & flags > 0);
        // I note that data[1..4] is reserved space and it stores previous set command for FPGA device.
        self.detected_midi_inputs.iter_mut().enumerate()
            .for_each(|(i, detected)| *detected = (1 << (8 + i)) & flags > 0);
        self.detected_midi_outputs.iter_mut().enumerate()
            .for_each(|(i, detected)| *detected = (1 << (8 + i)) & flags > 0);
        self.guitar_charging = (1 << 29) & flags > 0;
        self.guitar_stereo_connect = (1 << 30) & flags > 0;
        self.guitar_hex_signal = (1 << 31) & flags > 0;

        let phys_outputs = data[5] as usize;
        let phys_inputs = data[6] as usize;
        self.phys_output_meters.iter_mut().take(phys_outputs).enumerate()
            .for_each(|(i, val)| *val = Self::calc(data[9 + i]));
        self.phys_input_meters.iter_mut().take(phys_inputs).enumerate()
            .for_each(|(i, val)| *val = Self::calc(data[9 + i + phys_outputs]));
        Ok(())
    }

    fn calc(val: u32) -> i32 {
        (val >> 8) as i32
    }
}

pub struct EfwInfo {}

impl EfwInfo {
    const CMD_HWINFO: u32 = 0;
    const CMD_METER: u32 = 1;

    pub fn get_hwinfo(unit: &hinawa::SndEfw) -> Result<HwInfo, Error> {
        let mut data = [0; HwInfo::SIZE];
        let _ = unit.transaction_sync(u32::from(Category::Info), Self::CMD_HWINFO,
                                      None, Some(&mut data), TIMEOUT)?;
        HwInfo::new(&data)
    }

    pub fn get_meter(unit: &hinawa::SndEfw, meters: &mut HwMeter) -> Result<(), Error> {
        let mut params = [0; HwMeter::METER_SIZE];
        let _ = unit.transaction_sync(u32::from(Category::Info), Self::CMD_METER,
                                      None, Some(&mut params), TIMEOUT)?;
        meters.parse(&params)
    }
}

pub struct EfwHwCtl {}

impl EfwHwCtl {
    const CMD_SET_CLOCK: u32 = 0;
    const CMD_GET_CLOCK: u32 = 1;

    pub fn set_clock(
        unit: &hinawa::SndEfw,
        src: Option<ClkSrc>,
        rate: Option<u32>,
    ) -> Result<(), Error> {
        let mut args = [0, 0, 0];
        let mut params = [0, 0, 0];
        let (current_src, current_rate) = Self::get_clock(unit)?;
        args[0] = usize::from(match src {
            Some(s) => s,
            None => current_src,
        }) as u32;
        args[1] = match rate {
            Some(r) => r,
            None => current_rate,
        };
        let _ = unit.transaction_sync(
            u32::from(Category::HwCtl),
            Self::CMD_SET_CLOCK,
            Some(&args),
            Some(&mut params),
            TIMEOUT,
        );
        Ok(())
    }

    pub fn get_clock(unit: &hinawa::SndEfw) -> Result<(ClkSrc, u32), Error> {
        let mut params = [0, 0, 0];
        let _ = unit.transaction_sync(
            u32::from(Category::HwCtl),
            Self::CMD_GET_CLOCK,
            None,
            Some(&mut params),
            TIMEOUT,
        )?;
        Ok((ClkSrc::from(params[0] as usize), params[1]))
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum NominalLevel {
    PlusFour,
    Medium,
    MinusTen,
}

impl From<NominalLevel> for u32 {
    fn from(level: NominalLevel) -> Self {
        match level {
            NominalLevel::MinusTen => 2,
            NominalLevel::Medium => 1,
            NominalLevel::PlusFour => 0,
        }
    }
}

impl From<u32> for NominalLevel {
    fn from(val: u32) -> Self {
        match val {
            2 => NominalLevel::MinusTen,
            1 => NominalLevel::Medium,
            _ => NominalLevel::PlusFour,
        }
    }
}

pub struct EfwPhysOutput {}

impl EfwPhysOutput {
    const CMD_SET_VOL: u32 = 0;
    const CMD_GET_VOL: u32 = 1;
    const CMD_SET_MUTE: u32 = 2;
    const CMD_GET_MUTE: u32 = 3;
    const CMD_SET_NOMINAL: u32 = 8;
    const CMD_GET_NOMINAL: u32 = 9;

    pub fn set_vol(unit: &hinawa::SndEfw, ch: usize, vol: i32) -> Result<(), Error> {
        let args = [ch as u32, vol as u32];
        let mut params = [0; 2];
        let _ = unit.transaction_sync(
            u32::from(Category::PhysOutput),
            Self::CMD_SET_VOL,
            Some(&args),
            Some(&mut params),
            TIMEOUT,
        )?;
        Ok(())
    }

    pub fn get_vol(unit: &hinawa::SndEfw, ch: usize) -> Result<i32, Error> {
        let args = [ch as u32, 0];
        let mut params = [0; 2];
        let _ = unit.transaction_sync(
            u32::from(Category::PhysOutput),
            Self::CMD_GET_VOL,
            Some(&args),
            Some(&mut params),
            TIMEOUT,
        )?;
        Ok(params[1] as i32)
    }

    pub fn set_mute(unit: &hinawa::SndEfw, ch: usize, mute: bool) -> Result<(), Error> {
        let args = [ch as u32, mute as u32];
        let mut params = [0; 2];
        let _ = unit.transaction_sync(
            u32::from(Category::PhysOutput),
            Self::CMD_SET_MUTE,
            Some(&args),
            Some(&mut params),
            TIMEOUT,
        )?;
        Ok(())
    }

    pub fn get_mute(unit: &hinawa::SndEfw, ch: usize) -> Result<bool, Error> {
        let args = [ch as u32, 0];
        let mut params = [0; 2];
        let _ = unit.transaction_sync(
            u32::from(Category::PhysOutput),
            Self::CMD_GET_MUTE,
            Some(&args),
            Some(&mut params),
            TIMEOUT,
        )?;
        Ok(params[1] > 0)
    }

    pub fn set_nominal(unit: &hinawa::SndEfw, ch: usize, level: NominalLevel) -> Result<(), Error> {
        let args = [ch as u32, u32::from(level)];
        let mut params = [0; 2];
        let _ = unit.transaction_sync(
            u32::from(Category::PhysOutput),
            Self::CMD_SET_NOMINAL,
            Some(&args),
            Some(&mut params),
            TIMEOUT,
        )?;
        Ok(())
    }

    pub fn get_nominal(unit: &hinawa::SndEfw, ch: usize) -> Result<NominalLevel, Error> {
        let args = [ch as u32, 0];
        let mut params = [0; 2];
        let _ = unit.transaction_sync(
            u32::from(Category::PhysOutput),
            Self::CMD_GET_NOMINAL,
            Some(&args),
            Some(&mut params),
            TIMEOUT,
        )?;
        Ok(NominalLevel::from(params[1]))
    }
}

pub struct EfwPhysInput {}

impl EfwPhysInput {
    const CMD_SET_NOMINAL: u32 = 8;
    const CMD_GET_NOMINAL: u32 = 9;

    pub fn set_nominal(unit: &hinawa::SndEfw, ch: usize, level: NominalLevel) -> Result<(), Error> {
        let args = [ch as u32, u32::from(level)];
        let mut params = [0; 2];
        let _ = unit.transaction_sync(
            u32::from(Category::PhysInput),
            Self::CMD_SET_NOMINAL,
            Some(&args),
            Some(&mut params),
            TIMEOUT,
        )?;
        Ok(())
    }

    pub fn get_nominal(unit: &hinawa::SndEfw, ch: usize) -> Result<NominalLevel, Error> {
        let args = [ch as u32, 0];
        let mut params = [0; 2];
        let _ = unit.transaction_sync(
            u32::from(Category::PhysInput),
            Self::CMD_GET_NOMINAL,
            Some(&args),
            Some(&mut params),
            TIMEOUT,
        )?;
        Ok(NominalLevel::from(params[1]))
    }
}

pub struct EfwPlayback {}

impl EfwPlayback {
    const CMD_SET_VOL: u32 = 0;
    const CMD_GET_VOL: u32 = 1;
    const CMD_SET_MUTE: u32 = 2;
    const CMD_GET_MUTE: u32 = 3;
    const CMD_SET_SOLO: u32 = 4;
    const CMD_GET_SOLO: u32 = 5;

    pub fn set_vol(unit: &hinawa::SndEfw, ch: usize, vol: i32) -> Result<(), Error> {
        let args = [ch as u32, vol as u32];
        let mut params = [0; 2];
        let _ = unit.transaction_sync(
            u32::from(Category::Playback),
            Self::CMD_SET_VOL,
            Some(&args),
            Some(&mut params),
            TIMEOUT,
        )?;
        Ok(())
    }

    pub fn get_vol(unit: &hinawa::SndEfw, ch: usize) -> Result<i32, Error> {
        let args = [ch as u32, 0];
        let mut params = [0; 2];
        let _ = unit.transaction_sync(
            u32::from(Category::Playback),
            Self::CMD_GET_VOL,
            Some(&args),
            Some(&mut params),
            TIMEOUT,
        )?;
        Ok(params[1] as i32)
    }

    pub fn set_mute(unit: &hinawa::SndEfw, ch: usize, mute: bool) -> Result<(), Error> {
        let args = [ch as u32, mute as u32];
        let mut params = [0; 2];
        let _ = unit.transaction_sync(
            u32::from(Category::Playback),
            Self::CMD_SET_MUTE,
            Some(&args),
            Some(&mut params),
            TIMEOUT,
        )?;
        Ok(())
    }

    pub fn get_mute(unit: &hinawa::SndEfw, ch: usize) -> Result<bool, Error> {
        let args = [ch as u32, 0];
        let mut params = [0; 2];
        let _ = unit.transaction_sync(
            u32::from(Category::Playback),
            Self::CMD_GET_MUTE,
            Some(&args),
            Some(&mut params),
            TIMEOUT,
        )?;
        Ok(params[1] > 0)
    }

    pub fn set_solo(unit: &hinawa::SndEfw, ch: usize, solo: bool) -> Result<(), Error> {
        let args = [ch as u32, solo as u32];
        let mut params = [0; 2];
        let _ = unit.transaction_sync(
            u32::from(Category::Playback),
            Self::CMD_SET_SOLO,
            Some(&args),
            Some(&mut params),
            TIMEOUT,
        )?;
        Ok(())
    }

    pub fn get_solo(unit: &hinawa::SndEfw, ch: usize) -> Result<bool, Error> {
        let args = [ch as u32, 0];
        let mut params = [0; 2];
        let _ = unit.transaction_sync(
            u32::from(Category::Playback),
            Self::CMD_GET_SOLO,
            Some(&args),
            Some(&mut params),
            TIMEOUT,
        )?;
        Ok(params[1] > 0)
    }
}

pub struct EfwMonitor {}

impl EfwMonitor {
    const CMD_SET_VOL: u32 = 0;
    const CMD_GET_VOL: u32 = 1;
    const CMD_SET_MUTE: u32 = 2;
    const CMD_GET_MUTE: u32 = 3;
    const CMD_SET_SOLO: u32 = 4;
    const CMD_GET_SOLO: u32 = 5;
    const CMD_SET_PAN: u32 = 6;
    const CMD_GET_PAN: u32 = 7;

    pub fn set_vol(unit: &hinawa::SndEfw, dst: usize, src: usize, vol: i32) -> Result<(), Error> {
        let args = [src as u32, dst as u32, vol as u32];
        let mut params = [0; 3];
        let _ = unit.transaction_sync(
            u32::from(Category::Monitor),
            Self::CMD_SET_VOL,
            Some(&args),
            Some(&mut params),
            TIMEOUT,
        )?;
        Ok(())
    }

    pub fn get_vol(unit: &hinawa::SndEfw, dst: usize, src: usize) -> Result<i32, Error> {
        let args = [src as u32, dst as u32, 0];
        let mut params = [0; 3];
        let _ = unit.transaction_sync(
            u32::from(Category::Monitor),
            Self::CMD_GET_VOL,
            Some(&args),
            Some(&mut params),
            TIMEOUT,
        )?;
        Ok(params[2] as i32)
    }

    pub fn set_mute(
        unit: &hinawa::SndEfw,
        dst: usize,
        src: usize,
        mute: bool,
    ) -> Result<(), Error> {
        let args = [src as u32, dst as u32, mute as u32];
        let mut params = [0; 3];
        let _ = unit.transaction_sync(
            u32::from(Category::Monitor),
            Self::CMD_SET_MUTE,
            Some(&args),
            Some(&mut params),
            TIMEOUT,
        )?;
        Ok(())
    }

    pub fn get_mute(unit: &hinawa::SndEfw, dst: usize, src: usize) -> Result<bool, Error> {
        let args = [src as u32, dst as u32, 0];
        let mut params = [0; 3];
        let _ = unit.transaction_sync(
            u32::from(Category::Monitor),
            Self::CMD_GET_MUTE,
            Some(&args),
            Some(&mut params),
            TIMEOUT,
        )?;
        Ok(params[2] > 0)
    }

    pub fn set_solo(
        unit: &hinawa::SndEfw,
        dst: usize,
        src: usize,
        solo: bool,
    ) -> Result<(), Error> {
        let args = [src as u32, dst as u32, solo as u32];
        let mut params = [0; 3];
        let _ = unit.transaction_sync(
            u32::from(Category::Monitor),
            Self::CMD_SET_SOLO,
            Some(&args),
            Some(&mut params),
            TIMEOUT,
        )?;
        Ok(())
    }

    pub fn get_solo(unit: &hinawa::SndEfw, dst: usize, src: usize) -> Result<bool, Error> {
        let args = [src as u32, dst as u32, 0];
        let mut params = [0; 3];
        let _ = unit.transaction_sync(
            u32::from(Category::Monitor),
            Self::CMD_GET_SOLO,
            Some(&args),
            Some(&mut params),
            TIMEOUT,
        )?;
        Ok(params[2] > 0)
    }

    pub fn set_pan(unit: &hinawa::SndEfw, dst: usize, src: usize, pan: u8) -> Result<(), Error> {
        let args = [src as u32, dst as u32, pan as u32];
        let mut params = [0; 3];
        let _ = unit.transaction_sync(
            u32::from(Category::Monitor),
            Self::CMD_SET_PAN,
            Some(&args),
            Some(&mut params),
            TIMEOUT,
        )?;
        Ok(())
    }

    pub fn get_pan(unit: &hinawa::SndEfw, dst: usize, src: usize) -> Result<u8, Error> {
        let args = [src as u32, dst as u32, 0];
        let mut params = [0; 3];
        let _ = unit.transaction_sync(
            u32::from(Category::Monitor),
            Self::CMD_GET_PAN,
            Some(&args),
            Some(&mut params),
            TIMEOUT,
        )?;
        Ok(params[2] as u8)
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum DigitalMode {
    SpdifCoax,
    AesebuXlr,
    SpdifOpt,
    AdatOpt,
    Unknown(u32),
}

impl From<u32> for DigitalMode {
    fn from(val: u32) -> Self {
        match val {
            0 => DigitalMode::SpdifCoax,
            1 => DigitalMode::AesebuXlr,
            2 => DigitalMode::SpdifOpt,
            3 => DigitalMode::AdatOpt,
            _ => DigitalMode::Unknown(val),
        }
    }
}

impl From<DigitalMode> for u32 {
    fn from(mode: DigitalMode) -> Self {
        match mode {
            DigitalMode::SpdifCoax => 0,
            DigitalMode::AesebuXlr => 1,
            DigitalMode::SpdifOpt => 2,
            DigitalMode::AdatOpt => 3,
            DigitalMode::Unknown(val) => val,
        }
    }
}

pub struct EfwPortConf {}

impl EfwPortConf {
    const CMD_SET_MIRROR: u32 = 0;
    const CMD_GET_MIRROR: u32 = 1;
    const CMD_SET_DIG_MODE: u32 = 2;
    const CMD_GET_DIG_MODE: u32 = 3;
    const CMD_SET_PHANTOM: u32 = 4;
    const CMD_GET_PHANTOM: u32 = 5;
    const CMD_SET_STREAM_MAP: u32 = 6;
    const CMD_GET_STREAM_MAP: u32 = 7;

    const MAP_SIZE: usize = 70;

    pub fn set_output_mirror(unit: &hinawa::SndEfw, pair: usize) -> Result<(), Error> {
        let _ = unit.transaction_sync(
            u32::from(Category::PortConf),
            Self::CMD_SET_MIRROR,
            Some(&[pair as u32]),
            None,
            TIMEOUT,
        )?;
        Ok(())
    }

    pub fn get_output_mirror(unit: &hinawa::SndEfw) -> Result<usize, Error> {
        let mut params = [0];
        let _ = unit.transaction_sync(
            u32::from(Category::PortConf),
            Self::CMD_GET_MIRROR,
            None,
            Some(&mut params),
            TIMEOUT,
        )?;
        Ok(params[0] as usize)
    }

    pub fn set_digital_mode(unit: &hinawa::SndEfw, mode: DigitalMode) -> Result<(), Error> {
        let args = [u32::from(mode)];
        let _ = unit.transaction_sync(
            u32::from(Category::PortConf),
            Self::CMD_SET_DIG_MODE,
            Some(&args),
            None,
            TIMEOUT,
        )?;
        Ok(())
    }

    pub fn get_digital_mode(unit: &hinawa::SndEfw) -> Result<DigitalMode, Error> {
        let mut params = [0];
        let _ = unit.transaction_sync(
            u32::from(Category::PortConf),
            Self::CMD_GET_DIG_MODE,
            None,
            Some(&mut params),
            TIMEOUT,
        )?;
        Ok(DigitalMode::from(params[0]))
    }

    pub fn set_phantom_powering(unit: &hinawa::SndEfw, state: bool) -> Result<(), Error> {
        let _ = unit.transaction_sync(
            u32::from(Category::PortConf),
            Self::CMD_SET_PHANTOM,
            Some(&[state as u32]),
            None,
            TIMEOUT,
        )?;
        Ok(())
    }

    pub fn get_phantom_powering(unit: &hinawa::SndEfw) -> Result<bool, Error> {
        let mut params = [0];
        let _ = unit.transaction_sync(
            u32::from(Category::PortConf),
            Self::CMD_GET_PHANTOM,
            None,
            Some(&mut params),
            TIMEOUT,
        )?;
        Ok(params[0] > 0)
    }

    pub fn set_stream_map(
        unit: &hinawa::SndEfw,
        rx_map: Option<Vec<usize>>,
        tx_map: Option<Vec<usize>>,
    ) -> Result<(), Error> {
        let mut args = [0; Self::MAP_SIZE];
        let _ = unit.transaction_sync(
            u32::from(Category::PortConf),
            Self::CMD_GET_STREAM_MAP,
            None,
            Some(&mut args),
            TIMEOUT,
        )?;
        if let Some(entries) = rx_map {
            args[2] = entries.len() as u32;
            entries
                .iter()
                .enumerate()
                .for_each(|(pos, entry)| args[4 + pos] = 2 * *entry as u32);
        }
        if let Some(entries) = tx_map {
            args[36] = entries.len() as u32;
            entries
                .iter()
                .enumerate()
                .for_each(|(pos, entry)| args[38 + pos] = 2 * *entry as u32);
        }
        let _ = unit.transaction_sync(
            u32::from(Category::PortConf),
            Self::CMD_SET_STREAM_MAP,
            Some(&args),
            None,
            TIMEOUT,
        )?;
        Ok(())
    }

    pub fn get_stream_map(unit: &hinawa::SndEfw) -> Result<(Vec<usize>, Vec<usize>), Error> {
        let mut params = [0; Self::MAP_SIZE];
        let _ = unit.transaction_sync(
            u32::from(Category::PortConf),
            Self::CMD_GET_STREAM_MAP,
            None,
            Some(&mut params),
            TIMEOUT,
        )?;
        let rx_entry_count = params[2] as usize;
        let rx_entries: Vec<usize> = (0..rx_entry_count)
            .map(|pos| (params[4 + pos] / 2) as usize)
            .collect();
        let tx_entry_count = params[36] as usize;
        let tx_entries: Vec<usize> = (0..tx_entry_count)
            .map(|pos| (params[38 + pos] / 2) as usize)
            .collect();
        Ok((rx_entries, tx_entries))
    }
}

#[derive(Debug)]
pub struct GuitarChargeState {
    pub manual_charge: bool,
    pub auto_charge: bool,
    pub suspend_to_charge: u32,
}

pub struct EfwGuitar {}

impl EfwGuitar {
    const CMD_SET_CHARGE_STATE: u32 = 7;
    const CMD_GET_CHARGE_STATE: u32 = 8;

    pub fn get_charge_state(unit: &hinawa::SndEfw) -> Result<GuitarChargeState, Error> {
        let mut params = [0;3];
        let _ = unit.transaction_sync(
            u32::from(Category::Guitar),
            Self::CMD_GET_CHARGE_STATE,
            None,
            Some(&mut params),
            TIMEOUT,
        )?;
        let state = GuitarChargeState{
            manual_charge: params[0] > 0,
            auto_charge: params[1] > 0,
            suspend_to_charge: params[2],
        };
        Ok(state)
    }

    pub fn set_charge_state(unit: &hinawa::SndEfw, state: &GuitarChargeState) -> Result<(), Error> {
        let args = [
            state.manual_charge as u32,
            state.auto_charge as u32,
            state.suspend_to_charge,
        ];
        let mut params = [0;3];
        let _ = unit.transaction_sync(
            u32::from(Category::Guitar),
            Self::CMD_SET_CHARGE_STATE,
            Some(&args),
            Some(&mut params),
            TIMEOUT,
        )?;
        Ok(())
    }
}
