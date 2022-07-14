// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use {
    super::*,
    alsa_ctl_tlv_codec::DbInterval,
    efw_protocols::{hw_ctl::*, hw_info::*, monitor::*, playback::*},
};

#[derive(Default)]
pub struct MixerCtl {
    playbacks: usize,
    captures: usize,
    has_fpga: bool,
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

    pub fn load(&mut self, hwinfo: &HwInfo, card_cntr: &mut CardCntr) -> Result<(), Error> {
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
                ElemValueAccessor::<i32>::set_vals(elem_value, self.captures, |src| {
                    let val = unit.get_monitor_vol(dst, src, timeout_ms)?;
                    Ok(val)
                })?;
                Ok(true)
            }
            MONITOR_MUTE_NAME => {
                let dst = elem_id.index() as usize;
                ElemValueAccessor::<bool>::set_vals(elem_value, self.captures, |src| {
                    let val = unit.get_monitor_mute(dst, src, timeout_ms)?;
                    Ok(val)
                })?;
                Ok(true)
            }
            MONITOR_SOLO_NAME => {
                let dst = elem_id.index() as usize;
                ElemValueAccessor::<bool>::set_vals(elem_value, self.captures, |src| {
                    let val = unit.get_monitor_solo(dst, src, timeout_ms)?;
                    Ok(val)
                })?;
                Ok(true)
            }
            MONITOR_PAN_NAME => {
                let dst = elem_id.index() as usize;
                ElemValueAccessor::<i32>::set_vals(elem_value, self.captures, |src| {
                    let val = unit.get_monitor_pan(dst, src, timeout_ms)? as i32;
                    Ok(val)
                })?;
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
        new: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            PLAYBACK_VOL_NAME => {
                ElemValueAccessor::<i32>::get_vals(new, old, self.playbacks, |idx, val| {
                    unit.set_playback_vol(idx, val, timeout_ms)
                })?;
                Ok(true)
            }
            PLAYBACK_MUTE_NAME => {
                ElemValueAccessor::<bool>::get_vals(new, old, self.playbacks, |idx, val| {
                    unit.set_playback_mute(idx, val, timeout_ms)
                })?;
                Ok(true)
            }
            PLAYBACK_SOLO_NAME => {
                ElemValueAccessor::<bool>::get_vals(new, old, self.playbacks, |idx, val| {
                    unit.set_playback_solo(idx, val, timeout_ms)
                })?;
                Ok(true)
            }
            MONITOR_GAIN_NAME => {
                let dst = elem_id.index() as usize;
                ElemValueAccessor::<i32>::get_vals(new, old, self.captures, |src, val| {
                    unit.set_monitor_vol(dst, src, val, timeout_ms)
                })?;
                Ok(true)
            }
            MONITOR_MUTE_NAME => {
                let dst = elem_id.index() as usize;
                ElemValueAccessor::<bool>::get_vals(new, old, self.captures, |src, val| {
                    unit.set_monitor_mute(dst, src, val, timeout_ms)
                })?;
                Ok(true)
            }
            MONITOR_SOLO_NAME => {
                let dst = elem_id.index() as usize;
                ElemValueAccessor::<bool>::get_vals(new, old, self.captures, |src, val| {
                    unit.set_monitor_solo(dst, src, val, timeout_ms)
                })?;
                Ok(true)
            }
            MONITOR_PAN_NAME => {
                let dst = elem_id.index() as usize;
                ElemValueAccessor::<i32>::get_vals(new, old, self.captures, |src, val| {
                    unit.set_monitor_pan(dst, src, val as u8, timeout_ms)
                })?;
                Ok(true)
            }
            ENABLE_MIXER => {
                ElemValueAccessor::<bool>::get_val(new, |val| {
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
