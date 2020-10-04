// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use hinawa::{FwFcpExt, SndUnitExt};

use crate::card_cntr;

use crate::bebob::BebobAvc;

pub struct SpecialModel {
    avc: BebobAvc,
}

impl<'a> SpecialModel {
    pub fn new() -> Self {
        SpecialModel {
            avc: BebobAvc::new(),
        }
    }
}

impl card_cntr::CtlModel<hinawa::SndUnit> for SpecialModel {
    fn load(&mut self, unit: &hinawa::SndUnit, _: &mut card_cntr::CardCntr) -> Result<(), Error> {
        self.avc.fcp.bind(&unit.get_node())?;

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

impl<'a> card_cntr::MeasureModel<hinawa::SndUnit> for SpecialModel {
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
