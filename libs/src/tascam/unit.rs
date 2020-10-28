// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::{Error, FileError};

use hinawa::{FwNodeExt, FwNodeExtManual, SndUnitExt, SndTscmExt};

use crate::ieee1212;

use super::isoc_console_unit::IsocConsoleUnit;
use super::isoc_rack_unit::IsocRackUnit;
use super::async_unit::AsyncUnit;

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
                let name = detect_model_name(&node)?;
                match name.as_str() {
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

                let name = detect_model_name(&node)?;
                match name.as_str() {
                    "FE-8" => {
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

fn detect_model_name(node: &hinawa::FwNode) -> Result<String, Error> {
    let data = node.get_config_rom()?;

    ieee1212::get_root_entry_list(data).iter().find_map(|entry| {
        if entry.key == ieee1212::KeyType::Unit as u8 {
            if let ieee1212::EntryData::Directory(dir) = &entry.data {
                dir.iter().find_map(|de| {
                    if de.key == ieee1212::KeyType::DependentInfo as u8 {
                        if let ieee1212::EntryData::Directory(d) = &de.data {
                            d.iter().find_map(|e| {
                                if e.key == ieee1212::KeyType::BusDependentInfo as u8 {
                                    if let ieee1212::EntryData::Leaf(l) = &e.data {
                                        ieee1212::parse_leaf_entry_as_text(l)
                                    } else {
                                        None
                                    }
                                } else{
                                    None
                                }
                            })
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
            } else {
                None
            }
        } else {
            None
        }
    }).ok_or_else(|| {
        let label = "Invalid format of configuration ROM";
        Error::new(FileError::Nxio, &label)
    })
}
