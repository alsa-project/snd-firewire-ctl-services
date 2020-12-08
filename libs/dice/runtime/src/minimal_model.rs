// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use alsactl::{ElemId, ElemValue};

use hinawa::SndDice;

use core::card_cntr::*;

#[derive(Default)]
pub struct MinimalModel;

impl CtlModel<SndDice> for MinimalModel {
    fn load(&mut self, _: &SndDice, _: &mut CardCntr) -> Result<(), Error> {
        Ok(())
    }

    fn read(&mut self, _: &SndDice, _: &ElemId, _: &mut ElemValue)
        -> Result<bool, Error>
    {
        Ok(false)
    }

    fn write(&mut self, _: &SndDice, _: &ElemId, _: &ElemValue, _: &ElemValue)
        -> Result<bool, Error>
    {
        Ok(false)
    }
}
