// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol specific to Focusrite Saffire Pro 14.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by Focusrite for Saffire Pro 14.
//!
//! ## Diagram of internal signal flow for Saffire Pro 14.
//!
//! I note that optical input interface is available exclusively for ADAT input and S/PDIF input.
//!
//! ```text
//!                        ++===========++
//! mixer-input-0/1 -----> ||           || -> mixer-output-0/1
//! mixer-input-2/3 -----> ||           || -> mixer-output-2/3
//! mixer-input-4/5 -----> ||           || -> mixer-output-4/5
//! mixer-input-6/7 -----> ||   mixer   || -> mixer-output-6/7
//! mixer-input-8/9 -----> ||           || -> mixer-output-8/9
//! mixer-input-10/11 ---> ||  18 x 16  || -> mixer-output-10/11
//! mixer-input-12/13 ---> ||           || -> mixer-output-12/13
//! mixer-input-14/15 ---> ||           || -> mixer-output-14/15
//! mixer-input-16/17 ---> ||           ||
//!                        ++===========++
//!
//!                        ++===========++
//! mic-input-0/1 -------> ||           || -> mixer-input-0/1
//! line-input-0/1 ------> ||           || -> mixer-input-2/3
//! spdif-input-0/1 -----> ||           || -> mixer-input-4/5
//!                        ||  mixer    || -> mixer-input-6/7
//! stream-input-0/1 ----> ||  input    || -> mixer-input-8/9
//! stream-input-2/3 ----> ||  router   || -> mixer-input-10/11
//! stream-input-4/5 ----> ||           || -> mixer-input-12/13
//! stream-input-6/7 ----> ||   x 18    || -> mixer-input-14/15
//! stream-input-8/9 ----> ||           || -> mixer-input-16/17
//! stream-input-10/11 --> ||           ||
//!                        ++===========++
//!
//!                        ++===========++
//! mic-input-0/1 -------> ||           ||
//! line-input-0/1 ------> ||           ||
//! spdif-input-0/1 -----> ||           ||
//!                        ||  stream   ||
//! mixer-output-0/1 ----> ||  capture  || -> stream-output-0/1
//! mixer-output-2/3 ----> ||  router   || -> stream-output-2/3
//! mixer-output-4/5 ----> ||           || -> stream-output-4/5
//! mixer-output-6/7 ----> ||    x 8    || -> stream-output-6/7
//! mixer-output-8/9 ----> ||           ||
//! mixer-output-10/11 --> ||           ||
//! mixer-output-12/13 --> ||           ||
//! mixer-output-14/15 --> ||           ||
//!                        ++===========++
//!
//!                        ++===========++
//! mic-input-0/1 -------> ||           ||
//! line-input-0/1 ------> ||           ||
//! spdif-input-0/1 -----> ||           ||
//!                        ||           ||
//! stream-input-0/1 ----> ||           ||
//! stream-input-2/3 ----> ||           ||
//! stream-input-4/5 ----> ||           ||
//! stream-input-6/7 ----> ||           ||
//! stream-input-8/9 ----> ||  physical ||
//! stream-input-10/11 --> ||  output   || -> analog-output-0/1
//! stream-input-12/13 --> ||  router   || -> analog-output-2/3
//! stream-input-14/15 --> ||           || -> spdif-output-0/1
//! stream-input-16/17 --> ||   x 6     ||
//!                        ||           ||
//! mixer-output-0/1 ----> ||           ||
//! mixer-output-2/3 ----> ||           ||
//! mixer-output-4/5 ----> ||           ||
//! mixer-output-6/7 ----> ||           ||
//! mixer-output-8/9 ----> ||           ||
//! mixer-output-10/11 --> ||           ||
//! mixer-output-12/13 --> ||           ||
//! mixer-output-14/15 --> ||           ||
//!                        ++===========++
//!
//! ```

use super::{tcat::tcd22xx_spec::*, *};

/// Protocol implementation specific to Saffire Pro 14.
#[derive(Default, Debug)]
pub struct SPro14Protocol;

impl TcatOperation for SPro14Protocol {}

impl TcatGlobalSectionSpecification for SPro14Protocol {}

impl Tcd22xxSpecOperation for SPro14Protocol {
    const INPUTS: &'static [Input] = &[
        Input {
            id: SrcBlkId::Ins0,
            offset: 0,
            count: 4,
            label: None,
        },
        Input {
            id: SrcBlkId::Aes,
            offset: 6,
            count: 2,
            label: Some("S/PDIF"),
        },
    ];
    const OUTPUTS: &'static [Output] = &[
        Output {
            id: DstBlkId::Ins0,
            offset: 0,
            count: 4,
            label: None,
        },
        Output {
            id: DstBlkId::Aes,
            offset: 6,
            count: 2,
            label: Some("S/PDIF"),
        },
    ];
    // NOTE: The first 2 entries in router section are used to display signal detection.
    const FIXED: &'static [SrcBlk] = &[
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

impl SaffireproSwNoticeOperation for SPro14Protocol {
    const SW_NOTICE_OFFSET: usize = 0x000c;
}

impl SaffireproOutGroupOperation for SPro14Protocol {
    const OUT_GROUP_STATE_OFFSET: usize = 0x0010;

    const ENTRY_COUNT: usize = 4;
    const HAS_VOL_HWCTL: bool = false;

    const SRC_NOTICE: u32 = 0x00000001;
    const DIM_MUTE_NOTICE: u32 = 0x00000002;
}

impl SaffireproInputOperation for SPro14Protocol {
    const INPUT_PARAMS_OFFSET: usize = 0x005c;
}
