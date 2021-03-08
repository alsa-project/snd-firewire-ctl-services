// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto
use glib::{Error, FileError};

use hinawa::FwNodeExtManual;
use hinawa::{SndUnit, SndUnitExt};

use core::card_cntr::*;

use ieee1212_config_rom::*;

use ff_protocols::*;

use super::ff800_model::*;

use std::convert::TryFrom;

pub enum Model {
    Ff800(Ff800Model),
}

pub struct FfModel{
    model: Model,
}

impl FfModel {
    pub fn new(unit: &SndUnit) -> Result<FfModel, Error> {
        let node = unit.get_node();
        let raw = node.get_config_rom()?;
        let config_rom = ConfigRom::try_from(&raw[..])
            .map_err(|e| {
                let msg = format!("Malformed configuration ROM detected: {}", e);
                Error::new(FileError::Nxio, &msg)
            })?;
        let model_id = config_rom.get_model_id()
            .ok_or_else(|| Error::new(FileError::Nxio, "Unexpected format of configuration ROM"))?;

        let model = match model_id {
            0x00000001 => Model::Ff800(Ff800Model::default()),
            _ => Err(Error::new(FileError::Nxio, "Not supported."))?,
        };

        Ok(FfModel{model})
    }
    pub fn load(&mut self, unit: &SndUnit, card_cntr: &mut CardCntr) -> Result<(), Error> {
        match &mut self.model {
            Model::Ff800(m) => m.load(unit, card_cntr),
        }
    }

    pub fn dispatch_elem_event(&mut self, unit: &SndUnit, card_cntr: &mut CardCntr,
                               elem_id: &alsactl::ElemId, events: &alsactl::ElemEventMask)
        -> Result<(), Error>
    {
        match &mut self.model {
            Model::Ff800(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
        }
    }
}
