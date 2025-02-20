// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol defined by RME GmbH for Fireface 802.

use super::*;

/// Unique protocol for 802.
#[derive(Default, Debug)]
pub struct Ff802Protocol;

// For configuration register (0x'ffff'0000'0014).
const CFG_CLK_SRC_MASK: u32 = 0x00001c00;
const CFG_CLK_SRC_ADAT_B_FLAG: u32 = 0x00001000;
const CFG_CLK_SRC_ADAT_A_FLAG: u32 = 0x00000c00;
const CFG_CLK_SRC_AESEBU_FLAG: u32 = 0x00000800;
const CFG_CLK_SRC_WORD_CLK_FLAG: u32 = 0x00000400;
const CFG_CLK_SRC_INTERNAL_FLAG: u32 = 0x00000000;
const CFG_AESEBU_INPUT_FROM_OPT_IFACE_MASK: u32 = 0x00000200;
const CFG_AESEBU_OUTPUT_TO_OPT_IFACE_MASK: u32 = 0x00000100;
const CFG_DSP_EFFECT_ON_INPUT_MASK: u32 = 0x00000040;
const CFG_AESEBU_OUT_PRO_MASK: u32 = 0x00000020;
const CFG_WORD_OUT_SINGLE_MASK: u32 = 0x00000010;

/// Signal source of sampling clock.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

fn serialize_clock_source(src: &Ff802ClkSrc, quad: &mut u32) {
    *quad &= !CFG_CLK_SRC_MASK;
    *quad |= match src {
        Ff802ClkSrc::AdatB => CFG_CLK_SRC_ADAT_B_FLAG,
        Ff802ClkSrc::AdatA => CFG_CLK_SRC_ADAT_A_FLAG,
        Ff802ClkSrc::AesEbu => CFG_CLK_SRC_AESEBU_FLAG,
        Ff802ClkSrc::WordClk => CFG_CLK_SRC_WORD_CLK_FLAG,
        Ff802ClkSrc::Internal => CFG_CLK_SRC_INTERNAL_FLAG,
    };
}

fn deserialize_clock_source(src: &mut Ff802ClkSrc, quad: &u32) {
    *src = match *quad & CFG_CLK_SRC_MASK {
        CFG_CLK_SRC_ADAT_B_FLAG => Ff802ClkSrc::AdatB,
        CFG_CLK_SRC_ADAT_A_FLAG => Ff802ClkSrc::AdatA,
        CFG_CLK_SRC_AESEBU_FLAG => Ff802ClkSrc::AesEbu,
        CFG_CLK_SRC_WORD_CLK_FLAG => Ff802ClkSrc::WordClk,
        CFG_CLK_SRC_INTERNAL_FLAG => Ff802ClkSrc::Internal,
        _ => unreachable!(),
    };
}

/// Digital interface of S/PDIF signal for 802.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Ff802SpdifIface {
    Xlr,
    Optical,
}

impl Default for Ff802SpdifIface {
    fn default() -> Self {
        Self::Xlr
    }
}

/// Unique protocol for 802.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ff802Config {
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

impl RmeFfOffsetParamsSerialize<Ff802Config> for Ff802Protocol {
    fn serialize_offsets(state: &Ff802Config) -> Vec<u8> {
        let mut quad = 0;

        serialize_midi_tx_low_offset(&state.midi_tx_low_offset, &mut quad);
        serialize_clock_source(&state.clk_src, &mut quad);

        quad &= !CFG_AESEBU_INPUT_FROM_OPT_IFACE_MASK;
        if state.spdif_in_iface == Ff802SpdifIface::Optical {
            quad |= CFG_AESEBU_INPUT_FROM_OPT_IFACE_MASK;
        }

        quad &= !CFG_AESEBU_OUTPUT_TO_OPT_IFACE_MASK;
        if state.opt_out_signal == OpticalOutputSignal::Spdif {
            quad |= CFG_AESEBU_OUTPUT_TO_OPT_IFACE_MASK;
        }

        quad &= !CFG_DSP_EFFECT_ON_INPUT_MASK;
        if state.effect_on_inputs {
            quad |= CFG_DSP_EFFECT_ON_INPUT_MASK;
        }

        quad &= !CFG_AESEBU_OUT_PRO_MASK;
        if state.spdif_out_format == SpdifFormat::Professional {
            quad |= CFG_AESEBU_OUT_PRO_MASK;
        }

        quad &= !CFG_WORD_OUT_SINGLE_MASK;
        if state.word_out_single {
            quad |= CFG_WORD_OUT_SINGLE_MASK;
        }

        quad.to_le_bytes().to_vec()
    }
}

impl RmeFfOffsetParamsDeserialize<Ff802Config> for Ff802Protocol {
    fn deserialize_offsets(state: &mut Ff802Config, raw: &[u8]) {
        assert!(raw.len() >= LATTER_CONFIG_SIZE);

        let mut r = [0; 4];
        r.copy_from_slice(&raw[..4]);
        let quad = u32::from_le_bytes(r);

        deserialize_midi_tx_low_offset(&mut state.midi_tx_low_offset, &quad);
        deserialize_clock_source(&mut state.clk_src, &quad);

        state.spdif_in_iface = if quad & CFG_AESEBU_INPUT_FROM_OPT_IFACE_MASK > 0 {
            Ff802SpdifIface::Optical
        } else {
            Ff802SpdifIface::Xlr
        };

        state.opt_out_signal = if quad & CFG_AESEBU_OUTPUT_TO_OPT_IFACE_MASK > 0 {
            OpticalOutputSignal::Spdif
        } else {
            OpticalOutputSignal::Adat
        };

        state.effect_on_inputs = quad & CFG_DSP_EFFECT_ON_INPUT_MASK > 0;
        state.spdif_out_format = if quad & CFG_AESEBU_OUT_PRO_MASK > 0 {
            SpdifFormat::Professional
        } else {
            SpdifFormat::Consumer
        };
        state.word_out_single = quad & CFG_WORD_OUT_SINGLE_MASK > 0;
    }
}

impl RmeFfWhollyUpdatableParamsOperation<Ff802Config> for Ff802Protocol {
    fn update_wholly(
        req: &mut FwReq,
        node: &mut FwNode,
        params: &Ff802Config,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        write_config::<Ff802Protocol, Ff802Config>(req, node, params, timeout_ms)
    }
}
// For status register (0x'ffff'0000'001c).
const STATUS_ACTIVE_CLK_RATE_MASK: u32 = 0xf0000000;
const STATUS_ADAT_B_RATE_MASK: u32 = 0x0f000000;
const STATUS_ADAT_A_RATE_MASK: u32 = 0x00f00000;
const STATUS_SPDIF_RATE_MASK: u32 = 0x000f0000;
const STATUS_WORD_CLK_RATE_MASK: u32 = 0x0000f000;
const STATUS_ACTIVE_CLK_SRC_MASK: u32 = 0x00000e00;
const STATUS_ACTIVE_CLK_SRC_INTERNAL_FLAG: u32 = 0x00000e00;
const STATUS_ACTIVE_CLK_SRC_ADAT_A_FLAG: u32 = 0x00000800;
const STATUS_ACTIVE_CLK_SRC_ADAT_B_FLAG: u32 = 0x00000600;
const STATUS_ACTIVE_CLK_SRC_AESEBU_FLAG: u32 = 0x00000400;
const STATUS_ACTIVE_CLK_SRC_WORD_CLK_FLAG: u32 = 0x00000200;
const STATUS_SYNC_MASK: u32 = 0x000000f0;
const STATUS_SYNC_ADAT_B_MASK: u32 = 0x00000080;
const STATUS_SYNC_ADAT_A_MASK: u32 = 0x00000040;
const STATUS_SYNC_SPDIF_MASK: u32 = 0x00000020;
const STATUS_SYNC_WORD_CLK_MASK: u32 = 0x00000010;
const STATUS_LOCK_MASK: u32 = 0x0000000f;
const STATUS_LOCK_ADAT_B_MASK: u32 = 0x00000008;
const STATUS_LOCK_ADAT_A_MASK: u32 = 0x00000004;
const STATUS_LOCK_SPDIF_MASK: u32 = 0x00000002;
const STATUS_LOCK_WORD_CLK_MASK: u32 = 0x00000001;

/// Lock status of 802.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ff802ExtLockStatus {
    pub word_clk: bool,
    pub spdif: bool,
    pub adat_b: bool,
    pub adat_a: bool,
}

fn serialize_external_lock_status(status: &Ff802ExtLockStatus, quad: &mut u32) {
    *quad &= !STATUS_LOCK_MASK;

    if status.word_clk {
        *quad |= STATUS_LOCK_WORD_CLK_MASK;
    }
    if status.spdif {
        *quad |= STATUS_LOCK_SPDIF_MASK;
    }
    if status.adat_b {
        *quad |= STATUS_LOCK_ADAT_B_MASK;
    }
    if status.adat_a {
        *quad |= STATUS_LOCK_ADAT_A_MASK;
    }
}

fn deserialize_external_lock_status(status: &mut Ff802ExtLockStatus, quad: &u32) {
    status.word_clk = *quad & STATUS_LOCK_WORD_CLK_MASK > 0;
    status.spdif = *quad & STATUS_LOCK_SPDIF_MASK > 0;
    status.adat_b = *quad & STATUS_LOCK_ADAT_B_MASK > 0;
    status.adat_a = *quad & STATUS_LOCK_ADAT_A_MASK > 0;
}

/// Sync status of 802.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ff802ExtSyncStatus {
    pub word_clk: bool,
    pub spdif: bool,
    pub adat_b: bool,
    pub adat_a: bool,
}

fn serialize_external_sync_status(status: &Ff802ExtSyncStatus, quad: &mut u32) {
    *quad &= !STATUS_SYNC_MASK;

    if status.word_clk {
        *quad |= STATUS_SYNC_WORD_CLK_MASK;
    }
    if status.spdif {
        *quad |= STATUS_SYNC_SPDIF_MASK;
    }
    if status.adat_b {
        *quad |= STATUS_SYNC_ADAT_B_MASK;
    }
    if status.adat_a {
        *quad |= STATUS_SYNC_ADAT_A_MASK;
    }
}

fn deserialize_external_sync_status(status: &mut Ff802ExtSyncStatus, quad: &u32) {
    status.word_clk = *quad & STATUS_SYNC_WORD_CLK_MASK > 0;
    status.spdif = *quad & STATUS_SYNC_SPDIF_MASK > 0;
    status.adat_b = *quad & STATUS_SYNC_ADAT_B_MASK > 0;
    status.adat_a = *quad & STATUS_SYNC_ADAT_A_MASK > 0;
}

/// Sync status of 802.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ff802ExtRateStatus {
    pub word_clk: Option<ClkNominalRate>,
    pub spdif: Option<ClkNominalRate>,
    pub adat_b: Option<ClkNominalRate>,
    pub adat_a: Option<ClkNominalRate>,
}

fn serialize_external_rate(
    rate: &Option<ClkNominalRate>,
    quad: &mut u32,
    shift: usize,
    lock_flag: u32,
) {
    serialize_clock_rate_optional(rate, quad, shift);

    // NOTE: The lock flag should stand.
    if rate.is_some() {
        *quad |= lock_flag;
    }
}

// NOTE: The call of function touches lock flags in the quad argument when detecting corresponding rate.
fn serialize_external_rate_status(status: &Ff802ExtRateStatus, quad: &mut u32) {
    *quad &= !(STATUS_WORD_CLK_RATE_MASK
        | STATUS_SPDIF_RATE_MASK
        | STATUS_ADAT_A_RATE_MASK
        | STATUS_ADAT_B_RATE_MASK);
    serialize_external_rate(&status.word_clk, quad, 12, STATUS_LOCK_WORD_CLK_MASK);
    serialize_external_rate(&status.spdif, quad, 16, STATUS_LOCK_SPDIF_MASK);
    serialize_external_rate(&status.adat_a, quad, 20, STATUS_LOCK_ADAT_A_MASK);
    serialize_external_rate(&status.adat_b, quad, 24, STATUS_LOCK_ADAT_B_MASK);
}

fn deserialize_external_rate_status(status: &mut Ff802ExtRateStatus, quad: &u32) {
    if *quad & (STATUS_SYNC_WORD_CLK_MASK | STATUS_LOCK_WORD_CLK_MASK) > 0 {
        if *quad & (STATUS_SYNC_WORD_CLK_MASK | STATUS_LOCK_WORD_CLK_MASK) > 0 {
            deserialize_clock_rate_optional(&mut status.word_clk, quad, 12);
        } else {
            status.word_clk = None;
        }
        if *quad & (STATUS_SYNC_SPDIF_MASK | STATUS_LOCK_SPDIF_MASK) > 0 {
            deserialize_clock_rate_optional(&mut status.spdif, quad, 16);
        } else {
            status.spdif = None;
        }
        if *quad & (STATUS_SYNC_ADAT_B_MASK | STATUS_LOCK_ADAT_B_MASK) > 0 {
            deserialize_clock_rate_optional(&mut status.adat_b, quad, 24);
        } else {
            status.adat_b = None;
        }
        if *quad & (STATUS_SYNC_ADAT_A_MASK | STATUS_LOCK_ADAT_A_MASK) > 0 {
            deserialize_clock_rate_optional(&mut status.adat_a, quad, 20);
        } else {
            status.adat_a = None;
        }
    }
}

/// Status of 802.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ff802Status {
    pub ext_lock: Ff802ExtLockStatus,
    pub ext_sync: Ff802ExtSyncStatus,
    pub ext_rate: Ff802ExtRateStatus,
    pub active_clk_src: Ff802ClkSrc,
    pub active_clk_rate: ClkNominalRate,
}

impl RmeFfOffsetParamsSerialize<Ff802Status> for Ff802Protocol {
    fn serialize_offsets(status: &Ff802Status) -> Vec<u8> {
        let mut quad = 0;

        serialize_external_lock_status(&status.ext_lock, &mut quad);
        serialize_external_sync_status(&status.ext_sync, &mut quad);
        serialize_external_rate_status(&status.ext_rate, &mut quad);

        quad &= !STATUS_ACTIVE_CLK_RATE_MASK;
        serialize_clock_rate(&status.active_clk_rate, &mut quad, 28);

        quad &= !STATUS_ACTIVE_CLK_SRC_MASK;
        let val = match status.active_clk_src {
            Ff802ClkSrc::Internal => STATUS_ACTIVE_CLK_SRC_INTERNAL_FLAG,
            Ff802ClkSrc::AdatA => STATUS_ACTIVE_CLK_SRC_ADAT_A_FLAG,
            Ff802ClkSrc::AdatB => STATUS_ACTIVE_CLK_SRC_ADAT_B_FLAG,
            Ff802ClkSrc::AesEbu => STATUS_ACTIVE_CLK_SRC_AESEBU_FLAG,
            Ff802ClkSrc::WordClk => STATUS_ACTIVE_CLK_SRC_WORD_CLK_FLAG,
        };
        quad |= val;

        quad.to_le_bytes().to_vec()
    }
}

impl RmeFfOffsetParamsDeserialize<Ff802Status> for Ff802Protocol {
    fn deserialize_offsets(status: &mut Ff802Status, raw: &[u8]) {
        assert!(raw.len() >= LATTER_STATUS_SIZE);

        let mut r = [0; 4];
        r.copy_from_slice(&raw[..4]);
        let quad = u32::from_le_bytes(r);

        deserialize_external_lock_status(&mut status.ext_lock, &quad);
        deserialize_external_sync_status(&mut status.ext_sync, &quad);
        deserialize_external_rate_status(&mut status.ext_rate, &quad);

        deserialize_clock_rate(&mut status.active_clk_rate, &quad, 28);

        status.active_clk_src = match quad & STATUS_ACTIVE_CLK_SRC_MASK {
            STATUS_ACTIVE_CLK_SRC_INTERNAL_FLAG => Ff802ClkSrc::Internal,
            STATUS_ACTIVE_CLK_SRC_ADAT_A_FLAG => Ff802ClkSrc::AdatA,
            STATUS_ACTIVE_CLK_SRC_ADAT_B_FLAG => Ff802ClkSrc::AdatB,
            STATUS_ACTIVE_CLK_SRC_AESEBU_FLAG => Ff802ClkSrc::AesEbu,
            STATUS_ACTIVE_CLK_SRC_WORD_CLK_FLAG => Ff802ClkSrc::WordClk,
            _ => unreachable!(),
        };
    }
}

impl RmeFfCacheableParamsOperation<Ff802Status> for Ff802Protocol {
    fn cache_wholly(
        req: &mut FwReq,
        node: &mut FwNode,
        status: &mut Ff802Status,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        read_status::<Ff802Protocol, Ff802Status>(req, node, status, timeout_ms)
    }
}

impl RmeFfLatterSpecification for Ff802Protocol {
    const LINE_INPUT_COUNT: usize = 8;
    const MIC_INPUT_COUNT: usize = 4;
    const SPDIF_INPUT_COUNT: usize = 2;
    const ADAT_INPUT_COUNT: usize = 16;
    const STREAM_INPUT_COUNT: usize = 30;
    const FX_RETURN_COUNT: usize = 2;

    const LINE_OUTPUT_COUNT: usize = 8;
    const HP_OUTPUT_COUNT: usize = 4;
    const SPDIF_OUTPUT_COUNT: usize = 2;
    const ADAT_OUTPUT_COUNT: usize = 16;
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn clock_source_serdes() {
        [
            Ff802ClkSrc::AdatB,
            Ff802ClkSrc::AdatA,
            Ff802ClkSrc::AesEbu,
            Ff802ClkSrc::WordClk,
            Ff802ClkSrc::Internal,
        ]
        .iter()
        .for_each(|orig| {
            let mut quad = 0;
            serialize_clock_source(&orig, &mut quad);
            let mut target = Ff802ClkSrc::default();
            deserialize_clock_source(&mut target, &quad);

            assert_eq!(&target, orig);
        });
    }

    #[test]
    fn config_serdes() {
        let orig = Ff802Config {
            midi_tx_low_offset: FfLatterMidiTxLowOffset::A0180,
            clk_src: Ff802ClkSrc::AesEbu,
            spdif_in_iface: Ff802SpdifIface::Optical,
            opt_out_signal: OpticalOutputSignal::Spdif,
            effect_on_inputs: true,
            spdif_out_format: SpdifFormat::Professional,
            word_out_single: true,
        };
        let quads = Ff802Protocol::serialize_offsets(&orig);
        let mut target = Ff802Config::default();
        Ff802Protocol::deserialize_offsets(&mut target, &quads);

        assert_eq!(target, orig);
    }

    #[test]
    fn external_lock_status_serdes() {
        let orig = Ff802ExtLockStatus {
            word_clk: true,
            spdif: true,
            adat_b: true,
            adat_a: true,
        };
        let mut quad = 0;
        serialize_external_lock_status(&orig, &mut quad);
        let mut target = Ff802ExtLockStatus::default();
        deserialize_external_lock_status(&mut target, &quad);

        assert_eq!(target, orig);
    }

    #[test]
    fn external_sync_status_serdes() {
        let orig = Ff802ExtSyncStatus {
            word_clk: true,
            spdif: true,
            adat_b: true,
            adat_a: true,
        };
        let mut quad = 0;
        serialize_external_sync_status(&orig, &mut quad);
        let mut target = Ff802ExtSyncStatus::default();
        deserialize_external_sync_status(&mut target, &quad);

        assert_eq!(target, orig);
    }

    #[test]
    fn external_rate_status_serdes() {
        let orig = Ff802ExtRateStatus {
            word_clk: Some(ClkNominalRate::R88200),
            spdif: Some(ClkNominalRate::R192000),
            adat_b: Some(ClkNominalRate::R44100),
            adat_a: None,
        };
        let mut quad = 0;
        serialize_external_rate_status(&orig, &mut quad);
        let mut target = Ff802ExtRateStatus::default();
        deserialize_external_rate_status(&mut target, &quad);

        assert_eq!(target, orig);
    }

    #[test]
    fn status_serdes() {
        let orig = Ff802Status {
            ext_lock: Ff802ExtLockStatus {
                word_clk: true,
                spdif: false,
                adat_b: true,
                adat_a: false,
            },
            ext_sync: Ff802ExtSyncStatus {
                word_clk: true,
                spdif: false,
                adat_b: false,
                adat_a: false,
            },
            ext_rate: Ff802ExtRateStatus {
                word_clk: Some(ClkNominalRate::R88200),
                spdif: None,
                adat_b: Some(ClkNominalRate::R44100),
                adat_a: None,
            },
            active_clk_src: Ff802ClkSrc::AdatA,
            active_clk_rate: ClkNominalRate::R96000,
        };
        let raw = Ff802Protocol::serialize_offsets(&orig);
        let mut target = Ff802Status::default();
        Ff802Protocol::deserialize_offsets(&mut target, &raw);

        assert_eq!(target, orig);
    }
}
