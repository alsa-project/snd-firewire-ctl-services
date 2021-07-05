// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::{Error, FileError};

use core::card_cntr;
use card_cntr::{CtlModel, MeasureModel, NotifyModel};

use super::tascam_model::TascamModel;
use super::apogee_model::ApogeeModel;
use super::griffin_model::GriffinModel;
use super::lacie_model::LacieModel;
use super::loud_model::LinkFwModel;
use super::common_model::CommonModel;

enum OxfwCtlModel {
    Fireone(TascamModel),
    Duet(ApogeeModel),
    Firewave(GriffinModel),
    Speaker(LacieModel),
    TapcoLinkFw(LinkFwModel),
    Common(CommonModel),
}

pub struct OxfwModel{
    ctl_model: OxfwCtlModel,

    pub measure_elem_list: Vec<alsactl::ElemId>,
    pub notified_elem_list: Vec<alsactl::ElemId>,
}

impl OxfwModel {
    pub fn new(vendor_id: u32, model_id: u32) -> Result<Self, Error> {
        let ctl_model = match (vendor_id, model_id) {
            (0x00022e, 0x800007) => OxfwCtlModel::Fireone(Default::default()),
            (0x0003db, 0x01dddd) => OxfwCtlModel::Duet(Default::default()),
            (0x001292, 0x00f970) => OxfwCtlModel::Firewave(Default::default()),
            (0x00d04b, 0x00f970) => OxfwCtlModel::Speaker(Default::default()),
	        // Stanton Controllers & Systems 1 Deck (SCS.1d) has no audio functionality.
            (0x001260, 0x002000) => return Err(Error::new(FileError::Noent, "Not supported")),
            (0x000ff2, 0x000460) => OxfwCtlModel::TapcoLinkFw(Default::default()),
            _ => OxfwCtlModel::Common(Default::default()),
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
            OxfwCtlModel::Firewave(m) => m.load(unit, card_cntr),
            OxfwCtlModel::Speaker(m) => m.load(unit, card_cntr),
            OxfwCtlModel::TapcoLinkFw(m) => m.load(unit, card_cntr),
            OxfwCtlModel::Common(m) => m.load(unit, card_cntr),
        }?;

        match &mut self.ctl_model {
            OxfwCtlModel::Duet(m) => m.get_measure_elem_list(&mut self.measure_elem_list),
            _ => (),
        }

        match &mut self.ctl_model {
            OxfwCtlModel::Fireone(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
            OxfwCtlModel::Duet(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
            OxfwCtlModel::Firewave(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
            OxfwCtlModel::Speaker(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
            OxfwCtlModel::TapcoLinkFw(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
            OxfwCtlModel::Common(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
        }

        Ok(())
    }

    pub fn dispatch_elem_event(&mut self, unit: &mut hinawa::SndUnit, card_cntr: &mut card_cntr::CardCntr,
                               elem_id: &alsactl::ElemId, events: &alsactl::ElemEventMask)
        -> Result<(), Error>
    {
        match &mut self.ctl_model {
            OxfwCtlModel::Fireone(m) => card_cntr.dispatch_elem_event(unit, elem_id, events, m),
            OxfwCtlModel::Duet(m) => card_cntr.dispatch_elem_event(unit, elem_id, events, m),
            OxfwCtlModel::Firewave(m) => card_cntr.dispatch_elem_event(unit, elem_id, events, m),
            OxfwCtlModel::Speaker(m) => card_cntr.dispatch_elem_event(unit, elem_id, events, m),
            OxfwCtlModel::TapcoLinkFw(m) => card_cntr.dispatch_elem_event(unit, elem_id, events, m),
            OxfwCtlModel::Common(m) => card_cntr.dispatch_elem_event(unit, elem_id, events, m),
        }
    }

    pub fn measure_elems(&mut self, unit: &hinawa::SndUnit, card_cntr: &mut card_cntr::CardCntr)
        -> Result<(), Error>
    {
        match &mut self.ctl_model {
            OxfwCtlModel::Duet(m) => card_cntr.measure_elems(unit, &self.measure_elem_list, m),
            _ => Ok(()),
        }
    }

    pub fn dispatch_notification(&mut self, unit: &hinawa::SndUnit, card_cntr: &mut card_cntr::CardCntr, locked: bool)
        -> Result<(), Error>
    {
        match &mut self.ctl_model {
            OxfwCtlModel::Fireone(m) => card_cntr.dispatch_notification(unit, &locked, &self.notified_elem_list, m),
            OxfwCtlModel::Duet(m) => card_cntr.dispatch_notification(unit, &locked, &self.notified_elem_list, m),
            OxfwCtlModel::Firewave(m) => card_cntr.dispatch_notification(unit, &locked, &self.notified_elem_list, m),
            OxfwCtlModel::Speaker(m) => card_cntr.dispatch_notification(unit, &locked, &self.notified_elem_list, m),
            OxfwCtlModel::TapcoLinkFw(m) => card_cntr.dispatch_notification(unit, &locked, &self.notified_elem_list, m),
            OxfwCtlModel::Common(m) => card_cntr.dispatch_notification(unit, &locked, &self.notified_elem_list, m),
        }
    }
}
