// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Stream format section in protocol extension defined by TCAT for ASICs of DICE.
//!
//! The module includes structure, enumeration, and trait and its implementation for stream format
//! section in protocol extension defined by TCAT for ASICs of DICE.

use super::{caps_section::*, stream_format_entry::*, *};

/// The structure for protocol implementation of stream format section.
#[derive(Default)]
pub struct StreamFormatSectionProtocol;

impl StreamFormatSectionProtocol {
    pub fn read_stream_format_entries(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        caps: &ExtensionCaps,
        timeout_ms: u32,
    ) -> Result<(Vec<FormatEntry>, Vec<FormatEntry>), Error> {
        read_stream_format_entries(req, node, caps, sections.stream_format.offset, timeout_ms)
            .map_err(|e| Error::new(ProtocolExtensionError::StreamFormat, &e.to_string()))
    }

    pub fn write_stream_format_entries(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        caps: &ExtensionCaps,
        pair: &(Vec<FormatEntryData>, Vec<FormatEntryData>),
        timeout_ms: u32,
    ) -> Result<(), Error> {
        write_stream_format_entries(
            req,
            node,
            caps,
            sections.stream_format.offset,
            pair,
            timeout_ms,
        )
        .map_err(|e| Error::new(ProtocolExtensionError::StreamFormat, &e.to_string()))
    }
}
