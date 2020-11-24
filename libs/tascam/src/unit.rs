// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::{Error, FileError};

use hinawa::{FwNodeExt, FwNodeExtManual, SndUnitExt, SndTscmExt};

use ieee1212_config_rom::{*, entry::*};

use super::isoc_console_unit::IsocConsoleUnit;
use super::isoc_rack_unit::IsocRackUnit;
use super::async_unit::AsyncUnit;

use std::convert::TryFrom;

pub enum TascamUnit<'a> {
    IsocConsole(IsocConsoleUnit<'a>),
    IsocRack(IsocRackUnit<'a>),
    Async(AsyncUnit),
}

impl<'a> TascamUnit<'a> {
    pub fn new(subsystem: &String, sysnum: u32) -> Result<Self, Error> {
        match subsystem.as_str() {
            "snd" => {
                let unit = hinawa::SndTscm::new();
                let devnode = format!("/dev/snd/hwC{}D0", sysnum);
                unit.open(&devnode)?;

                let node = unit.get_node();
                let data = node.get_config_rom()?;
                let config_rom = ConfigRom::try_from(data)
                    .map_err(|e| {
                        let label = format!("Malformed configuration ROM detected: {}", e.to_string());
                        Error::new(FileError::Nxio, &label)
                    })?;
                let name = detect_model_name(&config_rom.root)?;
                match name {
                    "FW-1884" | "FW-1082" => {
                        let console_unit = IsocConsoleUnit::new(unit, name, sysnum)?;
                        Ok(Self::IsocConsole(console_unit))
                    }
                    "FW-1804" => {
                        let rack_unit = IsocRackUnit::new(unit, name, sysnum)?;
                        Ok(Self::IsocRack(rack_unit))
                    }
                    _ => Err(Error::new(FileError::Noent, "Not supported")),
                }
            }
            "fw" => {
                let node = hinawa::FwNode::new();
                let devnode = format!("/dev/fw{}", sysnum);
                node.open(&devnode)?;

                let data = node.get_config_rom()?;
                let config_rom = ConfigRom::try_from(data)
                    .map_err(|e| {
                        let label = format!("Malformed configuration ROM detected: {}", e.to_string());
                        Error::new(FileError::Nxio, &label)
                    })?;
                let name = detect_model_name(&config_rom.root)?;
                match name {
                    "FE-8" => {
                        let name = name.to_string();
                        let async_unit = AsyncUnit::new(node, name)?;
                        Ok(Self::Async(async_unit))
                    }
                    _ => Err(Error::new(FileError::Noent, "Not supported")),
                }
            }
            _ => {
                let label = "Invalid name of subsystem";
                Err(Error::new(FileError::Nodev, &label))
            }
        }
    }

    pub fn listen(&mut self) -> Result<(), Error> {
        match self {
            Self::IsocConsole(unit) => unit.listen(),
            Self::IsocRack(unit) => unit.listen(),
            Self::Async(unit) => unit.listen(),
        }
    }

    pub fn run(&mut self) {
        match self {
            Self::IsocConsole(unit) => unit.run(),
            Self::IsocRack(unit) => unit.run(),
            Self::Async(unit) => unit.run(),
        }
    }
}

fn detect_model_name<'a>(entries: &'a [Entry]) -> Result<&'a str, Error> {
    entries.iter().find_map(|entry| {
        EntryDataAccess::<&[Entry]>::get(entry, KeyType::Unit)
            .and_then(|entries| {
                entries.iter().find_map(|entry| {
                    EntryDataAccess::<&[Entry]>::get(entry, KeyType::DependentInfo)
                        .and_then(|entries| {
                            entries.iter().find_map(|entry| {
                                EntryDataAccess::<&str>::get(entry, KeyType::BusDependentInfo)
                            })
                        })
                })
            })
    })
    .ok_or_else(|| {
        let label = "Invalid format of configuration ROM";
        Error::new(FileError::Nxio, &label)
    })
}
