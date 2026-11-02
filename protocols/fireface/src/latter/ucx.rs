// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol defined by RME GmbH for Fireface UCX.

use super::*;

/// Unique protocol for UCX.
#[derive(Default, Debug)]
pub struct FfUcxProtocol;

// For configuration register (0x'ffff'0000'0014).
const CFG_CLK_SRC_MASK: u32 = 0x00000c00;
const CFG_CLK_SRC_WORD_CLK_FLAG: u32 = 0x00000c00;
const CFG_CLK_SRC_OPT_IFACE_FLAG: u32 = 0x00000800;
const CFG_CLK_SRC_COAX_IFACE_FLAG: u32 = 0x00000400;
const CFG_CLK_SRC_INTERNAL_FLAG: u32 = 0x00000000;
const CFG_SPDIF_OUT_TO_OPT_IFACE_MASK: u32 = 0x00000100;
const CFG_WORD_OUT_SINGLE_MASK: u32 = 0x00000010;
const CFG_DSP_EFFECT_ON_INPUT_MASK: u32 = 0x00000040;
const CFG_WORD_INPUT_TERMINATE_MASK: u32 = 0x00000008;
const CFG_SPDIF_OUT_PRO_MASK: u32 = 0x00000020;

/// Signal source of sampling clock.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

fn serialize_clock_source(src: &FfUcxClkSrc, quad: &mut u32) {
    *quad |= match src {
        FfUcxClkSrc::WordClk => CFG_CLK_SRC_WORD_CLK_FLAG,
        FfUcxClkSrc::Opt => CFG_CLK_SRC_OPT_IFACE_FLAG,
        FfUcxClkSrc::Coax => CFG_CLK_SRC_COAX_IFACE_FLAG,
        FfUcxClkSrc::Internal => CFG_CLK_SRC_INTERNAL_FLAG,
    };
}

fn deserialize_clock_source(src: &mut FfUcxClkSrc, quad: &u32) {
    *src = match *quad & CFG_CLK_SRC_MASK {
        CFG_CLK_SRC_WORD_CLK_FLAG => FfUcxClkSrc::WordClk,
        CFG_CLK_SRC_OPT_IFACE_FLAG => FfUcxClkSrc::Opt,
        CFG_CLK_SRC_COAX_IFACE_FLAG => FfUcxClkSrc::Coax,
        CFG_CLK_SRC_INTERNAL_FLAG => FfUcxClkSrc::Internal,
        _ => unreachable!(),
    };
}

/// Configuration for UCX.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct FfUcxConfig {
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

impl RmeFfOffsetParamsSerialize<FfUcxConfig> for FfUcxProtocol {
    fn serialize_offsets(state: &FfUcxConfig) -> Vec<u8> {
        let mut quad = 0;

        serialize_midi_tx_low_offset(&state.midi_tx_low_offset, &mut quad);
        serialize_clock_source(&state.clk_src, &mut quad);

        if state.opt_out_signal == OpticalOutputSignal::Spdif {
            quad |= CFG_SPDIF_OUT_TO_OPT_IFACE_MASK;
        }

        if state.word_out_single {
            quad |= CFG_WORD_OUT_SINGLE_MASK;
        }

        if state.effect_on_inputs {
            quad |= CFG_DSP_EFFECT_ON_INPUT_MASK;
        }

        if state.word_in_terminate {
            quad |= CFG_WORD_INPUT_TERMINATE_MASK;
        }

        if state.spdif_out_format == SpdifFormat::Professional {
            quad |= CFG_SPDIF_OUT_PRO_MASK;
        }

        quad.to_le_bytes().to_vec()
    }
}

impl RmeFfOffsetParamsDeserialize<FfUcxConfig> for FfUcxProtocol {
    fn deserialize_offsets(state: &mut FfUcxConfig, raw: &[u8]) {
        assert!(raw.len() >= LATTER_CONFIG_SIZE);

        let mut r = [0; 4];
        r.copy_from_slice(&raw[..4]);
        let quad = u32::from_le_bytes(r);

        deserialize_midi_tx_low_offset(&mut state.midi_tx_low_offset, &quad);
        deserialize_clock_source(&mut state.clk_src, &quad);

        state.opt_out_signal = if quad & CFG_SPDIF_OUT_TO_OPT_IFACE_MASK > 0 {
            OpticalOutputSignal::Spdif
        } else {
            OpticalOutputSignal::Adat
        };

        state.word_out_single = quad & CFG_WORD_OUT_SINGLE_MASK > 0;
        state.effect_on_inputs = quad & CFG_DSP_EFFECT_ON_INPUT_MASK > 0;
        state.word_in_terminate = quad & CFG_WORD_INPUT_TERMINATE_MASK > 0;
        state.spdif_out_format = if quad & CFG_SPDIF_OUT_PRO_MASK > 0 {
            SpdifFormat::Professional
        } else {
            SpdifFormat::Consumer
        };
    }
}

impl RmeFfWhollyUpdatableParamsOperation<FfUcxConfig> for FfUcxProtocol {
    fn update_wholly(
        req: &mut FwReq,
        node: &mut FwNode,
        params: &FfUcxConfig,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        write_config::<FfUcxProtocol, FfUcxConfig>(req, node, params, timeout_ms)
    }
}

// For status register (0x'ffff'0000'001c).
#[allow(dead_code)]
const STATUS_ACTIVE_CLK_RATE_MASK: u32 = 0x0f000000;
#[allow(dead_code)]
const STATUS_WORD_CLK_RATE_MASK: u32 = 0x00f00000;
#[allow(dead_code)]
const STATUS_OPT_IFACE_RATE_MASK: u32 = 0x000f0000;
#[allow(dead_code)]
const STATUS_COAX_IFACE_RATE_MASK: u32 = 0x0000f000;
const STATUS_ACTIVE_CLK_SRC_MASK: u32 = 0x00000e00;
const STATUS_ACTIVE_CLK_SRC_INTERNAL_FLAG: u32 = 0x00000e00;
const STATUS_ACTIVE_CLK_SRC_WORD_CLK_FLAG: u32 = 0x00000600;
const STATUS_ACTIVE_CLK_SRC_OPT_IFACE_FLAG: u32 = 0x00000400;
const STATUS_ACTIVE_CLK_SRC_COAX_IFACE_FLAG: u32 = 0x00000200;
const STATUS_OPT_OUT_IFACE_FOR_ADAT: u32 = 0x00000100;
const STATUS_SYNC_MASK: u32 = 0x00000070;
const STATUS_SYNC_WORD_CLK_MASK: u32 = 0x00000040;
const STATUS_SYNC_OPT_IFACE_MASK: u32 = 0x00000020;
const STATUS_SYNC_COAX_IFACE_MASK: u32 = 0x00000010;
const STATUS_LOCK_MASK: u32 = 0x00000007;
const STATUS_LOCK_WORD_CLK_MASK: u32 = 0x00000004;
const STATUS_LOCK_OPT_IFACE_MASK: u32 = 0x00000002;
const STATUS_LOCK_COAX_IFACE_MASK: u32 = 0x00000001;

/// Lock status of UCX.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct FfUcxExtLockStatus {
    pub word_clk: bool,
    pub opt_iface: bool,
    pub coax_iface: bool,
}

fn serialize_external_lock_status(status: &FfUcxExtLockStatus, quad: &mut u32) {
    *quad &= !STATUS_LOCK_MASK;
    if status.word_clk {
        *quad |= STATUS_LOCK_WORD_CLK_MASK;
    }
    if status.opt_iface {
        *quad |= STATUS_LOCK_OPT_IFACE_MASK;
    }
    if status.coax_iface {
        *quad |= STATUS_LOCK_COAX_IFACE_MASK;
    }
}

fn deserialize_external_lock_status(status: &mut FfUcxExtLockStatus, quad: &u32) {
    status.word_clk = *quad & STATUS_LOCK_WORD_CLK_MASK > 0;
    status.opt_iface = *quad & STATUS_LOCK_OPT_IFACE_MASK > 0;
    status.coax_iface = *quad & STATUS_LOCK_COAX_IFACE_MASK > 0;
}

/// Sync status of UCX.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct FfUcxExtSyncStatus {
    pub word_clk: bool,
    pub opt_iface: bool,
    pub coax_iface: bool,
}

fn serialize_external_sync_status(status: &FfUcxExtSyncStatus, quad: &mut u32) {
    *quad &= !STATUS_SYNC_MASK;
    if status.word_clk {
        *quad |= STATUS_SYNC_WORD_CLK_MASK;
    }
    if status.opt_iface {
        *quad |= STATUS_SYNC_OPT_IFACE_MASK;
    }
    if status.coax_iface {
        *quad |= STATUS_SYNC_COAX_IFACE_MASK;
    }
}

fn deserialize_external_sync_status(status: &mut FfUcxExtSyncStatus, quad: &u32) {
    status.word_clk = *quad & STATUS_SYNC_WORD_CLK_MASK > 0;
    status.opt_iface = *quad & STATUS_SYNC_OPT_IFACE_MASK > 0;
    status.coax_iface = *quad & STATUS_SYNC_COAX_IFACE_MASK > 0;
}

/// Sync status of UCX.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct FfUcxExtRateStatus {
    pub word_clk: Option<ClkNominalRate>,
    pub opt_iface: Option<ClkNominalRate>,
    pub coax_iface: Option<ClkNominalRate>,
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

fn serialize_external_rate_status(status: &FfUcxExtRateStatus, quad: &mut u32) {
    *quad &=
        !(STATUS_WORD_CLK_RATE_MASK | STATUS_OPT_IFACE_RATE_MASK | STATUS_COAX_IFACE_RATE_MASK);
    serialize_external_rate(&status.word_clk, quad, 20, STATUS_LOCK_WORD_CLK_MASK);
    serialize_external_rate(&status.opt_iface, quad, 16, STATUS_LOCK_OPT_IFACE_MASK);
    serialize_external_rate(&status.coax_iface, quad, 12, STATUS_LOCK_COAX_IFACE_MASK);
}

fn deserialize_external_rate_status(status: &mut FfUcxExtRateStatus, quad: &u32) {
    if *quad & (STATUS_SYNC_WORD_CLK_MASK | STATUS_LOCK_WORD_CLK_MASK) > 0 {
        deserialize_clock_rate_optional(&mut status.word_clk, quad, 20);
    } else {
        status.word_clk = None;
    }
    if *quad & (STATUS_SYNC_OPT_IFACE_MASK | STATUS_LOCK_OPT_IFACE_MASK) > 0 {
        deserialize_clock_rate_optional(&mut status.opt_iface, quad, 16);
    } else {
        status.opt_iface = None;
    }
    if *quad & (STATUS_SYNC_COAX_IFACE_MASK | STATUS_LOCK_COAX_IFACE_MASK) > 0 {
        deserialize_clock_rate_optional(&mut status.coax_iface, quad, 12);
    } else {
        status.coax_iface = None;
    }
}

/// Status of UCX.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct FfUcxStatus {
    pub ext_lock: FfUcxExtLockStatus,
    pub ext_sync: FfUcxExtSyncStatus,
    pub ext_rate: FfUcxExtRateStatus,
    pub opt_out_signal: OpticalOutputSignal,
    pub active_clk_src: FfUcxClkSrc,
    pub active_clk_rate: ClkNominalRate,
}

impl RmeFfOffsetParamsSerialize<FfUcxStatus> for FfUcxProtocol {
    fn serialize_offsets(state: &FfUcxStatus) -> Vec<u8> {
        let mut quad = 0;

        serialize_external_lock_status(&state.ext_lock, &mut quad);
        serialize_external_sync_status(&state.ext_sync, &mut quad);
        serialize_external_rate_status(&state.ext_rate, &mut quad);

        quad &= !STATUS_OPT_OUT_IFACE_FOR_ADAT;
        if state.opt_out_signal == OpticalOutputSignal::Adat {
            quad |= STATUS_OPT_OUT_IFACE_FOR_ADAT;
        }

        serialize_clock_rate(&state.active_clk_rate, &mut quad, 24);

        quad &= !STATUS_ACTIVE_CLK_SRC_MASK;
        let val = match state.active_clk_src {
            FfUcxClkSrc::Internal => STATUS_ACTIVE_CLK_SRC_INTERNAL_FLAG,
            FfUcxClkSrc::Coax => STATUS_ACTIVE_CLK_SRC_COAX_IFACE_FLAG,
            FfUcxClkSrc::Opt => STATUS_ACTIVE_CLK_SRC_OPT_IFACE_FLAG,
            FfUcxClkSrc::WordClk => STATUS_ACTIVE_CLK_SRC_WORD_CLK_FLAG,
        };
        quad |= val;

        quad.to_le_bytes().to_vec()
    }
}

impl RmeFfOffsetParamsDeserialize<FfUcxStatus> for FfUcxProtocol {
    fn deserialize_offsets(state: &mut FfUcxStatus, raw: &[u8]) {
        assert!(raw.len() >= LATTER_STATUS_SIZE);

        let mut r = [0; 4];
        r.copy_from_slice(&raw[..4]);
        let quad = u32::from_le_bytes(r);

        deserialize_external_lock_status(&mut state.ext_lock, &quad);
        deserialize_external_sync_status(&mut state.ext_sync, &quad);
        deserialize_external_rate_status(&mut state.ext_rate, &quad);

        state.opt_out_signal = if quad & STATUS_OPT_OUT_IFACE_FOR_ADAT > 0 {
            OpticalOutputSignal::Adat
        } else {
            OpticalOutputSignal::Spdif
        };

        deserialize_clock_rate(&mut state.active_clk_rate, &quad, 24);

        state.active_clk_src = match quad & STATUS_ACTIVE_CLK_SRC_MASK {
            STATUS_ACTIVE_CLK_SRC_INTERNAL_FLAG => FfUcxClkSrc::Internal,
            STATUS_ACTIVE_CLK_SRC_COAX_IFACE_FLAG => FfUcxClkSrc::Coax,
            STATUS_ACTIVE_CLK_SRC_OPT_IFACE_FLAG => FfUcxClkSrc::Opt,
            STATUS_ACTIVE_CLK_SRC_WORD_CLK_FLAG => FfUcxClkSrc::WordClk,
            _ => unreachable!(),
        };
    }
}

impl RmeFfCacheableParamsOperation<FfUcxStatus> for FfUcxProtocol {
    fn cache_wholly(
        req: &mut FwReq,
        node: &mut FwNode,
        status: &mut FfUcxStatus,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        read_status::<FfUcxProtocol, FfUcxStatus>(req, node, status, timeout_ms)
    }
}

impl RmeFfLatterSpecification for FfUcxProtocol {
    const LINE_INPUT_COUNT: usize = 6;
    const MIC_INPUT_COUNT: usize = 2;
    // The microphone preamps (front inputs 1-2) are at channel 0-1; confirmed by the phantom
    // powering command (0x08) to channel 1 lighting the 48V LED of input 2.
    const LINE_INPUT_OFFSET: usize = 2;
    const MIC_INPUT_OFFSET: usize = 0;
    const SPDIF_INPUT_COUNT: usize = 2;
    const ADAT_INPUT_COUNT: usize = 8;
    const STREAM_INPUT_COUNT: usize = 18;
    const FX_RETURN_COUNT: usize = 2;

    const LINE_OUTPUT_COUNT: usize = 6;
    const HP_OUTPUT_COUNT: usize = 2;
    const SPDIF_OUTPUT_COUNT: usize = 2;
    const ADAT_OUTPUT_COUNT: usize = 8;

    // 20 octlets, 20 quadlets for levels, the previous tag, then 3 quadlets for the tag.
    const METER_CHUNK_SIZE: usize = 256;
    const METER_LEVEL_OFFSET: usize = 160;
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn clock_source_serdes() {
        [
            FfUcxClkSrc::WordClk,
            FfUcxClkSrc::Opt,
            FfUcxClkSrc::Coax,
            FfUcxClkSrc::Internal,
        ]
        .iter()
        .for_each(|orig| {
            let mut quad = 0;
            serialize_clock_source(&orig, &mut quad);
            let mut target = FfUcxClkSrc::default();
            deserialize_clock_source(&mut target, &quad);

            assert_eq!(&target, orig);
        });
    }

    #[test]
    fn config_serdes() {
        let orig = FfUcxConfig {
            midi_tx_low_offset: FfLatterMidiTxLowOffset::A0180,
            clk_src: FfUcxClkSrc::Opt,
            opt_out_signal: OpticalOutputSignal::Spdif,
            word_out_single: true,
            effect_on_inputs: true,
            word_in_terminate: true,
            spdif_out_format: SpdifFormat::Professional,
        };
        let quads = FfUcxProtocol::serialize_offsets(&orig);
        let mut target = FfUcxConfig::default();
        FfUcxProtocol::deserialize_offsets(&mut target, &quads);

        assert_eq!(target, orig);
    }

    #[test]
    fn external_lock_status_serdes() {
        let orig = FfUcxExtLockStatus {
            word_clk: true,
            opt_iface: true,
            coax_iface: true,
        };
        let mut quad = 0;
        serialize_external_lock_status(&orig, &mut quad);
        let mut target = FfUcxExtLockStatus::default();
        deserialize_external_lock_status(&mut target, &quad);

        assert_eq!(target, orig);
    }

    #[test]
    fn external_sync_status_serdes() {
        let orig = FfUcxExtSyncStatus {
            word_clk: true,
            opt_iface: true,
            coax_iface: true,
        };
        let mut quad = 0;
        serialize_external_sync_status(&orig, &mut quad);
        let mut target = FfUcxExtSyncStatus::default();
        deserialize_external_sync_status(&mut target, &quad);

        assert_eq!(target, orig);
    }

    #[test]
    fn external_rate_status_serdes() {
        let orig = FfUcxExtRateStatus {
            word_clk: Some(ClkNominalRate::R88200),
            opt_iface: Some(ClkNominalRate::R192000),
            coax_iface: Some(ClkNominalRate::R44100),
        };
        let mut quad = 0;
        serialize_external_rate_status(&orig, &mut quad);
        let mut target = FfUcxExtRateStatus::default();
        deserialize_external_rate_status(&mut target, &quad);

        assert_eq!(target, orig);
    }

    #[test]
    fn status_serdes() {
        let orig = FfUcxStatus {
            ext_lock: FfUcxExtLockStatus {
                word_clk: true,
                opt_iface: false,
                coax_iface: true,
            },
            ext_sync: FfUcxExtSyncStatus {
                word_clk: false,
                opt_iface: false,
                coax_iface: true,
            },
            ext_rate: FfUcxExtRateStatus {
                word_clk: Some(ClkNominalRate::R176400),
                opt_iface: None,
                coax_iface: Some(ClkNominalRate::R48000),
            },
            opt_out_signal: OpticalOutputSignal::Spdif,
            active_clk_src: FfUcxClkSrc::Opt,
            active_clk_rate: ClkNominalRate::R88200,
        };
        let raw = FfUcxProtocol::serialize_offsets(&orig);
        let mut target = FfUcxStatus::default();
        FfUcxProtocol::deserialize_offsets(&mut target, &raw);

        assert_eq!(target, orig);
    }

    #[test]
    fn meter_chunk_layout() {
        // Captured from the UCX: 20 octlets, 20 quadlets of levels, the previous tag, then
        // three quadlets for the current tag.
        let chunk = |tag: u32, prev: u32, levels: &[(usize, u32)]| {
            let mut raw = vec![0u8; FfUcxProtocol::METER_CHUNK_SIZE];
            levels.iter().for_each(|&(i, v)| {
                let pos = 160 + i * 4;
                raw[pos..(pos + 4)].copy_from_slice(&v.to_le_bytes());
            });
            raw[240..244].copy_from_slice(&prev.to_le_bytes());
            (61..64).for_each(|q| raw[(q * 4)..(q * 4 + 4)].copy_from_slice(&tag.to_le_bytes()));
            raw
        };

        let mut state = FfUcxProtocol::create_meter_state();

        let raw = chunk(0x33333333, 0x22222222, &[(0, 0x00379790), (1, 0x00379740)]);
        FfUcxProtocol::deserialize_offsets(&mut state, &raw);
        assert_eq!(state.stream_inputs[0], 0x00379790);
        assert_eq!(state.stream_inputs[1], 0x00379740);
        assert_eq!(state.stream_inputs[2], 0);

        // Microphone inputs 1-2 come first, then line inputs 3-8.
        let raw = chunk(0x66666666, 0x55555555, &[(1, 0x01234560), (6, 0x00000da0), (17, 0x7fffff0)]);
        FfUcxProtocol::deserialize_offsets(&mut state, &raw);
        assert_eq!(state.mic_inputs[1], 0x01234560);
        assert_eq!(state.line_inputs[4], 0x00000da0);
        assert_eq!(state.adat_inputs[7], 0x07fffff0);

        let raw = chunk(0x11111111, 0x22666666, &[(0, 0x76d0), (6, 0x100), (17, 0x200)]);
        FfUcxProtocol::deserialize_offsets(&mut state, &raw);
        assert_eq!(state.line_outputs[0], 0x76d0);
        assert_eq!(state.hp_outputs[0], 0x100);
        assert_eq!(state.adat_outputs[7], 0x200);
    }

    #[test]
    fn mixer_source_channel_layout() {
        // Stream and fx return sources are at fixed channel offsets in each mixer, independent
        // of the number of physical inputs (18 in UCX).
        let mut state = FfUcxProtocol::create_mixer_parameters();
        state.0[1].adat_gains[7] = 0x9001;
        state.0[1].fx_return_gains[0] = 0x9002;
        state.0[1].fx_return_gains[1] = 0x9003;
        state.0[1].stream_gains[0] = 0x9004;
        state.0[1].stream_gains[17] = 0x9005;

        let cmds = FfUcxProtocol::serialize_commands(&state);
        assert_eq!(
            cmds.len(),
            FfUcxProtocol::OUTPUT_COUNT * (18 + 2 + 18),
        );

        let addr_of = |coef: u32| {
            let cmd = cmds.iter().find(|&&cmd| cmd & 0xffff == coef).unwrap();
            ((cmd & 0x3fffffff) >> 16) as u16
        };
        let base = FfUcxProtocol::MIXER_STEP;
        assert_eq!(addr_of(0x9001), base + 17);
        assert_eq!(addr_of(0x9002), base + 0x1e);
        assert_eq!(addr_of(0x9003), base + 0x1f);
        assert_eq!(addr_of(0x9004), base + 0x20);
        assert_eq!(addr_of(0x9005), base + 0x20 + 17);
    }

    #[test]
    fn input_channel_order() {
        // The microphone preamps are at channel 0-1, line inputs at 2-7, digital inputs from 8.
        let phys_ch = |cmd: u32| ((cmd & 0x7fffffff) >> 24) as u8;
        let phys_cmd = |cmd: u32| ((cmd >> 16) & 0xff) as u8;

        let mut state = FfUcxProtocol::create_input_parameters();
        state.mic_powers[1] = true;
        state.line_levels[0] = LatterInNominalLevel::Professional;
        let cmds = FfUcxProtocol::serialize_commands(&state);
        let set = |cmd_code: u8, ch: u8| {
            cmds.iter()
                .find(|&&c| phys_cmd(c) == cmd_code && phys_ch(c) == ch)
                .map(|&c| c & 0xffff)
        };
        // Phantom powering for input 2 at channel 1, line level of input 3 at channel 2.
        assert_eq!(set(0x08, 1), Some(1));
        assert_eq!(set(0x08, 0), Some(0));
        assert_eq!(set(0x08, 2), Some(1));
        assert_eq!(set(0x08, 6), Some(0));
        assert_eq!(set(0x08, 7), Some(0));

        let mut params = FfUcxProtocol::create_fx_sources_parameters();
        params.mic_input_gains[1] = 0x101;
        params.line_input_gains[0] = 0x102;
        params.spdif_input_gains[0] = 0x103;
        let cmds = FfUcxProtocol::serialize_commands(&params);
        let ch_of = |coef: u32| phys_ch(*cmds.iter().find(|&&c| c & 0xffff == coef).unwrap());
        assert_eq!(ch_of(0x101), 1);
        assert_eq!(ch_of(0x102), 2);
        assert_eq!(ch_of(0x103), 8);

        let mut mixer = FfUcxProtocol::create_mixer_parameters();
        mixer.0[0].mic_gains[1] = 0x9011;
        mixer.0[0].line_gains[5] = 0x9012;
        mixer.0[0].spdif_gains[0] = 0x9013;
        let cmds = FfUcxProtocol::serialize_commands(&mixer);
        let addr_of = |coef: u32| {
            let cmd = cmds.iter().find(|&&cmd| cmd & 0xffff == coef).unwrap();
            ((cmd & 0x3fffffff) >> 16) as u16
        };
        assert_eq!(addr_of(0x9011), 1);
        assert_eq!(addr_of(0x9012), 7);
        assert_eq!(addr_of(0x9013), 8);
    }
}
