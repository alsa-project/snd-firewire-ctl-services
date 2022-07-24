// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

#![doc = include_str!("../README.md")]

mod entry;
mod leaf;

pub use {entry::*, leaf::*};

use std::convert::TryFrom;

/// The structure to express content of configuration ROM in IEEE 1212.
///
/// The structure implements std::convert::TryFrom<&[u8]> to parse raw data of configuration ROM.
/// The structure refers to content of the raw data, thus has the same lifetime of the raw data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigRom<'a> {
    /// The content of bus information block.
    pub bus_info: &'a [u8],
    /// The directory entries in root directory block.
    pub root: Vec<Entry<'a>>,
}

/// The structure to express error cause to parse raw data of configuration ROM.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigRomParseError {
    pub ctx: Vec<ConfigRomParseCtx>,
    pub msg: String,
}

impl ConfigRomParseError {
    fn new(ctx: ConfigRomParseCtx, msg: String) -> Self {
        ConfigRomParseError {
            ctx: vec![ctx],
            msg,
        }
    }
}

impl std::fmt::Display for ConfigRomParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut ctx = String::new();

        let mut ctx_iter = self.ctx.iter();
        ctx_iter
            .by_ref()
            .nth(0)
            .map(|c| ctx.push_str(&c.to_string()));
        ctx_iter.for_each(|c| {
            ctx.push_str(" -> ");
            ctx.push_str(&c.to_string());
        });

        write!(f, "{}: {}", ctx, self.msg)
    }
}

/// The context to parse configuration ROM.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigRomParseCtx {
    BusInfo,
    RootDirectory,
    DirectoryEntry(u8),
}

impl std::fmt::Display for ConfigRomParseCtx {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigRomParseCtx::BusInfo => write!(f, "bus-info"),
            ConfigRomParseCtx::RootDirectory => write!(f, "root-directory"),
            ConfigRomParseCtx::DirectoryEntry(key) => write!(f, "directory-entry (key: {})", key),
        }
    }
}

impl<'a> TryFrom<&'a [u8]> for ConfigRom<'a> {
    type Error = ConfigRomParseError;

    fn try_from(raw: &'a [u8]) -> Result<Self, Self::Error> {
        // Get block of bus information.
        let ctx = ConfigRomParseCtx::BusInfo;
        let bus_info_length = 4 * raw[0] as usize;
        if 4 + bus_info_length > raw.len() {
            let msg = format!("length {} is greater than {}", bus_info_length, raw.len());
            Err(ConfigRomParseError::new(ctx, msg))
        } else {
            let bus_info = &raw[4..(4 + bus_info_length)];
            let data = &raw[(4 + bus_info_length)..];

            // Get block of root directory.
            let ctx = ConfigRomParseCtx::RootDirectory;
            let doublet = [data[0], data[1]];
            let root_directory_length = 4 * u16::from_be_bytes(doublet) as usize;
            if 4 + root_directory_length > raw.len() {
                let msg = format!(
                    "length {} is greater than {}",
                    root_directory_length,
                    raw.len()
                );
                Err(ConfigRomParseError::new(ctx, msg))
            } else {
                let root = &data[..(4 + root_directory_length)];
                let data = &data[(4 + root_directory_length)..];

                get_directory_entry_list(root, data)
                    .map_err(|mut e| {
                        e.ctx.insert(0, ctx);
                        e
                    })
                    .map(|root| ConfigRom { bus_info, root })
            }
        }
    }
}

fn get_directory_entry_list<'a>(
    mut directory: &'a [u8],
    data: &'a [u8],
) -> Result<Vec<Entry<'a>>, ConfigRomParseError> {
    let mut entries = Vec::new();

    directory = &directory[4..];
    while directory.len() > 0 {
        let entry_type = directory[0] >> 6;
        let key = directory[0] & 0x3f;
        let quadlet = [0, directory[1], directory[2], directory[3]];
        let value = u32::from_be_bytes(quadlet);

        let ctx = ConfigRomParseCtx::DirectoryEntry(key);

        match entry_type {
            0 => Ok(EntryData::Immediate(value)),
            1 => {
                // NOTE: The maximum value of value field in directory entry is 0x00ffffff. The
                // maximum value multipled by 4 is within 0x0fffffff, therefore no need to detect
                // error.
                let offset = 0xfffff0000000 + (4 * value as usize);
                Ok(EntryData::CsrOffset(offset))
            }
            2 | 3 => {
                let offset = 4 * value as usize;
                if offset < directory.len() {
                    let msg = format!("Offset {} reaches no block {}", offset, directory.len());
                    Err(ConfigRomParseError::new(ctx, msg))
                } else {
                    let start_offset = offset - directory.len();
                    if start_offset > data.len() {
                        let msg = format!(
                            "Start offset {} is over blocks {}",
                            start_offset,
                            directory.len()
                        );
                        Err(ConfigRomParseError::new(ctx, msg))
                    } else {
                        let doublet = [data[start_offset], data[start_offset + 1]];
                        let length = 4 * u16::from_be_bytes(doublet) as usize;
                        if length < 4 {
                            let msg = format!("Invalid length of block {}", length);
                            Err(ConfigRomParseError::new(ctx, msg))
                        } else {
                            let end_offset = start_offset + 4 + length;
                            if end_offset > data.len() {
                                let msg = format!(
                                    "End offset {} is over blocks {}",
                                    end_offset,
                                    data.len()
                                );
                                Err(ConfigRomParseError::new(ctx, msg))
                            } else {
                                if entry_type == 2 {
                                    let leaf = &data[(4 + start_offset)..end_offset];
                                    Ok(EntryData::Leaf(leaf))
                                } else {
                                    let directory = &data[start_offset..end_offset];
                                    get_directory_entry_list(directory, &data[end_offset..])
                                        .map_err(|mut e| {
                                            e.ctx.insert(0, ctx);
                                            e
                                        })
                                        .map(|entries| EntryData::Directory(entries))
                                }
                            }
                        }
                    }
                }
            }
            _ => {
                let msg = format!("Invalid type: {}", entry_type);
                Err(ConfigRomParseError::new(ctx, msg))
            }
        }
        .map(|entry_data| {
            entries.push(Entry {
                key: KeyType::from(key),
                data: entry_data,
            });
            directory = &directory[4..];
        })?;
    }

    Ok(entries)
}
