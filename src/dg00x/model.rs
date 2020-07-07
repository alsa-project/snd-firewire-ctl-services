// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::{Error, FileError};

use crate::ieee1212;
use crate::ta1394;

use crate::card_cntr;

use super::common_ctl::CommonCtl;
use super::monitor_ctl::MonitorCtl;

pub struct Dg00xModel {
    req: hinawa::FwReq,
    common: CommonCtl,
    monitor: MonitorCtl,
}

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

        let has_word_bnc = data.model_name.find("003") != None;

        let model = Dg00xModel{
            req: hinawa::FwReq::new(),
            common: CommonCtl::new(has_word_bnc),
            monitor: MonitorCtl::new(),
        };

        Ok(model)
    }
}

impl card_cntr::MonitorModel<hinawa::SndDg00x> for Dg00xModel {
    fn get_monitored_elems(&mut self, elem_id_list: &mut Vec<alsactl::ElemId>) {
        elem_id_list.extend_from_slice(&self.monitor.monitored_elems);
    }

    fn monitor_unit(&mut self, _: &hinawa::SndDg00x) -> Result<(), Error> {
        Ok(())
    }

    fn monitor_elems(
        &mut self,
        unit: &hinawa::SndDg00x,
        elem_id: &alsactl::ElemId,
        old: &alsactl::ElemValue,
        _: &mut alsactl::ElemValue,
    ) -> Result<bool, Error> {
        self.monitor.monitor_elems(unit, &self.req, elem_id, old)
    }
}

impl card_cntr::CtlModel<hinawa::SndDg00x> for Dg00xModel {
    fn load(
        &mut self,
        unit: &hinawa::SndDg00x,
        card_cntr: &mut card_cntr::CardCntr,
    ) -> Result<(), Error> {
        self.common.load(&unit, &self.req, card_cntr)?;
        self.monitor.load(&unit, &self.req, card_cntr)?;
        Ok(())
    }

    fn read(
        &mut self,
        unit: &hinawa::SndDg00x,
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
        unit: &hinawa::SndDg00x,
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
