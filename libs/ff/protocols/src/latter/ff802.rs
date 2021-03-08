// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol defined by RME GmbH for Fireface 802.

use hinawa::FwReq;

use super::*;
use crate::*;

/// The structure to represent unique protocol for Fireface 802.
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct Ff802Protocol(FwReq);

impl AsRef<FwReq> for Ff802Protocol {
    fn as_ref(&self) -> &FwReq {
        &self.0
    }
}

// For configuration register (0x'ffff'0000'0014).
const CFG_CLK_SRC_MASK: u32                     = 0x00001c00;
const   CFG_CLK_SRC_ADAT_B_FLAG: u32            = 0x00001000;
const   CFG_CLK_SRC_ADAT_A_FLAG: u32            = 0x00000c00;
const   CFG_CLK_SRC_AESEBU_FLAG: u32            = 0x00000800;
const   CFG_CLK_SRC_WORD_CLK_FLAG: u32          = 0x00000400;
const   CFG_CLK_SRC_INTERNAL_FLAG: u32          = 0x00000000;
const CFG_AESEBU_INPUT_FROM_OPT_IFACE_MASK: u32 = 0x00000200;
const CFG_AESEBU_OUTPUT_TO_OPT_IFACE_MASK: u32  = 0x00000100;
const CFG_DSP_EFFECT_ON_INPUT_MASK: u32         = 0x00000040;
const CFG_AESEBU_OUT_PRO_MASK: u32              = 0x00000020;
const CFG_WORD_OUT_SINGLE_MASK: u32             = 0x00000010;

/// The enumeration to represent source of sampling clock.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Ff802ClkSrc {
    Internal,
    AdatA,
    AdatB,
    AesEbu,
    WordClk,
}

impl Default for Ff802ClkSrc {
    fn default() -> Self {
        Self::Internal
    }
}

impl Ff802ClkSrc {
    fn build(&self, quad: &mut u32) {
        *quad |= match self {
            Self::AdatB => CFG_CLK_SRC_ADAT_B_FLAG,
            Self::AdatA => CFG_CLK_SRC_ADAT_A_FLAG,
            Self::AesEbu => CFG_CLK_SRC_AESEBU_FLAG,
            Self::WordClk => CFG_CLK_SRC_WORD_CLK_FLAG,
            Self::Internal => CFG_CLK_SRC_INTERNAL_FLAG,
        };
    }

    fn parse(&mut self, quad: &u32) {
        match *quad & CFG_CLK_SRC_MASK {
            CFG_CLK_SRC_ADAT_B_FLAG => Self::AdatB,
            CFG_CLK_SRC_ADAT_A_FLAG => Self::AdatA,
            CFG_CLK_SRC_AESEBU_FLAG => Self::AesEbu,
            CFG_CLK_SRC_WORD_CLK_FLAG => Self::WordClk,
            CFG_CLK_SRC_INTERNAL_FLAG => Self::Internal,
            _ => unreachable!(),
        };
    }
}

/// The enumeration to represent interface of S/PDIF signal for Fireface 802.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Ff802SpdifIface {
    Xlr,
    Optical,
}

impl Default for Ff802SpdifIface {
    fn default() -> Self {
        Self::Xlr
    }
}

/// The structure to represent unique protocol for Fireface 802.
#[derive(Default, Debug, Clone, Copy, Eq, PartialEq)]
pub struct Ff802Config{
    /// The low offset of destination address for MIDI messages.
    midi_tx_low_offset: FfLatterMidiTxLowOffset,
    /// The source of sampling clock.
    pub clk_src: Ff802ClkSrc,
    /// The input interface of S/PDIF signal.
    pub spdif_in_iface: Ff802SpdifIface,
    /// The type of signal to optical output interface.
    pub opt_out_signal: OpticalOutputSignal,
    /// Whether to enable DSP effect on inputs.
    pub effect_on_inputs: bool,
    /// For signal format of S/PDIF output.
    pub spdif_out_format: SpdifFormat,
    /// Whether to fix speed to single even if at double/quadruple rate.
    pub word_out_single: bool,
}

impl RmeFfLatterRegisterValueOperation for Ff802Config{
    fn build(&self, quad: &mut u32) {
        self.midi_tx_low_offset.build(quad);
        self.clk_src.build(quad);

        if self.spdif_in_iface == Ff802SpdifIface::Optical {
            *quad |= CFG_AESEBU_INPUT_FROM_OPT_IFACE_MASK;
        }

        if self.opt_out_signal == OpticalOutputSignal::Spdif {
            *quad |= CFG_AESEBU_OUTPUT_TO_OPT_IFACE_MASK;
        }

        if self.effect_on_inputs {
            *quad |= CFG_DSP_EFFECT_ON_INPUT_MASK;
        }

        if self.spdif_out_format == SpdifFormat::Professional {
            *quad |= CFG_AESEBU_OUT_PRO_MASK;
        }

        if self.word_out_single {
            *quad |= CFG_WORD_OUT_SINGLE_MASK;
        }
    }

    fn parse(&mut self, quad: &u32) {
        self.midi_tx_low_offset.parse(quad);
        self.clk_src.parse(quad);

        self.spdif_in_iface = if *quad & CFG_AESEBU_INPUT_FROM_OPT_IFACE_MASK > 0 {
            Ff802SpdifIface::Optical
        } else {
            Ff802SpdifIface::Xlr
        };

        self.opt_out_signal = if *quad & CFG_AESEBU_OUTPUT_TO_OPT_IFACE_MASK > 0 {
            OpticalOutputSignal::Spdif
        } else {
            OpticalOutputSignal::Adat
        };

        self.effect_on_inputs = *quad & CFG_DSP_EFFECT_ON_INPUT_MASK > 0;
        self.spdif_out_format = if *quad & CFG_AESEBU_OUT_PRO_MASK > 0 {
            SpdifFormat::Professional
        } else {
            SpdifFormat::Consumer
        };
        self.word_out_single = *quad & CFG_WORD_OUT_SINGLE_MASK > 0;
    }
}

impl<T: AsRef<FwNode>> RmeFfLatterConfigProtocol<T, Ff802Config> for Ff802Protocol {}
