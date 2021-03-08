// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol defined by RME GmbH for latter models of Fireface series.

pub mod ff802;

use glib::Error;
use hinawa::{FwNode, FwTcode, FwReq, FwReqExtManual};

use crate::*;

const CFG_OFFSET: usize = 0xffff00000014;
const DSP_OFFSET: usize = 0xffff0000001c;

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
