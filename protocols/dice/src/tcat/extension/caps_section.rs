// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Capabilities section in protocol extension defined by TCAT for ASICs of DICE.
//!
//! The module includes structure, enumeration, and trait and its implementation for capabilities
//! section in protocol extension defined by TCAT for ASICs of DICE.
use super::*;

/// Capability of router functionality.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct RouterCaps {
    /// Whether router configuration is exposed to owner software.
    pub is_exposed: bool,
    /// Whether router configuration is read only.
    pub is_readonly: bool,
    /// Whether router configuration is storable in on-board flash memory.
    pub is_storable: bool,
    /// The maximum number of entry for router.
    pub maximum_entry_count: u16,
}

impl RouterCaps {
    const SIZE: usize = 4;

    const IS_EXPOSED_FLAG: u32 = 0x00000001;
    const IS_READONLY_FLAG: u32 = 0x00000002;
    const IS_STORABLE_FLAG: u32 = 0x00000004;
    const MAX_ENTRY_COUNT_MASK: u32 = 0xffff0000;
    const MAX_ENTYR_COUNT_SHIFT: usize = 16;
}

#[cfg(test)]
fn serialize_router_caps(params: &RouterCaps, raw: &mut [u8]) -> Result<(), String> {
    assert!(raw.len() >= RouterCaps::SIZE);

    let mut val = 0u32;

    if params.is_exposed {
        val |= RouterCaps::IS_EXPOSED_FLAG;
    }
    if params.is_readonly {
        val |= RouterCaps::IS_READONLY_FLAG;
    }
    if params.is_storable {
        val |= RouterCaps::IS_STORABLE_FLAG;
    }
    val |= (params.maximum_entry_count as u32) << RouterCaps::MAX_ENTYR_COUNT_SHIFT;

    val.build_quadlet(raw);

    Ok(())
}

fn deserialize_router_caps(params: &mut RouterCaps, raw: &[u8]) -> Result<(), String> {
    assert!(raw.len() >= RouterCaps::SIZE);

    let mut val = 0u32;
    val.parse_quadlet(raw);

    params.is_exposed = val & RouterCaps::IS_EXPOSED_FLAG > 0;
    params.is_readonly = val & RouterCaps::IS_READONLY_FLAG > 0;
    params.is_storable = val & RouterCaps::IS_STORABLE_FLAG > 0;
    params.maximum_entry_count =
        ((val & RouterCaps::MAX_ENTRY_COUNT_MASK) >> RouterCaps::MAX_ENTYR_COUNT_SHIFT) as u16;

    Ok(())
}

/// Capability of mixer functionality.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct MixerCaps {
    /// Whether mixer configuration is exposed to owner software.
    pub is_exposed: bool,
    /// Whether mixer configuration is read only.
    pub is_readonly: bool,
    /// Whether mixer configuration is storable in on-board flash memory.
    pub is_storable: bool,
    /// The numeric identifier of input device.
    pub input_device_id: u8,
    /// The numeric identifier of output device.
    pub output_device_id: u8,
    /// The number of input devices.
    pub input_count: u8,
    /// The number of output devices.
    pub output_count: u8,
}

impl MixerCaps {
    const SIZE: usize = 0x04;

    const IS_EXPOSED_FLAG: u32 = 0x00000001;
    const IS_READONLY_FLAG: u32 = 0x00000002;
    const IS_STORABLE_FLAG: u32 = 0x00000004;

    const INPUT_DEVICE_ID_MASK: u32 = 0x000000f0;
    const OUTPUT_DEVICE_ID_MASK: u32 = 0x00000f00;

    const INPUT_DEVICE_ID_SHIFT: usize = 4;
    const OUTPUT_DEVICE_ID_SHIFT: usize = 8;

    const INPUT_COUNT_MASK: u32 = 0x00ff0000;
    const OUTPUT_COUNT_MASK: u32 = 0xff000000;

    const INPUT_COUNT_SHIFT: usize = 16;
    const OUTPUT_COUNT_SHIFT: usize = 24;
}

#[cfg(test)]
fn serialize_mixer_caps(params: &MixerCaps, raw: &mut [u8]) -> Result<(), String> {
    assert!(raw.len() >= MixerCaps::SIZE);

    let mut val = 0;

    if params.is_exposed {
        val |= MixerCaps::IS_EXPOSED_FLAG;
    }
    if params.is_readonly {
        val |= MixerCaps::IS_READONLY_FLAG;
    }
    if params.is_storable {
        val |= MixerCaps::IS_STORABLE_FLAG;
    }
    val |= (params.input_device_id as u32) << MixerCaps::INPUT_DEVICE_ID_SHIFT;
    val |= (params.output_device_id as u32) << MixerCaps::OUTPUT_DEVICE_ID_SHIFT;
    val |= (params.input_count as u32) << 16;
    val |= (params.output_count as u32) << 24;

    val.build_quadlet(raw);

    Ok(())
}

fn deserialize_mixer_caps(params: &mut MixerCaps, raw: &[u8]) -> Result<(), String> {
    assert!(raw.len() >= MixerCaps::SIZE);

    let mut val = 0u32;
    val.parse_quadlet(raw);

    params.is_exposed = val & MixerCaps::IS_EXPOSED_FLAG > 0;
    params.is_readonly = val & MixerCaps::IS_READONLY_FLAG > 0;
    params.is_storable = val & MixerCaps::IS_STORABLE_FLAG > 0;
    params.input_device_id =
        ((val & MixerCaps::INPUT_DEVICE_ID_MASK) >> MixerCaps::INPUT_DEVICE_ID_SHIFT) as u8;
    params.output_device_id =
        ((val & MixerCaps::OUTPUT_DEVICE_ID_MASK) >> MixerCaps::OUTPUT_DEVICE_ID_SHIFT) as u8;
    params.input_count =
        ((val & MixerCaps::INPUT_COUNT_MASK) >> MixerCaps::INPUT_COUNT_SHIFT) as u8;
    params.output_count =
        ((val & MixerCaps::OUTPUT_COUNT_MASK) >> MixerCaps::OUTPUT_COUNT_SHIFT) as u8;

    Ok(())
}

/// Type of ASIC.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum AsicType {
    /// DICE II.
    DiceII,
    /// TCD2210 a.k.a DICE Mini.
    Tcd2210,
    /// TCD2220 a.k.a DICE Jr.
    Tcd2220,
}

impl Default for AsicType {
    fn default() -> Self {
        AsicType::DiceII
    }
}

/// Capability of general functionality.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct GeneralCaps {
    /// Whether stream format is mutable dynamically.
    pub dynamic_stream_format: bool,
    /// Whether on-board flash memory is available.
    pub storage_avail: bool,
    /// Whether peak section is available.
    pub peak_avail: bool,
    /// The maximum number of tx streams.
    pub max_tx_streams: u8,
    /// The maximum number of rx streams.
    pub max_rx_streams: u8,
    /// Whether stream format configuration is storable in on-board flash memory.
    pub stream_format_is_storable: bool,
    /// The type of ASIC.
    pub asic_type: AsicType,
}

impl GeneralCaps {
    const SIZE: usize = 0x04;

    const DYNAMIC_STREAM_CONF_FLAG: u32 = 0x00000001;
    const STORAGE_AVAIL_FLAG: u32 = 0x00000002;
    const PEAK_AVAIL_FLAG: u32 = 0x00000004;

    const MAX_TX_STREAMS_MASK: u32 = 0x000000f0;
    const MAX_RX_STREAMS_MASK: u32 = 0x00000f00;

    const MAX_TX_STREAMS_SHIFT: usize = 4;
    const MAX_RX_STREAMS_SHIFT: usize = 8;

    const STREAM_CONF_IS_STORABLE_FLAG: u32 = 0x00001000;

    const DICE_II_VALUE: u16 = 0;
    const TCD2210_VALUE: u16 = 1;
    const TCD2220_VALUE: u16 = 2;

    const ASIC_TYPE_MASK: u32 = 0xffff0000;
    const ASIC_TYPE_SHIFT: usize = 16;
}

#[cfg(test)]
fn serialize_general_caps(params: &GeneralCaps, raw: &mut [u8]) -> Result<(), String> {
    assert!(raw.len() >= GeneralCaps::SIZE);

    let mut val = 0;

    if params.dynamic_stream_format {
        val |= GeneralCaps::DYNAMIC_STREAM_CONF_FLAG;
    }
    if params.storage_avail {
        val |= GeneralCaps::STORAGE_AVAIL_FLAG;
    }
    if params.peak_avail {
        val |= GeneralCaps::PEAK_AVAIL_FLAG;
    }
    val |= (params.max_tx_streams as u32) << GeneralCaps::MAX_TX_STREAMS_SHIFT;
    val |= (params.max_rx_streams as u32) << GeneralCaps::MAX_RX_STREAMS_SHIFT;
    if params.stream_format_is_storable {
        val |= GeneralCaps::STREAM_CONF_IS_STORABLE_FLAG;
    }
    let v = match params.asic_type {
        AsicType::DiceII => GeneralCaps::DICE_II_VALUE,
        AsicType::Tcd2210 => GeneralCaps::TCD2210_VALUE,
        AsicType::Tcd2220 => GeneralCaps::TCD2220_VALUE,
    };
    val |= (v as u32) << GeneralCaps::ASIC_TYPE_SHIFT;

    val.build_quadlet(raw);

    Ok(())
}

fn deserialize_general_caps(params: &mut GeneralCaps, raw: &[u8]) -> Result<(), String> {
    assert!(raw.len() >= GeneralCaps::SIZE);

    let mut val = 0u32;
    val.parse_quadlet(raw);

    params.dynamic_stream_format = val & GeneralCaps::DYNAMIC_STREAM_CONF_FLAG > 0;
    params.storage_avail = val & GeneralCaps::STORAGE_AVAIL_FLAG > 0;
    params.peak_avail = val & GeneralCaps::PEAK_AVAIL_FLAG > 0;
    params.max_tx_streams =
        ((val & GeneralCaps::MAX_TX_STREAMS_MASK) >> GeneralCaps::MAX_TX_STREAMS_SHIFT) as u8;
    params.max_rx_streams =
        ((val & GeneralCaps::MAX_RX_STREAMS_MASK) >> GeneralCaps::MAX_RX_STREAMS_SHIFT) as u8;
    params.stream_format_is_storable = val & GeneralCaps::STREAM_CONF_IS_STORABLE_FLAG > 0;
    let v = ((val & GeneralCaps::ASIC_TYPE_MASK) >> GeneralCaps::ASIC_TYPE_SHIFT) as u16;
    params.asic_type = match v {
        GeneralCaps::DICE_II_VALUE => AsicType::DiceII,
        GeneralCaps::TCD2210_VALUE => AsicType::Tcd2210,
        GeneralCaps::TCD2220_VALUE => AsicType::Tcd2220,
        _ => Err(format!("ASIC type not found for value {}", v))?,
    };

    Ok(())
}

/// Capabilities of each funtions.
#[derive(Default, Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExtensionCaps {
    /// Capabilities for router function.
    pub router: RouterCaps,
    /// Capabilities for mixer function.
    pub mixer: MixerCaps,
    /// Capabilities for general function.
    pub general: GeneralCaps,
}

#[cfg(test)]
fn serialize_extension_caps(params: &ExtensionCaps, raw: &mut [u8]) -> Result<(), String> {
    assert!(raw.len() >= ExtensionCaps::SIZE);

    serialize_router_caps(&params.router, &mut raw[..4])?;
    serialize_mixer_caps(&params.mixer, &mut raw[4..8])?;
    serialize_general_caps(&params.general, &mut raw[8..12])?;

    Ok(())
}

fn deserialize_extension_caps(params: &mut ExtensionCaps, raw: &[u8]) -> Result<(), String> {
    assert!(raw.len() >= ExtensionCaps::SIZE);

    deserialize_router_caps(&mut params.router, &raw[..4])?;
    deserialize_mixer_caps(&mut params.mixer, &raw[4..8])?;
    deserialize_general_caps(&mut params.general, &raw[8..12])?;

    Ok(())
}

impl ExtensionCaps {
    const SIZE: usize = RouterCaps::SIZE + MixerCaps::SIZE + GeneralCaps::SIZE;
}

/// Protocol implementation of capabilities section.
#[derive(Default)]
pub struct CapsSectionProtocol;

impl CapsSectionProtocol {
    pub fn read_caps(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        timeout_ms: u32,
    ) -> Result<ExtensionCaps, Error> {
        let mut raw = vec![0; ExtensionCaps::SIZE];
        extension_read(req, node, sections.caps.offset, &mut raw, timeout_ms)
            .map_err(|e| Error::new(ProtocolExtensionError::Caps, &e.to_string()))?;

        let mut caps = ExtensionCaps::default();
        deserialize_extension_caps(&mut caps, &raw)
            .map_err(|cause| Error::new(ProtocolExtensionError::Caps, &cause))?;

        Ok(caps)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn caps_serdes() {
        let raw = [
            0xff, 0x00, 0x00, 0x07, 0x23, 0x12, 0x0c, 0xe7, 0x00, 0x00, 0x1b, 0xa3,
        ];
        let caps = ExtensionCaps {
            router: RouterCaps {
                is_exposed: true,
                is_readonly: true,
                is_storable: true,
                maximum_entry_count: 0xff00,
            },
            mixer: MixerCaps {
                is_exposed: true,
                is_readonly: true,
                is_storable: true,
                input_device_id: 0x0e,
                output_device_id: 0x0c,
                input_count: 0x12,
                output_count: 0x23,
            },
            general: GeneralCaps {
                dynamic_stream_format: true,
                storage_avail: true,
                peak_avail: false,
                max_tx_streams: 0x0a,
                max_rx_streams: 0x0b,
                stream_format_is_storable: true,
                asic_type: AsicType::DiceII,
            },
        };
        let mut r = vec![0u8; ExtensionCaps::SIZE];
        serialize_extension_caps(&caps, &mut r).unwrap();
        assert_eq!(&raw[..], &r);

        let mut c = ExtensionCaps::default();
        deserialize_extension_caps(&mut c, &raw).unwrap();
        assert_eq!(caps, c);
    }
}
