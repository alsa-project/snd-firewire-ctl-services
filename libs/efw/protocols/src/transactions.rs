// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use hinawa::SndEfwExtManual;

const TIMEOUT: u32 = 200;

enum Category {
    PortConf,
    Guitar,
}

impl From<Category> for u32 {
    fn from(cat: Category) -> Self {
        match cat {
            Category::PortConf => 0x09,
            Category::Guitar => 0x0a,
        }
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
