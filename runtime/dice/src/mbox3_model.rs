// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use {
    super::{tcd22xx_ctl::*, *},
    protocols::{avid::*, tcat::extension::*},
};

#[derive(Default)]
pub struct Mbox3Model {
    req: FwReq,
    sections: GeneralSections,
    extension_sections: ExtensionSections,
    common_ctl: CommonCtl<Mbox3Protocol>,
    tcd22xx_ctls: Tcd22xxCtls<Mbox3Protocol>,
    specific_ctl: SpecificCtl,
}

const TIMEOUT_MS: u32 = 20;

impl Mbox3Model {
    pub(crate) fn store_configuration(&mut self, node: &FwNode) -> Result<(), Error> {
        self.tcd22xx_ctls.store_configuration(
            &mut self.req,
            node,
            &self.extension_sections,
            TIMEOUT_MS,
        )
    }
}

impl CtlModel<(SndDice, FwNode)> for Mbox3Model {
    fn cache(&mut self, unit: &mut (SndDice, FwNode)) -> Result<(), Error> {
        Mbox3Protocol::read_general_sections(&self.req, &unit.1, &mut self.sections, TIMEOUT_MS)?;

        self.common_ctl
            .cache_whole_params(&self.req, &unit.1, &mut self.sections, TIMEOUT_MS)?;

        Mbox3Protocol::read_extension_sections(
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

impl NotifyModel<(SndDice, FwNode), u32> for Mbox3Model {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.common_ctl.notified_elem_id_list);
        elem_id_list.extend_from_slice(&self.tcd22xx_ctls.notified_elem_id_list);
        elem_id_list.extend_from_slice(&self.specific_ctl.1);
    }

    fn parse_notification(
        &mut self,
        unit: &mut (SndDice, FwNode),
        &msg: &u32,
    ) -> Result<(), Error> {
        self.common_ctl.parse_notification(
            &self.req,
            &unit.1,
            &mut self.sections,
            msg,
            TIMEOUT_MS,
        )?;
        self.tcd22xx_ctls.parse_notification(
            &mut self.req,
            &mut unit.1,
            &self.extension_sections,
            &self.common_ctl.global_params,
            TIMEOUT_MS,
            msg,
        )?;
        self.specific_ctl.parse_notification(
            &self.req,
            &unit.1,
            &self.extension_sections,
            &self.tcd22xx_ctls.caps,
            msg,
            TIMEOUT_MS,
        )?;
        Ok(())
    }
}

impl MeasureModel<(SndDice, FwNode)> for Mbox3Model {
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

    fn measure_elem(
        &mut self,
        _: &(SndDice, FwNode),
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.common_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.tcd22xx_ctls.read(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

fn standalone_use_case_to_str(case: &StandaloneUseCase) -> &'static str {
    match case {
        StandaloneUseCase::Mixer => "Mixer",
        StandaloneUseCase::AdDa => "AD/DA",
        StandaloneUseCase::Preamp => "Preamp",
        StandaloneUseCase::Undefined => "Undefined",
    }
}

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

#[derive(Default, Debug)]
struct SpecificCtl(Mbox3SpecificParams, Vec<ElemId>);

impl SpecificCtl {
    const USE_CASE_NAME: &'static str = "standalone-usecase";

    const MASTER_KNOB_VALUE_NAME: &'static str = "master-knob-value";
    const MASTER_KNOB_ASSIGN_NAME: &'static str = "master-knob-assign";
    const DIM_LED_USAGE_NAME: &'static str = "dim-led";
    const HOLD_DURATION_NAME: &'static str = "hold-duration";
    const PHANTOM_POWERING_NAME: &'static str = "phantom-powering";
    const INPUT_HPF_NAME: &'static str = "input-hp-filter";
    const OUTPUT_TRIM_NAME: &'static str = "output-trim";

    const REVERB_TYPE_NAME: &'static str = "reverb-type";
    const REVERB_VOL_NAME: &'static str = "reverb-output-volume";
    const REVERB_DURATION_NAME: &'static str = "reverb-duration";
    const REVERB_FEEDBACK_NAME: &'static str = "reverb-feedback";

    const MUTE_BUTTON_NAME: &'static str = "mute-button";
    const MONO_BUTTON_NAME: &'static str = "mono-button";
    const SPKR_BUTTON_NAME: &'static str = "spkr-button";

    const USE_CASES: [StandaloneUseCase; 4] = [
        StandaloneUseCase::Mixer,
        StandaloneUseCase::AdDa,
        StandaloneUseCase::Preamp,
        StandaloneUseCase::Undefined,
    ];

    const MASTER_KNOB_VALUE_MIN: i32 = 0x00;
    const MASTER_KNOB_VALUE_MAX: i32 = 0xff;
    const MASTER_KNOB_VALUE_STEP: i32 = 1;

    const HOLD_DURATION_MAX: i32 = 1000;
    const HOLD_DURATION_MIN: i32 = 0;
    const HOLD_DURATION_STEP: i32 = 1;

    const INPUT_COUNT: usize = 4;
    const OUTPUT_COUNT: usize = 6;

    const REVERB_TYPES: [ReverbType; 8] = [
        ReverbType::Room1,
        ReverbType::Room2,
        ReverbType::Room3,
        ReverbType::Hall1,
        ReverbType::Hall2,
        ReverbType::Plate,
        ReverbType::Delay,
        ReverbType::Echo,
    ];

    const MUTE_LED_STATES: [MuteLedState; 3] =
        [MuteLedState::Off, MuteLedState::Blink, MuteLedState::On];

    const MONO_LED_STATES: [MonoLedState; 2] = [MonoLedState::Off, MonoLedState::On];

    const SPKR_LED_STATES: [SpkrLedState; 7] = [
        SpkrLedState::Off,
        SpkrLedState::Green,
        SpkrLedState::GreenBlink,
        SpkrLedState::Red,
        SpkrLedState::RedBlink,
        SpkrLedState::Orange,
        SpkrLedState::OrangeBlink,
    ];

    fn cache(
        &mut self,
        req: &FwReq,
        node: &FwNode,
        sections: &ExtensionSections,
        caps: &ExtensionCaps,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let res = Mbox3Protocol::cache_extension_whole_params(
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
        let labels: Vec<&str> = Self::USE_CASES
            .iter()
            .map(|c| standalone_use_case_to_str(c))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::USE_CASE_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let elem_id =
            ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::MASTER_KNOB_VALUE_NAME, 0);
        let _ = card_cntr
            .add_int_elems(
                &elem_id,
                1,
                Self::MASTER_KNOB_VALUE_MIN,
                Self::MASTER_KNOB_VALUE_MAX,
                Self::MASTER_KNOB_VALUE_STEP,
                1,
                None,
                false,
            )
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        let elem_id =
            ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::MASTER_KNOB_ASSIGN_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, Self::OUTPUT_COUNT, true);

        let labels: Vec<&str> = Self::MUTE_LED_STATES
            .iter()
            .map(|s| mute_led_state_to_str(s))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::MUTE_BUTTON_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, true)
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        let labels: Vec<&str> = Self::MONO_LED_STATES
            .iter()
            .map(|s| mono_led_state_to_str(s))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::MONO_BUTTON_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, true)
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        let labels: Vec<&str> = Self::SPKR_LED_STATES
            .iter()
            .map(|s| spkr_led_state_to_str(s))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::SPKR_BUTTON_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, true)
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::DIM_LED_USAGE_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, 1, 1, true)
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::HOLD_DURATION_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            1,
            Self::HOLD_DURATION_MIN,
            Self::HOLD_DURATION_MAX,
            Self::HOLD_DURATION_STEP,
            1,
            None,
            true,
        )?;

        let elem_id =
            ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::PHANTOM_POWERING_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, 1, 1, true)
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::INPUT_HPF_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, Self::INPUT_COUNT, true);

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::OUTPUT_TRIM_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                u8::MIN as i32,
                u8::MAX as i32,
                1,
                Self::OUTPUT_COUNT,
                None,
                true,
            )
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        let labels: Vec<&str> = Self::REVERB_TYPES
            .iter()
            .map(|t| reverb_type_to_str(t))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::REVERB_TYPE_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::REVERB_VOL_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            1,
            u8::MIN as i32,
            u8::MAX as i32,
            1,
            1,
            None,
            true,
        )?;

        let elem_id =
            ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::REVERB_DURATION_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            1,
            u8::MIN as i32,
            u8::MAX as i32,
            1,
            1,
            None,
            true,
        )?;

        let elem_id =
            ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::REVERB_FEEDBACK_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            1,
            u8::MIN as i32,
            u8::MAX as i32,
            1,
            1,
            None,
            true,
        )?;

        Ok(())
    }

    fn read(&self, elem_id: &ElemId, elem_value: &ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            Self::USE_CASE_NAME => {
                let params = &self.0;
                let pos = Self::USE_CASES
                    .iter()
                    .position(|c| params.standalone_use_case.eq(c))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            Self::MASTER_KNOB_VALUE_NAME => {
                let params = &self.0;
                elem_value.set_int(&[params.master_knob_value as i32]);
                Ok(true)
            }
            Self::MASTER_KNOB_ASSIGN_NAME => {
                let params = &self.0;
                elem_value.set_bool(&params.master_knob_assigns);
                Ok(true)
            }
            Self::DIM_LED_USAGE_NAME => {
                let params = &self.0;
                elem_value.set_bool(&[params.dim_led]);
                Ok(true)
            }
            Self::HOLD_DURATION_NAME => {
                let params = &self.0;
                elem_value.set_int(&[params.duration_hold as i32]);
                Ok(true)
            }
            Self::PHANTOM_POWERING_NAME => {
                let params = &self.0;
                elem_value.set_bool(&[params.phantom_powering]);
                Ok(true)
            }
            Self::INPUT_HPF_NAME => {
                let params = &self.0;
                elem_value.set_bool(&params.hpf_enables);
                Ok(true)
            }
            Self::OUTPUT_TRIM_NAME => {
                let params = &self.0;
                let vals: Vec<i32> = params.output_trims.iter().map(|&val| val as i32).collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            Self::REVERB_TYPE_NAME => {
                let params = &self.0;
                let pos = Self::REVERB_TYPES
                    .iter()
                    .position(|t| params.reverb_type.eq(t))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            Self::REVERB_VOL_NAME => {
                let params = &self.0;
                elem_value.set_int(&[params.reverb_volume as i32]);
                Ok(true)
            }
            Self::REVERB_DURATION_NAME => {
                let params = &self.0;
                elem_value.set_int(&[params.reverb_duration as i32]);
                Ok(true)
            }
            Self::REVERB_FEEDBACK_NAME => {
                let params = &self.0;
                elem_value.set_int(&[params.reverb_feedback as i32]);
                Ok(true)
            }
            Self::MUTE_BUTTON_NAME => {
                let params = &self.0;
                let pos = Self::MUTE_LED_STATES
                    .iter()
                    .position(|s| params.mute_led.eq(s))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            Self::MONO_BUTTON_NAME => {
                let params = &self.0;
                let pos = Self::MONO_LED_STATES
                    .iter()
                    .position(|s| params.mono_led.eq(s))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            Self::SPKR_BUTTON_NAME => {
                let params = &self.0;
                let pos = Self::SPKR_LED_STATES
                    .iter()
                    .position(|s| params.spkr_led.eq(s))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
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
            Self::USE_CASE_NAME => {
                let mut params = self.0.clone();
                let pos = elem_value.enumerated()[0] as usize;
                Self::USE_CASES
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| {
                        let msg = format!("Invalid value for standalone usecase: {}", pos);
                        Error::new(FileError::Inval, &msg)
                    })
                    .map(|&case| params.standalone_use_case = case)?;
                let res = Mbox3Protocol::update_extension_partial_params(
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
            Self::MASTER_KNOB_ASSIGN_NAME => {
                let mut params = self.0.clone();
                params
                    .master_knob_assigns
                    .iter_mut()
                    .zip(elem_value.boolean())
                    .for_each(|(assign, val)| *assign = val);
                let res = Mbox3Protocol::update_extension_partial_params(
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
            Self::MUTE_BUTTON_NAME => {
                let mut params = self.0.clone();
                let pos = elem_value.enumerated()[0] as usize;
                Self::MUTE_LED_STATES
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| {
                        let msg = format!("Mute LED state not found for position {}", pos);
                        Error::new(FileError::Inval, &msg)
                    })
                    .map(|&s| params.mute_led = s)?;
                let res = Mbox3Protocol::update_extension_partial_params(
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
            Self::MONO_BUTTON_NAME => {
                let mut params = self.0.clone();
                let pos = elem_value.enumerated()[0] as usize;
                Self::MONO_LED_STATES
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| {
                        let msg = format!("Mono LED state not found for position {}", pos);
                        Error::new(FileError::Inval, &msg)
                    })
                    .map(|&s| params.mono_led = s)?;
                let res = Mbox3Protocol::update_extension_partial_params(
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
            Self::SPKR_BUTTON_NAME => {
                let mut params = self.0.clone();
                let pos = elem_value.enumerated()[0] as usize;
                Self::SPKR_LED_STATES
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| {
                        let msg = format!("Speaker LED state not found for position {}", pos);
                        Error::new(FileError::Inval, &msg)
                    })
                    .map(|&s| params.spkr_led = s)?;
                let res = Mbox3Protocol::update_extension_partial_params(
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
            Self::DIM_LED_USAGE_NAME => {
                let mut params = self.0.clone();
                params.dim_led = elem_value.boolean()[0];
                let res = Mbox3Protocol::update_extension_partial_params(
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
            Self::HOLD_DURATION_NAME => {
                let mut params = self.0.clone();
                params.duration_hold = elem_value.int()[0] as u8;
                let res = Mbox3Protocol::update_extension_partial_params(
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
            Self::PHANTOM_POWERING_NAME => {
                let mut params = self.0.clone();
                params.phantom_powering = elem_value.boolean()[0];
                let res = Mbox3Protocol::update_extension_partial_params(
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
            Self::INPUT_HPF_NAME => {
                let mut params = self.0.clone();
                params
                    .hpf_enables
                    .iter_mut()
                    .zip(elem_value.boolean())
                    .for_each(|(enabled, val)| *enabled = val);
                let res = Mbox3Protocol::update_extension_partial_params(
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
            Self::OUTPUT_TRIM_NAME => {
                let mut params = self.0.clone();
                params
                    .output_trims
                    .iter_mut()
                    .zip(elem_value.int())
                    .for_each(|(trim, &val)| *trim = val as u8);
                let res = Mbox3Protocol::update_extension_partial_params(
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
            Self::REVERB_TYPE_NAME => {
                let mut params = self.0.clone();
                let pos = elem_value.enumerated()[0] as usize;
                Self::REVERB_TYPES
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| {
                        let msg = format!("Reverb type not found for position {}", pos);
                        Error::new(FileError::Inval, &msg)
                    })
                    .map(|&t| params.reverb_type = t)?;
                let res = Mbox3Protocol::update_extension_partial_params(
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
            Self::REVERB_VOL_NAME => {
                let mut params = self.0.clone();
                params.reverb_volume = elem_value.int()[0] as u8;
                let res = Mbox3Protocol::update_extension_partial_params(
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
            Self::REVERB_DURATION_NAME => {
                let mut params = self.0.clone();
                params.reverb_duration = elem_value.int()[0] as u8;
                let res = Mbox3Protocol::update_extension_partial_params(
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
            Self::REVERB_FEEDBACK_NAME => {
                let mut params = self.0.clone();
                params.reverb_feedback = elem_value.int()[0] as u8;
                let res = Mbox3Protocol::update_extension_partial_params(
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

    fn parse_notification(
        &mut self,
        req: &FwReq,
        node: &FwNode,
        sections: &ExtensionSections,
        caps: &ExtensionCaps,
        msg: u32,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let res = Mbox3Protocol::cache_extension_notified_params(
            req,
            node,
            sections,
            caps,
            &mut self.0,
            msg,
            timeout_ms,
        );
        debug!(params = ?self.0, ?res);
        res
    }
}
