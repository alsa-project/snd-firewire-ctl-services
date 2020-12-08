// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use alsactl::{ElemId, ElemValue};

use hinawa::FwReq;
use hinawa::{SndDice, SndUnitExt};

use core::card_cntr::*;

use dice_protocols::tcat::{*, global_section::*};
use dice_protocols::tcat::extension::*;

use super::common_ctl::*;
use super::tcd22xx_spec::*;

#[derive(Default)]
pub struct ExtensionModel{
    proto: FwReq,
    sections: GeneralSections,
    ctl: CommonCtl,
}

const TIMEOUT_MS: u32 = 20;

impl CtlModel<SndDice> for ExtensionModel {
    fn load(&mut self, unit: &SndDice, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let node = unit.get_node();

        self.sections = self.proto.read_general_sections(&node, TIMEOUT_MS)?;
        let caps = self.proto.read_clock_caps(&node, &self.sections, TIMEOUT_MS)?;
        let src_labels = self.proto.read_clock_source_labels(&node, &self.sections, TIMEOUT_MS)?;
        self.ctl.load(card_cntr, &caps, &src_labels)
    }

    fn read(&mut self, unit: &SndDice, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        self.ctl.read(unit, &self.proto, &self.sections, elem_id, elem_value, TIMEOUT_MS)
    }

    fn write(&mut self, unit: &SndDice, elem_id: &ElemId, old: &ElemValue, new: &ElemValue)
        -> Result<bool, Error>
    {
        self.ctl.write(unit, &self.proto, &self.sections, elem_id, old, new, TIMEOUT_MS)
    }
}

impl NotifyModel<SndDice, u32> for ExtensionModel {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.ctl.notified_elem_list);
    }

    fn parse_notification(&mut self, unit: &SndDice, msg: &u32) -> Result<(), Error> {
        self.ctl.parse_notification(unit, &self.proto, &self.sections, *msg, TIMEOUT_MS)
    }

    fn read_notified_elem(&mut self, _: &SndDice, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        self.ctl.read_notified_elem(elem_id, elem_value)
    }
}

impl MeasureModel<hinawa::SndDice> for ExtensionModel {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.ctl.measured_elem_list);
    }

    fn measure_states(&mut self, unit: &SndDice) -> Result<(), Error> {
        self.ctl.measure_states(unit, &self.proto, &self.sections, TIMEOUT_MS)
    }

    fn measure_elem(&mut self, _: &SndDice, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        self.ctl.measure_elem(elem_id, elem_value)
    }
}

#[derive(Default, Debug)]
struct ExtensionState(Tcd22xxState);

impl<'a> Tcd22xxSpec<'a> for  ExtensionState {
    const INPUTS: &'a [Input<'a>] = &[
        Input{id: SrcBlkId::Ins0, offset: 0, count: 16, label: None},
        Input{id: SrcBlkId::Ins1, offset: 0, count: 16, label: None},
        Input{id: SrcBlkId::Aes,  offset: 0, count: 2, label: None},
        Input{id: SrcBlkId::Aes,  offset: 2, count: 2, label: None},
        Input{id: SrcBlkId::Aes,  offset: 4, count: 2, label: None},
        Input{id: SrcBlkId::Aes,  offset: 6, count: 2, label: None},
        Input{id: SrcBlkId::Adat, offset: 0, count: 8, label: None},
        Input{id: SrcBlkId::Adat, offset: 8, count: 8, label: None},
    ];
    const OUTPUTS: &'a [Output<'a>] = &[
        Output{id: DstBlkId::Ins0, offset: 0, count: 16, label: None},
        Output{id: DstBlkId::Ins1, offset: 0, count: 16, label: None},
        Output{id: DstBlkId::Aes,  offset: 0, count: 2, label: None},
        Output{id: DstBlkId::Aes,  offset: 2, count: 2, label: None},
        Output{id: DstBlkId::Aes,  offset: 4, count: 2, label: None},
        Output{id: DstBlkId::Aes,  offset: 6, count: 2, label: None},
        Output{id: DstBlkId::Adat, offset: 0, count: 8, label: None},
        Output{id: DstBlkId::Adat, offset: 8, count: 8, label: None},
    ];
    const FIXED: &'a [SrcBlk] = &[
        SrcBlk{id: SrcBlkId::Ins0, ch: 0},
        SrcBlk{id: SrcBlkId::Ins0, ch: 1},
        SrcBlk{id: SrcBlkId::Ins0, ch: 2},
        SrcBlk{id: SrcBlkId::Ins0, ch: 3},
        SrcBlk{id: SrcBlkId::Ins1, ch: 0},
        SrcBlk{id: SrcBlkId::Ins1, ch: 1},
        SrcBlk{id: SrcBlkId::Ins1, ch: 2},
        SrcBlk{id: SrcBlkId::Ins1, ch: 3},
    ];
}

impl AsRef<Tcd22xxState> for ExtensionState {
    fn as_ref(&self) -> &Tcd22xxState {
        &self.0
    }
}

impl AsMut<Tcd22xxState> for ExtensionState {
    fn as_mut(&mut self) -> &mut Tcd22xxState {
        &mut self.0
    }
}
