// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use hinawa::SndMotu;

use core::card_cntr::{CardCntr, CtlModel};

use motu_protocols::version_2::*;

use super::v2_clk_ctls::V2ClkCtl;
use super::v2_port_ctls::V2PortCtl;

pub struct F8pre<'a> {
    proto: F8preProtocol,
    clk_ctls: V2ClkCtl<'a>,
    port_ctls: V2PortCtl<'a>,
}

impl<'a> F8pre<'a> {
    const CLK_RATE_LABELS: &'a [&'a str] = &[
        "44100", "48000",
        "88200", "96000",
    ];
    const CLK_RATE_VALS: &'a [u8] = &[0x00, 0x01, 0x02, 0x03];

    const CLK_SRC_LABELS: &'a [&'a str] = &[
        "Internal",
        "ADAT-on-opt",
    ];
    const CLK_SRC_VALS: &'a [u8] = &[0x00, 0x01];

    const PHONE_ASSIGN_LABELS: &'a [&'a str] = &[
        "Phone-1/2",
        "Main-1/2",
    ];
    const PHONE_ASSIGN_VALS: &'a [u8] = &[0x01, 0x02];

    pub fn new() -> Self {
        F8pre{
            proto: Default::default(),
            clk_ctls: V2ClkCtl::new(Self::CLK_RATE_LABELS, Self::CLK_RATE_VALS,
                                    Self::CLK_SRC_LABELS, Self::CLK_SRC_VALS, false),
            port_ctls: V2PortCtl::new(Self::PHONE_ASSIGN_LABELS, Self::PHONE_ASSIGN_VALS,
                                      false, false, true, false),
        }
    }
}

impl<'a> CtlModel<SndMotu> for F8pre<'a> {
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
        if self.clk_ctls.read(unit, &self.proto, elem_id, elem_value)? {
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
        if self.clk_ctls.write(unit, &self.proto, elem_id, old, new)? {
            Ok(true)
        } else if self.port_ctls.write(unit, &self.proto, elem_id, old, new)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
