// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Hardware specification and application protocol specific to Loud (Mackie) Onyx Blackbird.
//!
//! The module includes structure, enumeration, and trait and its implementation for hardware
//! specification and application protocol specific to Loud (Mackie) Onyx Blackbird.

use super::tcat::{extension::*, tcd22xx_spec::*};

/// Protocol implementation of Mackie Onyx Blackbird.
#[derive(Default)]
pub struct BlackbirdProtocol;

impl Tcd22xxSpecOperation for BlackbirdProtocol {
    const INPUTS: &'static [Input] = &[
        Input {
            id: SrcBlkId::Ins0,
            offset: 0,
            count: 8,
            label: None,
        },
        Input {
            id: SrcBlkId::Adat,
            offset: 0,
            count: 8,
            label: None,
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
            id: DstBlkId::Adat,
            offset: 0,
            count: 8,
            label: None,
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

    // NOTE: At low rate, digital 8 channels are composed by one optical interface in ADAT normal
    // mode. At middle rate, digital 8 channels are composed by two optical interfaces in ADAT II
    // mode. At higher rate, digital 4 channels are composed by them in ADAT IV mode. But the
    // highest rate is not available in a point of TCAT glocal protocol.
    const ADAT_CHANNELS: [u8; 3] = [8, 8, 4];
}
