// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use {
    super::{common_ctls::*, *},
    protocols::{stanton::*, *},
};

const FCP_TIMEOUT_MS: u32 = 100;

#[derive(Default, Debug)]
pub struct ScratchampModel {
    avc: BebobAvc,
    clk_ctl: ClkCtl,
    output_ctl: ScratchampOutputCtl,
    headphone_ctl: ScratchampHeadphoneCtl,
}

#[derive(Default, Debug)]
struct ClkCtl(Vec<ElemId>, MediaClockParameters, SamplingClockParameters);

impl MediaClkFreqCtlOperation<ScratchampClkProtocol> for ClkCtl {
    fn state(&self) -> &MediaClockParameters {
        &self.1
    }

    fn state_mut(&mut self) -> &mut MediaClockParameters {
        &mut self.1
    }
}

impl SamplingClkSrcCtlOperation<ScratchampClkProtocol> for ClkCtl {
    const SRC_LABELS: &'static [&'static str] = &["Internal"];

    fn state(&self) -> &SamplingClockParameters {
        &self.2
    }

    fn state_mut(&mut self) -> &mut SamplingClockParameters {
        &mut self.2
    }
}

#[derive(Debug)]
struct ScratchampOutputCtl(AvcLevelParameters);

impl Default for ScratchampOutputCtl {
    fn default() -> Self {
        Self(ScratchampOutputProtocol::create_level_parameters())
    }
}

impl AvcLevelCtlOperation<ScratchampOutputProtocol> for ScratchampOutputCtl {
    const LEVEL_NAME: &'static str = OUT_VOL_NAME;
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

#[derive(Debug)]
struct ScratchampHeadphoneCtl(AvcLevelParameters);

impl Default for ScratchampHeadphoneCtl {
    fn default() -> Self {
        Self(ScratchampHeadphoneProtocol::create_level_parameters())
    }
}

impl AvcLevelCtlOperation<ScratchampHeadphoneProtocol> for ScratchampHeadphoneCtl {
    const LEVEL_NAME: &'static str = "headphone-volume";
    const PORT_LABELS: &'static [&'static str] = &["headphone-1", "headphone-2"];

    fn state(&self) -> &AvcLevelParameters {
        &self.0
    }

    fn state_mut(&mut self) -> &mut AvcLevelParameters {
        &mut self.0
    }
}

impl CtlModel<(SndUnit, FwNode)> for ScratchampModel {
    fn cache(&mut self, unit: &mut (SndUnit, FwNode)) -> Result<(), Error> {
        self.avc.bind(&unit.1)?;

        self.clk_ctl.cache_freq(&self.avc, FCP_TIMEOUT_MS)?;
        self.clk_ctl.cache_src(&self.avc, FCP_TIMEOUT_MS)?;
        self.output_ctl.cache_levels(&self.avc, FCP_TIMEOUT_MS)?;
        self.headphone_ctl.cache_levels(&self.avc, FCP_TIMEOUT_MS)?;

        Ok(())
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.clk_ctl
            .load_freq(card_cntr)
            .map(|mut elem_id_list| self.clk_ctl.0.append(&mut elem_id_list))?;

        self.clk_ctl
            .load_src(card_cntr)
            .map(|mut elem_id_list| self.clk_ctl.0.append(&mut elem_id_list))?;

        self.output_ctl.load_level(card_cntr)?;
        self.headphone_ctl.load_level(card_cntr)?;

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
        } else if self.output_ctl.read_levels(elem_id, elem_value)? {
            Ok(true)
        } else if self.headphone_ctl.read_levels(elem_id, elem_value)? {
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
            FCP_TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self
            .output_ctl
            .write_level(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .headphone_ctl
            .write_level(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl NotifyModel<(SndUnit, FwNode), bool> for ScratchampModel {
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
    }

    #[test]
    fn test_level_ctl_definition() {
        let mut card_cntr = CardCntr::default();

        let mut ctl = ScratchampOutputCtl::default();
        let error = ctl.load_level(&mut card_cntr).unwrap_err();
        assert_eq!(error.kind::<CardError>(), Some(CardError::Failed));

        let mut ctl = ScratchampHeadphoneCtl::default();
        let error = ctl.load_level(&mut card_cntr).unwrap_err();
        assert_eq!(error.kind::<CardError>(), Some(CardError::Failed));
    }
}
