// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto
use crate::tcat::extension::*;
use crate::tcat::tcd22xx_spec::*;

#[derive(Default, Debug)]
pub struct FStudioMobileState(Tcd22xxState);

impl<'a> Tcd22xxSpec<'a> for  FStudioMobileState {
    const INPUTS: &'a [Input<'a>] = &[
        Input{id: SrcBlkId::Ins0, offset: 0, count: 8, label: None},
        Input{id: SrcBlkId::Aes,  offset: 2, count: 2, label: Some("S/PDIF")},
    ];
    const OUTPUTS: &'a [Output<'a>] = &[
        Output{id: DstBlkId::Ins0, offset: 0, count: 4, label: None},
        Output{id: DstBlkId::Aes,  offset: 2, count: 2, label: Some("S/PDIF")},
    ];
    const FIXED: &'a [SrcBlk] = &[
        SrcBlk{id: SrcBlkId::Ins0, ch: 0},
        SrcBlk{id: SrcBlkId::Ins0, ch: 1},
    ];
}

impl AsRef<Tcd22xxState> for FStudioMobileState {
    fn as_ref(&self) -> &Tcd22xxState {
        &self.0
    }
}

impl AsMut<Tcd22xxState> for FStudioMobileState {
    fn as_mut(&mut self) -> &mut Tcd22xxState {
        &mut self.0
    }
}
