// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol defined by RME GmbH for latter models of Fireface series.

pub mod ff802;

use glib::Error;
use hinawa::{FwNode, FwTcode, FwReq, FwReqExtManual};

use super::*;

const CFG_OFFSET: usize = 0xffff00000014;
const DSP_OFFSET: usize = 0xffff0000001c;
const METER_OFFSET: usize = 0xffffff000000;

// For configuration register (0x'ffff'0000'0014).
const CFG_MIDI_TX_LOW_OFFSET_MASK: u32          = 0x0001e000;
const   CFG_MIDI_TX_LOW_OFFSET_0180_FLAG: u32   = 0x00010000;
const   CFG_MIDI_TX_LOW_OFFSET_0100_FLAG: u32   = 0x00008000;
const   CFG_MIDI_TX_LOW_OFFSET_0080_FLAG: u32   = 0x00004000;
const   CFG_MIDI_TX_LOW_OFFSET_0000_FLAG: u32   = 0x00002000;

/// The enumeration to represent low offset of destination address for MIDI messages.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum FfLatterMidiTxLowOffset {
    /// Between 0x0000 to 0x007c.
    A0000,
    /// Between 0x0080 to 0x00fc.
    A0080,
    /// Between 0x0100 to 0x017c.
    A0100,
    /// Between 0x0180 to 0x01fc.
    A0180,
}

impl Default for FfLatterMidiTxLowOffset {
    fn default() -> Self {
        Self::A0000
    }
}

impl FfLatterMidiTxLowOffset {
    fn build(&self, quad: &mut u32) {
        *quad |= match self {
            Self::A0000 => CFG_MIDI_TX_LOW_OFFSET_0000_FLAG,
            Self::A0080 => CFG_MIDI_TX_LOW_OFFSET_0080_FLAG,
            Self::A0100 => CFG_MIDI_TX_LOW_OFFSET_0100_FLAG,
            Self::A0180 => CFG_MIDI_TX_LOW_OFFSET_0180_FLAG,
        };
    }

    fn parse(&mut self, quad: &u32) {
        *self = match *quad & CFG_MIDI_TX_LOW_OFFSET_MASK {
            CFG_MIDI_TX_LOW_OFFSET_0180_FLAG => Self::A0180,
            CFG_MIDI_TX_LOW_OFFSET_0100_FLAG => Self::A0100,
            CFG_MIDI_TX_LOW_OFFSET_0080_FLAG => Self::A0080,
            CFG_MIDI_TX_LOW_OFFSET_0000_FLAG => Self::A0000,
            _ => unreachable!(),
        }
    }
}

/// The trait to represent operation for configuration structure.
pub trait RmeFfLatterRegisterValueOperation {
    fn build(&self, quad: &mut u32);
    fn parse(&mut self, quad: &u32);
}

/// The trait to represent configuration protocol.
pub trait RmeFfLatterConfigProtocol<T, U> : AsRef<FwReq>
    where T: AsRef<FwNode>,
          U: RmeFfLatterRegisterValueOperation,
{
    fn write_cfg(&self, node: &T, cfg: &U, timeout_ms: u32) -> Result<(), Error> {
        let mut quad = 0u32;
        cfg.build(&mut quad);

        let mut raw = [0;4];
        raw.copy_from_slice(&quad.to_le_bytes());
        self.as_ref().transaction_sync(node.as_ref(), FwTcode::WriteQuadletRequest, CFG_OFFSET as u64,
                                       raw.len(), &mut raw, timeout_ms)
    }
}

// For status register (0x'ffff'0000'001c).
const STATUS_CLK_RATE_32000: u32 = 0x00;
const STATUS_CLK_RATE_44100: u32 = 0x01;
const STATUS_CLK_RATE_48000: u32 = 0x02;
const STATUS_CLK_RATE_64000: u32 = 0x04;
const STATUS_CLK_RATE_88200: u32 = 0x05;
const STATUS_CLK_RATE_96000: u32 = 0x06;
const STATUS_CLK_RATE_128000: u32 = 0x08;
const STATUS_CLK_RATE_176400: u32 = 0x09;
const STATUS_CLK_RATE_192000: u32 = 0x0a;
const STATUS_CLK_RATE_NONE: u32 = 0x0f;

fn val_from_clk_rate(clk_rate: &ClkNominalRate, quad: &mut u32, shift: usize) {
    let val = match clk_rate {
        ClkNominalRate::R32000 => STATUS_CLK_RATE_32000,
        ClkNominalRate::R44100 => STATUS_CLK_RATE_44100,
        ClkNominalRate::R48000 => STATUS_CLK_RATE_48000,
        ClkNominalRate::R64000 => STATUS_CLK_RATE_64000,
        ClkNominalRate::R88200 => STATUS_CLK_RATE_88200,
        ClkNominalRate::R96000 => STATUS_CLK_RATE_96000,
        ClkNominalRate::R128000 => STATUS_CLK_RATE_128000,
        ClkNominalRate::R176400 => STATUS_CLK_RATE_176400,
        ClkNominalRate::R192000 => STATUS_CLK_RATE_192000,
    };
    *quad |= val << shift;
}

fn val_to_clk_rate(clk_rate: &mut ClkNominalRate, quad: &u32, shift: usize) {
        *clk_rate = match (*quad >> shift) & 0x0000000f {
            STATUS_CLK_RATE_32000 => ClkNominalRate::R32000,
            STATUS_CLK_RATE_44100 => ClkNominalRate::R44100,
            STATUS_CLK_RATE_48000 => ClkNominalRate::R48000,
            STATUS_CLK_RATE_64000 => ClkNominalRate::R64000,
            STATUS_CLK_RATE_88200 => ClkNominalRate::R88200,
            STATUS_CLK_RATE_96000 => ClkNominalRate::R96000,
            STATUS_CLK_RATE_128000 => ClkNominalRate::R128000,
            STATUS_CLK_RATE_176400 => ClkNominalRate::R176400,
            STATUS_CLK_RATE_192000 => ClkNominalRate::R192000,
            _ => unreachable!(),
        };
}

fn optional_val_from_clk_rate(clk_rate: &Option<ClkNominalRate>, quad: &mut u32, shift: usize) {
    if let Some(r) = clk_rate {
        val_from_clk_rate(r, quad, shift)
    } else {
        *quad |= STATUS_CLK_RATE_NONE << shift;
    }
}

fn optional_val_to_clk_rate(clk_rate: &mut Option<ClkNominalRate>, quad: &u32, shift: usize) {
    if (*quad >> shift) & 0x0000000f != STATUS_CLK_RATE_NONE {
        let mut r = ClkNominalRate::default();
        val_to_clk_rate(&mut r, quad, shift);
        *clk_rate = Some(r);
    } else {
        *clk_rate = None;
    };
}

/// The trait to represent status protocol.
pub trait RmeFfLatterStatusProtocol<T, U> : AsRef<FwReq>
    where T: AsRef<FwNode>,
          U: RmeFfLatterRegisterValueOperation,
{
    fn read_status(&self, node: &T, status: &mut U, timeout_ms: u32) -> Result<(), Error> {
        let mut raw = [0;4];
        self.as_ref().transaction_sync(node.as_ref(), FwTcode::ReadQuadletRequest, DSP_OFFSET as u64,
                                       raw.len(), &mut raw, timeout_ms)
            .map(|_| {
                let quad = u32::from_le_bytes(raw);
                status.parse(&quad)
            })
    }
}

/// The structure to represent state of meters.
///
/// Each value is between 0x'0000'0000'0000'0000 and 0x'3fff'ffff'ffff'ffff. 0x'0000'0000'0000'001f
/// represents negative infinite.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct FfLatterMeterState{
    pub line_inputs: Vec<i32>,
    pub mic_inputs: Vec<i32>,
    pub spdif_inputs: Vec<i32>,
    pub adat_inputs: Vec<i32>,
    pub stream_inputs: Vec<i32>,
    pub line_outputs: Vec<i32>,
    pub hp_outputs: Vec<i32>,
    pub spdif_outputs: Vec<i32>,
    pub adat_outputs: Vec<i32>,
}

const METER32_MASK: i32 = 0x07fffff0;

// Read data retrieved by each block read transaction consists of below chunks in the order:
//  32 octlets for meters detected by DSP.
//  32 quadlets for meters detected by DSP.
//  2 quadlets for unknown meters.
//  2 quadlets for tags.
//
// The first tag represents the set of content:
//  0x11111111 - hardware outputs
//  0x22222222 - channel strip for hardware inputs
//  0x33333333 - stream inputs
//  0x55555555 - fx bus
//  0x66666666 - hardware inputs
//
//  The maximum value for quadlet is 0x07fffff0. The byte in LSB is 0xf at satulated.
fn parse_meter<U: RmeFfLatterMeterSpec>(s: &mut FfLatterMeterState, raw: &[u8]) {
    let mut quadlet = [0;4];
    quadlet.copy_from_slice(&raw[388..]);
    let target = u32::from_le_bytes(quadlet);

    match target {
        // For phys outputs.
        0x11111111 => {
            [
                (s.line_outputs.iter_mut(), 0),
                (s.hp_outputs.iter_mut(), U::LINE_OUTPUT_COUNT),
                (s.spdif_outputs.iter_mut(), U::LINE_OUTPUT_COUNT + U::HP_OUTPUT_COUNT),
                (s.adat_outputs.iter_mut(), U::LINE_OUTPUT_COUNT + U::HP_OUTPUT_COUNT + U::SPDIF_OUTPUT_COUNT),
            ].iter_mut()
                .for_each(|(iter, offset)| {
                    iter.enumerate()
                        .for_each(|(i, meter)| {
                            let pos = 256 + (*offset + i) * 4;
                            quadlet.copy_from_slice(&raw[pos..(pos + 4)]);
                            *meter = i32::from_le_bytes(quadlet) & METER32_MASK;
                        });
                });
        }
        // For stream inputs.
        0x33333333 => {
            s.stream_inputs.iter_mut()
                .enumerate()
                .for_each(|(i, meter)| {
                    let pos = 256 + i * 4;
                    quadlet.copy_from_slice(&raw[pos..(pos + 4)]);
                    *meter = i32::from_le_bytes(quadlet) & METER32_MASK;
                });
        }
        // For phys inputs.
        0x66666666 => {
            [
                (s.line_inputs.iter_mut(), 0),
                (s.mic_inputs.iter_mut(), U::LINE_INPUT_COUNT),
                (s.spdif_inputs.iter_mut(), U::LINE_INPUT_COUNT + U::MIC_INPUT_COUNT),
                (s.adat_inputs.iter_mut(), U::LINE_INPUT_COUNT + U::MIC_INPUT_COUNT + U::SPDIF_INPUT_COUNT),
            ].iter_mut()
                .for_each(|(iter, offset)| {
                    iter.enumerate()
                        .for_each(|(i, meter)| {
                            let pos = 256 + (*offset + i) * 4;
                            quadlet.copy_from_slice(&raw[pos..(pos + 4)]);
                            *meter = i32::from_le_bytes(quadlet) & METER32_MASK;
                        });
                });
        }
        _ => (),
    }
}

/// The trait to represent meter protocol.
pub trait RmeFfLatterMeterSpec {
    const LINE_INPUT_COUNT: usize;
    const MIC_INPUT_COUNT: usize;
    const SPDIF_INPUT_COUNT: usize;
    const ADAT_INPUT_COUNT: usize;
    const STREAM_INPUT_COUNT: usize;

    const LINE_OUTPUT_COUNT: usize;
    const HP_OUTPUT_COUNT: usize;
    const SPDIF_OUTPUT_COUNT: usize;
    const ADAT_OUTPUT_COUNT: usize;

    fn create_meter_state() -> FfLatterMeterState {
        FfLatterMeterState{
            line_inputs: vec![Default::default();Self::LINE_INPUT_COUNT],
            mic_inputs: vec![Default::default();Self::MIC_INPUT_COUNT],
            spdif_inputs: vec![Default::default();Self::SPDIF_INPUT_COUNT],
            adat_inputs: vec![Default::default();Self::ADAT_INPUT_COUNT],
            stream_inputs: vec![Default::default();Self::STREAM_INPUT_COUNT],
            line_outputs: vec![Default::default();Self::LINE_OUTPUT_COUNT],
            hp_outputs: vec![Default::default();Self::HP_OUTPUT_COUNT],
            spdif_outputs: vec![Default::default();Self::SPDIF_OUTPUT_COUNT],
            adat_outputs: vec![Default::default();Self::ADAT_OUTPUT_COUNT],
        }
    }
}

/// The trait to represent meter protocol.
pub trait RmeFfLatterMeterProtocol<T, U> : AsRef<FwReq>
    where T: AsRef<FwNode>,
          U: RmeFfLatterMeterSpec + AsRef<FfLatterMeterState> + AsMut<FfLatterMeterState>,
{
    fn read_meter(&self, node: &T, state: &mut U, timeout_ms: u32) -> Result<(), Error> {
        (0..5).try_for_each(|_| {
            let mut raw = vec![0;392];
            self.as_ref().transaction_sync(node.as_ref(), FwTcode::ReadBlockRequest, METER_OFFSET as u64,
                                           raw.len(), &mut raw, timeout_ms)
                .map(|_| parse_meter::<U>(state.as_mut(), &raw))
        })
    }
}

/// The structure to represent state of send effects (reverb and echo).
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct FfLatterDspState{
    pub input: FfLatterInputState,
    pub output: FfLatterOutputState,
}

/// The trait to represent specification of input and output of DSP.
pub trait RmeFfLatterDspSpec {
    const LINE_INPUT_COUNT: usize;
    const MIC_INPUT_COUNT: usize;
    const SPDIF_INPUT_COUNT: usize;
    const ADAT_INPUT_COUNT: usize;
    const STREAM_INPUT_COUNT: usize;

    const LINE_OUTPUT_COUNT: usize;
    const HP_OUTPUT_COUNT: usize;
    const SPDIF_OUTPUT_COUNT: usize;
    const ADAT_OUTPUT_COUNT: usize;

    const PHYS_INPUT_COUNT: usize =
        Self::LINE_INPUT_COUNT + Self::MIC_INPUT_COUNT + Self::SPDIF_INPUT_COUNT + Self::ADAT_INPUT_COUNT;
    const INPUT_COUNT: usize = Self::PHYS_INPUT_COUNT + Self::STREAM_INPUT_COUNT;
    const OUTPUT_COUNT: usize =
        Self::LINE_OUTPUT_COUNT + Self::HP_OUTPUT_COUNT + Self::SPDIF_OUTPUT_COUNT + Self::ADAT_OUTPUT_COUNT;

    const STREAM_OFFSET: u16 = 0x0020;
    const MIXER_STEP: u16 = Self::STREAM_OFFSET * 2;

    fn create_dsp_state() -> FfLatterDspState {
        FfLatterDspState{
            input: FfLatterInputState{
                stereo_links: vec![Default::default();Self::PHYS_INPUT_COUNT / 2],
                invert_phases: vec![Default::default();Self::PHYS_INPUT_COUNT],
                line_gains: vec![Default::default();Self::LINE_INPUT_COUNT],
                line_levels: vec![Default::default();Self::LINE_INPUT_COUNT],
                mic_powers: vec![Default::default();Self::MIC_INPUT_COUNT],
                mic_insts: vec![Default::default();Self::MIC_INPUT_COUNT],
            },
            output: FfLatterOutputState{
                vols: vec![Default::default();Self::OUTPUT_COUNT],
                stereo_balance: vec![Default::default();Self::OUTPUT_COUNT / 2],
                stereo_links: vec![Default::default();Self::OUTPUT_COUNT / 2],
                invert_phases: vec![Default::default();Self::OUTPUT_COUNT],
                line_levels: vec![Default::default();Self::LINE_OUTPUT_COUNT],
            },
        }
    }
}

const VIRT_PORT_CMD_FLAG: u32 = 0x40000000;
const ODD_PARITY_FLAG: u32 = 0x80000000;

fn create_phys_port_cmd(ch: u8, cmd: u8, coef: i16) -> u32 {
    ((ch as u32) << 24) | ((cmd as u32) << 16) | (u16::from_le_bytes(coef.to_le_bytes()) as u32)
}

fn create_virt_port_cmd(mixer_step: u16, mixer: u16, ch: u16, coef: u16) -> u32 {
    VIRT_PORT_CMD_FLAG | (((mixer_step * mixer + ch) as u32) << 16) | (coef as u32)
}

/// The trait to represent DSP protocol.
///
/// DSP is configurable by quadlet write request with command aligned to little endian, which
/// consists of two parts; 16 bit target and 16 bit coefficient. The command has odd parity
/// bit in its most significant bit against the rest of bits.
pub trait RmeFfLatterDspProtocol<T, U> : AsRef<FwReq>
    where T: AsRef<FwNode>,
          U: RmeFfLatterDspSpec + AsRef<FfLatterDspState> + AsMut<FfLatterDspState>,
{
    fn write_dsp_cmd(&self, node: &T, mut cmd: u32, timeout_ms: u32) -> Result<(), Error> {
        // Add odd parity.
        if (0..32).fold(0x01, |count, shift| count ^ (cmd >> shift) & 0x1) > 0 {
            cmd |= ODD_PARITY_FLAG;
        }
        let mut raw = cmd.to_le_bytes();
        self.as_ref().transaction_sync(node.as_ref(), FwTcode::WriteQuadletRequest, DSP_OFFSET as u64,
                                       raw.len(), &mut raw, timeout_ms)
    }

    fn write_dsp_cmds(&self, node: &T, curr: &[u32], cmds: &[u32], timeout_ms: u32) -> Result<(), Error> {
        cmds.iter()
            .zip(curr.iter())
            .filter(|(n, o)| !n.eq(o))
            .try_for_each(|(&cmd, _)| self.write_dsp_cmd(node, cmd, timeout_ms))
    }
}

const INPUT_STEREO_LINK_CMD: u8         = 0x02;
const INPUT_INVERT_PHASE_CMD: u8        = 0x06;
const INPUT_LINE_GAIN_CMD: u8           = 0x07;
const INPUT_LINE_LEVEL_CMD: u8          = 0x08;
const INPUT_MIC_POWER_CMD: u8           = 0x08;
const INPUT_MIC_INST_CMD: u8            = 0x09;

const OUTPUT_VOL_CMD: u8                = 0x00;
const OUTPUT_STEREO_BALANCE_CMD: u8     = 0x01;
const OUTPUT_STEREO_LINK_CMD: u8        = 0x04;
const OUTPUT_INVERT_PHASE_CMD: u8       = 0x07;
const OUTPUT_LINE_LEVEL_CMD: u8         = 0x08;

/// The structure to represent nominal level of analog input.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum LatterInNominalLevel {
    Low,
    /// +4 dBu.
    Professional,
}

impl Default for LatterInNominalLevel {
    fn default() -> Self {
        Self::Low
    }
}

impl From<LatterInNominalLevel> for i16 {
    fn from(level: LatterInNominalLevel) -> Self {
        match level {
            LatterInNominalLevel::Low => 0x0000,
            LatterInNominalLevel::Professional => 0x0001,
        }
    }
}

/// The structure to represent state of inputs.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct FfLatterInputState{
    /// Whether to link each pair of left and right ports.
    pub stereo_links: Vec<bool>,
    /// Whether to inverse the phase of analog, spdif, and adat inputs.
    pub invert_phases: Vec<bool>,
    /// The gain of analog line input. The value is between 0 and 120 to represent 0.00 dB and 12.00 dB.
    pub line_gains: Vec<i16>,
    /// The nominal level of analog line input.
    pub line_levels: Vec<LatterInNominalLevel>,
    /// Whether to enable powering for mic input.
    pub mic_powers: Vec<bool>,
    /// Whether to use mic input for instrument.
    pub mic_insts: Vec<bool>,

}

fn input_state_to_cmds<T: RmeFfLatterDspSpec>(state: &FfLatterInputState) -> Vec<u32> {
    assert_eq!(state.stereo_links.len(), T::PHYS_INPUT_COUNT / 2);
    assert_eq!(state.invert_phases.len(), T::PHYS_INPUT_COUNT);
    assert_eq!(state.line_gains.len(), T::LINE_INPUT_COUNT);
    assert_eq!(state.line_levels.len(), T::LINE_INPUT_COUNT);
    assert_eq!(state.mic_powers.len(), T::MIC_INPUT_COUNT);
    assert_eq!(state.mic_insts.len(), T::MIC_INPUT_COUNT);

    let mut cmds = Vec::new();

    state.stereo_links.iter()
        .enumerate()
        .for_each(|(i, &link)| {
            let ch = (i * 2) as u8;
            cmds.push(create_phys_port_cmd(ch, INPUT_STEREO_LINK_CMD, link as i16));
        });

    state.invert_phases.iter()
        .enumerate()
        .for_each(|(i, &invert_phase)| {
            let ch = i as u8;
            cmds.push(create_phys_port_cmd(ch, INPUT_INVERT_PHASE_CMD, invert_phase as i16));
        });

    state.line_gains.iter()
        .enumerate()
        .for_each(|(i, &gain)| {
            let ch = i as u8;
            cmds.push(create_phys_port_cmd(ch, INPUT_LINE_GAIN_CMD, gain as i16));
        });

    state.line_levels.iter()
        .enumerate()
        .for_each(|(i, &level)| {
            let ch = i as u8;
            cmds.push(create_phys_port_cmd(ch, INPUT_LINE_LEVEL_CMD, level as i16));
        });

    state.mic_powers.iter()
        .enumerate()
        .for_each(|(i, &power)| {
            let ch = i as u8;
            cmds.push(create_phys_port_cmd(ch, INPUT_MIC_POWER_CMD, power as i16));
        });

    state.mic_insts.iter()
        .enumerate()
        .for_each(|(i, &inst)| {
            let ch = i as u8;
            cmds.push(create_phys_port_cmd(ch as u8, INPUT_MIC_INST_CMD, inst as i16));
        });

    cmds
}

/// The trait to represent input protocol.
pub trait RmeFfLatterInputProtocol<T, U> : RmeFfLatterDspProtocol<T, U>
    where T: AsRef<FwNode>,
          U: RmeFfLatterDspSpec + AsRef<FfLatterDspState> + AsMut<FfLatterDspState>,
{
    fn init_input(&self, node: &T, state: &U, timeout_ms: u32) -> Result<(), Error> {
        let cmds = input_state_to_cmds::<U>(&state.as_ref().input);
        cmds.iter()
            .try_for_each(|&cmd| self.write_dsp_cmd(node, cmd, timeout_ms))
    }

    fn write_input(&self, node: &T, state: &mut U, input: FfLatterInputState, timeout_ms: u32)
        -> Result<(), Error>
    {
        let old = input_state_to_cmds::<U>(&state.as_ref().input);
        let new = input_state_to_cmds::<U>(&input);

        self.write_dsp_cmds(node, &old, &new, timeout_ms)
            .map(|_| state.as_mut().input = input)
    }
}

impl<T, U, O> RmeFfLatterInputProtocol<T, U> for O
    where T: AsRef<FwNode>,
          U: RmeFfLatterDspSpec + AsRef<FfLatterDspState> + AsMut<FfLatterDspState>,
          O: RmeFfLatterDspProtocol<T, U>,
{}

impl From<LineOutNominalLevel> for i16 {
    fn from(level: LineOutNominalLevel) -> Self {
        match level {
            LineOutNominalLevel::Consumer => 0x0000,
            LineOutNominalLevel::Professional => 0x0001,
            LineOutNominalLevel::High => 0x0002,
        }
    }
}

/// The structure to represent state of inputs.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct FfLatterOutputState{
    /// The level of volume. Each value is between -650 (0xfd76) and 60 (0x003c) to represent
    /// -65.00 dB and 6.00 dB.
    pub vols: Vec<i16>,
    /// The balance between left and right. The value is between -100 (0xff9c) and 100 (0x0064).
    pub stereo_balance: Vec<i16>,
    /// Whether to link each pair of left and right ports.
    pub stereo_links: Vec<bool>,
    /// Whether to inverse the phase of analog, spdif, and adat outputs.
    pub invert_phases: Vec<bool>,
    /// The nominal level of analog line output.
    pub line_levels: Vec<LineOutNominalLevel>,
}

fn output_state_to_cmds<T: RmeFfLatterDspSpec>(state: &FfLatterOutputState) -> Vec<u32> {
    assert_eq!(state.vols.len(), T::PHYS_INPUT_COUNT);
    assert_eq!(state.stereo_balance.len(), T::PHYS_INPUT_COUNT / 2);
    assert_eq!(state.stereo_links.len(), T::PHYS_INPUT_COUNT / 2);
    assert_eq!(state.invert_phases.len(), T::PHYS_INPUT_COUNT);
    assert_eq!(state.line_levels.len(), T::LINE_OUTPUT_COUNT);

    let mut cmds = Vec::new();

    state.vols.iter()
        .enumerate()
        .for_each(|(i, &vol)| {
            let ch = (T::PHYS_INPUT_COUNT + i) as u8;
            cmds.push(create_phys_port_cmd(ch, OUTPUT_VOL_CMD, vol));
        });

    state.stereo_balance.iter()
        .enumerate()
        .for_each(|(i, &balance)| {
            let ch = (T::PHYS_INPUT_COUNT + i * 2) as u8;
            cmds.push(create_phys_port_cmd(ch, OUTPUT_STEREO_BALANCE_CMD, balance));
        });

    state.stereo_links.iter()
        .enumerate()
        .for_each(|(i, &link)| {
            let ch = (T::PHYS_INPUT_COUNT + i * 2) as u8;
            cmds.push(create_phys_port_cmd(ch, OUTPUT_STEREO_LINK_CMD, link as i16));
        });

    state.invert_phases.iter()
        .enumerate()
        .for_each(|(i, &invert_phase)| {
            let ch = (T::PHYS_INPUT_COUNT + i) as u8;
            cmds.push(create_phys_port_cmd(ch, OUTPUT_INVERT_PHASE_CMD, invert_phase as i16));
        });

    state.line_levels.iter()
        .enumerate()
        .for_each(|(i, &line_level)| {
            let ch = (T::PHYS_INPUT_COUNT + i) as u8;
            cmds.push(create_phys_port_cmd(ch, OUTPUT_LINE_LEVEL_CMD, line_level as i16));
        });

    cmds
}

/// The trait to represent output protocol.
pub trait RmeFfLatterOutputProtocol<T, U> : RmeFfLatterDspProtocol<T, U>
    where T: AsRef<FwNode>,
          U: RmeFfLatterDspSpec + AsRef<FfLatterDspState> + AsMut<FfLatterDspState>,
{
    const CH_OFFSET: u8 = U::PHYS_INPUT_COUNT as u8;

    fn init_output(&self, node: &T, state: &U, timeout_ms: u32) -> Result<(), Error> {
        let cmds = output_state_to_cmds::<U>(&state.as_ref().output);
        cmds.iter()
            .try_for_each(|&cmd| self.write_dsp_cmd(node, cmd, timeout_ms))
    }

    fn write_output(&self, node: &T, state: &mut U, output: FfLatterOutputState, timeout_ms: u32)
        -> Result<(), Error>
    {
        let old = output_state_to_cmds::<U>(&state.as_ref().output);
        let new = output_state_to_cmds::<U>(&output);

        self.write_dsp_cmds(node, &old, &new, timeout_ms)
            .map(|_| state.as_mut().output = output)
    }
}

impl<T, U, O> RmeFfLatterOutputProtocol<T, U> for O
    where T: AsRef<FwNode>,
          U: RmeFfLatterDspSpec + AsRef<FfLatterDspState> + AsMut<FfLatterDspState>,
          O: RmeFfLatterDspProtocol<T, U>,
{}
