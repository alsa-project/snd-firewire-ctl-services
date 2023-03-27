// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use {
    super::*,
    alsa_ctl_tlv_codec::DbInterval,
    protocols::{monitor::*, playback::*},
};

const PLAYBACK_VOL_NAME: &str = "playback-volume";
const PLAYBACK_MUTE_NAME: &str = "playback-mute";
const PLAYBACK_SOLO_NAME: &str = "playback-solo";

const MONITOR_GAIN_NAME: &str = "monitor-gain";
const MONITOR_MUTE_NAME: &str = "monitor-mute";
const MONITOR_SOLO_NAME: &str = "monitor-solo";
const MONITOR_PAN_NAME: &str = "monitor-pan";

// The fixed point number of 8.24 format.
const COEF_MIN: i32 = 0x00000000;
const COEF_MAX: i32 = 0x02000000;
const COEF_STEP: i32 = 0x00000001;
const COEF_TLV: DbInterval = DbInterval {
    min: -14400,
    max: 600,
    linear: false,
    mute_avail: false,
};

#[derive(Debug)]
pub(crate) struct MonitorCtl<T>
where
    T: EfwHardwareSpecification
        + EfwWhollyCachableParamsOperation<SndEfw, EfwMonitorParameters>
        + EfwPartiallyUpdatableParamsOperation<SndEfw, EfwMonitorParameters>,
{
    pub elem_id_list: Vec<ElemId>,
    params: EfwMonitorParameters,
    _phantom: PhantomData<T>,
}

impl<T> Default for MonitorCtl<T>
where
    T: EfwHardwareSpecification
        + EfwWhollyCachableParamsOperation<SndEfw, EfwMonitorParameters>
        + EfwPartiallyUpdatableParamsOperation<SndEfw, EfwMonitorParameters>,
{
    fn default() -> Self {
        Self {
            elem_id_list: Default::default(),
            params: T::create_monitor_parameters(),
            _phantom: Default::default(),
        }
    }
}

impl<T> MonitorCtl<T>
where
    T: EfwHardwareSpecification
        + EfwWhollyCachableParamsOperation<SndEfw, EfwMonitorParameters>
        + EfwPartiallyUpdatableParamsOperation<SndEfw, EfwMonitorParameters>,
{
    const PAN_MIN: i32 = 0;
    const PAN_MAX: i32 = 255;
    const PAN_STEP: i32 = 1;

    pub(crate) fn cache(&mut self, unit: &mut SndEfw, timeout_ms: u32) -> Result<(), Error> {
        T::cache_wholly(unit, &mut self.params, timeout_ms)
    }

    pub(crate) fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MONITOR_GAIN_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                T::MONITOR_DESTINATION_COUNT,
                COEF_MIN,
                COEF_MAX,
                COEF_STEP,
                T::MONITOR_SOURCE_COUNT,
                Some(&Into::<Vec<u32>>::into(COEF_TLV)),
                true,
            )
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MONITOR_MUTE_NAME, 0);
        card_cntr
            .add_bool_elems(
                &elem_id,
                T::MONITOR_DESTINATION_COUNT,
                T::MONITOR_SOURCE_COUNT,
                true,
            )
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MONITOR_SOLO_NAME, 0);
        card_cntr
            .add_bool_elems(
                &elem_id,
                T::MONITOR_DESTINATION_COUNT,
                T::MONITOR_SOURCE_COUNT,
                true,
            )
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MONITOR_PAN_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                T::MONITOR_DESTINATION_COUNT,
                Self::PAN_MIN,
                Self::PAN_MAX,
                Self::PAN_STEP,
                T::MONITOR_SOURCE_COUNT,
                None,
                true,
            )
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        Ok(())
    }

    pub(crate) fn read(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            MONITOR_GAIN_NAME => {
                let dst = elem_id.index() as usize;
                let src = self.params.0.iter().nth(dst).ok_or_else(|| {
                    let msg = format!("Invalid argument for index of monitor destination: {}", dst);
                    Error::new(FileError::Inval, &msg)
                })?;
                elem_value.set_int(&src.gains);
                Ok(true)
            }
            MONITOR_MUTE_NAME => {
                let dst = elem_id.index() as usize;
                let src = self.params.0.iter().nth(dst).ok_or_else(|| {
                    let msg = format!("Invalid argument for index of monitor destination: {}", dst);
                    Error::new(FileError::Inval, &msg)
                })?;
                elem_value.set_bool(&src.mutes);
                Ok(true)
            }
            MONITOR_SOLO_NAME => {
                let dst = elem_id.index() as usize;
                let src = self.params.0.iter().nth(dst).ok_or_else(|| {
                    let msg = format!("Invalid argument for index of monitor destination: {}", dst);
                    Error::new(FileError::Inval, &msg)
                })?;
                elem_value.set_bool(&src.solos);
                Ok(true)
            }
            MONITOR_PAN_NAME => {
                let dst = elem_id.index() as usize;
                let src = self.params.0.iter().nth(dst).ok_or_else(|| {
                    let msg = format!("Invalid argument for index of monitor destination: {}", dst);
                    Error::new(FileError::Inval, &msg)
                })?;
                let params: Vec<i32> = src.pans.iter().map(|&pan| pan as i32).collect();
                elem_value.set_int(&params);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub(crate) fn write(
        &mut self,
        unit: &mut SndEfw,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            MONITOR_GAIN_NAME => {
                let dst = elem_id.index() as usize;
                let params = self.params.clone();
                let source = self.params.0.iter_mut().nth(dst).ok_or_else(|| {
                    let msg = format!("Invalid argument for index of monitor destination: {}", dst);
                    Error::new(FileError::Inval, &msg)
                })?;
                let vals = &elem_value.int()[..T::MONITOR_SOURCE_COUNT];
                source.gains.copy_from_slice(vals);
                T::update_partially(unit, &mut self.params, params, timeout_ms)?;
                Ok(true)
            }
            MONITOR_MUTE_NAME => {
                let dst = elem_id.index() as usize;
                let params = self.params.clone();
                let source = self.params.0.iter_mut().nth(dst).ok_or_else(|| {
                    let msg = format!("Invalid argument for index of monitor destination: {}", dst);
                    Error::new(FileError::Inval, &msg)
                })?;
                let vals = &elem_value.boolean()[..T::MONITOR_SOURCE_COUNT];
                source.mutes.copy_from_slice(vals);
                T::update_partially(unit, &mut self.params, params, timeout_ms)?;
                Ok(true)
            }
            MONITOR_SOLO_NAME => {
                let dst = elem_id.index() as usize;
                let params = self.params.clone();
                let source = self.params.0.iter_mut().nth(dst).ok_or_else(|| {
                    let msg = format!("Invalid argument for index of monitor destination: {}", dst);
                    Error::new(FileError::Inval, &msg)
                })?;
                let vals = &elem_value.boolean()[..T::MONITOR_SOURCE_COUNT];
                source.solos.copy_from_slice(vals);
                T::update_partially(unit, &mut self.params, params, timeout_ms)?;
                Ok(true)
            }
            MONITOR_PAN_NAME => {
                let dst = elem_id.index() as usize;
                let params = self.params.clone();
                let source = self.params.0.iter_mut().nth(dst).ok_or_else(|| {
                    let msg = format!("Invalid argument for index of monitor destination: {}", dst);
                    Error::new(FileError::Inval, &msg)
                })?;
                source
                    .pans
                    .iter_mut()
                    .zip(elem_value.int())
                    .for_each(|(pan, &val)| *pan = val as u8);
                T::update_partially(unit, &mut self.params, params, timeout_ms)?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

#[derive(Debug)]
pub(crate) struct PlaybackCtl<T>
where
    T: EfwHardwareSpecification
        + EfwWhollyCachableParamsOperation<SndEfw, EfwPlaybackParameters>
        + EfwPartiallyUpdatableParamsOperation<SndEfw, EfwPlaybackParameters>,
{
    pub elem_id_list: Vec<ElemId>,
    params: EfwPlaybackParameters,
    _phantom: PhantomData<T>,
}

impl<T> Default for PlaybackCtl<T>
where
    T: EfwHardwareSpecification
        + EfwWhollyCachableParamsOperation<SndEfw, EfwPlaybackParameters>
        + EfwPartiallyUpdatableParamsOperation<SndEfw, EfwPlaybackParameters>,
{
    fn default() -> Self {
        Self {
            elem_id_list: Default::default(),
            params: T::create_playback_parameters(),
            _phantom: Default::default(),
        }
    }
}

impl<T> PlaybackCtl<T>
where
    T: EfwHardwareSpecification
        + EfwWhollyCachableParamsOperation<SndEfw, EfwPlaybackParameters>
        + EfwPartiallyUpdatableParamsOperation<SndEfw, EfwPlaybackParameters>,
{
    pub(crate) fn cache(&mut self, unit: &mut SndEfw, timeout_ms: u32) -> Result<(), Error> {
        T::cache_wholly(unit, &mut self.params, timeout_ms)
    }

    pub(crate) fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, PLAYBACK_VOL_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                COEF_MIN,
                COEF_MAX,
                COEF_STEP,
                T::RX_CHANNEL_COUNTS[0],
                Some(&Into::<Vec<u32>>::into(COEF_TLV)),
                true,
            )
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, PLAYBACK_MUTE_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, 1, T::RX_CHANNEL_COUNTS[0], true)
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        Ok(())
    }

    pub(crate) fn read(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            PLAYBACK_VOL_NAME => {
                elem_value.set_int(&self.params.volumes);
                Ok(true)
            }
            PLAYBACK_MUTE_NAME => {
                elem_value.set_bool(&self.params.mutes);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub(crate) fn write(
        &mut self,
        unit: &mut SndEfw,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            PLAYBACK_VOL_NAME => {
                let vals = &elem_value.int()[..T::RX_CHANNEL_COUNTS[0]];
                let mut params = self.params.clone();
                params.volumes.copy_from_slice(vals);
                T::update_partially(unit, &mut self.params, params, timeout_ms)?;
                Ok(true)
            }
            PLAYBACK_MUTE_NAME => {
                let vals = &elem_value.boolean()[..T::RX_CHANNEL_COUNTS[0]];
                let mut params = self.params.clone();
                params.mutes.copy_from_slice(vals);
                T::update_partially(unit, &mut self.params, params, timeout_ms)?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

#[derive(Debug)]
pub(crate) struct PlaybackSoloCtl<T>
where
    T: EfwPlaybackSoloSpecification
        + EfwWhollyCachableParamsOperation<SndEfw, EfwPlaybackSoloParameters>
        + EfwPartiallyUpdatableParamsOperation<SndEfw, EfwPlaybackSoloParameters>,
{
    pub elem_id_list: Vec<ElemId>,
    params: EfwPlaybackSoloParameters,
    _phantom: PhantomData<T>,
}

impl<T> Default for PlaybackSoloCtl<T>
where
    T: EfwPlaybackSoloSpecification
        + EfwWhollyCachableParamsOperation<SndEfw, EfwPlaybackSoloParameters>
        + EfwPartiallyUpdatableParamsOperation<SndEfw, EfwPlaybackSoloParameters>,
{
    fn default() -> Self {
        Self {
            elem_id_list: Default::default(),
            params: T::create_playback_solo_parameters(),
            _phantom: Default::default(),
        }
    }
}

impl<T> PlaybackSoloCtl<T>
where
    T: EfwPlaybackSoloSpecification
        + EfwWhollyCachableParamsOperation<SndEfw, EfwPlaybackSoloParameters>
        + EfwPartiallyUpdatableParamsOperation<SndEfw, EfwPlaybackSoloParameters>,
{
    pub(crate) fn cache(&mut self, unit: &mut SndEfw, timeout_ms: u32) -> Result<(), Error> {
        T::cache_wholly(unit, &mut self.params, timeout_ms)
    }

    pub(crate) fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, PLAYBACK_SOLO_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, 1, T::RX_CHANNEL_COUNTS[0], true)
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))
    }

    pub(crate) fn read(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            PLAYBACK_SOLO_NAME => {
                elem_value.set_bool(&self.params.solos);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub(crate) fn write(
        &mut self,
        unit: &mut SndEfw,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            PLAYBACK_SOLO_NAME => {
                let vals = &elem_value.boolean()[..T::RX_CHANNEL_COUNTS[0]];
                let mut params = self.params.clone();
                params.solos.copy_from_slice(vals);
                T::update_partially(unit, &mut self.params, params, timeout_ms)?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
