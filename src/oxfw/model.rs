// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use crate::card_cntr;

pub struct OxfwModel;

impl OxfwModel {
    pub fn new(_: u32, _: u32) -> Result<Self, Error> {
        Ok(OxfwModel{})
    }

    pub fn load(&mut self, _: &hinawa::SndUnit, _: &mut card_cntr::CardCntr)
        -> Result<(), Error>
    {
        Ok(())
    }

    pub fn dispatch_elem_event(&mut self, _: &hinawa::SndUnit, _: &mut card_cntr::CardCntr,
                               _: &alsactl::ElemId, _: &alsactl::ElemEventMask)
        -> Result<(), Error>
    {
        Ok(())
    }
}
