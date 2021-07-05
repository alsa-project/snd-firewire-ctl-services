// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto
use glib::{Error, FileError};

use alsactl::{ElemId, ElemIfaceType, ElemValue, ElemValueExt, ElemValueExtManual};

use hinawa::FwReq;
use hinawa::{SndDice, SndUnitExt};

use core::card_cntr::*;
use core::elem_value_accessor::*;

use dice_protocols::tcat::{*, global_section::*};
use dice_protocols::tcat::extension::*;
use dice_protocols::focusrite::liquids56::*;

use crate::common_ctl::*;
use crate::tcd22xx_ctl::*;

use super::out_grp_ctl::*;

#[derive(Default)]
pub struct LiquidS56Model {
    proto: FwReq,
    sections: GeneralSections,
    extension_sections: ExtensionSections,
    ctl: CommonCtl,
    tcd22xx_ctl: Tcd22xxCtl<LiquidS56State>,
    out_grp_ctl: OutGroupCtl,
    specific_ctl: SpecificCtl,
}

const TIMEOUT_MS: u32 = 20;

impl CtlModel<SndDice> for LiquidS56Model {
    fn load(&mut self, unit: &mut SndDice, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let node = unit.get_node();

        self.sections = self.proto.read_general_sections(&node, TIMEOUT_MS)?;
        let caps = self.proto.read_clock_caps(&node, &self.sections, TIMEOUT_MS)?;
        let src_labels = self.proto.read_clock_source_labels(&node, &self.sections, TIMEOUT_MS)?;
        self.ctl.load(card_cntr, &caps, &src_labels)?;

        self.extension_sections = self.proto.read_extension_sections(&node, TIMEOUT_MS)?;
        self.tcd22xx_ctl.load(unit, &self.proto, &self.extension_sections, &caps, &src_labels,
                          TIMEOUT_MS, card_cntr)?;

        self.tcd22xx_ctl.cache(unit, &self.proto, &self.sections, &self.extension_sections, TIMEOUT_MS)?;

        self.out_grp_ctl.load(card_cntr, unit, &self.proto, &self.extension_sections,
                              &mut self.tcd22xx_ctl.state, TIMEOUT_MS)?;
        self.specific_ctl.load(card_cntr)?;

        Ok(())
    }

    fn read(&mut self, unit: &mut SndDice, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        if self.ctl.read(unit, &self.proto, &self.sections, elem_id, elem_value, TIMEOUT_MS)? {
            Ok(true)
        } else if self.tcd22xx_ctl.read(unit, &self.proto, &self.extension_sections, elem_id,
                                    elem_value, TIMEOUT_MS)? {
            Ok(true)
        } else if self.out_grp_ctl.read(&self.tcd22xx_ctl.state, elem_id, elem_value)? {
            Ok(true)
        } else if self.specific_ctl.read(unit, &self.proto, &self.extension_sections, elem_id, elem_value,
                                         TIMEOUT_MS)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(&mut self, unit: &mut SndDice, elem_id: &ElemId, old: &ElemValue, new: &ElemValue)
        -> Result<bool, Error>
    {
        if self.ctl.write(unit, &self.proto, &self.sections, elem_id, old, new, TIMEOUT_MS)? {
            Ok(true)
        } else if self.tcd22xx_ctl.write(unit, &self.proto, &self.extension_sections, elem_id,
                                     old, new, TIMEOUT_MS)? {
            Ok(true)
        } else if self.out_grp_ctl.write(unit, &self.proto, &self.extension_sections,
                                         &mut self.tcd22xx_ctl.state, elem_id, new, TIMEOUT_MS)? {
            Ok(true)
        } else if self.specific_ctl.write(unit, &self.proto, &self.extension_sections, elem_id,
                                          old, new, TIMEOUT_MS)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl NotifyModel<SndDice, u32> for LiquidS56Model {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.ctl.notified_elem_list);
        self.tcd22xx_ctl.get_notified_elem_list(elem_id_list);
        self.out_grp_ctl.get_notified_elem_list(elem_id_list);
    }

    fn parse_notification(&mut self, unit: &SndDice, msg: &u32) -> Result<(), Error> {
        self.ctl.parse_notification(unit, &self.proto, &self.sections, *msg, TIMEOUT_MS)?;
        self.tcd22xx_ctl.parse_notification(unit, &self.proto, &self.sections,
                                        &self.extension_sections, TIMEOUT_MS, *msg)?;
        self.out_grp_ctl.parse_notification(unit, &self.proto, &self.extension_sections,
                                            &mut self.tcd22xx_ctl.state, *msg, TIMEOUT_MS)?;
        Ok(())
    }

    fn read_notified_elem(&mut self, _: &SndDice, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        if self.ctl.read_notified_elem(elem_id, elem_value)? {
            Ok(true)
        } else if self.tcd22xx_ctl.read_notified_elem(elem_id, elem_value)? {
            Ok(true)
        } else if self.out_grp_ctl.read_notified_elem(&self.tcd22xx_ctl.state, elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl MeasureModel<hinawa::SndDice> for LiquidS56Model {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.ctl.measured_elem_list);
        self.tcd22xx_ctl.get_measured_elem_list(elem_id_list);
    }

    fn measure_states(&mut self, unit: &SndDice) -> Result<(), Error> {
        self.ctl.measure_states(unit, &self.proto, &self.sections, TIMEOUT_MS)?;
        self.tcd22xx_ctl.measure_states(unit, &self.proto, &self.extension_sections, TIMEOUT_MS)?;
        Ok(())
    }

    fn measure_elem(&mut self, _: &SndDice, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        if self.ctl.measure_elem(elem_id, elem_value)? {
            Ok(true)
        } else if self.tcd22xx_ctl.measure_elem(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

fn opt_out_iface_mode_to_string(mode: &OptOutIfaceMode) -> String {
    match mode {
        OptOutIfaceMode::Adat => "ADAT",
        OptOutIfaceMode::Spdif => "S/PDIF",
        OptOutIfaceMode::AesEbu => "AES/EBU",
    }.to_string()
}

fn analog_input_level_to_string(level: &AnalogInputLevel) -> String {
    match level {
        AnalogInputLevel::Mic => "Microphone".to_string(),
        AnalogInputLevel::Line => "Line".to_string(),
        AnalogInputLevel::Inst => "Instrument".to_string(),
        AnalogInputLevel::Reserved(val) => format!("Reserved-{}", val),
    }
}

fn mic_amp_emulation_type_to_string(emulation_type: &MicAmpEmulationType) -> String {
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
        MicAmpEmulationType::Reserved(_) => "Reserved",
    }.to_string()
}

#[derive(Default, Debug)]
struct SpecificCtl;

impl<'a> SpecificCtl {
    const ANALOG_OUT_0_1_PAD_NAME: &'a str = "analog-output-1/2-pad";
    const OPT_OUT_IFACE_MODE_NAME: &'a str = "optical-output-interface-mode";
    const MIC_AMP_TRANSFORMER_NAME: &'a str = "mic-amp-transformer";
    const ANALOG_INPUT_LEVEL_NAME: &'a str = "analog-input-levels";
    const MIC_AMP_EMULATION_TYPE_NAME: &'a str = "mic-amp-emulation-types";
    const MIC_AMP_HARMONICS_NAME: &'a str = "mic-amp-harmonics";
    const MIC_AMP_POLARITY_NAME: &'a str = "mic-amp-polarities";
    const LED_STATE_NAME: &'a str = "LED-states";
    const METER_DISPLAY_TARGETS_NAME: &'a str = "meter-display-targets";

    const OPT_OUT_IFACE_MODES: [OptOutIfaceMode;3] = [
        OptOutIfaceMode::Adat,
        OptOutIfaceMode::Spdif,
        OptOutIfaceMode::AesEbu,
    ];

    const ANALOG_INPUT_LEVELS: [AnalogInputLevel;3] = [
        AnalogInputLevel::Mic,
        AnalogInputLevel::Line,
        AnalogInputLevel::Inst,
    ];

    const MIC_AMP_EMULATION_TYPES: [MicAmpEmulationType;11] = [
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

    const HARMONICS_MIN: i32 = 0;
    const HARMONICS_MAX: i32 = 21;
    const HARMONICS_STEP: i32 = 1;

    const LED_STATE_LABELS: [&'a str;4] = ["ADAT1", "ADAT2", "S/PDIF", "MIDI-in"];

    const METER_DISPLAY_TARGETS: [&'a str;26] = [
        "Analog-input-1", "Analog-input-2", "Analog-input-3", "Analog-input-4",
        "Analog-input-5", "Analog-input-6", "Analog-input-7", "Analog-input-8",
        "S/PDIF-input-1", "S/PDIF-input-2",
        "ADAT-input-1", "ADAT-input-2", "ADAT-input-3", "ADAT-input-4",
        "ADAT-input-5", "ADAT-input-6", "ADAT-input-7", "ADAT-input-8",
        "ADAT-input-9", "ADAT-input-10", "ADAT-input-11", "ADAT-input-12",
        "ADAT-input-13", "ADAT-input-14", "ADAT-input-15", "ADAT-input-16",
    ];

    pub fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::ANALOG_OUT_0_1_PAD_NAME, 0);
        card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        let labels: Vec<String> = Self::OPT_OUT_IFACE_MODES.iter()
            .map(|mode| opt_out_iface_mode_to_string(mode))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::OPT_OUT_IFACE_MODE_NAME, 0);
        card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::MIC_AMP_TRANSFORMER_NAME, 0);
        card_cntr.add_bool_elems(&elem_id, 1, 2, true)?;

        let labels: Vec<String> = Self::ANALOG_INPUT_LEVELS.iter()
            .map(|level| analog_input_level_to_string(level))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::ANALOG_INPUT_LEVEL_NAME, 0);
        card_cntr.add_enum_elems(&elem_id, 1, 8, &labels, None, true)?;

        let labels: Vec<String> = Self::MIC_AMP_EMULATION_TYPES.iter()
            .map(|emulation_type| mic_amp_emulation_type_to_string(emulation_type))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::MIC_AMP_EMULATION_TYPE_NAME, 0);
        card_cntr.add_enum_elems(&elem_id, 1, 2, &labels, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::MIC_AMP_HARMONICS_NAME, 0);
        card_cntr.add_int_elems(&elem_id, 1, Self::HARMONICS_MIN, Self::HARMONICS_MAX, Self::HARMONICS_STEP,
                                2, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::MIC_AMP_POLARITY_NAME, 0);
        card_cntr.add_bool_elems(&elem_id, 1, 2, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::LED_STATE_NAME, 0);
        card_cntr.add_bool_elems(&elem_id, 1, Self::LED_STATE_LABELS.len(), true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::METER_DISPLAY_TARGETS_NAME, 0);
        card_cntr.add_enum_elems(&elem_id, 1, 8, &Self::METER_DISPLAY_TARGETS, None, true)?;

        Ok(())
    }

    pub fn read(&mut self, unit: &SndDice, proto: &FwReq, sections: &ExtensionSections, elem_id: &ElemId,
                elem_value: &mut ElemValue, timeout_ms: u32)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::ANALOG_OUT_0_1_PAD_NAME => {
                let enabled = proto.read_analog_out_0_1_pad_offset(&unit.get_node(), sections, timeout_ms)?;
                elem_value.set_bool(&[enabled]);
                Ok(true)
            }
            Self::OPT_OUT_IFACE_MODE_NAME => {
                let mode = proto.read_opt_out_iface_mode(&unit.get_node(), sections, timeout_ms)?;
                let pos = Self::OPT_OUT_IFACE_MODES.iter()
                    .position(|m| m.eq(&mode))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            Self::MIC_AMP_TRANSFORMER_NAME => {
                ElemValueAccessor::<bool>::set_vals(elem_value, 2, |idx| {
                    proto.read_mic_amp_transformer(&unit.get_node(), sections, idx, timeout_ms)
                })
                .map(|_| true)
            }
            Self::ANALOG_INPUT_LEVEL_NAME => {
                let mut levels = [AnalogInputLevel::Reserved(0);8];
                proto.read_analog_input_level(&unit.get_node(), sections, &mut levels, timeout_ms)?;
                let vals: Vec<u32> = levels.iter()
                    .map(|level| {
                        let pos = Self::ANALOG_INPUT_LEVELS.iter()
                            .position(|l| l.eq(level))
                            .unwrap();
                        pos as u32
                    })
                    .collect();
                elem_value.set_enum(&vals);
                Ok(true)
            }
            Self::MIC_AMP_EMULATION_TYPE_NAME => {
                ElemValueAccessor::<u32>::set_vals(elem_value, 2, |idx| {
                    proto.read_mic_amp_emulation_type(&unit.get_node(), sections, idx, timeout_ms)
                        .map(|emulation_type| {
                            let pos = Self::MIC_AMP_EMULATION_TYPES.iter()
                                .position(|t| t.eq(&emulation_type))
                                .unwrap();
                            pos as u32
                        })
                })
                .map(|_| true)
            }
            Self::MIC_AMP_HARMONICS_NAME => {
                ElemValueAccessor::<i32>::set_vals(elem_value, 2, |idx| {
                    proto.read_mic_amp_harmonics(&unit.get_node(), sections, idx, timeout_ms)
                        .map(|harmonics| harmonics as i32)
                })
                .map(|_| true)
            }
            Self::MIC_AMP_POLARITY_NAME => {
                ElemValueAccessor::<bool>::set_vals(elem_value, 2, |idx| {
                    proto.read_mic_amp_polarity(&unit.get_node(), sections, idx, timeout_ms)
                })
                .map(|_| true)
            }
            Self::LED_STATE_NAME => {
                let mut state = LedState::default();
                proto.read_led_state(&unit.get_node(), sections, &mut state, timeout_ms)?;
                let vals = [state.adat1, state.adat2, state.spdif, state.midi_in];
                elem_value.set_bool(&vals);
                Ok(true)
            }
            Self::METER_DISPLAY_TARGETS_NAME => {
                let mut targets = [0;8];
                proto.read_meter_display_targets(&unit.get_node(), sections, &mut targets, timeout_ms)?;
                let vals: Vec<u32> = targets.iter()
                    .map(|&target| {
                        if target < Self::METER_DISPLAY_TARGETS.len() {
                            target as u32
                        } else {
                            0
                        }
                    })
                    .collect();
                elem_value.set_enum(&vals);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub fn write(&mut self, unit: &SndDice, proto: &FwReq, sections: &ExtensionSections, elem_id: &ElemId,
                 old: &ElemValue, new: &ElemValue, timeout_ms: u32)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::ANALOG_OUT_0_1_PAD_NAME => {
                let mut vals = [false];
                new.get_bool(&mut vals);
                proto.write_analog_out_0_1_pad_offset(&unit.get_node(), sections, vals[0], timeout_ms)
                    .map(|_| true)
            }
            Self::OPT_OUT_IFACE_MODE_NAME => {
                let mut vals = [0];
                new.get_enum(&mut vals);
                let mode = Self::OPT_OUT_IFACE_MODES.iter()
                    .nth(vals[0] as usize)
                    .ok_or_else(|| {
                        let msg = format!("Invalid value for index of optical output interface mode: {}", vals[0]);
                        Error::new(FileError::Inval, &msg)
                    })?;
                proto.write_opt_out_iface_mode(&unit.get_node(), sections, *mode, timeout_ms)
                    .map(|_| true)
            }
            Self::MIC_AMP_TRANSFORMER_NAME => {
                ElemValueAccessor::<bool>::get_vals(new, old, 2, |idx, val| {
                    proto.write_mic_amp_transformer(&unit.get_node(), sections, idx, val, timeout_ms)
                })
                .map(|_| true)
            }
            Self::ANALOG_INPUT_LEVEL_NAME => {
                let mut vals = [0;8];
                new.get_enum(&mut vals);
                let mut levels = [AnalogInputLevel::Reserved(0);8];
                levels.iter_mut()
                    .zip(vals.iter())
                    .enumerate()
                    .try_for_each(|(i, (level, &val))| {
                        let l = Self::ANALOG_INPUT_LEVELS.iter()
                            .nth(val as usize)
                            .ok_or_else(|| {
                                let msg = format!("Invalid value for index of analog input level: {}", val);
                                Error::new(FileError::Inval, &msg)
                            })?;

                        if (i < 2 || i > 3) && *l == AnalogInputLevel::Inst {
                            let msg = "Instrument level is just available for channel 3 and 4";
                            Err(Error::new(FileError::Inval, &msg))
                        } else {
                            *level = *l;
                            Ok(())
                        }
                    })?;
                proto.write_analog_input_level(&unit.get_node(), sections, &levels, timeout_ms)
                    .map(|_| true)
            }
            Self::MIC_AMP_EMULATION_TYPE_NAME => {
                ElemValueAccessor::<u32>::get_vals(new, old, 2, |idx, val| {
                    Self::MIC_AMP_EMULATION_TYPES.iter()
                        .nth(val as usize)
                        .ok_or_else(|| {
                            let msg = format!("Invalid value for index of emulation type: {}", val);
                            Error::new(FileError::Inval, &msg)
                        })
                        .and_then(|&emulation_type| {
                            proto.write_mic_amp_emulation_type(&unit.get_node(), sections, idx,
                                                               emulation_type, timeout_ms)
                        })
                })
                .map(|_| true)
            }
            Self::MIC_AMP_HARMONICS_NAME => {
                ElemValueAccessor::<i32>::get_vals(new, old, 2, |idx, val| {
                    proto.write_mic_amp_harmonics(&unit.get_node(), sections, idx, val as u8, timeout_ms)
                })
                .map(|_| true)
            }
            Self::MIC_AMP_POLARITY_NAME => {
                ElemValueAccessor::<bool>::get_vals(new, old, 2, |idx, val| {
                    proto.write_mic_amp_polarity(&unit.get_node(), sections, idx, val, timeout_ms)
                })
                .map(|_| true)
            }
            Self::LED_STATE_NAME => {
                let mut vals = [false;4];
                new.get_bool(&mut vals);
                let state = LedState{adat1: vals[0], adat2: vals[1], spdif: vals[2], midi_in: vals[3]};
                proto.write_led_state(&unit.get_node(), sections, &state, timeout_ms)
                    .map(|_| true)
            }
            Self::METER_DISPLAY_TARGETS_NAME => {
                let mut vals = [0;8];
                new.get_enum(&mut vals);
                let mut targets = [0;8];
                targets.iter_mut()
                    .zip(vals.iter())
                    .try_for_each(|(target, &val)| {
                        if val < Self::METER_DISPLAY_TARGETS.len() as u32 {
                            *target = val as usize;
                            Ok(())
                        } else {
                            let msg = format!("Invalid value for index of meter display target: {}", val);
                            Err(Error::new(FileError::Inval, &msg))
                        }
                    })?;
                proto.write_meter_display_targets(&unit.get_node(), sections, &targets, timeout_ms)
                    .map(|_| true)
            }
            _ => Ok(false),
        }
    }
}
