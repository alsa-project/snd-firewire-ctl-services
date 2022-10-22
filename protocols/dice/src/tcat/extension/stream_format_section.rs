// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Stream format section in protocol extension defined by TCAT for ASICs of DICE.
//!
//! The module includes structure, enumeration, and trait and its implementation for stream format
//! section in protocol extension defined by TCAT for ASICs of DICE.

use super::{caps_section::*, stream_format_entry::*, *};

/// Protocol implementation of stream format section.
#[derive(Default)]
pub struct StreamFormatSectionProtocol;

impl StreamFormatSectionProtocol {
    /// Read from stream format section and deserialize entries.
    pub fn read_stream_format_whole_entries(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        caps: &ExtensionCaps,
        (tx_entries, rx_entries): (&mut Vec<FormatEntry>, &mut Vec<FormatEntry>),
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let size = calculate_stream_format_entries_size(
            caps.general.max_tx_streams as usize,
            caps.general.max_rx_streams as usize,
        );
        let size = std::cmp::min(sections.stream_format.size, size);
        let mut raw = vec![0u8; size];
        extension_read(
            req,
            node,
            sections.stream_format.offset,
            &mut raw,
            timeout_ms,
        )
        .map_err(|e| Error::new(ProtocolExtensionError::StreamFormat, &e.to_string()))?;

        let mut val = 0u32;
        val.parse_quadlet(&raw[..4]);
        let tx_entry_count = val as usize;

        val.parse_quadlet(&raw[4..8]);
        let rx_entry_count = val as usize;

        tx_entries.resize_with(tx_entry_count, Default::default);
        rx_entries.resize_with(rx_entry_count, Default::default);
        deserialize_stream_format_entries((tx_entries, rx_entries), &raw)
            .map_err(|e| Error::new(ProtocolExtensionError::StreamFormat, &e.to_string()))
    }

    /// Serialize entries and write to stream format section.
    pub fn write_stream_format_whole_entries(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        caps: &ExtensionCaps,
        (tx_entries, rx_entries): (&[FormatEntry], &[FormatEntry]),
        timeout_ms: u32,
    ) -> Result<(), Error> {
        if caps.general.dynamic_stream_format {
            let msg = "Stream format configuration is not mutable.";
            Err(Error::new(ProtocolExtensionError::StreamFormat, &msg))?;
        }

        let size = calculate_stream_format_entries_size(tx_entries.len(), rx_entries.len());
        let size = std::cmp::min(sections.stream_format.size, size);
        let mut raw = vec![0u8; size];
        serialize_stream_format_entries((tx_entries, rx_entries), &mut raw)
            .map_err(|e| Error::new(ProtocolExtensionError::StreamFormat, &e.to_string()))?;

        extension_write(
            req,
            node,
            sections.stream_format.offset,
            &mut raw,
            timeout_ms,
        )
        .map_err(|e| Error::new(ProtocolExtensionError::StreamFormat, &e.to_string()))
    }
}
