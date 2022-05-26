// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

use {
    super::*,
    bebob_protocols::{presonus::firebox::*, *},
};

#[derive(Default)]
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

#[derive(Default)]
struct ClkCtl(Vec<ElemId>);

impl MediaClkFreqCtlOperation<FireboxClkProtocol> for ClkCtl {}

impl SamplingClkSrcCtlOperation<FireboxClkProtocol> for ClkCtl {
    const SRC_LABELS: &'static [&'static str] = &["Internal", "S/PDIF"];
}

#[derive(Default)]
struct PhysOutputCtl;

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
}

impl AvcMuteCtlOperation<FireboxPhysOutputProtocol> for PhysOutputCtl {
    const MUTE_NAME: &'static str = "phys-output-mute";
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
}

#[derive(Default)]
struct HeadphoneCtl;

impl AvcLevelCtlOperation<FireboxHeadphoneProtocol> for HeadphoneCtl {
    const LEVEL_NAME: &'static str = "headphone-gain";
    const PORT_LABELS: &'static [&'static str] = &["headphone-1", "headphone-2"];
}

impl AvcMuteCtlOperation<FireboxHeadphoneProtocol> for HeadphoneCtl {
    const MUTE_NAME: &'static str = "headphone-mute";
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
}

#[derive(Default)]
struct MixerPhysSrcCtl;

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
}

impl AvcLrBalanceCtlOperation<FireboxMixerPhysSourceProtocol> for MixerPhysSrcCtl {
    const BALANCE_NAME: &'static str = "phys-input-balance";
}

impl AvcMuteCtlOperation<FireboxMixerPhysSourceProtocol> for MixerPhysSrcCtl {
    const MUTE_NAME: &'static str = "mixer-phys-source-mute";
}

#[derive(Default)]
struct MixerStreamSrcCtl;

impl AvcLevelCtlOperation<FireboxMixerStreamSourceProtocol> for MixerStreamSrcCtl {
    const LEVEL_NAME: &'static str = "mixer-stream-source-gain";
    const PORT_LABELS: &'static [&'static str] = &["stream-input-1/2"];
}

impl AvcMuteCtlOperation<FireboxMixerStreamSourceProtocol> for MixerStreamSrcCtl {
    const MUTE_NAME: &'static str = "mixer-stream-source-mute";
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
}

#[derive(Default)]
struct MixerOutputCtl;

impl AvcLevelCtlOperation<FireboxMixerOutputProtocol> for MixerOutputCtl {
    const LEVEL_NAME: &'static str = "mixer-output-volume";
    const PORT_LABELS: &'static [&'static str] = &["mixer-output-1", "mixer-output-2"];
}

impl AvcLrBalanceCtlOperation<FireboxMixerOutputProtocol> for MixerOutputCtl {
    const BALANCE_NAME: &'static str = "phys-input-balance";
}

impl AvcMuteCtlOperation<FireboxMixerOutputProtocol> for MixerOutputCtl {
    const MUTE_NAME: &'static str = "mixer-phys-source-mute";
}

#[derive(Default)]
struct AnalogInputCtl;

impl SwitchCtlOperation<FireboxAnalogInputProtocol> for AnalogInputCtl {
    const SWITCH_NAME: &'static str = "analog-input-boost";
}

impl CtlModel<(SndUnit, FwNode)> for FireboxModel {
    fn load(
        &mut self,
        unit: &mut (SndUnit, FwNode),
        card_cntr: &mut CardCntr,
    ) -> Result<(), Error> {
        self.avc.as_ref().bind(&unit.1)?;

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
        self.mixer_out_ctl.load_mute(card_cntr)?;

        self.analog_in_ctl.load_switch(card_cntr)?;

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
            .clk_ctl
            .read_src(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .phys_out_ctl
            .read_level(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .phys_out_ctl
            .read_mute(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .phys_out_ctl
            .read_selector(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .headphone_ctl
            .read_level(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .headphone_ctl
            .read_mute(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else if self.headphone_ctl.read_selector(
            &self.avc,
            elem_id,
            elem_value,
            FCP_TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self.mixer_phys_src_ctl.read_level(
            &self.avc,
            elem_id,
            elem_value,
            FCP_TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self.mixer_phys_src_ctl.read_balance(
            &self.avc,
            elem_id,
            elem_value,
            FCP_TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self.mixer_phys_src_ctl.read_mute(
            &self.avc,
            elem_id,
            elem_value,
            FCP_TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self.mixer_stream_src_ctl.read_level(
            &self.avc,
            elem_id,
            elem_value,
            FCP_TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self.mixer_stream_src_ctl.read_mute(
            &self.avc,
            elem_id,
            elem_value,
            FCP_TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self.mixer_stream_src_ctl.read_selector(
            &self.avc,
            elem_id,
            elem_value,
            FCP_TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self
            .mixer_out_ctl
            .read_level(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .mixer_out_ctl
            .read_balance(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .mixer_out_ctl
            .read_mute(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .analog_in_ctl
            .read_switch(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)?
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
            .headphone_ctl
            .write_level(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)?
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
        } else if self.mixer_phys_src_ctl.write_level(
            &self.avc,
            elem_id,
            old,
            new,
            FCP_TIMEOUT_MS,
        )? {
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
            .write_switch(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)?
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

trait SwitchCtlOperation<T: AvcSelectorOperation> {
    const SWITCH_NAME: &'static str;

    const CH_COUNT: usize = T::FUNC_BLOCK_ID_LIST.len();

    fn load_switch(&self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::SWITCH_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, 1, Self::CH_COUNT, true)
            .map(|_| ())
    }

    fn read_switch(
        &self,
        avc: &BebobAvc,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        if elem_id.get_name().as_str() == Self::SWITCH_NAME {
            ElemValueAccessor::<bool>::set_vals(elem_value, Self::CH_COUNT, |idx| {
                T::read_selector(avc, idx, timeout_ms).map(|val| val > 0)
            })
            .map(|_| true)
        } else {
            Ok(false)
        }
    }

    fn write_switch(
        &self,
        avc: &BebobAvc,
        elem_id: &ElemId,
        old: &ElemValue,
        new: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        if elem_id.get_name().as_str() == Self::SWITCH_NAME {
            ElemValueAccessor::<bool>::get_vals(new, old, Self::CH_COUNT, |idx, val| {
                T::write_selector(avc, idx, val as usize, timeout_ms)
            })
            .map(|_| true)
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
    fn test_clk_ctl_definition() {
        let mut card_cntr = CardCntr::new();
        let mut ctl = ClkCtl::default();

        let error = ctl.load_freq(&mut card_cntr).unwrap_err();
        assert_eq!(error.kind::<CardError>(), Some(CardError::Failed));

        let error = ctl.load_src(&mut card_cntr).unwrap_err();
        assert_eq!(error.kind::<CardError>(), Some(CardError::Failed));
    }

    #[test]
    fn test_level_ctl_definition() {
        let mut card_cntr = CardCntr::new();

        let ctl = PhysOutputCtl::default();
        let error = ctl.load_level(&mut card_cntr).unwrap_err();
        assert_eq!(error.kind::<CardError>(), Some(CardError::Failed));

        let ctl = HeadphoneCtl::default();
        let error = ctl.load_level(&mut card_cntr).unwrap_err();
        assert_eq!(error.kind::<CardError>(), Some(CardError::Failed));

        let ctl = MixerPhysSrcCtl::default();
        let error = ctl.load_level(&mut card_cntr).unwrap_err();
        assert_eq!(error.kind::<CardError>(), Some(CardError::Failed));

        let ctl = MixerStreamSrcCtl::default();
        let error = ctl.load_level(&mut card_cntr).unwrap_err();
        assert_eq!(error.kind::<CardError>(), Some(CardError::Failed));

        let ctl = MixerOutputCtl::default();
        let error = ctl.load_level(&mut card_cntr).unwrap_err();
        assert_eq!(error.kind::<CardError>(), Some(CardError::Failed));
    }

    #[test]
    fn test_selector_ctl_definition() {
        let mut card_cntr = CardCntr::new();

        let ctl = PhysOutputCtl::default();
        let error = ctl.load_selector(&mut card_cntr).unwrap_err();
        assert_eq!(error.kind::<CardError>(), Some(CardError::Failed));

        let ctl = HeadphoneCtl::default();
        let error = ctl.load_selector(&mut card_cntr).unwrap_err();
        assert_eq!(error.kind::<CardError>(), Some(CardError::Failed));

        let ctl = MixerStreamSrcCtl::default();
        let error = ctl.load_selector(&mut card_cntr).unwrap_err();
        assert_eq!(error.kind::<CardError>(), Some(CardError::Failed));
    }
}
