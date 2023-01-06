// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2023 Takashi Sakamoto

//! Protocol implementation for PreSonus FireStudio Tube.

use super::{tcat::extension::*, tcat::tcd22xx_spec::*, *};

/// Protocol implementation of PreSonus FireStudio Tube.
#[derive(Default, Debug)]
pub struct FStudioTubeProtocol;

impl TcatOperation for FStudioTubeProtocol {}

impl TcatGlobalSectionSpecification for FStudioTubeProtocol {}

impl TcatExtensionOperation for FStudioTubeProtocol {}

impl Tcd22xxSpecification for FStudioTubeProtocol {
    const INPUTS: &'static [Input] = &[Input {
        id: SrcBlkId::Ins0,
        offset: 0,
        count: 16,
        label: None,
    }];
    const OUTPUTS: &'static [Output] = &[Output {
        id: DstBlkId::Ins0,
        offset: 0,
        count: 8,
        label: None,
    }];
    // MEMO: Analog input 14/15 for Tube amplifier is always used for display metering ignoring
    // router entries.
    const FIXED: &'static [SrcBlk] = &[];
}
