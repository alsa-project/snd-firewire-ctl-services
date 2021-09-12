// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol specific to Focusrite Saffire Pro 26.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by Focusrite for Saffire Pro 26.

use crate::tcat::extension::*;
use crate::tcat::tcd22xx_spec::*;

use super::*;

/// The structure for protocol implementation specific to Saffire Pro 26.
#[derive(Default)]
pub struct SPro26Protocol;

/// The structure to represent state of TCD22xx on Saffire Pro 26.
#[derive(Debug)]
pub struct SPro26State{
    tcd22xx: Tcd22xxState,
}

impl Default for SPro26State {
    fn default() -> Self {
        SPro26State{
            tcd22xx: Tcd22xxState::default(),
        }
    }
}

impl Tcd22xxSpec for SPro26State {
    const INPUTS: &'static [Input] = &[
        Input{id: SrcBlkId::Ins0, offset: 0, count: 6, label: None},
        Input{id: SrcBlkId::Aes, offset: 4, count: 2, label: Some("S/PDIF-coax")},
        // NOTE: share the same optical interface.
        Input{id: SrcBlkId::Adat, offset: 0, count: 8, label: None},
        Input{id: SrcBlkId::Aes, offset: 6, count: 2, label: Some("S/PDIF-opt")},
    ];
    const OUTPUTS: &'static [Output] = &[
        Output{id: DstBlkId::Ins0, offset: 0, count: 6, label: None},
        Output{id: DstBlkId::Aes, offset: 4, count: 2, label: Some("S/PDIF-coax")},
        Output{id: DstBlkId::Adat, offset: 0, count: 8, label: None},
    ];
    // NOTE: The first 6 entries in router section are used to display hardware metering.
    const FIXED: &'static [SrcBlk] = &[
        SrcBlk{id: SrcBlkId::Ins0, ch: 0},
        SrcBlk{id: SrcBlkId::Ins0, ch: 1},
        SrcBlk{id: SrcBlkId::Ins0, ch: 2},
        SrcBlk{id: SrcBlkId::Ins0, ch: 3},
        SrcBlk{id: SrcBlkId::Ins0, ch: 4},
        SrcBlk{id: SrcBlkId::Ins0, ch: 5},
    ];
}

impl AsMut<Tcd22xxState> for SPro26State {
    fn as_mut(&mut self) -> &mut Tcd22xxState {
        &mut self.tcd22xx
    }
}

impl AsRef<Tcd22xxState> for SPro26State {
    fn as_ref(&self) -> &Tcd22xxState {
        &self.tcd22xx
    }
}

const SW_NOTICE_OFFSET: usize = 0x000c;

const SRC_SW_NOTICE: u32 = 0x00000001;
const DIM_MUTE_SW_NOTICE: u32 = 0x00000002;

impl SaffireproOutGroupOperation for SPro26Protocol {
    const ENTRY_COUNT: usize = 6;
    const HAS_VOL_HWCTL: bool = false;
    const OUT_CTL_OFFSET: usize = 0x0010;
    const SW_NOTICE_OFFSET: usize = SW_NOTICE_OFFSET;

    const SRC_NOTICE: u32 = SRC_SW_NOTICE;
    const DIM_MUTE_NOTICE: u32 = DIM_MUTE_SW_NOTICE;
}
