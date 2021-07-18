// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use glib::Error;

use hinawa::{FwFcpExt, FwReq};
use hinawa::{SndUnit, SndUnitExt};

use alsactl::{ElemId, ElemIfaceType, ElemValue};

use core::card_cntr::*;
use core::elem_value_accessor::ElemValueAccessor;

use ta1394::Ta1394Avc;
use ta1394::audio::{AUDIO_SUBUNIT_0_ADDR, CtlAttr, AudioCh, ProcessingCtl, AudioProcessing, AudioSelector};

use bebob_protocols::{*, maudio::normal::*};

use crate::common_ctls::*;

use super::*;
use super::normal_ctls::MixerCtl;

pub struct Fw410Model<'a>{
    avc: BebobAvc,
    req: FwReq,
    clk_ctl: ClkCtl,
    meter_ctl: MeterCtl,
    phys_input_ctl: PhysInputCtl,
    aux_src_ctl: AuxSourceCtl,
    aux_output_ctl: AuxOutputCtl,
    phys_output_ctl: PhysOutputCtl,
    hp_ctl: HeadphoneCtl,
    mixer_ctl: MixerCtl<'a>,
}

const FCP_TIMEOUT_MS: u32 = 100;
const TIMEOUT_MS: u32 = 50;

#[derive(Default)]
struct ClkCtl(Vec<ElemId>);

impl MediaClkFreqCtlOperation<Fw410ClkProtocol> for ClkCtl {}

impl SamplingClkSrcCtlOperation<Fw410ClkProtocol> for ClkCtl {
    const SRC_LABELS: &'static [&'static str] = &["Internal", "S/PDIF"];
}

struct MeterCtl(Vec<ElemId>, MaudioNormalMeter);

impl Default for MeterCtl {
    fn default() -> Self {
        Self(Default::default(), Fw410MeterProtocol::create_meter())
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

impl MaudioNormalMeterCtlOperation<Fw410MeterProtocol> for MeterCtl {}

#[derive(Default)]
struct PhysInputCtl;

impl AvcLevelCtlOperation<Fw410PhysInputProtocol> for PhysInputCtl {
    const LEVEL_NAME: &'static str = "phys-input-gain";
    const PORT_LABELS: &'static [&'static str] = &[
        "analog-input-1", "analog-input-2", "digital-input-1", "digital-input-2",
    ];
}

impl AvcLrBalanceCtlOperation<Fw410PhysInputProtocol> for PhysInputCtl {
    const BALANCE_NAME: &'static str = "phys-input-balance";
}

#[derive(Default)]
struct AuxSourceCtl;

impl AvcLevelCtlOperation<Fw410AuxSourceProtocol> for AuxSourceCtl {
    const LEVEL_NAME: &'static str = "aux-source-gain";
    const PORT_LABELS: &'static [&'static str] = &[
        "analog-input-1", "analog-input-2", "digital-input-1", "digital-input-2",
        "stream-input-1", "stream-input-2", "stream-input-3", "stream-input-4",
        "stream-input-5", "stream-input-6", "stream-input-7", "stream-input-8",
        "stream-input-9", "stream-input-10",
    ];
}

#[derive(Default)]
struct AuxOutputCtl;

impl AvcLevelCtlOperation<Fw410AuxOutputProtocol> for AuxOutputCtl {
    const LEVEL_NAME: &'static str = "aux-output-volume";
    const PORT_LABELS: &'static [&'static str] = &["aux-output-1", "aux-output-2"];
}

#[derive(Default)]
struct PhysOutputCtl;

impl AvcLevelCtlOperation<Fw410PhysOutputProtocol> for PhysOutputCtl {
    const LEVEL_NAME: &'static str = "output-volume";
    const PORT_LABELS: &'static [&'static str] = &[
        "analog-output-1", "analog-output-2", "analog-output-3", "analog-output-4",
        "analog-output-5", "analog-output-6", "analog-output-7", "analog-output-8",
        "digital-output-1", "digital-output-2",
    ];
}

#[derive(Default)]
struct HeadphoneCtl;

impl AvcLevelCtlOperation<Fw410HeadphoneProtocol> for HeadphoneCtl {
    const LEVEL_NAME: &'static str = "headphone-volume";
    const PORT_LABELS: &'static [&'static str] = &["headphone-1", "headphone-2"];
}

impl<'a> Fw410Model<'a> {
    const MIXER_DST_FB_IDS: &'a [u8] = &[0x01, 0x01, 0x01, 0x01, 0x01];
    const MIXER_LABELS: &'a [&'a str] = &[
        "mixer-1/2", "mixer-3/4", "mixer-5/6", "mixer-7/8",
        "mixer-9/10",
    ];
    const MIXER_PHYS_SRC_FB_IDS: &'a [u8] = &[0x02, 0x03];
    const PHYS_IN_LABELS: &'a [&'a str] = &["analog-1/2", "digital-1/2"];
    const MIXER_STREAM_SRC_FB_IDS: &'a [u8] = &[0x01, 0x00, 0x00, 0x00, 0x00];
    const STREAM_IN_LABELS: &'a [&'a str] = &[
        "stream-1/2", "stream-3/4", "stream-5/6", "stream-7/8",
        "stream-9/10",
    ];

    const PHYS_OUT_LABELS: &'a [&'a str] = &[
        "analog-1/2", "analog-3/4", "analog-5/6", "analog-7/8",
        "digital-1/2",
    ];
    const PHYS_OUT_SRC_FB_IDS: &'a [u8] = &[0x02, 0x03, 0x04, 0x05, 0x06];
}

impl<'a> Default for Fw410Model<'a> {
    fn default() -> Self {
        Self{
            avc: Default::default(),
            req: Default::default(),
            clk_ctl: Default::default(),
            meter_ctl: Default::default(),
            phys_input_ctl: Default::default(),
            aux_src_ctl: Default::default(),
            aux_output_ctl: Default::default(),
            phys_output_ctl: Default::default(),
            hp_ctl: Default::default(),
            mixer_ctl: MixerCtl::new(
                Self::MIXER_DST_FB_IDS, Self::MIXER_LABELS,
                Self::MIXER_PHYS_SRC_FB_IDS, Self::PHYS_IN_LABELS,
                Self::MIXER_STREAM_SRC_FB_IDS, Self::STREAM_IN_LABELS,
            ),
        }
    }
}

impl<'a> CtlModel<SndUnit> for Fw410Model<'a> {
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
        self.aux_src_ctl.load_level(card_cntr)?;
        self.aux_output_ctl.load_level(card_cntr)?;
        self.phys_output_ctl.load_level(card_cntr)?;
        self.hp_ctl.load_level(card_cntr)?;

        self.mixer_ctl.load(&self.avc, card_cntr)?;
        HpMixerCtl::load(&self.avc, card_cntr)?;

        SpdifSrcCtl::load(&self.avc, card_cntr)?;

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
        } else if self.aux_src_ctl.read_level(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.aux_output_ctl.read_level(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.phys_output_ctl.read_level(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.hp_ctl.read_level(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.mixer_ctl.read(&self.avc, elem_id, elem_value)? {
            Ok(true)
        } else if HpMixerCtl::read(&self.avc, elem_id, elem_value)? {
            Ok(true)
        } else if SpdifSrcCtl::read(&self.avc, elem_id, elem_value)? {
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
        } else if self.aux_src_ctl.write_level(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.aux_output_ctl.write_level(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.phys_output_ctl.write_level(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.hp_ctl.write_level(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.mixer_ctl.write(&self.avc, elem_id, old, new)? {
            Ok(true)
        } else if HpMixerCtl::write(&self.avc, elem_id, old, new)? {
            Ok(true)
        } else if SpdifSrcCtl::write(&self.avc, elem_id, old, new)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl<'a> MeasureModel<SndUnit> for Fw410Model<'a> {
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

impl<'a> NotifyModel<SndUnit, bool> for Fw410Model<'a> {
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

const HP_MIXER_SRC_NAME: &str = "headphone-mixer-source";
const HP_MIXER_DST_FB_ID: u8 = 0x07;
const HP_MIXER_SRC_FB_ID: u8 = 0x00;

const HP_MIXER_ON: i16 = 0x0000;
const HP_MIXER_OFF: i16 = (0x8000 as u16) as i16;

trait HpMixerCtl : Ta1394Avc {
    fn load(&self, card_cntr: &mut CardCntr,) -> Result<(), Error> {
        // For physical/stream inputs to headphone mixer.
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, HP_MIXER_SRC_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, Fw410Model::PHYS_OUT_LABELS.len(), true)?;

        Ok(())
    }

    fn read(&self, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            HP_MIXER_SRC_NAME => {
                ElemValueAccessor::<bool>::set_vals(elem_value, Fw410Model::PHYS_OUT_SRC_FB_IDS.len(), |idx| {
                    // NOTE: The value of 0/1 for out_ch has the same effect.
                    let mut op = AudioProcessing::new(HP_MIXER_DST_FB_ID, CtlAttr::Current, HP_MIXER_SRC_FB_ID,
                                                      AudioCh::Each(Fw410Model::PHYS_OUT_SRC_FB_IDS[idx]),
                                                      AudioCh::Each(0), ProcessingCtl::Mixer(vec![-1]));
                    self.status(&AUDIO_SUBUNIT_0_ADDR, &mut op, FCP_TIMEOUT_MS)?;
                    if let ProcessingCtl::Mixer(data) = op.ctl {
                        Ok(data[0] == HP_MIXER_ON)
                    } else {
                        unreachable!();
                    }
                })?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write(&self, elem_id: &ElemId, old: &ElemValue, new: &ElemValue)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            HP_MIXER_SRC_NAME => {
                ElemValueAccessor::<bool>::get_vals(new, old, Fw410Model::PHYS_OUT_SRC_FB_IDS.len(), |idx, val| {
                    let ctl = ProcessingCtl::Mixer(vec![if val { HP_MIXER_ON } else { HP_MIXER_OFF }]);
                    // NOTE: The value of 0/1 for out_ch has the same effect.
                    let mut op = AudioProcessing::new(HP_MIXER_DST_FB_ID, CtlAttr::Current, HP_MIXER_SRC_FB_ID,
                                        AudioCh::Each(Fw410Model::PHYS_OUT_SRC_FB_IDS[idx]), AudioCh::Each(0), ctl);
                    self.control(&AUDIO_SUBUNIT_0_ADDR, &mut op, FCP_TIMEOUT_MS)
                })?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

impl HpMixerCtl for BebobAvc {}

const SPDIF_SRC_NAME: &str = "S/PDIF-input-source";
const SPDIF_SRC_LABELS: &[&str] = &["coaxial", "optical"];
const SPDIF_SRC_FB_ID: u8 = 0x01;

trait SpdifSrcCtl : Ta1394Avc {
    fn load(&self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, SPDIF_SRC_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, SPDIF_SRC_LABELS, None, true)?;
        Ok(())
    }

    fn read(&self, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            SPDIF_SRC_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let mut op = AudioSelector::new(SPDIF_SRC_FB_ID, CtlAttr::Current, 0xff);
                    self.status(&AUDIO_SUBUNIT_0_ADDR, &mut op, FCP_TIMEOUT_MS)?;
                    Ok(op.input_plug_id as u32)
                })?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write(&self, elem_id: &ElemId, _: &ElemValue, new: &ElemValue)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            SPDIF_SRC_NAME => {
                ElemValueAccessor::<u32>::get_val(new, |val| {
                    let mut op = AudioSelector::new(SPDIF_SRC_FB_ID, CtlAttr::Current, val as u8);
                    self.control(&AUDIO_SUBUNIT_0_ADDR, &mut op, FCP_TIMEOUT_MS)
                })?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

impl SpdifSrcCtl for BebobAvc {}

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

        let ctl = AuxSourceCtl::default();
        let error = ctl.load_level(&mut card_cntr).unwrap_err();
        assert_eq!(error.kind::<CardError>(), Some(CardError::Failed));

        let ctl = AuxOutputCtl::default();
        let error = ctl.load_level(&mut card_cntr).unwrap_err();
        assert_eq!(error.kind::<CardError>(), Some(CardError::Failed));

        let ctl = PhysOutputCtl::default();
        let error = ctl.load_level(&mut card_cntr).unwrap_err();
        assert_eq!(error.kind::<CardError>(), Some(CardError::Failed));

        let ctl = HeadphoneCtl::default();
        let error = ctl.load_level(&mut card_cntr).unwrap_err();
        assert_eq!(error.kind::<CardError>(), Some(CardError::Failed));
    }
}
