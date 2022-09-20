// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

use {
    super::*,
    protocols::{presonus::fp10::*, *},
};

#[derive(Default)]
pub struct Fp10Model {
    avc: BebobAvc,
    clk_ctl: ClkCtl,
    phys_out_ctl: PhysOutputCtl,
}

const FCP_TIMEOUT_MS: u32 = 100;

#[derive(Default, Debug)]
struct ClkCtl(Vec<ElemId>, MediaClockParameters, SamplingClockParameters);

impl MediaClkFreqCtlOperation<Fp10ClkProtocol> for ClkCtl {
    fn state(&self) -> &MediaClockParameters {
        &self.1
    }

    fn state_mut(&mut self) -> &mut MediaClockParameters {
        &mut self.1
    }
}

impl SamplingClkSrcCtlOperation<Fp10ClkProtocol> for ClkCtl {
    const SRC_LABELS: &'static [&'static str] = &["Internal", "S/PDIF"];

    fn state(&self) -> &SamplingClockParameters {
        &self.2
    }

    fn state_mut(&mut self) -> &mut SamplingClockParameters {
        &mut self.2
    }
}

#[derive(Debug)]
struct PhysOutputCtl(AvcLevelParameters);

impl Default for PhysOutputCtl {
    fn default() -> Self {
        Self(Fp10PhysOutputProtocol::create_level_parameters())
    }
}

impl AvcLevelCtlOperation<Fp10PhysOutputProtocol> for PhysOutputCtl {
    const LEVEL_NAME: &'static str = OUT_VOL_NAME;

    const PORT_LABELS: &'static [&'static str] = &[
        "analog-output-1",
        "analog-output-2",
        "analog-output-3",
        "analog-output-4",
        "analog-output-5",
        "analog-output-6",
        "analog-output-7",
        "analog-output-8",
    ];

    fn state(&self) -> &AvcLevelParameters {
        &self.0
    }

    fn state_mut(&mut self) -> &mut AvcLevelParameters {
        &mut self.0
    }
}

impl AvcLrBalanceCtlOperation<Fp10PhysOutputProtocol> for PhysOutputCtl {
    const BALANCE_NAME: &'static str = "output-balance";
}

impl AvcMuteCtlOperation<Fp10PhysOutputProtocol> for PhysOutputCtl {
    const MUTE_NAME: &'static str = "output-mute";
}

impl CtlModel<(SndUnit, FwNode)> for Fp10Model {
    fn load(
        &mut self,
        unit: &mut (SndUnit, FwNode),
        card_cntr: &mut CardCntr,
    ) -> Result<(), Error> {
        self.avc.bind(&unit.1)?;

        self.clk_ctl
            .load_freq(card_cntr)
            .map(|mut elem_id_list| self.clk_ctl.0.append(&mut elem_id_list))?;

        self.clk_ctl
            .load_src(card_cntr)
            .map(|mut elem_id_list| self.clk_ctl.0.append(&mut elem_id_list))?;

        self.phys_out_ctl.load_level(card_cntr)?;
        self.phys_out_ctl.load_balance(card_cntr)?;
        self.phys_out_ctl.load_mute(card_cntr)?;

        self.clk_ctl.cache_freq(&self.avc, FCP_TIMEOUT_MS)?;
        self.clk_ctl.cache_src(&self.avc, FCP_TIMEOUT_MS)?;
        self.phys_out_ctl.cache_levels(&self.avc, FCP_TIMEOUT_MS)?;

        Ok(())
    }

    fn read(
        &mut self,
        _: &mut (SndUnit, FwNode),
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.clk_ctl.read_freq(elem_id, elem_value)? {
            Ok(true)
        } else if self.clk_ctl.read_src(elem_id, elem_value)? {
            Ok(true)
        } else if self.phys_out_ctl.read_levels(elem_id, elem_value)? {
            Ok(true)
        } else if self
            .phys_out_ctl
            .read_balance(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .phys_out_ctl
            .read_mute(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(
        &mut self,
        unit: &mut (SndUnit, FwNode),
        elem_id: &ElemId,
        old: &ElemValue,
        new: &ElemValue,
    ) -> Result<bool, Error> {
        if self.clk_ctl.write_freq(
            &mut unit.0,
            &self.avc,
            elem_id,
            old,
            new,
            FCP_TIMEOUT_MS * 3,
        )? {
            Ok(true)
        } else if self.clk_ctl.write_src(
            &mut unit.0,
            &self.avc,
            elem_id,
            old,
            new,
            FCP_TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self
            .phys_out_ctl
            .write_level(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .phys_out_ctl
            .write_balance(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .phys_out_ctl
            .write_mute(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl NotifyModel<(SndUnit, FwNode), bool> for Fp10Model {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.clk_ctl.0);
    }

    fn parse_notification(&mut self, _: &mut (SndUnit, FwNode), _: &bool) -> Result<(), Error> {
        Ok(())
    }

    fn read_notified_elem(
        &mut self,
        _: &(SndUnit, FwNode),
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        self.clk_ctl.read_freq(elem_id, elem_value)
    }
}
