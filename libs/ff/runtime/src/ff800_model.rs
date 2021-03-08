// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto
use glib::Error;

use alsactl::{ElemId, ElemValue};

use hinawa::SndUnit;

use core::card_cntr::*;

#[derive(Default, Debug)]
pub struct Ff800Model;

impl CtlModel<SndUnit> for Ff800Model {
    fn load(&mut self, _: &SndUnit, _: &mut CardCntr) -> Result<(), Error> {
        Ok(())
    }

    fn read(&mut self, _: &SndUnit, _: &ElemId, _: &mut ElemValue)
        -> Result<bool, Error>
    {
        Ok(false)
    }

    fn write(&mut self, _: &SndUnit, _: &ElemId, _: &ElemValue, _: &ElemValue)
        -> Result<bool, Error>
    {
        Ok(false)
    }
}
