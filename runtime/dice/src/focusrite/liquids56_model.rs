// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

use {super::*, protocols::focusrite::liquids56::*};

#[derive(Default)]
pub struct LiquidS56Model {
    req: FwReq,
    sections: GeneralSections,
    extension_sections: ExtensionSections,
    common_ctl: CommonCtl<LiquidS56Protocol>,
    tcd22xx_ctls: Tcd22xxCtls<LiquidS56Protocol>,
    out_grp_ctl: OutGroupCtl<LiquidS56Protocol>,
    io_params_ctl: IoParamsCtl<LiquidS56Protocol>,
    specific_ctl: SpecificCtl,
}

const TIMEOUT_MS: u32 = 20;

impl LiquidS56Model {
    pub(crate) fn store_configuration(&mut self, node: &FwNode) -> Result<(), Error> {
        self.tcd22xx_ctls.store_configuration(
            &mut self.req,
            node,
            &self.extension_sections,
            TIMEOUT_MS,
        )
    }
}

impl CtlModel<(SndDice, FwNode)> for LiquidS56Model {
    fn cache(&mut self, unit: &mut (SndDice, FwNode)) -> Result<(), Error> {
        LiquidS56Protocol::read_general_sections(
            &self.req,
            &unit.1,
            &mut self.sections,
            TIMEOUT_MS,
        )?;

        self.common_ctl
            .cache_whole_params(&self.req, &unit.1, &mut self.sections, TIMEOUT_MS)?;

        LiquidS56Protocol::read_extension_sections(
            &self.req,
            &unit.1,
            &mut self.extension_sections,
            TIMEOUT_MS,
        )?;

        self.tcd22xx_ctls.cache_whole_params(
            &mut self.req,
            &mut unit.1,
            &self.extension_sections,
            &self.common_ctl.global_params,
            TIMEOUT_MS,
        )?;

        self.out_grp_ctl.cache(
            &mut self.req,
            &mut unit.1,
            &self.extension_sections,
            &self.tcd22xx_ctls.caps,
            TIMEOUT_MS,
        )?;

        self.io_params_ctl.cache(
            &mut self.req,
            &mut unit.1,
            &self.extension_sections,
            &self.tcd22xx_ctls.caps,
            TIMEOUT_MS,
        )?;

        self.specific_ctl.cache(
            &mut self.req,
            &mut unit.1,
            &self.extension_sections,
            &self.tcd22xx_ctls.caps,
            TIMEOUT_MS,
        )?;

        Ok(())
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.common_ctl.load(card_cntr)?;

        self.tcd22xx_ctls.load(card_cntr)?;

        self.out_grp_ctl.load(card_cntr)?;
        self.io_params_ctl.load(card_cntr)?;
        self.specific_ctl.load(card_cntr)?;

        Ok(())
    }

    fn read(
        &mut self,
        _: &mut (SndDice, FwNode),
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.common_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.tcd22xx_ctls.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.out_grp_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.io_params_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.specific_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(
        &mut self,
        unit: &mut (SndDice, FwNode),
        elem_id: &ElemId,
        _: &ElemValue,
        new: &ElemValue,
    ) -> Result<bool, Error> {
        if self.common_ctl.write(
            &unit.0,
            &self.req,
            &unit.1,
            &mut self.sections,
            elem_id,
            new,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self.tcd22xx_ctls.write(
            &mut self.req,
            &mut unit.1,
            &self.extension_sections,
            elem_id,
            new,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self.out_grp_ctl.write(
            &mut self.req,
            &mut unit.1,
            &self.extension_sections,
            &self.tcd22xx_ctls.caps,
            elem_id,
            new,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self.io_params_ctl.write(
            &mut self.req,
            &mut unit.1,
            &self.extension_sections,
            &self.tcd22xx_ctls.caps,
            elem_id,
            new,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self.specific_ctl.write(
            &mut self.req,
            &mut unit.1,
            &self.extension_sections,
            &self.tcd22xx_ctls.caps,
            elem_id,
            new,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl NotifyModel<(SndDice, FwNode), u32> for LiquidS56Model {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.common_ctl.notified_elem_id_list);
        elem_id_list.extend_from_slice(&self.tcd22xx_ctls.notified_elem_id_list);
        elem_id_list.extend_from_slice(&self.out_grp_ctl.1);
    }

    fn parse_notification(&mut self, unit: &mut (SndDice, FwNode), msg: &u32) -> Result<(), Error> {
        self.common_ctl.parse_notification(
            &self.req,
            &unit.1,
            &mut self.sections,
            *msg,
            TIMEOUT_MS,
        )?;
        self.tcd22xx_ctls.parse_notification(
            &mut self.req,
            &mut unit.1,
            &self.extension_sections,
            &self.common_ctl.global_params,
            TIMEOUT_MS,
            *msg,
        )?;
        self.out_grp_ctl.parse_notification(
            &mut self.req,
            &mut unit.1,
            &self.extension_sections,
            &self.tcd22xx_ctls.caps,
            *msg,
            TIMEOUT_MS,
        )?;
        Ok(())
    }
}

impl MeasureModel<(SndDice, FwNode)> for LiquidS56Model {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.common_ctl.measured_elem_id_list);
        elem_id_list.extend_from_slice(&self.tcd22xx_ctls.measured_elem_id_list);
    }

    fn measure_states(&mut self, unit: &mut (SndDice, FwNode)) -> Result<(), Error> {
        self.common_ctl
            .cache_partial_params(&self.req, &unit.1, &mut self.sections, TIMEOUT_MS)?;
        self.tcd22xx_ctls.cache_partial_params(
            &mut self.req,
            &mut unit.1,
            &self.extension_sections,
            TIMEOUT_MS,
        )?;
        Ok(())
    }
}

fn analog_input_level_to_str(level: &AnalogInputLevel) -> &str {
    match level {
        AnalogInputLevel::Mic => "Microphone",
        AnalogInputLevel::Line => "Line",
        AnalogInputLevel::Inst => "Instrument",
    }
}

fn mic_amp_emulation_type_to_str(emulation_type: &MicAmpEmulationType) -> &str {
    match emulation_type {
        MicAmpEmulationType::Flat => "Flat",
        MicAmpEmulationType::Trany1h => "TRANY-1-H",
        MicAmpEmulationType::Silver2 => "SILVER-2",
        MicAmpEmulationType::FfRed1h => "FF-RED1-H",
        MicAmpEmulationType::Savillerow => "SAVILLEROW",
        MicAmpEmulationType::Dunk => "DUNK",
        MicAmpEmulationType::ClassA2a => "CLASS-A-2A",
        MicAmpEmulationType::OldTube => "OLD-TUBE",
        MicAmpEmulationType::Deutsch72 => "DEUTSCH-72",
        MicAmpEmulationType::Stellar1b => "STELLAR-1B",
        MicAmpEmulationType::NewAge => "NEW-AGE-1",
    }
}

fn meter_display_target_to_str(target: &MeterDisplayTarget) -> &str {
    match target {
        MeterDisplayTarget::AnalogInput0 => "Analog-input-1",
        MeterDisplayTarget::AnalogInput1 => "Analog-input-2",
        MeterDisplayTarget::AnalogInput2 => "Analog-input-3",
        MeterDisplayTarget::AnalogInput3 => "Analog-input-4",
        MeterDisplayTarget::AnalogInput4 => "Analog-input-5",
        MeterDisplayTarget::AnalogInput5 => "Analog-input-6",
        MeterDisplayTarget::AnalogInput6 => "Analog-input-7",
        MeterDisplayTarget::AnalogInput7 => "Analog-input-8",
        MeterDisplayTarget::SpdifInput0 => "S/PDIF-input-1",
        MeterDisplayTarget::SpdifInput1 => "S/PDIF-input-2",
        MeterDisplayTarget::AdatInput0 => "ADAT-input-1",
        MeterDisplayTarget::AdatInput1 => "ADAT-input-2",
        MeterDisplayTarget::AdatInput2 => "ADAT-input-3",
        MeterDisplayTarget::AdatInput3 => "ADAT-input-4",
        MeterDisplayTarget::AdatInput4 => "ADAT-input-5",
        MeterDisplayTarget::AdatInput5 => "ADAT-input-6",
        MeterDisplayTarget::AdatInput6 => "ADAT-input-7",
        MeterDisplayTarget::AdatInput7 => "ADAT-input-8",
        MeterDisplayTarget::AdatInput8 => "ADAT-input-9",
        MeterDisplayTarget::AdatInput9 => "ADAT-input-10",
        MeterDisplayTarget::AdatInput10 => "ADAT-input-11",
        MeterDisplayTarget::AdatInput11 => "ADAT-input-12",
        MeterDisplayTarget::AdatInput12 => "ADAT-input-13",
        MeterDisplayTarget::AdatInput13 => "ADAT-input-14",
        MeterDisplayTarget::AdatInput14 => "ADAT-input-15",
        MeterDisplayTarget::AdatInput15 => "ADAT-input-16",
    }
}

#[derive(Default, Debug)]
struct SpecificCtl(LiquidS56SpecificParams);

impl SpecificCtl {
    const ANALOG_INPUT_LEVEL_NAME: &'static str = "analog-input-levels";
    const MIC_AMP_EMULATION_TYPE_NAME: &'static str = "mic-amp-emulation-types";
    const MIC_AMP_HARMONICS_NAME: &'static str = "mic-amp-harmonics";
    const MIC_AMP_POLARITY_NAME: &'static str = "mic-amp-polarities";
    const LED_STATE_NAME: &'static str = "LED-states";
    const METER_DISPLAY_TARGETS_NAME: &'static str = "meter-display-targets";

    const ANALOG_INPUT_LEVELS: [AnalogInputLevel; 3] = [
        AnalogInputLevel::Mic,
        AnalogInputLevel::Line,
        AnalogInputLevel::Inst,
    ];

    const MIC_AMP_EMULATION_TYPES: [MicAmpEmulationType; 11] = [
        MicAmpEmulationType::Flat,
        MicAmpEmulationType::Trany1h,
        MicAmpEmulationType::Silver2,
        MicAmpEmulationType::FfRed1h,
        MicAmpEmulationType::Savillerow,
        MicAmpEmulationType::Dunk,
        MicAmpEmulationType::ClassA2a,
        MicAmpEmulationType::OldTube,
        MicAmpEmulationType::Deutsch72,
        MicAmpEmulationType::Stellar1b,
        MicAmpEmulationType::NewAge,
    ];

    const HARMONICS_MIN: i32 = LiquidS56Protocol::MIC_AMP_HARMONICS_MIN as i32;
    const HARMONICS_MAX: i32 = LiquidS56Protocol::MIC_AMP_HARMONICS_MAX as i32;
    const HARMONICS_STEP: i32 = 1;

    const LED_STATE_LABELS: [&'static str; 4] = ["ADAT1", "ADAT2", "S/PDIF", "MIDI-in"];

    const METER_DISPLAY_TARGETS: &'static [MeterDisplayTarget] = &[
        MeterDisplayTarget::AnalogInput0,
        MeterDisplayTarget::AnalogInput1,
        MeterDisplayTarget::AnalogInput2,
        MeterDisplayTarget::AnalogInput3,
        MeterDisplayTarget::AnalogInput4,
        MeterDisplayTarget::AnalogInput5,
        MeterDisplayTarget::AnalogInput6,
        MeterDisplayTarget::AnalogInput7,
        MeterDisplayTarget::SpdifInput0,
        MeterDisplayTarget::SpdifInput1,
        MeterDisplayTarget::AdatInput0,
        MeterDisplayTarget::AdatInput1,
        MeterDisplayTarget::AdatInput2,
        MeterDisplayTarget::AdatInput3,
        MeterDisplayTarget::AdatInput4,
        MeterDisplayTarget::AdatInput5,
        MeterDisplayTarget::AdatInput6,
        MeterDisplayTarget::AdatInput7,
        MeterDisplayTarget::AdatInput8,
        MeterDisplayTarget::AdatInput9,
        MeterDisplayTarget::AdatInput10,
        MeterDisplayTarget::AdatInput11,
        MeterDisplayTarget::AdatInput12,
        MeterDisplayTarget::AdatInput13,
        MeterDisplayTarget::AdatInput14,
        MeterDisplayTarget::AdatInput15,
    ];

    fn cache(
        &mut self,
        req: &FwReq,
        node: &FwNode,
        sections: &ExtensionSections,
        caps: &ExtensionCaps,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let res = LiquidS56Protocol::cache_extension_whole_params(
            req,
            node,
            sections,
            caps,
            &mut self.0,
            timeout_ms,
        );
        debug!(params = ?self.0, ?res);
        res
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let labels: Vec<&str> = Self::ANALOG_INPUT_LEVELS
            .iter()
            .map(|level| analog_input_level_to_str(level))
            .collect();
        let elem_id =
            ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::ANALOG_INPUT_LEVEL_NAME, 0);
        card_cntr.add_enum_elems(&elem_id, 1, 8, &labels, None, true)?;

        let labels: Vec<&str> = Self::MIC_AMP_EMULATION_TYPES
            .iter()
            .map(|emulation_type| mic_amp_emulation_type_to_str(emulation_type))
            .collect();
        let elem_id = ElemId::new_by_name(
            ElemIfaceType::Card,
            0,
            0,
            Self::MIC_AMP_EMULATION_TYPE_NAME,
            0,
        );
        card_cntr.add_enum_elems(&elem_id, 1, 2, &labels, None, true)?;

        let elem_id =
            ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::MIC_AMP_HARMONICS_NAME, 0);
        card_cntr.add_int_elems(
            &elem_id,
            1,
            Self::HARMONICS_MIN,
            Self::HARMONICS_MAX,
            Self::HARMONICS_STEP,
            2,
            None,
            true,
        )?;

        let elem_id =
            ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::MIC_AMP_POLARITY_NAME, 0);
        card_cntr.add_bool_elems(&elem_id, 1, 2, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::LED_STATE_NAME, 0);
        card_cntr.add_bool_elems(&elem_id, 1, Self::LED_STATE_LABELS.len(), true)?;

        let labels: Vec<&str> = Self::METER_DISPLAY_TARGETS
            .iter()
            .map(|target| meter_display_target_to_str(target))
            .collect();
        let elem_id = ElemId::new_by_name(
            ElemIfaceType::Card,
            0,
            0,
            Self::METER_DISPLAY_TARGETS_NAME,
            0,
        );
        card_cntr.add_enum_elems(&elem_id, 1, 8, &labels, None, true)?;

        Ok(())
    }

    fn read(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            Self::ANALOG_INPUT_LEVEL_NAME => {
                let params = &self.0;
                let vals: Vec<u32> = params
                    .analog_input_levels
                    .iter()
                    .map(|level| {
                        let pos = Self::ANALOG_INPUT_LEVELS
                            .iter()
                            .position(|l| level.eq(l))
                            .unwrap();
                        pos as u32
                    })
                    .collect();
                elem_value.set_enum(&vals);
                Ok(true)
            }
            Self::MIC_AMP_EMULATION_TYPE_NAME => {
                let params = &self.0;
                let vals: Vec<u32> = params
                    .mic_amp_emulation_types
                    .iter()
                    .map(|emulation_type| {
                        let pos = Self::MIC_AMP_EMULATION_TYPES
                            .iter()
                            .position(|t| emulation_type.eq(t))
                            .unwrap();
                        pos as u32
                    })
                    .collect();
                elem_value.set_enum(&vals);
                Ok(true)
            }
            Self::MIC_AMP_HARMONICS_NAME => {
                let params = &self.0;
                let vals: Vec<i32> = params
                    .mic_amp_harmonics
                    .iter()
                    .map(|&val| val as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            Self::MIC_AMP_POLARITY_NAME => {
                let params = &self.0;
                elem_value.set_bool(&params.mic_amp_polarities);
                Ok(true)
            }
            Self::LED_STATE_NAME => {
                let params = &self.0;
                let state = &params.led_states;
                elem_value.set_bool(&[state.adat1, state.adat2, state.spdif, state.midi_in]);
                Ok(true)
            }
            Self::METER_DISPLAY_TARGETS_NAME => {
                let params = &self.0;
                let vals: Vec<u32> = params
                    .meter_display_targets
                    .iter()
                    .map(|&target| target as u32)
                    .collect();
                elem_value.set_enum(&vals);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write(
        &mut self,
        req: &FwReq,
        node: &FwNode,
        sections: &ExtensionSections,
        caps: &ExtensionCaps,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            Self::ANALOG_INPUT_LEVEL_NAME => {
                let mut params = self.0.clone();
                params
                    .analog_input_levels
                    .iter_mut()
                    .zip(elem_value.enumerated())
                    .try_for_each(|(level, &val)| {
                        let pos = val as usize;
                        Self::ANALOG_INPUT_LEVELS
                            .iter()
                            .nth(pos)
                            .ok_or_else(|| {
                                let msg = format!("Invalid index of analog input level: {}", pos);
                                Error::new(FileError::Inval, &msg)
                            })
                            .map(|&l| *level = l)
                    })?;
                let res = LiquidS56Protocol::update_extension_partial_params(
                    req,
                    node,
                    sections,
                    caps,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            Self::MIC_AMP_EMULATION_TYPE_NAME => {
                let mut params = self.0.clone();
                params
                    .mic_amp_emulation_types
                    .iter_mut()
                    .zip(elem_value.enumerated())
                    .try_for_each(|(emulation_type, &val)| {
                        let pos = val as usize;
                        Self::MIC_AMP_EMULATION_TYPES
                            .iter()
                            .nth(pos)
                            .ok_or_else(|| {
                                let msg = format!("Invalid index of emulation type: {}", pos);
                                Error::new(FileError::Inval, &msg)
                            })
                            .map(|&t| *emulation_type = t)
                    })?;
                let res = LiquidS56Protocol::update_extension_partial_params(
                    req,
                    node,
                    sections,
                    caps,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            Self::MIC_AMP_HARMONICS_NAME => {
                let mut params = self.0.clone();
                params
                    .mic_amp_harmonics
                    .iter_mut()
                    .zip(elem_value.int())
                    .for_each(|(harmonics, &val)| *harmonics = val as u8);
                let res = LiquidS56Protocol::update_extension_partial_params(
                    req,
                    node,
                    sections,
                    caps,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            Self::MIC_AMP_POLARITY_NAME => {
                let mut params = self.0.clone();
                params
                    .mic_amp_polarities
                    .iter_mut()
                    .zip(elem_value.boolean())
                    .for_each(|(polarity, val)| *polarity = val);
                let res = LiquidS56Protocol::update_extension_partial_params(
                    req,
                    node,
                    sections,
                    caps,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            Self::LED_STATE_NAME => {
                let mut params = self.0.clone();
                let vals = elem_value.boolean();
                params.led_states.adat1 = vals[0];
                params.led_states.adat2 = vals[1];
                params.led_states.spdif = vals[2];
                params.led_states.midi_in = vals[3];
                let res = LiquidS56Protocol::update_extension_partial_params(
                    req,
                    node,
                    sections,
                    caps,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            Self::METER_DISPLAY_TARGETS_NAME => {
                let mut params = self.0.clone();
                params
                    .meter_display_targets
                    .iter_mut()
                    .zip(elem_value.int())
                    .try_for_each(|(target, &val)| {
                        let pos = val as usize;
                        Self::METER_DISPLAY_TARGETS
                            .iter()
                            .nth(pos)
                            .ok_or_else(|| {
                                let msg =
                                    format!("Meter display target not found for position {}", pos);
                                Error::new(FileError::Inval, &msg)
                            })
                            .map(|&t| *target = t)
                    })?;
                let res = LiquidS56Protocol::update_extension_partial_params(
                    req,
                    node,
                    sections,
                    caps,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            _ => Ok(false),
        }
    }
}
