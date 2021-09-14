// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol defined by RME GmbH for latter models of Fireface series.

pub mod ucx;
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
pub trait RmeFfLatterConfigOperation<U: RmeFfLatterRegisterValueOperation> {
    fn write_cfg(
        req: &mut FwReq,
        node: &mut FwNode,
        cfg: &U,
        timeout_ms: u32
    ) -> Result<(), Error> {
        let mut quad = 0u32;
        cfg.build(&mut quad);

        let mut raw = [0; 4];
        raw.copy_from_slice(&quad.to_le_bytes());
        req.transaction_sync(
            node,
            FwTcode::WriteQuadletRequest,
            CFG_OFFSET as u64,
            raw.len(),
            &mut raw,
            timeout_ms
        )
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
pub trait RmeFfLatterStatusOperation<U>
    where U: RmeFfLatterRegisterValueOperation,
{
    fn read_status(
        req: &mut FwReq,
        node: &mut FwNode,
        status: &mut U,
        timeout_ms: u32
    ) -> Result<(), Error> {
        let mut raw = [0; 4];
        req.transaction_sync(
            node,
            FwTcode::ReadQuadletRequest,
            DSP_OFFSET as u64,
            raw.len(),
            &mut raw,
            timeout_ms
        )
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
#[derive(Default, Debug, Clone, Eq, PartialEq)]
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

/// The trait to represent meter protocol.
pub trait RmeFfLatterMeterOperation {
    const LINE_INPUT_COUNT: usize;
    const MIC_INPUT_COUNT: usize;
    const SPDIF_INPUT_COUNT: usize;
    const ADAT_INPUT_COUNT: usize;
    const STREAM_INPUT_COUNT: usize;

    const LINE_OUTPUT_COUNT: usize;
    const HP_OUTPUT_COUNT: usize;
    const SPDIF_OUTPUT_COUNT: usize;
    const ADAT_OUTPUT_COUNT: usize;

    const LEVEL_MIN: i32 = 0x0;
    const LEVEL_MAX: i32 = 0x07fffff0;
    const LEVEL_STEP: i32 = 0x10;

    fn create_meter_state() -> FfLatterMeterState {
        FfLatterMeterState{
            line_inputs: vec![Default::default(); Self::LINE_INPUT_COUNT],
            mic_inputs: vec![Default::default(); Self::MIC_INPUT_COUNT],
            spdif_inputs: vec![Default::default(); Self::SPDIF_INPUT_COUNT],
            adat_inputs: vec![Default::default(); Self::ADAT_INPUT_COUNT],
            stream_inputs: vec![Default::default(); Self::STREAM_INPUT_COUNT],
            line_outputs: vec![Default::default(); Self::LINE_OUTPUT_COUNT],
            hp_outputs: vec![Default::default(); Self::HP_OUTPUT_COUNT],
            spdif_outputs: vec![Default::default(); Self::SPDIF_OUTPUT_COUNT],
            adat_outputs: vec![Default::default(); Self::ADAT_OUTPUT_COUNT],
        }
    }

    fn read_meter(
        req: &mut FwReq,
        node: &mut FwNode,
        state: &mut FfLatterMeterState,
        timeout_ms: u32
    ) -> Result<(), Error> {
        (0..5).try_for_each(|_| {
            let mut raw = vec![0;392];
            req.transaction_sync(
                node,
                FwTcode::ReadBlockRequest,
                METER_OFFSET as u64,
                raw.len(),
                &mut raw,
                timeout_ms
            )
                .map(|_| Self::parse_meter(state, &raw))
        })
    }

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
    fn parse_meter(s: &mut FfLatterMeterState, raw: &[u8]) {
        let mut quadlet = [0; 4];
        quadlet.copy_from_slice(&raw[388..]);
        let target = u32::from_le_bytes(quadlet);
    
        match target {
            // For phys outputs.
            0x11111111 => {
                [
                    (s.line_outputs.iter_mut(), 0),
                    (s.hp_outputs.iter_mut(), Self::LINE_OUTPUT_COUNT),
                    (
                        s.spdif_outputs.iter_mut(),
                        Self::LINE_OUTPUT_COUNT + Self::HP_OUTPUT_COUNT
                    ),
                    (
                        s.adat_outputs.iter_mut(),
                        Self::LINE_OUTPUT_COUNT + Self::HP_OUTPUT_COUNT + Self::SPDIF_OUTPUT_COUNT
                    ),
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
                    (s.mic_inputs.iter_mut(), Self::LINE_INPUT_COUNT),
                    (
                        s.spdif_inputs.iter_mut(),
                        Self::LINE_INPUT_COUNT + Self::MIC_INPUT_COUNT
                    ),
                    (
                        s.adat_inputs.iter_mut(),
                        Self::LINE_INPUT_COUNT + Self::MIC_INPUT_COUNT + Self::SPDIF_INPUT_COUNT
                    ),
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
}

/// The structure to represent state of send effects (reverb and echo).
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct FfLatterDspState {
    pub input: FfLatterInputState,
    pub output: FfLatterOutputState,
    pub mixer: Vec<FfLatterMixerState>,
    pub input_ch_strip: FfLatterInputChStripState,
    pub output_ch_strip: FfLatterOutputChStripState,
    pub fx: FfLatterFxState,
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
pub trait RmeFfLatterDspOperation {
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
        FfLatterDspState {
            input: FfLatterInputState {
                stereo_links: vec![Default::default(); Self::PHYS_INPUT_COUNT / 2],
                invert_phases: vec![Default::default(); Self::PHYS_INPUT_COUNT],
                line_gains: vec![Default::default(); Self::LINE_INPUT_COUNT],
                line_levels: vec![Default::default(); Self::LINE_INPUT_COUNT],
                mic_powers: vec![Default::default(); Self::MIC_INPUT_COUNT],
                mic_insts: vec![Default::default(); Self::MIC_INPUT_COUNT],
            },
            output: FfLatterOutputState {
                vols: vec![Default::default(); Self::OUTPUT_COUNT],
                stereo_balance: vec![Default::default(); Self::OUTPUT_COUNT / 2],
                stereo_links: vec![Default::default(); Self::OUTPUT_COUNT / 2],
                invert_phases: vec![Default::default(); Self::OUTPUT_COUNT],
                line_levels: vec![Default::default(); Self::LINE_OUTPUT_COUNT],
            },
            mixer: vec![FfLatterMixerState {
                line_gains: vec![Default::default(); Self::LINE_INPUT_COUNT],
                mic_gains: vec![Default::default(); Self::MIC_INPUT_COUNT],
                spdif_gains: vec![Default::default(); Self::SPDIF_INPUT_COUNT],
                adat_gains: vec![Default::default(); Self::ADAT_INPUT_COUNT],
                stream_gains: vec![Default::default(); Self::STREAM_INPUT_COUNT],
            }; Self::OUTPUT_COUNT],
            input_ch_strip: FfLatterInputChStripState(FfLatterChStripState {
                hpf: FfLatterHpfState {
                    activates: vec![Default::default(); Self::PHYS_INPUT_COUNT],
                    cut_offs: vec![Default::default(); Self::PHYS_INPUT_COUNT],
                    roll_offs: vec![Default::default(); Self::PHYS_INPUT_COUNT],
                },
                eq: FfLatterEqState{
                    activates: vec![Default::default(); Self::PHYS_INPUT_COUNT],
                    low_types: vec![Default::default(); Self::PHYS_INPUT_COUNT],
                    low_gains: vec![Default::default(); Self::PHYS_INPUT_COUNT],
                    low_freqs: vec![Default::default(); Self::PHYS_INPUT_COUNT],
                    low_qualities: vec![Default::default(); Self::PHYS_INPUT_COUNT],
                    middle_gains: vec![Default::default(); Self::PHYS_INPUT_COUNT],
                    middle_freqs: vec![Default::default(); Self::PHYS_INPUT_COUNT],
                    middle_qualities: vec![Default::default(); Self::PHYS_INPUT_COUNT],
                    high_types: vec![Default::default(); Self::PHYS_INPUT_COUNT],
                    high_gains: vec![Default::default(); Self::PHYS_INPUT_COUNT],
                    high_freqs: vec![Default::default(); Self::PHYS_INPUT_COUNT],
                    high_qualities: vec![Default::default(); Self::PHYS_INPUT_COUNT],
                },
                dynamics: FfLatterDynState {
                    activates: vec![Default::default(); Self::PHYS_INPUT_COUNT],
                    gains: vec![Default::default(); Self::PHYS_INPUT_COUNT],
                    attacks: vec![Default::default(); Self::PHYS_INPUT_COUNT],
                    releases: vec![Default::default(); Self::PHYS_INPUT_COUNT],
                    compressor_thresholds: vec![Default::default(); Self::PHYS_INPUT_COUNT],
                    compressor_ratios: vec![Default::default(); Self::PHYS_INPUT_COUNT],
                    expander_thresholds: vec![Default::default(); Self::PHYS_INPUT_COUNT],
                    expander_ratios: vec![Default::default(); Self::PHYS_INPUT_COUNT],
                },
                autolevel: FfLatterAutolevelState {
                    activates: vec![Default::default(); Self::PHYS_INPUT_COUNT],
                    max_gains: vec![Default::default(); Self::PHYS_INPUT_COUNT],
                    headrooms: vec![Default::default(); Self::PHYS_INPUT_COUNT],
                    rise_times: vec![Default::default(); Self::PHYS_INPUT_COUNT],
                },
            }),
            output_ch_strip: FfLatterOutputChStripState(FfLatterChStripState{
                hpf: FfLatterHpfState {
                    activates: vec![Default::default(); Self::OUTPUT_COUNT],
                    cut_offs: vec![Default::default(); Self::OUTPUT_COUNT],
                    roll_offs: vec![Default::default(); Self::OUTPUT_COUNT],
                },
                eq: FfLatterEqState {
                    activates: vec![Default::default(); Self::OUTPUT_COUNT],
                    low_types: vec![Default::default(); Self::OUTPUT_COUNT],
                    low_gains: vec![Default::default(); Self::OUTPUT_COUNT],
                    low_freqs: vec![Default::default(); Self::OUTPUT_COUNT],
                    low_qualities: vec![Default::default(); Self::OUTPUT_COUNT],
                    middle_gains: vec![Default::default(); Self::OUTPUT_COUNT],
                    middle_freqs: vec![Default::default(); Self::OUTPUT_COUNT],
                    middle_qualities: vec![Default::default(); Self::OUTPUT_COUNT],
                    high_types: vec![Default::default(); Self::OUTPUT_COUNT],
                    high_gains: vec![Default::default(); Self::OUTPUT_COUNT],
                    high_freqs: vec![Default::default(); Self::OUTPUT_COUNT],
                    high_qualities: vec![Default::default(); Self::OUTPUT_COUNT],
                },
                dynamics: FfLatterDynState {
                    activates: vec![Default::default(); Self::OUTPUT_COUNT],
                    gains: vec![Default::default(); Self::OUTPUT_COUNT],
                    attacks: vec![Default::default(); Self::OUTPUT_COUNT],
                    releases: vec![Default::default(); Self::OUTPUT_COUNT],
                    compressor_thresholds: vec![Default::default(); Self::OUTPUT_COUNT],
                    compressor_ratios: vec![Default::default(); Self::OUTPUT_COUNT],
                    expander_thresholds: vec![Default::default(); Self::OUTPUT_COUNT],
                    expander_ratios: vec![Default::default(); Self::OUTPUT_COUNT],
                },
                autolevel: FfLatterAutolevelState {
                    activates: vec![Default::default(); Self::OUTPUT_COUNT],
                    max_gains: vec![Default::default(); Self::OUTPUT_COUNT],
                    headrooms: vec![Default::default(); Self::OUTPUT_COUNT],
                    rise_times: vec![Default::default(); Self::OUTPUT_COUNT],
                },
            }),
            fx: FfLatterFxState {
                line_input_gains: vec![0; Self::LINE_INPUT_COUNT],
                mic_input_gains: vec![0; Self::MIC_INPUT_COUNT],
                spdif_input_gains: vec![0; Self::SPDIF_INPUT_COUNT],
                adat_input_gains: vec![0; Self::ADAT_INPUT_COUNT],
                stream_input_gains: vec![0; Self::STREAM_INPUT_COUNT],
                line_output_vols: vec![0; Self::LINE_OUTPUT_COUNT],
                hp_output_vols: vec![0; Self::HP_OUTPUT_COUNT],
                spdif_output_vols: vec![0; Self::SPDIF_OUTPUT_COUNT],
                adat_output_vols: vec![0; Self::ADAT_OUTPUT_COUNT],
                reverb: Default::default(),
                echo: Default::default(),
            }
        }
    }

    fn write_dsp_cmd(
        req: &mut FwReq,
        node: &mut FwNode,
        mut cmd: u32,
        timeout_ms: u32
    ) -> Result<(), Error> {
        // Add odd parity.
        if (0..32).fold(0x01, |count, shift| count ^ (cmd >> shift) & 0x1) > 0 {
            cmd |= ODD_PARITY_FLAG;
        }
        let mut raw = cmd.to_le_bytes();
        req.transaction_sync(
            node,
            FwTcode::WriteQuadletRequest,
            DSP_OFFSET as u64,
            raw.len(),
            &mut raw,
            timeout_ms
        )
    }

    fn write_dsp_cmds(
        req: &mut FwReq,
        node: &mut FwNode,
        curr: &[u32],
        cmds: &[u32],
        timeout_ms: u32
    ) -> Result<(), Error> {
        cmds.iter()
            .zip(curr.iter())
            .filter(|(n, o)| !n.eq(o))
            .try_for_each(|(&cmd, _)| Self::write_dsp_cmd(req, node, cmd, timeout_ms))
    }
}

const INPUT_TO_FX_CMD: u8               = 0x01;
const INPUT_STEREO_LINK_CMD: u8         = 0x02;
const INPUT_INVERT_PHASE_CMD: u8        = 0x06;
const INPUT_LINE_GAIN_CMD: u8           = 0x07;
const INPUT_LINE_LEVEL_CMD: u8          = 0x08;
const INPUT_MIC_POWER_CMD: u8           = 0x08;
const INPUT_MIC_INST_CMD: u8            = 0x09;

const OUTPUT_VOL_CMD: u8                = 0x00;
const OUTPUT_STEREO_BALANCE_CMD: u8     = 0x01;
const OUTPUT_FROM_FX_CMD: u8            = 0x03;
const OUTPUT_STEREO_LINK_CMD: u8        = 0x04;
const OUTPUT_INVERT_PHASE_CMD: u8       = 0x07;
const OUTPUT_LINE_LEVEL_CMD: u8         = 0x08;

const HPF_ACTIVATE_CMD: u8              = 0x20;
const HPF_CUT_OFF_CMD: u8               = 0x21;
const HPF_ROLL_OFF_CMD: u8              = 0x22;
const EQ_ACTIVATE_CMD: u8               = 0x40;
const EQ_LOW_TYPE_CMD: u8               = 0x41;
const EQ_LOW_GAIN_CMD: u8               = 0x42;
const EQ_LOW_FREQ_CMD: u8               = 0x43;
const EQ_LOW_QUALITY_CMD: u8            = 0x44;
const EQ_MIDDLE_GAIN_CMD: u8            = 0x45;
const EQ_MIDDLE_FREQ_CMD: u8            = 0x46;
const EQ_MIDDLE_QUALITY_CMD: u8         = 0x47;
const EQ_HIGH_TYPE_CMD: u8              = 0x48;
const EQ_HIGH_GAIN_CMD: u8              = 0x49;
const EQ_HIGH_FREQ_CMD: u8              = 0x4a;
const EQ_HIGH_QUALITY_CMD: u8           = 0x4b;
const DYN_ACTIVATE_CMD: u8              = 0x60;
const DYN_GAIN_CMD: u8                  = 0x61;
const DYN_ATTACK_CMD: u8                = 0x62;
const DYN_RELEASE_CMD: u8               = 0x63;
const DYN_COMPR_THRESHOLD_CMD: u8       = 0x64;
const DYN_COMPR_RATIO_CMD: u8           = 0x65;
const DYN_EXPANDER_THRESHOLD_CMD: u8    = 0x66;
const DYN_EXPANDER_RATIO_CMD: u8        = 0x67;
const AUTOLEVEL_ACTIVATE_CMD: u8        = 0x80;
const AUTOLEVEL_MAX_GAIN_CMD: u8        = 0x81;
const AUTOLEVEL_HEADROOM_CMD: u8        = 0x82;
const AUTOLEVEL_RISE_TIME_CMD: u8       = 0x83;

const FX_REVERB_ACTIVATE_CMD: u8        = 0x00;
const FX_REVERB_TYPE_CMD: u8            = 0x01;
const FX_REVERB_PRE_DELAY_CMD: u8       = 0x02;
const FX_REVERB_PRE_HPF_FREQ_CMD: u8    = 0x03;
const FX_REVERB_ROOM_SCALE_CMD: u8      = 0x04;
const FX_REVERB_ATTACK_CMD: u8          = 0x05;
const FX_REVERB_HOLD_CMD: u8            = 0x06;
const FX_REVERB_RELEASE_CMD: u8         = 0x07;
const FX_REVERB_POST_LPF_FREQ_CMD: u8   = 0x08;
const FX_REVERB_TIME_CMD: u8            = 0x09;
const FX_REVERB_DAMPING_FREQ_CMD: u8    = 0x0a;
const FX_REVERB_SMOOTH_CMD: u8          = 0x0b;
const FX_REVERB_VOLUME_CMD: u8          = 0x0c;
const FX_REVERB_STEREO_WIDTH_CMD: u8    = 0x0d;

const FX_ECHO_ACTIVATE_CMD: u8          = 0x20;
const FX_ECHO_TYPE_CMD: u8              = 0x21;
const FX_ECHO_DELAY_CMD: u8             = 0x22;
const FX_ECHO_FEEDBACK_CMD: u8          = 0x23;
const FX_ECHO_LPF_FREQ_CMD: u8          = 0x24;
const FX_ECHO_VOLUME_CMD: u8            = 0x25;
const FX_ECHO_STEREO_WIDTH_CMD: u8      = 0x26;

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
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct FfLatterInputState {
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

/// The trait to represent input protocol.
pub trait RmeFfLatterInputOperation: RmeFfLatterDspOperation {
    const PHYS_INPUT_GAIN_MIN: i32 = 0;
    const PHYS_INPUT_GAIN_MAX: i32 = 120;
    const PHYS_INPUT_GAIN_STEP: i32 = 1;

    fn init_input(
        req: &mut FwReq,
        node: &mut FwNode,
        state: &FfLatterDspState,
        timeout_ms: u32
    ) -> Result<(), Error> {
        let cmds = Self::input_state_to_cmds(&state.input);
        cmds.iter().try_for_each(|&cmd| Self::write_dsp_cmd(req, node, cmd, timeout_ms))
    }

    fn write_input(
        req: &mut FwReq,
        node: &mut FwNode,
        state: &mut FfLatterDspState,
        input: FfLatterInputState,
        timeout_ms: u32
    ) -> Result<(), Error> {
        let old = Self::input_state_to_cmds(&state.input);
        let new = Self::input_state_to_cmds(&input);

        Self::write_dsp_cmds(req, node, &old, &new, timeout_ms).map(|_| state.input = input)
    }

    fn input_state_to_cmds(state: &FfLatterInputState) -> Vec<u32> {
        assert_eq!(state.stereo_links.len(), Self::PHYS_INPUT_COUNT / 2);
        assert_eq!(state.invert_phases.len(), Self::PHYS_INPUT_COUNT);
        assert_eq!(state.line_gains.len(), Self::LINE_INPUT_COUNT);
        assert_eq!(state.line_levels.len(), Self::LINE_INPUT_COUNT);
        assert_eq!(state.mic_powers.len(), Self::MIC_INPUT_COUNT);
        assert_eq!(state.mic_insts.len(), Self::MIC_INPUT_COUNT);

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
}

impl<O: RmeFfLatterDspOperation> RmeFfLatterInputOperation for O {}

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
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct FfLatterOutputState {
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

/// The trait to represent output protocol.
pub trait RmeFfLatterOutputOperation: RmeFfLatterDspOperation {
    const PHYS_OUTPUT_VOL_MIN: i32 = -650;
    const PHYS_OUTPUT_VOL_MAX: i32 = 60;
    const PHYS_OUTPUT_VOL_STEP: i32 = 1;

    const PHYS_OUTPUT_BALANCE_MIN: i32 = -100;
    const PHYS_OUTPUT_BALANCE_MAX: i32 = 100;
    const PHYS_OUTPUT_BALANCE_STEP: i32 = 1;

    const CH_OFFSET: u8 = Self::PHYS_INPUT_COUNT as u8;

    fn init_output(
        req: &mut FwReq,
        node: &mut FwNode,
        state: &FfLatterDspState,
        timeout_ms: u32
    ) -> Result<(), Error> {
        let cmds = Self::output_state_to_cmds(&state.output);
        cmds.iter()
            .try_for_each(|&cmd| Self::write_dsp_cmd(req, node, cmd, timeout_ms))
    }

    fn write_output(
        req: &mut FwReq,
        node: &mut FwNode,
        state: &mut FfLatterDspState,
        output: FfLatterOutputState,
        timeout_ms: u32
    ) -> Result<(), Error> {
        let old = Self::output_state_to_cmds(&state.output);
        let new = Self::output_state_to_cmds(&output);

        Self::write_dsp_cmds(req, node, &old, &new, timeout_ms)
            .map(|_| state.output = output)
    }

    fn output_state_to_cmds(state: &FfLatterOutputState) -> Vec<u32> {
        assert_eq!(state.vols.len(), Self::PHYS_INPUT_COUNT);
        assert_eq!(state.stereo_balance.len(), Self::PHYS_INPUT_COUNT / 2);
        assert_eq!(state.stereo_links.len(), Self::PHYS_INPUT_COUNT / 2);
        assert_eq!(state.invert_phases.len(), Self::PHYS_INPUT_COUNT);
        assert_eq!(state.line_levels.len(), Self::LINE_OUTPUT_COUNT);

        let mut cmds = Vec::new();

        state.vols.iter()
            .enumerate()
            .for_each(|(i, &vol)| {
                let ch = (Self::PHYS_INPUT_COUNT + i) as u8;
                cmds.push(create_phys_port_cmd(ch, OUTPUT_VOL_CMD, vol));
            });

        state.stereo_balance.iter()
            .enumerate()
            .for_each(|(i, &balance)| {
                let ch = (Self::PHYS_INPUT_COUNT + i * 2) as u8;
                cmds.push(create_phys_port_cmd(ch, OUTPUT_STEREO_BALANCE_CMD, balance));
            });

        state.stereo_links.iter()
            .enumerate()
            .for_each(|(i, &link)| {
                let ch = (Self::PHYS_INPUT_COUNT + i * 2) as u8;
                cmds.push(create_phys_port_cmd(ch, OUTPUT_STEREO_LINK_CMD, link as i16));
            });

        state.invert_phases.iter()
            .enumerate()
            .for_each(|(i, &invert_phase)| {
                let ch = (Self::PHYS_INPUT_COUNT + i) as u8;
                cmds.push(create_phys_port_cmd(ch, OUTPUT_INVERT_PHASE_CMD, invert_phase as i16));
            });

        state.line_levels.iter()
            .enumerate()
            .for_each(|(i, &line_level)| {
                let ch = (Self::PHYS_INPUT_COUNT + i) as u8;
                cmds.push(create_phys_port_cmd(ch, OUTPUT_LINE_LEVEL_CMD, line_level as i16));
            });

        cmds
    }
}

impl<O: RmeFfLatterDspOperation> RmeFfLatterOutputOperation for O {}

/// The structure to represent state of mixer.
///
/// Each value is between 0x0000 and 0xa000 through 0x9000 to represent -65.00 dB and 6.00 dB
/// through 0.00 dB.
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct FfLatterMixerState {
    pub line_gains: Vec<u16>,
    pub mic_gains: Vec<u16>,
    pub spdif_gains: Vec<u16>,
    pub adat_gains: Vec<u16>,
    pub stream_gains: Vec<u16>,
}

/// The trait to represent mixer protocol.
pub trait RmeFfLatterMixerOperation: RmeFfLatterDspOperation {
    const MIXER_INPUT_GAIN_MIN: i32 = 0x0000;
    const MIXER_INPUT_GAIN_ZERO: i32 = 0x9000;
    const MIXER_INPUT_GAIN_MAX: i32 = 0xa000;
    const MIXER_INPUT_GAIN_STEP: i32 = 1;

    fn init_mixers(
        req: &mut FwReq,
        node: &mut FwNode,
        state: &FfLatterDspState,
        timeout_ms: u32
    ) -> Result<(), Error> {
        let mixers = &state.mixer;

        mixers.iter()
            .enumerate()
            .try_for_each(|(i, mixer)| {
                let ch = i as u16;
                let cmds = Self::mixer_state_to_cmds(&mixer, ch);
                cmds.iter()
                    .try_for_each(|&cmd| Self::write_dsp_cmd(req, node, cmd, timeout_ms))
            })
    }

    fn write_mixer(
        req: &mut FwReq,
        node: &mut FwNode,
        state: &mut FfLatterDspState,
        index: usize,
        mixer: FfLatterMixerState,
        timeout_ms: u32
    ) -> Result<(), Error> {
        let old = Self::mixer_state_to_cmds(&state.mixer[index], index as u16);
        let new = Self::mixer_state_to_cmds(&mixer, index as u16);

        Self::write_dsp_cmds(req, node, &old, &new, timeout_ms)
            .map(|_| state.mixer[index] = mixer)
    }

    fn mixer_state_to_cmds(state: &FfLatterMixerState, index: u16) -> Vec<u32> {
        let mut cmds = Vec::new();

        state.line_gains.iter()
            .chain(state.mic_gains.iter())
            .chain(state.spdif_gains.iter())
            .chain(state.adat_gains.iter())
            .enumerate()
            .for_each(|(i, &gain)| {
                let ch = i as u16;
                cmds.push(create_virt_port_cmd(Self::MIXER_STEP, index, ch, gain));
            });

        state.stream_gains.iter()
            .enumerate()
            .for_each(|(i, &gain)| {
                let ch = (Self::STREAM_OFFSET as u16) + i as u16;
                cmds.push(create_virt_port_cmd(Self::MIXER_STEP, index, ch, gain));
            });

        cmds
    }
}

impl<O: RmeFfLatterDspOperation> RmeFfLatterMixerOperation for O {}

/// The enum to represent level of roll off in high pass filter.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum FfLatterHpfRollOffLevel {
    L6,
    L12,
    L18,
    L24,
}

impl Default for FfLatterHpfRollOffLevel {
    fn default() -> Self {
        Self::L6
    }
}

impl From<FfLatterHpfRollOffLevel> for i16 {
    fn from(freq: FfLatterHpfRollOffLevel) -> Self {
        match freq {
            FfLatterHpfRollOffLevel::L6 => 0,
            FfLatterHpfRollOffLevel::L12 => 1,
            FfLatterHpfRollOffLevel::L18 => 2,
            FfLatterHpfRollOffLevel::L24 => 3,
        }
    }
}

/// The structure to represent state of high pass filter in channel strip effect.
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct FfLatterHpfState {
    /// Whether to activate high pass filter.
    pub activates: Vec<bool>,
    /// The frequency to cut between 20 and 500 Hz.
    pub cut_offs: Vec<u16>,
    /// The ratio to decline gain.
    pub roll_offs: Vec<FfLatterHpfRollOffLevel>,
}

fn hpf_state_to_cmds(state: &FfLatterHpfState, ch_offset: u8) -> Vec<u32> {
    assert_eq!(state.cut_offs.len(), state.activates.len());
    assert_eq!(state.roll_offs.len(), state.activates.len());

    let mut cmds = Vec::new();

    (0..state.activates.len())
        .for_each(|i| {
            let ch = ch_offset + i as u8;
            cmds.push(create_phys_port_cmd(ch, HPF_ACTIVATE_CMD, state.activates[i] as i16));
            cmds.push(create_phys_port_cmd(ch, HPF_CUT_OFF_CMD, state.cut_offs[i] as i16));
            cmds.push(create_phys_port_cmd(ch, HPF_ROLL_OFF_CMD, i16::from(state.roll_offs[i])));
        });

    cmds
}

/// The enum to represent type of bandwidth equalizing.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum FfLatterChStripEqType {
    Peak,
    Shelf,
    LowPass,
}

impl Default for FfLatterChStripEqType {
    fn default() -> Self {
        Self::Peak
    }
}

impl From<FfLatterChStripEqType> for i16 {
    fn from(eq_type: FfLatterChStripEqType) -> Self {
        match eq_type {
            FfLatterChStripEqType::Peak => 0x0000,
            FfLatterChStripEqType::Shelf => 0x0001,
            FfLatterChStripEqType::LowPass => 0x0002,
        }
    }
}

/// The structure to represent state of equalizer in channel strip effect.
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct FfLatterEqState {
    /// Whether to activate equalizer.
    pub activates: Vec<bool>,
    /// The type of equalizer at low bandwidth.
    pub low_types: Vec<FfLatterChStripEqType>,
    /// The gain of equalizer at low bandwidth between -20 and 20.
    pub low_gains: Vec<i16>,
    /// The frequency of equalizer at low bandwidth between 20 and 20000.
    pub low_freqs: Vec<u16>,
    /// The quality of equalizer at low bandwidth between 7 and 50, displayed by 1/10.
    pub low_qualities: Vec<u16>,
    /// The gain of equalizer at middle bandwidth between -20 and 20.
    pub middle_gains: Vec<i16>,
    /// The frequency of equalizer at middle bandwidth between 20 and 20000.
    pub middle_freqs: Vec<u16>,
    /// The quality of equalizer at middle bandwidth between 7 and 50, displayed by 1/10.
    pub middle_qualities: Vec<u16>,
    /// The type of equalizer at high bandwidth.
    pub high_types: Vec<FfLatterChStripEqType>,
    /// The gain of equalizer at high bandwidth between -20 and 20.
    pub high_gains: Vec<i16>,
    /// The frequency of equalizer at high bandwidth between 20 and 20000.
    pub high_freqs: Vec<u16>,
    /// The quality of equalizer at high bandwidth between 7 and 50, displayed by 1/10.
    pub high_qualities: Vec<u16>,
}

fn eq_state_to_cmds(state: &FfLatterEqState, ch_offset: u8) -> Vec<u32> {
    assert_eq!(state.low_types.len(), state.activates.len());
    assert_eq!(state.low_gains.len(), state.activates.len());
    assert_eq!(state.low_freqs.len(), state.activates.len());
    assert_eq!(state.low_qualities.len(), state.activates.len());
    assert_eq!(state.middle_gains.len(), state.activates.len());
    assert_eq!(state.middle_freqs.len(), state.activates.len());
    assert_eq!(state.middle_qualities.len(), state.activates.len());
    assert_eq!(state.high_types.len(), state.activates.len());
    assert_eq!(state.high_gains.len(), state.activates.len());
    assert_eq!(state.high_freqs.len(), state.activates.len());
    assert_eq!(state.high_qualities.len(), state.activates.len());

    let mut cmds = Vec::new();

    (0..state.activates.len())
        .for_each(|i| {
            let ch = ch_offset + i as u8;
            cmds.push(create_phys_port_cmd(ch, EQ_ACTIVATE_CMD, state.activates[i] as i16));
            cmds.push(create_phys_port_cmd(ch, EQ_LOW_TYPE_CMD, state.low_types[i] as i16));
            cmds.push(create_phys_port_cmd(ch, EQ_LOW_GAIN_CMD, state.low_gains[i] as i16));
            cmds.push(create_phys_port_cmd(ch, EQ_LOW_FREQ_CMD, state.low_freqs[i] as i16));
            cmds.push(create_phys_port_cmd(ch, EQ_LOW_QUALITY_CMD, state.low_qualities[i] as i16));
            cmds.push(create_phys_port_cmd(ch, EQ_MIDDLE_GAIN_CMD, state.middle_gains[i] as i16));
            cmds.push(create_phys_port_cmd(ch, EQ_MIDDLE_FREQ_CMD, state.middle_freqs[i] as i16));
            cmds.push(create_phys_port_cmd(ch, EQ_MIDDLE_QUALITY_CMD, state.middle_qualities[i] as i16));
            cmds.push(create_phys_port_cmd(ch, EQ_HIGH_TYPE_CMD, state.high_types[i] as i16));
            cmds.push(create_phys_port_cmd(ch, EQ_HIGH_GAIN_CMD, state.high_gains[i] as i16));
            cmds.push(create_phys_port_cmd(ch, EQ_HIGH_FREQ_CMD, state.high_freqs[i] as i16));
            cmds.push(create_phys_port_cmd(ch, EQ_HIGH_QUALITY_CMD, state.high_qualities[i] as i16));
        });

    cmds
}

/// The structure to represent state of dynamics in channel strip effect.
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct FfLatterDynState {
    /// Whether to activate dynamics.
    pub activates: Vec<bool>,
    /// The gain of dynamics between -300 and 300, displayed by 1/10.
    pub gains: Vec<i16>,
    /// The rise time of dynamics between 0 and 200 ms.
    pub attacks: Vec<u16>,
    /// The release time of dynamics between 100 and 999 ms.
    pub releases: Vec<u16>,
    /// The threshold of compressor between -600 and 0, displayed by 1/10.
    pub compressor_thresholds: Vec<i16>,
    /// The ratio of compressor between 10 and 100.
    pub compressor_ratios: Vec<u16>,
    /// The threshold of expander between -990 and -200, displayed by 1/10.
    pub expander_thresholds: Vec<i16>,
    /// The ratio of expander between 10 and 100.
    pub expander_ratios: Vec<u16>,
}

fn dyn_state_to_cmds(state: &FfLatterDynState, ch_offset: u8) -> Vec<u32> {
    assert_eq!(state.gains.len(), state.activates.len());
    assert_eq!(state.attacks.len(), state.activates.len());
    assert_eq!(state.releases.len(), state.activates.len());
    assert_eq!(state.compressor_thresholds.len(), state.activates.len());
    assert_eq!(state.compressor_ratios.len(), state.activates.len());
    assert_eq!(state.expander_thresholds.len(), state.activates.len());
    assert_eq!(state.expander_ratios.len(), state.activates.len());

    let mut cmds = Vec::new();

    (0..state.activates.len())
        .for_each(|i| {
            let ch = ch_offset + i as u8;
            cmds.push(create_phys_port_cmd(ch, DYN_ACTIVATE_CMD, state.activates[i] as i16));
            cmds.push(create_phys_port_cmd(ch, DYN_GAIN_CMD, state.gains[i] as i16));
            cmds.push(create_phys_port_cmd(ch, DYN_ATTACK_CMD, state.attacks[i] as i16));
            cmds.push(create_phys_port_cmd(ch, DYN_RELEASE_CMD, state.releases[i] as i16));
            cmds.push(create_phys_port_cmd(ch, DYN_COMPR_THRESHOLD_CMD, state.compressor_thresholds[i]));
            cmds.push(create_phys_port_cmd(ch, DYN_COMPR_RATIO_CMD, state.compressor_ratios[i] as i16));
            cmds.push(create_phys_port_cmd(ch, DYN_EXPANDER_THRESHOLD_CMD, state.expander_thresholds[i]));
            cmds.push(create_phys_port_cmd(ch, DYN_EXPANDER_RATIO_CMD, state.expander_ratios[i] as i16));
        });

    cmds
}

/// The structure to represent state of autolevel in channel strip effects.
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct FfLatterAutolevelState {
    /// Whether to activate auto level.
    pub activates: Vec<bool>,
    /// The maximum level of amplification between 0 and 180, displayed by 1/10 for dB.
    pub max_gains: Vec<u16>,
    /// The level of head room to decline signal peak between 30 and 120, displayed by 1/10 for dB.
    pub headrooms: Vec<u16>,
    /// The speed of level increase between 1 and 99, displayed by 1/10 for seconds.
    pub rise_times: Vec<u16>,
}

fn autolevel_state_to_cmds(state: &FfLatterAutolevelState, ch_offset: u8) -> Vec<u32> {
    assert_eq!(state.max_gains.len(), state.activates.len());
    assert_eq!(state.headrooms.len(), state.activates.len());
    assert_eq!(state.rise_times.len(), state.activates.len());

    let mut cmds = Vec::new();

    (0..state.activates.len())
        .for_each(|i| {
            let ch = ch_offset + i as u8;
            cmds.push(create_phys_port_cmd(ch, AUTOLEVEL_ACTIVATE_CMD, state.activates[i] as i16));
            cmds.push(create_phys_port_cmd(ch, AUTOLEVEL_MAX_GAIN_CMD, state.max_gains[i] as i16));
            cmds.push(create_phys_port_cmd(ch, AUTOLEVEL_HEADROOM_CMD, state.headrooms[i] as i16));
            cmds.push(create_phys_port_cmd(ch, AUTOLEVEL_RISE_TIME_CMD, state.rise_times[i] as i16));
        });

    cmds
}

/// The structure to represent state of channel strip effect.
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct FfLatterChStripState {
    pub hpf: FfLatterHpfState,
    pub eq: FfLatterEqState,
    pub dynamics: FfLatterDynState,
    pub autolevel: FfLatterAutolevelState,
}

/// The trait to represent channel strip protocol.
pub trait RmeFfLatterChStripOperation<T>: RmeFfLatterDspOperation {
    const CH_COUNT: usize;
    const CH_OFFSET: u8;

    const HPF_CUT_OFF_MIN: i32 = 20;
    const HPF_CUT_OFF_MAX: i32 = 500;
    const HPF_CUT_OFF_STEP: i32 = 1;

    const EQ_GAIN_MIN: i32 = -20;
    const EQ_GAIN_MAX: i32 = 20;
    const EQ_GAIN_STEP: i32 = 1;

    const EQ_FREQ_MIN: i32 = 20;
    const EQ_FREQ_MAX: i32 = 20000;
    const EQ_FREQ_STEP: i32 = 1;

    const EQ_QUALITY_MIN: i32 = 7;
    const EQ_QUALITY_MAX: i32 = 50;
    const EQ_QUALITY_STEP: i32 = 1;

    const DYN_GAIN_MIN: i32 = -300;
    const DYN_GAIN_MAX: i32 = 300;
    const DYN_GAIN_STEP: i32 = 1;

    const DYN_ATTACK_MIN: i32 = 0;
    const DYN_ATTACK_MAX: i32 = 200;
    const DYN_ATTACK_STEP: i32 = 1;

    const DYN_RELEASE_MIN: i32 = 100;
    const DYN_RELEASE_MAX: i32 = 999;
    const DYN_RELEASE_STEP: i32 = 1;

    const DYN_COMP_THRESHOLD_MIN: i32 = -600;
    const DYN_COMP_THRESHOLD_MAX: i32 = 0;
    const DYN_COMP_THRESHOLD_STEP: i32 = 1;

    const DYN_RATIO_MIN: i32 = 10;
    const DYN_RATIO_MAX: i32 = 100;
    const DYN_RATIO_STEP: i32 = 1;

    const DYN_EX_THRESHOLD_MIN: i32 = -999;
    const DYN_EX_THRESHOLD_MAX: i32 = -200;
    const DYN_EX_THRESHOLD_STEP: i32 = 1;

    const AUTOLEVEL_MAX_GAIN_MIN: i32 = 0;
    const AUTOLEVEL_MAX_GAIN_MAX: i32 = 180;
    const AUTOLEVEL_MAX_GAIN_STEP: i32 = 1;

    const AUTOLEVEL_HEAD_ROOM_MIN: i32 = 30;
    const AUTOLEVEL_HEAD_ROOM_MAX: i32 = 120;
    const AUTOLEVEL_HEAD_ROOM_STEP: i32 = 1;

    const AUTOLEVEL_RISE_TIME_MIN: i32 = 1;
    const AUTOLEVEL_RISE_TIME_MAX: i32 = 99;
    const AUTOLEVEL_RISE_TIME_STEP: i32 = 1;

    fn ch_strip(state: &FfLatterDspState) -> &FfLatterChStripState;
    fn ch_strip_mut(state: &mut FfLatterDspState) -> &mut FfLatterChStripState;

    fn init_ch_strip(
        req: &mut FwReq,
        node: &mut FwNode,
        state: &FfLatterDspState,
        timeout_ms: u32
    ) -> Result<(), Error> {
        let mut cmds = Vec::new();
        cmds.append(&mut hpf_state_to_cmds(&Self::ch_strip(state).hpf, Self::CH_OFFSET));
        cmds.append(&mut eq_state_to_cmds(&Self::ch_strip(state).eq, Self::CH_OFFSET));
        cmds.append(&mut dyn_state_to_cmds(&Self::ch_strip(state).dynamics, Self::CH_OFFSET));
        cmds.append(&mut autolevel_state_to_cmds(&Self::ch_strip(state).autolevel, Self::CH_OFFSET));

        cmds.iter().try_for_each(|&cmd| Self::write_dsp_cmd(req, node, cmd, timeout_ms))
    }

    fn write_ch_strip_hpf(
        req: &mut FwReq,
        node: &mut FwNode,
        state: &mut FfLatterDspState,
        hpf: FfLatterHpfState,
        timeout_ms: u32
    ) -> Result<(), Error> {
        let old = hpf_state_to_cmds(&Self::ch_strip(state).hpf, Self::CH_OFFSET);
        let new = hpf_state_to_cmds(&hpf, Self::CH_OFFSET);

        Self::write_dsp_cmds(req, node, &old, &new, timeout_ms)
            .map(|_| Self::ch_strip_mut(state).hpf = hpf)
    }

    fn write_ch_strip_eq(
        req: &mut FwReq,
        node: &mut FwNode,
        state: &mut FfLatterDspState,
        eq: FfLatterEqState,
        timeout_ms: u32
    ) -> Result<(), Error> {
        let old = eq_state_to_cmds(&Self::ch_strip(state).eq, Self::CH_OFFSET);
        let new = eq_state_to_cmds(&eq, Self::CH_OFFSET);

        Self::write_dsp_cmds(req, node, &old, &new, timeout_ms)
            .map(|_| Self::ch_strip_mut(state).eq = eq)
    }

    fn write_ch_strip_dynamics(
        req: &mut FwReq,
        node: &mut FwNode,
        state: &mut FfLatterDspState,
        dynamics: FfLatterDynState,
        timeout_ms: u32
    ) -> Result<(), Error> {
        let old = dyn_state_to_cmds(&Self::ch_strip(state).dynamics, Self::CH_OFFSET);
        let new = dyn_state_to_cmds(&dynamics, Self::CH_OFFSET);

        Self::write_dsp_cmds(req, node, &old, &new, timeout_ms)
            .map(|_| Self::ch_strip_mut(state).dynamics = dynamics)
    }

    fn write_ch_strip_autolevel(
        req: &mut FwReq,
        node: &mut FwNode,
        state: &mut FfLatterDspState,
        autolevel: FfLatterAutolevelState,
        timeout_ms: u32
    ) -> Result<(), Error> {
        let old = autolevel_state_to_cmds(&Self::ch_strip(state).autolevel, Self::CH_OFFSET);
        let new = autolevel_state_to_cmds(&autolevel, Self::CH_OFFSET);

        Self::write_dsp_cmds(req, node, &old, &new, timeout_ms)
            .map(|_| Self::ch_strip_mut(state).autolevel = autolevel)
    }
}

/// The structure to represent state of input channel strip effect.
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct FfLatterInputChStripState(pub FfLatterChStripState);

impl<O: RmeFfLatterDspOperation> RmeFfLatterChStripOperation<FfLatterInputChStripState> for O {
    const CH_COUNT: usize = Self::PHYS_INPUT_COUNT;
    const CH_OFFSET: u8 = 0x00;

    fn ch_strip(state: &FfLatterDspState) -> &FfLatterChStripState {
        &state.input_ch_strip.0
    }

    fn ch_strip_mut(state: &mut FfLatterDspState) -> &mut FfLatterChStripState {
        &mut state.input_ch_strip.0
    }
}

/// The structure to represent state of output channel strip effect.
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct FfLatterOutputChStripState(pub FfLatterChStripState);

impl<O: RmeFfLatterDspOperation> RmeFfLatterChStripOperation<FfLatterOutputChStripState> for O {
    const CH_COUNT: usize = Self::OUTPUT_COUNT;
    const CH_OFFSET: u8 = Self::PHYS_INPUT_COUNT as u8;

    fn ch_strip(state: &FfLatterDspState) -> &FfLatterChStripState {
        &state.output_ch_strip.0
    }

    fn ch_strip_mut(state: &mut FfLatterDspState) -> &mut FfLatterChStripState {
        &mut state.output_ch_strip.0
    }
}

/// The enumeration to represent type of reverb effect.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum FfLatterFxReverbType {
    SmallRoom,
    MediumRoom,
    LargeRoom,
    Walls,
    Shorty,
    Attack,
    Swagger,
    OldSchool,
    Echoistic,
    EightPlusNine,
    GrandWide,
    Thicker,
    Envelope,
    Gated,
    Space,
}

impl Default for FfLatterFxReverbType {
    fn default() -> Self {
        Self::SmallRoom
    }
}

impl From<FfLatterFxReverbType> for i16 {
    fn from(reverb_type: FfLatterFxReverbType) -> Self {
        match reverb_type {
            FfLatterFxReverbType::SmallRoom => 0x0000,
            FfLatterFxReverbType::MediumRoom => 0x0001,
            FfLatterFxReverbType::LargeRoom => 0x0002,
            FfLatterFxReverbType::Walls => 0x0003,
            FfLatterFxReverbType::Shorty => 0x0004,
            FfLatterFxReverbType::Attack => 0x0005,
            FfLatterFxReverbType::Swagger => 0x0006,
            FfLatterFxReverbType::OldSchool => 0x0007,
            FfLatterFxReverbType::Echoistic => 0x0008,
            FfLatterFxReverbType::EightPlusNine => 0x0009,
            FfLatterFxReverbType::GrandWide => 0x000a,
            FfLatterFxReverbType::Thicker => 0x000b,
            FfLatterFxReverbType::Envelope => 0x000c,
            FfLatterFxReverbType::Gated => 0x000d,
            FfLatterFxReverbType::Space => 0x000e,
        }
    }
}

/// The enumeration to represent type of echo effect.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum FfLatterFxEchoType {
    StereoEcho,
    StereoCross,
    PongEcho,
}

impl Default for FfLatterFxEchoType {
    fn default() -> Self {
        Self::StereoEcho
    }
}

impl From<FfLatterFxEchoType> for i16 {
    fn from(echo_type: FfLatterFxEchoType) -> Self {
        match echo_type {
            FfLatterFxEchoType::StereoEcho => 0x0000,
            FfLatterFxEchoType::StereoCross => 0x0001,
            FfLatterFxEchoType::PongEcho => 0x0002,
        }
    }
}

/// The enumeration to represent frequency of low pass filter for echo effect.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum FfLatterFxEchoLpfFreq {
    Off,
    H2000,
    H4000,
    H8000,
    H12000,
    H16000,
}

impl Default for FfLatterFxEchoLpfFreq {
    fn default() -> Self {
        Self::Off
    }
}

impl From<FfLatterFxEchoLpfFreq> for i16 {
    fn from(lpf_freq: FfLatterFxEchoLpfFreq) -> Self {
        match lpf_freq {
            FfLatterFxEchoLpfFreq::Off => 0x0000,
            FfLatterFxEchoLpfFreq::H2000 => 0x0005,
            FfLatterFxEchoLpfFreq::H4000 => 0x0004,
            FfLatterFxEchoLpfFreq::H8000 => 0x0003,
            FfLatterFxEchoLpfFreq::H12000 => 0x0002,
            FfLatterFxEchoLpfFreq::H16000 => 0x0001,
        }
    }
}

/// The structure to represent state of reverb in send effect.
#[derive(Default, Debug, Clone, Copy, Eq, PartialEq)]
pub struct FfLatterFxReverbState {
    /// Whether to activate reverb effect.
    pub activate: bool,
    /// The type of reverb effect.
    pub reverb_type: FfLatterFxReverbType,
    /// The pre-delay of reverb effect between 0 and 999.
    pub pre_delay: u16,
    /// The frequency of high pass filter before reverb generation between 20 and 500 Hz.
    pub pre_hpf: u16,
    /// The scale of room between 50 and 300, displayed by 1/10.
    pub room_scale: u16,
    /// The time for increase volume between 5 and 400 ms.
    pub attack: u16,
    /// The time for fixed volume between 5 and 400 ms.
    pub hold: u16,
    /// The time for volume decrease between 5 and 500 ms.
    pub release: u16,
    /// The frequency of low pass filter after reverb generation between 200 and 20000 Hz.
    pub post_lpf: u16,
    /// The time for volume drop between 1 and 49, displayed by 1/10 sec.
    pub time: u16,
    /// The frequency of treble dampling for reverb generation between 2000 and 20000 Hz.
    pub damping: u16,
    /// The level of softener between 0 and 100.
    pub smooth: u16,
    /// The level of output between -650 and 60, displayed by 1/10.
    pub volume: i16,
    /// The stereo width between 0(monaural) and 100(stereo).
    pub stereo_width: u16,
}

const FX_CH: u8 = 0x3c;

fn reverb_state_to_cmds(state: &FfLatterFxReverbState) -> Vec<u32> {
    let mut cmds = Vec::new();

    cmds.push(create_phys_port_cmd(FX_CH, FX_REVERB_ACTIVATE_CMD, state.activate as i16));
    cmds.push(create_phys_port_cmd(FX_CH, FX_REVERB_TYPE_CMD, i16::from(state.reverb_type)));
    cmds.push(create_phys_port_cmd(FX_CH, FX_REVERB_PRE_DELAY_CMD, state.pre_delay as i16));
    cmds.push(create_phys_port_cmd(FX_CH, FX_REVERB_PRE_HPF_FREQ_CMD, state.pre_hpf as i16));
    cmds.push(create_phys_port_cmd(FX_CH, FX_REVERB_ROOM_SCALE_CMD, state.room_scale as i16));
    cmds.push(create_phys_port_cmd(FX_CH, FX_REVERB_ATTACK_CMD, state.attack as i16));
    cmds.push(create_phys_port_cmd(FX_CH, FX_REVERB_HOLD_CMD, state.hold as i16));
    cmds.push(create_phys_port_cmd(FX_CH, FX_REVERB_RELEASE_CMD, state.release as i16));
    cmds.push(create_phys_port_cmd(FX_CH, FX_REVERB_POST_LPF_FREQ_CMD, state.post_lpf as i16));
    cmds.push(create_phys_port_cmd(FX_CH, FX_REVERB_TIME_CMD, state.time as i16));
    cmds.push(create_phys_port_cmd(FX_CH, FX_REVERB_DAMPING_FREQ_CMD, state.damping as i16));
    cmds.push(create_phys_port_cmd(FX_CH, FX_REVERB_SMOOTH_CMD, state.smooth as i16));
    cmds.push(create_phys_port_cmd(FX_CH, FX_REVERB_VOLUME_CMD, state.volume));
    cmds.push(create_phys_port_cmd(FX_CH, FX_REVERB_STEREO_WIDTH_CMD, state.stereo_width as i16));

    cmds
}

/// The structure to represent state of echo in send effect.
#[derive(Default, Debug, Clone, Copy, Eq, PartialEq)]
pub struct FfLatterFxEchoState {
    /// Whether to activate echo effect.
    pub activate: bool,
    /// The type of echo effect.
    pub echo_type: FfLatterFxEchoType,
    /// The time to delay for echo between 0 and 100.
    pub delay: u16,
    /// The level of feedback for further echo between 0 and 100.
    pub feedback: u16,
    /// The frequency of low pass filter.
    pub lpf: FfLatterFxEchoLpfFreq,
    /// The level of output between -650 and 0, displayed by 1/10.
    pub volume: i16,
    /// The stereo width between 0(monaural) and 100(stereo).
    pub stereo_width: u16,
}

fn echo_state_to_cmds(state: &FfLatterFxEchoState) -> Vec<u32> {
    let mut cmds = Vec::new();

    cmds.push(create_phys_port_cmd(FX_CH, FX_ECHO_ACTIVATE_CMD, state.activate as i16));
    cmds.push(create_phys_port_cmd(FX_CH, FX_ECHO_TYPE_CMD, i16::from(state.echo_type)));
    cmds.push(create_phys_port_cmd(FX_CH, FX_ECHO_DELAY_CMD, state.delay as i16));
    cmds.push(create_phys_port_cmd(FX_CH, FX_ECHO_FEEDBACK_CMD, state.feedback as i16));
    cmds.push(create_phys_port_cmd(FX_CH, FX_ECHO_LPF_FREQ_CMD, i16::from(state.lpf)));
    cmds.push(create_phys_port_cmd(FX_CH, FX_ECHO_VOLUME_CMD, state.volume));
    cmds.push(create_phys_port_cmd(FX_CH, FX_ECHO_STEREO_WIDTH_CMD, state.stereo_width as i16));

    cmds
}

/// The structure to represent state of send effects (reverb and echo).
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct FfLatterFxState {
    /// The gain of line inputs. Each value is between 0xfd76 (-65.0 dB) and 0x0000 (0.0 dB).
    pub line_input_gains: Vec<i16>,
    /// The gain of mic inputs. Each value is between 0xfd76 (-65.0 dB) and 0x0000 (0.0 dB).
    pub mic_input_gains: Vec<i16>,
    /// The gain of S/PDIF inputs. Each value is between 0xfd76 (-65.0 dB) and 0x0000 (0.0 dB).
    pub spdif_input_gains: Vec<i16>,
    /// The gain of ADAT inputs. Each value is between 0xfd76 (-65.0 dB) and 0x0000 (0.0 dB).
    pub adat_input_gains: Vec<i16>,
    /// The gain of stream inputs. Each value is between 0x0000 (-65.0 dB) and 0x8b5c (0.0 dB).
    pub stream_input_gains: Vec<u16>,
    /// The volume to line outputs. Each value is between 0xfd76 (-65.0 dB) and 0x0000 (0.0 dB).
    pub line_output_vols: Vec<i16>,
    /// The volume to hp outputs. Each value is between 0xfd76 (-65.0 dB) and 0x0000 (0.0 dB).
    pub hp_output_vols: Vec<i16>,
    /// The volume to S/PDIF outputs. Each value is between 0xfd76 (-65.0 dB) and 0x0000 (0.0 dB).
    pub spdif_output_vols: Vec<i16>,
    /// The volume to ADAT outputs. Each value is between 0xfd76 (-65.0 dB) and 0x0000 (0.0 dB).
    pub adat_output_vols: Vec<i16>,
    /// The state of reverb effect.
    pub reverb: FfLatterFxReverbState,
    /// The state of echo effect.
    pub echo: FfLatterFxEchoState,
}

const FX_MIXER_0: u16 = 0x1e;
const FX_MIXER_1: u16 = 0x1f;

/// The trait to represent mixer protocol.
pub trait RmeFfLatterFxOperation: RmeFfLatterDspOperation {
    const FX_PHYS_LEVEL_MIN: i32 = -650;
    const FX_PHYS_LEVEL_MAX: i32 = 0;
    const FX_PHYS_LEVEL_STEP: i32 = 1;

    const FX_VIRT_LEVEL_MIN: i32 = 0;
    const FX_VIRT_LEVEL_MAX: i32 = 35676;
    const FX_VIRT_LEVEL_STEP: i32 = 1;

    const REVERB_PRE_DELAY_MIN: i32 = 0;
    const REVERB_PRE_DELAY_MAX: i32 = 999;
    const REVERB_PRE_DELAY_STEP: i32 = 1;

    const REVERB_ATTACK_MIN: i32 = 5;
    const REVERB_ATTACK_MAX: i32 = 400;
    const REVERB_ATTACK_STEP: i32 = 1;

    const REVERB_HOLD_MIN: i32 = 5;
    const REVERB_HOLD_MAX: i32 = 400;
    const REVERB_HOLD_STEP: i32 = 1;

    const REVERB_RELEASE_MIN: i32 = 5;
    const REVERB_RELEASE_MAX: i32 = 500;
    const REVERB_RELEASE_STEP: i32 = 1;

    const REVERB_POST_LPF_FREQ_MIN: i32 = 200;
    const REVERB_POST_LPF_FREQ_MAX: i32 = 20000;
    const REVERB_POST_LPF_FREQ_STEP: i32 = 1;

    const REVERB_TIME_MIN: i32 = 1;
    const REVERB_TIME_MAX: i32 = 49;
    const REVERB_TIME_STEP: i32 = 1;

    const REVERB_DAMPING_MIN: i32 = 2000;
    const REVERB_DAMPING_MAX: i32 = 20000;
    const REVERB_DAMPING_STEP: i32 = 1;

    const REVERB_SMOOTH_MIN: i32 = 0;
    const REVERB_SMOOTH_MAX: i32 = 100;
    const REVERB_SMOOTH_STEP: i32 = 1;

    const REVERB_VOL_MIN: i32 = -650;
    const REVERB_VOL_MAX: i32 = 60;
    const REVERB_VOL_STEP: i32 = 1;

    const REVERB_STEREO_WIDTH_MIN: i32 = 0;
    const REVERB_STEREO_WIDTH_MAX: i32 = 100;
    const REVERB_STEREO_WIDTH_STEP: i32 = 1;

    const ECHO_DELAY_MIN: i32 = 0;
    const ECHO_DELAY_MAX: i32 = 100;
    const ECHO_DELAY_STEP: i32 = 1;

    const ECHO_FEEDBACK_MIN: i32 = 0;
    const ECHO_FEEDBACK_MAX: i32 = 100;
    const ECHO_FEEDBACK_STEP: i32 = 1;

    const ECHO_VOL_MIN: i32 = -650;
    const ECHO_VOL_MAX: i32 = 0;
    const ECHO_VOL_STEP: i32 = 1;

    const ECHO_STEREO_WIDTH_MIN: i32 = 0;
    const ECHO_STEREO_WIDTH_MAX: i32 = 100;
    const ECHO_STEREO_WIDTH_STEP: i32 = 1;

    fn init_fx(
        req: &mut FwReq,
        node: &mut FwNode,
        state: &FfLatterDspState,
        timeout_ms: u32
    ) -> Result<(), Error> {
        let mut cmds = Vec::new();
        cmds.append(&mut Self::fx_input_state_to_cmds(&state.fx));
        cmds.append(&mut Self::fx_output_state_to_cmds(&state.fx));
        cmds.append(&mut reverb_state_to_cmds(&state.fx.reverb));
        cmds.append(&mut echo_state_to_cmds(&state.fx.echo));

        cmds.iter()
            .try_for_each(|&cmd| Self::write_dsp_cmd(req, node, cmd, timeout_ms))
    }

    fn write_fx_input_gains(
        req: &mut FwReq,
        node: &mut FwNode,
        state: &mut FfLatterDspState,
        fx: FfLatterFxState,
        timeout_ms: u32
    ) -> Result<(), Error> {
        let old = Self::fx_input_state_to_cmds(&state.fx);
        let new = Self::fx_input_state_to_cmds(&fx);

        Self::write_dsp_cmds(req, node, &old, &new, timeout_ms)
            .map(|_| state.fx = fx)
    }

    fn write_fx_output_volumes(
        req: &mut FwReq,
        node: &mut FwNode,
        state: &mut FfLatterDspState,
        fx: FfLatterFxState,
        timeout_ms: u32
    ) -> Result<(), Error> {
        let old = Self::fx_output_state_to_cmds(&state.fx);
        let new = Self::fx_output_state_to_cmds(&fx);

        Self::write_dsp_cmds(req, node, &old, &new, timeout_ms)
            .map(|_| state.fx = fx)
    }

    fn write_fx_reverb(
        req: &mut FwReq,
        node: &mut FwNode,
        state: &mut FfLatterDspState,
        reverb: &FfLatterFxReverbState,
        timeout_ms: u32
    ) -> Result<(), Error> {
        let old = reverb_state_to_cmds(&state.fx.reverb);
        let new = reverb_state_to_cmds(reverb);

        Self::write_dsp_cmds(req, node, &old, &new, timeout_ms)
            .map(|_| state.fx.reverb = *reverb)
    }

    fn write_fx_echo(
        req: &mut FwReq,
        node: &mut FwNode,
        state: &mut FfLatterDspState,
        echo: &FfLatterFxEchoState,
        timeout_ms: u32
    ) -> Result<(), Error> {
        let old = echo_state_to_cmds(&state.fx.echo);
        let new = echo_state_to_cmds(echo);

        Self::write_dsp_cmds(req, node, &old, &new, timeout_ms)
            .map(|_| state.fx.echo = *echo)
    }

    fn fx_input_state_to_cmds(state: &FfLatterFxState) -> Vec<u32> {
        assert_eq!(state.line_input_gains.len(), Self::LINE_INPUT_COUNT);
        assert_eq!(state.mic_input_gains.len(), Self::MIC_INPUT_COUNT);
        assert_eq!(state.spdif_input_gains.len(), Self::SPDIF_INPUT_COUNT);
        assert_eq!(state.adat_input_gains.len(), Self::ADAT_INPUT_COUNT);
        assert_eq!(state.stream_input_gains.len(), Self::STREAM_INPUT_COUNT);

        let mut cmds = Vec::new();

        state.line_input_gains.iter()
            .chain(state.mic_input_gains.iter())
            .chain(state.spdif_input_gains.iter())
            .chain(state.adat_input_gains.iter())
            .enumerate()
            .for_each(|(i, &gain)| {
                let ch = i as u8;
                cmds.push(create_phys_port_cmd(ch, INPUT_TO_FX_CMD, gain));
            });

        state.stream_input_gains.iter()
            .enumerate()
            .for_each(|(i, &gain)| {
                cmds.push(create_virt_port_cmd(
                        Self::MIXER_STEP,
                        FX_MIXER_0,
                        Self::STREAM_OFFSET + i as u16,
                        gain
                    ));
                cmds.push(create_virt_port_cmd(
                        Self::MIXER_STEP,
                        FX_MIXER_1,
                        Self::STREAM_OFFSET + i as u16,
                        gain
                    ));
            });

        cmds
    }

    fn fx_output_state_to_cmds(state: &FfLatterFxState) -> Vec<u32> {
        assert_eq!(state.line_output_vols.len(), Self::LINE_OUTPUT_COUNT);
        assert_eq!(state.hp_output_vols.len(), Self::HP_OUTPUT_COUNT);
        assert_eq!(state.spdif_output_vols.len(), Self::SPDIF_OUTPUT_COUNT);
        assert_eq!(state.adat_output_vols.len(), Self::ADAT_OUTPUT_COUNT);

        let mut cmds = Vec::new();

        state.line_output_vols.iter()
            .chain(state.hp_output_vols.iter())
            .chain(state.spdif_output_vols.iter())
            .chain(state.adat_output_vols.iter())
            .enumerate()
            .for_each(|(i, &gain)| {
                let ch = (Self::PHYS_INPUT_COUNT + i) as u8;
                cmds.push(create_phys_port_cmd(ch, OUTPUT_FROM_FX_CMD, gain));
            });

        cmds
    }
}

impl<O: RmeFfLatterDspOperation> RmeFfLatterFxOperation for O {}
