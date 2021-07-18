// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use glib::Error;

use hinawa::{FwFcpExt, FwReq};
use hinawa::{SndUnit, SndUnitExt};

use alsactl::{ElemId, ElemValue};

use core::card_cntr::*;

use bebob_protocols::{*, maudio::normal::*};

use crate::common_ctls::*;

use super::*;
use super::normal_ctls::MixerCtl;

pub struct SoloModel<'a>{
    avc: BebobAvc,
    clk_ctl: ClkCtl,
    req: FwReq,
    meter_ctl: MeterCtl,
    phys_input_ctl: PhysInputCtl,
    stream_input_ctl: StreamInputCtl,
    spdif_output_ctl: SpdifOutputCtl,
    mixer_ctl: MixerCtl<'a>,
}

const FCP_TIMEOUT_MS: u32 = 100;
const TIMEOUT_MS: u32 = 50;

#[derive(Default)]
struct ClkCtl(Vec<ElemId>);

impl MediaClkFreqCtlOperation<SoloClkProtocol> for ClkCtl {}

impl SamplingClkSrcCtlOperation<SoloClkProtocol> for ClkCtl {
    const SRC_LABELS: &'static [&'static str] = &["Internal", "S/PDIF"];
}

struct MeterCtl(Vec<ElemId>, MaudioNormalMeter);

impl Default for MeterCtl {
    fn default() -> Self {
        Self(Default::default(), SoloMeterProtocol::create_meter())
    }
}

impl AsMut<MaudioNormalMeter> for MeterCtl {
    fn as_mut(&mut self) -> &mut MaudioNormalMeter {
        &mut self.1
    }
}

impl AsRef<MaudioNormalMeter> for MeterCtl {
    fn as_ref(&self) -> &MaudioNormalMeter {
        &self.1
    }
}

impl MaudioNormalMeterCtlOperation<SoloMeterProtocol> for MeterCtl {}

#[derive(Default)]
struct PhysInputCtl;

impl AvcLevelCtlOperation<SoloPhysInputProtocol> for PhysInputCtl {
    const LEVEL_NAME: &'static str = "phys-input-gain";
    const PORT_LABELS: &'static [&'static str] = &[
        "analog-input-1", "analog-input-2", "digital-input-1", "digital-input-2",
    ];
}

impl AvcLrBalanceCtlOperation<SoloPhysInputProtocol> for PhysInputCtl {
    const BALANCE_NAME: &'static str = "phys-input-balance";
}

#[derive(Default)]
struct StreamInputCtl;

impl AvcLevelCtlOperation<SoloStreamInputProtocol> for StreamInputCtl {
    const LEVEL_NAME: &'static str = "stream-input-gain";
    const PORT_LABELS: &'static [&'static str] = &[
        "stream-input-1", "stream-input-2", "stream-input-3", "stream-input-4",
    ];
}

#[derive(Default)]
struct SpdifOutputCtl;

impl AvcSelectorCtlOperation<SoloSpdifOutputProtocol> for SpdifOutputCtl {
    const SELECTOR_NAME: &'static str = "S/PDIF-output-source";
    const SELECTOR_LABELS: &'static [&'static str] = &["S/PDIF-output-1/2"];
    const ITEM_LABELS: &'static [&'static str] = &["stream-input-3/4", "mixer-output-3/4"];
}

impl<'a> SoloModel<'a> {
    const MIXER_DST_FB_IDS: &'a [u8] = &[0x01, 0x01];
    const MIXER_LABELS: &'a [&'a str] = &["mixer-1/2", "mixer-3/4"];
    const MIXER_PHYS_SRC_FB_IDS: &'a [u8] = &[0x00, 0x01];
    const PHYS_IN_LABELS: &'a [&'a str] = &["analog-1/2", "digital-1/2"];
    const MIXER_STREAM_SRC_FB_IDS: &'a [u8] = &[0x02, 0x03];
    const STREAM_IN_LABELS: &'a [&'a str] = &["stream-1/2", "stream-1/2"];
}

impl<'a> Default for SoloModel<'a> {
    fn default() -> Self {
        Self{
            avc: Default::default(),
            req: Default::default(),
            clk_ctl: Default::default(),
            meter_ctl: Default::default(),
            phys_input_ctl: Default::default(),
            stream_input_ctl: Default::default(),
            spdif_output_ctl: Default::default(),
            mixer_ctl: MixerCtl::new(
                Self::MIXER_DST_FB_IDS, Self::MIXER_LABELS,
                Self::MIXER_PHYS_SRC_FB_IDS, Self::PHYS_IN_LABELS,
                Self::MIXER_STREAM_SRC_FB_IDS, Self::STREAM_IN_LABELS,
            ),
        }
    }
}

impl<'a> CtlModel<SndUnit> for SoloModel<'a> {
    fn load(&mut self, unit: &mut SndUnit, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.avc.as_ref().bind(&unit.get_node())?;

        self.clk_ctl.load_freq(card_cntr)
            .map(|mut elem_id_list| self.clk_ctl.0.append(&mut elem_id_list))?;

        self.clk_ctl.load_src(card_cntr)
            .map(|mut elem_id_list| self.clk_ctl.0.append(&mut elem_id_list))?;

        self.meter_ctl.load_meter(card_cntr, &self.req, &unit.get_node(), TIMEOUT_MS)
            .map(|mut elem_id_list| self.meter_ctl.0.append(&mut elem_id_list))?;

        self.phys_input_ctl.load_level(card_cntr)?;
        self.phys_input_ctl.load_balance(card_cntr)?;
        self.stream_input_ctl.load_level(card_cntr)?;

        self.spdif_output_ctl.load_selector(card_cntr)?;

        self.mixer_ctl.load(&self.avc, card_cntr)?;

        Ok(())
    }

    fn read(&mut self, _: &mut SndUnit, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        if self.clk_ctl.read_freq(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.clk_ctl.read_src(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.meter_ctl.read_meter(elem_id, elem_value)? {
            Ok(true)
        } else if self.phys_input_ctl.read_level(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.phys_input_ctl.read_balance(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.stream_input_ctl.read_level(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)? {
             Ok(true)
        } else if self.spdif_output_ctl.read_selector(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)? {
             Ok(true)
        } else if self.mixer_ctl.read(&self.avc, elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(&mut self, unit: &mut SndUnit, elem_id: &ElemId, old: &ElemValue, new: &ElemValue)
        -> Result<bool, Error>
    {
        if self.clk_ctl.write_freq(unit, &self.avc, elem_id, old, new, FCP_TIMEOUT_MS * 3)? {
            Ok(true)
        } else if self.clk_ctl.write_src(unit, &self.avc, elem_id, old, new, FCP_TIMEOUT_MS * 3)? {
            Ok(true)
        } else if self.phys_input_ctl.write_level(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.phys_input_ctl.write_balance(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.stream_input_ctl.write_level(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.spdif_output_ctl.write_selector(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)? {
             Ok(true)
        } else if self.mixer_ctl.write(&self.avc, elem_id, old, new)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl<'a> MeasureModel<SndUnit> for SoloModel<'a> {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.meter_ctl.0);
    }

    fn measure_states(&mut self, unit: &mut SndUnit) -> Result<(), Error> {
        self.meter_ctl.measure_meter(&self.req, &unit.get_node(), &self.avc, TIMEOUT_MS)
    }

    fn measure_elem(&mut self, _: &SndUnit, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        self.meter_ctl.read_meter(elem_id, elem_value)
    }
}

impl<'a> NotifyModel<SndUnit, bool> for SoloModel<'a> {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.clk_ctl.0);
    }

    fn parse_notification(&mut self, _: &mut SndUnit, _: &bool) -> Result<(), Error> {
        Ok(())
    }

    fn read_notified_elem(&mut self, _: &SndUnit, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        self.clk_ctl.read_freq(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)
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

        let ctl = PhysInputCtl::default();
        let error = ctl.load_level(&mut card_cntr).unwrap_err();
        assert_eq!(error.kind::<CardError>(), Some(CardError::Failed));

        let ctl = StreamInputCtl::default();
        let error = ctl.load_level(&mut card_cntr).unwrap_err();
        assert_eq!(error.kind::<CardError>(), Some(CardError::Failed));
    }

    #[test]
    fn test_selector_ctl_definition() {
        let mut card_cntr = CardCntr::new();

        let ctl = SpdifOutputCtl::default();
        let error = ctl.load_selector(&mut card_cntr).unwrap_err();
        assert_eq!(error.kind::<CardError>(), Some(CardError::Failed));
    }
}
