// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Stream format section in protocol extension defined by TCAT for ASICs of DICE.
//!
//! The module includes structure, enumeration, and trait and its implementation for stream format
//! section in protocol extension defined by TCAT for ASICs of DICE.

use super::{*, caps_section::*, stream_format_entry::*};

pub trait StreamFormatSectionProtocol<T> : ProtocolExtension<T>
    where T: AsRef<FwNode>,
{
    fn read_stream_format_entries(&self, node: &T, sections: &ExtensionSections, caps: &ExtensionCaps,
                                  timeout_ms: u32)
        -> Result<(Vec<FormatEntry>, Vec<FormatEntry>), Error>
    {
        StreamFormatEntryProtocol::read_stream_format_entries(&self, node, caps, sections.stream_format.offset,
                                                              timeout_ms)
            .map_err(|e| Error::new(ProtocolExtensionError::StreamFormat, &e.to_string()))
    }

    fn write_stream_format_entries(&self, node: &T, sections: &ExtensionSections, caps: &ExtensionCaps,
                                   pair: &(Vec<FormatEntryData>, Vec<FormatEntryData>), timeout_ms: u32)
        -> Result<(), Error>
    {
        StreamFormatEntryProtocol::write_stream_format_entries(&self, node, caps, sections.stream_format.offset,
                                                               pair, timeout_ms)
            .map_err(|e| Error::new(ProtocolExtensionError::StreamFormat, &e.to_string()))
    }
}

impl<O: AsRef<FwReq>, T: AsRef<FwNode>> StreamFormatSectionProtocol<T> for O {}
