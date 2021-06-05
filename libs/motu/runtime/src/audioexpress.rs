// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use hinawa::SndMotu;

use core::card_cntr::{CardCntr, CtlModel};

use motu_protocols::version_3::*;

use super::v3_ctls::*;
use super::v3_port_ctls::V3PortCtl;

const TIMEOUT_MS: u32 = 100;

pub struct AudioExpress<'a> {
    proto: AudioExpressProtocol,
    clk_ctls: V3ClkCtl,
    port_ctls: V3PortCtl<'a>,
}

impl<'a> AudioExpress<'a> {
    const PORT_ASSIGN_LABELS: &'a [&'a str] = &[
        "Phone-1/2",    // = Stream-1/2
        "Main-1/2",     // = Stream-5/6
        "Andlog-1/2",   // = Stream-3/4
        "S/PDIF-1/2",   // = Stream-7/8
        // Blank for Stream-9/10
    ];
    const PORT_ASSIGN_VALS: &'a [u8] = &[0x01, 0x02, 0x06, 0x07];

    pub fn new() -> Self {
        AudioExpress{
            proto: Default::default(),
            clk_ctls: Default::default(),
            port_ctls: V3PortCtl::new(Self::PORT_ASSIGN_LABELS, Self::PORT_ASSIGN_VALS,
                                      false, false, false, false),
        }
    }
}

impl<'a> CtlModel<SndMotu> for AudioExpress<'a> {
    fn load(&mut self, unit: &SndMotu, card_cntr: &mut CardCntr)
        -> Result<(), Error>
    {
        self.clk_ctls.load(&self.proto, card_cntr)?;
        self.port_ctls.load(unit, card_cntr)?;
        Ok(())
    }

    fn read(&mut self, unit: &SndMotu, elem_id: &alsactl::ElemId,
            elem_value: &mut alsactl::ElemValue)
        -> Result<bool, Error>
    {
        if self.clk_ctls.read(unit, &self.proto, elem_id, elem_value, TIMEOUT_MS)? {
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
        } else if self.port_ctls.write(unit, &self.proto, elem_id, old, new)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
