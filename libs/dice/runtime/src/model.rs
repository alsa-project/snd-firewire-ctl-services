// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::{Error, FileError};

use hinawa::FwNodeExtManual;
use hinawa::{SndDice, SndUnitExt};

use core::card_cntr::*;

use ieee1212_config_rom::*;
use dice_protocols::tcat::config_rom::*;

use std::convert::TryFrom;

use super::minimal_model::MinimalModel;

enum Model {
    Minimal(MinimalModel),
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
            _ => Model::Minimal(MinimalModel::default()),
        };

        let notified_elem_list = Vec::new();
        let measured_elem_list = Vec::new();

        Ok(DiceModel{model, notified_elem_list, measured_elem_list})
    }
    pub fn load(&mut self, unit: &SndDice, card_cntr: &mut CardCntr)
        -> Result<(), Error>
    {
        match &mut self.model {
            Model::Minimal(m) => m.load(unit, card_cntr),
        }?;

        match &mut self.model {
            Model::Minimal(m) => m.get_notified_elem_list(&mut self.notified_elem_list),
        }

        match &mut self.model {
            Model::Minimal(m) => m.get_measure_elem_list(&mut self.measured_elem_list),
        }

        Ok(())
    }

    pub fn dispatch_elem_event(&mut self, unit: &SndDice, card_cntr: &mut CardCntr,
                               elem_id: &alsactl::ElemId, events: &alsactl::ElemEventMask)
        -> Result<(), Error>
    {
        match &mut self.model {
            Model::Minimal(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
        }
    }

    pub fn dispatch_msg(&mut self, unit: &SndDice, card_cntr: &mut CardCntr, msg: u32)
        -> Result<(), Error>
    {
        match &mut self.model {
            Model::Minimal(m) => card_cntr.dispatch_notification(unit, &msg, &self.notified_elem_list, m),
        }
    }

    pub fn measure_elems(&mut self, unit: &hinawa::SndDice, card_cntr: &mut CardCntr)
        -> Result<(), Error>
    {
        match &mut self.model {
            Model::Minimal(m) => card_cntr.measure_elems(unit, &self.measured_elem_list, m),
        }
    }
}
