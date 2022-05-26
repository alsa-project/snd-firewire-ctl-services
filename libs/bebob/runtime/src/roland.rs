// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

use {
    super::{common_ctls::*, *},
    bebob_protocols::{roland::*, *},
};

pub type Fa66Model = FaModel<Fa66MixerAnalogSourceProtocol>;
pub type Fa101Model = FaModel<Fa101MixerAnalogSourceProtocol>;

#[derive(Default)]
pub struct FaModel<T: AvcLevelOperation + AvcLrBalanceOperation + Default> {
    avc: BebobAvc,
    clk_ctl: ClkCtl,
    analog_in_ctl: MixerAnalogSourceCtl<T>,
}

const FCP_TIMEOUT_MS: u32 = 100;

// Read only, configured by hardware only.
#[derive(Default)]
struct ClkCtl(Vec<ElemId>);

impl MediaClkFreqCtlOperation<FaClkProtocol> for ClkCtl {
    fn write_freq(
        &self,
        _: &mut SndUnit,
        _: &BebobAvc,
        elem_id: &ElemId,
        _: &ElemValue,
        _: &ElemValue,
        _: u32,
    ) -> Result<bool, Error> {
        if elem_id.get_name().as_str() == CLK_RATE_NAME {
            Err(Error::new(
                FileError::Nxio,
                "Sampling rate is immutable from software",
            ))
        } else {
            Ok(false)
        }
    }
}

#[derive(Default)]
struct MixerAnalogSourceCtl<O: AvcLevelOperation + Default>(O);

impl AvcLevelCtlOperation<Fa66MixerAnalogSourceProtocol>
    for MixerAnalogSourceCtl<Fa66MixerAnalogSourceProtocol>
{
    const LEVEL_NAME: &'static str = "mixer-source-gain";

    const PORT_LABELS: &'static [&'static str] = &[
        "analog-input-1",
        "analog-input-2",
        "analog-input-3",
        "analog-input-4",
        "analog-input-5",
        "analog-input-6",
    ];
}

impl AvcLrBalanceCtlOperation<Fa66MixerAnalogSourceProtocol>
    for MixerAnalogSourceCtl<Fa66MixerAnalogSourceProtocol>
{
    const BALANCE_NAME: &'static str = "mixer-source-balance";
}

impl AvcLevelCtlOperation<Fa101MixerAnalogSourceProtocol>
    for MixerAnalogSourceCtl<Fa101MixerAnalogSourceProtocol>
{
    const LEVEL_NAME: &'static str = "mixer-source-gain";

    const PORT_LABELS: &'static [&'static str] = &[
        "analog-input-1",
        "analog-input-2",
        "analog-input-3",
        "analog-input-4",
        "analog-input-5",
        "analog-input-6",
        "analog-input-7",
        "analog-input-8",
        "analog-input-9",
        "analog-input-10",
    ];
}

impl AvcLrBalanceCtlOperation<Fa101MixerAnalogSourceProtocol>
    for MixerAnalogSourceCtl<Fa101MixerAnalogSourceProtocol>
{
    const BALANCE_NAME: &'static str = "mixer-source-balance";
}

impl CtlModel<(SndUnit, FwNode)> for FaModel<Fa66MixerAnalogSourceProtocol> {
    fn load(
        &mut self,
        unit: &mut (SndUnit, FwNode),
        card_cntr: &mut CardCntr,
    ) -> Result<(), Error> {
        self.avc.as_ref().bind(&unit.1)?;

        self.clk_ctl.load_freq(card_cntr)?;
        self.analog_in_ctl.load_level(card_cntr)?;
        self.analog_in_ctl.load_balance(card_cntr)?;

        Ok(())
    }

    fn read(
        &mut self,
        _: &mut (SndUnit, FwNode),
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self
            .clk_ctl
            .read_freq(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .analog_in_ctl
            .read_level(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .analog_in_ctl
            .read_balance(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)?
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
        } else if self
            .analog_in_ctl
            .write_level(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .analog_in_ctl
            .write_balance(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl CtlModel<(SndUnit, FwNode)> for FaModel<Fa101MixerAnalogSourceProtocol> {
    fn load(
        &mut self,
        unit: &mut (SndUnit, FwNode),
        card_cntr: &mut CardCntr,
    ) -> Result<(), Error> {
        self.avc.as_ref().bind(&unit.1)?;

        self.clk_ctl.load_freq(card_cntr)?;
        self.analog_in_ctl.load_level(card_cntr)?;
        self.analog_in_ctl.load_balance(card_cntr)?;

        Ok(())
    }

    fn read(
        &mut self,
        _: &mut (SndUnit, FwNode),
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self
            .clk_ctl
            .read_freq(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .analog_in_ctl
            .read_level(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .analog_in_ctl
            .read_balance(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)?
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
        } else if self
            .analog_in_ctl
            .write_level(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .analog_in_ctl
            .write_balance(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use alsactl::CardError;

    #[test]
    fn test_level_ctl_definition() {
        let mut card_cntr = CardCntr::new();

        let ctl = MixerAnalogSourceCtl::<Fa66MixerAnalogSourceProtocol>::default();
        let error = ctl.load_level(&mut card_cntr).unwrap_err();
        assert_eq!(error.kind::<CardError>(), Some(CardError::Failed));

        let ctl = MixerAnalogSourceCtl::<Fa66MixerAnalogSourceProtocol>::default();
        let error = ctl.load_level(&mut card_cntr).unwrap_err();
        assert_eq!(error.kind::<CardError>(), Some(CardError::Failed));
    }

    #[test]
    fn test_clk_ctl_definition() {
        let mut card_cntr = CardCntr::new();
        let mut ctl = ClkCtl::default();

        let error = ctl.load_freq(&mut card_cntr).unwrap_err();
        assert_eq!(error.kind::<CardError>(), Some(CardError::Failed));
    }
}
