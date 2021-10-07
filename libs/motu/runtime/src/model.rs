// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::{Error, FileError};

use core::card_cntr::{CardCntr, CtlModel, NotifyModel};

use motu_protocols::*;

use super::f828::F828;
use super::f896::F896;
use super::f828mk2::F828mk2;
use super::f896hd::F896hd;
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
    F896hd(F896hd),
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
            0x000005 => MotuCtlModel::F896hd(Default::default()),
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

    pub fn load(&mut self, unit: &mut hinawa::SndMotu, card_cntr: &mut CardCntr)
        -> Result<(), Error>
    {
        match &mut self.ctl_model {
            MotuCtlModel::F828(m) => m.load(unit, card_cntr),
            MotuCtlModel::F896(m) => m.load(unit, card_cntr),
            MotuCtlModel::F828mk2(m) => m.load(unit, card_cntr),
            MotuCtlModel::F896hd(m) => m.load(unit, card_cntr),
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
            MotuCtlModel::F896hd(m) => card_cntr.dispatch_elem_event(unit, elem_id, events, m),
            MotuCtlModel::Traveler(m) => card_cntr.dispatch_elem_event(unit, elem_id, events, m),
            MotuCtlModel::UltraLite(m) => card_cntr.dispatch_elem_event(unit, elem_id, events, m),
            MotuCtlModel::F8pre(m) => card_cntr.dispatch_elem_event(unit, elem_id, events, m),
            MotuCtlModel::UltraLiteMk3(m) => card_cntr.dispatch_elem_event(unit, elem_id, events, m),
            MotuCtlModel::AudioExpress(m) => card_cntr.dispatch_elem_event(unit, elem_id, events, m),
            MotuCtlModel::F828mk3(m) => card_cntr.dispatch_elem_event(unit, elem_id, events, m),
            MotuCtlModel::H4pre(m) => card_cntr.dispatch_elem_event(unit, elem_id, events, m),
        }
    }

    pub fn dispatch_notification(&mut self, unit: &mut hinawa::SndMotu, msg: &u32, card_cntr: &mut CardCntr)
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

pub fn clk_rate_to_str(rate: &ClkRate) -> &'static str {
    match rate {
        ClkRate::R44100 => "44100",
        ClkRate::R48000 => "48000",
        ClkRate::R88200 => "88200",
        ClkRate::R96000 => "96000",
        ClkRate::R176400 => "176400",
        ClkRate::R192000 => "192000",
    }
}

pub fn target_port_to_str(port: &TargetPort) -> &'static str {
    match port {
        TargetPort::Disabled => "Disabled",
        TargetPort::AnalogPair0 => "Analog-1/2",
        TargetPort::AnalogPair1 => "Analog-3/4",
        TargetPort::AnalogPair2 => "Analog-5/6",
        TargetPort::AnalogPair3 => "Analog-7/8",
        TargetPort::AesEbuPair0 => "AES/EBU-1/2",
        TargetPort::PhonePair0 => "Phone-1/2",
        TargetPort::MainPair0 => "Main-1/2",
        TargetPort::SpdifPair0 => "SPDIF-1/2",
        TargetPort::AdatPair0 => "ADAT-1/2",
        TargetPort::AdatPair1 => "ADAT-3/4",
        TargetPort::AdatPair2 => "ADAT-5/6",
        TargetPort::AdatPair3 => "ADAT-7/8",
        TargetPort::Analog0 => "Analog-1",
        TargetPort::Analog1 => "Analog-2",
        TargetPort::Analog2 => "Analog-3",
        TargetPort::Analog3 => "Analog-4",
        TargetPort::Analog4 => "Analog-5",
        TargetPort::Analog5 => "Analog-6",
        TargetPort::Analog6 => "Analog-7",
        TargetPort::Analog7 => "Analog-8",
        TargetPort::AesEbu0 => "AES/EBU-1",
        TargetPort::AesEbu1 => "AES/EBU-2",
        TargetPort::Analog6Pairs => "Analog-1/2/3/4/5/6",
        TargetPort::Analog8Pairs => "Analog-1/2/3/4/5/6/7/8",
        TargetPort::OpticalAPair0 => "Optical-A-1/2",
        TargetPort::OpticalAPair1 => "Optical-A-3/4",
        TargetPort::OpticalAPair2 => "Optical-A-5/6",
        TargetPort::OpticalAPair3 => "Optical-A-7/8",
        TargetPort::OpticalBPair0 => "Optical-B-1/2",
        TargetPort::OpticalBPair1 => "Optical-B-3/4",
        TargetPort::OpticalBPair2 => "Optical-B-5/6",
        TargetPort::OpticalBPair3 => "Optical-B-7/8",
        TargetPort::Mic0 => "Mic-1",
        TargetPort::Mic1 => "Mic-2",
        TargetPort::Spdif0 => "S/PDIF-1",
        TargetPort::Spdif1 => "S/PDIF-2",
        TargetPort::Adat0 => "ADAT-1",
        TargetPort::Adat1 => "ADAT-2",
        TargetPort::Adat2 => "ADAT-3",
        TargetPort::Adat3 => "ADAT-4",
        TargetPort::Adat4 => "ADAT-5",
        TargetPort::Adat5 => "ADAT-6",
        TargetPort::Adat6 => "ADAT-7",
        TargetPort::Adat7 => "ADAT-8",
    }
}

pub fn nominal_signal_level_to_str(level: &NominalSignalLevel) -> &'static str {
    match level {
        NominalSignalLevel::Consumer => "-10dBu",
        NominalSignalLevel::Professional => "+4dBV",
    }
}