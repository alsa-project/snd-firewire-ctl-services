// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use hinawa::{SndUnitExt, FwFcpExt};

use core::card_cntr;

use super::common_ctl::CommonCtl;

pub struct CommonModel {
    avc: hinawa::FwFcp,
    common_ctl: CommonCtl,
}

impl<'a> CommonModel {
    const FCP_TIMEOUT_MS: u32 = 100;

    pub fn new() -> Self {
        CommonModel{
            avc: hinawa::FwFcp::new(),
            common_ctl: CommonCtl::new(),
        }
    }
}

impl card_cntr::CtlModel<hinawa::SndUnit> for CommonModel {
    fn load(&mut self, unit: &hinawa::SndUnit, card_cntr: &mut card_cntr::CardCntr) -> Result<(), Error> {
        self.avc.bind(&unit.get_node())?;

        self.common_ctl.load(&self.avc, card_cntr, Self::FCP_TIMEOUT_MS)?;

        Ok(())
    }

    fn read(&mut self, _: &hinawa::SndUnit, elem_id: &alsactl::ElemId, elem_value: &mut alsactl::ElemValue)
        -> Result<bool, Error>
    {
        if self.common_ctl.read(&self.avc, elem_id, elem_value, Self::FCP_TIMEOUT_MS)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(&mut self, unit: &hinawa::SndUnit, elem_id: &alsactl::ElemId, _: &alsactl::ElemValue,
             new: &alsactl::ElemValue) -> Result<bool, Error>
    {
        if self.common_ctl.write(unit, &self.avc, elem_id, new, Self::FCP_TIMEOUT_MS)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl card_cntr::NotifyModel<hinawa::SndUnit, bool> for CommonModel {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<alsactl::ElemId>) {
        elem_id_list.extend_from_slice(&self.common_ctl.notified_elem_list);
    }

    fn parse_notification(&mut self, _: &hinawa::SndUnit, _: &bool) -> Result<(), Error> {
        Ok(())
    }

    fn read_notified_elem(&mut self, _: &hinawa::SndUnit, elem_id: &alsactl::ElemId, elem_value: &mut alsactl::ElemValue)
        -> Result<bool, Error>
    {
        self.common_ctl.read(&self.avc, elem_id, elem_value, Self::FCP_TIMEOUT_MS)
    }
}
