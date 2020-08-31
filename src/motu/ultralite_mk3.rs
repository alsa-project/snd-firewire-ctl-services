// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use hinawa::{SndMotu, FwReq};

use crate::card_cntr::{CardCntr, CtlModel};

use super::v3_clk_ctls::V3ClkCtl;
use super::v3_port_ctls::V3PortCtl;

pub struct UltraLiteMk3<'a> {
    req: FwReq,
    clk_ctls: V3ClkCtl<'a>,
    port_ctls: V3PortCtl<'a>,
}

impl<'a> UltraLiteMk3<'a> {
    const CLK_RATE_LABELS: &'a [&'a str] = &[
        "44100", "48000",
        "88200", "96000",
    ];
    const CLK_RATE_VALS: &'a [u8] = &[0x00, 0x01, 0x02, 0x03];

    const CLK_SRC_LABELS: &'a [&'a str] = &[
        "Internal",
        "S/PDIF-on-coax",
    ];
    const CLK_SRC_VALS: &'a [u8] = &[0x00, 0x01];

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
            req: FwReq::new(),
            clk_ctls: V3ClkCtl::new(Self::CLK_RATE_LABELS, Self::CLK_RATE_VALS,
                                    Self::CLK_SRC_LABELS, Self::CLK_SRC_VALS, true),
            port_ctls: V3PortCtl::new(Self::PORT_ASSIGN_LABELS, Self::PORT_ASSIGN_VALS,
                                      true, true, false, false),
        }
    }
}

impl<'a> CtlModel<SndMotu> for UltraLiteMk3<'a> {
    fn load(&mut self, unit: &SndMotu, card_cntr: &mut CardCntr)
        -> Result<(), Error>
    {
        self.clk_ctls.load(unit, card_cntr)?;
        self.port_ctls.load(unit, card_cntr)?;
        Ok(())
    }

    fn read(&mut self, unit: &SndMotu, elem_id: &alsactl::ElemId,
            elem_value: &mut alsactl::ElemValue)
        -> Result<bool, Error>
    {
        if self.clk_ctls.read(unit, &self.req, elem_id, elem_value)? {
            Ok(true)
        } else if self.port_ctls.read(unit, &self.req, elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(&mut self, unit: &SndMotu, elem_id: &alsactl::ElemId, old: &alsactl::ElemValue,
             new: &alsactl::ElemValue)
        -> Result<bool, Error>
    {
        if self.clk_ctls.write(unit, &self.req, elem_id, old, new)? {
            Ok(true)
        } else if self.port_ctls.write(unit, &self.req, elem_id, old, new)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
