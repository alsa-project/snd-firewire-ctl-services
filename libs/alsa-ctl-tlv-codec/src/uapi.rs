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
pub const SNDRV_CHMAP_UNKNOWN: u32 = 0;
pub const SNDRV_CHMAP_NA: u32 = 1;
pub const SNDRV_CHMAP_MONO: u32 = 2;
pub const SNDRV_CHMAP_FL: u32 = 3;
pub const SNDRV_CHMAP_FR: u32 = 4;
pub const SNDRV_CHMAP_RL: u32 = 5;
pub const SNDRV_CHMAP_RR: u32 = 6;
pub const SNDRV_CHMAP_FC: u32 = 7;
pub const SNDRV_CHMAP_LFE: u32 = 8;
pub const SNDRV_CHMAP_SL: u32 = 9;
pub const SNDRV_CHMAP_SR: u32 = 10;
pub const SNDRV_CHMAP_RC: u32 = 11;
pub const SNDRV_CHMAP_FLC: u32 = 12;
pub const SNDRV_CHMAP_FRC: u32 = 13;
pub const SNDRV_CHMAP_RLC: u32 = 14;
pub const SNDRV_CHMAP_RRC: u32 = 15;
pub const SNDRV_CHMAP_FLW: u32 = 16;
pub const SNDRV_CHMAP_FRW: u32 = 17;
pub const SNDRV_CHMAP_FLH: u32 = 18;
pub const SNDRV_CHMAP_FCH: u32 = 19;
pub const SNDRV_CHMAP_FRH: u32 = 20;
pub const SNDRV_CHMAP_TC: u32 = 21;
pub const SNDRV_CHMAP_TFL: u32 = 22;
pub const SNDRV_CHMAP_TFR: u32 = 23;
pub const SNDRV_CHMAP_TFC: u32 = 24;
pub const SNDRV_CHMAP_TRL: u32 = 25;
pub const SNDRV_CHMAP_TRR: u32 = 26;
pub const SNDRV_CHMAP_TRC: u32 = 27;
pub const SNDRV_CHMAP_TFLC: u32 = 28;
pub const SNDRV_CHMAP_TFRC: u32 = 29;
pub const SNDRV_CHMAP_TSL: u32 = 30;
pub const SNDRV_CHMAP_TSR: u32 = 31;
pub const SNDRV_CHMAP_LLFE: u32 = 32;
pub const SNDRV_CHMAP_RLFE: u32 = 33;
pub const SNDRV_CHMAP_BC: u32 = 34;
pub const SNDRV_CHMAP_BLC: u32 = 35;
pub const SNDRV_CHMAP_BRC: u32 = 36;

pub const SNDRV_CHMAP_POSITION_MASK: u32 = 0x0000ffff;
pub const SNDRV_CHMAP_PHASE_INVERSE: u32 = 0x00010000;
pub const SNDRV_CHMAP_DRIVER_SPEC: u32 = 0x00020000;
