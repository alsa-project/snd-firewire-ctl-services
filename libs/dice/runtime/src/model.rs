// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::{Error, FileError};

use hinawa::FwNodeExtManual;
use hinawa::SndUnitExt;
use hinawa::SndDice;

use core::card_cntr::*;

use ieee1212_config_rom::*;
use dice_protocols::tcat::config_rom::*;

use std::convert::TryFrom;

pub struct DiceModel{
    model: (),
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
            _ => (),
        };

        Ok(DiceModel{model})
    }
    pub fn load(&mut self, _: &SndDice, _: &mut CardCntr)
        -> Result<(), Error>
    {
        Ok(())
    }

    pub fn dispatch_elem_event(&mut self, _: &SndDice, _: &mut CardCntr,
                               _: &alsactl::ElemId, _: &alsactl::ElemEventMask)
        -> Result<(), Error>
    {
        Ok(())
    }
}
