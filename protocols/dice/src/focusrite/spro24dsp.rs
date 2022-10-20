// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol specific to Focusrite Saffire Pro 24 DSP.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by Focusrite for Saffire Pro 24 DSP.
//!
//! ## Diagram of internal signal flow for Saffire Pro 24 DSP.
//!
//! I note that optical input interface is available exclusively for ADAT input and S/PDIF input.
//!
//! ```text
//!                          ++===========++
//!                          || equalizer ||
//! ch-strip-input-0/1 ----> ||     &     || -> ch-strip-output-0/1
//!                          ||compressor ||
//!                          ++===========++
//!
//!                          ++===========++
//! reverb-input-0/1 ------> ||  reverb   || -> reverb-output-0/1
//!                          ++===========++
//!
//!                          ++===========++
//! mixer-input-0/1 -------> ||           || -> mixer-output-0/1
//! mixer-input-2/3 -------> ||           || -> mixer-output-2/3
//! mixer-input-4/5 -------> ||           || -> mixer-output-4/5
//! mixer-input-6/7 -------> ||   mixer   || -> mixer-output-6/7
//! mixer-input-8/9 -------> ||           || -> mixer-output-8/9
//! mixer-input-10/11 -----> ||  18 x 16  || -> mixer-output-10/11
//! mixer-input-12/13 -----> ||           || -> mixer-output-12/13
//! mixer-input-14/15 -----> ||           || -> mixer-output-14/15
//! mixer-input-16/17 -----> ||           ||
//!                          ++===========++
//!
//!                          ++===========++
//! mic-input-0/1 ---------> ||           ||
//! line-input-0/1 --------> ||           ||
//! spdif-coax-input-0/1 --> ||           ||
//! adat-input-0/1 --------> ||           ||
//! adat-input-2/3 --------> ||           || -> mixer-input-0/1
//! adat-input-4/5 --------> ||  mixer    || -> mixer-input-2/3
//! adat-input-6/7 --------> ||  input    || -> mixer-input-4/5
//! spdif-opt-input-0/1 ---> ||  router   || -> mixer-input-6/7
//!                          ||           || -> mixer-input-8/9
//! stream-input-0/1 ------> ||   x 18    || -> mixer-input-10/11
//! stream-input-2/3 ------> ||           || -> mixer-input-12/13
//! stream-input-4/5 ------> ||           || -> mixer-input-14/15
//! stream-input-6/7 ------> ||           || -> mixer-input-16/17
//!                          ||           ||
//! ch-strip-output-0/1 ---> ||           ||
//! reverb-output-0/1 -----> ||           ||
//!                          ++===========++
//!
//!                          ++===========++
//! mic-input-0/1 ---------> ||           ||
//! line-input-0/1 --------> ||           ||
//! spdif-coax-input-0/1 --> ||           ||
//! adat-input-0/1 --------> ||           ||
//! adat-input-2/3 --------> ||           ||
//! adat-input-4/5 --------> ||           ||
//! adat-input-6/7 --------> ||           || -> stream-output-0/1
//! spdif-opt-input-0/1 ---> ||  stream   || -> stream-output-2/3
//!                          ||  capture  || -> stream-output-4/5
//! mixer-output-0/1 ------> ||  router   || -> stream-output-6/7
//! mixer-output-2/3 ------> ||           || -> stream-output-8/9
//! mixer-output-4/5 ------> ||   x 16    || -> stream-output-10/11
//! mixer-output-6/7 ------> ||           || -> stream-output-12/13
//! mixer-output-8/9 ------> ||           || -> stream-output-14/15
//! mixer-output-10/11 ----> ||           ||
//! mixer-output-12/13 ----> ||           ||
//! mixer-output-14/15 ----> ||           ||
//!                          ||           ||
//! ch-strip-0/1 ----------> ||           ||
//! reverb-0/1 ------------> ||           ||
//!                          ++===========++
//!
//!                          ++===========++
//! mic-input-0/1 ---------> ||           ||
//! line-input-0/1 --------> ||           ||
//! spdif-coax-input-0/1 --> ||           ||
//! adat-input-0/1 --------> ||           ||
//! adat-input-2/3 --------> ||           ||
//! adat-input-4/5 --------> ||           ||
//! adat-input-6/7 --------> ||           ||
//! spdif-opt-input-0/1 ---> ||           ||
//!                          ||           ||
//! stream-input-0/1 ------> ||           || -> analog-output-0/1
//! stream-input-2/3 ------> ||  physical || -> analog-output-2/3
//! stream-input-4/5 ------> ||  output   || -> analog-output-4/5
//! stream-input-6/7 ------> ||  router   || -> spdif-output-0/1
//!                          ||           ||
//! mixer-output-0/1 ------> ||   x 12    || -> ch-strip-input-0/1
//! mixer-output-2/3 ------> ||           || -> reverb-input-0/1
//! mixer-output-4/5 ------> ||           ||
//! mixer-output-6/7 ------> ||           ||
//! mixer-output-8/9 ------> ||           ||
//! mixer-output-10/11 ----> ||           ||
//! mixer-output-12/13 ----> ||           ||
//! mixer-output-14/15 ----> ||           ||
//!                          ||           ||
//! ch-strip-output-0/1 ---> ||           ||
//! reverb-output-0/1 -----> ||           ||
//!                          ++===========++
//!
//! ```
//!
//! ## Data layout in TCAT application section for DSP effect
//!
//! The offset of TCAT application section is 0x6dd4. Any change by write transaction is firstly
//! effective when software message is written to 0x5ec.
//!
//! ### Data Layout for DSP effect
//!
//! * 0x000: DSP enable/disable (sw msg: 0x1c)
//! * 0x008: flags for channel strip effects (sw msg: 0x05)
//!     * 0x00000001: ch 0 equalizer enable
//!     * 0x00010000: ch 1 equalizer enable
//!     * 0x00000002: ch 0 compressor enable
//!     * 0x00020000: ch 1 compressor enable
//!     * 0x00000004: ch 0 equalizer after compressor
//!     * 0x00040000: ch 1 equalizer after compressor
//!
//! blk 0 | blk 2 | blk 4 | blk 6 | count   | purpose                         | sw msg |
//! ----- | ----- | ----- | ----- | --------| ------------------------------- | ------ |
//! 0x120 | 0x230 | 0x340 | 0x450 | 6 quads | ch 0 comp                       |   0x06 |
//! 0x138 | 0x248 | 0x358 | 0x468 | 2 quads | ch 0/ch 1 eq output             |   0z09 |
//! 0x140 | 0x250 | 0x360 | 0x470 | 5 quads | ch 0 eq low freq filter         |   0x0c |
//! 0x154 | 0x264 | 0x374 | 0x484 | 5 quads | ch 0 eq low-middle freq filter  |   0x0f |
//! 0x168 | 0x278 | 0x388 | 0x498 | 5 quads | ch 0 eq high-middle freq filter |   0x12 |
//! 0x17c | 0x28c | 0x39c | 0x4ac | 5 quads | ch 0 eq high freq filter        |   0x15 |
//! 0x190 | 0x2a0 | 0x3b0 | 0x4c0 | 6 quads | ch 0 reverb                     |   0x1a |
//!
//! blk 1 | blk 3 | blk 5 | blk 7 | count   | purpose                         | sw msg |
//! ----- | ----- | ----- | ----- | --------| ------------------------------- | ------ |
//! 0x1a8 | 0x2b8 | 0x3c8 | 0x4d8 | 6 quads | ch 1 comp                       |   0x07 |
//! 0x1c0 | 0x2d0 | 0x3e0 | 0x4f0 | 2 quads | ch 0/ch 1 eq output             |   0x0a |
//! 0x1c8 | 0x2d8 | 0x3e8 | 0x4f8 | 5 quads | ch 1 eq low freq filter         |   0x0d |
//! 0x1dc | 0x2ec | 0x3fc | 0x50c | 5 quads | ch 1 eq low-middle freq filter  |   0x10 |
//! 0x1f0 | 0x300 | 0x410 | 0x520 | 5 quads | ch 1 eq high-middle freq filter |   0x13 |
//! 0x204 | 0x314 | 0x424 | 0x534 | 5 quads | ch 1 eq high freq filter        |   0x16 |
//! 0x218 | 0x328 | 0x438 | 0x548 | 6 quads | ch 1 reverb                     |   0x1a |
//!
//! ### Compressor coefficients (6 quadlets)
//!
//! Actually change to block 2 is effective.
//!
//! quad | purpose          | min value  | max value  | min repr | max repr |
//! ---- | -------------    | ---------- | ---------- | -------- | -------- |
//!    0 | unknown          | 0x3f800000 | 0x3f800000 |    -     |    -     |
//!    1 | output volume    | 0x00000000 | 0x42800000 | -36.0 dB | +36.0 dB |
//!    2 | threshold        | 0xbfa00000 | 0x00000000 | -80.0 dB |   0.0 dB |
//!    3 | ratio            | 0x3d000000 | 0x3f000000 |  1.1:1   |  inf:1   |
//!    4 | attack           | 0xbf800000 | 0xbf700000 |   2ms    |  100ms   |
//!    5 | release          | 0x3f700000 | 0x3f800000 |  100ms   |   3s     |
//!
//! ### Equalizer output coefficients (2 quadlets)
//!
//! Actually change to block 2 is effective.
//!
//! quad | purpose          | min value  | max value  | min repr | max repr |
//! ---- | ---------------- | ---------- | ---------- | -------- | -------- |
//!    0 | left volume      | 0x00000000 | 0x3f800000 | -36.0 dB | +36.0 dB |
//!    1 | right volume     | 0x00000000 | 0x3f800000 | -36.0 dB | +36.0 dB |
//!
//! ### Equalizer coefficients (5 quadlets)
//!
//! Actually change to block 2 is effective.
//!
//! quad | purpose          | min value  | max value  | min repr | max repr |
//! ---- | ---------------- | ---------- | ---------- | -------- | -------- |
//!    0 | unknown          |     -      |     -      |    -     |    -     |
//!    1 | unknown          |     -      |     -      |    -     |    -     |
//!    2 | unknown          |     -      |     -      |    -     |    -     |
//!    3 | unknown          |     -      |     -      |    -     |    -     |
//!    4 | unknown          |     -      |     -      |    -     |    -     |
//!
//! ### Reverb coefficients (6 quadlets)
//!
//! Actually change to block 3 is effective.
//!
//! quad | purpose          | min value  | max value  | min repr | max repr |
//! ---- | ---------------- | ---------- | ---------- | -------- | -------- |
//!    0 | room size        | 0x00000000 | 0x3f800000 | 0 %      | 100 %    |
//!    1 | air              | 0x00000000 | 0x3f800000 | 100 %    | 0 %      |
//!    2 | enabled          | 0x00000000 | 0x3f800000 | false    | true     |
//!    3 | disabled         | 0x00000000 | 0x3f800000 | false    | true     |
//!    4 | pre filter value | 0x00000000 | 0x3f800000 | 5.0      | 0.0      |
//!    5 | pre filter sign  | 0x00000000 | 0x3f800000 | negative | positive |

use super::{tcat::tcd22xx_spec::*, *};

/// Protocol implementation specific to Saffire Pro 24 DSP.
#[derive(Default, Debug)]
pub struct SPro24DspProtocol;

impl TcatOperation for SPro24DspProtocol {}

impl TcatGlobalSectionSpecification for SPro24DspProtocol {}

impl Tcd22xxSpecOperation for SPro24DspProtocol {
    const INPUTS: &'static [Input] = &[
        Input {
            id: SrcBlkId::Ins0,
            offset: 2,
            count: 2,
            label: Some("Mic"),
        },
        Input {
            id: SrcBlkId::Ins0,
            offset: 0,
            count: 2,
            label: Some("Line"),
        },
        Input {
            id: SrcBlkId::Ins0,
            offset: 8,
            count: 2,
            label: Some("Ch-strip"),
        },
        // Input{id: SrcBlkId::Ins0, offset: 4, count: 2, label: Some("Ch-strip")}, at 88.2/96.0 kHz.
        Input {
            id: SrcBlkId::Ins0,
            offset: 14,
            count: 2,
            label: Some("Reverb"),
        },
        // Input{id: SrcBlkId::Ins0, offset: 6, count: 2, label: Some("Reverb")}, at 88.2/96.0 kHz.
        Input {
            id: SrcBlkId::Aes,
            offset: 6,
            count: 2,
            label: Some("S/PDIF-coax"),
        },
        // NOTE: share the same optical interface.
        Input {
            id: SrcBlkId::Adat,
            offset: 0,
            count: 8,
            label: None,
        },
        Input {
            id: SrcBlkId::Aes,
            offset: 4,
            count: 2,
            label: Some("S/PDIF-opt"),
        },
    ];

    const OUTPUTS: &'static [Output] = &[
        Output {
            id: DstBlkId::Ins0,
            offset: 0,
            count: 6,
            label: None,
        },
        Output {
            id: DstBlkId::Aes,
            offset: 6,
            count: 2,
            label: Some("S/PDIF-coax"),
        },
        Output {
            id: DstBlkId::Ins0,
            offset: 8,
            count: 2,
            label: Some("Ch-strip"),
        },
        // Output{id: DstBlkId::Ins0, offset: 4, count: 2, label: Some("Ch-strip")}, at 88.2/96.0 kHz.
        Output {
            id: DstBlkId::Ins0,
            offset: 14,
            count: 2,
            label: Some("Reverb"),
        },
        // Output{id: DstBlkId::Ins0, offset: 6, count: 2, label: Some("Reverb")}, at 88.2/96.0 kHz.
    ];

    // NOTE: The first 4 entries in router section are used to display hardware metering.
    const FIXED: &'static [SrcBlk] = &[
        SrcBlk {
            id: SrcBlkId::Ins0,
            ch: 2,
        },
        SrcBlk {
            id: SrcBlkId::Ins0,
            ch: 3,
        },
        SrcBlk {
            id: SrcBlkId::Ins0,
            ch: 0,
        },
        SrcBlk {
            id: SrcBlkId::Ins0,
            ch: 1,
        },
    ];
}

impl SaffireproSwNoticeOperation for SPro24DspProtocol {
    const SW_NOTICE_OFFSET: usize = 0x05ec;
}

impl SaffireproOutGroupOperation for SPro24DspProtocol {
    const OUT_GROUP_STATE_OFFSET: usize = 0x000c;

    const ENTRY_COUNT: usize = 6;
    const HAS_VOL_HWCTL: bool = false;

    const SRC_NOTICE: u32 = 0x00000001;
    const DIM_MUTE_NOTICE: u32 = 0x00000002;
}

impl SaffireproInputOperation for SPro24DspProtocol {
    const INPUT_PARAMS_OFFSET: usize = 0x0058;
}

// When VRM mode is enabled, write 0x00000001 to the offset
#[allow(dead_code)]
const DSP_ENABLE_OFFSET: usize = 0x0070; // sw notice: 0x1c.
const CH_STRIP_FLAG_OFFSET: usize = 0x0078;
const CH_STRIP_FLAG_EQ_ENABLE: u16 = 0x0001;
const CH_STRIP_FLAG_COMP_ENABLE: u16 = 0x0002;
const CH_STRIP_FLAG_EQ_AFTER_COMP: u16 = 0x0004;

const CH_STRIP_FLAG_SW_NOTICE: u32 = 0x00000005;

const COEF_OFFSET: usize = 0x0190;
const COEF_BLOCK_SIZE: usize = 0x88;
//const COEF_BLOCK_COUNT: usize = 8;

const EQ_COEF_COUNT: usize = 5;

const COMP_OUTPUT_OFFSET: usize = 0x04;
const COMP_THRESHOLD_OFFSET: usize = 0x08;
const COMP_RATIO_OFFSET: usize = 0x0c;
const COMP_ATTACK_OFFSET: usize = 0x10;
const COMP_RELEASE_OFFSET: usize = 0x14;

const COMP_CH0_SW_NOTICE: u32 = 0x00000006;
const COMP_CH1_SW_NOTICE: u32 = 0x00000007;

const EQ_OUTPUT_OFFSET: usize = 0x18;
const EQ_LOW_FREQ_OFFSET: usize = 0x20;
//const EQ_LOW_MIDDLE_FREQ_OFFSET: usize = 0x34;
//const EQ_HIGH_MIDDLE_FREQ_OFFSET: usize = 0x48;
//const EQ_HIGH_FREQ_OFFSET: usize = 0x5c;

const EQ_OUTPUT_CH0_SW_NOTICE: u32 = 0x09;
const EQ_OUTPUT_CH1_SW_NOTICE: u32 = 0x0a;
const EQ_LOW_FREQ_CH0_SW_NOTICE: u32 = 0x0c;
const EQ_LOW_FREQ_CH1_SW_NOTICE: u32 = 0x0c;
const EQ_LOW_MIDDLE_FREQ_CH0_SW_NOTICE: u32 = 0x0f;
const EQ_LOW_MIDDLE_FREQ_CH1_SW_NOTICE: u32 = 0x10;
const EQ_HIGH_MIDDLE_FREQ_CH0_SW_NOTICE: u32 = 0x12;
const EQ_HIGH_MIDDLE_FREQ_CH1_SW_NOTICE: u32 = 0x13;
const EQ_HIGH_FREQ_CH0_SW_NOTICE: u32 = 0x15;
const EQ_HIGH_FREQ_CH1_SW_NOTICE: u32 = 0x16;

const REVERB_SIZE_OFFSET: usize = 0x70;
const REVERB_AIR_OFFSET: usize = 0x74;
const REVERB_ENABLE_OFFSET: usize = 0x78;
const REVERB_DISABLE_OFFSET: usize = 0x7c;
const REVERB_PRE_FILTER_VALUE_OFFSET: usize = 0x80;
const REVERB_PRE_FILTER_SIGN_OFFSET: usize = 0x84;

const REVERB_SW_NOTICE: u32 = 0x0000001a;

fn serialize_f32(val: &f32, raw: &mut [u8]) -> Result<(), String> {
    assert!(raw.len() >= 4);

    raw[..4].copy_from_slice(&val.to_be_bytes());

    Ok(())
}

fn deserialize_f32(val: &mut f32, raw: &[u8]) -> Result<(), String> {
    assert!(raw.len() >= 4);

    let mut quadlet = [0; 4];
    quadlet.copy_from_slice(&raw[..4]);
    *val = f32::from_be_bytes(quadlet);

    Ok(())
}

/// State of compressor effect.
#[derive(Default, Debug, Copy, Clone, PartialEq)]
pub struct Spro24DspCompressorState {
    /// The volume of output, between 0.0 to 64.0.
    pub output: [f32; 2],
    /// The threshold, between -1.25 to 0.0.
    pub threshold: [f32; 2],
    /// The ratio, between 0.03125 to 0.5.
    pub ratio: [f32; 2],
    /// The attack, between -0.9375 to -1.0.
    pub attack: [f32; 2],
    /// The release, between 0.9375 to 1.0.
    pub release: [f32; 2],
}

/// Coefficients per frequency band.
#[derive(Default, Debug, Copy, Clone, PartialEq)]
pub struct Spro24DspEqualizerFrequencyBandState([f32; EQ_COEF_COUNT]);

/// State of equalizer effect.
#[derive(Default, Debug, Copy, Clone, PartialEq)]
pub struct Spro24DspEqualizerState {
    /// The volume of output, between 0.0 to 1.0.
    pub output: [f32; 2],
    // TODO: how to convert these coefficients to friendly parameter.
    pub low_coef: [Spro24DspEqualizerFrequencyBandState; 2],
    pub low_middle_coef: [Spro24DspEqualizerFrequencyBandState; 2],
    pub high_middle_coef: [Spro24DspEqualizerFrequencyBandState; 2],
    pub high_coef: [Spro24DspEqualizerFrequencyBandState; 2],
}

/// State of reverb effect.
#[derive(Default, Debug, Copy, Clone, PartialEq)]
pub struct Spro24DspReverbState {
    /// The size of room, between 0.0 to 1.0.
    pub size: f32,
    /// The amount to reduce dumping, between 0.0 to 1.0.
    pub air: f32,
    /// Whether the reverb effect is enabled or not.
    pub enabled: bool,
    /// The ratio of high-pass/low-pass filter, between -1.0 to 1.0.
    pub pre_filter: f32,
}

/// General parameters of DSP effects.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Spro24DspEffectGeneralParams {
    /// Use equalizer after compressor.
    pub eq_after_comp: [bool; 2],
    /// Whether to enable compressor.
    pub comp_enable: [bool; 2],
    /// Whether to enable equalizer.
    pub eq_enable: [bool; 2],
}

const COEF_BLOCK_COMP: usize = 2;
const COEF_BLOCK_EQ: usize = 2;
const COEF_BLOCK_REVERB: usize = 3;

// Serialize to a pair of coefficient block.
fn serialize_compressor_state(
    state: &Spro24DspCompressorState,
    raw: &mut [u8],
) -> Result<(), String> {
    assert!(raw.len() >= COEF_BLOCK_SIZE * 2);

    (0..2).try_for_each(|ch| {
        let base_offset = COEF_BLOCK_SIZE * ch;

        let pos = base_offset + COMP_OUTPUT_OFFSET;
        serialize_f32(&state.output[ch], &mut raw[pos..(pos + 4)])?;

        let pos = base_offset + COMP_THRESHOLD_OFFSET;
        serialize_f32(&state.threshold[ch], &mut raw[pos..(pos + 4)])?;

        let pos = base_offset + COMP_RATIO_OFFSET;
        serialize_f32(&state.ratio[ch], &mut raw[pos..(pos + 4)])?;

        let pos = base_offset + COMP_ATTACK_OFFSET;
        serialize_f32(&state.attack[ch], &mut raw[pos..(pos + 4)])?;

        let pos = base_offset + COMP_RELEASE_OFFSET;
        serialize_f32(&state.release[ch], &mut raw[pos..(pos + 4)])
    })
}

// Deserialize from a pair of coefficient block.
fn deserialize_compressor_state(
    state: &mut Spro24DspCompressorState,
    raw: &[u8],
) -> Result<(), String> {
    assert!(raw.len() >= COEF_BLOCK_SIZE * 2);

    (0..2).try_for_each(|ch| {
        let base_offset = COEF_BLOCK_SIZE * ch;

        let pos = base_offset + COMP_OUTPUT_OFFSET;
        deserialize_f32(&mut state.output[ch], &raw[pos..(pos + 4)])?;

        let pos = base_offset + COMP_THRESHOLD_OFFSET;
        deserialize_f32(&mut state.threshold[ch], &raw[pos..(pos + 4)])?;

        let pos = base_offset + COMP_RATIO_OFFSET;
        deserialize_f32(&mut state.ratio[ch], &raw[pos..(pos + 4)])?;

        let pos = base_offset + COMP_ATTACK_OFFSET;
        deserialize_f32(&mut state.attack[ch], &raw[pos..(pos + 4)])?;

        let pos = base_offset + COMP_RELEASE_OFFSET;
        deserialize_f32(&mut state.release[ch], &raw[pos..(pos + 4)])
    })
}

// Serialize to a pair of coefficient block.
fn serialize_equalizer_state(
    state: &Spro24DspEqualizerState,
    raw: &mut [u8],
) -> Result<(), String> {
    assert!(raw.len() >= COEF_BLOCK_SIZE * 2);

    (0..2).try_for_each(|ch| {
        let base_offset = COEF_BLOCK_SIZE * ch;

        let pos = base_offset + EQ_OUTPUT_OFFSET;
        serialize_f32(&state.output[ch], &mut raw[pos..(pos + 4)])?;

        state.low_coef[ch]
            .0
            .iter()
            .chain(state.low_middle_coef[ch].0.iter())
            .chain(state.high_middle_coef[ch].0.iter())
            .chain(state.high_coef[ch].0.iter())
            .enumerate()
            .try_for_each(|(i, coef)| {
                let pos = base_offset + EQ_LOW_FREQ_OFFSET + i * 4;
                serialize_f32(coef, &mut raw[pos..(pos + 4)])
            })
    })
}

// Deserialize from a pair of coefficient block.
fn deserialize_equalizer_state(
    state: &mut Spro24DspEqualizerState,
    raw: &[u8],
) -> Result<(), String> {
    assert!(raw.len() >= COEF_BLOCK_SIZE * 2);

    (0..2).try_for_each(|ch| {
        let base_offset = COEF_BLOCK_SIZE * ch;

        let pos = base_offset + EQ_OUTPUT_OFFSET;
        deserialize_f32(&mut state.output[ch], &raw[pos..(pos + 4)])?;

        state.low_coef[ch]
            .0
            .iter_mut()
            .chain(state.low_middle_coef[ch].0.iter_mut())
            .chain(state.high_middle_coef[ch].0.iter_mut())
            .chain(state.high_coef[ch].0.iter_mut())
            .enumerate()
            .try_for_each(|(i, coef)| {
                let pos = base_offset + EQ_LOW_FREQ_OFFSET + i * 4;
                deserialize_f32(coef, &raw[pos..(pos + 4)])
            })
    })
}

// Serialize to a coefficient block.
fn serialize_reverb_state(state: &Spro24DspReverbState, raw: &mut [u8]) -> Result<(), String> {
    assert!(raw.len() >= COEF_BLOCK_SIZE);

    let pos = REVERB_SIZE_OFFSET;
    serialize_f32(&state.size, &mut raw[pos..(pos + 4)])?;

    let pos = REVERB_AIR_OFFSET;
    serialize_f32(&state.air, &mut raw[pos..(pos + 4)])?;

    let enabled_pos = REVERB_ENABLE_OFFSET;
    let disabled_pos = REVERB_DISABLE_OFFSET;
    let vals = if state.enabled {
        [1.0, 0.0]
    } else {
        [0.0, 1.0]
    };
    serialize_f32(&vals[0], &mut raw[enabled_pos..(enabled_pos + 4)])?;
    serialize_f32(&vals[1], &mut raw[disabled_pos..(disabled_pos + 4)])?;

    let pos = REVERB_ENABLE_OFFSET;
    let val = if state.enabled { 1.0 } else { 0.0 };
    serialize_f32(&val, &mut raw[pos..(pos + 4)])?;

    let pos = REVERB_PRE_FILTER_VALUE_OFFSET;
    let val = state.pre_filter.abs();
    serialize_f32(&val, &mut raw[pos..(pos + 4)])?;

    let pos = REVERB_PRE_FILTER_SIGN_OFFSET;
    let sign = if state.pre_filter > 0.0 { 1.0 } else { 0.0 };
    serialize_f32(&sign, &mut raw[pos..(pos + 4)])?;

    Ok(())
}

// Deserialize from a coefficient block.
fn deserialize_reverb_state(state: &mut Spro24DspReverbState, raw: &[u8]) -> Result<(), String> {
    assert!(raw.len() >= COEF_BLOCK_SIZE);

    let pos = REVERB_SIZE_OFFSET;
    deserialize_f32(&mut state.size, &raw[pos..(pos + 4)])?;

    let pos = REVERB_AIR_OFFSET;
    deserialize_f32(&mut state.air, &raw[pos..(pos + 4)])?;

    let mut val = 0.0;
    let pos = REVERB_ENABLE_OFFSET;
    deserialize_f32(&mut val, &raw[pos..(pos + 4)])?;
    state.enabled = val > 0.0;

    let mut val = 0.0;
    let pos = REVERB_PRE_FILTER_VALUE_OFFSET;
    deserialize_f32(&mut val, &raw[pos..(pos + 4)])?;

    let mut sign = 0.0;
    let pos = REVERB_PRE_FILTER_SIGN_OFFSET;
    deserialize_f32(&mut sign, &raw[pos..(pos + 4)])?;
    if sign == 0.0 {
        val *= -1.0;
    }
    state.pre_filter = val;

    Ok(())
}

fn serialize_effect_general_params(
    params: &Spro24DspEffectGeneralParams,
    raw: &mut [u8],
) -> Result<(), String> {
    assert!(raw.len() >= 4);

    let mut val = 0u32;

    (0..2).for_each(|i| {
        let mut flags = 0u16;
        if params.eq_after_comp[i] {
            flags |= CH_STRIP_FLAG_EQ_AFTER_COMP;
        }
        if params.comp_enable[i] {
            flags |= CH_STRIP_FLAG_COMP_ENABLE;
        }
        if params.eq_enable[i] {
            flags |= CH_STRIP_FLAG_EQ_ENABLE;
        }

        val |= (flags as u32) << (16 * i);
    });

    val.build_quadlet(raw);

    Ok(())
}

fn deserialize_effect_general_params(
    params: &mut Spro24DspEffectGeneralParams,
    raw: &[u8],
) -> Result<(), String> {
    assert!(raw.len() >= 4);

    let mut val = 0u32;
    val.parse_quadlet(raw);
    (0..2).for_each(|i| {
        let flags = (val >> (i * 16)) as u16;
        params.eq_after_comp[i] = flags & CH_STRIP_FLAG_EQ_AFTER_COMP > 0;
        params.comp_enable[i] = flags & CH_STRIP_FLAG_COMP_ENABLE > 0;
        params.eq_enable[i] = flags & CH_STRIP_FLAG_EQ_ENABLE > 0;
    });

    Ok(())
}

impl SPro24DspProtocol {
    pub const COMPRESSOR_OUTPUT_MIN: f32 = 0.0;
    pub const COMPRESSOR_OUTPUT_MAX: f32 = 64.0;

    pub const COMPRESSOR_THRESHOLD_MIN: f32 = -1.25;
    pub const COMPRESSOR_THRESHOLD_MAX: f32 = 0.0;

    pub const COMPRESSOR_RATIO_MIN: f32 = 0.03125;
    pub const COMPRESSOR_RATIO_MAX: f32 = 0.5;

    pub const COMPRESSOR_ATTACK_MIN: f32 = -1.0;
    pub const COMPRESSOR_ATTACK_MAX: f32 = -0.9375;

    pub const COMPRESSOR_RELEASE_MIN: f32 = 0.9375;
    pub const COMPRESSOR_RELEASE_MAX: f32 = 1.0;

    pub const EQUALIZER_OUTPUT_MIN: f32 = 0.0;
    pub const EQUALIZER_OUTPUT_MAX: f32 = 1.0;

    pub const REVERB_SIZE_MIN: f32 = 0.0;
    pub const REVERB_SIZE_MAX: f32 = 1.0;

    pub const REVERB_AIR_MIN: f32 = 0.0;
    pub const REVERB_AIR_MAX: f32 = 1.0;

    pub const REVERB_PRE_FILTER_MIN: f32 = -1.0;
    pub const REVERB_PRE_FILTER_MAX: f32 = 1.0;

    pub fn cache_effect_general(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        params: &mut Spro24DspEffectGeneralParams,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut raw = vec![0u8; 4];

        ApplSectionProtocol::read_appl_data(
            req,
            node,
            sections,
            CH_STRIP_FLAG_OFFSET,
            &mut raw,
            timeout_ms,
        )?;

        serialize_effect_general_params(params, &mut raw)
            .map_err(|cause| Error::new(ProtocolExtensionError::Appl, &cause))
    }

    pub fn update_effect_general(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        params: &Spro24DspEffectGeneralParams,
        prev: &mut Spro24DspEffectGeneralParams,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut new = vec![0u8; 4];
        serialize_effect_general_params(params, &mut new)
            .map_err(|cause| Error::new(ProtocolExtensionError::Appl, &cause))?;

        let mut old = vec![0u8; 4];
        serialize_effect_general_params(prev, &mut old)
            .map_err(|cause| Error::new(ProtocolExtensionError::Appl, &cause))?;

        if new != old {
            ApplSectionProtocol::write_appl_data(
                req,
                node,
                sections,
                CH_STRIP_FLAG_OFFSET,
                &mut new,
                timeout_ms,
            )?;
            Self::write_sw_notice(req, node, sections, CH_STRIP_FLAG_SW_NOTICE, timeout_ms)?;
        }
        deserialize_effect_general_params(prev, &new)
            .map_err(|cause| Error::new(ProtocolExtensionError::Appl, &cause))
    }

    pub fn cache_whole_comp_params(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        params: &mut Spro24DspCompressorState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut raw = vec![0u8; COEF_BLOCK_SIZE * 2];

        ApplSectionProtocol::read_appl_data(
            req,
            node,
            sections,
            COEF_OFFSET + COEF_BLOCK_SIZE * COEF_BLOCK_COMP,
            &mut raw,
            timeout_ms,
        )?;

        serialize_compressor_state(params, &mut raw)
            .map_err(|cause| Error::new(ProtocolExtensionError::Appl, &cause))
    }

    pub fn update_partial_comp_params(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        comp: &Spro24DspCompressorState,
        prev: &mut Spro24DspCompressorState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut new = vec![0u8; COEF_BLOCK_SIZE * 2];
        serialize_compressor_state(comp, &mut new)
            .map_err(|cause| Error::new(ProtocolExtensionError::Appl, &cause))?;

        let mut old = vec![0u8; COEF_BLOCK_SIZE * 2];
        serialize_compressor_state(prev, &mut old)
            .map_err(|cause| Error::new(ProtocolExtensionError::Appl, &cause))?;

        (0..(COEF_BLOCK_SIZE * 2)).step_by(4).try_for_each(|pos| {
            if new[pos..(pos + 4)] != old[pos..(pos + 4)] {
                ApplSectionProtocol::write_appl_data(
                    req,
                    node,
                    sections,
                    COEF_OFFSET + COEF_BLOCK_SIZE * COEF_BLOCK_COMP + pos,
                    &mut new[pos..(pos + 4)],
                    timeout_ms,
                )
            } else {
                Ok(())
            }
        })?;
        Self::write_sw_notice(req, node, sections, COMP_CH0_SW_NOTICE, timeout_ms)?;
        Self::write_sw_notice(req, node, sections, COMP_CH1_SW_NOTICE, timeout_ms)?;

        deserialize_compressor_state(prev, &new)
            .map_err(|cause| Error::new(ProtocolExtensionError::Appl, &cause))
    }

    pub fn cache_whole_eq_params(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        params: &mut Spro24DspEqualizerState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut raw = vec![0u8; COEF_BLOCK_SIZE * 2];

        ApplSectionProtocol::read_appl_data(
            req,
            node,
            sections,
            COEF_OFFSET + COEF_BLOCK_SIZE * COEF_BLOCK_EQ,
            &mut raw,
            timeout_ms,
        )?;

        serialize_equalizer_state(params, &mut raw)
            .map_err(|cause| Error::new(ProtocolExtensionError::Appl, &cause))
    }

    pub fn update_partial_eq_params(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        eq: &Spro24DspEqualizerState,
        prev: &mut Spro24DspEqualizerState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut new = vec![0u8; COEF_BLOCK_SIZE * 2];
        serialize_equalizer_state(eq, &mut new)
            .map_err(|cause| Error::new(ProtocolExtensionError::Appl, &cause))?;

        let mut old = vec![0u8; COEF_BLOCK_SIZE * 2];
        serialize_equalizer_state(prev, &mut old)
            .map_err(|cause| Error::new(ProtocolExtensionError::Appl, &cause))?;

        (0..(COEF_BLOCK_SIZE * 2)).step_by(4).try_for_each(|pos| {
            if new[pos..(pos + 4)] != old[pos..(pos + 4)] {
                ApplSectionProtocol::write_appl_data(
                    req,
                    node,
                    sections,
                    COEF_OFFSET + COEF_BLOCK_SIZE * COEF_BLOCK_EQ + pos,
                    &mut new[pos..(pos + 4)],
                    timeout_ms,
                )
            } else {
                Ok(())
            }
        })?;
        Self::write_sw_notice(req, node, sections, EQ_OUTPUT_CH0_SW_NOTICE, timeout_ms)?;
        Self::write_sw_notice(req, node, sections, EQ_OUTPUT_CH1_SW_NOTICE, timeout_ms)?;
        Self::write_sw_notice(req, node, sections, EQ_LOW_FREQ_CH0_SW_NOTICE, timeout_ms)?;
        Self::write_sw_notice(req, node, sections, EQ_LOW_FREQ_CH1_SW_NOTICE, timeout_ms)?;
        Self::write_sw_notice(
            req,
            node,
            sections,
            EQ_LOW_MIDDLE_FREQ_CH0_SW_NOTICE,
            timeout_ms,
        )?;
        Self::write_sw_notice(
            req,
            node,
            sections,
            EQ_LOW_MIDDLE_FREQ_CH1_SW_NOTICE,
            timeout_ms,
        )?;
        Self::write_sw_notice(
            req,
            node,
            sections,
            EQ_HIGH_MIDDLE_FREQ_CH0_SW_NOTICE,
            timeout_ms,
        )?;
        Self::write_sw_notice(
            req,
            node,
            sections,
            EQ_HIGH_MIDDLE_FREQ_CH1_SW_NOTICE,
            timeout_ms,
        )?;
        Self::write_sw_notice(req, node, sections, EQ_HIGH_FREQ_CH0_SW_NOTICE, timeout_ms)?;
        Self::write_sw_notice(req, node, sections, EQ_HIGH_FREQ_CH1_SW_NOTICE, timeout_ms)?;

        deserialize_equalizer_state(prev, &new)
            .map_err(|cause| Error::new(ProtocolExtensionError::Appl, &cause))
    }

    pub fn cache_whole_reverb_params(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        params: &mut Spro24DspReverbState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut raw = vec![0u8; COEF_BLOCK_SIZE];

        ApplSectionProtocol::read_appl_data(
            req,
            node,
            sections,
            COEF_OFFSET + COEF_BLOCK_SIZE * COEF_BLOCK_REVERB,
            &mut raw,
            timeout_ms,
        )?;

        serialize_reverb_state(params, &mut raw)
            .map_err(|cause| Error::new(ProtocolExtensionError::Appl, &cause))
    }

    pub fn update_partial_reverb_params(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        reverb: &Spro24DspReverbState,
        prev: &mut Spro24DspReverbState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut new = vec![0u8; COEF_BLOCK_SIZE];
        serialize_reverb_state(reverb, &mut new)
            .map_err(|cause| Error::new(ProtocolExtensionError::Appl, &cause))?;

        let mut old = vec![0u8; COEF_BLOCK_SIZE];
        serialize_reverb_state(prev, &mut old)
            .map_err(|cause| Error::new(ProtocolExtensionError::Appl, &cause))?;

        (0..(COEF_BLOCK_SIZE * 2)).step_by(4).try_for_each(|pos| {
            if new[pos..(pos + 4)] != old[pos..(pos + 4)] {
                ApplSectionProtocol::write_appl_data(
                    req,
                    node,
                    sections,
                    COEF_OFFSET + COEF_BLOCK_SIZE * COEF_BLOCK_REVERB + pos,
                    &mut new[pos..(pos + 4)],
                    timeout_ms,
                )
            } else {
                Ok(())
            }
        })?;
        Self::write_sw_notice(req, node, sections, REVERB_SW_NOTICE, timeout_ms)?;
        deserialize_reverb_state(prev, &new)
            .map_err(|cause| Error::new(ProtocolExtensionError::Appl, &cause))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn compressor_state_serdes() {
        let state = Spro24DspCompressorState {
            output: [0.04, 0.05],
            threshold: [0.16, 0.17],
            ratio: [0.20, 0.21],
            attack: [0.32, 0.33],
            release: [0.44, 0.45],
        };

        let mut raw = [0u8; COEF_BLOCK_SIZE * 2];
        serialize_compressor_state(&state, &mut raw).unwrap();

        let mut s = Spro24DspCompressorState::default();
        deserialize_compressor_state(&mut s, &raw).unwrap();

        assert_eq!(state, s);
    }

    #[test]
    fn equalizer_state_serdes() {
        let state = Spro24DspEqualizerState {
            output: [0.06, 0.07],
            low_coef: [
                Spro24DspEqualizerFrequencyBandState([0.00, 0.01, 0.02, 0.03, 0.04]),
                Spro24DspEqualizerFrequencyBandState([0.10, 0.11, 0.12, 0.13, 0.14]),
            ],
            low_middle_coef: [
                Spro24DspEqualizerFrequencyBandState([0.20, 0.21, 0.22, 0.23, 0.24]),
                Spro24DspEqualizerFrequencyBandState([0.30, 0.31, 0.32, 0.33, 0.34]),
            ],
            high_middle_coef: [
                Spro24DspEqualizerFrequencyBandState([0.40, 0.41, 0.42, 0.43, 0.44]),
                Spro24DspEqualizerFrequencyBandState([0.50, 0.51, 0.52, 0.53, 0.54]),
            ],
            high_coef: [
                Spro24DspEqualizerFrequencyBandState([0.60, 0.61, 0.62, 0.63, 0.64]),
                Spro24DspEqualizerFrequencyBandState([0.70, 0.71, 0.72, 0.73, 0.74]),
            ],
        };

        let mut raw = [0u8; COEF_BLOCK_SIZE * 2];
        serialize_equalizer_state(&state, &mut raw).unwrap();

        let mut s = Spro24DspEqualizerState::default();
        deserialize_equalizer_state(&mut s, &raw).unwrap();

        assert_eq!(state, s);
    }

    #[test]
    fn reverb_state_serdes() {
        let state = Spro24DspReverbState {
            size: 0.04,
            air: 0.14,
            enabled: false,
            pre_filter: -0.1,
        };
        let mut raw = [0u8; COEF_BLOCK_SIZE];
        serialize_reverb_state(&state, &mut raw).unwrap();

        let mut s = Spro24DspReverbState::default();
        deserialize_reverb_state(&mut s, &raw).unwrap();

        assert_eq!(state, s);
    }

    #[test]
    fn effect_general_params_serdes() {
        let params = Spro24DspEffectGeneralParams {
            eq_after_comp: [false, true],
            comp_enable: [true, false],
            eq_enable: [false, true],
        };
        let mut raw = [0u8; 4];
        serialize_effect_general_params(&params, &mut raw).unwrap();

        let mut p = Spro24DspEffectGeneralParams::default();
        deserialize_effect_general_params(&mut p, &raw).unwrap();

        assert_eq!(params, p);
    }
}
