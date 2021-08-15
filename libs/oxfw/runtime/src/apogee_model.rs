// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::{Error, FileError};

use hinawa::{FwReq, FwFcp, FwFcpExt};
use hinawa::{SndUnit, SndUnitExt};

use alsactl::{ElemId, ElemIfaceType, ElemValue, ElemValueExt, ElemValueExtManual};

use core::card_cntr::*;
use core::elem_value_accessor::*;

use ta1394::{Ta1394Avc, AvcAddr, AvcSubunitType};
use ta1394::general::UnitInfo;

use oxfw_protocols::apogee::*;

use super::common_ctl::CommonCtl;

#[derive(Default, Debug)]
pub struct ApogeeModel{
    req: FwReq,
    avc: FwFcp,
    company_id: [u8; 3],
    common_ctl: CommonCtl,
    meter_ctl: MeterCtl,
    knob_ctl: KnobCtl,
    output_ctl: OutputCtl,
    input_ctl: InputCtl,
    mixer_ctl: MixerCtl,
    display_ctl: DisplayCtl,
}

const TIMEOUT_MS: u32 = 50;

impl ApogeeModel {
    const FCP_TIMEOUT_MS: u32 = 100;
}

impl CtlModel<hinawa::SndUnit> for ApogeeModel {
    fn load(&mut self, unit: &mut hinawa::SndUnit, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.avc.bind(&unit.get_node())?;

        let mut op = UnitInfo{
            unit_type: AvcSubunitType::Reserved(0xff),
            unit_id: 0xff,
            company_id: [0xff;3],
        };
        self.avc.status(&AvcAddr::Unit, &mut op, 100)?;
        self.company_id.copy_from_slice(&op.company_id);

        self.common_ctl.load(&self.avc, card_cntr, Self::FCP_TIMEOUT_MS)?;
        self.meter_ctl.load_state(card_cntr, unit, &mut self.req, TIMEOUT_MS)?;
        self.knob_ctl.load_state(card_cntr, &mut self.avc, Self::FCP_TIMEOUT_MS)?;
        self.output_ctl.load_params(card_cntr, &mut self.avc, Self::FCP_TIMEOUT_MS)?;
        self.input_ctl.load_params(card_cntr, &mut self.avc, Self::FCP_TIMEOUT_MS)?;
        self.mixer_ctl.load_params(card_cntr)?;
        self.display_ctl.load_params(card_cntr)?;

        Ok(())
    }

    fn read(&mut self, _: &mut hinawa::SndUnit, elem_id: &alsactl::ElemId,
            elem_value: &mut alsactl::ElemValue)
        -> Result<bool, Error>
    {
        if self.common_ctl.read(&self.avc, elem_id, elem_value, Self::FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.meter_ctl.read_state(elem_id, elem_value)? {
            Ok(true)
        } else if self.knob_ctl.read_state(elem_id, elem_value)? {
            Ok(true)
        } else if self.output_ctl.read_params(&mut self.avc, elem_id, elem_value, Self::FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.input_ctl.read_params(&mut self.avc, elem_id, elem_value, Self::FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.mixer_ctl.read_params(&mut self.avc, elem_id, elem_value, Self::FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.display_ctl.read_params(&mut self.avc, elem_id, elem_value, Self::FCP_TIMEOUT_MS)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(&mut self, unit: &mut hinawa::SndUnit, elem_id: &alsactl::ElemId, old: &alsactl::ElemValue,
             new: &alsactl::ElemValue)
        -> Result<bool, Error>
    {
        if self.common_ctl.write(unit, &self.avc, elem_id, new, Self::FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.output_ctl.write_params(&mut self.avc, elem_id, new, Self::FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.input_ctl.write_params(&mut self.avc, elem_id, old, new, Self::FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.mixer_ctl.write_params(&mut self.avc, elem_id, old, new, Self::FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.display_ctl.write_params(&mut self.avc, elem_id, new, Self::FCP_TIMEOUT_MS)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl MeasureModel<hinawa::SndUnit> for ApogeeModel {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<alsactl::ElemId>) {
        elem_id_list.extend_from_slice(&self.meter_ctl.2);
        elem_id_list.extend_from_slice(&self.knob_ctl.1);
        elem_id_list.extend_from_slice(&self.output_ctl.2);
        elem_id_list.extend_from_slice(&self.input_ctl.1);
    }

    fn measure_states(&mut self, unit: &mut hinawa::SndUnit) -> Result<(), Error> {
        self.meter_ctl.measure_state(unit, &mut self.req, TIMEOUT_MS)?;
        self.knob_ctl.measure_state(&mut self.avc, TIMEOUT_MS)?;

        self.output_ctl.0 = self.knob_ctl.0.output_mute;
        self.output_ctl.1 = self.knob_ctl.0.output_volume;
        self.input_ctl.0[0] = self.knob_ctl.0.input_gains[0];
        self.input_ctl.0[1] = self.knob_ctl.0.input_gains[1];

        Ok(())
    }

    fn measure_elem(&mut self, _: &hinawa::SndUnit, elem_id: &alsactl::ElemId,
                    elem_value: &mut alsactl::ElemValue)
        -> Result<bool, Error>
    {
        if self.meter_ctl.read_state(elem_id, elem_value)? {
            Ok(true)
        } else if self.knob_ctl.read_state(elem_id, elem_value)? {
            Ok(true)
        } else if self.output_ctl.measure_params(elem_id, elem_value)? {
            Ok(true)
        } else if self.input_ctl.measure_params(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl NotifyModel<hinawa::SndUnit, bool> for ApogeeModel {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<alsactl::ElemId>) {
        elem_id_list.extend_from_slice(&self.common_ctl.notified_elem_list);
    }

    fn parse_notification(&mut self, _: &mut hinawa::SndUnit, _: &bool) -> Result<(), Error> {
        Ok(())
    }

    fn read_notified_elem(&mut self, _: &hinawa::SndUnit, elem_id: &alsactl::ElemId, elem_value: &mut alsactl::ElemValue)
        -> Result<bool, Error>
    {
        self.common_ctl.read(&self.avc, elem_id, elem_value, Self::FCP_TIMEOUT_MS)
    }
}

#[derive(Default, Debug)]
struct MeterCtl(DuetFwInputMeterState, DuetFwMixerMeterState, Vec<ElemId>);

const ANALOG_INPUT_METER_NAME: &str = "analog-input-meters";
const STREAM_INPUT_METER_NAME: &str = "stream-input-meters";
const MIXER_OUTPUT_METER_NAME: &str = "mixer-output-meters";

impl MeterCtl {
    const ANALOG_INPUT_LABELS: [&'static str; 2] = ["analog-input-1", "analog-input-2"];
    const STREAM_INPUT_LABELS: [&'static str; 2] = ["stream-input-1", "stream-input-2"];
    const MIXER_OUTPUT_LABELS: [&'static str; 2] = ["mixer-output-1", "mixer-output-2"];

    fn load_state(
        &mut self,
        card_cntr: &mut CardCntr,
        unit: &mut SndUnit,
        req: &mut FwReq,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, ANALOG_INPUT_METER_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                DuetFwInputMeterProtocol::LEVEL_MIN,
                DuetFwInputMeterProtocol::LEVEL_MAX,
                DuetFwInputMeterProtocol::LEVEL_STEP,
                Self::ANALOG_INPUT_LABELS.len(),
                None,
                false,
            )
            .map(|mut elem_id_list| self.2.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, STREAM_INPUT_METER_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                DuetFwMixerMeterProtocol::LEVEL_MIN,
                DuetFwMixerMeterProtocol::LEVEL_MAX,
                DuetFwMixerMeterProtocol::LEVEL_STEP,
                Self::STREAM_INPUT_LABELS.len(),
                None,
                false,
            )
            .map(|mut elem_id_list| self.2.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIXER_OUTPUT_METER_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                DuetFwMixerMeterProtocol::LEVEL_MIN,
                DuetFwMixerMeterProtocol::LEVEL_MAX,
                DuetFwMixerMeterProtocol::LEVEL_STEP,
                Self::MIXER_OUTPUT_LABELS.len(),
                None,
                false,
            )
            .map(|mut elem_id_list| self.2.append(&mut elem_id_list))?;

        self.measure_state(unit, req, timeout_ms)
    }

    fn measure_state(
        &mut self,
        unit: &mut SndUnit,
        req: &mut FwReq,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let node = &mut unit.get_node();
        DuetFwInputMeterProtocol::read_state(req, node, &mut self.0, timeout_ms)?;
        DuetFwMixerMeterProtocol::read_state(req, node, &mut self.1, timeout_ms)?;
        Ok(())
    }

    fn read_state(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            ANALOG_INPUT_METER_NAME => {
                elem_value.set_int(&self.0.0);
                Ok(true)
            }
            STREAM_INPUT_METER_NAME => {
                elem_value.set_int(&self.1.stream_inputs);
                Ok(true)
            }
            MIXER_OUTPUT_METER_NAME => {
                elem_value.set_int(&self.1.mixer_outputs);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

#[derive(Default, Debug)]
struct KnobCtl(DuetFwKnobState, Vec<ElemId>);

fn knob_target_to_str(target: &DuetFwKnobTarget) -> &str {
    match target {
        DuetFwKnobTarget::OutputPair0 => "OUT",
        DuetFwKnobTarget::InputPair0 => "IN-1",
        DuetFwKnobTarget::InputPair1 => "IN-2",
    }
}

const KNOB_TARGET_NAME: &'static str = "knob-target";

impl KnobCtl {
    const KNOB_TARGETS: [DuetFwKnobTarget; 3] = [
        DuetFwKnobTarget::OutputPair0,
        DuetFwKnobTarget::InputPair0,
        DuetFwKnobTarget::InputPair1,
    ];

    fn load_state(
        &mut self,
        card_cntr: &mut CardCntr,
        fcp: &mut FwFcp,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let labels: Vec<&str> = Self::KNOB_TARGETS.iter()
            .map(|t| knob_target_to_str(t))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, KNOB_TARGET_NAME, 0);
        card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, false)
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        self.measure_state(fcp, timeout_ms)
    }

    fn measure_state(&mut self, fcp: &mut FwFcp, timeout_ms: u32) -> Result<(), Error> {
        DuetFwKnobProtocol::read_state(fcp, &mut self.0, timeout_ms)
    }

    fn read_state(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            KNOB_TARGET_NAME => {
                let pos = Self::KNOB_TARGETS.iter()
                    .position(|t| t.eq(&self.0.target))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

#[derive(Default, Debug)]
struct OutputCtl(bool, u8, Vec<ElemId>);

fn output_source_to_str(src: &DuetFwOutputSource) -> &str {
    match src {
        DuetFwOutputSource::StreamInputPair0 => "stream-input-1/2",
        DuetFwOutputSource::MixerOutputPair0 => "mixer-output-1/2",
    }
}

fn output_nominal_level_to_str(level: &DuetFwOutputNominalLevel) -> &str {
    match level {
        DuetFwOutputNominalLevel::Instrument => "instrument",
        DuetFwOutputNominalLevel::Consumer => "-10dB",
    }
}

fn output_mute_mode_to_str(mode: &DuetFwOutputMuteMode) -> &str {
    match mode {
        DuetFwOutputMuteMode::Never => "never",
        DuetFwOutputMuteMode::Normal => "normal",
        DuetFwOutputMuteMode::Swapped => "swapped",
    }
}

const OUTPUT_MUTE_NAME: &str = "output-mute";
const OUTPUT_VOLUME_NAME: &str = "output-volume";
const OUTPUT_SRC_NAME: &str = "output-source";
const OUTPUT_NOMINAL_LEVEL_NAME: &str = "output-level";
const OUTPUT_MUTE_FOR_LINE_OUT: &str = "mute-for-line-out";
const OUTPUT_MUTE_FOR_HP_OUT: &str = "mute-for-hp-out";

impl OutputCtl {
    const SOURCES: [DuetFwOutputSource; 2] = [
        DuetFwOutputSource::StreamInputPair0,
        DuetFwOutputSource::MixerOutputPair0,
    ];

    const NOMINAL_LEVELS: [DuetFwOutputNominalLevel; 2] = [
        DuetFwOutputNominalLevel::Instrument,
        DuetFwOutputNominalLevel::Consumer,
    ];

    const MUTE_MODES: [DuetFwOutputMuteMode; 3] = [
        DuetFwOutputMuteMode::Never,
        DuetFwOutputMuteMode::Normal,
        DuetFwOutputMuteMode::Swapped,
    ];

    fn load_params(
        &mut self,
        card_cntr: &mut CardCntr,
        avc: &mut FwFcp,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, OUTPUT_MUTE_NAME, 0);
        card_cntr.add_bool_elems(&elem_id, 1, 1, true)
            .map(|mut elem_id_list| self.2.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, OUTPUT_VOLUME_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                DuetFwOutputProtocol::VOLUME_MIN as i32,
                DuetFwOutputProtocol::VOLUME_MAX as i32,
                DuetFwOutputProtocol::VOLUME_STEP as i32,
                1,
                None,
                true,
            )
            .map(|mut elem_id_list| self.2.append(&mut elem_id_list))?;

        let labels: Vec<&str> = Self::SOURCES.iter()
            .map(|s| output_source_to_str(s))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, OUTPUT_SRC_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let labels: Vec<&str> = Self::NOMINAL_LEVELS.iter()
            .map(|l| output_nominal_level_to_str(l))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, OUTPUT_NOMINAL_LEVEL_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let labels: Vec<&str> = Self::MUTE_MODES.iter()
            .map(|m| output_mute_mode_to_str(m))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, OUTPUT_MUTE_FOR_LINE_OUT, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, OUTPUT_MUTE_FOR_HP_OUT, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        DuetFwOutputProtocol::read_mute(avc, &mut self.0, timeout_ms)?;
        DuetFwOutputProtocol::read_volume(avc, &mut self.1, timeout_ms)?;

        Ok(())
    }

    fn read_params(
        &mut self,
        avc: &mut FwFcp,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            OUTPUT_SRC_NAME => {
                let mut src = DuetFwOutputSource::default();
                DuetFwOutputProtocol::read_src(avc, &mut src, timeout_ms)?;
                let pos = Self::SOURCES.iter()
                    .position(|s| s.eq(&src))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            OUTPUT_NOMINAL_LEVEL_NAME => {
                let mut level = DuetFwOutputNominalLevel::default();
                DuetFwOutputProtocol::read_nominal_level(avc, &mut level, timeout_ms)?;
                let pos = Self::NOMINAL_LEVELS.iter()
                    .position(|l| l.eq(&level))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            OUTPUT_MUTE_FOR_LINE_OUT => {
                let mut mode = DuetFwOutputMuteMode::default();
                DuetFwOutputProtocol::read_mute_mode_for_analog_output(avc, &mut mode, timeout_ms)?;
                let pos = Self::MUTE_MODES.iter()
                    .position(|m| m.eq(&mode))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            OUTPUT_MUTE_FOR_HP_OUT => {
                let mut mode = DuetFwOutputMuteMode::default();
                DuetFwOutputProtocol::read_mute_mode_for_hp(avc, &mut mode, timeout_ms)?;
                let pos = Self::MUTE_MODES.iter()
                    .position(|m| m.eq(&mode))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            _ => self.measure_params(elem_id, elem_value),
        }
    }

    fn measure_params(
        &mut self,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            OUTPUT_MUTE_NAME => {
                elem_value.set_bool(&[self.0]);
                Ok(true)
            }
            OUTPUT_VOLUME_NAME => {
                elem_value.set_int(&[self.1 as i32]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write_params(
        &mut self,
        avc: &mut FwFcp,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            OUTPUT_MUTE_NAME => {
                ElemValueAccessor::<bool>::get_val(elem_value, |val| {
                    DuetFwOutputProtocol::write_mute(avc, val, timeout_ms)
                })
                .map(|_| true)
            }
            OUTPUT_VOLUME_NAME => {
                ElemValueAccessor::<i32>::get_val(elem_value, |val| {
                    DuetFwOutputProtocol::write_volume(avc, val as u8, timeout_ms)
                })
                .map(|_| true)
            }
            OUTPUT_SRC_NAME => {
                ElemValueAccessor::<u32>::get_val(elem_value, |val| {
                    let &src = Self::SOURCES.iter()
                        .nth(val as usize)
                        .ok_or_else(|| {
                            let msg = format!("Invalid index for source of output: {}", val);
                            Error::new(FileError::Inval, &msg)
                        })?;
                    DuetFwOutputProtocol::write_src(avc, src, timeout_ms)
                })
                .map(|_| true)
            }
            OUTPUT_NOMINAL_LEVEL_NAME => {
                ElemValueAccessor::<u32>::get_val(elem_value, |val| {
                    let &level = Self::NOMINAL_LEVELS.iter()
                        .nth(val as usize)
                        .ok_or_else(|| {
                            let msg = format!("Invalid index for source of output: {}", val);
                            Error::new(FileError::Inval, &msg)
                        })?;
                    DuetFwOutputProtocol::write_nominal_level(avc, level, timeout_ms)
                })
                .map(|_| true)
            }
            OUTPUT_MUTE_FOR_LINE_OUT => {
                ElemValueAccessor::<u32>::get_val(elem_value, |val| {
                    let &mode = Self::MUTE_MODES.iter()
                        .nth(val as usize)
                        .ok_or_else(|| {
                            let msg = format!("Invalid index for mute modes: {}", val);
                            Error::new(FileError::Inval, &msg)
                        })?;
                    DuetFwOutputProtocol::write_mute_mode_for_analog_output(avc, mode, timeout_ms)
                })
                .map(|_| true)
            }
            OUTPUT_MUTE_FOR_HP_OUT => {
                ElemValueAccessor::<u32>::get_val(elem_value, |val| {
                    let &mode = Self::MUTE_MODES.iter()
                        .nth(val as usize)
                        .ok_or_else(|| {
                            let msg = format!("Invalid index for mute modes: {}", val);
                            Error::new(FileError::Inval, &msg)
                        })?;
                    DuetFwOutputProtocol::write_mute_mode_for_hp(avc, mode, timeout_ms)
                })
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

#[derive(Default, Debug)]
struct InputCtl([u8; 2], Vec<ElemId>);

fn input_source_to_str(src: &DuetFwInputSource) -> &str {
    match src {
        DuetFwInputSource::Xlr => "XLR",
        DuetFwInputSource::Phone => "Phone",
    }
}

fn input_xlr_nominal_level_to_str(level: &DuetFwInputXlrNominalLevel) -> &str {
    match level {
        DuetFwInputXlrNominalLevel::Microphone => "Microphone",
        DuetFwInputXlrNominalLevel::Professional => "+4dBu",
        DuetFwInputXlrNominalLevel::Consumer => "-10dBV",
    }
}

const INPUT_GAIN_NAME: &str = "input-gain";
const INPUT_POLARITY_NAME: &str = "input-polarity";
const INPUT_XLR_NOMINAL_LEVEL_NAME: &str = "input-xlr-nominal-level";
const INPUT_PHANTOM_NAME: &str = "input-phantom";
const INPUT_SOURCE_NAME: &str = "input-source";
const INPUT_CLICKLESS_NAME: &str = "input-clickless";

impl InputCtl {
    const LABELS: [&'static str; 2] = ["analog-input-1", "analog-input-2"];
    const MIC_LABELS: [&'static str; 2] = ["Mic-1", "Mic-2"];

    const SOURCES: [DuetFwInputSource; 2] = [
        DuetFwInputSource::Xlr,
        DuetFwInputSource::Phone,
    ];

    const XLR_NOMINAL_LEVELS: [DuetFwInputXlrNominalLevel; 3] = [
        DuetFwInputXlrNominalLevel::Microphone,
        DuetFwInputXlrNominalLevel::Professional,
        DuetFwInputXlrNominalLevel::Consumer,
    ];

    fn load_params(
        &mut self,
        card_cntr: &mut CardCntr,
        avc: &mut FwFcp,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, INPUT_GAIN_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                DuetFwInputProtocol::GAIN_MIN as i32,
                DuetFwInputProtocol::GAIN_MAX as i32,
                DuetFwInputProtocol::GAIN_STEP as i32,
                Self::LABELS.len(),
                None,
                true,
            )
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, INPUT_POLARITY_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, Self::MIC_LABELS.len(), true)?;

        let labels: Vec<&str> = Self::XLR_NOMINAL_LEVELS.iter()
            .map(|l| input_xlr_nominal_level_to_str(l))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, INPUT_XLR_NOMINAL_LEVEL_NAME, 0);
        let _ = card_cntr
            .add_enum_elems(
                &elem_id,
                1,
                Self::MIC_LABELS.len(),
                &labels,
                None,
                true,
            )?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, INPUT_PHANTOM_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, Self::MIC_LABELS.len(), true)?;

        let labels: Vec<&str> = Self::SOURCES.iter()
            .map(|s| input_source_to_str(s))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, INPUT_SOURCE_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, Self::LABELS.len(), &labels, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, INPUT_CLICKLESS_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        self.0.iter_mut()
            .enumerate()
            .try_for_each(|(i, gain)| DuetFwInputProtocol::read_gain(avc, i, gain, timeout_ms))
    }

    fn read_params(
        &mut self,
        avc: &mut FwFcp,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            INPUT_POLARITY_NAME => {
                ElemValueAccessor::<bool>::set_vals(elem_value, 2, |idx| {
                    let mut val = false;
                    DuetFwInputProtocol::read_polarity(avc, idx, &mut val, timeout_ms)
                        .map(|_| val)
                })
                .map(|_| true)
            }
            INPUT_XLR_NOMINAL_LEVEL_NAME => {
                ElemValueAccessor::<u32>::set_vals(elem_value, 2, |idx| {
                    let mut level = DuetFwInputXlrNominalLevel::default();
                    DuetFwInputProtocol::read_xlr_nominal_level(avc, idx, &mut level, timeout_ms)
                        .map(|_| {
                            Self::XLR_NOMINAL_LEVELS.iter()
                                .position(|l| l.eq(&level))
                                .unwrap() as u32
                        })
                })
                .map(|_| true)
            }
            INPUT_PHANTOM_NAME => {
                ElemValueAccessor::<bool>::set_vals(elem_value, 2, |idx| {
                    let mut enable = false;
                    DuetFwInputProtocol::read_phantom_powering(avc, idx, &mut enable, timeout_ms)
                        .map(|_| enable)
                })
                .map(|_| true)
            }
            INPUT_SOURCE_NAME => {
                ElemValueAccessor::<u32>::set_vals(elem_value, 2, |idx| {
                    let mut src = DuetFwInputSource::default();
                    DuetFwInputProtocol::read_src(avc, idx, &mut src, timeout_ms)?;
                    let pos = Self::SOURCES.iter()
                        .position(|s| s.eq(&src))
                        .unwrap();
                    Ok(pos as u32)
                })
                .map(|_| true)
            }
            INPUT_CLICKLESS_NAME => {
                let mut val = false;
                DuetFwInputProtocol::read_clickless(avc, &mut val, timeout_ms)?;
                elem_value.set_bool(&[val]);
                Ok(true)
            }
            _ => self.measure_params(elem_id, elem_value),
        }
    }

    fn measure_params(
        &mut self,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            INPUT_GAIN_NAME => {
                let vals: Vec<i32> = self.0.iter()
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
        avc: &mut FwFcp,
        elem_id: &ElemId,
        old: &ElemValue,
        new: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            INPUT_GAIN_NAME => {
                ElemValueAccessor::<i32>::get_vals(new, old, 2, |idx, val| {
                    DuetFwInputProtocol::write_gain(avc, idx, val as u8, timeout_ms)
                })
                .map(|_| true)
            }
            INPUT_POLARITY_NAME => {
                ElemValueAccessor::<bool>::get_vals(new, old, 2, |idx, val| {
                    DuetFwInputProtocol::write_polarity(avc, idx, val, timeout_ms)
                })
                .map(|_| true)
            }
            INPUT_XLR_NOMINAL_LEVEL_NAME => {
                ElemValueAccessor::<u32>::get_vals(new, old, 2, |idx, val| {
                    let &level = Self::XLR_NOMINAL_LEVELS.iter()
                        .nth(val as usize)
                        .ok_or_else(|| {
                            let msg = format!("Invalid index for nominal levels: {}", val);
                            Error::new(FileError::Inval, &msg)
                        })?;
                    DuetFwInputProtocol::write_xlr_nominal_level(avc, idx, level, timeout_ms)
                })
                .map(|_| true)
            }
            INPUT_PHANTOM_NAME => {
                ElemValueAccessor::<bool>::get_vals(new, old, 2, |idx, val| {
                    DuetFwInputProtocol::write_phantom_powering(avc, idx, val, timeout_ms)
                })
                .map(|_| true)
            }
            INPUT_SOURCE_NAME => {
                ElemValueAccessor::<u32>::get_vals(new, old, 2, |idx, val| {
                    let &src = Self::SOURCES.iter()
                        .nth(val as usize)
                        .ok_or_else(||{
                            let msg = format!("Invalid index for input sources: {}", val);
                            Error::new(FileError::Inval, &msg)
                        })?;
                    DuetFwInputProtocol::write_src(avc, idx, src, timeout_ms)
                })
                .map(|_| true)
            }
            INPUT_CLICKLESS_NAME => {
                let mut vals = [false];
                new.get_bool(&mut vals);
                DuetFwInputProtocol::write_clickless(avc, vals[0], timeout_ms)
                    .map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

#[derive(Default, Debug)]
struct MixerCtl;

const MIXER_SOURCE_GAIN_NAME: &str = "mixer-source-gain";

impl MixerCtl {
    const DST_LABELS: [&'static str; 2] = ["mixer-output-1", "mixer-output-2"];
    const SRC_LABELS: [&'static str; 4] = [
        "stream-input-1", "stream-input-2", "analog-input-1", "analog-input-2",
    ];

    fn load_params(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIXER_SOURCE_GAIN_NAME, 0);
        let _ = card_cntr
            .add_int_elems(
                &elem_id,
                Self::DST_LABELS.len(),
                DuetFwMixerProtocol::GAIN_MIN as i32,
                DuetFwMixerProtocol::GAIN_MAX as i32,
                DuetFwMixerProtocol::GAIN_STEP as i32,
                Self::SRC_LABELS.len(),
                None,
                true,
            )?;

        Ok(())
    }

    fn read_params(
        &mut self,
        avc: &mut FwFcp,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            MIXER_SOURCE_GAIN_NAME => {
                let dst = elem_id.get_index() as usize;
                ElemValueAccessor::<i32>::set_vals(elem_value, Self::SRC_LABELS.len(), |src| {
                    let mut gain = 0;
                    DuetFwMixerProtocol::read_source_gain(avc, dst, src, &mut gain, timeout_ms)
                        .map(|_| gain as i32)
                })
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }

    fn write_params(
        &mut self,
        avc: &mut FwFcp,
        elem_id: &ElemId,
        old: &ElemValue,
        new: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            MIXER_SOURCE_GAIN_NAME => {
                let dst = elem_id.get_index() as usize;
                ElemValueAccessor::<i32>::get_vals(new, old, Self::SRC_LABELS.len(), |src, gain| {
                    DuetFwMixerProtocol::write_source_gain(avc, dst, src, gain as u16, timeout_ms)
                })
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

#[derive(Default, Debug)]
struct DisplayCtl;

fn display_target_to_str(target: &DuetFwDisplayTarget) -> &str {
    match target {
        DuetFwDisplayTarget::Output => "output",
        DuetFwDisplayTarget::Input => "input",
    }
}

fn display_mode_to_str(mode: &DuetFwDisplayMode) -> &str {
    match mode {
        DuetFwDisplayMode::Independent => "independent",
        DuetFwDisplayMode::FollowingToKnobTarget => "follow-to-knob",
    }
}

fn display_overhold_to_str(mode: &DuetFwDisplayOverhold) -> &str {
    match mode {
        DuetFwDisplayOverhold::Infinite => "infinite",
        DuetFwDisplayOverhold::TwoSeconds => "2seconds",
    }
}

const DISPLAY_TARGET_NAME: &'static str = "meter-target";
const DISPLAY_MODE_NAME: &'static str = "meter-mode";
const DISPLAY_OVERHOLDS_NAME: &'static str = "meter-overhold";

impl DisplayCtl {
    const TARGETS: [DuetFwDisplayTarget; 2] = [
        DuetFwDisplayTarget::Output,
        DuetFwDisplayTarget::Input,
    ];

    const MODES: [DuetFwDisplayMode; 2] = [
        DuetFwDisplayMode::Independent,
        DuetFwDisplayMode::FollowingToKnobTarget,
    ];

    const OVERHOLDS: [DuetFwDisplayOverhold; 2] = [
        DuetFwDisplayOverhold::Infinite,
        DuetFwDisplayOverhold::TwoSeconds,
    ];

    fn load_params(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let labels: Vec<&str> = Self::TARGETS.iter()
            .map(|t| display_target_to_str(t))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, DISPLAY_TARGET_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let labels: Vec<&str> = Self::MODES.iter()
            .map(|t| display_mode_to_str(t))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, DISPLAY_MODE_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let labels: Vec<&str> = Self::OVERHOLDS.iter()
            .map(|t| display_overhold_to_str(t))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, DISPLAY_OVERHOLDS_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        Ok(())
    }

    fn read_params(
        &mut self,
        avc: &mut FwFcp,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            DISPLAY_TARGET_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let mut target = DuetFwDisplayTarget::default();
                    DuetFwDisplayProtocol::read_target(avc, &mut target, timeout_ms)
                        .map(|_| {
                            Self::TARGETS.iter()
                                .position(|t| t.eq(&target))
                                .unwrap() as u32
                        })
                })
                .map(|_| true)
            }
            DISPLAY_MODE_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let mut mode = DuetFwDisplayMode::default();
                    DuetFwDisplayProtocol::read_mode(avc, &mut mode, timeout_ms)
                        .map(|_| {
                            Self::MODES.iter()
                                .position(|m| m.eq(&mode))
                                .unwrap() as u32
                        })
                })
                .map(|_| true)
            }
            DISPLAY_OVERHOLDS_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let mut mode = DuetFwDisplayOverhold::default();
                    DuetFwDisplayProtocol::read_overhold(avc, &mut mode, timeout_ms)
                        .map(|_| {
                            Self::OVERHOLDS.iter()
                                .position(|m| m.eq(&mode))
                                .unwrap() as u32
                        })
                })
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }

    fn write_params(
        &mut self,
        avc: &mut FwFcp,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            DISPLAY_TARGET_NAME => {
                let mut vals = [0];
                elem_value.get_enum(&mut vals);
                let &target = Self::TARGETS.iter()
                    .nth(vals[0] as usize)
                    .ok_or_else(|| {
                        let msg = format!("Invalid index for display targets: {}", vals[0]);
                        Error::new(FileError::Inval, &msg)
                    })?;
                DuetFwDisplayProtocol::write_target(avc, target, timeout_ms)
                    .map(|_| true)
            }
            DISPLAY_MODE_NAME => {
                let mut vals = [0];
                elem_value.get_enum(&mut vals);
                let &mode = Self::MODES.iter()
                    .nth(vals[0] as usize)
                    .ok_or_else(|| {
                        let msg = format!("Invalid index for display modes: {}", vals[0]);
                        Error::new(FileError::Inval, &msg)
                    })?;
                DuetFwDisplayProtocol::write_mode(avc, mode, timeout_ms)
                    .map(|_| true)
            }
            DISPLAY_OVERHOLDS_NAME => {
                let mut vals = [0];
                elem_value.get_enum(&mut vals);
                let &mode = Self::OVERHOLDS.iter()
                    .nth(vals[0] as usize)
                    .ok_or_else(|| {
                        let msg = format!("Invalid index for display overholds: {}", vals[0]);
                        Error::new(FileError::Inval, &msg)
                    })?;
                DuetFwDisplayProtocol::write_overhold(avc, mode, timeout_ms)
                    .map(|_| true)
            }
            _ => Ok(false),
        }
    }
}
