// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Protocol defined by TCAT for ASICs of DICE.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol defined
//! by TC Applied Technologies (TCAT) for ASICs of Digital Interface Communication Engine (DICE).
//!
//! In the protocol, all of features are categorized to several parts. Each part is represented in
//! range of registers accessible by IEEE 1394 asynchronous transaction. In the crate, the range
//! is called as `section`, therefore the features are categorized to the section.

pub mod ext_sync_section;
pub mod global_section;
pub mod rx_stream_format_section;
pub mod tx_stream_format_section;

pub mod extension;
pub mod tcd22xx_spec;

pub mod config_rom;

use {
    super::*,
    glib::{error::ErrorDomain, Quark},
    hinawa::{prelude::FwReqExtManual, FwTcode},
    std::{convert::TryFrom, fmt::Debug},
};

mod utils;

/// Section in control and status register (CSR) of node.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Section<T: Default + Debug> {
    pub params: T,
    pub offset: usize,
    pub size: usize,
    raw: Vec<u8>,
}

const SECTION_ENTRY_SIZE: usize = 8;

impl<T: Default + Debug> From<&[u8]> for Section<T> {
    fn from(data: &[u8]) -> Self {
        assert!(data.len() >= SECTION_ENTRY_SIZE);
        let mut quadlet = [0; 4];
        quadlet.copy_from_slice(&data[..4]);
        let offset = 4 * u32::from_be_bytes(quadlet) as usize;

        quadlet.copy_from_slice(&data[4..8]);
        let size = 4 * u32::from_be_bytes(quadlet) as usize;

        let params = Default::default();
        let raw = vec![0; size];

        Section {
            params,
            offset,
            size,
            raw,
        }
    }
}

/// The sset of sections in CSR of node.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct GeneralSections {
    pub global: Section<()>,
    pub tx_stream_format: Section<()>,
    pub rx_stream_format: Section<()>,
    pub ext_sync: Section<()>,
    pub reserved: Section<()>,
}

impl GeneralSections {
    const SECTION_COUNT: usize = 5;
    const SIZE: usize = SECTION_ENTRY_SIZE * Self::SECTION_COUNT;
}

impl From<&[u8]> for GeneralSections {
    fn from(raw: &[u8]) -> Self {
        GeneralSections {
            global: Section::from(&raw[..8]),
            tx_stream_format: Section::from(&raw[8..16]),
            rx_stream_format: Section::from(&raw[16..24]),
            ext_sync: Section::from(&raw[24..32]),
            reserved: Section::from(&raw[32..40]),
        }
    }
}

/// Any error of general protocol.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum GeneralProtocolError {
    Global,
    TxStreamFormat,
    RxStreamFormat,
    Invalid(i32),
}

impl std::fmt::Display for GeneralProtocolError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let msg = match self {
            GeneralProtocolError::Global => "global",
            GeneralProtocolError::TxStreamFormat => "tx-stream-format",
            GeneralProtocolError::RxStreamFormat => "rx-stream-format",
            GeneralProtocolError::Invalid(_) => "invalid",
        };

        write!(f, "GeneralProtocolError::{}", msg)
    }
}

impl ErrorDomain for GeneralProtocolError {
    fn domain() -> Quark {
        Quark::from_str("tcat-general-protocol-error-quark")
    }

    fn code(self) -> i32 {
        match self {
            GeneralProtocolError::Global => 0,
            GeneralProtocolError::TxStreamFormat => 1,
            GeneralProtocolError::RxStreamFormat => 2,
            GeneralProtocolError::Invalid(v) => v,
        }
    }

    fn from(code: i32) -> Option<Self> {
        let enumeration = match code {
            0 => GeneralProtocolError::Global,
            1 => GeneralProtocolError::TxStreamFormat,
            2 => GeneralProtocolError::RxStreamFormat,
            _ => GeneralProtocolError::Invalid(code),
        };
        Some(enumeration)
    }
}

/// Protocol implementation of TCAT general protocol.
#[derive(Default)]
pub struct GeneralProtocol;

const BASE_ADDR: u64 = 0xffffe0000000;

impl GeneralProtocol {
    const MAX_FRAME_SIZE: usize = 512;

    pub fn read(
        req: &mut FwReq,
        node: &mut FwNode,
        offset: usize,
        mut frames: &mut [u8],
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut addr = BASE_ADDR + offset as u64;

        while frames.len() > 0 {
            let len = std::cmp::min(frames.len(), Self::MAX_FRAME_SIZE);
            let tcode = if len == 4 {
                FwTcode::ReadQuadletRequest
            } else {
                FwTcode::ReadBlockRequest
            };

            req.transaction_sync(node, tcode, addr, len, &mut frames[0..len], timeout_ms)?;

            addr += len as u64;
            frames = &mut frames[len..];
        }

        Ok(())
    }

    pub fn write(
        req: &mut FwReq,
        node: &mut FwNode,
        offset: usize,
        mut frames: &mut [u8],
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut addr = BASE_ADDR + (offset as u64);

        while frames.len() > 0 {
            let len = std::cmp::min(frames.len(), Self::MAX_FRAME_SIZE);
            let tcode = if len == 4 {
                FwTcode::WriteQuadletRequest
            } else {
                FwTcode::WriteBlockRequest
            };

            req.transaction_sync(node, tcode, addr, len, &mut frames[0..len], timeout_ms)?;

            addr += len as u64;
            frames = &mut frames[len..];
        }

        Ok(())
    }

    pub fn read_general_sections(
        req: &mut FwReq,
        node: &mut FwNode,
        timeout_ms: u32,
    ) -> Result<GeneralSections, Error> {
        let mut data = [0; GeneralSections::SIZE];
        GeneralProtocol::read(req, node, 0, &mut data, timeout_ms)
            .map(|_| GeneralSections::from(&data[..]))
    }
}

/// Parameter of stream format for IEC 60958.
#[derive(Default, Clone, Copy, Debug, Eq, PartialEq)]
pub struct Iec60958Param {
    pub cap: bool,
    pub enable: bool,
}

impl GeneralProtocol {
    const NOTIFY_RX_CFG_CHG: u32 = 0x00000001;
    const NOTIFY_TX_CFG_CHG: u32 = 0x00000002;
    const NOTIFY_LOCK_CHG: u32 = 0x00000010;
    const NOTIFY_CLOCK_ACCEPTED: u32 = 0x00000020;
    const NOTIFY_EXT_STATUS: u32 = 0x00000040;

    pub fn has_rx_config_changed(msg: u32) -> bool {
        msg & msg & Self::NOTIFY_RX_CFG_CHG > 0
    }

    pub fn has_tx_config_changed(msg: u32) -> bool {
        msg & Self::NOTIFY_TX_CFG_CHG > 0
    }

    pub fn has_lock_changed(msg: u32) -> bool {
        msg & Self::NOTIFY_LOCK_CHG > 0
    }

    pub fn has_clock_accepted(msg: u32) -> bool {
        msg & Self::NOTIFY_CLOCK_ACCEPTED > 0
    }

    pub fn has_ext_status_changed(msg: u32) -> bool {
        msg & Self::NOTIFY_EXT_STATUS > 0
    }
}
