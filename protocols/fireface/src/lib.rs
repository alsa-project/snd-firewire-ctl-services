// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

#![doc = include_str!("../README.md")]

pub mod former;
pub mod latter;

use {
    glib::Error,
    hinawa::{prelude::FwReqExtManual, FwNode, FwReq, FwTcode},
    ieee1212_config_rom::*,
};

const RME_OUI: u32 = 0x00000a35;

/// Parser of configuration rom for RME Fireface series.
pub trait FfConfigRom {
    fn get_model_id(&self) -> Option<u32>;
}

impl<'a> FfConfigRom for ConfigRom<'a> {
    fn get_model_id(&self) -> Option<u32> {
        self.root
            .iter()
            .find_map(|entry| EntryDataAccess::<u32>::get(entry, KeyType::Vendor))
            .filter(|vendor_id| vendor_id.eq(&RME_OUI))
            .and_then(|_| {
                self.root
                    .iter()
                    .find_map(|entry| EntryDataAccess::<&[Entry]>::get(entry, KeyType::Unit))
                    .and_then(|entries| {
                        entries
                            .iter()
                            .find_map(|entry| EntryDataAccess::<u32>::get(entry, KeyType::Version))
                    })
            })
    }
}

/// Nominal frequency of sampling clock.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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

/// Format of S/PDIF signal.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SpdifFormat {
    Consumer,
    Professional,
}

impl Default for SpdifFormat {
    fn default() -> Self {
        Self::Consumer
    }
}

/// Digital interface of S/PDIF signal.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SpdifIface {
    Coaxial,
    Optical,
}

impl Default for SpdifIface {
    fn default() -> Self {
        Self::Coaxial
    }
}

/// Configuration of S/PDIF input.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct SpdifInput {
    /// The interface of S/PDIF signal.
    pub iface: SpdifIface,
    /// Whether to deliver preamble information by corresponding audio data channel of tx stream.
    pub use_preemble: bool,
}

/// Type of signal to optical output interface.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum OpticalOutputSignal {
    Adat,
    Spdif,
}

impl Default for OpticalOutputSignal {
    fn default() -> Self {
        Self::Adat
    }
}

/// Nominal level of line outputs.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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

/// Serialize offsets for parameters.
pub trait RmeFfOffsetParamsSerialize<T> {
    /// Serialize parameters from raw data.
    fn serialize_offsets(params: &T) -> Vec<u8>;
}

/// Deserialize offsets for parameters.
pub trait RmeFfOffsetParamsDeserialize<T> {
    /// Deserialize parameters into raw data.
    fn deserialize_offsets(params: &mut T, raw: &[u8]);
}

/// Operation for parameters which can be updated wholly at once.
pub trait RmeFfWhollyUpdatableParamsOperation<T> {
    /// Update registers for whole parameters.
    fn update_wholly(
        req: &mut FwReq,
        node: &mut FwNode,
        params: &T,
        timeout_ms: u32,
    ) -> Result<(), Error>;
}

/// Operation for parameters which can be updated partially.
pub trait RmeFfPartiallyUpdatableParamsOperation<T> {
    fn update_partially(
        req: &mut FwReq,
        node: &mut FwNode,
        params: &mut T,
        update: T,
        timeout_ms: u32,
    ) -> Result<(), Error>;
}

/// Operation for parameters which can be cached wholly at once.
pub trait RmeFfCacheableParamsOperation<T> {
    /// Cache whole parameters from registers.
    fn cache_wholly(
        req: &mut FwReq,
        node: &mut FwNode,
        params: &mut T,
        timeout_ms: u32,
    ) -> Result<(), Error>;
}
