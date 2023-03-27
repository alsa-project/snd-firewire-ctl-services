// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2023 Takashi Sakamoto

use super::*;

#[derive(Default, Debug)]
pub struct Audiofire9 {}

impl Audiofire9 {
    pub(crate) fn cache(&mut self, _: &mut SndEfw) -> Result<(), Error> {
        Ok(())
    }
}

impl CtlModel<SndEfw> for Audiofire9 {
    fn load(&mut self, _: &mut SndEfw, _: &mut CardCntr) -> Result<(), Error> {
        Ok(())
    }

    fn read(&mut self, _: &mut SndEfw, _: &ElemId, _: &mut ElemValue) -> Result<bool, Error> {
        Ok(false)
    }

    fn write(
        &mut self,
        _: &mut SndEfw,
        _: &ElemId,
        _: &ElemValue,
        _: &ElemValue,
    ) -> Result<bool, Error> {
        Ok(false)
    }
}

impl MeasureModel<SndEfw> for Audiofire9 {
    fn get_measure_elem_list(&mut self, _: &mut Vec<ElemId>) {}

    fn measure_states(&mut self, _: &mut SndEfw) -> Result<(), Error> {
        Ok(())
    }

    fn measure_elem(&mut self, _: &SndEfw, _: &ElemId, _: &mut ElemValue) -> Result<bool, Error> {
        Ok(false)
    }
}

impl NotifyModel<SndEfw, bool> for Audiofire9 {
    fn get_notified_elem_list(&mut self, _: &mut Vec<ElemId>) {}

    fn parse_notification(&mut self, _: &mut SndEfw, &_: &bool) -> Result<(), Error> {
        Ok(())
    }

    fn read_notified_elem(
        &mut self,
        _: &SndEfw,
        _: &ElemId,
        _: &mut ElemValue,
    ) -> Result<bool, Error> {
        Ok(false)
    }
}
