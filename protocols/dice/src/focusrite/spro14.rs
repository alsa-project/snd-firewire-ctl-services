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
//!
//! XLR input 1 ------+---------+
//! Phone input 1-----+         |
//!                             |
//! XLR input 2 ------+---------+
//! Phone input 2 ----+         |
//!                             +----------------> analog-input-1/2
//! Phone input 3/4 -----------------------------> analog-input-3/4
//! Coaxial input 1/2 ---------------------------> spdif-input-1/2
//!
//!                        ++=============++
//! analog-input-1/2 ----> ||   34 x 32   || ----> analog-output-1/2
//! analog-input-3/4 ----> ||   router    || ----> analog-output-3/4
//! spdif-input-1/2 -----> ||   up to     || ----> spdif-output-1/2
//!                        || 128 entries ||
//! stream-input-1/2 ----> ||             || ----> stream-output-1/2
//! stream-input-3/4 ----> ||             || ----> stream-output-3/4
//! stream-input-5/6 ----> ||             || ----> stream-output-5/6
//! stream-input-7/8 ----> ||             || ----> stream-output-7/8
//! stream-input-9/10 ---> ||             ||
//! stream-input-11/12 --> ||             ||
//!                        ||             ||
//! mixer-output-1/2 ----> ||             || ----> mixer-input-1/2
//! mixer-output-3/4 ----> ||             || ----> mixer-input-3/4
//! mixer-output-5/6 ----> ||             || ----> mixer-input-5/6
//! mixer-output-7/8 ----> ||             || ----> mixer-input-7/8
//! mixer-output-9/10 ---> ||             || ----> mixer-input-9/10
//! mixer-output-11/12 --> ||             || ----> mixer-input-11/12
//! mixer-output-13/14 --> ||             || ----> mixer-input-13/14
//! mixer-output-15/16 --> ||             || ----> mixer-input-15/16
//!                        ||             || ----> mixer-input-17/18
//!                        ++=============++
//!
//!                        ++=============++
//! mixer-input-1/2 -----> ||             || ----> mixer-output-1/2
//! mixer-input-3/4 -----> ||             || ----> mixer-output-3/4
//! mixer-input-5/6 -----> ||             || ----> mixer-output-5/6
//! mixer-input-7/8 -----> ||    mixer    || ----> mixer-output-7/8
//! mixer-input-9/10 ----> ||             || ----> mixer-output-9/10
//! mixer-input-11/12 ---> ||   18 x 16   || ----> mixer-output-10/12
//! mixer-input-13/14 ---> ||             || ----> mixer-output-12/14
//! mixer-input-15/16 ---> ||             || ----> mixer-output-14/16
//! mixer-input-17/18 ---> ||             ||
//!                        ++=============++
//!
//!                        ++=============++
//!                        ||             || ----> Phone output 1/2
//! analog-output-1/2 ---> ||   output    ||
//! analog-output-3/4 ---> ||    group    || --+-> Phone output 3/4
//!                        ||             ||   +-> Headphone output 1/2
//!                        ++=============++
//!
//! spdif-output-1/2 ----------------------------> Coaxial output 1/2
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
