// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use hinawa::{FwFcpExt, SndUnitExt};

use crate::card_cntr;

use crate::ta1394::{Ta1394Avc, AvcAddr};
use crate::ta1394::general::UnitInfo;

use super::tascam_proto::TascamAvc;

pub struct TascamModel{
    avc: TascamAvc,
}

impl TascamModel {
    const FCP_TIMEOUT_MS: u32 = 100;

    pub fn new() -> Self {
        TascamModel{
            avc: TascamAvc::new(),
        }
    }
}

impl card_cntr::CtlModel<hinawa::SndUnit> for TascamModel {
    fn load(&mut self, unit: &hinawa::SndUnit, _: &mut card_cntr::CardCntr) -> Result<(), Error> {
        self.avc.fcp.bind(&unit.get_node())?;

        let mut op = UnitInfo::new();
        self.avc.status(&AvcAddr::Unit, &mut op, Self::FCP_TIMEOUT_MS)?;
        self.avc.company_id.copy_from_slice(&op.company_id);

        Ok(())
    }

    fn read(&mut self, _: &hinawa::SndUnit, _: &alsactl::ElemId, _: &mut alsactl::ElemValue)
        -> Result<bool, Error>
    {
        Ok(false)
    }

    fn write(&mut self, _: &hinawa::SndUnit, _: &alsactl::ElemId, _: &alsactl::ElemValue,
             _: &alsactl::ElemValue) -> Result<bool, Error>
    {
        Ok(false)
    }
}
