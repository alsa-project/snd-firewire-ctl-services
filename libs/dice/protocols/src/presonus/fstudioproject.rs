// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol implementation for PreSonus FireStudio Project.

use super::tcat::{extension::*, tcd22xx_spec::*};

/// The structure for protocol implementation of PreSonus FireStudio Project.
#[derive(Default)]
pub struct FStudioProjectProtocol;

impl Tcd22xxSpecOperation for FStudioProjectProtocol {
    const INPUTS: &'static [Input] = &[
        Input {
            id: SrcBlkId::Ins0,
            offset: 0,
            count: 8,
            label: None,
        },
        Input {
            id: SrcBlkId::Aes,
            offset: 2,
            count: 2,
            label: Some("S/PDIF"),
        },
    ];
    const OUTPUTS: &'static [Output] = &[
        Output {
            id: DstBlkId::Ins0,
            offset: 0,
            count: 8,
            label: None,
        },
        Output {
            id: DstBlkId::Aes,
            offset: 2,
            count: 2,
            label: Some("S/PDIF"),
        },
    ];
    const FIXED: &'static [SrcBlk] = &[
        SrcBlk {
            id: SrcBlkId::Ins0,
            ch: 0,
        },
        SrcBlk {
            id: SrcBlkId::Ins0,
            ch: 1,
        },
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
            ch: 4,
        },
        SrcBlk {
            id: SrcBlkId::Ins0,
            ch: 5,
        },
        SrcBlk {
            id: SrcBlkId::Ins0,
            ch: 6,
        },
        SrcBlk {
            id: SrcBlkId::Ins0,
            ch: 7,
        },
    ];
}
