// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

use {
    super::*,
    protocols::{presonus::firebox::*, *},
};

#[derive(Default, Debug)]
pub struct FireboxModel {
    avc: BebobAvc,
    clk_ctl: ClkCtl,
    phys_out_ctl: PhysOutputCtl,
    headphone_ctl: HeadphoneCtl,
    mixer_phys_src_ctl: MixerPhysSrcCtl,
    mixer_stream_src_ctl: MixerStreamSrcCtl,
    mixer_out_ctl: MixerOutputCtl,
    analog_in_ctl: AnalogInputCtl,
}

const FCP_TIMEOUT_MS: u32 = 100;

#[derive(Default, Debug)]
struct ClkCtl(Vec<ElemId>, MediaClockParameters, SamplingClockParameters);

impl MediaClkFreqCtlOperation<FireboxClkProtocol> for ClkCtl {
    fn state(&self) -> &MediaClockParameters {
        &self.1
    }

    fn state_mut(&mut self) -> &mut MediaClockParameters {
        &mut self.1
    }
}

impl SamplingClkSrcCtlOperation<FireboxClkProtocol> for ClkCtl {
    const SRC_LABELS: &'static [&'static str] = &["Internal", "S/PDIF"];

    fn state(&self) -> &SamplingClockParameters {
        &self.2
    }

    fn state_mut(&mut self) -> &mut SamplingClockParameters {
        &mut self.2
    }
}

#[derive(Debug)]
struct PhysOutputCtl(AvcLevelParameters, AvcMuteParameters, AvcSelectorParameters);

impl Default for PhysOutputCtl {
    fn default() -> Self {
        Self(
            FireboxPhysOutputProtocol::create_level_parameters(),
            FireboxPhysOutputProtocol::create_mute_parameters(),
            FireboxPhysOutputProtocol::create_selector_parameters(),
        )
    }
}

impl AvcLevelCtlOperation<FireboxPhysOutputProtocol> for PhysOutputCtl {
    const LEVEL_NAME: &'static str = OUT_VOL_NAME;
    const PORT_LABELS: &'static [&'static str] = &[
        "analog-output-1",
        "analog-output-2",
        "analog-output-3",
        "analog-output-4",
        "analog-output-5",
        "analog-output-6",
    ];

    fn state(&self) -> &AvcLevelParameters {
        &self.0
    }

    fn state_mut(&mut self) -> &mut AvcLevelParameters {
        &mut self.0
    }
}

impl AvcMuteCtlOperation<FireboxPhysOutputProtocol> for PhysOutputCtl {
    const MUTE_NAME: &'static str = "phys-output-mute";

    fn state(&self) -> &AvcMuteParameters {
        &self.1
    }

    fn state_mut(&mut self) -> &mut AvcMuteParameters {
        &mut self.1
    }
}

impl AvcSelectorCtlOperation<FireboxPhysOutputProtocol> for PhysOutputCtl {
    const SELECTOR_NAME: &'static str = "output-source";
    const SELECTOR_LABELS: &'static [&'static str] = &[
        "analog-output-1/2",
        "analog-output-3/4",
        "analog-output-5/6",
        "analog-output-7/8",
    ];
    const ITEM_LABELS: &'static [&'static str] = &["stream-input", "mixer-output-1/2"];

    fn state(&self) -> &AvcSelectorParameters {
        &self.2
    }

    fn state_mut(&mut self) -> &mut AvcSelectorParameters {
        &mut self.2
    }
}

#[derive(Debug)]
struct HeadphoneCtl(AvcLevelParameters, AvcMuteParameters, AvcSelectorParameters);

impl Default for HeadphoneCtl {
    fn default() -> Self {
        Self(
            FireboxHeadphoneProtocol::create_level_parameters(),
            FireboxHeadphoneProtocol::create_mute_parameters(),
            FireboxHeadphoneProtocol::create_selector_parameters(),
        )
    }
}

impl AvcLevelCtlOperation<FireboxHeadphoneProtocol> for HeadphoneCtl {
    const LEVEL_NAME: &'static str = "headphone-gain";
    const PORT_LABELS: &'static [&'static str] = &["headphone-1", "headphone-2"];

    fn state(&self) -> &AvcLevelParameters {
        &self.0
    }

    fn state_mut(&mut self) -> &mut AvcLevelParameters {
        &mut self.0
    }
}

impl AvcMuteCtlOperation<FireboxHeadphoneProtocol> for HeadphoneCtl {
    const MUTE_NAME: &'static str = "headphone-mute";

    fn state(&self) -> &AvcMuteParameters {
        &self.1
    }

    fn state_mut(&mut self) -> &mut AvcMuteParameters {
        &mut self.1
    }
}

impl AvcSelectorCtlOperation<FireboxHeadphoneProtocol> for HeadphoneCtl {
    const SELECTOR_NAME: &'static str = "headphone-source";
    const SELECTOR_LABELS: &'static [&'static str] = &["headphone-1/2"];
    const ITEM_LABELS: &'static [&'static str] = &[
        "stream-input-1/2",
        "stream-input-3/4",
        "stream-input-5/6",
        "stream-input-7/8",
        "mixer-output-1/2",
    ];

    fn state(&self) -> &AvcSelectorParameters {
        &self.2
    }

    fn state_mut(&mut self) -> &mut AvcSelectorParameters {
        &mut self.2
    }
}

#[derive(Debug)]
struct MixerPhysSrcCtl(
    AvcLevelParameters,
    AvcLrBalanceParameters,
    AvcMuteParameters,
);

impl Default for MixerPhysSrcCtl {
    fn default() -> Self {
        Self(
            FireboxMixerPhysSourceProtocol::create_level_parameters(),
            FireboxMixerPhysSourceProtocol::create_lr_balance_parameters(),
            FireboxMixerPhysSourceProtocol::create_mute_parameters(),
        )
    }
}

impl AvcLevelCtlOperation<FireboxMixerPhysSourceProtocol> for MixerPhysSrcCtl {
    const LEVEL_NAME: &'static str = "mixer-phys-source-gain";
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

impl AvcLrBalanceCtlOperation<FireboxMixerPhysSourceProtocol> for MixerPhysSrcCtl {
    const BALANCE_NAME: &'static str = "phys-input-balance";

    fn state(&self) -> &AvcLrBalanceParameters {
        &self.1
    }

    fn state_mut(&mut self) -> &mut AvcLrBalanceParameters {
        &mut self.1
    }
}

impl AvcMuteCtlOperation<FireboxMixerPhysSourceProtocol> for MixerPhysSrcCtl {
    const MUTE_NAME: &'static str = "mixer-phys-source-mute";

    fn state(&self) -> &AvcMuteParameters {
        &self.2
    }

    fn state_mut(&mut self) -> &mut AvcMuteParameters {
        &mut self.2
    }
}

#[derive(Debug)]
struct MixerStreamSrcCtl(AvcLevelParameters, AvcMuteParameters, AvcSelectorParameters);

impl Default for MixerStreamSrcCtl {
    fn default() -> Self {
        Self(
            FireboxMixerStreamSourceProtocol::create_level_parameters(),
            FireboxMixerStreamSourceProtocol::create_mute_parameters(),
            FireboxMixerStreamSourceProtocol::create_selector_parameters(),
        )
    }
}

impl AvcLevelCtlOperation<FireboxMixerStreamSourceProtocol> for MixerStreamSrcCtl {
    const LEVEL_NAME: &'static str = "mixer-stream-source-gain";
    const PORT_LABELS: &'static [&'static str] = &["stream-input-1/2"];

    fn state(&self) -> &AvcLevelParameters {
        &self.0
    }

    fn state_mut(&mut self) -> &mut AvcLevelParameters {
        &mut self.0
    }
}

impl AvcMuteCtlOperation<FireboxMixerStreamSourceProtocol> for MixerStreamSrcCtl {
    const MUTE_NAME: &'static str = "mixer-stream-source-mute";

    fn state(&self) -> &AvcMuteParameters {
        &self.1
    }

    fn state_mut(&mut self) -> &mut AvcMuteParameters {
        &mut self.1
    }
}

impl AvcSelectorCtlOperation<FireboxMixerStreamSourceProtocol> for MixerStreamSrcCtl {
    const SELECTOR_NAME: &'static str = "mixer-stream-source";
    const SELECTOR_LABELS: &'static [&'static str] = &["mixer-stream-source-1/2"];
    const ITEM_LABELS: &'static [&'static str] = &[
        "stream-input-1/2",
        "stream-input-3/4",
        "stream-input-5/6",
        "stream-input-7/8",
    ];

    fn state(&self) -> &AvcSelectorParameters {
        &self.2
    }

    fn state_mut(&mut self) -> &mut AvcSelectorParameters {
        &mut self.2
    }
}

#[derive(Debug)]
struct MixerOutputCtl(
    AvcLevelParameters,
    AvcLrBalanceParameters,
    AvcMuteParameters,
);

impl Default for MixerOutputCtl {
    fn default() -> Self {
        Self(
            FireboxMixerOutputProtocol::create_level_parameters(),
            FireboxMixerOutputProtocol::create_lr_balance_parameters(),
            FireboxMixerOutputProtocol::create_mute_parameters(),
        )
    }
}

impl AvcLevelCtlOperation<FireboxMixerOutputProtocol> for MixerOutputCtl {
    const LEVEL_NAME: &'static str = "mixer-output-volume";
    const PORT_LABELS: &'static [&'static str] = &["mixer-output-1", "mixer-output-2"];

    fn state(&self) -> &AvcLevelParameters {
        &self.0
    }

    fn state_mut(&mut self) -> &mut AvcLevelParameters {
        &mut self.0
    }
}

impl AvcLrBalanceCtlOperation<FireboxMixerOutputProtocol> for MixerOutputCtl {
    const BALANCE_NAME: &'static str = "phys-input-balance";

    fn state(&self) -> &AvcLrBalanceParameters {
        &self.1
    }

    fn state_mut(&mut self) -> &mut AvcLrBalanceParameters {
        &mut self.1
    }
}

impl AvcMuteCtlOperation<FireboxMixerOutputProtocol> for MixerOutputCtl {
    const MUTE_NAME: &'static str = "mixer-phys-source-mute";

    fn state(&self) -> &AvcMuteParameters {
        &self.2
    }

    fn state_mut(&mut self) -> &mut AvcMuteParameters {
        &mut self.2
    }
}

#[derive(Default, Debug)]
struct AnalogInputCtl(FireboxAnalogInputParameters);

impl AnalogInputCtl {
    const SWITCH_NAME: &'static str = "analog-input-boost";

    fn load(&self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::SWITCH_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, 1, self.0.boosts.len(), true)
            .map(|_| ())
    }

    fn cache(&mut self, avc: &BebobAvc, timeout_ms: u32) -> Result<(), Error> {
        let res = FireboxAnalogInputProtocol::cache(avc, &mut self.0, timeout_ms);
        debug!(params = ?self.0, ?res);
        res
    }

    fn read(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            Self::SWITCH_NAME => {
                elem_value.set_bool(&self.0.boosts);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write(
        &mut self,
        avc: &BebobAvc,
        elem_id: &ElemId,
        _: &ElemValue,
        new: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            Self::SWITCH_NAME => {
                let mut params = self.0.clone();
                let vals = &new.boolean()[..params.boosts.len()];
                params.boosts.copy_from_slice(&vals);
                let res = FireboxAnalogInputProtocol::update(avc, &params, &mut self.0, timeout_ms);
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

impl CtlModel<(SndUnit, FwNode)> for FireboxModel {
    fn cache(&mut self, unit: &mut (SndUnit, FwNode)) -> Result<(), Error> {
        self.avc.bind(&unit.1)?;

        self.clk_ctl.cache_freq(&self.avc, FCP_TIMEOUT_MS)?;
        self.clk_ctl.cache_src(&self.avc, FCP_TIMEOUT_MS)?;
        self.phys_out_ctl.cache_levels(&self.avc, FCP_TIMEOUT_MS)?;
        self.headphone_ctl.cache_levels(&self.avc, FCP_TIMEOUT_MS)?;
        self.mixer_phys_src_ctl
            .cache_levels(&self.avc, FCP_TIMEOUT_MS)?;
        self.mixer_stream_src_ctl
            .cache_levels(&self.avc, FCP_TIMEOUT_MS)?;
        self.mixer_out_ctl.cache_levels(&self.avc, FCP_TIMEOUT_MS)?;
        self.mixer_phys_src_ctl
            .cache_balances(&self.avc, FCP_TIMEOUT_MS)?;
        self.mixer_out_ctl
            .cache_balances(&self.avc, FCP_TIMEOUT_MS)?;
        self.phys_out_ctl.cache_mutes(&self.avc, FCP_TIMEOUT_MS)?;
        self.headphone_ctl.cache_mutes(&self.avc, FCP_TIMEOUT_MS)?;
        self.mixer_phys_src_ctl
            .cache_mutes(&self.avc, FCP_TIMEOUT_MS)?;
        self.mixer_stream_src_ctl
            .cache_mutes(&self.avc, FCP_TIMEOUT_MS)?;
        self.mixer_out_ctl.cache_mutes(&self.avc, FCP_TIMEOUT_MS)?;
        self.phys_out_ctl
            .cache_selectors(&self.avc, FCP_TIMEOUT_MS)?;
        self.headphone_ctl
            .cache_selectors(&self.avc, FCP_TIMEOUT_MS)?;
        self.mixer_stream_src_ctl
            .cache_selectors(&self.avc, FCP_TIMEOUT_MS)?;
        self.analog_in_ctl.cache(&self.avc, FCP_TIMEOUT_MS)?;

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
        self.headphone_ctl.load_level(card_cntr)?;
        self.headphone_ctl.load_mute(card_cntr)?;
        self.headphone_ctl.load_selector(card_cntr)?;
        self.mixer_phys_src_ctl.load_level(card_cntr)?;
        self.mixer_phys_src_ctl.load_balance(card_cntr)?;
        self.mixer_phys_src_ctl.load_mute(card_cntr)?;
        self.mixer_stream_src_ctl.load_level(card_cntr)?;
        self.mixer_stream_src_ctl.load_mute(card_cntr)?;
        self.mixer_stream_src_ctl.load_selector(card_cntr)?;
        self.mixer_out_ctl.load_level(card_cntr)?;
        self.mixer_out_ctl.load_mute(card_cntr)?;
        self.mixer_out_ctl.load_balance(card_cntr)?;
        self.analog_in_ctl.load(card_cntr)?;

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
        } else if self.headphone_ctl.read_levels(elem_id, elem_value)? {
            Ok(true)
        } else if self.headphone_ctl.read_mutes(elem_id, elem_value)? {
            Ok(true)
        } else if self.headphone_ctl.read_selectors(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_phys_src_ctl.read_levels(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_phys_src_ctl.read_balances(elem_id, elem_value)? {
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
        } else if self.mixer_out_ctl.read_balances(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_out_ctl.read_mutes(elem_id, elem_value)? {
            Ok(true)
        } else if self.analog_in_ctl.read(elem_id, elem_value)? {
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
            .write_mute(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .phys_out_ctl
            .write_selector(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .headphone_ctl
            .write_level(&self.avc, elem_id, new, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .headphone_ctl
            .write_mute(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .headphone_ctl
            .write_selector(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .mixer_phys_src_ctl
            .write_level(&self.avc, elem_id, new, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else if self.mixer_phys_src_ctl.write_balance(
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
        } else if self
            .mixer_stream_src_ctl
            .write_level(&self.avc, elem_id, new, FCP_TIMEOUT_MS)?
        {
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
            .write_level(&self.avc, elem_id, new, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .mixer_out_ctl
            .write_balance(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .mixer_out_ctl
            .write_mute(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .analog_in_ctl
            .write(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl NotifyModel<(SndUnit, FwNode), bool> for FireboxModel {
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

        let mut ctl = HeadphoneCtl::default();
        let error = ctl.load_level(&mut card_cntr).unwrap_err();
        assert_eq!(error.kind::<CardError>(), Some(CardError::Failed));

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

        let mut ctl = PhysOutputCtl::default();
        let error = ctl.load_selector(&mut card_cntr).unwrap_err();
        assert_eq!(error.kind::<CardError>(), Some(CardError::Failed));

        let mut ctl = HeadphoneCtl::default();
        let error = ctl.load_selector(&mut card_cntr).unwrap_err();
        assert_eq!(error.kind::<CardError>(), Some(CardError::Failed));

        let mut ctl = MixerStreamSrcCtl::default();
        let error = ctl.load_selector(&mut card_cntr).unwrap_err();
        assert_eq!(error.kind::<CardError>(), Some(CardError::Failed));
    }
}
