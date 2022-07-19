// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

// The content comes from 'include/uapi/sound/tlv.h' of Linux kernel.
pub const SNDRV_CTL_TLVT_CONTAINER: u32 = 0;
pub const SNDRV_CTL_TLVT_DB_SCALE: u32 = 1;
pub const SNDRV_CTL_TLVT_DB_LINEAR: u32 = 2;
pub const SNDRV_CTL_TLVT_DB_RANGE: u32 = 3;
pub const SNDRV_CTL_TLVT_DB_MINMAX: u32 = 4;
pub const SNDRV_CTL_TLVT_DB_MINMAX_MUTE: u32 = 5;

pub const SNDRV_CTL_TLVT_CHMAP_FIXED: u32 = 0x101;
pub const SNDRV_CTL_TLVT_CHMAP_VAR: u32 = 0x102;
pub const SNDRV_CTL_TLVT_CHMAP_PAIRED: u32 = 0x103;

pub const SNDRV_CTL_TLVD_DB_SCALE_MASK: u32 = 0xffff;
pub const SNDRV_CTL_TLVD_DB_SCALE_MUTE: u32 = 0x10000;

pub const SNDRV_CTL_TLVO_DB_SCALE_MIN: u32 = 2;
pub const SNDRV_CTL_TLVO_DB_SCALE_MUTE_AND_STEP: u32 = 3;

pub const SNDRV_CTL_TLVO_DB_MINMAX_MIN: u32 = 2;
pub const SNDRV_CTL_TLVO_DB_MINMAX_MAX: u32 = 3;

pub const SNDRV_CTL_TLVO_DB_LINEAR_MIN: u32 = 2;
pub const SNDRV_CTL_TLVO_DB_LINEAR_MAX: u32 = 3;

pub const SNDRV_CTL_TLVD_DB_GAIN_MUTE: i32 = -9999999;

// The content comes from 'include/uapi/sound/asound.h' of Linux kernel.
pub const SNDRV_CHMAP_UNKNOWN: u16 = 0;
pub const SNDRV_CHMAP_NA: u16 = 1;
pub const SNDRV_CHMAP_MONO: u16 = 2;
pub const SNDRV_CHMAP_FL: u16 = 3;
pub const SNDRV_CHMAP_FR: u16 = 4;
pub const SNDRV_CHMAP_RL: u16 = 5;
pub const SNDRV_CHMAP_RR: u16 = 6;
pub const SNDRV_CHMAP_FC: u16 = 7;
pub const SNDRV_CHMAP_LFE: u16 = 8;
pub const SNDRV_CHMAP_SL: u16 = 9;
pub const SNDRV_CHMAP_SR: u16 = 10;
pub const SNDRV_CHMAP_RC: u16 = 11;
pub const SNDRV_CHMAP_FLC: u16 = 12;
pub const SNDRV_CHMAP_FRC: u16 = 13;
pub const SNDRV_CHMAP_RLC: u16 = 14;
pub const SNDRV_CHMAP_RRC: u16 = 15;
pub const SNDRV_CHMAP_FLW: u16 = 16;
pub const SNDRV_CHMAP_FRW: u16 = 17;
pub const SNDRV_CHMAP_FLH: u16 = 18;
pub const SNDRV_CHMAP_FCH: u16 = 19;
pub const SNDRV_CHMAP_FRH: u16 = 20;
pub const SNDRV_CHMAP_TC: u16 = 21;
pub const SNDRV_CHMAP_TFL: u16 = 22;
pub const SNDRV_CHMAP_TFR: u16 = 23;
pub const SNDRV_CHMAP_TFC: u16 = 24;
pub const SNDRV_CHMAP_TRL: u16 = 25;
pub const SNDRV_CHMAP_TRR: u16 = 26;
pub const SNDRV_CHMAP_TRC: u16 = 27;
pub const SNDRV_CHMAP_TFLC: u16 = 28;
pub const SNDRV_CHMAP_TFRC: u16 = 29;
pub const SNDRV_CHMAP_TSL: u16 = 30;
pub const SNDRV_CHMAP_TSR: u16 = 31;
pub const SNDRV_CHMAP_LLFE: u16 = 32;
pub const SNDRV_CHMAP_RLFE: u16 = 33;
pub const SNDRV_CHMAP_BC: u16 = 34;
pub const SNDRV_CHMAP_BLC: u16 = 35;
pub const SNDRV_CHMAP_BRC: u16 = 36;

pub const SNDRV_CHMAP_POSITION_MASK: u32 = 0x0000ffff;
pub const SNDRV_CHMAP_PHASE_INVERSE: u32 = 0x00010000;
pub const SNDRV_CHMAP_DRIVER_SPEC: u32 = 0x00020000;
