// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Parser for Configuration ROM.
//!
//! The module includes structure, enumeration, and trait and its implementaion to parse
//! configuration ROM according to specification defined by Mark of the Unicorn.

use ieee1212_config_rom::*;

/// Data in unit directory of Configuration ROM.
#[derive(Default, Debug, Clone, Copy)]
pub struct UnitData {
    pub model_id: u32,
    pub version: u32,
}

const OUI_MOTU: u32 = 0x0001f2;

/// The trait for parser of configuration ROM.
pub trait MotuConfigRom {
    fn get_unit_data(&self) -> Option<UnitData>;
}

impl<'a> MotuConfigRom for ConfigRom<'a> {
    fn get_unit_data(&self) -> Option<UnitData> {
        self.root
            .iter()
            .find_map(|entry| {
                EntryDataAccess::<u32>::get(entry, KeyType::Vendor).and_then(|vendor_id| {
                    if vendor_id == OUI_MOTU {
                        Some(vendor_id)
                    } else {
                        None
                    }
                })
            })
            .and_then(|_| {
                self.root
                    .iter()
                    .find_map(|entry| EntryDataAccess::<&[Entry]>::get(entry, KeyType::Unit))
                    .and_then(|entries| {
                        entries
                            .iter()
                            .find_map(|entry| {
                                EntryDataAccess::<u32>::get(entry, KeyType::SpecifierId).and_then(
                                    |specifier_id| {
                                        if specifier_id == OUI_MOTU {
                                            Some(specifier_id)
                                        } else {
                                            None
                                        }
                                    },
                                )
                            })
                            .and_then(|_| {
                                // NOTE: It's odd but version field is used for model ID and model field is
                                // used for version in MOTU case.
                                entries
                                    .iter()
                                    .find_map(|entry| {
                                        EntryDataAccess::<u32>::get(entry, KeyType::Version)
                                    })
                                    .and_then(|model_id| {
                                        entries
                                            .iter()
                                            .find_map(|entry| {
                                                EntryDataAccess::<u32>::get(entry, KeyType::Model)
                                            })
                                            .map(|version| UnitData { model_id, version })
                                    })
                            })
                    })
            })
    }
}
