// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! The crate includes structures, enumerations, traits and its implementations to process
//! content of Configuration ROM in IEEE 1212.
//!
//! ## Structures and enumerations
//!
//! `ConfigRom` structure represents structured data of Configuration ROM. The structure
//! implements std::convert::TryFrom<&[u8]> to parse raw data of Configuration ROM. The
//! lifetime of `ConfigRom` structure is the same as the one of raw data, to save memory
//! consumption for string.
//!
//! The `root` member of `ConfigRom` structure is a vector of `Entry` structure, which
//! represents directory entry. In the `Entry` structure, `key` member is typed as `KeyType`
//! for the type of key, and `data` member is typed as `EntryData` to dispatch four types
//! of data in the entry.
//!
//! In IEEE 1212, text descriptor of leaf entry includes string information. `Descriptor`
//! structure is used to parse the descriptor.
//!
//! For convenience, `EntryDataAccess` trait is available to access several type of data in
//! each entry by key.
//!
//! ## Usage
//!
//! ```rust
//! use ieee1212_config_rom::ConfigRom;
//! use ieee1212_config_rom::entry::{Entry, KeyType, EntryData, EntryDataAccess};
//! use ieee1212_config_rom::desc::{Descriptor, DescriptorData, TextualDescriptorData};
//! use std::convert::TryFrom;
//!
//! // Prepare raw data of Configuration ROM as array with u8 elements aligned by big endian.
//! let raw =  [
//!     0x04, 0x04, 0x7f, 0x1a, 0x31, 0x33, 0x39, 0x34,
//!     0xf0, 0x00, 0xb2, 0x23, 0x08, 0x00, 0x28, 0x51,
//!     0x01, 0x00, 0x36, 0x22, 0x00, 0x05, 0x1b, 0x70,
//!     0x0c, 0x00, 0x83, 0xc0, 0x03, 0x00, 0x1f, 0x11,
//!     0x81, 0x00, 0x00, 0x03, 0x17, 0x02, 0x39, 0x01,
//!     0x81, 0x00, 0x00, 0x08, 0x00, 0x06, 0x4c, 0xb7,
//!     0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
//!     0x4c, 0x69, 0x6e, 0x75, 0x78, 0x20, 0x46, 0x69,
//!     0x72, 0x65, 0x77, 0x69, 0x72, 0x65, 0x00, 0x00,
//!     0x00, 0x03, 0xff, 0x1c, 0x00, 0x00, 0x00, 0x00,
//!     0x00, 0x00, 0x00, 0x00, 0x4a, 0x75, 0x6a, 0x75,
//! ];
//!
//! let config_rom = ConfigRom::try_from(&raw[..]).unwrap();
//! assert_eq!(KeyType::Vendor, config_rom.root[1].key);
//!
//! if let EntryData::Immediate(value) = config_rom.root[1].data {
//!     assert_eq!(0x001f11, value);
//! } else {
//!     unreachable!();
//! }
//! let desc = Descriptor::try_from(&config_rom.root[2]).unwrap();
//! if let DescriptorData::Textual(d) = desc.data {
//!     assert_eq!("Linux Firewire", d.text);
//! } else {
//!     unreachable!();
//! }
//!
//! let model_id = EntryDataAccess::<u32>::get(&config_rom.root[3], KeyType::Model).unwrap();
//! assert_eq!(0x023901, model_id);
//!
//! let model_name = EntryDataAccess::<&str>::get(&config_rom.root[4], KeyType::Descriptor).unwrap();
//! assert_eq!("Juju", model_name);
//! ```
//!
//! ## Utilities
//!
//! Some programs are available under `src/bin` directory.
//!
//! ### src/bin/config-rom-parser
//!
//! This program parses raw data of Configuration ROM from stdin, or image file as arguments of
//! command line.
//!
//! Without any command line argument, it prints help message and exit.
//!
//! ```sh
//! $ cargo run --bin config-rom-parser
//! Usage:
//!   config-rom-parser FILENAME | "-"
//!
//!   where:
//!     FILENAME:       the name of file for the image of configuration ROM to parse
//!     "-":            the content of configuration ROM to parse. It should be aligned to big endian.
//! ```
//!
//! For data of Configuration ROM in file:
//!
//! ```sh
//! $ cargo run --bin config-rom-parser -- /sys/bus/firewire/devices/fw0/config_rom
//! ```
//!
//! For data of Configuration ROM from stdin:
//!
//! ```sh
//! $ cat /sys/bus/firewire/devices/fw0/config_rom  | cargo run --bin config-rom-parser -- -
//! ```
//!
//! In both cases, the content to be parsed should be aligned to big-endian order.

pub mod entry;
pub mod desc;

use entry::*;

use std::convert::TryFrom;

/// The structure to represent content of configuration ROM in IEEE 1212.
///
/// The structure implements std::convert::TryFrom<&[u8]> to parse raw data of configuration ROM.
/// The structure refers to content of the raw data, thus has the same lifetime of the raw data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigRom<'a>{
    /// The content of bus information block.
    pub bus_info: &'a [u8],
    /// The directory entries in root directory block.
    pub root: Vec<Entry<'a>>,
}

/// The structure to represent error cause to parse raw data of configuration ROM.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigRomParseError{
    pub ctx: Vec<ConfigRomParseCtx>,
    pub msg: String,
}

impl ConfigRomParseError {
    fn new(ctx: ConfigRomParseCtx, msg: String) -> Self {
        ConfigRomParseError{ctx: vec![ctx], msg}
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
        ctx_iter
            .for_each(|c| {
                ctx.push_str(" -> ");
                ctx.push_str(&c.to_string());
            });

        write!(f, "{}: {}", ctx, self.msg)
    }
}

/// The enumeration to represent context of parsing.
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
            Err(ConfigRomParseError::new(ctx, msg))?
        }
        let bus_info = &raw[..(4 + bus_info_length)];
        let data = &raw[(4 + bus_info_length)..];

        // Get block of root directory.
        let ctx = ConfigRomParseCtx::RootDirectory;
        let doublet = [data[0], data[1]];
        let root_directory_length = 4 * u16::from_be_bytes(doublet) as usize;
        if 4 + root_directory_length > raw.len() {
            let msg = format!("length {} is greater than {}", root_directory_length, raw.len());
            Err(ConfigRomParseError::new(ctx, msg))?
        }
        let root = &data[..(4 + root_directory_length)];
        let data = &data[(4 + root_directory_length)..];

        let root = get_directory_entry_list(root, data)
            .map_err(|mut e| {
                e.ctx.insert(0, ctx);
                e
            })?;

        let rom = ConfigRom{
            bus_info,
            root,
        };
        Ok(rom)
    }
}

fn get_directory_entry_list<'a>(mut directory: &'a [u8], data: &'a [u8])
    -> Result<Vec<Entry<'a>>, ConfigRomParseError>
{
    let mut entries = Vec::new();

    directory = &directory[4..];
    while directory.len() > 0 {
        let entry_type = directory[0] >> 6;
        let key = directory[0] & 0x3f;
        let quadlet = [0, directory[1], directory[2], directory[3]];
        let value = u32::from_be_bytes(quadlet);

        let ctx = ConfigRomParseCtx::DirectoryEntry(key);

        let entry_data = match entry_type {
            0 => EntryData::Immediate(value),
            1 => {
                let offset = 0xfffff0000000 + (value as usize);
                EntryData::CsrOffset(offset)
            }
            2 | 3 => {
                let offset = 4 * value as usize;
                if offset < directory.len() {
                    let msg = format!("Offset {} reaches no block {}", offset, directory.len());
                    Err(ConfigRomParseError::new(ctx, msg))?
                }
                let start_offset = offset - directory.len();
                if start_offset > data.len() {
                    let msg = format!("Start offset {} is over blocks {}", start_offset, directory.len());
                    Err(ConfigRomParseError::new(ctx, msg))?
                }
                let doublet = [data[start_offset], data[start_offset + 1]];
                let length = 4 * u16::from_be_bytes(doublet) as usize;
                if length < 8 {
                    let msg = format!("Invalid length of block {}", length);
                    Err(ConfigRomParseError::new(ctx, msg))?
                }
                let end_offset = start_offset + 4 + length;
                if end_offset > data.len() {
                    let msg = format!("End offset {} is over blocks {}", end_offset, data.len());
                    Err(ConfigRomParseError::new(ctx, msg))?
                }
                if entry_type == 2 {
                    let leaf = &data[(4 + start_offset)..end_offset];
                    EntryData::Leaf(leaf)
                } else {
                    let directory = &data[start_offset..end_offset];
                    let entries = get_directory_entry_list(directory, &data[end_offset..])
                        .map_err(|mut e| {
                            e.ctx.insert(0, ctx);
                            e
                        })?;
                    EntryData::Directory(entries)
                }
            }
            _ => {
                let msg = format!("Invalid type: {}", entry_type);
                Err(ConfigRomParseError::new(ctx, msg))?
            }
        };
        let entry = Entry {
            key: KeyType::from(key),
            data: entry_data,
        };
        entries.push(entry);

        directory = &directory[4..];
    }

    Ok(entries)
}
