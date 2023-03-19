// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

use {super::*, alsa_ctl_tlv_codec::DbInterval, std::marker::PhantomData};

const VOL_NAME: &str = "output-volume";

#[derive(Debug)]
pub struct FormerOutputCtl<T>(pub Vec<ElemId>, pub FormerOutputVolumeState, PhantomData<T>)
where
    T: RmeFormerOutputSpecification
        + RmeFfWhollyUpdatableParamsOperation<FormerOutputVolumeState>
        + RmeFfPartiallyUpdatableParamsOperation<FormerOutputVolumeState>;

impl<T> Default for FormerOutputCtl<T>
where
    T: RmeFormerOutputSpecification
        + RmeFfWhollyUpdatableParamsOperation<FormerOutputVolumeState>
        + RmeFfPartiallyUpdatableParamsOperation<FormerOutputVolumeState>,
{
    fn default() -> Self {
        let mut state = T::create_output_volume_state();
        state.0.iter_mut().for_each(|vol| *vol = T::VOL_ZERO);
        Self(Default::default(), state, Default::default())
    }
}

impl<T> FormerOutputCtl<T>
where
    T: RmeFormerOutputSpecification
        + RmeFfWhollyUpdatableParamsOperation<FormerOutputVolumeState>
        + RmeFfPartiallyUpdatableParamsOperation<FormerOutputVolumeState>,
{
    const VOL_TLV: DbInterval = DbInterval {
        min: -9000,
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
        let res = T::update_wholly(req, node, &mut self.1, timeout_ms);
        debug!(params = ?self.1, ?res);
        res
    }

    pub fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, VOL_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                T::VOL_MIN,
                T::VOL_MAX,
                T::VOL_STEP,
                T::PHYS_OUTPUT_COUNT,
                Some(&Vec::<u32>::from(&Self::VOL_TLV)),
                true,
            )
            .map(|mut elem_id_list| self.0.append(&mut elem_id_list))
    }

    pub fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            VOL_NAME => {
                let params = self.1.clone();
                elem_value.set_int(&params.0);
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
                let mut params = self.1.clone();
                params
                    .0
                    .iter_mut()
                    .zip(elem_value.int())
                    .for_each(|(vol, &val)| *vol = val);
                let res = T::update_partially(req, node, &mut self.1, params, timeout_ms);
                debug!(params = ?self.1, ?res);
                res.map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

const ANALOG_SRC_GAIN_NAME: &str = "mixer:analog-source-gain";
const SPDIF_SRC_GAIN_NAME: &str = "mixer:spdif-source-gain";
const ADAT_SRC_GAIN_NAME: &str = "mixer:adat-source-gain";
const STREAM_SRC_GAIN_NAME: &str = "mixer:stream-source-gain";

#[derive(Debug)]
pub struct FormerMixerCtl<T>(FormerMixerState, PhantomData<T>)
where
    T: RmeFormerMixerSpecification
        + RmeFfWhollyUpdatableParamsOperation<FormerMixerState>
        + RmeFfPartiallyUpdatableParamsOperation<FormerMixerState>;

impl<T: RmeFormerMixerSpecification> Default for FormerMixerCtl<T> {
    fn default() -> Self {
        let mut state = T::create_mixer_state();

        state.0.iter_mut().enumerate().for_each(|(i, mixer)| {
            mixer
                .analog_gains
                .iter_mut()
                .for_each(|gain| *gain = T::GAIN_MIN);
            mixer
                .spdif_gains
                .iter_mut()
                .for_each(|gain| *gain = T::GAIN_MIN);
            mixer
                .adat_gains
                .iter_mut()
                .for_each(|gain| *gain = T::GAIN_MIN);
            mixer
                .stream_gains
                .iter_mut()
                .nth(i)
                .map(|gain| *gain = T::GAIN_ZERO);
        });

        Self(state, Default::default())
    }
}

impl<T> FormerMixerCtl<T>
where
    T: RmeFormerMixerSpecification
        + RmeFfWhollyUpdatableParamsOperation<FormerMixerState>
        + RmeFfPartiallyUpdatableParamsOperation<FormerMixerState>,
{
    const GAIN_TLV: DbInterval = DbInterval {
        min: -9000,
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
        let res = T::update_wholly(req, node, &mut self.0, timeout_ms);
        debug!(params = ?self.0, ?res);
        res
    }

    pub fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        [
            (ANALOG_SRC_GAIN_NAME, T::ANALOG_INPUT_COUNT),
            (SPDIF_SRC_GAIN_NAME, T::SPDIF_INPUT_COUNT),
            (ADAT_SRC_GAIN_NAME, T::ADAT_INPUT_COUNT),
            (STREAM_SRC_GAIN_NAME, T::STREAM_INPUT_COUNT),
        ]
        .iter()
        .try_for_each(|&(name, input_count)| {
            let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
            card_cntr
                .add_int_elems(
                    &elem_id,
                    T::DST_COUNT,
                    T::GAIN_MIN,
                    T::GAIN_MAX,
                    T::GAIN_STEP,
                    input_count,
                    Some(&Vec::<u32>::from(&Self::GAIN_TLV)),
                    true,
                )
                .map(|_| ())
        })
    }

    pub fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            ANALOG_SRC_GAIN_NAME => {
                let index = elem_id.index() as usize;
                let params = &self.0;
                elem_value.set_int(&params.0[index].analog_gains);
                Ok(true)
            }
            SPDIF_SRC_GAIN_NAME => {
                let index = elem_id.index() as usize;
                let params = &self.0;
                elem_value.set_int(&params.0[index].spdif_gains);
                Ok(true)
            }
            ADAT_SRC_GAIN_NAME => {
                let index = elem_id.index() as usize;
                let params = &self.0;
                elem_value.set_int(&params.0[index].adat_gains);
                Ok(true)
            }
            STREAM_SRC_GAIN_NAME => {
                let index = elem_id.index() as usize;
                let params = &self.0;
                elem_value.set_int(&params.0[index].stream_gains);
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
            ANALOG_SRC_GAIN_NAME => {
                let index = elem_id.index() as usize;
                let mut params = self.0.clone();
                let mixer = params.0.iter_mut().nth(index).ok_or_else(|| {
                    let msg = format!("Invalid index {} for mixers", index);
                    Error::new(FileError::Inval, &msg)
                })?;
                mixer
                    .analog_gains
                    .iter_mut()
                    .zip(elem_value.int())
                    .for_each(|(gain, &val)| *gain = val);
                let res = T::update_partially(req, node, &mut self.0, params, timeout_ms);
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            SPDIF_SRC_GAIN_NAME => {
                let index = elem_id.index() as usize;
                let mut params = self.0.clone();
                let mixer = params.0.iter_mut().nth(index).ok_or_else(|| {
                    let msg = format!("Invalid index {} for mixers", index);
                    Error::new(FileError::Inval, &msg)
                })?;
                mixer
                    .spdif_gains
                    .iter_mut()
                    .zip(elem_value.int())
                    .for_each(|(gain, &val)| *gain = val);
                let res = T::update_partially(req, node, &mut self.0, params, timeout_ms);
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            ADAT_SRC_GAIN_NAME => {
                let index = elem_id.index() as usize;
                let mut params = self.0.clone();
                let mixer = params.0.iter_mut().nth(index).ok_or_else(|| {
                    let msg = format!("Invalid index {} for mixers", index);
                    Error::new(FileError::Inval, &msg)
                })?;
                mixer
                    .adat_gains
                    .iter_mut()
                    .zip(elem_value.int())
                    .for_each(|(gain, &val)| *gain = val);
                let res = T::update_partially(req, node, &mut self.0, params, timeout_ms);
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            STREAM_SRC_GAIN_NAME => {
                let index = elem_id.index() as usize;
                let mut params = self.0.clone();
                let mixer = params.0.iter_mut().nth(index).ok_or_else(|| {
                    let msg = format!("Invalid index {} for mixers", index);
                    Error::new(FileError::Inval, &msg)
                })?;
                mixer
                    .stream_gains
                    .iter_mut()
                    .zip(elem_value.int())
                    .for_each(|(gain, &val)| *gain = val);
                let res = T::update_partially(req, node, &mut self.0, params, timeout_ms);
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

const ANALOG_INPUT_NAME: &str = "meter:analog-input";
const SPDIF_INPUT_NAME: &str = "meter:spdif-input";
const ADAT_INPUT_NAME: &str = "meter:adat-input";
const STREAM_INPUT_NAME: &str = "meter:stream-input";

const ANALOG_OUTPUT_NAME: &str = "meter:analog-output";
const SPDIF_OUTPUT_NAME: &str = "meter:spdif-output";
const ADAT_OUTPUT_NAME: &str = "meter:adat-output";

#[derive(Debug)]
pub struct FormerMeterCtl<T>(pub Vec<ElemId>, FormerMeterState, PhantomData<T>)
where
    T: RmeFfFormerMeterSpecification + RmeFfCacheableParamsOperation<FormerMeterState>;

impl<T: RmeFfFormerMeterSpecification> Default for FormerMeterCtl<T> {
    fn default() -> Self {
        Self(
            Default::default(),
            T::create_meter_state(),
            Default::default(),
        )
    }
}

impl<T> FormerMeterCtl<T>
where
    T: RmeFfFormerMeterSpecification + RmeFfCacheableParamsOperation<FormerMeterState>,
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
            (ANALOG_INPUT_NAME, T::ANALOG_INPUT_COUNT),
            (SPDIF_INPUT_NAME, T::SPDIF_INPUT_COUNT),
            (ADAT_INPUT_NAME, T::ADAT_INPUT_COUNT),
            (STREAM_INPUT_NAME, T::STREAM_INPUT_COUNT),
            (ANALOG_OUTPUT_NAME, T::ANALOG_OUTPUT_COUNT),
            (SPDIF_OUTPUT_NAME, T::SPDIF_OUTPUT_COUNT),
            (ADAT_OUTPUT_NAME, T::ADAT_OUTPUT_COUNT),
        ]
        .iter()
        .try_for_each(|&(name, count)| {
            let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
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

    pub fn read(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            ANALOG_INPUT_NAME => {
                elem_value.set_int(&self.1.analog_inputs);
                Ok(true)
            }
            SPDIF_INPUT_NAME => {
                elem_value.set_int(&self.1.spdif_inputs);
                Ok(true)
            }
            ADAT_INPUT_NAME => {
                elem_value.set_int(&self.1.adat_inputs);
                Ok(true)
            }
            STREAM_INPUT_NAME => {
                elem_value.set_int(&self.1.stream_inputs);
                Ok(true)
            }
            ANALOG_OUTPUT_NAME => {
                elem_value.set_int(&self.1.analog_outputs);
                Ok(true)
            }
            SPDIF_OUTPUT_NAME => {
                elem_value.set_int(&self.1.spdif_outputs);
                Ok(true)
            }
            ADAT_OUTPUT_NAME => {
                elem_value.set_int(&self.1.adat_outputs);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
