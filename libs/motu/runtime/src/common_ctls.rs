// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

use glib::{Error, FileError};

use hinawa::FwReq;
use hinawa::{SndMotu, SndUnitExt};

use alsactl::{ElemId, ElemIfaceType, ElemValue};

use core::card_cntr::CardCntr;
use core::elem_value_accessor::ElemValueAccessor;

use motu_protocols::*;

#[derive(Default)]
pub struct CommonPhoneCtl(pub Vec<ElemId>);

impl CommonPhoneCtl {
    const PHONE_ASSIGN_NAME: &'static str = "phone-assign";

    pub fn load<O>(&mut self, _: &O, card_cntr: &mut CardCntr) -> Result<(), Error>
    where
        O: AssignProtocol,
    {
        let labels: Vec<String> = O::ASSIGN_PORTS.iter().map(|e| e.0.to_string()).collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::PHONE_ASSIGN_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, true)
            .map(|elem_id_list| self.0.extend_from_slice(&elem_id_list))?;

        Ok(())
    }

    pub fn read<O>(
        &mut self,
        unit: &mut SndMotu,
        req: &mut FwReq,
        proto: &O,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error>
    where
        O: AssignProtocol,
    {
        match elem_id.get_name().as_str() {
            Self::PHONE_ASSIGN_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    proto
                        .get_phone_assign(req, &mut unit.get_node(), timeout_ms)
                        .map(|val| val as u32)
                })?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub fn write<O>(
        &mut self,
        unit: &mut SndMotu,
        req: &mut FwReq,
        proto: &O,
        elem_id: &ElemId,
        _: &ElemValue,
        new: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error>
    where
        O: AssignProtocol,
    {
        match elem_id.get_name().as_str() {
            Self::PHONE_ASSIGN_NAME => {
                ElemValueAccessor::<u32>::get_val(new, |val| {
                    proto.set_phone_assign(req, &mut unit.get_node(), val as usize, timeout_ms)
                })?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

fn word_clk_speed_mode_to_label(mode: &WordClkSpeedMode) -> String {
    match mode {
        WordClkSpeedMode::ForceLowRate => "Force 44.1/48.0 kHz",
        WordClkSpeedMode::FollowSystemClk => "Follow to system clock",
    }
    .to_string()
}

#[derive(Default)]
pub struct CommonWordClkCtl(pub Vec<ElemId>);

impl CommonWordClkCtl {
    const WORD_OUT_MODE_NAME: &'static str = "word-out-mode";

    const WORD_OUT_MODES: [WordClkSpeedMode; 2] = [
        WordClkSpeedMode::ForceLowRate,
        WordClkSpeedMode::FollowSystemClk,
    ];

    pub fn load<O>(&mut self, _: &O, card_cntr: &mut CardCntr) -> Result<(), Error>
    where
        O: WordClkProtocol,
    {
        let labels: Vec<String> = Self::WORD_OUT_MODES
            .iter()
            .map(|m| word_clk_speed_mode_to_label(m))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::WORD_OUT_MODE_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, true)
            .map(|elem_id_list| self.0.extend_from_slice(&elem_id_list))?;

        Ok(())
    }

    pub fn read<O>(
        &mut self,
        unit: &mut SndMotu,
        req: &mut FwReq,
        proto: &O,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error>
    where
        O: WordClkProtocol,
    {
        match elem_id.get_name().as_str() {
            Self::WORD_OUT_MODE_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    proto.get_word_out(req, &mut unit.get_node(), timeout_ms).map(|mode| {
                        Self::WORD_OUT_MODES
                            .iter()
                            .position(|&m| m == mode)
                            .map(|pos| pos as u32)
                            .unwrap()
                    })
                })?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub fn write<O>(
        &mut self,
        unit: &mut SndMotu,
        req: &mut FwReq,
        proto: &O,
        elem_id: &ElemId,
        _: &ElemValue,
        new: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error>
    where
        O: WordClkProtocol,
    {
        match elem_id.get_name().as_str() {
            Self::WORD_OUT_MODE_NAME => {
                ElemValueAccessor::<u32>::get_val(new, |val| {
                    let idx = val as usize;
                    if idx < Self::WORD_OUT_MODES.len() {
                        let mode = Self::WORD_OUT_MODES[idx];
                        proto.set_word_out(req, &mut unit.get_node(), mode, timeout_ms)
                    } else {
                        let msg =
                            format!("Invalid argument for index of word clock speed: {}", idx);
                        Err(Error::new(FileError::Inval, &msg))
                    }
                })?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

fn aesebu_rate_convert_mode_to_string(mode: &AesebuRateConvertMode) -> String {
    match mode {
        AesebuRateConvertMode::None => "None",
        AesebuRateConvertMode::InputToSystem => "input-is-converted",
        AesebuRateConvertMode::OutputDependsInput => "output-depends-on-input",
        AesebuRateConvertMode::OutputDoubleSystem => "output-is-double",
    }
    .to_string()
}

#[derive(Default)]
pub struct CommonAesebuRateConvertCtl;

impl CommonAesebuRateConvertCtl {
    const AESEBU_RATE_CONVERT_MODE_NAME: &'static str = "AES/EBU-rate-convert";

    pub fn load<O>(&mut self, _: &O, card_cntr: &mut CardCntr) -> Result<(), Error>
    where
        O: AesebuRateConvertProtocol,
    {
        let labels: Vec<String> = O::AESEBU_RATE_CONVERT_MODES
            .iter()
            .map(|l| aesebu_rate_convert_mode_to_string(&l))
            .collect();
        let elem_id = ElemId::new_by_name(
            ElemIfaceType::Card,
            0,
            0,
            Self::AESEBU_RATE_CONVERT_MODE_NAME,
            0,
        );
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        Ok(())
    }

    pub fn read<O>(
        &mut self,
        unit: &mut SndMotu,
        req: &mut FwReq,
        proto: &O,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error>
    where
        O: AesebuRateConvertProtocol,
    {
        match elem_id.get_name().as_str() {
            Self::AESEBU_RATE_CONVERT_MODE_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    proto
                        .get_aesebu_rate_convert_mode(req, &mut unit.get_node(), timeout_ms)
                        .map(|val| val as u32)
                })?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub fn write<O>(
        &mut self,
        unit: &mut SndMotu,
        req: &mut FwReq,
        proto: &O,
        elem_id: &ElemId,
        _: &ElemValue,
        new: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error>
    where
        O: AesebuRateConvertProtocol,
    {
        match elem_id.get_name().as_str() {
            Self::AESEBU_RATE_CONVERT_MODE_NAME => {
                ElemValueAccessor::<u32>::get_val(new, |val| {
                    proto.set_aesebu_rate_convert_mode(req, &mut unit.get_node(), val as usize, timeout_ms)
                })?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

fn level_meters_hold_time_mode_to_string(mode: &LevelMetersHoldTimeMode) -> String {
    match mode {
        LevelMetersHoldTimeMode::Off => "off",
        LevelMetersHoldTimeMode::Sec2 => "2sec",
        LevelMetersHoldTimeMode::Sec4 => "4sec",
        LevelMetersHoldTimeMode::Sec10 => "10sec",
        LevelMetersHoldTimeMode::Sec60 => "1min",
        LevelMetersHoldTimeMode::Sec300 => "5min",
        LevelMetersHoldTimeMode::Sec480 => "8min",
        LevelMetersHoldTimeMode::Infinite => "infinite",
    }
    .to_string()
}

fn level_meters_aesebu_mode_to_string(mode: &LevelMetersAesebuMode) -> String {
    match mode {
        LevelMetersAesebuMode::Output => "output",
        LevelMetersAesebuMode::Input => "input",
    }
    .to_string()
}

fn level_meters_programmable_mode_to_string(mode: &LevelMetersProgrammableMode) -> String {
    match mode {
        LevelMetersProgrammableMode::AnalogOutput => "analog-output",
        LevelMetersProgrammableMode::AdatInput => "ADAT-input",
        LevelMetersProgrammableMode::AdatOutput => "ADAT-output",
    }
    .to_string()
}

#[derive(Default)]
pub struct CommonLevelMetersCtl;

impl CommonLevelMetersCtl {
    const PEAK_HOLD_TIME_MODE_NAME: &'static str = "meter-peak-hold-time";
    const CLIP_HOLD_TIME_MODE_NAME: &'static str = "meter-clip-hold-time";
    const AESEBU_MODE_NAME: &'static str = "AES/EBU-meter";
    const PROGRAMMABLE_MODE_NAME: &'static str = "programmable-meter";

    pub fn load<O>(&mut self, _: &O, card_cntr: &mut CardCntr) -> Result<(), Error>
    where
        O: LevelMetersProtocol,
    {
        let labels: Vec<String> = O::LEVEL_METERS_HOLD_TIME_MODES
            .iter()
            .map(|l| level_meters_hold_time_mode_to_string(&l))
            .collect();
        let elem_id =
            ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::PEAK_HOLD_TIME_MODE_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let elem_id =
            ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::CLIP_HOLD_TIME_MODE_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let labels: Vec<String> = O::LEVEL_METERS_AESEBU_MODES
            .iter()
            .map(|l| level_meters_aesebu_mode_to_string(&l))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::AESEBU_MODE_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let labels: Vec<String> = O::LEVEL_METERS_PROGRAMMABLE_MODES
            .iter()
            .map(|l| level_meters_programmable_mode_to_string(&l))
            .collect();
        let elem_id =
            ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::PROGRAMMABLE_MODE_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        Ok(())
    }

    pub fn read<O>(
        &mut self,
        unit: &mut SndMotu,
        proto: &O,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error>
    where
        O: LevelMetersProtocol,
    {
        match elem_id.get_name().as_str() {
            Self::PEAK_HOLD_TIME_MODE_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    proto
                        .get_level_meters_peak_hold_time_mode(&mut unit.get_node(), timeout_ms)
                        .map(|val| val as u32)
                })?;
                Ok(true)
            }
            Self::CLIP_HOLD_TIME_MODE_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    proto
                        .get_level_meters_clip_hold_time_mode(&mut unit.get_node(), timeout_ms)
                        .map(|val| val as u32)
                })?;
                Ok(true)
            }
            Self::AESEBU_MODE_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    proto
                        .get_level_meters_aesebu_mode(&mut unit.get_node(), timeout_ms)
                        .map(|val| val as u32)
                })?;
                Ok(true)
            }
            Self::PROGRAMMABLE_MODE_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    proto
                        .get_level_meters_programmable_mode(&mut unit.get_node(), timeout_ms)
                        .map(|val| val as u32)
                })?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub fn write<O>(
        &mut self,
        unit: &mut SndMotu,
        proto: &O,
        elem_id: &ElemId,
        _: &ElemValue,
        new: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error>
    where
        O: LevelMetersProtocol,
    {
        match elem_id.get_name().as_str() {
            Self::PEAK_HOLD_TIME_MODE_NAME => {
                ElemValueAccessor::<u32>::get_val(new, |val| {
                    proto.set_level_meters_peak_hold_time_mode(&mut unit.get_node(), val as usize, timeout_ms)
                })?;
                Ok(true)
            }
            Self::CLIP_HOLD_TIME_MODE_NAME => {
                ElemValueAccessor::<u32>::get_val(new, |val| {
                    proto.set_level_meters_clip_hold_time_mode(&mut unit.get_node(), val as usize, timeout_ms)
                })?;
                Ok(true)
            }
            Self::AESEBU_MODE_NAME => {
                ElemValueAccessor::<u32>::get_val(new, |val| {
                    proto.set_level_meters_aesebu_mode(&mut unit.get_node(), val as usize, timeout_ms)
                })?;
                Ok(true)
            }
            Self::PROGRAMMABLE_MODE_NAME => {
                ElemValueAccessor::<u32>::get_val(new, |val| {
                    proto.set_level_meters_programmable_mode(&mut unit.get_node(), val as usize, timeout_ms)
                })?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
