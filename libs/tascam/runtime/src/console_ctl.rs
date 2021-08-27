// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use hinawa::SndUnitExt;

use core::card_cntr;
use core::elem_value_accessor::ElemValueAccessor;

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
                ElemValueAccessor::<bool>::set_val(elem_value, || {
                    let val = req.get_master_fader_assign(&node)?;
                    Ok(val)
                })?;
                Ok(true)
            }
            Self::HOST_MODE_NAME => {
                ElemValueAccessor::<bool>::set_val(elem_value, || Ok(self.host_mode))?;
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
                ElemValueAccessor::<bool>::get_val(new, |val| req.set_master_fader_assign(&node, val))?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
