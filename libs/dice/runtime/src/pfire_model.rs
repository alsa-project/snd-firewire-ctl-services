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
use super::tcd22xx_ctl::*;

#[derive(Default)]
pub struct PfireModel<S>
    where for<'a> S: AsRef<Tcd22xxState> + AsMut<Tcd22xxState> + Tcd22xxSpec<'a>,
{
    proto: FwReq,
    sections: GeneralSections,
    extension_sections: ExtensionSections,
    ctl: CommonCtl,
    tcd22xx_ctl: Tcd22xxCtl<S>,
}

pub type Pfire2626Model = PfireModel<Pfire2626State>;

const TIMEOUT_MS: u32 = 20;

impl<S> CtlModel<SndDice> for PfireModel<S>
    where for<'a> S: AsRef<Tcd22xxState> + AsMut<Tcd22xxState> + Tcd22xxSpec<'a>,
{
    fn load(&mut self, unit: &SndDice, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let node = unit.get_node();

        self.sections = self.proto.read_general_sections(&node, TIMEOUT_MS)?;
        let caps = self.proto.read_clock_caps(&node, &self.sections, TIMEOUT_MS)?;
        let src_labels = self.proto.read_clock_source_labels(&node, &self.sections, TIMEOUT_MS)?;
        self.ctl.load(card_cntr, &caps, &src_labels)?;

        self.extension_sections = self.proto.read_extension_sections(&node, TIMEOUT_MS)?;
        self.tcd22xx_ctl.load(unit, &self.proto, &self.extension_sections, &caps, &src_labels,
                          TIMEOUT_MS, card_cntr)?;

        self.tcd22xx_ctl.cache(unit, &self.proto, &self.sections, &self.extension_sections, TIMEOUT_MS)?;

        Ok(())
    }

    fn read(&mut self, unit: &SndDice, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        if self.ctl.read(unit, &self.proto, &self.sections, elem_id, elem_value, TIMEOUT_MS)? {
            Ok(true)
        } else if self.tcd22xx_ctl.read(unit, &self.proto, &self.extension_sections, elem_id,
                                    elem_value, TIMEOUT_MS)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(&mut self, unit: &SndDice, elem_id: &ElemId, old: &ElemValue, new: &ElemValue)
        -> Result<bool, Error>
    {
        if self.ctl.write(unit, &self.proto, &self.sections, elem_id, old, new, TIMEOUT_MS)? {
            Ok(true)
        } else if self.tcd22xx_ctl.write(unit, &self.proto, &self.extension_sections, elem_id,
                                     old, new, TIMEOUT_MS)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl<S> NotifyModel<SndDice, u32> for PfireModel<S>
    where for<'a> S: AsRef<Tcd22xxState> + AsMut<Tcd22xxState> + Tcd22xxSpec<'a>,
{
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.ctl.notified_elem_list);
        self.tcd22xx_ctl.get_notified_elem_list(elem_id_list);
    }

    fn parse_notification(&mut self, unit: &SndDice, msg: &u32) -> Result<(), Error> {
        self.ctl.parse_notification(unit, &self.proto, &self.sections, *msg, TIMEOUT_MS)?;
        self.tcd22xx_ctl.parse_notification(unit, &self.proto, &self.sections,
                                        &self.extension_sections, TIMEOUT_MS, *msg)?;
        Ok(())
    }

    fn read_notified_elem(&mut self, _: &SndDice, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        if self.ctl.read_notified_elem(elem_id, elem_value)? {
            Ok(true)
        } else if self.tcd22xx_ctl.read_notified_elem(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl<S> MeasureModel<hinawa::SndDice> for PfireModel<S>
    where for<'a> S: AsRef<Tcd22xxState> + AsMut<Tcd22xxState> + Tcd22xxSpec<'a>,
{
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.ctl.measured_elem_list);
        self.tcd22xx_ctl.get_measured_elem_list(elem_id_list);
    }

    fn measure_states(&mut self, unit: &SndDice) -> Result<(), Error> {
        self.ctl.measure_states(unit, &self.proto, &self.sections, TIMEOUT_MS)?;
        self.tcd22xx_ctl.measure_states(unit, &self.proto, &self.extension_sections, TIMEOUT_MS)?;
        Ok(())
    }

    fn measure_elem(&mut self, _: &SndDice, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        if self.ctl.measure_elem(elem_id, elem_value)? {
            Ok(true)
        } else if self.tcd22xx_ctl.measure_elem(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[derive(Default, Debug)]
pub struct Pfire2626State(Tcd22xxState);

impl<'a> Tcd22xxSpec<'a> for Pfire2626State {
    const INPUTS: &'a [Input<'a>] = &[
        Input{id: SrcBlkId::Ins0, offset: 0, count: 8, label: None},
        Input{id: SrcBlkId::Adat, offset: 0, count: 8, label: None},
        Input{id: SrcBlkId::Adat, offset: 0, count: 8, label: None},
        Input{id: SrcBlkId::Aes, offset: 0, count: 2, label: None},
    ];
    const OUTPUTS: &'a [Output<'a>] = &[
        Output{id: DstBlkId::Ins0, offset: 0, count: 8, label: None},
        Output{id: DstBlkId::Adat, offset: 0, count: 8, label: None},
        Output{id: DstBlkId::Adat, offset: 0, count: 8, label: None},
        Output{id: DstBlkId::Aes, offset: 0, count: 2, label: None},
    ];
    const FIXED: &'a [SrcBlk] = &[
        SrcBlk{id: SrcBlkId::Ins0, ch: 0},
        SrcBlk{id: SrcBlkId::Ins0, ch: 1},
        SrcBlk{id: SrcBlkId::Ins0, ch: 2},
        SrcBlk{id: SrcBlkId::Ins0, ch: 3},
        SrcBlk{id: SrcBlkId::Ins0, ch: 4},
        SrcBlk{id: SrcBlkId::Ins0, ch: 5},
        SrcBlk{id: SrcBlkId::Ins0, ch: 6},
        SrcBlk{id: SrcBlkId::Ins0, ch: 7},
    ];
}

impl AsMut<Tcd22xxState> for Pfire2626State {
    fn as_mut(&mut self) -> &mut Tcd22xxState {
        &mut self.0
    }
}

impl AsRef<Tcd22xxState> for Pfire2626State {
    fn as_ref(&self) -> &Tcd22xxState {
        &self.0
    }
}
