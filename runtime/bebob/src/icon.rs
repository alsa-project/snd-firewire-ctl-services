// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

use {
    super::{common_ctls::*, *},
    protocols::{icon::*, *},
};

#[derive(Default, Debug)]
pub struct FirexonModel {
    avc: BebobAvc,
    clk_ctl: ClkCtl,
    phys_out_ctl: PhysOutputCtl,
    mon_src_ctl: MonitorSrcCtl,
    mixer_src_ctl: MixerSrcCtl,
}

const FCP_TIMEOUT_MS: u32 = 100;

#[derive(Default, Debug)]
struct ClkCtl(Vec<ElemId>, MediaClockParameters, SamplingClockParameters);

impl MediaClkFreqCtlOperation<FirexonClkProtocol> for ClkCtl {
    fn state(&self) -> &MediaClockParameters {
        &self.1
    }

    fn state_mut(&mut self) -> &mut MediaClockParameters {
        &mut self.1
    }
}

impl SamplingClkSrcCtlOperation<FirexonClkProtocol> for ClkCtl {
    const SRC_LABELS: &'static [&'static str] = &["Internal", "S/PDIF"];

    fn state(&self) -> &SamplingClockParameters {
        &self.2
    }

    fn state_mut(&mut self) -> &mut SamplingClockParameters {
        &mut self.2
    }
}

#[derive(Debug)]
struct PhysOutputCtl(
    AvcLevelParameters,
    AvcLrBalanceParameters,
    AvcMuteParameters,
    AvcSelectorParameters,
);

impl Default for PhysOutputCtl {
    fn default() -> Self {
        Self(
            FirexonPhysOutputProtocol::create_level_parameters(),
            FirexonPhysOutputProtocol::create_lr_balance_parameters(),
            FirexonPhysOutputProtocol::create_mute_parameters(),
            FirexonPhysOutputProtocol::create_selector_parameters(),
        )
    }
}

impl AvcLevelCtlOperation<FirexonPhysOutputProtocol> for PhysOutputCtl {
    const LEVEL_NAME: &'static str = "analog-output-volume";
    const PORT_LABELS: &'static [&'static str] = &[
        "analog-output-1",
        "analog-output-2",
        "analog-output-3",
        "analog-output-4",
    ];

    fn state(&self) -> &AvcLevelParameters {
        &self.0
    }

    fn state_mut(&mut self) -> &mut AvcLevelParameters {
        &mut self.0
    }
}

impl AvcLrBalanceCtlOperation<FirexonPhysOutputProtocol> for PhysOutputCtl {
    const BALANCE_NAME: &'static str = "analog-output-balance";

    fn state(&self) -> &AvcLrBalanceParameters {
        &self.1
    }

    fn state_mut(&mut self) -> &mut AvcLrBalanceParameters {
        &mut self.1
    }
}

impl AvcMuteCtlOperation<FirexonPhysOutputProtocol> for PhysOutputCtl {
    const MUTE_NAME: &'static str = "analog-output-mute";

    fn state(&self) -> &AvcMuteParameters {
        &self.2
    }

    fn state_mut(&mut self) -> &mut AvcMuteParameters {
        &mut self.2
    }
}

impl AvcSelectorCtlOperation<FirexonPhysOutputProtocol> for PhysOutputCtl {
    const SELECTOR_NAME: &'static str = "analog-output-3/4-source";
    const SELECTOR_LABELS: &'static [&'static str] = &["analog-output-3/4"];
    const ITEM_LABELS: &'static [&'static str] =
        &["mixer-output-1/2", "stream-input-3/4", "stream-input-5/6"];

    fn state(&self) -> &AvcSelectorParameters {
        &self.3
    }

    fn state_mut(&mut self) -> &mut AvcSelectorParameters {
        &mut self.3
    }
}

#[derive(Debug)]
struct MonitorSrcCtl(
    AvcLevelParameters,
    AvcLrBalanceParameters,
    AvcMuteParameters,
);

impl Default for MonitorSrcCtl {
    fn default() -> Self {
        Self(
            FirexonMonitorSourceProtocol::create_level_parameters(),
            FirexonMonitorSourceProtocol::create_lr_balance_parameters(),
            FirexonMonitorSourceProtocol::create_mute_parameters(),
        )
    }
}

impl AvcLevelCtlOperation<FirexonMonitorSourceProtocol> for MonitorSrcCtl {
    const LEVEL_NAME: &'static str = "monitor-source-gain";
    const PORT_LABELS: &'static [&'static str] = &[
        "analog-input-1",
        "analog-input-2",
        "analog-input-3",
        "analog-input-4",
        "digital-input-1",
        "digital-input-2",
    ];

    fn state(&self) -> &AvcLevelParameters {
        &self.0
    }

    fn state_mut(&mut self) -> &mut AvcLevelParameters {
        &mut self.0
    }
}

impl AvcLrBalanceCtlOperation<FirexonMonitorSourceProtocol> for MonitorSrcCtl {
    const BALANCE_NAME: &'static str = "monitor-source-balance";

    fn state(&self) -> &AvcLrBalanceParameters {
        &self.1
    }

    fn state_mut(&mut self) -> &mut AvcLrBalanceParameters {
        &mut self.1
    }
}

impl AvcMuteCtlOperation<FirexonMonitorSourceProtocol> for MonitorSrcCtl {
    const MUTE_NAME: &'static str = "monitor-source-mute";

    fn state(&self) -> &AvcMuteParameters {
        &self.2
    }

    fn state_mut(&mut self) -> &mut AvcMuteParameters {
        &mut self.2
    }
}

#[derive(Debug)]
struct MixerSrcCtl(AvcLevelParameters);

impl Default for MixerSrcCtl {
    fn default() -> Self {
        Self(FirexonMixerSourceProtocol::create_level_parameters())
    }
}

impl AvcLevelCtlOperation<FirexonMixerSourceProtocol> for MixerSrcCtl {
    const LEVEL_NAME: &'static str = "mixer-source-gain";
    const PORT_LABELS: &'static [&'static str] = &["stream-input-1/2", "monitor-output-1/2"];

    fn state(&self) -> &AvcLevelParameters {
        &self.0
    }

    fn state_mut(&mut self) -> &mut AvcLevelParameters {
        &mut self.0
    }
}

impl CtlModel<(SndUnit, FwNode)> for FirexonModel {
    fn cache(&mut self, unit: &mut (SndUnit, FwNode)) -> Result<(), Error> {
        self.avc.bind(&unit.1)?;

        self.clk_ctl.cache_freq(&self.avc, FCP_TIMEOUT_MS)?;
        self.clk_ctl.cache_src(&self.avc, FCP_TIMEOUT_MS)?;
        self.phys_out_ctl.cache_levels(&self.avc, FCP_TIMEOUT_MS)?;
        self.mon_src_ctl.cache_levels(&self.avc, FCP_TIMEOUT_MS)?;
        self.mixer_src_ctl.cache_levels(&self.avc, FCP_TIMEOUT_MS)?;
        self.phys_out_ctl
            .cache_balances(&self.avc, FCP_TIMEOUT_MS)?;
        self.mon_src_ctl.cache_balances(&self.avc, FCP_TIMEOUT_MS)?;
        self.phys_out_ctl.cache_mutes(&self.avc, FCP_TIMEOUT_MS)?;
        self.mon_src_ctl.cache_mutes(&self.avc, FCP_TIMEOUT_MS)?;
        self.phys_out_ctl
            .cache_selectors(&self.avc, FCP_TIMEOUT_MS)?;

        Ok(())
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.clk_ctl
            .load_freq(card_cntr)
            .map(|mut elem_id_list| self.clk_ctl.0.append(&mut elem_id_list))?;

        self.clk_ctl
            .load_src(card_cntr)
            .map(|mut elem_id_list| self.clk_ctl.0.append(&mut elem_id_list))?;

        self.phys_out_ctl.load_level(card_cntr)?;
        self.phys_out_ctl.load_balance(card_cntr)?;
        self.phys_out_ctl.load_mute(card_cntr)?;
        self.phys_out_ctl.load_selector(card_cntr)?;
        self.mon_src_ctl.load_level(card_cntr)?;
        self.mon_src_ctl.load_balance(card_cntr)?;
        self.mon_src_ctl.load_mute(card_cntr)?;
        self.mixer_src_ctl.load_level(card_cntr)?;

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
        } else if self.phys_out_ctl.read_balances(elem_id, elem_value)? {
            Ok(true)
        } else if self.phys_out_ctl.read_mutes(elem_id, elem_value)? {
            Ok(true)
        } else if self.phys_out_ctl.read_selectors(elem_id, elem_value)? {
            Ok(true)
        } else if self.mon_src_ctl.read_levels(elem_id, elem_value)? {
            Ok(true)
        } else if self.mon_src_ctl.read_balances(elem_id, elem_value)? {
            Ok(true)
        } else if self.mon_src_ctl.read_mutes(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_src_ctl.read_levels(elem_id, elem_value)? {
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
        if self
            .clk_ctl
            .write_freq(&mut unit.0, &self.avc, elem_id, new, FCP_TIMEOUT_MS * 3)?
        {
            Ok(true)
        } else if self
            .clk_ctl
            .write_src(&mut unit.0, &self.avc, elem_id, new, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .phys_out_ctl
            .write_level(&self.avc, elem_id, new, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .phys_out_ctl
            .write_balance(&self.avc, elem_id, new, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .phys_out_ctl
            .write_mute(&self.avc, elem_id, new, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .phys_out_ctl
            .write_selector(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .mon_src_ctl
            .write_level(&self.avc, elem_id, new, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .mon_src_ctl
            .write_balance(&self.avc, elem_id, new, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .mon_src_ctl
            .write_mute(&self.avc, elem_id, new, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .mixer_src_ctl
            .write_level(&self.avc, elem_id, new, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl NotifyModel<(SndUnit, FwNode), bool> for FirexonModel {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.clk_ctl.0);
    }

    fn parse_notification(
        &mut self,
        _: &mut (SndUnit, FwNode),
        &locked: &bool,
    ) -> Result<(), Error> {
        if locked {
            self.clk_ctl.cache_freq(&self.avc, FCP_TIMEOUT_MS)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use alsactl::CardError;

    #[test]
    fn test_clk_ctl_definition() {
        let mut card_cntr = CardCntr::default();
        let mut ctl = ClkCtl::default();

        let error = ctl.load_freq(&mut card_cntr).unwrap_err();
        assert_eq!(error.kind::<CardError>(), Some(CardError::Failed));

        let error = ctl.load_src(&mut card_cntr).unwrap_err();
        assert_eq!(error.kind::<CardError>(), Some(CardError::Failed));
    }

    #[test]
    fn test_level_ctl_definition() {
        let mut card_cntr = CardCntr::default();

        let mut ctl = PhysOutputCtl::default();
        let error = ctl.load_level(&mut card_cntr).unwrap_err();
        assert_eq!(error.kind::<CardError>(), Some(CardError::Failed));

        let mut ctl = MonitorSrcCtl::default();
        let error = ctl.load_level(&mut card_cntr).unwrap_err();
        assert_eq!(error.kind::<CardError>(), Some(CardError::Failed));

        let mut ctl = MixerSrcCtl::default();
        let error = ctl.load_level(&mut card_cntr).unwrap_err();
        assert_eq!(error.kind::<CardError>(), Some(CardError::Failed));
    }

    #[test]
    fn test_selector_ctl_definition() {
        let mut card_cntr = CardCntr::default();

        let mut ctl = PhysOutputCtl::default();
        let error = ctl.load_selector(&mut card_cntr).unwrap_err();
        assert_eq!(error.kind::<CardError>(), Some(CardError::Failed));
    }
}
