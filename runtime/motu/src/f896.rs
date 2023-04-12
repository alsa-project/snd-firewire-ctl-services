// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

use super::{common_ctls::*, v1_ctls::*, v1_runtime::*};

const TIMEOUT_MS: u32 = 100;

#[derive(Default)]
pub struct F896 {
    req: FwReq,
    clk_ctls: V1ClkCtl<F896Protocol>,
    monitor_input_ctl: V1MonitorInputCtl<F896Protocol>,
    word_clk_ctl: WordClockCtl<F896Protocol>,
    aesebu_rate_convert_ctl: AesebuRateConvertCtl<F896Protocol>,
    level_meters_ctl: LevelMetersCtl<F896Protocol>,
}

impl CtlModel<(SndMotu, FwNode)> for F896 {
    fn cache(&mut self, (_, node): &mut (SndMotu, FwNode)) -> Result<(), Error> {
        self.clk_ctls.cache(&mut self.req, node, TIMEOUT_MS)?;
        self.word_clk_ctl.cache(&mut self.req, node, TIMEOUT_MS)?;
        self.aesebu_rate_convert_ctl
            .cache(&mut self.req, node, TIMEOUT_MS)?;
        self.level_meters_ctl
            .cache(&mut self.req, node, TIMEOUT_MS)?;
        Ok(())
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.clk_ctls.load(card_cntr)?;
        self.monitor_input_ctl.load(card_cntr)?;
        self.word_clk_ctl.load(card_cntr)?;
        self.aesebu_rate_convert_ctl.load(card_cntr)?;
        self.level_meters_ctl.load(card_cntr)?;
        Ok(())
    }

    fn read(
        &mut self,
        _: &mut (SndMotu, FwNode),
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.clk_ctls.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.monitor_input_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.word_clk_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.aesebu_rate_convert_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.level_meters_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(
        &mut self,
        (unit, node): &mut (SndMotu, FwNode),
        elem_id: &ElemId,
        _: &ElemValue,
        elem_value: &ElemValue,
    ) -> Result<bool, Error> {
        if self
            .clk_ctls
            .write(unit, &mut self.req, node, elem_id, elem_value, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self.monitor_input_ctl.write(
            &mut self.req,
            node,
            elem_id,
            elem_value,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self
            .word_clk_ctl
            .write(&mut self.req, node, elem_id, elem_value, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self.aesebu_rate_convert_ctl.write(
            &mut self.req,
            node,
            elem_id,
            elem_value,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self.level_meters_ctl.write(
            &mut self.req,
            node,
            elem_id,
            elem_value,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl NotifyModel<(SndMotu, FwNode), u32> for F896 {
    fn get_notified_elem_list(&mut self, _: &mut Vec<ElemId>) {}

    fn parse_notification(&mut self, _: &mut (SndMotu, FwNode), _: &u32) -> Result<(), Error> {
        // TODO: what kind of event is preferable for NOTIFY_FOOTSWITCH_MASK?
        Ok(())
    }
}
