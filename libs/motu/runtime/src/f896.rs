// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto
use glib::Error;

use hinawa::FwReq;
use hinawa::SndMotu;

use alsactl::{ElemId, ElemValue};

use core::card_cntr::{CardCntr, CtlModel};

use motu_protocols::version_1::*;

use super::common_ctls::*;
use super::v1_ctls::*;

const TIMEOUT_MS: u32 = 100;

#[derive(Default)]
pub struct F896 {
    req: FwReq,
    clk_ctls: ClkCtl,
    monitor_input_ctl: MonitorInputCtl,
    word_clk_ctl: WordClkCtl,
    aesebu_rate_convert_ctl: AesebuRateConvertCtl,
    level_meters_ctl: LevelMetersCtl,
}

#[derive(Default)]
struct WordClkCtl;

impl WordClkCtlOperation<F896Protocol> for WordClkCtl {}

#[derive(Default)]
struct AesebuRateConvertCtl;

impl AesebuRateConvertCtlOperation<F896Protocol> for AesebuRateConvertCtl {}

#[derive(Default)]
struct LevelMetersCtl;

impl LevelMetersCtlOperation<F896Protocol> for LevelMetersCtl {}

#[derive(Default)]
struct ClkCtl;

impl V1ClkCtlOperation<F896Protocol> for ClkCtl {}

#[derive(Default)]
struct MonitorInputCtl;

impl V1MonitorInputCtlOperation<F896Protocol> for MonitorInputCtl {}

impl CtlModel<SndMotu> for F896 {
    fn load(&mut self, _: &mut SndMotu, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.clk_ctls.load(card_cntr)?;
        self.monitor_input_ctl.load(card_cntr)?;
        let _ = self.word_clk_ctl.load(card_cntr)?;
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
            .monitor_input_ctl
            .read(unit, &mut self.req, elem_id, elem_value, TIMEOUT_MS)?
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
            .read(unit, &mut self.req,  elem_id, elem_value, TIMEOUT_MS)?
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
        _: &ElemValue,
        new: &ElemValue,
    ) -> Result<bool, Error> {
        if self
            .clk_ctls
            .write(unit, &mut self.req, elem_id, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .monitor_input_ctl
            .write(unit, &mut self.req, elem_id, new, TIMEOUT_MS)?
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
