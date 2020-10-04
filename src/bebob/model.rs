// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::{Error, FileError};

use crate::card_cntr;
use card_cntr::{CtlModel, MeasureModel, NotifyModel};

use super::apogee::apogee_model::EnsembleModel;
use super::maudio::ozonic_model::OzonicModel;
use super::maudio::solo_model::SoloModel;
use super::maudio::audiophile_model::AudiophileModel;
use super::maudio::fw410_model::Fw410Model;

pub struct BebobModel<'a>{
    ctl_model: BebobCtlModel<'a>,
    pub measure_elem_list: Vec<alsactl::ElemId>,
    pub notified_elem_list: Vec<alsactl::ElemId>,
}

enum BebobCtlModel<'a> {
    ApogeeEnsemble(EnsembleModel<'a>),
    MaudioOzonic(OzonicModel),
    MaudioSolo(SoloModel),
    MaudioAudiophile(AudiophileModel),
    MaudioFw410(Fw410Model),
}

impl<'a> BebobModel<'a> {
    pub fn new(vendor_id: u32, model_id: u32) -> Result<Self, Error> {
        let ctl_model = match (vendor_id, model_id) {
            (0x0003db, 0x01eeee) => BebobCtlModel::ApogeeEnsemble(EnsembleModel::new()),
            (0x000d6c, 0x00000a) => BebobCtlModel::MaudioOzonic(OzonicModel::new()),
            (0x000d6c, 0x010062) => BebobCtlModel::MaudioSolo(SoloModel::new()),
            (0x000d6c, 0x010060) => BebobCtlModel::MaudioAudiophile(AudiophileModel::new()),
            (0x0007f5, 0x010046) => BebobCtlModel::MaudioFw410(Fw410Model::new()),
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

    pub fn load(&mut self, unit: &hinawa::SndUnit, card_cntr: &mut card_cntr::CardCntr)
        -> Result<(), Error>
    {
        match &mut self.ctl_model {
            BebobCtlModel::ApogeeEnsemble(m) => m.load(unit, card_cntr),
            BebobCtlModel::MaudioOzonic(m) => m.load(unit, card_cntr),
            BebobCtlModel::MaudioSolo(m) => m.load(unit, card_cntr),
            BebobCtlModel::MaudioAudiophile(m) => m.load(unit, card_cntr),
            BebobCtlModel::MaudioFw410(m) => m.load(unit, card_cntr),
        }?;

        match &mut self.ctl_model {
            BebobCtlModel::ApogeeEnsemble(m) => m.get_measure_elem_list(&mut self.measure_elem_list),
            BebobCtlModel::MaudioOzonic(m) => m.get_measure_elem_list(&mut self.measure_elem_list),
            BebobCtlModel::MaudioSolo(m) => m.get_measure_elem_list(&mut self.measure_elem_list),
            BebobCtlModel::MaudioAudiophile(m) => m.get_measure_elem_list(&mut self.measure_elem_list),
            BebobCtlModel::MaudioFw410(m) => m.get_measure_elem_list(&mut self.measure_elem_list),
        }

        match &mut self.ctl_model {
            BebobCtlModel::ApogeeEnsemble(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
            BebobCtlModel::MaudioOzonic(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
            BebobCtlModel::MaudioSolo(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
            BebobCtlModel::MaudioAudiophile(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
            BebobCtlModel::MaudioFw410(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
        }

        Ok(())
    }

    pub fn dispatch_elem_event(&mut self, unit: &hinawa::SndUnit, card_cntr: &mut card_cntr::CardCntr,
                               elem_id: &alsactl::ElemId, events: &alsactl::ElemEventMask)
        -> Result<(), Error>
    {
        match &mut self.ctl_model {
            BebobCtlModel::ApogeeEnsemble(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
            BebobCtlModel::MaudioOzonic(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
            BebobCtlModel::MaudioSolo(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
            BebobCtlModel::MaudioAudiophile(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
            BebobCtlModel::MaudioFw410(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
        }
    }

    pub fn measure_elems(&mut self, unit: &hinawa::SndUnit, card_cntr: &mut card_cntr::CardCntr)
        -> Result<(), Error>
    {
        match &mut self.ctl_model {
            BebobCtlModel::ApogeeEnsemble(m) => card_cntr.measure_elems(unit, &self.measure_elem_list, m),
            BebobCtlModel::MaudioOzonic(m) => card_cntr.measure_elems(unit, &self.measure_elem_list, m),
            BebobCtlModel::MaudioSolo(m) => card_cntr.measure_elems(unit, &self.measure_elem_list, m),
            BebobCtlModel::MaudioAudiophile(m) => card_cntr.measure_elems(unit, &self.measure_elem_list, m),
            BebobCtlModel::MaudioFw410(m) => card_cntr.measure_elems(unit, &self.measure_elem_list, m),
        }
    }

    pub fn dispatch_stream_lock(&mut self, unit: &hinawa::SndUnit, card_cntr: &mut card_cntr::CardCntr, notice: bool)
        -> Result<(), Error>
    {
        match &mut self.ctl_model {
            BebobCtlModel::ApogeeEnsemble(m) => card_cntr.dispatch_notification(unit, &notice, &self.notified_elem_list, m),
            BebobCtlModel::MaudioOzonic(m) => card_cntr.dispatch_notification(unit, &notice, &self.notified_elem_list, m),
            BebobCtlModel::MaudioSolo(m) => card_cntr.dispatch_notification(unit, &notice, &self.notified_elem_list, m),
            BebobCtlModel::MaudioAudiophile(m) => card_cntr.dispatch_notification(unit, &notice, &self.notified_elem_list, m),
            BebobCtlModel::MaudioFw410(m) => card_cntr.dispatch_notification(unit, &notice, &self.notified_elem_list, m),
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
