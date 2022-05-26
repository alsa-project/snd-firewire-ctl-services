// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol about hardware information.
//!
//! The module includes protocol about hardware information defined by Echo Audio Digital Corporation
//! for Fireworks board module.

use super::*;

const CATEGORY_TRANSPORT: u32 = 2;

const CMD_SET_TRANSMIT_MODE: u32 = 0;

/// The format of packet in transmission.
pub enum TxPacketFormat {
    /// Unique.
    Unique,
    /// Compliant to IEC 61883-1/6, actually with some quirks.
    Iec61883,
}

impl From<TxPacketFormat> for u32 {
    fn from(fmt: TxPacketFormat) -> Self {
        match fmt {
            TxPacketFormat::Unique => 0,
            TxPacketFormat::Iec61883 => 1,
        }
    }
}

/// Protocol about transmission for Fireworks board module.
pub trait TransportProtocol: EfwProtocolExtManual {
    fn get_hw_info(&mut self, fmt: TxPacketFormat, timeout_ms: u32) -> Result<(), Error> {
        let args = [u32::from(fmt)];
        self.transaction(
            CATEGORY_TRANSPORT,
            CMD_SET_TRANSMIT_MODE,
            &args,
            &mut Vec::new(),
            timeout_ms,
        )
    }
}

impl<O: EfwProtocolExtManual> TransportProtocol for O {}
