// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::{Error, FileError};

use core::card_cntr;
use card_cntr::{CtlModel, MeasureModel, NotifyModel};

use super::apogee::apogee_model::EnsembleModel;
use super::maudio::ozonic_model::OzonicModel;
use super::maudio::solo_model::SoloModel;
use super::maudio::audiophile_model::AudiophileModel;
use super::maudio::fw410_model::Fw410Model;
use super::maudio::profirelightbridge_model::ProfirelightbridgeModel;
use super::maudio::special_model::SpecialModel;
use super::behringer::firepower_model::FirepowerModel;
use super::stanton::ScratchampModel;
use super::esi::QuatafireModel;

pub struct BebobModel<'a>{
    ctl_model: BebobCtlModel<'a>,
    pub measure_elem_list: Vec<alsactl::ElemId>,
    pub notified_elem_list: Vec<alsactl::ElemId>,
}

enum BebobCtlModel<'a> {
    ApogeeEnsemble(EnsembleModel<'a>),
    MaudioOzonic(OzonicModel<'a>),
    MaudioSolo(SoloModel<'a>),
    MaudioAudiophile(AudiophileModel<'a>),
    MaudioFw410(Fw410Model<'a>),
    MaudioPlb(ProfirelightbridgeModel<'a>),
    MaudioSpecial(SpecialModel),
    BehringerFirepower(FirepowerModel<'a>),
    StantonScratchamp(ScratchampModel<'a>),
    EsiQuatafire(QuatafireModel<'a>),
}

impl<'a> BebobModel<'a> {
    pub fn new(vendor_id: u32, model_id: u32) -> Result<Self, Error> {
        let ctl_model = match (vendor_id, model_id) {
            (0x0003db, 0x01eeee) => BebobCtlModel::ApogeeEnsemble(Default::default()),
            (0x000d6c, 0x00000a) => BebobCtlModel::MaudioOzonic(Default::default()),
            (0x000d6c, 0x010062) => BebobCtlModel::MaudioSolo(Default::default()),
            (0x000d6c, 0x010060) => BebobCtlModel::MaudioAudiophile(Default::default()),
            (0x0007f5, 0x010046) => BebobCtlModel::MaudioFw410(Default::default()),
            (0x000d6c, 0x0100a1) => BebobCtlModel::MaudioPlb(Default::default()),
            (0x000d6c, 0x010071) => BebobCtlModel::MaudioSpecial(SpecialModel::new(true)),
            (0x000d6c, 0x010091) => BebobCtlModel::MaudioSpecial(SpecialModel::new(false)),
            (0x001564, 0x000610) => BebobCtlModel::BehringerFirepower(Default::default()),
            (0x001260, 0x000001) => BebobCtlModel::StantonScratchamp(Default::default()),
            (0x000f1b, 0x010064) => BebobCtlModel::EsiQuatafire(Default::default()),
            _ => {
                return Err(Error::new(FileError::Noent, "Not supported"));
            }
        };

        let model = BebobModel{
            ctl_model,
            measure_elem_list: Vec::new(),
            notified_elem_list: Vec::new(),
        };

        Ok(model)
    }

    pub fn load(&mut self, unit: &mut hinawa::SndUnit, card_cntr: &mut card_cntr::CardCntr)
        -> Result<(), Error>
    {
        match &mut self.ctl_model {
            BebobCtlModel::ApogeeEnsemble(m) => m.load(unit, card_cntr),
            BebobCtlModel::MaudioOzonic(m) => m.load(unit, card_cntr),
            BebobCtlModel::MaudioSolo(m) => m.load(unit, card_cntr),
            BebobCtlModel::MaudioAudiophile(m) => m.load(unit, card_cntr),
            BebobCtlModel::MaudioFw410(m) => m.load(unit, card_cntr),
            BebobCtlModel::MaudioPlb(m) => m.load(unit, card_cntr),
            BebobCtlModel::MaudioSpecial(m) => m.load(unit, card_cntr),
            BebobCtlModel::BehringerFirepower(m) => m.load(unit, card_cntr),
            BebobCtlModel::StantonScratchamp(m) => m.load(unit, card_cntr),
            BebobCtlModel::EsiQuatafire(m) => m.load(unit, card_cntr),
        }?;

        match &mut self.ctl_model {
            BebobCtlModel::ApogeeEnsemble(m) => m.get_measure_elem_list(&mut self.measure_elem_list),
            BebobCtlModel::MaudioOzonic(m) => m.get_measure_elem_list(&mut self.measure_elem_list),
            BebobCtlModel::MaudioSolo(m) => m.get_measure_elem_list(&mut self.measure_elem_list),
            BebobCtlModel::MaudioAudiophile(m) => m.get_measure_elem_list(&mut self.measure_elem_list),
            BebobCtlModel::MaudioFw410(m) => m.get_measure_elem_list(&mut self.measure_elem_list),
            BebobCtlModel::MaudioPlb(m) => m.get_measure_elem_list(&mut self.measure_elem_list),
            BebobCtlModel::MaudioSpecial(m) => m.get_measure_elem_list(&mut self.measure_elem_list),
            _ => (),
        }

        match &mut self.ctl_model {
            BebobCtlModel::ApogeeEnsemble(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
            BebobCtlModel::MaudioOzonic(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
            BebobCtlModel::MaudioSolo(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
            BebobCtlModel::MaudioAudiophile(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
            BebobCtlModel::MaudioFw410(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
            BebobCtlModel::MaudioPlb(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
            BebobCtlModel::MaudioSpecial(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
            BebobCtlModel::BehringerFirepower(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
            BebobCtlModel::StantonScratchamp(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
            BebobCtlModel::EsiQuatafire(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
        }

        Ok(())
    }

    pub fn dispatch_elem_event(&mut self, unit: &mut hinawa::SndUnit, card_cntr: &mut card_cntr::CardCntr,
                               elem_id: &alsactl::ElemId, events: &alsactl::ElemEventMask)
        -> Result<(), Error>
    {
        match &mut self.ctl_model {
            BebobCtlModel::ApogeeEnsemble(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
            BebobCtlModel::MaudioOzonic(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
            BebobCtlModel::MaudioSolo(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
            BebobCtlModel::MaudioAudiophile(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
            BebobCtlModel::MaudioFw410(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
            BebobCtlModel::MaudioPlb(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
            BebobCtlModel::MaudioSpecial(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
            BebobCtlModel::BehringerFirepower(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
            BebobCtlModel::StantonScratchamp(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
            BebobCtlModel::EsiQuatafire(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
        }
    }

    pub fn measure_elems(&mut self, unit: &mut hinawa::SndUnit, card_cntr: &mut card_cntr::CardCntr)
        -> Result<(), Error>
    {
        match &mut self.ctl_model {
            BebobCtlModel::ApogeeEnsemble(m) => card_cntr.measure_elems(unit, &self.measure_elem_list, m),
            BebobCtlModel::MaudioOzonic(m) => card_cntr.measure_elems(unit, &self.measure_elem_list, m),
            BebobCtlModel::MaudioSolo(m) => card_cntr.measure_elems(unit, &self.measure_elem_list, m),
            BebobCtlModel::MaudioAudiophile(m) => card_cntr.measure_elems(unit, &self.measure_elem_list, m),
            BebobCtlModel::MaudioFw410(m) => card_cntr.measure_elems(unit, &self.measure_elem_list, m),
            BebobCtlModel::MaudioPlb(m) => card_cntr.measure_elems(unit, &self.measure_elem_list, m),
            BebobCtlModel::MaudioSpecial(m) => card_cntr.measure_elems(unit, &self.measure_elem_list, m),
            _ => Ok(()),
        }
    }

    pub fn dispatch_stream_lock(&mut self, unit: &mut hinawa::SndUnit, card_cntr: &mut card_cntr::CardCntr, notice: bool)
        -> Result<(), Error>
    {
        match &mut self.ctl_model {
            BebobCtlModel::ApogeeEnsemble(m) => card_cntr.dispatch_notification(unit, &notice, &self.notified_elem_list, m),
            BebobCtlModel::MaudioOzonic(m) => card_cntr.dispatch_notification(unit, &notice, &self.notified_elem_list, m),
            BebobCtlModel::MaudioSolo(m) => card_cntr.dispatch_notification(unit, &notice, &self.notified_elem_list, m),
            BebobCtlModel::MaudioAudiophile(m) => card_cntr.dispatch_notification(unit, &notice, &self.notified_elem_list, m),
            BebobCtlModel::MaudioFw410(m) => card_cntr.dispatch_notification(unit, &notice, &self.notified_elem_list, m),
            BebobCtlModel::MaudioPlb(m) => card_cntr.dispatch_notification(unit, &notice, &self.notified_elem_list, m),
            BebobCtlModel::MaudioSpecial(m) => card_cntr.dispatch_notification(unit, &notice, &self.notified_elem_list, m),
            BebobCtlModel::BehringerFirepower(m) => card_cntr.dispatch_notification(unit, &notice, &self.notified_elem_list, m),
            BebobCtlModel::StantonScratchamp(m) => card_cntr.dispatch_notification(unit, &notice, &self.notified_elem_list, m),
            BebobCtlModel::EsiQuatafire(m) => card_cntr.dispatch_notification(unit, &notice, &self.notified_elem_list, m),
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
