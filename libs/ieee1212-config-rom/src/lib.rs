// SPDX-License-Identifier: MIT
// Copyright (c) 2020 Takashi Sakamoto

#![doc = include_str!("../README.md")]

mod entry;
mod leaf;

pub use {entry::*, leaf::*};

use std::convert::TryFrom;

/// The structure to express content of configuration ROM in IEEE 1212.
///
/// The structure implements std::convert::TryFrom<&[u8]> to parse raw data of configuration ROM,
/// aligned to big-endian. The structure refers to content of the raw data, thus has the same
/// lifetime of the raw data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigRom<'a> {
    /// The content of bus information block.
    pub bus_info: &'a [u8],
    /// The directory entries in root directory block.
    pub root: Vec<Entry<'a>>,
}

/// The error to parse Configuration ROM.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigRomParseError {
    /// The error to parse bus information block.
    BusInfo(
        /// The cause of error.
        BusInfoParseError,
    ),
    /// The error to parse root directory entry.
    RootDirectory(
        /// The start offset of root directory.
        usize,
        /// The index of root directory entry.
        usize,
        /// The cause of error to parse block referred by the root directory entry.
        BlockParseError,
    ),
}

impl std::fmt::Display for ConfigRomParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BusInfo(err) => {
                write!(f, "Detect ill-formed bus info: {}", err)
            }
            Self::RootDirectory(pos, entry_index, err) => {
                write!(
                    f,
                    "Detect ill-formed root directory, pos  {}, entry  {}: {}",
                    pos, entry_index, err
                )
            }
        }
    }
}

/// The error to parse bus information block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BusInfoParseError {
    /// The start offset is beyond boundary.
    OffsetBeyondBoundary(
        /// The offset of block.
        usize,
    ),
    /// The detected length is invalid.
    InvalidLength(
        /// The length of content.
        usize,
    ),
    /// The content is beyond boundary.
    ContentBeyondBoundary(
        /// The start offset of content.
        usize,
        /// The length of content.
        usize,
    ),
    /// The error to parse directory entry.
    Directory(
        /// The start offset of directory.
        usize,
        /// The index of directory entry.
        usize,
        /// The cause of error to parse block referred by the directory entry.
        Box<BlockParseError>,
    ),
}

impl std::fmt::Display for BusInfoParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OffsetBeyondBoundary(offset) => write!(f, "offset {}", offset),
            Self::InvalidLength(length) => {
                write!(f, "invalid length {}", length)
            }
            Self::ContentBeyondBoundary(offset, length) => {
                write!(
                    f,
                    "content beyond boundary, offset {}, length {}",
                    offset, length
                )
            }
            Self::Directory(pos, entry_index, err) => {
                write!(
                    f,
                    "ill-formed directory, pos: {}, entry {}: {}",
                    pos, entry_index, err
                )
            }
        }
    }
}

/// The error to parse block for leaf or directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockParseError {
    /// The start offset is beyond boundary.
    OffsetBeyondBoundary(
        /// The offset of block.
        usize,
    ),
    /// The detected length is invalid.
    InvalidLength(
        /// The length of content.
        usize,
    ),
    /// The content is beyond boundary.
    ContentBeyondBoundary(
        /// The start offset of content.
        usize,
        /// The length of content.
        usize,
    ),
    /// The error to parse directory entry.
    Directory(
        /// The start offset of directory.
        usize,
        /// The index of directory entry.
        usize,
        /// The cause of error.
        Box<BlockParseError>,
    ),
}

impl std::fmt::Display for BlockParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OffsetBeyondBoundary(offset) => {
                write!(f, "offset {}", offset)
            }
            Self::InvalidLength(length) => {
                write!(f, "invalid length {}", length)
            }
            Self::ContentBeyondBoundary(offset, length) => {
                write!(
                    f,
                    "content beyond boundary, offset {}, length {}",
                    offset, length
                )
            }
            Self::Directory(pos, entry_index, err) => {
                write!(
                    f,
                    "ill-formed directory, pos: {}, entry {}: {}",
                    pos, entry_index, err
                )
            }
        }
    }
}

fn detect_block(raw: &[u8], pos: usize, offset: usize) -> Result<(usize, usize), BlockParseError> {
    let mut start_offset = pos + offset;
    if start_offset > raw.len() {
        Err(BlockParseError::OffsetBeyondBoundary(start_offset))
    } else {
        let doublet = [raw[start_offset], raw[start_offset + 1]];
        let length = 4 * u16::from_be_bytes(doublet) as usize;
        if length < 4 {
            Err(BlockParseError::InvalidLength(length))
        } else {
            start_offset += 4;
            if start_offset + length > raw.len() {
                Err(BlockParseError::ContentBeyondBoundary(start_offset, length))
            } else {
                Ok((start_offset, length))
            }
        }
    }
}

impl<'a> TryFrom<&'a [u8]> for ConfigRom<'a> {
    type Error = ConfigRomParseError;

    fn try_from(raw: &'a [u8]) -> Result<Self, Self::Error> {
        let mut pos = 0;

        let bus_info_length = 4 * raw[pos] as usize;
        pos += 4;

        if pos + bus_info_length > raw.len() {
            Err(Self::Error::BusInfo(BusInfoParseError::InvalidLength(
                bus_info_length,
            )))
        } else {
            let bus_info = &raw[pos..(pos + bus_info_length)];
            pos += bus_info_length;

            detect_block(raw, pos, 0)
                .and_then(|(start_offset, length)| {
                    get_directory_entry_list(raw, start_offset, length)
                        .map(|root| ConfigRom { bus_info, root })
                })
                .map_err(|err| Self::Error::RootDirectory(pos, 0, err))
        }
    }
}

const ENTRY_KEY_IMMEDIATE: u8 = 0;
const ENTRY_KEY_CSR_OFFSET: u8 = 1;
const ENTRY_KEY_LEAF: u8 = 2;
const ENTRY_KEY_DIRECTORY: u8 = 3;

fn get_directory_entry_list<'a>(
    raw: &'a [u8],
    directory_pos: usize,
    directory_length: usize,
) -> Result<Vec<Entry<'a>>, BlockParseError> {
    let mut entries = Vec::new();

    let mut pos = directory_pos;

    while pos < directory_pos + directory_length {
        let entry_type = raw[pos] >> 6;
        let key = raw[pos] & 0x3f;
        let quadlet = [0, raw[pos + 1], raw[pos + 2], raw[pos + 3]];
        let value = u32::from_be_bytes(quadlet);

        match entry_type {
            ENTRY_KEY_IMMEDIATE => Ok(EntryData::Immediate(value)),
            ENTRY_KEY_CSR_OFFSET => {
                // NOTE: The maximum value of value field in directory entry is 0x00ffffff. The
                // maximum value multipled by 4 is within 0x0fffffff, therefore no need to detect
                // error.
                let offset = 0xfffff0000000 + (4 * value as usize);
                Ok(EntryData::CsrOffset(offset))
            }
            ENTRY_KEY_LEAF => {
                let offset = 4 * value as usize;
                detect_block(raw, pos, offset)
                    .map_err(|err| BlockParseError::Directory(pos, offset, Box::new(err)))
                    .map(|(start_offset, length)| {
                        let leaf = &raw[start_offset..(start_offset + length)];
                        EntryData::Leaf(leaf)
                    })
            }
            ENTRY_KEY_DIRECTORY => {
                let offset = 4 * value as usize;
                detect_block(raw, pos, offset)
                    .and_then(|(start_offset, length)| {
                        get_directory_entry_list(raw, start_offset + 4, length)
                            .map(|entries| EntryData::Directory(entries))
                    })
                    .map_err(|err| BlockParseError::Directory(pos, offset, Box::new(err)))
            }
            // NOTE: The field of key has two bits, thus it can not be over 0x03.
            _ => unreachable!(),
        }
        .map(|entry_data| {
            entries.push(Entry {
                key: KeyType::from(key),
                data: entry_data,
            });
            pos += 4;
        })?;
    }

    Ok(entries)
}
