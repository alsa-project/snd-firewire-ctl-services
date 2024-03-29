// SPDX-License-Identifier: MIT
// Copyright (c) 2020 Takashi Sakamoto

//! Leaf entry has structured data. The module includes structure, enumeration and trait
//! implementation to parse it. The structure implements TryFrom trait to convert from the
//! content of leaf entry.
//!
//! Descriptor structure expresss descriptor itself. DescriptorData enumeration expresss data
//! of descriptor. At present, Textual descriptor is supported. TextualDescriptorData expresss
//! the data of Texual descriptor.
//!
//! EUI-64 structure expresss 64-bit EUI.
//!
//! Unit_Location structure expresss a pair of base address and upper bound for data of specific
//! unit.

use super::*;

fn detect_leaf_from_entry<'a>(entry: &'a Entry<'a>) -> Option<&'a [u8]> {
    if let EntryData::Leaf(leaf) = entry.data {
        Some(leaf)
    } else {
        None
    }
}

/// The structure expresss data of textual descriptor.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextualDescriptorData<'a> {
    pub width: u8,
    pub character_set: u16,
    pub language: u16,
    pub text: &'a str,
}

impl<'a> TryFrom<&'a [u8]> for TextualDescriptorData<'a> {
    type Error = DescriptorLeafParseError;

    fn try_from(raw: &'a [u8]) -> Result<Self, Self::Error> {
        if raw.len() < 4 {
            Err(Self::Error::TooShort(raw.len()))
        } else {
            let mut quadlet = [0; 4];
            quadlet.copy_from_slice(&raw[..4]);
            let meta = u32::from_be_bytes(quadlet);
            let width = ((meta & 0xf0000000) >> 28) as u8;
            let character_set = ((meta & 0x0fff0000) >> 16) as u16;
            let language = (meta & 0x0000ffff) as u16;
            let literal = &raw[4..];
            literal
                .iter()
                .position(|&c| c == 0x00)
                .ok_or(Self::Error::InvalidTextString)
                .and_then(|pos| {
                    std::str::from_utf8(&literal[..pos]).map_err(|_| Self::Error::InvalidTextString)
                })
                .or_else(|_| {
                    std::str::from_utf8(literal).map_err(|_| Self::Error::InvalidTextString)
                })
                .map(|text| TextualDescriptorData {
                    width,
                    character_set,
                    language,
                    text,
                })
        }
    }
}

/// The enumeration expresss data of descriptor according to its type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DescriptorData<'a> {
    Textual(TextualDescriptorData<'a>),
    // NOTE: it's possible to implement icon type but I have no need.
    Reserved(&'a [u8]),
}

impl<'a> DescriptorData<'a> {
    const TEXTUAL_DESCRIPTOR_TYPE: u8 = 0;
}

impl<'a> TryFrom<&'a [u8]> for DescriptorData<'a> {
    type Error = DescriptorLeafParseError;

    fn try_from(raw: &'a [u8]) -> Result<Self, Self::Error> {
        match raw[0] {
            Self::TEXTUAL_DESCRIPTOR_TYPE => {
                if raw.len() < 4 {
                    Err(Self::Error::TooShort(raw.len()))
                } else {
                    TextualDescriptorData::try_from(&raw[4..]).map(|d| Self::Textual(d))
                }
            }
            _ => Err(Self::Error::UnsupportedType(raw[0])),
        }
    }
}

/// The structure expresss descriptor in content of leaf entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DescriptorLeaf<'a> {
    pub spec_id: u32,
    pub data: DescriptorData<'a>,
}

impl<'a> TryFrom<&'a [u8]> for DescriptorLeaf<'a> {
    type Error = DescriptorLeafParseError;

    fn try_from(raw: &'a [u8]) -> Result<Self, Self::Error> {
        if raw.len() < 4 {
            Err(Self::Error::TooShort(raw.len()))
        } else {
            let mut quadlet = [0; 4];
            quadlet.copy_from_slice(&raw[..4]);
            let spec_id = u32::from_be_bytes(quadlet) & 0x00ffffff;

            DescriptorData::try_from(raw).map(|data| Self { spec_id, data })
        }
    }
}

impl<'a> TryFrom<&'a Entry<'a>> for DescriptorLeaf<'a> {
    type Error = DescriptorLeafParseError;

    fn try_from(entry: &'a Entry<'a>) -> Result<Self, Self::Error> {
        detect_leaf_from_entry(entry)
            .ok_or_else(|| Self::Error::WrongDirectoryEntry)
            .and_then(|leaf| Self::try_from(leaf))
    }
}

/// The error to parse leaf entry for descriptor data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DescriptorLeafParseError {
    /// Insufficient data to parse.
    TooShort(usize),
    /// Failed to parse the leaf as text string.
    InvalidTextString,
    /// Unsupported type of descriptor.
    UnsupportedType(
        /// The value of descriptor type.
        u8,
    ),
    /// The entry is not for leaf.
    WrongDirectoryEntry,
}

impl std::fmt::Display for DescriptorLeafParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TooShort(length) => write!(f, "The length of leaf {} is too short", length),
            Self::InvalidTextString => write!(f, "invalid text string in leaf"),
            Self::UnsupportedType(desc_type) => write!(f, "unsupported type {}", desc_type),
            Self::WrongDirectoryEntry => write!(f, "wrong directory entry"),
        }
    }
}

/// The structure to express data of EUI-64 leaf.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Eui64Leaf(pub u64);

impl TryFrom<&[u8]> for Eui64Leaf {
    type Error = Eui64LeafParseError;

    fn try_from(raw: &[u8]) -> Result<Self, Self::Error> {
        if raw.len() < 8 {
            Err(Self::Error::TooShort(raw.len()))
        } else {
            let mut quadlet = [0; 4];
            quadlet.copy_from_slice(&raw[..4]);
            let high = u32::from_be_bytes(quadlet) as u64;
            quadlet.copy_from_slice(&raw[4..8]);
            let low = u32::from_be_bytes(quadlet) as u64;
            Ok(Eui64Leaf((high << 32) | low))
        }
    }
}

impl<'a> TryFrom<&Entry<'a>> for Eui64Leaf {
    type Error = Eui64LeafParseError;

    fn try_from(entry: &Entry<'a>) -> Result<Self, Self::Error> {
        detect_leaf_from_entry(entry)
            .ok_or_else(|| Self::Error::WrongDirectoryEntry)
            .and_then(|leaf| Eui64Leaf::try_from(leaf))
    }
}

/// The error to parse leaf entry with EUI64 data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Eui64LeafParseError {
    /// Insufficient data to parse, 8 bytes at least.
    TooShort(usize),
    /// The entry is not for leaf.
    WrongDirectoryEntry,
}

impl std::fmt::Display for Eui64LeafParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TooShort(length) => write!(f, "The length of leaf {} too short", length),
            Self::WrongDirectoryEntry => write!(f, "wrong directory entry"),
        }
    }
}

/// The structure to express data of unit location leaf.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnitLocationLeaf {
    pub base_addr: u64,
    pub upper_bound: u64,
}

impl TryFrom<&[u8]> for UnitLocationLeaf {
    type Error = UnitLocationParseError;

    fn try_from(raw: &[u8]) -> Result<Self, Self::Error> {
        if raw.len() < 16 {
            Err(Self::Error::TooShort(raw.len()))
        } else {
            let mut quadlet = [0; 4];

            quadlet.copy_from_slice(&raw[..4]);
            let high = u32::from_be_bytes(quadlet) as u64;
            quadlet.copy_from_slice(&raw[4..8]);
            let low = u32::from_be_bytes(quadlet) as u64;
            let base_addr = (high << 32) | low;

            quadlet.copy_from_slice(&raw[8..12]);
            let high = u32::from_be_bytes(quadlet) as u64;
            quadlet.copy_from_slice(&raw[12..16]);
            let low = u32::from_be_bytes(quadlet) as u64;
            let upper_bound = (high << 32) | low;

            Ok(UnitLocationLeaf {
                base_addr,
                upper_bound,
            })
        }
    }
}

impl<'a> TryFrom<&Entry<'a>> for UnitLocationLeaf {
    type Error = UnitLocationParseError;

    fn try_from(entry: &Entry<'a>) -> Result<Self, Self::Error> {
        detect_leaf_from_entry(entry)
            .ok_or_else(|| Self::Error::WrongDirectoryEntry)
            .and_then(|leaf| UnitLocationLeaf::try_from(leaf))
    }
}

/// The error to parse leaf entry for unit location.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnitLocationParseError {
    /// Insufficient data to parse, 8 bytes at least.
    TooShort(usize),
    /// The entry is not for leaf.
    WrongDirectoryEntry,
}

impl std::fmt::Display for UnitLocationParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TooShort(length) => write!(f, "The length of leaf {} is too short", length),
            Self::WrongDirectoryEntry => write!(f, "wrong directory entry"),
        }
    }
}

#[cfg(test)]
mod test {
    use super::leaf::*;

    #[test]
    fn textual_desc_from_leaf_entry() {
        let raw = [
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x4c, 0x69, 0x6e, 0x75, 0x78, 0x20,
            0x46, 0x69, 0x72, 0x65, 0x77, 0x69, 0x72, 0x65, 0x00, 0x00,
        ];
        let entry = Entry {
            key: KeyType::Descriptor,
            data: EntryData::Leaf(&raw[..]),
        };
        let desc = DescriptorLeaf::try_from(&entry).unwrap();
        assert_eq!(0, desc.spec_id);
        if let DescriptorData::Textual(d) = desc.data {
            assert_eq!(0, d.width);
            assert_eq!(0, d.character_set);
            assert_eq!(0, d.language);
            assert_eq!(&"Linux Firewire", &d.text);
        }
    }

    #[test]
    fn eui64_from_leaf_entry() {
        let raw = [0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07];
        let entry = Entry {
            key: KeyType::Eui64,
            data: EntryData::Leaf(&raw[..]),
        };
        let eui64 = Eui64Leaf::try_from(&entry).unwrap();
        assert_eq!(0x0001020304050607, eui64.0);
    }

    #[test]
    fn unit_location_from_leaf_entry() {
        let raw = [
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d,
            0x0e, 0x0f,
        ];
        let entry = Entry {
            key: KeyType::UnitLocation,
            data: EntryData::Leaf(&raw[..]),
        };
        let unit_location = UnitLocationLeaf::try_from(&entry).unwrap();
        assert_eq!(0x0001020304050607, unit_location.base_addr);
        assert_eq!(0x08090a0b0c0d0e0f, unit_location.upper_bound);
    }
}
