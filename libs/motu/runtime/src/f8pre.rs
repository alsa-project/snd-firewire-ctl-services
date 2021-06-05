// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use hinawa::SndMotu;

use core::card_cntr::{CardCntr, CtlModel};

use motu_protocols::version_2::*;

use super::common_ctls::*;
use super::v2_ctls::*;
use super::v2_port_ctls::V2PortCtl;

const TIMEOUT_MS: u32 = 100;

pub struct F8pre{
    proto: F8preProtocol,
    clk_ctls: V2ClkCtl,
    opt_iface_ctl: V2OptIfaceCtl,
    phone_assign_ctl: CommonPhoneCtl,
    port_ctls: V2PortCtl,
}

impl F8pre {
    pub fn new() -> Self {
        F8pre{
            proto: Default::default(),
            clk_ctls: Default::default(),
            opt_iface_ctl: Default::default(),
            phone_assign_ctl: Default::default(),
            port_ctls: V2PortCtl::new(&[], &[], false, false, true, false),
        }
    }
}

impl CtlModel<SndMotu> for F8pre {
    fn load(&mut self, unit: &SndMotu, card_cntr: &mut CardCntr)
        -> Result<(), Error>
    {
        self.clk_ctls.load(&self.proto, card_cntr)?;
        self.opt_iface_ctl.load(&self.proto, card_cntr)?;
        self.phone_assign_ctl.load(&self.proto, card_cntr)?;
        self.port_ctls.load(unit, card_cntr)?;
        Ok(())
    }

    fn read(&mut self, unit: &SndMotu, elem_id: &alsactl::ElemId,
            elem_value: &mut alsactl::ElemValue)
        -> Result<bool, Error>
    {
        if self.clk_ctls.read(unit, &self.proto, elem_id, elem_value, TIMEOUT_MS)? {
            Ok(true)
        } else if self.opt_iface_ctl.read(unit, &self.proto, elem_id, elem_value, TIMEOUT_MS)? {
            Ok(true)
        } else if self.phone_assign_ctl.read(unit, &self.proto, elem_id, elem_value, TIMEOUT_MS)? {
            Ok(true)
        } else if self.port_ctls.read(unit, &self.proto, elem_id, elem_value)? {
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
        } else if self.opt_iface_ctl.write(unit, &self.proto, elem_id, old, new, TIMEOUT_MS)? {
            Ok(true)
        } else if self.phone_assign_ctl.write(unit, &self.proto, elem_id, old, new, TIMEOUT_MS)? {
            Ok(true)
        } else if self.port_ctls.write(unit, &self.proto, elem_id, old, new)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
