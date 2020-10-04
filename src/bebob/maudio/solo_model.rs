// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use hinawa::{FwFcpExt, SndUnitExt};

use crate::card_cntr;
use card_cntr::{CtlModel, MeasureModel};

use crate::ta1394::{AvcAddr, Ta1394Avc};
use crate::ta1394::general::UnitInfo;

use crate::bebob::BebobAvc;

use super::common_proto::FCP_TIMEOUT_MS;

pub struct SoloModel{
    avc: BebobAvc,
}

impl SoloModel {
    pub fn new() -> Self {
        SoloModel{
            avc: BebobAvc::new(),
        }
    }
}

impl CtlModel<hinawa::SndUnit> for SoloModel {
    fn load(&mut self, unit: &hinawa::SndUnit, _: &mut card_cntr::CardCntr) -> Result<(), Error> {
        self.avc.fcp.bind(&unit.get_node())?;

        let mut op = UnitInfo::new();
        self.avc.status(&AvcAddr::Unit, &mut op, FCP_TIMEOUT_MS)?;
        self.avc.company_id = op.company_id;

        Ok(())
    }

    fn read(&mut self, _: &hinawa::SndUnit, _: &alsactl::ElemId, _: &mut alsactl::ElemValue)
        -> Result<bool, Error>
    {
        Ok(false)
    }

    fn write(&mut self, _: &hinawa::SndUnit, _: &alsactl::ElemId, _: &alsactl::ElemValue, _: &alsactl::ElemValue)
        -> Result<bool, Error>
    {
        Ok(false)
    }
}

impl MeasureModel<hinawa::SndUnit> for SoloModel {
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

impl card_cntr::NotifyModel<hinawa::SndUnit, bool> for SoloModel {
    fn get_notified_elem_list(&mut self, _: &mut Vec<alsactl::ElemId>) {
    }

    fn parse_notification(&mut self, _: &hinawa::SndUnit, _: &bool) -> Result<(), Error> {
        Ok(())
    }

    fn read_notified_elem(&mut self, _: &hinawa::SndUnit, _: &alsactl::ElemId, _: &mut alsactl::ElemValue)
        -> Result<bool, Error>
    {
        Ok(false)
    }
}
