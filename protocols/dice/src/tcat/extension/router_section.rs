// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Router section in protocol extension defined by TCAT for ASICs of DICE.
//!
//! The module includes structure, enumeration, and trait and its implementation for router section
//! in protocol extension defined by TCAT for ASICs of DICE.

use super::{caps_section::*, router_entry::*, *};

/// Protocol implementation of router section.
#[derive(Default)]
pub struct RouterSectionProtocol;

impl RouterSectionProtocol {
    /// Read from router section and deserialize entries.
    pub fn read_router_whole_entries(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        caps: &ExtensionCaps,
        entries: &mut Vec<RouterEntry>,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let size = 4 + calculate_router_entries_size(caps.router.maximum_entry_count as usize);
        let size = std::cmp::min(sections.router.size, size);
        let mut raw = vec![0u8; size];

        extension_read(req, node, sections.router.offset, &mut raw, timeout_ms)
            .map_err(|e| Error::new(ProtocolExtensionError::Router, &e.to_string()))?;

        let mut val = 0u32;
        val.parse_quadlet(&raw[..4]);
        entries.resize_with(val as usize, Default::default);

        deserialize_router_entries(entries, &mut raw[4..])
            .map_err(|cause| Error::new(ProtocolExtensionError::Router, &cause))
    }

    /// Serialize entries and write to router section.
    pub fn write_router_whole_entries(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        caps: &ExtensionCaps,
        entries: &[RouterEntry],
        timeout_ms: u32,
    ) -> Result<(), Error> {
        if entries.len() >= caps.router.maximum_entry_count as usize {
            let msg = format!(
                "The number of router entries should be less than{}, but {} given",
                caps.router.maximum_entry_count,
                entries.len(),
            );
            Err(Error::new(ProtocolExtensionError::Router, &msg))?;
        }

        let size = 4 + calculate_router_entries_size(entries.len() as usize);
        let mut raw = vec![0u8; size];

        (entries.len() as u32).build_quadlet(&mut raw[..4]);
        serialize_router_entries(entries, &mut raw[4..])
            .map_err(|cause| Error::new(ProtocolExtensionError::Router, &cause))?;

        extension_write(req, node, sections.router.offset, &mut raw, timeout_ms)
    }

    pub fn read_router_entries(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        caps: &ExtensionCaps,
        timeout_ms: u32,
    ) -> Result<Vec<RouterEntry>, Error> {
        let mut data = [0; 4];
        extension_read(req, node, sections.router.offset, &mut data, timeout_ms)
            .map_err(|e| Error::new(ProtocolExtensionError::Router, &e.to_string()))?;

        let entry_count = std::cmp::min(
            u32::from_be_bytes(data) as usize,
            caps.router.maximum_entry_count as usize,
        );
        let mut entries = vec![RouterEntry::default(); entry_count];
        read_router_entries(
            req,
            node,
            caps,
            sections.router.offset + 4,
            &mut entries,
            timeout_ms,
        )
        .map_err(|e| Error::new(ProtocolExtensionError::Router, &e.to_string()))
        .map(|_| entries)
    }

    pub fn write_router_entries(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        caps: &ExtensionCaps,
        entries: &[RouterEntry],
        timeout_ms: u32,
    ) -> Result<(), Error> {
        write_router_entries(req, node, caps, sections.router.offset, entries, timeout_ms)
            .map_err(|e| Error::new(ProtocolExtensionError::Router, &e.to_string()))
    }
}
