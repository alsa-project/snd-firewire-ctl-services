// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use {
    super::*,
    alsa_ctl_tlv_codec::DbInterval,
    protocols::{monitor::*, playback::*},
};

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

#[derive(Default)]
pub struct MixerCtl {
    has_fpga: bool,
    playback_params: EfwPlaybackParameters,
    playback_solo_params: EfwPlaybackSoloParameters,
    monitor_params: EfwMonitorParameters,
    flags: EfwHwCtlFlags,
}

const PLAYBACK_VOL_NAME: &str = "playback-volume";
const PLAYBACK_MUTE_NAME: &str = "playback-mute";
const PLAYBACK_SOLO_NAME: &str = "playback-solo";

const MONITOR_GAIN_NAME: &str = "monitor-gain";
const MONITOR_MUTE_NAME: &str = "monitor-mute";
const MONITOR_SOLO_NAME: &str = "monitor-solo";
const MONITOR_PAN_NAME: &str = "monitor-pan";

const ENABLE_MIXER: &str = "enable-mixer";

impl MixerCtl {
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

    const PAN_MIN: i32 = 0;
    const PAN_MAX: i32 = 255;
    const PAN_STEP: i32 = 1;

    fn cache(&mut self, hwinfo: &HwInfo, unit: &mut SndEfw, timeout_ms: u32) -> Result<(), Error> {
        self.playback_params = Default::default();
        (0..hwinfo.mixer_playbacks).try_for_each(|i| {
            unit.get_playback_vol(i, timeout_ms)
                .map(|vol| self.playback_params.volumes.push(vol))?;
            unit.get_playback_mute(i, timeout_ms)
                .map(|enabled| self.playback_params.mutes.push(enabled))?;
            Ok::<(), Error>(())
        })?;

        self.playback_solo_params = Default::default();
        let playback_solo_supported = hwinfo
            .caps
            .iter()
            .find(|c| HwCap::PlaybackSoloUnsupported.eq(c))
            .is_none();
        if playback_solo_supported {
            (0..hwinfo.mixer_playbacks).try_for_each(|i| {
                unit.get_playback_solo(i, timeout_ms)
                    .map(|enabled| self.playback_solo_params.solos.push(enabled))
            })?;
        }

        self.monitor_params = Default::default();
        (0..hwinfo.mixer_playbacks).try_for_each(|i| {
            let mut src = EfwMonitorSourceParameters::default();
            (0..hwinfo.mixer_captures)
                .try_for_each(|j| {
                    unit.get_monitor_vol(i, j, timeout_ms)
                        .map(|vol| src.gains.push(vol))?;
                    unit.get_monitor_mute(i, j, timeout_ms)
                        .map(|enabled| src.mutes.push(enabled))?;
                    unit.get_monitor_solo(i, j, timeout_ms)
                        .map(|enabled| src.solos.push(enabled))?;
                    unit.get_monitor_pan(i, j, timeout_ms)
                        .map(|pan| src.pans.push(pan))?;
                    Ok::<(), Error>(())
                })
                .map(|_| self.monitor_params.0.push(src))
        })?;

        unit.get_flags(timeout_ms).map(|flags| self.flags.0 = flags)
    }

    pub fn load(
        &mut self,
        hwinfo: &HwInfo,
        unit: &mut SndEfw,
        card_cntr: &mut CardCntr,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        self.cache(hwinfo, unit, timeout_ms)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, PLAYBACK_VOL_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            1,
            Self::COEF_MIN,
            Self::COEF_MAX,
            Self::COEF_STEP,
            hwinfo.mixer_playbacks,
            Some(&Into::<Vec<u32>>::into(Self::COEF_TLV)),
            true,
        )?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, PLAYBACK_MUTE_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, hwinfo.mixer_playbacks, true)?;

        if hwinfo
            .caps
            .iter()
            .find(|cap| HwCap::ControlRoom.eq(cap))
            .is_none()
        {
            let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, PLAYBACK_SOLO_NAME, 0);
            let _ = card_cntr.add_bool_elems(&elem_id, 1, hwinfo.mixer_playbacks, true)?;
        }

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MONITOR_GAIN_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            hwinfo.mixer_playbacks,
            Self::COEF_MIN,
            Self::COEF_MAX,
            Self::COEF_STEP,
            hwinfo.mixer_captures,
            Some(&Into::<Vec<u32>>::into(Self::COEF_TLV)),
            true,
        )?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MONITOR_MUTE_NAME, 0);
        let _ = card_cntr.add_bool_elems(
            &elem_id,
            hwinfo.mixer_playbacks,
            hwinfo.mixer_captures,
            true,
        )?;

        if hwinfo
            .caps
            .iter()
            .find(|c| HwCap::PlaybackSoloUnsupported.eq(c))
            .is_none()
        {
            let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MONITOR_SOLO_NAME, 0);
            let _ = card_cntr.add_bool_elems(
                &elem_id,
                hwinfo.mixer_playbacks,
                hwinfo.mixer_captures,
                true,
            )?;
        }

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MONITOR_PAN_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            hwinfo.mixer_playbacks,
            Self::PAN_MIN,
            Self::PAN_MAX,
            Self::PAN_STEP,
            hwinfo.mixer_captures,
            None,
            true,
        )?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, ENABLE_MIXER, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        // Onyx 1200f has both DSP and FPGA.
        let has_dsp = hwinfo.caps.iter().find(|cap| HwCap::Dsp.eq(cap)).is_some();
        let has_fpga = hwinfo.caps.iter().find(|cap| HwCap::Fpga.eq(cap)).is_some();
        self.has_fpga = !has_dsp && has_fpga;

        Ok(())
    }

    pub fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            PLAYBACK_VOL_NAME => {
                elem_value.set_int(&self.playback_params.volumes);
                Ok(true)
            }
            PLAYBACK_MUTE_NAME => {
                elem_value.set_bool(&self.playback_params.mutes);
                Ok(true)
            }
            PLAYBACK_SOLO_NAME => {
                elem_value.set_bool(&self.playback_solo_params.solos);
                Ok(true)
            }
            MONITOR_GAIN_NAME => {
                let dst = elem_id.index() as usize;
                let src = self.monitor_params.0.iter().nth(dst).ok_or_else(|| {
                    let msg = format!("Invalid argument for index of monitor destination: {}", dst);
                    Error::new(FileError::Inval, &msg)
                })?;
                elem_value.set_int(&src.gains);
                Ok(true)
            }
            MONITOR_MUTE_NAME => {
                let dst = elem_id.index() as usize;
                let src = self.monitor_params.0.iter().nth(dst).ok_or_else(|| {
                    let msg = format!("Invalid argument for index of monitor destination: {}", dst);
                    Error::new(FileError::Inval, &msg)
                })?;
                elem_value.set_bool(&src.mutes);
                Ok(true)
            }
            MONITOR_SOLO_NAME => {
                let dst = elem_id.index() as usize;
                let src = self.monitor_params.0.iter().nth(dst).ok_or_else(|| {
                    let msg = format!("Invalid argument for index of monitor destination: {}", dst);
                    Error::new(FileError::Inval, &msg)
                })?;
                elem_value.set_bool(&src.solos);
                Ok(true)
            }
            MONITOR_PAN_NAME => {
                let dst = elem_id.index() as usize;
                let src = self.monitor_params.0.iter().nth(dst).ok_or_else(|| {
                    let msg = format!("Invalid argument for index of monitor destination: {}", dst);
                    Error::new(FileError::Inval, &msg)
                })?;
                let params: Vec<i32> = src.pans.iter().map(|&pan| pan as i32).collect();
                elem_value.set_int(&params);
                Ok(true)
            }
            ENABLE_MIXER => {
                let val = self
                    .flags
                    .0
                    .iter()
                    .find(|&flag| *flag == HwCtlFlag::MixerEnabled)
                    .is_some();
                elem_value.set_bool(&[val]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub fn write(
        &mut self,
        unit: &mut SndEfw,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            PLAYBACK_VOL_NAME => {
                self.playback_params
                    .volumes
                    .iter_mut()
                    .zip(elem_value.int())
                    .enumerate()
                    .filter(|(_, (o, n))| !o.eq(n))
                    .try_for_each(|(i, (o, &val))| {
                        unit.set_playback_vol(i, val, timeout_ms).map(|_| *o = val)
                    })?;
                Ok(true)
            }
            PLAYBACK_MUTE_NAME => {
                self.playback_params
                    .mutes
                    .iter_mut()
                    .zip(elem_value.boolean())
                    .enumerate()
                    .filter(|(_, (o, n))| !o.eq(&n))
                    .try_for_each(|(i, (o, val))| {
                        unit.set_playback_mute(i, val, timeout_ms).map(|_| *o = val)
                    })?;
                Ok(true)
            }
            PLAYBACK_SOLO_NAME => {
                self.playback_solo_params
                    .solos
                    .iter_mut()
                    .zip(elem_value.boolean())
                    .enumerate()
                    .filter(|(_, (o, n))| !o.eq(&n))
                    .try_for_each(|(i, (o, val))| {
                        unit.set_playback_solo(i, val, timeout_ms).map(|_| *o = val)
                    })?;
                Ok(true)
            }
            MONITOR_GAIN_NAME => {
                let dst = elem_id.index() as usize;
                let params = self.monitor_params.0.iter_mut().nth(dst).ok_or_else(|| {
                    let msg = format!("Invalid argument for index of monitor destination: {}", dst);
                    Error::new(FileError::Inval, &msg)
                })?;
                params
                    .gains
                    .iter_mut()
                    .zip(elem_value.int())
                    .enumerate()
                    .filter(|(_, (o, n))| !o.eq(n))
                    .try_for_each(|(src, (o, &val))| {
                        unit.set_monitor_vol(dst, src, val, timeout_ms)
                            .map(|_| *o = val)
                    })
                    .map(|_| true)
            }
            MONITOR_MUTE_NAME => {
                let dst = elem_id.index() as usize;
                let params = self.monitor_params.0.iter_mut().nth(dst).ok_or_else(|| {
                    let msg = format!("Invalid argument for index of monitor destination: {}", dst);
                    Error::new(FileError::Inval, &msg)
                })?;
                params
                    .mutes
                    .iter_mut()
                    .zip(elem_value.boolean())
                    .enumerate()
                    .filter(|(_, (o, n))| !o.eq(&n))
                    .try_for_each(|(src, (o, val))| {
                        unit.set_monitor_mute(dst, src, val, timeout_ms)
                            .map(|_| *o = val)
                    })
                    .map(|_| true)
            }
            MONITOR_SOLO_NAME => {
                let dst = elem_id.index() as usize;
                let params = self.monitor_params.0.iter_mut().nth(dst).ok_or_else(|| {
                    let msg = format!("Invalid argument for index of monitor destination: {}", dst);
                    Error::new(FileError::Inval, &msg)
                })?;
                params
                    .solos
                    .iter_mut()
                    .zip(elem_value.boolean())
                    .enumerate()
                    .filter(|(_, (o, n))| !o.eq(&n))
                    .try_for_each(|(src, (o, val))| {
                        unit.set_monitor_solo(dst, src, val, timeout_ms)
                            .map(|_| *o = val)
                    })
                    .map(|_| true)
            }
            MONITOR_PAN_NAME => {
                let dst = elem_id.index() as usize;
                let params = self.monitor_params.0.iter_mut().nth(dst).ok_or_else(|| {
                    let msg = format!("Invalid argument for index of monitor destination: {}", dst);
                    Error::new(FileError::Inval, &msg)
                })?;
                params
                    .pans
                    .iter_mut()
                    .zip(elem_value.int())
                    .enumerate()
                    .filter(|(_, (o, n))| !(**o as i32).eq(&n))
                    .try_for_each(|(src, (o, n))| {
                        let val = *n as u8;
                        unit.set_monitor_pan(dst, src, val, timeout_ms)
                            .map(|_| *o = val)
                    })
                    .map(|_| true)
            }
            ENABLE_MIXER => {
                let val = elem_value.boolean()[0];
                let flags = [HwCtlFlag::MixerEnabled];
                if val {
                    unit.set_flags(Some(&flags), None, timeout_ms)
                        .map(|_| self.flags.0.push(HwCtlFlag::MixerEnabled))?;
                } else {
                    unit.set_flags(None, Some(&flags), timeout_ms)
                        .map(|_| self.flags.0.retain(|flag| HwCtlFlag::MixerEnabled.eq(flag)))?;
                }

                // The above operation immediately has an effect for DSP model, but not for FPGA
                // model. For workaround, configure each monitor with input 0 to activate the
                // configuration.
                if self.has_fpga {
                    (0..self.monitor_params.0.len()).try_for_each(|src| {
                        let vol = unit.get_monitor_vol(0, src, timeout_ms)?;
                        unit.set_monitor_vol(0, src, vol, timeout_ms)
                    })?;
                }

                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
