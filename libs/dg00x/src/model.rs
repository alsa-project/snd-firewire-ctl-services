// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::{Error, FileError};

use core::card_cntr;

use ieee1212_config_rom::ConfigRom;
use ta1394::config_rom::Ta1394ConfigRom;

use super::common_ctl::CommonCtl;
use super::monitor_ctl::MonitorCtl;

use std::convert::TryFrom;

pub struct Dg00xModel {
    req: hinawa::FwReq,
    common: CommonCtl,
    monitor: MonitorCtl,
}

impl Dg00xModel {
    pub fn new(raw: &[u8]) -> Result<Self, Error> {
        let config_rom = ConfigRom::try_from(raw)
            .map_err(|e| {
                let label = format!("Malformed configuration ROM detected: {}", e);
                Error::new(FileError::Nxio, &label)
            })?;

        let model = config_rom.get_model()
            .ok_or(Error::new(FileError::Nxio, "Configuration ROM is not for 1394TA standard"))?;

        match model.model_id {
            0x000001 | 0x000002 => Ok(()),
            _ => Err(Error::new(FileError::Nxio, "Not supported.")),
        }?;

        let has_word_bnc = model.model_name.find("003") != None;

        let model = Dg00xModel{
            req: hinawa::FwReq::new(),
            common: CommonCtl::new(has_word_bnc),
            monitor: MonitorCtl::new(),
        };

        Ok(model)
    }
}

impl card_cntr::NotifyModel<hinawa::SndDg00x, bool> for Dg00xModel {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<alsactl::ElemId>) {
        elem_id_list.extend_from_slice(&self.monitor.notified_elems);
    }

    fn parse_notification(&mut self, _: &hinawa::SndDg00x, _: &bool) -> Result<(), Error> {
        Ok(())
    }

    fn read_notified_elem(&mut self, unit: &hinawa::SndDg00x, elem_id: &alsactl::ElemId,
                          elem_value: &mut alsactl::ElemValue)
        -> Result<bool, Error>
    {
        self.monitor.read_notified_elems(unit, &self.req, elem_id, elem_value)
    }
}

impl card_cntr::CtlModel<hinawa::SndDg00x> for Dg00xModel {
    fn load(
        &mut self,
        unit: &mut hinawa::SndDg00x,
        card_cntr: &mut card_cntr::CardCntr,
    ) -> Result<(), Error> {
        self.common.load(&unit, &self.req, card_cntr)?;
        self.monitor.load(&unit, &self.req, card_cntr)?;
        Ok(())
    }

    fn read(
        &mut self,
        unit: &mut hinawa::SndDg00x,
        elem_id: &alsactl::ElemId,
        elem_value: &mut alsactl::ElemValue,
    ) -> Result<bool, Error> {
        if self.common.read(unit, &self.req, elem_id, elem_value)? {
            Ok(true)
        } else if self.monitor.read(unit, &self.req, elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(
        &mut self,
        unit: &mut hinawa::SndDg00x,
        elem_id: &alsactl::ElemId,
        old: &alsactl::ElemValue,
        new: &alsactl::ElemValue,
    ) -> Result<bool, Error> {
        if self.common.write(unit, &self.req, elem_id, old, new)? {
            Ok(true)
        } else if self.monitor.write(unit, &self.req, elem_id, old, new)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
