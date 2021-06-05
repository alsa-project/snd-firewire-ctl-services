// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use hinawa::SndMotu;

use core::card_cntr::{CardCntr, CtlModel, NotifyModel};

use motu_protocols::version_3::*;

use super::common_ctls::*;
use super::v3_ctls::*;

const TIMEOUT_MS: u32 = 100;

pub struct UltraLiteMk3{
    proto: UltraliteMk3Protocol,
    clk_ctls: V3ClkCtl,
    port_assign_ctl: V3PortAssignCtl,
    phone_assign_ctl: CommonPhoneCtl,
    msg_cache: u32,
}

impl UltraLiteMk3 {
    const NOTIFY_OPERATED: u32 = 0x40000000;
    const NOTIFY_COMPLETED: u32 = 0x00000002;
    const NOTIFY_OPERATED_AND_COMPLETED: u32 = Self::NOTIFY_OPERATED | Self::NOTIFY_COMPLETED;

    pub fn new() -> Self {
        UltraLiteMk3{
            proto: Default::default(),
            clk_ctls: Default::default(),
            port_assign_ctl: Default::default(),
            phone_assign_ctl: Default::default(),
            msg_cache: 0,
        }
    }
}

impl CtlModel<SndMotu> for UltraLiteMk3 {
    fn load(&mut self, _: &SndMotu, card_cntr: &mut CardCntr)
        -> Result<(), Error>
    {
        self.clk_ctls.load(&self.proto, card_cntr)?;
        self.port_assign_ctl.load(&self.proto, card_cntr)?;
        self.phone_assign_ctl.load(&self.proto, card_cntr)?;
        Ok(())
    }

    fn read(&mut self, unit: &SndMotu, elem_id: &alsactl::ElemId,
            elem_value: &mut alsactl::ElemValue)
        -> Result<bool, Error>
    {
        if self.clk_ctls.read(unit, &self.proto, elem_id, elem_value, TIMEOUT_MS)? {
            Ok(true)
        } else if self.port_assign_ctl.read(unit, &self.proto, elem_id, elem_value, TIMEOUT_MS)? {
            Ok(true)
        } else if self.phone_assign_ctl.read(unit, &self.proto, elem_id, elem_value, TIMEOUT_MS)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(&mut self, unit: &SndMotu, elem_id: &alsactl::ElemId, old: &alsactl::ElemValue,
             new: &alsactl::ElemValue)
        -> Result<bool, Error>
    {
        if self.clk_ctls.write(unit, &self.proto, elem_id, old, new, TIMEOUT_MS)? {
            Ok(true)
        } else if self.port_assign_ctl.write(unit, &self.proto, elem_id, old, new, TIMEOUT_MS)? {
            Ok(true)
        } else if self.phone_assign_ctl.write(unit, &self.proto, elem_id, old, new, TIMEOUT_MS)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl NotifyModel<SndMotu, u32> for UltraLiteMk3 {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<alsactl::ElemId>) {
        elem_id_list.extend_from_slice(&self.port_assign_ctl.0);
        elem_id_list.extend_from_slice(&self.phone_assign_ctl.0);
    }

    fn parse_notification(&mut self, _: &SndMotu, msg: &u32) -> Result<(), Error> {
        self.msg_cache = *msg;
        Ok(())
    }

    fn read_notified_elem(&mut self, unit: &SndMotu, elem_id: &alsactl::ElemId,
                          elem_value: &mut alsactl::ElemValue)
        -> Result<bool, Error>
    {
        if self.msg_cache & (Self::NOTIFY_OPERATED_AND_COMPLETED) == Self::NOTIFY_OPERATED_AND_COMPLETED {
            if self.port_assign_ctl.read(unit, &self.proto, elem_id, elem_value, TIMEOUT_MS)? {
                Ok(true)
            } else if self.phone_assign_ctl.read(unit, &self.proto, elem_id, elem_value, TIMEOUT_MS)? {
                Ok(true)
            } else {
                Ok(false)
            }
        } else {
            Ok(false)
        }
    }
}
