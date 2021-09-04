// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use hinawa::FwReq;
use hinawa::SndMotu;

use alsactl::{ElemId, ElemValue};

use core::card_cntr::{CardCntr, CtlModel, NotifyModel};

use motu_protocols::version_2::*;

use super::common_ctls::*;
use super::v2_ctls::*;

const TIMEOUT_MS: u32 = 100;

#[derive(Default)]
pub struct Traveler {
    req: FwReq,
    proto: TravelerProtocol,
    clk_ctls: V2ClkCtl,
    opt_iface_ctl: V2OptIfaceCtl,
    phone_assign_ctl: CommonPhoneCtl,
    word_clk_ctl: CommonWordClkCtl,
    msg_cache: u32,
}

impl Traveler {
    const NOTIFY_PORT_CHANGE: u32 = 0x40000000;
    //const NOTIFY_FORMAT_CHANGE: u32 = 0x08000000; // The format for payload of isochronous packet is changed.
}

impl CtlModel<SndMotu> for Traveler {
    fn load(&mut self, _: &mut SndMotu, card_cntr: &mut CardCntr)
        -> Result<(), Error>
    {
        self.clk_ctls.load(&self.proto, card_cntr)?;
        self.opt_iface_ctl.load(&self.proto, card_cntr)?;
        self.phone_assign_ctl.load(&self.proto, card_cntr)?;
        self.word_clk_ctl.load(&self.proto, card_cntr)?;
        Ok(())
    }

    fn read(&mut self, unit: &mut SndMotu, elem_id: &alsactl::ElemId,
            elem_value: &mut alsactl::ElemValue)
        -> Result<bool, Error>
    {
        if self.clk_ctls.read(unit, &self.proto, elem_id, elem_value, TIMEOUT_MS)? {
            Ok(true)
        } else if self.opt_iface_ctl.read(unit, &self.proto, elem_id, elem_value, TIMEOUT_MS)? {
            Ok(true)
        } else if self.phone_assign_ctl.read(unit, &mut self.req, &self.proto, elem_id, elem_value, TIMEOUT_MS)? {
            Ok(true)
        } else if self.word_clk_ctl.read(unit, &self.proto, elem_id, elem_value, TIMEOUT_MS)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(&mut self, unit: &mut SndMotu, elem_id: &alsactl::ElemId, old: &alsactl::ElemValue,
             new: &alsactl::ElemValue)
        -> Result<bool, Error>
    {
        if self.clk_ctls.write(unit, &self.proto, elem_id, old, new, TIMEOUT_MS)? {
            Ok(true)
        } else if self.opt_iface_ctl.write(unit, &self.proto, elem_id, old, new, TIMEOUT_MS)? {
            Ok(true)
        } else if self.phone_assign_ctl.write(unit, &mut self.req, &self.proto, elem_id, old, new, TIMEOUT_MS)? {
            Ok(true)
        } else if self.word_clk_ctl.write(unit, &self.proto, elem_id, old, new, TIMEOUT_MS)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl NotifyModel<SndMotu, u32> for Traveler {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<alsactl::ElemId>) {
        elem_id_list.extend_from_slice(&self.phone_assign_ctl.0);
    }

    fn parse_notification(&mut self, _: &mut SndMotu, msg: &u32) -> Result<(), Error> {
        self.msg_cache = *msg;
        Ok(())
    }

    fn read_notified_elem(
        &mut self,
        _: &SndMotu,
        _: &ElemId,
        _: &mut ElemValue
    ) -> Result<bool, Error> {
        if self.msg_cache & Self::NOTIFY_PORT_CHANGE > 0 {
            //if self.phone_assign_ctl.read(unit, &self.proto, elem_id, elem_value, TIMEOUT_MS)? {
            //    Ok(true)
            //} else {
                Ok(false)
            //}
        } else {
            Ok(false)
        }
    }
}
