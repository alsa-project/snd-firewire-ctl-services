// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2024 Takashi Sakamoto

use {super::*, protocols::weiss::avc::*};

#[derive(Default)]
pub struct WeissMan301Model {
    req: FwReq,
    avc: WeissAvc,
    sections: GeneralSections,
    common_ctl: CommonCtl<WeissMan301Protocol>,
    analog_output_ctls: AnalogOutputCtls,
    digital_output_ctls: DigitalOutputCtls,
}

const TIMEOUT_MS: u32 = 20;
const FCP_TIMEOUT_MS: u32 = 40;

impl CtlModel<(SndDice, FwNode)> for WeissMan301Model {
    fn cache(&mut self, (_, node): &mut (SndDice, FwNode)) -> Result<(), Error> {
        WeissMan301Protocol::read_general_sections(
            &self.req,
            &node,
            &mut self.sections,
            TIMEOUT_MS,
        )?;

        self.common_ctl
            .cache_whole_params(&self.req, &node, &mut self.sections, TIMEOUT_MS)?;

        self.avc.bind(node)?;

        self.analog_output_ctls.cache(&self.avc, FCP_TIMEOUT_MS)?;
        self.digital_output_ctls.cache(&self.avc, FCP_TIMEOUT_MS)?;

        Ok(())
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.common_ctl.load(card_cntr)?;

        self.analog_output_ctls.load(card_cntr)?;
        self.digital_output_ctls.load(card_cntr)?;

        Ok(())
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        if self.common_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.analog_output_ctls.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.digital_output_ctls.read(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(
        &mut self,
        (unit, node): &mut (SndDice, FwNode),
        elem_id: &ElemId,
        elem_value: &ElemValue,
    ) -> Result<bool, Error> {
        if self.common_ctl.write(
            &unit,
            &self.req,
            &node,
            &mut self.sections,
            elem_id,
            elem_value,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self
            .analog_output_ctls
            .write(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .digital_output_ctls
            .write(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl NotifyModel<(SndDice, FwNode), u32> for WeissMan301Model {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.common_ctl.notified_elem_id_list);
    }

    fn parse_notification(
        &mut self,
        (_, node): &mut (SndDice, FwNode),
        msg: &u32,
    ) -> Result<(), Error> {
        self.common_ctl
            .parse_notification(&self.req, &node, &mut self.sections, *msg, TIMEOUT_MS)
    }
}

impl MeasureModel<(SndDice, FwNode)> for WeissMan301Model {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.common_ctl.measured_elem_id_list);
    }

    fn measure_states(&mut self, (_, node): &mut (SndDice, FwNode)) -> Result<(), Error> {
        self.common_ctl
            .cache_partial_params(&self.req, &node, &mut self.sections, TIMEOUT_MS)
    }
}

#[derive(Default, Debug)]
struct AnalogOutputCtls {
    polarity_inversion: WeissAvcAnalogOutputPolarityInversion,
    filter_type: WeissAvcAnalogOutputFilterType,
    mute: WeissAvcAnalogOutputMute,
    level: WeissAvcAnalogOutputLevel,
}

const ANALOG_OUTPUT_POLARITY_INVERSION_NAME: &str = "DAC::DAC Polarity Inversion Playback Switch";
const ANALOG_OUTPUT_FILTER_TYPE_NAME: &str = "DAC::DAC Filter Type";
const ANALOG_OUTPUT_UNMUTE_NAME: &str = "DAC::DAC Output Playback Switch";
const ANALOG_OUTPUT_LEVEL_NAME: &str = "DAC::Analog Output Level";

fn analog_output_filter_type_to_str(filter_type: &WeissAvcAnalogOutputFilterType) -> &str {
    match filter_type {
        WeissAvcAnalogOutputFilterType::A => "A",
        WeissAvcAnalogOutputFilterType::B => "B",
    }
}

const ANALOG_OUTPUT_FILTER_TYPES: &[WeissAvcAnalogOutputFilterType] = &[
    WeissAvcAnalogOutputFilterType::A,
    WeissAvcAnalogOutputFilterType::B,
];

fn analog_output_level_to_str(level: &WeissAvcAnalogOutputLevel) -> &str {
    match level {
        WeissAvcAnalogOutputLevel::Zero => "0 dB",
        WeissAvcAnalogOutputLevel::NegativeTen => "-10 dB",
        WeissAvcAnalogOutputLevel::NegativeTwenty => "-20 dB",
        WeissAvcAnalogOutputLevel::NegativeThirty => "-30 dB",
    }
}

const ANALOG_OUTPUT_LEVELS: &[WeissAvcAnalogOutputLevel] = &[
    WeissAvcAnalogOutputLevel::Zero,
    WeissAvcAnalogOutputLevel::NegativeTen,
    WeissAvcAnalogOutputLevel::NegativeTwenty,
    WeissAvcAnalogOutputLevel::NegativeThirty,
];

impl AnalogOutputCtls {
    fn cache(&mut self, avc: &WeissAvc, timeout_ms: u32) -> Result<(), Error> {
        let res = WeissMan301Protocol::cache_param(avc, &mut self.polarity_inversion, timeout_ms);
        debug!(param = ?self.polarity_inversion, ?res);
        res?;
        let res = WeissMan301Protocol::cache_param(avc, &mut self.filter_type, timeout_ms);
        debug!(param = ?self.filter_type, ?res);
        res?;
        let res = WeissMan301Protocol::cache_param(avc, &mut self.mute, timeout_ms);
        debug!(param = ?self.mute, ?res);
        res?;
        let res = WeissMan301Protocol::cache_param(avc, &mut self.level, timeout_ms);
        debug!(param = ?self.level, ?res);
        res?;

        Ok(())
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(
            ElemIfaceType::Mixer,
            0,
            0,
            ANALOG_OUTPUT_POLARITY_INVERSION_NAME,
            0,
        );
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        let labels: Vec<&str> = ANALOG_OUTPUT_FILTER_TYPES
            .iter()
            .map(|t| analog_output_filter_type_to_str(t))
            .collect();
        let elem_id = ElemId::new_by_name(
            ElemIfaceType::Mixer,
            0,
            0,
            ANALOG_OUTPUT_FILTER_TYPE_NAME,
            0,
        );
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, ANALOG_OUTPUT_UNMUTE_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        let labels: Vec<&str> = ANALOG_OUTPUT_LEVELS
            .iter()
            .map(|t| analog_output_level_to_str(t))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, ANALOG_OUTPUT_LEVEL_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        Ok(())
    }

    fn read(&self, elem_id: &ElemId, elem_value: &ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            ANALOG_OUTPUT_POLARITY_INVERSION_NAME => {
                elem_value.set_bool(&[self.polarity_inversion.0]);
                Ok(true)
            }
            ANALOG_OUTPUT_FILTER_TYPE_NAME => {
                let pos = ANALOG_OUTPUT_FILTER_TYPES
                    .iter()
                    .position(|filter_type| filter_type.eq(&self.filter_type))
                    .unwrap() as u32;
                elem_value.set_enum(&[pos]);
                Ok(true)
            }
            ANALOG_OUTPUT_UNMUTE_NAME => {
                elem_value.set_bool(&[!self.mute.0]);
                Ok(true)
            }
            ANALOG_OUTPUT_LEVEL_NAME => {
                let pos = ANALOG_OUTPUT_LEVELS
                    .iter()
                    .position(|level| level.eq(&self.level))
                    .unwrap() as u32;
                elem_value.set_enum(&[pos]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write(
        &mut self,
        avc: &WeissAvc,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            ANALOG_OUTPUT_POLARITY_INVERSION_NAME => {
                let mut param = self.polarity_inversion.clone();
                param.0 = elem_value.boolean()[0];
                let res = WeissMan301Protocol::update_param(avc, &mut param, timeout_ms)
                    .map(|_| self.polarity_inversion = param);
                debug!(param = ?self.polarity_inversion, ?res);
                res.map(|_| true)
            }
            ANALOG_OUTPUT_FILTER_TYPE_NAME => {
                let pos = elem_value.enumerated()[0] as usize;
                let mut param = ANALOG_OUTPUT_FILTER_TYPES
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| {
                        let msg = format!("Invalid index of analog output filter type: {}", pos);
                        Error::new(FileError::Inval, &msg)
                    })
                    .map(|filter_type| *filter_type)?;
                let res = WeissMan301Protocol::update_param(avc, &mut param, timeout_ms)
                    .map(|_| self.filter_type = param);
                debug!(param = ?self.filter_type, ?res);
                res.map(|_| true)
            }
            ANALOG_OUTPUT_UNMUTE_NAME => {
                let mut param = self.mute.clone();
                param.0 = !elem_value.boolean()[0];
                let res = WeissMan301Protocol::update_param(avc, &mut param, timeout_ms)
                    .map(|_| self.mute = param);
                debug!(param = ?self.mute, ?res);
                res.map(|_| true)
            }
            ANALOG_OUTPUT_LEVEL_NAME => {
                let pos = elem_value.enumerated()[0] as usize;
                let mut param = ANALOG_OUTPUT_LEVELS
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| {
                        let msg = format!("Invalid index of analog output level: {}", pos);
                        Error::new(FileError::Inval, &msg)
                    })
                    .map(|level| *level)?;
                let res = WeissMan301Protocol::update_param(avc, &mut param, timeout_ms)
                    .map(|_| self.level = param);
                debug!(param = ?self.level, ?res);
                res.map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

#[derive(Default, Debug)]
struct DigitalOutputCtls {
    mode: WeissAvcDigitalOutputMode,
    word_clock_half_rate: WeissAvcWordClockOutputHalfRate,
    aesebu_xlr_mute: WeissAvcAesebuXlrOutputMute,
    spdif_coaxial_mute: WeissAvcSpdifCoaxialOutputMute,
}

const DIGITAL_OUTPUT_MODE_NAME: &str = "Dual Wire Mode Switch";
const WORD_CLOCK_OUTPUT_HALF_RATE_NAME: &str = "Dual Wire Word Clock Half Rate Switch";
const AESEBU_XLR_OUTPUT_UNMUTE_NAME: &str = "XLR::XLR Output Playback Switch";
const SPDIF_COAXIAL_OUTPUT_UNMUTE_NAME: &str = "RCA::DCA Output Playback Switch";

impl DigitalOutputCtls {
    fn cache(&mut self, avc: &WeissAvc, timeout_ms: u32) -> Result<(), Error> {
        let res = WeissMan301Protocol::cache_param(avc, &mut self.mode, timeout_ms);
        debug!(param = ?self.mode, ?res);
        res?;
        let res = WeissMan301Protocol::cache_param(avc, &mut self.word_clock_half_rate, timeout_ms);
        debug!(param = ?self.word_clock_half_rate, ?res);
        res?;
        let res = WeissMan301Protocol::cache_param(avc, &mut self.aesebu_xlr_mute, timeout_ms);
        debug!(param = ?self.aesebu_xlr_mute, ?res);
        res?;
        let res = WeissMan301Protocol::cache_param(avc, &mut self.spdif_coaxial_mute, timeout_ms);
        debug!(param = ?self.spdif_coaxial_mute, ?res);
        res?;

        Ok(())
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        [
            (ElemIfaceType::Card, DIGITAL_OUTPUT_MODE_NAME),
            (ElemIfaceType::Card, WORD_CLOCK_OUTPUT_HALF_RATE_NAME),
            (ElemIfaceType::Mixer, AESEBU_XLR_OUTPUT_UNMUTE_NAME),
            (ElemIfaceType::Mixer, SPDIF_COAXIAL_OUTPUT_UNMUTE_NAME),
        ]
        .iter()
        .try_for_each(|&(iface, name)| {
            let elem_id = ElemId::new_by_name(iface, 0, 0, name, 0);
            card_cntr.add_bool_elems(&elem_id, 1, 1, true).map(|_| ())
        })
    }

    fn read(&self, elem_id: &ElemId, elem_value: &ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            DIGITAL_OUTPUT_MODE_NAME => {
                elem_value.set_bool(&[self.mode.0]);
                Ok(true)
            }
            WORD_CLOCK_OUTPUT_HALF_RATE_NAME => {
                elem_value.set_bool(&[self.word_clock_half_rate.0]);
                Ok(true)
            }
            AESEBU_XLR_OUTPUT_UNMUTE_NAME => {
                elem_value.set_bool(&[!self.aesebu_xlr_mute.0]);
                Ok(true)
            }
            SPDIF_COAXIAL_OUTPUT_UNMUTE_NAME => {
                elem_value.set_bool(&[!self.spdif_coaxial_mute.0]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write(
        &mut self,
        avc: &WeissAvc,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            DIGITAL_OUTPUT_MODE_NAME => {
                let mut param = self.mode.clone();
                param.0 = elem_value.boolean()[0];
                let res = WeissMan301Protocol::update_param(avc, &mut param, timeout_ms)
                    .map(|_| self.mode = param);
                debug!(param = ?self.mode, ?res);
                res.map(|_| true)
            }
            WORD_CLOCK_OUTPUT_HALF_RATE_NAME => {
                let mut param = self.word_clock_half_rate.clone();
                param.0 = elem_value.boolean()[0];
                let res = WeissMan301Protocol::update_param(avc, &mut param, timeout_ms)
                    .map(|_| self.word_clock_half_rate = param);
                debug!(param = ?self.word_clock_half_rate, ?res);
                res.map(|_| true)
            }
            AESEBU_XLR_OUTPUT_UNMUTE_NAME => {
                let mut param = self.aesebu_xlr_mute.clone();
                param.0 = !elem_value.boolean()[0];
                let res = WeissMan301Protocol::update_param(avc, &mut param, timeout_ms)
                    .map(|_| self.aesebu_xlr_mute = param);
                debug!(param = ?self.aesebu_xlr_mute, ?res);
                res.map(|_| true)
            }
            SPDIF_COAXIAL_OUTPUT_UNMUTE_NAME => {
                let mut param = self.spdif_coaxial_mute.clone();
                param.0 = !elem_value.boolean()[0];
                let res = WeissMan301Protocol::update_param(avc, &mut param, timeout_ms)
                    .map(|_| self.spdif_coaxial_mute = param);
                debug!(param = ?self.spdif_coaxial_mute, ?res);
                res.map(|_| true)
            }
            _ => Ok(false),
        }
    }
}
