// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Protocol defined by TCAT for ASICs of DICE.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol defined
//! by TC Applied Technologies (TCAT) for ASICs of Digital Interface Communication Engine (DICE).
//!
//! In the protocol, all of features are categorized to several parts. Each part is represented in
//! range of registers accessible by IEEE 1394 asynchronous transaction. In the crate, the range
//! is called as `section`, therefore the features are categorized to the section.
//!
//! ## Utilities
//!
//! Some programs are available under 'src/bin' directory.
//!
//! ### src/bin/tcat-general-parser.rs
//!
//! This program retrieves information from node of target device according to general protocol,
//! then print the information.
//!
//! Without any command line argument, it prints help message and exit.
//!
//! ```sh
//! $ cargo run --bin tcat-general-parser
//! Usage:
//!   tcat-general-parser CDEV
//!
//!   where:
//!     CDEV:   The path to special file of firewire character device, typically '/dev/fw1'.
//! ```
//!
//! Please run with an argument for firewire character device:
//!
//! ```sh
//! $ cargo run --bin tcat-general-parser /dev/fw1
//! ...
//! ```
//!
//! ### src/bin/tcat-extension-parser.rs
//!
//! This program retrieves information from node of target device according to protocol extension,
//! then print the information.
//!
//! Without any command line argument, it prints help message and exit.
//!
//! ```sh
//! $ cargo run --bin tcat-extension-parser
//! Usage:
//!   tcat-extension-parser CDEV
//!
//!   where:
//!     CDEV:       The path to special file of firewire character device, typically '/dev/fw1'.
//! ```
//!
//! Please run with an argument for firewire character device:
//!
//! ```sh
//! $ cargo run --bin tcat-extension-parser /dev/fw1
//! ...
//! ```
//!
//! ### src/bin/tcat-config-rom-parser.rs
//!
//! This program parse the content of configuration ROM, then print information in it.
//!
//! Without any command line argument, it prints help message and exit.
//!
//! ```sh
//! $ cargo run --bin tcat-config-rom-parser
//! Usage:
//!   tcat-config-rom-parser CDEV | "-"
//!
//!   where:
//!     CDEV:       the path to special file of firewire character device, typically '/dev/fw1'.
//!     "-"         use STDIN for the content of configuration ROM to parse. It should be aligned to big endian.
//! ```
//!
//! Please run with an argument for firewire character device:
//!
//! ```sh
//! $ cargo run --bin tcat-config-rom-parser -- /dev/fw1
//! ...
//! ```
//!
//! Or give content of configuration ROM via STDIN:
//!
//! ```sh
//! $ cat /sys/bus/firewire/devices/fw0/config_rom  | cargo run --bin tcat-config-rom-parser -- -
//! ...
//! ```
//!
//! In the above case, the content should be aligned to big-endian order.

pub mod global_section;
pub mod tx_stream_format_section;
pub mod rx_stream_format_section;
pub mod ext_sync_section;

pub mod extension;
pub mod tcd22xx_spec;

pub mod config_rom;

use glib::{Error, error::ErrorDomain, Quark};

use hinawa::{FwNode, FwTcode, FwReq, FwReqExtManual};

mod utils;

/// The structure to represent section in control and status register (CSR) of node.
#[derive(Default, Clone, Copy, Debug, Eq, PartialEq)]
pub struct Section {
    pub offset: usize,
    pub size: usize,
}

impl Section {
    pub const SIZE: usize = 8;
}

impl From <&[u8]> for Section {
    fn from(data: &[u8]) -> Self {
        assert!(data.len() >= Self::SIZE);
        let mut quadlet = [0;4];
        quadlet.copy_from_slice(&data[..4]);
        let offset = 4 * u32::from_be_bytes(quadlet) as usize;

        quadlet.copy_from_slice(&data[4..8]);
        let size = 4 * u32::from_be_bytes(quadlet) as usize;

        Section{offset, size}
    }
}

/// The structure to represent the set of sections in CSR of node.
#[derive(Default, Clone, Copy, Debug, Eq, PartialEq)]
pub struct GeneralSections {
    pub global: Section,
    pub tx_stream_format: Section,
    pub rx_stream_format: Section,
    pub ext_sync: Section,
    pub reserved: Section,
}

impl GeneralSections {
    const SECTION_COUNT: usize = 5;
    const SIZE: usize = Section::SIZE * Self::SECTION_COUNT;
}

impl From<&[u8]> for GeneralSections {
    fn from(raw: &[u8]) -> Self {
        GeneralSections{
            global: Section::from(&raw[..8]),
            tx_stream_format: Section::from(&raw[8..16]),
            rx_stream_format: Section::from(&raw[16..24]),
            ext_sync: Section::from(&raw[24..32]),
            reserved: Section::from(&raw[32..40]),
        }
    }
}

/// The enumeration to represent any error of general protocol.
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
        Quark::from_string("tcat-general-protocol-error-quark")
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

/// The structure for protocol implementation of TCAT general protocol.
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
        timeout_ms: u32
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
        timeout_ms: u32
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
        timeout_ms: u32
    ) -> Result<GeneralSections, Error> {
        let mut data = [0; GeneralSections::SIZE];
        GeneralProtocol::read(req, node, 0, &mut data, timeout_ms)
            .map(|_| GeneralSections::from(&data[..]))
    }
}

/// The structure to represent parameter of stream format for IEC 60958.
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
