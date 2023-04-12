// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

use {
    super::{common_ctls::*, *},
    protocols::{terratec::phase88::*, *},
};

#[derive(Default, Debug)]
pub struct Phase88Model {
    avc: BebobAvc,
    clk_ctl: ClkCtl,
    phys_in_ctl: PhysInputCtl,
    mixer_phys_src_ctl: MixerPhysSrcCtl,
    mixer_stream_src_ctl: MixerStreamSrcCtl,
    mixer_out_ctl: MixerOutputCtl,
}

const FCP_TIMEOUT_MS: u32 = 100;

#[derive(Default, Debug)]
struct ClkCtl(Vec<ElemId>, MediaClockParameters, SamplingClockParameters);

impl MediaClkFreqCtlOperation<Phase88ClkProtocol> for ClkCtl {
    fn state(&self) -> &MediaClockParameters {
        &self.1
    }

    fn state_mut(&mut self) -> &mut MediaClockParameters {
        &mut self.1
    }
}

impl SamplingClkSrcCtlOperation<Phase88ClkProtocol> for ClkCtl {
    const SRC_LABELS: &'static [&'static str] = &["Internal", "S/PDIF", "Word-clock"];

    fn state(&self) -> &SamplingClockParameters {
        &self.2
    }

    fn state_mut(&mut self) -> &mut SamplingClockParameters {
        &mut self.2
    }
}

#[derive(Debug)]
struct PhysInputCtl(AvcSelectorParameters);

impl Default for PhysInputCtl {
    fn default() -> Self {
        Self(Phase88PhysInputProtocol::create_selector_parameters())
    }
}

impl AvcSelectorCtlOperation<Phase88PhysInputProtocol> for PhysInputCtl {
    const SELECTOR_NAME: &'static str = "analog-input-7/8-source";
    const SELECTOR_LABELS: &'static [&'static str] = &["analog-intput-7/8"];
    const ITEM_LABELS: &'static [&'static str] = &["line", "mic"];

    fn state(&self) -> &AvcSelectorParameters {
        &self.0
    }

    fn state_mut(&mut self) -> &mut AvcSelectorParameters {
        &mut self.0
    }
}

#[derive(Debug)]
struct MixerPhysSrcCtl(AvcLevelParameters, AvcMuteParameters);

impl Default for MixerPhysSrcCtl {
    fn default() -> Self {
        Self(
            Phase88MixerPhysSourceProtocol::create_level_parameters(),
            Phase88MixerPhysSourceProtocol::create_mute_parameters(),
        )
    }
}

impl AvcLevelCtlOperation<Phase88MixerPhysSourceProtocol> for MixerPhysSrcCtl {
    const LEVEL_NAME: &'static str = "mixer-phys-source-gain";
    const PORT_LABELS: &'static [&'static str] = &[
        "analog-input-1",
        "analog-input-2",
        "analog-input-3",
        "analog-input-4",
        "analog-input-5",
        "analog-input-6",
        "analog-input-7",
        "analog-input-8",
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

impl AvcMuteCtlOperation<Phase88MixerPhysSourceProtocol> for MixerPhysSrcCtl {
    const MUTE_NAME: &'static str = "mixer-phys-source-mute";

    fn state(&self) -> &AvcMuteParameters {
        &self.1
    }

    fn state_mut(&mut self) -> &mut AvcMuteParameters {
        &mut self.1
    }
}

#[derive(Debug)]
struct MixerStreamSrcCtl(AvcLevelParameters, AvcMuteParameters, AvcSelectorParameters);

impl Default for MixerStreamSrcCtl {
    fn default() -> Self {
        Self(
            Phase88MixerStreamSourceProtocol::create_level_parameters(),
            Phase88MixerStreamSourceProtocol::create_mute_parameters(),
            Phase88MixerStreamSourceProtocol::create_selector_parameters(),
        )
    }
}

impl AvcLevelCtlOperation<Phase88MixerStreamSourceProtocol> for MixerStreamSrcCtl {
    const LEVEL_NAME: &'static str = "mixer-stream-source-gain";
    const PORT_LABELS: &'static [&'static str] = &["stream-source-1", "stream-source-2"];

    fn state(&self) -> &AvcLevelParameters {
        &self.0
    }

    fn state_mut(&mut self) -> &mut AvcLevelParameters {
        &mut self.0
    }
}

impl AvcMuteCtlOperation<Phase88MixerStreamSourceProtocol> for MixerStreamSrcCtl {
    const MUTE_NAME: &'static str = "mixer-stream-source-mute";

    fn state(&self) -> &AvcMuteParameters {
        &self.1
    }

    fn state_mut(&mut self) -> &mut AvcMuteParameters {
        &mut self.1
    }
}

impl AvcSelectorCtlOperation<Phase88MixerStreamSourceProtocol> for MixerStreamSrcCtl {
    const SELECTOR_NAME: &'static str = "mixer-stream-source";
    const SELECTOR_LABELS: &'static [&'static str] = &["stream-source-1/2"];
    const ITEM_LABELS: &'static [&'static str] = &[
        "stream-input-1/2",
        "stream-input-3/4",
        "stream-input-5/6",
        "stream-input-7/8",
        "stream-input-9/10",
    ];

    fn state(&self) -> &AvcSelectorParameters {
        &self.2
    }

    fn state_mut(&mut self) -> &mut AvcSelectorParameters {
        &mut self.2
    }
}

#[derive(Debug)]
struct MixerOutputCtl(AvcLevelParameters, AvcMuteParameters, AvcSelectorParameters);

impl Default for MixerOutputCtl {
    fn default() -> Self {
        Self(
            Phase88MixerOutputProtocol::create_level_parameters(),
            Phase88MixerOutputProtocol::create_mute_parameters(),
            Phase88MixerOutputProtocol::create_selector_parameters(),
        )
    }
}

impl AvcLevelCtlOperation<Phase88MixerOutputProtocol> for MixerOutputCtl {
    const LEVEL_NAME: &'static str = "mixer-output-volume";
    const PORT_LABELS: &'static [&'static str] = &["mixer-output-1", "mixer-output-2"];

    fn state(&self) -> &AvcLevelParameters {
        &self.0
    }

    fn state_mut(&mut self) -> &mut AvcLevelParameters {
        &mut self.0
    }
}

impl AvcMuteCtlOperation<Phase88MixerOutputProtocol> for MixerOutputCtl {
    const MUTE_NAME: &'static str = "mixer-output-mute";

    fn state(&self) -> &AvcMuteParameters {
        &self.1
    }

    fn state_mut(&mut self) -> &mut AvcMuteParameters {
        &mut self.1
    }
}

impl AvcSelectorCtlOperation<Phase88MixerOutputProtocol> for MixerOutputCtl {
    const SELECTOR_NAME: &'static str = "mixer-output-volume";
    const SELECTOR_LABELS: &'static [&'static str] = &["mixer-output-1/2"];
    const ITEM_LABELS: &'static [&'static str] = &[
        "analog-output-1/2",
        "analog-output-3/4",
        "analog-output-5/6",
        "analog-output-7/8",
        "digital-output-1/2",
        "unused",
    ];

    fn state(&self) -> &AvcSelectorParameters {
        &self.2
    }

    fn state_mut(&mut self) -> &mut AvcSelectorParameters {
        &mut self.2
    }
}

impl CtlModel<(SndUnit, FwNode)> for Phase88Model {
    fn cache(&mut self, unit: &mut (SndUnit, FwNode)) -> Result<(), Error> {
        self.avc.bind(&unit.1)?;

        self.clk_ctl.cache_freq(&self.avc, FCP_TIMEOUT_MS)?;
        self.clk_ctl.cache_src(&self.avc, FCP_TIMEOUT_MS)?;
        self.mixer_phys_src_ctl
            .cache_levels(&self.avc, FCP_TIMEOUT_MS)?;
        self.mixer_stream_src_ctl
            .cache_levels(&self.avc, FCP_TIMEOUT_MS)?;
        self.mixer_out_ctl.cache_levels(&self.avc, FCP_TIMEOUT_MS)?;
        self.mixer_phys_src_ctl
            .cache_mutes(&self.avc, FCP_TIMEOUT_MS)?;
        self.mixer_stream_src_ctl
            .cache_mutes(&self.avc, FCP_TIMEOUT_MS)?;
        self.mixer_out_ctl.cache_mutes(&self.avc, FCP_TIMEOUT_MS)?;

        self.phys_in_ctl
            .cache_selectors(&self.avc, FCP_TIMEOUT_MS)?;
        self.mixer_stream_src_ctl
            .cache_selectors(&self.avc, FCP_TIMEOUT_MS)?;
        self.mixer_out_ctl
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

        self.phys_in_ctl.load_selector(card_cntr)?;
        self.mixer_phys_src_ctl.load_level(card_cntr)?;
        self.mixer_phys_src_ctl.load_mute(card_cntr)?;
        self.mixer_stream_src_ctl.load_level(card_cntr)?;
        self.mixer_stream_src_ctl.load_mute(card_cntr)?;
        self.mixer_stream_src_ctl.load_selector(card_cntr)?;
        self.mixer_out_ctl.load_level(card_cntr)?;
        self.mixer_out_ctl.load_mute(card_cntr)?;
        self.mixer_out_ctl.load_selector(card_cntr)?;

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
        } else if self.mixer_phys_src_ctl.read_levels(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_phys_src_ctl.read_mutes(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_stream_src_ctl.read_levels(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_stream_src_ctl.read_mutes(elem_id, elem_value)? {
            Ok(true)
        } else if self
            .mixer_stream_src_ctl
            .read_selectors(elem_id, elem_value)?
        {
            Ok(true)
        } else if self.mixer_out_ctl.read_levels(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_out_ctl.read_mutes(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_out_ctl.read_selectors(elem_id, elem_value)? {
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
        } else if self.mixer_phys_src_ctl.write_level(
            &self.avc,
            elem_id,
            old,
            new,
            FCP_TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self.mixer_phys_src_ctl.write_mute(
            &self.avc,
            elem_id,
            old,
            new,
            FCP_TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self.mixer_stream_src_ctl.write_level(
            &self.avc,
            elem_id,
            old,
            new,
            FCP_TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self.mixer_stream_src_ctl.write_mute(
            &self.avc,
            elem_id,
            old,
            new,
            FCP_TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self.mixer_stream_src_ctl.write_selector(
            &self.avc,
            elem_id,
            old,
            new,
            FCP_TIMEOUT_MS,
        )? {
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
            .mixer_out_ctl
            .write_selector(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl NotifyModel<(SndUnit, FwNode), bool> for Phase88Model {
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

        let mut ctl = MixerPhysSrcCtl::default();
        let error = ctl.load_level(&mut card_cntr).unwrap_err();
        assert_eq!(error.kind::<CardError>(), Some(CardError::Failed));

        let mut ctl = MixerStreamSrcCtl::default();
        let error = ctl.load_level(&mut card_cntr).unwrap_err();
        assert_eq!(error.kind::<CardError>(), Some(CardError::Failed));

        let mut ctl = MixerOutputCtl::default();
        let error = ctl.load_level(&mut card_cntr).unwrap_err();
        assert_eq!(error.kind::<CardError>(), Some(CardError::Failed));
    }

    #[test]
    fn test_selector_ctl_definition() {
        let mut card_cntr = CardCntr::default();

        let mut ctl = PhysInputCtl::default();
        let error = ctl.load_selector(&mut card_cntr).unwrap_err();
        assert_eq!(error.kind::<CardError>(), Some(CardError::Failed));

        let mut ctl = MixerStreamSrcCtl::default();
        let error = ctl.load_selector(&mut card_cntr).unwrap_err();
        assert_eq!(error.kind::<CardError>(), Some(CardError::Failed));

        let mut ctl = MixerOutputCtl::default();
        let error = ctl.load_selector(&mut card_cntr).unwrap_err();
        assert_eq!(error.kind::<CardError>(), Some(CardError::Failed));
    }
}
