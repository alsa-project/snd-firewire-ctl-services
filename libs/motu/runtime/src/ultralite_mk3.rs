// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use hinawa::SndMotu;

use core::card_cntr::{CardCntr, CtlModel, NotifyModel};

use motu_protocols::version_3::*;

use super::v3_ctls::*;
use super::v3_port_ctls::V3PortCtl;

const TIMEOUT_MS: u32 = 100;

pub struct UltraLiteMk3<'a> {
    proto: UltraliteMk3Protocol,
    clk_ctls: V3ClkCtl,
    port_assign_ctl: V3PortAssignCtl,
    port_ctls: V3PortCtl<'a>,
    msg_cache: u32,
}

impl<'a> UltraLiteMk3<'a> {
    const NOTIFY_OPERATED: u32 = 0x40000000;
    const NOTIFY_COMPLETED: u32 = 0x00000002;
    const NOTIFY_OPERATED_AND_COMPLETED: u32 = Self::NOTIFY_OPERATED | Self::NOTIFY_COMPLETED;

    const PORT_ASSIGN_LABELS: &'a [&'a str] = &[
        "Main-1/2",     // = Stream-1/2
        "Analog-1/2",   // = Stream-3/4
        "Analog-3/4",   // = Stream-5/6
        "Analog-5/6",   // = Stream-7/8
        "Analog-7/8",   // = Stream-9/10
        "S/PDIF-1/2",   // = Stream-13/14
        "Phone-1/2",    // = Stream-11/12
    ];
    const PORT_ASSIGN_VALS: &'a [u8] = &[0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06];

    pub fn new() -> Self {
        UltraLiteMk3{
            proto: Default::default(),
            clk_ctls: Default::default(),
            port_assign_ctl: Default::default(),
            port_ctls: V3PortCtl::new(Self::PORT_ASSIGN_LABELS, Self::PORT_ASSIGN_VALS,
                                      true, true, false, false),
            msg_cache: 0,
        }
    }
}

impl<'a> CtlModel<SndMotu> for UltraLiteMk3<'a> {
    fn load(&mut self, unit: &SndMotu, card_cntr: &mut CardCntr)
        -> Result<(), Error>
    {
        self.clk_ctls.load(&self.proto, card_cntr)?;
        self.port_assign_ctl.load(&self.proto, card_cntr)?;
        self.port_ctls.load(unit, card_cntr)?;
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
        } else if self.port_assign_ctl.write(unit, &self.proto, elem_id, old, new, TIMEOUT_MS)? {
            Ok(true)
        } else if self.port_ctls.write(unit, &self.proto, elem_id, old, new)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl<'a> NotifyModel<SndMotu, u32> for UltraLiteMk3<'a> {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<alsactl::ElemId>) {
        elem_id_list.extend_from_slice(&self.port_ctls.notified_elems);
        elem_id_list.extend_from_slice(&self.port_assign_ctl.0);
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
            if self.port_ctls.read(unit, &self.proto, elem_id, elem_value)? {
                Ok(true)
            } else if self.port_assign_ctl.read(unit, &self.proto, elem_id, elem_value, TIMEOUT_MS)? {
                Ok(true)
            } else {
                Ok(false)
            }
        } else {
            Ok(false)
        }
    }
}
