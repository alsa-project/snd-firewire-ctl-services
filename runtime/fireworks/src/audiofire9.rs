// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2023 Takashi Sakamoto

use {super::*, protocols::audiofire::Audiofire9Protocol};

#[derive(Default, Debug)]
pub struct Audiofire9 {
    clk_ctl: SamplingClockCtl<Audiofire9Protocol>,
}

const TIMEOUT_MS: u32 = 100;

impl Audiofire9 {
    pub(crate) fn cache(&mut self, unit: &mut SndEfw) -> Result<(), Error> {
        self.clk_ctl.cache(unit, TIMEOUT_MS)?;

        Ok(())
    }
}

impl CtlModel<SndEfw> for Audiofire9 {
    fn load(&mut self, _: &mut SndEfw, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.clk_ctl.load(card_cntr, false)?;
        Ok(())
    }

    fn read(
        &mut self,
        _: &mut SndEfw,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.clk_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(
        &mut self,
        unit: &mut SndEfw,
        elem_id: &ElemId,
        _: &ElemValue,
        elem_value: &ElemValue,
    ) -> Result<bool, Error> {
        if self.clk_ctl.write(unit, elem_id, elem_value, TIMEOUT_MS)? {
            Ok(true)
        } else {
            Ok(false)
        }
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
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.clk_ctl.elem_id_list);
    }

    fn parse_notification(&mut self, unit: &mut SndEfw, &locked: &bool) -> Result<(), Error> {
        if locked {
            self.clk_ctl.cache(unit, TIMEOUT_MS)?;
        }
        Ok(())
    }

    fn read_notified_elem(
        &mut self,
        _: &SndEfw,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.clk_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
