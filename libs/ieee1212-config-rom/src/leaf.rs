// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Leaf entry has structured data. The module includes structure, enumeration and trait
//! implementation to parse it. The structure implements TryFrom trait to convert from the
//! content of leaf entry.
//!
//! Descriptor structure represents descriptor itself. DescriptorData enumeration represents data
//! of descriptor. At present, Textual descriptor is supported. TextualDescriptorData represents
//! the data of Texual descriptor.
//!
//! EUI-64 structure represents 64-bit EUI.
//!
//! Unit_Location structure represents a pair of base address and upper bound for data of specific
//! unit.

use super::*;

use std::convert::TryFrom;

/// The structure to represent error cause.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeafParseError<T>
    where T: std::fmt::Debug + std::fmt::Display + Clone + Copy + PartialEq + Eq,
{
    ctx: T,
    msg: String
}

impl<T> LeafParseError<T>
    where T: std::fmt::Debug + std::fmt::Display + Clone + Copy + PartialEq + Eq,
{
    pub fn new(ctx: T, msg: String) -> Self {
        LeafParseError{ctx, msg}
    }
}

impl<T> std::fmt::Display for LeafParseError<T>
    where T: std::fmt::Debug + std::fmt::Display + Clone + Copy + PartialEq + Eq,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.ctx, self.msg)
    }
}

/// The structure represents data of textual descriptor.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextualDescriptorData<'a>{
    pub width: u8,
    pub character_set: u16,
    pub language: u16,
    pub text: &'a str,
}

impl<'a> TryFrom<&'a [u8]> for TextualDescriptorData<'a> {
    type Error = LeafParseError<DescriptorLeafParseCtx>;

    fn try_from(raw: &'a [u8]) -> Result<Self, Self::Error> {
        let mut quadlet = [0;4];
        quadlet.copy_from_slice(&raw[..4]);
        let meta = u32::from_be_bytes(quadlet);
        let width = ((meta & 0xf0000000) >> 28) as u8;
        let character_set = ((meta & 0x0fff0000) >> 16) as u16;
        let language = (meta & 0x0000ffff) as u16;
        let literal = &raw[4..];
        let text = literal.iter().position(|&c| c == 0x00)
            .ok_or(String::new())
            .and_then(|pos| {
                std::str::from_utf8(&literal[..pos])
                    .map_err(|e| e.to_string())
            })
            .or_else(|_| {
                std::str::from_utf8(literal)
                    .map_err(|e| e.to_string())
            })
            .map_err(|msg| {
                Self::Error::new(DescriptorLeafParseCtx::InvalidTextString, msg)
            })?;
        Ok(TextualDescriptorData{width, character_set, language, text})
    }
}

/// The enumeration represents data of descriptor according to its type.
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
    type Error = LeafParseError<DescriptorLeafParseCtx>;

    fn try_from(raw: &'a [u8]) -> Result<Self, Self::Error> {
        match raw[0] {
            Self::TEXTUAL_DESCRIPTOR_TYPE => {
                let d = TextualDescriptorData::try_from(&raw[4..])?;
                Ok(DescriptorData::Textual(d))
            }
            _ => {
                let msg = format!("{} type", raw[0]);
                Err(Self::Error::new(DescriptorLeafParseCtx::UnsupportedType, msg))
            }
        }
    }
}

/// The structure represents descriptor in content of leaf.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DescriptorLeaf<'a>{
    pub spec_id: u32,
    pub data: DescriptorData<'a>,
}

impl<'a> TryFrom<&'a [u8]> for DescriptorLeaf<'a> {
    type Error = LeafParseError<DescriptorLeafParseCtx>;

    fn try_from(raw: &'a [u8]) -> Result<Self, Self::Error> {
        let mut quadlet = [0;4];
        quadlet.copy_from_slice(&raw[..4]);
        let spec_id = u32::from_be_bytes(quadlet) & 0x00ffffff;

        let data = DescriptorData::try_from(raw)?;
        Ok(Self{spec_id, data})
    }
}

impl<'a> TryFrom<&'a Entry<'a>> for DescriptorLeaf<'a> {
    type Error = LeafParseError<DescriptorLeafParseCtx>;

    fn try_from(entry: &'a Entry<'a>) -> Result<Self, Self::Error> {
        if let EntryData::Leaf(leaf) = &entry.data {
            let desc = Self::try_from(&leaf[..])?;
            Ok(desc)
        } else {
            let label = if let EntryData::Immediate(_) = &entry.data {
                "immediate"
            } else if let EntryData::CsrOffset(_) = &entry.data {
                "csr-offset"
            } else if let EntryData::Directory(_) = &entry.data {
                "directory"
            } else {
                unreachable!()
            };
            let msg = format!("{} entry", label);
            Err(Self::Error::new(DescriptorLeafParseCtx::WrongDirectoryEntry, msg))
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DescriptorLeafParseCtx {
    InvalidTextString,
    UnsupportedType,
    WrongDirectoryEntry,
}

impl std::fmt::Display for DescriptorLeafParseCtx {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ctx = match self {
            DescriptorLeafParseCtx::InvalidTextString => "invalid text string in leaf",
            DescriptorLeafParseCtx::UnsupportedType => "unsupported type",
            DescriptorLeafParseCtx::WrongDirectoryEntry => "wrong directory entry",
        };
        write!(f, "{}", ctx)
    }
}

/// The structure to represent data of EUI-64 leaf.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Eui64Leaf(pub u64);

impl TryFrom<&[u8]> for Eui64Leaf {
    type Error = LeafParseError<Eui64LeafParseCtx>;

    fn try_from(raw: &[u8]) -> Result<Self, Self::Error> {
        if raw.len() < 8 {
            let msg = format!("8 bytes required but {}", raw.len());
            Err(Self::Error::new(Eui64LeafParseCtx::TooShort, msg))
        } else {
            let mut quadlet = [0;4];
            quadlet.copy_from_slice(&raw[..4]);
            let high = u32::from_be_bytes(quadlet) as u64;
            quadlet.copy_from_slice(&raw[4..8]);
            let low = u32::from_be_bytes(quadlet) as u64;
            Ok(Eui64Leaf((high << 32) | low))
        }
    }
}

impl<'a> TryFrom<&Entry<'a>> for Eui64Leaf {
    type Error = LeafParseError<Eui64LeafParseCtx>;

    fn try_from(entry: &Entry<'a>) -> Result<Self, Self::Error> {
        if let EntryData::Leaf(leaf) = &entry.data {
            let eui64 = Eui64Leaf::try_from(&leaf[..])?;
            Ok(eui64)
        } else {
            let label = if let EntryData::Immediate(_) = &entry.data {
                "immediate"
            } else if let EntryData::CsrOffset(_) = &entry.data {
                "csr-offset"
            } else if let EntryData::Directory(_) = &entry.data {
                "directory"
            } else {
                unreachable!()
            };
            let msg = format!("{} entry is not available", label);
            Err(Self::Error::new(Eui64LeafParseCtx::WrongDirectoryEntry, msg))
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Eui64LeafParseCtx {
    TooShort,
    WrongDirectoryEntry,
}

impl std::fmt::Display for Eui64LeafParseCtx {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ctx = match self {
            Self::TooShort => "Size of leaf too short",
            Self::WrongDirectoryEntry => "wrong directory entry",
        };
        write!(f, "{}", ctx)
    }
}

/// The structure to represent data of unit location leaf.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnitLocationLeaf{
    pub base_addr: u64,
    pub upper_bound: u64,
}

impl TryFrom<&[u8]> for UnitLocationLeaf {
    type Error = LeafParseError<UnitLocationParseCtx>;

    fn try_from(raw: &[u8]) -> Result<Self, Self::Error> {
        if raw.len() < 16 {
            let msg = format!("16 bytes required but {}", raw.len());
            Err(Self::Error::new(UnitLocationParseCtx::TooShort, msg))
        } else {
            let mut quadlet = [0;4];

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

            Ok(UnitLocationLeaf{base_addr, upper_bound})
        }
    }
}

impl<'a> TryFrom<&Entry<'a>> for UnitLocationLeaf {
    type Error = LeafParseError<UnitLocationParseCtx>;

    fn try_from(entry: &Entry<'a>) -> Result<Self, Self::Error> {
        if let EntryData::Leaf(leaf) = &entry.data {
            let unit_location = UnitLocationLeaf::try_from(&leaf[..])?;
            Ok(unit_location)
        } else {
            let label = if let EntryData::Immediate(_) = &entry.data {
                "immediate"
            } else if let EntryData::CsrOffset(_) = &entry.data {
                "csr-offset"
            } else if let EntryData::Directory(_) = &entry.data {
                "directory"
            } else {
                unreachable!()
            };
            let msg = format!("{} entry is not available", label);
            Err(Self::Error::new(UnitLocationParseCtx::WrongDirectoryEntry, msg))
        }
    }
}

/// The error context to parse data of unit location leaf.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnitLocationParseCtx {
    TooShort,
    WrongDirectoryEntry,
}

impl std::fmt::Display for UnitLocationParseCtx {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ctx = match self {
            Self::TooShort => "Size of leaf too short",
            Self::WrongDirectoryEntry => "wrong directory entry",
        };
        write!(f, "{}", ctx)
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
        let entry = Entry{key: KeyType::Descriptor, data: EntryData::Leaf(&raw[..])};
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
        let entry = Entry{key: KeyType::Eui64, data: EntryData::Leaf(&raw[..])};
        let eui64 = Eui64Leaf::try_from(&entry).unwrap();
        assert_eq!(0x0001020304050607, eui64.0);
    }

    #[test]
    fn unit_location_from_leaf_entry() {
        let raw = [
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07,
            0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
        ];
        let entry = Entry{key: KeyType::UnitLocation, data: EntryData::Leaf(&raw[..])};
        let unit_location = UnitLocationLeaf::try_from(&entry).unwrap();
        assert_eq!(0x0001020304050607, unit_location.base_addr);
        assert_eq!(0x08090a0b0c0d0e0f, unit_location.upper_bound);
    }
}
