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
use super::normal_ctls::{MixerCtl, InputCtl, AuxCtl, OutputCtl, HpCtl};

pub struct AudiophileModel<'a>{
    avc: BebobAvc,
    req: FwReq,
    clk_ctl: ClkCtl,
    meter_ctl: MeterCtl,
    mixer_ctl: MixerCtl<'a>,
    input_ctl: InputCtl<'a>,
    aux_ctl: AuxCtl<'a>,
    output_ctl: OutputCtl<'a>,
    hp_ctl: HpCtl<'a>,
}

const FCP_TIMEOUT_MS: u32 = 100;
const TIMEOUT_MS: u32 = 50;

#[derive(Default)]
struct ClkCtl(Vec<ElemId>);

impl MediaClkFreqCtlOperation<AudiophileClkProtocol> for ClkCtl {}

impl SamplingClkSrcCtlOperation<AudiophileClkProtocol> for ClkCtl {
    const SRC_LABELS: &'static [&'static str] = &["Internal", "S/PDIF"];
}

struct MeterCtl(Vec<ElemId>, MaudioNormalMeter);

impl Default for MeterCtl {
    fn default() -> Self {
        Self(Default::default(), AudiophileMeterProtocol::create_meter())
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

impl MaudioNormalMeterCtlOperation<AudiophileMeterProtocol> for MeterCtl {}

impl<'a> AudiophileModel<'a> {
    const MIXER_DST_FB_IDS: &'a [u8] = &[0x01, 0x02, 0x03];
    const MIXER_LABELS: &'a [&'a str] = &["mixer-1/2", "mixer-3/4", "mixer-5/6"];
    const MIXER_PHYS_SRC_FB_IDS: &'a [u8] = &[0x03, 0x04];
    const PHYS_IN_LABELS: &'a [&'a str] = &["analog-1/2", "digital-1/2"];
    const MIXER_STREAM_SRC_FB_IDS: &'a [u8] = &[0x00, 0x01, 0x02];
    const STREAM_IN_LABELS: &'a [&'a str] = &["stream-1/2", "stream-3/4", "stream-5/6"];
    const HP_SRC_LABELS: &'a [&'a str] = &["mixer-1/2", "mixer-3/4", "mixer-5/6", "aux-1/2"];

    const PHYS_IN_FB_IDS: &'a [u8] = &[0x04, 0x05];
    const STREAM_IN_FB_IDS: &'a [u8] = &[0x01, 0x02, 0x03];

    const AUX_OUT_FB_ID: u8 = 0x0b;
    const AUX_PHYS_SRC_FB_IDS: &'a [u8] = &[0x09, 0x0a];
    const AUX_STREAM_SRC_FB_IDS: &'a [u8] = &[0x06, 0x07, 0x08];

    const PHYS_OUT_LABELS: &'a [&'a str] = &["analog-1/2", "analog-3/4", "digital-1/2"];
    const PHYS_OUT_FB_IDS: &'a [u8] = &[0x0c, 0x0d, 0x0e];
    const PHYS_OUT_SRC_FB_IDS: &'a [u8] = &[0x01, 0x02, 0x03];

    const HP_SRC_FB_ID: u8 = 0x04;
    const HP_OUT_FB_ID: u8 = 0x0f;
}

impl<'a> Default for AudiophileModel<'a> {
    fn default() -> Self {
        Self{
            avc: Default::default(),
            req: Default::default(),
            clk_ctl: Default::default(),
            meter_ctl: Default::default(),
            mixer_ctl: MixerCtl::new(
                Self::MIXER_DST_FB_IDS, Self::MIXER_LABELS,
                Self::MIXER_PHYS_SRC_FB_IDS, Self::PHYS_IN_LABELS,
                Self::MIXER_STREAM_SRC_FB_IDS, Self::STREAM_IN_LABELS,
            ),
            input_ctl: InputCtl::new(
                Self::PHYS_IN_FB_IDS, Self::PHYS_IN_LABELS,
                Self::STREAM_IN_FB_IDS, Self::STREAM_IN_LABELS,
            ),
            aux_ctl: AuxCtl::new(Self::AUX_OUT_FB_ID,
                Self::AUX_PHYS_SRC_FB_IDS, Self::PHYS_IN_LABELS,
                Self::AUX_STREAM_SRC_FB_IDS, Self::STREAM_IN_LABELS,
            ),
            output_ctl: OutputCtl::new(
                Self::PHYS_OUT_LABELS,
                Self::PHYS_OUT_FB_IDS,
                Self::PHYS_OUT_SRC_FB_IDS,
            ),
            hp_ctl: HpCtl::new(Self::HP_OUT_FB_ID, Self::HP_SRC_FB_ID, Self::HP_SRC_LABELS),
        }
    }
}

impl<'a> CtlModel<SndUnit> for AudiophileModel<'a> {
    fn load(&mut self, unit: &mut SndUnit, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.avc.as_ref().bind(&unit.get_node())?;

        self.clk_ctl.load_freq(card_cntr)
            .map(|mut elem_id_list| self.clk_ctl.0.append(&mut elem_id_list))?;

        self.clk_ctl.load_src(card_cntr)
            .map(|mut elem_id_list| self.clk_ctl.0.append(&mut elem_id_list))?;

        self.meter_ctl.load_meter(card_cntr, &self.req, &unit.get_node(), TIMEOUT_MS)
            .map(|mut elem_id_list| self.meter_ctl.0.append(&mut elem_id_list))?;

        self.mixer_ctl.load(&self.avc, card_cntr)?;
        self.input_ctl.load(&self.avc, card_cntr)?;
        self.aux_ctl.load(&self.avc, card_cntr)?;
        self.output_ctl.load(&self.avc, card_cntr)?;
        self.hp_ctl.load(&self.avc, card_cntr)?;

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
        } else if self.mixer_ctl.read(&self.avc, elem_id, elem_value)? {
            Ok(true)
        } else if self.input_ctl.read(&self.avc, elem_id, elem_value)? {
            Ok(true)
        } else if self.aux_ctl.read(&self.avc, elem_id, elem_value)? {
            Ok(true)
        } else if self.output_ctl.read(&self.avc, elem_id, elem_value)? {
            Ok(true)
        } else if self.hp_ctl.read(&self.avc, elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(&mut self, unit: &mut SndUnit, elem_id: &ElemId,
             old: &ElemValue, new: &ElemValue)
        -> Result<bool, Error>
    {
        if self.clk_ctl.write_freq(unit, &self.avc, elem_id, old, new, FCP_TIMEOUT_MS * 3)? {
            Ok(true)
        } else if self.clk_ctl.write_src(unit, &self.avc, elem_id, old, new, FCP_TIMEOUT_MS * 3)? {
            Ok(true)
        } else if self.meter_ctl.write_meter(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.mixer_ctl.write(&self.avc, elem_id, old, new)? {
            Ok(true)
        } else if self.input_ctl.write(&self.avc, elem_id, old, new)? {
            Ok(true)
        } else if self.aux_ctl.write(&self.avc, elem_id, old, new)? {
            Ok(true)
        } else if self.output_ctl.write(&self.avc, elem_id, old, new)? {
            Ok(true)
        } else if self.hp_ctl.write(&self.avc, elem_id, old, new)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl<'a> MeasureModel<SndUnit> for AudiophileModel<'a> {
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

impl<'a> NotifyModel<SndUnit, bool> for AudiophileModel<'a> {
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
}
