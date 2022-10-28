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

/// Parameters of router entries in current configuration section.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct CurrentRouterParams {
    pub entries: RouterParams,
    pub rate_mode: RateMode,
}

/// Parameters of stream format entries in current configuration section.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct CurrentStreamFormatParams {
    pub pair: StreamFormatParams,
    pub rate_mode: RateMode,
}

const LOW_ROUTER_CONFIG_OFFSET: usize = 0x0000;
const LOW_STREAM_CONFIG_OFFSET: usize = 0x1000;
const MID_ROUTER_CONFIG_OFFSET: usize = 0x2000;
const MID_STREAM_CONFIG_OFFSET: usize = 0x3000;
const HIGH_ROUTER_CONFIG_OFFSET: usize = 0x4000;
const HIGH_STREAM_CONFIG_OFFSET: usize = 0x5000;

impl<O: TcatExtensionOperation> TcatExtensionSectionParamsOperation<CurrentRouterParams> for O {
    fn cache_extension_whole_params(
        req: &FwReq,
        node: &FwNode,
        sections: &ExtensionSections,
        caps: &ExtensionCaps,
        params: &mut CurrentRouterParams,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        if !caps.router.is_exposed {
            let msg = "Router configuration is not exposed.";
            Err(Error::new(ProtocolExtensionError::CurrentConfig, &msg))?;
        }

        let offset = match params.rate_mode {
            RateMode::Low => LOW_ROUTER_CONFIG_OFFSET,
            RateMode::Middle => MID_ROUTER_CONFIG_OFFSET,
            RateMode::High => HIGH_ROUTER_CONFIG_OFFSET,
        };

        let mut raw = vec![0u8; 4];
        Self::read_extension(
            req,
            node,
            &sections.current_config,
            offset,
            &mut raw,
            timeout_ms,
        )?;

        let mut val = 0u32;
        deserialize_u32(&mut val, &raw[..4]);

        let entry_count = std::cmp::min(val as usize, caps.router.maximum_entry_count as usize);

        params.entries.0.resize_with(entry_count, Default::default);
        raw.resize_with(
            4 + calculate_router_entries_size(entry_count),
            Default::default,
        );

        Self::read_extension(
            req,
            node,
            &sections.current_config,
            offset + 4,
            &mut raw[4..],
            timeout_ms,
        )?;

        deserialize_router_entries(&mut params.entries.0, &raw[4..])
            .map_err(|cause| Error::new(ProtocolExtensionError::CurrentConfig, &cause))
    }
}

impl<O: TcatExtensionOperation> TcatExtensionSectionParamsOperation<CurrentStreamFormatParams> for O {
    fn cache_extension_whole_params(
        req: &FwReq,
        node: &FwNode,
        sections: &ExtensionSections,
        caps: &ExtensionCaps,
        params: &mut CurrentStreamFormatParams,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let offset = match params.rate_mode {
            RateMode::Low => LOW_STREAM_CONFIG_OFFSET,
            RateMode::Middle => MID_STREAM_CONFIG_OFFSET,
            RateMode::High => HIGH_STREAM_CONFIG_OFFSET,
        };

        let size = calculate_stream_format_entries_size(
            caps.general.max_tx_streams as usize,
            caps.general.max_rx_streams as usize,
        );
        let mut raw = vec![0u8; size];
        Self::read_extension(
            req,
            node,
            &sections.current_config,
            offset,
            &mut raw,
            timeout_ms,
        )?;

        let mut val = 0u32;
        deserialize_u32(&mut val, &raw[..4]);
        let tx_entries_count = val as usize;
        params
            .pair
            .tx_entries
            .resize_with(tx_entries_count, Default::default);

        deserialize_u32(&mut val, &raw[4..8]);
        let rx_entries_count = val as usize;
        params
            .pair
            .rx_entries
            .resize_with(rx_entries_count, Default::default);

        deserialize_stream_format_entries(
            (&mut params.pair.tx_entries, &mut params.pair.rx_entries),
            &raw,
        )
        .map_err(|cause| Error::new(ProtocolExtensionError::CurrentConfig, &cause))
    }
}
