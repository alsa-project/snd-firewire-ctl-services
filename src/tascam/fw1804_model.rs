// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use crate::card_cntr;

pub struct Fw1804Model {}

impl Fw1804Model {
    pub fn new() -> Self {
        Fw1804Model{}
    }
}

impl card_cntr::CtlModel<hinawa::SndTscm> for Fw1804Model {
    fn load(
        &mut self,
        _: &hinawa::SndTscm,
        _: &mut card_cntr::CardCntr,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn read(
        &mut self,
        _: &hinawa::SndTscm,
        _: &alsactl::ElemId,
        _: &mut alsactl::ElemValue,
    ) -> Result<bool, Error> {
        Ok(false)
    }

    fn write(
        &mut self,
        _: &hinawa::SndTscm,
        _: &alsactl::ElemId,
        _: &alsactl::ElemValue,
        _: &alsactl::ElemValue,
    ) -> Result<bool, Error> {
        Ok(false)
    }
}
