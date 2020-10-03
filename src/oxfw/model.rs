// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::{Error, FileError};

use crate::card_cntr;
use crate::card_cntr::{CtlModel, MeasureModel, NotifyModel};

use super::tascam_model::TascamModel;
use super::apogee_model::ApogeeModel;

enum OxfwCtlModel {
    Fireone(TascamModel),
    Duet(ApogeeModel),
}

pub struct OxfwModel{
    ctl_model: OxfwCtlModel,

    pub measure_elem_list: Vec<alsactl::ElemId>,
    pub notified_elem_list: Vec<alsactl::ElemId>,
}

impl OxfwModel {
    pub fn new(vendor_id: u32, model_id: u32) -> Result<Self, Error> {
        let ctl_model = match (vendor_id, model_id) {
            (0x00022e, 0x800007) => OxfwCtlModel::Fireone(TascamModel::new()),
            (0x0003db, 0x01dddd) => OxfwCtlModel::Duet(ApogeeModel::new()),
            _ => return Err(Error::new(FileError::Noent, "Not supported")),
        };
        let model = OxfwModel{
            ctl_model,
            measure_elem_list: Vec::new(),
            notified_elem_list: Vec::new(),
        };
        Ok(model)
    }

    pub fn load(&mut self, unit: &hinawa::SndUnit, card_cntr: &mut card_cntr::CardCntr)
        -> Result<(), Error>
    {
        match &mut self.ctl_model {
            OxfwCtlModel::Fireone(m) => m.load(unit, card_cntr),
            OxfwCtlModel::Duet(m) => m.load(unit, card_cntr),
        }?;

        match &mut self.ctl_model {
            OxfwCtlModel::Duet(m) => m.get_measure_elem_list(&mut self.measure_elem_list),
            _ => (),
        }

        match &mut self.ctl_model {
            OxfwCtlModel::Fireone(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
            OxfwCtlModel::Duet(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
        }

        Ok(())
    }

    pub fn dispatch_elem_event(&mut self, unit: &hinawa::SndUnit, card_cntr: &mut card_cntr::CardCntr,
                               elem_id: &alsactl::ElemId, events: &alsactl::ElemEventMask)
        -> Result<(), Error>
    {
        match &mut self.ctl_model {
            OxfwCtlModel::Fireone(m) => card_cntr.dispatch_elem_event(unit, elem_id, events, m),
            OxfwCtlModel::Duet(m) => card_cntr.dispatch_elem_event(unit, elem_id, events, m),
        }
    }

    pub fn measure_elems(&mut self, _: &hinawa::SndUnit, _: &mut card_cntr::CardCntr)
        -> Result<(), Error>
    {
        Ok(())
    }

    pub fn dispatch_notification(&mut self, unit: &hinawa::SndUnit, card_cntr: &mut card_cntr::CardCntr, locked: bool)
        -> Result<(), Error>
    {
        match &mut self.ctl_model {
            OxfwCtlModel::Fireone(m) => card_cntr.dispatch_notification(unit, &locked, &self.notified_elem_list, m),
            OxfwCtlModel::Duet(m) => card_cntr.dispatch_notification(unit, &locked, &self.notified_elem_list, m),
        }
    }
}
