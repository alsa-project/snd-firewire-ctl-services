// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use super::*;

#[derive(Default, Debug)]
pub struct OzonicModel {
    avc: BebobAvc,
    req: FwReq,
    clk_ctl: ClkCtl,
    meter_ctl: MeterCtl,
    phys_input_ctl: PhysInputCtl,
    stream_input_ctl: StreamInputCtl,
    mixer_ctl: MixerCtl,
}

const FCP_TIMEOUT_MS: u32 = 100;
const TIMEOUT_MS: u32 = 50;

#[derive(Default, Debug)]
struct ClkCtl(Vec<ElemId>, MediaClockParameters, SamplingClockParameters);

impl MediaClkFreqCtlOperation<OzonicClkProtocol> for ClkCtl {
    fn state(&self) -> &MediaClockParameters {
        &self.1
    }

    fn state_mut(&mut self) -> &mut MediaClockParameters {
        &mut self.1
    }
}

impl SamplingClkSrcCtlOperation<OzonicClkProtocol> for ClkCtl {
    const SRC_LABELS: &'static [&'static str] = &["Internal"];

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
        Self(Default::default(), OzonicMeterProtocol::create_meter())
    }
}

impl MaudioNormalMeterCtlOperation<OzonicMeterProtocol> for MeterCtl {
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
            OzonicPhysInputProtocol::create_level_parameters(),
            OzonicPhysInputProtocol::create_lr_balance_parameters(),
        )
    }
}

impl AvcLevelCtlOperation<OzonicPhysInputProtocol> for PhysInputCtl {
    const LEVEL_NAME: &'static str = "phys-input-gain";
    const PORT_LABELS: &'static [&'static str] = &[
        "analog-input-1",
        "analog-input-2",
        "analog-input-3",
        "analog-input-4",
    ];

    fn state(&self) -> &AvcLevelParameters {
        &self.0
    }

    fn state_mut(&mut self) -> &mut AvcLevelParameters {
        &mut self.0
    }
}

impl AvcLrBalanceCtlOperation<OzonicPhysInputProtocol> for PhysInputCtl {
    const BALANCE_NAME: &'static str = "phys-input-balance";

    fn state(&self) -> &AvcLrBalanceParameters {
        &self.1
    }

    fn state_mut(&mut self) -> &mut AvcLrBalanceParameters {
        &mut self.1
    }
}

#[derive(Debug)]
struct StreamInputCtl(AvcLevelParameters);

impl Default for StreamInputCtl {
    fn default() -> Self {
        Self(OzonicStreamInputProtocol::create_level_parameters())
    }
}

impl AvcLevelCtlOperation<OzonicStreamInputProtocol> for StreamInputCtl {
    const LEVEL_NAME: &'static str = "stream-input-gain";
    const PORT_LABELS: &'static [&'static str] = &[
        "stream-input-1",
        "stream-input-2",
        "stream-input-3",
        "stream-input-4",
    ];

    fn state(&self) -> &AvcLevelParameters {
        &self.0
    }

    fn state_mut(&mut self) -> &mut AvcLevelParameters {
        &mut self.0
    }
}

#[derive(Debug)]
struct MixerCtl(MaudioNormalMixerParameters);

impl Default for MixerCtl {
    fn default() -> Self {
        Self(OzonicMixerProtocol::create_mixer_parameters())
    }
}

impl MaudioNormalMixerCtlOperation<OzonicMixerProtocol> for MixerCtl {
    const MIXER_NAME: &'static str = "mixer-source";
    const DST_LABELS: &'static [&'static str] = &["mixer-1/2", "mixer-3/4"];
    const SRC_LABELS: &'static [&'static str] = &[
        "analog-input-1/2",
        "digital-input-1/2",
        "stream-input-1/2",
        "stream-input-3/4",
    ];

    fn state(&self) -> &MaudioNormalMixerParameters {
        &self.0
    }

    fn state_mut(&mut self) -> &mut MaudioNormalMixerParameters {
        &mut self.0
    }
}

impl OzonicModel {
    pub fn cache(&mut self, unit: &mut (SndUnit, FwNode)) -> Result<(), Error> {
        self.avc.bind(&unit.1)?;

        self.clk_ctl.cache_freq(&self.avc, FCP_TIMEOUT_MS)?;
        self.clk_ctl.cache_src(&self.avc, FCP_TIMEOUT_MS)?;
        self.phys_input_ctl
            .cache_levels(&self.avc, FCP_TIMEOUT_MS)?;
        self.stream_input_ctl
            .cache_levels(&self.avc, FCP_TIMEOUT_MS)?;
        self.phys_input_ctl
            .cache_balances(&self.avc, FCP_TIMEOUT_MS)?;
        self.mixer_ctl.cache(&self.avc, FCP_TIMEOUT_MS)?;

        self.meter_ctl
            .cache_meter(&self.req, &unit.1, &self.avc, TIMEOUT_MS)?;

        Ok(())
    }
}

impl CtlModel<(SndUnit, FwNode)> for OzonicModel {
    fn load(
        &mut self,
        _: &mut (SndUnit, FwNode),
        card_cntr: &mut CardCntr,
    ) -> Result<(), Error> {
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
        self.stream_input_ctl.load_level(card_cntr)?;
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
        } else if self.stream_input_ctl.read_levels(elem_id, elem_value)? {
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
            FCP_TIMEOUT_MS * 3,
        )? {
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
            .stream_input_ctl
            .write_level(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)?
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

impl MeasureModel<(SndUnit, FwNode)> for OzonicModel {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.meter_ctl.0);
    }

    fn measure_states(&mut self, unit: &mut (SndUnit, FwNode)) -> Result<(), Error> {
        self.meter_ctl
            .cache_meter(&self.req, &unit.1, &self.avc, TIMEOUT_MS)
    }

    fn measure_elem(
        &mut self,
        _: &(SndUnit, FwNode),
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        self.meter_ctl.read_meter(elem_id, elem_value)
    }
}

impl NotifyModel<(SndUnit, FwNode), bool> for OzonicModel {
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

        let mut ctl = StreamInputCtl::default();
        let error = ctl.load_level(&mut card_cntr).unwrap_err();
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
