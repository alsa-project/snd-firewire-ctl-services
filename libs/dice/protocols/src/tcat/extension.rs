// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Protocol extension defined by TCAT for ASICs of DICE.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! extension defined by TC Applied Technologies (TCAT) for ASICs of Digital Interface Communication
//! Engine (DICE).

pub mod caps_section;
pub mod cmd_section;

use super::*;

/// The structure to represent sections for protocol extension.
#[derive(Default, Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExtensionSections{
    pub caps: Section,
    pub cmd: Section,
    pub mixer: Section,
    pub peak: Section,
    pub router: Section,
    pub stream_format: Section,
    pub current_config: Section,
    pub standalone: Section,
    pub application: Section,
}

impl ExtensionSections {
    const SECTION_COUNT: usize = 9;
    const SIZE: usize = Section::SIZE * Self::SECTION_COUNT;
}

impl From<&[u8]> for ExtensionSections {
    fn from(raw: &[u8]) -> Self {
        ExtensionSections{
            caps: Section::from(&raw[..8]),
            cmd: Section::from(&raw[8..16]),
            mixer: Section::from(&raw[16..24]),
            peak: Section::from(&raw[24..32]),
            router: Section::from(&raw[32..40]),
            stream_format: Section::from(&raw[40..48]),
            current_config: Section::from(&raw[48..56]),
            standalone: Section::from(&raw[56..64]),
            application: Section::from(&raw[64..72]),
        }
    }
}

/// The enumeration to represent any error of protocol extension.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ProtocolExtensionError {
    Caps,
    Cmd,
    Invalid(i32),
}

impl std::fmt::Display for ProtocolExtensionError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let msg = match self {
            ProtocolExtensionError::Caps => "caps",
            ProtocolExtensionError::Cmd => "command",
            ProtocolExtensionError::Invalid(_) => "invalid",
        };

        write!(f, "ProtocolExtensionError::{}", msg)
    }
}

impl ErrorDomain for ProtocolExtensionError {
    fn domain() -> Quark {
        Quark::from_string("tcat-protocol-extension-error-quark")
    }

    fn code(self) -> i32 {
        match self {
            ProtocolExtensionError::Caps => 0,
            ProtocolExtensionError::Cmd => 1,
            ProtocolExtensionError::Invalid(v) => v,
        }
    }

    fn from(code: i32) -> Option<Self> {
        let enumeration = match code {
            0 => ProtocolExtensionError::Caps,
            1 => ProtocolExtensionError::Cmd,
            _ => ProtocolExtensionError::Invalid(code),
        };
        Some(enumeration)
    }
}

pub trait ProtocolExtension<T: AsRef<FwNode>> : GeneralProtocol<T> {
    const EXTENSION_OFFSET: usize = 0x00200000;

    fn read(&self, node: &T, offset: usize, frames: &mut [u8], timeout_ms: u32)
        -> Result<(), Error>
    {
        GeneralProtocol::read(self, node, Self::EXTENSION_OFFSET + offset, frames, timeout_ms)
    }

    fn write(&self, node: &T, offset: usize, frames: &mut [u8], timeout_ms: u32)
        -> Result<(), Error>
    {
        GeneralProtocol::write(self, node, Self::EXTENSION_OFFSET + offset, frames, timeout_ms)
    }

    fn read_extension_sections(&self, node: &T, timeout_ms: u32) -> Result<ExtensionSections, Error> {
        let mut data = [0;ExtensionSections::SIZE];
        ProtocolExtension::read(self, node, 0, &mut data, timeout_ms)
            .map(|_| ExtensionSections::from(&data[..]))
    }
}

impl<O: AsRef<FwReq>, T: AsRef<FwNode>> ProtocolExtension<T> for O {}

#[cfg(test)]
mod test {
    use super::Section;
    use super::ExtensionSections;

    #[test]
    fn section_from() {
        let raw = [
            0x00, 0x00, 0x00, 0x11, 0x00, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00, 0x0f, 0x00, 0x00,
            0x00, 0x0e, 0x00, 0x00, 0x00, 0x0d, 0x00, 0x00, 0x00, 0x0c, 0x00, 0x00, 0x00, 0x0b,
            0x00, 0x00, 0x00, 0x0a, 0x00, 0x00, 0x00, 0x09, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00,
            0x00, 0x07, 0x00, 0x00, 0x00, 0x06, 0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x04,
            0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00,
            0x00, 0x00,
        ];
        let space = ExtensionSections{
            caps: Section{offset: 0x44, size: 0x40},
            cmd: Section{offset: 0x3c, size: 0x38},
            mixer: Section{offset: 0x34, size: 0x30},
            peak: Section{offset: 0x2c, size: 0x28},
            router: Section{offset: 0x24, size: 0x20},
            stream_format: Section{offset: 0x1c, size: 0x18},
            current_config: Section{offset: 0x14, size: 0x10},
            standalone: Section{offset: 0x0c, size: 0x08},
            application: Section{offset: 0x04, size: 0x00},
        };
        assert_eq!(space, ExtensionSections::from(&raw[..]));
    }
}
