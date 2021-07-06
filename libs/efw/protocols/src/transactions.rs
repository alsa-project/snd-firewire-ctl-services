// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use super::ClkSrc;
use hinawa::SndEfwExtManual;

const TIMEOUT: u32 = 200;

enum Category {
    HwCtl,
    PhysOutput,
    PhysInput,
    Playback,
    Monitor,
    PortConf,
    Guitar,
}

impl From<Category> for u32 {
    fn from(cat: Category) -> Self {
        match cat {
            Category::HwCtl => 0x03,
            Category::PhysOutput => 0x04,
            Category::PhysInput => 0x05,
            Category::Playback => 0x06,
            Category::Monitor => 0x08,
            Category::PortConf => 0x09,
            Category::Guitar => 0x0a,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum HwCtlFlag {
    MixerEnabled,
    SpdifPro,
    SpdifNoneAudio,
    CtlRoomSelect,
    OutputLevelBypass,
    MeterInMode,
    MeterOutMode,
    SoftClip,
    GuitarHexInput,
    GuitarAutoCharging,
    PhantomPowering,
    Invalid(usize),
}

impl From<HwCtlFlag> for usize {
    fn from(flag: HwCtlFlag) -> Self {
        match flag {
            HwCtlFlag::MixerEnabled => 0,
            HwCtlFlag::SpdifPro => 1,
            HwCtlFlag::SpdifNoneAudio => 2,
            HwCtlFlag::CtlRoomSelect => 8, // B if it stands, else A.
            HwCtlFlag::OutputLevelBypass => 9, // B if it stands, else A.
            HwCtlFlag::MeterInMode => 12,  // D2 if stands, else D1.
            HwCtlFlag::MeterOutMode => 13,
            HwCtlFlag::SoftClip => 18,
            HwCtlFlag::GuitarHexInput => 29,
            HwCtlFlag::GuitarAutoCharging => 30,
            HwCtlFlag::PhantomPowering => 31,
            HwCtlFlag::Invalid(idx) => idx,
        }
    }
}

impl From<usize> for HwCtlFlag {
    fn from(pos: usize) -> Self {
        match pos {
            0 => HwCtlFlag::MixerEnabled,
            1 => HwCtlFlag::SpdifPro,
            2 => HwCtlFlag::SpdifNoneAudio,
            8 => HwCtlFlag::CtlRoomSelect,
            9 => HwCtlFlag::OutputLevelBypass,
            12 => HwCtlFlag::MeterInMode,
            13 => HwCtlFlag::MeterOutMode,
            18 => HwCtlFlag::SoftClip,
            29 => HwCtlFlag::GuitarHexInput,
            30 => HwCtlFlag::GuitarAutoCharging,
            31 => HwCtlFlag::PhantomPowering,
            _ => HwCtlFlag::Invalid(pos),
        }
    }
}

pub struct EfwHwCtl {}

impl EfwHwCtl {
    const CMD_SET_CLOCK: u32 = 0;
    const CMD_GET_CLOCK: u32 = 1;
    const CMD_SET_FLAGS: u32 = 3;
    const CMD_GET_FLAGS: u32 = 4;

    pub fn set_clock(
        unit: &hinawa::SndEfw,
        src: Option<ClkSrc>,
        rate: Option<u32>,
    ) -> Result<(), Error> {
        let mut args = [0, 0, 0];
        let mut params = [0, 0, 0];
        let (current_src, current_rate) = Self::get_clock(unit)?;
        args[0] = usize::from(match src {
            Some(s) => s,
            None => current_src,
        }) as u32;
        args[1] = match rate {
            Some(r) => r,
            None => current_rate,
        };
        let _ = unit.transaction_sync(
            u32::from(Category::HwCtl),
            Self::CMD_SET_CLOCK,
            Some(&args),
            Some(&mut params),
            TIMEOUT,
        );
        Ok(())
    }

    pub fn get_clock(unit: &hinawa::SndEfw) -> Result<(ClkSrc, u32), Error> {
        let mut params = [0, 0, 0];
        let _ = unit.transaction_sync(
            u32::from(Category::HwCtl),
            Self::CMD_GET_CLOCK,
            None,
            Some(&mut params),
            TIMEOUT,
        )?;
        Ok((ClkSrc::from(params[0] as usize), params[1]))
    }

    pub fn set_flags(
        unit: &hinawa::SndEfw,
        enable: &[HwCtlFlag],
        disable: &[HwCtlFlag],
    ) -> Result<(), Error> {
        let mut args = [0, 0];

        args[0] = enable
            .iter()
            .fold(0, |mask, flag| mask | (1 << usize::from(*flag)));
        args[1] = disable
            .iter()
            .fold(0, |mask, flag| mask | (1 << usize::from(*flag)));
        let _ = unit.transaction_sync(
            u32::from(Category::HwCtl),
            Self::CMD_SET_FLAGS,
            Some(&args),
            None,
            TIMEOUT,
        );
        Ok(())
    }

    pub fn get_flags(unit: &hinawa::SndEfw) -> Result<Vec<HwCtlFlag>, Error> {
        let mut params = [0];
        let _ = unit.transaction_sync(
            u32::from(Category::HwCtl),
            Self::CMD_GET_FLAGS,
            None,
            Some(&mut params),
            TIMEOUT,
        )?;
        let flags = (0..32)
            .filter(|i| params[0] & (1 << i) > 0)
            .map(|i| HwCtlFlag::from(i))
            .collect();
        Ok(flags)
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum NominalLevel {
    PlusFour,
    Medium,
    MinusTen,
}

impl From<NominalLevel> for u32 {
    fn from(level: NominalLevel) -> Self {
        match level {
            NominalLevel::MinusTen => 2,
            NominalLevel::Medium => 1,
            NominalLevel::PlusFour => 0,
        }
    }
}

impl From<u32> for NominalLevel {
    fn from(val: u32) -> Self {
        match val {
            2 => NominalLevel::MinusTen,
            1 => NominalLevel::Medium,
            _ => NominalLevel::PlusFour,
        }
    }
}

pub struct EfwPhysOutput {}

impl EfwPhysOutput {
    const CMD_SET_VOL: u32 = 0;
    const CMD_GET_VOL: u32 = 1;
    const CMD_SET_MUTE: u32 = 2;
    const CMD_GET_MUTE: u32 = 3;
    const CMD_SET_NOMINAL: u32 = 8;
    const CMD_GET_NOMINAL: u32 = 9;

    pub fn set_vol(unit: &hinawa::SndEfw, ch: usize, vol: i32) -> Result<(), Error> {
        let args = [ch as u32, vol as u32];
        let mut params = [0; 2];
        let _ = unit.transaction_sync(
            u32::from(Category::PhysOutput),
            Self::CMD_SET_VOL,
            Some(&args),
            Some(&mut params),
            TIMEOUT,
        )?;
        Ok(())
    }

    pub fn get_vol(unit: &hinawa::SndEfw, ch: usize) -> Result<i32, Error> {
        let args = [ch as u32, 0];
        let mut params = [0; 2];
        let _ = unit.transaction_sync(
            u32::from(Category::PhysOutput),
            Self::CMD_GET_VOL,
            Some(&args),
            Some(&mut params),
            TIMEOUT,
        )?;
        Ok(params[1] as i32)
    }

    pub fn set_mute(unit: &hinawa::SndEfw, ch: usize, mute: bool) -> Result<(), Error> {
        let args = [ch as u32, mute as u32];
        let mut params = [0; 2];
        let _ = unit.transaction_sync(
            u32::from(Category::PhysOutput),
            Self::CMD_SET_MUTE,
            Some(&args),
            Some(&mut params),
            TIMEOUT,
        )?;
        Ok(())
    }

    pub fn get_mute(unit: &hinawa::SndEfw, ch: usize) -> Result<bool, Error> {
        let args = [ch as u32, 0];
        let mut params = [0; 2];
        let _ = unit.transaction_sync(
            u32::from(Category::PhysOutput),
            Self::CMD_GET_MUTE,
            Some(&args),
            Some(&mut params),
            TIMEOUT,
        )?;
        Ok(params[1] > 0)
    }

    pub fn set_nominal(unit: &hinawa::SndEfw, ch: usize, level: NominalLevel) -> Result<(), Error> {
        let args = [ch as u32, u32::from(level)];
        let mut params = [0; 2];
        let _ = unit.transaction_sync(
            u32::from(Category::PhysOutput),
            Self::CMD_SET_NOMINAL,
            Some(&args),
            Some(&mut params),
            TIMEOUT,
        )?;
        Ok(())
    }

    pub fn get_nominal(unit: &hinawa::SndEfw, ch: usize) -> Result<NominalLevel, Error> {
        let args = [ch as u32, 0];
        let mut params = [0; 2];
        let _ = unit.transaction_sync(
            u32::from(Category::PhysOutput),
            Self::CMD_GET_NOMINAL,
            Some(&args),
            Some(&mut params),
            TIMEOUT,
        )?;
        Ok(NominalLevel::from(params[1]))
    }
}

pub struct EfwPhysInput {}

impl EfwPhysInput {
    const CMD_SET_NOMINAL: u32 = 8;
    const CMD_GET_NOMINAL: u32 = 9;

    pub fn set_nominal(unit: &hinawa::SndEfw, ch: usize, level: NominalLevel) -> Result<(), Error> {
        let args = [ch as u32, u32::from(level)];
        let mut params = [0; 2];
        let _ = unit.transaction_sync(
            u32::from(Category::PhysInput),
            Self::CMD_SET_NOMINAL,
            Some(&args),
            Some(&mut params),
            TIMEOUT,
        )?;
        Ok(())
    }

    pub fn get_nominal(unit: &hinawa::SndEfw, ch: usize) -> Result<NominalLevel, Error> {
        let args = [ch as u32, 0];
        let mut params = [0; 2];
        let _ = unit.transaction_sync(
            u32::from(Category::PhysInput),
            Self::CMD_GET_NOMINAL,
            Some(&args),
            Some(&mut params),
            TIMEOUT,
        )?;
        Ok(NominalLevel::from(params[1]))
    }
}

pub struct EfwPlayback {}

impl EfwPlayback {
    const CMD_SET_VOL: u32 = 0;
    const CMD_GET_VOL: u32 = 1;
    const CMD_SET_MUTE: u32 = 2;
    const CMD_GET_MUTE: u32 = 3;
    const CMD_SET_SOLO: u32 = 4;
    const CMD_GET_SOLO: u32 = 5;

    pub fn set_vol(unit: &hinawa::SndEfw, ch: usize, vol: i32) -> Result<(), Error> {
        let args = [ch as u32, vol as u32];
        let mut params = [0; 2];
        let _ = unit.transaction_sync(
            u32::from(Category::Playback),
            Self::CMD_SET_VOL,
            Some(&args),
            Some(&mut params),
            TIMEOUT,
        )?;
        Ok(())
    }

    pub fn get_vol(unit: &hinawa::SndEfw, ch: usize) -> Result<i32, Error> {
        let args = [ch as u32, 0];
        let mut params = [0; 2];
        let _ = unit.transaction_sync(
            u32::from(Category::Playback),
            Self::CMD_GET_VOL,
            Some(&args),
            Some(&mut params),
            TIMEOUT,
        )?;
        Ok(params[1] as i32)
    }

    pub fn set_mute(unit: &hinawa::SndEfw, ch: usize, mute: bool) -> Result<(), Error> {
        let args = [ch as u32, mute as u32];
        let mut params = [0; 2];
        let _ = unit.transaction_sync(
            u32::from(Category::Playback),
            Self::CMD_SET_MUTE,
            Some(&args),
            Some(&mut params),
            TIMEOUT,
        )?;
        Ok(())
    }

    pub fn get_mute(unit: &hinawa::SndEfw, ch: usize) -> Result<bool, Error> {
        let args = [ch as u32, 0];
        let mut params = [0; 2];
        let _ = unit.transaction_sync(
            u32::from(Category::Playback),
            Self::CMD_GET_MUTE,
            Some(&args),
            Some(&mut params),
            TIMEOUT,
        )?;
        Ok(params[1] > 0)
    }

    pub fn set_solo(unit: &hinawa::SndEfw, ch: usize, solo: bool) -> Result<(), Error> {
        let args = [ch as u32, solo as u32];
        let mut params = [0; 2];
        let _ = unit.transaction_sync(
            u32::from(Category::Playback),
            Self::CMD_SET_SOLO,
            Some(&args),
            Some(&mut params),
            TIMEOUT,
        )?;
        Ok(())
    }

    pub fn get_solo(unit: &hinawa::SndEfw, ch: usize) -> Result<bool, Error> {
        let args = [ch as u32, 0];
        let mut params = [0; 2];
        let _ = unit.transaction_sync(
            u32::from(Category::Playback),
            Self::CMD_GET_SOLO,
            Some(&args),
            Some(&mut params),
            TIMEOUT,
        )?;
        Ok(params[1] > 0)
    }
}

pub struct EfwMonitor {}

impl EfwMonitor {
    const CMD_SET_VOL: u32 = 0;
    const CMD_GET_VOL: u32 = 1;
    const CMD_SET_MUTE: u32 = 2;
    const CMD_GET_MUTE: u32 = 3;
    const CMD_SET_SOLO: u32 = 4;
    const CMD_GET_SOLO: u32 = 5;
    const CMD_SET_PAN: u32 = 6;
    const CMD_GET_PAN: u32 = 7;

    pub fn set_vol(unit: &hinawa::SndEfw, dst: usize, src: usize, vol: i32) -> Result<(), Error> {
        let args = [src as u32, dst as u32, vol as u32];
        let mut params = [0; 3];
        let _ = unit.transaction_sync(
            u32::from(Category::Monitor),
            Self::CMD_SET_VOL,
            Some(&args),
            Some(&mut params),
            TIMEOUT,
        )?;
        Ok(())
    }

    pub fn get_vol(unit: &hinawa::SndEfw, dst: usize, src: usize) -> Result<i32, Error> {
        let args = [src as u32, dst as u32, 0];
        let mut params = [0; 3];
        let _ = unit.transaction_sync(
            u32::from(Category::Monitor),
            Self::CMD_GET_VOL,
            Some(&args),
            Some(&mut params),
            TIMEOUT,
        )?;
        Ok(params[2] as i32)
    }

    pub fn set_mute(
        unit: &hinawa::SndEfw,
        dst: usize,
        src: usize,
        mute: bool,
    ) -> Result<(), Error> {
        let args = [src as u32, dst as u32, mute as u32];
        let mut params = [0; 3];
        let _ = unit.transaction_sync(
            u32::from(Category::Monitor),
            Self::CMD_SET_MUTE,
            Some(&args),
            Some(&mut params),
            TIMEOUT,
        )?;
        Ok(())
    }

    pub fn get_mute(unit: &hinawa::SndEfw, dst: usize, src: usize) -> Result<bool, Error> {
        let args = [src as u32, dst as u32, 0];
        let mut params = [0; 3];
        let _ = unit.transaction_sync(
            u32::from(Category::Monitor),
            Self::CMD_GET_MUTE,
            Some(&args),
            Some(&mut params),
            TIMEOUT,
        )?;
        Ok(params[2] > 0)
    }

    pub fn set_solo(
        unit: &hinawa::SndEfw,
        dst: usize,
        src: usize,
        solo: bool,
    ) -> Result<(), Error> {
        let args = [src as u32, dst as u32, solo as u32];
        let mut params = [0; 3];
        let _ = unit.transaction_sync(
            u32::from(Category::Monitor),
            Self::CMD_SET_SOLO,
            Some(&args),
            Some(&mut params),
            TIMEOUT,
        )?;
        Ok(())
    }

    pub fn get_solo(unit: &hinawa::SndEfw, dst: usize, src: usize) -> Result<bool, Error> {
        let args = [src as u32, dst as u32, 0];
        let mut params = [0; 3];
        let _ = unit.transaction_sync(
            u32::from(Category::Monitor),
            Self::CMD_GET_SOLO,
            Some(&args),
            Some(&mut params),
            TIMEOUT,
        )?;
        Ok(params[2] > 0)
    }

    pub fn set_pan(unit: &hinawa::SndEfw, dst: usize, src: usize, pan: u8) -> Result<(), Error> {
        let args = [src as u32, dst as u32, pan as u32];
        let mut params = [0; 3];
        let _ = unit.transaction_sync(
            u32::from(Category::Monitor),
            Self::CMD_SET_PAN,
            Some(&args),
            Some(&mut params),
            TIMEOUT,
        )?;
        Ok(())
    }

    pub fn get_pan(unit: &hinawa::SndEfw, dst: usize, src: usize) -> Result<u8, Error> {
        let args = [src as u32, dst as u32, 0];
        let mut params = [0; 3];
        let _ = unit.transaction_sync(
            u32::from(Category::Monitor),
            Self::CMD_GET_PAN,
            Some(&args),
            Some(&mut params),
            TIMEOUT,
        )?;
        Ok(params[2] as u8)
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum DigitalMode {
    SpdifCoax,
    AesebuXlr,
    SpdifOpt,
    AdatOpt,
    Unknown(u32),
}

impl From<u32> for DigitalMode {
    fn from(val: u32) -> Self {
        match val {
            0 => DigitalMode::SpdifCoax,
            1 => DigitalMode::AesebuXlr,
            2 => DigitalMode::SpdifOpt,
            3 => DigitalMode::AdatOpt,
            _ => DigitalMode::Unknown(val),
        }
    }
}

impl From<DigitalMode> for u32 {
    fn from(mode: DigitalMode) -> Self {
        match mode {
            DigitalMode::SpdifCoax => 0,
            DigitalMode::AesebuXlr => 1,
            DigitalMode::SpdifOpt => 2,
            DigitalMode::AdatOpt => 3,
            DigitalMode::Unknown(val) => val,
        }
    }
}

pub struct EfwPortConf {}

impl EfwPortConf {
    const CMD_SET_MIRROR: u32 = 0;
    const CMD_GET_MIRROR: u32 = 1;
    const CMD_SET_DIG_MODE: u32 = 2;
    const CMD_GET_DIG_MODE: u32 = 3;
    const CMD_SET_PHANTOM: u32 = 4;
    const CMD_GET_PHANTOM: u32 = 5;
    const CMD_SET_STREAM_MAP: u32 = 6;
    const CMD_GET_STREAM_MAP: u32 = 7;

    const MAP_SIZE: usize = 70;

    pub fn set_output_mirror(unit: &hinawa::SndEfw, pair: usize) -> Result<(), Error> {
        let _ = unit.transaction_sync(
            u32::from(Category::PortConf),
            Self::CMD_SET_MIRROR,
            Some(&[pair as u32]),
            None,
            TIMEOUT,
        )?;
        Ok(())
    }

    pub fn get_output_mirror(unit: &hinawa::SndEfw) -> Result<usize, Error> {
        let mut params = [0];
        let _ = unit.transaction_sync(
            u32::from(Category::PortConf),
            Self::CMD_GET_MIRROR,
            None,
            Some(&mut params),
            TIMEOUT,
        )?;
        Ok(params[0] as usize)
    }

    pub fn set_digital_mode(unit: &hinawa::SndEfw, mode: DigitalMode) -> Result<(), Error> {
        let args = [u32::from(mode)];
        let _ = unit.transaction_sync(
            u32::from(Category::PortConf),
            Self::CMD_SET_DIG_MODE,
            Some(&args),
            None,
            TIMEOUT,
        )?;
        Ok(())
    }

    pub fn get_digital_mode(unit: &hinawa::SndEfw) -> Result<DigitalMode, Error> {
        let mut params = [0];
        let _ = unit.transaction_sync(
            u32::from(Category::PortConf),
            Self::CMD_GET_DIG_MODE,
            None,
            Some(&mut params),
            TIMEOUT,
        )?;
        Ok(DigitalMode::from(params[0]))
    }

    pub fn set_phantom_powering(unit: &hinawa::SndEfw, state: bool) -> Result<(), Error> {
        let _ = unit.transaction_sync(
            u32::from(Category::PortConf),
            Self::CMD_SET_PHANTOM,
            Some(&[state as u32]),
            None,
            TIMEOUT,
        )?;
        Ok(())
    }

    pub fn get_phantom_powering(unit: &hinawa::SndEfw) -> Result<bool, Error> {
        let mut params = [0];
        let _ = unit.transaction_sync(
            u32::from(Category::PortConf),
            Self::CMD_GET_PHANTOM,
            None,
            Some(&mut params),
            TIMEOUT,
        )?;
        Ok(params[0] > 0)
    }

    pub fn set_stream_map(
        unit: &hinawa::SndEfw,
        rx_map: Option<Vec<usize>>,
        tx_map: Option<Vec<usize>>,
    ) -> Result<(), Error> {
        let mut args = [0; Self::MAP_SIZE];
        let _ = unit.transaction_sync(
            u32::from(Category::PortConf),
            Self::CMD_GET_STREAM_MAP,
            None,
            Some(&mut args),
            TIMEOUT,
        )?;
        if let Some(entries) = rx_map {
            args[2] = entries.len() as u32;
            entries
                .iter()
                .enumerate()
                .for_each(|(pos, entry)| args[4 + pos] = 2 * *entry as u32);
        }
        if let Some(entries) = tx_map {
            args[36] = entries.len() as u32;
            entries
                .iter()
                .enumerate()
                .for_each(|(pos, entry)| args[38 + pos] = 2 * *entry as u32);
        }
        let _ = unit.transaction_sync(
            u32::from(Category::PortConf),
            Self::CMD_SET_STREAM_MAP,
            Some(&args),
            None,
            TIMEOUT,
        )?;
        Ok(())
    }

    pub fn get_stream_map(unit: &hinawa::SndEfw) -> Result<(Vec<usize>, Vec<usize>), Error> {
        let mut params = [0; Self::MAP_SIZE];
        let _ = unit.transaction_sync(
            u32::from(Category::PortConf),
            Self::CMD_GET_STREAM_MAP,
            None,
            Some(&mut params),
            TIMEOUT,
        )?;
        let rx_entry_count = params[2] as usize;
        let rx_entries: Vec<usize> = (0..rx_entry_count)
            .map(|pos| (params[4 + pos] / 2) as usize)
            .collect();
        let tx_entry_count = params[36] as usize;
        let tx_entries: Vec<usize> = (0..tx_entry_count)
            .map(|pos| (params[38 + pos] / 2) as usize)
            .collect();
        Ok((rx_entries, tx_entries))
    }
}

#[derive(Debug)]
pub struct GuitarChargeState {
    pub manual_charge: bool,
    pub auto_charge: bool,
    pub suspend_to_charge: u32,
}

pub struct EfwGuitar {}

impl EfwGuitar {
    const CMD_SET_CHARGE_STATE: u32 = 7;
    const CMD_GET_CHARGE_STATE: u32 = 8;

    pub fn get_charge_state(unit: &hinawa::SndEfw) -> Result<GuitarChargeState, Error> {
        let mut params = [0; 3];
        let _ = unit.transaction_sync(
            u32::from(Category::Guitar),
            Self::CMD_GET_CHARGE_STATE,
            None,
            Some(&mut params),
            TIMEOUT,
        )?;
        let state = GuitarChargeState {
            manual_charge: params[0] > 0,
            auto_charge: params[1] > 0,
            suspend_to_charge: params[2],
        };
        Ok(state)
    }

    pub fn set_charge_state(unit: &hinawa::SndEfw, state: &GuitarChargeState) -> Result<(), Error> {
        let args = [
            state.manual_charge as u32,
            state.auto_charge as u32,
            state.suspend_to_charge,
        ];
        let mut params = [0; 3];
        let _ = unit.transaction_sync(
            u32::from(Category::Guitar),
            Self::CMD_SET_CHARGE_STATE,
            Some(&args),
            Some(&mut params),
            TIMEOUT,
        )?;
        Ok(())
    }
}
