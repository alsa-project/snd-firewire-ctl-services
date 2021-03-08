// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol defined by RME GmbH for Fireface series.
//!
//! The crate includes various kind of protocols defined by RME GmbH for models of its Fireface
//! series. The protocols are categorized by two generations; i.e. former and latter.

pub mod former;

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

/// The enumeration to represent nominal frequency of sampling clock.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ClkNominalRate {
    R32000,
    R44100,
    R48000,
    R64000,
    R88200,
    R96000,
    R128000,
    R176400,
    R192000,
}

impl Default for ClkNominalRate {
    fn default() -> Self {
        Self::R44100
    }
}

/// The enumeration to represent format of S/PDIF signal.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SpdifFormat {
    Consumer,
    Professional,
}

impl Default for SpdifFormat {
    fn default() -> Self {
        Self::Consumer
    }
}

/// The enumeration to represent interface of S/PDIF signal.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SpdifIface {
    Coaxial,
    Optical,
}

impl Default for SpdifIface {
    fn default() -> Self {
        Self::Coaxial
    }
}

/// The structure to represent configuration of S/PDIF input.
#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
pub struct SpdifInput {
    /// The interface of S/PDIF signal.
    pub iface: SpdifIface,
    /// Whether to deliver preamble information by corresponding audio data channel of tx stream.
    pub use_preemble: bool,
}

/// The enumeration to represent the type of signal to optical output interface.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum OpticalOutputSignal {
    Adat,
    Spdif,
}

impl Default for OpticalOutputSignal {
    fn default() -> Self {
        Self::Adat
    }
}

/// The enumeration to represent nominal level of line outputs.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum LineOutNominalLevel {
    High,
    /// -10 dBV.
    Consumer,
    /// +4 dBu.
    Professional,
}

impl Default for LineOutNominalLevel {
    fn default() -> Self {
        Self::High
    }
}
