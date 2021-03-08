// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto
use glib::Error;

use alsactl::{ElemId, ElemValue};

use hinawa::SndUnit;

use core::card_cntr::*;

#[derive(Default, Debug)]
pub struct Ff400Model;

impl CtlModel<SndUnit> for Ff400Model {
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

impl MeasureModel<SndUnit> for Ff400Model {
    fn get_measure_elem_list(&mut self, _: &mut Vec<ElemId>) {
    }

    fn measure_states(&mut self, _: &SndUnit) -> Result<(), Error> {
        Ok(())
    }

    fn measure_elem(&mut self, _: &SndUnit, _: &ElemId, _: &mut ElemValue)
        -> Result<bool, Error>
    {
        Ok(false)
    }
}
