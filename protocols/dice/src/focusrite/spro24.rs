// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol specific to Focusrite Saffire Pro 24.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by Focusrite for Saffire Pro 24.
//!
//! ## Diagram of internal signal flow for Saffire Pro 24.
//!
//! I note that optical input interface is available exclusively for ADAT input and S/PDIF input.
//!
//! ```text
//!
//!
//! XLR input 1 ------+---------+
//! Phone input 1-----+         |
//!                             |
//! XLR input 2 ------+---------+
//! Phone input 2 ----+         |
//!                             +---------------> analog-input-1/2
//! Phone input 3/4 ----------------------------> analog-input-3/4
//! Phone input 5/6 ----------------------------> analog-input-5/6
//! Coaxial input 1/2 --------------------------> spdif-input-1/2
//! Optical input --------------or--------------> spdif-input-3/4
//!                             +---------------> adat-input-1..8
//!
//!                          ++=============++
//! analog-input-1/2 ------> ||   42 x 42   || -> analog-output-1/2
//! analog-input-3/4 ------> ||   router    || -> analog-output-3/4
//! analog-input-5/6 ------> ||   up to     || -> analog-output-5/6
//! spdif-input-1/2 -------> || 128 entries || -> spdif-output-1/2
//! spdif-input-3/4 -------> ||             ||
//! adat-input-1/2 --------> ||             ||
//! adat-input-3/4 --------> ||             ||
//! adat-input-5/6 --------> ||             ||
//! adat-input-7/8 --------> ||             ||
//!                          ||             ||
//! stream-input-1/2 ------> ||             || -> stream-output-1/2
//! stream-input-3/4 ------> ||             || -> stream-output-3/4
//! stream-input-5/6 ------> ||             || -> stream-output-5/6
//! stream-input-7/8 ------> ||             || -> stream-output-7/8
//!                          ||             || -> stream-output-9/10
//!                          ||             || -> stream-output-11/12
//!                          ||             || -> stream-output-13/14
//!                          ||             || -> stream-output-15/16
//!                          ||             ||
//! mixer-output-1/2 ------> ||             || -> mixer-input-1/2
//! mixer-output-3/4 ------> ||             || -> mixer-input-3/4
//! mixer-output-5/6 ------> ||             || -> mixer-input-5/6
//! mixer-output-7/8 ------> ||             || -> mixer-input-7/8
//! mixer-output-9/10 -----> ||             || -> mixer-input-9/10
//! mixer-output-11/12 ----> ||             || -> mixer-input-11/12
//! mixer-output-13/14 ----> ||             || -> mixer-input-13/14
//! mixer-output-15/16 ----> ||             || -> mixer-input-15/16
//!                          ||             || -> mixer-input-17/18
//!                          ++=============++
//!
//!                          ++=============++
//! mixer-input-1/2 -------> ||             || -> mixer-output-1/2
//! mixer-input-3/4 -------> ||             || -> mixer-output-3/4
//! mixer-input-5/6 -------> ||             || -> mixer-output-5/6
//! mixer-input-7/8 -------> ||    mixer    || -> mixer-output-7/8
//! mixer-input-9/10 ------> ||             || -> mixer-output-9/10
//! mixer-input-11/12 -----> ||   18 x 16   || -> mixer-output-10/12
//! mixer-input-13/14 -----> ||             || -> mixer-output-12/14
//! mixer-input-15/16 -----> ||             || -> mixer-output-14/16
//! mixer-input-17/18 -----> ||             ||
//!                          ++=============++
//!
//!                          ++=============++
//!                          ||             || ----> Phone output 1/2
//!                          ||             ||
//! analog-output-1/2 -----> ||   output    || --+-> Phone output 3/4
//! analog-output-3/4 -----> ||   group     ||   +-> Headphone output 1/2
//! analog-output-5/6 -----> ||             ||
//!                          ||             || --+-> Phone output 5/6
//!                          ++=============++
//!
//! spdif-output-1/2 ---------------------------> Coaxial output 1/2
//!
//! ```

use super::{tcat::tcd22xx_spec::*, *};

/// Protocol implementation specific to Saffire Pro 24.
#[derive(Default, Debug)]
pub struct SPro24Protocol;

impl TcatOperation for SPro24Protocol {}

impl TcatGlobalSectionSpecification for SPro24Protocol {}

impl TcatExtensionOperation for SPro24Protocol {}

impl Tcd22xxSpecOperation for SPro24Protocol {
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

impl SaffireproSwNoticeOperation for SPro24Protocol {
    const SW_NOTICE_OFFSET: usize = 0x0068;
}

impl SaffireproOutGroupOperation for SPro24Protocol {
    const OUT_GROUP_STATE_OFFSET: usize = 0x000c;

    const ENTRY_COUNT: usize = 6;
    const HAS_VOL_HWCTL: bool = false;

    const SRC_NOTICE: u32 = 0x00000001;
    const DIM_MUTE_NOTICE: u32 = 0x00000002;
}

impl SaffireproInputOperation for SPro24Protocol {
    const INPUT_PARAMS_OFFSET: usize = 0x0058;
}
