// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Router section in protocol extension defined by TCAT for ASICs of DICE.
//!
//! The module includes structure, enumeration, and trait and its implementation for router section
//! in protocol extension defined by TCAT for ASICs of DICE.

use super::{router_entry::*, *};

/// Parameter of entries in router section.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct RouterParams(pub Vec<RouterEntry>);

impl<O: TcatExtensionOperation> TcatExtensionSectionParamsOperation<RouterParams> for O {
    fn cache_extension_whole_params(
        req: &FwReq,
        node: &FwNode,
        sections: &ExtensionSections,
        caps: &ExtensionCaps,
        params: &mut RouterParams,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let size = 4 + calculate_router_entries_size(caps.router.maximum_entry_count as usize);
        let size = std::cmp::min(sections.router.size, size);
        let mut raw = vec![0u8; size];

        Self::read_extension(req, node, &sections.router, 0, &mut raw, timeout_ms)?;

        let mut val = 0u32;
        deserialize_u32(&mut val, &raw[..4]);
        params.0.resize_with(val as usize, Default::default);

        deserialize_router_entries(&mut params.0, &mut raw[4..])
            .map_err(|cause| Error::new(ProtocolExtensionError::Router, &cause))
    }
}

impl<O: TcatExtensionOperation> TcatExtensionSectionWholeMutableParamsOperation<RouterParams>
    for O
{
    fn update_extension_whole_params(
        req: &FwReq,
        node: &FwNode,
        sections: &ExtensionSections,
        caps: &ExtensionCaps,
        params: &RouterParams,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        if params.0.len() >= caps.router.maximum_entry_count as usize {
            let msg = format!(
                "The number of router entries should be less than{}, but {} given",
                caps.router.maximum_entry_count,
                params.0.len(),
            );
            Err(Error::new(ProtocolExtensionError::Router, &msg))?;
        }

        let size = 4 + calculate_router_entries_size(params.0.len() as usize);
        let mut raw = vec![0u8; size];

        serialize_u32(&(params.0.len() as u32), &mut raw[..4]);
        serialize_router_entries(&params.0, &mut raw[4..])
            .map_err(|cause| Error::new(ProtocolExtensionError::Router, &cause))?;

        Self::write_extension(req, node, &sections.router, 0, &mut raw, timeout_ms)
    }
}
