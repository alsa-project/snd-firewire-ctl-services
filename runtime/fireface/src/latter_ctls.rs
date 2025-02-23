// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

use {super::*, alsa_ctl_tlv_codec::DbInterval, std::marker::PhantomData};

const LINE_INPUT_METER: &str = "meter:line-input";
const MIC_INPUT_METER: &str = "meter:mic-input";
const SPDIF_INPUT_METER: &str = "meter:spdif-input";
const ADAT_INPUT_METER: &str = "meter:adat-input";
const STREAM_INPUT_METER: &str = "meter:stream-input";
const LINE_OUTPUT_METER: &str = "meter:line-output";
const HP_OUTPUT_METER: &str = "meter:hp-output";
const SPDIF_OUTPUT_METER: &str = "meter:spdif-output";
const ADAT_OUTPUT_METER: &str = "meter:adat-output";

#[derive(Debug)]
pub struct LatterMeterCtl<T>(pub Vec<ElemId>, FfLatterMeterState, PhantomData<T>)
where
    T: RmeFfLatterMeterSpecification + RmeFfCacheableParamsOperation<FfLatterMeterState>;

impl<T: RmeFfLatterMeterSpecification> Default for LatterMeterCtl<T> {
    fn default() -> Self {
        Self(
            Default::default(),
            T::create_meter_state(),
            Default::default(),
        )
    }
}

impl<T> LatterMeterCtl<T>
where
    T: RmeFfLatterMeterSpecification + RmeFfCacheableParamsOperation<FfLatterMeterState>,
{
    const LEVEL_TLV: DbInterval = DbInterval {
        min: -9003,
        max: 600,
        linear: false,
        mute_avail: false,
    };

    pub fn cache(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let res = T::cache_wholly(req, node, &mut self.1, timeout_ms);
        debug!(params = ?self.1, ?res);
        res
    }

    pub fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        [
            (LINE_INPUT_METER, T::LINE_INPUT_COUNT),
            (MIC_INPUT_METER, T::MIC_INPUT_COUNT),
            (SPDIF_INPUT_METER, T::SPDIF_INPUT_COUNT),
            (ADAT_INPUT_METER, T::ADAT_INPUT_COUNT),
            (STREAM_INPUT_METER, T::STREAM_INPUT_COUNT),
            (LINE_OUTPUT_METER, T::LINE_OUTPUT_COUNT),
            (HP_OUTPUT_METER, T::HP_OUTPUT_COUNT),
            (SPDIF_OUTPUT_METER, T::SPDIF_OUTPUT_COUNT),
            (ADAT_OUTPUT_METER, T::ADAT_OUTPUT_COUNT),
        ]
        .iter()
        .try_for_each(|&(name, count)| {
            let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, name, 0);
            card_cntr
                .add_int_elems(
                    &elem_id,
                    1,
                    T::LEVEL_MIN,
                    T::LEVEL_MAX,
                    T::LEVEL_STEP,
                    count,
                    Some(&Vec::<u32>::from(&Self::LEVEL_TLV)),
                    false,
                )
                .map(|mut elem_id_list| self.0.append(&mut elem_id_list))
        })
    }

    pub fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            LINE_INPUT_METER => {
                elem_value.set_int(&self.1.line_inputs);
                Ok(true)
            }
            MIC_INPUT_METER => {
                elem_value.set_int(&self.1.mic_inputs);
                Ok(true)
            }
            SPDIF_INPUT_METER => {
                elem_value.set_int(&self.1.spdif_inputs);
                Ok(true)
            }
            ADAT_INPUT_METER => {
                elem_value.set_int(&self.1.adat_inputs);
                Ok(true)
            }
            STREAM_INPUT_METER => {
                elem_value.set_int(&self.1.stream_inputs);
                Ok(true)
            }
            LINE_OUTPUT_METER => {
                elem_value.set_int(&self.1.line_outputs);
                Ok(true)
            }
            HP_OUTPUT_METER => {
                elem_value.set_int(&self.1.hp_outputs);
                Ok(true)
            }
            SPDIF_OUTPUT_METER => {
                elem_value.set_int(&self.1.spdif_outputs);
                Ok(true)
            }
            ADAT_OUTPUT_METER => {
                elem_value.set_int(&self.1.adat_outputs);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

const INPUT_STEREO_LINK_NAME: &str = "input:stereo-link";
const INPUT_LINE_GAIN_NAME: &str = "input:line-gain";
const INPUT_LINE_LEVEL_NAME: &str = "input:line-level";
const INPUT_MIC_POWER_NAME: &str = "input:mic-power";
const INPUT_MIC_INST_NAME: &str = "input:mic-instrument";
const INPUT_INVERT_PHASE_NAME: &str = "input:invert-phase";

#[derive(Debug)]
pub struct LatterInputCtl<T>
where
    T: RmeFfLatterInputSpecification
        + RmeFfWhollyCommandableParamsOperation<FfLatterInputState>
        + RmeFfPartiallyCommandableParamsOperation<FfLatterInputState>,
{
    pub elem_id_list: Vec<ElemId>,
    params: FfLatterInputState,
    _phantom: PhantomData<T>,
}

impl<T> Default for LatterInputCtl<T>
where
    T: RmeFfLatterInputSpecification
        + RmeFfWhollyCommandableParamsOperation<FfLatterInputState>
        + RmeFfPartiallyCommandableParamsOperation<FfLatterInputState>,
{
    fn default() -> Self {
        Self {
            elem_id_list: Default::default(),
            params: T::create_input_parameters(),
            _phantom: Default::default(),
        }
    }
}

fn latter_line_in_nominal_level_to_str(level: &LatterInNominalLevel) -> &str {
    match level {
        LatterInNominalLevel::Low => "Low",
        LatterInNominalLevel::Professional => "+4dBu",
    }
}

impl<T> LatterInputCtl<T>
where
    T: RmeFfLatterInputSpecification
        + RmeFfWhollyCommandableParamsOperation<FfLatterInputState>
        + RmeFfPartiallyCommandableParamsOperation<FfLatterInputState>,
{
    const INPUT_GAIN_TLV: DbInterval = DbInterval {
        min: 0,
        max: 1200,
        linear: false,
        mute_avail: false,
    };

    const INPUT_LINE_LEVELS: &'static [LatterInNominalLevel] = &[
        LatterInNominalLevel::Low,
        LatterInNominalLevel::Professional,
    ];

    pub fn cache(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let res = T::command_wholly(req, node, &mut self.params, timeout_ms);
        debug!(params = ?self.params, ?res);
        res
    }

    pub fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, INPUT_STEREO_LINK_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, 1, T::PHYS_INPUT_COUNT / 2, true)
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, INPUT_LINE_GAIN_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                T::PHYS_INPUT_GAIN_MIN,
                T::PHYS_INPUT_GAIN_MAX,
                T::PHYS_INPUT_GAIN_STEP,
                T::PHYS_INPUT_COUNT,
                Some(&Vec::<u32>::from(&Self::INPUT_GAIN_TLV)),
                true,
            )
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let labels: Vec<&str> = Self::INPUT_LINE_LEVELS
            .iter()
            .map(|l| latter_line_in_nominal_level_to_str(l))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, INPUT_LINE_LEVEL_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, T::LINE_INPUT_COUNT, &labels, None, true)
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, INPUT_MIC_POWER_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, 1, T::MIC_INPUT_COUNT, true)
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, INPUT_MIC_INST_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, 1, T::MIC_INPUT_COUNT, true)
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, INPUT_INVERT_PHASE_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, 1, T::MIC_INPUT_COUNT, true)
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        Ok(())
    }

    pub fn read(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            INPUT_STEREO_LINK_NAME => {
                elem_value.set_bool(&self.params.stereo_links);
                Ok(true)
            }
            INPUT_LINE_GAIN_NAME => {
                let vals: Vec<i32> = self
                    .params
                    .line_gains
                    .iter()
                    .map(|&gain| gain as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            INPUT_LINE_LEVEL_NAME => {
                let vals: Vec<u32> = self
                    .params
                    .line_levels
                    .iter()
                    .map(|level| {
                        let pos = Self::INPUT_LINE_LEVELS
                            .iter()
                            .position(|l| l.eq(level))
                            .unwrap();
                        pos as u32
                    })
                    .collect();
                elem_value.set_enum(&vals);
                Ok(true)
            }
            INPUT_MIC_POWER_NAME => {
                elem_value.set_bool(&self.params.mic_powers);
                Ok(true)
            }
            INPUT_MIC_INST_NAME => {
                elem_value.set_bool(&self.params.mic_insts);
                Ok(true)
            }
            INPUT_INVERT_PHASE_NAME => {
                elem_value.set_bool(&self.params.invert_phases);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub fn write(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            INPUT_STEREO_LINK_NAME => {
                let mut params = self.params.clone();
                params
                    .stereo_links
                    .iter_mut()
                    .zip(elem_value.boolean())
                    .for_each(|(d, s)| *d = s);
                let res = T::command_partially(req, node, &mut self.params, params, timeout_ms);
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            INPUT_LINE_GAIN_NAME => {
                let mut params = self.params.clone();
                params
                    .line_gains
                    .iter_mut()
                    .zip(elem_value.int())
                    .for_each(|(d, s)| *d = *s as i16);
                let res = T::command_partially(req, node, &mut self.params, params, timeout_ms);
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            INPUT_LINE_LEVEL_NAME => {
                let mut params = self.params.clone();
                params
                    .line_levels
                    .iter_mut()
                    .zip(elem_value.enumerated())
                    .try_for_each(|(level, &val)| {
                        let pos = val as usize;
                        Self::INPUT_LINE_LEVELS
                            .iter()
                            .nth(pos)
                            .ok_or_else(|| {
                                let msg = format!("Invalid index of input nominal level: {}", pos);
                                Error::new(FileError::Inval, &msg)
                            })
                            .map(|&l| *level = l)
                    })?;
                let res = T::command_partially(req, node, &mut self.params, params, timeout_ms);
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            INPUT_MIC_POWER_NAME => {
                let mut params = self.params.clone();
                params
                    .mic_powers
                    .iter_mut()
                    .zip(elem_value.boolean())
                    .for_each(|(d, s)| *d = s);
                let res = T::command_partially(req, node, &mut self.params, params, timeout_ms);
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            INPUT_MIC_INST_NAME => {
                let mut params = self.params.clone();
                params
                    .mic_insts
                    .iter_mut()
                    .zip(elem_value.boolean())
                    .for_each(|(d, s)| *d = s);
                let res = T::command_partially(req, node, &mut self.params, params, timeout_ms);
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            INPUT_INVERT_PHASE_NAME => {
                let mut params = self.params.clone();
                params
                    .invert_phases
                    .iter_mut()
                    .zip(elem_value.boolean())
                    .for_each(|(d, s)| *d = s);
                let res = T::command_partially(req, node, &mut self.params, params, timeout_ms);
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

const VOL_NAME: &str = "output:volume";
const STEREO_BALANCE_NAME: &str = "output:stereo-balance";
const STEREO_LINK_NAME: &str = "output:stereo-link";
const INVERT_PHASE_NAME: &str = "output:invert-phase";
const LINE_LEVEL_NAME: &str = "output:line-level";

#[derive(Debug)]
pub struct LatterOutputCtl<T>
where
    T: RmeFfLatterOutputSpecification
        + RmeFfWhollyCommandableParamsOperation<FfLatterOutputState>
        + RmeFfPartiallyCommandableParamsOperation<FfLatterOutputState>,
{
    pub elem_id_list: Vec<ElemId>,
    params: FfLatterOutputState,
    _phantom: PhantomData<T>,
}

impl<T> Default for LatterOutputCtl<T>
where
    T: RmeFfLatterOutputSpecification
        + RmeFfWhollyCommandableParamsOperation<FfLatterOutputState>
        + RmeFfPartiallyCommandableParamsOperation<FfLatterOutputState>,
{
    fn default() -> Self {
        let mut params = T::create_output_parameters();
        params
            .vols
            .iter_mut()
            .for_each(|vol| *vol = T::PHYS_OUTPUT_VOL_ZERO as i16);

        Self {
            elem_id_list: Default::default(),
            params,
            _phantom: Default::default(),
        }
    }
}

impl<T> LatterOutputCtl<T>
where
    T: RmeFfLatterOutputSpecification
        + RmeFfWhollyCommandableParamsOperation<FfLatterOutputState>
        + RmeFfPartiallyCommandableParamsOperation<FfLatterOutputState>,
{
    const VOL_TLV: DbInterval = DbInterval {
        min: -6500,
        max: 600,
        linear: false,
        mute_avail: false,
    };

    const OUTPUT_LINE_LEVELS: [LineOutNominalLevel; 3] = [
        LineOutNominalLevel::Consumer,
        LineOutNominalLevel::Professional,
        LineOutNominalLevel::High,
    ];

    pub fn cache(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let res = T::command_wholly(req, node, &mut self.params, timeout_ms);
        debug!(params = ?self.params, ?res);
        res
    }

    pub fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, VOL_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                T::PHYS_OUTPUT_VOL_MIN,
                T::PHYS_OUTPUT_VOL_MAX,
                T::PHYS_OUTPUT_VOL_STEP,
                T::OUTPUT_COUNT,
                Some(&Vec::<u32>::from(&Self::VOL_TLV)),
                true,
            )
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, STEREO_BALANCE_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                T::PHYS_OUTPUT_BALANCE_MIN,
                T::PHYS_OUTPUT_BALANCE_MAX,
                T::PHYS_OUTPUT_BALANCE_STEP,
                T::OUTPUT_COUNT / 2,
                None,
                true,
            )
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, STEREO_LINK_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, 1, T::OUTPUT_COUNT / 2, true)
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, INVERT_PHASE_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, 1, T::OUTPUT_COUNT, true)
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let labels: Vec<&str> = Self::OUTPUT_LINE_LEVELS
            .iter()
            .map(|l| line_out_nominal_level_to_str(l))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, LINE_LEVEL_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, T::LINE_OUTPUT_COUNT, &labels, None, true)
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        Ok(())
    }

    pub fn read(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            VOL_NAME => {
                let vals: Vec<i32> = self.params.vols.iter().map(|&vol| vol as i32).collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            STEREO_BALANCE_NAME => {
                let vals: Vec<i32> = self
                    .params
                    .stereo_balance
                    .iter()
                    .map(|&balance| balance as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            STEREO_LINK_NAME => {
                elem_value.set_bool(&self.params.stereo_links);
                Ok(true)
            }
            INVERT_PHASE_NAME => {
                elem_value.set_bool(&self.params.invert_phases);
                Ok(true)
            }
            LINE_LEVEL_NAME => {
                let vals: Vec<u32> = self
                    .params
                    .line_levels
                    .iter()
                    .map(|level| {
                        let pos = Self::OUTPUT_LINE_LEVELS
                            .iter()
                            .position(|l| l.eq(level))
                            .unwrap();
                        pos as u32
                    })
                    .collect();
                elem_value.set_enum(&vals);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub fn write(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            VOL_NAME => {
                let mut params = self.params.clone();
                params
                    .vols
                    .iter_mut()
                    .zip(elem_value.int())
                    .for_each(|(d, s)| *d = *s as i16);
                let res = T::command_partially(req, node, &mut self.params, params, timeout_ms);
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            STEREO_BALANCE_NAME => {
                let mut params = self.params.clone();
                params
                    .stereo_balance
                    .iter_mut()
                    .zip(elem_value.int())
                    .for_each(|(d, s)| *d = *s as i16);
                let res = T::command_partially(req, node, &mut self.params, params, timeout_ms);
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            STEREO_LINK_NAME => {
                let mut params = self.params.clone();
                params
                    .stereo_links
                    .iter_mut()
                    .zip(elem_value.boolean())
                    .for_each(|(d, s)| *d = s);
                let res = T::command_partially(req, node, &mut self.params, params, timeout_ms);
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            INVERT_PHASE_NAME => {
                let mut params = self.params.clone();
                params
                    .invert_phases
                    .iter_mut()
                    .zip(elem_value.boolean())
                    .for_each(|(d, s)| *d = s);
                let res = T::command_partially(req, node, &mut self.params, params, timeout_ms);
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            LINE_LEVEL_NAME => {
                let mut params = self.params.clone();
                params
                    .line_levels
                    .iter_mut()
                    .zip(elem_value.enumerated())
                    .try_for_each(|(level, &val)| {
                        let pos = val as usize;
                        Self::OUTPUT_LINE_LEVELS
                            .iter()
                            .nth(pos)
                            .ok_or_else(|| {
                                let msg =
                                    format!("Invalid for index of output nominal level: {}", pos);
                                Error::new(FileError::Inval, &msg)
                            })
                            .map(|&l| *level = l)
                    })?;
                let res = T::command_partially(req, node, &mut self.params, params, timeout_ms);
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

const MIXER_LINE_SRC_GAIN_NAME: &str = "mixer:line-source-gain";
const MIXER_MIC_SRC_GAIN_NAME: &str = "mixer:mic-source-gain";
const MIXER_SPDIF_SRC_GAIN_NAME: &str = "mixer:spdif-source-gain";
const MIXER_ADAT_SRC_GAIN_NAME: &str = "mixer:adat-source-gain";
const MIXER_STREAM_SRC_GAIN_NAME: &str = "mixer:stream-source-gain";

#[derive(Debug)]
pub struct LatterMixerCtl<T>
where
    T: RmeFfLatterMixerSpecification
        + RmeFfWhollyCommandableParamsOperation<FfLatterMixerState>
        + RmeFfPartiallyCommandableParamsOperation<FfLatterMixerState>,
{
    pub elem_id_list: Vec<ElemId>,
    params: FfLatterMixerState,
    _phantom: PhantomData<T>,
}

impl<T> Default for LatterMixerCtl<T>
where
    T: RmeFfLatterMixerSpecification
        + RmeFfWhollyCommandableParamsOperation<FfLatterMixerState>
        + RmeFfPartiallyCommandableParamsOperation<FfLatterMixerState>,
{
    fn default() -> Self {
        let mut params = T::create_mixer_parameters();
        params.0.iter_mut().enumerate().for_each(|(i, mixer)| {
            mixer
                .stream_gains
                .iter_mut()
                .nth(i)
                .map(|gain| *gain = T::MIXER_INPUT_GAIN_ZERO as u16);
        });

        Self {
            elem_id_list: Default::default(),
            params,
            _phantom: Default::default(),
        }
    }
}

impl<T> LatterMixerCtl<T>
where
    T: RmeFfLatterMixerSpecification
        + RmeFfWhollyCommandableParamsOperation<FfLatterMixerState>
        + RmeFfPartiallyCommandableParamsOperation<FfLatterMixerState>,
{
    const SRC_GAIN_TLV: DbInterval = DbInterval {
        min: -6500,
        max: 600,
        linear: false,
        mute_avail: false,
    };

    pub fn cache(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let res = T::command_wholly(req, node, &mut self.params, timeout_ms);
        debug!(params = ?self.params, ?res);
        res
    }

    pub fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIXER_LINE_SRC_GAIN_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                T::OUTPUT_COUNT,
                T::MIXER_INPUT_GAIN_MIN,
                T::MIXER_INPUT_GAIN_MAX,
                T::MIXER_INPUT_GAIN_STEP,
                T::LINE_INPUT_COUNT,
                Some(&Vec::<u32>::from(&Self::SRC_GAIN_TLV)),
                true,
            )
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIXER_MIC_SRC_GAIN_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                T::OUTPUT_COUNT,
                T::MIXER_INPUT_GAIN_MIN,
                T::MIXER_INPUT_GAIN_MAX,
                T::MIXER_INPUT_GAIN_STEP,
                T::MIC_INPUT_COUNT,
                Some(&Vec::<u32>::from(&Self::SRC_GAIN_TLV)),
                true,
            )
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIXER_SPDIF_SRC_GAIN_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                T::OUTPUT_COUNT,
                T::MIXER_INPUT_GAIN_MIN,
                T::MIXER_INPUT_GAIN_MAX,
                T::MIXER_INPUT_GAIN_STEP,
                T::SPDIF_INPUT_COUNT,
                Some(&Vec::<u32>::from(&Self::SRC_GAIN_TLV)),
                true,
            )
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIXER_ADAT_SRC_GAIN_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                T::OUTPUT_COUNT,
                T::MIXER_INPUT_GAIN_MIN,
                T::MIXER_INPUT_GAIN_MAX,
                T::MIXER_INPUT_GAIN_STEP,
                T::ADAT_INPUT_COUNT,
                Some(&Vec::<u32>::from(&Self::SRC_GAIN_TLV)),
                true,
            )
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id =
            ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIXER_STREAM_SRC_GAIN_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                T::OUTPUT_COUNT,
                T::MIXER_INPUT_GAIN_MIN,
                T::MIXER_INPUT_GAIN_MAX,
                T::MIXER_INPUT_GAIN_STEP,
                T::STREAM_INPUT_COUNT,
                Some(&Vec::<u32>::from(&Self::SRC_GAIN_TLV)),
                true,
            )
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        Ok(())
    }

    pub fn read(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            MIXER_LINE_SRC_GAIN_NAME => {
                let index = elem_id.index() as usize;
                let mixer = self.params.0.iter().nth(index).ok_or_else(|| {
                    let msg = format!("Invalid index {} for mixers", index);
                    Error::new(FileError::Inval, &msg)
                })?;
                let vals: Vec<i32> = mixer.line_gains.iter().map(|&gain| gain as i32).collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            MIXER_MIC_SRC_GAIN_NAME => {
                let index = elem_id.index() as usize;
                let mixer = self.params.0.iter().nth(index).ok_or_else(|| {
                    let msg = format!("Invalid index {} for mixers", index);
                    Error::new(FileError::Inval, &msg)
                })?;
                let vals: Vec<i32> = mixer.mic_gains.iter().map(|&gain| gain as i32).collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            MIXER_SPDIF_SRC_GAIN_NAME => {
                let index = elem_id.index() as usize;
                let mixer = self.params.0.iter().nth(index).ok_or_else(|| {
                    let msg = format!("Invalid index {} for mixers", index);
                    Error::new(FileError::Inval, &msg)
                })?;
                let vals: Vec<i32> = mixer.spdif_gains.iter().map(|&gain| gain as i32).collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            MIXER_ADAT_SRC_GAIN_NAME => {
                let index = elem_id.index() as usize;
                let mixer = self.params.0.iter().nth(index).ok_or_else(|| {
                    let msg = format!("Invalid index {} for mixers", index);
                    Error::new(FileError::Inval, &msg)
                })?;
                let vals: Vec<i32> = mixer.adat_gains.iter().map(|&gain| gain as i32).collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            MIXER_STREAM_SRC_GAIN_NAME => {
                let index = elem_id.index() as usize;
                let mixer = self.params.0.iter().nth(index).ok_or_else(|| {
                    let msg = format!("Invalid index {} for mixers", index);
                    Error::new(FileError::Inval, &msg)
                })?;
                let vals: Vec<i32> = mixer.stream_gains.iter().map(|&gain| gain as i32).collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub fn write(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            MIXER_LINE_SRC_GAIN_NAME => {
                let mut params = self.params.clone();
                let index = elem_id.index() as usize;
                let mixer = params.0.iter_mut().nth(index).ok_or_else(|| {
                    let msg = format!("Invalid index {} for mixers", index);
                    Error::new(FileError::Inval, &msg)
                })?;
                mixer
                    .line_gains
                    .iter_mut()
                    .zip(elem_value.int())
                    .for_each(|(d, s)| *d = *s as u16);
                let res = T::command_partially(req, node, &mut self.params, params, timeout_ms);
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            MIXER_MIC_SRC_GAIN_NAME => {
                let mut params = self.params.clone();
                let index = elem_id.index() as usize;
                let mixer = params.0.iter_mut().nth(index).ok_or_else(|| {
                    let msg = format!("Invalid index {} for mixers", index);
                    Error::new(FileError::Inval, &msg)
                })?;
                mixer
                    .mic_gains
                    .iter_mut()
                    .zip(elem_value.int())
                    .for_each(|(d, s)| *d = *s as u16);
                let res = T::command_partially(req, node, &mut self.params, params, timeout_ms);
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            MIXER_SPDIF_SRC_GAIN_NAME => {
                let mut params = self.params.clone();
                let index = elem_id.index() as usize;
                let mixer = params.0.iter_mut().nth(index).ok_or_else(|| {
                    let msg = format!("Invalid index {} for mixers", index);
                    Error::new(FileError::Inval, &msg)
                })?;
                mixer
                    .spdif_gains
                    .iter_mut()
                    .zip(elem_value.int())
                    .for_each(|(d, s)| *d = *s as u16);
                let res = T::command_partially(req, node, &mut self.params, params, timeout_ms);
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            MIXER_ADAT_SRC_GAIN_NAME => {
                let mut params = self.params.clone();
                let index = elem_id.index() as usize;
                let mixer = params.0.iter_mut().nth(index).ok_or_else(|| {
                    let msg = format!("Invalid index {} for mixers", index);
                    Error::new(FileError::Inval, &msg)
                })?;
                mixer
                    .adat_gains
                    .iter_mut()
                    .zip(elem_value.int())
                    .for_each(|(d, s)| *d = *s as u16);
                let res = T::command_partially(req, node, &mut self.params, params, timeout_ms);
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            MIXER_STREAM_SRC_GAIN_NAME => {
                let mut params = self.params.clone();
                let index = elem_id.index() as usize;
                let mixer = params.0.iter_mut().nth(index).ok_or_else(|| {
                    let msg = format!("Invalid index {} for mixers", index);
                    Error::new(FileError::Inval, &msg)
                })?;
                mixer
                    .stream_gains
                    .iter_mut()
                    .zip(elem_value.int())
                    .for_each(|(d, s)| *d = *s as u16);
                let res = T::command_partially(req, node, &mut self.params, params, timeout_ms);
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

fn hpf_roll_off_level_to_str(level: &FfLatterHpfRollOffLevel) -> &str {
    match level {
        FfLatterHpfRollOffLevel::L6 => "6dB/octave",
        FfLatterHpfRollOffLevel::L12 => "12dB/octave",
        FfLatterHpfRollOffLevel::L18 => "18dB/octave",
        FfLatterHpfRollOffLevel::L24 => "24dB/octave",
    }
}

pub trait FfLatterHpfCtlOperation<T, U>
where
    T: RmeFfLatterHpfSpecification
        + RmeFfWhollyCommandableParamsOperation<U>
        + RmeFfPartiallyCommandableParamsOperation<U>,
    U: std::fmt::Debug + Clone + AsRef<FfLatterHpfState> + AsMut<FfLatterHpfState>,
{
    const HPF_ACTIVATE_NAME: &'static str;
    const HPF_CUT_OFF_NAME: &'static str;
    const HPF_ROLL_OFF_NAME: &'static str;

    const HPF_ROLL_OFF_LEVELS: &'static [FfLatterHpfRollOffLevel] = &[
        FfLatterHpfRollOffLevel::L6,
        FfLatterHpfRollOffLevel::L12,
        FfLatterHpfRollOffLevel::L18,
        FfLatterHpfRollOffLevel::L24,
    ];

    fn params(&self) -> &U;
    fn params_mut(&mut self) -> &mut U;

    fn elem_id_list_mut(&mut self) -> &mut Vec<ElemId>;

    const CH_COUNT: usize;

    fn cache(&mut self, req: &mut FwReq, node: &mut FwNode, timeout_ms: u32) -> Result<(), Error> {
        let res = T::command_wholly(req, node, self.params_mut(), timeout_ms);
        debug!(params = ?self.params(), ?res);
        res
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let mut elem_id_list = Vec::new();

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::HPF_ACTIVATE_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, 1, Self::CH_COUNT, true)
            .map(|mut list| elem_id_list.append(&mut list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::HPF_CUT_OFF_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                T::HPF_CUT_OFF_MIN,
                T::HPF_CUT_OFF_MAX,
                T::HPF_CUT_OFF_STEP,
                Self::CH_COUNT,
                None,
                true,
            )
            .map(|mut list| elem_id_list.append(&mut list))?;

        let labels: Vec<&str> = Self::HPF_ROLL_OFF_LEVELS
            .iter()
            .map(|l| hpf_roll_off_level_to_str(l))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::HPF_ROLL_OFF_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, Self::CH_COUNT, &labels, None, true)
            .map(|mut list| elem_id_list.append(&mut list))?;

        self.elem_id_list_mut().append(&mut elem_id_list);

        Ok(())
    }

    fn read(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        let n = elem_id.name();

        if n == Self::HPF_ACTIVATE_NAME {
            let hpf = self.params().as_ref();
            elem_value.set_bool(&hpf.activates);
            Ok(true)
        } else if n == Self::HPF_CUT_OFF_NAME {
            let hpf = self.params().as_ref();
            let vals: Vec<i32> = hpf.cut_offs.iter().map(|&cut_off| cut_off as i32).collect();
            elem_value.set_int(&vals);
            Ok(true)
        } else if n == Self::HPF_ROLL_OFF_NAME {
            let hpf = self.params().as_ref();
            let vals: Vec<u32> = hpf
                .roll_offs
                .iter()
                .map(|roll_off| {
                    let pos = Self::HPF_ROLL_OFF_LEVELS
                        .iter()
                        .position(|l| roll_off.eq(l))
                        .unwrap();
                    pos as u32
                })
                .collect();
            elem_value.set_enum(&vals);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        let n = elem_id.name();

        if n == Self::HPF_ACTIVATE_NAME {
            let mut params = self.params().clone();
            let hpf = params.as_mut();
            hpf.activates
                .iter_mut()
                .zip(elem_value.boolean())
                .for_each(|(activate, val)| *activate = val);
            let res = T::command_partially(req, node, self.params_mut(), params, timeout_ms);
            debug!(params = ?self.params(), ?res);
            res.map(|_| true)
        } else if n == Self::HPF_CUT_OFF_NAME {
            let mut params = self.params().clone();
            let hpf = params.as_mut();
            hpf.cut_offs
                .iter_mut()
                .zip(elem_value.int())
                .for_each(|(dst, &val)| *dst = val as u16);
            let res = T::command_partially(req, node, self.params_mut(), params, timeout_ms);
            debug!(params = ?self.params(), ?res);
            res.map(|_| true)
        } else if n == Self::HPF_ROLL_OFF_NAME {
            let mut params = self.params().clone();
            let hpf = params.as_mut();
            hpf.roll_offs
                .iter_mut()
                .zip(elem_value.enumerated())
                .try_for_each(|(level, &val)| {
                    let pos = val as usize;
                    Self::HPF_ROLL_OFF_LEVELS
                        .iter()
                        .nth(pos)
                        .ok_or_else(|| {
                            let msg = format!("Invalid index of roll off levels: {}", pos);
                            Error::new(FileError::Inval, &msg)
                        })
                        .map(|&l| *level = l)
                })?;
            let res = T::command_partially(req, node, self.params_mut(), params, timeout_ms);
            debug!(params = ?self.params(), ?res);
            res.map(|_| true)
        } else {
            Ok(false)
        }
    }
}

#[derive(Debug)]
pub struct LatterInputHpfCtl<T>
where
    T: RmeFfLatterInputSpecification
        + RmeFfLatterHpfSpecification
        + RmeFfWhollyCommandableParamsOperation<FfLatterInputHpfParameters>
        + RmeFfPartiallyCommandableParamsOperation<FfLatterInputHpfParameters>,
{
    pub elem_id_list: Vec<ElemId>,
    params: FfLatterInputHpfParameters,
    _phantom: PhantomData<T>,
}

impl<T> Default for LatterInputHpfCtl<T>
where
    T: RmeFfLatterInputSpecification
        + RmeFfLatterHpfSpecification
        + RmeFfWhollyCommandableParamsOperation<FfLatterInputHpfParameters>
        + RmeFfPartiallyCommandableParamsOperation<FfLatterInputHpfParameters>,
{
    fn default() -> Self {
        Self {
            elem_id_list: Default::default(),
            params: T::create_input_hpf_parameters(),
            _phantom: Default::default(),
        }
    }
}

impl<T> FfLatterHpfCtlOperation<T, FfLatterInputHpfParameters> for LatterInputHpfCtl<T>
where
    T: RmeFfLatterInputSpecification
        + RmeFfLatterHpfSpecification
        + RmeFfWhollyCommandableParamsOperation<FfLatterInputHpfParameters>
        + RmeFfPartiallyCommandableParamsOperation<FfLatterInputHpfParameters>,
{
    const HPF_ACTIVATE_NAME: &'static str = "input:hpf-activate";
    const HPF_CUT_OFF_NAME: &'static str = "input:hpf-cut-off";
    const HPF_ROLL_OFF_NAME: &'static str = "input:hpf-roll-off";

    fn params(&self) -> &FfLatterInputHpfParameters {
        &self.params
    }

    fn params_mut(&mut self) -> &mut FfLatterInputHpfParameters {
        &mut self.params
    }

    fn elem_id_list_mut(&mut self) -> &mut Vec<ElemId> {
        &mut self.elem_id_list
    }

    const CH_COUNT: usize = T::PHYS_INPUT_COUNT;
}

#[derive(Debug)]
pub struct LatterOutputHpfCtl<T>
where
    T: RmeFfLatterOutputSpecification
        + RmeFfLatterHpfSpecification
        + RmeFfWhollyCommandableParamsOperation<FfLatterOutputHpfParameters>
        + RmeFfPartiallyCommandableParamsOperation<FfLatterOutputHpfParameters>,
{
    pub elem_id_list: Vec<ElemId>,
    params: FfLatterOutputHpfParameters,
    _phantom: PhantomData<T>,
}

impl<T> Default for LatterOutputHpfCtl<T>
where
    T: RmeFfLatterOutputSpecification
        + RmeFfLatterHpfSpecification
        + RmeFfWhollyCommandableParamsOperation<FfLatterOutputHpfParameters>
        + RmeFfPartiallyCommandableParamsOperation<FfLatterOutputHpfParameters>,
{
    fn default() -> Self {
        Self {
            elem_id_list: Default::default(),
            params: T::create_output_hpf_parameters(),
            _phantom: Default::default(),
        }
    }
}

impl<T> FfLatterHpfCtlOperation<T, FfLatterOutputHpfParameters> for LatterOutputHpfCtl<T>
where
    T: RmeFfLatterOutputSpecification
        + RmeFfLatterHpfSpecification
        + RmeFfWhollyCommandableParamsOperation<FfLatterOutputHpfParameters>
        + RmeFfPartiallyCommandableParamsOperation<FfLatterOutputHpfParameters>,
{
    const HPF_ACTIVATE_NAME: &'static str = "output:hpf-activate";
    const HPF_CUT_OFF_NAME: &'static str = "output:hpf-cut-off";
    const HPF_ROLL_OFF_NAME: &'static str = "output:hpf-roll-off";

    fn params(&self) -> &FfLatterOutputHpfParameters {
        &self.params
    }

    fn params_mut(&mut self) -> &mut FfLatterOutputHpfParameters {
        &mut self.params
    }

    fn elem_id_list_mut(&mut self) -> &mut Vec<ElemId> {
        &mut self.elem_id_list
    }

    const CH_COUNT: usize = T::OUTPUT_COUNT;
}

fn eq_type_to_str(eq_type: &FfLatterChStripEqType) -> &str {
    match eq_type {
        FfLatterChStripEqType::Peak => "Peak",
        FfLatterChStripEqType::Shelf => "Shelf",
        FfLatterChStripEqType::LowPass => "Lowpass",
    }
}

pub trait FfLatterEqualizerCtlOperation<T, U>
where
    T: RmeFfLatterEqualizerSpecification
        + RmeFfWhollyCommandableParamsOperation<U>
        + RmeFfPartiallyCommandableParamsOperation<U>,
    U: std::fmt::Debug + Clone + AsRef<FfLatterEqState> + AsMut<FfLatterEqState>,
{
    const EQ_ACTIVATE_NAME: &'static str;
    const EQ_LOW_TYPE_NAME: &'static str;
    const EQ_LOW_GAIN_NAME: &'static str;
    const EQ_LOW_FREQ_NAME: &'static str;
    const EQ_LOW_QUALITY_NAME: &'static str;
    const EQ_MIDDLE_GAIN_NAME: &'static str;
    const EQ_MIDDLE_FREQ_NAME: &'static str;
    const EQ_MIDDLE_QUALITY_NAME: &'static str;
    const EQ_HIGH_TYPE_NAME: &'static str;
    const EQ_HIGH_GAIN_NAME: &'static str;
    const EQ_HIGH_FREQ_NAME: &'static str;
    const EQ_HIGH_QUALITY_NAME: &'static str;

    fn params(&self) -> &U;
    fn params_mut(&mut self) -> &mut U;

    fn elem_id_list_mut(&mut self) -> &mut Vec<ElemId>;

    const CH_COUNT: usize;

    const EQ_TYPES: &'static [FfLatterChStripEqType] = &[
        FfLatterChStripEqType::Peak,
        FfLatterChStripEqType::Shelf,
        FfLatterChStripEqType::LowPass,
    ];

    fn cache(&mut self, req: &mut FwReq, node: &mut FwNode, timeout_ms: u32) -> Result<(), Error> {
        let res = T::command_wholly(req, node, self.params_mut(), timeout_ms);
        debug!(params = ?self.params(), ?res);
        res
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let mut elem_id_list = Vec::new();

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::EQ_ACTIVATE_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, 1, Self::CH_COUNT, true)
            .map(|mut list| elem_id_list.append(&mut list))?;

        let labels: Vec<&str> = Self::EQ_TYPES.iter().map(|t| eq_type_to_str(t)).collect();

        [Self::EQ_LOW_TYPE_NAME, Self::EQ_HIGH_TYPE_NAME]
            .iter()
            .try_for_each(|name| {
                let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
                card_cntr
                    .add_enum_elems(&elem_id, 1, Self::CH_COUNT, &labels, None, true)
                    .map(|mut list| elem_id_list.append(&mut list))
            })?;

        [
            Self::EQ_LOW_GAIN_NAME,
            Self::EQ_MIDDLE_GAIN_NAME,
            Self::EQ_HIGH_GAIN_NAME,
        ]
        .iter()
        .try_for_each(|name| {
            let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
            card_cntr
                .add_int_elems(
                    &elem_id,
                    1,
                    T::EQ_GAIN_MIN,
                    T::EQ_GAIN_MAX,
                    T::EQ_GAIN_STEP,
                    Self::CH_COUNT,
                    None,
                    true,
                )
                .map(|mut list| elem_id_list.append(&mut list))
        })?;

        [
            Self::EQ_LOW_FREQ_NAME,
            Self::EQ_MIDDLE_FREQ_NAME,
            Self::EQ_HIGH_FREQ_NAME,
        ]
        .iter()
        .try_for_each(|name| {
            let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
            card_cntr
                .add_int_elems(
                    &elem_id,
                    1,
                    T::EQ_FREQ_MIN,
                    T::EQ_FREQ_MAX,
                    T::EQ_FREQ_STEP,
                    Self::CH_COUNT,
                    None,
                    true,
                )
                .map(|mut list| elem_id_list.append(&mut list))
        })?;

        [
            Self::EQ_LOW_QUALITY_NAME,
            Self::EQ_MIDDLE_QUALITY_NAME,
            Self::EQ_HIGH_QUALITY_NAME,
        ]
        .iter()
        .try_for_each(|name| {
            let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
            card_cntr
                .add_int_elems(
                    &elem_id,
                    1,
                    T::EQ_QUALITY_MIN,
                    T::EQ_QUALITY_MAX,
                    T::EQ_QUALITY_STEP,
                    Self::CH_COUNT,
                    None,
                    true,
                )
                .map(|mut list| elem_id_list.append(&mut list))
        })?;

        self.elem_id_list_mut().append(&mut elem_id_list);

        Ok(())
    }

    fn read(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        let n = elem_id.name();

        if n == Self::EQ_ACTIVATE_NAME {
            let params = self.params().as_ref();
            elem_value.set_bool(&params.activates);
            Ok(true)
        } else if n == Self::EQ_LOW_TYPE_NAME {
            let params = self.params().as_ref();
            let vals: Vec<u32> = params
                .low_types
                .iter()
                .map(|eq_type| {
                    let pos = Self::EQ_TYPES.iter().position(|t| t.eq(eq_type)).unwrap();
                    pos as u32
                })
                .collect();
            elem_value.set_enum(&vals);
            Ok(true)
        } else if n == Self::EQ_HIGH_TYPE_NAME {
            let params = self.params().as_ref();
            let vals: Vec<u32> = params
                .high_types
                .iter()
                .map(|eq_type| {
                    let pos = Self::EQ_TYPES.iter().position(|t| t.eq(eq_type)).unwrap();
                    pos as u32
                })
                .collect();
            elem_value.set_enum(&vals);
            Ok(true)
        } else if n == Self::EQ_LOW_GAIN_NAME {
            let params = self.params().as_ref();
            let vals: Vec<i32> = params.low_gains.iter().map(|&gain| gain as i32).collect();
            elem_value.set_int(&vals);
            Ok(true)
        } else if n == Self::EQ_MIDDLE_GAIN_NAME {
            let params = self.params().as_ref();
            let vals: Vec<i32> = params
                .middle_gains
                .iter()
                .map(|&gain| gain as i32)
                .collect();
            elem_value.set_int(&vals);
            Ok(true)
        } else if n == Self::EQ_HIGH_GAIN_NAME {
            let params = self.params().as_ref();
            let vals: Vec<i32> = params.high_gains.iter().map(|&gain| gain as i32).collect();
            elem_value.set_int(&vals);
            Ok(true)
        } else if n == Self::EQ_LOW_FREQ_NAME {
            let params = self.params().as_ref();
            let vals: Vec<i32> = params.low_freqs.iter().map(|&gain| gain as i32).collect();
            elem_value.set_int(&vals);
            Ok(true)
        } else if n == Self::EQ_MIDDLE_FREQ_NAME {
            let params = self.params().as_ref();
            let vals: Vec<i32> = params
                .middle_freqs
                .iter()
                .map(|&gain| gain as i32)
                .collect();
            elem_value.set_int(&vals);
            Ok(true)
        } else if n == Self::EQ_HIGH_FREQ_NAME {
            let params = self.params().as_ref();
            let vals: Vec<i32> = params.high_freqs.iter().map(|&gain| gain as i32).collect();
            elem_value.set_int(&vals);
            Ok(true)
        } else if n == Self::EQ_LOW_QUALITY_NAME {
            let params = self.params().as_ref();
            let vals: Vec<i32> = params
                .low_qualities
                .iter()
                .map(|&gain| gain as i32)
                .collect();
            elem_value.set_int(&vals);
            Ok(true)
        } else if n == Self::EQ_MIDDLE_QUALITY_NAME {
            let params = self.params().as_ref();
            let vals: Vec<i32> = params
                .middle_qualities
                .iter()
                .map(|&gain| gain as i32)
                .collect();
            elem_value.set_int(&vals);
            Ok(true)
        } else if n == Self::EQ_HIGH_QUALITY_NAME {
            let params = self.params().as_ref();
            let vals: Vec<i32> = params
                .high_qualities
                .iter()
                .map(|&gain| gain as i32)
                .collect();
            elem_value.set_int(&vals);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        let n = elem_id.name();

        if n == Self::EQ_ACTIVATE_NAME {
            let mut params = self.params().clone();
            params
                .as_mut()
                .activates
                .iter_mut()
                .zip(elem_value.boolean())
                .for_each(|(activate, val)| *activate = val);
            let res = T::command_partially(req, node, self.params_mut(), params, timeout_ms);
            debug!(params = ?self.params(), ?res);
            res.map(|_| true)
        } else if n == Self::EQ_LOW_TYPE_NAME {
            let mut params = self.params().clone();
            params
                .as_mut()
                .low_types
                .iter_mut()
                .zip(elem_value.enumerated())
                .try_for_each(|(eq_type, &val)| {
                    let pos = val as usize;
                    Self::EQ_TYPES
                        .iter()
                        .nth(pos)
                        .ok_or_else(|| {
                            let msg = format!("Invalid index of equalizer types: {}", pos);
                            Error::new(FileError::Inval, &msg)
                        })
                        .map(|&t| *eq_type = t)
                })?;
            let res = T::command_partially(req, node, self.params_mut(), params, timeout_ms);
            debug!(params = ?self.params(), ?res);
            res.map(|_| true)
        } else if n == Self::EQ_HIGH_TYPE_NAME {
            let mut params = self.params().clone();
            params
                .as_mut()
                .high_types
                .iter_mut()
                .zip(elem_value.enumerated())
                .try_for_each(|(eq_type, &val)| {
                    let pos = val as usize;
                    Self::EQ_TYPES
                        .iter()
                        .nth(pos)
                        .ok_or_else(|| {
                            let msg = format!("Invalid index of equalizer types: {}", pos);
                            Error::new(FileError::Inval, &msg)
                        })
                        .map(|&t| *eq_type = t)
                })?;
            let res = T::command_partially(req, node, self.params_mut(), params, timeout_ms);
            debug!(params = ?self.params(), ?res);
            res.map(|_| true)
        } else if n == Self::EQ_LOW_GAIN_NAME {
            let mut params = self.params().clone();
            params
                .as_mut()
                .low_gains
                .iter_mut()
                .zip(elem_value.int())
                .for_each(|(gain, &val)| *gain = val as i16);
            let res = T::command_partially(req, node, self.params_mut(), params, timeout_ms);
            debug!(params = ?self.params(), ?res);
            res.map(|_| true)
        } else if n == Self::EQ_MIDDLE_GAIN_NAME {
            let mut params = self.params().clone();
            params
                .as_mut()
                .middle_gains
                .iter_mut()
                .zip(elem_value.int())
                .for_each(|(gain, &val)| *gain = val as i16);
            let res = T::command_partially(req, node, self.params_mut(), params, timeout_ms);
            debug!(params = ?self.params(), ?res);
            res.map(|_| true)
        } else if n == Self::EQ_HIGH_GAIN_NAME {
            let mut params = self.params().clone();
            params
                .as_mut()
                .high_gains
                .iter_mut()
                .zip(elem_value.int())
                .for_each(|(gain, &val)| *gain = val as i16);
            let res = T::command_partially(req, node, self.params_mut(), params, timeout_ms);
            debug!(params = ?self.params(), ?res);
            res.map(|_| true)
        } else if n == Self::EQ_LOW_FREQ_NAME {
            let mut params = self.params().clone();
            params
                .as_mut()
                .low_freqs
                .iter_mut()
                .zip(elem_value.int())
                .for_each(|(freq, &val)| *freq = val as u16);
            let res = T::command_partially(req, node, self.params_mut(), params, timeout_ms);
            debug!(params = ?self.params(), ?res);
            res.map(|_| true)
        } else if n == Self::EQ_MIDDLE_FREQ_NAME {
            let mut params = self.params().clone();
            params
                .as_mut()
                .middle_freqs
                .iter_mut()
                .zip(elem_value.int())
                .for_each(|(freq, &val)| *freq = val as u16);
            let res = T::command_partially(req, node, self.params_mut(), params, timeout_ms);
            debug!(params = ?self.params(), ?res);
            res.map(|_| true)
        } else if n == Self::EQ_HIGH_FREQ_NAME {
            let mut params = self.params().clone();
            params
                .as_mut()
                .high_freqs
                .iter_mut()
                .zip(elem_value.int())
                .for_each(|(freq, &val)| *freq = val as u16);
            let res = T::command_partially(req, node, self.params_mut(), params, timeout_ms);
            debug!(params = ?self.params(), ?res);
            res.map(|_| true)
        } else if n == Self::EQ_LOW_QUALITY_NAME {
            let mut params = self.params().clone();
            params
                .as_mut()
                .low_qualities
                .iter_mut()
                .zip(elem_value.int())
                .for_each(|(quality, &val)| *quality = val as u16);
            let res = T::command_partially(req, node, self.params_mut(), params, timeout_ms);
            debug!(params = ?self.params(), ?res);
            res.map(|_| true)
        } else if n == Self::EQ_MIDDLE_QUALITY_NAME {
            let mut params = self.params().clone();
            params
                .as_mut()
                .middle_qualities
                .iter_mut()
                .zip(elem_value.int())
                .for_each(|(quality, &val)| *quality = val as u16);
            let res = T::command_partially(req, node, self.params_mut(), params, timeout_ms);
            debug!(params = ?self.params(), ?res);
            res.map(|_| true)
        } else if n == Self::EQ_HIGH_QUALITY_NAME {
            let mut params = self.params().clone();
            params
                .as_mut()
                .high_qualities
                .iter_mut()
                .zip(elem_value.int())
                .for_each(|(quality, &val)| *quality = val as u16);
            let res = T::command_partially(req, node, self.params_mut(), params, timeout_ms);
            debug!(params = ?self.params(), ?res);
            res.map(|_| true)
        } else {
            Ok(false)
        }
    }
}

#[derive(Debug)]
pub struct LatterInputEqualizerCtl<T>
where
    T: RmeFfLatterInputSpecification
        + RmeFfLatterEqualizerSpecification
        + RmeFfWhollyCommandableParamsOperation<FfLatterInputEqualizerParameters>
        + RmeFfPartiallyCommandableParamsOperation<FfLatterInputEqualizerParameters>,
{
    pub elem_id_list: Vec<ElemId>,
    params: FfLatterInputEqualizerParameters,
    _phantom: PhantomData<T>,
}

impl<T> Default for LatterInputEqualizerCtl<T>
where
    T: RmeFfLatterInputSpecification
        + RmeFfLatterEqualizerSpecification
        + RmeFfWhollyCommandableParamsOperation<FfLatterInputEqualizerParameters>
        + RmeFfPartiallyCommandableParamsOperation<FfLatterInputEqualizerParameters>,
{
    fn default() -> Self {
        Self {
            elem_id_list: Default::default(),
            params: T::create_input_equalizer_parameters(),
            _phantom: Default::default(),
        }
    }
}

impl<T> FfLatterEqualizerCtlOperation<T, FfLatterInputEqualizerParameters>
    for LatterInputEqualizerCtl<T>
where
    T: RmeFfLatterInputSpecification
        + RmeFfLatterEqualizerSpecification
        + RmeFfWhollyCommandableParamsOperation<FfLatterInputEqualizerParameters>
        + RmeFfPartiallyCommandableParamsOperation<FfLatterInputEqualizerParameters>,
{
    const EQ_ACTIVATE_NAME: &'static str = "input:eq-activate";
    const EQ_LOW_TYPE_NAME: &'static str = "input:eq-low-type";
    const EQ_LOW_GAIN_NAME: &'static str = "input:eq-low-gain";
    const EQ_LOW_FREQ_NAME: &'static str = "input:eq-low-freq";
    const EQ_LOW_QUALITY_NAME: &'static str = "input:eq-low-quality";
    const EQ_MIDDLE_GAIN_NAME: &'static str = "input:eq-middle-gain";
    const EQ_MIDDLE_FREQ_NAME: &'static str = "input:eq-middle-freq";
    const EQ_MIDDLE_QUALITY_NAME: &'static str = "input:eq-middle-quality";
    const EQ_HIGH_TYPE_NAME: &'static str = "input:eq-high-type";
    const EQ_HIGH_GAIN_NAME: &'static str = "input:eq-high-gain";
    const EQ_HIGH_FREQ_NAME: &'static str = "input:eq-high-freq";
    const EQ_HIGH_QUALITY_NAME: &'static str = "input:eq-high-quality";

    const CH_COUNT: usize = T::PHYS_INPUT_COUNT;

    fn params(&self) -> &FfLatterInputEqualizerParameters {
        &self.params
    }

    fn params_mut(&mut self) -> &mut FfLatterInputEqualizerParameters {
        &mut self.params
    }

    fn elem_id_list_mut(&mut self) -> &mut Vec<ElemId> {
        &mut self.elem_id_list
    }
}

#[derive(Debug)]
pub struct LatterOutputEqualizerCtl<T>
where
    T: RmeFfLatterOutputSpecification
        + RmeFfLatterEqualizerSpecification
        + RmeFfWhollyCommandableParamsOperation<FfLatterOutputEqualizerParameters>
        + RmeFfPartiallyCommandableParamsOperation<FfLatterOutputEqualizerParameters>,
{
    pub elem_id_list: Vec<ElemId>,
    params: FfLatterOutputEqualizerParameters,
    _phantom: PhantomData<T>,
}

impl<T> Default for LatterOutputEqualizerCtl<T>
where
    T: RmeFfLatterOutputSpecification
        + RmeFfLatterEqualizerSpecification
        + RmeFfWhollyCommandableParamsOperation<FfLatterOutputEqualizerParameters>
        + RmeFfPartiallyCommandableParamsOperation<FfLatterOutputEqualizerParameters>,
{
    fn default() -> Self {
        Self {
            elem_id_list: Default::default(),
            params: T::create_output_equalizer_parameters(),
            _phantom: Default::default(),
        }
    }
}

impl<T> FfLatterEqualizerCtlOperation<T, FfLatterOutputEqualizerParameters>
    for LatterOutputEqualizerCtl<T>
where
    T: RmeFfLatterOutputSpecification
        + RmeFfLatterEqualizerSpecification
        + RmeFfWhollyCommandableParamsOperation<FfLatterOutputEqualizerParameters>
        + RmeFfPartiallyCommandableParamsOperation<FfLatterOutputEqualizerParameters>,
{
    const EQ_ACTIVATE_NAME: &'static str = "output:eq-activate";
    const EQ_LOW_TYPE_NAME: &'static str = "output:eq-low-type";
    const EQ_LOW_GAIN_NAME: &'static str = "output:eq-low-gain";
    const EQ_LOW_FREQ_NAME: &'static str = "output:eq-low-freq";
    const EQ_LOW_QUALITY_NAME: &'static str = "output:eq-low-quality";
    const EQ_MIDDLE_GAIN_NAME: &'static str = "output:eq-middle-gain";
    const EQ_MIDDLE_FREQ_NAME: &'static str = "output:eq-middle-freq";
    const EQ_MIDDLE_QUALITY_NAME: &'static str = "output:eq-middle-quality";
    const EQ_HIGH_TYPE_NAME: &'static str = "output:eq-high-type";
    const EQ_HIGH_GAIN_NAME: &'static str = "output:eq-high-gain";
    const EQ_HIGH_FREQ_NAME: &'static str = "output:eq-high-freq";
    const EQ_HIGH_QUALITY_NAME: &'static str = "output:eq-high-quality";

    const CH_COUNT: usize = T::OUTPUT_COUNT;

    fn params(&self) -> &FfLatterOutputEqualizerParameters {
        &self.params
    }

    fn params_mut(&mut self) -> &mut FfLatterOutputEqualizerParameters {
        &mut self.params
    }

    fn elem_id_list_mut(&mut self) -> &mut Vec<ElemId> {
        &mut self.elem_id_list
    }
}

pub trait FfLatterDynamicsCtlOperation<T, U>
where
    T: RmeFfLatterDynamicsSpecification
        + RmeFfWhollyCommandableParamsOperation<U>
        + RmeFfPartiallyCommandableParamsOperation<U>,
    U: std::fmt::Debug + Clone + AsRef<FfLatterDynState> + AsMut<FfLatterDynState>,
{
    const DYN_ACTIVATE_NAME: &'static str;
    const DYN_GAIN_NAME: &'static str;
    const DYN_ATTACK_NAME: &'static str;
    const DYN_RELEASE_NAME: &'static str;
    const DYN_COMP_THRESHOLD_NAME: &'static str;
    const DYN_COMP_RATIO_NAME: &'static str;
    const DYN_EX_THRESHOLD_NAME: &'static str;
    const DYN_EX_RATIO_NAME: &'static str;

    fn params(&self) -> &U;
    fn params_mut(&mut self) -> &mut U;

    fn elem_id_list_mut(&mut self) -> &mut Vec<ElemId>;

    const CH_COUNT: usize;

    fn cache(&mut self, req: &mut FwReq, node: &mut FwNode, timeout_ms: u32) -> Result<(), Error> {
        let res = T::command_wholly(req, node, &mut self.params_mut(), timeout_ms);
        debug!(params = ?self.params(), ?res);
        res
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let mut elem_id_list = Vec::new();

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::DYN_ACTIVATE_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, 1, Self::CH_COUNT, true)
            .map(|mut list| elem_id_list.append(&mut list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::DYN_GAIN_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                T::DYN_GAIN_MIN,
                T::DYN_GAIN_MAX,
                T::DYN_GAIN_STEP,
                Self::CH_COUNT,
                None,
                true,
            )
            .map(|mut list| elem_id_list.append(&mut list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::DYN_ATTACK_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                T::DYN_ATTACK_MIN,
                T::DYN_ATTACK_MAX,
                T::DYN_ATTACK_STEP,
                Self::CH_COUNT,
                None,
                true,
            )
            .map(|mut list| elem_id_list.append(&mut list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::DYN_RELEASE_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                T::DYN_RELEASE_MIN,
                T::DYN_RELEASE_MAX,
                T::DYN_RELEASE_STEP,
                Self::CH_COUNT,
                None,
                true,
            )
            .map(|mut list| elem_id_list.append(&mut list))?;

        let elem_id =
            ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::DYN_COMP_THRESHOLD_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                T::DYN_COMP_THRESHOLD_MIN,
                T::DYN_COMP_THRESHOLD_MAX,
                T::DYN_COMP_THRESHOLD_STEP,
                Self::CH_COUNT,
                None,
                true,
            )
            .map(|mut list| elem_id_list.append(&mut list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::DYN_COMP_RATIO_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                T::DYN_RATIO_MIN,
                T::DYN_RATIO_MAX,
                T::DYN_RATIO_STEP,
                Self::CH_COUNT,
                None,
                true,
            )
            .map(|mut list| elem_id_list.append(&mut list))?;

        let elem_id =
            ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::DYN_EX_THRESHOLD_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                T::DYN_EX_THRESHOLD_MIN,
                T::DYN_EX_THRESHOLD_MAX,
                T::DYN_EX_THRESHOLD_STEP,
                Self::CH_COUNT,
                None,
                true,
            )
            .map(|mut list| elem_id_list.append(&mut list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::DYN_EX_RATIO_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                T::DYN_RATIO_MIN,
                T::DYN_RATIO_MAX,
                T::DYN_RATIO_STEP,
                Self::CH_COUNT,
                None,
                true,
            )
            .map(|mut list| elem_id_list.append(&mut list))?;

        self.elem_id_list_mut().append(&mut elem_id_list);

        Ok(())
    }

    fn read(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        let n = elem_id.name();

        if n == Self::DYN_ACTIVATE_NAME {
            let params = self.params().as_ref();
            elem_value.set_bool(&params.activates);
            Ok(true)
        } else if n == Self::DYN_GAIN_NAME {
            let params = self.params().as_ref();
            let vals: Vec<i32> = params.gains.iter().map(|&gain| gain as i32).collect();
            elem_value.set_int(&vals);
            Ok(true)
        } else if n == Self::DYN_ATTACK_NAME {
            let params = self.params().as_ref();
            let vals: Vec<i32> = params.attacks.iter().map(|&attack| attack as i32).collect();
            elem_value.set_int(&vals);
            Ok(true)
        } else if n == Self::DYN_RELEASE_NAME {
            let params = self.params().as_ref();
            let vals: Vec<i32> = params
                .releases
                .iter()
                .map(|&release| release as i32)
                .collect();
            elem_value.set_int(&vals);
            Ok(true)
        } else if n == Self::DYN_COMP_THRESHOLD_NAME {
            let params = self.params().as_ref();
            let vals: Vec<i32> = params
                .compressor_thresholds
                .iter()
                .map(|&th| th as i32)
                .collect();
            elem_value.set_int(&vals);
            Ok(true)
        } else if n == Self::DYN_COMP_RATIO_NAME {
            let params = self.params().as_ref();
            let vals: Vec<i32> = params
                .compressor_ratios
                .iter()
                .map(|&ratio| ratio as i32)
                .collect();
            elem_value.set_int(&vals);
            Ok(true)
        } else if n == Self::DYN_EX_THRESHOLD_NAME {
            let params = self.params().as_ref();
            let vals: Vec<i32> = params
                .expander_thresholds
                .iter()
                .map(|&th| th as i32)
                .collect();
            elem_value.set_int(&vals);
            Ok(true)
        } else if n == Self::DYN_EX_RATIO_NAME {
            let params = self.params().as_ref();
            let vals: Vec<i32> = params
                .expander_ratios
                .iter()
                .map(|&ratio| ratio as i32)
                .collect();
            elem_value.set_int(&vals);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        let n = elem_id.name();

        if n == Self::DYN_ACTIVATE_NAME {
            let mut params = self.params().clone();
            params
                .as_mut()
                .activates
                .iter_mut()
                .zip(elem_value.boolean())
                .for_each(|(activate, val)| *activate = val);
            let res = T::command_partially(req, node, self.params_mut(), params, timeout_ms);
            debug!(params = ?self.params(), ?res);
            res.map(|_| true)
        } else if n == Self::DYN_GAIN_NAME {
            let mut params = self.params().clone();
            params
                .as_mut()
                .gains
                .iter_mut()
                .zip(elem_value.int())
                .for_each(|(gain, &val)| *gain = val as i16);
            let res = T::command_partially(req, node, self.params_mut(), params, timeout_ms);
            debug!(params = ?self.params(), ?res);
            res.map(|_| true)
        } else if n == Self::DYN_ATTACK_NAME {
            let mut params = self.params().clone();
            params
                .as_mut()
                .attacks
                .iter_mut()
                .zip(elem_value.int())
                .for_each(|(attack, &val)| *attack = val as u16);
            let res = T::command_partially(req, node, self.params_mut(), params, timeout_ms);
            debug!(params = ?self.params(), ?res);
            res.map(|_| true)
        } else if n == Self::DYN_RELEASE_NAME {
            let mut params = self.params().clone();
            params
                .as_mut()
                .releases
                .iter_mut()
                .zip(elem_value.int())
                .for_each(|(release, &val)| *release = val as u16);
            let res = T::command_partially(req, node, self.params_mut(), params, timeout_ms);
            debug!(params = ?self.params(), ?res);
            res.map(|_| true)
        } else if n == Self::DYN_COMP_THRESHOLD_NAME {
            let mut params = self.params().clone();
            params
                .as_mut()
                .compressor_thresholds
                .iter_mut()
                .zip(elem_value.int())
                .for_each(|(threshold, &val)| *threshold = val as i16);
            let res = T::command_partially(req, node, self.params_mut(), params, timeout_ms);
            debug!(params = ?self.params(), ?res);
            res.map(|_| true)
        } else if n == Self::DYN_COMP_RATIO_NAME {
            let mut params = self.params().clone();
            params
                .as_mut()
                .compressor_ratios
                .iter_mut()
                .zip(elem_value.int())
                .for_each(|(ratio, &val)| *ratio = val as u16);
            let res = T::command_partially(req, node, self.params_mut(), params, timeout_ms);
            debug!(params = ?self.params(), ?res);
            res.map(|_| true)
        } else if n == Self::DYN_EX_THRESHOLD_NAME {
            let mut params = self.params().clone();
            params
                .as_mut()
                .expander_thresholds
                .iter_mut()
                .zip(elem_value.int())
                .for_each(|(threshold, &val)| *threshold = val as i16);
            let res = T::command_partially(req, node, self.params_mut(), params, timeout_ms);
            debug!(params = ?self.params(), ?res);
            res.map(|_| true)
        } else if n == Self::DYN_EX_RATIO_NAME {
            let mut params = self.params().clone();
            params
                .as_mut()
                .expander_ratios
                .iter_mut()
                .zip(elem_value.int())
                .for_each(|(ratio, &val)| *ratio = val as u16);
            let res = T::command_partially(req, node, self.params_mut(), params, timeout_ms);
            debug!(params = ?self.params(), ?res);
            res.map(|_| true)
        } else {
            Ok(false)
        }
    }
}

#[derive(Debug)]
pub struct LatterInputDynamicsCtl<T>
where
    T: RmeFfLatterInputSpecification
        + RmeFfLatterDynamicsSpecification
        + RmeFfWhollyCommandableParamsOperation<FfLatterInputDynamicsParameters>
        + RmeFfPartiallyCommandableParamsOperation<FfLatterInputDynamicsParameters>,
{
    pub elem_id_list: Vec<ElemId>,
    params: FfLatterInputDynamicsParameters,
    _phantom: PhantomData<T>,
}

impl<T> Default for LatterInputDynamicsCtl<T>
where
    T: RmeFfLatterInputSpecification
        + RmeFfLatterDynamicsSpecification
        + RmeFfWhollyCommandableParamsOperation<FfLatterInputDynamicsParameters>
        + RmeFfPartiallyCommandableParamsOperation<FfLatterInputDynamicsParameters>,
{
    fn default() -> Self {
        Self {
            elem_id_list: Default::default(),
            params: T::create_input_dynamics_parameters(),
            _phantom: Default::default(),
        }
    }
}

impl<T> FfLatterDynamicsCtlOperation<T, FfLatterInputDynamicsParameters>
    for LatterInputDynamicsCtl<T>
where
    T: RmeFfLatterInputSpecification
        + RmeFfLatterDynamicsSpecification
        + RmeFfWhollyCommandableParamsOperation<FfLatterInputDynamicsParameters>
        + RmeFfPartiallyCommandableParamsOperation<FfLatterInputDynamicsParameters>,
{
    const DYN_ACTIVATE_NAME: &'static str = "input:dyn-activate";
    const DYN_GAIN_NAME: &'static str = "input:dyn-gain";
    const DYN_ATTACK_NAME: &'static str = "input:dyn-attack";
    const DYN_RELEASE_NAME: &'static str = "input:dyn-release";
    const DYN_COMP_THRESHOLD_NAME: &'static str = "input:dyn-compressor-threshold";
    const DYN_COMP_RATIO_NAME: &'static str = "input:dyn-compressor-ratio";
    const DYN_EX_THRESHOLD_NAME: &'static str = "input:dyn-expander-threshold";
    const DYN_EX_RATIO_NAME: &'static str = "input:dyn-expander-ratio";

    fn params(&self) -> &FfLatterInputDynamicsParameters {
        &self.params
    }

    fn params_mut(&mut self) -> &mut FfLatterInputDynamicsParameters {
        &mut self.params
    }

    fn elem_id_list_mut(&mut self) -> &mut Vec<ElemId> {
        &mut self.elem_id_list
    }

    const CH_COUNT: usize = T::PHYS_INPUT_COUNT;
}

#[derive(Debug)]
pub struct LatterOutputDynamicsCtl<T>
where
    T: RmeFfLatterOutputSpecification
        + RmeFfLatterDynamicsSpecification
        + RmeFfWhollyCommandableParamsOperation<FfLatterOutputDynamicsParameters>
        + RmeFfPartiallyCommandableParamsOperation<FfLatterOutputDynamicsParameters>,
{
    pub elem_id_list: Vec<ElemId>,
    params: FfLatterOutputDynamicsParameters,
    _phantom: PhantomData<T>,
}

impl<T> Default for LatterOutputDynamicsCtl<T>
where
    T: RmeFfLatterOutputSpecification
        + RmeFfLatterDynamicsSpecification
        + RmeFfWhollyCommandableParamsOperation<FfLatterOutputDynamicsParameters>
        + RmeFfPartiallyCommandableParamsOperation<FfLatterOutputDynamicsParameters>,
{
    fn default() -> Self {
        Self {
            elem_id_list: Default::default(),
            params: T::create_output_dynamics_parameters(),
            _phantom: Default::default(),
        }
    }
}

impl<T> FfLatterDynamicsCtlOperation<T, FfLatterOutputDynamicsParameters>
    for LatterOutputDynamicsCtl<T>
where
    T: RmeFfLatterOutputSpecification
        + RmeFfLatterDynamicsSpecification
        + RmeFfWhollyCommandableParamsOperation<FfLatterOutputDynamicsParameters>
        + RmeFfPartiallyCommandableParamsOperation<FfLatterOutputDynamicsParameters>,
{
    const DYN_ACTIVATE_NAME: &'static str = "output:dyn-activate";
    const DYN_GAIN_NAME: &'static str = "output:dyn-gain";
    const DYN_ATTACK_NAME: &'static str = "output:dyn-attack";
    const DYN_RELEASE_NAME: &'static str = "output:dyn-release";
    const DYN_COMP_THRESHOLD_NAME: &'static str = "output:dyn-compressor-threshold";
    const DYN_COMP_RATIO_NAME: &'static str = "output:dyn-compressor-ratio";
    const DYN_EX_THRESHOLD_NAME: &'static str = "output:dyn-expander-threshold";
    const DYN_EX_RATIO_NAME: &'static str = "output:dyn-expander-ratio";

    fn params(&self) -> &FfLatterOutputDynamicsParameters {
        &self.params
    }

    fn params_mut(&mut self) -> &mut FfLatterOutputDynamicsParameters {
        &mut self.params
    }

    fn elem_id_list_mut(&mut self) -> &mut Vec<ElemId> {
        &mut self.elem_id_list
    }

    const CH_COUNT: usize = T::OUTPUT_COUNT;
}

pub trait FfLatterAutolevelCtlOperation<T, U>
where
    T: RmeFfLatterAutolevelSpecification
        + RmeFfWhollyCommandableParamsOperation<U>
        + RmeFfPartiallyCommandableParamsOperation<U>,
    U: std::fmt::Debug + Clone + AsRef<FfLatterAutolevelState> + AsMut<FfLatterAutolevelState>,
{
    const AUTOLEVEL_ACTIVATE_NAME: &'static str;
    const AUTOLEVEL_MAX_GAIN_NAME: &'static str;
    const AUTOLEVEL_HEAD_ROOM_NAME: &'static str;
    const AUTOLEVEL_RISE_TIME_NAME: &'static str;

    fn params(&self) -> &U;
    fn params_mut(&mut self) -> &mut U;

    fn elem_id_list_mut(&mut self) -> &mut Vec<ElemId>;

    const CH_COUNT: usize;

    fn cache(&mut self, req: &mut FwReq, node: &mut FwNode, timeout_ms: u32) -> Result<(), Error> {
        let res = T::command_wholly(req, node, self.params_mut(), timeout_ms);
        debug!(params = ?self.params(), ?res);
        res
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let mut elem_id_list = Vec::new();

        let elem_id =
            ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::AUTOLEVEL_ACTIVATE_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, 1, Self::CH_COUNT, true)
            .map(|mut list| elem_id_list.append(&mut list))?;

        let elem_id =
            ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::AUTOLEVEL_MAX_GAIN_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                T::AUTOLEVEL_MAX_GAIN_MIN,
                T::AUTOLEVEL_MAX_GAIN_MAX,
                T::AUTOLEVEL_MAX_GAIN_STEP,
                Self::CH_COUNT,
                None,
                true,
            )
            .map(|mut list| elem_id_list.append(&mut list))?;

        let elem_id = ElemId::new_by_name(
            ElemIfaceType::Mixer,
            0,
            0,
            Self::AUTOLEVEL_HEAD_ROOM_NAME,
            0,
        );
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                T::AUTOLEVEL_HEAD_ROOM_MIN,
                T::AUTOLEVEL_HEAD_ROOM_MAX,
                T::AUTOLEVEL_HEAD_ROOM_STEP,
                Self::CH_COUNT,
                None,
                true,
            )
            .map(|mut list| elem_id_list.append(&mut list))?;

        let elem_id = ElemId::new_by_name(
            ElemIfaceType::Mixer,
            0,
            0,
            Self::AUTOLEVEL_RISE_TIME_NAME,
            0,
        );
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                T::AUTOLEVEL_RISE_TIME_MIN,
                T::AUTOLEVEL_RISE_TIME_MAX,
                T::AUTOLEVEL_RISE_TIME_STEP,
                Self::CH_COUNT,
                None,
                true,
            )
            .map(|mut list| elem_id_list.append(&mut list))?;

        Ok(())
    }

    fn read(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        let n = elem_id.name();

        if n == Self::AUTOLEVEL_ACTIVATE_NAME {
            let params = self.params().as_ref();
            let vals = params.activates.clone();
            elem_value.set_bool(&vals);
            Ok(true)
        } else if n == Self::AUTOLEVEL_MAX_GAIN_NAME {
            let params = self.params().as_ref();
            let vals: Vec<i32> = params.max_gains.iter().map(|&gain| gain as i32).collect();
            elem_value.set_int(&vals);
            Ok(true)
        } else if n == Self::AUTOLEVEL_HEAD_ROOM_NAME {
            let params = self.params().as_ref();
            let vals: Vec<i32> = params.headrooms.iter().map(|&gain| gain as i32).collect();
            elem_value.set_int(&vals);
            Ok(true)
        } else if n == Self::AUTOLEVEL_RISE_TIME_NAME {
            let params = self.params().as_ref();
            let vals: Vec<i32> = params.rise_times.iter().map(|&gain| gain as i32).collect();
            elem_value.set_int(&vals);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        let n = elem_id.name();

        if n == Self::AUTOLEVEL_ACTIVATE_NAME {
            let mut params = self.params().clone();
            params
                .as_mut()
                .activates
                .iter_mut()
                .zip(elem_value.boolean())
                .for_each(|(activate, val)| *activate = val);
            let res = T::command_partially(req, node, self.params_mut(), params, timeout_ms);
            debug!(params = ?self.params(), ?res);
            res.map(|_| true)
        } else if n == Self::AUTOLEVEL_MAX_GAIN_NAME {
            let mut params = self.params().clone();
            params
                .as_mut()
                .max_gains
                .iter_mut()
                .zip(elem_value.int())
                .for_each(|(gain, &val)| *gain = val as u16);
            let res = T::command_partially(req, node, self.params_mut(), params, timeout_ms);
            debug!(params = ?self.params(), ?res);
            res.map(|_| true)
        } else if n == Self::AUTOLEVEL_HEAD_ROOM_NAME {
            let mut params = self.params().clone();
            params
                .as_mut()
                .headrooms
                .iter_mut()
                .zip(elem_value.int())
                .for_each(|(headroom, &val)| *headroom = val as u16);
            let res = T::command_partially(req, node, self.params_mut(), params, timeout_ms);
            debug!(params = ?self.params(), ?res);
            res.map(|_| true)
        } else if n == Self::AUTOLEVEL_RISE_TIME_NAME {
            let mut params = self.params().clone();
            params
                .as_mut()
                .rise_times
                .iter_mut()
                .zip(elem_value.int())
                .for_each(|(time, &val)| *time = val as u16);
            let res = T::command_partially(req, node, self.params_mut(), params, timeout_ms);
            debug!(params = ?self.params(), ?res);
            res.map(|_| true)
        } else {
            Ok(false)
        }
    }
}

#[derive(Debug)]
pub struct LatterInputAutolevelCtl<T>
where
    T: RmeFfLatterInputSpecification
        + RmeFfLatterAutolevelSpecification
        + RmeFfWhollyCommandableParamsOperation<FfLatterInputAutolevelParameters>
        + RmeFfPartiallyCommandableParamsOperation<FfLatterInputAutolevelParameters>,
{
    pub elem_id_list: Vec<ElemId>,
    params: FfLatterInputAutolevelParameters,
    _phantom: PhantomData<T>,
}

impl<T> Default for LatterInputAutolevelCtl<T>
where
    T: RmeFfLatterInputSpecification
        + RmeFfLatterAutolevelSpecification
        + RmeFfWhollyCommandableParamsOperation<FfLatterInputAutolevelParameters>
        + RmeFfPartiallyCommandableParamsOperation<FfLatterInputAutolevelParameters>,
{
    fn default() -> Self {
        Self {
            elem_id_list: Default::default(),
            params: T::create_input_autolevel_parameters(),
            _phantom: Default::default(),
        }
    }
}

impl<T> FfLatterAutolevelCtlOperation<T, FfLatterInputAutolevelParameters>
    for LatterInputAutolevelCtl<T>
where
    T: RmeFfLatterInputSpecification
        + RmeFfLatterAutolevelSpecification
        + RmeFfWhollyCommandableParamsOperation<FfLatterInputAutolevelParameters>
        + RmeFfPartiallyCommandableParamsOperation<FfLatterInputAutolevelParameters>,
{
    const AUTOLEVEL_ACTIVATE_NAME: &'static str = "input:autolevel-activate";
    const AUTOLEVEL_MAX_GAIN_NAME: &'static str = "input:autolevel-max-gain";
    const AUTOLEVEL_HEAD_ROOM_NAME: &'static str = "input:autolevel-head-room";
    const AUTOLEVEL_RISE_TIME_NAME: &'static str = "input:autolevel-rise-time";

    fn params(&self) -> &FfLatterInputAutolevelParameters {
        &self.params
    }

    fn params_mut(&mut self) -> &mut FfLatterInputAutolevelParameters {
        &mut self.params
    }

    fn elem_id_list_mut(&mut self) -> &mut Vec<ElemId> {
        &mut self.elem_id_list
    }

    const CH_COUNT: usize = T::PHYS_INPUT_COUNT;
}

#[derive(Debug)]
pub struct LatterOutputAutolevelCtl<T>
where
    T: RmeFfLatterOutputSpecification
        + RmeFfLatterAutolevelSpecification
        + RmeFfWhollyCommandableParamsOperation<FfLatterOutputAutolevelParameters>
        + RmeFfPartiallyCommandableParamsOperation<FfLatterOutputAutolevelParameters>,
{
    pub elem_id_list: Vec<ElemId>,
    params: FfLatterOutputAutolevelParameters,
    _phantom: PhantomData<T>,
}

impl<T> Default for LatterOutputAutolevelCtl<T>
where
    T: RmeFfLatterOutputSpecification
        + RmeFfLatterAutolevelSpecification
        + RmeFfWhollyCommandableParamsOperation<FfLatterOutputAutolevelParameters>
        + RmeFfPartiallyCommandableParamsOperation<FfLatterOutputAutolevelParameters>,
{
    fn default() -> Self {
        Self {
            elem_id_list: Default::default(),
            params: T::create_output_autolevel_parameters(),
            _phantom: Default::default(),
        }
    }
}

impl<T> FfLatterAutolevelCtlOperation<T, FfLatterOutputAutolevelParameters>
    for LatterOutputAutolevelCtl<T>
where
    T: RmeFfLatterOutputSpecification
        + RmeFfLatterAutolevelSpecification
        + RmeFfWhollyCommandableParamsOperation<FfLatterOutputAutolevelParameters>
        + RmeFfPartiallyCommandableParamsOperation<FfLatterOutputAutolevelParameters>,
{
    const AUTOLEVEL_ACTIVATE_NAME: &'static str = "output:autolevel-activate";
    const AUTOLEVEL_MAX_GAIN_NAME: &'static str = "output:autolevel-max-gain";
    const AUTOLEVEL_HEAD_ROOM_NAME: &'static str = "output:autolevel-head-room";
    const AUTOLEVEL_RISE_TIME_NAME: &'static str = "output:autolevel-rise-time";

    fn params(&self) -> &FfLatterOutputAutolevelParameters {
        &self.params
    }

    fn params_mut(&mut self) -> &mut FfLatterOutputAutolevelParameters {
        &mut self.params
    }

    fn elem_id_list_mut(&mut self) -> &mut Vec<ElemId> {
        &mut self.elem_id_list
    }

    const CH_COUNT: usize = T::OUTPUT_COUNT;
}

const LINE_SRC_GAIN_NAME: &str = "fx:line-source-gain";
const MIC_SRC_GAIN_NAME: &str = "fx:mic-source-gain";
const SPDIF_SRC_GAIN_NAME: &str = "fx:spdif-source-gain";
const ADAT_SRC_GAIN_NAME: &str = "fx:adat-source-gain";
const STREAM_SRC_GAIN_NAME: &str = "fx:stream-source-gain";

#[derive(Debug)]
pub struct LatterFxSourceCtl<T>
where
    T: RmeFfLatterDspSpecification
        + RmeFfLatterFxSpecification
        + RmeFfWhollyCommandableParamsOperation<FfLatterFxSourceParameters>
        + RmeFfPartiallyCommandableParamsOperation<FfLatterFxSourceParameters>,
{
    pub elem_id_list: Vec<ElemId>,
    params: FfLatterFxSourceParameters,
    _phantom: PhantomData<T>,
}

impl<T> Default for LatterFxSourceCtl<T>
where
    T: RmeFfLatterDspSpecification
        + RmeFfLatterFxSpecification
        + RmeFfWhollyCommandableParamsOperation<FfLatterFxSourceParameters>
        + RmeFfPartiallyCommandableParamsOperation<FfLatterFxSourceParameters>,
{
    fn default() -> Self {
        Self {
            elem_id_list: Default::default(),
            params: T::create_fx_sources_parameters(),
            _phantom: Default::default(),
        }
    }
}

impl<T> LatterFxSourceCtl<T>
where
    T: RmeFfLatterDspSpecification
        + RmeFfLatterFxSpecification
        + RmeFfWhollyCommandableParamsOperation<FfLatterFxSourceParameters>
        + RmeFfPartiallyCommandableParamsOperation<FfLatterFxSourceParameters>,
{
    const PHYS_LEVEL_TLV: DbInterval = DbInterval {
        min: -6500,
        max: 000,
        linear: false,
        mute_avail: false,
    };
    const VIRT_LEVEL_TLV: DbInterval = DbInterval {
        min: -6500,
        max: 000,
        linear: false,
        mute_avail: false,
    };

    pub fn cache(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let res = T::command_wholly(req, node, &mut self.params, timeout_ms);
        debug!(params = ?self.params, ?res);
        res
    }

    pub fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        [
            (LINE_SRC_GAIN_NAME, T::LINE_INPUT_COUNT),
            (MIC_SRC_GAIN_NAME, T::MIC_INPUT_COUNT),
            (SPDIF_SRC_GAIN_NAME, T::SPDIF_INPUT_COUNT),
            (ADAT_SRC_GAIN_NAME, T::ADAT_INPUT_COUNT),
        ]
        .iter()
        .try_for_each(|&(name, count)| {
            let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
            card_cntr
                .add_int_elems(
                    &elem_id,
                    1,
                    T::FX_PHYS_LEVEL_MIN,
                    T::FX_PHYS_LEVEL_MAX,
                    T::FX_PHYS_LEVEL_STEP,
                    count,
                    Some(&Vec::<u32>::from(&Self::PHYS_LEVEL_TLV)),
                    true,
                )
                .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))
        })?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, STREAM_SRC_GAIN_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                T::FX_VIRT_LEVEL_MIN,
                T::FX_VIRT_LEVEL_MAX,
                T::FX_VIRT_LEVEL_STEP,
                T::STREAM_INPUT_COUNT,
                Some(&Vec::<u32>::from(&Self::VIRT_LEVEL_TLV)),
                true,
            )
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        Ok(())
    }

    pub fn read(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            LINE_SRC_GAIN_NAME => {
                let vals: Vec<i32> = self
                    .params
                    .line_input_gains
                    .iter()
                    .map(|&gain| gain as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            MIC_SRC_GAIN_NAME => {
                let vals: Vec<i32> = self
                    .params
                    .mic_input_gains
                    .iter()
                    .map(|&gain| gain as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            SPDIF_SRC_GAIN_NAME => {
                let vals: Vec<i32> = self
                    .params
                    .spdif_input_gains
                    .iter()
                    .map(|&gain| gain as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            ADAT_SRC_GAIN_NAME => {
                let vals: Vec<i32> = self
                    .params
                    .adat_input_gains
                    .iter()
                    .map(|&gain| gain as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            STREAM_SRC_GAIN_NAME => {
                let vals: Vec<i32> = self
                    .params
                    .stream_input_gains
                    .iter()
                    .map(|&gain| gain as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub fn write(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            LINE_SRC_GAIN_NAME => {
                let mut params = self.params.clone();
                params
                    .line_input_gains
                    .iter_mut()
                    .zip(elem_value.int())
                    .for_each(|(d, s)| *d = *s as i16);
                let res = T::command_partially(req, node, &mut self.params, params, timeout_ms);
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            MIC_SRC_GAIN_NAME => {
                let mut params = self.params.clone();
                params
                    .mic_input_gains
                    .iter_mut()
                    .zip(elem_value.int())
                    .for_each(|(d, s)| *d = *s as i16);
                let res = T::command_partially(req, node, &mut self.params, params, timeout_ms);
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            SPDIF_SRC_GAIN_NAME => {
                let mut params = self.params.clone();
                params
                    .spdif_input_gains
                    .iter_mut()
                    .zip(elem_value.int())
                    .for_each(|(d, s)| *d = *s as i16);
                let res = T::command_partially(req, node, &mut self.params, params, timeout_ms);
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            ADAT_SRC_GAIN_NAME => {
                let mut params = self.params.clone();
                params
                    .adat_input_gains
                    .iter_mut()
                    .zip(elem_value.int())
                    .for_each(|(d, s)| *d = *s as i16);
                let res = T::command_partially(req, node, &mut self.params, params, timeout_ms);
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            STREAM_SRC_GAIN_NAME => {
                let mut params = self.params.clone();
                params
                    .stream_input_gains
                    .iter_mut()
                    .zip(elem_value.int())
                    .for_each(|(d, s)| *d = *s as u16);
                let res = T::command_partially(req, node, &mut self.params, params, timeout_ms);
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

const LINE_OUT_VOL_NAME: &str = "fx:line-output-volume";
const HP_OUT_VOL_NAME: &str = "fx:hp-output-volume";
const SPDIF_OUT_VOL_NAME: &str = "fx:spdif-output-volume";
const ADAT_OUT_VOL_NAME: &str = "fx:adat-output-volume";

#[derive(Debug)]
pub struct LatterFxOutputCtl<T>
where
    T: RmeFfLatterDspSpecification
        + RmeFfLatterFxSpecification
        + RmeFfWhollyCommandableParamsOperation<FfLatterFxOutputParameters>
        + RmeFfPartiallyCommandableParamsOperation<FfLatterFxOutputParameters>,
{
    pub elem_id_list: Vec<ElemId>,
    params: FfLatterFxOutputParameters,
    _phantom: PhantomData<T>,
}

impl<T> Default for LatterFxOutputCtl<T>
where
    T: RmeFfLatterDspSpecification
        + RmeFfLatterFxSpecification
        + RmeFfWhollyCommandableParamsOperation<FfLatterFxOutputParameters>
        + RmeFfPartiallyCommandableParamsOperation<FfLatterFxOutputParameters>,
{
    fn default() -> Self {
        Self {
            elem_id_list: Default::default(),
            params: T::create_fx_output_parameters(),
            _phantom: Default::default(),
        }
    }
}

impl<T> LatterFxOutputCtl<T>
where
    T: RmeFfLatterDspSpecification
        + RmeFfLatterFxSpecification
        + RmeFfWhollyCommandableParamsOperation<FfLatterFxOutputParameters>
        + RmeFfPartiallyCommandableParamsOperation<FfLatterFxOutputParameters>,
{
    const PHYS_LEVEL_TLV: DbInterval = DbInterval {
        min: -6500,
        max: 000,
        linear: false,
        mute_avail: false,
    };

    pub fn cache(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let res = T::command_wholly(req, node, &mut self.params, timeout_ms);
        debug!(params = ?self.params, ?res);
        res
    }

    pub fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        [
            (LINE_OUT_VOL_NAME, T::LINE_OUTPUT_COUNT),
            (HP_OUT_VOL_NAME, T::HP_OUTPUT_COUNT),
            (SPDIF_OUT_VOL_NAME, T::SPDIF_OUTPUT_COUNT),
            (ADAT_OUT_VOL_NAME, T::ADAT_OUTPUT_COUNT),
        ]
        .iter()
        .try_for_each(|&(name, count)| {
            let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
            card_cntr
                .add_int_elems(
                    &elem_id,
                    1,
                    T::FX_PHYS_LEVEL_MIN,
                    T::FX_PHYS_LEVEL_MAX,
                    T::FX_PHYS_LEVEL_STEP,
                    count,
                    Some(&Vec::<u32>::from(&Self::PHYS_LEVEL_TLV)),
                    true,
                )
                .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))
        })
    }

    pub fn read(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            LINE_OUT_VOL_NAME => {
                let vals: Vec<i32> = self
                    .params
                    .line_output_vols
                    .iter()
                    .map(|&vol| vol as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            HP_OUT_VOL_NAME => {
                let vals: Vec<i32> = self
                    .params
                    .hp_output_vols
                    .iter()
                    .map(|&vol| vol as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            SPDIF_OUT_VOL_NAME => {
                let vals: Vec<i32> = self
                    .params
                    .spdif_output_vols
                    .iter()
                    .map(|&vol| vol as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            ADAT_OUT_VOL_NAME => {
                let vals: Vec<i32> = self
                    .params
                    .adat_output_vols
                    .iter()
                    .map(|&vol| vol as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub fn write(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            LINE_OUT_VOL_NAME => {
                let mut params = self.params.clone();
                params
                    .line_output_vols
                    .iter_mut()
                    .zip(elem_value.int())
                    .for_each(|(d, s)| *d = *s as i16);
                let res = T::command_partially(req, node, &mut self.params, params, timeout_ms);
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            HP_OUT_VOL_NAME => {
                let mut params = self.params.clone();
                params
                    .hp_output_vols
                    .iter_mut()
                    .zip(elem_value.int())
                    .for_each(|(d, s)| *d = *s as i16);
                let res = T::command_partially(req, node, &mut self.params, params, timeout_ms);
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            SPDIF_OUT_VOL_NAME => {
                let mut params = self.params.clone();
                params
                    .spdif_output_vols
                    .iter_mut()
                    .zip(elem_value.int())
                    .for_each(|(d, s)| *d = *s as i16);
                let res = T::command_partially(req, node, &mut self.params, params, timeout_ms);
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            ADAT_OUT_VOL_NAME => {
                let mut params = self.params.clone();
                params
                    .adat_output_vols
                    .iter_mut()
                    .zip(elem_value.int())
                    .for_each(|(d, s)| *d = *s as i16);
                let res = T::command_partially(req, node, &mut self.params, params, timeout_ms);
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

const REVERB_ACTIVATE_NAME: &str = "fx:reverb-activate";
const REVERB_TYPE_NAME: &str = "fx:reverb-type";
const REVERB_PRE_DELAY_NAME: &str = "fx:reverb-pre-delay";
const REVERB_PRE_HPF_FREQ_NAME: &str = "fx:reverb-pre-hpf-freq";
const REVERB_ROOM_SCALE_NAME: &str = "fx:reverb-room-scale";
const REVERB_ATTACK_NAME: &str = "fx:reverb-attack";
const REVERB_HOLD_NAME: &str = "fx:reverb-hold";
const REVERB_RELEASE_NAME: &str = "fx:reverb-release";
const REVERB_POST_LPF_FREQ_NAME: &str = "fx:reverb-post-lpf-freq";
const REVERB_TIME_NAME: &str = "fx:reverb-time";
const REVERB_DAMPING_NAME: &str = "fx:reverb-damping";
const REVERB_SMOOTH_NAME: &str = "fx:reverb-smooth";
const REVERB_VOL_NAME: &str = "fx:reverb-volume";
const REVERB_STEREO_WIDTH_NAME: &str = "fx:reverb-stereo-width";

#[derive(Default, Debug)]
pub struct LatterFxReverbCtl<T>
where
    T: RmeFfLatterFxSpecification
        + RmeFfWhollyCommandableParamsOperation<FfLatterFxReverbState>
        + RmeFfPartiallyCommandableParamsOperation<FfLatterFxReverbState>,
{
    pub elem_id_list: Vec<ElemId>,
    params: FfLatterFxReverbState,
    _phantom: PhantomData<T>,
}

fn fx_reverb_type_to_str(reverb_type: &FfLatterFxReverbType) -> &str {
    match reverb_type {
        FfLatterFxReverbType::SmallRoom => "Small-room",
        FfLatterFxReverbType::MediumRoom => "Medium-room",
        FfLatterFxReverbType::LargeRoom => "Large-room",
        FfLatterFxReverbType::Walls => "Walls",
        FfLatterFxReverbType::Shorty => "Shorty",
        FfLatterFxReverbType::Attack => "Attack",
        FfLatterFxReverbType::Swagger => "Swagger",
        FfLatterFxReverbType::OldSchool => "Old-school",
        FfLatterFxReverbType::Echoistic => "Echoistic",
        FfLatterFxReverbType::EightPlusNine => "8-plus-9",
        FfLatterFxReverbType::GrandWide => "Grand-wide",
        FfLatterFxReverbType::Thicker => "Thicker",
        FfLatterFxReverbType::Envelope => "Envelope",
        FfLatterFxReverbType::Gated => "Gated",
        FfLatterFxReverbType::Space => "Space",
    }
}

impl<T> LatterFxReverbCtl<T>
where
    T: RmeFfLatterFxSpecification
        + RmeFfWhollyCommandableParamsOperation<FfLatterFxReverbState>
        + RmeFfPartiallyCommandableParamsOperation<FfLatterFxReverbState>,
{
    const REVERB_TYPES: &'static [FfLatterFxReverbType] = &[
        FfLatterFxReverbType::SmallRoom,
        FfLatterFxReverbType::MediumRoom,
        FfLatterFxReverbType::LargeRoom,
        FfLatterFxReverbType::Walls,
        FfLatterFxReverbType::Shorty,
        FfLatterFxReverbType::Attack,
        FfLatterFxReverbType::Swagger,
        FfLatterFxReverbType::OldSchool,
        FfLatterFxReverbType::Echoistic,
        FfLatterFxReverbType::EightPlusNine,
        FfLatterFxReverbType::GrandWide,
        FfLatterFxReverbType::Thicker,
        FfLatterFxReverbType::Envelope,
        FfLatterFxReverbType::Gated,
        FfLatterFxReverbType::Space,
    ];

    pub fn cache(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let res = T::command_wholly(req, node, &mut self.params, timeout_ms);
        debug!(params = ?self.params, ?res);
        res
    }

    pub fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, REVERB_ACTIVATE_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, 1, 1, true)
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let labels: Vec<&str> = Self::REVERB_TYPES
            .iter()
            .map(|t| fx_reverb_type_to_str(t))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, REVERB_TYPE_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, true)
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, REVERB_PRE_DELAY_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                T::REVERB_PRE_DELAY_MIN,
                T::REVERB_PRE_DELAY_MAX,
                T::REVERB_PRE_DELAY_STEP,
                1,
                None,
                true,
            )
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, REVERB_PRE_HPF_FREQ_NAME, 0);
        card_cntr
            .add_int_elems(&elem_id, 1, 20, 500, 1, 1, None, true)
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, REVERB_ROOM_SCALE_NAME, 0);
        card_cntr
            .add_int_elems(&elem_id, 1, 50, 300, 1, 1, None, true)
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, REVERB_ATTACK_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                T::REVERB_ATTACK_MIN,
                T::REVERB_ATTACK_MAX,
                T::REVERB_ATTACK_STEP,
                1,
                None,
                true,
            )
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, REVERB_HOLD_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                T::REVERB_HOLD_MIN,
                T::REVERB_HOLD_MAX,
                T::REVERB_HOLD_STEP,
                1,
                None,
                true,
            )
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, REVERB_RELEASE_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                T::REVERB_RELEASE_MIN,
                T::REVERB_RELEASE_MAX,
                T::REVERB_RELEASE_STEP,
                1,
                None,
                true,
            )
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, REVERB_POST_LPF_FREQ_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                T::REVERB_POST_LPF_FREQ_MIN,
                T::REVERB_POST_LPF_FREQ_MAX,
                T::REVERB_POST_LPF_FREQ_STEP,
                1,
                None,
                true,
            )
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, REVERB_TIME_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                T::REVERB_TIME_MIN,
                T::REVERB_TIME_MAX,
                T::REVERB_TIME_STEP,
                1,
                None,
                true,
            )
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, REVERB_DAMPING_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                T::REVERB_DAMPING_MIN,
                T::REVERB_DAMPING_MAX,
                T::REVERB_DAMPING_STEP,
                1,
                None,
                true,
            )
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, REVERB_SMOOTH_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                T::REVERB_SMOOTH_MIN,
                T::REVERB_SMOOTH_MAX,
                T::REVERB_SMOOTH_STEP,
                1,
                None,
                true,
            )
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, REVERB_VOL_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                T::REVERB_VOL_MIN,
                T::REVERB_VOL_MAX,
                T::REVERB_VOL_STEP,
                1,
                None,
                true,
            )
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, REVERB_STEREO_WIDTH_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                T::REVERB_STEREO_WIDTH_MIN,
                T::REVERB_STEREO_WIDTH_MAX,
                T::REVERB_STEREO_WIDTH_STEP,
                1,
                None,
                true,
            )
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        Ok(())
    }

    pub fn read(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            REVERB_ACTIVATE_NAME => {
                elem_value.set_bool(&[self.params.activate]);
                Ok(true)
            }
            REVERB_TYPE_NAME => {
                let val = Self::REVERB_TYPES
                    .iter()
                    .position(|t| self.params.reverb_type.eq(t))
                    .unwrap();
                elem_value.set_enum(&[val as u32]);
                Ok(true)
            }
            REVERB_PRE_DELAY_NAME => {
                elem_value.set_int(&[self.params.pre_delay as i32]);
                Ok(true)
            }
            REVERB_PRE_HPF_FREQ_NAME => {
                elem_value.set_int(&[self.params.pre_hpf as i32]);
                Ok(true)
            }
            REVERB_ROOM_SCALE_NAME => {
                elem_value.set_int(&[self.params.room_scale as i32]);
                Ok(true)
            }
            REVERB_ATTACK_NAME => {
                elem_value.set_int(&[self.params.attack as i32]);
                Ok(true)
            }
            REVERB_HOLD_NAME => {
                elem_value.set_int(&[self.params.hold as i32]);
                Ok(true)
            }
            REVERB_RELEASE_NAME => {
                elem_value.set_int(&[self.params.release as i32]);
                Ok(true)
            }
            REVERB_POST_LPF_FREQ_NAME => {
                elem_value.set_int(&[self.params.post_lpf as i32]);
                Ok(true)
            }
            REVERB_TIME_NAME => {
                elem_value.set_int(&[self.params.time as i32]);
                Ok(true)
            }
            REVERB_DAMPING_NAME => {
                elem_value.set_int(&[self.params.damping as i32]);
                Ok(true)
            }
            REVERB_SMOOTH_NAME => {
                elem_value.set_int(&[self.params.smooth as i32]);
                Ok(true)
            }
            REVERB_VOL_NAME => {
                elem_value.set_int(&[self.params.volume as i32]);
                Ok(true)
            }
            REVERB_STEREO_WIDTH_NAME => {
                elem_value.set_int(&[self.params.stereo_width as i32]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub fn write(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            REVERB_ACTIVATE_NAME => {
                let mut params = self.params.clone();
                params.activate = elem_value.boolean()[0];
                let res = T::command_partially(req, node, &mut self.params, params, timeout_ms);
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            REVERB_TYPE_NAME => {
                let mut params = self.params.clone();
                let pos = elem_value.enumerated()[0] as usize;
                Self::REVERB_TYPES
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| {
                        let msg = format!("Invalid index of type of reverb effect: {}", pos);
                        Error::new(FileError::Inval, &msg)
                    })
                    .map(|&t| params.reverb_type = t)?;
                let res = T::command_partially(req, node, &mut self.params, params, timeout_ms);
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            REVERB_PRE_DELAY_NAME => {
                let mut params = self.params.clone();
                params.pre_delay = elem_value.int()[0] as u16;
                let res = T::command_partially(req, node, &mut self.params, params, timeout_ms);
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            REVERB_PRE_HPF_FREQ_NAME => {
                let mut params = self.params.clone();
                params.pre_hpf = elem_value.int()[0] as u16;
                let res = T::command_partially(req, node, &mut self.params, params, timeout_ms);
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            REVERB_ROOM_SCALE_NAME => {
                let mut params = self.params.clone();
                params.room_scale = elem_value.int()[0] as u16;
                let res = T::command_partially(req, node, &mut self.params, params, timeout_ms);
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            REVERB_ATTACK_NAME => {
                let mut params = self.params.clone();
                params.attack = elem_value.int()[0] as u16;
                let res = T::command_partially(req, node, &mut self.params, params, timeout_ms);
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            REVERB_HOLD_NAME => {
                let mut params = self.params.clone();
                params.hold = elem_value.int()[0] as u16;
                let res = T::command_partially(req, node, &mut self.params, params, timeout_ms);
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            REVERB_RELEASE_NAME => {
                let mut params = self.params.clone();
                params.release = elem_value.int()[0] as u16;
                let res = T::command_partially(req, node, &mut self.params, params, timeout_ms);
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            REVERB_POST_LPF_FREQ_NAME => {
                let mut params = self.params.clone();
                params.post_lpf = elem_value.int()[0] as u16;
                let res = T::command_partially(req, node, &mut self.params, params, timeout_ms);
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            REVERB_TIME_NAME => {
                let mut params = self.params.clone();
                params.time = elem_value.int()[0] as u16;
                let res = T::command_partially(req, node, &mut self.params, params, timeout_ms);
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            REVERB_DAMPING_NAME => {
                let mut params = self.params.clone();
                params.damping = elem_value.int()[0] as u16;
                let res = T::command_partially(req, node, &mut self.params, params, timeout_ms);
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            REVERB_SMOOTH_NAME => {
                let mut params = self.params.clone();
                params.smooth = elem_value.int()[0] as u16;
                let res = T::command_partially(req, node, &mut self.params, params, timeout_ms);
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            REVERB_VOL_NAME => {
                let mut params = self.params.clone();
                params.volume = elem_value.int()[0] as i16;
                let res = T::command_partially(req, node, &mut self.params, params, timeout_ms);
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            REVERB_STEREO_WIDTH_NAME => {
                let mut params = self.params.clone();
                params.stereo_width = elem_value.int()[0] as u16;
                let res = T::command_partially(req, node, &mut self.params, params, timeout_ms);
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

const ECHO_ACTIVATE_NAME: &str = "fx:echo-activate";
const ECHO_TYPE_NAME: &str = "fx:echo-type";
const ECHO_DELAY_NAME: &str = "fx:echo-delay";
const ECHO_FEEDBACK_NAME: &str = "fx:echo-feedback";
const ECHO_LPF_FREQ_NAME: &str = "fx:echo-lpf-freq";
const ECHO_VOL_NAME: &str = "fx:echo-volume";
const ECHO_STEREO_WIDTH_NAME: &str = "fx:echo-stereo-width";

#[derive(Default, Debug)]
pub struct LatterFxEchoCtl<T>
where
    T: RmeFfLatterFxSpecification
        + RmeFfWhollyCommandableParamsOperation<FfLatterFxEchoState>
        + RmeFfPartiallyCommandableParamsOperation<FfLatterFxEchoState>,
{
    pub elem_id_list: Vec<ElemId>,
    params: FfLatterFxEchoState,
    _phantom: PhantomData<T>,
}

fn fx_echo_type_to_str(echo_type: &FfLatterFxEchoType) -> &str {
    match echo_type {
        FfLatterFxEchoType::StereoEcho => "Stereo-echo",
        FfLatterFxEchoType::StereoCross => "Stereo-cross",
        FfLatterFxEchoType::PongEcho => "Pong-echo",
    }
}

fn fx_echo_lpf_freq_to_str(lpf_freq: &FfLatterFxEchoLpfFreq) -> &str {
    match lpf_freq {
        FfLatterFxEchoLpfFreq::Off => "Off",
        FfLatterFxEchoLpfFreq::H2000 => "2kHz",
        FfLatterFxEchoLpfFreq::H4000 => "4kHz",
        FfLatterFxEchoLpfFreq::H8000 => "8kHz",
        FfLatterFxEchoLpfFreq::H12000 => "12kHz",
        FfLatterFxEchoLpfFreq::H16000 => "16kHz",
    }
}

impl<T> LatterFxEchoCtl<T>
where
    T: RmeFfLatterFxSpecification
        + RmeFfWhollyCommandableParamsOperation<FfLatterFxEchoState>
        + RmeFfPartiallyCommandableParamsOperation<FfLatterFxEchoState>,
{
    const ECHO_TYPES: &'static [FfLatterFxEchoType] = &[
        FfLatterFxEchoType::StereoEcho,
        FfLatterFxEchoType::StereoCross,
        FfLatterFxEchoType::PongEcho,
    ];

    const ECHO_LPF_FREQS: &'static [FfLatterFxEchoLpfFreq] = &[
        FfLatterFxEchoLpfFreq::Off,
        FfLatterFxEchoLpfFreq::H2000,
        FfLatterFxEchoLpfFreq::H4000,
        FfLatterFxEchoLpfFreq::H8000,
        FfLatterFxEchoLpfFreq::H12000,
        FfLatterFxEchoLpfFreq::H16000,
    ];

    pub fn cache(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let res = T::command_wholly(req, node, &mut self.params, timeout_ms);
        debug!(params = ?self.params, ?res);
        res
    }

    pub fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, ECHO_ACTIVATE_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, 1, 1, true)
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let labels: Vec<&str> = Self::ECHO_TYPES
            .iter()
            .map(|t| fx_echo_type_to_str(t))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, ECHO_TYPE_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, true)
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, ECHO_DELAY_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                T::ECHO_DELAY_MIN,
                T::ECHO_DELAY_MAX,
                T::ECHO_DELAY_STEP,
                1,
                None,
                true,
            )
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, ECHO_FEEDBACK_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                T::ECHO_FEEDBACK_MIN,
                T::ECHO_FEEDBACK_MAX,
                T::ECHO_FEEDBACK_STEP,
                1,
                None,
                true,
            )
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let labels: Vec<&str> = Self::ECHO_LPF_FREQS
            .iter()
            .map(|t| fx_echo_lpf_freq_to_str(t))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, ECHO_LPF_FREQ_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, true)
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, ECHO_VOL_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                T::ECHO_VOL_MIN,
                T::ECHO_VOL_MAX,
                T::ECHO_VOL_STEP,
                1,
                None,
                true,
            )
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, ECHO_STEREO_WIDTH_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                T::ECHO_STEREO_WIDTH_MIN,
                T::ECHO_STEREO_WIDTH_MAX,
                T::ECHO_STEREO_WIDTH_STEP,
                1,
                None,
                true,
            )
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        Ok(())
    }

    pub fn read(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            ECHO_ACTIVATE_NAME => {
                elem_value.set_bool(&[self.params.activate]);
                Ok(true)
            }
            ECHO_TYPE_NAME => {
                let pos = Self::ECHO_TYPES
                    .iter()
                    .position(|t| self.params.echo_type.eq(t))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            ECHO_DELAY_NAME => {
                elem_value.set_int(&[self.params.delay as i32]);
                Ok(true)
            }
            ECHO_FEEDBACK_NAME => {
                elem_value.set_int(&[self.params.feedback as i32]);
                Ok(true)
            }
            ECHO_LPF_FREQ_NAME => {
                let pos = Self::ECHO_LPF_FREQS
                    .iter()
                    .position(|f| self.params.lpf.eq(f))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            ECHO_VOL_NAME => {
                elem_value.set_int(&[self.params.volume as i32]);
                Ok(true)
            }
            ECHO_STEREO_WIDTH_NAME => {
                elem_value.set_int(&[self.params.stereo_width as i32]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub fn write(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            ECHO_ACTIVATE_NAME => {
                let mut params = self.params.clone();
                params.activate = elem_value.boolean()[0];
                let res = T::command_partially(req, node, &mut self.params, params, timeout_ms);
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            ECHO_TYPE_NAME => {
                let mut params = self.params.clone();
                let pos = elem_value.enumerated()[0] as usize;
                Self::ECHO_TYPES
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| {
                        let msg = format!("Invalid index of type of echo effect: {}", pos);
                        Error::new(FileError::Inval, &msg)
                    })
                    .map(|&t| params.echo_type = t)?;
                let res = T::command_partially(req, node, &mut self.params, params, timeout_ms);
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            ECHO_DELAY_NAME => {
                let mut params = self.params.clone();
                params.delay = elem_value.int()[0] as u16;
                let res = T::command_partially(req, node, &mut self.params, params, timeout_ms);
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            ECHO_FEEDBACK_NAME => {
                let mut params = self.params.clone();
                params.feedback = elem_value.int()[0] as u16;
                let res = T::command_partially(req, node, &mut self.params, params, timeout_ms);
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            ECHO_LPF_FREQ_NAME => {
                let mut params = self.params.clone();
                let pos = elem_value.enumerated()[0] as usize;
                Self::ECHO_LPF_FREQS
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| {
                        let msg = format!("Invalid index of type of echo HPF frequency: {}", pos);
                        Error::new(FileError::Inval, &msg)
                    })
                    .map(|&t| params.lpf = t)?;
                let res = T::command_partially(req, node, &mut self.params, params, timeout_ms);
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            ECHO_VOL_NAME => {
                let mut params = self.params.clone();
                params.volume = elem_value.int()[0] as i16;
                let res = T::command_partially(req, node, &mut self.params, params, timeout_ms);
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            ECHO_STEREO_WIDTH_NAME => {
                let mut params = self.params.clone();
                params.stereo_width = elem_value.int()[0] as u16;
                let res = T::command_partially(req, node, &mut self.params, params, timeout_ms);
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            _ => Ok(false),
        }
    }
}
