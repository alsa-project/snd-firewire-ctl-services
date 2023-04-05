// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

pub(crate) use {
    super::*,
    alsactl::*,
    core::{card_cntr::*, elem_value_accessor::*},
    hinawa::FwReq,
};

const PHONE_ASSIGN_NAME: &str = "phone-assign";

pub trait PhoneAssignCtlOperation<T: AssignOperation> {
    fn state(&self) -> &usize;
    fn state_mut(&mut self) -> &mut usize;

    fn load(
        &mut self,
        card_cntr: &mut CardCntr,
        unit: &mut (SndMotu, FwNode),
        req: &mut FwReq,
        timeout_ms: u32,
    ) -> Result<Vec<ElemId>, Error> {
        self.cache(unit, req, timeout_ms)?;

        let labels: Vec<String> = T::ASSIGN_PORTS
            .iter()
            .map(|e| target_port_to_string(&e.0))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, PHONE_ASSIGN_NAME, 0);
        card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)
    }

    fn cache(
        &mut self,
        unit: &mut (SndMotu, FwNode),
        req: &mut FwReq,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        T::get_phone_assign(req, &mut unit.1, timeout_ms).map(|val| *self.state_mut() = val)
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            PHONE_ASSIGN_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || Ok(*self.state() as u32))
                    .map(|_| true)
            }
            _ => Ok(false),
        }
    }

    fn write(
        &mut self,
        unit: &mut (SndMotu, FwNode),
        req: &mut FwReq,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            PHONE_ASSIGN_NAME => ElemValueAccessor::<u32>::get_val(elem_value, |val| {
                T::set_phone_assign(req, &mut unit.1, val as usize, timeout_ms)
                    .map(|_| *self.state_mut() = val as usize)
            })
            .map(|_| true),
            _ => Ok(false),
        }
    }
}

#[derive(Default, Debug)]
pub(crate) struct WordClockCtl<T: WordClkOperation> {
    pub elem_id_list: Vec<ElemId>,
    speed_mode: WordClkSpeedMode,
    _phantom: PhantomData<T>,
}

fn word_clk_speed_mode_to_str(mode: &WordClkSpeedMode) -> &'static str {
    match mode {
        WordClkSpeedMode::ForceLowRate => "Force 44.1/48.0 kHz",
        WordClkSpeedMode::FollowSystemClk => "Follow to system clock",
    }
}

const WORD_OUT_MODE_NAME: &str = "word-out-mode";

const WORD_OUT_MODES: [WordClkSpeedMode; 2] = [
    WordClkSpeedMode::ForceLowRate,
    WordClkSpeedMode::FollowSystemClk,
];

impl<T: WordClkOperation> WordClockCtl<T> {
    pub(crate) fn cache(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        T::get_word_out(req, node, timeout_ms).map(|mode| self.speed_mode = mode)
    }

    pub(crate) fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let labels: Vec<&str> = WORD_OUT_MODES
            .iter()
            .map(|m| word_clk_speed_mode_to_str(m))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, WORD_OUT_MODE_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, true)
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))
    }

    pub(crate) fn read(
        &mut self,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            WORD_OUT_MODE_NAME => {
                let pos = WORD_OUT_MODES
                    .iter()
                    .position(|m| self.speed_mode.eq(m))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub(crate) fn write(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            WORD_OUT_MODE_NAME => {
                let pos = elem_value.enumerated()[0] as usize;
                let &mode = WORD_OUT_MODES.iter().nth(pos).ok_or_else(|| {
                    let msg = format!("Invalid argument for index of word clock speed: {}", pos);
                    Error::new(FileError::Inval, &msg)
                })?;
                T::set_word_out(req, node, mode, timeout_ms).map(|_| self.speed_mode = mode)?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

#[derive(Default, Debug)]
pub(crate) struct AesebuRateConvertCtl<T: AesebuRateConvertOperation> {
    pub elem_id_list: Vec<ElemId>,
    mode: usize,
    _phantom: PhantomData<T>,
}

fn aesebu_rate_convert_mode_to_str(mode: &AesebuRateConvertMode) -> &'static str {
    match mode {
        AesebuRateConvertMode::None => "None",
        AesebuRateConvertMode::InputToSystem => "input-is-converted",
        AesebuRateConvertMode::OutputDependsInput => "output-depends-on-input",
        AesebuRateConvertMode::OutputDoubleSystem => "output-is-double",
    }
}

const AESEBU_RATE_CONVERT_MODE_NAME: &str = "AES/EBU-rate-convert";

impl<T: AesebuRateConvertOperation> AesebuRateConvertCtl<T> {
    pub(crate) fn cache(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        T::get_aesebu_rate_convert_mode(req, node, timeout_ms).map(|mode| self.mode = mode)
    }

    pub(crate) fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let labels: Vec<&str> = T::AESEBU_RATE_CONVERT_MODES
            .iter()
            .map(|l| aesebu_rate_convert_mode_to_str(l))
            .collect();
        let elem_id =
            ElemId::new_by_name(ElemIfaceType::Card, 0, 0, AESEBU_RATE_CONVERT_MODE_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, true)
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))
    }

    pub(crate) fn read(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            AESEBU_RATE_CONVERT_MODE_NAME => {
                elem_value.set_enum(&[self.mode as u32]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub(crate) fn write(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            AESEBU_RATE_CONVERT_MODE_NAME => {
                let val = elem_value.enumerated()[0] as usize;
                T::set_aesebu_rate_convert_mode(req, node, val as usize, timeout_ms)
                    .map(|_| self.mode = val)?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

#[derive(Default, Debug)]
pub(crate) struct LevelMetersCtl<T: LevelMetersOperation> {
    pub elem_id_list: Vec<ElemId>,
    peak_hold_time: usize,
    clip_hold_time: usize,
    aesebu_mode: usize,
    programmable_mode: usize,
    _phantom: PhantomData<T>,
}

fn level_meters_hold_time_mode_to_string(mode: &LevelMetersHoldTimeMode) -> &'static str {
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
}

fn level_meters_aesebu_mode_to_string(mode: &LevelMetersAesebuMode) -> &'static str {
    match mode {
        LevelMetersAesebuMode::Output => "output",
        LevelMetersAesebuMode::Input => "input",
    }
}

fn level_meters_programmable_mode_to_string(mode: &LevelMetersProgrammableMode) -> &'static str {
    match mode {
        LevelMetersProgrammableMode::AnalogOutput => "analog-output",
        LevelMetersProgrammableMode::AdatInput => "ADAT-input",
        LevelMetersProgrammableMode::AdatOutput => "ADAT-output",
    }
}

const PEAK_HOLD_TIME_MODE_NAME: &str = "meter-peak-hold-time";
const CLIP_HOLD_TIME_MODE_NAME: &str = "meter-clip-hold-time";
const AESEBU_MODE_NAME: &str = "AES/EBU-meter";
const PROGRAMMABLE_MODE_NAME: &str = "programmable-meter";

impl<T: LevelMetersOperation> LevelMetersCtl<T> {
    pub(crate) fn cache(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        T::get_level_meters_peak_hold_time_mode(req, node, timeout_ms)
            .map(|val| self.peak_hold_time = val)?;

        T::get_level_meters_clip_hold_time_mode(req, node, timeout_ms)
            .map(|val| self.clip_hold_time = val)?;

        T::get_level_meters_aesebu_mode(req, node, timeout_ms).map(|idx| {
            self.aesebu_mode = idx;
        })?;

        T::get_level_meters_programmable_mode(req, node, timeout_ms).map(|idx| {
            self.programmable_mode = idx;
        })?;

        Ok(())
    }

    pub(crate) fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let labels: Vec<&str> = T::LEVEL_METERS_HOLD_TIME_MODES
            .iter()
            .map(|l| level_meters_hold_time_mode_to_string(&l))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, PEAK_HOLD_TIME_MODE_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, true)
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, CLIP_HOLD_TIME_MODE_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, true)
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let labels: Vec<&str> = T::LEVEL_METERS_AESEBU_MODES
            .iter()
            .map(|l| level_meters_aesebu_mode_to_string(&l))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, AESEBU_MODE_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, true)
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let labels: Vec<&str> = T::LEVEL_METERS_PROGRAMMABLE_MODES
            .iter()
            .map(|l| level_meters_programmable_mode_to_string(&l))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, PROGRAMMABLE_MODE_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, true)
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        Ok(())
    }

    pub(crate) fn read(
        &mut self,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            PEAK_HOLD_TIME_MODE_NAME => {
                elem_value.set_enum(&[self.peak_hold_time as u32]);
                Ok(true)
            }
            CLIP_HOLD_TIME_MODE_NAME => {
                elem_value.set_enum(&[self.clip_hold_time as u32]);
                Ok(true)
            }
            AESEBU_MODE_NAME => {
                elem_value.set_enum(&[self.aesebu_mode as u32]);
                Ok(true)
            }
            PROGRAMMABLE_MODE_NAME => {
                elem_value.set_enum(&[self.programmable_mode as u32]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub(crate) fn write(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            PEAK_HOLD_TIME_MODE_NAME => {
                let val = elem_value.enumerated()[0] as usize;
                T::set_level_meters_peak_hold_time_mode(req, node, val, timeout_ms)
                    .map(|_| self.peak_hold_time = val)?;
                Ok(true)
            }
            CLIP_HOLD_TIME_MODE_NAME => {
                let val = elem_value.enumerated()[0] as usize;
                T::set_level_meters_clip_hold_time_mode(req, node, val, timeout_ms)
                    .map(|_| self.clip_hold_time = val)?;
                Ok(true)
            }
            AESEBU_MODE_NAME => {
                let val = elem_value.enumerated()[0] as usize;
                T::set_level_meters_aesebu_mode(req, node, val, timeout_ms)
                    .map(|_| self.aesebu_mode = val)?;
                Ok(true)
            }
            PROGRAMMABLE_MODE_NAME => {
                let val = elem_value.enumerated()[0] as usize;
                T::set_level_meters_programmable_mode(req, node, val, timeout_ms)
                    .map(|_| self.programmable_mode = val)?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
