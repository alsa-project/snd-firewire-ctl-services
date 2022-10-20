// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol specific to Focusrite Saffire Pro 40.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by Focusrite for Saffire Pro 40.

use super::{tcat::tcd22xx_spec::*, *};

/// Protocol implementation specific to Saffire Pro 40.
#[derive(Default, Debug)]
pub struct SPro40Protocol;

impl TcatOperation for SPro40Protocol {}

impl TcatGlobalSectionSpecification for SPro40Protocol {}

impl Tcd22xxSpecOperation for SPro40Protocol {
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

impl SaffireproOutGroupOperation for SPro40Protocol {
    const OUT_GROUP_STATE_OFFSET: usize = 0x000c;

    const ENTRY_COUNT: usize = 10;
    const HAS_VOL_HWCTL: bool = true;

    const SRC_NOTICE: u32 = SRC_SW_NOTICE;
    const DIM_MUTE_NOTICE: u32 = DIM_MUTE_SW_NOTICE;
}

impl SaffireproIoParamsOperation for SPro40Protocol {
    const AESEBU_IS_SUPPORTED: bool = false;
    const MIC_PREAMP_TRANSFORMER_IS_SUPPORTED: bool = false;
}
