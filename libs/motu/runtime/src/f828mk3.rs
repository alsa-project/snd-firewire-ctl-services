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
pub struct F828mk3 {
    req: FwReq,
    clk_ctls: ClkCtl,
    port_assign_ctl: PortAssignCtl,
    opt_iface_ctl: OptIfaceCtl,
    phone_assign_ctl: PhoneAssignCtl,
    word_clk_ctl: WordClkCtl,
    msg_cache: u32,
}

#[derive(Default)]
struct PhoneAssignCtl(Vec<ElemId>);

impl PhoneAssignCtlOperation<F828mk3Protocol> for PhoneAssignCtl {}

#[derive(Default)]
struct WordClkCtl(Vec<ElemId>);

impl WordClkCtlOperation<F828mk3Protocol> for WordClkCtl {}

#[derive(Default)]
struct ClkCtl;

impl V3ClkCtlOperation<F828mk3Protocol> for ClkCtl {}

#[derive(Default)]
struct PortAssignCtl(Vec<ElemId>);

impl V3PortAssignCtlOperation<F828mk3Protocol> for PortAssignCtl {}

#[derive(Default)]
struct OptIfaceCtl;

impl V3OptIfaceCtlOperation<F828mk3Protocol> for OptIfaceCtl {}

impl F828mk3 {
    const NOTIFY_OPERATED: u32 = 0x40000000;
    const NOTIFY_COMPLETED: u32 = 0x00000002;
    const NOTIFY_OPERATED_AND_COMPLETED: u32 = Self::NOTIFY_OPERATED | Self::NOTIFY_COMPLETED;
}

impl CtlModel<SndMotu> for F828mk3 {
    fn load(&mut self, _: &mut SndMotu, card_cntr: &mut CardCntr)
        -> Result<(), Error>
    {
        self.clk_ctls.load(card_cntr)?;
        self.port_assign_ctl.load(card_cntr)
            .map(|mut elem_id_list| self.port_assign_ctl.0.append(&mut elem_id_list))?;
        self.opt_iface_ctl.load(card_cntr)?;
        self.phone_assign_ctl.load(card_cntr)
            .map(|mut elem_id_list| self.phone_assign_ctl.0.append(&mut elem_id_list))?;
        self.word_clk_ctl.load(card_cntr)
            .map(|mut elem_id_list| self.word_clk_ctl.0.append(&mut elem_id_list))?;
        Ok(())
    }

    fn read(&mut self, unit: &mut SndMotu, elem_id: &alsactl::ElemId,
            elem_value: &mut alsactl::ElemValue)
        -> Result<bool, Error>
    {
        if self.clk_ctls.read(unit, &mut self.req, elem_id, elem_value, TIMEOUT_MS)? {
            Ok(true)
        } else if self.port_assign_ctl.read(unit, &mut self.req,  elem_id, elem_value, TIMEOUT_MS)? {
            Ok(true)
        } else if self.opt_iface_ctl.read(unit, &mut self.req, elem_id, elem_value, TIMEOUT_MS)? {
            Ok(true)
        } else if self.phone_assign_ctl.read(unit, &mut self.req, elem_id, elem_value, TIMEOUT_MS)? {
            Ok(true)
        } else if self.word_clk_ctl.read(unit, &mut self.req, elem_id, elem_value, TIMEOUT_MS)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(&mut self, unit: &mut SndMotu, elem_id: &alsactl::ElemId, old: &alsactl::ElemValue,
             new: &alsactl::ElemValue)
        -> Result<bool, Error>
    {
        if self.clk_ctls.write(unit, &mut self.req, elem_id, new, TIMEOUT_MS)? {
            Ok(true)
        } else if self.port_assign_ctl.write(unit, &mut self.req, elem_id, new, TIMEOUT_MS)? {
            Ok(true)
        } else if self.opt_iface_ctl.write(unit, &mut self.req, elem_id, old, new, TIMEOUT_MS)? {
            Ok(true)
        } else if self.phone_assign_ctl.write(unit, &mut self.req, elem_id, new, TIMEOUT_MS)? {
            Ok(true)
        } else if self.word_clk_ctl.write(unit, &mut self.req, elem_id, new, TIMEOUT_MS)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl NotifyModel<SndMotu, u32> for F828mk3 {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<alsactl::ElemId>) {
        elem_id_list.extend_from_slice(&self.port_assign_ctl.0);
        elem_id_list.extend_from_slice(&self.phone_assign_ctl.0);
        elem_id_list.extend_from_slice(&self.word_clk_ctl.0);
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
            //} else if self.word_clk_ctl.read(unit, &self.proto, elem_id, elem_value, TIMEOUT_MS)? {
            //    Ok(true)
            //} else {
                Ok(false)
            //}
        } else {
            Ok(false)
        }
    }
}
