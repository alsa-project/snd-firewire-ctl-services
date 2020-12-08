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
use super::extension_model::ExtensionModel;
use super::pfire_model::*;

enum Model {
    Minimal(MinimalModel),
    Extension(ExtensionModel),
    MaudioPfire2626(Pfire2626Model),
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
            (0x000d6c, 0x000010) => Model::MaudioPfire2626(Pfire2626Model::default()),
            _ => Model::Minimal(MinimalModel::default()),
        };

        let notified_elem_list = Vec::new();
        let measured_elem_list = Vec::new();

        Ok(DiceModel{model, notified_elem_list, measured_elem_list})
    }
    pub fn load(&mut self, unit: &SndDice, card_cntr: &mut CardCntr)
        -> Result<(), Error>
    {
        // Replace model data when protocol extension is available.
        if let Model::Minimal(_) = &mut self.model {
            let proto = FwReq::default();
            if proto.read_extension_sections(&unit.get_node(), 100).is_ok() {
                self.model = Model::Extension(ExtensionModel::default());
            } else {
                // MEMO: workaround for old firmware. Invalidate a negative effect by failure of
                // previous transaction.
                let _ = proto.read_general_sections(&unit.get_node(), 100);
            }
        }

        match &mut self.model {
            Model::Minimal(m) => m.load(unit, card_cntr),
            Model::Extension(m) => m.load(unit, card_cntr),
            Model::MaudioPfire2626(m) => m.load(unit, card_cntr),
        }?;

        match &mut self.model {
            Model::Minimal(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
            Model::Extension(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
            Model::MaudioPfire2626(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
        }

        match &mut self.model {
            Model::Minimal(m) => m.get_measure_elem_list(&mut self.measured_elem_list),
            Model::Extension(m) => m.get_measure_elem_list(&mut self.measured_elem_list),
            Model::MaudioPfire2626(m) => m.get_measure_elem_list(&mut self.measured_elem_list),
        }

        Ok(())
    }

    pub fn dispatch_elem_event(&mut self, unit: &SndDice, card_cntr: &mut CardCntr,
                               elem_id: &alsactl::ElemId, events: &alsactl::ElemEventMask)
        -> Result<(), Error>
    {
        match &mut self.model {
            Model::Minimal(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
            Model::Extension(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
            Model::MaudioPfire2626(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
        }
    }

    pub fn dispatch_msg(&mut self, unit: &SndDice, card_cntr: &mut CardCntr, msg: u32)
        -> Result<(), Error>
    {
        match &mut self.model {
            Model::Minimal(m) => card_cntr.dispatch_notification(unit, &msg, &self.notified_elem_list, m),
            Model::Extension(m) => card_cntr.dispatch_notification(unit, &msg, &self.notified_elem_list, m),
            Model::MaudioPfire2626(m) => card_cntr.dispatch_notification(unit, &msg, &self.notified_elem_list, m),
        }
    }

    pub fn measure_elems(&mut self, unit: &hinawa::SndDice, card_cntr: &mut CardCntr)
        -> Result<(), Error>
    {
        match &mut self.model {
            Model::Minimal(m) => card_cntr.measure_elems(unit, &self.measured_elem_list, m),
            Model::Extension(m) => card_cntr.measure_elems(unit, &self.measured_elem_list, m),
            Model::MaudioPfire2626(m) => card_cntr.measure_elems(unit, &self.measured_elem_list, m),
        }
    }
}
