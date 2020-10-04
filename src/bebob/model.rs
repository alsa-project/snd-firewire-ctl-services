// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::{Error, FileError};

use crate::card_cntr;
use card_cntr::{CtlModel, MeasureModel};

use super::apogee::apogee_model::EnsembleModel;

pub struct BebobModel<'a>{
    ctl_model: BebobCtlModel<'a>,
    pub measure_elem_list: Vec<alsactl::ElemId>,
}

enum BebobCtlModel<'a> {
    ApogeeEnsemble(EnsembleModel<'a>),
}

impl<'a> BebobModel<'a> {
    pub fn new(vendor_id: u32, model_id: u32) -> Result<Self, Error> {
        let ctl_model = match (vendor_id, model_id) {
            (0x0003db, 0x01eeee) => BebobCtlModel::ApogeeEnsemble(EnsembleModel::new()),
            _ => {
                return Err(Error::new(FileError::Noent, "Not supported"));
            }
        };

        let model = BebobModel{
            ctl_model,
            measure_elem_list: Vec::new(),
        };

        Ok(model)
    }

    pub fn load(&mut self, unit: &hinawa::SndUnit, card_cntr: &mut card_cntr::CardCntr)
        -> Result<(), Error>
    {
        match &mut self.ctl_model {
            BebobCtlModel::ApogeeEnsemble(m) => m.load(unit, card_cntr),
        }?;

        match &mut self.ctl_model {
            BebobCtlModel::ApogeeEnsemble(m) => m.get_measure_elem_list(&mut self.measure_elem_list),
        }

        Ok(())
    }

    pub fn dispatch_elem_event(&mut self, unit: &hinawa::SndUnit, card_cntr: &mut card_cntr::CardCntr,
                               elem_id: &alsactl::ElemId, events: &alsactl::ElemEventMask)
        -> Result<(), Error>
    {
        match &mut self.ctl_model {
            BebobCtlModel::ApogeeEnsemble(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
        }
    }

    pub fn measure_elems(&mut self, unit: &hinawa::SndUnit, card_cntr: &mut card_cntr::CardCntr)
        -> Result<(), Error>
    {
        match &mut self.ctl_model {
            BebobCtlModel::ApogeeEnsemble(m) => card_cntr.measure_elems(unit, &self.measure_elem_list, m),
        }
    }
}

pub const CLK_RATE_NAME: &str = "clock-rate";
pub const CLK_SRC_NAME: &str = "clock-source";

pub const OUT_SRC_NAME: &str = "output-source";
pub const OUT_VOL_NAME: &str = "output-volume";

pub const HP_SRC_NAME: &str = "headphone-source";

pub const IN_METER_NAME: &str = "input-meters";
pub const OUT_METER_NAME: &str = "output-meters";
