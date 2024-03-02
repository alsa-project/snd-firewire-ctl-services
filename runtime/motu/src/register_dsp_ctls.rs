// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

pub(crate) use super::{common_ctls::PhoneAssignCtl, register_dsp_runtime::*};

#[derive(Default, Debug)]
pub(crate) struct RegisterDspPhoneAssignCtl<T>(pub PhoneAssignCtl<T>)
where
    T: MotuPortAssignSpecification
        + MotuWhollyCacheableParamsOperation<PhoneAssignParameters>
        + MotuWhollyUpdatableParamsOperation<PhoneAssignParameters>
        + MotuRegisterDspImageOperation<PhoneAssignParameters, SndMotuRegisterDspParameter>
        + MotuRegisterDspEventOperation<PhoneAssignParameters>;

impl<T> RegisterDspPhoneAssignCtl<T>
where
    T: MotuPortAssignSpecification
        + MotuWhollyCacheableParamsOperation<PhoneAssignParameters>
        + MotuWhollyUpdatableParamsOperation<PhoneAssignParameters>
        + MotuRegisterDspImageOperation<PhoneAssignParameters, SndMotuRegisterDspParameter>
        + MotuRegisterDspEventOperation<PhoneAssignParameters>,
{
    pub(crate) fn parse_dsp_parameter(&mut self, params: &SndMotuRegisterDspParameter) {
        T::parse_image(&mut self.0.params, params);
    }

    pub(crate) fn parse_dsp_event(&mut self, event: &RegisterDspEvent) -> bool {
        let res = T::parse_event(&mut self.0.params, event);
        if res {
            debug!(params = ?self.0.params, ?event);
        }
        res
    }
}

#[derive(Default, Debug)]
pub(crate) struct RegisterDspMixerOutputCtl<T>
where
    T: MotuRegisterDspSpecification
        + MotuWhollyCacheableParamsOperation<RegisterDspMixerOutputState>
        + MotuPartiallyUpdatableParamsOperation<RegisterDspMixerOutputState>
        + MotuRegisterDspImageOperation<RegisterDspMixerOutputState, SndMotuRegisterDspParameter>
        + MotuRegisterDspEventOperation<RegisterDspMixerOutputState>,
{
    pub elem_id_list: Vec<ElemId>,
    params: RegisterDspMixerOutputState,
    _phantom: PhantomData<T>,
}

const MIXER_OUTPUT_VOLUME_NAME: &str = "mixer-output-volume";
const MIXER_OUTPUT_MUTE_NAME: &str = "mixer-output-mute";
const MIXER_OUTPUT_DST_NAME: &str = "mixer-output-destination";

fn copy_int_to_elem_value<T: Copy + Into<i32>>(elem_value: &mut ElemValue, data: &[T]) {
    let vals: Vec<i32> = data.iter().map(|&val| val.into()).collect();
    elem_value.set_int(&vals);
}

impl<T> RegisterDspMixerOutputCtl<T>
where
    T: MotuRegisterDspSpecification
        + MotuWhollyCacheableParamsOperation<RegisterDspMixerOutputState>
        + MotuPartiallyUpdatableParamsOperation<RegisterDspMixerOutputState>
        + MotuRegisterDspImageOperation<RegisterDspMixerOutputState, SndMotuRegisterDspParameter>
        + MotuRegisterDspEventOperation<RegisterDspMixerOutputState>,
{
    const VOL_TLV: DbInterval = DbInterval {
        min: 0,
        max: 63,
        linear: true,
        mute_avail: false,
    };

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
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIXER_OUTPUT_VOLUME_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                T::MIXER_OUTPUT_VOLUME_MIN as i32,
                T::MIXER_OUTPUT_VOLUME_MAX as i32,
                T::MIXER_OUTPUT_VOLUME_STEP as i32,
                T::MIXER_COUNT,
                Some(&Vec::<u32>::from(&Self::VOL_TLV)),
                true,
            )
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIXER_OUTPUT_MUTE_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, 1, T::MIXER_COUNT, true)
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        if T::MIXER_OUTPUT_DESTINATIONS.len() > 0 {
            let labels: Vec<String> = T::MIXER_OUTPUT_DESTINATIONS
                .iter()
                .map(|p| target_port_to_string(p))
                .collect();
            let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIXER_OUTPUT_DST_NAME, 0);
            card_cntr
                .add_enum_elems(&elem_id, 1, T::MIXER_COUNT, &labels, None, true)
                .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;
        }

        Ok(())
    }

    pub(crate) fn read(
        &mut self,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            MIXER_OUTPUT_VOLUME_NAME => {
                copy_int_to_elem_value(elem_value, &self.params.volume);
                Ok(true)
            }
            MIXER_OUTPUT_MUTE_NAME => {
                elem_value.set_bool(&self.params.mute);
                Ok(true)
            }
            MIXER_OUTPUT_DST_NAME => {
                let vals: Vec<u32> = self
                    .params
                    .destination
                    .iter()
                    .map(|dst| {
                        T::MIXER_OUTPUT_DESTINATIONS
                            .iter()
                            .position(|p| dst.eq(p))
                            .map(|p| p as u32)
                            .unwrap()
                    })
                    .collect();
                elem_value.set_enum(&vals);
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
            MIXER_OUTPUT_VOLUME_NAME => {
                let mut params = self.params.clone();
                params
                    .volume
                    .iter_mut()
                    .zip(elem_value.int())
                    .for_each(|(vol, &val)| *vol = val as u8);
                let res = T::update_partially(req, node, &mut self.params, params, timeout_ms);
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            MIXER_OUTPUT_MUTE_NAME => {
                let mut params = self.params.clone();
                let vals = &elem_value.boolean()[..T::MIXER_COUNT];
                params.mute.copy_from_slice(vals);
                let res = T::update_partially(req, node, &mut self.params, params, timeout_ms);
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            MIXER_OUTPUT_DST_NAME => {
                let mut params = self.params.clone();
                params
                    .destination
                    .iter_mut()
                    .zip(elem_value.enumerated())
                    .try_for_each(|(dest, &val)| {
                        let pos = val as usize;
                        T::MIXER_OUTPUT_DESTINATIONS
                            .iter()
                            .nth(pos)
                            .ok_or_else(|| {
                                let msg = format!("Invalid index for ourput destination: {}", val);
                                Error::new(FileError::Inval, &msg)
                            })
                            .map(|&port| *dest = port)
                    })?;
                let res = T::update_partially(req, node, &mut self.params, params, timeout_ms);
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            _ => Ok(false),
        }
    }

    pub(crate) fn parse_dsp_parameter(&mut self, image: &SndMotuRegisterDspParameter) {
        T::parse_image(&mut self.params, image)
    }

    pub(crate) fn parse_dsp_event(&mut self, event: &RegisterDspEvent) -> bool {
        let res = T::parse_event(&mut self.params, event);
        if res {
            debug!(params = ?self.params, ?event);
        }
        res
    }
}

#[derive(Default, Debug)]
pub(crate) struct RegisterDspMixerReturnCtl<T>
where
    T: MotuRegisterDspSpecification
        + MotuWhollyCacheableParamsOperation<RegisterDspMixerReturnParameters>
        + MotuWhollyUpdatableParamsOperation<RegisterDspMixerReturnParameters>,
{
    pub elem_id_list: Vec<ElemId>,
    params: RegisterDspMixerReturnParameters,
    _phantom: PhantomData<T>,
}

const MIXER_RETURN_ENABLE_NAME: &str = "mixer-return-enable";

impl<T> RegisterDspMixerReturnCtl<T>
where
    T: MotuRegisterDspSpecification
        + MotuWhollyCacheableParamsOperation<RegisterDspMixerReturnParameters>
        + MotuWhollyUpdatableParamsOperation<RegisterDspMixerReturnParameters>,
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
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIXER_RETURN_ENABLE_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, 1, 1, true)
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))
    }

    pub(crate) fn read(
        &mut self,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            MIXER_RETURN_ENABLE_NAME => {
                elem_value.set_bool(&[self.params.0]);
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
            MIXER_RETURN_ENABLE_NAME => {
                let mut params = self.params.clone();
                params.0 = elem_value.boolean()[0];
                let res =
                    T::update_wholly(req, node, &params, timeout_ms).map(|_| self.params = params);
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

#[derive(Debug)]
pub(crate) struct RegisterDspMixerMonauralSourceCtl<T>
where
    T: MotuRegisterDspMixerMonauralSourceSpecification
        + MotuWhollyCacheableParamsOperation<RegisterDspMixerMonauralSourceState>
        + MotuPartiallyUpdatableParamsOperation<RegisterDspMixerMonauralSourceState>
        + MotuRegisterDspImageOperation<
            RegisterDspMixerMonauralSourceState,
            SndMotuRegisterDspParameter,
        > + MotuRegisterDspEventOperation<RegisterDspMixerMonauralSourceState>,
{
    pub elem_id_list: Vec<ElemId>,
    params: RegisterDspMixerMonauralSourceState,
    _phantom: PhantomData<T>,
}

const MIXER_SOURCE_GAIN_NAME: &str = "mixer-source-gain";
const MIXER_SOURCE_PAN_NAME: &str = "mixer-source-pan";
const MIXER_SOURCE_MUTE_NAME: &str = "mixer-source-mute";
const MIXER_SOURCE_SOLO_NAME: &str = "mixer-source-solo";

impl<T> Default for RegisterDspMixerMonauralSourceCtl<T>
where
    T: MotuRegisterDspMixerMonauralSourceSpecification
        + MotuWhollyCacheableParamsOperation<RegisterDspMixerMonauralSourceState>
        + MotuPartiallyUpdatableParamsOperation<RegisterDspMixerMonauralSourceState>
        + MotuRegisterDspImageOperation<
            RegisterDspMixerMonauralSourceState,
            SndMotuRegisterDspParameter,
        > + MotuRegisterDspEventOperation<RegisterDspMixerMonauralSourceState>,
{
    fn default() -> Self {
        Self {
            elem_id_list: Default::default(),
            params: T::create_mixer_monaural_source_state(),
            _phantom: Default::default(),
        }
    }
}

fn mixer_monaural_source_entry(
    params: &RegisterDspMixerMonauralSourceState,
    index: usize,
) -> Result<&RegisterDspMixerMonauralSourceEntry, Error> {
    params.0.iter().nth(index).ok_or_else(|| {
        let msg = format!("Invalid index for mixer sources: {}", index);
        Error::new(FileError::Inval, &msg)
    })
}

fn mixer_monaural_source_entry_mut(
    params: &mut RegisterDspMixerMonauralSourceState,
    index: usize,
) -> Result<&mut RegisterDspMixerMonauralSourceEntry, Error> {
    params.0.iter_mut().nth(index).ok_or_else(|| {
        let msg = format!("Invalid index for mixer sources: {}", index);
        Error::new(FileError::Inval, &msg)
    })
}

impl<T> RegisterDspMixerMonauralSourceCtl<T>
where
    T: MotuRegisterDspMixerMonauralSourceSpecification
        + MotuWhollyCacheableParamsOperation<RegisterDspMixerMonauralSourceState>
        + MotuPartiallyUpdatableParamsOperation<RegisterDspMixerMonauralSourceState>
        + MotuRegisterDspImageOperation<
            RegisterDspMixerMonauralSourceState,
            SndMotuRegisterDspParameter,
        > + MotuRegisterDspEventOperation<RegisterDspMixerMonauralSourceState>,
{
    const GAIN_TLV: DbInterval = DbInterval {
        min: -6400,
        max: 0,
        linear: true,
        mute_avail: false,
    };

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
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIXER_SOURCE_GAIN_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                T::MIXER_COUNT,
                T::SOURCE_GAIN_MIN as i32,
                T::SOURCE_GAIN_MAX as i32,
                T::SOURCE_GAIN_STEP as i32,
                T::MIXER_SOURCES.len(),
                Some(&Vec::<u32>::from(&Self::GAIN_TLV)),
                true,
            )
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIXER_SOURCE_PAN_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                T::MIXER_COUNT,
                T::SOURCE_PAN_MIN as i32,
                T::SOURCE_PAN_MAX as i32,
                T::SOURCE_PAN_STEP as i32,
                T::MIXER_SOURCES.len(),
                None,
                true,
            )
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIXER_SOURCE_MUTE_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, T::MIXER_COUNT, T::MIXER_SOURCES.len(), true)
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIXER_SOURCE_SOLO_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, T::MIXER_COUNT, T::MIXER_SOURCES.len(), true)
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        Ok(())
    }

    pub(crate) fn read(
        &mut self,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            MIXER_SOURCE_GAIN_NAME => {
                let mixer = elem_id.index() as usize;
                let src = mixer_monaural_source_entry(&self.params, mixer)?;
                copy_int_to_elem_value(elem_value, &src.gain);
                Ok(true)
            }
            MIXER_SOURCE_PAN_NAME => {
                let mixer = elem_id.index() as usize;
                let src = mixer_monaural_source_entry(&self.params, mixer)?;
                copy_int_to_elem_value(elem_value, &src.pan);
                Ok(true)
            }
            MIXER_SOURCE_MUTE_NAME => {
                let mixer = elem_id.index() as usize;
                let src = mixer_monaural_source_entry(&self.params, mixer)?;
                elem_value.set_bool(&src.mute);
                Ok(true)
            }
            MIXER_SOURCE_SOLO_NAME => {
                let mixer = elem_id.index() as usize;
                let src = mixer_monaural_source_entry(&self.params, mixer)?;
                elem_value.set_bool(&src.solo);
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
            MIXER_SOURCE_GAIN_NAME => {
                let mut params = self.params.clone();
                let mixer = elem_id.index() as usize;
                let src = mixer_monaural_source_entry_mut(&mut params, mixer)?;
                src.gain
                    .iter_mut()
                    .zip(elem_value.int())
                    .for_each(|(gain, &val)| *gain = val as u8);
                let res = T::update_partially(req, node, &mut self.params, params, timeout_ms);
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            MIXER_SOURCE_PAN_NAME => {
                let mut params = self.params.clone();
                let mixer = elem_id.index() as usize;
                let src = mixer_monaural_source_entry_mut(&mut params, mixer)?;
                src.pan
                    .iter_mut()
                    .zip(elem_value.int())
                    .for_each(|(pan, &val)| *pan = val as u8);
                let res = T::update_partially(req, node, &mut self.params, params, timeout_ms);
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            MIXER_SOURCE_MUTE_NAME => {
                let mut params = self.params.clone();
                let mixer = elem_id.index() as usize;
                let src = mixer_monaural_source_entry_mut(&mut params, mixer)?;
                let vals = &elem_value.boolean()[..src.mute.len()];
                src.mute.copy_from_slice(vals);
                let res = T::update_partially(req, node, &mut self.params, params, timeout_ms);
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            MIXER_SOURCE_SOLO_NAME => {
                let mut params = self.params.clone();
                let mixer = elem_id.index() as usize;
                let src = mixer_monaural_source_entry_mut(&mut params, mixer)?;
                let vals = &elem_value.boolean()[..src.mute.len()];
                src.solo.copy_from_slice(vals);
                let res = T::update_partially(req, node, &mut self.params, params, timeout_ms);
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            _ => Ok(false),
        }
    }

    pub(crate) fn parse_dsp_parameter(&mut self, image: &SndMotuRegisterDspParameter) {
        T::parse_image(&mut self.params, image)
    }

    pub(crate) fn parse_dsp_event(&mut self, event: &RegisterDspEvent) -> bool {
        let res = T::parse_event(&mut self.params, event);
        if res {
            debug!(params = ?self.params, ?event);
        }
        res
    }
}

#[derive(Default, Debug)]
pub(crate) struct RegisterDspMixerStereoSourceCtl<T>
where
    T: MotuRegisterDspMixerStereoSourceSpecification
        + MotuWhollyCacheableParamsOperation<RegisterDspMixerStereoSourceState>
        + MotuPartiallyUpdatableParamsOperation<RegisterDspMixerStereoSourceState>
        + MotuRegisterDspImageOperation<
            RegisterDspMixerStereoSourceState,
            SndMotuRegisterDspParameter,
        > + MotuRegisterDspEventOperation<RegisterDspMixerStereoSourceState>,
{
    pub elem_id_list: Vec<ElemId>,
    params: RegisterDspMixerStereoSourceState,
    _phantom: PhantomData<T>,
}

const MIXER_SOURCE_STEREO_BALANCE_NAME: &str = "mixer-source-stereo-balance";
const MIXER_SOURCE_STEREO_WIDTH_NAME: &str = "mixer-source-stereo-width";

fn mixer_stereo_source_entry(
    params: &RegisterDspMixerStereoSourceState,
    index: usize,
) -> Result<&RegisterDspMixerStereoSourceEntry, Error> {
    params.0.iter().nth(index).ok_or_else(|| {
        let msg = format!("Invalid index for mixer sources: {}", index);
        Error::new(FileError::Inval, &msg)
    })
}

fn mixer_stereo_source_entry_mut(
    params: &mut RegisterDspMixerStereoSourceState,
    index: usize,
) -> Result<&mut RegisterDspMixerStereoSourceEntry, Error> {
    params.0.iter_mut().nth(index).ok_or_else(|| {
        let msg = format!("Invalid index for mixer sources: {}", index);
        Error::new(FileError::Inval, &msg)
    })
}

impl<T> RegisterDspMixerStereoSourceCtl<T>
where
    T: MotuRegisterDspMixerStereoSourceSpecification
        + MotuWhollyCacheableParamsOperation<RegisterDspMixerStereoSourceState>
        + MotuPartiallyUpdatableParamsOperation<RegisterDspMixerStereoSourceState>
        + MotuRegisterDspImageOperation<
            RegisterDspMixerStereoSourceState,
            SndMotuRegisterDspParameter,
        > + MotuRegisterDspEventOperation<RegisterDspMixerStereoSourceState>,
{
    const GAIN_TLV: DbInterval = DbInterval {
        min: -6400,
        max: 0,
        linear: true,
        mute_avail: false,
    };

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
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIXER_SOURCE_GAIN_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                T::MIXER_COUNT,
                T::SOURCE_GAIN_MIN as i32,
                T::SOURCE_GAIN_MAX as i32,
                T::SOURCE_GAIN_STEP as i32,
                T::MIXER_SOURCES.len(),
                Some(&Vec::<u32>::from(&Self::GAIN_TLV)),
                true,
            )
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIXER_SOURCE_PAN_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                T::MIXER_COUNT,
                T::SOURCE_PAN_MIN as i32,
                T::SOURCE_PAN_MAX as i32,
                T::SOURCE_PAN_STEP as i32,
                T::MIXER_SOURCES.len(),
                Some(&Vec::<u32>::from(&Self::GAIN_TLV)),
                true,
            )
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIXER_SOURCE_MUTE_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, T::MIXER_COUNT, T::MIXER_SOURCES.len(), true)
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIXER_SOURCE_SOLO_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, T::MIXER_COUNT, T::MIXER_SOURCES.len(), true)
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(
            ElemIfaceType::Mixer,
            0,
            0,
            MIXER_SOURCE_STEREO_BALANCE_NAME,
            0,
        );
        card_cntr
            .add_int_elems(
                &elem_id,
                T::MIXER_COUNT,
                T::SOURCE_STEREO_BALANCE_MIN as i32,
                T::SOURCE_STEREO_BALANCE_MAX as i32,
                T::SOURCE_STEREO_BALANCE_STEP as i32,
                T::MIXER_SOURCE_PAIR_COUNT,
                Some(&Vec::<u32>::from(&Self::GAIN_TLV)),
                true,
            )
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(
            ElemIfaceType::Mixer,
            0,
            0,
            MIXER_SOURCE_STEREO_WIDTH_NAME,
            0,
        );
        card_cntr
            .add_int_elems(
                &elem_id,
                T::MIXER_COUNT,
                T::SOURCE_STEREO_WIDTH_MIN as i32,
                T::SOURCE_STEREO_WIDTH_MAX as i32,
                T::SOURCE_STEREO_WIDTH_STEP as i32,
                T::MIXER_SOURCE_PAIR_COUNT,
                Some(&Vec::<u32>::from(&Self::GAIN_TLV)),
                true,
            )
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        Ok(())
    }

    pub(crate) fn read(
        &mut self,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            MIXER_SOURCE_GAIN_NAME => {
                let mixer = elem_id.index() as usize;
                let src = mixer_stereo_source_entry(&self.params, mixer)?;
                copy_int_to_elem_value(elem_value, &src.gain);
                Ok(true)
            }
            MIXER_SOURCE_PAN_NAME => {
                let mixer = elem_id.index() as usize;
                let src = mixer_stereo_source_entry(&self.params, mixer)?;
                copy_int_to_elem_value(elem_value, &src.pan);
                Ok(true)
            }
            MIXER_SOURCE_MUTE_NAME => {
                let mixer = elem_id.index() as usize;
                let src = mixer_stereo_source_entry(&self.params, mixer)?;
                elem_value.set_bool(&src.mute);
                Ok(true)
            }
            MIXER_SOURCE_SOLO_NAME => {
                let mixer = elem_id.index() as usize;
                let src = mixer_stereo_source_entry(&self.params, mixer)?;
                elem_value.set_bool(&src.solo);
                Ok(true)
            }
            MIXER_SOURCE_STEREO_BALANCE_NAME => {
                let mixer = elem_id.index() as usize;
                let src = mixer_stereo_source_entry(&self.params, mixer)?;
                copy_int_to_elem_value(elem_value, &src.balance);
                Ok(true)
            }
            MIXER_SOURCE_STEREO_WIDTH_NAME => {
                let mixer = elem_id.index() as usize;
                let src = mixer_stereo_source_entry(&self.params, mixer)?;
                copy_int_to_elem_value(elem_value, &src.width);
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
            MIXER_SOURCE_GAIN_NAME => {
                let mut params = self.params.clone();
                let mixer = elem_id.index() as usize;
                let src = mixer_stereo_source_entry_mut(&mut params, mixer)?;
                src.gain
                    .iter_mut()
                    .zip(elem_value.int())
                    .for_each(|(gain, &val)| *gain = val as u8);
                let res = T::update_partially(req, node, &mut self.params, params, timeout_ms);
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            MIXER_SOURCE_PAN_NAME => {
                let mut params = self.params.clone();
                let mixer = elem_id.index() as usize;
                let src = mixer_stereo_source_entry_mut(&mut params, mixer)?;
                src.pan
                    .iter_mut()
                    .zip(elem_value.int())
                    .for_each(|(pan, &val)| *pan = val as u8);
                let res = T::update_partially(req, node, &mut self.params, params, timeout_ms);
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            MIXER_SOURCE_MUTE_NAME => {
                let mut params = self.params.clone();
                let mixer = elem_id.index() as usize;
                let src = mixer_stereo_source_entry_mut(&mut params, mixer)?;
                let vals = &elem_value.boolean()[..src.mute.len()];
                src.mute.copy_from_slice(vals);
                let res = T::update_partially(req, node, &mut self.params, params, timeout_ms);
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            MIXER_SOURCE_SOLO_NAME => {
                let mut params = self.params.clone();
                let mixer = elem_id.index() as usize;
                let src = mixer_stereo_source_entry_mut(&mut params, mixer)?;
                let vals = &elem_value.boolean()[..src.solo.len()];
                src.solo.copy_from_slice(vals);
                let res = T::update_partially(req, node, &mut self.params, params, timeout_ms);
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            MIXER_SOURCE_STEREO_BALANCE_NAME => {
                let mut params = self.params.clone();
                let mixer = elem_id.index() as usize;
                let src = mixer_stereo_source_entry_mut(&mut params, mixer)?;
                src.balance
                    .iter_mut()
                    .zip(elem_value.int())
                    .for_each(|(balance, &val)| *balance = val as u8);
                let res = T::update_partially(req, node, &mut self.params, params, timeout_ms);
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            MIXER_SOURCE_STEREO_WIDTH_NAME => {
                let mut params = self.params.clone();
                let mixer = elem_id.index() as usize;
                let src = mixer_stereo_source_entry_mut(&mut params, mixer)?;
                src.width
                    .iter_mut()
                    .zip(elem_value.int())
                    .for_each(|(width, &val)| *width = val as u8);
                let res = T::update_partially(req, node, &mut self.params, params, timeout_ms);
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            _ => Ok(false),
        }
    }

    pub(crate) fn parse_dsp_parameter(&mut self, image: &SndMotuRegisterDspParameter) {
        T::parse_image(&mut self.params, image)
    }

    pub(crate) fn parse_dsp_event(&mut self, event: &RegisterDspEvent) -> bool {
        let res = T::parse_event(&mut self.params, event);
        if res {
            debug!(params = ?self.params, ?event);
        }
        res
    }
}

#[derive(Default, Debug)]
pub(crate) struct RegisterDspOutputCtl<T>
where
    T: MotuRegisterDspSpecification
        + MotuWhollyCacheableParamsOperation<RegisterDspOutputState>
        + MotuPartiallyUpdatableParamsOperation<RegisterDspOutputState>
        + MotuRegisterDspImageOperation<RegisterDspOutputState, SndMotuRegisterDspParameter>
        + MotuRegisterDspEventOperation<RegisterDspOutputState>,
{
    pub elem_id_list: Vec<ElemId>,
    params: RegisterDspOutputState,
    _phantom: PhantomData<T>,
}

const MASTER_OUTPUT_VOLUME_NAME: &str = "master-output-volume";
const PHONE_VOLUME_NAME: &str = "headphone-volume";

impl<T> RegisterDspOutputCtl<T>
where
    T: MotuRegisterDspSpecification
        + MotuWhollyCacheableParamsOperation<RegisterDspOutputState>
        + MotuPartiallyUpdatableParamsOperation<RegisterDspOutputState>
        + MotuRegisterDspImageOperation<RegisterDspOutputState, SndMotuRegisterDspParameter>
        + MotuRegisterDspEventOperation<RegisterDspOutputState>,
{
    const VOL_TLV: DbInterval = DbInterval {
        min: -6400,
        max: 0,
        linear: true,
        mute_avail: false,
    };

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
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MASTER_OUTPUT_VOLUME_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                T::OUTPUT_VOLUME_MIN as i32,
                T::OUTPUT_VOLUME_MAX as i32,
                T::OUTPUT_VOLUME_STEP as i32,
                1,
                Some(&Vec::<u32>::from(&Self::VOL_TLV)),
                true,
            )
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, PHONE_VOLUME_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                T::OUTPUT_VOLUME_MIN as i32,
                T::OUTPUT_VOLUME_MAX as i32,
                T::OUTPUT_VOLUME_STEP as i32,
                1,
                Some(&Vec::<u32>::from(&Self::VOL_TLV)),
                true,
            )
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        Ok(())
    }

    pub(crate) fn read(
        &mut self,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            MASTER_OUTPUT_VOLUME_NAME => {
                elem_value.set_int(&[self.params.master_volume as i32]);
                Ok(true)
            }
            PHONE_VOLUME_NAME => {
                elem_value.set_int(&[self.params.phone_volume as i32]);
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
            MASTER_OUTPUT_VOLUME_NAME => {
                let mut params = self.params.clone();
                params.master_volume = elem_value.int()[0] as u8;
                let res = T::update_partially(req, node, &mut self.params, params, timeout_ms)
                    .map(|_| self.params = params);
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            PHONE_VOLUME_NAME => {
                let mut params = self.params.clone();
                params.phone_volume = elem_value.int()[0] as u8;
                let res = T::update_partially(req, node, &mut self.params, params, timeout_ms)
                    .map(|_| self.params = params);
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            _ => Ok(false),
        }
    }

    pub(crate) fn parse_dsp_parameter(&mut self, image: &SndMotuRegisterDspParameter) {
        T::parse_image(&mut self.params, image)
    }

    pub(crate) fn parse_dsp_event(&mut self, event: &RegisterDspEvent) -> bool {
        let res = T::parse_event(&mut self.params, event);
        if res {
            debug!(params = ?self.params, ?event);
        }
        res
    }
}

#[derive(Debug)]
pub(crate) struct RegisterDspLineInputCtl<T>
where
    T: MotuRegisterDspLineInputSpecification
        + MotuWhollyCacheableParamsOperation<RegisterDspLineInputState>
        + MotuPartiallyUpdatableParamsOperation<RegisterDspLineInputState>
        + MotuRegisterDspImageOperation<RegisterDspLineInputState, SndMotuRegisterDspParameter>
        + MotuRegisterDspEventOperation<RegisterDspLineInputState>,
{
    pub elem_id_list: Vec<ElemId>,
    params: RegisterDspLineInputState,
    _phantom: PhantomData<T>,
}

const INPUT_NOMINAL_LEVEL_NAME: &str = "input-nominal-level";
const INPUT_BOOST_NAME: &str = "input-boost";

impl<T> Default for RegisterDspLineInputCtl<T>
where
    T: MotuRegisterDspLineInputSpecification
        + MotuWhollyCacheableParamsOperation<RegisterDspLineInputState>
        + MotuPartiallyUpdatableParamsOperation<RegisterDspLineInputState>
        + MotuRegisterDspImageOperation<RegisterDspLineInputState, SndMotuRegisterDspParameter>
        + MotuRegisterDspEventOperation<RegisterDspLineInputState>,
{
    fn default() -> Self {
        Self {
            elem_id_list: Default::default(),
            params: T::create_line_input_state(),
            _phantom: Default::default(),
        }
    }
}

impl<T> RegisterDspLineInputCtl<T>
where
    T: MotuRegisterDspLineInputSpecification
        + MotuWhollyCacheableParamsOperation<RegisterDspLineInputState>
        + MotuPartiallyUpdatableParamsOperation<RegisterDspLineInputState>
        + MotuRegisterDspImageOperation<RegisterDspLineInputState, SndMotuRegisterDspParameter>
        + MotuRegisterDspEventOperation<RegisterDspLineInputState>,
{
    const NOMINAL_LEVELS: &'static [NominalSignalLevel] = &[
        NominalSignalLevel::Consumer,
        NominalSignalLevel::Professional,
    ];

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
        let labels: Vec<&str> = Self::NOMINAL_LEVELS
            .iter()
            .map(|l| nominal_signal_level_to_str(l))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, INPUT_NOMINAL_LEVEL_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, T::LINE_INPUT_COUNT, &labels, None, true)
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, INPUT_BOOST_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, 1, T::LINE_INPUT_COUNT, true)
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        Ok(())
    }

    pub(crate) fn read(
        &mut self,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            INPUT_NOMINAL_LEVEL_NAME => {
                let vals: Vec<u32> = self
                    .params
                    .level
                    .iter()
                    .map(|level| {
                        Self::NOMINAL_LEVELS
                            .iter()
                            .position(|l| level.eq(l))
                            .map(|pos| pos as u32)
                            .unwrap()
                    })
                    .collect();
                elem_value.set_enum(&vals);
                Ok(true)
            }
            INPUT_BOOST_NAME => {
                elem_value.set_bool(&self.params.boost);
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
            INPUT_NOMINAL_LEVEL_NAME => {
                let mut params = self.params.clone();
                params
                    .level
                    .iter_mut()
                    .zip(elem_value.enumerated())
                    .try_for_each(|(level, &val)| {
                        let pos = val as usize;
                        Self::NOMINAL_LEVELS
                            .iter()
                            .nth(pos)
                            .ok_or_else(|| {
                                let msg = format!("Invalid index of nominal signal level: {}", pos);
                                Error::new(FileError::Inval, &msg)
                            })
                            .map(|&l| *level = l)
                    })?;
                let res = T::update_partially(req, node, &mut self.params, params, timeout_ms);
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            INPUT_BOOST_NAME => {
                let mut params = self.params.clone();
                let vals = &elem_value.boolean()[..params.boost.len()];
                params.boost.copy_from_slice(vals);
                let res = T::update_partially(req, node, &mut self.params, params, timeout_ms);
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            _ => Ok(false),
        }
    }

    pub(crate) fn parse_dsp_parameter(&mut self, image: &SndMotuRegisterDspParameter) {
        T::parse_image(&mut self.params, image)
    }

    pub(crate) fn parse_dsp_event(&mut self, event: &RegisterDspEvent) -> bool {
        let res = T::parse_event(&mut self.params, event);
        if res {
            debug!(params = ?self.params, ?event);
        }
        res
    }
}

#[derive(Default, Debug)]
pub(crate) struct RegisterDspMonauralInputCtl<T>
where
    T: MotuRegisterDspMonauralInputSpecification
        + MotuWhollyCacheableParamsOperation<RegisterDspMonauralInputState>
        + MotuPartiallyUpdatableParamsOperation<RegisterDspMonauralInputState>
        + MotuRegisterDspImageOperation<RegisterDspMonauralInputState, SndMotuRegisterDspParameter>
        + MotuRegisterDspEventOperation<RegisterDspMonauralInputState>,
{
    pub elem_id_list: Vec<ElemId>,
    params: RegisterDspMonauralInputState,
    _phantom: PhantomData<T>,
}

const INPUT_GAIN_NAME: &str = "input-gain";
const INPUT_INVERT_NAME: &str = "input-invert";
const MIC_PHANTOM_NAME: &str = "mic-phantom";
const MIC_PAD_NAME: &str = "mic-pad";
const INPUT_JACK_NAME: &str = "input-jack";
const INPUT_PAIRED_NAME: &str = "input-paired";

impl<T> RegisterDspMonauralInputCtl<T>
where
    T: MotuRegisterDspMonauralInputSpecification
        + MotuWhollyCacheableParamsOperation<RegisterDspMonauralInputState>
        + MotuPartiallyUpdatableParamsOperation<RegisterDspMonauralInputState>
        + MotuRegisterDspImageOperation<RegisterDspMonauralInputState, SndMotuRegisterDspParameter>
        + MotuRegisterDspEventOperation<RegisterDspMonauralInputState>,
{
    const GAIN_TLV: DbInterval = DbInterval {
        min: 0,
        max: 2400,
        linear: false,
        mute_avail: false,
    };

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
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, INPUT_GAIN_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                T::INPUT_GAIN_MIN as i32,
                T::INPUT_MIC_GAIN_MAX as i32,
                T::INPUT_GAIN_STEP as i32,
                T::INPUT_COUNT,
                Some(&Vec::<u32>::from(&Self::GAIN_TLV)),
                true,
            )
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, INPUT_INVERT_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, 1, T::INPUT_COUNT, true)
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        Ok(())
    }

    pub(crate) fn read(
        &mut self,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            INPUT_GAIN_NAME => {
                copy_int_to_elem_value(elem_value, &self.params.gain);
                Ok(true)
            }
            INPUT_INVERT_NAME => {
                elem_value.set_bool(&self.params.invert);
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
            INPUT_GAIN_NAME => {
                let mut params = self.params.clone();
                params
                    .gain
                    .iter_mut()
                    .zip(elem_value.int())
                    .for_each(|(gain, &val)| *gain = val as u8);
                let res = T::update_partially(req, node, &mut self.params, params, timeout_ms);
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            INPUT_INVERT_NAME => {
                let mut params = self.params.clone();
                let vals = &elem_value.boolean()[..params.invert.len()];
                params.invert.copy_from_slice(vals);
                let res = T::update_partially(req, node, &mut self.params, params, timeout_ms);
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            _ => Ok(false),
        }
    }

    pub(crate) fn parse_dsp_parameter(&mut self, image: &SndMotuRegisterDspParameter) {
        T::parse_image(&mut self.params, image)
    }

    pub(crate) fn parse_dsp_event(&mut self, event: &RegisterDspEvent) -> bool {
        let res = T::parse_event(&mut self.params, event);
        if res {
            debug!(params = ?self.params, ?event);
        }
        res
    }
}

#[derive(Debug)]
pub(crate) struct RegisterDspStereoInputCtl<T>
where
    T: MotuRegisterDspStereoInputSpecification
        + MotuWhollyCacheableParamsOperation<RegisterDspStereoInputState>
        + MotuPartiallyUpdatableParamsOperation<RegisterDspStereoInputState>
        + MotuRegisterDspImageOperation<RegisterDspStereoInputState, SndMotuRegisterDspParameter>
        + MotuRegisterDspEventOperation<RegisterDspStereoInputState>,
{
    pub elem_id_list: Vec<ElemId>,
    params: RegisterDspStereoInputState,
    _phantom: PhantomData<T>,
}

impl<T> Default for RegisterDspStereoInputCtl<T>
where
    T: MotuRegisterDspStereoInputSpecification
        + MotuWhollyCacheableParamsOperation<RegisterDspStereoInputState>
        + MotuPartiallyUpdatableParamsOperation<RegisterDspStereoInputState>
        + MotuRegisterDspImageOperation<RegisterDspStereoInputState, SndMotuRegisterDspParameter>
        + MotuRegisterDspEventOperation<RegisterDspStereoInputState>,
{
    fn default() -> Self {
        Self {
            elem_id_list: Default::default(),
            params: T::create_stereo_input_state(),
            _phantom: Default::default(),
        }
    }
}

impl<T> RegisterDspStereoInputCtl<T>
where
    T: MotuRegisterDspStereoInputSpecification
        + MotuWhollyCacheableParamsOperation<RegisterDspStereoInputState>
        + MotuPartiallyUpdatableParamsOperation<RegisterDspStereoInputState>
        + MotuRegisterDspImageOperation<RegisterDspStereoInputState, SndMotuRegisterDspParameter>
        + MotuRegisterDspEventOperation<RegisterDspStereoInputState>,
{
    const GAIN_TLV: DbInterval = DbInterval {
        min: 0,
        max: 6400,
        linear: true,
        mute_avail: false,
    };

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
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, INPUT_GAIN_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                T::INPUT_GAIN_MIN as i32,
                T::INPUT_MIC_GAIN_MAX as i32, // TODO: differentiate mic and line.
                T::INPUT_GAIN_STEP as i32,
                T::INPUT_COUNT,
                Some(&Vec::<u32>::from(&Self::GAIN_TLV)),
                true,
            )
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, INPUT_INVERT_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, 1, T::INPUT_COUNT, true)
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIC_PHANTOM_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, 1, T::MIC_COUNT, true)
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIC_PAD_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, 1, T::MIC_COUNT, true)
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, INPUT_JACK_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, 1, T::MIC_COUNT, true)
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, INPUT_PAIRED_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, 1, T::INPUT_PAIR_COUNT, true)
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        Ok(())
    }

    pub(crate) fn read(
        &mut self,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            INPUT_GAIN_NAME => {
                copy_int_to_elem_value(elem_value, &self.params.gain);
                Ok(true)
            }
            INPUT_INVERT_NAME => {
                elem_value.set_bool(&self.params.invert);
                Ok(true)
            }
            MIC_PHANTOM_NAME => {
                elem_value.set_bool(&self.params.phantom);
                Ok(true)
            }
            MIC_PAD_NAME => {
                elem_value.set_bool(&self.params.pad);
                Ok(true)
            }
            INPUT_JACK_NAME => {
                elem_value.set_bool(&self.params.jack);
                Ok(true)
            }
            INPUT_PAIRED_NAME => {
                elem_value.set_bool(&self.params.paired);
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
            INPUT_GAIN_NAME => {
                let mut params = self.params.clone();
                params
                    .gain
                    .iter_mut()
                    .zip(elem_value.int())
                    .for_each(|(gain, &val)| *gain = val as u8);
                let res = T::update_partially(req, node, &mut self.params, params, timeout_ms);
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            INPUT_INVERT_NAME => {
                let mut params = self.params.clone();
                let vals = &elem_value.boolean()[..params.invert.len()];
                params.invert.copy_from_slice(vals);
                let res = T::update_partially(req, node, &mut self.params, params, timeout_ms);
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            MIC_PHANTOM_NAME => {
                let mut params = self.params.clone();
                let vals = &elem_value.boolean()[..params.phantom.len()];
                params.phantom.copy_from_slice(vals);
                let res = T::update_partially(req, node, &mut self.params, params, timeout_ms);
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            MIC_PAD_NAME => {
                let mut params = self.params.clone();
                let vals = &elem_value.boolean()[..params.pad.len()];
                params.pad.copy_from_slice(vals);
                let res = T::update_partially(req, node, &mut self.params, params, timeout_ms);
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            INPUT_PAIRED_NAME => {
                let mut params = self.params.clone();
                let vals = &elem_value.boolean()[..params.paired.len()];
                params.paired.copy_from_slice(vals);
                let res = T::update_partially(req, node, &mut self.params, params, timeout_ms);
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            _ => Ok(false),
        }
    }

    pub(crate) fn parse_dsp_parameter(&mut self, image: &SndMotuRegisterDspParameter) {
        T::parse_image(&mut self.params, image)
    }

    pub(crate) fn parse_dsp_event(&mut self, event: &RegisterDspEvent) -> bool {
        let res = T::parse_event(&mut self.params, event);
        if res {
            debug!(params = ?self.params, ?event);
        }
        res
    }
}

#[derive(Debug)]
pub(crate) struct RegisterDspMeterCtl<T>
where
    T: MotuRegisterDspMeterSpecification
        + MotuRegisterDspImageOperation<RegisterDspMeterState, [u8; 48]>,
{
    pub elem_id_list: Vec<ElemId>,
    params: RegisterDspMeterState,
    image: [u8; 48],
    _phantom: PhantomData<T>,
}

const INPUT_METER_NAME: &str = "input-meter";
const OUTPUT_METER_NAME: &str = "output-meter";

impl<T> Default for RegisterDspMeterCtl<T>
where
    T: MotuRegisterDspMeterSpecification
        + MotuRegisterDspImageOperation<RegisterDspMeterState, [u8; 48]>,
{
    fn default() -> Self {
        Self {
            elem_id_list: Default::default(),
            params: T::create_meter_state(),
            image: [0; 48],
            _phantom: Default::default(),
        }
    }
}

impl<T> RegisterDspMeterCtl<T>
where
    T: MotuRegisterDspMeterSpecification
        + MotuRegisterDspImageOperation<RegisterDspMeterState, [u8; 48]>,
{
    pub(crate) fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, INPUT_METER_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                T::LEVEL_MIN as i32,
                T::LEVEL_MAX as i32,
                T::LEVEL_STEP as i32,
                T::INPUT_PORTS.len(),
                None,
                false,
            )
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, OUTPUT_METER_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                T::LEVEL_MIN as i32,
                T::LEVEL_MAX as i32,
                T::LEVEL_STEP as i32,
                T::OUTPUT_PORT_COUNT,
                None,
                false,
            )
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        Ok(())
    }

    pub(crate) fn read(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            INPUT_METER_NAME => {
                copy_int_to_elem_value(elem_value, &self.params.inputs);
                Ok(true)
            }
            OUTPUT_METER_NAME => {
                copy_int_to_elem_value(elem_value, &self.params.outputs);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub(crate) fn read_dsp_meter(&mut self, unit: &SndMotu) -> Result<(), Error> {
        unit.read_byte_meter(&mut self.image)?;
        T::parse_image(&mut self.params, &self.image);
        debug!(params = ?self.params);
        Ok(())
    }
}

#[derive(Debug, Default)]
pub(crate) struct RegisterDspMeterOutputTargetCtl<T>
where
    T: MotuRegisterDspMeterOutputTargetSpecification
        + MotuWhollyUpdatableParamsOperation<RegisterDspMeterOutputTarget>,
{
    pub elem_id_list: Vec<ElemId>,
    params: RegisterDspMeterOutputTarget,
    _phantom: PhantomData<T>,
}

const OUTPUT_METER_TARGET_NAME: &str = "output-meter-target";

impl<T> RegisterDspMeterOutputTargetCtl<T>
where
    T: MotuRegisterDspMeterOutputTargetSpecification
        + MotuWhollyUpdatableParamsOperation<RegisterDspMeterOutputTarget>,
{
    pub(crate) fn cache(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let res = T::update_wholly(req, node, &self.params, timeout_ms);
        debug!(params = ?self.params, ?res);
        res
    }

    pub(crate) fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let labels: Vec<String> = T::OUTPUT_PORT_PAIRS
            .iter()
            .map(|p| target_port_to_string(p))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, OUTPUT_METER_TARGET_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, true)
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))
    }

    pub(crate) fn read(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            OUTPUT_METER_TARGET_NAME => {
                elem_value.set_enum(&[self.params.0 as u32]);
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
            OUTPUT_METER_TARGET_NAME => {
                let mut params = self.params.clone();
                params.0 = elem_value.enumerated()[0] as usize;
                let res =
                    T::update_wholly(req, node, &params, timeout_ms).map(|_| self.params = params);
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            _ => Ok(false),
        }
    }
}
