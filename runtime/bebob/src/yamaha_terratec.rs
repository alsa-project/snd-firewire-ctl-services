// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

use {
    super::{common_ctls::*, *},
    protocols::{yamaha_terratec::*, *},
};

#[derive(Default, Debug)]
pub struct GoPhase24CoaxModel {
    avc: BebobAvc,
    clk_ctl: ClkCtl,
    phys_in_ctl: CoaxPhysInputCtl,
    phys_out_ctl: CoaxPhysOutputCtl,
    hp_ctl: CoaxHeadphoneCtl,
    mixer_src_ctl: MixerSourceCtl,
    mixer_out_ctl: CoaxMixerOutputCtl,
}

#[derive(Default, Debug)]
pub struct GoPhase24OptModel {
    avc: BebobAvc,
    clk_ctl: ClkCtl,
    phys_out_ctl: OptPhysOutputCtl,
    mixer_src_ctl: MixerSourceCtl,
    mixer_out_ctl: OptMixerOutputCtl,
}

const FCP_TIMEOUT_MS: u32 = 100;

#[derive(Default, Debug)]
struct ClkCtl(Vec<ElemId>, MediaClockParameters, SamplingClockParameters);

impl MediaClkFreqCtlOperation<GoPhase24ClkProtocol> for ClkCtl {
    fn state(&self) -> &MediaClockParameters {
        &self.1
    }

    fn state_mut(&mut self) -> &mut MediaClockParameters {
        &mut self.1
    }
}

impl SamplingClkSrcCtlOperation<GoPhase24ClkProtocol> for ClkCtl {
    const SRC_LABELS: &'static [&'static str] = &["Internal", "S/PDIF"];

    fn state(&self) -> &SamplingClockParameters {
        &self.2
    }

    fn state_mut(&mut self) -> &mut SamplingClockParameters {
        &mut self.2
    }
}

#[derive(Debug)]
struct MixerSourceCtl(AvcLevelParameters, AvcMuteParameters);

impl Default for MixerSourceCtl {
    fn default() -> Self {
        Self(
            GoPhase24MixerSourceProtocol::create_level_parameters(),
            GoPhase24MixerSourceProtocol::create_mute_parameters(),
        )
    }
}

impl AvcLevelCtlOperation<GoPhase24MixerSourceProtocol> for MixerSourceCtl {
    const LEVEL_NAME: &'static str = "mixer-source-gain";
    const PORT_LABELS: &'static [&'static str] = &[
        "analog-input-1",
        "analog-input-2",
        "digital-input-1",
        "digital-input-2",
        "stream-input-1",
        "stream-input-2",
        "stream-input-3",
        "stream-input-4",
        "stream-input-5",
        "stream-input-6",
    ];

    fn state(&self) -> &AvcLevelParameters {
        &self.0
    }

    fn state_mut(&mut self) -> &mut AvcLevelParameters {
        &mut self.0
    }
}

impl AvcMuteCtlOperation<GoPhase24MixerSourceProtocol> for MixerSourceCtl {
    const MUTE_NAME: &'static str = "mixer-source-mute";

    fn state(&self) -> &AvcMuteParameters {
        &self.1
    }

    fn state_mut(&mut self) -> &mut AvcMuteParameters {
        &mut self.1
    }
}

#[derive(Debug)]
struct CoaxPhysInputCtl(AvcSelectorParameters);

impl Default for CoaxPhysInputCtl {
    fn default() -> Self {
        Self(GoPhase24CoaxPhysInputProtocol::create_selector_parameters())
    }
}

impl AvcSelectorCtlOperation<GoPhase24CoaxPhysInputProtocol> for CoaxPhysInputCtl {
    const SELECTOR_NAME: &'static str = "analog-input-level";
    const SELECTOR_LABELS: &'static [&'static str] = &["analog-input-1/2"];
    const ITEM_LABELS: &'static [&'static str] = &["low", "middle", "high"];

    fn state(&self) -> &AvcSelectorParameters {
        &self.0
    }

    fn state_mut(&mut self) -> &mut AvcSelectorParameters {
        &mut self.0
    }
}

#[derive(Debug)]
struct CoaxPhysOutputCtl(AvcSelectorParameters);

impl Default for CoaxPhysOutputCtl {
    fn default() -> Self {
        Self(GoPhase24CoaxPhysOutputProtocol::create_selector_parameters())
    }
}

impl AvcSelectorCtlOperation<GoPhase24CoaxPhysOutputProtocol> for CoaxPhysOutputCtl {
    const SELECTOR_NAME: &'static str = "phys-output-source";
    const SELECTOR_LABELS: &'static [&'static str] = &["analog-output-1/2", "analog-output-3/4"];
    const ITEM_LABELS: &'static [&'static str] = &[
        "stream-input-1/2",
        "stream-input-3/4",
        "analog-input-1/2",
        "digital-input-1/2",
        "mixer-output-1/2",
        "stream-input-5/6",
    ];

    fn state(&self) -> &AvcSelectorParameters {
        &self.0
    }

    fn state_mut(&mut self) -> &mut AvcSelectorParameters {
        &mut self.0
    }
}

#[derive(Debug)]
struct CoaxHeadphoneCtl(AvcSelectorParameters);

impl Default for CoaxHeadphoneCtl {
    fn default() -> Self {
        Self(GoPhase24CoaxHeadphoneProtocol::create_selector_parameters())
    }
}

impl AvcSelectorCtlOperation<GoPhase24CoaxHeadphoneProtocol> for CoaxHeadphoneCtl {
    const SELECTOR_NAME: &'static str = "headphone-source";
    const SELECTOR_LABELS: &'static [&'static str] = &["headphone-1/2"];
    const ITEM_LABELS: &'static [&'static str] = &[
        "stream-input-1/2",
        "stream-input-3/4",
        "analog-input-1/2",
        "digital-input-1/2",
        "mixer-output-1/2",
        "stream-input-5/6",
    ];

    fn state(&self) -> &AvcSelectorParameters {
        &self.0
    }

    fn state_mut(&mut self) -> &mut AvcSelectorParameters {
        &mut self.0
    }
}

#[derive(Debug)]
struct OptPhysOutputCtl(AvcLevelParameters, AvcMuteParameters, AvcSelectorParameters);

impl Default for OptPhysOutputCtl {
    fn default() -> Self {
        Self(
            GoPhase24OptPhysOutputProtocol::create_level_parameters(),
            GoPhase24OptPhysOutputProtocol::create_mute_parameters(),
            GoPhase24OptPhysOutputProtocol::create_selector_parameters(),
        )
    }
}

impl AvcLevelCtlOperation<GoPhase24OptPhysOutputProtocol> for OptPhysOutputCtl {
    const LEVEL_NAME: &'static str = "phy-output-volume";
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

impl AvcMuteCtlOperation<GoPhase24OptPhysOutputProtocol> for OptPhysOutputCtl {
    const MUTE_NAME: &'static str = "phys-output-mute";

    fn state(&self) -> &AvcMuteParameters {
        &self.1
    }

    fn state_mut(&mut self) -> &mut AvcMuteParameters {
        &mut self.1
    }
}

impl AvcSelectorCtlOperation<GoPhase24OptPhysOutputProtocol> for OptPhysOutputCtl {
    const SELECTOR_NAME: &'static str = "phys-output-source";
    const SELECTOR_LABELS: &'static [&'static str] = &[
        "analog-output-1/2",
        "analog-output-3/4",
        "digital-output-1/2",
    ];
    const ITEM_LABELS: &'static [&'static str] = &[
        "stream-input-1/2",
        "stream-input-3/4",
        "analog-input-1/2",
        "digital-input-1/2",
        "mixer-output-1/2",
        "stream-input-5/6",
    ];

    fn state(&self) -> &AvcSelectorParameters {
        &self.2
    }

    fn state_mut(&mut self) -> &mut AvcSelectorParameters {
        &mut self.2
    }
}

#[derive(Debug)]
struct CoaxMixerOutputCtl(AvcLevelParameters, AvcMuteParameters);

impl Default for CoaxMixerOutputCtl {
    fn default() -> Self {
        Self(
            GoPhase24CoaxMixerOutputProtocol::create_level_parameters(),
            GoPhase24CoaxMixerOutputProtocol::create_mute_parameters(),
        )
    }
}

impl AvcLevelCtlOperation<GoPhase24CoaxMixerOutputProtocol> for CoaxMixerOutputCtl {
    const LEVEL_NAME: &'static str = "mixer-output-volume";
    const PORT_LABELS: &'static [&'static str] = &["mixer-output-1", "mixer-output-2"];

    fn state(&self) -> &AvcLevelParameters {
        &self.0
    }

    fn state_mut(&mut self) -> &mut AvcLevelParameters {
        &mut self.0
    }
}

impl AvcMuteCtlOperation<GoPhase24CoaxMixerOutputProtocol> for CoaxMixerOutputCtl {
    const MUTE_NAME: &'static str = "mixer-output-mute";

    fn state(&self) -> &AvcMuteParameters {
        &self.1
    }

    fn state_mut(&mut self) -> &mut AvcMuteParameters {
        &mut self.1
    }
}

#[derive(Debug)]
struct OptMixerOutputCtl(AvcLevelParameters, AvcMuteParameters);

impl Default for OptMixerOutputCtl {
    fn default() -> Self {
        Self(
            GoPhase24OptMixerOutputProtocol::create_level_parameters(),
            GoPhase24OptMixerOutputProtocol::create_mute_parameters(),
        )
    }
}

impl AvcLevelCtlOperation<GoPhase24OptMixerOutputProtocol> for OptMixerOutputCtl {
    const LEVEL_NAME: &'static str = "mixer-output-volume";
    const PORT_LABELS: &'static [&'static str] = &["mixer-output-1", "mixer-output-2"];

    fn state(&self) -> &AvcLevelParameters {
        &self.0
    }

    fn state_mut(&mut self) -> &mut AvcLevelParameters {
        &mut self.0
    }
}

impl AvcMuteCtlOperation<GoPhase24OptMixerOutputProtocol> for OptMixerOutputCtl {
    const MUTE_NAME: &'static str = "mixer-output-mute";

    fn state(&self) -> &AvcMuteParameters {
        &self.1
    }

    fn state_mut(&mut self) -> &mut AvcMuteParameters {
        &mut self.1
    }
}

impl CtlModel<(SndUnit, FwNode)> for GoPhase24CoaxModel {
    fn cache(&mut self, unit: &mut (SndUnit, FwNode)) -> Result<(), Error> {
        self.avc.bind(&unit.1)?;

        self.clk_ctl.cache_freq(&self.avc, FCP_TIMEOUT_MS)?;
        self.clk_ctl.cache_src(&self.avc, FCP_TIMEOUT_MS)?;
        self.mixer_src_ctl.cache_levels(&self.avc, FCP_TIMEOUT_MS)?;
        self.mixer_out_ctl.cache_levels(&self.avc, FCP_TIMEOUT_MS)?;
        self.mixer_src_ctl.cache_mutes(&self.avc, FCP_TIMEOUT_MS)?;
        self.mixer_out_ctl.cache_mutes(&self.avc, FCP_TIMEOUT_MS)?;
        self.phys_in_ctl
            .cache_selectors(&self.avc, FCP_TIMEOUT_MS)?;
        self.phys_out_ctl
            .cache_selectors(&self.avc, FCP_TIMEOUT_MS)?;
        self.hp_ctl.cache_selectors(&self.avc, FCP_TIMEOUT_MS)?;

        Ok(())
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.clk_ctl
            .load_freq(card_cntr)
            .map(|mut elem_id_list| self.clk_ctl.0.append(&mut elem_id_list))?;

        self.clk_ctl
            .load_src(card_cntr)
            .map(|mut elem_id_list| self.clk_ctl.0.append(&mut elem_id_list))?;

        self.phys_in_ctl.load_selector(card_cntr)?;
        self.phys_out_ctl.load_selector(card_cntr)?;
        self.hp_ctl.load_selector(card_cntr)?;
        self.mixer_src_ctl.load_level(card_cntr)?;
        self.mixer_src_ctl.load_mute(card_cntr)?;
        self.mixer_out_ctl.load_level(card_cntr)?;
        self.mixer_out_ctl.load_mute(card_cntr)?;

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
        } else if self.phys_in_ctl.read_selectors(elem_id, elem_value)? {
            Ok(true)
        } else if self.phys_out_ctl.read_selectors(elem_id, elem_value)? {
            Ok(true)
        } else if self.hp_ctl.read_selectors(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_src_ctl.read_levels(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_src_ctl.read_mutes(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_out_ctl.read_levels(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_out_ctl.read_mutes(elem_id, elem_value)? {
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
            .phys_in_ctl
            .write_selector(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .phys_out_ctl
            .write_selector(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .hp_ctl
            .write_selector(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .mixer_src_ctl
            .write_level(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .mixer_src_ctl
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
        } else {
            Ok(false)
        }
    }
}

impl NotifyModel<(SndUnit, FwNode), bool> for GoPhase24CoaxModel {
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

impl CtlModel<(SndUnit, FwNode)> for GoPhase24OptModel {
    fn cache(&mut self, unit: &mut (SndUnit, FwNode)) -> Result<(), Error> {
        self.avc.bind(&unit.1)?;

        self.phys_out_ctl.cache_levels(&self.avc, FCP_TIMEOUT_MS)?;
        self.mixer_src_ctl.cache_levels(&self.avc, FCP_TIMEOUT_MS)?;
        self.mixer_out_ctl.cache_levels(&self.avc, FCP_TIMEOUT_MS)?;
        self.phys_out_ctl.cache_mutes(&self.avc, FCP_TIMEOUT_MS)?;
        self.mixer_src_ctl.cache_mutes(&self.avc, FCP_TIMEOUT_MS)?;
        self.mixer_out_ctl.cache_mutes(&self.avc, FCP_TIMEOUT_MS)?;

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
        self.phys_out_ctl.load_mute(card_cntr)?;
        self.phys_out_ctl.load_selector(card_cntr)?;
        self.mixer_src_ctl.load_level(card_cntr)?;
        self.mixer_src_ctl.load_mute(card_cntr)?;
        self.mixer_out_ctl.load_level(card_cntr)?;
        self.mixer_out_ctl.load_mute(card_cntr)?;

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
        } else if self.phys_out_ctl.read_mutes(elem_id, elem_value)? {
            Ok(true)
        } else if self.phys_out_ctl.read_selectors(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_src_ctl.read_levels(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_src_ctl.read_mutes(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_out_ctl.read_levels(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_out_ctl.read_mutes(elem_id, elem_value)? {
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
            .write_mute(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .phys_out_ctl
            .write_selector(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .mixer_src_ctl
            .write_level(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .mixer_src_ctl
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
        } else {
            Ok(false)
        }
    }
}

impl NotifyModel<(SndUnit, FwNode), bool> for GoPhase24OptModel {
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

        let mut ctl = OptPhysOutputCtl::default();
        let error = ctl.load_level(&mut card_cntr).unwrap_err();
        assert_eq!(error.kind::<CardError>(), Some(CardError::Failed));

        let mut ctl = CoaxMixerOutputCtl::default();
        let error = ctl.load_level(&mut card_cntr).unwrap_err();
        assert_eq!(error.kind::<CardError>(), Some(CardError::Failed));

        let mut ctl = MixerSourceCtl::default();
        let error = ctl.load_level(&mut card_cntr).unwrap_err();
        assert_eq!(error.kind::<CardError>(), Some(CardError::Failed));

        let mut ctl = OptMixerOutputCtl::default();
        let error = ctl.load_level(&mut card_cntr).unwrap_err();
        assert_eq!(error.kind::<CardError>(), Some(CardError::Failed));
    }

    #[test]
    fn test_selector_ctl_definition() {
        let mut card_cntr = CardCntr::default();

        let mut ctl = CoaxPhysInputCtl::default();
        let error = ctl.load_selector(&mut card_cntr).unwrap_err();
        assert_eq!(error.kind::<CardError>(), Some(CardError::Failed));

        let mut ctl = CoaxPhysOutputCtl::default();
        let error = ctl.load_selector(&mut card_cntr).unwrap_err();
        assert_eq!(error.kind::<CardError>(), Some(CardError::Failed));

        let mut ctl = CoaxHeadphoneCtl::default();
        let error = ctl.load_selector(&mut card_cntr).unwrap_err();
        assert_eq!(error.kind::<CardError>(), Some(CardError::Failed));

        let mut ctl = OptPhysOutputCtl::default();
        let error = ctl.load_selector(&mut card_cntr).unwrap_err();
        assert_eq!(error.kind::<CardError>(), Some(CardError::Failed));
    }
}
