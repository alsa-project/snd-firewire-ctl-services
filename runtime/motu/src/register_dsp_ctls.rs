// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

pub(crate) use super::{common_ctls::PhoneAssignCtl, register_dsp_runtime::*};

#[derive(Default, Debug)]
pub(crate) struct RegisterDspPhoneAssignCtl<T>(pub PhoneAssignCtl<T>)
where
    T: MotuPortAssignSpecification
        + MotuWhollyCacheableParamsOperation<PhoneAssignParameters>
        + MotuWhollyUpdatableParamsOperation<PhoneAssignParameters>;

impl<T> RegisterDspPhoneAssignCtl<T>
where
    T: MotuPortAssignSpecification
        + MotuWhollyCacheableParamsOperation<PhoneAssignParameters>
        + MotuWhollyUpdatableParamsOperation<PhoneAssignParameters>,
{
    pub(crate) fn parse_dsp_parameter(&mut self, params: &SndMotuRegisterDspParameter) {
        let idx = params.headphone_output_paired_assignment() as usize;
        let _ = T::ASSIGN_PORT_TARGETS
            .iter()
            .nth(idx)
            .map(|&p| self.0.params.0 = p);
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
        T::cache_wholly(req, node, &mut self.params, timeout_ms)
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
                res.map(|_| true)
            }
            MIXER_OUTPUT_MUTE_NAME => {
                let mut params = self.params.clone();
                let vals = &elem_value.boolean()[..T::MIXER_COUNT];
                params.mute.copy_from_slice(vals);
                let res = T::update_partially(req, node, &mut self.params, params, timeout_ms);
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
                res.map(|_| true)
            }
            _ => Ok(false),
        }
    }

    pub(crate) fn parse_dsp_parameter(&mut self, image: &SndMotuRegisterDspParameter) {
        T::parse_image(&mut self.params, image)
    }

    pub(crate) fn parse_dsp_event(&mut self, event: &RegisterDspEvent) -> bool {
        T::parse_event(&mut self.params, event)
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
        T::cache_wholly(req, node, &mut self.params, timeout_ms)
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
                res.map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

#[derive(Debug)]
pub(crate) struct RegisterDspMixerMonauralSourceCtl<T: RegisterDspMixerMonauralSourceOperation> {
    pub elem_id_list: Vec<ElemId>,
    state: RegisterDspMixerMonauralSourceState,
    _phantom: PhantomData<T>,
}

const MIXER_SOURCE_GAIN_NAME: &str = "mixer-source-gain";
const MIXER_SOURCE_PAN_NAME: &str = "mixer-source-pan";
const MIXER_SOURCE_MUTE_NAME: &str = "mixer-source-mute";
const MIXER_SOURCE_SOLO_NAME: &str = "mixer-source-solo";

impl<T: RegisterDspMixerMonauralSourceOperation> Default for RegisterDspMixerMonauralSourceCtl<T> {
    fn default() -> Self {
        Self {
            elem_id_list: Default::default(),
            state: T::create_mixer_monaural_source_state(),
            _phantom: Default::default(),
        }
    }
}

impl<T: RegisterDspMixerMonauralSourceOperation> RegisterDspMixerMonauralSourceCtl<T> {
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
        T::read_mixer_monaural_source_state(req, node, &mut self.state, timeout_ms)
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
                copy_int_to_elem_value(elem_value, &self.state.0[mixer].gain);
                Ok(true)
            }
            MIXER_SOURCE_PAN_NAME => {
                let mixer = elem_id.index() as usize;
                copy_int_to_elem_value(elem_value, &self.state.0[mixer].pan);
                Ok(true)
            }
            MIXER_SOURCE_MUTE_NAME => {
                let mixer = elem_id.index() as usize;
                elem_value.set_bool(&self.state.0[mixer].mute);
                Ok(true)
            }
            MIXER_SOURCE_SOLO_NAME => {
                let mixer = elem_id.index() as usize;
                elem_value.set_bool(&self.state.0[mixer].solo);
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
                let vals = &elem_value.int()[..T::MIXER_SOURCES.len()];
                let gain: Vec<u8> = vals.iter().map(|&val| val as u8).collect();
                let mixer = elem_id.index() as usize;
                T::write_mixer_monaural_source_gain(
                    req,
                    node,
                    mixer,
                    &gain,
                    &mut self.state,
                    timeout_ms,
                )
                .map(|_| true)
            }
            MIXER_SOURCE_PAN_NAME => {
                let vals = &elem_value.int()[..T::MIXER_SOURCES.len()];
                let pan: Vec<u8> = vals.iter().map(|&val| val as u8).collect();
                let mixer = elem_id.index() as usize;
                T::write_mixer_monaural_source_pan(
                    req,
                    node,
                    mixer,
                    &pan,
                    &mut self.state,
                    timeout_ms,
                )
                .map(|_| true)
            }
            MIXER_SOURCE_MUTE_NAME => {
                let mute = &elem_value.boolean()[..T::MIXER_SOURCES.len()];
                let mixer = elem_id.index() as usize;
                T::write_mixer_monaural_source_mute(
                    req,
                    node,
                    mixer,
                    &mute,
                    &mut self.state,
                    timeout_ms,
                )
                .map(|_| true)
            }
            MIXER_SOURCE_SOLO_NAME => {
                let solo = &elem_value.boolean()[..T::MIXER_SOURCES.len()];
                let mixer = elem_id.index() as usize;
                T::write_mixer_monaural_source_solo(
                    req,
                    node,
                    mixer,
                    &solo,
                    &mut self.state,
                    timeout_ms,
                )
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }

    pub(crate) fn parse_dsp_parameter(&mut self, params: &SndMotuRegisterDspParameter) {
        T::parse_dsp_parameter(&mut self.state, params)
    }

    pub(crate) fn parse_dsp_event(&mut self, event: &RegisterDspEvent) -> bool {
        T::parse_dsp_event(&mut self.state, event)
    }
}

#[derive(Debug)]
pub(crate) struct RegisterDspMixerStereoSourceCtl<T: RegisterDspMixerStereoSourceOperation> {
    pub elem_id_list: Vec<ElemId>,
    state: RegisterDspMixerStereoSourceState,
    _phantom: PhantomData<T>,
}

const MIXER_SOURCE_STEREO_BALANCE_NAME: &str = "mixer-source-stereo-balance";
const MIXER_SOURCE_STEREO_WIDTH_NAME: &str = "mixer-source-stereo-width";

impl<T: RegisterDspMixerStereoSourceOperation> Default for RegisterDspMixerStereoSourceCtl<T> {
    fn default() -> Self {
        Self {
            elem_id_list: Default::default(),
            state: T::create_mixer_stereo_source_state(),
            _phantom: Default::default(),
        }
    }
}

impl<T: RegisterDspMixerStereoSourceOperation> RegisterDspMixerStereoSourceCtl<T> {
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
        T::read_mixer_stereo_source_state(req, node, &mut self.state, timeout_ms)
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
                copy_int_to_elem_value(elem_value, &self.state.0[mixer].gain);
                Ok(true)
            }
            MIXER_SOURCE_PAN_NAME => {
                let mixer = elem_id.index() as usize;
                copy_int_to_elem_value(elem_value, &self.state.0[mixer].pan);
                Ok(true)
            }
            MIXER_SOURCE_MUTE_NAME => {
                let mixer = elem_id.index() as usize;
                elem_value.set_bool(&self.state.0[mixer].mute);
                Ok(true)
            }
            MIXER_SOURCE_SOLO_NAME => {
                let mixer = elem_id.index() as usize;
                elem_value.set_bool(&self.state.0[mixer].solo);
                Ok(true)
            }
            MIXER_SOURCE_STEREO_BALANCE_NAME => {
                let mixer = elem_id.index() as usize;
                copy_int_to_elem_value(elem_value, &self.state.0[mixer].balance);
                Ok(true)
            }
            MIXER_SOURCE_STEREO_WIDTH_NAME => {
                let mixer = elem_id.index() as usize;
                copy_int_to_elem_value(elem_value, &self.state.0[mixer].width);
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
                let vals = &elem_value.int()[..T::MIXER_SOURCES.len()];
                let gain: Vec<u8> = vals.iter().map(|&val| val as u8).collect();
                let mixer = elem_id.index() as usize;
                T::write_mixer_stereo_source_gain(
                    req,
                    node,
                    mixer,
                    &gain,
                    &mut self.state,
                    timeout_ms,
                )
                .map(|_| true)
            }
            MIXER_SOURCE_PAN_NAME => {
                let vals = &elem_value.int()[..T::MIXER_SOURCES.len()];
                let pan: Vec<u8> = vals.iter().map(|&val| val as u8).collect();
                let mixer = elem_id.index() as usize;
                T::write_mixer_stereo_source_pan(
                    req,
                    node,
                    mixer,
                    &pan,
                    &mut self.state,
                    timeout_ms,
                )
                .map(|_| true)
            }
            MIXER_SOURCE_MUTE_NAME => {
                let mute = &elem_value.boolean()[..T::MIXER_SOURCES.len()];
                let mixer = elem_id.index() as usize;
                T::write_mixer_stereo_source_mute(
                    req,
                    node,
                    mixer,
                    &mute,
                    &mut self.state,
                    timeout_ms,
                )
                .map(|_| true)
            }
            MIXER_SOURCE_SOLO_NAME => {
                let solo = &elem_value.boolean()[..T::MIXER_SOURCES.len()];
                let mixer = elem_id.index() as usize;
                T::write_mixer_stereo_source_mute(
                    req,
                    node,
                    mixer,
                    &solo,
                    &mut self.state,
                    timeout_ms,
                )
                .map(|_| true)
            }
            MIXER_SOURCE_STEREO_BALANCE_NAME => {
                let vals = &elem_value.int()[..T::MIXER_SOURCE_PAIR_COUNT];
                let balance: Vec<u8> = vals.iter().map(|&val| val as u8).collect();
                let mixer = elem_id.index() as usize;
                T::write_mixer_stereo_source_balance(
                    req,
                    node,
                    mixer,
                    &balance,
                    &mut self.state,
                    timeout_ms,
                )
                .map(|_| true)
            }
            MIXER_SOURCE_STEREO_WIDTH_NAME => {
                let vals = &elem_value.int()[..T::MIXER_SOURCE_PAIR_COUNT];
                let width: Vec<u8> = vals.iter().map(|&val| val as u8).collect();
                let mixer = elem_id.index() as usize;
                T::write_mixer_stereo_source_width(
                    req,
                    node,
                    mixer,
                    &width,
                    &mut self.state,
                    timeout_ms,
                )
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }

    pub(crate) fn parse_dsp_parameter(&mut self, params: &SndMotuRegisterDspParameter) {
        T::parse_dsp_parameter(&mut self.state, params)
    }

    pub(crate) fn parse_dsp_event(&mut self, event: &RegisterDspEvent) -> bool {
        T::parse_dsp_event(&mut self.state, event)
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
        T::cache_wholly(req, node, &mut self.params, timeout_ms)
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
                res.map(|_| true)
            }
            PHONE_VOLUME_NAME => {
                let mut params = self.params.clone();
                params.phone_volume = elem_value.int()[0] as u8;
                let res = T::update_partially(req, node, &mut self.params, params, timeout_ms)
                    .map(|_| self.params = params);
                res.map(|_| true)
            }
            _ => Ok(false),
        }
    }

    pub(crate) fn parse_dsp_parameter(&mut self, image: &SndMotuRegisterDspParameter) {
        T::parse_image(&mut self.params, image)
    }

    pub(crate) fn parse_dsp_event(&mut self, event: &RegisterDspEvent) -> bool {
        T::parse_event(&mut self.params, event)
    }
}

#[derive(Debug)]
pub(crate) struct RegisterDspLineInputCtl<T: Traveler828mk2LineInputOperation> {
    pub elem_id_list: Vec<ElemId>,
    state: RegisterDspLineInputState,
    _phantom: PhantomData<T>,
}

const INPUT_NOMINAL_LEVEL_NAME: &str = "input-nominal-level";
const INPUT_BOOST_NAME: &str = "input-boost";

impl<T: Traveler828mk2LineInputOperation> Default for RegisterDspLineInputCtl<T> {
    fn default() -> Self {
        Self {
            elem_id_list: Default::default(),
            state: T::create_line_input_state(),
            _phantom: Default::default(),
        }
    }
}

impl<T: Traveler828mk2LineInputOperation> RegisterDspLineInputCtl<T> {
    const NOMINAL_LEVELS: [NominalSignalLevel; 2] = [
        NominalSignalLevel::Consumer,
        NominalSignalLevel::Professional,
    ];

    pub(crate) fn cache(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        T::read_line_input_state(req, node, &mut self.state, timeout_ms)
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
                    .state
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
                elem_value.set_bool(&self.state.boost);
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
                let mut level = Vec::new();
                elem_value
                    .enumerated()
                    .iter()
                    .take(T::LINE_INPUT_COUNT)
                    .try_for_each(|&val| {
                        Self::NOMINAL_LEVELS
                            .iter()
                            .nth(val as usize)
                            .ok_or_else(|| {
                                let msg = format!("Invalid index of nominal signal level: {}", val);
                                Error::new(FileError::Inval, &msg)
                            })
                            .map(|&l| level.push(l))
                    })?;
                T::write_line_input_level(req, node, &level, &mut self.state, timeout_ms)?;
                Ok(true)
            }
            INPUT_BOOST_NAME => {
                let vals = &elem_value.boolean()[..T::LINE_INPUT_COUNT];
                T::write_line_input_boost(req, node, &vals, &mut self.state, timeout_ms)?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub(crate) fn parse_dsp_parameter(&mut self, params: &SndMotuRegisterDspParameter) {
        T::parse_dsp_parameter(&mut self.state, params)
    }

    pub(crate) fn parse_dsp_event(&mut self, event: &RegisterDspEvent) -> bool {
        T::parse_dsp_event(&mut self.state, event)
    }
}

#[derive(Debug)]
pub(crate) struct RegisterDspMonauralInputCtl<T: RegisterDspMonauralInputOperation> {
    pub elem_id_list: Vec<ElemId>,
    state: RegisterDspMonauralInputState,
    _phantom: PhantomData<T>,
}

const INPUT_GAIN_NAME: &str = "input-gain";
const INPUT_INVERT_NAME: &str = "input-invert";
const MIC_PHANTOM_NAME: &str = "mic-phantom";
const MIC_PAD_NAME: &str = "mic-pad";
const INPUT_JACK_NAME: &str = "input-jack";
const INPUT_PAIRED_NAME: &str = "input-paired";

impl<T: RegisterDspMonauralInputOperation> Default for RegisterDspMonauralInputCtl<T> {
    fn default() -> Self {
        Self {
            elem_id_list: Default::default(),
            state: T::create_monaural_input_state(),
            _phantom: Default::default(),
        }
    }
}

impl<T: RegisterDspMonauralInputOperation> RegisterDspMonauralInputCtl<T> {
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
        T::read_monaural_input_state(req, node, &mut self.state, timeout_ms)
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
                copy_int_to_elem_value(elem_value, &self.state.gain);
                Ok(true)
            }
            INPUT_INVERT_NAME => {
                elem_value.set_bool(&self.state.invert);
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
                let vals = &elem_value.int()[..T::INPUT_COUNT];
                let gain: Vec<u8> = vals.iter().map(|&val| val as u8).collect();
                T::write_monaural_input_gain(req, node, &gain, &mut self.state, timeout_ms)?;
                Ok(true)
            }
            INPUT_INVERT_NAME => {
                let invert = &elem_value.boolean()[..T::INPUT_COUNT];
                T::write_monaural_input_invert(req, node, &invert, &mut self.state, timeout_ms)?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub(crate) fn parse_dsp_parameter(&mut self, params: &SndMotuRegisterDspParameter) {
        T::parse_dsp_parameter(&mut self.state, params)
    }

    pub(crate) fn parse_dsp_event(&mut self, event: &RegisterDspEvent) -> bool {
        T::parse_dsp_event(&mut self.state, event)
    }
}

#[derive(Debug)]
pub(crate) struct RegisterDspStereoInputCtl<T: RegisterDspStereoInputOperation> {
    pub elem_id_list: Vec<ElemId>,
    state: RegisterDspStereoInputState,
    _phantom: PhantomData<T>,
}

impl<T: RegisterDspStereoInputOperation> Default for RegisterDspStereoInputCtl<T> {
    fn default() -> Self {
        Self {
            elem_id_list: Default::default(),
            state: T::create_stereo_input_state(),
            _phantom: Default::default(),
        }
    }
}

impl<T: RegisterDspStereoInputOperation> RegisterDspStereoInputCtl<T> {
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
        T::read_stereo_input_state(req, node, &mut self.state, timeout_ms)
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
                copy_int_to_elem_value(elem_value, &self.state.gain);
                Ok(true)
            }
            INPUT_INVERT_NAME => {
                elem_value.set_bool(&self.state.invert);
                Ok(true)
            }
            MIC_PHANTOM_NAME => {
                elem_value.set_bool(&self.state.phantom);
                Ok(true)
            }
            MIC_PAD_NAME => {
                elem_value.set_bool(&self.state.pad);
                Ok(true)
            }
            INPUT_JACK_NAME => {
                elem_value.set_bool(&self.state.jack);
                Ok(true)
            }
            INPUT_PAIRED_NAME => {
                elem_value.set_bool(&self.state.paired);
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
                let vals = &elem_value.int()[..T::INPUT_COUNT];
                let gain: Vec<u8> = vals.iter().map(|&val| val as u8).collect();
                T::write_stereo_input_gain(req, node, &gain, &mut self.state, timeout_ms)?;
                Ok(true)
            }
            INPUT_INVERT_NAME => {
                let invert = &elem_value.boolean()[..T::INPUT_COUNT];
                T::write_stereo_input_invert(req, node, &invert, &mut self.state, timeout_ms)?;
                Ok(true).map(|_| true)
            }
            MIC_PHANTOM_NAME => {
                let phantom = &elem_value.boolean()[..T::MIC_COUNT];
                T::write_mic_phantom(req, node, &phantom, &mut self.state, timeout_ms)?;
                Ok(true)
            }
            MIC_PAD_NAME => {
                let pad = &elem_value.boolean()[..T::MIC_COUNT];
                T::write_mic_pad(req, node, &pad, &mut self.state, timeout_ms)?;
                Ok(true)
            }
            INPUT_PAIRED_NAME => {
                let paired = &elem_value.boolean()[..T::INPUT_PAIR_COUNT];
                T::write_stereo_input_paired(req, node, &paired, &mut self.state, timeout_ms)?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub(crate) fn parse_dsp_parameter(&mut self, params: &SndMotuRegisterDspParameter) {
        T::parse_dsp_parameter(&mut self.state, params)
    }

    pub(crate) fn parse_dsp_event(&mut self, event: &RegisterDspEvent) -> bool {
        T::parse_dsp_event(&mut self.state, event)
    }
}

#[derive(Debug)]
pub(crate) struct RegisterDspMeterCtl<T: RegisterDspMeterOperation> {
    pub elem_id_list: Vec<ElemId>,
    state: RegisterDspMeterState,
    image: [u8; 48],
    _phantom: PhantomData<T>,
}

const INPUT_METER_NAME: &str = "input-meter";
const OUTPUT_METER_NAME: &str = "output-meter";
const OUTPUT_METER_TARGET_NAME: &str = "output-meter-target";

impl<T: RegisterDspMeterOperation> Default for RegisterDspMeterCtl<T> {
    fn default() -> Self {
        Self {
            elem_id_list: Default::default(),
            state: T::create_meter_state(),
            image: [0; 48],
            _phantom: Default::default(),
        }
    }
}

impl<T: RegisterDspMeterOperation> RegisterDspMeterCtl<T> {
    pub(crate) fn cache(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        if T::SELECTABLE {
            T::select_output(req, node, 0, &mut self.state, timeout_ms)?;
        }

        Ok(())
    }

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

        if T::SELECTABLE {
            let labels: Vec<String> = T::OUTPUT_PORT_PAIRS
                .iter()
                .map(|(p, _)| target_port_to_string(p))
                .collect();
            let elem_id =
                ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, OUTPUT_METER_TARGET_NAME, 0);
            card_cntr
                .add_enum_elems(&elem_id, 1, 1, &labels, None, true)
                .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;
        }

        Ok(())
    }

    pub(crate) fn read(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            INPUT_METER_NAME => {
                copy_int_to_elem_value(elem_value, &self.state.inputs);
                Ok(true)
            }
            OUTPUT_METER_NAME => {
                copy_int_to_elem_value(elem_value, &self.state.outputs);
                Ok(true)
            }
            OUTPUT_METER_TARGET_NAME => {
                if T::SELECTABLE {
                    if let Some(selected) = self.state.selected {
                        elem_value.set_enum(&[selected as u32]);
                        Ok(true)
                    } else {
                        unreachable!();
                    }
                } else {
                    Err(Error::new(FileError::Inval, "Not supported"))
                }
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
                if T::SELECTABLE {
                    let target = elem_value.enumerated()[0] as usize;
                    if target < T::OUTPUT_PORT_PAIRS.len() {
                        T::select_output(req, node, target, &mut self.state, timeout_ms)
                            .map(|_| true)
                    } else {
                        let msg = format!("Invalid index for output meter pair: {}", target);
                        Err(Error::new(FileError::Inval, &msg))
                    }
                } else {
                    Err(Error::new(FileError::Inval, "Not supported"))
                }
            }
            _ => Ok(false),
        }
    }

    pub(crate) fn read_dsp_meter(&mut self, unit: &SndMotu) -> Result<(), Error> {
        unit.read_byte_meter(&mut self.image)?;
        T::parse_dsp_meter(&mut self.state, &self.image);
        Ok(())
    }
}
