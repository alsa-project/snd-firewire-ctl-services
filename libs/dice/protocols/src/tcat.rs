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
//! All of the protocol is implemented to trait per section in which `AsRef<hinawa::FwReq>` is
//! used for supertrait. Any call of method in the trait initiates asynchronous transaction to
//! operate the registers.

use glib::{Error, error::ErrorDomain, Quark};

use hinawa::{FwNode, FwTcode, FwReq, FwReqExtManual};

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
    Invalid(i32),
}

impl std::fmt::Display for GeneralProtocolError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let msg = match self {
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
            GeneralProtocolError::Invalid(v) => v,
        }
    }

    fn from(code: i32) -> Option<Self> {
        let enumeration = match code {
            _ => GeneralProtocolError::Invalid(code),
        };
        Some(enumeration)
    }
}

/// The trait for general protocol.
pub trait GeneralProtocol<T: AsRef<FwNode>> : AsRef<FwReq> {
    const BASE_ADDR: u64 = 0xffffe0000000;
    const MAX_FRAME_SIZE: usize = 512;

    fn read(&self, node: &T, offset: usize, mut frames: &mut [u8], timeout_ms: u32)
        -> Result<(), Error>
    {
        let mut addr = Self::BASE_ADDR + offset as u64;

        while frames.len() > 0 {
            let len = std::cmp::min(frames.len(), Self::MAX_FRAME_SIZE);
            let tcode = if len == 4 {
                FwTcode::ReadQuadletRequest
            } else {
                FwTcode::ReadBlockRequest
            };

            self.as_ref().transaction_sync(node.as_ref(), tcode, addr, len, &mut frames[0..len], timeout_ms)?;

            addr += len as u64;
            frames = &mut frames[len..];
        }

        Ok(())
    }

    fn write(&self, node: &T, offset: usize, mut frames: &mut [u8], timeout_ms: u32)
        -> Result<(), Error>
    {
        let n = node.as_ref();
        let mut addr = Self::BASE_ADDR + (offset as u64);

        while frames.len() > 0 {
            let len = std::cmp::min(frames.len(), Self::MAX_FRAME_SIZE);
            let tcode = if len == 4 {
                FwTcode::WriteQuadletRequest
            } else {
                FwTcode::WriteBlockRequest
            };

            self.as_ref().transaction_sync(n, tcode, addr, len, &mut frames[0..len], timeout_ms)?;

            addr += len as u64;
            frames = &mut frames[len..];
        }

        Ok(())
    }

    fn read_general_sections(&self, node: &T, timeout_ms: u32) -> Result<GeneralSections, Error> {
        let mut data = [0;GeneralSections::SIZE];
        self.read(node, 0, &mut data, timeout_ms)
            .map(|_| GeneralSections::from(&data[..]))
    }
}

impl<O: AsRef<FwReq>, T: AsRef<FwNode>> GeneralProtocol<T> for O {}
