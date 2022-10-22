// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Current configuration section in protocol extension defined by TCAT for ASICs of DICE.
//!
//! The module includes structure, enumeration, and trait and its implementation for current
//! configuration section in protocol extension defined by TCAT for ASICs of DICE.

use super::{
    router_entry::*,
    stream_format_entry::*,
    {caps_section::*, cmd_section::*, *},
};

/// Protocol implementation of current configuration section.
#[derive(Default)]
pub struct CurrentConfigSectionProtocol;

impl CurrentConfigSectionProtocol {
    const LOW_ROUTER_CONFIG_OFFSET: usize = 0x0000;
    const LOW_STREAM_CONFIG_OFFSET: usize = 0x1000;
    const MID_ROUTER_CONFIG_OFFSET: usize = 0x2000;
    const MID_STREAM_CONFIG_OFFSET: usize = 0x3000;
    const HIGH_ROUTER_CONFIG_OFFSET: usize = 0x4000;
    const HIGH_STREAM_CONFIG_OFFSET: usize = 0x5000;

    /// Cache state of hardware for current router entries.
    pub fn cache_current_config_router_entries(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        caps: &ExtensionCaps,
        mode: RateMode,
        entries: &mut Vec<RouterEntry>,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        if !caps.router.is_exposed {
            let msg = "Router configuration is not exposed.";
            Err(Error::new(ProtocolExtensionError::CurrentConfig, &msg))?;
        }

        let offset = match mode {
            RateMode::Low => Self::LOW_ROUTER_CONFIG_OFFSET,
            RateMode::Middle => Self::MID_ROUTER_CONFIG_OFFSET,
            RateMode::High => Self::HIGH_ROUTER_CONFIG_OFFSET,
        };

        let mut raw = vec![0u8; 4];
        extension_read(
            req,
            node,
            sections.current_config.offset + offset,
            &mut raw,
            timeout_ms,
        )
        .map_err(|e| Error::new(ProtocolExtensionError::CurrentConfig, &e.to_string()))?;

        let mut val = 0u32;
        val.parse_quadlet(&raw);

        let entry_count = std::cmp::min(val as usize, caps.router.maximum_entry_count as usize);

        entries.resize_with(entry_count, Default::default);
        raw.resize_with(
            4 + calculate_router_entries_size(entry_count),
            Default::default,
        );

        extension_read(
            req,
            node,
            sections.current_config.offset + offset + 4,
            &mut raw[4..],
            timeout_ms,
        )?;

        deserialize_router_entries(entries, &raw[4..])
            .map_err(|cause| Error::new(ProtocolExtensionError::CurrentConfig, &cause))
    }

    /// Cache state of hardware for current stream format entries.
    pub fn cache_current_config_stream_format_entries(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        caps: &ExtensionCaps,
        mode: RateMode,
        (tx_entries, rx_entries): (&mut Vec<FormatEntry>, &mut Vec<FormatEntry>),
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let offset = match mode {
            RateMode::Low => Self::LOW_STREAM_CONFIG_OFFSET,
            RateMode::Middle => Self::MID_STREAM_CONFIG_OFFSET,
            RateMode::High => Self::HIGH_STREAM_CONFIG_OFFSET,
        };

        let size = calculate_stream_format_entries_size(
            caps.general.max_tx_streams as usize,
            caps.general.max_rx_streams as usize,
        );
        let mut raw = vec![0u8; size];
        extension_read(
            req,
            node,
            sections.current_config.offset + offset,
            &mut raw,
            timeout_ms,
        )?;

        let mut val = 0u32;
        val.parse_quadlet(&raw[..4]);
        let tx_entries_count = val as usize;
        tx_entries.resize_with(tx_entries_count, Default::default);

        val.parse_quadlet(&raw[4..8]);
        let rx_entries_count = val as usize;
        rx_entries.resize_with(rx_entries_count, Default::default);

        deserialize_stream_format_entries((tx_entries, rx_entries), &raw)
            .map_err(|cause| Error::new(ProtocolExtensionError::CurrentConfig, &cause))
    }

    pub fn read_current_router_entries(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        caps: &ExtensionCaps,
        mode: RateMode,
        timeout_ms: u32,
    ) -> Result<Vec<RouterEntry>, Error> {
        let offset = match mode {
            RateMode::Low => Self::LOW_ROUTER_CONFIG_OFFSET,
            RateMode::Middle => Self::MID_ROUTER_CONFIG_OFFSET,
            RateMode::High => Self::HIGH_ROUTER_CONFIG_OFFSET,
        };

        let mut data = [0; 4];
        let offset = sections.current_config.offset + offset;
        extension_read(req, node, offset, &mut data, timeout_ms)
            .map_err(|e| Error::new(ProtocolExtensionError::CurrentConfig, &e.to_string()))?;

        let entry_count = std::cmp::min(
            u32::from_be_bytes(data) as usize,
            caps.router.maximum_entry_count as usize,
        );

        let mut entries = vec![RouterEntry::default(); entry_count];
        read_router_entries(req, node, caps, offset + 4, &mut entries, timeout_ms)
            .map_err(|e| Error::new(ProtocolExtensionError::CurrentConfig, &e.to_string()))
            .map(|_| entries)
    }

    pub fn read_current_stream_format_entries(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        caps: &ExtensionCaps,
        mode: RateMode,
        timeout_ms: u32,
    ) -> Result<(Vec<FormatEntry>, Vec<FormatEntry>), Error> {
        let offset = match mode {
            RateMode::Low => Self::LOW_STREAM_CONFIG_OFFSET,
            RateMode::Middle => Self::MID_STREAM_CONFIG_OFFSET,
            RateMode::High => Self::HIGH_STREAM_CONFIG_OFFSET,
        };
        let offset = sections.current_config.offset + offset;
        read_stream_format_entries(req, node, caps, offset, timeout_ms)
            .map_err(|e| Error::new(ProtocolExtensionError::CurrentConfig, &e.to_string()))
    }
}
