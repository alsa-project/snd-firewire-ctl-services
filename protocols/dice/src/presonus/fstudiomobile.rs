// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol implementation for PreSonus FireStudio Mobile.

use super::{tcat::extension::*, tcat::tcd22xx_spec::*, *};

/// Protocol implementation of PreSonus FireStudio Mobile.
#[derive(Default, Debug)]
pub struct FStudioMobileProtocol;

impl TcatOperation for FStudioMobileProtocol {}

impl TcatGlobalSectionSpecification for FStudioMobileProtocol {}

impl TcatExtensionOperation for FStudioMobileProtocol {}

impl Tcd22xxSpecification for FStudioMobileProtocol {
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
            count: 4,
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
    ];
}
