// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use {
    super::*,
    alsa_ctl_tlv_codec::DbInterval,
    protocols::{hw_ctl::*, hw_info::*, monitor::*, playback::*},
};

#[derive(Default)]
pub struct MixerCtl {
    playbacks: usize,
    captures: usize,
    has_fpga: bool,
    monitor_params: EfwMonitorParameters,
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
                    Ok(())
                })
                .map(|_| self.monitor_params.0.push(src))
        })
    }

    pub fn load(
        &mut self,
        hwinfo: &HwInfo,
        unit: &mut SndEfw,
        card_cntr: &mut CardCntr,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        self.cache(hwinfo, unit, timeout_ms)?;

        self.playbacks = hwinfo.mixer_playbacks;
        self.captures = hwinfo.mixer_captures;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, PLAYBACK_VOL_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            1,
            Self::COEF_MIN,
            Self::COEF_MAX,
            Self::COEF_STEP,
            self.playbacks,
            Some(&Into::<Vec<u32>>::into(Self::COEF_TLV)),
            true,
        )?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, PLAYBACK_MUTE_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, self.playbacks, true)?;

        if hwinfo
            .caps
            .iter()
            .find(|cap| HwCap::ControlRoom.eq(cap))
            .is_none()
        {
            let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, PLAYBACK_SOLO_NAME, 0);
            let _ = card_cntr.add_bool_elems(&elem_id, 1, self.playbacks, true)?;
        }

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MONITOR_GAIN_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            self.playbacks,
            Self::COEF_MIN,
            Self::COEF_MAX,
            Self::COEF_STEP,
            self.captures,
            Some(&Into::<Vec<u32>>::into(Self::COEF_TLV)),
            true,
        )?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MONITOR_MUTE_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, self.playbacks, self.captures, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MONITOR_SOLO_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, self.playbacks, self.captures, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MONITOR_PAN_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            self.playbacks,
            Self::PAN_MIN,
            Self::PAN_MAX,
            Self::PAN_STEP,
            self.captures,
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

    pub fn read(
        &mut self,
        unit: &mut SndEfw,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            PLAYBACK_VOL_NAME => {
                ElemValueAccessor::<i32>::set_vals(elem_value, self.playbacks, |idx| {
                    unit.get_playback_vol(idx, timeout_ms)
                })?;
                Ok(true)
            }
            PLAYBACK_MUTE_NAME => {
                ElemValueAccessor::<bool>::set_vals(elem_value, self.playbacks, |idx| {
                    unit.get_playback_mute(idx, timeout_ms)
                })?;
                Ok(true)
            }
            PLAYBACK_SOLO_NAME => {
                ElemValueAccessor::<bool>::set_vals(elem_value, self.playbacks, |idx| {
                    unit.get_playback_solo(idx, timeout_ms)
                })?;
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
                ElemValueAccessor::<bool>::set_val(elem_value, || {
                    let flags = unit.get_flags(timeout_ms)?;
                    Ok(flags
                        .iter()
                        .find(|&flag| *flag == HwCtlFlag::MixerEnabled)
                        .is_some())
                })?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub fn write(
        &mut self,
        unit: &mut SndEfw,
        elem_id: &ElemId,
        old: &ElemValue,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            PLAYBACK_VOL_NAME => {
                ElemValueAccessor::<i32>::get_vals(elem_value, old, self.playbacks, |idx, val| {
                    unit.set_playback_vol(idx, val, timeout_ms)
                })?;
                Ok(true)
            }
            PLAYBACK_MUTE_NAME => {
                ElemValueAccessor::<bool>::get_vals(
                    elem_value,
                    old,
                    self.playbacks,
                    |idx, val| unit.set_playback_mute(idx, val, timeout_ms),
                )?;
                Ok(true)
            }
            PLAYBACK_SOLO_NAME => {
                ElemValueAccessor::<bool>::get_vals(
                    elem_value,
                    old,
                    self.playbacks,
                    |idx, val| unit.set_playback_solo(idx, val, timeout_ms),
                )?;
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
                ElemValueAccessor::<bool>::get_val(elem_value, |val| {
                    if val {
                        let flags = [HwCtlFlag::MixerEnabled];
                        unit.set_flags(Some(&flags), None, timeout_ms)?;
                    } else {
                        let flags = [HwCtlFlag::MixerEnabled];
                        unit.set_flags(None, Some(&flags), timeout_ms)?;
                    }

                    // The above operation immediately has an effect for DSP model, but not for FPGA
                    // model. For workaround, configure each monitor with input 0 to activate the
                    // configuration.
                    if self.has_fpga {
                        (0..self.playbacks).try_for_each(|i| {
                            let vol = unit.get_monitor_vol(0, i, timeout_ms)?;
                            unit.set_monitor_vol(0, i, vol, timeout_ms)
                        })?;
                    }

                    Ok(())
                })?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
