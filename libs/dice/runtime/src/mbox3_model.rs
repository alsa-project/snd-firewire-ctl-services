// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::{Error, FileError};

use alsactl::{ElemId, ElemIfaceType, ElemValue, ElemValueExtManual};

use hinawa::FwReq;
use hinawa::{SndDice, SndUnitExt};

use core::card_cntr::*;
use core::elem_value_accessor::*;

use dice_protocols::tcat::{*, global_section::*};
use dice_protocols::tcat::extension::*;
use dice_protocols::avid::*;

use super::common_ctl::*;
use super::tcd22xx_ctl::*;

#[derive(Default)]
pub struct Mbox3Model{
    req: FwReq,
    sections: GeneralSections,
    extension_sections: ExtensionSections,
    ctl: CommonCtl,
    tcd22xx_ctl: Tcd22xxCtl<Mbox3State>,
    standalone_ctl: StandaloneCtl,
    hw_ctl: HwCtl,
    reverb_ctl: ReverbCtl,
    button_ctl: ButtonCtl,
}

const TIMEOUT_MS: u32 = 20;

impl CtlModel<SndDice> for Mbox3Model {
    fn load(&mut self, unit: &mut SndDice, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let mut node = unit.get_node();

        self.sections = self.req.read_general_sections(&mut node, TIMEOUT_MS)?;
        let caps = self.req.read_clock_caps(&mut node, &self.sections, TIMEOUT_MS)?;
        let src_labels = self.req.read_clock_source_labels(&mut node, &self.sections, TIMEOUT_MS)?;
        self.ctl.load(card_cntr, &caps, &src_labels)?;

        self.extension_sections = self.req.read_extension_sections(&mut node, TIMEOUT_MS)?;
        self.tcd22xx_ctl.load(unit, &mut self.req, &self.extension_sections, &caps, &src_labels,
                          TIMEOUT_MS, card_cntr)?;
        self.standalone_ctl.load(card_cntr)?;
        self.hw_ctl.load(card_cntr)?;
        self.reverb_ctl.load(card_cntr)?;
        self.button_ctl.load(unit, &mut self.req, &self.extension_sections, TIMEOUT_MS, card_cntr)?;

        self.tcd22xx_ctl.cache(unit, &mut self.req, &self.sections, &self.extension_sections, TIMEOUT_MS)?;

        Ok(())
    }

    fn read(&mut self, unit: &mut SndDice, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        if self.ctl.read(unit, &mut self.req, &self.sections, elem_id, elem_value, TIMEOUT_MS)? {
            Ok(true)
        } else if self.tcd22xx_ctl.read(unit, &mut self.req, &self.extension_sections, elem_id,
                                    elem_value, TIMEOUT_MS)? {
            Ok(true)
        } else if self.standalone_ctl.read(unit, &mut self.req, &self.extension_sections, elem_id,
                                           elem_value, TIMEOUT_MS)? {
            Ok(true)
        } else if self.hw_ctl.read(unit, &mut self.req, &self.extension_sections, elem_id,
                                   elem_value, TIMEOUT_MS)? {
            Ok(true)
        } else if self.reverb_ctl.read(unit, &mut self.req, &self.extension_sections, elem_id,
                                       elem_value, TIMEOUT_MS)? {
            Ok(true)
        } else if self.button_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(&mut self, unit: &mut SndDice, elem_id: &ElemId, old: &ElemValue, new: &ElemValue)
        -> Result<bool, Error>
    {
        if self.ctl.write(unit, &mut self.req, &self.sections, elem_id, old, new, TIMEOUT_MS)? {
            Ok(true)
        } else if self.tcd22xx_ctl.write(unit, &mut self.req, &self.extension_sections, elem_id,
                                     old, new, TIMEOUT_MS)? {
            Ok(true)
        } else if self.standalone_ctl.write(unit, &mut self.req, &self.extension_sections, elem_id,
                                            old, new, TIMEOUT_MS)? {
            Ok(true)
        } else if self.hw_ctl.write(unit, &mut self.req, &self.extension_sections, elem_id, old, new,
                                    TIMEOUT_MS)? {
            Ok(true)
        } else if self.reverb_ctl.write(unit, &mut self.req, &self.extension_sections, elem_id, old, new,
                                        TIMEOUT_MS)? {
            Ok(true)
        } else if self.button_ctl.write(unit, &mut self.req, &self.extension_sections, elem_id, new,
                                        TIMEOUT_MS)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl NotifyModel<SndDice, u32> for Mbox3Model {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.ctl.notified_elem_list);
        self.tcd22xx_ctl.get_notified_elem_list(elem_id_list);
        elem_id_list.extend_from_slice(&self.button_ctl.1);
    }

    fn parse_notification(&mut self, unit: &mut SndDice, msg: &u32) -> Result<(), Error> {
        self.ctl.parse_notification(unit, &mut self.req, &self.sections, *msg, TIMEOUT_MS)?;
        self.tcd22xx_ctl.parse_notification(unit, &mut self.req, &self.sections,
                                        &self.extension_sections, TIMEOUT_MS, *msg)?;
        self.button_ctl.parse_notification(unit, &mut self.req, &self.extension_sections, TIMEOUT_MS, *msg)?;
        Ok(())
    }

    fn read_notified_elem(&mut self, _: &SndDice, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        if self.ctl.read_notified_elem(elem_id, elem_value)? {
            Ok(true)
        } else if self.tcd22xx_ctl.read_notified_elem(elem_id, elem_value)? {
            Ok(true)
        } else if self.button_ctl.read_notified_elem(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl MeasureModel<SndDice> for Mbox3Model {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.ctl.measured_elem_list);
        self.tcd22xx_ctl.get_measured_elem_list(elem_id_list);
    }

    fn measure_states(&mut self, unit: &mut SndDice) -> Result<(), Error> {
        self.ctl.measure_states(unit, &mut self.req, &self.sections, TIMEOUT_MS)?;
        self.tcd22xx_ctl.measure_states(unit, &mut self.req, &self.extension_sections, TIMEOUT_MS)?;
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

#[derive(Default)]
struct StandaloneCtl;

fn standalone_use_case_to_str(case: &StandaloneUseCase) -> &'static str {
    match case {
        StandaloneUseCase::Mixer => "Mixer",
        StandaloneUseCase::AdDa => "AD/DA",
        StandaloneUseCase::Preamp => "Preamp",
    }
}

impl StandaloneCtl {
    const USE_CASE_NAME: &'static str = "standalone-usecase";

    const USE_CASES: [StandaloneUseCase; 3] = [
        StandaloneUseCase::Mixer,
        StandaloneUseCase::AdDa,
        StandaloneUseCase::Preamp,
    ];

    fn load(&self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let labels: Vec<&str> = Self::USE_CASES.iter()
            .map(|c| standalone_use_case_to_str(c))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::USE_CASE_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;
        Ok(())
    }

    fn read(
        &self,
        unit: &mut SndDice,
        req: &mut FwReq,
        sections: &ExtensionSections,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
        timeout_ms: u32
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            Self::USE_CASE_NAME=> {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let usecase = req.read_standalone_use_case(
                        &mut unit.get_node(),
                        sections,
                        timeout_ms
                    )?;
                    let pos = Self::USE_CASES.iter().position(|c| usecase.eq(c)).unwrap();
                    Ok(pos as u32)
                })
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }

    fn write(
        &mut self,
        unit: &mut SndDice,
        req: &mut FwReq,
        sections: &ExtensionSections,
        elem_id: &ElemId,
        _: &ElemValue,
        new: &ElemValue,
        timeout_ms: u32
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            Self::USE_CASE_NAME => {
                ElemValueAccessor::<u32>::get_val(new, |val| {
                    let &usecase = Self::USE_CASES.iter().nth(val as usize).ok_or_else(|| {
                        let msg = format!("Invalid value for standalone usecase: {}", val);
                        Error::new(FileError::Inval, &msg)
                    })?;
                    AvidMbox3StandaloneProtocol::write_standalone_use_case(
                        req,
                        &mut unit.get_node(),
                        sections,
                        usecase,
                        timeout_ms
                    )
                })
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

#[derive(Default)]
struct HwCtl;

impl HwCtl {
    const MASTER_KNOB_ASSIGN_NAME: &'static str = "master-knob-assign";
    const DIM_LED_USAGE_NAME: &'static str = "dim-led";
    const HOLD_DURATION_NAME: &'static str = "hold-duration";
    const INPUT_HPF_NAME: &'static str = "input-hp-filter";
    const OUTPUT_TRIM_NAME: &'static str = "output-trim";

    const HOLD_DURATION_MAX: i32 = 1000;
    const HOLD_DURATION_MIN: i32 = 0;
    const HOLD_DURATION_STEP: i32 = 1;

    const INPUT_COUNT: usize = 4;
    const OUTPUT_COUNT: usize = 6;

    fn load(&self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::MASTER_KNOB_ASSIGN_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, Self::OUTPUT_COUNT, true);

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::DIM_LED_USAGE_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true);

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::HOLD_DURATION_NAME, 0);
        let _ = card_cntr.add_int_elems(&elem_id, 1,
                    Self::HOLD_DURATION_MIN, Self::HOLD_DURATION_MAX, Self::HOLD_DURATION_STEP,
                    1, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::INPUT_HPF_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, Self::INPUT_COUNT, true);

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::OUTPUT_TRIM_NAME, 0);
        let _ = card_cntr.add_int_elems(&elem_id, 1, u8::MIN as i32, u8::MAX as i32, 1, 1, None, true)?;

        Ok(())
    }

    fn read(
        &self,
        unit: &mut SndDice,
        req: &mut FwReq,
        sections: &ExtensionSections,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            Self::MASTER_KNOB_ASSIGN_NAME => {
                let mut assigns = MasterKnobAssigns::default();
                req.read_hw_master_knob_assign(
                    &mut unit.get_node(),
                    sections,
                    &mut assigns,
                    timeout_ms
                )
                    .map(|_| {
                        elem_value.set_bool(&assigns);
                        true
                    })
            }
            Self::DIM_LED_USAGE_NAME => {
                req.read_hw_dim_led_usage(&mut unit.get_node(), sections, timeout_ms)
                    .map(|usage| {
                        elem_value.set_bool(&[usage]);
                        true
                    })
            }
            Self::HOLD_DURATION_NAME => {
                ElemValueAccessor::<i32>::set_val(elem_value, || {
                    req.read_hw_hold_duration(&mut unit.get_node(), sections, timeout_ms)
                        .map(|duration| duration as i32)
                })
                .map(|_| true)
            }
            Self::INPUT_HPF_NAME => {
                let mut vals = [false;Self::INPUT_COUNT];
                req.read_hw_hpf_enable(&mut unit.get_node(), sections, &mut vals, timeout_ms)
                    .map(|_| {
                        elem_value.set_bool(&vals);
                        true
                    })
            }
            Self::OUTPUT_TRIM_NAME => {
                ElemValueAccessor::<i32>::set_vals(elem_value, Self::OUTPUT_COUNT, |idx| {
                    req.read_hw_output_trim(&mut unit.get_node(), sections, idx, timeout_ms)
                        .map(|trim| trim as i32)
                })
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }

    fn write(
        &mut self,
        unit: &mut SndDice,
        req: &mut FwReq,
        sections: &ExtensionSections,
        elem_id: &ElemId,
        old: &ElemValue,
        new: &ElemValue,
        timeout_ms: u32
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            Self::MASTER_KNOB_ASSIGN_NAME => {
                let mut assign = MasterKnobAssigns::default();
                new.get_bool(&mut assign);
                req.write_hw_master_knob_assign(&mut unit.get_node(), sections, &assign, timeout_ms)
                    .map(|_| true)
            }
            Self::DIM_LED_USAGE_NAME => {
                ElemValueAccessor::<bool>::get_val(new, |val| {
                    req.write_hw_dim_led_usage(&mut unit.get_node(), sections, val, timeout_ms)
                })
                .map(|_| true)
            }
            Self::HOLD_DURATION_NAME => {
                ElemValueAccessor::<i32>::get_val(new, |val| {
                    req.write_hw_hold_duration(&mut unit.get_node(), sections, val as u8, timeout_ms)
                })
                .map(|_| true)
            }
            Self::INPUT_HPF_NAME => {
                let mut vals = [false;Self::INPUT_COUNT];
                new.get_bool(&mut vals);
                req.write_hw_hpf_enable(&mut unit.get_node(), sections, vals, timeout_ms)?;
                Ok(true)
            }
            Self::OUTPUT_TRIM_NAME => {
                ElemValueAccessor::<i32>::get_vals(new, old, Self::OUTPUT_COUNT, |idx, val| {
                    req.write_hw_output_trim(&mut unit.get_node(), sections, idx, val as u8, timeout_ms)
                })
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

#[derive(Default)]
struct ReverbCtl;

fn reverb_type_to_str(reverb_type: &ReverbType) -> &'static str {
    match reverb_type {
        ReverbType::Room1 => "Room-1",
        ReverbType::Room2 => "Room-2",
        ReverbType::Room3 => "Room-3",
        ReverbType::Hall1 => "Hall-1",
        ReverbType::Hall2 => "Hall-2",
        ReverbType::Plate => "Plate",
        ReverbType::Delay => "Echo",
        ReverbType::Echo => "Delay",
    }
}

impl ReverbCtl {
    const TYPE_NAME: &'static str = "reverb-type";
    const VOL_NAME: &'static str = "reverb-output-volume";
    const DURATION_NAME: &'static str = "reverb-duration";
    const FEEDBACK_NAME: &'static str = "reverb-feedback";

    const TYPES: [ReverbType; 8] = [
        ReverbType::Room1,
        ReverbType::Room2,
        ReverbType::Room3,
        ReverbType::Hall1,
        ReverbType::Hall2,
        ReverbType::Plate,
        ReverbType::Delay,
        ReverbType::Echo,
    ];

    fn load(&self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let labels: Vec<&str> = Self::TYPES.iter().map(|t| reverb_type_to_str(t)).collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::TYPE_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::VOL_NAME, 0);
        let _ = card_cntr.add_int_elems(&elem_id, 1, u8::MIN as i32, u8::MAX as i32, 1,
                                        1, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::DURATION_NAME, 0);
        let _ = card_cntr.add_int_elems(&elem_id, 1, u8::MIN as i32, u8::MAX as i32, 1,
                                        1, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::FEEDBACK_NAME, 0);
        let _ = card_cntr.add_int_elems(&elem_id, 1, u8::MIN as i32, u8::MAX as i32, 1,
                                        1, None, true)?;

        Ok(())
    }

    fn read(
        &self,
        unit: &mut SndDice,
        req: &mut FwReq,
        sections: &ExtensionSections,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
        timeout_ms: u32
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            Self::TYPE_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, ||{
                    let reverb_type = req.read_reverb_type(
                        &mut unit.get_node(),
                        sections,
                        timeout_ms
                    )?;
                    let pos = Self::TYPES.iter().position(|t| reverb_type.eq(t)).unwrap();
                    Ok(pos as u32)
                })
                .map(|_| true)
            }
            Self::VOL_NAME => {
                ElemValueAccessor::<i32>::set_val(elem_value, || {
                    req.read_reverb_volume(&mut unit.get_node(), sections, timeout_ms)
                        .map(|vol| vol as i32)
                })
                .map(|_| true)
            }
            Self::DURATION_NAME => {
                ElemValueAccessor::<i32>::set_val(elem_value, || {
                    req.read_reverb_duration(&mut unit.get_node(), sections, timeout_ms)
                        .map(|duration| duration as i32)
                })
                .map(|_| true)
            }
            Self::FEEDBACK_NAME => {
                ElemValueAccessor::<i32>::set_val(elem_value, || {
                    req.read_reverb_feedback(&mut unit.get_node(), sections, timeout_ms)
                        .map(|feedback| feedback as i32)
                })
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }

    fn write(
        &mut self,
        unit: &mut SndDice,
        req: &mut FwReq,
        sections: &ExtensionSections,
        elem_id: &ElemId,
        _: &ElemValue,
        new: &ElemValue,
        timeout_ms: u32
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            Self::TYPE_NAME => {
                ElemValueAccessor::<u32>::get_val(new, |val| {
                    let &reverb_type = Self::TYPES.iter().nth(val as usize).ok_or_else(|| {
                        let msg = format!("Invalid value for index of reverb type: {}", val);
                        Error::new(FileError::Inval, &msg)
                    })?;
                    req.write_reverb_type(&mut unit.get_node(), sections, reverb_type, timeout_ms)
                })
                .map(|_| true)
            }
            Self::VOL_NAME => {
                ElemValueAccessor::<i32>::get_val(new, |val| {
                    req.write_reverb_volume(&mut unit.get_node(), sections, val as u8, timeout_ms)
                })
                .map(|_| true)
            }
            Self::DURATION_NAME => {
                ElemValueAccessor::<i32>::get_val(new, |val| {
                    req.write_reverb_duration(&mut unit.get_node(), sections, val as u8, timeout_ms)
                })
                .map(|_| true)
            }
            Self::FEEDBACK_NAME => {
                ElemValueAccessor::<i32>::get_val(new, |val| {
                    req.write_reverb_feedback(&mut unit.get_node(), sections, val as u8, timeout_ms)
                })
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

#[derive(Default)]
struct ButtonCtl(ButtonLedState, Vec<ElemId>);

fn mute_led_state_to_str(state: &MuteLedState) -> &'static str {
    match state {
        MuteLedState::Off => "Off",
        MuteLedState::Blink => "Blink",
        MuteLedState::On => "On",
    }
}

fn mono_led_state_to_str(state: &MonoLedState) -> &'static str {
    match state {
        MonoLedState::Off => "Off",
        MonoLedState::On => "On",
    }
}

fn spkr_led_state_to_str(state: &SpkrLedState) -> &'static str {
    match state {
        SpkrLedState::Off => "Off",
        SpkrLedState::Green => "Green",
        SpkrLedState::GreenBlink => "Green-Blink",
        SpkrLedState::Red => "Red",
        SpkrLedState::RedBlink => "Red-Blink",
        SpkrLedState::Orange => "Orange",
        SpkrLedState::OrangeBlink => "Orange-Blink",
    }
}

impl ButtonCtl {
    const MUTE_BUTTON_NAME: &'static str = "mute-button";
    const MONO_BUTTON_NAME: &'static str = "mono-button";
    const SPKR_BUTTON_NAME: &'static str = "spkr-button";

    const MUTE_LED_STATES: [MuteLedState; 3] = [
        MuteLedState::Off,
        MuteLedState::Blink,
        MuteLedState::On,
    ];

    const MONO_LED_STATES: [MonoLedState; 2] = [
        MonoLedState::Off,
        MonoLedState::On,
    ];

    const SPKR_LED_STATES: [SpkrLedState; 7] = [
        SpkrLedState::Off,
        SpkrLedState::Green,
        SpkrLedState::GreenBlink,
        SpkrLedState::Red,
        SpkrLedState::RedBlink,
        SpkrLedState::Orange,
        SpkrLedState::OrangeBlink,
    ];

    fn load(
        &mut self,
        unit: &mut SndDice,
        req: &mut FwReq,
        sections: &ExtensionSections,
        timeout_ms: u32,
        card_cntr: &mut CardCntr
    ) -> Result<(), Error> {
        let labels: Vec<&str> = Self::MUTE_LED_STATES.iter()
            .map(|s| mute_led_state_to_str(s))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::MUTE_BUTTON_NAME, 0);
        let mut elem_id_list = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;
        self.1.append(&mut elem_id_list);

        let labels: Vec<&str> = Self::MONO_LED_STATES.iter()
            .map(|s| mono_led_state_to_str(s))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::MONO_BUTTON_NAME, 0);
        let mut elem_id_list = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;
        self.1.append(&mut elem_id_list);

        let labels: Vec<&str> = Self::SPKR_LED_STATES.iter()
            .map(|s| spkr_led_state_to_str(s))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::SPKR_BUTTON_NAME, 0);
        let mut elem_id_list = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;
        self.1.append(&mut elem_id_list);

        req.read_hw_button_led_state(&mut unit.get_node(), sections, timeout_ms)
            .map(|state| self.0 = state)?;

        Ok(())
    }

    fn read(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            Self::MUTE_BUTTON_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let pos = Self::MUTE_LED_STATES.iter().position(|s| self.0.mute.eq(s)).unwrap();
                    Ok(pos as u32)
                })
                .map(|_| true)
            }
            Self::MONO_BUTTON_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let pos = Self::MONO_LED_STATES.iter().position(|s| self.0.mono.eq(s)).unwrap();
                    Ok(pos as u32)
                })
                .map(|_| true)
            }
            Self::SPKR_BUTTON_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let pos = Self::SPKR_LED_STATES.iter().position(|s| self.0.spkr.eq(s)).unwrap();
                    Ok(pos as u32)
                })
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }

    fn write(
        &mut self,
        unit: &mut SndDice,
        req: &mut FwReq,
        sections: &ExtensionSections,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            Self::MUTE_BUTTON_NAME => {
                ElemValueAccessor::<u32>::get_val(elem_value, |val| {
                    let mut state = self.0.clone();
                    state.mute = Self::MUTE_LED_STATES
                        .iter()
                        .nth(val as usize)
                        .map(|&s| s)
                        .ok_or_else(|| {
                            let msg = format!("Invalid value for index of mute button state: {}", val);
                            Error::new(FileError::Inval, &msg)
                        })?;
                    req.write_hw_button_led_state(
                        &mut unit.get_node(),
                        sections,
                        &state,
                        timeout_ms
                    )
                        .map(|_| self.0 = state)
                })
                .map(|_| true)
            }
            Self::MONO_BUTTON_NAME => {
                ElemValueAccessor::<u32>::get_val(elem_value, |val| {
                    let mut state = self.0.clone();
                    state.mono = Self::MONO_LED_STATES
                        .iter()
                        .nth(val as usize)
                        .map(|&s| s)
                        .ok_or_else(|| {
                            let msg = format!("Invalid value for index of mono button state: {}", val);
                            Error::new(FileError::Inval, &msg)
                        })?;
                    req.write_hw_button_led_state(
                        &mut unit.get_node(),
                        sections,
                        &state,
                        timeout_ms
                    )
                        .map(|_| self.0 = state)
                })
                .map(|_| true)
            }
            Self::SPKR_BUTTON_NAME => {
                ElemValueAccessor::<u32>::get_val(elem_value, |val| {
                    let mut state = self.0.clone();
                    state.spkr = Self::SPKR_LED_STATES
                        .iter()
                        .nth(val as usize)
                        .map(|&s| s)
                        .ok_or_else(|| {
                            let msg = format!("Invalid value for index of mono button state: {}", val);
                            Error::new(FileError::Inval, &msg)
                        })?;
                    req.write_hw_button_led_state(
                        &mut unit.get_node(),
                        sections,
                        &state,
                        timeout_ms
                    )
                        .map(|_| self.0 = state)
                })
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }

    fn parse_notification(
        &mut self,
        unit: &mut SndDice,
        req: &mut FwReq,
        sections: &ExtensionSections,
        timeout_ms: u32,
        msg: u32
    ) -> Result<(), Error> {
        let mut changed = false;

        if msg.has_spkr_button_pushed() {
            let state = match self.0.spkr {
                SpkrLedState::Off => SpkrLedState::Green,
                SpkrLedState::GreenBlink => SpkrLedState::Green,
                SpkrLedState::Green => SpkrLedState::Red,
                SpkrLedState::RedBlink => SpkrLedState::Red,
                SpkrLedState::Red => SpkrLedState::Orange,
                SpkrLedState::OrangeBlink => SpkrLedState::Orange,
                SpkrLedState::Orange => SpkrLedState::Off,
            };
            self.0.spkr = state;
            changed = true;
        }

        if msg.has_spkr_button_held() {
            let state = match self.0.spkr {
                SpkrLedState::Off => SpkrLedState::Off,
                SpkrLedState::GreenBlink => SpkrLedState::Green,
                SpkrLedState::Green => SpkrLedState::GreenBlink,
                SpkrLedState::RedBlink => SpkrLedState::Red,
                SpkrLedState::Red => SpkrLedState::RedBlink,
                SpkrLedState::OrangeBlink => SpkrLedState::Orange,
                SpkrLedState::Orange => SpkrLedState::OrangeBlink,
            };
            self.0.spkr = state;
            changed = true;
        }

        if msg.has_mono_button_pushed() {
            let state = match self.0.mono {
                MonoLedState::Off => MonoLedState::On,
                MonoLedState::On => MonoLedState::Off,
            };
            self.0.mono = state;
            changed = true;
        }

        if msg.has_mute_button_pushed() {
            let state = match self.0.mute {
                MuteLedState::Off => MuteLedState::On,
                MuteLedState::Blink => MuteLedState::On,
                MuteLedState::On => MuteLedState::Off,
            };
            self.0.mute = state;
            changed = true;
        }

        if msg.has_mute_button_held() {
            let state = match self.0.mute {
                MuteLedState::Off => MuteLedState::Off,
                MuteLedState::Blink => MuteLedState::On,
                MuteLedState::On => MuteLedState::Blink,
            };
            self.0.mute = state;
            changed = true;
        }

        if changed {
            req.write_hw_button_led_state(
                &mut unit.get_node(),
                sections,
                &self.0,
                timeout_ms
            )?;
        }

        Ok(())
    }

    fn read_notified_elem(
        &self,
        elem_id: &ElemId,
        elem_value: &mut ElemValue
    ) -> Result<bool, Error> {
        self.read(elem_id, elem_value)
    }
}
