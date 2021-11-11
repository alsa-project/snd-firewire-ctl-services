// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol about port configuration.
//!
//! The crate includes protocol about port configuration defined by Echo Audio Digital Corporation
//! for Fireworks board module.

use glib::Error;

use super::EfwProtocol;

const CATEGORY_PORT_CONF: u32 = 9;

const CMD_SET_MIRROR: u32 = 0;
const CMD_GET_MIRROR: u32 = 1;
const CMD_SET_DIG_MODE: u32 = 2;
const CMD_GET_DIG_MODE: u32 = 3;
const CMD_SET_PHANTOM: u32 = 4;
const CMD_GET_PHANTOM: u32 = 5;
const CMD_SET_STREAM_MAP: u32 = 6;
const CMD_GET_STREAM_MAP: u32 = 7;

/// The enumeration to express the type of audio signal for dignal input and output.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum DigitalMode {
    /// Coaxial interface for S/PDIF signal.
    SpdifCoax,
    /// XLR interface for AES/EBU signal.
    AesebuXlr,
    /// Optical interface for S/PDIF signal.
    SpdifOpt,
    /// Optical interface for ADAT signal.
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

const MAP_SIZE: usize = 70;

/// Protocol about port configuration for Fireworks board module.
pub trait PortConfProtocol: EfwProtocol {
    fn set_output_mirror(&mut self, pair: usize, timeout_ms: u32) -> Result<(), Error> {
        self.transaction_sync(
            CATEGORY_PORT_CONF,
            CMD_SET_MIRROR,
            Some(&[pair as u32]),
            None,
            timeout_ms,
        )
    }

    fn get_output_mirror(&mut self, timeout_ms: u32) -> Result<usize, Error> {
        let mut params = [0];
        self.transaction_sync(
            CATEGORY_PORT_CONF,
            CMD_GET_MIRROR,
            None,
            Some(&mut params),
            timeout_ms,
        )
        .map(|_| params[0] as usize)
    }

    fn set_digital_mode(&mut self, mode: DigitalMode, timeout_ms: u32) -> Result<(), Error> {
        let args = [u32::from(mode)];
        self.transaction_sync(
            CATEGORY_PORT_CONF,
            CMD_SET_DIG_MODE,
            Some(&args),
            None,
            timeout_ms,
        )
    }

    fn get_digital_mode(&mut self, timeout_ms: u32) -> Result<DigitalMode, Error> {
        let mut params = [0];
        self.transaction_sync(
            CATEGORY_PORT_CONF,
            CMD_GET_DIG_MODE,
            None,
            Some(&mut params),
            timeout_ms,
        )
        .map(|_| DigitalMode::from(params[0]))
    }

    fn set_phantom_powering(&mut self, state: bool, timeout_ms: u32) -> Result<(), Error> {
        self.transaction_sync(
            CATEGORY_PORT_CONF,
            CMD_SET_PHANTOM,
            Some(&[state as u32]),
            None,
            timeout_ms,
        )
    }

    fn get_phantom_powering(&mut self, timeout_ms: u32) -> Result<bool, Error> {
        let mut params = [0];
        self.transaction_sync(
            CATEGORY_PORT_CONF,
            CMD_GET_PHANTOM,
            None,
            Some(&mut params),
            timeout_ms,
        )
        .map(|_| params[0] > 0)
    }

    fn set_stream_map(
        &mut self,
        rate: u32,
        rx_map: Option<Vec<usize>>,
        tx_map: Option<Vec<usize>>,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut args = [rate];
        let mut params = [0; MAP_SIZE];
        self.transaction_sync(
            CATEGORY_PORT_CONF,
            CMD_GET_STREAM_MAP,
            Some(&mut args),
            Some(&mut params),
            timeout_ms,
        )?;
        let mut args = [0; MAP_SIZE];
        args[0] = rate;
        if let Some(entries) = rx_map {
            args[2] = entries.len() as u32;
            args[3] = params[3];
            entries
                .iter()
                .enumerate()
                .for_each(|(pos, entry)| args[4 + pos] = 2 * *entry as u32);
        }
        if let Some(entries) = tx_map {
            args[36] = entries.len() as u32;
            args[37] = params[37];
            entries
                .iter()
                .enumerate()
                .for_each(|(pos, entry)| args[38 + pos] = 2 * *entry as u32);
        }
        self.transaction_sync(
            CATEGORY_PORT_CONF,
            CMD_SET_STREAM_MAP,
            Some(&args),
            None,
            timeout_ms,
        )
    }

    fn get_stream_map(
        &mut self,
        rate: u32,
        timeout_ms: u32
    ) -> Result<(Vec<usize>, Vec<usize>), Error> {
        let args = [rate];
        let mut params = [0; MAP_SIZE];
        self.transaction_sync(
            CATEGORY_PORT_CONF,
            CMD_GET_STREAM_MAP,
            Some(&args),
            Some(&mut params),
            timeout_ms,
        )
        .map(|_| {
            let rx_entry_count = params[2] as usize;
            let rx_entries: Vec<usize> = (0..rx_entry_count)
                .map(|pos| (params[4 + pos] / 2) as usize)
                .collect();
            let tx_entry_count = params[36] as usize;
            let tx_entries: Vec<usize> = (0..tx_entry_count)
                .map(|pos| (params[38 + pos] / 2) as usize)
                .collect();
            (rx_entries, tx_entries)
        })
    }
}

impl<O: EfwProtocol> PortConfProtocol for O {}
