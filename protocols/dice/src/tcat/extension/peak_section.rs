// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Peak section in protocol extension defined by TCAT for ASICs of DICE.
//!
//! The module includes structure, enumeration, and trait and its implementation for peak section
//! in protocol extension defined by TCAT for ASICs of DICE.
use super::{caps_section::*, router_entry::*, *};

/// Parameters of meter detections in peak section.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct PeakParams(pub RouterParams);

impl<O: TcatExtensionOperation> TcatExtensionSectionParamsOperation<PeakParams> for O {
    fn cache_extension_whole_params(
        req: &FwReq,
        node: &FwNode,
        sections: &ExtensionSections,
        caps: &ExtensionCaps,
        params: &mut PeakParams,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        if !caps.general.peak_avail {
            Err(Error::new(
                ProtocolExtensionError::Peak,
                "Peak is not available",
            ))?
        }

        params
            .0
             .0
            .resize_with(caps.router.maximum_entry_count as usize, Default::default);
        let size = calculate_router_entries_size(params.0 .0.len());
        let mut raw = vec![0u8; size];

        Self::read_extension(req, node, &sections.peak, 0, &mut raw, timeout_ms)?;

        deserialize_router_entries(&mut params.0 .0, &raw)
            .map_err(|cause| Error::new(ProtocolExtensionError::Peak, &cause))
    }
}
