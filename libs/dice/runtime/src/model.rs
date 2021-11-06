// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::{Error, FileError};

use hinawa::{FwReq, FwNodeExtManual};
use hinawa::{SndDice, SndUnitExt};

use core::card_cntr::*;

use ieee1212_config_rom::*;
use dice_protocols::tcat::{*, config_rom::*, extension::*};

use std::convert::TryFrom;

use super::minimal_model::MinimalModel;
use super::tcelectronic::k24d_model::*;
use super::tcelectronic::k8_model::*;
use super::tcelectronic::studiok48_model::*;
use super::tcelectronic::klive_model::*;
use super::tcelectronic::desktopk6_model::*;
use super::tcelectronic::itwin_model::*;
use super::io_fw_model::*;
use super::ionix_model::*;
use super::presonus::fstudio_model::*;
use super::extension_model::ExtensionModel;
use super::pfire_model::*;
use super::mbox3_model::*;
use super::blackbird_model::*;
use super::focusrite::spro40_model::*;
use super::focusrite::liquids56_model::*;
use super::focusrite::spro24_model::*;
use super::focusrite::spro24dsp_model::*;
use super::focusrite::spro14_model::*;
use super::focusrite::spro26_model::*;
use super::presonus::fstudioproject_model::*;
use super::presonus::fstudiomobile_model::*;

enum Model {
    Minimal(MinimalModel),
    TcK24d(K24dModel),
    TcK8(K8Model),
    TcStudiok48(Studiok48Model),
    TcKlive(KliveModel),
    TcDesktopk6(Desktopk6Model),
    TcItwin(ItwinModel),
    AlesisIoFw(IoFwModel),
    LexiconIonix(IonixModel),
    PresonusFStudio(FStudioModel),
    Extension(ExtensionModel),
    MaudioPfire2626(Pfire2626Model),
    MaudioPfire610(Pfire610Model),
    AvidMbox3(Mbox3Model),
    LoudBlackbird(BlackbirdModel),
    FocusriteSPro40(SPro40Model),
    FocusriteLiquidS56(LiquidS56Model),
    FocusriteSPro24(SPro24Model),
    FocusriteSPro24Dsp(SPro24DspModel),
    FocusriteSPro14(SPro14Model),
    FocusriteSPro26(SPro26Model),
    PresonusFStudioProject(FStudioProjectModel),
    PresonusFStudioMobile(FStudioMobileModel),
}

pub struct DiceModel{
    model: Model,
    notified_elem_list: Vec<alsactl::ElemId>,
    pub measured_elem_list: Vec<alsactl::ElemId>,
}

impl DiceModel {
    pub fn new(unit: &SndDice) -> Result<DiceModel, Error> {
        let node = unit.get_node();
        let raw = node.get_config_rom()?;
        let config_rom = ConfigRom::try_from(&raw[..])
            .map_err(|e| {
                let msg = format!("Malformed configuration ROM detected: {}", e);
                Error::new(FileError::Nxio, &msg)
            })?;
        let data = config_rom.get_root_data()
            .and_then(|root| {
                config_rom.get_unit_data()
                    .map(|unit| (root.vendor_id, unit.model_id))
            })
            .ok_or_else(|| {
                Error::new(FileError::Nxio, "Fail to detect information in configuration ROM")
            })?;

        let model = match data {
            (0x000166, 0x000020) => Model::TcK24d(K24dModel::default()),
            (0x000166, 0x000021) => Model::TcK8(K8Model::default()),
            (0x000166, 0x000022) => Model::TcStudiok48(Studiok48Model::default()),
            (0x000166, 0x000023) => Model::TcKlive(KliveModel::default()),
            (0x000166, 0x000024) => Model::TcDesktopk6(Desktopk6Model::default()),
            (0x000166, 0x000027) => Model::TcItwin(ItwinModel::default()),
            (0x000595, 0x000001) => Model::AlesisIoFw(IoFwModel::default()),
            (0x000fd7, 0x000001) => Model::LexiconIonix(IonixModel::default()),
            (0x000a92, 0x000008) => Model::PresonusFStudio(FStudioModel::default()),
            (0x000d6c, 0x000010) => Model::MaudioPfire2626(Pfire2626Model::default()),
            (0x000d6c, 0x000011) => Model::MaudioPfire610(Pfire610Model::default()),
            (0x00a07e, 0x000004) => Model::AvidMbox3(Mbox3Model::default()),
            (0x000ff2, 0x000007) => Model::LoudBlackbird(BlackbirdModel::default()),
            (0x00130e, 0x000005) => Model::FocusriteSPro40(SPro40Model::default()),
            (0x00130e, 0x000006) => Model::FocusriteLiquidS56(LiquidS56Model::default()),
            (0x00130e, 0x000007) => Model::FocusriteSPro24(Default::default()),
            (0x00130e, 0x000008) => Model::FocusriteSPro24Dsp(Default::default()),
            (0x00130e, 0x000009) => Model::FocusriteSPro14(Default::default()),
            (0x00130e, 0x000012) => Model::FocusriteSPro26(SPro26Model::default()),
            (0x000a92, 0x00000b) => Model::PresonusFStudioProject(FStudioProjectModel::default()),
            (0x000a92, 0x000011) => Model::PresonusFStudioMobile(FStudioMobileModel::default()),
            (0x000166, 0x000030) |  // TC Electronic Digital Konnekt x32.
            (0x000595, 0x000000) |  // Alesis MultiMix 8/12/16 FireWire.
            (0x000595, 0x000002) |  // Alesis MasterControl.
            _ => Model::Minimal(MinimalModel::default()),
        };

        let notified_elem_list = Vec::new();
        let measured_elem_list = Vec::new();

        Ok(DiceModel{model, notified_elem_list, measured_elem_list})
    }

    pub fn load(&mut self, unit: &mut SndDice, card_cntr: &mut CardCntr)
        -> Result<(), Error>
    {
        // Replace model data when protocol extension is available.
        if let Model::Minimal(_) = &mut self.model {
            let mut req = FwReq::default();
            if ProtocolExtension::read_extension_sections(&mut req, &mut unit.get_node(), 100).is_ok() {
                self.model = Model::Extension(ExtensionModel::default());
            } else {
                // MEMO: workaround for old firmware. Invalidate a negative effect by failure of
                // previous transaction.
                let _ = GeneralProtocol::read_general_sections(
                    &mut req,
                    &mut unit.get_node(),
                    100
                )?;
            }
        }

        match &mut self.model {
            Model::Minimal(m) => m.load(unit, card_cntr),
            Model::TcK24d(m) => m.load(unit, card_cntr),
            Model::TcK8(m) => m.load(unit, card_cntr),
            Model::TcStudiok48(m) => m.load(unit, card_cntr),
            Model::TcKlive(m) => m.load(unit, card_cntr),
            Model::TcDesktopk6(m) => m.load(unit, card_cntr),
            Model::TcItwin(m) => m.load(unit, card_cntr),
            Model::AlesisIoFw(m) => m.load(unit, card_cntr),
            Model::LexiconIonix(m) => m.load(unit, card_cntr),
            Model::PresonusFStudio(m) => m.load(unit, card_cntr),
            Model::Extension(m) => m.load(unit, card_cntr),
            Model::MaudioPfire2626(m) => m.load(unit, card_cntr),
            Model::MaudioPfire610(m) => m.load(unit, card_cntr),
            Model::AvidMbox3(m) => m.load(unit, card_cntr),
            Model::LoudBlackbird(m) => m.load(unit, card_cntr),
            Model::FocusriteSPro40(m) => m.load(unit, card_cntr),
            Model::FocusriteLiquidS56(m) => m.load(unit, card_cntr),
            Model::FocusriteSPro24(m) => m.load(unit, card_cntr),
            Model::FocusriteSPro24Dsp(m) => m.load(unit, card_cntr),
            Model::FocusriteSPro14(m) => m.load(unit, card_cntr),
            Model::FocusriteSPro26(m) => m.load(unit, card_cntr),
            Model::PresonusFStudioProject(m) => m.load(unit, card_cntr),
            Model::PresonusFStudioMobile(m) => m.load(unit, card_cntr),
        }?;

        match &mut self.model {
            Model::Minimal(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
            Model::TcK24d(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
            Model::TcK8(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
            Model::TcStudiok48(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
            Model::TcKlive(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
            Model::TcDesktopk6(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
            Model::TcItwin(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
            Model::AlesisIoFw(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
            Model::LexiconIonix(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
            Model::PresonusFStudio(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
            Model::Extension(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
            Model::MaudioPfire2626(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
            Model::MaudioPfire610(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
            Model::AvidMbox3(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
            Model::LoudBlackbird(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
            Model::FocusriteSPro40(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
            Model::FocusriteLiquidS56(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
            Model::FocusriteSPro24(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
            Model::FocusriteSPro24Dsp(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
            Model::FocusriteSPro14(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
            Model::FocusriteSPro26(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
            Model::PresonusFStudioProject(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
            Model::PresonusFStudioMobile(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
        }

        match &mut self.model {
            Model::Minimal(m) => m.get_measure_elem_list(&mut self.measured_elem_list),
            Model::TcK24d(m) => m.get_measure_elem_list(&mut self.measured_elem_list),
            Model::TcK8(m) => m.get_measure_elem_list(&mut self.measured_elem_list),
            Model::TcStudiok48(m) => m.get_measure_elem_list(&mut self.measured_elem_list),
            Model::TcKlive(m) => m.get_measure_elem_list(&mut self.measured_elem_list),
            Model::TcDesktopk6(m) => m.get_measure_elem_list(&mut self.measured_elem_list),
            Model::TcItwin(m) => m.get_measure_elem_list(&mut self.measured_elem_list),
            Model::AlesisIoFw(m) => m.get_measure_elem_list(&mut self.measured_elem_list),
            Model::LexiconIonix(m) => m.get_measure_elem_list(&mut self.measured_elem_list),
            Model::PresonusFStudio(m) => m.get_measure_elem_list(&mut self.measured_elem_list),
            Model::Extension(m) => m.get_measure_elem_list(&mut self.measured_elem_list),
            Model::MaudioPfire2626(m) => m.get_measure_elem_list(&mut self.measured_elem_list),
            Model::MaudioPfire610(m) => m.get_measure_elem_list(&mut self.measured_elem_list),
            Model::AvidMbox3(m) => m.get_measure_elem_list(&mut self.measured_elem_list),
            Model::LoudBlackbird(m) => m.get_measure_elem_list(&mut self.measured_elem_list),
            Model::FocusriteSPro40(m) => m.get_measure_elem_list(&mut self.measured_elem_list),
            Model::FocusriteLiquidS56(m) => m.get_measure_elem_list(&mut self.measured_elem_list),
            Model::FocusriteSPro24(m) => m.get_measure_elem_list(&mut self.measured_elem_list),
            Model::FocusriteSPro24Dsp(m) => m.get_measure_elem_list(&mut self.measured_elem_list),
            Model::FocusriteSPro14(m) => m.get_measure_elem_list(&mut self.measured_elem_list),
            Model::FocusriteSPro26(m) => m.get_measure_elem_list(&mut self.measured_elem_list),
            Model::PresonusFStudioProject(m) => m.get_measure_elem_list(&mut self.measured_elem_list),
            Model::PresonusFStudioMobile(m) => m.get_measure_elem_list(&mut self.measured_elem_list),
        }

        Ok(())
    }

    pub fn dispatch_elem_event(&mut self, unit: &mut SndDice, card_cntr: &mut CardCntr,
                               elem_id: &alsactl::ElemId, events: &alsactl::ElemEventMask)
        -> Result<(), Error>
    {
        match &mut self.model {
            Model::Minimal(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
            Model::TcK24d(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
            Model::TcK8(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
            Model::TcStudiok48(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
            Model::TcKlive(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
            Model::TcDesktopk6(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
            Model::TcItwin(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
            Model::AlesisIoFw(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
            Model::LexiconIonix(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
            Model::PresonusFStudio(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
            Model::Extension(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
            Model::MaudioPfire2626(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
            Model::MaudioPfire610(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
            Model::AvidMbox3(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
            Model::LoudBlackbird(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
            Model::FocusriteSPro40(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
            Model::FocusriteLiquidS56(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
            Model::FocusriteSPro24(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
            Model::FocusriteSPro24Dsp(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
            Model::FocusriteSPro14(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
            Model::FocusriteSPro26(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
            Model::PresonusFStudioProject(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
            Model::PresonusFStudioMobile(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
        }
    }

    pub fn dispatch_msg(&mut self, unit: &mut SndDice, card_cntr: &mut CardCntr, msg: u32)
        -> Result<(), Error>
    {
        match &mut self.model {
            Model::Minimal(m) => card_cntr.dispatch_notification(unit, &msg, &self.notified_elem_list, m),
            Model::TcK24d(m) => card_cntr.dispatch_notification(unit, &msg, &self.notified_elem_list, m),
            Model::TcK8(m) => card_cntr.dispatch_notification(unit, &msg, &self.notified_elem_list, m),
            Model::TcStudiok48(m) => card_cntr.dispatch_notification(unit, &msg, &self.notified_elem_list, m),
            Model::TcKlive(m) => card_cntr.dispatch_notification(unit, &msg, &self.notified_elem_list, m),
            Model::TcDesktopk6(m) => card_cntr.dispatch_notification(unit, &msg, &self.notified_elem_list, m),
            Model::TcItwin(m) => card_cntr.dispatch_notification(unit, &msg, &self.notified_elem_list, m),
            Model::AlesisIoFw(m) => card_cntr.dispatch_notification(unit, &msg, &self.notified_elem_list, m),
            Model::LexiconIonix(m) => card_cntr.dispatch_notification(unit, &msg, &self.notified_elem_list, m),
            Model::PresonusFStudio(m) => card_cntr.dispatch_notification(unit, &msg, &self.notified_elem_list, m),
            Model::Extension(m) => card_cntr.dispatch_notification(unit, &msg, &self.notified_elem_list, m),
            Model::MaudioPfire2626(m) => card_cntr.dispatch_notification(unit, &msg, &self.notified_elem_list, m),
            Model::MaudioPfire610(m) => card_cntr.dispatch_notification(unit, &msg, &self.notified_elem_list, m),
            Model::AvidMbox3(m) => card_cntr.dispatch_notification(unit, &msg, &self.notified_elem_list, m),
            Model::LoudBlackbird(m) => card_cntr.dispatch_notification(unit, &msg, &self.notified_elem_list, m),
            Model::FocusriteSPro40(m) => card_cntr.dispatch_notification(unit, &msg, &self.notified_elem_list, m),
            Model::FocusriteLiquidS56(m) => card_cntr.dispatch_notification(unit, &msg, &self.notified_elem_list, m),
            Model::FocusriteSPro24(m) => card_cntr.dispatch_notification(unit, &msg, &self.notified_elem_list, m),
            Model::FocusriteSPro24Dsp(m) => card_cntr.dispatch_notification(unit, &msg, &self.notified_elem_list, m),
            Model::FocusriteSPro14(m) => card_cntr.dispatch_notification(unit, &msg, &self.notified_elem_list, m),
            Model::FocusriteSPro26(m) => card_cntr.dispatch_notification(unit, &msg, &self.notified_elem_list, m),
            Model::PresonusFStudioProject(m) => card_cntr.dispatch_notification(unit, &msg, &self.notified_elem_list, m),
            Model::PresonusFStudioMobile(m) => card_cntr.dispatch_notification(unit, &msg, &self.notified_elem_list, m),
        }
    }

    pub fn measure_elems(&mut self, unit: &mut hinawa::SndDice, card_cntr: &mut CardCntr)
        -> Result<(), Error>
    {
        match &mut self.model {
            Model::Minimal(m) => card_cntr.measure_elems(unit, &self.measured_elem_list, m),
            Model::TcK24d(m) => card_cntr.measure_elems(unit, &self.measured_elem_list, m),
            Model::TcK8(m) => card_cntr.measure_elems(unit, &self.measured_elem_list, m),
            Model::TcStudiok48(m) => card_cntr.measure_elems(unit, &self.measured_elem_list, m),
            Model::TcKlive(m) => card_cntr.measure_elems(unit, &self.measured_elem_list, m),
            Model::TcDesktopk6(m) => card_cntr.measure_elems(unit, &self.measured_elem_list, m),
            Model::TcItwin(m) => card_cntr.measure_elems(unit, &self.measured_elem_list, m),
            Model::AlesisIoFw(m) => card_cntr.measure_elems(unit, &self.measured_elem_list, m),
            Model::LexiconIonix(m) => card_cntr.measure_elems(unit, &self.measured_elem_list, m),
            Model::PresonusFStudio(m) => card_cntr.measure_elems(unit, &self.measured_elem_list, m),
            Model::Extension(m) => card_cntr.measure_elems(unit, &self.measured_elem_list, m),
            Model::MaudioPfire2626(m) => card_cntr.measure_elems(unit, &self.measured_elem_list, m),
            Model::MaudioPfire610(m) => card_cntr.measure_elems(unit, &self.measured_elem_list, m),
            Model::AvidMbox3(m) => card_cntr.measure_elems(unit, &self.measured_elem_list, m),
            Model::LoudBlackbird(m) => card_cntr.measure_elems(unit, &self.measured_elem_list, m),
            Model::FocusriteSPro40(m) => card_cntr.measure_elems(unit, &self.measured_elem_list, m),
            Model::FocusriteLiquidS56(m) => card_cntr.measure_elems(unit, &self.measured_elem_list, m),
            Model::FocusriteSPro24(m) => card_cntr.measure_elems(unit, &self.measured_elem_list, m),
            Model::FocusriteSPro24Dsp(m) => card_cntr.measure_elems(unit, &self.measured_elem_list, m),
            Model::FocusriteSPro14(m) => card_cntr.measure_elems(unit, &self.measured_elem_list, m),
            Model::FocusriteSPro26(m) => card_cntr.measure_elems(unit, &self.measured_elem_list, m),
            Model::PresonusFStudioProject(m) => card_cntr.measure_elems(unit, &self.measured_elem_list, m),
            Model::PresonusFStudioMobile(m) => card_cntr.measure_elems(unit, &self.measured_elem_list, m),
        }
    }
}
