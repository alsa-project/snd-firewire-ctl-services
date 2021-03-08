// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto
use glib::Error;

use alsactl::{ElemId, ElemValue};

use hinawa::SndUnit;

use core::card_cntr::*;

use ff_protocols::former::ff400::*;

use super::former_ctls::*;

#[derive(Default, Debug)]
pub struct Ff400Model{
    proto: Ff400Protocol,
    meter_ctl: FormerMeterCtl<Ff400MeterState>,
    out_ctl: FormerOutCtl<Ff400OutputVolumeState>,
}

const TIMEOUT_MS: u32 = 100;

impl CtlModel<SndUnit> for Ff400Model {
    fn load(&mut self, unit: &SndUnit, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.meter_ctl.load(unit, &self.proto, card_cntr, TIMEOUT_MS)?;
        self.out_ctl.load(unit, &self.proto, card_cntr, TIMEOUT_MS)?;
        Ok(())
    }

    fn read(&mut self, _: &SndUnit, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        if self.out_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(&mut self, unit: &SndUnit, elem_id: &ElemId, _: &ElemValue, new: &ElemValue)
        -> Result<bool, Error>
    {
        if self.out_ctl.write(unit, &self.proto, elem_id, new, TIMEOUT_MS)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl MeasureModel<SndUnit> for Ff400Model {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        self.meter_ctl.get_measured_elem_list(elem_id_list);
    }

    fn measure_states(&mut self, unit: &SndUnit) -> Result<(), Error> {
        self.meter_ctl.measure_states(unit, &self.proto, TIMEOUT_MS)
    }

    fn measure_elem(&mut self, _: &SndUnit, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        self.meter_ctl.measure_elem(elem_id, elem_value)
    }
}
