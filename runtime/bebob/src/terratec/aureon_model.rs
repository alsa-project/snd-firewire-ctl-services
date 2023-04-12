// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

use {
    super::{common_ctls::*, *},
    protocols::{terratec::aureon::*, *},
};

#[derive(Default, Debug)]
pub struct AureonModel {
    avc: BebobAvc,
    clk_ctl: ClkCtl,
    phys_in_ctl: PhysInputCtl,
    mon_src_ctl: MonitorSourceCtl,
    mon_out_ctl: MonitorOutputCtl,
    mixer_out_ctl: MixerOutputCtl,
    spdif_out_ctl: SpdifOutputCtl,
}

const FCP_TIMEOUT_MS: u32 = 100;

#[derive(Default, Debug)]
struct ClkCtl(Vec<ElemId>, MediaClockParameters);

impl MediaClkFreqCtlOperation<AureonClkProtocol> for ClkCtl {
    fn state(&self) -> &MediaClockParameters {
        &self.1
    }

    fn state_mut(&mut self) -> &mut MediaClockParameters {
        &mut self.1
    }
}

#[derive(Debug)]
struct PhysInputCtl(AvcLevelParameters);

impl Default for PhysInputCtl {
    fn default() -> Self {
        Self(AureonPhysInputProtocol::create_level_parameters())
    }
}

impl AvcLevelCtlOperation<AureonPhysInputProtocol> for PhysInputCtl {
    const LEVEL_NAME: &'static str = "analog-input-gain";
    const PORT_LABELS: &'static [&'static str] = &["analog-input-1/2", "analog-input-3/4"];

    fn state(&self) -> &AvcLevelParameters {
        &self.0
    }

    fn state_mut(&mut self) -> &mut AvcLevelParameters {
        &mut self.0
    }
}

#[derive(Debug)]
struct MonitorSourceCtl(AvcSelectorParameters);

impl Default for MonitorSourceCtl {
    fn default() -> Self {
        Self(AureonMonitorSourceProtocol::create_selector_parameters())
    }
}

impl AvcSelectorCtlOperation<AureonMonitorSourceProtocol> for MonitorSourceCtl {
    const SELECTOR_NAME: &'static str = "monitor-source";
    const SELECTOR_LABELS: &'static [&'static str] = &["monitor-source-1/2"];
    const ITEM_LABELS: &'static [&'static str] = &[
        "analog-input-1/2",
        "analog-input-3/4",
        "analog-input-5/6",
        "digital-input-1/2",
    ];

    fn state(&self) -> &AvcSelectorParameters {
        &self.0
    }

    fn state_mut(&mut self) -> &mut AvcSelectorParameters {
        &mut self.0
    }
}

#[derive(Debug)]
struct MonitorOutputCtl(AvcLevelParameters, AvcMuteParameters);

impl Default for MonitorOutputCtl {
    fn default() -> Self {
        Self(
            AureonMonitorOutputProtocol::create_level_parameters(),
            AureonMonitorOutputProtocol::create_mute_parameters(),
        )
    }
}

impl AvcLevelCtlOperation<AureonMonitorOutputProtocol> for MonitorOutputCtl {
    const LEVEL_NAME: &'static str = "monitor-output-volume";
    const PORT_LABELS: &'static [&'static str] = &["monitor-output-1/2"];

    fn state(&self) -> &AvcLevelParameters {
        &self.0
    }

    fn state_mut(&mut self) -> &mut AvcLevelParameters {
        &mut self.0
    }
}

impl AvcMuteCtlOperation<AureonMonitorOutputProtocol> for MonitorOutputCtl {
    const MUTE_NAME: &'static str = "monitor-output-mute";

    fn state(&self) -> &AvcMuteParameters {
        &self.1
    }

    fn state_mut(&mut self) -> &mut AvcMuteParameters {
        &mut self.1
    }
}

#[derive(Debug)]
struct MixerOutputCtl(AvcLevelParameters, AvcMuteParameters);

impl Default for MixerOutputCtl {
    fn default() -> Self {
        Self(
            AureonMixerOutputProtocol::create_level_parameters(),
            AureonMixerOutputProtocol::create_mute_parameters(),
        )
    }
}

#[derive(Debug)]
struct SpdifOutputCtl(AvcSelectorParameters);

impl Default for SpdifOutputCtl {
    fn default() -> Self {
        Self(AureonSpdifOutputProtocol::create_selector_parameters())
    }
}

impl AvcSelectorCtlOperation<AureonSpdifOutputProtocol> for SpdifOutputCtl {
    const SELECTOR_NAME: &'static str = "spdif-output-source";
    const SELECTOR_LABELS: &'static [&'static str] = &["spdif-output-1/2"];
    const ITEM_LABELS: &'static [&'static str] = &["mixer-output-1/2", "stream-input-9/10"];

    fn state(&self) -> &AvcSelectorParameters {
        &self.0
    }

    fn state_mut(&mut self) -> &mut AvcSelectorParameters {
        &mut self.0
    }
}

impl AvcLevelCtlOperation<AureonMixerOutputProtocol> for MixerOutputCtl {
    const LEVEL_NAME: &'static str = "mixer-output-volume";
    const PORT_LABELS: &'static [&'static str] = &[
        "mixer-output-1",
        "mixer-output-2",
        "mixer-output-3",
        "mixer-output-4",
        "mixer-output-5",
        "mixer-output-6",
        "mixer-output-7",
        "mixer-output-8",
    ];

    fn state(&self) -> &AvcLevelParameters {
        &self.0
    }

    fn state_mut(&mut self) -> &mut AvcLevelParameters {
        &mut self.0
    }
}

impl AvcMuteCtlOperation<AureonMixerOutputProtocol> for MixerOutputCtl {
    const MUTE_NAME: &'static str = "mixer-output-mute";

    fn state(&self) -> &AvcMuteParameters {
        &self.1
    }

    fn state_mut(&mut self) -> &mut AvcMuteParameters {
        &mut self.1
    }
}

impl CtlModel<(SndUnit, FwNode)> for AureonModel {
    fn cache(&mut self, unit: &mut (SndUnit, FwNode)) -> Result<(), Error> {
        self.avc.bind(&unit.1)?;

        self.clk_ctl.cache_freq(&self.avc, FCP_TIMEOUT_MS)?;
        self.phys_in_ctl.cache_levels(&self.avc, FCP_TIMEOUT_MS)?;
        self.mon_out_ctl.cache_levels(&self.avc, FCP_TIMEOUT_MS)?;
        self.mixer_out_ctl.cache_levels(&self.avc, FCP_TIMEOUT_MS)?;
        self.mon_out_ctl.cache_mutes(&self.avc, FCP_TIMEOUT_MS)?;
        self.mixer_out_ctl.cache_mutes(&self.avc, FCP_TIMEOUT_MS)?;
        self.mon_src_ctl
            .cache_selectors(&self.avc, FCP_TIMEOUT_MS)?;
        self.spdif_out_ctl
            .cache_selectors(&self.avc, FCP_TIMEOUT_MS)?;

        Ok(())
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.clk_ctl
            .load_freq(card_cntr)
            .map(|mut elem_id_list| self.clk_ctl.0.append(&mut elem_id_list))?;

        self.phys_in_ctl.load_level(card_cntr)?;
        self.mon_src_ctl.load_selector(card_cntr)?;
        self.mon_out_ctl.load_level(card_cntr)?;
        self.mon_out_ctl.load_mute(card_cntr)?;
        self.mixer_out_ctl.load_level(card_cntr)?;
        self.mixer_out_ctl.load_mute(card_cntr)?;
        self.spdif_out_ctl.load_selector(card_cntr)?;

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
        } else if self.phys_in_ctl.read_levels(elem_id, elem_value)? {
            Ok(true)
        } else if self.mon_src_ctl.read_selectors(elem_id, elem_value)? {
            Ok(true)
        } else if self.mon_out_ctl.read_levels(elem_id, elem_value)? {
            Ok(true)
        } else if self.mon_out_ctl.read_mutes(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_out_ctl.read_levels(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_out_ctl.read_mutes(elem_id, elem_value)? {
            Ok(true)
        } else if self.spdif_out_ctl.read_selectors(elem_id, elem_value)? {
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
        } else if self
            .phys_in_ctl
            .write_level(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .mon_src_ctl
            .write_selector(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .mon_out_ctl
            .write_level(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .mon_out_ctl
            .write_mute(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .mixer_out_ctl
            .write_level(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .mixer_out_ctl
            .write_mute(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .spdif_out_ctl
            .write_selector(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl NotifyModel<(SndUnit, FwNode), bool> for AureonModel {
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

    fn read_notified_elem(
        &mut self,
        _: &(SndUnit, FwNode),
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        self.clk_ctl.read_freq(elem_id, elem_value)
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
    }

    #[test]
    fn test_level_ctl_definition() {
        let mut card_cntr = CardCntr::default();

        let mut ctl = PhysInputCtl::default();
        let error = ctl.load_level(&mut card_cntr).unwrap_err();
        assert_eq!(error.kind::<CardError>(), Some(CardError::Failed));

        let mut ctl = MonitorOutputCtl::default();
        let error = ctl.load_level(&mut card_cntr).unwrap_err();
        assert_eq!(error.kind::<CardError>(), Some(CardError::Failed));

        let mut ctl = MixerOutputCtl::default();
        let error = ctl.load_level(&mut card_cntr).unwrap_err();
        assert_eq!(error.kind::<CardError>(), Some(CardError::Failed));
    }

    #[test]
    fn test_selector_ctl_definition() {
        let mut card_cntr = CardCntr::default();

        let mut ctl = MonitorSourceCtl::default();
        let error = ctl.load_selector(&mut card_cntr).unwrap_err();
        assert_eq!(error.kind::<CardError>(), Some(CardError::Failed));

        let mut ctl = SpdifOutputCtl::default();
        let error = ctl.load_selector(&mut card_cntr).unwrap_err();
        assert_eq!(error.kind::<CardError>(), Some(CardError::Failed));
    }
}
