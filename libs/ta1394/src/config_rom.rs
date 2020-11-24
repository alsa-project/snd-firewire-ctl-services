// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use ieee1212_config_rom::{*, entry::*};

use std::convert::TryFrom;

#[derive(Clone, Debug)]
pub struct VendorData {
    pub vendor_id: u32,
    pub vendor_name: String,
}

#[derive(Clone, Debug)]
pub struct UnitData {
    pub model_id: u32,
    pub model_name: String,
    pub specifier_id: u32,
    pub version: u32,
}

pub trait Ta1394ConfigRom {
    fn get_vendor(&self) -> Option<VendorData>;
    fn get_model(&self) -> Option<UnitData>;
}

impl<'a> Ta1394ConfigRom for ConfigRom<'a> {
    fn get_vendor(&self) -> Option<VendorData> {
        detect_desc_text(&self.root, KeyType::Vendor)
            .map(|(vendor_id, name)| VendorData{vendor_id, vendor_name: name.to_string()})
    }

    fn get_model(&self) -> Option<UnitData> {
        get_unit_data(&self.root, 0)
    }
}

pub fn parse_entries(data: &[u8]) -> Option<(VendorData, UnitData)> {
    ConfigRom::try_from(data)
        .ok()
        .and_then(|config_rom| {
            detect_desc_text(&config_rom.root, KeyType::Vendor)
                .map(|(vendor_id, name)| VendorData{vendor_id, vendor_name: name.to_string()})
                .and_then(|vendor| {
                    get_unit_data(&config_rom.root, 0)
                        .map(|model| (vendor, model))
                })
        })
}

fn get_unit_data(entries: &[Entry], directory_id: u32) -> Option<UnitData> {
    entries.iter().filter_map(|entry| {
        EntryDataAccess::<&[Entry]>::get(entry, KeyType::Unit)
    })
    .nth(directory_id as usize)
    .and_then(|entries| {
        entries.iter().find_map(|entry| {
            EntryDataAccess::<u32>::get(entry, KeyType::SpecifierId)
        })
        .and_then(|specifier_id| {
            entries.iter().find_map(|entry| {
                EntryDataAccess::<u32>::get(entry, KeyType::Version)
            })
            .and_then(|version| {
                detect_desc_text(entries, KeyType::Model)
                    .map(|(model_id, name)| {
                        UnitData{model_id, model_name: name.to_string(), specifier_id, version}
                    })
            })
        })
    })
}

fn detect_desc_text<'a>(entries: &'a [Entry], key_type: KeyType) -> Option<(u32, &'a str)> {
    let mut peekable = entries.iter().peekable();

    while let Some(entry) = peekable.next()
    {
        let result = EntryDataAccess::<u32>::get(entry, key_type)
            .and_then(|value| {
                peekable.peek()
                    .and_then(|&next| {
                        EntryDataAccess::<&str>::get(next, KeyType::Descriptor)
                            .map(|name| (value, name))
                    })
            });

        if result.is_some() {
            return result;
        }
    }

    None
}
