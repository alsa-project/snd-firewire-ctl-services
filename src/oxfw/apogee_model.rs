// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use crate::card_cntr;

pub struct ApogeeModel;

impl<'a> ApogeeModel {
    pub fn new() -> Self {
        ApogeeModel{}
    }
}

impl card_cntr::CtlModel<hinawa::SndUnit> for ApogeeModel {
    fn load(&mut self, _: &hinawa::SndUnit, _: &mut card_cntr::CardCntr) -> Result<(), Error> {
        Ok(())
    }

    fn read(&mut self, _: &hinawa::SndUnit, _: &alsactl::ElemId, _: &mut alsactl::ElemValue)
        -> Result<bool, Error>
    {
        Ok(false)
    }

    fn write(&mut self, _: &hinawa::SndUnit, _: &alsactl::ElemId, _: &alsactl::ElemValue,
             _: &alsactl::ElemValue)
        -> Result<bool, Error>
    {
        Ok(false)
    }
}

impl card_cntr::NotifyModel<hinawa::SndUnit, bool> for ApogeeModel {
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
