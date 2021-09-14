// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol defined by RME GmbH for Fireface UCX.

use super::*;
use crate::*;

/// The structure to represent unique protocol for Fireface UCX.
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct FfUcxProtocol;

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

impl RmeFfLatterConfigOperation<FfUcxConfig> for FfUcxProtocol {}

// For status register (0x'ffff'0000'001c).
#[allow(dead_code)]
const STATUS_ACTIVE_CLK_RATE_MASK: u32              = 0x0f000000;
#[allow(dead_code)]
const STATUS_WORD_CLK_RATE_MASK: u32                = 0x00f00000;
#[allow(dead_code)]
const STATUS_OPT_IFACE_RATE_MASK: u32               = 0x000f0000;
#[allow(dead_code)]
const STATUS_COAX_IFACE_RATE_MASK: u32              = 0x0000f000;
const STATUS_ACTIVE_CLK_SRC_MASK: u32               = 0x00000e00;
const   STATUS_ACTIVE_CLK_SRC_INTERNAL_FLAG: u32    = 0x00000e00;
const   STATUS_ACTIVE_CLK_SRC_WORD_CLK_FLAG: u32    = 0x00000600;
const   STATUS_ACTIVE_CLK_SRC_OPT_IFACE_FLAG: u32   = 0x00000400;
const   STATUS_ACTIVE_CLK_SRC_COAX_IFACE_FLAG: u32  = 0x00000200;
const STATUS_OPT_OUT_IFACE_FOR_ADAT: u32            = 0x00000100;
const STATUS_SYNC_WORD_CLK_MASK: u32                = 0x00000040;
const STATUS_SYNC_OPT_IFACE_MASK: u32               = 0x00000020;
const STATUS_SYNC_COAX_IFACE_MASK: u32              = 0x00000010;
const STATUS_LOCK_WORD_CLK_MASK: u32                = 0x00000004;
const STATUS_LOCK_OPT_IFACE_MASK: u32               = 0x00000002;
const STATUS_LOCK_COAX_IFACE_MASK: u32              = 0x00000001;

/// The structure to represent lock status of UCX.
#[derive(Default, Debug, Clone, Copy, Eq, PartialEq)]
pub struct FfUcxExtLockStatus{
    pub word_clk: bool,
    pub opt_iface: bool,
    pub coax_iface: bool,
}

impl FfUcxExtLockStatus {
    fn build(&self, quad: &mut u32) {
        if self.word_clk {
            *quad |= STATUS_LOCK_WORD_CLK_MASK;
        }
        if self.opt_iface {
            *quad |= STATUS_LOCK_OPT_IFACE_MASK;
        }
        if self.coax_iface {
            *quad |= STATUS_LOCK_COAX_IFACE_MASK;
        }
    }

    fn parse(&mut self, quad: &u32) {
        self.word_clk = *quad & STATUS_LOCK_WORD_CLK_MASK > 0;
        self.opt_iface = *quad & STATUS_LOCK_OPT_IFACE_MASK > 0;
        self.coax_iface = *quad & STATUS_LOCK_COAX_IFACE_MASK > 0;
    }
}

/// The structure to represent sync status of UCX.
#[derive(Default, Debug, Clone, Copy, Eq, PartialEq)]
pub struct FfUcxExtSyncStatus{
    pub word_clk: bool,
    pub opt_iface: bool,
    pub coax_iface: bool,
}

impl FfUcxExtSyncStatus {
    fn build(&self, quad: &mut u32) {
        if self.word_clk {
            *quad |= STATUS_SYNC_WORD_CLK_MASK;
        }
        if self.opt_iface {
            *quad |= STATUS_SYNC_OPT_IFACE_MASK;
        }
        if self.coax_iface {
            *quad |= STATUS_SYNC_COAX_IFACE_MASK;
        }
    }

    fn parse(&mut self, quad: &u32) {
        self.word_clk = *quad & STATUS_SYNC_WORD_CLK_MASK > 0;
        self.opt_iface = *quad & STATUS_SYNC_OPT_IFACE_MASK > 0;
        self.coax_iface = *quad & STATUS_SYNC_COAX_IFACE_MASK > 0;
    }
}

/// The structure to represent sync status of UCX.
#[derive(Default, Debug, Clone, Copy, Eq, PartialEq)]
pub struct FfUcxExtRateStatus{
    pub word_clk: Option<ClkNominalRate>,
    pub opt_iface: Option<ClkNominalRate>,
    pub coax_iface: Option<ClkNominalRate>,
}

impl FfUcxExtRateStatus {
    fn build(&self, quad: &mut u32) {
        optional_val_from_clk_rate(&self.word_clk, quad, 20);
        optional_val_from_clk_rate(&self.opt_iface, quad, 16);
        optional_val_from_clk_rate(&self.coax_iface, quad, 12);
    }

    fn parse(&mut self, quad: &u32) {
        if *quad & (STATUS_SYNC_WORD_CLK_MASK | STATUS_LOCK_WORD_CLK_MASK) > 0 {
            optional_val_to_clk_rate(&mut self.word_clk, quad, 20);
        } else {
            self.word_clk = None;
        }
        if *quad & (STATUS_SYNC_OPT_IFACE_MASK | STATUS_LOCK_OPT_IFACE_MASK) > 0 {
            optional_val_to_clk_rate(&mut self.opt_iface, quad, 16);
        } else {
            self.opt_iface = None;
        }
        if *quad & (STATUS_SYNC_COAX_IFACE_MASK | STATUS_LOCK_COAX_IFACE_MASK) > 0 {
            optional_val_to_clk_rate(&mut self.coax_iface, quad, 12);
        } else {
            self.coax_iface = None;
        }
    }
}

/// The structure to represent status of UCX.
#[derive(Default, Debug, Clone, Copy, Eq, PartialEq)]
pub struct FfUcxStatus{
    pub ext_lock: FfUcxExtLockStatus,
    pub ext_sync: FfUcxExtSyncStatus,
    pub ext_rate: FfUcxExtRateStatus,
    pub opt_out_signal: OpticalOutputSignal,
    pub active_clk_src: FfUcxClkSrc,
    pub active_clk_rate: ClkNominalRate,
}

impl RmeFfLatterRegisterValueOperation for FfUcxStatus {
    fn build(&self, quad: &mut u32) {
        self.ext_lock.build(quad);
        self.ext_sync.build(quad);
        self.ext_rate.build(quad);

        if self.opt_out_signal == OpticalOutputSignal::Adat {
            *quad |= STATUS_OPT_OUT_IFACE_FOR_ADAT;
        }

        val_from_clk_rate(&self.active_clk_rate, quad, 24);

        let val = match self.active_clk_src {
            FfUcxClkSrc::Internal => STATUS_ACTIVE_CLK_SRC_INTERNAL_FLAG,
            FfUcxClkSrc::Coax => STATUS_ACTIVE_CLK_SRC_COAX_IFACE_FLAG,
            FfUcxClkSrc::Opt => STATUS_ACTIVE_CLK_SRC_OPT_IFACE_FLAG,
            FfUcxClkSrc::WordClk => STATUS_ACTIVE_CLK_SRC_WORD_CLK_FLAG,
        };
        *quad |= val;
    }

    fn parse(&mut self, quad: &u32) {
        self.ext_lock.parse(quad);
        self.ext_sync.parse(quad);
        self.ext_rate.parse(quad);

        self.opt_out_signal = if *quad & STATUS_OPT_OUT_IFACE_FOR_ADAT > 0 {
            OpticalOutputSignal::Adat
        } else {
            OpticalOutputSignal::Spdif
        };

        val_to_clk_rate(&mut self.active_clk_rate, quad, 24);

        self.active_clk_src = match *quad & STATUS_ACTIVE_CLK_SRC_MASK {
            STATUS_ACTIVE_CLK_SRC_INTERNAL_FLAG => FfUcxClkSrc::Internal,
            STATUS_ACTIVE_CLK_SRC_COAX_IFACE_FLAG => FfUcxClkSrc::Coax,
            STATUS_ACTIVE_CLK_SRC_OPT_IFACE_FLAG => FfUcxClkSrc::Opt,
            STATUS_ACTIVE_CLK_SRC_WORD_CLK_FLAG => FfUcxClkSrc::WordClk,
            _ => unreachable!(),
        };
    }
}

impl RmeFfLatterStatusOperation<FfUcxStatus> for FfUcxProtocol {}

const LINE_INPUT_COUNT: usize = 6;
const MIC_INPUT_COUNT: usize = 2;
const SPDIF_INPUT_COUNT: usize = 2;
const ADAT_INPUT_COUNT: usize = 8;
const STREAM_INPUT_COUNT: usize = 18;

const LINE_OUTPUT_COUNT: usize = 6;
const HP_OUTPUT_COUNT: usize = 2;
const SPDIF_OUTPUT_COUNT: usize = 2;
const ADAT_OUTPUT_COUNT: usize = 8;

impl RmeFfLatterMeterOperation for FfUcxProtocol {
    const LINE_INPUT_COUNT: usize = LINE_INPUT_COUNT;
    const MIC_INPUT_COUNT: usize = MIC_INPUT_COUNT;
    const SPDIF_INPUT_COUNT: usize = SPDIF_INPUT_COUNT;
    const ADAT_INPUT_COUNT: usize = ADAT_INPUT_COUNT;
    const STREAM_INPUT_COUNT: usize = STREAM_INPUT_COUNT;

    const LINE_OUTPUT_COUNT: usize = LINE_OUTPUT_COUNT;
    const HP_OUTPUT_COUNT: usize = HP_OUTPUT_COUNT;
    const SPDIF_OUTPUT_COUNT: usize = SPDIF_OUTPUT_COUNT;
    const ADAT_OUTPUT_COUNT: usize = ADAT_OUTPUT_COUNT;
}

impl RmeFfLatterDspOperation for FfUcxProtocol {
    const LINE_INPUT_COUNT: usize = LINE_INPUT_COUNT;
    const MIC_INPUT_COUNT: usize = MIC_INPUT_COUNT;
    const SPDIF_INPUT_COUNT: usize = SPDIF_INPUT_COUNT;
    const ADAT_INPUT_COUNT: usize = ADAT_INPUT_COUNT;
    const STREAM_INPUT_COUNT: usize = STREAM_INPUT_COUNT;

    const LINE_OUTPUT_COUNT: usize = LINE_OUTPUT_COUNT;
    const HP_OUTPUT_COUNT: usize = HP_OUTPUT_COUNT;
    const SPDIF_OUTPUT_COUNT: usize = SPDIF_OUTPUT_COUNT;
    const ADAT_OUTPUT_COUNT: usize = ADAT_OUTPUT_COUNT;
}
