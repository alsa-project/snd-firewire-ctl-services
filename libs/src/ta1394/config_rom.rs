// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use crate::ieee1212;

use ieee1212::{Entry, EntryData, KeyType};

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

pub fn parse_entries(data: &[u8]) -> Option<(VendorData, UnitData)> {
    let entries = ieee1212::get_root_entry_list(&data);

    match get_vendor_data(&entries) {
        Some(vendor) => match get_unit_data(&entries, 0) {
            Some(model) => Some((vendor, model)),
            None => None,
        }
        None => None,
    }
}

pub fn get_vendor_data(entries: &Vec<Entry>) -> Option<VendorData> {
    match detect_desc_text(entries, KeyType::Vendor) {
        Some((vendor_id, vendor_name)) => Some(VendorData{vendor_id, vendor_name}),
        None => None,
    }
}

pub fn get_unit_data(entries: &Vec<Entry>, directory_id: u32) -> Option<UnitData> {
    match detect_unit_directory(entries, directory_id) {
        Some(directory) => match detect_immediate_value(directory, KeyType::SpecifierId) {
            Some(specifier_id) => match detect_immediate_value(directory, KeyType::Version) {
                Some(version) => match detect_desc_text(directory, KeyType::Model) {
                    Some((model_id, model_name)) => Some(UnitData{model_id, model_name, specifier_id, version}),
                    None => None,
                }
                None => None,
            },
            None => None,
        },
        None => None,
    }
}

fn detect_immediate_value(entries: &Vec<Entry>, key: KeyType) -> Option<u32> {
    entries.iter().find_map(|entry| {
        if entry.key == key as u8 {
            match entry.data {
                EntryData::Immediate(spec_id) => Some(spec_id),
                _ => None,
            }
        } else {
            None
        }
    })
}

fn detect_unit_directory(entries: &Vec<Entry>, directory_id: u32) -> Option<&Vec<Entry>> {
    let mut count = 0;

    entries.iter().find_map(|entry| {
        if entry.key == KeyType::Unit as u8 {
            match &entry.data {
                EntryData::Directory(directory) => {
                    match count == directory_id {
                        true => Some(directory),
                        false => {
                            count += 1;
                            None
                        }
                    }
                },
                _ => None,
            }
        } else {
            None
        }
    })
}

fn detect_desc_text(entries: &Vec<Entry>, key: KeyType) -> Option<(u32, String)> {
    let mut peekable = entries.iter().peekable();

    while let Some(entry) = peekable.next() {
        if entry.key != key as u8 {
            continue;
        }

        let next = match peekable.peek() {
            Some(n) => {
                if n.key == KeyType::Descriptor as u8 {
                    n
                } else {
                    continue
                }
            },
            None => continue,
        };

        if let EntryData::Immediate(value) = entry.data {
            if let EntryData::Leaf(leaf) = &next.data {
                if let Some(name) = ieee1212::parse_leaf_entry_as_text(&leaf) {
                    return Some((value, name));
                }
            }
        }
    }

    None
}
