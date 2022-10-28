// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Stream format section in protocol extension defined by TCAT for ASICs of DICE.
//!
//! The module includes structure, enumeration, and trait and its implementation for stream format
//! section in protocol extension defined by TCAT for ASICs of DICE.

use super::{stream_format_entry::*, *};

/// Parameters of entries in stream format section.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct StreamFormatParams {
    pub tx_entries: Vec<FormatEntry>,
    pub rx_entries: Vec<FormatEntry>,
}

impl<O: TcatExtensionOperation> TcatExtensionSectionParamsOperation<StreamFormatParams> for O {
    fn cache_extension_whole_params(
        req: &FwReq,
        node: &FwNode,
        sections: &ExtensionSections,
        caps: &ExtensionCaps,
        params: &mut StreamFormatParams,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let size = calculate_stream_format_entries_size(
            caps.general.max_tx_streams as usize,
            caps.general.max_rx_streams as usize,
        );
        let size = std::cmp::min(sections.stream_format.size, size);
        let mut raw = vec![0u8; size];
        Self::read_extension(req, node, &sections.stream_format, 0, &mut raw, timeout_ms)?;

        let mut tx_entry_count = 0usize;
        deserialize_usize(&mut tx_entry_count, &raw[..4]);

        let mut rx_entry_count = 0usize;
        deserialize_usize(&mut rx_entry_count, &raw[4..8]);

        params
            .tx_entries
            .resize_with(tx_entry_count, Default::default);
        params
            .rx_entries
            .resize_with(rx_entry_count, Default::default);
        deserialize_stream_format_entries((&mut params.tx_entries, &mut params.rx_entries), &raw)
            .map_err(|e| Error::new(ProtocolExtensionError::StreamFormat, &e.to_string()))
    }
}

impl<O: TcatExtensionOperation> TcatExtensionSectionWholeMutableParamsOperation<StreamFormatParams>
    for O
{
    fn update_extension_whole_params(
        req: &FwReq,
        node: &FwNode,
        sections: &ExtensionSections,
        caps: &ExtensionCaps,
        params: &StreamFormatParams,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        if caps.general.dynamic_stream_format {
            let msg = "Stream format configuration is not mutable.";
            Err(Error::new(ProtocolExtensionError::StreamFormat, &msg))?;
        }

        let size =
            calculate_stream_format_entries_size(params.tx_entries.len(), params.rx_entries.len());
        let size = std::cmp::min(sections.stream_format.size, size);
        let mut raw = vec![0u8; size];
        serialize_stream_format_entries((&params.tx_entries, &params.rx_entries), &mut raw)
            .map_err(|e| Error::new(ProtocolExtensionError::StreamFormat, &e.to_string()))?;

        Self::write_extension(req, node, &sections.stream_format, 0, &mut raw, timeout_ms)
            .map_err(|e| Error::new(ProtocolExtensionError::StreamFormat, &e.to_string()))
    }
}
