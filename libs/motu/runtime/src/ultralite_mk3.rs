// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use hinawa::FwReq;
use hinawa::SndMotu;

use alsactl::{ElemId, ElemValue};

use core::card_cntr::{CardCntr, CtlModel, NotifyModel};

use motu_protocols::version_3::*;

use super::common_ctls::*;
use super::v3_ctls::*;

const TIMEOUT_MS: u32 = 100;

#[derive(Default)]
pub struct UltraLiteMk3 {
    req: FwReq,
    clk_ctls: ClkCtl,
    port_assign_ctl: PortAssignCtl,
    phone_assign_ctl: PhoneAssignCtl,
    msg_cache: u32,
}

#[derive(Default)]
struct PhoneAssignCtl(Vec<ElemId>);

impl PhoneAssignCtlOperation<UltraliteMk3Protocol> for PhoneAssignCtl {}

#[derive(Default)]
struct ClkCtl;

impl V3ClkCtlOperation<UltraliteMk3Protocol> for ClkCtl {}

#[derive(Default)]
struct PortAssignCtl(Vec<ElemId>);

impl V3PortAssignCtlOperation<UltraliteMk3Protocol> for PortAssignCtl {}

impl UltraLiteMk3 {
    const NOTIFY_OPERATED: u32 = 0x40000000;
    const NOTIFY_COMPLETED: u32 = 0x00000002;
    const NOTIFY_OPERATED_AND_COMPLETED: u32 = Self::NOTIFY_OPERATED | Self::NOTIFY_COMPLETED;
}

impl CtlModel<SndMotu> for UltraLiteMk3 {
    fn load(&mut self, _: &mut SndMotu, card_cntr: &mut CardCntr)
        -> Result<(), Error>
    {
        self.clk_ctls.load(card_cntr)?;
        self.port_assign_ctl.load(card_cntr)
            .map(|mut elem_id_list| self.port_assign_ctl.0.append(&mut elem_id_list))?;
        self.phone_assign_ctl.load(card_cntr)
            .map(|mut elem_id_list| self.phone_assign_ctl.0.append(&mut elem_id_list))?;
        Ok(())
    }

    fn read(&mut self, unit: &mut SndMotu, elem_id: &alsactl::ElemId,
            elem_value: &mut alsactl::ElemValue)
        -> Result<bool, Error>
    {
        if self.clk_ctls.read(unit, &mut self.req, elem_id, elem_value, TIMEOUT_MS)? {
            Ok(true)
        } else if self.port_assign_ctl.read(unit, &mut self.req, elem_id, elem_value, TIMEOUT_MS)? {
            Ok(true)
        } else if self.phone_assign_ctl.read(unit, &mut self.req, elem_id, elem_value, TIMEOUT_MS)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(&mut self, unit: &mut SndMotu, elem_id: &alsactl::ElemId, _: &alsactl::ElemValue,
             new: &alsactl::ElemValue)
        -> Result<bool, Error>
    {
        if self.clk_ctls.write(unit, &mut self.req, elem_id, new, TIMEOUT_MS)? {
            Ok(true)
        } else if self.port_assign_ctl.write(unit, &mut self.req, elem_id, new, TIMEOUT_MS)? {
            Ok(true)
        } else if self.phone_assign_ctl.write(unit, &mut self.req, elem_id, new, TIMEOUT_MS)? {
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
        if self.msg_cache & (Self::NOTIFY_OPERATED_AND_COMPLETED) == Self::NOTIFY_OPERATED_AND_COMPLETED {
            //if self.port_assign_ctl.read(unit, &self.proto, elem_id, elem_value, TIMEOUT_MS)? {
            //    Ok(true)
            //} else if self.phone_assign_ctl.read(unit, &self.proto, elem_id, elem_value, TIMEOUT_MS)? {
            //    Ok(true)
            //} else {
                Ok(false)
            //}
        } else {
            Ok(false)
        }
    }
}
