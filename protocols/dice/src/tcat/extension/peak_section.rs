// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Peak section in protocol extension defined by TCAT for ASICs of DICE.
//!
//! The module includes structure, enumeration, and trait and its implementation for peak section
//! in protocol extension defined by TCAT for ASICs of DICE.
use super::{caps_section::*, router_entry::*, *};

/// Protocol implementation of peak section.
#[derive(Default)]
pub struct PeakSectionProtocol;

impl PeakSectionProtocol {
    /// Cache state of hardware for peak entries.
    pub fn cache_peak_whole_entries(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        caps: &ExtensionCaps,
        entries: &mut Vec<RouterEntry>,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        if !caps.general.peak_avail {
            Err(Error::new(
                ProtocolExtensionError::Peak,
                "Peak is not available",
            ))?
        }

        entries.resize_with(caps.router.maximum_entry_count as usize, Default::default);
        let size = calculate_router_entries_size(entries.len());
        let mut raw = vec![0u8; size];

        extension_read(req, node, sections.peak.offset, &mut raw, timeout_ms)?;

        deserialize_router_entries(entries, &raw)
            .map_err(|cause| Error::new(ProtocolExtensionError::Peak, &cause))
    }

    pub fn read_peak_entries(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        caps: &ExtensionCaps,
        timeout_ms: u32,
    ) -> Result<Vec<RouterEntry>, Error> {
        if !caps.general.peak_avail {
            Err(Error::new(
                ProtocolExtensionError::Peak,
                "Peak is not available",
            ))?
        }

        let entry_count = caps.router.maximum_entry_count as usize;
        let mut entries = vec![RouterEntry::default(); entry_count];
        read_router_entries(
            req,
            node,
            caps,
            sections.peak.offset,
            &mut entries,
            timeout_ms,
        )
        .map_err(|e| Error::new(ProtocolExtensionError::Peak, &e.to_string()))
        .map(|_| entries)
    }
}
