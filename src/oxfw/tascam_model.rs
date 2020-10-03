// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use hinawa::{FwFcpExt, SndUnitExt};

use crate::card_cntr;

use crate::ta1394::{Ta1394Avc, AvcAddr};
use crate::ta1394::general::UnitInfo;

use super::tascam_proto::TascamAvc;

use super::common_ctl::CommonCtl;

pub struct TascamModel{
    avc: TascamAvc,
    common_ctl: CommonCtl,
}

impl TascamModel {
    const FCP_TIMEOUT_MS: u32 = 100;

    pub fn new() -> Self {
        TascamModel{
            avc: TascamAvc::new(),
            common_ctl: CommonCtl::new(),
        }
    }
}

impl card_cntr::CtlModel<hinawa::SndUnit> for TascamModel {
    fn load(&mut self, unit: &hinawa::SndUnit, card_cntr: &mut card_cntr::CardCntr) -> Result<(), Error> {
        self.avc.fcp.bind(&unit.get_node())?;

        let mut op = UnitInfo::new();
        self.avc.status(&AvcAddr::Unit, &mut op, Self::FCP_TIMEOUT_MS)?;
        self.avc.company_id.copy_from_slice(&op.company_id);

        self.common_ctl.load(&self.avc, card_cntr, Self::FCP_TIMEOUT_MS)?;

        Ok(())
    }

    fn read(&mut self, _: &hinawa::SndUnit, elem_id: &alsactl::ElemId,
            elem_value: &mut alsactl::ElemValue)
        -> Result<bool, Error>
    {
        if self.common_ctl.read(&self.avc, elem_id, elem_value, Self::FCP_TIMEOUT_MS)? {
            return Ok(true);
        } else {
            Ok(false)
        }
    }

    fn write(&mut self, unit: &hinawa::SndUnit, elem_id: &alsactl::ElemId, _: &alsactl::ElemValue,
             new: &alsactl::ElemValue) -> Result<bool, Error>
    {
        if self.common_ctl.write(unit, &self.avc, elem_id, new, Self::FCP_TIMEOUT_MS)? {
            return Ok(true);
        } else {
            Ok(false)
        }
    }
}
