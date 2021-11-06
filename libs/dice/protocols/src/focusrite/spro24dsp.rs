// SPDX-License-Identifier: GPL-3.0-or-later
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

use crate::{focusrite::*, tcat::{extension::*, tcd22xx_spec::*}};

/// The structure for protocol implementation specific to Saffire Pro 24 DSP.
#[derive(Debug)]
pub struct SPro24DspProtocol;

impl Tcd22xxSpecOperation for SPro24DspProtocol {
    const INPUTS: &'static [Input] = &[
        Input{id: SrcBlkId::Ins0, offset: 2, count: 2, label: Some("Mic")},
        Input{id: SrcBlkId::Ins0, offset: 0, count: 2, label: Some("Line")},
        // NOTE: share the same optical interface.
        Input{id: SrcBlkId::Adat, offset: 0, count: 8, label: None},
        Input{id: SrcBlkId::Aes, offset: 4, count: 2, label: Some("S/PDIF-opt")},
    ];

    const OUTPUTS: &'static [Output] = &[
        Output{id: DstBlkId::Ins0, offset: 0, count: 6, label: None},
        Output{id: DstBlkId::Aes, offset: 6, count: 2, label: Some("S/PDIF-coax")},
    ];

    // NOTE: The first 4 entries in router section are used to display hardware metering.
    const FIXED: &'static [SrcBlk] = &[
        SrcBlk{id: SrcBlkId::Ins0, ch: 2},
        SrcBlk{id: SrcBlkId::Ins0, ch: 3},
        SrcBlk{id: SrcBlkId::Ins0, ch: 0},
        SrcBlk{id: SrcBlkId::Ins0, ch: 1},
    ];
}

impl SaffireproSwNoticeOperation for SPro24DspProtocol {
    const SW_NOTICE_OFFSET: usize = 0x05ec;
}

impl SaffireproOutGroupOperation for SPro24DspProtocol {
    const ENTRY_COUNT: usize = 6;
    const HAS_VOL_HWCTL: bool = false;
    const OUT_CTL_OFFSET: usize = 0x000c;

    const SRC_NOTICE: u32 = 0x00000001;
    const DIM_MUTE_NOTICE: u32 = 0x00000002;
}
