// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol specific to Focusrite Saffire Pro 40.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by Focusrite for Saffire Pro 40.
//!
//! ## Diagram of internal signal flow for Saffire Pro 26.
//!
//! I note that optical input interface is available exclusively for ADAT input and S/PDIF input.
//!
//! ```text
//!
//! XLR input 1/2 ---------------------------------> analog-input-1/2
//! XLR input 3/4 ---------------------------------> analog-input-3/4
//! XLR input 5/6 ---------------------------------> analog-input-5/6
//! XLR input 7/8 ---------------------------------> analog-input-7/8
//! Coaxial input 1/2 -----------------------------> spdif-input-1/2
//! Optical input --------------or-----------------> spdif-input-3/4
//!                             +------------------> adat-input-1..8
//!
//!                          ++=============++
//! analog-input-1/2 ------> ||   56 x 60   || ----> analog-output-1/2
//! analog-input-3/4 ------> ||   router    || ----> analog-output-3/4
//! analog-input-5/6 ------> ||   up to     || ----> analog-output-5/6
//! analog-input-7/8 ------> || 128 entries || ----> analog-output-7/8
//!                          ||             || ----> analog-output-9/10
//! spdif-input-1/2 -------> ||             || ----> spdif-output-1/2
//! spdif-input-3/4 -------> ||             || ----> spdif-output-3/4
//! adat-input-1/2 --------> ||             || ----> adat-input-1/2
//! adat-input-3/4 --------> ||             || ----> adat-input-3/4
//! adat-input-5/6 --------> ||             || ----> adat-input-5/6
//! adat-input-7/8 --------> ||             || ----> adat-input-7/8
//!                          ||             ||
//! stream-input-A-1/2 ----> ||             || ----> stream-output-A-1/2
//! stream-input-A-3/4 ----> ||             || ----> stream-output-A-3/4
//! stream-input-A-5/6 ----> ||             || ----> stream-output-A-5/6
//! stream-input-A-7/8 ----> ||             || ----> stream-output-A-7/8
//! stream-input-A-9/10 ---> ||             || ----> stream-output-A-9/10
//! stream-input-A-11/12 --> ||             ||
//!                          ||             ||
//! stream-input-B-1/2 ----> ||             || ----> stream-output-B-1/2
//! stream-input-B-3/4 ----> ||             || ----> stream-output-B-3/4
//! stream-input-B-5/6 ----> ||             || ----> stream-output-B-5/6
//! stream-input-B-7/8 ----> ||             || ----> stream-output-B-7/8
//!                          ||             || ----> stream-output-B-9/10
//!                          ||             ||
//! mixer-output-1/2 ------> ||             || ----> mixer-input-1/2
//! mixer-output-3/4 ------> ||             || ----> mixer-input-3/4
//! mixer-output-5/6 ------> ||             || ----> mixer-input-5/6
//! mixer-output-7/8 ------> ||             || ----> mixer-input-7/8
//! mixer-output-9/10 -----> ||             || ----> mixer-input-9/10
//! mixer-output-11/12 ----> ||             || ----> mixer-input-11/12
//! mixer-output-13/14 ----> ||             || ----> mixer-input-13/14
//! mixer-output-15/16 ----> ||             || ----> mixer-input-15/16
//!                          ||             || ----> mixer-input-17/18
//!                          ++=============++
//!
//!                          ++=============++
//!                          ||             || ----> Phone output 1/2
//!                          ||             || ----> Phone output 3/4
//! analog-output-1/2 -----> ||             || ----> Phone output 5/6
//! analog-output-3/4 -----> ||   output    ||
//! analog-output-5/6 -----> ||             || --+-> Phone output 7/8
//! analog-output-7/8 -----> ||   group     ||   +-> Headphone output 1/2
//! analog-output-9/10 ----> ||             ||
//!                          ||             || --+-> Phone output 9/10
//!                          ||             ||   +-> Headphone output 3/4
//!                          ||             ||
//!                          ++=============++
//!
//! spdif-output-1/2 ------------------------------> Coaxial output 1/2
//!
//! spdif-output-3/4 -------------or---------------> Optical output
//! adat-output-1..8 -------------+
//!
//! ```

use super::{tcat::tcd22xx_spec::*, *};

/// Protocol implementation specific to Saffire Pro 40.
#[derive(Default, Debug)]
pub struct SPro40Protocol;

impl TcatOperation for SPro40Protocol {}

impl TcatGlobalSectionSpecification for SPro40Protocol {}

impl TcatExtensionOperation for SPro40Protocol {}

impl Tcd22xxSpecification for SPro40Protocol {
    const INPUTS: &'static [Input] = &[
        Input {
            id: SrcBlkId::Ins1,
            offset: 0,
            count: 8,
            label: None,
        },
        Input {
            id: SrcBlkId::Aes,
            offset: 0,
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
            count: 2,
            label: None,
        },
        Output {
            id: DstBlkId::Ins1,
            offset: 0,
            count: 8,
            label: None,
        },
        Output {
            id: DstBlkId::Aes,
            offset: 0,
            count: 2,
            label: Some("S/PDIF-coax"),
        },
        // NOTE: share the same optical interface.
        Output {
            id: DstBlkId::Adat,
            offset: 0,
            count: 8,
            label: None,
        },
        Output {
            id: DstBlkId::Aes,
            offset: 4,
            count: 2,
            label: Some("S/PDIF-opt"),
        },
    ];
    // NOTE: The first 8 entries in router section are used to display hardware metering.
    const FIXED: &'static [SrcBlk] = &[
        SrcBlk {
            id: SrcBlkId::Ins1,
            ch: 0,
        },
        SrcBlk {
            id: SrcBlkId::Ins1,
            ch: 1,
        },
        SrcBlk {
            id: SrcBlkId::Ins1,
            ch: 2,
        },
        SrcBlk {
            id: SrcBlkId::Ins1,
            ch: 3,
        },
        SrcBlk {
            id: SrcBlkId::Ins1,
            ch: 4,
        },
        SrcBlk {
            id: SrcBlkId::Ins1,
            ch: 5,
        },
        SrcBlk {
            id: SrcBlkId::Ins1,
            ch: 6,
        },
        SrcBlk {
            id: SrcBlkId::Ins1,
            ch: 7,
        },
    ];
}

impl SaffireproSwNoticeOperation for SPro40Protocol {
    const SW_NOTICE_OFFSET: usize = 0x0068;
}

const SRC_SW_NOTICE: u32 = 0x00000001;
const DIM_MUTE_SW_NOTICE: u32 = 0x00000002;

impl SaffireproOutGroupSpecification for SPro40Protocol {
    const OUT_GROUP_STATE_OFFSET: usize = 0x000c;

    const ENTRY_COUNT: usize = 10;
    const HAS_VOL_HWCTL: bool = true;

    const SRC_NOTICE: u32 = SRC_SW_NOTICE;
    const DIM_MUTE_NOTICE: u32 = DIM_MUTE_SW_NOTICE;
}

impl SaffireproIoParamsSpecification for SPro40Protocol {
    const AESEBU_IS_SUPPORTED: bool = false;
    const MIC_PREAMP_TRANSFORMER_IS_SUPPORTED: bool = false;
}
