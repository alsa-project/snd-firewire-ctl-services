// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use hinawa::FwReq;
use hinawa::SndMotu;

use core::card_cntr::{CardCntr, CtlModel};

use motu_protocols::version_2::*;

use super::common_ctls::*;
use super::v2_ctls::*;

const TIMEOUT_MS: u32 = 100;

#[derive(Default)]
pub struct F8pre{
    req: FwReq,
    clk_ctls: ClkCtl,
    opt_iface_ctl: OptIfaceCtl,
    phone_assign_ctl: PhoneAssignCtl,
}

#[derive(Default)]
struct PhoneAssignCtl;

impl PhoneAssignCtlOperation<F8preProtocol> for PhoneAssignCtl {}

#[derive(Default)]
struct ClkCtl;

impl V2ClkCtlOperation<F8preProtocol> for ClkCtl {}

#[derive(Default)]
struct OptIfaceCtl;

impl V2OptIfaceCtlOperation<F8preProtocol> for OptIfaceCtl {}

impl CtlModel<SndMotu> for F8pre {
    fn load(&mut self, _: &mut SndMotu, card_cntr: &mut CardCntr)
        -> Result<(), Error>
    {
        self.clk_ctls.load(card_cntr)?;
        self.opt_iface_ctl.load(card_cntr)?;
        let _ = self.phone_assign_ctl.load(card_cntr)?;
        Ok(())
    }

    fn read(&mut self, unit: &mut SndMotu, elem_id: &alsactl::ElemId,
            elem_value: &mut alsactl::ElemValue)
        -> Result<bool, Error>
    {
        if self.clk_ctls.read(unit, &mut self.req, elem_id, elem_value, TIMEOUT_MS)? {
            Ok(true)
        } else if self.opt_iface_ctl.read(unit, &mut self.req, elem_id, elem_value, TIMEOUT_MS)? {
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
        } else if self.opt_iface_ctl.write(unit, &mut self.req, elem_id, new, TIMEOUT_MS)? {
            Ok(true)
        } else if self.phone_assign_ctl.write(unit, &mut self.req, elem_id, new, TIMEOUT_MS)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
