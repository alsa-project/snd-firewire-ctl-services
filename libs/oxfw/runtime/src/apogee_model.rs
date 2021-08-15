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
use super::apogee_ctls::{MixerCtl, InputCtl, DisplayCtl, HwState};

#[derive(Default, Debug)]
pub struct ApogeeModel{
    req: FwReq,
    avc: FwFcp,
    company_id: [u8; 3],
    common_ctl: CommonCtl,
    meter_ctl: MeterCtl,
    knob_ctl: KnobCtl,
    output_ctl: OutputCtl,
    mixer_ctl: MixerCtl,
    input_ctl: InputCtl,
    display_ctl: DisplayCtl,
    hwstate: HwState,
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
        self.mixer_ctl.load(&self.avc, card_cntr)?;
        self.input_ctl.load(&self.avc, card_cntr)?;
        self.display_ctl.load(&self.avc, card_cntr)?;
        self.hwstate.load(&self.avc, card_cntr)?;

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
        } else if self.mixer_ctl.read(&self.avc, &self.company_id, elem_id, elem_value)? {
            Ok(true)
        } else if self.input_ctl.read(&self.avc, &self.company_id, elem_id, elem_value)? {
            Ok(true)
        } else if self.display_ctl.read(&self.avc, &self.company_id, elem_id, elem_value)? {
            Ok(true)
        } else if self.hwstate.read(&self.avc, &self.company_id, elem_id, elem_value)? {
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
        } else if self.mixer_ctl.write(&self.avc, &self.company_id, elem_id, old, new)? {
            Ok(true)
        } else if self.input_ctl.write(&self.avc, &self.company_id, elem_id, old, new)? {
            Ok(true)
        } else if self.display_ctl.write(&self.avc, &self.company_id, elem_id, old, new)? {
            Ok(true)
        } else if self.hwstate.write(&self.avc, &self.company_id, elem_id, old, new)? {
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
        elem_id_list.extend_from_slice(&self.hwstate.measure_elems);
    }

    fn measure_states(&mut self, unit: &mut hinawa::SndUnit) -> Result<(), Error> {
        self.meter_ctl.measure_state(unit, &mut self.req, TIMEOUT_MS)?;
        self.knob_ctl.measure_state(&mut self.avc, TIMEOUT_MS)?;

        self.output_ctl.0 = self.knob_ctl.0.output_mute;
        self.output_ctl.1 = self.knob_ctl.0.output_volume;
        self.hwstate.states[4] = self.knob_ctl.0.input_gains[0];
        self.hwstate.states[5] = self.knob_ctl.0.input_gains[1];

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
        } else if self.hwstate.measure_elems(elem_id, elem_value)? {
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
