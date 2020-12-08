// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Peak section in protocol extension defined by TCAT for ASICs of DICE.
//!
//! The module includes structure, enumeration, and trait and its implementation for peak section
//! in protocol extension defined by TCAT for ASICs of DICE.
use super::{*, caps_section::*, router_entry::*};

pub trait PeakSectionProtocol<T> : ProtocolExtension<T>
    where T: AsRef<FwNode>,
{
    fn read_peak_entries(&self, node: &T, sections: &ExtensionSections, caps: &ExtensionCaps,
                         timeout_ms: u32)
        -> Result<Vec<RouterEntryData>, Error>
    {
        if !caps.general.peak_avail {
            Err(Error::new(ProtocolExtensionError::Peak, "Peak is not available"))?
        }

        let entries = caps.router.maximum_entry_count as usize;
        RouterEntryProtocol::read_router_entries(&self, node, caps, sections.peak.offset, entries,
                                                 timeout_ms)
            .map_err(|e| Error::new(ProtocolExtensionError::Peak, &e.to_string()))
    }
}

impl<O: AsRef<FwReq>, T: AsRef<FwNode>> PeakSectionProtocol<T> for O {}
