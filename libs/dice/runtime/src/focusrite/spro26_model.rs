// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto
use glib::Error;

use alsactl::{ElemId, ElemValue};

use hinawa::FwReq;
use hinawa::{SndDice, SndUnitExt};

use core::card_cntr::*;

use dice_protocols::tcat::{*, global_section::*};
use dice_protocols::tcat::extension::*;
use dice_protocols::focusrite::spro26::*;

use crate::common_ctl::*;
use crate::tcd22xx_ctl::*;

use super::*;

#[derive(Default)]
pub struct SPro26Model {
    req: FwReq,
    sections: GeneralSections,
    extension_sections: ExtensionSections,
    ctl: CommonCtl,
    tcd22xx_ctl: Tcd22xxCtl<SPro26State>,
    out_grp_ctl: OutGroupCtl,
}

const TIMEOUT_MS: u32 = 20;

#[derive(Default)]
struct OutGroupCtl(OutGroupState, Vec<ElemId>);

impl AsRef<OutGroupState> for OutGroupCtl {
    fn as_ref(&self) -> &OutGroupState {
        &self.0
    }
}

impl AsMut<OutGroupState> for OutGroupCtl {
    fn as_mut(&mut self) -> &mut OutGroupState {
        &mut self.0
    }
}

impl OutGroupCtlOperation<SPro26Protocol> for OutGroupCtl {}

impl CtlModel<SndDice> for SPro26Model {
    fn load(&mut self, unit: &mut SndDice, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let mut node = unit.get_node();

        self.sections = self.req.read_general_sections(&mut node, TIMEOUT_MS)?;
        let caps = self.req.read_clock_caps(&mut node, &self.sections, TIMEOUT_MS)?;
        let src_labels = self.req.read_clock_source_labels(&mut node, &self.sections, TIMEOUT_MS)?;
        self.ctl.load(card_cntr, &caps, &src_labels)?;

        self.extension_sections = self.req.read_extension_sections(&mut node, TIMEOUT_MS)?;
        self.tcd22xx_ctl.load(unit, &mut self.req, &self.extension_sections, &caps, &src_labels,
                          TIMEOUT_MS, card_cntr)?;

        self.tcd22xx_ctl.cache(unit, &mut self.req, &self.sections, &self.extension_sections, TIMEOUT_MS)?;

        self.out_grp_ctl.load(card_cntr, unit, &mut self.req, &self.extension_sections, TIMEOUT_MS)
            .map(|mut elem_id_list| self.out_grp_ctl.1.append(&mut elem_id_list))?;

        Ok(())
    }

    fn read(&mut self, unit: &mut SndDice, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        if self.ctl.read(unit, &mut self.req, &self.sections, elem_id, elem_value, TIMEOUT_MS)? {
            Ok(true)
        } else if self.tcd22xx_ctl.read(unit, &mut self.req, &self.extension_sections, elem_id,
                                    elem_value, TIMEOUT_MS)? {
            Ok(true)
        } else if self.out_grp_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(&mut self, unit: &mut SndDice, elem_id: &ElemId, old: &ElemValue, new: &ElemValue)
        -> Result<bool, Error>
    {
        if self.ctl.write(unit, &mut self.req, &self.sections, elem_id, old, new, TIMEOUT_MS)? {
            Ok(true)
        } else if self.tcd22xx_ctl.write(unit, &mut self.req, &self.extension_sections, elem_id,
                                     old, new, TIMEOUT_MS)? {
            Ok(true)
        } else if self.out_grp_ctl.write(unit, &mut self.req, &self.extension_sections,
                                         elem_id, new, TIMEOUT_MS)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl NotifyModel<SndDice, u32> for SPro26Model {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.ctl.notified_elem_list);
        self.tcd22xx_ctl.get_notified_elem_list(elem_id_list);
        elem_id_list.extend_from_slice(&self.out_grp_ctl.1);
    }

    fn parse_notification(&mut self, unit: &mut SndDice, msg: &u32) -> Result<(), Error> {
        self.ctl.parse_notification(unit, &mut self.req, &self.sections, *msg, TIMEOUT_MS)?;
        self.tcd22xx_ctl.parse_notification(unit, &mut self.req, &self.sections,
                                        &self.extension_sections, TIMEOUT_MS, *msg)?;
        self.out_grp_ctl.parse_notification(unit, &mut self.req, &self.extension_sections,
                                            *msg, TIMEOUT_MS)?;
        Ok(())
    }

    fn read_notified_elem(&mut self, _: &SndDice, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        if self.ctl.read_notified_elem(elem_id, elem_value)? {
            Ok(true)
        } else if self.tcd22xx_ctl.read_notified_elem(elem_id, elem_value)? {
            Ok(true)
        } else if self.out_grp_ctl.read_notified_elem(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl MeasureModel<SndDice> for SPro26Model {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.ctl.measured_elem_list);
        self.tcd22xx_ctl.get_measured_elem_list(elem_id_list);
    }

    fn measure_states(&mut self, unit: &mut SndDice) -> Result<(), Error> {
        self.ctl.measure_states(unit, &mut self.req, &self.sections, TIMEOUT_MS)?;
        self.tcd22xx_ctl.measure_states(unit, &mut self.req, &self.extension_sections, TIMEOUT_MS)?;
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
