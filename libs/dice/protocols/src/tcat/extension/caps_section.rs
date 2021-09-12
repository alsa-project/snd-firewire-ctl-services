// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Capabilities section in protocol extension defined by TCAT for ASICs of DICE.
//!
//! The module includes structure, enumeration, and trait and its implementation for capabilities
//! section in protocol extension defined by TCAT for ASICs of DICE.
use super::*;

/// The structure to represent capability of router functionality.
#[derive(Default, Clone, Copy, Debug, Eq, PartialEq)]
pub struct RouterCaps {
    pub is_exposed: bool,
    pub is_readonly: bool,
    pub is_storable: bool,
    pub maximum_entry_count: u16,
}

impl RouterCaps {
    const SIZE: usize = 4;

    const IS_EXPOSED_OFFSET: usize = 3;
    const IS_READONLY_OFFSET: usize = 3;
    const IS_STORABLE_OFFSET: usize = 3;

    const IS_EXPOSED_FLAG: u8 = 0x01;
    const IS_READONLY_FLAG: u8 = 0x02;
    const IS_STORABLE_FLAG: u8 = 0x04;
}

impl From<&[u8]> for RouterCaps {
    fn from(raw: &[u8]) -> Self {
        let mut doublet = [0;2];
        doublet.copy_from_slice(&raw[..2]);
        RouterCaps {
            is_exposed: raw[Self::IS_EXPOSED_OFFSET] & Self::IS_EXPOSED_FLAG > 0,
            is_readonly: raw[Self::IS_READONLY_OFFSET] & Self::IS_READONLY_FLAG  > 0,
            is_storable: raw[Self::IS_STORABLE_OFFSET] & Self::IS_STORABLE_FLAG > 0,
            maximum_entry_count: u16::from_be_bytes(doublet),
        }
    }
}

/// The structure to represent capability of mixer functionality.
#[derive(Default, Clone, Copy, Debug, Eq, PartialEq)]
pub struct MixerCaps {
    pub is_exposed: bool,
    pub is_readonly: bool,
    pub is_storable: bool,
    pub input_device_id: u8,
    pub output_device_id: u8,
    pub input_count: u8,
    pub output_count: u8,
}

impl MixerCaps {
    const SIZE: usize = 0x04;

    const IS_EXPOSED_OFFSET: usize = 3;
    const IS_READONLY_OFFSET: usize = 3;
    const IS_STORABLE_OFFSET: usize = 3;
    const INPUT_DEVICE_ID_OFFSET: usize = 3;
    const OUTPUT_DEVICE_ID_OFFSET: usize = 2;
    const INPUT_COUNT_OFFSET: usize = 1;
    const OUTPUT_COUNT_OFFSET: usize = 0;

    const IS_EXPOSED_FLAG: u8 = 0x01;
    const IS_READONLY_FLAG: u8 = 0x02;
    const IS_STORABLE_FLAG: u8 = 0x04;
    const INPUT_DEVICE_ID_MASK: u8 = 0x0f;
    const OUTPUT_DEVICE_ID_MASK: u8 = 0x0f;

    const INPUT_DEVICE_ID_SHIFT: usize = 4;
}

impl From<&[u8]> for MixerCaps {
    fn from(raw: &[u8]) -> Self {
        MixerCaps {
            is_exposed: raw[Self::IS_EXPOSED_OFFSET] & Self::IS_EXPOSED_FLAG > 0,
            is_readonly: raw[Self::IS_READONLY_OFFSET] & Self::IS_READONLY_FLAG > 0,
            is_storable: raw[Self::IS_STORABLE_OFFSET] & Self::IS_STORABLE_FLAG > 0,
            input_device_id:
                (raw[Self::INPUT_DEVICE_ID_OFFSET] >> Self::INPUT_DEVICE_ID_SHIFT) & Self::INPUT_DEVICE_ID_MASK,
            output_device_id: raw[Self::OUTPUT_DEVICE_ID_OFFSET] & Self::OUTPUT_DEVICE_ID_MASK,
            input_count: raw[Self::INPUT_COUNT_OFFSET],
            output_count: raw[Self::OUTPUT_COUNT_OFFSET],
        }
    }
}

/// The enumeration to represent the type of ASIC.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AsicType {
    DiceII,
    Tcd2210,
    Tcd2220,
    Reserved(u8),
}

impl Default<> for AsicType {
    fn default() -> Self {
        AsicType::Reserved(0xff)
    }
}

impl From<u8> for AsicType {
    fn from(val: u8) -> Self {
        match val {
            0 => Self::DiceII,
            1 => Self::Tcd2210,
            2 => Self::Tcd2220,
            _ => Self::Reserved(val),
        }
    }
}

/// The structure to represent capability of general functionality.
#[derive(Default, Clone, Copy, Debug, Eq, PartialEq)]
pub struct GeneralCaps {
    pub dynamic_stream_format: bool,
    pub storage_avail: bool,
    pub peak_avail: bool,
    pub max_tx_streams: u8,
    pub max_rx_streams: u8,
    pub stream_format_is_storable: bool,
    pub asic_type: AsicType,
}

impl GeneralCaps {
    const SIZE: usize = 0x04;

    const DYNAMIC_STREAM_CONF_OFFSET: usize = 0x03;
    const STORAGE_AVAIL_OFFSET: usize = 0x03;
    const PEAK_AVAIL_OFFSET: usize = 0x03;
    const MAX_TX_STREAMS_OFFSET: usize = 0x03;
    const MAX_RX_STREAMS_OFFSET: usize = 0x02;
    const STREAM_CONF_IS_STORABLE_OFFSET: usize = 0x02;
    const ASIC_TYPE_OFFSET: usize = 0x01;

    const DYNAMIC_STREAM_CONF_FLAG: u8 = 0x01;
    const STORAGE_AVAIL_FLAG: u8 = 0x02;
    const PEAK_AVAIL_FLAG: u8 = 0x04;
    const MAX_TX_STREAMS_MASK: u8 = 0x0f;
    const MAX_RX_STREAMS_MASK: u8 = 0x0f;
    const STREAM_CONF_IS_STORABLE_FLAG: u8 = 0x10;

    const MAX_TX_STREAMS_SHIFT: usize = 4;
}

impl From<&[u8]> for GeneralCaps {
    fn from(raw: &[u8]) -> Self {
        GeneralCaps {
            dynamic_stream_format:
                raw[Self::DYNAMIC_STREAM_CONF_OFFSET] & Self::DYNAMIC_STREAM_CONF_FLAG > 0,
            storage_avail:
                raw[Self::STORAGE_AVAIL_OFFSET] & Self::STORAGE_AVAIL_FLAG > 0,
            peak_avail:
                raw[Self::PEAK_AVAIL_OFFSET] & Self::PEAK_AVAIL_FLAG > 0,
            max_tx_streams:
                (raw[Self::MAX_TX_STREAMS_OFFSET] >> Self::MAX_TX_STREAMS_SHIFT) & Self::MAX_TX_STREAMS_MASK,
            max_rx_streams:
                raw[Self::MAX_RX_STREAMS_OFFSET] & Self::MAX_RX_STREAMS_MASK,
            stream_format_is_storable:
                raw[Self::STREAM_CONF_IS_STORABLE_OFFSET] & Self::STREAM_CONF_IS_STORABLE_FLAG > 0,
            asic_type:
                AsicType::from(raw[Self::ASIC_TYPE_OFFSET]),
        }
    }
}

/// The structure to represent capabilities of each funtions.
#[derive(Default, Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExtensionCaps {
    pub router: RouterCaps,
    pub mixer: MixerCaps,
    pub general: GeneralCaps,
}

impl ExtensionCaps {
    const SIZE: usize = RouterCaps::SIZE + MixerCaps::SIZE + GeneralCaps::SIZE;
}

impl From<&[u8]> for ExtensionCaps {
    fn from(raw: &[u8]) -> Self {
        ExtensionCaps{
            router: RouterCaps::from(&raw[..RouterCaps::SIZE]),
            mixer: MixerCaps::from(&raw[RouterCaps::SIZE..(RouterCaps::SIZE + MixerCaps::SIZE)]),
            general: GeneralCaps::from(&raw[(RouterCaps::SIZE + MixerCaps::SIZE)..]),
        }
    }
}

/// The structure for protocol implementation of capabilities section.
#[derive(Default)]
pub struct CapsSectionProtocol;

impl CapsSectionProtocol {
    pub fn read_caps(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        timeout_ms: u32
    ) -> Result<ExtensionCaps, Error> {
        let mut data = [0; ExtensionCaps::SIZE];
        ProtocolExtension::read(req, node, sections.caps.offset, &mut data, timeout_ms)
            .map_err(|e| Error::new(ProtocolExtensionError::Caps, &e.to_string()))
            .map(|_| ExtensionCaps::from(&data[..]))
    }
}

#[cfg(test)]
mod test {
    use super::{ExtensionCaps, RouterCaps, MixerCaps, GeneralCaps, AsicType};

    #[test]
    fn caps_from() {
        let raw = [0xff, 0x00, 0x00, 0x07, 0x23, 0x12, 0x0c, 0xe7, 0x00, 0x00, 0x1b, 0xa3];
        let caps = ExtensionCaps{
            router: RouterCaps{
                is_exposed: true,
                is_readonly: true,
                is_storable: true,
                maximum_entry_count: 0xff00,
            },
            mixer: MixerCaps{
                is_exposed: true,
                is_readonly: true,
                is_storable: true,
                input_device_id: 0x0e,
                output_device_id: 0x0c,
                input_count: 0x12,
                output_count: 0x23,
            },
            general: GeneralCaps{
                dynamic_stream_format: true,
                storage_avail: true,
                peak_avail: false,
                max_tx_streams: 0x0a,
                max_rx_streams: 0x0b,
                stream_format_is_storable: true,
                asic_type: AsicType::DiceII,
            },
        };
        assert_eq!(caps, ExtensionCaps::from(&raw[..]));
    }
}
