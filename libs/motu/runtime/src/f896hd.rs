// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto
use glib::Error;

use hinawa::FwReq;
use hinawa::SndMotu;

use alsactl::{ElemId, ElemValue};

use core::card_cntr::{CardCntr, CtlModel};

use motu_protocols::version_2::*;

use super::common_ctls::*;
use super::v2_ctls::*;

const TIMEOUT_MS: u32 = 100;

#[derive(Default)]
pub struct F896hd {
    req: FwReq,
    proto: F896hdProtocol,
    clk_ctls: ClkCtl,
    opt_iface_ctl: V2OptIfaceCtl,
    word_clk_ctl: WordClkCtl,
    aesebu_rate_convert_ctl: AesebuRateConvertCtl,
    level_meters_ctl: LevelMetersCtl,
}

#[derive(Default)]
struct WordClkCtl;

impl WordClkCtlOperation<F896hdProtocol> for WordClkCtl {}

#[derive(Default)]
struct AesebuRateConvertCtl;

impl AesebuRateConvertCtlOperation<F896hdProtocol> for AesebuRateConvertCtl {}

#[derive(Default)]
struct LevelMetersCtl;

impl LevelMetersCtlOperation<F896hdProtocol> for LevelMetersCtl {}

#[derive(Default)]
struct ClkCtl;

impl V2ClkCtlOperation<F896hdProtocol> for ClkCtl {}

impl CtlModel<SndMotu> for F896hd {
    fn load(&mut self, _: &mut SndMotu, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.clk_ctls.load(card_cntr)?;
        self.opt_iface_ctl.load(&self.proto, card_cntr)?;
        self.word_clk_ctl.load(card_cntr)?;
        self.aesebu_rate_convert_ctl.load(card_cntr)?;
        self.level_meters_ctl.load(card_cntr)?;
        Ok(())
    }

    fn read(
        &mut self,
        unit: &mut SndMotu,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self
            .clk_ctls
            .read(unit, &mut self.req, elem_id, elem_value, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .opt_iface_ctl
            .read(unit, &mut self.req, &self.proto, elem_id, elem_value, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .word_clk_ctl
            .read(unit, &mut self.req, elem_id, elem_value, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self.aesebu_rate_convert_ctl.read(
            unit,
            &mut self.req,
            elem_id,
            elem_value,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self
            .level_meters_ctl
            .read(unit, &mut self.req, elem_id, elem_value, TIMEOUT_MS)?
        {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(
        &mut self,
        unit: &mut SndMotu,
        elem_id: &ElemId,
        old: &ElemValue,
        new: &ElemValue,
    ) -> Result<bool, Error> {
        if self
            .clk_ctls
            .write(unit, &mut self.req, elem_id, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .opt_iface_ctl
            .write(unit, &mut self.req, &self.proto, elem_id, old, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .word_clk_ctl
            .write(unit, &mut self.req, elem_id, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self.aesebu_rate_convert_ctl.write(
            unit,
            &mut self.req,
            elem_id,
            new,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self
            .level_meters_ctl
            .write(unit, &mut self.req, elem_id, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
