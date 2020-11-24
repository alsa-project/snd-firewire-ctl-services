// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! For directory entries, the module includes structure, enumeration and trait implementation.
//!
//! Entry structure represents directory entry. KeyType enumerations represents key of entry.
//! EntryData enumeration represents type of directory entry, including its content.

/// The structure to represent directory entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry<'a> {
    pub key: KeyType,
    pub data: EntryData<'a>,
}

/// The enumeration to represent key of directory entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyType {
    Descriptor,
    BusDependentInfo,
    Vendor,
    HardwareVersion,
    Module,
    NodeCapabilities,
    Eui64,
    Unit,
    SpecifierId,
    Version,
    DependentInfo,
    UnitLocation,
    Model,
    Instance,
    Keyword,
    Feature,
    ModifiableDescriptor,
    DirectoryId,
    Reserved(u8),
}

impl From<u8> for KeyType {
    fn from(val: u8) -> Self {
        match val {
            0x01 => KeyType::Descriptor,
            0x02 => KeyType::BusDependentInfo,
            0x03 => KeyType::Vendor,
            0x04 => KeyType::HardwareVersion,
            0x07 => KeyType::Module,
            0x0c => KeyType::NodeCapabilities,
            0x0d => KeyType::Eui64,
            0x11 => KeyType::Unit,
            0x12 => KeyType::SpecifierId,
            0x13 => KeyType::Version,
            0x14 => KeyType::DependentInfo,
            0x15 => KeyType::UnitLocation,
            0x17 => KeyType::Model,
            0x18 => KeyType::Instance,
            0x19 => KeyType::Keyword,
            0x1a => KeyType::Feature,
            0x1f => KeyType::ModifiableDescriptor,
            0x20 => KeyType::DirectoryId,
            _ => KeyType::Reserved(val),
        }
    }
}

/// The enumeration to represent type of directory entry and its content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntryData<'a> {
    Immediate(u32),
    CsrOffset(usize),
    Leaf(&'a [u8]),
    Directory(Vec<Entry<'a>>),
}
