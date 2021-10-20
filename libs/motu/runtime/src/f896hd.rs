// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto
use glib::Error;

use hinawa::FwReq;
use hinawa::SndMotu;

use alsactl::{ElemId, ElemValue};

use core::card_cntr::*;

use motu_protocols::{register_dsp::*, version_2::*, *};

use super::{common_ctls::*, register_dsp_ctls::*, v2_ctls::*};

const TIMEOUT_MS: u32 = 100;

#[derive(Default)]
pub struct F896hd {
    req: FwReq,
    clk_ctls: ClkCtl,
    opt_iface_ctl: OptIfaceCtl,
    word_clk_ctl: WordClkCtl,
    aesebu_rate_convert_ctl: AesebuRateConvertCtl,
    level_meters_ctl: LevelMetersCtl,
    mixer_output_ctl: MixerOutputCtl,
    mixer_return_ctl: MixerReturnCtl,
    mixer_source_ctl: MixerSourceCtl,
    output_ctl: OutputCtl,
}

#[derive(Default)]
struct WordClkCtl(WordClkSpeedMode, Vec<ElemId>);

impl WordClkCtlOperation<F896hdProtocol> for WordClkCtl {
    fn state(&self) -> &WordClkSpeedMode {
        &self.0
    }

    fn state_mut(&mut self) -> &mut WordClkSpeedMode {
        &mut self.0
    }
}

#[derive(Default)]
struct AesebuRateConvertCtl;

impl AesebuRateConvertCtlOperation<F896hdProtocol> for AesebuRateConvertCtl {}

#[derive(Default)]
struct LevelMetersCtl(LevelMeterState, Vec<ElemId>);

impl LevelMetersCtlOperation<F896hdProtocol> for LevelMetersCtl {
    fn state(&self) -> &LevelMeterState {
        &self.0
    }

    fn state_mut(&mut self) -> &mut LevelMeterState {
        &mut self.0
    }
}

#[derive(Default)]
struct ClkCtl;

impl V2ClkCtlOperation<F896hdProtocol> for ClkCtl {}

#[derive(Default)]
struct OptIfaceCtl((usize, usize), Vec<ElemId>);

impl V2OptIfaceCtlOperation<F896hdProtocol> for OptIfaceCtl {
    fn state(&self) -> &(usize, usize) {
        &self.0
    }

    fn state_mut(&mut self) -> &mut (usize, usize) {
        &mut self.0
    }
}

#[derive(Default)]
struct MixerOutputCtl(RegisterDspMixerOutputState, Vec<ElemId>);

impl RegisterDspMixerOutputCtlOperation<F896hdProtocol> for MixerOutputCtl {
    fn state(&self) -> &RegisterDspMixerOutputState {
        &self.0
    }

    fn state_mut(&mut self) -> &mut RegisterDspMixerOutputState {
        &mut self.0
    }
}

#[derive(Default)]
struct MixerReturnCtl(RegisterDspMixerReturnState, Vec<ElemId>);

impl RegisterDspMixerReturnCtlOperation<F896hdProtocol> for MixerReturnCtl {
    fn state(&self) -> &RegisterDspMixerReturnState {
        &self.0
    }

    fn state_mut(&mut self) -> &mut RegisterDspMixerReturnState {
        &mut self.0
    }
}

#[derive(Default)]
struct MixerSourceCtl(RegisterDspMixerMonauralSourceState, Vec<ElemId>);

impl RegisterDspMixerMonauralSourceCtlOperation<F896hdProtocol> for MixerSourceCtl {
    fn state(&self) -> &RegisterDspMixerMonauralSourceState {
        &self.0
    }

    fn state_mut(&mut self) -> &mut RegisterDspMixerMonauralSourceState {
        &mut self.0
    }
}

#[derive(Default)]
struct OutputCtl(RegisterDspOutputState, Vec<ElemId>);

impl RegisterDspOutputCtlOperation<F896hdProtocol> for OutputCtl {
    fn state(&self) -> &RegisterDspOutputState {
        &self.0
    }

    fn state_mut(&mut self) -> &mut RegisterDspOutputState {
        &mut self.0
    }
}

impl CtlModel<SndMotu> for F896hd {
    fn load(&mut self, unit: &mut SndMotu, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.clk_ctls.load(card_cntr)?;
        self.opt_iface_ctl.load(card_cntr, unit, &mut self.req, TIMEOUT_MS)
            .map(|mut elem_id_list| self.opt_iface_ctl.1.append(&mut elem_id_list))?;
        self.word_clk_ctl.load(card_cntr, unit, &mut self.req, TIMEOUT_MS)
            .map(|mut elem_id_list| self.word_clk_ctl.1.append(&mut elem_id_list))?;
        self.aesebu_rate_convert_ctl.load(card_cntr)?;
        self.level_meters_ctl.load(card_cntr, unit, &mut self.req, TIMEOUT_MS)
            .map(|mut elem_id_list| self.level_meters_ctl.1.append(&mut elem_id_list))?;
        self.mixer_output_ctl.load(card_cntr, unit, &mut self.req, TIMEOUT_MS)
            .map(|elem_id_list| self.mixer_output_ctl.1 = elem_id_list)?;
        self.mixer_return_ctl.load(card_cntr, unit, &mut self.req, TIMEOUT_MS)
            .map(|elem_id_list| self.mixer_return_ctl.1 = elem_id_list)?;
        self.mixer_source_ctl.load(card_cntr, unit, &mut self.req, TIMEOUT_MS)
            .map(|elem_id_list| self.mixer_source_ctl.1 = elem_id_list)?;
        self.output_ctl.load(card_cntr, unit, &mut self.req, TIMEOUT_MS)
            .map(|elem_id_list| self.output_ctl.1 = elem_id_list)?;
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
        } else if self.opt_iface_ctl.read(elem_id, elem_value)? {
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
        } else if self
            .level_meters_ctl
            .read(unit, &mut self.req, elem_id, elem_value, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self.mixer_output_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_return_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_source_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.output_ctl.read(elem_id, elem_value)? {
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
            .opt_iface_ctl
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
        } else if self.mixer_output_ctl.write(unit, &mut self.req, elem_id, new, TIMEOUT_MS)? {
            Ok(true)
        } else if self.mixer_return_ctl.write(unit, &mut self.req, elem_id, new, TIMEOUT_MS)? {
            Ok(true)
        } else if self.mixer_source_ctl.write(unit, &mut self.req, elem_id, new, TIMEOUT_MS)? {
            Ok(true)
        } else if self.output_ctl.write(unit, &mut self.req, elem_id, new, TIMEOUT_MS)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl NotifyModel<SndMotu, u32> for F896hd {
    fn get_notified_elem_list(&mut self, _: &mut Vec<ElemId>) {}

    fn parse_notification(&mut self, _: &mut SndMotu, _: &u32) -> Result<(), Error> {
        Ok(())
    }

    fn read_notified_elem(
        &mut self,
        _: &SndMotu,
        _: &ElemId,
        _: &mut ElemValue
    ) -> Result<bool, Error> {
        Ok(false)
    }
}
