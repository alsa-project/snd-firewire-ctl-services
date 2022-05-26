// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Parser of configuration ROM for Tascam FireWire series.
//!
//! In Tascam FireWire series, configration ROM does not follow to the standard defined by
//! 1394 Trading Association. This module includes the parse of unit directory in the
//! configuration ROM.

use {
    super::*,
    ieee1212_config_rom::{entry::*, *},
};

/// The structure for data in unit directory of configuration ROM.
#[derive(Default, Debug, Clone, Copy)]
pub struct UnitData<'a> {
    pub specifier_id: u32,
    pub version: u32,
    pub vendor_name: &'a str,
    pub model_name: &'a str,
}

/// The trait for parser of configuration ROM.
pub trait TascamConfigRom<'a> {
    fn get_unit_data(&'a self) -> Result<UnitData<'a>, Error>;
}

impl<'a> TascamConfigRom<'a> for ConfigRom<'a> {
    fn get_unit_data(&'a self) -> Result<UnitData<'a>, Error> {
        let unit_directory = self
            .root
            .iter()
            .find_map(|entry| EntryDataAccess::<&[Entry]>::get(entry, KeyType::Unit))
            .ok_or(Error::new(FileError::Nxio, "Unit directory is missing"))?;

        let &specifier_id = unit_directory
            .iter()
            .find_map(|entry| EntryDataAccess::<&u32>::get(entry, KeyType::SpecifierId))
            .ok_or(Error::new(FileError::Nxio, "Specifier ID is missing"))?;

        let &version = unit_directory
            .iter()
            .find_map(|entry| EntryDataAccess::<&u32>::get(entry, KeyType::Version))
            .ok_or(Error::new(FileError::Nxio, "Version is missing"))?;

        let dependent_info_directory = unit_directory
            .iter()
            .find_map(|entry| EntryDataAccess::<&[Entry]>::get(entry, KeyType::DependentInfo))
            .ok_or(Error::new(
                FileError::Nxio,
                "Dependent information directory is missing",
            ))?;

        let vendor_name = dependent_info_directory
            .iter()
            .find_map(|entry| EntryDataAccess::<&str>::get(entry, KeyType::Descriptor))
            .ok_or(Error::new(FileError::Nxio, "Vendor name is missing"))?;

        let model_name = dependent_info_directory
            .iter()
            .find_map(|entry| EntryDataAccess::<&str>::get(entry, KeyType::BusDependentInfo))
            .ok_or(Error::new(FileError::Nxio, "Model name is missing"))?;

        Ok(UnitData {
            specifier_id,
            version,
            vendor_name,
            model_name,
        })
    }
}
