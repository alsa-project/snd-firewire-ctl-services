// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use {super::*, protocols::apogee::*};

#[derive(Default, Debug)]
pub struct ApogeeModel {
    req: FwReq,
    avc: OxfwAvc,
    common_ctl: CommonCtl<OxfwAvc, DuetFwProtocol>,
    meter_ctl: MeterCtl,
    knob_ctl: KnobCtl,
    output_ctl: OutputCtl,
    input_ctl: InputCtl,
    mixer_ctl: MixerCtl,
    display_ctl: DisplayCtl,
}

const TIMEOUT_MS: u32 = 50;

const FCP_TIMEOUT_MS: u32 = 100;

impl CtlModel<(SndUnit, FwNode)> for ApogeeModel {
    fn cache(&mut self, unit: &mut (SndUnit, FwNode)) -> Result<(), Error> {
        self.avc.bind(&unit.1)?;

        self.common_ctl.detect(&mut self.avc, FCP_TIMEOUT_MS)?;

        self.common_ctl.cache(&mut self.avc, FCP_TIMEOUT_MS)?;
        self.meter_ctl.cache(&self.req, &unit.1, TIMEOUT_MS)?;
        self.knob_ctl.cache(&mut self.avc, FCP_TIMEOUT_MS)?;
        self.output_ctl.cache(&mut self.avc, FCP_TIMEOUT_MS)?;
        self.input_ctl.cache(&mut self.avc, FCP_TIMEOUT_MS)?;
        self.mixer_ctl.cache(&mut self.avc, FCP_TIMEOUT_MS)?;
        self.display_ctl.cache(&mut self.avc, FCP_TIMEOUT_MS)?;

        Ok(())
    }
    fn load(&mut self, _: &mut (SndUnit, FwNode), card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.common_ctl.load(card_cntr)?;
        self.meter_ctl.load(card_cntr)?;
        self.knob_ctl.load(card_cntr)?;
        self.output_ctl.load(card_cntr)?;
        self.input_ctl.load(card_cntr)?;
        self.mixer_ctl.load(card_cntr)?;
        self.display_ctl.load(card_cntr)?;

        Ok(())
    }

    fn read(
        &mut self,
        _: &mut (SndUnit, FwNode),
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.common_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.meter_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.knob_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.output_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.input_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.display_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(
        &mut self,
        unit: &mut (SndUnit, FwNode),
        elem_id: &ElemId,
        _: &ElemValue,
        new: &ElemValue,
    ) -> Result<bool, Error> {
        if self
            .common_ctl
            .write(&unit.0, &mut self.avc, elem_id, new, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .output_ctl
            .write(&mut self.avc, elem_id, new, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .input_ctl
            .write(&mut self.avc, elem_id, new, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .mixer_ctl
            .write_params(&mut self.avc, elem_id, new, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .display_ctl
            .write_params(&mut self.avc, elem_id, new, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl MeasureModel<(SndUnit, FwNode)> for ApogeeModel {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.meter_ctl.2);
        elem_id_list.extend_from_slice(&self.knob_ctl.1);
        elem_id_list.extend_from_slice(&self.output_ctl.1);
        elem_id_list.extend_from_slice(&self.input_ctl.1);
    }

    fn measure_states(&mut self, unit: &mut (SndUnit, FwNode)) -> Result<(), Error> {
        self.meter_ctl.cache(&self.req, &unit.1, TIMEOUT_MS)?;
        self.knob_ctl.cache(&mut self.avc, FCP_TIMEOUT_MS)?;

        self.output_ctl.0.mute = self.knob_ctl.0.output_mute;
        self.output_ctl.0.volume = self.knob_ctl.0.output_volume;

        self.input_ctl.0.gains[0] = self.knob_ctl.0.input_gains[0];
        self.input_ctl.0.gains[1] = self.knob_ctl.0.input_gains[1];

        Ok(())
    }

    fn measure_elem(
        &mut self,
        _: &(SndUnit, FwNode),
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.meter_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.knob_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.output_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.input_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl NotifyModel<(SndUnit, FwNode), bool> for ApogeeModel {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.common_ctl.notified_elem_id_list);
    }

    fn parse_notification(
        &mut self,
        _: &mut (SndUnit, FwNode),
        &locked: &bool,
    ) -> Result<(), Error> {
        if locked {
            self.common_ctl.cache(&mut self.avc, FCP_TIMEOUT_MS)?;
        }
        Ok(())
    }

    fn read_notified_elem(
        &mut self,
        _: &(SndUnit, FwNode),
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        self.common_ctl.read(elem_id, elem_value)
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

    fn cache(&mut self, req: &FwReq, node: &FwNode, timeout_ms: u32) -> Result<(), Error> {
        let res = DuetFwProtocol::cache_meter(req, node, &mut self.0, timeout_ms);
        debug!(params = ?self.0, ?res);
        res?;
        let res = DuetFwProtocol::cache_meter(req, node, &mut self.1, timeout_ms);
        debug!(params = ?self.1, ?res);
        res?;
        Ok(())
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, ANALOG_INPUT_METER_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                DuetFwProtocol::METER_LEVEL_MIN,
                DuetFwProtocol::METER_LEVEL_MAX,
                DuetFwProtocol::METER_LEVEL_STEP,
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
                DuetFwProtocol::METER_LEVEL_MIN,
                DuetFwProtocol::METER_LEVEL_MAX,
                DuetFwProtocol::METER_LEVEL_STEP,
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
                DuetFwProtocol::METER_LEVEL_MIN,
                DuetFwProtocol::METER_LEVEL_MAX,
                DuetFwProtocol::METER_LEVEL_STEP,
                Self::MIXER_OUTPUT_LABELS.len(),
                None,
                false,
            )
            .map(|mut elem_id_list| self.2.append(&mut elem_id_list))?;

        Ok(())
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            ANALOG_INPUT_METER_NAME => {
                let params = &self.0;
                elem_value.set_int(&params.0);
                Ok(true)
            }
            STREAM_INPUT_METER_NAME => {
                let params = &self.1;
                elem_value.set_int(&params.stream_inputs);
                Ok(true)
            }
            MIXER_OUTPUT_METER_NAME => {
                let params = &self.1;
                elem_value.set_int(&params.mixer_outputs);
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

    fn cache(&mut self, avc: &mut OxfwAvc, timeout_ms: u32) -> Result<(), Error> {
        let res = DuetFwProtocol::cache(avc, &mut self.0, timeout_ms);
        debug!(params = ?self.0, ?res);
        res
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let labels: Vec<&str> = Self::KNOB_TARGETS
            .iter()
            .map(|t| knob_target_to_str(t))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, KNOB_TARGET_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, false)
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))
    }

    fn read(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            KNOB_TARGET_NAME => {
                let pos = Self::KNOB_TARGETS
                    .iter()
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
struct OutputCtl(DuetFwOutputParams, Vec<ElemId>);

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

    fn cache(&mut self, avc: &mut OxfwAvc, timeout_ms: u32) -> Result<(), Error> {
        let res = DuetFwProtocol::cache(avc, &mut self.0, timeout_ms);
        debug!(params = ?self.0, ?res);
        res
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, OUTPUT_MUTE_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, 1, 1, true)
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, OUTPUT_VOLUME_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                DuetFwProtocol::OUTPUT_VOLUME_MIN as i32,
                DuetFwProtocol::OUTPUT_VOLUME_MAX as i32,
                1,
                1,
                None,
                true,
            )
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        let labels: Vec<&str> = Self::SOURCES
            .iter()
            .map(|s| output_source_to_str(s))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, OUTPUT_SRC_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let labels: Vec<&str> = Self::NOMINAL_LEVELS
            .iter()
            .map(|l| output_nominal_level_to_str(l))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, OUTPUT_NOMINAL_LEVEL_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let labels: Vec<&str> = Self::MUTE_MODES
            .iter()
            .map(|m| output_mute_mode_to_str(m))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, OUTPUT_MUTE_FOR_LINE_OUT, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, OUTPUT_MUTE_FOR_HP_OUT, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        Ok(())
    }

    fn read(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            OUTPUT_MUTE_NAME => {
                let params = &self.0;
                elem_value.set_bool(&[params.mute]);
                Ok(true)
            }
            OUTPUT_VOLUME_NAME => {
                let params = &self.0;
                elem_value.set_int(&[params.volume as i32]);
                Ok(true)
            }
            OUTPUT_SRC_NAME => {
                let params = &self.0;
                let pos = Self::SOURCES
                    .iter()
                    .position(|s| params.source.eq(s))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            OUTPUT_NOMINAL_LEVEL_NAME => {
                let params = &self.0;
                let pos = Self::NOMINAL_LEVELS
                    .iter()
                    .position(|l| params.nominal_level.eq(l))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            OUTPUT_MUTE_FOR_LINE_OUT => {
                let params = &self.0;
                let pos = Self::MUTE_MODES
                    .iter()
                    .position(|m| params.line_mute_mode.eq(m))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            OUTPUT_MUTE_FOR_HP_OUT => {
                let params = &self.0;
                let pos = Self::MUTE_MODES
                    .iter()
                    .position(|m| params.hp_mute_mode.eq(m))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write(
        &mut self,
        avc: &mut OxfwAvc,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            OUTPUT_MUTE_NAME => {
                let mut params = self.0.clone();
                params.mute = elem_value.boolean()[0];
                let res = DuetFwProtocol::update(avc, &params, &mut self.0, timeout_ms);
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            OUTPUT_VOLUME_NAME => {
                let mut params = self.0.clone();
                params.volume = elem_value.int()[0] as u8;
                let res = DuetFwProtocol::update(avc, &params, &mut self.0, timeout_ms);
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            OUTPUT_SRC_NAME => {
                let mut params = self.0.clone();
                let pos = elem_value.enumerated()[0] as usize;
                params.source = Self::SOURCES
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| {
                        let msg = format!("Output source not found for position {}", pos);
                        Error::new(FileError::Inval, &msg)
                    })
                    .copied()?;
                let res = DuetFwProtocol::update(avc, &params, &mut self.0, timeout_ms);
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            OUTPUT_NOMINAL_LEVEL_NAME => {
                let mut params = self.0.clone();
                let pos = elem_value.enumerated()[0] as usize;
                params.source = Self::SOURCES
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| {
                        let msg = format!("Output nominal level not found for position {}", pos);
                        Error::new(FileError::Inval, &msg)
                    })
                    .copied()?;
                let res = DuetFwProtocol::update(avc, &params, &mut self.0, timeout_ms);
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            OUTPUT_MUTE_FOR_LINE_OUT => {
                let mut params = self.0.clone();
                let pos = elem_value.enumerated()[0] as usize;
                params.line_mute_mode = Self::MUTE_MODES
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| {
                        let msg = format!("Output mute mode not found for position {}", pos);
                        Error::new(FileError::Inval, &msg)
                    })
                    .copied()?;
                let res = DuetFwProtocol::update(avc, &params, &mut self.0, timeout_ms);
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            OUTPUT_MUTE_FOR_HP_OUT => {
                let mut params = self.0.clone();
                let pos = elem_value.enumerated()[0] as usize;
                params.hp_mute_mode = Self::MUTE_MODES
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| {
                        let msg = format!("Output mute mode not found for position {}", pos);
                        Error::new(FileError::Inval, &msg)
                    })
                    .copied()?;
                let res = DuetFwProtocol::update(avc, &params, &mut self.0, timeout_ms);
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

#[derive(Default, Debug)]
struct InputCtl(DuetFwInputParameters, Vec<ElemId>);

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

    const SOURCES: [DuetFwInputSource; 2] = [DuetFwInputSource::Xlr, DuetFwInputSource::Phone];

    const XLR_NOMINAL_LEVELS: [DuetFwInputXlrNominalLevel; 3] = [
        DuetFwInputXlrNominalLevel::Microphone,
        DuetFwInputXlrNominalLevel::Professional,
        DuetFwInputXlrNominalLevel::Consumer,
    ];

    fn cache(&mut self, avc: &mut OxfwAvc, timeout_ms: u32) -> Result<(), Error> {
        let res = DuetFwProtocol::cache(avc, &mut self.0, timeout_ms);
        debug!(params = ?self.0, ?res);
        res
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, INPUT_GAIN_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                DuetFwProtocol::INPUT_GAIN_MIN as i32,
                DuetFwProtocol::INPUT_GAIN_MAX as i32,
                1,
                Self::LABELS.len(),
                None,
                true,
            )
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, INPUT_POLARITY_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, Self::MIC_LABELS.len(), true)?;

        let labels: Vec<&str> = Self::XLR_NOMINAL_LEVELS
            .iter()
            .map(|l| input_xlr_nominal_level_to_str(l))
            .collect();
        let elem_id =
            ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, INPUT_XLR_NOMINAL_LEVEL_NAME, 0);
        let _ =
            card_cntr.add_enum_elems(&elem_id, 1, Self::MIC_LABELS.len(), &labels, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, INPUT_PHANTOM_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, Self::MIC_LABELS.len(), true)?;

        let labels: Vec<&str> = Self::SOURCES
            .iter()
            .map(|s| input_source_to_str(s))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, INPUT_SOURCE_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, Self::LABELS.len(), &labels, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, INPUT_CLICKLESS_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        Ok(())
    }

    fn read(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            INPUT_POLARITY_NAME => {
                let params = &self.0;
                elem_value.set_bool(&params.polarities);
                Ok(true)
            }
            INPUT_XLR_NOMINAL_LEVEL_NAME => {
                let params = &self.0;
                let vals: Vec<u32> = params
                    .xlr_nominal_levels
                    .iter()
                    .map(|xlr_nominal_level| {
                        let pos = Self::XLR_NOMINAL_LEVELS
                            .iter()
                            .position(|l| xlr_nominal_level.eq(l))
                            .unwrap();
                        pos as u32
                    })
                    .collect();
                elem_value.set_enum(&vals);
                Ok(true)
            }
            INPUT_PHANTOM_NAME => {
                let params = &self.0;
                elem_value.set_bool(&params.phantom_powerings);
                Ok(true)
            }
            INPUT_SOURCE_NAME => {
                let params = &self.0;
                let vals: Vec<u32> = params
                    .srcs
                    .iter()
                    .map(|src| {
                        let pos = Self::SOURCES.iter().position(|s| src.eq(s)).unwrap();
                        pos as u32
                    })
                    .collect();
                elem_value.set_enum(&vals);
                Ok(true)
            }
            INPUT_CLICKLESS_NAME => {
                let params = &self.0;
                elem_value.set_bool(&[params.clickless]);
                Ok(true)
            }
            INPUT_GAIN_NAME => {
                let params = &self.0;
                let vals: Vec<i32> = params.gains.iter().map(|&val| val as i32).collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write(
        &mut self,
        avc: &mut OxfwAvc,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            INPUT_GAIN_NAME => {
                let mut params = self.0.clone();
                params
                    .gains
                    .iter_mut()
                    .zip(elem_value.int())
                    .for_each(|(gain, &val)| *gain = val as u8);
                let res = DuetFwProtocol::update(avc, &params, &mut self.0, timeout_ms);
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            INPUT_POLARITY_NAME => {
                let mut params = self.0.clone();
                params
                    .polarities
                    .iter_mut()
                    .zip(elem_value.boolean())
                    .for_each(|(polarity, val)| *polarity = val);
                let res = DuetFwProtocol::update(avc, &params, &mut self.0, timeout_ms);
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            INPUT_XLR_NOMINAL_LEVEL_NAME => {
                let mut params = self.0.clone();
                params
                    .xlr_nominal_levels
                    .iter_mut()
                    .zip(elem_value.enumerated())
                    .try_for_each(|(xlr_nominal_level, &val)| {
                        let pos = val as usize;
                        Self::XLR_NOMINAL_LEVELS
                            .iter()
                            .nth(pos)
                            .ok_or_else(|| {
                                let msg =
                                    format!("XLR nominal level not found for position {}", pos);
                                Error::new(FileError::Inval, &msg)
                            })
                            .map(|&level| *xlr_nominal_level = level)
                    })?;
                let res = DuetFwProtocol::update(avc, &params, &mut self.0, timeout_ms);
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            INPUT_PHANTOM_NAME => {
                let mut params = self.0.clone();
                params
                    .phantom_powerings
                    .iter_mut()
                    .zip(elem_value.boolean())
                    .for_each(|(enabled, val)| *enabled = val);
                let res = DuetFwProtocol::update(avc, &params, &mut self.0, timeout_ms);
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            INPUT_SOURCE_NAME => {
                let mut params = self.0.clone();
                params
                    .srcs
                    .iter_mut()
                    .zip(elem_value.enumerated())
                    .try_for_each(|(src, &val)| {
                        let pos = val as usize;
                        Self::SOURCES
                            .iter()
                            .nth(pos)
                            .ok_or_else(|| {
                                let msg = format!("Input source not found for position {}", val);
                                Error::new(FileError::Inval, &msg)
                            })
                            .map(|&s| *src = s)
                    })?;
                let res = DuetFwProtocol::update(avc, &params, &mut self.0, timeout_ms);
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            INPUT_CLICKLESS_NAME => {
                let mut params = self.0.clone();
                params.clickless = elem_value.boolean()[0];
                let res = DuetFwProtocol::update(avc, &params, &mut self.0, timeout_ms);
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

#[derive(Default, Debug)]
struct MixerCtl(DuetFwMixerParams);

const MIXER_SOURCE_GAIN_NAME: &str = "mixer-source-gain";

impl MixerCtl {
    const DST_LABELS: [&'static str; 2] = ["mixer-output-1", "mixer-output-2"];
    const SRC_LABELS: [&'static str; 4] = [
        "stream-input-1",
        "stream-input-2",
        "analog-input-1",
        "analog-input-2",
    ];

    fn cache(&mut self, avc: &mut OxfwAvc, timeout_ms: u32) -> Result<(), Error> {
        let res = DuetFwProtocol::cache(avc, &mut self.0, timeout_ms);
        debug!(params = ?self.0, ?res);
        res
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIXER_SOURCE_GAIN_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            Self::DST_LABELS.len(),
            DuetFwProtocol::MIXER_SOURCE_GAIN_MIN as i32,
            DuetFwProtocol::MIXER_SOURCE_GAIN_MAX as i32,
            1,
            Self::SRC_LABELS.len(),
            None,
            true,
        )?;

        Ok(())
    }

    fn read(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            MIXER_SOURCE_GAIN_NAME => {
                let dst = elem_id.index() as usize;
                let mixer = self.0 .0.iter().nth(dst).ok_or_else(|| {
                    let msg = format!("Mixer not found for position {}", dst);
                    Error::new(FileError::Inval, &msg)
                })?;
                let vals: Vec<i32> = mixer
                    .stream_inputs
                    .iter()
                    .chain(&mixer.analog_inputs)
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
        avc: &mut OxfwAvc,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            MIXER_SOURCE_GAIN_NAME => {
                let dst = elem_id.index() as usize;
                let mut params = self.0.clone();
                let mixer = params.0.iter_mut().nth(dst).ok_or_else(|| {
                    let msg = format!("Mixer not found for position {}", dst);
                    Error::new(FileError::Inval, &msg)
                })?;
                mixer
                    .stream_inputs
                    .iter_mut()
                    .chain(&mut mixer.analog_inputs)
                    .zip(elem_value.int())
                    .for_each(|(coef, &val)| *coef = val as u16);
                let res = DuetFwProtocol::update(avc, &params, &mut self.0, timeout_ms);
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

#[derive(Default, Debug)]
struct DisplayCtl(DuetFwDisplayParams);

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
    const TARGETS: [DuetFwDisplayTarget; 2] =
        [DuetFwDisplayTarget::Output, DuetFwDisplayTarget::Input];

    const MODES: [DuetFwDisplayMode; 2] = [
        DuetFwDisplayMode::Independent,
        DuetFwDisplayMode::FollowingToKnobTarget,
    ];

    const OVERHOLDS: [DuetFwDisplayOverhold; 2] = [
        DuetFwDisplayOverhold::Infinite,
        DuetFwDisplayOverhold::TwoSeconds,
    ];

    fn cache(&mut self, avc: &mut OxfwAvc, timeout_ms: u32) -> Result<(), Error> {
        let res = DuetFwProtocol::cache(avc, &mut self.0, timeout_ms);
        debug!(params = ?self.0, ?res);
        res
    }

    fn load(&self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let labels: Vec<&str> = Self::TARGETS
            .iter()
            .map(|t| display_target_to_str(t))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, DISPLAY_TARGET_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let labels: Vec<&str> = Self::MODES.iter().map(|t| display_mode_to_str(t)).collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, DISPLAY_MODE_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let labels: Vec<&str> = Self::OVERHOLDS
            .iter()
            .map(|t| display_overhold_to_str(t))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, DISPLAY_OVERHOLDS_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        Ok(())
    }

    fn read(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            DISPLAY_TARGET_NAME => {
                let params = &self.0;
                let pos = Self::TARGETS
                    .iter()
                    .position(|t| params.target.eq(t))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            DISPLAY_MODE_NAME => {
                let params = &self.0;
                let pos = Self::MODES.iter().position(|t| params.mode.eq(t)).unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            DISPLAY_OVERHOLDS_NAME => {
                let params = &self.0;
                let pos = Self::OVERHOLDS
                    .iter()
                    .position(|t| params.overhold.eq(t))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write_params(
        &mut self,
        avc: &mut OxfwAvc,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            DISPLAY_TARGET_NAME => {
                let mut params = self.0.clone();
                let pos = elem_value.enumerated()[0] as usize;
                Self::TARGETS
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| {
                        let msg = format!("Display target not found for position {}", pos);
                        Error::new(FileError::Inval, &msg)
                    })
                    .map(|&t| params.target = t)?;
                let res = DuetFwProtocol::update(avc, &params, &mut self.0, timeout_ms);
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            DISPLAY_MODE_NAME => {
                let mut params = self.0.clone();
                let pos = elem_value.enumerated()[0] as usize;
                Self::MODES
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| {
                        let msg = format!("Display mode not found for position {}", pos);
                        Error::new(FileError::Inval, &msg)
                    })
                    .map(|&m| params.mode = m)?;
                let res = DuetFwProtocol::update(avc, &params, &mut self.0, timeout_ms);
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            DISPLAY_OVERHOLDS_NAME => {
                let mut params = self.0.clone();
                let pos = elem_value.enumerated()[0] as usize;
                Self::OVERHOLDS
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| {
                        let msg = format!("Display overhold not found for position {}", pos);
                        Error::new(FileError::Inval, &msg)
                    })
                    .map(|&o| params.overhold = o)?;
                let res = DuetFwProtocol::update(avc, &params, &mut self.0, timeout_ms);
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            _ => Ok(false),
        }
    }
}
