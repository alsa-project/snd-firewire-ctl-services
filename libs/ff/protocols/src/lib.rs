// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol defined by RME GmbH for Fireface series.
//!
//! The crate includes various kind of protocols defined by RME GmbH for models of its Fireface
//! series. The protocols are categorized by two generations; i.e. former and latter.

use ieee1212_config_rom::{*, entry::*};

const RME_OUI: u32 = 0x00000a35;

/// The trait to represent parser of configuration rom for RME Fireface series.
pub trait FfConfigRom {
    fn get_model_id(&self) -> Option<u32>;
}

impl<'a> FfConfigRom for ConfigRom<'a> {
    fn get_model_id(&self) -> Option<u32> {
        self.root.iter().find_map(|entry| {
            EntryDataAccess::<u32>::get(entry, KeyType::Vendor)
        })
        .filter(|vendor_id| vendor_id.eq(&RME_OUI))
        .and_then(|_| {
            self.root.iter().find_map(|entry| {
                EntryDataAccess::<&[Entry]>::get(entry, KeyType::Unit)
            })
            .and_then(|entries| {
                entries.iter().find_map(|entry| {
                    EntryDataAccess::<u32>::get(entry, KeyType::Version)
                })
            })
        })
    }
}
