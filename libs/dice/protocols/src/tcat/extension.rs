// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Protocol extension defined by TCAT for ASICs of DICE.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! extension defined by TC Applied Technologies (TCAT) for ASICs of Digital Interface Communication
//! Engine (DICE).

pub mod caps_section;
pub mod cmd_section;
pub mod mixer_section;
#[doc(hidden)]
mod router_entry;
pub mod peak_section;

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
    Mixer,
    RouterEntry,
    Peak,
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
            ProtocolExtensionError::Mixer => 2,
            ProtocolExtensionError::RouterEntry => 3,
            ProtocolExtensionError::Peak => 4,
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

/// The enumeration to represent ID of destination block.
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

impl From<u8> for DstBlkId {
    fn from(val: u8) -> Self {
        match val {
            0 => Self::Aes,
            1 => Self::Adat,
            2 => Self::MixerTx0,
            3 => Self::MixerTx1,
            4 => Self::Ins0,
            5 => Self::Ins1,
            10 => Self::ArmApbAudio,
            11 => Self::Avs0,
            12 => Self::Avs1,
            _ => Self::Reserved(val),
        }
    }
}

impl From<DstBlkId> for u8 {
    fn from(id: DstBlkId) -> Self {
        match id {
            DstBlkId::Aes => 0,
            DstBlkId::Adat => 1,
            DstBlkId::MixerTx0 => 2,
            DstBlkId::MixerTx1 => 3,
            DstBlkId::Ins0 => 4,
            DstBlkId::Ins1 => 5,
            DstBlkId::ArmApbAudio => 10,
            DstBlkId::Avs0 => 11,
            DstBlkId::Avs1 => 12,
            DstBlkId::Reserved(val) => val,
        }
    }
}

/// The structure to represent destination block.
#[derive(Default, Debug, Clone, Copy, Eq, PartialEq)]
pub struct DstBlk {
    pub id: DstBlkId,
    pub ch: u8,
}

impl DstBlk {
    pub const ID_MASK: u8 = 0xf0;
    pub const ID_SHIFT: usize = 4;
    pub const CH_MASK: u8 = 0x0f;
    pub const CH_SHIFT: usize = 0;
}

impl From<u8> for DstBlk {
    fn from(val: u8) -> Self {
        DstBlk {
            id: DstBlkId::from((val & DstBlk::ID_MASK) >> DstBlk::ID_SHIFT),
            ch: (val & DstBlk::CH_MASK) >> DstBlk::CH_SHIFT,
        }
    }
}

impl From<DstBlk> for u8 {
    fn from(blk: DstBlk) -> Self {
        (u8::from(blk.id) << DstBlk::ID_SHIFT) | blk.ch
    }
}

/// The enumeration to represent ID of source block.
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

impl From<u8> for SrcBlkId {
    fn from(val: u8) -> Self {
        match val {
            0 => Self::Aes,
            1 => Self::Adat,
            2 => Self::Mixer,
            4 => Self::Ins0,
            5 => Self::Ins1,
            10 => Self::ArmAprAudio,
            11 => Self::Avs0,
            12 => Self::Avs1,
            15 => Self::Mute,
            _ => Self::Reserved(val),
        }
    }
}

impl From<SrcBlkId> for u8 {
    fn from(id: SrcBlkId) -> Self {
        match id {
            SrcBlkId::Aes => 0,
            SrcBlkId::Adat => 1,
            SrcBlkId::Mixer => 2,
            SrcBlkId::Ins0 => 4,
            SrcBlkId::Ins1 => 5,
            SrcBlkId::ArmAprAudio => 10,
            SrcBlkId::Avs0 => 11,
            SrcBlkId::Avs1 => 12,
            SrcBlkId::Mute => 15,
            SrcBlkId::Reserved(val) => val,
        }
    }
}

/// The structure to represent source block.
#[derive(Default, Debug, Clone, Copy, Eq, PartialEq)]
pub struct SrcBlk {
    pub id: SrcBlkId,
    pub ch: u8,
}

impl SrcBlk {
    pub const ID_MASK: u8 = 0xf0;
    pub const ID_SHIFT: usize = 4;
    pub const CH_MASK: u8 = 0x0f;
    pub const CH_SHIFT: usize = 0;
}

impl From<u8> for SrcBlk {
    fn from(val: u8) -> Self {
        SrcBlk {
            id: SrcBlkId::from((val & SrcBlk::ID_MASK) >> SrcBlk::ID_SHIFT),
            ch: (val & SrcBlk::CH_MASK) >> SrcBlk::CH_SHIFT,
        }
    }
}

impl From<SrcBlk> for u8 {
    fn from(blk: SrcBlk) -> Self {
        (u8::from(blk.id) << SrcBlk::ID_SHIFT) | blk.ch
    }
}

/// The alternative type of data for router entry.
pub type RouterEntryData = [u8;RouterEntry::SIZE];

/// The structure to represent entry of route.
#[derive(Default, Debug, Clone, Copy, Eq, PartialEq)]
pub struct RouterEntry {
    pub dst: DstBlk,
    pub src: SrcBlk,
    pub peak: u16,
}

impl RouterEntry {
    const SIZE: usize = 4;

    pub const SRC_OFFSET: usize = 2;
    pub const DST_OFFSET: usize = 3;
}

impl From<&RouterEntryData> for RouterEntry {
    fn from(raw: &RouterEntryData) -> Self {
        let mut doublet = [0;2];
        doublet.copy_from_slice(&raw[..2]);
        RouterEntry {
            dst: raw[Self::DST_OFFSET].into(),
            src: raw[Self::SRC_OFFSET].into(),
            peak: u16::from_be_bytes(doublet),
        }
    }
}

impl From<&RouterEntry> for RouterEntryData {
    fn from(entry: &RouterEntry) -> RouterEntryData {
        let mut raw = RouterEntryData::default();
        raw.copy_from_slice(&entry.peak.to_be_bytes());
        raw[RouterEntry::SRC_OFFSET] = entry.src.into();
        raw[RouterEntry::DST_OFFSET] = entry.dst.into();
        raw
    }
}

#[cfg(test)]
mod test {
    use super::Section;
    use super::ExtensionSections;
    use super::{DstBlk, SrcBlk, DstBlkId, SrcBlkId};

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

    #[test]
    fn dst_blk_from() {
        let blk = DstBlk {
            id: DstBlkId::ArmApbAudio,
            ch: 0x04,
        };
        assert_eq!(blk, DstBlk::from(u8::from(blk)));
    }

    #[test]
    fn src_blk_from() {
        let blk = SrcBlk {
            id: SrcBlkId::ArmAprAudio,
            ch: 0x04,
        };
        assert_eq!(blk, SrcBlk::from(u8::from(blk)));
    }
}
