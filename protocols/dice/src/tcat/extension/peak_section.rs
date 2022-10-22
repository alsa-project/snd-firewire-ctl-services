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
