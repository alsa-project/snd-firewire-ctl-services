// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

use super::v1_runtime::*;

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
struct WordClkCtl(WordClkSpeedMode);

impl WordClkCtlOperation<F896Protocol> for WordClkCtl {
    fn state(&self) -> &WordClkSpeedMode {
        &self.0
    }

    fn state_mut(&mut self) -> &mut WordClkSpeedMode {
        &mut self.0
    }
}

#[derive(Default)]
struct AesebuRateConvertCtl;

impl AesebuRateConvertCtlOperation<F896Protocol> for AesebuRateConvertCtl {}

#[derive(Default)]
struct LevelMetersCtl(LevelMeterState);

impl LevelMetersCtlOperation<F896Protocol> for LevelMetersCtl {
    fn state(&self) -> &LevelMeterState {
        &self.0
    }

    fn state_mut(&mut self) -> &mut LevelMeterState {
        &mut self.0
    }
}

#[derive(Default)]
struct ClkCtl;

impl V1ClkCtlOperation<F896Protocol> for ClkCtl {}

#[derive(Default)]
struct MonitorInputCtl;

impl V1MonitorInputCtlOperation<F896Protocol> for MonitorInputCtl {}

impl CtlModel<SndMotu> for F896 {
    fn load(&mut self, unit: &mut SndMotu, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.clk_ctls.load(card_cntr)?;
        self.monitor_input_ctl.load(card_cntr)?;
        let _ = self
            .word_clk_ctl
            .load(card_cntr, unit, &mut self.req, TIMEOUT_MS)?;
        self.aesebu_rate_convert_ctl.load(card_cntr)?;
        let _ = self
            .level_meters_ctl
            .load(card_cntr, unit, &mut self.req, TIMEOUT_MS)?;
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
        } else if self.monitor_input_ctl.read(
            unit,
            &mut self.req,
            elem_id,
            elem_value,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self.word_clk_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.aesebu_rate_convert_ctl.read(
            unit,
            &mut self.req,
            elem_id,
            elem_value,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self.level_meters_ctl.read(
            unit,
            &mut self.req,
            elem_id,
            elem_value,
            TIMEOUT_MS,
        )? {
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

impl NotifyModel<SndMotu, u32> for F896 {
    fn get_notified_elem_list(&mut self, _: &mut Vec<ElemId>) {}

    fn parse_notification(&mut self, _: &mut SndMotu, _: &u32) -> Result<(), Error> {
        // TODO: what kind of event is preferable for NOTIFY_FOOTSWITCH_MASK?
        Ok(())
    }

    fn read_notified_elem(
        &mut self,
        _: &SndMotu,
        _: &ElemId,
        _: &mut ElemValue,
    ) -> Result<bool, Error> {
        Ok(false)
    }
}
