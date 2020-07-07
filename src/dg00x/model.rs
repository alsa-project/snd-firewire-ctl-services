// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::{Error, FileError};

use crate::ieee1212;
use crate::ta1394;

use crate::card_cntr;

pub struct Dg00xModel {}

impl Dg00xModel {
    pub fn new(config_rom: &[u8]) -> Result<Self, Error> {
        let entries = ieee1212::get_root_entry_list(&config_rom);

        let data = match ta1394::config_rom::get_unit_data(&entries, 0) {
            Some(d) => d,
            None => return Err(Error::new(FileError::Nxio, "Not supported.")),
        };

        match data.model_id {
            0x000001 | 0x000002 => (),
            _ => return Err(Error::new(FileError::Nxio, "Not supported.")),
        }

        let model = Dg00xModel{};

        Ok(model)
    }
}

impl card_cntr::CtlModel<hinawa::SndDg00x> for Dg00xModel {
    fn load(
        &mut self,
        _: &hinawa::SndDg00x,
        _: &mut card_cntr::CardCntr,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn read(
        &mut self,
        _: &hinawa::SndDg00x,
        _: &alsactl::ElemId,
        _: &mut alsactl::ElemValue,
    ) -> Result<bool, Error> {
        Ok(false)
    }

    fn write(
        &mut self,
        _: &hinawa::SndDg00x,
        _: &alsactl::ElemId,
        _: &alsactl::ElemValue,
        _: &alsactl::ElemValue,
    ) -> Result<bool, Error> {
        Ok(false)
    }
}
