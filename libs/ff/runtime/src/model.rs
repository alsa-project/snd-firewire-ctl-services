// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

use glib::Error;

use hinawa::SndUnit;

use core::card_cntr::*;

pub struct FfModel;

impl FfModel {
    pub fn new(_: &SndUnit) -> Result<FfModel, Error> {
        Ok(FfModel{})
    }
    pub fn load(&mut self, _: &SndUnit, _: &mut CardCntr) -> Result<(), Error> {
        Ok(())
    }

    pub fn dispatch_elem_event(&mut self, _: &SndUnit, _: &mut CardCntr,
                               _: &alsactl::ElemId, _: &alsactl::ElemEventMask)
        -> Result<(), Error>
    {
        Ok(())
    }
}
