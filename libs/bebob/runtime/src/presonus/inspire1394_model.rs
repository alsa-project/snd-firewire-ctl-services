// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

use glib::Error;

use hinawa::{FwFcpExt, FwReq};
use hinawa::{SndUnit, SndUnitExt};

use alsactl::{ElemId, ElemIfaceType, ElemValue, ElemValueExt};

use alsa_ctl_tlv_codec::items::DbInterval;

use core::card_cntr::*;

use bebob_protocols::{*, presonus::inspire1394::*};

use crate::common_ctls::*;
use crate::model::{IN_METER_NAME, OUT_METER_NAME};

#[derive(Default)]
pub struct Inspire1394Model {
    avc: BebobAvc,
    req: FwReq,
    clk_ctl: ClkCtl,
    meter_ctl: MeterCtl,
    phys_in_ctl: PhysInputCtl,
    phys_out_ctl: PhysOutputCtl,
    hp_ctl: HeadphoneCtl,
    mixer_phys_src_ctl: MixerPhysSourceCtl,
    mixer_stream_src_ctl: MixerStreamSourceCtl,
}

const FCP_TIMEOUT_MS: u32 = 100;
const TIMEOUT_MS: u32 = 50;

#[derive(Default)]
struct ClkCtl(Vec<ElemId>);

impl MediaClkFreqCtlOperation<Inspire1394ClkProtocol> for ClkCtl {}

impl SamplingClkSrcCtlOperation<Inspire1394ClkProtocol> for ClkCtl {
    const SRC_LABELS: &'static [&'static str] = &["Internal"];
}

#[derive(Default)]
struct MeterCtl(Vec<ElemId>, Inspire1394Meter);

impl AsRef<Inspire1394Meter> for MeterCtl {
    fn as_ref(&self) -> &Inspire1394Meter {
        &self.1
    }
}

impl AsMut<Inspire1394Meter> for MeterCtl {
    fn as_mut(&mut self) -> &mut Inspire1394Meter {
        &mut self.1
    }
}

impl MeterCtlOperation<Inspire1394MeterProtocol> for MeterCtl {}

#[derive(Default)]
struct PhysInputCtl;

impl AvcLevelCtlOperation<Inspire1394PhysInputProtocol> for PhysInputCtl {
    const LEVEL_NAME: &'static str = "analog-input-gain";
    const PORT_LABELS: &'static [&'static str] = &[
        "analog-input-1", "analog-input-2", "analog-input-3", "analog-input-4",
    ];
}

#[derive(Default)]
struct PhysOutputCtl;

impl AvcLevelCtlOperation<Inspire1394PhysOutputProtocol> for PhysOutputCtl {
    const LEVEL_NAME: &'static str = "analog-output-volume";
    const PORT_LABELS: &'static [&'static str] = &["analog-output-1", "analog-output-2"];
}

#[derive(Default)]
struct HeadphoneCtl;

impl AvcLevelCtlOperation<Inspire1394HeadphoneProtocol> for HeadphoneCtl {
    const LEVEL_NAME: &'static str = "headphone-volume";
    const PORT_LABELS: &'static [&'static str] = &["headphone-1", "headphone-2"];
}

#[derive(Default)]
struct MixerPhysSourceCtl;

impl AvcLevelCtlOperation<Inspire1394MixerAnalogSourceProtocol> for MixerPhysSourceCtl {
    const LEVEL_NAME: &'static str = "mixer-analog-source-gain";
    const PORT_LABELS: &'static [&'static str] = &[
        "analog-input-1", "analog-input-2", "analog-input-3", "analog-input-4",
    ];
}

impl AvcLrBalanceCtlOperation<Inspire1394MixerAnalogSourceProtocol> for MixerPhysSourceCtl {
    const BALANCE_NAME: &'static str = "mixer-analog-source-balance";
}

#[derive(Default)]
struct MixerStreamSourceCtl;

impl AvcLevelCtlOperation<Inspire1394MixerStreamSourceProtocol> for MixerStreamSourceCtl {
    const LEVEL_NAME: &'static str = "mixer-stream-source-gain";
    const PORT_LABELS: &'static [&'static str] = &["stream-input-1/2"];
}

impl CtlModel<SndUnit> for Inspire1394Model {
    fn load(&mut self, unit: &mut SndUnit, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.avc.as_ref().bind(&unit.get_node())?;

        self.clk_ctl.load_freq(card_cntr)
            .map(|mut elem_id_list| self.clk_ctl.0.append(&mut elem_id_list))?;

        self.clk_ctl.load_src(card_cntr)
            .map(|mut elem_id_list| self.clk_ctl.0.append(&mut elem_id_list))?;

        self.meter_ctl.load_meter(card_cntr, &self.req, unit, TIMEOUT_MS)
            .map(|mut elem_id_list| self.meter_ctl.0.append(&mut elem_id_list))?;

        self.phys_in_ctl.load_level(card_cntr)?;
        self.phys_out_ctl.load_level(card_cntr)?;
        self.hp_ctl.load_level(card_cntr)?;
        self.mixer_phys_src_ctl.load_level(card_cntr)?;
        self.mixer_phys_src_ctl.load_balance(card_cntr)?;
        self.mixer_stream_src_ctl.load_level(card_cntr)?;

        Ok(())
    }

    fn read(&mut self, _: &mut SndUnit, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        if self.clk_ctl.read_freq(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.clk_ctl.read_src(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.phys_in_ctl.read_level(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.phys_out_ctl.read_level(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.hp_ctl.read_level(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.mixer_phys_src_ctl.read_level(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.mixer_phys_src_ctl.read_balance(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.mixer_stream_src_ctl.read_level(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)? {
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
        } else if self.clk_ctl.write_src(unit, &self.avc, elem_id, old, new, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.phys_in_ctl.write_level(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.phys_out_ctl.write_level(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.hp_ctl.write_level(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.mixer_phys_src_ctl.write_level(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.mixer_phys_src_ctl.write_balance(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.mixer_stream_src_ctl.write_level(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl NotifyModel<SndUnit, bool> for Inspire1394Model {
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

impl MeasureModel<SndUnit> for Inspire1394Model {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.meter_ctl.0);
    }

    fn measure_states(&mut self, unit: &mut SndUnit) -> Result<(), Error> {
        self.meter_ctl.measure_meter(&self.req, unit, TIMEOUT_MS)
    }

    fn measure_elem(
        &mut self,
        _: &SndUnit,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        self.meter_ctl.read_meter(elem_id, elem_value)
    }
}

const STREAM_INPUT_METER_NAME: &str = "stream-input";

const METER_TLV: DbInterval = DbInterval{min: -14400, max: 0, linear: false, mute_avail: false};

trait MeterCtlOperation<T: Inspire1394MeterOperation>: AsRef<Inspire1394Meter> + AsMut<Inspire1394Meter> {
    fn load_meter(
        &mut self,
        card_cntr: &mut CardCntr,
        req: &FwReq,
        unit: &SndUnit,
        timeout_ms: u32,
    ) -> Result<Vec<ElemId>, Error> {
        T::read_meter(req, &unit.get_node(), self.as_mut(), timeout_ms)?;

        let mut measured_elem_id_list = Vec::new();

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, IN_METER_NAME, 0);
        card_cntr.add_int_elems(&elem_id, 1, T::LEVEL_MIN, T::LEVEL_MAX, T::LEVEL_STEP,
                                4, Some(&Into::<Vec<u32>>::into(METER_TLV)), false)
            .map(|mut elem_id_list| measured_elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, STREAM_INPUT_METER_NAME, 0);
        card_cntr.add_int_elems(&elem_id, 1, T::LEVEL_MIN, T::LEVEL_MAX, T::LEVEL_STEP,
                                2, Some(&Into::<Vec<u32>>::into(METER_TLV)), false)
            .map(|mut elem_id_list| measured_elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, OUT_METER_NAME, 0);
        card_cntr.add_int_elems(&elem_id, 1, T::LEVEL_MIN, T::LEVEL_MAX, T::LEVEL_STEP,
                                2, Some(&Into::<Vec<u32>>::into(METER_TLV)), false)
            .map(|mut elem_id_list| measured_elem_id_list.append(&mut elem_id_list))?;

        Ok(measured_elem_id_list)
    }

    fn measure_meter(
        &mut self,
        req: &FwReq,
        unit: &SndUnit,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        T::read_meter(req, &unit.get_node(), self.as_mut(), timeout_ms)
    }

    fn read_meter(
        &self,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            IN_METER_NAME => {
                elem_value.set_int(&self.as_ref().phys_inputs);
                Ok(true)
            }
            STREAM_INPUT_METER_NAME => {
                elem_value.set_int(&self.as_ref().stream_inputs);
                Ok(true)
            }
            OUT_METER_NAME => {
                elem_value.set_int(&self.as_ref().phys_outputs);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
