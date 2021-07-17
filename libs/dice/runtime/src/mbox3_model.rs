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
    proto: FwReq,
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
        let node = unit.get_node();

        self.sections = self.proto.read_general_sections(&node, TIMEOUT_MS)?;
        let caps = self.proto.read_clock_caps(&node, &self.sections, TIMEOUT_MS)?;
        let src_labels = self.proto.read_clock_source_labels(&node, &self.sections, TIMEOUT_MS)?;
        self.ctl.load(card_cntr, &caps, &src_labels)?;

        self.extension_sections = self.proto.read_extension_sections(&node, TIMEOUT_MS)?;
        self.tcd22xx_ctl.load(unit, &self.proto, &self.extension_sections, &caps, &src_labels,
                          TIMEOUT_MS, card_cntr)?;
        self.standalone_ctl.load(card_cntr)?;
        self.hw_ctl.load(card_cntr)?;
        self.reverb_ctl.load(card_cntr)?;
        self.button_ctl.load(unit, &self.proto, &self.extension_sections, TIMEOUT_MS, card_cntr)?;

        self.tcd22xx_ctl.cache(unit, &self.proto, &self.sections, &self.extension_sections, TIMEOUT_MS)?;

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
        } else if self.standalone_ctl.read(unit, &self.proto, &self.extension_sections, elem_id,
                                           elem_value, TIMEOUT_MS)? {
            Ok(true)
        } else if self.hw_ctl.read(unit, &self.proto, &self.extension_sections, elem_id,
                                   elem_value, TIMEOUT_MS)? {
            Ok(true)
        } else if self.reverb_ctl.read(unit, &self.proto, &self.extension_sections, elem_id,
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
        if self.ctl.write(unit, &self.proto, &self.sections, elem_id, old, new, TIMEOUT_MS)? {
            Ok(true)
        } else if self.tcd22xx_ctl.write(unit, &self.proto, &self.extension_sections, elem_id,
                                     old, new, TIMEOUT_MS)? {
            Ok(true)
        } else if self.standalone_ctl.write(unit, &self.proto, &self.extension_sections, elem_id,
                                            old, new, TIMEOUT_MS)? {
            Ok(true)
        } else if self.hw_ctl.write(unit, &self.proto, &self.extension_sections, elem_id, old, new,
                                    TIMEOUT_MS)? {
            Ok(true)
        } else if self.reverb_ctl.write(unit, &self.proto, &self.extension_sections, elem_id, old, new,
                                        TIMEOUT_MS)? {
            Ok(true)
        } else if self.button_ctl.write(unit, &self.proto, &self.extension_sections, elem_id, new,
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
        elem_id_list.extend_from_slice(&self.button_ctl.notified_elem_list);
    }

    fn parse_notification(&mut self, unit: &mut SndDice, msg: &u32) -> Result<(), Error> {
        self.ctl.parse_notification(unit, &self.proto, &self.sections, *msg, TIMEOUT_MS)?;
        self.tcd22xx_ctl.parse_notification(unit, &self.proto, &self.sections,
                                        &self.extension_sections, TIMEOUT_MS, *msg)?;
        self.button_ctl.parse_notification(unit, &self.proto, &self.extension_sections, TIMEOUT_MS, *msg)?;
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

impl MeasureModel<hinawa::SndDice> for Mbox3Model {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.ctl.measured_elem_list);
        self.tcd22xx_ctl.get_measured_elem_list(elem_id_list);
    }

    fn measure_states(&mut self, unit: &mut SndDice) -> Result<(), Error> {
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

#[derive(Default)]
struct StandaloneCtl;

impl StandaloneCtl {
    const USE_CASE_NAME: &'static str = "standalone-usecase";

    const USE_CASE_LABELS: [&'static str;3] = [
        "Mixer",
        "AD/DA",
        "Preamp",
    ];

    fn load(&self, card_cntr: &mut CardCntr) -> Result<(), Error>
    {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::USE_CASE_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &Self::USE_CASE_LABELS, None, true)?;
        Ok(())
    }

    fn read(&self, unit: &SndDice, proto: &FwReq, sections: &ExtensionSections,
            elem_id: &ElemId, elem_value: &ElemValue, timeout_ms: u32)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::USE_CASE_NAME=> {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let usecase = proto.read_standalone_use_case(&unit.get_node(), sections, timeout_ms)?;
                    let val = match usecase {
                        StandaloneUseCase::Mixer => 0,
                        StandaloneUseCase::AdDa => 1,
                        StandaloneUseCase::Preamp => 2,
                    };
                    Ok(val)
                })
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }

    fn write(&mut self, unit: &SndDice, proto: &FwReq, sections: &ExtensionSections,
             elem_id: &ElemId, _: &ElemValue, new: &ElemValue, timeout_ms: u32)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::USE_CASE_NAME => {
                ElemValueAccessor::<u32>::get_val(new, |val| {
                    let usecase = match val {
                        0 => StandaloneUseCase::Mixer,
                        1 => StandaloneUseCase::AdDa,
                        2 => StandaloneUseCase::Preamp,
                        _ => {
                            let msg = format!("Invalid value for standalone usecase: {}", val);
                            Err(Error::new(FileError::Inval, &msg))?
                        }
                    };
                    proto.write_standalone_use_case(&unit.get_node(), sections, usecase, timeout_ms)
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

    fn load(&self, card_cntr: &mut CardCntr) -> Result<(), Error>
    {
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

    fn read(&self, unit: &SndDice, proto: &FwReq, sections: &ExtensionSections,
            elem_id: &ElemId, elem_value: &ElemValue, timeout_ms: u32)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::MASTER_KNOB_ASSIGN_NAME => {
                let mut assigns = MasterKnobAssigns::default();
                proto.read_hw_master_knob_assign(&unit.get_node(), sections, &mut assigns, timeout_ms)
                    .map(|_| {
                        elem_value.set_bool(&assigns);
                        true
                    })
            }
            Self::DIM_LED_USAGE_NAME => {
                proto.read_hw_dim_led_usage(&unit.get_node(), sections, timeout_ms)
                    .map(|usage| {
                        elem_value.set_bool(&[usage]);
                        true
                    })
            }
            Self::HOLD_DURATION_NAME => {
                ElemValueAccessor::<i32>::set_val(elem_value, || {
                    proto.read_hw_hold_duration(&unit.get_node(), sections, timeout_ms)
                        .map(|duration| duration as i32)
                })
                .map(|_| true)
            }
            Self::INPUT_HPF_NAME => {
                let mut vals = [false;Self::INPUT_COUNT];
                proto.read_hw_hpf_enable(&unit.get_node(), sections, &mut vals, timeout_ms)
                    .map(|_| {
                        elem_value.set_bool(&vals);
                        true
                    })
            }
            Self::OUTPUT_TRIM_NAME => {
                ElemValueAccessor::<i32>::set_vals(elem_value, Self::OUTPUT_COUNT, |idx| {
                    proto.read_hw_output_trim(&unit.get_node(), sections, idx, timeout_ms)
                        .map(|trim| trim as i32)
                })
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }

    fn write(&mut self, unit: &SndDice, proto: &FwReq, sections: &ExtensionSections,
             elem_id: &ElemId, old: &ElemValue, new: &ElemValue, timeout_ms: u32)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::MASTER_KNOB_ASSIGN_NAME => {
                let mut assign = MasterKnobAssigns::default();
                new.get_bool(&mut assign);
                proto.write_hw_master_knob_assign(&unit.get_node(), sections, &assign, timeout_ms)
                    .map(|_| true)
            }
            Self::DIM_LED_USAGE_NAME => {
                ElemValueAccessor::<bool>::get_val(new, |val| {
                    proto.write_hw_dim_led_usage(&unit.get_node(), sections, val, timeout_ms)
                })
                .map(|_| true)
            }
            Self::HOLD_DURATION_NAME => {
                ElemValueAccessor::<i32>::get_val(new, |val| {
                    proto.write_hw_hold_duration(&unit.get_node(), sections, val as u8, timeout_ms)
                })
                .map(|_| true)
            }
            Self::INPUT_HPF_NAME => {
                let mut vals = [false;Self::INPUT_COUNT];
                new.get_bool(&mut vals);
                proto.write_hw_hpf_enable(&unit.get_node(), sections, vals, timeout_ms)?;
                Ok(true)
            }
            Self::OUTPUT_TRIM_NAME => {
                ElemValueAccessor::<i32>::get_vals(new, old, Self::OUTPUT_COUNT, |idx, val| {
                    proto.write_hw_output_trim(&unit.get_node(), sections, idx, val as u8, timeout_ms)
                })
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

#[derive(Default)]
struct ReverbCtl;

impl ReverbCtl {
    const TYPE_NAME: &'static str = "reverb-type";
    const VOL_NAME: &'static str = "reverb-output-volume";
    const DURATION_NAME: &'static str = "reverb-duration";
    const FEEDBACK_NAME: &'static str = "reverb-feedback";

    const TYPE_LABELS: [&'static str;9] = [
        "Room-1", "Room-2", "Room-3", "Room-4",
        "Hall-1", "Hall-2", "Plate", "Echo",
        "Delay",
    ];

    fn load(&self, card_cntr: &mut CardCntr) -> Result<(), Error>
    {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::TYPE_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &Self::TYPE_LABELS, None, true)?;

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

    fn read(&self, unit: &SndDice, proto: &FwReq, sections: &ExtensionSections,
            elem_id: &ElemId, elem_value: &ElemValue, timeout_ms: u32)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::TYPE_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, ||{
                    proto.read_reverb_type(&unit.get_node(), sections, timeout_ms)
                        .map(|reverb_type| {
                            match reverb_type {
                                ReverbType::Room1 => 0,
                                ReverbType::Room2 => 1,
                                ReverbType::Room3 => 2,
                                ReverbType::Hall1 => 3,
                                ReverbType::Hall2 => 4,
                                ReverbType::Plate => 5,
                                ReverbType::Delay => 6,
                                ReverbType::Echo => 7,
                            }
                        })
                })
                .map(|_| true)
            }
            Self::VOL_NAME => {
                ElemValueAccessor::<i32>::set_val(elem_value, || {
                    proto.read_reverb_volume(&unit.get_node(), sections, timeout_ms)
                        .map(|vol| vol as i32)
                })
                .map(|_| true)
            }
            Self::DURATION_NAME => {
                ElemValueAccessor::<i32>::set_val(elem_value, || {
                    proto.read_reverb_duration(&unit.get_node(), sections, timeout_ms)
                        .map(|duration| duration as i32)
                })
                .map(|_| true)
            }
            Self::FEEDBACK_NAME => {
                ElemValueAccessor::<i32>::set_val(elem_value, || {
                    proto.read_reverb_feedback(&unit.get_node(), sections, timeout_ms)
                        .map(|feedback| feedback as i32)
                })
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }

    fn write(&mut self, unit: &SndDice, proto: &FwReq, sections: &ExtensionSections,
             elem_id: &ElemId, _: &ElemValue, new: &ElemValue, timeout_ms: u32)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::TYPE_NAME => {
                ElemValueAccessor::<u32>::get_val(new, |val| {
                    let reverb_type = match val {
                        0 => ReverbType::Room1,
                        1 => ReverbType::Room2,
                        2 => ReverbType::Room3,
                        3 => ReverbType::Hall1,
                        4 => ReverbType::Hall2,
                        5 => ReverbType::Plate,
                        6 => ReverbType::Delay,
                        7 => ReverbType::Echo,
                        _ => {
                            let msg = format!("Invalid value for index of reverb type: {}", val);
                            Err(Error::new(FileError::Inval, &msg))?
                        }
                    };
                    proto.write_reverb_type(&unit.get_node(), sections, reverb_type, timeout_ms)
                })
                .map(|_| true)
            }
            Self::VOL_NAME => {
                ElemValueAccessor::<i32>::get_val(new, |val| {
                    proto.write_reverb_volume(&unit.get_node(), sections, val as u8, timeout_ms)
                })
                .map(|_| true)
            }
            Self::DURATION_NAME => {
                ElemValueAccessor::<i32>::get_val(new, |val| {
                    proto.write_reverb_duration(&unit.get_node(), sections, val as u8, timeout_ms)
                })
                .map(|_| true)
            }
            Self::FEEDBACK_NAME => {
                ElemValueAccessor::<i32>::get_val(new, |val| {
                    proto.write_reverb_feedback(&unit.get_node(), sections, val as u8, timeout_ms)
                })
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

#[derive(Default)]
struct ButtonCtl{
    button_state: ButtonLedState,
    pub notified_elem_list: Vec<ElemId>,
}

impl ButtonCtl {
    const MUTE_BUTTON_NAME: &'static str = "mute-button";
    const MONO_BUTTON_NAME: &'static str = "mono-button";
    const SPKR_BUTTON_NAME: &'static str = "spkr-button";

    const MUTE_BUTTON_LABELS: [&'static str;3] = [
        "Off",
        "Blink",
        "On",
    ];

    const MONO_BUTTON_LABELS: [&'static str;2] = [
        "Off",
        "On",
    ];

    const SPKR_BUTTON_LABELS: [&'static str;7] = [
        "Off",
        "Green",
        "Green-Blink",
        "Red",
        "Red-Blink",
        "Orange",
        "Orange-Blink",
    ];

    fn load(&mut self, unit: &SndDice, proto: &FwReq, sections: &ExtensionSections, timeout_ms: u32,
            card_cntr: &mut CardCntr) -> Result<(), Error>
    {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::MUTE_BUTTON_NAME, 0);
        let mut elem_id_list = card_cntr.add_enum_elems(&elem_id, 1, 1, &Self::MUTE_BUTTON_LABELS, None, true)?;
        self.notified_elem_list.append(&mut elem_id_list);

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::MONO_BUTTON_NAME, 0);
        let mut elem_id_list = card_cntr.add_enum_elems(&elem_id, 1, 1, &Self::MONO_BUTTON_LABELS, None, true)?;
        self.notified_elem_list.append(&mut elem_id_list);

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::SPKR_BUTTON_NAME, 0);
        let mut elem_id_list = card_cntr.add_enum_elems(&elem_id, 1, 1, &Self::SPKR_BUTTON_LABELS, None, true)?;
        self.notified_elem_list.append(&mut elem_id_list);

        proto.read_hw_button_led_state(&unit.get_node(), sections, timeout_ms)
            .map(|state| self.button_state = state)?;

        Ok(())
    }

    fn read(&self, elem_id: &ElemId, elem_value: &ElemValue) -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::MUTE_BUTTON_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let val = match self.button_state.mute {
                        MuteLedState::Off => 0,
                        MuteLedState::Blink => 1,
                        MuteLedState::On => 2,
                    };
                    Ok(val)
                })
                .map(|_| true)
            }
            Self::MONO_BUTTON_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let val = match self.button_state.mono {
                        MonoLedState::Off => 0,
                        MonoLedState::On => 1,
                    };
                    Ok(val)
                })
                .map(|_| true)
            }
            Self::SPKR_BUTTON_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let val = match self.button_state.spkr {
                        SpkrLedState::Off => 0,
                        SpkrLedState::Green => 1,
                        SpkrLedState::GreenBlink => 2,
                        SpkrLedState::Red => 3,
                        SpkrLedState::RedBlink => 4,
                        SpkrLedState::Orange => 5,
                        SpkrLedState::OrangeBlink => 6,
                    };
                    Ok(val)
                })
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }

    fn write(&mut self, unit: &SndDice, proto: &FwReq, sections: &ExtensionSections,
             elem_id: &ElemId, elem_value: &ElemValue, timeout_ms: u32) -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::MUTE_BUTTON_NAME => {
                ElemValueAccessor::<u32>::get_val(elem_value, |val| {
                    self.button_state.mute = match val {
                        0 => MuteLedState::Off,
                        1 => MuteLedState::Blink,
                        2 => MuteLedState::On,
                        _ => {
                            let msg = format!("Invalid value for index of mute button state: {}", val);
                            Err(Error::new(FileError::Inval, &msg))?
                        }
                    };
                    proto.write_hw_button_led_state(&unit.get_node(), sections, &self.button_state,
                                                    timeout_ms)
                })
                .map(|_| true)
            }
            Self::MONO_BUTTON_NAME => {
                ElemValueAccessor::<u32>::get_val(elem_value, |val| {
                    self.button_state.mono = match val {
                        0 => MonoLedState::Off,
                        1 => MonoLedState::On,
                        _ => {
                            let msg = format!("Invalid value for index of mono button state: {}", val);
                            Err(Error::new(FileError::Inval, &msg))?
                        }
                    };
                    Ok(())
                })
                .map(|_| true)
            }
            Self::SPKR_BUTTON_NAME => {
                ElemValueAccessor::<u32>::get_val(elem_value, |val| {
                    self.button_state.spkr = match val {
                        0 => SpkrLedState::Off,
                        1 => SpkrLedState::GreenBlink,
                        2 => SpkrLedState::Green,
                        3 => SpkrLedState::RedBlink,
                        4 => SpkrLedState::Red,
                        5 => SpkrLedState::OrangeBlink,
                        6 => SpkrLedState::Orange,
                        _ => {
                            let msg = format!("Invalid value for index of mono button state: {}", val);
                            Err(Error::new(FileError::Inval, &msg))?
                        }
                    };
                    Ok(())
                })
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }

    fn parse_notification(&mut self, unit: &SndDice, proto: &FwReq, sections: &ExtensionSections,
                          timeout_ms: u32, msg: u32)
        -> Result<(), Error>
    {
        let mut changed = false;

        if msg.has_spkr_button_pushed() {
            let state = match self.button_state.spkr {
                SpkrLedState::Off => SpkrLedState::Green,
                SpkrLedState::GreenBlink => SpkrLedState::Green,
                SpkrLedState::Green => SpkrLedState::Red,
                SpkrLedState::RedBlink => SpkrLedState::Red,
                SpkrLedState::Red => SpkrLedState::Orange,
                SpkrLedState::OrangeBlink => SpkrLedState::Orange,
                SpkrLedState::Orange => SpkrLedState::Off,
            };
            self.button_state.spkr = state;
            changed = true;
        }

        if msg.has_spkr_button_held() {
            let state = match self.button_state.spkr {
                SpkrLedState::Off => SpkrLedState::Off,
                SpkrLedState::GreenBlink => SpkrLedState::Green,
                SpkrLedState::Green => SpkrLedState::GreenBlink,
                SpkrLedState::RedBlink => SpkrLedState::Red,
                SpkrLedState::Red => SpkrLedState::RedBlink,
                SpkrLedState::OrangeBlink => SpkrLedState::Orange,
                SpkrLedState::Orange => SpkrLedState::OrangeBlink,
            };
            self.button_state.spkr = state;
            changed = true;
        }

        if msg.has_mono_button_pushed() {
            let state = match self.button_state.mono {
                MonoLedState::Off => MonoLedState::On,
                MonoLedState::On => MonoLedState::Off,
            };
            self.button_state.mono = state;
            changed = true;
        }

        if msg.has_mute_button_pushed() {
            let state = match self.button_state.mute {
                MuteLedState::Off => MuteLedState::On,
                MuteLedState::Blink => MuteLedState::On,
                MuteLedState::On => MuteLedState::Off,
            };
            self.button_state.mute = state;
            changed = true;
        }

        if msg.has_mute_button_held() {
            let state = match self.button_state.mute {
                MuteLedState::Off => MuteLedState::Off,
                MuteLedState::Blink => MuteLedState::On,
                MuteLedState::On => MuteLedState::Blink,
            };
            self.button_state.mute = state;
            changed = true;
        }

        if changed {
            proto.write_hw_button_led_state(&unit.get_node(), sections, &self.button_state, timeout_ms)?;
        }

        Ok(())
    }

    fn read_notified_elem(&self, elem_id: &ElemId, elem_value: &ElemValue)
        -> Result<bool, Error>
    {
        self.read(elem_id, elem_value)
    }
}
