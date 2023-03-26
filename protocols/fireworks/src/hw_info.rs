// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol about hardware information.
//!
//! The module includes protocol about hardware information defined by Echo Audio Digital Corporation
//! for Fireworks board module.
//!
//! ## Table for capabilities retrieved from each Fireworks model
//!
//! capabilities       | 1200f | 400f | af12(old) | af12(new) | af8(old) | af9 | af4 | af2 | rip |
//! ------------------ | ----- | ---- | --------- | --------- | -------- | --- | --- | --- | --- |
//! ChangeableRespAddr |   *   |   *  |     *     |     *     |    *     |  *  |  *  |  *  |  *  |
//! ControlRoom        |       |   *  |           |           |          |     |     |     |     |
//! OptionalSpdifCoax  |   *   |      |           |           |          |  *  |     |     |     |
//! OptionalAesebuXlr  |   *   |      |           |           |          |     |     |     |     |
//! Dsp                |   *   |   *  |     *     |     *     |    *     |     |     |     |     |
//! Fpga               |   *   |      |           |           |          |  *  |  *  |  *  |  *  |
//! PhantomPowering    |       |      |           |           |          |     |  *  |     |     |
//! OutputMapping      |       |      |           |           |          |     |  *  |  *  |     |
//! InputGain          |       |      |           |     *     |          |     |     |     |     |
//! OptionalSpdifOpt   |       |      |           |           |          |  *  |     |     |     |
//! OptionalAdatOpt    |       |      |           |           |          |  *  |     |     |     |
//! NominalInput       |       |      |           |           |          |     |     |     |     |
//! NominalOutput      |       |      |           |           |          |     |     |     |     |
//! SoftClip           |       |      |           |           |          |     |     |     |     |
//! RobotGuitar        |       |      |           |           |          |     |     |     |  *  |
//! GuitarCharging     |       |      |           |           |          |     |     |     |  *  |

use super::*;

const CATEGORY_HWINFO: u32 = 0;

const CMD_HWINFO: u32 = 0;
const CMD_METER: u32 = 1;
const CMD_CHANGE_RESP_ADDR: u32 = 2;
const CMD_READ_SESSION_BLOCK: u32 = 3;

/// Information of hardware.
#[derive(Debug, Clone, PartialEq, Eq)]
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

impl Default for HwInfo {
    fn default() -> Self {
        Self {
            caps: Vec::new(),
            guid: 0,
            hw_type: 0,
            hw_version: 0,
            vendor_name: String::new(),
            model_name: String::new(),
            clk_srcs: Vec::new(),
            rx_channels: [0; 3],
            tx_channels: [0; 3],
            phys_outputs: Vec::new(),
            phys_inputs: Vec::new(),
            midi_outputs: 0,
            midi_inputs: 0,
            clk_rates: Vec::new(),
            dsp_version: 0,
            arm_version: 0,
            mixer_playbacks: 0,
            mixer_captures: 0,
            fpga_version: 0,
        }
    }
}

// Known models.
#[allow(dead_code)]
const O400F: u32 = 0x0000400f;
const O1200F: u32 = 0x0001200f;
const AF2: u32 = 0x00000af2;
const AF4: u32 = 0x00000af4;
const AF8: u32 = 0x00000af8;
const AFP8: u32 = 0x00000af9;
const AF12: u32 = 0x0000af12;

impl HwInfo {
    fn parse(&mut self, quads: &[u32]) -> Result<(), Error> {
        self.caps = Self::parse_caps(quads[0], quads[3]);
        self.guid = ((quads[1] as u64) << 32) | (quads[2] as u64);
        self.hw_type = quads[3];
        self.hw_version = quads[4];
        self.vendor_name = Self::parse_text(&quads[5..13])?;
        self.model_name = Self::parse_text(&quads[13..21])?;
        self.clk_srcs = Self::parse_supported_clk_srcs(quads[21]);
        self.rx_channels = [quads[22] as usize, quads[45] as usize, quads[47] as usize];
        self.tx_channels = [quads[23] as usize, quads[46] as usize, quads[48] as usize];
        self.phys_outputs = Self::parse_phys_groups(&quads[26..31]);
        self.phys_inputs = Self::parse_phys_groups(&quads[31..36]);
        self.midi_outputs = quads[36] as usize;
        self.midi_inputs = quads[37] as usize;
        self.clk_rates = Self::parse_supported_clk_rates(quads[38], quads[39]);
        self.dsp_version = quads[40];
        self.arm_version = quads[41];
        self.mixer_playbacks = quads[42] as usize;
        self.mixer_captures = quads[43] as usize;
        self.fpga_version = quads[44];

        Ok(())
    }

    fn parse_caps(flags: u32, hw_type: u32) -> Vec<HwCap> {
        let mut caps: Vec<HwCap> = (0..16)
            .filter(|i| (1 << i) & flags > 0)
            .map(|i| {
                let mut cap = HwCap::default();
                deserialize_hw_cap(&mut cap, i);
                cap
            })
            .collect();

        match hw_type {
            AF2 | AF4 | AF8 | AFP8 | AF12 => {
                caps.push(HwCap::NominalInput);
                caps.push(HwCap::NominalOutput);
            }
            O400F => {
                caps.push(HwCap::PlaybackSoloUnsupported);
            }
            O1200F => {
                caps.push(HwCap::ControlRoom);
                caps.push(HwCap::PlaybackSoloUnsupported);
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
        (0..6)
            .filter(|&i| (1 << i) & flags > 0)
            .map(|i| {
                let mut src = ClkSrc::default();
                deserialize_clock_source(&mut src, i as u32);
                src
            })
            .collect()
    }

    fn parse_supported_clk_rates(max: u32, min: u32) -> Vec<u32> {
        [32000, 44100, 48000, 88200, 96000, 176400, 192000]
            .iter()
            .filter(|&r| *r >= min && *r <= max)
            .copied()
            .collect()
    }

    fn parse_phys_groups(quads: &[u32]) -> Vec<PhysGroupEntry> {
        let count = quads[0] as usize;

        let mut bytes = Vec::new();
        quads[1..].iter().for_each(|quad| {
            bytes.extend_from_slice(&quad.to_be_bytes());
        });

        (0..count)
            .map(|i| {
                let pos = i * 2;
                let mut entry = PhysGroupEntry::default();
                deserialize_phys_group_type(&mut entry.group_type, bytes[pos]);
                entry.group_count = bytes[pos + 1] as usize;
                entry
            })
            .collect()
    }
}

const HWINFO_QUADS: usize = 65;

impl<O, P> EfwWhollyCachableParamsOperation<P, HwInfo> for O
where
    O: EfwHardwareSpecification,
    P: EfwProtocolExtManual,
{
    fn cache_wholly(proto: &mut P, states: &mut HwInfo, timeout_ms: u32) -> Result<(), Error> {
        let mut params = vec![0; HWINFO_QUADS];
        proto
            .transaction(CATEGORY_HWINFO, CMD_HWINFO, &[], &mut params, timeout_ms)
            .and_then(|_| states.parse(&params))
    }
}

/// Hardware meter.
#[derive(Debug, Clone, PartialEq, Eq)]
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

impl Default for HwMeter {
    fn default() -> Self {
        Self {
            detected_clk_srcs: Vec::new(),
            detected_midi_inputs: [false; 2],
            detected_midi_outputs: [false; 2],
            guitar_charging: false,
            guitar_stereo_connect: false,
            guitar_hex_signal: false,
            phys_output_meters: Vec::new(),
            phys_input_meters: Vec::new(),
        }
    }
}

impl HwMeter {
    /// The constructor for structure expressing hardware meter.
    pub fn new(clk_srcs: &[ClkSrc], phys_inputs: usize, phys_outputs: usize) -> Self {
        let mut meter = Self::default();

        meter.detected_clk_srcs = clk_srcs.iter().map(|&src| (src, false)).collect();
        meter.phys_output_meters = vec![0; phys_outputs];
        meter.phys_input_meters = vec![0; phys_inputs];

        meter
    }

    fn parse(&mut self, quads: &[u32]) {
        let flags = quads[0];

        self.detected_clk_srcs
            .iter_mut()
            .for_each(|(src, detected)| {
                let pos = serialize_clock_source(src);
                *detected = (1 << pos) & flags > 0
            });

        // I note that quads[1..4] is reserved space and it stores previous set command for FPGA device.

        self.detected_midi_inputs
            .iter_mut()
            .enumerate()
            .for_each(|(i, detected)| *detected = (1 << (8 + i)) & flags > 0);
        self.detected_midi_outputs
            .iter_mut()
            .enumerate()
            .for_each(|(i, detected)| *detected = (1 << (8 + i)) & flags > 0);

        self.guitar_charging = (1 << 29) & flags > 0;
        self.guitar_stereo_connect = (1 << 30) & flags > 0;
        self.guitar_hex_signal = (1 << 31) & flags > 0;

        let phys_outputs = quads[5] as usize;
        let phys_inputs = quads[6] as usize;
        self.phys_output_meters
            .iter_mut()
            .take(phys_outputs)
            .enumerate()
            .for_each(|(i, val)| *val = (quads[9 + i] >> 8) as i32);
        self.phys_input_meters
            .iter_mut()
            .take(phys_inputs)
            .enumerate()
            .for_each(|(i, val)| *val = (quads[9 + i + phys_outputs] >> 8) as i32);
    }
}

const METER_QUADS: usize = 110;

impl<O, P> EfwWhollyCachableParamsOperation<P, HwMeter> for O
where
    O: EfwHardwareSpecification,
    P: EfwProtocolExtManual,
{
    fn cache_wholly(proto: &mut P, states: &mut HwMeter, timeout_ms: u32) -> Result<(), Error> {
        let mut params = vec![0; METER_QUADS];
        proto
            .transaction(CATEGORY_HWINFO, CMD_METER, &[], &mut params, timeout_ms)
            .map(|_| states.parse(&params))
    }
}

/// The parameter of response address.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct EfwRespAddr(u64);

impl<O, P> EfwWhollyUpdatableParamsOperation<P, EfwRespAddr> for O
where
    O: EfwHardwareSpecification,
    P: EfwProtocolExtManual,
{
    fn update_wholly(proto: &mut P, states: &EfwRespAddr, timeout_ms: u32) -> Result<(), Error> {
        let args = [(states.0 >> 32) as u32, (states.0 & 0xffffffff) as u32];
        let mut params = Vec::new();
        proto.transaction(
            CATEGORY_HWINFO,
            CMD_CHANGE_RESP_ADDR,
            &args,
            &mut params,
            timeout_ms,
        )
    }
}

/// Protocol about hardware information for Fireworks board module.
pub trait HwInfoProtocol: EfwProtocolExtManual {
    /// Read hardware information.
    fn get_hw_info(&mut self, info: &mut HwInfo, timeout_ms: u32) -> Result<(), Error> {
        let mut params = vec![0; HWINFO_QUADS];
        self.transaction(CATEGORY_HWINFO, CMD_HWINFO, &[], &mut params, timeout_ms)
            .and_then(|_| info.parse(&params))
    }

    /// Read hardware meters.
    fn get_hw_meter(&mut self, meters: &mut HwMeter, timeout_ms: u32) -> Result<(), Error> {
        let mut params = vec![0; METER_QUADS];
        self.transaction(CATEGORY_HWINFO, CMD_METER, &[], &mut params, timeout_ms)
            .map(|_| meters.parse(&params))
    }

    /// Register response address as long as HwCap::ChangeableRespAddr is supported.
    fn set_hw_resp_addr(&mut self, addr: u64, timeout_ms: u32) -> Result<(), Error> {
        let args = [(addr >> 32) as u32, (addr & 0xffffffff) as u32];
        self.transaction(
            CATEGORY_HWINFO,
            CMD_CHANGE_RESP_ADDR,
            &args,
            &mut Vec::new(),
            timeout_ms,
        )
    }

    /// Read data from session block.
    fn get_hw_session_block(
        &mut self,
        offset: u32,
        data: &mut [u32],
        timeout_ms: u32,
    ) -> Result<(), Error> {
        // The first argument should be quadlet count.
        let args = [offset / 4, data.len() as u32];
        let mut params = vec![0; 2 + data.len()];
        self.transaction(
            CATEGORY_HWINFO,
            CMD_READ_SESSION_BLOCK,
            &args,
            &mut params,
            timeout_ms,
        )
        .map(|_| data.copy_from_slice(&params[2..]))
    }
}

impl<O: EfwProtocolExtManual> HwInfoProtocol for O {}
