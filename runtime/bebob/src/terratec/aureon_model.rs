// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

use {
    super::{common_ctls::*, *},
    protocols::{terratec::aureon::*, *},
};

#[derive(Default)]
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

#[derive(Default)]
struct PhysInputCtl;

impl AvcLevelCtlOperation<AureonPhysInputProtocol> for PhysInputCtl {
    const LEVEL_NAME: &'static str = "analog-input-gain";
    const PORT_LABELS: &'static [&'static str] = &["analog-input-1/2", "analog-input-3/4"];
}

#[derive(Default)]
struct MonitorSourceCtl;

impl AvcSelectorCtlOperation<AureonMonitorSourceProtocol> for MonitorSourceCtl {
    const SELECTOR_NAME: &'static str = "monitor-source";
    const SELECTOR_LABELS: &'static [&'static str] = &["monitor-source-1/2"];
    const ITEM_LABELS: &'static [&'static str] = &[
        "analog-input-1/2",
        "analog-input-3/4",
        "analog-input-5/6",
        "digital-input-1/2",
    ];
}

#[derive(Default)]
struct MonitorOutputCtl;

impl AvcLevelCtlOperation<AureonMonitorOutputProtocol> for MonitorOutputCtl {
    const LEVEL_NAME: &'static str = "monitor-output-volume";
    const PORT_LABELS: &'static [&'static str] = &["monitor-output-1/2"];
}

impl AvcMuteCtlOperation<AureonMonitorOutputProtocol> for MonitorOutputCtl {
    const MUTE_NAME: &'static str = "monitor-output-mute";
}

#[derive(Default)]
struct MixerOutputCtl;

#[derive(Default)]
struct SpdifOutputCtl;

impl AvcSelectorCtlOperation<AureonSpdifOutputProtocol> for SpdifOutputCtl {
    const SELECTOR_NAME: &'static str = "spdif-output-source";
    const SELECTOR_LABELS: &'static [&'static str] = &["spdif-output-1/2"];
    const ITEM_LABELS: &'static [&'static str] = &["mixer-output-1/2", "stream-input-9/10"];
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
}

impl AvcMuteCtlOperation<AureonMixerOutputProtocol> for MixerOutputCtl {
    const MUTE_NAME: &'static str = "mixer-output-mute";
}

impl CtlModel<(SndUnit, FwNode)> for AureonModel {
    fn load(
        &mut self,
        unit: &mut (SndUnit, FwNode),
        card_cntr: &mut CardCntr,
    ) -> Result<(), Error> {
        self.avc.bind(&unit.1)?;

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
        if self
            .clk_ctl
            .read_freq(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .phys_in_ctl
            .read_level(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .mon_src_ctl
            .read_selector(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .mon_out_ctl
            .read_level(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .mon_out_ctl
            .read_mute(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .mixer_out_ctl
            .read_level(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .mixer_out_ctl
            .read_mute(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else if self.spdif_out_ctl.read_selector(
            &self.avc,
            elem_id,
            elem_value,
            FCP_TIMEOUT_MS,
        )? {
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

    fn parse_notification(&mut self, _: &mut (SndUnit, FwNode), _: &bool) -> Result<(), Error> {
        Ok(())
    }

    fn read_notified_elem(
        &mut self,
        _: &(SndUnit, FwNode),
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        self.clk_ctl
            .read_freq(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)
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

        let ctl = PhysInputCtl::default();
        let error = ctl.load_level(&mut card_cntr).unwrap_err();
        assert_eq!(error.kind::<CardError>(), Some(CardError::Failed));

        let ctl = MonitorOutputCtl::default();
        let error = ctl.load_level(&mut card_cntr).unwrap_err();
        assert_eq!(error.kind::<CardError>(), Some(CardError::Failed));

        let ctl = MixerOutputCtl::default();
        let error = ctl.load_level(&mut card_cntr).unwrap_err();
        assert_eq!(error.kind::<CardError>(), Some(CardError::Failed));
    }

    #[test]
    fn test_selector_ctl_definition() {
        let mut card_cntr = CardCntr::default();

        let ctl = MonitorSourceCtl::default();
        let error = ctl.load_selector(&mut card_cntr).unwrap_err();
        assert_eq!(error.kind::<CardError>(), Some(CardError::Failed));

        let ctl = SpdifOutputCtl::default();
        let error = ctl.load_selector(&mut card_cntr).unwrap_err();
        assert_eq!(error.kind::<CardError>(), Some(CardError::Failed));
    }
}
