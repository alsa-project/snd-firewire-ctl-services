// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
mod isoc_console_runtime;
mod isoc_rack_runtime;
mod async_runtime;

mod fw1082_model;
mod fw1884_model;
mod fw1804_model;
mod fe8_model;

mod protocol;

mod common_ctl;
mod optical_ctl;
mod console_ctl;
mod rack_ctl;

mod isoch_ctls;

mod seq_cntr;

use glib::{Error, FileError};

use hinawa::{FwNodeExt, FwNodeExtManual, SndUnitExt, SndTscmExt};

use core::RuntimeOperation;

use ieee1212_config_rom::{*, entry::*};

use isoc_console_runtime::IsocConsoleRuntime;
use isoc_rack_runtime::IsocRackRuntime;
use async_runtime::AsyncRuntime;

use std::convert::TryFrom;

pub enum TascamRuntime<'a> {
    IsocConsole(IsocConsoleRuntime<'a>),
    IsocRack(IsocRackRuntime<'a>),
    Async(AsyncRuntime),
}

impl<'a> RuntimeOperation<(String, u32)> for TascamRuntime<'a> {
    fn new((subsystem, sysnum): (String, u32)) -> Result<Self, Error> {
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
                        let runtime = IsocConsoleRuntime::new(unit, name, sysnum)?;
                        Ok(Self::IsocConsole(runtime))
                    }
                    "FW-1804" => {
                        let runtime = IsocRackRuntime::new(unit, name, sysnum)?;
                        Ok(Self::IsocRack(runtime))
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
                        let runtime = AsyncRuntime::new(node, name)?;
                        Ok(Self::Async(runtime))
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

    fn listen(&mut self) -> Result<(), Error> {
        match self {
            Self::IsocConsole(unit) => unit.listen(),
            Self::IsocRack(unit) => unit.listen(),
            Self::Async(unit) => unit.listen(),
        }
    }

    fn run(&mut self) -> Result<(), Error> {
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
