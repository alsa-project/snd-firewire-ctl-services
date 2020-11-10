// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use core::card_cntr;
use core::elem_value_accessor::ElemValueAccessor;

use super::transactions::{HwInfo, EfwPlayback, EfwMonitor, HwCap, EfwHwCtl, HwCtlFlag};

pub struct MixerCtl {
    playbacks: usize,
    captures: usize,
    has_fpga: bool,
}

impl<'a> MixerCtl {
    const PLAYBACK_VOL_NAME: &'a str = "playback-volume";
    const PLAYBACK_MUTE_NAME: &'a str = "playback-mute";
    const PLAYBACK_SOLO_NAME: &'a str = "playback-solo";

    const MONITOR_GAIN_NAME: &'a str = "monitor-gain";
    const MONITOR_MUTE_NAME: &'a str = "monitor-mute";
    const MONITOR_SOLO_NAME: &'a str = "monitor-solo";
    const MONITOR_PAN_NAME: &'a str = "monitor-pan";

    const ENABLE_MIXER: &'a str = "enable-mixer";

    // The fixed point number of 8.24 format.
    const COEF_MIN: i32 = 0x00000000;
    const COEF_MAX: i32 = 0x02000000;
    const COEF_STEP: i32 = 0x00000001;
    const COEF_TLV: &'a [u32] = &[5, 8, -14400i32 as u32, 6];

    const PAN_MIN: i32 = 0;
    const PAN_MAX: i32 = 255;
    const PAN_STEP: i32 = 1;

    pub fn new() -> Self {
        MixerCtl {
            playbacks: 0,
            captures: 0,
            has_fpga: false,
        }
    }

    pub fn load(
        &mut self,
        hwinfo: &HwInfo,
        card_cntr: &mut card_cntr::CardCntr,
    ) -> Result<(), Error> {
        self.playbacks = hwinfo.mixer_playbacks;
        self.captures = hwinfo.mixer_captures;

        let elem_id = alsactl::ElemId::new_by_name(
            alsactl::ElemIfaceType::Mixer, 0, 0, Self::PLAYBACK_VOL_NAME, 0);
        let _ = card_cntr.add_int_elems(&elem_id, 1,
            Self::COEF_MIN, Self::COEF_MAX, Self::COEF_STEP,
            self.playbacks, Some(Self::COEF_TLV), true)?;

        let elem_id = alsactl::ElemId::new_by_name(
            alsactl::ElemIfaceType::Mixer, 0, 0, Self::PLAYBACK_MUTE_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, self.playbacks, true)?;

        let elem_id = alsactl::ElemId::new_by_name(
            alsactl::ElemIfaceType::Mixer, 0, 0, Self::PLAYBACK_SOLO_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, self.playbacks, true)?;

        let elem_id = alsactl::ElemId::new_by_name(
            alsactl::ElemIfaceType::Mixer, 0, 0, Self::MONITOR_GAIN_NAME, 0);
        let _ = card_cntr.add_int_elems(&elem_id, self.playbacks,
            Self::COEF_MIN, Self::COEF_MAX, Self::COEF_STEP,
            self.captures, Some(Self::COEF_TLV), true)?;

        let elem_id = alsactl::ElemId::new_by_name(
            alsactl::ElemIfaceType::Mixer, 0, 0, Self::MONITOR_MUTE_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, self.playbacks, self.captures, true)?;

        let elem_id = alsactl::ElemId::new_by_name(
            alsactl::ElemIfaceType::Mixer, 0, 0, Self::MONITOR_SOLO_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, self.playbacks, self.captures, true)?;

        let elem_id = alsactl::ElemId::new_by_name(
            alsactl::ElemIfaceType::Mixer, 0, 0, Self::MONITOR_PAN_NAME, 0);
        let _ = card_cntr.add_int_elems(&elem_id, self.playbacks,
            Self::PAN_MIN, Self::PAN_MAX, Self::PAN_STEP,
            self.captures, None, true)?;

        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Mixer, 0, 0,
                                                   Self::ENABLE_MIXER, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;
        self.has_fpga = hwinfo.caps.iter().find(|&cap| *cap == HwCap::Fpga).is_some();

        Ok(())
    }

    pub fn read(
        &mut self,
        unit: &hinawa::SndEfw,
        elem_id: &alsactl::ElemId,
        elem_value: &mut alsactl::ElemValue,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            Self::PLAYBACK_VOL_NAME => {
                ElemValueAccessor::<i32>::set_vals(elem_value, self.playbacks, |idx| {
                    let val = EfwPlayback::get_vol(unit, idx)?;
                    Ok(val)
                })?;
                Ok(true)
            }
            Self::PLAYBACK_MUTE_NAME => {
                ElemValueAccessor::<bool>::set_vals(elem_value, self.playbacks, |idx| {
                    let val = EfwPlayback::get_mute(unit, idx)?;
                    Ok(val)
                })?;
                Ok(true)
            }
            Self::PLAYBACK_SOLO_NAME => {
                ElemValueAccessor::<bool>::set_vals(elem_value, self.playbacks, |idx| {
                    let val = EfwPlayback::get_solo(unit, idx)?;
                    Ok(val)
                })?;
                Ok(true)
            }
            Self::MONITOR_GAIN_NAME => {
                let dst = elem_id.get_index() as usize;
                ElemValueAccessor::<i32>::set_vals(elem_value, self.captures, |src| {
                    let val = EfwMonitor::get_vol(unit, dst, src)?;
                    Ok(val)
                })?;
                Ok(true)
            }
            Self::MONITOR_MUTE_NAME => {
                let dst = elem_id.get_index() as usize;
                ElemValueAccessor::<bool>::set_vals(elem_value, self.captures, |src| {
                    let val = EfwMonitor::get_mute(unit, dst, src)?;
                    Ok(val)
                })?;
                Ok(true)
            }
            Self::MONITOR_SOLO_NAME => {
                let dst = elem_id.get_index() as usize;
                ElemValueAccessor::<bool>::set_vals(elem_value, self.captures, |src| {
                    let val = EfwMonitor::get_solo(unit, dst, src)?;
                    Ok(val)
                })?;
                Ok(true)
            }
            Self::MONITOR_PAN_NAME => {
                let dst = elem_id.get_index() as usize;
                ElemValueAccessor::<i32>::set_vals(elem_value, self.captures, |src| {
                    let val = EfwMonitor::get_pan(unit, dst, src)? as i32;
                    Ok(val)
                })?;
                Ok(true)
            }
            Self::ENABLE_MIXER=> {
                ElemValueAccessor::<bool>::set_val(elem_value, || {
                    let flags = EfwHwCtl::get_flags(unit)?;
                    Ok(flags.iter().find(|&flag| *flag == HwCtlFlag::MixerEnabled).is_some())
                })?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub fn write(
        &mut self,
        unit: &hinawa::SndEfw,
        elem_id: &alsactl::ElemId,
        old: &alsactl::ElemValue,
        new: &alsactl::ElemValue,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            Self::PLAYBACK_VOL_NAME => {
                ElemValueAccessor::<i32>::get_vals(new, old, self.playbacks, |idx, val| {
                    EfwPlayback::set_vol(unit, idx, val)
                })?;
                Ok(true)
            }
            Self::PLAYBACK_MUTE_NAME => {
                ElemValueAccessor::<bool>::get_vals(new, old, self.playbacks, |idx, val| {
                    EfwPlayback::set_mute(unit, idx, val)
                })?;
                Ok(true)
            }
            Self::PLAYBACK_SOLO_NAME => {
                ElemValueAccessor::<bool>::get_vals(new, old, self.playbacks, |idx, val| {
                    EfwPlayback::set_solo(unit, idx, val)
                })?;
                Ok(true)
            }
            Self::MONITOR_GAIN_NAME => {
                let dst = elem_id.get_index() as usize;
                ElemValueAccessor::<i32>::get_vals(new, old, self.captures, |src, val| {
                    EfwMonitor::set_vol(unit, dst, src, val)
                })?;
                Ok(true)
            }
            Self::MONITOR_MUTE_NAME => {
                let dst = elem_id.get_index() as usize;
                ElemValueAccessor::<bool>::get_vals(new, old, self.captures, |src, val| {
                    EfwMonitor::set_mute(unit, dst, src, val)
                })?;
                Ok(true)
            }
            Self::MONITOR_SOLO_NAME => {
                let dst = elem_id.get_index() as usize;
                ElemValueAccessor::<bool>::get_vals(new, old, self.captures, |src, val| {
                    EfwMonitor::set_solo(unit, dst, src, val)
                })?;
                Ok(true)
            }
            Self::MONITOR_PAN_NAME => {
                let dst = elem_id.get_index() as usize;
                ElemValueAccessor::<i32>::get_vals(new, old, self.captures, |src, val| {
                    EfwMonitor::set_pan(unit, dst, src, val as u8)
                })?;
                Ok(true)
            }
            Self::ENABLE_MIXER=> {
                ElemValueAccessor::<bool>::get_val(new, |val| {
                    if val {
                        EfwHwCtl::set_flags(unit, &[HwCtlFlag::MixerEnabled], &[])?;
                    } else {
                        EfwHwCtl::set_flags(unit, &[], &[HwCtlFlag::MixerEnabled])?;
                    }

                    // The above operation immediately has an effect for DSP model, but not for FPGA
                    // model. For workaround, configure each monitor with input 0 to activate the
                    // configuration.
                    if self.has_fpga {
                        (0..self.playbacks).try_for_each(|i| {
                            let vol = EfwMonitor::get_vol(unit, 0, i)?;
                            EfwMonitor::set_vol(unit, 0, i, vol)
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
