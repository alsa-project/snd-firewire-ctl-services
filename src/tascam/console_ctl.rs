// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use hinawa::SndUnitExt;

use alsactl::ElemValueExtManual;

use crate::card_cntr;

use super::protocol::ConsoleProtocol;

pub struct ConsoleCtl {
    host_mode: bool,
    monitored_elems: Vec<alsactl::ElemId>,
}

impl<'a> ConsoleCtl {
    const MASTER_FADER_ASSIGN_NAME: &'a str = "master-fader-assign";
    const HOST_MODE_NAME: &'a str = "host-mode";

    pub fn new() -> Self {
        ConsoleCtl {
            host_mode: false,
            monitored_elems: Vec::new(),
        }
    }

    pub fn parse_states(&mut self, states: &[u32; 64]) {
        self.host_mode = (states[5] & 0xff000000) != 0xff000000;
    }

    pub fn get_monitored_elems(&self) -> &Vec<alsactl::ElemId> {
        &self.monitored_elems
    }

    pub fn load(
        &mut self,
        _: &hinawa::SndTscm,
        _: &hinawa::FwReq,
        card_cntr: &mut card_cntr::CardCntr,
    ) -> Result<(), Error> {
        // For assignment of master fader to analog output.
        let elem_id = alsactl::ElemId::new_by_name(
            alsactl::ElemIfaceType::Mixer,
            0,
            0,
            Self::MASTER_FADER_ASSIGN_NAME,
            0,
        );
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        // For host mode.
        let elem_id = alsactl::ElemId::new_by_name(
            alsactl::ElemIfaceType::Mixer,
            0,
            0,
            Self::HOST_MODE_NAME,
            0,
        );
        let mut elem_id_list = card_cntr.add_bool_elems(&elem_id, 1, 1, false)?;
        self.monitored_elems.push(elem_id_list.remove(0));

        Ok(())
    }

    pub fn read(
        &mut self,
        unit: &hinawa::SndTscm,
        req: &hinawa::FwReq,
        elem_id: &alsactl::ElemId,
        elem_value: &mut alsactl::ElemValue,
    ) -> Result<bool, Error> {
        let node = unit.get_node();

        match elem_id.get_name().as_str() {
            Self::MASTER_FADER_ASSIGN_NAME => {
                let val = req.get_master_fader_assign(&node)?;
                elem_value.set_bool(&[val]);
                Ok(true)
            }
            Self::HOST_MODE_NAME => {
                elem_value.set_bool(&[self.host_mode]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub fn write(
        &mut self,
        unit: &hinawa::SndTscm,
        req: &hinawa::FwReq,
        elem_id: &alsactl::ElemId,
        _: &alsactl::ElemValue,
        new: &alsactl::ElemValue,
    ) -> Result<bool, Error> {
        let node = unit.get_node();

        match elem_id.get_name().as_str() {
            Self::MASTER_FADER_ASSIGN_NAME => {
                let mut vals = [false];
                new.get_bool(&mut vals);
                req.set_master_fader_assign(&node, vals[0])?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
