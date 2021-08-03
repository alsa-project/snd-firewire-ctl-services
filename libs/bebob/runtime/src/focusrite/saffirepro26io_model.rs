// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

use glib::Error;

use hinawa::FwFcpExt;
use hinawa::{SndUnit, SndUnitExt};

use alsactl::{ElemId, ElemValue};

use core::card_cntr::*;

use bebob_protocols::*;

#[derive(Default)]
pub struct SaffirePro26ioModel {
    avc: BebobAvc,
}

impl CtlModel<SndUnit> for SaffirePro26ioModel {
    fn load(
        &mut self,
        unit: &mut SndUnit,
        _: &mut CardCntr,
    ) -> Result<(), Error> {
        self.avc.as_ref().bind(&unit.get_node())?;

        Ok(())
    }

    fn read(
        &mut self,
        _: &mut SndUnit,
        _: &ElemId,
        _: &mut ElemValue,
    ) -> Result<bool, Error> {
        Ok(false)
    }

    fn write(
        &mut self,
        _: &mut SndUnit,
        _: &ElemId,
        _: &ElemValue,
        _: &ElemValue,
    ) -> Result<bool, Error> {
        Ok(false)
    }
}
