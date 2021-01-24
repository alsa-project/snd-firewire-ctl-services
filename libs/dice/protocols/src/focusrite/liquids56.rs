// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol specific to Focusrite Liquid Saffire 56.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by Focusrite for Liquid Saffire 56.

use crate::tcat::extension::*;
use crate::tcat::tcd22xx_spec::*;

/// The structure to represent state of TCD22xx on Liquid Saffire 56.
#[derive(Default, Debug)]
pub struct LiquidS56State{
    tcd22xx: Tcd22xxState,
}

impl<'a> Tcd22xxSpec<'a> for LiquidS56State {
    const INPUTS: &'a [Input<'a>] = &[
        Input{id: SrcBlkId::Ins0, offset: 0, count: 2, label: None},
        Input{id: SrcBlkId::Ins1, offset: 0, count: 6, label: None},
        Input{id: SrcBlkId::Adat, offset: 0, count: 8, label: None},
        Input{id: SrcBlkId::Aes, offset: 0, count: 2, label: Some("S/PDIF-coax")},
        // NOTE: share the same optical interface.
        Input{id: SrcBlkId::Adat, offset: 8, count: 8, label: None},
        Input{id: SrcBlkId::Aes, offset: 6, count: 2, label: Some("S/PDIF-opt")},
    ];
    const OUTPUTS: &'a [Output<'a>] = &[
        Output{id: DstBlkId::Ins0, offset: 0, count: 2, label: None},
        Output{id: DstBlkId::Ins1, offset: 0, count: 8, label: None},
        Output{id: DstBlkId::Adat, offset: 0, count: 8, label: None},
        Output{id: DstBlkId::Aes, offset: 0, count: 2, label: Some("S/PDIF-coax")},
        // NOTE: share the same optical interface.
        Output{id: DstBlkId::Adat, offset: 8, count: 8, label: None},
        Output{id: DstBlkId::Aes, offset: 6, count: 2, label: Some("S/PDIF-opt")},
    ];
    // NOTE: The 8 entries are selected by unique protocol from the first 26 entries in router
    // section are used to display hardware metering.
    const FIXED: &'a [SrcBlk] = &[
        SrcBlk{id: SrcBlkId::Ins1, ch: 0},
        SrcBlk{id: SrcBlkId::Ins1, ch: 1},
        SrcBlk{id: SrcBlkId::Ins1, ch: 2},
        SrcBlk{id: SrcBlkId::Ins1, ch: 3},
        SrcBlk{id: SrcBlkId::Ins1, ch: 4},
        SrcBlk{id: SrcBlkId::Ins1, ch: 5},
        SrcBlk{id: SrcBlkId::Ins1, ch: 6},
        SrcBlk{id: SrcBlkId::Ins1, ch: 7},
        SrcBlk{id: SrcBlkId::Aes, ch: 0},
        SrcBlk{id: SrcBlkId::Aes, ch: 1},
        SrcBlk{id: SrcBlkId::Adat, ch: 0},
        SrcBlk{id: SrcBlkId::Adat, ch: 1},
        SrcBlk{id: SrcBlkId::Adat, ch: 2},
        SrcBlk{id: SrcBlkId::Adat, ch: 3},
        SrcBlk{id: SrcBlkId::Adat, ch: 4},
        SrcBlk{id: SrcBlkId::Adat, ch: 5},
        SrcBlk{id: SrcBlkId::Adat, ch: 6},
        SrcBlk{id: SrcBlkId::Adat, ch: 7},
        SrcBlk{id: SrcBlkId::Adat, ch: 8},
        SrcBlk{id: SrcBlkId::Adat, ch: 9},
        SrcBlk{id: SrcBlkId::Adat, ch: 10},
        SrcBlk{id: SrcBlkId::Adat, ch: 11},
        SrcBlk{id: SrcBlkId::Adat, ch: 12},
        SrcBlk{id: SrcBlkId::Adat, ch: 13},
        SrcBlk{id: SrcBlkId::Adat, ch: 14},
        SrcBlk{id: SrcBlkId::Adat, ch: 15},
    ];
}

impl AsMut<Tcd22xxState> for LiquidS56State {
    fn as_mut(&mut self) -> &mut Tcd22xxState {
        &mut self.tcd22xx
    }
}

impl AsRef<Tcd22xxState> for LiquidS56State {
    fn as_ref(&self) -> &Tcd22xxState {
        &self.tcd22xx
    }
}
