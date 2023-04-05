// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

pub(crate) use super::{common_ctls::PhoneAssignCtlOperation, register_dsp_runtime::*};

pub trait RegisterDspPhoneAssignCtlOperation<T: AssignOperation>:
    PhoneAssignCtlOperation<T>
{
    fn parse_dsp_parameter(&mut self, params: &SndMotuRegisterDspParameter) {
        let idx = params.headphone_output_paired_assignment() as usize;
        if idx < T::ASSIGN_PORTS.len() {
            *self.state_mut() = idx;
        }
    }
}

const MIXER_OUTPUT_VOLUME_NAME: &str = "mixer-output-volume";
const MIXER_OUTPUT_MUTE_NAME: &str = "mixer-output-mute";
const MIXER_OUTPUT_DST_NAME: &str = "mixer-output-destination";

fn copy_int_to_elem_value<T: Copy + Into<i32>>(elem_value: &mut ElemValue, data: &[T]) {
    let vals: Vec<i32> = data.iter().map(|&val| val.into()).collect();
    elem_value.set_int(&vals);
}

pub trait RegisterDspMixerOutputCtlOperation<T: RegisterDspMixerOutputOperation> {
    fn state(&self) -> &RegisterDspMixerOutputState;
    fn state_mut(&mut self) -> &mut RegisterDspMixerOutputState;

    const VOL_TLV: DbInterval = DbInterval {
        min: 0,
        max: 63,
        linear: true,
        mute_avail: false,
    };

    fn load(
        &mut self,
        card_cntr: &mut CardCntr,
        unit: &mut (SndMotu, FwNode),
        req: &mut FwReq,
        timeout_ms: u32,
    ) -> Result<Vec<ElemId>, Error> {
        T::read_mixer_output_state(req, &mut unit.1, self.state_mut(), timeout_ms)?;

        let mut notified_elem_id_list = Vec::new();

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
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIXER_OUTPUT_MUTE_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, 1, T::MIXER_COUNT, true)
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        if T::OUTPUT_DESTINATIONS.len() > 0 {
            let labels: Vec<String> = T::OUTPUT_DESTINATIONS
                .iter()
                .map(|p| target_port_to_string(p))
                .collect();
            let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIXER_OUTPUT_DST_NAME, 0);
            card_cntr
                .add_enum_elems(&elem_id, 1, T::MIXER_COUNT, &labels, None, true)
                .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;
        }

        Ok(notified_elem_id_list)
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            MIXER_OUTPUT_VOLUME_NAME => {
                copy_int_to_elem_value(elem_value, &self.state().volume);
                Ok(true)
            }
            MIXER_OUTPUT_MUTE_NAME => {
                elem_value.set_bool(&self.state().mute);
                Ok(true)
            }
            MIXER_OUTPUT_DST_NAME => {
                ElemValueAccessor::<u32>::set_vals(elem_value, T::MIXER_COUNT, |idx| {
                    let val = T::OUTPUT_DESTINATIONS
                        .iter()
                        .position(|p| self.state().destination[idx].eq(p))
                        .unwrap();
                    Ok(val as u32)
                })
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
            MIXER_OUTPUT_VOLUME_NAME => {
                let vals = &elem_value.int()[..T::MIXER_COUNT];
                let vols: Vec<u8> = vals.iter().map(|&vol| vol as u8).collect();
                T::write_mixer_output_volume(req, &mut unit.1, &vols, self.state_mut(), timeout_ms)
                    .map(|_| true)
            }
            MIXER_OUTPUT_MUTE_NAME => {
                let mute = &elem_value.boolean()[..T::MIXER_COUNT];
                T::write_mixer_output_mute(req, &mut unit.1, &mute, self.state_mut(), timeout_ms)
                    .map(|_| true)
            }
            MIXER_OUTPUT_DST_NAME => {
                let vals = &elem_value.enumerated()[..T::MIXER_COUNT];
                let mut dst = Vec::new();
                vals.iter().try_for_each(|&val| {
                    T::OUTPUT_DESTINATIONS
                        .iter()
                        .nth(val as usize)
                        .ok_or_else(|| {
                            let msg = format!("Invalid index for ourput destination: {}", val);
                            Error::new(FileError::Inval, &msg)
                        })
                        .map(|&port| dst.push(port))
                })?;
                T::write_mixer_output_destination(
                    req,
                    &mut unit.1,
                    &dst,
                    self.state_mut(),
                    timeout_ms,
                )
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }

    fn parse_dsp_parameter(&mut self, params: &SndMotuRegisterDspParameter) {
        T::parse_dsp_parameter(self.state_mut(), params)
    }

    fn parse_dsp_event(&mut self, event: &RegisterDspEvent) -> bool {
        T::parse_dsp_event(self.state_mut(), event)
    }
}

const MIXER_RETURN_ENABLE_NAME: &str = "mixer-return-enable";

pub trait RegisterDspMixerReturnCtlOperation<T: RegisterDspMixerReturnOperation> {
    fn state(&self) -> &bool;
    fn state_mut(&mut self) -> &mut bool;

    fn load(
        &mut self,
        card_cntr: &mut CardCntr,
        unit: &mut (SndMotu, FwNode),
        req: &mut FwReq,
        timeout_ms: u32,
    ) -> Result<Vec<ElemId>, Error> {
        T::read_mixer_return_enable(req, &mut unit.1, self.state_mut(), timeout_ms)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIXER_RETURN_ENABLE_NAME, 0);
        card_cntr.add_bool_elems(&elem_id, 1, 1, true)
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            MIXER_RETURN_ENABLE_NAME => {
                elem_value.set_bool(&[*self.state()]);
                Ok(true)
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
            MIXER_RETURN_ENABLE_NAME => {
                let val = elem_value.boolean()[0];
                T::write_mixer_return_enable(req, &mut unit.1, val, timeout_ms).map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

const MIXER_SOURCE_GAIN_NAME: &str = "mixer-source-gain";
const MIXER_SOURCE_PAN_NAME: &str = "mixer-source-pan";
const MIXER_SOURCE_MUTE_NAME: &str = "mixer-source-mute";
const MIXER_SOURCE_SOLO_NAME: &str = "mixer-source-solo";

pub trait RegisterDspMixerMonauralSourceCtlOperation<T: RegisterDspMixerMonauralSourceOperation> {
    fn state(&self) -> &RegisterDspMixerMonauralSourceState;
    fn state_mut(&mut self) -> &mut RegisterDspMixerMonauralSourceState;

    const GAIN_TLV: DbInterval = DbInterval {
        min: -6400,
        max: 0,
        linear: true,
        mute_avail: false,
    };

    fn load(
        &mut self,
        card_cntr: &mut CardCntr,
        unit: &mut (SndMotu, FwNode),
        req: &mut FwReq,
        timeout_ms: u32,
    ) -> Result<Vec<ElemId>, Error> {
        T::read_mixer_monaural_source_state(req, &mut unit.1, self.state_mut(), timeout_ms)?;

        let mut notified_elem_id_list = Vec::new();

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
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

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
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIXER_SOURCE_MUTE_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, T::MIXER_COUNT, T::MIXER_SOURCES.len(), true)
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIXER_SOURCE_SOLO_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, T::MIXER_COUNT, T::MIXER_SOURCES.len(), true)
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        Ok(notified_elem_id_list)
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            MIXER_SOURCE_GAIN_NAME => {
                let mixer = elem_id.index() as usize;
                copy_int_to_elem_value(elem_value, &self.state().0[mixer].gain);
                Ok(true)
            }
            MIXER_SOURCE_PAN_NAME => {
                let mixer = elem_id.index() as usize;
                copy_int_to_elem_value(elem_value, &self.state().0[mixer].pan);
                Ok(true)
            }
            MIXER_SOURCE_MUTE_NAME => {
                let mixer = elem_id.index() as usize;
                elem_value.set_bool(&self.state().0[mixer].mute);
                Ok(true)
            }
            MIXER_SOURCE_SOLO_NAME => {
                let mixer = elem_id.index() as usize;
                elem_value.set_bool(&self.state().0[mixer].solo);
                Ok(true)
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
            MIXER_SOURCE_GAIN_NAME => {
                let vals = &elem_value.int()[..T::MIXER_SOURCES.len()];
                let gain: Vec<u8> = vals.iter().map(|&val| val as u8).collect();
                let mixer = elem_id.index() as usize;
                T::write_mixer_monaural_source_gain(
                    req,
                    &mut unit.1,
                    mixer,
                    &gain,
                    self.state_mut(),
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
                    &mut unit.1,
                    mixer,
                    &pan,
                    self.state_mut(),
                    timeout_ms,
                )
                .map(|_| true)
            }
            MIXER_SOURCE_MUTE_NAME => {
                let mute = &elem_value.boolean()[..T::MIXER_SOURCES.len()];
                let mixer = elem_id.index() as usize;
                T::write_mixer_monaural_source_mute(
                    req,
                    &mut unit.1,
                    mixer,
                    &mute,
                    self.state_mut(),
                    timeout_ms,
                )
                .map(|_| true)
            }
            MIXER_SOURCE_SOLO_NAME => {
                let solo = &elem_value.boolean()[..T::MIXER_SOURCES.len()];
                let mixer = elem_id.index() as usize;
                T::write_mixer_monaural_source_solo(
                    req,
                    &mut unit.1,
                    mixer,
                    &solo,
                    self.state_mut(),
                    timeout_ms,
                )
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }

    fn parse_dsp_parameter(&mut self, params: &SndMotuRegisterDspParameter) {
        T::parse_dsp_parameter(self.state_mut(), params)
    }

    fn parse_dsp_event(&mut self, event: &RegisterDspEvent) -> bool {
        T::parse_dsp_event(self.state_mut(), event)
    }
}

const MIXER_SOURCE_STEREO_BALANCE_NAME: &str = "mixer-source-stereo-balance";
const MIXER_SOURCE_STEREO_WIDTH_NAME: &str = "mixer-source-stereo-width";

pub trait RegisterDspMixerStereoSourceCtlOperation<T: RegisterDspMixerStereoSourceOperation> {
    fn state(&self) -> &RegisterDspMixerStereoSourceState;
    fn state_mut(&mut self) -> &mut RegisterDspMixerStereoSourceState;

    const GAIN_TLV: DbInterval = DbInterval {
        min: -6400,
        max: 0,
        linear: true,
        mute_avail: false,
    };

    fn load(
        &mut self,
        card_cntr: &mut CardCntr,
        unit: &mut (SndMotu, FwNode),
        req: &mut FwReq,
        params: &SndMotuRegisterDspParameter,
        timeout_ms: u32,
    ) -> Result<Vec<ElemId>, Error> {
        self.parse_dsp_parameter(params);
        T::read_mixer_stereo_source_state(req, &mut unit.1, self.state_mut(), timeout_ms)?;

        let mut notified_elem_id_list = Vec::new();

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
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

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
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIXER_SOURCE_MUTE_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, T::MIXER_COUNT, T::MIXER_SOURCES.len(), true)
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIXER_SOURCE_SOLO_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, T::MIXER_COUNT, T::MIXER_SOURCES.len(), true)
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

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
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

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
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        Ok(notified_elem_id_list)
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            MIXER_SOURCE_GAIN_NAME => {
                let mixer = elem_id.index() as usize;
                copy_int_to_elem_value(elem_value, &self.state().0[mixer].gain);
                Ok(true)
            }
            MIXER_SOURCE_PAN_NAME => {
                let mixer = elem_id.index() as usize;
                copy_int_to_elem_value(elem_value, &self.state().0[mixer].pan);
                Ok(true)
            }
            MIXER_SOURCE_MUTE_NAME => {
                let mixer = elem_id.index() as usize;
                elem_value.set_bool(&self.state().0[mixer].mute);
                Ok(true)
            }
            MIXER_SOURCE_SOLO_NAME => {
                let mixer = elem_id.index() as usize;
                elem_value.set_bool(&self.state().0[mixer].solo);
                Ok(true)
            }
            MIXER_SOURCE_STEREO_BALANCE_NAME => {
                let mixer = elem_id.index() as usize;
                copy_int_to_elem_value(elem_value, &self.state().0[mixer].balance);
                Ok(true)
            }
            MIXER_SOURCE_STEREO_WIDTH_NAME => {
                let mixer = elem_id.index() as usize;
                copy_int_to_elem_value(elem_value, &self.state().0[mixer].width);
                Ok(true)
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
            MIXER_SOURCE_GAIN_NAME => {
                let vals = &elem_value.int()[..T::MIXER_SOURCES.len()];
                let gain: Vec<u8> = vals.iter().map(|&val| val as u8).collect();
                let mixer = elem_id.index() as usize;
                T::write_mixer_stereo_source_gain(
                    req,
                    &mut unit.1,
                    mixer,
                    &gain,
                    self.state_mut(),
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
                    &mut unit.1,
                    mixer,
                    &pan,
                    self.state_mut(),
                    timeout_ms,
                )
                .map(|_| true)
            }
            MIXER_SOURCE_MUTE_NAME => {
                let mute = &elem_value.boolean()[..T::MIXER_SOURCES.len()];
                let mixer = elem_id.index() as usize;
                T::write_mixer_stereo_source_mute(
                    req,
                    &mut unit.1,
                    mixer,
                    &mute,
                    self.state_mut(),
                    timeout_ms,
                )
                .map(|_| true)
            }
            MIXER_SOURCE_SOLO_NAME => {
                let solo = &elem_value.boolean()[..T::MIXER_SOURCES.len()];
                let mixer = elem_id.index() as usize;
                T::write_mixer_stereo_source_mute(
                    req,
                    &mut unit.1,
                    mixer,
                    &solo,
                    self.state_mut(),
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
                    &mut unit.1,
                    mixer,
                    &balance,
                    self.state_mut(),
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
                    &mut unit.1,
                    mixer,
                    &width,
                    self.state_mut(),
                    timeout_ms,
                )
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }

    fn parse_dsp_parameter(&mut self, params: &SndMotuRegisterDspParameter) {
        T::parse_dsp_parameter(self.state_mut(), params)
    }

    fn parse_dsp_event(&mut self, event: &RegisterDspEvent) -> bool {
        T::parse_dsp_event(self.state_mut(), event)
    }
}

const MASTER_OUTPUT_VOLUME_NAME: &str = "master-output-volume";
const PHONE_VOLUME_NAME: &str = "headphone-volume";

pub trait RegisterDspOutputCtlOperation<T: RegisterDspOutputOperation> {
    fn state(&self) -> &RegisterDspOutputState;
    fn state_mut(&mut self) -> &mut RegisterDspOutputState;

    const VOL_TLV: DbInterval = DbInterval {
        min: -6400,
        max: 0,
        linear: true,
        mute_avail: false,
    };

    fn load(
        &mut self,
        card_cntr: &mut CardCntr,
        unit: &mut (SndMotu, FwNode),
        req: &mut FwReq,
        timeout_ms: u32,
    ) -> Result<Vec<ElemId>, Error> {
        T::read_output_state(req, &mut unit.1, self.state_mut(), timeout_ms)?;

        let mut notified_elem_id_list = Vec::new();

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MASTER_OUTPUT_VOLUME_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                T::VOLUME_MIN as i32,
                T::VOLUME_MAX as i32,
                T::VOLUME_STEP as i32,
                1,
                Some(&Vec::<u32>::from(&Self::VOL_TLV)),
                true,
            )
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, PHONE_VOLUME_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                T::VOLUME_MIN as i32,
                T::VOLUME_MAX as i32,
                T::VOLUME_STEP as i32,
                1,
                Some(&Vec::<u32>::from(&Self::VOL_TLV)),
                true,
            )
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        Ok(notified_elem_id_list)
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            MASTER_OUTPUT_VOLUME_NAME => {
                elem_value.set_int(&[self.state().master_volume as i32]);
                Ok(true)
            }
            PHONE_VOLUME_NAME => {
                elem_value.set_int(&[self.state().phone_volume as i32]);
                Ok(true)
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
            MASTER_OUTPUT_VOLUME_NAME => {
                let val = elem_value.int()[0];
                T::write_output_master_volume(
                    req,
                    &mut unit.1,
                    val as u8,
                    self.state_mut(),
                    timeout_ms,
                )
                .map(|_| true)
            }
            PHONE_VOLUME_NAME => {
                let val = elem_value.int()[0];
                T::write_output_phone_volume(
                    req,
                    &mut unit.1,
                    val as u8,
                    self.state_mut(),
                    timeout_ms,
                )
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }

    fn parse_dsp_parameter(&mut self, params: &SndMotuRegisterDspParameter) {
        T::parse_dsp_parameter(self.state_mut(), params)
    }

    fn parse_dsp_event(&mut self, event: &RegisterDspEvent) -> bool {
        T::parse_dsp_event(self.state_mut(), event)
    }
}

const INPUT_NOMINAL_LEVEL_NAME: &str = "input-nominal-level";
const INPUT_BOOST_NAME: &str = "input-boost";

pub trait RegisterDspLineInputCtlOperation<T: Traveler828mk2LineInputOperation> {
    fn state(&self) -> &RegisterDspLineInputState;
    fn state_mut(&mut self) -> &mut RegisterDspLineInputState;

    const NOMINAL_LEVELS: [NominalSignalLevel; 2] = [
        NominalSignalLevel::Consumer,
        NominalSignalLevel::Professional,
    ];

    fn load(
        &mut self,
        card_cntr: &mut CardCntr,
        unit: &mut (SndMotu, FwNode),
        req: &mut FwReq,
        timeout_ms: u32,
    ) -> Result<Vec<ElemId>, Error> {
        T::read_line_input_state(req, &mut unit.1, self.state_mut(), timeout_ms)?;

        let mut notified_elem_id_list = Vec::new();

        let labels: Vec<&str> = Self::NOMINAL_LEVELS
            .iter()
            .map(|l| nominal_signal_level_to_str(l))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, INPUT_NOMINAL_LEVEL_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, T::LINE_INPUT_COUNT, &labels, None, true)
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, INPUT_BOOST_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, 1, T::LINE_INPUT_COUNT, true)
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        Ok(notified_elem_id_list)
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            INPUT_NOMINAL_LEVEL_NAME => {
                ElemValueAccessor::<u32>::set_vals(elem_value, T::LINE_INPUT_COUNT, |idx| {
                    let pos = Self::NOMINAL_LEVELS
                        .iter()
                        .position(|l| self.state().level[idx].eq(l))
                        .unwrap();
                    Ok(pos as u32)
                })
                .map(|_| true)
            }
            INPUT_BOOST_NAME => {
                elem_value.set_bool(&self.state().boost);
                Ok(true)
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
            INPUT_NOMINAL_LEVEL_NAME => {
                let vals = &elem_value.enumerated()[..T::LINE_INPUT_COUNT];
                let mut level = Vec::new();
                vals.iter().try_for_each(|&val| {
                    Self::NOMINAL_LEVELS
                        .iter()
                        .nth(val as usize)
                        .ok_or_else(|| {
                            let msg = format!("Invalid index of nominal signal level: {}", val);
                            Error::new(FileError::Inval, &msg)
                        })
                        .map(|&l| level.push(l))
                })?;
                T::write_line_input_level(req, &mut unit.1, &level, self.state_mut(), timeout_ms)
                    .map(|_| true)
            }
            INPUT_BOOST_NAME => {
                let vals = &elem_value.boolean()[..T::LINE_INPUT_COUNT];
                T::write_line_input_boost(req, &mut unit.1, &vals, self.state_mut(), timeout_ms)
                    .map(|_| true)
            }
            _ => Ok(false),
        }
    }

    fn parse_dsp_parameter(&mut self, params: &SndMotuRegisterDspParameter) {
        T::parse_dsp_parameter(self.state_mut(), params)
    }

    fn parse_dsp_event(&mut self, event: &RegisterDspEvent) -> bool {
        T::parse_dsp_event(self.state_mut(), event)
    }
}

const INPUT_GAIN_NAME: &str = "input-gain";
const INPUT_INVERT_NAME: &str = "input-invert";
const MIC_PHANTOM_NAME: &str = "mic-phantom";
const MIC_PAD_NAME: &str = "mic-pad";
const INPUT_JACK_NAME: &str = "input-jack";
const INPUT_PAIRED_NAME: &str = "input-paired";

pub trait RegisterDspMonauralInputCtlOperation<T: RegisterDspMonauralInputOperation> {
    fn state(&self) -> &RegisterDspMonauralInputState;
    fn state_mut(&mut self) -> &mut RegisterDspMonauralInputState;

    const GAIN_TLV: DbInterval = DbInterval {
        min: 0,
        max: 2400,
        linear: false,
        mute_avail: false,
    };

    fn load(
        &mut self,
        card_cntr: &mut CardCntr,
        unit: &mut (SndMotu, FwNode),
        req: &mut FwReq,
        timeout_ms: u32,
    ) -> Result<Vec<ElemId>, Error> {
        T::read_monaural_input_state(req, &mut unit.1, self.state_mut(), timeout_ms)?;

        let mut notified_elem_id_list = Vec::new();

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
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, INPUT_INVERT_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, 1, T::INPUT_COUNT, true)
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        Ok(notified_elem_id_list)
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            INPUT_GAIN_NAME => {
                copy_int_to_elem_value(elem_value, &self.state().gain);
                Ok(true)
            }
            INPUT_INVERT_NAME => {
                elem_value.set_bool(&self.state().invert);
                Ok(true)
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
            INPUT_GAIN_NAME => {
                let vals = &elem_value.int()[..T::INPUT_COUNT];
                let gain: Vec<u8> = vals.iter().map(|&val| val as u8).collect();
                T::write_monaural_input_gain(req, &mut unit.1, &gain, self.state_mut(), timeout_ms)
                    .map(|_| true)
            }
            INPUT_INVERT_NAME => {
                let invert = &elem_value.boolean()[..T::INPUT_COUNT];
                T::write_monaural_input_invert(
                    req,
                    &mut unit.1,
                    &invert,
                    self.state_mut(),
                    timeout_ms,
                )
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }

    fn parse_dsp_parameter(&mut self, params: &SndMotuRegisterDspParameter) {
        T::parse_dsp_parameter(self.state_mut(), params)
    }

    fn parse_dsp_event(&mut self, event: &RegisterDspEvent) -> bool {
        T::parse_dsp_event(self.state_mut(), event)
    }
}

pub trait RegisterDspStereoInputCtlOperation<T: RegisterDspStereoInputOperation> {
    fn state(&self) -> &RegisterDspStereoInputState;
    fn state_mut(&mut self) -> &mut RegisterDspStereoInputState;

    const GAIN_TLV: DbInterval = DbInterval {
        min: 0,
        max: 6400,
        linear: true,
        mute_avail: false,
    };

    fn load(
        &mut self,
        card_cntr: &mut CardCntr,
        unit: &mut (SndMotu, FwNode),
        req: &mut FwReq,
        timeout_ms: u32,
    ) -> Result<Vec<ElemId>, Error> {
        T::read_stereo_input_state(req, &mut unit.1, self.state_mut(), timeout_ms)?;

        let mut notified_elem_id_list = Vec::new();

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
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, INPUT_INVERT_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, 1, T::INPUT_COUNT, true)
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIC_PHANTOM_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, 1, T::MIC_COUNT, true)
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIC_PAD_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, 1, T::MIC_COUNT, true)
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, INPUT_JACK_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, 1, T::MIC_COUNT, true)
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, INPUT_PAIRED_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, 1, T::INPUT_PAIR_COUNT, true)
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        Ok(notified_elem_id_list)
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            INPUT_GAIN_NAME => {
                copy_int_to_elem_value(elem_value, &self.state().gain);
                Ok(true)
            }
            INPUT_INVERT_NAME => {
                elem_value.set_bool(&self.state().invert);
                Ok(true)
            }
            MIC_PHANTOM_NAME => {
                elem_value.set_bool(&self.state().phantom);
                Ok(true)
            }
            MIC_PAD_NAME => {
                elem_value.set_bool(&self.state().pad);
                Ok(true)
            }
            INPUT_JACK_NAME => {
                elem_value.set_bool(&self.state().jack);
                Ok(true)
            }
            INPUT_PAIRED_NAME => {
                elem_value.set_bool(&self.state().paired);
                Ok(true)
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
            INPUT_GAIN_NAME => {
                let vals = &elem_value.int()[..T::INPUT_COUNT];
                let gain: Vec<u8> = vals.iter().map(|&val| val as u8).collect();
                T::write_stereo_input_gain(req, &mut unit.1, &gain, self.state_mut(), timeout_ms)
                    .map(|_| true)
            }
            INPUT_INVERT_NAME => {
                let invert = &elem_value.boolean()[..T::INPUT_COUNT];
                T::write_stereo_input_invert(
                    req,
                    &mut unit.1,
                    &invert,
                    self.state_mut(),
                    timeout_ms,
                )
                .map(|_| true)
            }
            MIC_PHANTOM_NAME => {
                let phantom = &elem_value.boolean()[..T::MIC_COUNT];
                T::write_mic_phantom(req, &mut unit.1, &phantom, self.state_mut(), timeout_ms)
                    .map(|_| true)
            }
            MIC_PAD_NAME => {
                let pad = &elem_value.boolean()[..T::MIC_COUNT];
                T::write_mic_pad(req, &mut unit.1, &pad, self.state_mut(), timeout_ms).map(|_| true)
            }
            INPUT_PAIRED_NAME => {
                let paired = &elem_value.boolean()[..T::INPUT_PAIR_COUNT];
                T::write_stereo_input_paired(
                    req,
                    &mut unit.1,
                    &paired,
                    self.state_mut(),
                    timeout_ms,
                )
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }

    fn parse_dsp_parameter(&mut self, params: &SndMotuRegisterDspParameter) {
        T::parse_dsp_parameter(self.state_mut(), params)
    }

    fn parse_dsp_event(&mut self, event: &RegisterDspEvent) -> bool {
        T::parse_dsp_event(self.state_mut(), event)
    }
}

pub struct RegisterDspMeterImage([u8; 48]);

impl Default for RegisterDspMeterImage {
    fn default() -> Self {
        Self([0u8; 48])
    }
}

const INPUT_METER_NAME: &str = "input-meter";
const OUTPUT_METER_NAME: &str = "output-meter";
const OUTPUT_METER_TARGET_NAME: &str = "output-meter-target";

pub trait RegisterDspMeterCtlOperation<T: RegisterDspMeterOperation> {
    fn state(&self) -> &RegisterDspMeterState;
    fn state_mut(&mut self) -> &mut RegisterDspMeterState;

    fn load(
        &mut self,
        card_cntr: &mut CardCntr,
        unit: &mut (SndMotu, FwNode),
        req: &mut FwReq,
        timeout_ms: u32,
    ) -> Result<Vec<ElemId>, Error> {
        if T::SELECTABLE {
            T::select_output(req, &mut unit.1, 0, self.state_mut(), timeout_ms)?;
        }

        let mut measured_elem_id_list = Vec::new();

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
            .map(|mut elem_id_list| measured_elem_id_list.append(&mut elem_id_list))?;

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
            .map(|mut elem_id_list| measured_elem_id_list.append(&mut elem_id_list))?;

        if T::SELECTABLE {
            let labels: Vec<String> = T::OUTPUT_PORT_PAIRS
                .iter()
                .map(|(p, _)| target_port_to_string(p))
                .collect();
            let elem_id =
                ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, OUTPUT_METER_TARGET_NAME, 0);
            card_cntr
                .add_enum_elems(&elem_id, 1, 1, &labels, None, true)
                .map(|mut elem_id_list| measured_elem_id_list.append(&mut elem_id_list))?;
        }

        Ok(measured_elem_id_list)
    }

    fn read(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            INPUT_METER_NAME => {
                copy_int_to_elem_value(elem_value, &self.state().inputs);
                Ok(true)
            }
            OUTPUT_METER_NAME => {
                copy_int_to_elem_value(elem_value, &self.state().outputs);
                Ok(true)
            }
            OUTPUT_METER_TARGET_NAME => {
                if T::SELECTABLE {
                    if let Some(selected) = self.state().selected {
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

    fn write(
        &mut self,
        unit: &mut (SndMotu, FwNode),
        req: &mut FwReq,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            OUTPUT_METER_TARGET_NAME => {
                if T::SELECTABLE {
                    let target = elem_value.enumerated()[0] as usize;
                    if target < T::OUTPUT_PORT_PAIRS.len() {
                        T::select_output(req, &mut unit.1, target, self.state_mut(), timeout_ms)
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

    fn read_dsp_meter(
        &mut self,
        unit: &SndMotu,
        image: &mut RegisterDspMeterImage,
    ) -> Result<(), Error> {
        unit.read_byte_meter(&mut image.0)
            .map(|_| T::parse_dsp_meter(self.state_mut(), &image.0))
    }
}
