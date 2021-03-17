// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto
use glib::Error;

use hinawa::{SndUnit, SndUnitExt, FwFcp, FwFcpExt};

use alsactl::{ElemId, ElemValue};

use core::card_cntr::{CardCntr, CtlModel, NotifyModel};

use super::common_ctl::CommonCtl;

#[derive(Default, Debug)]
pub struct LinkFwModel {
    avc: FwFcp,
    common_ctl: CommonCtl,
}

const FCP_TIMEOUT_MS: u32 = 100;

impl CtlModel<SndUnit> for LinkFwModel {
    fn load(&mut self, unit: &SndUnit, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.avc.bind(&unit.get_node())?;

        self.common_ctl.load(&self.avc, card_cntr, FCP_TIMEOUT_MS)?;

        Ok(())
    }

    fn read(&mut self, _: &SndUnit, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        self.common_ctl.read(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)
    }

    fn write(&mut self, unit: &SndUnit, elem_id: &ElemId, _: &ElemValue, new: &ElemValue)
        -> Result<bool, Error>
    {
        self.common_ctl.write(unit, &self.avc, elem_id, new, FCP_TIMEOUT_MS)
    }
}

impl NotifyModel<SndUnit, bool> for LinkFwModel {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.common_ctl.notified_elem_list);
    }

    fn parse_notification(&mut self, _: &SndUnit, _: &bool) -> Result<(), Error> {
        Ok(())
    }

    fn read_notified_elem(&mut self, _: &SndUnit, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        self.common_ctl.read(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)
    }
}
