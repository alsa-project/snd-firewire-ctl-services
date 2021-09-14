// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto
use glib::Error;

use hinawa::FwReq;
use hinawa::{SndUnit, SndUnitExt};

use alsactl::{ElemId, ElemIfaceType, ElemValue, ElemValueExt, ElemValueExtManual};

use alsa_ctl_tlv_codec::items::DbInterval;

use core::card_cntr::*;

use ff_protocols::former::*;

const VOL_MIN: i32 = 0x00000000;
const VOL_ZERO: i32 = 0x00008000;
const VOL_MAX: i32 = 0x00010000;
const VOL_STEP: i32 = 1;
const VOL_TLV: DbInterval = DbInterval{min: -9000, max: 600, linear: false, mute_avail: false};

const VOL_NAME: &str = "output-volume";

pub trait FormerOutputCtlOperation<U, V>
    where
        U: RmeFormerOutputOperation<V>,
        V: AsRef<[i32]> + AsMut<[i32]>,
{
    fn state(&self) -> &V;
    fn state_mut(&mut self) -> &mut V;

    fn load(
        &mut self,
        unit: &mut SndUnit,
        req: &mut FwReq,
        card_cntr: &mut CardCntr,
        timeout_ms: u32
    ) -> Result<(), Error> {
        self.state_mut().as_mut().iter_mut()
            .for_each(|vol| *vol = VOL_ZERO);
        U::init_output_vols(req, &mut unit.get_node(), &self.state(), timeout_ms)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, VOL_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            1,
            VOL_MIN,
            VOL_MAX,
            VOL_STEP,
            self.state().as_ref().len(),
            Some(&Vec::<u32>::from(&VOL_TLV)),
            true
        )?;

        Ok(())
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            VOL_NAME => {
                elem_value.set_int(&mut self.state().as_ref());
                Ok(true)
            },
            _ => Ok(false),
        }
    }

    fn write(
        &mut self,
        unit: &mut SndUnit,
        req: &mut FwReq,
        elem_id: &ElemId,
        new: &ElemValue,
        timeout_ms: u32
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            VOL_NAME => {
                let mut vals = self.state().as_ref().to_vec();
                new.get_int(&mut vals);
                U::write_output_vols(req, &mut unit.get_node(), self.state_mut(), &vals, timeout_ms)
                    .map(|_| true)
            },
            _ => Ok(false),
        }
    }
}

const GAIN_MIN: i32 = 0x00000000;
const GAIN_ZERO: i32 = 0x00008000;
const GAIN_MAX: i32 = 0x00010000;
const GAIN_STEP: i32 = 1;
const GAIN_TLV: DbInterval = DbInterval{min: -9000, max: 600, linear: false, mute_avail: false};

const ANALOG_SRC_GAIN_NAME: &str = "mixer:analog-source-gain";
const SPDIF_SRC_GAIN_NAME: &str = "mixer:spdif-source-gain";
const ADAT_SRC_GAIN_NAME: &str = "mixer:adat-source-gain";
const STREAM_SRC_GAIN_NAME: &str = "mixer:stream-source-gain";

pub trait FormerMixerCtlOperation<U, V>
where
    U: RmeFormerMixerOperation<V>,
    V: RmeFormerMixerSpec + AsRef<[FormerMixerSrc]> + AsMut<[FormerMixerSrc]>,
{
    fn state(&self) -> &V;
    fn state_mut(&mut self) -> &mut V;

    fn load(
        &mut self,
        unit: &SndUnit,
        req: &mut FwReq,
        card_cntr: &mut CardCntr,
        timeout_ms: u32
    ) -> Result<(), Error> {
        self.state_mut().as_mut().iter_mut()
            .enumerate()
            .for_each(|(i, mixer)| {
                mixer.analog_gains.iter_mut()
                    .for_each(|gain| *gain = GAIN_MIN);
                mixer.spdif_gains.iter_mut()
                    .for_each(|gain| *gain = GAIN_MIN);
                mixer.adat_gains.iter_mut()
                    .for_each(|gain| *gain = GAIN_MIN);
                mixer.stream_gains.iter_mut()
                    .nth(i)
                    .map(|gain| *gain = GAIN_ZERO);
            });

        (0..self.state().as_ref().len())
            .try_for_each(|i| {
                U::init_mixer_src_gains(req, &mut unit.get_node(), self.state_mut(), i, timeout_ms)
            })?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, ANALOG_SRC_GAIN_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            V::DST_COUNT,
            GAIN_MIN,
            GAIN_MAX,
            GAIN_STEP,
            V::ANALOG_INPUT_COUNT,
            Some(&Vec::<u32>::from(&GAIN_TLV)),
            true
        )?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, SPDIF_SRC_GAIN_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            V::DST_COUNT,
            GAIN_MIN,
            GAIN_MAX,
            GAIN_STEP,
            V::SPDIF_INPUT_COUNT,
            Some(&Vec::<u32>::from(&GAIN_TLV)),
            true
        )?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, ADAT_SRC_GAIN_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            V::DST_COUNT,
            GAIN_MIN,
            GAIN_MAX,
            GAIN_STEP,
            V::ADAT_INPUT_COUNT,
            Some(&Vec::<u32>::from(&GAIN_TLV)),
            true
        )?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, STREAM_SRC_GAIN_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            V::DST_COUNT,
            GAIN_MIN,
            GAIN_MAX,
            GAIN_STEP,
            V::STREAM_INPUT_COUNT,
            Some(&Vec::<u32>::from(&GAIN_TLV)),
            true
        )?;

        Ok(())
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            ANALOG_SRC_GAIN_NAME => {
                let index = elem_id.get_index() as usize;
                elem_value.set_int(&self.state().as_ref()[index].analog_gains);
                Ok(true)
            }
            SPDIF_SRC_GAIN_NAME => {
                let index = elem_id.get_index() as usize;
                elem_value.set_int(&self.state().as_ref()[index].spdif_gains);
                Ok(true)
            }
            ADAT_SRC_GAIN_NAME => {
                let index = elem_id.get_index() as usize;
                elem_value.set_int(&self.state().as_ref()[index].adat_gains);
                Ok(true)
            }
            STREAM_SRC_GAIN_NAME => {
                let index = elem_id.get_index() as usize;
                elem_value.set_int(&self.state().as_ref()[index].stream_gains);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write(
        &mut self,
        unit: &SndUnit,
        req: &mut FwReq,
        elem_id: &ElemId,
        new: &ElemValue,
        timeout_ms: u32
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            ANALOG_SRC_GAIN_NAME => {
                let index = elem_id.get_index() as usize;
                let mut gains = self.state_mut().as_mut()[index].analog_gains.clone();
                new.get_int(&mut gains);
                U::write_mixer_analog_gains(
                    req,
                    &mut unit.get_node(),
                    self.state_mut(),
                    index,
                    &gains,
                    timeout_ms
                )
                    .map(|_| true)
            }
            SPDIF_SRC_GAIN_NAME => {
                let index = elem_id.get_index() as usize;
                let mut gains = self.state_mut().as_mut()[index].spdif_gains.clone();
                new.get_int(&mut gains);
                U::write_mixer_spdif_gains(
                    req,
                    &mut unit.get_node(),
                    self.state_mut(),
                    index,
                    &gains,
                    timeout_ms
                )
                    .map(|_| true)
            }
            ADAT_SRC_GAIN_NAME => {
                let index = elem_id.get_index() as usize;
                let mut gains = self.state_mut().as_mut()[index].adat_gains.clone();
                new.get_int(&mut gains);
                U::write_mixer_adat_gains(
                    req,
                    &mut unit.get_node(),
                    self.state_mut(),
                    index,
                    &gains,
                    timeout_ms
                )
                    .map(|_| true)
            }
            STREAM_SRC_GAIN_NAME => {
                let index = elem_id.get_index() as usize;
                let mut gains = self.state_mut().as_mut()[index].stream_gains.clone();
                new.get_int(&mut gains);
                U::write_mixer_stream_gains(
                    req,
                    &mut unit.get_node(),
                    self.state_mut(),
                    index,
                    &gains,
                    timeout_ms
                )
                    .map(|_| true)
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

pub trait FormerMeterCtlOperation<T: RmeFfFormerMeterOperation> {
    fn meter(&self) -> &FormerMeterState;
    fn meter_mut(&mut self) -> &mut FormerMeterState;

    const LEVEL_TLV: DbInterval = DbInterval{min: -9003, max: 600, linear: false, mute_avail: false};

    fn load(
        &mut self,
        unit: &mut SndUnit,
        req: &mut FwReq,
        card_cntr: &mut CardCntr,
        timeout_ms: u32
    ) -> Result<Vec<ElemId>, Error> {
        let mut meter = T::create_meter_state();
        T::read_meter(req, &mut unit.get_node(), &mut meter, timeout_ms)?;
        *self.meter_mut() = meter;

        let mut measured_elem_id_list = Vec::new();

        [
            (ANALOG_INPUT_NAME, T::ANALOG_INPUT_COUNT),
            (SPDIF_INPUT_NAME, T::SPDIF_INPUT_COUNT),
            (ADAT_INPUT_NAME, T::ADAT_INPUT_COUNT),
            (STREAM_INPUT_NAME, T::STREAM_INPUT_COUNT),
            (ANALOG_OUTPUT_NAME, T::ANALOG_OUTPUT_COUNT),
            (SPDIF_OUTPUT_NAME, T::SPDIF_OUTPUT_COUNT),
            (ADAT_OUTPUT_NAME, T::ADAT_OUTPUT_COUNT),
        ].iter()
            .try_for_each(|&(name, count)| {
                let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
                card_cntr.add_int_elems(
                    &elem_id,
                    1,
                    T::LEVEL_MIN,
                    T::LEVEL_MAX,
                    T::LEVEL_STEP,
                    count,
                    Some(&Vec::<u32>::from(&Self::LEVEL_TLV)),
                    false
                )
                    .map(|mut elem_id_list| measured_elem_id_list.append(&mut elem_id_list))
            })
                .map(|_| measured_elem_id_list)
    }

    fn measure_states(
        &mut self,
        unit: &SndUnit,
        req: &mut FwReq,
        timeout_ms: u32
    ) -> Result<(), Error> {
        T::read_meter(req, &mut unit.get_node(), self.meter_mut(), timeout_ms)
    }

    fn measure_elem(&self, elem_id: &ElemId, elem_value: &ElemValue) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            ANALOG_INPUT_NAME => {
                elem_value.set_int(&self.meter().analog_inputs);
                Ok(true)
            }
            SPDIF_INPUT_NAME => {
                elem_value.set_int(&self.meter().spdif_inputs);
                Ok(true)
            }
            ADAT_INPUT_NAME => {
                elem_value.set_int(&self.meter().adat_inputs);
                Ok(true)
            }
            STREAM_INPUT_NAME => {
                elem_value.set_int(&self.meter().stream_inputs);
                Ok(true)
            }
            ANALOG_OUTPUT_NAME => {
                elem_value.set_int(&self.meter().analog_outputs);
                Ok(true)
            }
            SPDIF_OUTPUT_NAME => {
                elem_value.set_int(&self.meter().spdif_outputs);
                Ok(true)
            }
            ADAT_OUTPUT_NAME => {
                elem_value.set_int(&self.meter().adat_outputs);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
