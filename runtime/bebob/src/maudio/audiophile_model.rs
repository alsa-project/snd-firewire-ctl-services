// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use super::*;

#[derive(Default, Debug)]
pub struct AudiophileModel {
    avc: BebobAvc,
    req: FwReq,
    clk_ctl: ClkCtl,
    meter_ctl: MeterCtl,
    phys_input_ctl: PhysInputCtl,
    aux_src_ctl: AuxSourceCtl,
    aux_output_ctl: AuxOutputCtl,
    phys_output_ctl: PhysOutputCtl,
    hp_ctl: HeadphoneCtl,
    mixer_ctl: MixerCtl,
}

const FCP_TIMEOUT_MS: u32 = 100;
const TIMEOUT_MS: u32 = 50;

#[derive(Default, Debug)]
struct ClkCtl(Vec<ElemId>, MediaClockParameters, SamplingClockParameters);

impl MediaClkFreqCtlOperation<AudiophileClkProtocol> for ClkCtl {
    fn state(&self) -> &MediaClockParameters {
        &self.1
    }

    fn state_mut(&mut self) -> &mut MediaClockParameters {
        &mut self.1
    }
}

impl SamplingClkSrcCtlOperation<AudiophileClkProtocol> for ClkCtl {
    const SRC_LABELS: &'static [&'static str] = &["Internal", "S/PDIF"];

    fn state(&self) -> &SamplingClockParameters {
        &self.2
    }

    fn state_mut(&mut self) -> &mut SamplingClockParameters {
        &mut self.2
    }
}

#[derive(Debug)]
struct MeterCtl(Vec<ElemId>, MaudioNormalMeter);

impl Default for MeterCtl {
    fn default() -> Self {
        Self(Default::default(), AudiophileMeterProtocol::create_meter())
    }
}

impl MaudioNormalMeterCtlOperation<AudiophileMeterProtocol> for MeterCtl {
    fn state(&self) -> &MaudioNormalMeter {
        &self.1
    }

    fn state_mut(&mut self) -> &mut MaudioNormalMeter {
        &mut self.1
    }
}

#[derive(Debug)]
struct PhysInputCtl(AvcLevelParameters, AvcLrBalanceParameters);

impl Default for PhysInputCtl {
    fn default() -> Self {
        Self(
            AudiophilePhysInputProtocol::create_level_parameters(),
            AudiophilePhysInputProtocol::create_lr_balance_parameters(),
        )
    }
}

impl AvcLevelCtlOperation<AudiophilePhysInputProtocol> for PhysInputCtl {
    const LEVEL_NAME: &'static str = "phys-input-gain";
    const PORT_LABELS: &'static [&'static str] = &[
        "analog-input-1",
        "analog-input-2",
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

impl AvcLrBalanceCtlOperation<AudiophilePhysInputProtocol> for PhysInputCtl {
    const BALANCE_NAME: &'static str = "phys-input-balance";

    fn state(&self) -> &AvcLrBalanceParameters {
        &self.1
    }

    fn state_mut(&mut self) -> &mut AvcLrBalanceParameters {
        &mut self.1
    }
}

#[derive(Debug)]
struct AuxSourceCtl(AvcLevelParameters);

impl Default for AuxSourceCtl {
    fn default() -> Self {
        Self(AudiophileAuxSourceProtocol::create_level_parameters())
    }
}

impl AvcLevelCtlOperation<AudiophileAuxSourceProtocol> for AuxSourceCtl {
    const LEVEL_NAME: &'static str = "aux-source-gain";
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

#[derive(Debug)]
struct AuxOutputCtl(AvcLevelParameters);

impl Default for AuxOutputCtl {
    fn default() -> Self {
        Self(AudiophileAuxOutputProtocol::create_level_parameters())
    }
}

impl AvcLevelCtlOperation<AudiophileAuxOutputProtocol> for AuxOutputCtl {
    const LEVEL_NAME: &'static str = "aux-ouput-gain";
    const PORT_LABELS: &'static [&'static str] = &["aux-output-1", "aux-output-2"];

    fn state(&self) -> &AvcLevelParameters {
        &self.0
    }

    fn state_mut(&mut self) -> &mut AvcLevelParameters {
        &mut self.0
    }
}

#[derive(Debug)]
struct PhysOutputCtl(AvcLevelParameters, AvcSelectorParameters);

impl Default for PhysOutputCtl {
    fn default() -> Self {
        Self(
            AudiophilePhysOutputProtocol::create_level_parameters(),
            AudiophilePhysOutputProtocol::create_selector_parameters(),
        )
    }
}

impl AvcLevelCtlOperation<AudiophilePhysOutputProtocol> for PhysOutputCtl {
    const LEVEL_NAME: &'static str = "output-volume";
    const PORT_LABELS: &'static [&'static str] = &[
        "analog-output-1",
        "analog-output-2",
        "analog-output-3",
        "analog-output-4",
        "digital-output-1",
        "digital-output-2",
    ];

    fn state(&self) -> &AvcLevelParameters {
        &self.0
    }

    fn state_mut(&mut self) -> &mut AvcLevelParameters {
        &mut self.0
    }
}

impl AvcSelectorCtlOperation<AudiophilePhysOutputProtocol> for PhysOutputCtl {
    const SELECTOR_NAME: &'static str = "output-source";
    const SELECTOR_LABELS: &'static [&'static str] = &[
        "analog-output-1/2",
        "analog-output-3/4",
        "analog-output-5/6",
    ];
    const ITEM_LABELS: &'static [&'static str] = &["mixer-output", "aux-output-1/2"];

    fn state(&self) -> &AvcSelectorParameters {
        &self.1
    }

    fn state_mut(&mut self) -> &mut AvcSelectorParameters {
        &mut self.1
    }
}

#[derive(Debug)]
struct HeadphoneCtl(AvcLevelParameters, AvcSelectorParameters);

impl Default for HeadphoneCtl {
    fn default() -> Self {
        Self(
            AudiophileHeadphoneProtocol::create_level_parameters(),
            AudiophileHeadphoneProtocol::create_selector_parameters(),
        )
    }
}

impl AvcLevelCtlOperation<AudiophileHeadphoneProtocol> for HeadphoneCtl {
    const LEVEL_NAME: &'static str = "headphone-volume";
    const PORT_LABELS: &'static [&'static str] = &["headphone-1", "headphone-2"];

    fn state(&self) -> &AvcLevelParameters {
        &self.0
    }

    fn state_mut(&mut self) -> &mut AvcLevelParameters {
        &mut self.0
    }
}

impl AvcSelectorCtlOperation<AudiophileHeadphoneProtocol> for HeadphoneCtl {
    const SELECTOR_NAME: &'static str = "headphone-source";
    const SELECTOR_LABELS: &'static [&'static str] = &["headphone-1/2"];
    const ITEM_LABELS: &'static [&'static str] = &[
        "mixer-output-1/2",
        "mixer-output-3/4",
        "mixer-output-5/6",
        "aux-output-1/2",
    ];

    fn state(&self) -> &AvcSelectorParameters {
        &self.1
    }

    fn state_mut(&mut self) -> &mut AvcSelectorParameters {
        &mut self.1
    }
}

#[derive(Debug)]
struct MixerCtl(MaudioNormalMixerParameters);

impl Default for MixerCtl {
    fn default() -> Self {
        Self(AudiophileMixerProtocol::create_mixer_parameters())
    }
}

impl MaudioNormalMixerCtlOperation<AudiophileMixerProtocol> for MixerCtl {
    const MIXER_NAME: &'static str = "mixer-source";

    const DST_LABELS: &'static [&'static str] = &["mixer-1/2", "mixer-3/4", "mixer-5/6"];

    const SRC_LABELS: &'static [&'static str] = &[
        "analog-input-1/2",
        "digital-input-1/2",
        "stream-input-1/2",
        "stream-input-3/4",
        "stream-input-5/6",
    ];

    fn state(&self) -> &MaudioNormalMixerParameters {
        &self.0
    }

    fn state_mut(&mut self) -> &mut MaudioNormalMixerParameters {
        &mut self.0
    }
}

impl CtlModel<(SndUnit, FwNode)> for AudiophileModel {
    fn cache(&mut self, unit: &mut (SndUnit, FwNode)) -> Result<(), Error> {
        self.avc.bind(&unit.1)?;

        self.clk_ctl.cache_freq(&self.avc, FCP_TIMEOUT_MS)?;
        self.clk_ctl.cache_src(&self.avc, FCP_TIMEOUT_MS)?;
        self.phys_input_ctl
            .cache_levels(&self.avc, FCP_TIMEOUT_MS)?;
        self.aux_src_ctl.cache_levels(&self.avc, FCP_TIMEOUT_MS)?;
        self.aux_output_ctl
            .cache_levels(&self.avc, FCP_TIMEOUT_MS)?;
        self.phys_output_ctl
            .cache_levels(&self.avc, FCP_TIMEOUT_MS)?;
        self.hp_ctl.cache_levels(&self.avc, FCP_TIMEOUT_MS)?;
        self.phys_input_ctl
            .cache_balances(&self.avc, FCP_TIMEOUT_MS)?;
        self.phys_output_ctl
            .cache_selectors(&self.avc, FCP_TIMEOUT_MS)?;
        self.hp_ctl.cache_selectors(&self.avc, FCP_TIMEOUT_MS)?;
        self.mixer_ctl.cache(&self.avc, FCP_TIMEOUT_MS)?;
        self.meter_ctl
            .cache_meter(&self.req, &unit.1, &self.avc, TIMEOUT_MS)?;

        Ok(())
    }
    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.clk_ctl
            .load_freq(card_cntr)
            .map(|mut elem_id_list| self.clk_ctl.0.append(&mut elem_id_list))?;

        self.clk_ctl
            .load_src(card_cntr)
            .map(|mut elem_id_list| self.clk_ctl.0.append(&mut elem_id_list))?;

        self.meter_ctl
            .load_meter(card_cntr)
            .map(|mut elem_id_list| self.meter_ctl.0.append(&mut elem_id_list))?;

        self.phys_input_ctl.load_level(card_cntr)?;
        self.phys_input_ctl.load_balance(card_cntr)?;
        self.aux_src_ctl.load_level(card_cntr)?;
        self.aux_output_ctl.load_level(card_cntr)?;
        self.phys_output_ctl.load_level(card_cntr)?;
        self.phys_output_ctl.load_selector(card_cntr)?;
        self.hp_ctl.load_level(card_cntr)?;
        self.hp_ctl.load_selector(card_cntr)?;
        self.mixer_ctl.load_src_state(card_cntr)?;

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
        } else if self.meter_ctl.read_meter(elem_id, elem_value)? {
            Ok(true)
        } else if self.phys_input_ctl.read_levels(elem_id, elem_value)? {
            Ok(true)
        } else if self.phys_input_ctl.read_balances(elem_id, elem_value)? {
            Ok(true)
        } else if self.aux_src_ctl.read_levels(elem_id, elem_value)? {
            Ok(true)
        } else if self.aux_output_ctl.read_levels(elem_id, elem_value)? {
            Ok(true)
        } else if self.phys_output_ctl.read_levels(elem_id, elem_value)? {
            Ok(true)
        } else if self.phys_output_ctl.read_selectors(elem_id, elem_value)? {
            Ok(true)
        } else if self.hp_ctl.read_levels(elem_id, elem_value)? {
            Ok(true)
        } else if self.hp_ctl.read_selectors(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_ctl.read_src_state(elem_id, elem_value)? {
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
        } else if self.clk_ctl.write_src(
            &mut unit.0,
            &self.avc,
            elem_id,
            old,
            new,
            FCP_TIMEOUT_MS * 3,
        )? {
            Ok(true)
        } else if self
            .meter_ctl
            .write_meter(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .phys_input_ctl
            .write_level(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .phys_input_ctl
            .write_balance(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .aux_src_ctl
            .write_level(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .aux_output_ctl
            .write_level(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .phys_output_ctl
            .write_level(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else if self.phys_output_ctl.write_selector(
            &self.avc,
            elem_id,
            old,
            new,
            FCP_TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self
            .hp_ctl
            .write_level(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)?
        {
            Ok(false)
        } else if self
            .hp_ctl
            .write_selector(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .mixer_ctl
            .write_src_state(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl MeasureModel<(SndUnit, FwNode)> for AudiophileModel {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.meter_ctl.0);
    }

    fn measure_states(&mut self, unit: &mut (SndUnit, FwNode)) -> Result<(), Error> {
        self.meter_ctl
            .cache_meter(&self.req, &unit.1, &self.avc, TIMEOUT_MS)
    }
}

impl NotifyModel<(SndUnit, FwNode), bool> for AudiophileModel {
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

        let mut ctl = PhysInputCtl::default();
        let error = ctl.load_level(&mut card_cntr).unwrap_err();
        assert_eq!(error.kind::<CardError>(), Some(CardError::Failed));

        let mut ctl = AuxSourceCtl::default();
        let error = ctl.load_level(&mut card_cntr).unwrap_err();
        assert_eq!(error.kind::<CardError>(), Some(CardError::Failed));

        let mut ctl = AuxOutputCtl::default();
        let error = ctl.load_level(&mut card_cntr).unwrap_err();
        assert_eq!(error.kind::<CardError>(), Some(CardError::Failed));

        let mut ctl = PhysOutputCtl::default();
        let error = ctl.load_level(&mut card_cntr).unwrap_err();
        assert_eq!(error.kind::<CardError>(), Some(CardError::Failed));

        let mut ctl = HeadphoneCtl::default();
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
    }

    #[test]
    fn test_mixer_ctl_definition() {
        let mut card_cntr = CardCntr::default();

        let mut ctl = MixerCtl::default();
        let error = ctl.load_src_state(&mut card_cntr).unwrap_err();
        assert_eq!(error.kind::<CardError>(), Some(CardError::Failed));
    }
}
