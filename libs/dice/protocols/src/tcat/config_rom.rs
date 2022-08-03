// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Parser for Configuration ROM according to specification defined by TCAT for ASICs of DICE.
//!
//! The module includes structure, enumeration, and trait and its implementaion to parse
//! configuration ROM according to specification defined by TCAT for ASICs of DICE.

use ieee1212_config_rom::*;

/// The structure to represent identifier in bus information block of Configuration ROM.
#[derive(Default, Debug, Clone, Copy)]
pub struct Identifier {
    pub vendor_id: u32,
    pub category: u8,
    pub product_id: u16,
    pub serial: u32,
}

/// The structure to represent data in root directory block of Configuration ROM.
#[derive(Default, Debug, Clone, Copy)]
pub struct RootData<'a> {
    pub vendor_id: u32,
    pub vendor_name: &'a str,
    pub product_id: u32,
    pub product_name: &'a str,
}

/// The structure to represent data in unit directory block of Configuration ROM.
#[derive(Default, Debug, Clone, Copy)]
pub struct UnitData<'a> {
    pub model_id: u32,
    pub model_name: &'a str,
    pub specifier_id: u32,
    pub version: u32,
}

pub trait DiceConfigRom<'a> {
    const VENDOR_ID_MASK: u32 = 0xffffff00;
    const VENDOR_ID_SHIFT: usize = 8;
    const CATEGORY_MASK: u32 = 0x000000ff;
    const CATEGORY_SHIFT: usize = 0;
    const PRODUCT_ID_MASK: u32 = 0xffc00000;
    const PRODUCT_ID_SHIFT: usize = 22;
    const SERIAL_MASK: u32 = 0x003fffff;
    const SERIAL_SHIFT: usize = 0;

    fn get_identifier(&self) -> Option<Identifier>;
    fn get_root_data(&'a self) -> Option<RootData<'a>>;
    fn get_unit_data(&'a self) -> Option<UnitData<'a>>;
}

impl<'a> DiceConfigRom<'a> for ConfigRom<'a> {
    fn get_identifier(&self) -> Option<Identifier> {
        if self.bus_info.len() < 12 {
            None
        } else {
            let mut quadlet = [0; 4];

            quadlet.copy_from_slice(&self.bus_info[8..12]);
            let val = u32::from_be_bytes(quadlet);
            let vendor_id = (val & Self::VENDOR_ID_MASK) >> Self::VENDOR_ID_SHIFT;
            let category = ((val & Self::CATEGORY_MASK) >> Self::CATEGORY_SHIFT) as u8;

            quadlet.copy_from_slice(&self.bus_info[12..16]);
            let val = u32::from_be_bytes(quadlet);
            let product_id = ((val & Self::PRODUCT_ID_MASK) >> Self::PRODUCT_ID_SHIFT) as u16;
            let serial = (val & Self::SERIAL_MASK) >> Self::SERIAL_SHIFT;

            Some(Identifier {
                vendor_id,
                category,
                product_id,
                serial,
            })
        }
    }

    fn get_root_data(&'a self) -> Option<RootData<'a>> {
        detect_desc_text(&self.root, KeyType::Vendor).and_then(|(vendor_id, vendor_name)| {
            detect_desc_text(&self.root, KeyType::Model).map(|(product_id, product_name)| {
                RootData {
                    vendor_id,
                    vendor_name,
                    product_id,
                    product_name,
                }
            })
        })
    }

    fn get_unit_data(&'a self) -> Option<UnitData<'a>> {
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
