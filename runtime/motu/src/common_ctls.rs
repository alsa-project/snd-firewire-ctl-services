// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

pub(crate) use {super::*, alsactl::*, core::card_cntr::*, hinawa::FwReq};

#[derive(Default, Debug)]
pub(crate) struct PhoneAssignCtl<T>
where
    T: MotuPortAssignSpecification
        + MotuWhollyCacheableParamsOperation<PhoneAssignParameters>
        + MotuWhollyUpdatableParamsOperation<PhoneAssignParameters>,
{
    pub elem_id_list: Vec<ElemId>,
    pub params: PhoneAssignParameters,
    _phantom: PhantomData<T>,
}

const PHONE_ASSIGN_NAME: &str = "phone-assign";

impl<T> PhoneAssignCtl<T>
where
    T: MotuPortAssignSpecification
        + MotuWhollyCacheableParamsOperation<PhoneAssignParameters>
        + MotuWhollyUpdatableParamsOperation<PhoneAssignParameters>,
{
    pub(crate) fn cache(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let res = T::cache_wholly(req, node, &mut self.params, timeout_ms);
        debug!(params = ?self.params, ?res);
        res
    }

    pub(crate) fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let labels: Vec<String> = T::ASSIGN_PORT_TARGETS
            .iter()
            .map(|p| target_port_to_string(p))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, PHONE_ASSIGN_NAME, 0);
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
            PHONE_ASSIGN_NAME => {
                let pos = T::ASSIGN_PORT_TARGETS
                    .iter()
                    .position(|p| self.params.0.eq(p))
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
            PHONE_ASSIGN_NAME => {
                let mut params = self.params.clone();
                let pos = elem_value.enumerated()[0] as usize;
                T::ASSIGN_PORT_TARGETS
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| {
                        let msg = format!("Invalid argument for phone assignment: {}", pos);
                        Error::new(FileError::Inval, &msg)
                    })
                    .map(|&p| params.0 = p)?;
                let res =
                    T::update_wholly(req, node, &params, timeout_ms).map(|_| self.params = params);
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

#[derive(Default, Debug)]
pub(crate) struct WordClockCtl<T>
where
    T: MotuWordClockOutputSpecification
        + MotuWhollyCacheableParamsOperation<WordClkSpeedMode>
        + MotuWhollyUpdatableParamsOperation<WordClkSpeedMode>,
{
    pub elem_id_list: Vec<ElemId>,
    params: WordClkSpeedMode,
    _phantom: PhantomData<T>,
}

fn word_clk_speed_mode_to_str(mode: &WordClkSpeedMode) -> &'static str {
    match mode {
        WordClkSpeedMode::ForceLowRate => "Force 44.1/48.0 kHz",
        WordClkSpeedMode::FollowSystemClk => "Follow to system clock",
    }
}

const WORD_OUT_MODE_NAME: &str = "word-out-mode";

impl<T> WordClockCtl<T>
where
    T: MotuWordClockOutputSpecification
        + MotuWhollyCacheableParamsOperation<WordClkSpeedMode>
        + MotuWhollyUpdatableParamsOperation<WordClkSpeedMode>,
{
    pub(crate) fn cache(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let res = T::cache_wholly(req, node, &mut self.params, timeout_ms);
        debug!(params = ?self.params, ?res);
        res
    }

    pub(crate) fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let labels: Vec<&str> = T::WORD_CLOCK_OUTPUT_SPEED_MODES
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
                let pos = T::WORD_CLOCK_OUTPUT_SPEED_MODES
                    .iter()
                    .position(|m| self.params.eq(m))
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
                let params = T::WORD_CLOCK_OUTPUT_SPEED_MODES
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| {
                        let msg =
                            format!("Invalid argument for index of word clock speed: {}", pos);
                        Error::new(FileError::Inval, &msg)
                    })
                    .copied()?;
                let res =
                    T::update_wholly(req, node, &params, timeout_ms).map(|_| self.params = params);
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

#[derive(Default, Debug)]
pub(crate) struct AesebuRateConvertCtl<T>
where
    T: MotuAesebuRateConvertSpecification
        + MotuWhollyCacheableParamsOperation<AesebuRateConvertMode>
        + MotuWhollyUpdatableParamsOperation<AesebuRateConvertMode>,
{
    pub elem_id_list: Vec<ElemId>,
    params: AesebuRateConvertMode,
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

impl<T> AesebuRateConvertCtl<T>
where
    T: MotuAesebuRateConvertSpecification
        + MotuWhollyCacheableParamsOperation<AesebuRateConvertMode>
        + MotuWhollyUpdatableParamsOperation<AesebuRateConvertMode>,
{
    pub(crate) fn cache(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let res = T::cache_wholly(req, node, &mut self.params, timeout_ms);
        debug!(params = ?self.params, ?res);
        res
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
                let pos = T::AESEBU_RATE_CONVERT_MODES
                    .iter()
                    .position(|m| self.params.eq(m))
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
            AESEBU_RATE_CONVERT_MODE_NAME => {
                let pos = elem_value.enumerated()[0] as usize;
                let params = T::AESEBU_RATE_CONVERT_MODES
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| {
                        let msg =
                            format!("Invalid argument for mode of AES/EBU rate convert: {}", pos);
                        Error::new(FileError::Inval, &msg)
                    })
                    .copied()?;
                let res =
                    T::update_wholly(req, node, &params, timeout_ms).map(|_| self.params = params);
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

#[derive(Default, Debug)]
pub(crate) struct LevelMetersCtl<T>
where
    T: MotuLevelMetersSpecification
        + MotuWhollyCacheableParamsOperation<LevelMetersParameters>
        + MotuWhollyUpdatableParamsOperation<LevelMetersParameters>,
{
    pub elem_id_list: Vec<ElemId>,
    params: LevelMetersParameters,
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

impl<T> LevelMetersCtl<T>
where
    T: MotuLevelMetersSpecification
        + MotuWhollyCacheableParamsOperation<LevelMetersParameters>
        + MotuWhollyUpdatableParamsOperation<LevelMetersParameters>,
{
    pub(crate) fn cache(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let res = T::cache_wholly(req, node, &mut self.params, timeout_ms);
        debug!(params = ?self.params, ?res);
        res
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
                let pos = T::LEVEL_METERS_HOLD_TIME_MODES
                    .iter()
                    .position(|m| self.params.peak_hold_time.eq(m))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            CLIP_HOLD_TIME_MODE_NAME => {
                let pos = T::LEVEL_METERS_HOLD_TIME_MODES
                    .iter()
                    .position(|m| self.params.clip_hold_time.eq(m))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            AESEBU_MODE_NAME => {
                let pos = T::LEVEL_METERS_AESEBU_MODES
                    .iter()
                    .position(|m| self.params.aesebu_mode.eq(m))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            PROGRAMMABLE_MODE_NAME => {
                let pos = T::LEVEL_METERS_PROGRAMMABLE_MODES
                    .iter()
                    .position(|m| self.params.programmable_mode.eq(m))
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
            PEAK_HOLD_TIME_MODE_NAME => {
                let mut params = self.params.clone();
                let pos = elem_value.enumerated()[0] as usize;
                T::LEVEL_METERS_HOLD_TIME_MODES
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| {
                        let msg = format!(
                            "Invalid argument for peak hold time of level meter: {}",
                            pos
                        );
                        Error::new(FileError::Inval, &msg)
                    })
                    .map(|&mode| params.peak_hold_time = mode)?;
                let res =
                    T::update_wholly(req, node, &params, timeout_ms).map(|_| self.params = params);
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            CLIP_HOLD_TIME_MODE_NAME => {
                let mut params = self.params.clone();
                let pos = elem_value.enumerated()[0] as usize;
                T::LEVEL_METERS_HOLD_TIME_MODES
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| {
                        let msg = format!(
                            "Invalid argument for clip hold time of level meter: {}",
                            pos
                        );
                        Error::new(FileError::Inval, &msg)
                    })
                    .map(|&mode| params.clip_hold_time = mode)?;
                let res =
                    T::update_wholly(req, node, &params, timeout_ms).map(|_| self.params = params);
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            AESEBU_MODE_NAME => {
                let mut params = self.params.clone();
                let pos = elem_value.enumerated()[0] as usize;
                T::LEVEL_METERS_AESEBU_MODES
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| {
                        let msg =
                            format!("Invalid argument for AES/EBU mode of level meter: {}", pos);
                        Error::new(FileError::Inval, &msg)
                    })
                    .map(|&mode| params.aesebu_mode = mode)?;
                let res =
                    T::update_wholly(req, node, &params, timeout_ms).map(|_| self.params = params);
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            PROGRAMMABLE_MODE_NAME => {
                let mut params = self.params.clone();
                let pos = elem_value.enumerated()[0] as usize;
                T::LEVEL_METERS_PROGRAMMABLE_MODES
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| {
                        let msg = format!(
                            "Invalid argument for programmable mode of level meter: {}",
                            pos
                        );
                        Error::new(FileError::Inval, &msg)
                    })
                    .map(|&mode| params.programmable_mode = mode)?;
                let res =
                    T::update_wholly(req, node, &params, timeout_ms).map(|_| self.params = params);
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            _ => Ok(false),
        }
    }
}
