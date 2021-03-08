// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto
use glib::Error;

use alsactl::{ElemId, ElemIfaceType, ElemValue, ElemValueExt, ElemValueExtManual};

use hinawa::{SndUnit, SndUnitExt};

use alsa_ctl_tlv_codec::items::DbInterval;

use core::card_cntr::*;

use ff_protocols::former::ff400::*;

use super::former_ctls::*;

#[derive(Default, Debug)]
pub struct Ff400Model{
    proto: Ff400Protocol,
    meter_ctl: FormerMeterCtl<Ff400MeterState>,
    out_ctl: FormerOutCtl<Ff400OutputVolumeState>,
    input_gain_ctl: InputGainCtl,
    mixer_ctl: FormerMixerCtl<Ff400MixerState>,
}

const TIMEOUT_MS: u32 = 100;

impl CtlModel<SndUnit> for Ff400Model {
    fn load(&mut self, unit: &SndUnit, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.meter_ctl.load(unit, &self.proto, card_cntr, TIMEOUT_MS)?;
        self.out_ctl.load(unit, &self.proto, card_cntr, TIMEOUT_MS)?;
        self.mixer_ctl.load(unit, &self.proto, card_cntr, TIMEOUT_MS)?;
        self.input_gain_ctl.load(unit, &self.proto, card_cntr, TIMEOUT_MS)?;
        Ok(())
    }

    fn read(&mut self, _: &SndUnit, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        if self.out_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.input_gain_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(&mut self, unit: &SndUnit, elem_id: &ElemId, _: &ElemValue, new: &ElemValue)
        -> Result<bool, Error>
    {
        if self.out_ctl.write(unit, &self.proto, elem_id, new, TIMEOUT_MS)? {
            Ok(true)
        } else if self.mixer_ctl.write(unit, &self.proto, elem_id, new, TIMEOUT_MS)? {
            Ok(true)
        } else if self.input_gain_ctl.write(unit, &self.proto, elem_id, new, TIMEOUT_MS)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl MeasureModel<SndUnit> for Ff400Model {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        self.meter_ctl.get_measured_elem_list(elem_id_list);
    }

    fn measure_states(&mut self, unit: &SndUnit) -> Result<(), Error> {
        self.meter_ctl.measure_states(unit, &self.proto, TIMEOUT_MS)
    }

    fn measure_elem(&mut self, _: &SndUnit, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        self.meter_ctl.measure_elem(elem_id, elem_value)
    }
}

#[derive(Default, Debug)]
struct InputGainCtl{
    status: Ff400InputGainStatus,
}

impl<'a> InputGainCtl {
    const MIC_GAIN_NAME: &'a str = "mic-input-gain";
    const LINE_GAIN_NAME: &'a str = "line-input-gain";

    const MIC_GAIN_MIN: i32 = 0;
    const MIC_GAIN_MAX: i32 = 65;
    const MIC_GAIN_STEP: i32 = 1;
    const MIC_GAIN_TLV: DbInterval = DbInterval{min: 0, max: 6500, linear: false, mute_avail: false};

    const LINE_GAIN_MIN: i32 = 0;
    const LINE_GAIN_MAX: i32 = 36;
    const LINE_GAIN_STEP: i32 = 1;
    const LINE_GAIN_TLV: DbInterval = DbInterval{min: 0, max: 18000, linear: false, mute_avail: false};

    fn load(&mut self, unit: &SndUnit, proto: &Ff400Protocol, card_cntr: &mut CardCntr, timeout_ms: u32)
        -> Result<(), Error>
    {
        proto.init_input_gains(&unit.get_node(), &mut self.status, timeout_ms)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::MIC_GAIN_NAME, 0);
        card_cntr.add_int_elems(&elem_id, 1, Self::MIC_GAIN_MIN, Self::MIC_GAIN_MAX, Self::MIC_GAIN_STEP,
                                2, Some(&Vec::<u32>::from(&Self::MIC_GAIN_TLV)), true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::LINE_GAIN_NAME, 0);
        card_cntr.add_int_elems(&elem_id, 1, Self::LINE_GAIN_MIN, Self::LINE_GAIN_MAX, Self::LINE_GAIN_STEP,
                                2, Some(&Vec::<u32>::from(&Self::LINE_GAIN_TLV)), true)?;

        Ok(())
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            Self::MIC_GAIN_NAME => {
                let vals: Vec<i32> = self.status.mic.iter()
                    .map(|&gain| gain as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            Self::LINE_GAIN_NAME => {
                let vals: Vec<i32> = self.status.line.iter()
                    .map(|&gain| gain as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write(&mut self, unit: &SndUnit, proto: &Ff400Protocol, elem_id: &ElemId,
             elem_value: &alsactl::ElemValue, timeout_ms: u32)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::MIC_GAIN_NAME => {
                let mut vals = [0;2];
                elem_value.get_int(&mut vals);
                let gains: Vec<i8> = vals.iter()
                    .map(|&val| val as i8)
                    .collect();
                proto.write_input_mic_gains(&unit.get_node(), &mut self.status, &gains, timeout_ms)
                    .map(|_| true)
            }
            Self::LINE_GAIN_NAME => {
                let mut vals = [0;2];
                elem_value.get_int(&mut vals);
                let gains: Vec<i8> = vals.iter()
                    .map(|&val| val as i8)
                    .collect();
                proto.write_input_line_gains(&unit.get_node(), &mut self.status, &gains, timeout_ms)
                    .map(|_| true)
            }
            _ => Ok(false),
        }
    }
}
