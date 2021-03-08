// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto
use glib::Error;

use alsactl::{ElemId, ElemIfaceType, ElemValue, ElemValueExt, ElemValueExtManual};
use hinawa::{FwNode, SndUnit, SndUnitExt};

use alsa_ctl_tlv_codec::items::DbInterval;

use core::card_cntr::*;

use ff_protocols::former::*;

const VOL_MIN: i32 = 0x00000000;
const VOL_ZERO: i32 = 0x00008000;
const VOL_MAX: i32 = 0x00010000;
const VOL_STEP: i32 = 1;
const VOL_TLV: DbInterval = DbInterval{min: -9000, max: 600, linear: false, mute_avail: false};

#[derive(Default, Debug)]
pub struct FormerOutCtl<V>
    where V: AsRef<[i32]> + AsMut<[i32]>,
{
    state: V,
}

impl<'a, V> FormerOutCtl<V>
    where V: AsRef<[i32]> + AsMut<[i32]>,
{
    const VOL_NAME: &'a str = "output-volume";

    pub fn load<U>(&mut self, unit: &SndUnit, proto: &U, card_cntr: &mut CardCntr, timeout_ms: u32)
        -> Result<(), Error>
        where U: RmeFormerOutputProtocol<FwNode, V>,
              V: AsRef<[i32]> + AsMut<[i32]>,
    {
        self.state.as_mut().iter_mut()
            .for_each(|vol| *vol = VOL_ZERO);
        proto.init_output_vols(&unit.get_node(), &self.state, timeout_ms)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::VOL_NAME, 0);
        let _ = card_cntr.add_int_elems(&elem_id, 1, VOL_MIN, VOL_MAX, VOL_STEP,
                                        self.state.as_ref().len(), Some(&Vec::<u32>::from(&VOL_TLV)), true)?;

        Ok(())
    }

    pub fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            Self::VOL_NAME => {
                elem_value.set_int(&self.state.as_ref());
                Ok(true)
            },
            _ => Ok(false),
        }
    }

    pub fn write<U>(&mut self, unit: &SndUnit, proto: &U, elem_id: &ElemId, new: &alsactl::ElemValue,
                    timeout_ms: u32)
        -> Result<bool, Error>
        where U: RmeFormerOutputProtocol<FwNode, V>,
              V: AsRef<[i32]> + AsMut<[i32]>,
    {
        match elem_id.get_name().as_str() {
            Self::VOL_NAME => {
                let mut vals = self.state.as_ref().to_vec();
                new.get_int(&mut vals);
                proto.write_output_vols(&unit.get_node(), &mut self.state, &vals, timeout_ms)
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

#[derive(Default, Debug)]
pub struct FormerMixerCtl<V>
    where V: RmeFormerMixerSpec + AsRef<[FormerMixerSrc]> + AsMut<[FormerMixerSrc]>,
{
    state: V,
}

impl<'a, V> FormerMixerCtl<V>
    where V: RmeFormerMixerSpec + AsRef<[FormerMixerSrc]> + AsMut<[FormerMixerSrc]>,
{
    const ANALOG_SRC_GAIN_NAME: &'a str = "mixer:analog-source-gain";
    const SPDIF_SRC_GAIN_NAME: &'a str = "mixer:spdif-source-gain";
    const ADAT_SRC_GAIN_NAME: &'a str = "mixer:adat-source-gain";
    const STREAM_SRC_GAIN_NAME: &'a str = "mixer:stream-source-gain";

    pub fn load<U>(&mut self, unit: &SndUnit, proto: &U, card_cntr: &mut CardCntr, timeout_ms: u32)
        -> Result<(), Error>
        where U: RmeFormerMixerProtocol<FwNode, V>,
              V: RmeFormerMixerSpec + AsRef<[FormerMixerSrc]> + AsMut<[FormerMixerSrc]>,
    {
        self.state.as_mut().iter_mut()
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

        (0..self.state.as_ref().len())
            .try_for_each(|i| {
                proto.init_mixer_src_gains(&unit.get_node(), &mut self.state, i, timeout_ms)
            })?;

        let mixers = self.state.as_ref();

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::ANALOG_SRC_GAIN_NAME, 0);
        let _ = card_cntr.add_int_elems(&elem_id, mixers.len(), GAIN_MIN, GAIN_MAX, GAIN_STEP,
                                        mixers[0].analog_gains.len(), Some(&Vec::<u32>::from(&GAIN_TLV)), true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::SPDIF_SRC_GAIN_NAME, 0);
        let _ = card_cntr.add_int_elems(&elem_id, mixers.len(), GAIN_MIN, GAIN_MAX, GAIN_STEP,
                                        mixers[0].spdif_gains.len(), Some(&Vec::<u32>::from(&GAIN_TLV)), true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::ADAT_SRC_GAIN_NAME, 0);
        let _ = card_cntr.add_int_elems(&elem_id, mixers.len(), GAIN_MIN, GAIN_MAX, GAIN_STEP,
                                        mixers[0].adat_gains.len(), Some(&Vec::<u32>::from(&GAIN_TLV)), true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::STREAM_SRC_GAIN_NAME, 0);
        let _ = card_cntr.add_int_elems(&elem_id, mixers.len(), GAIN_MIN, GAIN_MAX, GAIN_STEP,
                                        mixers[0].stream_gains.len(), Some(&Vec::<u32>::from(&GAIN_TLV)), true)?;

        Ok(())
    }

    pub fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            Self::ANALOG_SRC_GAIN_NAME => {
                let index = elem_id.get_index() as usize;
                elem_value.set_int(&self.state.as_ref()[index].analog_gains);
                Ok(true)
            }
            Self::SPDIF_SRC_GAIN_NAME => {
                let index = elem_id.get_index() as usize;
                elem_value.set_int(&self.state.as_ref()[index].spdif_gains);
                Ok(true)
            }
            Self::ADAT_SRC_GAIN_NAME => {
                let index = elem_id.get_index() as usize;
                elem_value.set_int(&self.state.as_ref()[index].adat_gains);
                Ok(true)
            }
            Self::STREAM_SRC_GAIN_NAME => {
                let index = elem_id.get_index() as usize;
                elem_value.set_int(&self.state.as_ref()[index].stream_gains);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub fn write<U>(&mut self, unit: &SndUnit, proto: &U, elem_id: &ElemId, new: &alsactl::ElemValue,
                    timeout_ms: u32)
        -> Result<bool, Error>
        where U: RmeFormerMixerProtocol<FwNode, V>,
              V: RmeFormerMixerSpec + AsRef<[FormerMixerSrc]> + AsMut<[FormerMixerSrc]>,
    {
        match elem_id.get_name().as_str() {
            Self::ANALOG_SRC_GAIN_NAME => {
                let index = elem_id.get_index() as usize;
                let mut gains = self.state.as_mut()[index].analog_gains.clone();
                new.get_int(&mut gains);
                proto.write_mixer_analog_gains(&unit.get_node(), &mut self.state, index, &gains, timeout_ms)
                    .map(|_| true)
            }
            Self::SPDIF_SRC_GAIN_NAME => {
                let index = elem_id.get_index() as usize;
                let mut gains = self.state.as_mut()[index].spdif_gains.clone();
                new.get_int(&mut gains);
                proto.write_mixer_spdif_gains(&unit.get_node(), &mut self.state, index, &gains, timeout_ms)
                    .map(|_| true)
            }
            Self::ADAT_SRC_GAIN_NAME => {
                let index = elem_id.get_index() as usize;
                let mut gains = self.state.as_mut()[index].adat_gains.clone();
                new.get_int(&mut gains);
                proto.write_mixer_adat_gains(&unit.get_node(), &mut self.state, index, &gains, timeout_ms)
                    .map(|_| true)
            }
            Self::STREAM_SRC_GAIN_NAME => {
                let index = elem_id.get_index() as usize;
                let mut gains = self.state.as_mut()[index].stream_gains.clone();
                new.get_int(&mut gains);
                proto.write_mixer_stream_gains(&unit.get_node(), &mut self.state, index, &gains, timeout_ms)
                    .map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

#[derive(Default, Debug)]
pub struct FormerMeterCtl<V>
    where V: FormerMeterSpec + AsRef<FormerMeterState> + AsMut<FormerMeterState>,
{
    state: V,
    measured_elem_list: Vec<ElemId>,
}

const LEVEL_MIN: i32 = 0x00000000;
const LEVEL_MAX: i32 = 0x7fffff00;
const LEVEL_STEP: i32 = 0x100;
const LEVEL_TLV: DbInterval = DbInterval{min: -9003, max: 600, linear: false, mute_avail: false};

impl<'a, V> FormerMeterCtl<V>
    where V: FormerMeterSpec + AsRef<FormerMeterState> + AsMut<FormerMeterState>,
{
    const ANALOG_INPUT_NAME: &'a str = "meter:analog-input";
    const SPDIF_INPUT_NAME: &'a str = "meter:spdif-input";
    const ADAT_INPUT_NAME: &'a str = "meter:adat-input";
    const STREAM_INPUT_NAME: &'a str = "meter:stream-input";

    const ANALOG_OUTPUT_NAME: &'a str = "meter:analog-output";
    const SPDIF_OUTPUT_NAME: &'a str = "meter:spdif-output";
    const ADAT_OUTPUT_NAME: &'a str = "meter:adat-output";

    pub fn load<U>(&mut self, unit: &SndUnit, proto: &U, card_cntr: &mut CardCntr, timeout_ms: u32)
        -> Result<(), Error>
        where U: RmeFfFormerMeterProtocol<FwNode, V>,
              V: FormerMeterSpec + AsRef<FormerMeterState> + AsMut<FormerMeterState>,
    {
        proto.read_meter(&unit.get_node(), &mut self.state, timeout_ms)?;

        let s = self.state.as_ref();
        [
            (Self::ANALOG_INPUT_NAME, s.analog_inputs.len()),
            (Self::SPDIF_INPUT_NAME, s.spdif_inputs.len()),
            (Self::ADAT_INPUT_NAME, s.adat_inputs.len()),
            (Self::STREAM_INPUT_NAME, s.stream_inputs.len()),
            (Self::ANALOG_OUTPUT_NAME, s.analog_outputs.len()),
            (Self::SPDIF_OUTPUT_NAME, s.spdif_outputs.len()),
            (Self::ADAT_OUTPUT_NAME, s.adat_outputs.len()),
        ].iter()
            .try_for_each(|&(name, count)| {
                let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
                card_cntr.add_int_elems(&elem_id, 1, LEVEL_MIN, LEVEL_MAX, LEVEL_STEP, count,
                                        Some(&Vec::<u32>::from(&LEVEL_TLV)), false)
                    .map(|mut elem_id_list| self.measured_elem_list.append(&mut elem_id_list))
            })
    }

    pub fn get_measured_elem_list(&self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.measured_elem_list);
    }

    pub fn measure_states<U>(&mut self, unit: &SndUnit, proto: &U, timeout_ms: u32)
        -> Result<(), Error>
        where U: RmeFfFormerMeterProtocol<FwNode, V>,
              V: FormerMeterSpec + AsRef<FormerMeterState> + AsMut<FormerMeterState>,
    {
        proto.read_meter(&unit.get_node(), &mut self.state, timeout_ms)
    }

    pub fn measure_elem(&self, elem_id: &ElemId, elem_value: &ElemValue)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::ANALOG_INPUT_NAME => {
                elem_value.set_int(&self.state.as_ref().analog_inputs);
                Ok(true)
            }
            Self::SPDIF_INPUT_NAME => {
                elem_value.set_int(&self.state.as_ref().spdif_inputs);
                Ok(true)
            }
            Self::ADAT_INPUT_NAME => {
                elem_value.set_int(&self.state.as_ref().adat_inputs);
                Ok(true)
            }
            Self::STREAM_INPUT_NAME => {
                elem_value.set_int(&self.state.as_ref().stream_inputs);
                Ok(true)
            }
            Self::ANALOG_OUTPUT_NAME => {
                elem_value.set_int(&self.state.as_ref().analog_outputs);
                Ok(true)
            }
            Self::SPDIF_OUTPUT_NAME => {
                elem_value.set_int(&self.state.as_ref().spdif_outputs);
                Ok(true)
            }
            Self::ADAT_OUTPUT_NAME => {
                elem_value.set_int(&self.state.as_ref().adat_outputs);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
