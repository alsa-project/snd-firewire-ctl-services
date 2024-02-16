// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Parser for Configuration ROM according to specification defined by TCAT for ASICs of DICE.
//!
//! The module includes structure, enumeration, and trait and its implementaion to parse
//! configuration ROM according to specification defined by TCAT for ASICs of DICE.

use ieee1212_config_rom::*;

/// Identifier in bus information block of Configuration ROM.
#[derive(Default, Debug, Clone, Copy)]
pub struct Identifier {
    /// The numeric identifier of vendor, usually Organizationally Unique Identifier (OUI)
    /// registered in IEEE.
    pub vendor_id: u32,
    /// The numeric value of category.
    pub category: u8,
    /// The numeric identifier of product.
    pub product_id: u16,
    /// The serial number.
    pub serial: u32,
}

/// Data in root directory block of Configuration ROM.
#[derive(Default, Debug, Clone, Copy)]
pub struct RootData<'a> {
    /// The numeric identifier of vendor, usually Organizationally Unique Identifier (OUI)
    /// registered in IEEE.
    pub vendor_id: u32,
    /// The name of vendor.
    pub vendor_name: &'a str,
    /// The numeric identifier of product.
    pub product_id: u32,
    /// The name of product.
    pub product_name: &'a str,
}

/// Data in unit directory block of Configuration ROM.
#[derive(Default, Debug, Clone, Copy)]
pub struct UnitData<'a> {
    /// The numeric identifier of model.
    pub model_id: u32,
    /// The name of model.
    pub model_name: &'a str,
    /// The specifier identifier.
    pub specifier_id: u32,
    /// Version.
    pub version: u32,
}

const VENDOR_ID_MASK: u32 = 0xffffff00;
const VENDOR_ID_SHIFT: usize = 8;
const CATEGORY_MASK: u32 = 0x000000ff;
const CATEGORY_SHIFT: usize = 0;
const PRODUCT_ID_MASK: u32 = 0xffc00000;
const PRODUCT_ID_SHIFT: usize = 22;
const SERIAL_MASK: u32 = 0x003fffff;
const SERIAL_SHIFT: usize = 0;

/// Parser of configuration ROM which has layout specific to DICE.
pub trait DiceConfigRom<'a> {
    /// Get identifier.
    fn get_identifier(&self) -> Option<Identifier>;
    /// Get data in root directory.
    fn get_root_data(&'a self) -> Option<RootData<'a>>;
    /// Get data in unit directory.
    fn get_unit_data(&'a self) -> Vec<UnitData<'a>>;
}

impl<'a> DiceConfigRom<'a> for ConfigRom<'a> {
    fn get_identifier(&self) -> Option<Identifier> {
        if self.bus_info.len() < 12 {
            None
        } else {
            let mut quadlet = [0; 4];

            quadlet.copy_from_slice(&self.bus_info[8..12]);
            let val = u32::from_be_bytes(quadlet);
            let vendor_id = (val & VENDOR_ID_MASK) >> VENDOR_ID_SHIFT;
            let category = ((val & CATEGORY_MASK) >> CATEGORY_SHIFT) as u8;

            quadlet.copy_from_slice(&self.bus_info[12..16]);
            let val = u32::from_be_bytes(quadlet);
            let product_id = ((val & PRODUCT_ID_MASK) >> PRODUCT_ID_SHIFT) as u16;
            let serial = (val & SERIAL_MASK) >> SERIAL_SHIFT;

            Some(Identifier {
                vendor_id,
                category,
                product_id,
                serial,
            })
        }
    }

    fn get_root_data(&'a self) -> Option<RootData<'a>> {
        let (vendor_id, vendor_name) = detect_desc_text(&self.root, KeyType::Vendor)?;
        let (product_id, product_name) = detect_desc_text(&self.root, KeyType::Model)?;
        let data = RootData {
            vendor_id,
            vendor_name,
            product_id,
            product_name,
        };
        Some(data)
    }

    fn get_unit_data(&'a self) -> Vec<UnitData<'a>> {
        self.root
            .iter()
            .filter_map(|entry| {
                let entries = EntryDataAccess::<&[Entry]>::get(entry, KeyType::Unit)?;
                let specifier_id = entries
                    .iter()
                    .find_map(|entry| EntryDataAccess::<u32>::get(entry, KeyType::SpecifierId))?;
                let version = entries
                    .iter()
                    .find_map(|entry| EntryDataAccess::<u32>::get(entry, KeyType::Version))?;
                let (model_id, model_name) = detect_desc_text(entries, KeyType::Model)?;
                let data = UnitData {
                    model_id,
                    model_name,
                    specifier_id,
                    version,
                };
                Some(data)
            })
            .collect()
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
