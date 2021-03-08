// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol defined by RME GmbH for Fireface UCX.

use hinawa::FwReq;

use super::*;
use crate::*;

/// The structure to represent unique protocol for Fireface UCX.
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct FfUcxProtocol(FwReq);

impl AsRef<FwReq> for FfUcxProtocol {
    fn as_ref(&self) -> &FwReq {
        &self.0
    }
}

// For configuration register (0x'ffff'0000'0014).
const CFG_CLK_SRC_MASK: u32                         = 0x00000c00;
const   CFG_CLK_SRC_WORD_CLK_FLAG: u32              = 0x00000c00;
const   CFG_CLK_SRC_OPT_IFACE_FLAG: u32             = 0x00000800;
const   CFG_CLK_SRC_COAX_IFACE_FLAG: u32            = 0x00000400;
const   CFG_CLK_SRC_INTERNAL_FLAG: u32              = 0x00000000;
const CFG_SPDIF_OUT_TO_OPT_IFACE_MASK: u32          = 0x00000100;
const CFG_WORD_OUT_SINGLE_MASK: u32                 = 0x00000010;
const CFG_DSP_EFFECT_ON_INPUT_MASK: u32             = 0x00000040;
const CFG_WORD_INPUT_TERMINATE_MASK: u32        = 0x00000008;
const CFG_SPDIF_OUT_PRO_MASK: u32                   = 0x00000020;

/// The enumeration to represent source of sampling clock.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum FfUcxClkSrc {
    Internal,
    Coax,
    Opt,
    WordClk,
}

impl Default for FfUcxClkSrc {
    fn default() -> Self {
        Self::Internal
    }
}

impl FfUcxClkSrc {
    fn build(&self, quad: &mut u32) {
        *quad |= match self {
            Self::WordClk => CFG_CLK_SRC_WORD_CLK_FLAG,
            Self::Opt => CFG_CLK_SRC_OPT_IFACE_FLAG,
            Self::Coax => CFG_CLK_SRC_COAX_IFACE_FLAG,
            Self::Internal => CFG_CLK_SRC_INTERNAL_FLAG,
        };
    }

    fn parse(&mut self, quad: &u32) {
        match *quad & CFG_CLK_SRC_MASK {
            CFG_CLK_SRC_WORD_CLK_FLAG => Self::WordClk,
            CFG_CLK_SRC_OPT_IFACE_FLAG => Self::Opt,
            CFG_CLK_SRC_COAX_IFACE_FLAG => Self::Coax,
            CFG_CLK_SRC_INTERNAL_FLAG => Self::Internal,
            _ => unreachable!(),
        };
    }
}

/// The structure to represent unique protocol for Fireface UCX.
#[derive(Default, Debug, Clone, Copy, Eq, PartialEq)]
pub struct FfUcxConfig{
    /// The low offset of destination address for MIDI messages.
    midi_tx_low_offset: FfLatterMidiTxLowOffset,
    /// The source of sampling clock.
    pub clk_src: FfUcxClkSrc,
    /// The type of signal to optical output interface.
    pub opt_out_signal: OpticalOutputSignal,
    /// Whether to fix speed to single even if at double/quadruple rate.
    pub word_out_single: bool,
    /// Whether to enable DSP effect on inputs.
    pub effect_on_inputs: bool,
    /// Whether to terminate word clock input.
    pub word_in_terminate: bool,
    /// For signal format of S/PDIF output.
    pub spdif_out_format: SpdifFormat,
}

impl RmeFfLatterRegisterValueOperation for FfUcxConfig{
    fn build(&self, quad: &mut u32) {
        self.midi_tx_low_offset.build(quad);
        self.clk_src.build(quad);

        if self.opt_out_signal == OpticalOutputSignal::Spdif {
            *quad |= CFG_SPDIF_OUT_TO_OPT_IFACE_MASK;
        }

        if self.word_out_single {
            *quad |= CFG_WORD_OUT_SINGLE_MASK;
        }

        if self.effect_on_inputs {
            *quad |= CFG_DSP_EFFECT_ON_INPUT_MASK;
        }

        if self.word_in_terminate {
            *quad |= CFG_WORD_INPUT_TERMINATE_MASK;
        }

        if self.spdif_out_format == SpdifFormat::Professional {
            *quad |= CFG_SPDIF_OUT_PRO_MASK;
        }
    }

    fn parse(&mut self, quad: &u32) {
        self.midi_tx_low_offset.parse(quad);
        self.clk_src.parse(quad);

        self.opt_out_signal = if *quad & CFG_SPDIF_OUT_TO_OPT_IFACE_MASK > 0 {
            OpticalOutputSignal::Spdif
        } else {
            OpticalOutputSignal::Adat
        };

        self.word_out_single = *quad & CFG_WORD_OUT_SINGLE_MASK > 0;
        self.effect_on_inputs = *quad & CFG_DSP_EFFECT_ON_INPUT_MASK > 0;
        self.word_in_terminate = *quad & CFG_WORD_INPUT_TERMINATE_MASK > 0;
        self.spdif_out_format = if *quad & CFG_SPDIF_OUT_PRO_MASK > 0 {
            SpdifFormat::Professional
        } else {
            SpdifFormat::Consumer
        };
    }
}

impl<T: AsRef<FwNode>> RmeFfLatterConfigProtocol<T, FfUcxConfig> for FfUcxProtocol {}
