// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::{Error, FileError};

use crate::card_cntr::{CardCntr, CtlModel, NotifyModel};

use super::f828mk2::F828mk2;
use super::traveler::Traveler;
use super::ultralite::UltraLite;
use super::f8pre::F8pre;
use super::ultralite_mk3::UltraLiteMk3;
use super::audioexpress::AudioExpress;
use super::f828mk3::F828mk3;
use super::h4pre::H4pre;

pub struct MotuModel<'a> {
    firmware_version: u32,
    ctl_model: MotuCtlModel<'a>,
    notified_elems: Vec<alsactl::ElemId>,
}

enum MotuCtlModel<'a> {
    F828mk2(F828mk2<'a>),
    Traveler(Traveler<'a>),
    UltraLite(UltraLite<'a>),
    F8pre(F8pre<'a>),
    UltraLiteMk3(UltraLiteMk3<'a>),
    AudioExpress(AudioExpress<'a>),
    F828mk3(F828mk3<'a>),
    H4pre(H4pre<'a>),
}

impl<'a> MotuModel<'a> {
    pub fn new(model_id: u32, version: u32) -> Result<Self, Error> {
        let ctl_model = match model_id {
            0x000003 => MotuCtlModel::F828mk2(F828mk2::new()),
            0x000009 => MotuCtlModel::Traveler(Traveler::new()),
            0x00000d => MotuCtlModel::UltraLite(UltraLite::new()),
            0x00000f => MotuCtlModel::F8pre(F8pre::new()),
            0x000019 => MotuCtlModel::UltraLiteMk3(UltraLiteMk3::new()),
            0x000033 => MotuCtlModel::AudioExpress(AudioExpress::new()),
            0x000015 |  // Firewire only.
            0x000035 => MotuCtlModel::F828mk3(F828mk3::new()),
            0x000045 => MotuCtlModel::H4pre(H4pre::new()),
            _ => {
                let label = format!("Unsupported model ID: 0x{:06x}", model_id);
                return Err(Error::new(FileError::Noent, &label));
            }
        };
        let model = MotuModel{
            firmware_version: version,
            ctl_model,
            notified_elems: Vec::new(),
        };
        Ok(model)
    }

    pub fn load(&mut self, unit: &hinawa::SndMotu, card_cntr: &mut CardCntr)
        -> Result<(), Error>
    {
        match &mut self.ctl_model {
            MotuCtlModel::F828mk2(m) => m.load(unit, card_cntr),
            MotuCtlModel::Traveler(m) => m.load(unit, card_cntr),
            MotuCtlModel::UltraLite(m) => m.load(unit, card_cntr),
            MotuCtlModel::F8pre(m) => m.load(unit, card_cntr),
            MotuCtlModel::UltraLiteMk3(m) => m.load(unit, card_cntr),
            MotuCtlModel::AudioExpress(m) => m.load(unit, card_cntr),
            MotuCtlModel::F828mk3(m) => m.load(unit, card_cntr),
            MotuCtlModel::H4pre(m) => m.load(unit, card_cntr),
        }?;

        match &mut self.ctl_model {
            MotuCtlModel::UltraLite(m) => m.get_notified_elem_list(&mut self.notified_elems),
            MotuCtlModel::UltraLiteMk3(m) => m.get_notified_elem_list(&mut self.notified_elems),
            MotuCtlModel::F828mk3(m) => m.get_notified_elem_list(&mut self.notified_elems),
            _ => (),
        }

        Ok(())
    }

    pub fn dispatch_elem_event(&mut self, unit: &hinawa::SndMotu, card_cntr: &mut CardCntr,
                               elem_id: &alsactl::ElemId, events: &alsactl::ElemEventMask)
        -> Result<(), Error>
    {
        match &mut self.ctl_model {
            MotuCtlModel::F828mk2(m) => card_cntr.dispatch_elem_event(unit, elem_id, events, m),
            MotuCtlModel::Traveler(m) => card_cntr.dispatch_elem_event(unit, elem_id, events, m),
            MotuCtlModel::UltraLite(m) => card_cntr.dispatch_elem_event(unit, elem_id, events, m),
            MotuCtlModel::F8pre(m) => card_cntr.dispatch_elem_event(unit, elem_id, events, m),
            MotuCtlModel::UltraLiteMk3(m) => card_cntr.dispatch_elem_event(unit, elem_id, events, m),
            MotuCtlModel::AudioExpress(m) => card_cntr.dispatch_elem_event(unit, elem_id, events, m),
            MotuCtlModel::F828mk3(m) => card_cntr.dispatch_elem_event(unit, elem_id, events, m),
            MotuCtlModel::H4pre(m) => card_cntr.dispatch_elem_event(unit, elem_id, events, m),
        }
    }

    pub fn dispatch_notification(&mut self, unit: &hinawa::SndMotu, msg: &u32, card_cntr: &mut CardCntr)
        -> Result<(), Error>
    {
        let elem_id_list = &self.notified_elems;

        match &mut self.ctl_model {
            MotuCtlModel::UltraLite(m) => card_cntr.dispatch_notification(unit, msg, elem_id_list, m),
            MotuCtlModel::UltraLiteMk3(m) => card_cntr.dispatch_notification(unit, msg, elem_id_list, m),
            MotuCtlModel::F828mk3(m) => card_cntr.dispatch_notification(unit, msg, elem_id_list, m),
            _ => Ok(()),
        }
    }
}
