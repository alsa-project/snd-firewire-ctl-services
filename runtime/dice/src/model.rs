// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use {
    super::{
        blackbird_model::*,
        extension_model::*,
        focusrite::liquids56_model::*,
        focusrite::spro14_model::*,
        focusrite::spro24_model::*,
        focusrite::spro24dsp_model::*,
        focusrite::spro26_model::*,
        focusrite::spro40_model::*,
        focusrite::spro40d3_model::*,
        io_fw_model::*,
        ionix_model::*,
        mbox3_model::*,
        minimal_model::*,
        pfire_model::*,
        presonus::fstudio_model::*,
        presonus::fstudiomobile_model::*,
        presonus::fstudioproject_model::*,
        presonus::fstudiotube_model::*,
        tcelectronic::desktopk6_model::*,
        tcelectronic::itwin_model::*,
        tcelectronic::k24d_model::*,
        tcelectronic::k8_model::*,
        tcelectronic::klive_model::*,
        tcelectronic::studiok48_model::*,
        weiss::{avc::*, normal::*},
        *,
    },
    ieee1212_config_rom::*,
    protocols::tcat::config_rom::*,
    std::convert::TryFrom,
};

enum Model {
    Minimal(MinimalModel),
    TcK24d(K24dModel),
    TcK8(K8Model),
    TcStudiok48(Studiok48Model),
    TcKlive(KliveModel),
    TcDesktopk6(Desktopk6Model),
    TcItwin(ItwinModel),
    AlesisIo14fw(Io14fwModel),
    AlesisIo26fw(Io26fwModel),
    LexiconIonix(IonixModel),
    PresonusFStudio(FStudioModel),
    Extension(ExtensionModel),
    MaudioPfire2626(Pfire2626Model),
    MaudioPfire610(Pfire610Model),
    AvidMbox3(Mbox3Model),
    LoudBlackbird(BlackbirdModel),
    FocusriteSPro40(SPro40Model),
    FocusriteSPro40D3(SPro40D3Model),
    FocusriteLiquidS56(LiquidS56Model),
    FocusriteSPro24(SPro24Model),
    FocusriteSPro24Dsp(SPro24DspModel),
    FocusriteSPro14(SPro14Model),
    FocusriteSPro26(SPro26Model),
    PresonusFStudioProject(FStudioProjectModel),
    PresonusFStudioTube(FStudioTubeModel),
    PresonusFStudioMobile(FStudioMobileModel),
    WeissAdc2Model(Adc2Model),
    WeissVestaModel(VestaModel),
    WeissDac2Model(Dac2Model),
    WeissAfi1Model(Afi1Model),
    WeissDac202Model(Dac202Model),
    WeissInt203Model(Int203Model),
    WeissMan301Model(WeissMan301Model),
}

pub struct DiceModel {
    model: Model,
    notified_elem_list: Vec<alsactl::ElemId>,
    pub measured_elem_list: Vec<alsactl::ElemId>,
}

impl DiceModel {
    pub fn new(node: &FwNode) -> Result<DiceModel, Error> {
        let raw = node.config_rom()?;
        let config_rom = ConfigRom::try_from(&raw[..]).map_err(|e| {
            let msg = format!("Malformed configuration ROM detected: {}", e);
            Error::new(FileError::Nxio, &msg)
        })?;
        let data = config_rom
            .get_root_data()
            .and_then(|root| {
                // Use the first unit to detect model, ignoring protocols specified by specifier_i
                // and version fields in each unit directory.
                config_rom
                    .get_unit_data()
                    .iter()
                    .nth(0)
                    .map(|unit| (root.vendor_id, unit.model_id))
            })
            .ok_or_else(|| {
                Error::new(
                    FileError::Nxio,
                    "Fail to detect information in configuration ROM",
                )
            })?;

        let model = match data {
            (0x000166, 0x000020) => Model::TcK24d(K24dModel::default()),
            (0x000166, 0x000021) => Model::TcK8(K8Model::default()),
            (0x000166, 0x000022) => Model::TcStudiok48(Studiok48Model::default()),
            (0x000166, 0x000023) => Model::TcKlive(KliveModel::default()),
            (0x000166, 0x000024) => Model::TcDesktopk6(Desktopk6Model::default()),
            (0x000166, 0x000027) => Model::TcItwin(ItwinModel::default()),
            (0x000595, 0x000001) => {
                // NOTE: Both iO 14 and 26 FireWire have the same identifier. Let us to detect
                // actual model later.
                Model::AlesisIo14fw(Default::default())
            }
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
            (0x00130e, 0x0000de) => Model::FocusriteSPro40D3(SPro40D3Model::default()),
            (0x000a92, 0x00000b) => Model::PresonusFStudioProject(FStudioProjectModel::default()),
            (0x000a92, 0x00000c) => Model::PresonusFStudioTube(FStudioTubeModel::default()),
            (0x000a92, 0x000011) => Model::PresonusFStudioMobile(FStudioMobileModel::default()),
            (0x001c6a, 0x000001) => Model::WeissAdc2Model(Default::default()),
            (0x001c6a, 0x000002) => Model::WeissVestaModel(Default::default()),
            (0x001c6a, 0x000003) => Model::WeissDac2Model(Default::default()),
            (0x001c6a, 0x000004) => Model::WeissAfi1Model(Default::default()),
            (0x001c6a, 0x000007) |
            (0x001c6a, 0x000008) => Model::WeissDac202Model(Default::default()),
            (0x001c6a, 0x000006) |
            (0x001c6a, 0x00000a) => Model::WeissInt203Model(Default::default()),
            (0x001c6a, 0x00000b) => Model::WeissMan301Model(Default::default()),
            (0x000166, 0x000030) |  // TC Electronic Digital Konnekt x32.
            (0x000595, 0x000000) |  // Alesis MultiMix 8/12/16 FireWire.
            (0x000595, 0x000002) |  // Alesis MasterControl.
            _ => Model::Minimal(MinimalModel::default()),
        };

        let notified_elem_list = Vec::new();
        let measured_elem_list = Vec::new();

        Ok(DiceModel {
            model,
            notified_elem_list,
            measured_elem_list,
        })
    }

    pub fn cache(&mut self, unit: &mut (SndDice, FwNode)) -> Result<(), Error> {
        // Replace model data when protocol extension is available.
        if let Model::Minimal(_) = &mut self.model {
            if detect_extended_model(&mut unit.1) {
                self.model = Model::Extension(Default::default());
            }
        }

        // Replace model data when it is Alesis iO 26 FireWire.
        if let Model::AlesisIo14fw(_) = &self.model {
            if detect_io26fw_model(&mut unit.1)? {
                self.model = Model::AlesisIo26fw(Default::default())
            }
        };

        match &mut self.model {
            Model::Minimal(m) => m.cache(unit),
            Model::TcK24d(m) => m.cache(unit),
            Model::TcK8(m) => m.cache(unit),
            Model::TcStudiok48(m) => m.cache(unit),
            Model::TcKlive(m) => m.cache(unit),
            Model::TcDesktopk6(m) => m.cache(unit),
            Model::TcItwin(m) => m.cache(unit),
            Model::AlesisIo14fw(m) => m.cache(unit),
            Model::AlesisIo26fw(m) => m.cache(unit),
            Model::LexiconIonix(m) => m.cache(unit),
            Model::PresonusFStudio(m) => m.cache(unit),
            Model::Extension(m) => m.cache(unit),
            Model::MaudioPfire2626(m) => m.cache(unit),
            Model::MaudioPfire610(m) => m.cache(unit),
            Model::AvidMbox3(m) => m.cache(unit),
            Model::LoudBlackbird(m) => m.cache(unit),
            Model::FocusriteSPro40(m) => m.cache(unit),
            Model::FocusriteSPro40D3(m) => m.cache(unit),
            Model::FocusriteLiquidS56(m) => m.cache(unit),
            Model::FocusriteSPro24(m) => m.cache(unit),
            Model::FocusriteSPro24Dsp(m) => m.cache(unit),
            Model::FocusriteSPro14(m) => m.cache(unit),
            Model::FocusriteSPro26(m) => m.cache(unit),
            Model::PresonusFStudioProject(m) => m.cache(unit),
            Model::PresonusFStudioTube(m) => m.cache(unit),
            Model::PresonusFStudioMobile(m) => m.cache(unit),
            Model::WeissAdc2Model(m) => m.cache(unit),
            Model::WeissVestaModel(m) => m.cache(unit),
            Model::WeissDac2Model(m) => m.cache(unit),
            Model::WeissAfi1Model(m) => m.cache(unit),
            Model::WeissDac202Model(m) => m.cache(unit),
            Model::WeissInt203Model(m) => m.cache(unit),
            Model::WeissMan301Model(m) => m.cache(unit),
        }
    }

    pub fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        match &mut self.model {
            Model::Minimal(m) => m.load(card_cntr),
            Model::TcK24d(m) => m.load(card_cntr),
            Model::TcK8(m) => m.load(card_cntr),
            Model::TcStudiok48(m) => m.load(card_cntr),
            Model::TcKlive(m) => m.load(card_cntr),
            Model::TcDesktopk6(m) => m.load(card_cntr),
            Model::TcItwin(m) => m.load(card_cntr),
            Model::AlesisIo14fw(m) => m.load(card_cntr),
            Model::AlesisIo26fw(m) => m.load(card_cntr),
            Model::LexiconIonix(m) => m.load(card_cntr),
            Model::PresonusFStudio(m) => m.load(card_cntr),
            Model::Extension(m) => m.load(card_cntr),
            Model::MaudioPfire2626(m) => m.load(card_cntr),
            Model::MaudioPfire610(m) => m.load(card_cntr),
            Model::AvidMbox3(m) => m.load(card_cntr),
            Model::LoudBlackbird(m) => m.load(card_cntr),
            Model::FocusriteSPro40(m) => m.load(card_cntr),
            Model::FocusriteSPro40D3(m) => m.load(card_cntr),
            Model::FocusriteLiquidS56(m) => m.load(card_cntr),
            Model::FocusriteSPro24(m) => m.load(card_cntr),
            Model::FocusriteSPro24Dsp(m) => m.load(card_cntr),
            Model::FocusriteSPro14(m) => m.load(card_cntr),
            Model::FocusriteSPro26(m) => m.load(card_cntr),
            Model::PresonusFStudioProject(m) => m.load(card_cntr),
            Model::PresonusFStudioTube(m) => m.load(card_cntr),
            Model::PresonusFStudioMobile(m) => m.load(card_cntr),
            Model::WeissAdc2Model(m) => m.load(card_cntr),
            Model::WeissVestaModel(m) => m.load(card_cntr),
            Model::WeissDac2Model(m) => m.load(card_cntr),
            Model::WeissAfi1Model(m) => m.load(card_cntr),
            Model::WeissDac202Model(m) => m.load(card_cntr),
            Model::WeissInt203Model(m) => m.load(card_cntr),
            Model::WeissMan301Model(m) => m.load(card_cntr),
        }?;

        match &mut self.model {
            Model::Minimal(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
            Model::TcK24d(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
            Model::TcK8(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
            Model::TcStudiok48(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
            Model::TcKlive(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
            Model::TcDesktopk6(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
            Model::TcItwin(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
            Model::AlesisIo14fw(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
            Model::AlesisIo26fw(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
            Model::LexiconIonix(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
            Model::PresonusFStudio(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
            Model::Extension(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
            Model::MaudioPfire2626(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
            Model::MaudioPfire610(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
            Model::AvidMbox3(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
            Model::LoudBlackbird(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
            Model::FocusriteSPro40(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
            Model::FocusriteSPro40D3(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
            Model::FocusriteLiquidS56(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
            Model::FocusriteSPro24(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
            Model::FocusriteSPro24Dsp(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
            Model::FocusriteSPro14(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
            Model::FocusriteSPro26(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
            Model::PresonusFStudioProject(m) => {
                m.get_notified_elem_list(&mut self.notified_elem_list)
            }
            Model::PresonusFStudioTube(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
            Model::PresonusFStudioMobile(m) => {
                m.get_notified_elem_list(&mut self.notified_elem_list)
            }
            Model::WeissAdc2Model(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
            Model::WeissVestaModel(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
            Model::WeissDac2Model(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
            Model::WeissAfi1Model(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
            Model::WeissDac202Model(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
            Model::WeissInt203Model(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
            Model::WeissMan301Model(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
        }

        match &mut self.model {
            Model::Minimal(m) => m.get_measure_elem_list(&mut self.measured_elem_list),
            Model::TcK24d(m) => m.get_measure_elem_list(&mut self.measured_elem_list),
            Model::TcK8(m) => m.get_measure_elem_list(&mut self.measured_elem_list),
            Model::TcStudiok48(m) => m.get_measure_elem_list(&mut self.measured_elem_list),
            Model::TcKlive(m) => m.get_measure_elem_list(&mut self.measured_elem_list),
            Model::TcDesktopk6(m) => m.get_measure_elem_list(&mut self.measured_elem_list),
            Model::TcItwin(m) => m.get_measure_elem_list(&mut self.measured_elem_list),
            Model::AlesisIo14fw(m) => m.get_measure_elem_list(&mut self.measured_elem_list),
            Model::AlesisIo26fw(m) => m.get_measure_elem_list(&mut self.measured_elem_list),
            Model::LexiconIonix(m) => m.get_measure_elem_list(&mut self.measured_elem_list),
            Model::PresonusFStudio(m) => m.get_measure_elem_list(&mut self.measured_elem_list),
            Model::Extension(m) => m.get_measure_elem_list(&mut self.measured_elem_list),
            Model::MaudioPfire2626(m) => m.get_measure_elem_list(&mut self.measured_elem_list),
            Model::MaudioPfire610(m) => m.get_measure_elem_list(&mut self.measured_elem_list),
            Model::AvidMbox3(m) => m.get_measure_elem_list(&mut self.measured_elem_list),
            Model::LoudBlackbird(m) => m.get_measure_elem_list(&mut self.measured_elem_list),
            Model::FocusriteSPro40(m) => m.get_measure_elem_list(&mut self.measured_elem_list),
            Model::FocusriteSPro40D3(m) => m.get_measure_elem_list(&mut self.measured_elem_list),
            Model::FocusriteLiquidS56(m) => m.get_measure_elem_list(&mut self.measured_elem_list),
            Model::FocusriteSPro24(m) => m.get_measure_elem_list(&mut self.measured_elem_list),
            Model::FocusriteSPro24Dsp(m) => m.get_measure_elem_list(&mut self.measured_elem_list),
            Model::FocusriteSPro14(m) => m.get_measure_elem_list(&mut self.measured_elem_list),
            Model::FocusriteSPro26(m) => m.get_measure_elem_list(&mut self.measured_elem_list),
            Model::PresonusFStudioProject(m) => {
                m.get_measure_elem_list(&mut self.measured_elem_list)
            }
            Model::PresonusFStudioTube(m) => m.get_measure_elem_list(&mut self.measured_elem_list),
            Model::PresonusFStudioMobile(m) => {
                m.get_measure_elem_list(&mut self.measured_elem_list)
            }
            Model::WeissAdc2Model(m) => m.get_measure_elem_list(&mut self.measured_elem_list),
            Model::WeissVestaModel(m) => m.get_measure_elem_list(&mut self.measured_elem_list),
            Model::WeissDac2Model(m) => m.get_measure_elem_list(&mut self.measured_elem_list),
            Model::WeissAfi1Model(m) => m.get_measure_elem_list(&mut self.measured_elem_list),
            Model::WeissDac202Model(m) => m.get_measure_elem_list(&mut self.measured_elem_list),
            Model::WeissInt203Model(m) => m.get_measure_elem_list(&mut self.measured_elem_list),
            Model::WeissMan301Model(m) => m.get_measure_elem_list(&mut self.measured_elem_list),
        }

        Ok(())
    }

    pub fn dispatch_elem_event(
        &mut self,
        unit: &mut (SndDice, FwNode),
        card_cntr: &mut CardCntr,
        elem_id: &alsactl::ElemId,
        events: &alsactl::ElemEventMask,
    ) -> Result<(), Error> {
        match &mut self.model {
            Model::Minimal(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
            Model::TcK24d(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
            Model::TcK8(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
            Model::TcStudiok48(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
            Model::TcKlive(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
            Model::TcDesktopk6(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
            Model::TcItwin(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
            Model::AlesisIo14fw(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
            Model::AlesisIo26fw(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
            Model::LexiconIonix(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
            Model::PresonusFStudio(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
            Model::Extension(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
            Model::MaudioPfire2626(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
            Model::MaudioPfire610(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
            Model::AvidMbox3(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
            Model::LoudBlackbird(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
            Model::FocusriteSPro40(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
            Model::FocusriteSPro40D3(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
            Model::FocusriteLiquidS56(m) => {
                card_cntr.dispatch_elem_event(unit, &elem_id, &events, m)
            }
            Model::FocusriteSPro24(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
            Model::FocusriteSPro24Dsp(m) => {
                card_cntr.dispatch_elem_event(unit, &elem_id, &events, m)
            }
            Model::FocusriteSPro14(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
            Model::FocusriteSPro26(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
            Model::PresonusFStudioProject(m) => {
                card_cntr.dispatch_elem_event(unit, &elem_id, &events, m)
            }
            Model::PresonusFStudioTube(m) => {
                card_cntr.dispatch_elem_event(unit, &elem_id, &events, m)
            }
            Model::PresonusFStudioMobile(m) => {
                card_cntr.dispatch_elem_event(unit, &elem_id, &events, m)
            }
            Model::WeissAdc2Model(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
            Model::WeissVestaModel(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
            Model::WeissDac2Model(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
            Model::WeissAfi1Model(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
            Model::WeissDac202Model(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
            Model::WeissInt203Model(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
            Model::WeissMan301Model(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
        }
    }

    pub fn dispatch_msg(
        &mut self,
        unit: &mut (SndDice, FwNode),
        card_cntr: &mut CardCntr,
        msg: u32,
    ) -> Result<(), Error> {
        match &mut self.model {
            Model::Minimal(m) => {
                card_cntr.dispatch_notification(unit, &msg, &self.notified_elem_list, m)
            }
            Model::TcK24d(m) => {
                card_cntr.dispatch_notification(unit, &msg, &self.notified_elem_list, m)
            }
            Model::TcK8(m) => {
                card_cntr.dispatch_notification(unit, &msg, &self.notified_elem_list, m)
            }
            Model::TcStudiok48(m) => {
                card_cntr.dispatch_notification(unit, &msg, &self.notified_elem_list, m)
            }
            Model::TcKlive(m) => {
                card_cntr.dispatch_notification(unit, &msg, &self.notified_elem_list, m)
            }
            Model::TcDesktopk6(m) => {
                card_cntr.dispatch_notification(unit, &msg, &self.notified_elem_list, m)
            }
            Model::TcItwin(m) => {
                card_cntr.dispatch_notification(unit, &msg, &self.notified_elem_list, m)
            }
            Model::AlesisIo14fw(m) => {
                card_cntr.dispatch_notification(unit, &msg, &self.notified_elem_list, m)
            }
            Model::AlesisIo26fw(m) => {
                card_cntr.dispatch_notification(unit, &msg, &self.notified_elem_list, m)
            }
            Model::LexiconIonix(m) => {
                card_cntr.dispatch_notification(unit, &msg, &self.notified_elem_list, m)
            }
            Model::PresonusFStudio(m) => {
                card_cntr.dispatch_notification(unit, &msg, &self.notified_elem_list, m)
            }
            Model::Extension(m) => {
                card_cntr.dispatch_notification(unit, &msg, &self.notified_elem_list, m)
            }
            Model::MaudioPfire2626(m) => {
                card_cntr.dispatch_notification(unit, &msg, &self.notified_elem_list, m)
            }
            Model::MaudioPfire610(m) => {
                card_cntr.dispatch_notification(unit, &msg, &self.notified_elem_list, m)
            }
            Model::AvidMbox3(m) => {
                card_cntr.dispatch_notification(unit, &msg, &self.notified_elem_list, m)
            }
            Model::LoudBlackbird(m) => {
                card_cntr.dispatch_notification(unit, &msg, &self.notified_elem_list, m)
            }
            Model::FocusriteSPro40(m) => {
                card_cntr.dispatch_notification(unit, &msg, &self.notified_elem_list, m)
            }
            Model::FocusriteSPro40D3(m) => {
                card_cntr.dispatch_notification(unit, &msg, &self.notified_elem_list, m)
            }
            Model::FocusriteLiquidS56(m) => {
                card_cntr.dispatch_notification(unit, &msg, &self.notified_elem_list, m)
            }
            Model::FocusriteSPro24(m) => {
                card_cntr.dispatch_notification(unit, &msg, &self.notified_elem_list, m)
            }
            Model::FocusriteSPro24Dsp(m) => {
                card_cntr.dispatch_notification(unit, &msg, &self.notified_elem_list, m)
            }
            Model::FocusriteSPro14(m) => {
                card_cntr.dispatch_notification(unit, &msg, &self.notified_elem_list, m)
            }
            Model::FocusriteSPro26(m) => {
                card_cntr.dispatch_notification(unit, &msg, &self.notified_elem_list, m)
            }
            Model::PresonusFStudioProject(m) => {
                card_cntr.dispatch_notification(unit, &msg, &self.notified_elem_list, m)
            }
            Model::PresonusFStudioTube(m) => {
                card_cntr.dispatch_notification(unit, &msg, &self.notified_elem_list, m)
            }
            Model::PresonusFStudioMobile(m) => {
                card_cntr.dispatch_notification(unit, &msg, &self.notified_elem_list, m)
            }
            Model::WeissAdc2Model(m) => {
                card_cntr.dispatch_notification(unit, &msg, &self.notified_elem_list, m)
            }
            Model::WeissVestaModel(m) => {
                card_cntr.dispatch_notification(unit, &msg, &self.notified_elem_list, m)
            }
            Model::WeissDac2Model(m) => {
                card_cntr.dispatch_notification(unit, &msg, &self.notified_elem_list, m)
            }
            Model::WeissAfi1Model(m) => {
                card_cntr.dispatch_notification(unit, &msg, &self.notified_elem_list, m)
            }
            Model::WeissDac202Model(m) => {
                card_cntr.dispatch_notification(unit, &msg, &self.notified_elem_list, m)
            }
            Model::WeissInt203Model(m) => {
                card_cntr.dispatch_notification(unit, &msg, &self.notified_elem_list, m)
            }
            Model::WeissMan301Model(m) => {
                card_cntr.dispatch_notification(unit, &msg, &self.notified_elem_list, m)
            }
        }
    }

    pub fn measure_elems(
        &mut self,
        unit: &mut (SndDice, FwNode),
        card_cntr: &mut CardCntr,
    ) -> Result<(), Error> {
        match &mut self.model {
            Model::Minimal(m) => card_cntr.measure_elems(unit, &self.measured_elem_list, m),
            Model::TcK24d(m) => card_cntr.measure_elems(unit, &self.measured_elem_list, m),
            Model::TcK8(m) => card_cntr.measure_elems(unit, &self.measured_elem_list, m),
            Model::TcStudiok48(m) => card_cntr.measure_elems(unit, &self.measured_elem_list, m),
            Model::TcKlive(m) => card_cntr.measure_elems(unit, &self.measured_elem_list, m),
            Model::TcDesktopk6(m) => card_cntr.measure_elems(unit, &self.measured_elem_list, m),
            Model::TcItwin(m) => card_cntr.measure_elems(unit, &self.measured_elem_list, m),
            Model::AlesisIo14fw(m) => card_cntr.measure_elems(unit, &self.measured_elem_list, m),
            Model::AlesisIo26fw(m) => card_cntr.measure_elems(unit, &self.measured_elem_list, m),
            Model::LexiconIonix(m) => card_cntr.measure_elems(unit, &self.measured_elem_list, m),
            Model::PresonusFStudio(m) => card_cntr.measure_elems(unit, &self.measured_elem_list, m),
            Model::Extension(m) => card_cntr.measure_elems(unit, &self.measured_elem_list, m),
            Model::MaudioPfire2626(m) => card_cntr.measure_elems(unit, &self.measured_elem_list, m),
            Model::MaudioPfire610(m) => card_cntr.measure_elems(unit, &self.measured_elem_list, m),
            Model::AvidMbox3(m) => card_cntr.measure_elems(unit, &self.measured_elem_list, m),
            Model::LoudBlackbird(m) => card_cntr.measure_elems(unit, &self.measured_elem_list, m),
            Model::FocusriteSPro40(m) => card_cntr.measure_elems(unit, &self.measured_elem_list, m),
            Model::FocusriteSPro40D3(m) => card_cntr.measure_elems(unit, &self.measured_elem_list, m),
            Model::FocusriteLiquidS56(m) => {
                card_cntr.measure_elems(unit, &self.measured_elem_list, m)
            }
            Model::FocusriteSPro24(m) => card_cntr.measure_elems(unit, &self.measured_elem_list, m),
            Model::FocusriteSPro24Dsp(m) => {
                card_cntr.measure_elems(unit, &self.measured_elem_list, m)
            }
            Model::FocusriteSPro14(m) => card_cntr.measure_elems(unit, &self.measured_elem_list, m),
            Model::FocusriteSPro26(m) => card_cntr.measure_elems(unit, &self.measured_elem_list, m),
            Model::PresonusFStudioProject(m) => {
                card_cntr.measure_elems(unit, &self.measured_elem_list, m)
            }
            Model::PresonusFStudioTube(m) => {
                card_cntr.measure_elems(unit, &self.measured_elem_list, m)
            }
            Model::PresonusFStudioMobile(m) => {
                card_cntr.measure_elems(unit, &self.measured_elem_list, m)
            }
            Model::WeissAdc2Model(m) => card_cntr.measure_elems(unit, &self.measured_elem_list, m),
            Model::WeissVestaModel(m) => card_cntr.measure_elems(unit, &self.measured_elem_list, m),
            Model::WeissDac2Model(m) => card_cntr.measure_elems(unit, &self.measured_elem_list, m),
            Model::WeissAfi1Model(m) => card_cntr.measure_elems(unit, &self.measured_elem_list, m),
            Model::WeissDac202Model(m) => {
                card_cntr.measure_elems(unit, &self.measured_elem_list, m)
            }
            Model::WeissInt203Model(m) => {
                card_cntr.measure_elems(unit, &self.measured_elem_list, m)
            }
            Model::WeissMan301Model(m) => {
                card_cntr.measure_elems(unit, &self.measured_elem_list, m)
            }
        }
    }

    pub fn store_configuration(&mut self, node: &FwNode) -> Result<(), Error> {
        match &mut self.model {
            Model::Extension(m) => m.store_configuration(node),
            Model::MaudioPfire2626(m) => m.store_configuration(node),
            Model::MaudioPfire610(m) => m.store_configuration(node),
            Model::AvidMbox3(m) => m.store_configuration(node),
            Model::LoudBlackbird(m) => m.store_configuration(node),
            Model::FocusriteSPro40(m) => m.store_configuration(node),
            Model::FocusriteLiquidS56(m) => m.store_configuration(node),
            Model::FocusriteSPro24(m) => m.store_configuration(node),
            Model::FocusriteSPro24Dsp(m) => m.store_configuration(node),
            Model::FocusriteSPro14(m) => m.store_configuration(node),
            Model::FocusriteSPro26(m) => m.store_configuration(node),
            Model::PresonusFStudioProject(m) => m.store_configuration(node),
            Model::PresonusFStudioTube(m) => m.store_configuration(node),
            Model::PresonusFStudioMobile(m) => m.store_configuration(node),
            _ => Ok(()),
        }
    }
}
