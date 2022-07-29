// SPDX-License-Identifier: MIT
// Copyright (c) 2022 Takashi Sakamoto

//! Typical data layout of Configuration ROM for AV/C devices defined by 1394 Trading Association.

use ieee1212_config_rom::*;

/// The data of vendor.
#[derive(Clone, Debug)]
pub struct VendorData<'a> {
    pub vendor_id: u32,
    pub vendor_name: &'a str,
}

/// The data of unit.
#[derive(Clone, Debug)]
pub struct UnitData<'a> {
    pub model_id: u32,
    pub model_name: &'a str,
    pub specifier_id: u32,
    pub version: u32,
}

/// For detection of typical layout.
pub trait Ta1394ConfigRom<'a> {
    fn get_vendor(&'a self) -> Option<VendorData<'a>>;
    fn get_model(&'a self) -> Option<UnitData<'a>>;
}

impl<'a> Ta1394ConfigRom<'a> for ConfigRom<'a> {
    fn get_vendor(&'a self) -> Option<VendorData<'a>> {
        detect_desc_text(&self.root, KeyType::Vendor).map(|(vendor_id, vendor_name)| VendorData {
            vendor_id,
            vendor_name,
        })
    }

    fn get_model(&'a self) -> Option<UnitData<'a>> {
        self.root
            .iter()
            .find_map(|entry| EntryDataAccess::<&[Entry]>::get(entry, KeyType::Unit))
            .and_then(|entries| {
                entries
                    .iter()
                    .find_map(|entry| EntryDataAccess::<u32>::get(entry, KeyType::SpecifierId))
                    .and_then(|specifier_id| {
                        entries
                            .iter()
                            .find_map(|entry| EntryDataAccess::<u32>::get(entry, KeyType::Version))
                            .and_then(|version| {
                                detect_desc_text(entries, KeyType::Model).map(
                                    |(model_id, model_name)| UnitData {
                                        model_id,
                                        model_name,
                                        specifier_id,
                                        version,
                                    },
                                )
                            })
                    })
            })
    }
}

fn detect_desc_text<'a>(entries: &'a [Entry], key_type: KeyType) -> Option<(u32, &'a str)> {
    let mut peekable = entries.iter().peekable();

    while let Some(entry) = peekable.next() {
        let result = EntryDataAccess::<u32>::get(entry, key_type).and_then(|value| {
            peekable.peek().and_then(|&next| {
                EntryDataAccess::<&str>::get(next, KeyType::Descriptor).map(|name| (value, name))
            })
        });

        if result.is_some() {
            return result;
        }
    }

    None
}
