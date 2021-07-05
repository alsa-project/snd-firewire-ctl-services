// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::{Error, FileError};

use core::card_cntr::{CardCntr, CtlModel, NotifyModel};

use motu_protocols::ClkRate;

use super::f828::F828;
use super::f896::F896;
use super::f828mk2::F828mk2;
use super::traveler::Traveler;
use super::ultralite::UltraLite;
use super::f8pre::F8pre;
use super::ultralite_mk3::UltraLiteMk3;
use super::audioexpress::AudioExpress;
use super::f828mk3::F828mk3;
use super::h4pre::H4pre;

pub struct MotuModel {
    #[allow(dead_code)]
    firmware_version: u32,
    ctl_model: MotuCtlModel,
    notified_elems: Vec<alsactl::ElemId>,
}

enum MotuCtlModel {
    F828(F828),
    F896(F896),
    F828mk2(F828mk2),
    Traveler(Traveler),
    UltraLite(UltraLite),
    F8pre(F8pre),
    UltraLiteMk3(UltraLiteMk3),
    AudioExpress(AudioExpress),
    F828mk3(F828mk3),
    H4pre(H4pre),
}

impl MotuModel {
    pub fn new(model_id: u32, version: u32) -> Result<Self, Error> {
        let ctl_model = match model_id {
            0x000001 => MotuCtlModel::F828(Default::default()),
            0x000002 => MotuCtlModel::F896(Default::default()),
            0x000003 => MotuCtlModel::F828mk2(Default::default()),
            0x000009 => MotuCtlModel::Traveler(Default::default()),
            0x00000d => MotuCtlModel::UltraLite(Default::default()),
            0x00000f => MotuCtlModel::F8pre(Default::default()),
            0x000019 => MotuCtlModel::UltraLiteMk3(Default::default()),
            0x000033 => MotuCtlModel::AudioExpress(Default::default()),
            0x000015 |  // Firewire only.
            0x000035 => MotuCtlModel::F828mk3(Default::default()),
            0x000045 => MotuCtlModel::H4pre(Default::default()),
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
            MotuCtlModel::F828(m) => m.load(unit, card_cntr),
            MotuCtlModel::F896(m) => m.load(unit, card_cntr),
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
            MotuCtlModel::F828mk2(m) => m.get_notified_elem_list(&mut self.notified_elems),
            MotuCtlModel::Traveler(m) => m.get_notified_elem_list(&mut self.notified_elems),
            MotuCtlModel::UltraLite(m) => m.get_notified_elem_list(&mut self.notified_elems),
            MotuCtlModel::UltraLiteMk3(m) => m.get_notified_elem_list(&mut self.notified_elems),
            MotuCtlModel::F828mk3(m) => m.get_notified_elem_list(&mut self.notified_elems),
            _ => (),
        }

        Ok(())
    }

    pub fn dispatch_elem_event(&mut self, unit: &mut hinawa::SndMotu, card_cntr: &mut CardCntr,
                               elem_id: &alsactl::ElemId, events: &alsactl::ElemEventMask)
        -> Result<(), Error>
    {
        match &mut self.ctl_model {
            MotuCtlModel::F828(m) => card_cntr.dispatch_elem_event(unit, elem_id, events, m),
            MotuCtlModel::F896(m) => card_cntr.dispatch_elem_event(unit, elem_id, events, m),
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
            MotuCtlModel::F828mk2(m) => card_cntr.dispatch_notification(unit, msg, elem_id_list, m),
            MotuCtlModel::Traveler(m) => card_cntr.dispatch_notification(unit, msg, elem_id_list, m),
            MotuCtlModel::UltraLite(m) => card_cntr.dispatch_notification(unit, msg, elem_id_list, m),
            MotuCtlModel::UltraLiteMk3(m) => card_cntr.dispatch_notification(unit, msg, elem_id_list, m),
            MotuCtlModel::F828mk3(m) => card_cntr.dispatch_notification(unit, msg, elem_id_list, m),
            _ => Ok(()),
        }
    }
}

pub fn clk_rate_to_string(rate: &ClkRate) -> String {
    match rate {
        ClkRate::R44100 => "44100",
        ClkRate::R48000 => "48000",
        ClkRate::R88200 => "88200",
        ClkRate::R96000 => "96000",
        ClkRate::R176400 => "176400",
        ClkRate::R192000 => "192000",
    }.to_string()
}
