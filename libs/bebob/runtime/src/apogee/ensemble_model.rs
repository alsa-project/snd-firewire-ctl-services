// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

use glib::{Error, FileError};

use hinawa::FwFcpExt;
use hinawa::{SndUnit, SndUnitExt};

use alsactl::{ElemId, ElemIfaceType, ElemValue, ElemValueExt, ElemValueExtManual};

use core::card_cntr::*;
use core::elem_value_accessor::ElemValueAccessor;

use alsa_ctl_tlv_codec::items::DbInterval;

use bebob_protocols::{*, apogee::ensemble::*};

use crate::model::{IN_METER_NAME, OUT_METER_NAME, HP_SRC_NAME, OUT_SRC_NAME};

use crate::common_ctls::*;
use super::apogee_ctls::HwCtl;

const FCP_TIMEOUT_MS: u32 = 100;

pub struct EnsembleModel {
    avc: BebobAvc,
    clk_ctl: ClkCtl,
    meter_ctl: MeterCtl,
    convert_ctl: ConvertCtl,
    display_ctl: DisplayCtl,
    input_ctl: InputCtl,
    output_ctl: OutputCtl,
    route_ctl: RouteCtl,
    mixer_ctl: MixerCtl,
    hw_ctls: HwCtl,
}

#[derive(Default)]
struct ClkCtl(Vec<ElemId>);

impl MediaClkFreqCtlOperation<EnsembleClkProtocol> for ClkCtl {}

impl SamplingClkSrcCtlOperation<EnsembleClkProtocol> for ClkCtl {
    const SRC_LABELS: &'static [&'static str] = &[
        "Internal",
        "S/PDIF-coax",
        "Optical",
        "Word Clock",
    ];
}

impl Default for EnsembleModel {
    fn default() -> Self {
        Self {
            avc: Default::default(),
            clk_ctl: Default::default(),
            meter_ctl: Default::default(),
            convert_ctl: Default::default(),
            display_ctl: Default::default(),
            input_ctl: Default::default(),
            output_ctl: Default::default(),
            route_ctl: Default::default(),
            mixer_ctl: Default::default(),
            hw_ctls: HwCtl::new(),
        }
    }
}

fn input_output_copy_from_meter(model: &mut EnsembleModel) {
    let m = &model.meter_ctl.0;
    model.input_ctl.0.gains.copy_from_slice(&m.knob_input_vals);
    model.output_ctl.0.vol = m.knob_output_vals[0];
    model.output_ctl.0.headphone_vols.copy_from_slice(&m.knob_output_vals[1..]);
}

impl CtlModel<SndUnit> for EnsembleModel {
    fn load(&mut self, unit: &mut SndUnit, card_cntr: &mut CardCntr)
        -> Result<(), Error>
    {
        self.avc.as_ref().bind(&unit.get_node())?;

        self.clk_ctl.load_freq(card_cntr)
            .map(|mut elem_id_list| self.clk_ctl.0.append(&mut elem_id_list))?;

        self.clk_ctl.load_src(card_cntr)
            .map(|mut elem_id_list| self.clk_ctl.0.append(&mut elem_id_list))?;

        self.meter_ctl.load_state(card_cntr, &mut self.avc, FCP_TIMEOUT_MS)
            .map(|_| input_output_copy_from_meter(self))?;

        self.convert_ctl.load_params(card_cntr, &mut self.avc, FCP_TIMEOUT_MS)?;

        self.display_ctl.load_params(card_cntr, &mut self.avc, FCP_TIMEOUT_MS)?;

        self.input_ctl.load_params(card_cntr, &mut self.avc, FCP_TIMEOUT_MS)?;

        self.output_ctl.load_params(card_cntr, &mut self.avc, FCP_TIMEOUT_MS)?;

        self.route_ctl.load_params(card_cntr, &mut self.avc, FCP_TIMEOUT_MS)?;

        self.mixer_ctl.load_params(card_cntr, &mut self.avc, FCP_TIMEOUT_MS)?;

        self.hw_ctls.load(&self.avc, card_cntr, FCP_TIMEOUT_MS)?;

        Ok(())
    }

    fn read(&mut self, _: &mut SndUnit, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        if self.clk_ctl.read_freq(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.clk_ctl.read_src(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.meter_ctl.read_state(elem_id, elem_value)? {
            Ok(true)
        } else if self.convert_ctl.read_params(elem_id, elem_value)? {
            Ok(true)
        } else if self.display_ctl.read_params(elem_id, elem_value)? {
            Ok(true)
        } else if self.input_ctl.read_params(elem_id, elem_value)? {
            Ok(true)
        } else if self.output_ctl.read_params(elem_id, elem_value)? {
            Ok(true)
        } else if self.route_ctl.read_params(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_ctl.read_params(elem_id, elem_value)? {
            Ok(true)
        } else if self.hw_ctls.read(elem_id, elem_value)? {
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
        } else if self.convert_ctl.write_params(&mut self.avc, elem_id, new, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.display_ctl.write_params(&mut self.avc, elem_id, new, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.input_ctl.write_params(&mut self.avc, elem_id, new, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.output_ctl.write_params(&mut self.avc, elem_id, new, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.route_ctl.write_params(&mut self.avc, elem_id, new, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.mixer_ctl.write_params(&mut self.avc, elem_id, new, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.hw_ctls.write(unit, &self.avc, elem_id, old, new, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else {
            Ok(true)
        }
    }
}

impl MeasureModel<SndUnit> for EnsembleModel {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.meter_ctl.1);
        elem_id_list.extend_from_slice(&self.input_ctl.1);
        elem_id_list.extend_from_slice(&self.output_ctl.1);
    }

    fn measure_states(&mut self, _: &mut SndUnit) -> Result<(), Error> {
        self.meter_ctl.measure_state(&mut self.avc, FCP_TIMEOUT_MS)
            .map(|_| input_output_copy_from_meter(self))
    }

    fn measure_elem(&mut self, _: &SndUnit, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        if self.meter_ctl.read_state(elem_id, elem_value)? {
            Ok(true)
        } else if self.input_ctl.read_params(elem_id, elem_value)? {
            Ok(true)
        } else if self.output_ctl.read_params(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl NotifyModel<SndUnit, bool> for EnsembleModel {
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

#[derive(Default)]
struct MeterCtl(EnsembleMeter, Vec<ElemId>);

const KNOB_IN_TARGET_NAME: &str = "knob-input-target";
const KNOB_OUT_TARGET_NAME: &str = "knob-output-target";

fn knob_input_target_to_str(target: &KnobInputTarget) -> &str {
    match target {
        KnobInputTarget::Mic0 => "mic-1",
        KnobInputTarget::Mic1 => "mic-2",
        KnobInputTarget::Mic2 => "mic-3",
        KnobInputTarget::Mic3 => "mic-4",
    }
}

fn knob_output_target_to_str(target: &KnobOutputTarget) -> &str {
    match target {
        KnobOutputTarget::AnalogOutputPair0 => "main",
        KnobOutputTarget::HeadphonePair0 => "headphone-1/2",
        KnobOutputTarget::HeadphonePair1 => "headphone-3/4",
    }
}

impl MeterCtl {
    const METER_TLV: DbInterval = DbInterval{min: -4800, max: 0, linear: false, mute_avail: false};

    const IN_METER_LABELS: [&'static str; 18] = [
        "analog-input-1", "analog-input-2", "analog-input-3", "analog-input-4",
        "analog-input-5", "analog-input-6", "analog-input-7", "analog-input-8",
        "spdif-input-1", "spdif-input-2",
        "adat-input-1", "adat-input-2", "adat-input-3", "adat-input-4",
        "adat-input-5", "adat-input-6", "adat-input-7", "adat-input-8",
    ];

    const OUT_METER_LABELS: [&'static str; 16] = [
        "analog-output-1", "analog-output-2", "analog-output-3", "analog-output-4",
        "analog-output-5", "analog-output-6", "analog-output-7", "analog-output-8",
        "spdif-output-1", "spdif-output-2",
        "adat-output-1", "adat-output-2", "adat-output-3", "adat-output-4",
        "adat-output-5", "adat-output-6",
        //"adat-output-7", "adat-output-8",
    ];

    const KNOB_INPUT_TARGETS: [KnobInputTarget; 4] = [
        KnobInputTarget::Mic0,
        KnobInputTarget::Mic1,
        KnobInputTarget::Mic2,
        KnobInputTarget::Mic3,
    ];

    const KNOB_OUTPUT_TARGETS: [KnobOutputTarget; 3] = [
        KnobOutputTarget::AnalogOutputPair0,
        KnobOutputTarget::HeadphonePair0,
        KnobOutputTarget::HeadphonePair1,
    ];

    const LEVEL_MIN: i32 = EnsembleMeterProtocol::LEVEL_MIN as i32;
    const LEVEL_MAX: i32 = EnsembleMeterProtocol::LEVEL_MAX as i32;
    const LEVEL_STEP: i32 = EnsembleMeterProtocol::LEVEL_STEP as i32;

    fn load_state(
        &mut self,
        card_cntr: &mut CardCntr,
        avc: &mut BebobAvc,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let labels: Vec<&str> = Self::KNOB_INPUT_TARGETS.iter()
            .map(|t| knob_input_target_to_str(t))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, KNOB_IN_TARGET_NAME, 0);
        card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        let labels: Vec<&str> = Self::KNOB_OUTPUT_TARGETS.iter()
            .map(|t| knob_output_target_to_str(t))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, KNOB_OUT_TARGET_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, true)
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, IN_METER_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                Self::LEVEL_MIN,
                Self::LEVEL_MAX,
                Self::LEVEL_STEP,
                Self::IN_METER_LABELS.len(),
                Some(&Into::<Vec<u32>>::into(Self::METER_TLV)),
                false,
            )
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, OUT_METER_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                Self::LEVEL_MIN,
                Self::LEVEL_MAX,
                Self::LEVEL_STEP,
                Self::OUT_METER_LABELS.len(),
                Some(&Into::<Vec<u32>>::into(Self::METER_TLV)),
                false,
            )
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        self.measure_state(avc, timeout_ms)
    }

    fn measure_state(&mut self, avc: &mut BebobAvc, timeout_ms: u32) -> Result<(), Error> {
        EnsembleMeterProtocol::measure_meter(avc, &mut self.0, timeout_ms)
    }

    fn read_state(
        &self,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            KNOB_IN_TARGET_NAME => {
                let idx = Self::KNOB_INPUT_TARGETS.iter()
                    .position(|t| t.eq(&self.0.knob_input_target))
                    .unwrap();
                elem_value.set_enum(&[idx as u32]);
                Ok(true)
            }
            KNOB_OUT_TARGET_NAME => {
                let idx = Self::KNOB_OUTPUT_TARGETS.iter()
                    .position(|t| t.eq(&self.0.knob_output_target))
                    .unwrap();
                elem_value.set_enum(&[idx as u32]);
                Ok(true)
            }
            IN_METER_NAME => {
                ElemValueAccessor::<i32>::set_vals(elem_value, 18, |idx| {
                    Ok(self.0.phys_inputs[idx] as i32)
                })?;
                Ok(true)
            }
            OUT_METER_NAME => {
                ElemValueAccessor::<i32>::set_vals(elem_value, 16, |idx| {
                    Ok(self.0.phys_outputs[idx] as i32)
                })?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

const FORMAT_CONVERT_TARGET_NAME: &str = "sample-format-convert-target";
const RATE_CONVERT_TARGET_NAME: &str = "sample-rate-convert-target";
const RATE_CONVERT_RATE_NAME: &str = "sample-rate-convert-rate";
const CD_MODE_NAME: &str = "cd-mode";

#[derive(Default)]
struct ConvertCtl(EnsembleConvertParameters);

fn format_convert_target_to_str(target: &FormatConvertTarget) -> &str {
    match target {
        FormatConvertTarget::Disabled => "disabled",
        FormatConvertTarget::AnalogInputPair0 => "analog-input-1/2",
        FormatConvertTarget::AnalogInputPair1 => "analog-input-3/4",
        FormatConvertTarget::AnalogInputPair2 => "analog-input-5/6",
        FormatConvertTarget::AnalogInputPair3 => "analog-input-7/8",
        FormatConvertTarget::SpdifOpticalInputPair0 => "spdif-opt-input-1/2",
        FormatConvertTarget::SpdifCoaxialInputPair0 => "spdif-coax-input-1/2",
        FormatConvertTarget::SpdifCoaxialOutputPair0 => "spdif-coax-output-1/2",
        FormatConvertTarget::SpdifOpticalOutputPair0 => "spdif-opt-output-1/2",
    }
}

fn rate_convert_target_to_str(target: &RateConvertTarget) -> &str {
    match target {
        RateConvertTarget::Disabled => "disabled",
        RateConvertTarget::SpdifOpticalOutputPair0 => "spdif-opt-output-1/2",
        RateConvertTarget::SpdifCoaxialOutputPair0 => "spdif-coax-output-1/2",
        RateConvertTarget::SpdifOpticalInputPair0 => "spdif-opt-input-1/2",
        RateConvertTarget::SpdifCoaxialInputPair0 => "spdif-coax-input-1/2",
    }
}

fn rate_convert_rate_to_str(rate: &RateConvertRate) -> &str {
    match rate {
        RateConvertRate::R44100 => "44100",
        RateConvertRate::R48000 => "48000",
        RateConvertRate::R88200 => "88200",
        RateConvertRate::R96000 => "96000",
        RateConvertRate::R176400 => "176400",
        RateConvertRate::R192000 => "192000",
    }
}

impl ConvertCtl {
    const FORMAT_CONVERT_TARGETS: [FormatConvertTarget; 9] = [
        FormatConvertTarget::Disabled,
        FormatConvertTarget::AnalogInputPair0,
        FormatConvertTarget::AnalogInputPair1,
        FormatConvertTarget::AnalogInputPair2,
        FormatConvertTarget::AnalogInputPair3,
        FormatConvertTarget::SpdifOpticalInputPair0,
        FormatConvertTarget::SpdifCoaxialInputPair0,
        FormatConvertTarget::SpdifCoaxialOutputPair0,
        FormatConvertTarget::SpdifOpticalOutputPair0,
     ];

    const RATE_CONVERT_TARGETS: [RateConvertTarget; 5] = [
        RateConvertTarget::Disabled,
        RateConvertTarget::SpdifOpticalOutputPair0,
        RateConvertTarget::SpdifCoaxialOutputPair0,
        RateConvertTarget::SpdifOpticalInputPair0,
        RateConvertTarget::SpdifCoaxialInputPair0,
    ];

    const RATE_CONVERT_RATES: [RateConvertRate; 6] = [
        RateConvertRate::R44100,
        RateConvertRate::R48000,
        RateConvertRate::R88200,
        RateConvertRate::R96000,
        RateConvertRate::R176400,
        RateConvertRate::R192000,
    ];

    fn load_params(
        &mut self,
        card_cntr: &mut CardCntr,
        avc: &mut BebobAvc,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let labels: Vec<&str> = Self::FORMAT_CONVERT_TARGETS.iter()
            .map(|t| format_convert_target_to_str(t))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, FORMAT_CONVERT_TARGET_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let labels: Vec<&str> = Self::RATE_CONVERT_TARGETS.iter()
            .map(|t| rate_convert_target_to_str(t))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, RATE_CONVERT_TARGET_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let labels: Vec<&str> = Self::RATE_CONVERT_RATES.iter()
            .map(|r| rate_convert_rate_to_str(r))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, RATE_CONVERT_RATE_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, CD_MODE_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        avc.init_params(&mut self.0, timeout_ms)
    }

    fn read_params(
        &mut self,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            FORMAT_CONVERT_TARGET_NAME => {
                let pos = Self::FORMAT_CONVERT_TARGETS.iter()
                    .position(|t| t.eq(&self.0.format_target))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            RATE_CONVERT_TARGET_NAME => {
                let pos = Self::RATE_CONVERT_TARGETS.iter()
                    .position(|t| t.eq(&self.0.rate_target))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            RATE_CONVERT_RATE_NAME => {
                let pos = Self::RATE_CONVERT_RATES.iter()
                    .position(|t| t.eq(&self.0.converted_rate))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            CD_MODE_NAME => {
                elem_value.set_bool(&[self.0.cd_mode]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write_params(
        &mut self,
        avc: &mut BebobAvc,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            FORMAT_CONVERT_TARGET_NAME => {
                let mut vals = [0];
                elem_value.get_enum(&mut vals);
                let &target = Self::FORMAT_CONVERT_TARGETS.iter()
                    .nth(vals[0] as usize)
                    .ok_or_else(|| {
                        let msg = format!("Invalid index of format convert target: {}",
                                          vals[0]);
                        Error::new(FileError::Inval, &msg)
                    })?;
                let mut params = self.0.clone();
                params.format_target = target;
                avc.update_params(&params, &mut self.0, timeout_ms)
                    .map(|_| true)
            }
            CD_MODE_NAME => {
                let mut vals = [false];
                elem_value.get_bool(&mut vals);
                let mut params = self.0.clone();
                params.cd_mode = vals[0];
                avc.update_params(&params, &mut self.0, timeout_ms)
                    .map(|_| true)
            }
            RATE_CONVERT_TARGET_NAME => {
                let mut vals = [0];
                elem_value.get_enum(&mut vals);
                let &target = Self::RATE_CONVERT_TARGETS.iter()
                    .nth(vals[0] as usize)
                    .ok_or_else(|| {
                        let msg = format!("Invalid index of rate convert target: {}",
                                          vals[0]);
                        Error::new(FileError::Inval, &msg)
                    })?;
                let mut params = self.0.clone();
                params.rate_target = target;
                avc.update_params(&params, &mut self.0, timeout_ms)
                    .map(|_| true)
            }
            RATE_CONVERT_RATE_NAME => {
                let mut vals = [0];
                elem_value.get_enum(&mut vals);
                let &converted_rate = Self::RATE_CONVERT_RATES.iter()
                    .nth(vals[0] as usize)
                    .ok_or_else(|| {
                        let msg = format!("Invalid index of rate convert target: {}",
                                          vals[0]);
                        Error::new(FileError::Inval, &msg)
                    })?;
                let mut params = self.0.clone();
                params.converted_rate = converted_rate;
                avc.update_params(&params, &mut self.0, timeout_ms)
                    .map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

#[derive(Default)]
struct DisplayCtl(EnsembleDisplayParameters);

fn display_meter_target_to_str(target: &DisplayMeterTarget) -> &str {
    match target {
        DisplayMeterTarget::Output => "output",
        DisplayMeterTarget::Input => "input",
    }
}

const DISPLAY_ENABLE_NAME: &str = "display-enable";
const DISPLAY_ILLUMINATE_NAME: &str = "display-illuminate";
const DISPLAY_TARGET_NAME: &str = "display-target";
const DISPLAY_OVERHOLD_NAME: &str = "display-overhold";

impl DisplayCtl {
    const DISPLAY_METER_TARGETS: [DisplayMeterTarget; 2] = [
        DisplayMeterTarget::Output,
        DisplayMeterTarget::Input,
    ];

    fn load_params(
        &mut self,
        card_cntr: &mut CardCntr,
        avc: &mut BebobAvc,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, DISPLAY_ENABLE_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, DISPLAY_ILLUMINATE_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        let labels: Vec<&str> = Self::DISPLAY_METER_TARGETS.iter()
            .map(|t| display_meter_target_to_str(t))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, DISPLAY_TARGET_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, DISPLAY_OVERHOLD_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        avc.init_params(&mut self.0, timeout_ms)
    }

    fn read_params(
        &mut self,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            DISPLAY_ENABLE_NAME => {
                elem_value.set_bool(&[self.0.enabled]);
                Ok(true)
            }
            DISPLAY_ILLUMINATE_NAME => {
                elem_value.set_bool(&[self.0.illuminate]);
                Ok(true)
            }
            DISPLAY_TARGET_NAME => {
                let pos = Self::DISPLAY_METER_TARGETS.iter()
                    .position(|t| t.eq(&self.0.target))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            DISPLAY_OVERHOLD_NAME => {
                elem_value.set_bool(&[self.0.overhold]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write_params(
        &mut self,
        avc: &mut BebobAvc,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            DISPLAY_ENABLE_NAME => {
                let mut vals = [false];
                elem_value.get_bool(&mut vals);
                let mut params = self.0.clone();
                params.enabled = vals[0];
                avc.update_params(&params, &mut self.0, timeout_ms)
                    .map(|_| true)
            }
            DISPLAY_ILLUMINATE_NAME => {
                let mut vals = [false];
                elem_value.get_bool(&mut vals);
                let mut params = self.0.clone();
                params.illuminate = vals[0];
                avc.update_params(&params, &mut self.0, timeout_ms)
                    .map(|_| true)
            }
            DISPLAY_TARGET_NAME => {
                let mut vals = [0];
                elem_value.get_enum(&mut vals);
                let &target = Self::DISPLAY_METER_TARGETS.iter()
                    .nth(vals[0] as usize)
                    .ok_or_else(|| {
                        let msg = format!("Invalid index of display meter mode: {}", vals[0]);
                        Error::new(FileError::Inval, &msg)
                    })?;
                let mut params = self.0.clone();
                params.target = target;
                avc.update_params(&params, &mut self.0, timeout_ms)
                    .map(|_| true)
            }
            DISPLAY_OVERHOLD_NAME => {
                let mut vals = [false];
                elem_value.get_bool(&mut vals);
                let mut params = self.0.clone();
                params.overhold = vals[0];
                avc.update_params(&params, &mut self.0, timeout_ms)
                    .map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

#[derive(Default)]
struct InputCtl(EnsembleInputParameters, Vec<ElemId>);

const INPUT_LIMIT_NAME: &str = "input-limit";
const INPUT_LEVEL_NAME: &str = "input-level";
const MIC_GAIN_NAME: &str = "mic-gain";
const MIC_PHANTOM_NAME: &str = "mic-phantom";
const MIC_POLARITY_NAME: &str = "mic-polarity";
const INPUT_OPT_IFACE_MODE_NAME: &str = "input-optical-mode";

fn input_nominal_level_to_str(level: &InputNominalLevel) -> &str {
    match level {
        InputNominalLevel::Professional => "+4dB",
        InputNominalLevel::Consumer => "-10dB",
        InputNominalLevel::Microphone => "Mic",
    }
}

fn opt_iface_mode_to_str(mode: &OptIfaceMode) -> &str {
    match mode {
        OptIfaceMode::Spdif => "S/PDIF",
        OptIfaceMode::Adat => "ADAT/SMUX",
    }
}

const OPT_IFACE_MODES: [OptIfaceMode; 2] = [
    OptIfaceMode::Spdif,
    OptIfaceMode::Adat,
];

impl InputCtl {
    const INPUT_LABELS: &'static [&'static str] = &[
        "analog-1", "analog-2", "analog-3", "analog-4", "analog-5", "analog-6", "analog-7",
        "analog-8",
    ];
    const MIC_LABELS: &'static [&'static str] = &["mci-1", "mic-2", "mic-3", "mic-4"];

    const NOMINAL_LEVELS: [InputNominalLevel; 3] = [
        InputNominalLevel::Professional,
        InputNominalLevel::Consumer,
        InputNominalLevel::Microphone,
    ];

    fn load_params(
        &mut self,
        card_cntr: &mut CardCntr,
        avc: &mut BebobAvc,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, INPUT_LIMIT_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, Self::INPUT_LABELS.len(), true)?;

        let labels: Vec<&str> = Self::NOMINAL_LEVELS.iter()
            .map(|l| input_nominal_level_to_str(l))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, INPUT_LEVEL_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, Self::INPUT_LABELS.len(), &labels, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIC_GAIN_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                EnsembleInputParameters::GAIN_MIN as i32,
                EnsembleInputParameters::GAIN_MAX as i32,
                EnsembleInputParameters::GAIN_STEP as i32,
                Self::MIC_LABELS.len(),
                None,
                true,
            )
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIC_PHANTOM_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, Self::MIC_LABELS.len(), true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIC_POLARITY_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, Self::MIC_LABELS.len(), true)?;

        let labels: Vec<&str> = OPT_IFACE_MODES.iter()
            .map(|m| opt_iface_mode_to_str(m))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, INPUT_OPT_IFACE_MODE_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        avc.init_params(&mut self.0, timeout_ms)
    }

    fn read_params(
        &mut self,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            INPUT_LIMIT_NAME => {
                elem_value.set_bool(&self.0.limits);
                Ok(true)
            }
            INPUT_LEVEL_NAME => {
                ElemValueAccessor::<u32>::set_vals(elem_value, Self::INPUT_LABELS.len(), |idx| {
                    let pos = Self::NOMINAL_LEVELS.iter()
                        .position(|l| l.eq(&self.0.levels[idx]))
                        .unwrap();
                    Ok(pos as u32)
                })
                .map(|_| true)
            }
            MIC_GAIN_NAME => {
                let vals: Vec<i32> = self.0.gains.iter()
                    .map(|&val| val as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            MIC_PHANTOM_NAME => {
                elem_value.set_bool(&self.0.phantoms);
                Ok(true)
            }
            MIC_POLARITY_NAME => {
                elem_value.set_bool(&self.0.polarities);
                Ok(true)
            }
            INPUT_OPT_IFACE_MODE_NAME => {
                let pos = OPT_IFACE_MODES.iter()
                    .position(|m| m.eq(&self.0.opt_iface_mode))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write_params(
        &mut self,
        avc: &mut BebobAvc,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            INPUT_LIMIT_NAME => {
                let mut params = self.0.clone();
                elem_value.get_bool(&mut params.limits);
                avc.update_params(&params, &mut self.0, timeout_ms)
                    .map(|_| true)
            }
            INPUT_LEVEL_NAME => {
                let mut vals = [0; Self::INPUT_LABELS.len()];
                elem_value.get_enum(&mut vals);
                let mut params = self.0.clone();
                params.levels.iter_mut()
                    .zip(vals.iter())
                    .try_for_each(|(level, &val)| {
                        Self::NOMINAL_LEVELS.iter()
                            .nth(val as usize)
                            .ok_or_else(|| {
                                let msg = format!("Invalid index of input nominal level: {}", val);
                                Error::new(FileError::Inval, &msg)
                            })
                            .map(|&l| *level = l)
                    })?;
                avc.update_params(&params, &mut self.0, timeout_ms)
                    .map(|_| true)
            }
            MIC_GAIN_NAME => {
                let mut vals = [0; Self::MIC_LABELS.len()];
                elem_value.get_int(&mut vals);
                let mut params = self.0.clone();
                params.gains.iter_mut()
                    .enumerate()
                    .for_each(|(i, gain)| *gain = vals[i] as u8);
                avc.update_params(&params, &mut self.0, timeout_ms)
                    .map(|_| true)
            }
            MIC_PHANTOM_NAME => {
                let mut params = self.0.clone();
                elem_value.get_bool(&mut params.phantoms);
                avc.update_params(&params, &mut self.0, timeout_ms)
                    .map(|_| true)
            }
            MIC_POLARITY_NAME => {
                let mut params = self.0.clone();
                elem_value.get_bool(&mut params.polarities);
                avc.update_params(&params, &mut self.0, timeout_ms)
                    .map(|_| true)
            }
            INPUT_OPT_IFACE_MODE_NAME => {
                let mut vals = [0];
                elem_value.set_enum(&mut vals);
                let &mode = OPT_IFACE_MODES.iter()
                    .nth(vals[0] as usize)
                    .ok_or_else(|| {
                        let msg = format!("Invalid index of optical iface mode: {}", vals[0]);
                        Error::new(FileError::Inval, &msg)
                    })?;
                let mut params = self.0.clone();
                params.opt_iface_mode = mode;
                avc.update_params(&params, &mut self.0, timeout_ms)
                    .map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

fn output_nominal_level_to_str(level: &OutputNominalLevel) -> &str {
    match level {
        OutputNominalLevel::Professional => "+4dB",
        OutputNominalLevel::Consumer => "-10dB",
    }
}

#[derive(Default)]
struct OutputCtl(EnsembleOutputParameters, Vec<ElemId>);

const OUTPUT_LEVEL_NAME: &str = "output-level";
const OUTPUT_VOL_NAME: &str = "output-volume";
const HP_VOL_NAME: &str = "headphone-volume";
const OUTPUT_OPT_IFACE_MODE_NAME: &str = "output-optical-mode";

impl<'a> OutputCtl {
    const OUT_LABELS: [&'static str; 8] = [
        "analog-1", "analog-2", "analog-3", "analog-4", "analog-5", "analog-6", "analog-7",
        "analog-8",
    ];

    const HP_LABELS: [&'static str; 2] = ["headphone-1/2", "headphone-3/4"];

    const NOMINAL_LEVELS: [OutputNominalLevel; 2] = [
        OutputNominalLevel::Professional,
        OutputNominalLevel::Consumer,
    ];

    fn load_params(
        &mut self,
        card_cntr: &mut CardCntr,
        avc: &mut BebobAvc,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let labels: Vec<&str> = Self::NOMINAL_LEVELS.iter()
            .map(|l| output_nominal_level_to_str(l))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, OUTPUT_LEVEL_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, Self::OUT_LABELS.len(), &labels, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, OUTPUT_VOL_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                EnsembleOutputParameters::VOL_MIN as i32,
                EnsembleOutputParameters::VOL_MAX as i32,
                EnsembleOutputParameters::VOL_STEP as i32,
                1,
                None,
                true,
            )
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, HP_VOL_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                EnsembleOutputParameters::VOL_MIN as i32,
                EnsembleOutputParameters::VOL_MAX as i32,
                EnsembleOutputParameters::VOL_STEP as i32,
                Self::HP_LABELS.len(),
                None,
                true,
            )
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        let labels: Vec<&str> = OPT_IFACE_MODES.iter()
            .map(|m| opt_iface_mode_to_str(m))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, OUTPUT_OPT_IFACE_MODE_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        avc.init_params(&mut self.0, timeout_ms)
    }

    fn read_params(
        &mut self,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            OUTPUT_LEVEL_NAME => {
                ElemValueAccessor::<u32>::set_vals(elem_value, Self::OUT_LABELS.len(), |i| {
                    let pos = Self::NOMINAL_LEVELS.iter()
                        .position(|l| l.eq(&self.0.levels[i]))
                        .unwrap();
                    Ok(pos as u32)
                })
                .map(|_| true)
            }
            OUTPUT_VOL_NAME => {
                elem_value.set_int(&[self.0.vol as i32]);
                Ok(true)
            }
            HP_VOL_NAME => {
                let vals: Vec<i32> = self.0.headphone_vols.iter()
                    .map(|&vol| vol as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            OUTPUT_OPT_IFACE_MODE_NAME => {
                let pos = OPT_IFACE_MODES.iter()
                    .position(|m| m.eq(&self.0.opt_iface_mode))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write_params(
        &mut self,
        avc: &mut BebobAvc,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            OUTPUT_LEVEL_NAME => {
                let mut vals = [0; Self::OUT_LABELS.len()];
                elem_value.get_enum(&mut vals);
                let mut params = self.0.clone();
                params.levels.iter_mut()
                    .zip(vals.iter())
                    .try_for_each(|(level, &val)| {
                        Self::NOMINAL_LEVELS.iter()
                            .nth(val as usize)
                            .ok_or_else(|| {
                                let msg = format!("Invalid value for index of input nominal level: {}",
                                                  val);
                                Error::new(FileError::Inval, &msg)
                            })
                            .map(|&l| *level = l)
                    })?;
                avc.update_params(&params, &mut self.0, timeout_ms)
                    .map(|_| true)
            }
            OUTPUT_VOL_NAME => {
                let mut vals = [0];
                elem_value.get_int(&mut vals);
                let mut params = self.0.clone();
                params.vol = vals[0] as u8;
                avc.update_params(&params, &mut self.0, timeout_ms)
                    .map(|_| true)
            }
            HP_VOL_NAME => {
                let mut vals = [0; Self::HP_LABELS.len()];
                elem_value.get_int(&mut vals);
                let mut params = self.0.clone();
                params.headphone_vols.iter_mut()
                    .zip(vals.iter())
                    .for_each(|(vol, &val)| *vol = val as u8);
                avc.update_params(&params, &mut self.0, timeout_ms)
                    .map(|_| true)
            }
            OUTPUT_OPT_IFACE_MODE_NAME => {
                let mut vals = [0];
                elem_value.get_enum(&mut vals);
                let mut params = self.0.clone();
                let &mode = OPT_IFACE_MODES.iter()
                    .nth(vals[0] as usize)
                    .ok_or_else(|| {
                        let msg = format!("Invalid value for index of input nominal level: {}",
                                          vals[0]);
                        Error::new(FileError::Inval, &msg)
                    })?;
                params.opt_iface_mode = mode;
                avc.update_params(&params, &mut self.0, timeout_ms)
                    .map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

const CAPTURE_SOURCE_NAME: &str = "capture-source";

#[derive(Default)]
struct RouteCtl(EnsembleSourceParameters);

impl RouteCtl {
    const OUTPUT_LABELS: [&'static str; 18] = [
        "analog-output-1", "analog-output-2", "analog-output-3", "analog-output-4",
        "analog-output-5", "analog-output-6", "analog-output-7", "analog-output-8",
        "spdif-output-1", "spdif-output-2",
        "adat-output-1", "adat-output-2", "adat-output-3", "adat-output-4",
        "adat-output-5", "adat-output-6", "adat-output-7", "adat-output-8",
    ];

    const CAPTURE_LABELS: [&'static str; 18] = [
        "stream-output-1", "stream-output-2", "stream-output-3", "stream-output-4",
        "stream-output-5", "stream-output-6", "stream-output-7", "stream-output-8",
        "stream-output-9", "stream-output-10", "stream-output-11", "stream-output-12",
        "stream-output-13", "stream-output-14", "stream-output-15", "stream-output-16",
        "stream-output-17", "stream-output-18",
    ];

    const HEADPHONE_LABELS: [&'static str; 2] = [
        "headpone-3/4", "headpone-1/2",
    ];

    const OUTPUT_SOURCE_LABELS: [&'static str; 40] = [
        "analog-input-1", "analog-input-2", "analog-input-3", "analog-input-4",
        "analog-input-5", "analog-input-6", "analog-input-7", "analog-input-8",
        "stream-input-1", "stream-input-2", "stream-input-3", "stream-input-4",
        "stream-input-5", "stream-input-6", "stream-input-7", "stream-input-8",
        "stream-input-9", "stream-input-10", "stream-input-11", "stream-input-12",
        "stream-input-13", "stream-input-14", "stream-input-15", "stream-input-16",
        "stream-input-17", "stream-input-18",
        "spdif-input-1", "spdif-input-2",
        "adat-input-1", "adat-input-2", "adat-input-3", "adat-input-4",
        "adat-input-5", "adat-input-6", "adat-input-7", "adat-input-8",
        "mixer-output-1", "mixer-output-2", "mixer-output-3", "mixer-output-4",
    ];

    const CAPTURE_SOURCE_LABELS: [&'static str; 18] = [
        "analog-input-1", "analog-input-2", "analog-input-3", "analog-input-4",
        "analog-input-5", "analog-input-6", "analog-input-7", "analog-input-8",
        "spdif-input-1", "spdif-input-2",
        "adat-input-1", "adat-input-2", "adat-input-3", "adat-input-4",
        "adat-input-5", "adat-input-6", "adat-input-7", "adat-input-8",
    ];

    const HEADPHONE_SOURCE_LABELS: [&'static str; 6] = [
        "analog-output-1/2",
        "analog-output-3/4",
        "analog-output-5/6",
        "analog-output-7/8",
        "spdif-output-1/2",
        "none",
    ];

    fn load_params(
        &mut self,
        card_cntr: &mut CardCntr,
        avc: &mut BebobAvc,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, OUT_SRC_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, Self::OUTPUT_LABELS.len(),
                                         &Self::OUTPUT_SOURCE_LABELS, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, CAPTURE_SOURCE_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, Self::CAPTURE_LABELS.len(),
                                         &Self::CAPTURE_SOURCE_LABELS, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, HP_SRC_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, Self::HEADPHONE_LABELS.len(),
                                         &Self::HEADPHONE_SOURCE_LABELS, None, true)?;

        avc.init_params(&mut self.0, timeout_ms)
    }

    fn read_params(
        &mut self,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            OUT_SRC_NAME => {
                let vals: Vec<u32> = self.0.output_sources.iter()
                    .map(|&val| val as u32)
                    .collect();
                elem_value.set_enum(&vals);
                Ok(true)
            }
            CAPTURE_SOURCE_NAME => {
                let vals: Vec<u32> = self.0.capture_sources.iter()
                    .map(|&val| val as u32)
                    .collect();
                elem_value.set_enum(&vals);
                Ok(true)
            }
            HP_SRC_NAME => {
                let vals: Vec<u32> = self.0.headphone_sources.iter()
                    .map(|&val| val as u32)
                    .collect();
                elem_value.set_enum(&vals);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write_params(
        &mut self,
        avc: &mut BebobAvc,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            OUT_SRC_NAME => {
                let mut vals = [0; Self::OUTPUT_LABELS.len()];
                elem_value.get_enum(&mut vals);
                let mut params = self.0.clone();
                params.output_sources.iter_mut()
                    .zip(vals.iter())
                    .for_each(|(src, &val)| *src = val as usize);
                avc.update_params(&params, &mut self.0, timeout_ms)
                    .map(|_| true)
            }
            CAPTURE_SOURCE_NAME => {
                let mut vals = [0; Self::CAPTURE_LABELS.len()];
                elem_value.get_enum(&mut vals);
                let mut params = self.0.clone();
                params.capture_sources.iter_mut()
                    .zip(vals.iter())
                    .for_each(|(src, &val)| *src = val as usize);
                avc.update_params(&params, &mut self.0, timeout_ms)
                    .map(|_| true)
            }
            HP_SRC_NAME => {
                let mut vals = [0; Self::HEADPHONE_LABELS.len()];
                elem_value.get_enum(&mut vals);
                let mut params = self.0.clone();
                params.headphone_sources.iter_mut()
                    .zip(vals.iter())
                    .for_each(|(src, &val)| *src = val as usize);
                avc.update_params(&params, &mut self.0, timeout_ms)
                    .map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

#[derive(Default)]
struct MixerCtl(EnsembleMixerParameters);

const MIXER_SRC_GAIN_NAME: &str = "mixer-source-gain";

impl MixerCtl {
    const MIXER_LABELS: [&'static str; 4] = [
        "mixer-output-1", "mixer-output-2", "mixer-output-3", "mixer-output-4",
    ];

    const MIXER_SRC_LABELS: [&'static str; 36] = [
        "analog-input-1", "analog-input-2", "analog-input-3", "analog-input-4",
        "analog-input-5", "analog-input-6", "analog-input-7", "analog-input-8",
        "stream-input-1", "stream-input-2", "stream-input-3", "stream-input-4",
        "stream-input-5", "stream-input-6", "stream-input-7", "stream-input-8",
        "stream-input-9", "stream-input-10", "stream-input-11", "stream-input-12",
        "stream-input-13", "stream-input-14", "stream-input-15", "stream-input-16",
        "stream-input-17", "stream-input-18",
        "adat-input-1", "adat-input-2", "adat-input-3", "adat-input-4",
        "adat-input-5", "adat-input-6", "adat-input-7", "adat-input-8",
        "spdif-input-1", "spdif-input-2",
    ];

    const GAIN_TLV: DbInterval = DbInterval { min: -4800, max: 0, linear: false, mute_avail: true };

    fn load_params(
        &mut self,
        card_cntr: &mut CardCntr,
        avc: &mut BebobAvc,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIXER_SRC_GAIN_NAME, 0);
        let _ = card_cntr
            .add_int_elems(
                &elem_id,
                Self::MIXER_LABELS.len(),
                EnsembleMixerParameters::GAIN_MIN as i32,
                EnsembleMixerParameters::GAIN_MAX as i32,
                EnsembleMixerParameters::GAIN_STEP as i32,
                Self::MIXER_SRC_LABELS.len(),
                Some(&Into::<Vec<u32>>::into(Self::GAIN_TLV)),
                true,
            )?;

        avc.init_params(&mut self.0, timeout_ms)
    }

    fn read_params(
        &mut self,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            MIXER_SRC_GAIN_NAME => {
                let index = elem_id.get_index() as usize;
                let vals: Vec<i32> = self.0.src_gains[index].iter()
                    .map(|&val| val as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write_params(
        &mut self,
        avc: &mut BebobAvc,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            MIXER_SRC_GAIN_NAME => {
                let mut vals = [0; Self::MIXER_SRC_LABELS.len()];
                elem_value.get_int(&mut vals);

                let index = elem_id.get_index() as usize;

                let mut params = self.0.clone();
                params.src_gains[index].iter_mut()
                    .zip(vals.iter())
                    .for_each(|(gain, &val)| *gain = val as i16);
                avc.update_params(&params, &mut self.0, timeout_ms)
                    .map(|_| true)
            }
            _ => Ok(false),
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
}
