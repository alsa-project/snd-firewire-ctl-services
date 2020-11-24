// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! For directory entries, the module includes structure, enumeration and trait implementation.
//!
//! Entry structure represents directory entry. KeyType enumerations represents key of entry.
//! EntryData enumeration represents type of directory entry, including its content.

use super::desc::*;

use std::convert::TryFrom;

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

/// The trait to access to data of entry according to key and data type.
pub trait EntryDataAccess<'a, T> {
    fn get(&'a self, key_type: KeyType) -> Option<T>;
}

impl<'a> EntryDataAccess<'a, &'a u32> for Entry<'a> {
    fn get(&'a self, key_type: KeyType) -> Option<&'a u32> {
        if self.key == key_type {
            if let EntryData::Immediate(v) = &self.data {
                Some(v)
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl<'a> EntryDataAccess<'a, &'a usize> for Entry<'a> {
    fn get(&'a self, key_type: KeyType) -> Option<&'a usize> {
        if self.key == key_type {
            if let EntryData::CsrOffset(o) = &self.data {
                Some(o)
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl<'a> EntryDataAccess<'a, &'a [Entry<'a>]> for Entry<'a> {
    fn get(&'a self, key_type: KeyType) -> Option<&'a [Entry<'a>]> {
        if self.key == key_type {
            if let EntryData::Directory(d) = &self.data {
                Some(d)
            } else {
                None
            }
        } else {
            None
        }
    }
}

// Cloned type.
impl<'a> EntryDataAccess<'a, u32> for Entry<'a> {
    fn get(&'a self, key_type: KeyType) -> Option<u32> {
        EntryDataAccess::<&u32>::get(self, key_type)
            .map(|v| *v)
    }
}

impl<'a> EntryDataAccess<'a, usize> for Entry<'a> {
    fn get(&'a self, key_type: KeyType) -> Option<usize> {
        EntryDataAccess::<&usize>::get(self, key_type)
            .map(|v| *v)
    }
}

// Via descriptor data.
impl<'a> EntryDataAccess<'a, Descriptor<'a>> for Entry<'a> {
    fn get(&'a self, key_type: KeyType) -> Option<Descriptor<'a>> {
        if self.key == key_type {
            Descriptor::try_from(self)
                .ok()
        } else {
            None
        }
    }
}

impl<'a> EntryDataAccess<'a, TextualDescriptorData<'a>> for Entry<'a> {
    fn get(&'a self, key_type: KeyType) -> Option<TextualDescriptorData<'a>> {
        EntryDataAccess::<Descriptor<'a>>::get(self, key_type)
            .and_then(|desc| {
                if let DescriptorData::Textual(d) = desc.data {
                    Some(d)
                } else {
                    None
                }
            })
    }
}

impl<'a> EntryDataAccess<'a, &'a str> for Entry<'a> {
    fn get(&'a self, key_type: KeyType) -> Option<&'a str> {
        EntryDataAccess::<TextualDescriptorData<'a>>::get(self, key_type)
            .map(|data| data.text)
    }
}

impl<'a> EntryDataAccess<'a, String> for Entry<'a> {
    fn get(&'a self, key_type: KeyType) -> Option<String> {
        EntryDataAccess::<&str>::get(self, key_type)
            .map(|text| text.to_string())
    }
}
