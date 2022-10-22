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
