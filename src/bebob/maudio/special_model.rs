// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use hinawa::{FwFcpExt, SndUnitExt};

use crate::card_cntr;

use crate::bebob::BebobAvc;

use super::special_ctls::ClkCtl;

pub struct SpecialModel {
    avc: BebobAvc,
    clk_ctl: ClkCtl,
}

impl SpecialModel {
    pub fn new(is_fw1814: bool) -> Self {
        SpecialModel {
            avc: BebobAvc::new(),
            clk_ctl: ClkCtl::new(is_fw1814),
        }
    }
}

impl card_cntr::CtlModel<hinawa::SndUnit> for SpecialModel {
    fn load(&mut self, unit: &hinawa::SndUnit, card_cntr: &mut card_cntr::CardCntr) -> Result<(), Error> {
        self.avc.fcp.bind(&unit.get_node())?;

        self.clk_ctl.load(card_cntr)?;

        Ok(())
    }

    fn read(&mut self, _: &hinawa::SndUnit, elem_id: &alsactl::ElemId, elem_value: &mut alsactl::ElemValue)
        -> Result<bool, Error>
    {
        if self.clk_ctl.read(&self.avc, elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(&mut self, unit: &hinawa::SndUnit, elem_id: &alsactl::ElemId,
             old: &alsactl::ElemValue, new: &alsactl::ElemValue)
        -> Result<bool, Error>
    {
        if self.clk_ctl.write(unit, &self.avc, elem_id, old, new)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl card_cntr::MeasureModel<hinawa::SndUnit> for SpecialModel {
    fn get_measure_elem_list(&mut self, _: &mut Vec<alsactl::ElemId>) {
    }

    fn measure_states(&mut self, _: &hinawa::SndUnit) -> Result<(), Error> {
        Ok(())
    }

    fn measure_elem(&mut self, _: &hinawa::SndUnit, _: &alsactl::ElemId, _: &mut alsactl::ElemValue)
        -> Result<bool, Error>
    {
        Ok(false)
    }
}

impl card_cntr::NotifyModel<hinawa::SndUnit, bool> for SpecialModel {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<alsactl::ElemId>) {
        elem_id_list.extend_from_slice(&self.clk_ctl.notified_elem_list);
    }

    fn parse_notification(&mut self, _: &hinawa::SndUnit, _: &bool) -> Result<(), Error> {
        Ok(())
    }

    fn read_notified_elem(&mut self, _: &hinawa::SndUnit, elem_id: &alsactl::ElemId, elem_value: &mut alsactl::ElemValue)
        -> Result<bool, Error>
    {
        self.clk_ctl.read(&self.avc, elem_id, elem_value)
    }
}
