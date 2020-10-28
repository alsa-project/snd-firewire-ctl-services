// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use crate::card_cntr;

use alsactl::{ElemValueExt, ElemValueExtManual};

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
    const COEF_TLV: &'a [i32] = &[5, 8, -14400, 6];

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
                let mut vals = vec![0; self.playbacks];
                vals.iter_mut().enumerate().try_for_each(|(i, val)| {
                    *val = EfwPlayback::get_vol(unit, i)?;
                    Ok(())
                })?;
                elem_value.set_int(&vals);
                Ok(true)
            }
            Self::PLAYBACK_MUTE_NAME => {
                let mut vals = vec![false; self.playbacks];
                vals.iter_mut().enumerate().try_for_each(|(i, val)| {
                    *val = EfwPlayback::get_mute(unit, i)?;
                    Ok(())
                })?;
                elem_value.set_bool(&vals);
                Ok(true)
            }
            Self::PLAYBACK_SOLO_NAME => {
                let mut vals = vec![false; self.playbacks];
                vals.iter_mut().enumerate().try_for_each(|(i, val)| {
                    *val = EfwPlayback::get_solo(unit, i)?;
                    Ok(())
                })?;
                elem_value.set_bool(&vals);
                Ok(true)
            }
            Self::MONITOR_GAIN_NAME => {
                let dst = elem_id.get_index() as usize;
                let mut vals = vec![0; self.captures];
                vals.iter_mut().enumerate().try_for_each(|(src, val)| {
                    *val = EfwMonitor::get_vol(unit, dst, src)?;
                    Ok(())
                })?;
                elem_value.set_int(&vals);
                Ok(true)
            }
            Self::MONITOR_MUTE_NAME => {
                let dst = elem_id.get_index() as usize;
                let mut vals = vec![false; self.captures];
                vals.iter_mut().enumerate().try_for_each(|(src, val)| {
                    *val = EfwMonitor::get_mute(unit, dst, src)?;
                    Ok(())
                })?;
                elem_value.set_bool(&vals);
                Ok(true)
            }
            Self::MONITOR_SOLO_NAME => {
                let dst = elem_id.get_index() as usize;
                let mut vals = vec![false; self.captures];
                vals.iter_mut().enumerate().try_for_each(|(src, val)| {
                    *val = EfwMonitor::get_solo(unit, dst, src)?;
                    Ok(())
                })?;
                elem_value.set_bool(&vals);
                Ok(true)
            }
            Self::MONITOR_PAN_NAME => {
                let dst = elem_id.get_index() as usize;
                let mut vals = vec![0; self.captures];
                vals.iter_mut().enumerate().try_for_each(|(src, val)| {
                    *val = EfwMonitor::get_pan(unit, dst, src)? as i32;
                    Ok(())
                })?;
                elem_value.set_int(&vals);
                Ok(true)
            }
            Self::ENABLE_MIXER=> {
                let flags = EfwHwCtl::get_flags(unit)?;
                let state = flags.iter().find(|&flag| *flag == HwCtlFlag::MixerEnabled).is_some();
                elem_value.set_bool(&[state]);
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
                let mut vals = vec![0; self.playbacks * 2];
                new.get_int(&mut vals[..self.playbacks]);
                old.get_int(&mut vals[self.playbacks..]);
                (0..self.playbacks).try_for_each(|i| {
                    if vals[i] != vals[self.playbacks + i] {
                        EfwPlayback::set_vol(unit, i, vals[i])?;
                    }
                    Ok(())
                })?;
                Ok(true)
            }
            Self::PLAYBACK_MUTE_NAME => {
                let mut vals = vec![false; self.playbacks * 2];
                new.get_bool(&mut vals[..self.playbacks]);
                old.get_bool(&mut vals[self.playbacks..]);
                (0..self.playbacks).try_for_each(|i| {
                    if vals[i] != vals[self.playbacks + i] {
                        EfwPlayback::set_mute(unit, i, vals[i])?;
                    }
                    Ok(())
                })?;
                Ok(true)
            }
            Self::PLAYBACK_SOLO_NAME => {
                let mut vals = vec![false; self.playbacks * 2];
                new.get_bool(&mut vals[..self.playbacks]);
                old.get_bool(&mut vals[self.playbacks..]);
                (0..self.playbacks).try_for_each(|i| {
                    if vals[i] != vals[self.playbacks + i] {
                        EfwPlayback::set_solo(unit, i, vals[i])?;
                    }
                    Ok(())
                })?;
                Ok(true)
            }
            Self::MONITOR_GAIN_NAME => {
                let dst = elem_id.get_index() as usize;
                let mut vals = vec![0; self.captures * 2];
                new.get_int(&mut vals[..self.captures]);
                old.get_int(&mut vals[self.captures..]);
                (0..self.captures).try_for_each(|i| {
                    if vals[i] != vals[self.captures + i] {
                        EfwMonitor::set_vol(unit, dst, i, vals[i])?;
                    }
                    Ok(())
                })?;
                Ok(true)
            }
            Self::MONITOR_MUTE_NAME => {
                let dst = elem_id.get_index() as usize;
                let mut vals = vec![false; self.captures * 2];
                new.get_bool(&mut vals[..self.captures]);
                old.get_bool(&mut vals[self.captures..]);
                (0..self.captures).try_for_each(|src| {
                    if vals[src] != vals[self.captures + src] {
                        EfwMonitor::set_mute(unit, dst, src, vals[src])?;
                    }
                    Ok(())
                })?;
                Ok(true)
            }
            Self::MONITOR_SOLO_NAME => {
                let dst = elem_id.get_index() as usize;
                let mut vals = vec![false; self.captures * 2];
                new.get_bool(&mut vals[..self.captures]);
                old.get_bool(&mut vals[self.captures..]);
                (0..self.captures).try_for_each(|src| {
                    if vals[src] != vals[self.captures + src] {
                        EfwMonitor::set_solo(unit, dst, src, vals[src])?;
                    }
                    Ok(())
                })?;
                Ok(true)
            }
            Self::MONITOR_PAN_NAME => {
                let dst = elem_id.get_index() as usize;
                let mut vals = vec![0; self.captures * 2];
                new.get_int(&mut vals[..self.captures]);
                old.get_int(&mut vals[self.captures..]);
                (0..self.captures).try_for_each(|src| {
                    if vals[src] != vals[self.captures + src] {
                        EfwMonitor::set_pan(unit, dst, src, vals[src] as u8)?;
                    }
                    Ok(())
                })?;
                Ok(true)
            }
            Self::ENABLE_MIXER=> {
                let mut vals = [false];
                new.get_bool(&mut vals);
                if vals[0] {
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
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
