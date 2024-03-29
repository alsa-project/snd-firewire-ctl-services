// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Protocol extension defined by TCAT for ASICs of DICE.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! extension defined by TC Applied Technologies (TCAT) for ASICs of Digital Interface Communication
//! Engine (DICE).

pub mod caps_section;
pub mod cmd_section;
pub mod current_config_section;
pub mod mixer_section;
pub mod peak_section;
#[doc(hidden)]
pub(crate) mod router_entry;
pub mod router_section;
pub mod standalone_section;
#[doc(hidden)]
mod stream_format_entry;
pub mod stream_format_section;

use {
    super::{global_section::ClockRate, *},
    std::cmp::Ordering,
};

pub use {
    caps_section::ExtensionCaps,
    current_config_section::{CurrentRouterParams, CurrentStreamFormatParams},
    mixer_section::{MixerCoefficientParams, MixerSaturationParams},
    peak_section::PeakParams,
    router_section::RouterParams,
    standalone_section::StandaloneParameters,
    stream_format_section::StreamFormatParams,
};

/// Sections for protocol extension.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct ExtensionSections {
    /// Capability.
    pub caps: Section,
    /// Command.
    pub cmd: Section,
    /// Mixer.
    pub mixer: Section,
    /// Peak.
    pub peak: Section,
    /// Router.
    pub router: Section,
    /// Stream format configuration.
    pub stream_format: Section,
    /// Current configurations.
    pub current_config: Section,
    /// Stand alone configuration.
    pub standalone: Section,
    /// Application specific configurations.
    pub application: Section,
}

impl ExtensionSections {
    const SECTION_COUNT: usize = 9;
    const SIZE: usize = Section::SIZE * Self::SECTION_COUNT;
}

#[cfg(test)]
fn serialize_extension_sections(
    sections: &ExtensionSections,
    raw: &mut [u8],
) -> Result<(), String> {
    assert!(raw.len() >= ExtensionSections::SIZE);

    serialize_section(&sections.caps, &mut raw[..8])?;
    serialize_section(&sections.cmd, &mut raw[8..16])?;
    serialize_section(&sections.mixer, &mut raw[16..24])?;
    serialize_section(&sections.peak, &mut raw[24..32])?;
    serialize_section(&sections.router, &mut raw[32..40])?;
    serialize_section(&sections.stream_format, &mut raw[40..48])?;
    serialize_section(&sections.current_config, &mut raw[48..56])?;
    serialize_section(&sections.standalone, &mut raw[56..64])?;
    serialize_section(&sections.application, &mut raw[64..72])?;

    Ok(())
}

fn deserialize_extension_sections(
    sections: &mut ExtensionSections,
    raw: &[u8],
) -> Result<(), String> {
    assert!(raw.len() >= ExtensionSections::SIZE);

    deserialize_section(&mut sections.caps, &raw[..8])?;
    deserialize_section(&mut sections.cmd, &raw[8..16])?;
    deserialize_section(&mut sections.mixer, &raw[16..24])?;
    deserialize_section(&mut sections.peak, &raw[24..32])?;
    deserialize_section(&mut sections.router, &raw[32..40])?;
    deserialize_section(&mut sections.stream_format, &raw[40..48])?;
    deserialize_section(&mut sections.current_config, &raw[48..56])?;
    deserialize_section(&mut sections.standalone, &raw[56..64])?;
    deserialize_section(&mut sections.application, &raw[64..72])?;

    Ok(())
}

/// Any error of protocol extension.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ProtocolExtensionError {
    /// Capability.
    Caps,
    /// Command.
    Cmd,
    /// Mixer.
    Mixer,
    /// Entry of router.
    RouterEntry,
    /// Peak.
    Peak,
    /// Router.
    Router,
    /// Entry of stream format.
    StreamFormatEntry,
    /// Stream format configuration.
    StreamFormat,
    /// Current configurations.
    CurrentConfig,
    /// Application specific configuration.
    Appl,
    /// Stand alone configuration.
    Standalone,
    Invalid(i32),
}

impl std::fmt::Display for ProtocolExtensionError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let msg = match self {
            ProtocolExtensionError::Caps => "caps",
            ProtocolExtensionError::Cmd => "command",
            ProtocolExtensionError::Mixer => "mixer",
            ProtocolExtensionError::RouterEntry => "router-entry",
            ProtocolExtensionError::Peak => "peak",
            ProtocolExtensionError::Router => "router",
            ProtocolExtensionError::StreamFormatEntry => "stream-format-entry",
            ProtocolExtensionError::StreamFormat => "stream-format",
            ProtocolExtensionError::CurrentConfig => "current-config",
            ProtocolExtensionError::Appl => "application",
            ProtocolExtensionError::Standalone => "standalone",
            ProtocolExtensionError::Invalid(_) => "invalid",
        };

        write!(f, "ProtocolExtensionError::{}", msg)
    }
}

impl ErrorDomain for ProtocolExtensionError {
    fn domain() -> Quark {
        Quark::from_str("tcat-protocol-extension-error-quark")
    }

    fn code(self) -> i32 {
        match self {
            ProtocolExtensionError::Caps => 0,
            ProtocolExtensionError::Cmd => 1,
            ProtocolExtensionError::Mixer => 2,
            ProtocolExtensionError::RouterEntry => 3,
            ProtocolExtensionError::Peak => 4,
            ProtocolExtensionError::Router => 5,
            ProtocolExtensionError::StreamFormatEntry => 6,
            ProtocolExtensionError::StreamFormat => 7,
            ProtocolExtensionError::CurrentConfig => 8,
            ProtocolExtensionError::Appl => 9,
            ProtocolExtensionError::Standalone => 10,
            ProtocolExtensionError::Invalid(v) => v,
        }
    }

    fn from(code: i32) -> Option<Self> {
        let enumeration = match code {
            0 => ProtocolExtensionError::Caps,
            1 => ProtocolExtensionError::Cmd,
            2 => ProtocolExtensionError::Mixer,
            3 => ProtocolExtensionError::RouterEntry,
            4 => ProtocolExtensionError::Peak,
            5 => ProtocolExtensionError::Router,
            7 => ProtocolExtensionError::StreamFormat,
            8 => ProtocolExtensionError::CurrentConfig,
            9 => ProtocolExtensionError::Appl,
            10 => ProtocolExtensionError::Standalone,
            _ => ProtocolExtensionError::Invalid(code),
        };
        Some(enumeration)
    }
}

const EXTENSION_OFFSET: usize = 0x00200000;

/// Operation of TCAT protocol extension.
pub trait TcatExtensionOperation: TcatOperation {
    /// Initiate read transaction to offset in specific address space and finish it.
    fn read_extension(
        req: &FwReq,
        node: &FwNode,
        section: &Section,
        offset: usize,
        frames: &mut [u8],
        timeout_ms: u32,
    ) -> Result<(), Error> {
        Self::read(
            req,
            node,
            EXTENSION_OFFSET + section.offset + offset,
            frames,
            timeout_ms,
        )
    }

    /// Initiate write transaction to offset in specific address space and finish it.
    fn write_extension(
        req: &FwReq,
        node: &FwNode,
        section: &Section,
        offset: usize,
        frames: &mut [u8],
        timeout_ms: u32,
    ) -> Result<(), Error> {
        Self::write(
            req,
            node,
            EXTENSION_OFFSET + section.offset + offset,
            frames,
            timeout_ms,
        )
    }

    /// Read section layout.
    fn read_extension_sections(
        req: &FwReq,
        node: &FwNode,
        sections: &mut ExtensionSections,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut raw = [0; ExtensionSections::SIZE];
        Self::read(req, node, EXTENSION_OFFSET, &mut raw, timeout_ms)
            .map(|_| deserialize_extension_sections(sections, &raw).unwrap())
    }
}

/// Operation for whole parameters in section of TCAT protocol extension.
pub trait TcatExtensionSectionParamsOperation<T: Debug> {
    /// Cache state of hardware for whole parameters.
    fn cache_extension_whole_params(
        req: &FwReq,
        node: &FwNode,
        sections: &ExtensionSections,
        caps: &ExtensionCaps,
        params: &mut T,
        timeout_ms: u32,
    ) -> Result<(), Error>;
}

/// Operation for whole mutable parameters in section of TCAT protocol extension.
pub trait TcatExtensionSectionWholeMutableParamsOperation<T: Debug> {
    /// Update state of hardware for whole parameters.
    fn update_extension_whole_params(
        req: &FwReq,
        node: &FwNode,
        sections: &ExtensionSections,
        caps: &ExtensionCaps,
        params: &T,
        timeout_ms: u32,
    ) -> Result<(), Error>;
}

/// Operation for partial mutable parameters in section of TCAT protocol extension.
pub trait TcatExtensionSectionPartialMutableParamsOperation<T: Debug> {
    /// Update state of hardware for partial parameters.
    fn update_extension_partial_params(
        req: &FwReq,
        node: &FwNode,
        sections: &ExtensionSections,
        caps: &ExtensionCaps,
        params: &T,
        prev: &mut T,
        timeout_ms: u32,
    ) -> Result<(), Error>;
}
/// Operation for parameters in which any change is notified to owner application in TCAT protocol
/// extension.
pub trait TcatExtensionSectionNotifiedParamsOperation<T: Debug> {
    /// Cache state of hardware for notified parameters.
    fn cache_extension_notified_params(
        req: &FwReq,
        node: &FwNode,
        sections: &ExtensionSections,
        caps: &ExtensionCaps,
        params: &mut T,
        msg: u32,
        timeout_ms: u32,
    ) -> Result<(), Error>;
}

/// Identifier of destination block.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DstBlkId {
    Aes,
    Adat,
    MixerTx0,
    MixerTx1,
    Ins0,
    Ins1,
    ArmApbAudio,
    Avs0,
    Avs1,
    Reserved(u8),
}

impl Default for DstBlkId {
    fn default() -> Self {
        DstBlkId::Reserved(0xff)
    }
}

/// Destination block in router function.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct DstBlk {
    /// The identifier of destination block.
    pub id: DstBlkId,
    /// The channel number.
    pub ch: u8,
}

/// Identifier of source block.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SrcBlkId {
    /// AES, S/PDIF, and TOS outputs.
    Aes,
    /// ADAT outputs.
    Adat,
    /// mixer outputs.
    Mixer,
    /// Analog outputs A (Inter IC sound interface).
    Ins0,
    /// Analog outputs B (Inter IC sound interface).
    Ins1,
    /// Standard AMBA 2.0 compliant APB interface.
    ArmAprAudio,
    /// 1394 Audio video system A.
    Avs0,
    /// 1394 Audio video system B.
    Avs1,
    /// Discard audio signal.
    Mute,
    Reserved(u8),
}

impl Default for SrcBlkId {
    fn default() -> Self {
        SrcBlkId::Reserved(0xff)
    }
}

/// Source block in router function.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct SrcBlk {
    /// The identifier of source block.
    pub id: SrcBlkId,
    /// The channel number.
    pub ch: u8,
}

/// Entry of route in router function.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct RouterEntry {
    /// Destination block.
    pub dst: DstBlk,
    /// Source block.
    pub src: SrcBlk,
    /// Detected level of audio signal.
    pub peak: u16,
}

/// Entry of stream format.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct FormatEntry {
    /// The number of PCM channels.
    pub pcm_count: u8,
    /// The number of MIDI channels.
    pub midi_count: u8,
    /// Labels for the channels.
    pub labels: Vec<String>,
    /// AC3 capabilities.
    pub enable_ac3: [bool; AC3_CHANNELS],
}

/// The number of channels in stream format for AC3 channels.
pub const AC3_CHANNELS: usize = 32;

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn section_from() {
        let raw = [
            0x00, 0x00, 0x00, 0x11, 0x00, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00, 0x0f, 0x00, 0x00,
            0x00, 0x0e, 0x00, 0x00, 0x00, 0x0d, 0x00, 0x00, 0x00, 0x0c, 0x00, 0x00, 0x00, 0x0b,
            0x00, 0x00, 0x00, 0x0a, 0x00, 0x00, 0x00, 0x09, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00,
            0x00, 0x07, 0x00, 0x00, 0x00, 0x06, 0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x04,
            0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00,
            0x00, 0x00,
        ];
        let space = ExtensionSections {
            caps: Section {
                offset: 0x44,
                size: 0x40,
            },
            cmd: Section {
                offset: 0x3c,
                size: 0x38,
            },
            mixer: Section {
                offset: 0x34,
                size: 0x30,
            },
            peak: Section {
                offset: 0x2c,
                size: 0x28,
            },
            router: Section {
                offset: 0x24,
                size: 0x20,
            },
            stream_format: Section {
                offset: 0x1c,
                size: 0x18,
            },
            current_config: Section {
                offset: 0x14,
                size: 0x10,
            },
            standalone: Section {
                offset: 0x0c,
                size: 0x08,
            },
            application: Section {
                offset: 0x04,
                size: 0x00,
            },
        };

        let mut r = vec![0u8; raw.len()];
        serialize_extension_sections(&space, &mut r).unwrap();
        assert_eq!(&raw[..], &r);

        let mut s = ExtensionSections::default();
        deserialize_extension_sections(&mut s, &raw).unwrap();
        assert_eq!(space, s);
    }
}
