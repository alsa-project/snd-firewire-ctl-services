// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto
use glib::Error;

use alsactl::{ElemId, ElemValue};

use hinawa::FwReq;
use hinawa::{SndDice, SndUnitExt};

use core::card_cntr::*;

use dice_protocols::tcat::{*, global_section::*};
use dice_protocols::tcat::extension::*;
use dice_protocols::focusrite::liquids56::*;

use crate::common_ctl::*;
use crate::tcd22xx_ctl::*;

use super::out_grp_ctl::*;

#[derive(Default)]
pub struct LiquidS56Model {
    proto: FwReq,
    sections: GeneralSections,
    extension_sections: ExtensionSections,
    ctl: CommonCtl,
    tcd22xx_ctl: Tcd22xxCtl<LiquidS56State>,
    out_grp_ctl: OutGroupCtl,
}

const TIMEOUT_MS: u32 = 20;

impl CtlModel<SndDice> for LiquidS56Model {
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

        self.out_grp_ctl.load(card_cntr, unit, &self.proto, &self.extension_sections,
                              &mut self.tcd22xx_ctl.state, TIMEOUT_MS)?;

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
        } else if self.out_grp_ctl.read(&self.tcd22xx_ctl.state, elem_id, elem_value)? {
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
        } else if self.out_grp_ctl.write(unit, &self.proto, &self.extension_sections,
                                         &mut self.tcd22xx_ctl.state, elem_id, new, TIMEOUT_MS)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl NotifyModel<SndDice, u32> for LiquidS56Model {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.ctl.notified_elem_list);
        self.tcd22xx_ctl.get_notified_elem_list(elem_id_list);
        self.out_grp_ctl.get_notified_elem_list(elem_id_list);
    }

    fn parse_notification(&mut self, unit: &SndDice, msg: &u32) -> Result<(), Error> {
        self.ctl.parse_notification(unit, &self.proto, &self.sections, *msg, TIMEOUT_MS)?;
        self.tcd22xx_ctl.parse_notification(unit, &self.proto, &self.sections,
                                        &self.extension_sections, TIMEOUT_MS, *msg)?;
        self.out_grp_ctl.parse_notification(unit, &self.proto, &self.extension_sections,
                                            &mut self.tcd22xx_ctl.state, *msg, TIMEOUT_MS)?;
        Ok(())
    }

    fn read_notified_elem(&mut self, _: &SndDice, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        if self.ctl.read_notified_elem(elem_id, elem_value)? {
            Ok(true)
        } else if self.tcd22xx_ctl.read_notified_elem(elem_id, elem_value)? {
            Ok(true)
        } else if self.out_grp_ctl.read_notified_elem(&self.tcd22xx_ctl.state, elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl MeasureModel<hinawa::SndDice> for LiquidS56Model {
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
