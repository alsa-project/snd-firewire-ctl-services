// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol specific to Focusrite Saffire Pro 24.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by Focusrite for Saffire Pro 24.

use crate::{tcat::{extension::*, tcd22xx_spec::*}};

/// The structure for protocol implementation specific to Saffire Pro 24.
#[derive(Debug)]
pub struct SPro24Protocol;

impl Tcd22xxSpecOperation for SPro24Protocol {
    const INPUTS: &'static [Input] = &[
        Input{id: SrcBlkId::Ins0, offset: 2, count: 2, label: Some("Mic")},
        Input{id: SrcBlkId::Ins0, offset: 0, count: 2, label: Some("Line")},
        Input{id: SrcBlkId::Aes, offset: 6, count: 2, label: Some("S/PDIF-coax")},
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
