// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use glib::{Error, FileError};

use core::card_cntr;
use card_cntr::{CtlModel, MeasureModel, NotifyModel};

use super::apogee::ensemble_model::EnsembleModel;
use super::maudio::ozonic_model::OzonicModel;
use super::maudio::solo_model::SoloModel;
use super::maudio::audiophile_model::AudiophileModel;
use super::maudio::fw410_model::Fw410Model;
use super::maudio::profirelightbridge_model::PflModel;
use super::maudio::special_model::*;
use super::behringer::*;
use super::stanton::ScratchampModel;
use super::esi::Quatafire610Model;
use super::icon::FirexonModel;
use super::presonus::fp10_model::*;
use super::presonus::firebox_model::*;
use super::presonus::inspire1394_model::*;
use super::roland::*;
use super::yamaha_terratec::{GoPhase24CoaxModel, GoPhase24OptModel};

pub struct BebobModel {
    ctl_model: Model,
    pub measure_elem_list: Vec<alsactl::ElemId>,
    pub notified_elem_list: Vec<alsactl::ElemId>,
}

enum Model {
    ApogeeEnsemble(EnsembleModel),
    BehringerFca610(Fca610Model),
    EsiQuatafire610(Quatafire610Model),
    IconFirexon(FirexonModel),
    MaudioOzonic(OzonicModel),
    MaudioSolo(SoloModel),
    MaudioAudiophile(AudiophileModel),
    MaudioFw410(Fw410Model),
    MaudioPfl(PflModel),
    MaudioFw1814(Fw1814Model),
    MaudioProjectMix(ProjectMixModel),
    PresonusFp10(Fp10Model),
    PresonusFirebox(FireboxModel),
    PresonusInspire1394(Inspire1394Model),
    RolandFa101(Fa101Model),
    RolandFa66(Fa66Model),
    StantonScratchamp(ScratchampModel),
    TerratecPhase24(GoPhase24CoaxModel),
    TerratecPhaseX24(GoPhase24OptModel),
    YamahaGo44(GoPhase24CoaxModel),
    YamahaGo46(GoPhase24OptModel),
}

impl BebobModel {
    pub fn new(vendor_id: u32, model_id: u32) -> Result<Self, Error> {
        let ctl_model = match (vendor_id, model_id) {
            (0x0003db, 0x01eeee) => Model::ApogeeEnsemble(Default::default()),
            (0x001564, 0x000610) => Model::BehringerFca610(Default::default()),
            (0x000f1b, 0x010064) => Model::EsiQuatafire610(Default::default()),
            (0x001a9e, 0x000001) => Model::IconFirexon(Default::default()),
            (0x000d6c, 0x00000a) => Model::MaudioOzonic(Default::default()),
            (0x000d6c, 0x010062) => Model::MaudioSolo(Default::default()),
            (0x000d6c, 0x010060) => Model::MaudioAudiophile(Default::default()),
            (0x0007f5, 0x010046) => Model::MaudioFw410(Default::default()),
            (0x000d6c, 0x0100a1) => Model::MaudioPfl(Default::default()),
            (0x000d6c, 0x010071) => Model::MaudioFw1814(Default::default()),
            (0x000d6c, 0x010091) => Model::MaudioProjectMix(Default::default()),
            (0x000a92, 0x010066) => Model::PresonusFp10(Default::default()),
            (0x000a92, 0x010000) => Model::PresonusFirebox(Default::default()),
            (0x000a92, 0x010001) => Model::PresonusInspire1394(Default::default()),
            (0x0040ab, 0x010048) => Model::RolandFa101(Default::default()),
            (0x0040ab, 0x010049) => Model::RolandFa66(Default::default()),
            (0x001260, 0x000001) => Model::StantonScratchamp(Default::default()),
            (0x000aac, 0x000004) => Model::TerratecPhase24(Default::default()),
            (0x000aac, 0x000007) => Model::TerratecPhaseX24(Default::default()),
            (0x00a0de, 0x10000b) => Model::YamahaGo44(Default::default()),
            (0x00a0de, 0x10000c) => Model::YamahaGo46(Default::default()),
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
            Model::ApogeeEnsemble(m) => m.load(unit, card_cntr),
            Model::BehringerFca610(m) => m.load(unit, card_cntr),
            Model::EsiQuatafire610(m) => m.load(unit, card_cntr),
            Model::IconFirexon(m) => m.load(unit, card_cntr),
            Model::MaudioOzonic(m) => m.load(unit, card_cntr),
            Model::MaudioSolo(m) => m.load(unit, card_cntr),
            Model::MaudioAudiophile(m) => m.load(unit, card_cntr),
            Model::MaudioFw410(m) => m.load(unit, card_cntr),
            Model::MaudioPfl(m) => m.load(unit, card_cntr),
            Model::MaudioFw1814(m) => m.load(unit, card_cntr),
            Model::MaudioProjectMix(m) => m.load(unit, card_cntr),
            Model::PresonusFp10(m) => m.load(unit, card_cntr),
            Model::PresonusFirebox(m) => m.load(unit, card_cntr),
            Model::PresonusInspire1394(m) => m.load(unit, card_cntr),
            Model::RolandFa101(m) => m.load(unit, card_cntr),
            Model::RolandFa66(m) => m.load(unit, card_cntr),
            Model::StantonScratchamp(m) => m.load(unit, card_cntr),
            Model::TerratecPhase24(m) => m.load(unit, card_cntr),
            Model::TerratecPhaseX24(m) => m.load(unit, card_cntr),
            Model::YamahaGo44(m) => m.load(unit, card_cntr),
            Model::YamahaGo46(m) => m.load(unit, card_cntr),
        }?;

        match &mut self.ctl_model {
            Model::ApogeeEnsemble(m) => m.get_measure_elem_list(&mut self.measure_elem_list),
            Model::MaudioOzonic(m) => m.get_measure_elem_list(&mut self.measure_elem_list),
            Model::MaudioSolo(m) => m.get_measure_elem_list(&mut self.measure_elem_list),
            Model::MaudioAudiophile(m) => m.get_measure_elem_list(&mut self.measure_elem_list),
            Model::MaudioFw410(m) => m.get_measure_elem_list(&mut self.measure_elem_list),
            Model::MaudioPfl(m) => m.get_measure_elem_list(&mut self.measure_elem_list),
            Model::MaudioFw1814(m) => m.get_measure_elem_list(&mut self.measure_elem_list),
            Model::MaudioProjectMix(m) => m.get_measure_elem_list(&mut self.measure_elem_list),
            Model::PresonusInspire1394(m) => m.get_measure_elem_list(&mut self.measure_elem_list),
            _ => (),
        }

        match &mut self.ctl_model {
            Model::ApogeeEnsemble(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
            Model::BehringerFca610(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
            Model::EsiQuatafire610(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
            Model::MaudioOzonic(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
            Model::MaudioSolo(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
            Model::MaudioAudiophile(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
            Model::MaudioFw410(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
            Model::MaudioPfl(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
            Model::MaudioFw1814(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
            Model::MaudioProjectMix(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
            Model::PresonusFirebox(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
            Model::PresonusInspire1394(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
            Model::StantonScratchamp(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
            Model::TerratecPhase24(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
            Model::TerratecPhaseX24(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
            Model::YamahaGo44(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
            Model::YamahaGo46(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
            _ => (),
        }

        Ok(())
    }

    pub fn dispatch_elem_event(&mut self, unit: &mut hinawa::SndUnit, card_cntr: &mut card_cntr::CardCntr,
                               elem_id: &alsactl::ElemId, events: &alsactl::ElemEventMask)
        -> Result<(), Error>
    {
        match &mut self.ctl_model {
            Model::ApogeeEnsemble(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
            Model::BehringerFca610(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
            Model::EsiQuatafire610(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
            Model::IconFirexon(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
            Model::MaudioOzonic(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
            Model::MaudioSolo(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
            Model::MaudioAudiophile(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
            Model::MaudioFw410(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
            Model::MaudioPfl(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
            Model::MaudioFw1814(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
            Model::MaudioProjectMix(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
            Model::PresonusFp10(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
            Model::PresonusFirebox(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
            Model::PresonusInspire1394(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
            Model::RolandFa101(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
            Model::RolandFa66(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
            Model::StantonScratchamp(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
            Model::TerratecPhase24(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
            Model::TerratecPhaseX24(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
            Model::YamahaGo44(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
            Model::YamahaGo46(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
        }
    }

    pub fn measure_elems(&mut self, unit: &mut hinawa::SndUnit, card_cntr: &mut card_cntr::CardCntr)
        -> Result<(), Error>
    {
        match &mut self.ctl_model {
            Model::ApogeeEnsemble(m) => card_cntr.measure_elems(unit, &self.measure_elem_list, m),
            Model::MaudioOzonic(m) => card_cntr.measure_elems(unit, &self.measure_elem_list, m),
            Model::MaudioSolo(m) => card_cntr.measure_elems(unit, &self.measure_elem_list, m),
            Model::MaudioAudiophile(m) => card_cntr.measure_elems(unit, &self.measure_elem_list, m),
            Model::MaudioFw410(m) => card_cntr.measure_elems(unit, &self.measure_elem_list, m),
            Model::MaudioPfl(m) => card_cntr.measure_elems(unit, &self.measure_elem_list, m),
            Model::MaudioFw1814(m) => card_cntr.measure_elems(unit, &self.measure_elem_list, m),
            Model::MaudioProjectMix(m) => card_cntr.measure_elems(unit, &self.measure_elem_list, m),
            Model::PresonusInspire1394(m) => card_cntr.measure_elems(unit, &self.measure_elem_list, m),
            _ => Ok(()),
        }
    }

    pub fn dispatch_stream_lock(&mut self, unit: &mut hinawa::SndUnit, card_cntr: &mut card_cntr::CardCntr, notice: bool)
        -> Result<(), Error>
    {
        match &mut self.ctl_model {
            Model::ApogeeEnsemble(m) => card_cntr.dispatch_notification(unit, &notice, &self.notified_elem_list, m),
            Model::BehringerFca610(m) => card_cntr.dispatch_notification(unit, &notice, &self.notified_elem_list, m),
            Model::EsiQuatafire610(m) => card_cntr.dispatch_notification(unit, &notice, &self.notified_elem_list, m),
            Model::MaudioOzonic(m) => card_cntr.dispatch_notification(unit, &notice, &self.notified_elem_list, m),
            Model::MaudioSolo(m) => card_cntr.dispatch_notification(unit, &notice, &self.notified_elem_list, m),
            Model::MaudioAudiophile(m) => card_cntr.dispatch_notification(unit, &notice, &self.notified_elem_list, m),
            Model::MaudioFw410(m) => card_cntr.dispatch_notification(unit, &notice, &self.notified_elem_list, m),
            Model::MaudioPfl(m) => card_cntr.dispatch_notification(unit, &notice, &self.notified_elem_list, m),
            Model::MaudioFw1814(m) => card_cntr.dispatch_notification(unit, &notice, &self.notified_elem_list, m),
            Model::MaudioProjectMix(m) => card_cntr.dispatch_notification(unit, &notice, &self.notified_elem_list, m),
            Model::PresonusFirebox(m) => card_cntr.dispatch_notification(unit, &notice, &self.notified_elem_list, m),
            Model::PresonusInspire1394(m) => card_cntr.dispatch_notification(unit, &notice, &self.notified_elem_list, m),
            Model::StantonScratchamp(m) => card_cntr.dispatch_notification(unit, &notice, &self.notified_elem_list, m),
            Model::TerratecPhase24(m) => card_cntr.dispatch_notification(unit, &notice, &self.notified_elem_list, m),
            Model::TerratecPhaseX24(m) => card_cntr.dispatch_notification(unit, &notice, &self.notified_elem_list, m),
            Model::YamahaGo44(m) => card_cntr.dispatch_notification(unit, &notice, &self.notified_elem_list, m),
            Model::YamahaGo46(m) => card_cntr.dispatch_notification(unit, &notice, &self.notified_elem_list, m),
            _ => Ok(()),
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
