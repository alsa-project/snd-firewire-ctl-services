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

// For status register (0x'ffff'0000'001c).
#[allow(dead_code)]
const STATUS_ACTIVE_CLK_RATE_MASK: u32              = 0xf0000000;
#[allow(dead_code)]
const STATUS_ADAT_B_RATE_MASK: u32                  = 0x0f000000;
#[allow(dead_code)]
const STATUS_ADAT_A_RATE_MASK: u32                  = 0x00f00000;
#[allow(dead_code)]
const STATUS_SPDIF_RATE_MASK: u32                   = 0x000f0000;
#[allow(dead_code)]
const STATUS_WORD_CLK_RATE_MASK: u32                = 0x0000f000;
const STATUS_ACTIVE_CLK_SRC_MASK: u32               = 0x00000e00;
const   STATUS_ACTIVE_CLK_SRC_INTERNAL_FLAG: u32    = 0x00000e00;
const   STATUS_ACTIVE_CLK_SRC_ADAT_A_FLAG: u32      = 0x00000800;
const   STATUS_ACTIVE_CLK_SRC_ADAT_B_FLAG: u32      = 0x00000600;
const   STATUS_ACTIVE_CLK_SRC_AESEBU_FLAG: u32      = 0x00000400;
const   STATUS_ACTIVE_CLK_SRC_WORD_CLK_FLAG: u32    = 0x00000200;
const STATUS_SYNC_ADAT_B_MASK: u32                  = 0x00000080;
const STATUS_SYNC_ADAT_A_MASK: u32                  = 0x00000040;
const STATUS_SYNC_SPDIF_MASK: u32                   = 0x00000020;
const STATUS_SYNC_WORD_CLK_MASK: u32                = 0x00000010;
const STATUS_LOCK_ADAT_B_MASK: u32                  = 0x00000008;
const STATUS_LOCK_ADAT_A_MASK: u32                  = 0x00000004;
const STATUS_LOCK_SPDIF_MASK: u32                   = 0x00000002;
const STATUS_LOCK_WORD_CLK_MASK: u32                = 0x00000001;

/// The structure to represent lock status of 802.
#[derive(Default, Debug, Clone, Copy, Eq, PartialEq)]
pub struct Ff802ExtLockStatus{
    pub word_clk: bool,
    pub spdif: bool,
    pub adat_b: bool,
    pub adat_a: bool,
}

impl Ff802ExtLockStatus {
    fn build(&self, quad: &mut u32) {
        if self.word_clk {
            *quad |= STATUS_LOCK_WORD_CLK_MASK;
        }
        if self.spdif {
            *quad |= STATUS_LOCK_SPDIF_MASK;
        }
        if self.adat_b {
            *quad |= STATUS_LOCK_ADAT_B_MASK;
        }
        if self.adat_a {
            *quad |= STATUS_LOCK_ADAT_A_MASK;
        }
    }

    fn parse(&mut self, quad: &u32) {
        self.word_clk = *quad & STATUS_LOCK_WORD_CLK_MASK > 0;
        self.spdif = *quad & STATUS_LOCK_SPDIF_MASK > 0;
        self.adat_b = *quad & STATUS_LOCK_ADAT_B_MASK > 0;
        self.adat_a = *quad & STATUS_LOCK_ADAT_A_MASK > 0;
    }
}

/// The structure to represent sync status of 802.
#[derive(Default, Debug, Clone, Copy, Eq, PartialEq)]
pub struct Ff802ExtSyncStatus{
    pub word_clk: bool,
    pub spdif: bool,
    pub adat_b: bool,
    pub adat_a: bool,
}

impl Ff802ExtSyncStatus {
    fn build(&self, quad: &mut u32) {
        if self.word_clk {
            *quad |= STATUS_SYNC_WORD_CLK_MASK;
        }
        if self.spdif {
            *quad |= STATUS_SYNC_SPDIF_MASK;
        }
        if self.adat_b {
            *quad |= STATUS_SYNC_ADAT_B_MASK;
        }
        if self.adat_a {
            *quad |= STATUS_SYNC_ADAT_A_MASK;
        }
    }

    fn parse(&mut self, quad: &u32) {
        self.word_clk = *quad & STATUS_SYNC_WORD_CLK_MASK > 0;
        self.spdif = *quad & STATUS_SYNC_SPDIF_MASK > 0;
        self.adat_b = *quad & STATUS_SYNC_ADAT_B_MASK > 0;
        self.adat_a = *quad & STATUS_SYNC_ADAT_A_MASK > 0;
    }
}

/// The structure to represent sync status of 802.
#[derive(Default, Debug, Clone, Copy, Eq, PartialEq)]
pub struct Ff802ExtRateStatus{
    pub word_clk: Option<ClkNominalRate>,
    pub spdif: Option<ClkNominalRate>,
    pub adat_b: Option<ClkNominalRate>,
    pub adat_a: Option<ClkNominalRate>,
}

impl Ff802ExtRateStatus {
    fn build(&self, quad: &mut u32) {
        optional_val_from_clk_rate(&self.word_clk, quad, 12);
        optional_val_from_clk_rate(&self.spdif, quad, 16);
        optional_val_from_clk_rate(&self.adat_b, quad, 24);
        optional_val_from_clk_rate(&self.adat_a, quad, 20);
    }

    fn parse(&mut self, quad: &u32) {
        if *quad & (STATUS_SYNC_WORD_CLK_MASK | STATUS_LOCK_WORD_CLK_MASK) > 0 {
            optional_val_to_clk_rate(&mut self.word_clk, quad, 12);
        } else {
            self.word_clk = None;
        }
        if *quad & (STATUS_SYNC_SPDIF_MASK | STATUS_LOCK_SPDIF_MASK) > 0 {
            optional_val_to_clk_rate(&mut self.spdif, quad, 16);
        } else {
            self.spdif = None;
        }
        if *quad & (STATUS_SYNC_ADAT_B_MASK | STATUS_LOCK_ADAT_B_MASK) > 0 {
            optional_val_to_clk_rate(&mut self.adat_b, quad, 24);
        } else {
            self.adat_b = None;
        }
        if *quad & (STATUS_SYNC_ADAT_A_MASK | STATUS_LOCK_ADAT_A_MASK) > 0 {
            optional_val_to_clk_rate(&mut self.adat_a, quad, 20);
        } else {
            self.adat_a = None;
        }
    }
}

/// The structure to represent status of 802.
#[derive(Default, Debug, Clone, Copy, Eq, PartialEq)]
pub struct Ff802Status{
    pub ext_lock: Ff802ExtLockStatus,
    pub ext_sync: Ff802ExtSyncStatus,
    pub ext_rate: Ff802ExtRateStatus,
    pub active_clk_src: Ff802ClkSrc,
    pub active_clk_rate: ClkNominalRate,
}

impl RmeFfLatterRegisterValueOperation for Ff802Status {
    fn build(&self, quad: &mut u32) {
        self.ext_lock.build(quad);
        self.ext_sync.build(quad);
        self.ext_rate.build(quad);

        val_from_clk_rate(&self.active_clk_rate, quad, 28);

        let val = match self.active_clk_src {
            Ff802ClkSrc::Internal => STATUS_ACTIVE_CLK_SRC_INTERNAL_FLAG,
            Ff802ClkSrc::AdatA => STATUS_ACTIVE_CLK_SRC_ADAT_A_FLAG,
            Ff802ClkSrc::AdatB => STATUS_ACTIVE_CLK_SRC_ADAT_B_FLAG,
            Ff802ClkSrc::AesEbu => STATUS_ACTIVE_CLK_SRC_AESEBU_FLAG,
            Ff802ClkSrc::WordClk => STATUS_ACTIVE_CLK_SRC_WORD_CLK_FLAG,
        };
        *quad |= val;
    }

    fn parse(&mut self, quad: &u32) {
        self.ext_lock.parse(quad);
        self.ext_sync.parse(quad);
        self.ext_rate.parse(quad);

        val_to_clk_rate(&mut self.active_clk_rate, quad, 28);

        self.active_clk_src = match *quad & STATUS_ACTIVE_CLK_SRC_MASK {
            STATUS_ACTIVE_CLK_SRC_INTERNAL_FLAG => Ff802ClkSrc::Internal,
            STATUS_ACTIVE_CLK_SRC_ADAT_A_FLAG => Ff802ClkSrc::AdatA,
            STATUS_ACTIVE_CLK_SRC_ADAT_B_FLAG => Ff802ClkSrc::AdatB,
            STATUS_ACTIVE_CLK_SRC_AESEBU_FLAG => Ff802ClkSrc::AesEbu,
            STATUS_ACTIVE_CLK_SRC_WORD_CLK_FLAG => Ff802ClkSrc::WordClk,
            _ => unreachable!(),
        };
    }
}

impl<T: AsRef<FwNode>> RmeFfLatterStatusProtocol<T, Ff802Status> for Ff802Protocol {}

const LINE_INPUT_COUNT: usize = 8;
const MIC_INPUT_COUNT: usize = 4;
const SPDIF_INPUT_COUNT: usize = 2;
const ADAT_INPUT_COUNT: usize = 16;
const STREAM_INPUT_COUNT: usize = 30;

const LINE_OUTPUT_COUNT: usize = 8;
const HP_OUTPUT_COUNT: usize = 4;
const SPDIF_OUTPUT_COUNT: usize = 2;
const ADAT_OUTPUT_COUNT: usize = 16;

/// The structure to represent state of meter.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Ff802MeterState(FfLatterMeterState);

impl AsRef<FfLatterMeterState> for Ff802MeterState {
    fn as_ref(&self) -> &FfLatterMeterState {
        &self.0
    }
}

impl AsMut<FfLatterMeterState> for Ff802MeterState {
    fn as_mut(&mut self) -> &mut FfLatterMeterState {
        &mut self.0
    }
}

impl RmeFfLatterMeterSpec for Ff802MeterState {
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

impl Default for Ff802MeterState {
    fn default() -> Self {
        Self(Self::create_meter_state())
    }
}

impl<T: AsRef<FwNode>> RmeFfLatterMeterProtocol<T, Ff802MeterState> for Ff802Protocol {}

/// The structure to represent state of DSP.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Ff802DspState(FfLatterDspState);

impl AsRef<FfLatterDspState> for Ff802DspState {
    fn as_ref(&self) -> &FfLatterDspState {
        &self.0
    }
}

impl AsMut<FfLatterDspState> for Ff802DspState {
    fn as_mut(&mut self) -> &mut FfLatterDspState {
        &mut self.0
    }
}

impl RmeFfLatterDspSpec for Ff802DspState {
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

impl Default for Ff802DspState {
    fn default() -> Self {
        Self(Self::create_dsp_state())
    }
}

impl<T: AsRef<FwNode>> RmeFfLatterDspProtocol<T, Ff802DspState> for Ff802Protocol {}
