// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Protocol extension defined by TCAT for ASICs of DICE.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! extension defined by TC Applied Technologies (TCAT) for ASICs of Digital Interface Communication
//! Engine (DICE).

pub mod appl_section;
pub mod caps_section;
pub mod cmd_section;
pub mod current_config_section;
pub mod mixer_section;
pub mod peak_section;
#[doc(hidden)]
mod router_entry;
pub mod router_section;
pub mod standalone_section;
#[doc(hidden)]
mod stream_format_entry;
pub mod stream_format_section;

use {
    super::{global_section::ClockRate, utils::*, *},
    std::cmp::Ordering,
};

/// Section in control and status register (CSR) of node.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct ExtensionSection {
    /// The offset of section in specific address space.
    pub offset: usize,
    /// The size of section.
    pub size: usize,
}

impl ExtensionSection {
    const SIZE: usize = 8;
}

#[cfg(test)]
fn serialize_extension_section(section: &ExtensionSection, raw: &mut [u8]) -> Result<(), String> {
    assert!(raw.len() >= ExtensionSection::SIZE);

    let val = (section.offset / 4) as u32;
    val.build_quadlet(&mut raw[..4]);

    let val = (section.size / 4) as u32;
    val.build_quadlet(&mut raw[4..8]);

    Ok(())
}

fn deserialize_extension_section(section: &mut ExtensionSection, raw: &[u8]) -> Result<(), String> {
    assert!(raw.len() >= ExtensionSection::SIZE);

    let mut val = 0u32;
    val.parse_quadlet(&raw[..4]);
    section.offset = 4 * val as usize;

    val.parse_quadlet(&raw[4..8]);
    section.size = 4 * val as usize;

    Ok(())
}

/// Sections for protocol extension.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct ExtensionSections {
    /// Capability.
    pub caps: ExtensionSection,
    /// Command.
    pub cmd: ExtensionSection,
    /// Mixer.
    pub mixer: ExtensionSection,
    /// Peak.
    pub peak: ExtensionSection,
    /// Router.
    pub router: ExtensionSection,
    /// Stream format configuration.
    pub stream_format: ExtensionSection,
    /// Current configurations.
    pub current_config: ExtensionSection,
    /// Stand alone configuration.
    pub standalone: ExtensionSection,
    /// Application specific configurations.
    pub application: ExtensionSection,
}

impl ExtensionSections {
    const SECTION_COUNT: usize = 9;
    const SIZE: usize = ExtensionSection::SIZE * Self::SECTION_COUNT;
}

#[cfg(test)]
fn serialize_extension_sections(
    sections: &ExtensionSections,
    raw: &mut [u8],
) -> Result<(), String> {
    assert!(raw.len() >= ExtensionSections::SIZE);

    serialize_extension_section(&sections.caps, &mut raw[..8])?;
    serialize_extension_section(&sections.cmd, &mut raw[8..16])?;
    serialize_extension_section(&sections.mixer, &mut raw[16..24])?;
    serialize_extension_section(&sections.peak, &mut raw[24..32])?;
    serialize_extension_section(&sections.router, &mut raw[32..40])?;
    serialize_extension_section(&sections.stream_format, &mut raw[40..48])?;
    serialize_extension_section(&sections.current_config, &mut raw[48..56])?;
    serialize_extension_section(&sections.standalone, &mut raw[56..64])?;
    serialize_extension_section(&sections.application, &mut raw[64..72])?;

    Ok(())
}

fn deserialize_extension_sections(
    sections: &mut ExtensionSections,
    raw: &[u8],
) -> Result<(), String> {
    assert!(raw.len() >= ExtensionSections::SIZE);

    deserialize_extension_section(&mut sections.caps, &raw[..8])?;
    deserialize_extension_section(&mut sections.cmd, &raw[8..16])?;
    deserialize_extension_section(&mut sections.mixer, &raw[16..24])?;
    deserialize_extension_section(&mut sections.peak, &raw[24..32])?;
    deserialize_extension_section(&mut sections.router, &raw[32..40])?;
    deserialize_extension_section(&mut sections.stream_format, &raw[40..48])?;
    deserialize_extension_section(&mut sections.current_config, &raw[48..56])?;
    deserialize_extension_section(&mut sections.standalone, &raw[56..64])?;
    deserialize_extension_section(&mut sections.application, &raw[64..72])?;

    Ok(())
}

/// Any error of protocol extension.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ProtocolExtensionError {
    /// Capability.
    Caps,
    /// Command.
    Cmd,
    /// Mixer.
    Mixer,
    /// Entry of router.
    RouterEntry,
    /// Peak.
    Peak,
    /// Router.
    Router,
    /// Entry of stream format.
    StreamFormatEntry,
    /// Stream format configuration.
    StreamFormat,
    /// Current configurations.
    CurrentConfig,
    /// Application specific configuration.
    Appl,
    /// Stand alone configuration.
    Standalone,
    Invalid(i32),
}

impl std::fmt::Display for ProtocolExtensionError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let msg = match self {
            ProtocolExtensionError::Caps => "caps",
            ProtocolExtensionError::Cmd => "command",
            ProtocolExtensionError::Mixer => "mixer",
            ProtocolExtensionError::RouterEntry => "router-entry",
            ProtocolExtensionError::Peak => "peak",
            ProtocolExtensionError::Router => "router",
            ProtocolExtensionError::StreamFormatEntry => "stream-format-entry",
            ProtocolExtensionError::StreamFormat => "stream-format",
            ProtocolExtensionError::CurrentConfig => "current-config",
            ProtocolExtensionError::Appl => "application",
            ProtocolExtensionError::Standalone => "standalone",
            ProtocolExtensionError::Invalid(_) => "invalid",
        };

        write!(f, "ProtocolExtensionError::{}", msg)
    }
}

impl ErrorDomain for ProtocolExtensionError {
    fn domain() -> Quark {
        Quark::from_str("tcat-protocol-extension-error-quark")
    }

    fn code(self) -> i32 {
        match self {
            ProtocolExtensionError::Caps => 0,
            ProtocolExtensionError::Cmd => 1,
            ProtocolExtensionError::Mixer => 2,
            ProtocolExtensionError::RouterEntry => 3,
            ProtocolExtensionError::Peak => 4,
            ProtocolExtensionError::Router => 5,
            ProtocolExtensionError::StreamFormatEntry => 6,
            ProtocolExtensionError::StreamFormat => 7,
            ProtocolExtensionError::CurrentConfig => 8,
            ProtocolExtensionError::Appl => 9,
            ProtocolExtensionError::Standalone => 10,
            ProtocolExtensionError::Invalid(v) => v,
        }
    }

    fn from(code: i32) -> Option<Self> {
        let enumeration = match code {
            0 => ProtocolExtensionError::Caps,
            1 => ProtocolExtensionError::Cmd,
            2 => ProtocolExtensionError::Mixer,
            3 => ProtocolExtensionError::RouterEntry,
            4 => ProtocolExtensionError::Peak,
            5 => ProtocolExtensionError::Router,
            7 => ProtocolExtensionError::StreamFormat,
            8 => ProtocolExtensionError::CurrentConfig,
            9 => ProtocolExtensionError::Appl,
            10 => ProtocolExtensionError::Standalone,
            _ => ProtocolExtensionError::Invalid(code),
        };
        Some(enumeration)
    }
}

const EXTENSION_OFFSET: usize = 0x00200000;

/// Operation of TCAT protocol extension.
pub trait TcatExtensionOperation: TcatOperation {
    /// Initiate read transaction to offset in specific address space and finish it.
    fn read_extension(
        req: &FwReq,
        node: &FwNode,
        section: &ExtensionSection,
        offset: usize,
        frames: &mut [u8],
        timeout_ms: u32,
    ) -> Result<(), Error> {
        Self::read(
            req,
            node,
            EXTENSION_OFFSET + section.offset + offset,
            frames,
            timeout_ms,
        )
    }

    /// Initiate write transaction to offset in specific address space and finish it.
    fn write_extension(
        req: &FwReq,
        node: &FwNode,
        section: &ExtensionSection,
        offset: usize,
        frames: &mut [u8],
        timeout_ms: u32,
    ) -> Result<(), Error> {
        Self::write(
            req,
            node,
            EXTENSION_OFFSET + section.offset + offset,
            frames,
            timeout_ms,
        )
    }

    /// Read section layout.
    fn read_extension_sections(
        req: &FwReq,
        node: &FwNode,
        sections: &mut ExtensionSections,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut raw = [0; ExtensionSections::SIZE];
        Self::read(req, node, EXTENSION_OFFSET, &mut raw, timeout_ms)
            .map(|_| deserialize_extension_sections(sections, &raw).unwrap())
    }
}

fn extension_read(
    req: &mut FwReq,
    node: &mut FwNode,
    offset: usize,
    frames: &mut [u8],
    timeout_ms: u32,
) -> Result<(), Error> {
    GeneralProtocol::read(req, node, EXTENSION_OFFSET + offset, frames, timeout_ms)
}

fn extension_write(
    req: &mut FwReq,
    node: &mut FwNode,
    offset: usize,
    frames: &mut [u8],
    timeout_ms: u32,
) -> Result<(), Error> {
    GeneralProtocol::write(req, node, EXTENSION_OFFSET + offset, frames, timeout_ms)
}

/// Identifier of destination block.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DstBlkId {
    Aes,
    Adat,
    MixerTx0,
    MixerTx1,
    Ins0,
    Ins1,
    ArmApbAudio,
    Avs0,
    Avs1,
    Reserved(u8),
}

impl Default for DstBlkId {
    fn default() -> Self {
        DstBlkId::Reserved(0xff)
    }
}

/// Destination block in router function.
#[derive(Default, Debug, Clone, Copy, Eq, PartialEq)]
pub struct DstBlk {
    pub id: DstBlkId,
    pub ch: u8,
}

/// Identifier of source block.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SrcBlkId {
    Aes,
    Adat,
    Mixer,
    Ins0,
    Ins1,
    ArmAprAudio,
    Avs0,
    Avs1,
    Mute,
    Reserved(u8),
}

impl Default for SrcBlkId {
    fn default() -> Self {
        SrcBlkId::Reserved(0xff)
    }
}

/// Source block in router function.
#[derive(Default, Debug, Clone, Copy, Eq, PartialEq)]
pub struct SrcBlk {
    pub id: SrcBlkId,
    pub ch: u8,
}

/// Entry of route in router function.
#[derive(Default, Debug, Clone, Copy, Eq, PartialEq)]
pub struct RouterEntry {
    pub dst: DstBlk,
    pub src: SrcBlk,
    pub peak: u16,
}

/// Entry of stream format.
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct FormatEntry {
    pub pcm_count: u8,
    pub midi_count: u8,
    pub labels: Vec<String>,
    pub enable_ac3: [bool; AC3_CHANNELS],
}

/// The number of channels in stream format for AC3 channels.
pub const AC3_CHANNELS: usize = 32;

#[cfg(test)]
mod test {
    use super::*;

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
        let space = ExtensionSections {
            caps: ExtensionSection {
                offset: 0x44,
                size: 0x40,
            },
            cmd: ExtensionSection {
                offset: 0x3c,
                size: 0x38,
            },
            mixer: ExtensionSection {
                offset: 0x34,
                size: 0x30,
            },
            peak: ExtensionSection {
                offset: 0x2c,
                size: 0x28,
            },
            router: ExtensionSection {
                offset: 0x24,
                size: 0x20,
            },
            stream_format: ExtensionSection {
                offset: 0x1c,
                size: 0x18,
            },
            current_config: ExtensionSection {
                offset: 0x14,
                size: 0x10,
            },
            standalone: ExtensionSection {
                offset: 0x0c,
                size: 0x08,
            },
            application: ExtensionSection {
                offset: 0x04,
                size: 0x00,
            },
        };

        let mut r = vec![0u8; raw.len()];
        serialize_extension_sections(&space, &mut r).unwrap();
        assert_eq!(&raw[..], &r);

        let mut s = ExtensionSections::default();
        deserialize_extension_sections(&mut s, &raw).unwrap();
        assert_eq!(space, s);
    }
}
