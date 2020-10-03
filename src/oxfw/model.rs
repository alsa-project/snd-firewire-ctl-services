// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::{Error, FileError};

use crate::card_cntr;
use crate::card_cntr::CtlModel;

use super::tascam_model::TascamModel;

enum OxfwCtlModel {
    Fireone(TascamModel),
}

pub struct OxfwModel{
    ctl_model: OxfwCtlModel,
}

impl OxfwModel {
    pub fn new(vendor_id: u32, model_id: u32) -> Result<Self, Error> {
        let ctl_model = match (vendor_id, model_id) {
            (0x00022e, 0x800007) => OxfwCtlModel::Fireone(TascamModel::new()),
            _ => return Err(Error::new(FileError::Noent, "Not supported")),
        };
        let model = OxfwModel{
            ctl_model,
        };
        Ok(model)
    }

    pub fn load(&mut self, unit: &hinawa::SndUnit, card_cntr: &mut card_cntr::CardCntr)
        -> Result<(), Error>
    {
        match &mut self.ctl_model {
            OxfwCtlModel::Fireone(m) => m.load(unit, card_cntr),
        }
    }

    pub fn dispatch_elem_event(&mut self, unit: &hinawa::SndUnit, card_cntr: &mut card_cntr::CardCntr,
                               elem_id: &alsactl::ElemId, events: &alsactl::ElemEventMask)
        -> Result<(), Error>
    {
        match &mut self.ctl_model {
            OxfwCtlModel::Fireone(m) => card_cntr.dispatch_elem_event(unit, elem_id, events, m),
        }
    }
}
