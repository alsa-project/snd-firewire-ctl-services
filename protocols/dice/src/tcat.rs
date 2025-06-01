// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Protocol defined by TCAT for ASICs of DICE.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol defined
//! by TC Applied Technologies (TCAT) for ASICs of Digital Interface Communication Engine (DICE).
//!
//! In the protocol, all of features are categorized to several parts. Each part is represented in
//! range of registers accessible by IEEE 1394 asynchronous transaction. In the crate, the range
//! is called as `section`, therefore the features are categorized to the section.

pub mod ext_sync_section;
pub mod global_section;
pub mod rx_stream_format_section;
pub mod tx_stream_format_section;

pub mod extension;
pub mod tcd22xx_spec;

pub mod config_rom;

use {
    super::*,
    glib::{error::ErrorDomain, Quark},
    hinawa::{prelude::FwReqExtManual, FwTcode},
    std::fmt::Debug,
};

pub use {
    ext_sync_section::ExtendedSyncParameters,
    global_section::{GlobalParameters, TcatGlobalSectionSpecification},
    rx_stream_format_section::RxStreamFormatParameters,
    tx_stream_format_section::TxStreamFormatParameters,
};

/// Section in control and status register (CSR) of node.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Section {
    /// The offset of section in specific address space.
    pub offset: usize,
    /// The size of section.
    pub size: usize,
}

impl Section {
    pub(crate) const SIZE: usize = 8;
}

#[cfg(test)]
pub(crate) fn serialize_section(section: &Section, raw: &mut [u8]) -> Result<(), String> {
    assert!(raw.len() >= Section::SIZE);

    let val = (section.offset as u32) / 4;
    serialize_u32(&val, &mut raw[..4]);

    let val = (section.size as u32) / 4;
    serialize_u32(&val, &mut raw[4..8]);

    Ok(())
}

pub(crate) fn deserialize_section(section: &mut Section, raw: &[u8]) -> Result<(), String> {
    assert!(raw.len() >= Section::SIZE);

    let mut val = 0u32;
    deserialize_u32(&mut val, &raw[..4]);
    section.offset = 4 * val as usize;

    deserialize_u32(&mut val, &raw[4..8]);
    section.size = 4 * val as usize;

    Ok(())
}

/// The sset of sections in CSR of node.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct GeneralSections {
    /// For global settings.
    pub global: Section,
    /// For tx stream format settings.
    pub tx_stream_format: Section,
    /// For rx stream format settings.
    pub rx_stream_format: Section,
    /// For extended status of synchronization for signal sources of sampling clock.
    pub ext_sync: Section,
    pub reserved: Section,
}

impl GeneralSections {
    const SECTION_COUNT: usize = 5;
    const SIZE: usize = Section::SIZE * Self::SECTION_COUNT;
}

#[cfg(test)]
fn serialize_general_sections(sections: &GeneralSections, raw: &mut [u8]) -> Result<(), String> {
    assert!(raw.len() >= GeneralSections::SIZE);

    serialize_section(&sections.global, &mut raw[..8])?;
    serialize_section(&sections.tx_stream_format, &mut raw[8..16])?;
    serialize_section(&sections.rx_stream_format, &mut raw[16..24])?;
    serialize_section(&sections.ext_sync, &mut raw[24..32])?;
    serialize_section(&sections.reserved, &mut raw[32..40])?;

    Ok(())
}

fn deserialize_general_sections(sections: &mut GeneralSections, raw: &[u8]) -> Result<(), String> {
    assert!(raw.len() >= GeneralSections::SIZE);

    deserialize_section(&mut sections.global, &raw[..8])?;
    deserialize_section(&mut sections.tx_stream_format, &raw[8..16])?;
    deserialize_section(&mut sections.rx_stream_format, &raw[16..24])?;
    deserialize_section(&mut sections.ext_sync, &raw[24..32])?;
    deserialize_section(&mut sections.reserved, &raw[32..40])?;

    Ok(())
}

/// Serializer and deserializer for parameters in TCAT section.
pub trait TcatSectionSerdes<T> {
    /// Minimum size of section for parameters.
    const MIN_SIZE: usize;

    /// The type of error.
    const ERROR_TYPE: GeneralProtocolError;

    /// Serialize parameters for section.
    fn serialize(params: &T, raw: &mut [u8]) -> Result<(), String>;

    /// Deserialize section for parameters.
    fn deserialize(params: &mut T, raw: &[u8]) -> Result<(), String>;
}

/// Any error of general protocol.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum GeneralProtocolError {
    /// Error to operate for global settings.
    Global,
    /// Error to operate for tx stream format settings.
    TxStreamFormat,
    /// Error to operate for rx stream format settings.
    RxStreamFormat,
    /// Error to operate for external synchronization states.
    ExtendedSync,
    /// Any error in application implementation developed by vendors.
    VendorDependent,
    Invalid(i32),
}

impl std::fmt::Display for GeneralProtocolError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let msg = match self {
            GeneralProtocolError::Global => "global",
            GeneralProtocolError::TxStreamFormat => "tx-stream-format",
            GeneralProtocolError::RxStreamFormat => "rx-stream-format",
            GeneralProtocolError::ExtendedSync => "external-sync",
            GeneralProtocolError::VendorDependent => "vendor-dependent",
            GeneralProtocolError::Invalid(_) => "invalid",
        };

        write!(f, "GeneralProtocolError::{}", msg)
    }
}

impl ErrorDomain for GeneralProtocolError {
    fn domain() -> Quark {
        Quark::from_str("tcat-general-protocol-error-quark")
    }

    fn code(self) -> i32 {
        match self {
            GeneralProtocolError::Global => 0,
            GeneralProtocolError::TxStreamFormat => 1,
            GeneralProtocolError::RxStreamFormat => 2,
            GeneralProtocolError::ExtendedSync => 3,
            GeneralProtocolError::VendorDependent => 4,
            GeneralProtocolError::Invalid(v) => v,
        }
    }

    fn from(code: i32) -> Option<Self> {
        let enumeration = match code {
            0 => GeneralProtocolError::Global,
            1 => GeneralProtocolError::TxStreamFormat,
            2 => GeneralProtocolError::RxStreamFormat,
            3 => GeneralProtocolError::ExtendedSync,
            4 => GeneralProtocolError::VendorDependent,
            _ => GeneralProtocolError::Invalid(code),
        };
        Some(enumeration)
    }
}

const MAX_FRAME_SIZE: usize = 512;

/// Operation of TCAT general protocol.
pub trait TcatOperation {
    /// Initiate read transaction to offset in specific address space and finish it.
    fn read(
        req: &FwReq,
        node: &FwNode,
        offset: usize,
        mut frames: &mut [u8],
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut addr = BASE_ADDR + offset as u64;

        while frames.len() > 0 {
            let len = std::cmp::min(frames.len(), MAX_FRAME_SIZE);
            let tcode = if len == 4 {
                FwTcode::ReadQuadletRequest
            } else {
                FwTcode::ReadBlockRequest
            };

            req.transaction(node, tcode, addr, len, &mut frames[0..len], timeout_ms)?;

            addr += len as u64;
            frames = &mut frames[len..];
        }

        Ok(())
    }

    /// Initiate write transaction to offset in specific address space and finish it.
    fn write(
        req: &FwReq,
        node: &FwNode,
        offset: usize,
        mut frames: &mut [u8],
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut addr = BASE_ADDR + (offset as u64);

        while frames.len() > 0 {
            let len = std::cmp::min(frames.len(), MAX_FRAME_SIZE);
            let tcode = if len == 4 {
                FwTcode::WriteQuadletRequest
            } else {
                FwTcode::WriteBlockRequest
            };

            req.transaction(node, tcode, addr, len, &mut frames[0..len], timeout_ms)?;

            addr += len as u64;
            frames = &mut frames[len..];
        }

        Ok(())
    }

    /// Read section layout.
    fn read_general_sections(
        req: &FwReq,
        node: &FwNode,
        sections: &mut GeneralSections,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut raw = [0; GeneralSections::SIZE];
        Self::read(req, node, 0, &mut raw, timeout_ms)?;
        deserialize_general_sections(sections, &raw)
            .map_err(|cause| Error::new(GeneralProtocolError::Invalid(0), &cause))
    }
}

fn check_section_cache(
    section: &Section,
    min_size: usize,
    error_type: GeneralProtocolError,
) -> Result<(), Error> {
    if section.size < min_size {
        let msg = format!(
            "The size of section should be larger than {}, actually {}",
            min_size, section.size
        );
        Err(Error::new(error_type, &msg))
    } else {
        Ok(())
    }
}

/// Operation for parameters in section of TCAT general protocol.
pub trait TcatSectionOperation<T>: TcatOperation + TcatSectionSerdes<T>
where
    T: Default + Debug,
{
    /// Cache whole section and deserialize for parameters.
    fn whole_cache(
        req: &FwReq,
        node: &FwNode,
        section: &Section,
        params: &mut T,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        check_section_cache(section, Self::MIN_SIZE, Self::ERROR_TYPE)?;
        let mut raw = vec![0u8; section.size];
        Self::read(req, node, section.offset, &mut raw, timeout_ms)?;
        Self::deserialize(params, &raw).map_err(|msg| Error::new(Self::ERROR_TYPE, &msg))
    }
}

/// Operation to change content in section of TCAT general protocol for parameters.
pub trait TcatMutableSectionOperation<T>: TcatOperation + TcatSectionSerdes<T>
where
    T: Default + Debug,
{
    /// Update whole section by the parameters.
    fn whole_update(
        req: &FwReq,
        node: &FwNode,
        section: &Section,
        params: &T,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        check_section_cache(section, Self::MIN_SIZE, Self::ERROR_TYPE)?;
        let mut raw = vec![0u8; section.size];
        Self::serialize(params, &mut raw).map_err(|msg| Error::new(Self::ERROR_TYPE, &msg))?;
        Self::write(req, node, section.offset, &mut raw, timeout_ms)
    }

    /// Update part of section for any change at the parameters.
    fn partial_update(
        req: &FwReq,
        node: &FwNode,
        section: &Section,
        params: &T,
        prev: &mut T,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        check_section_cache(section, Self::MIN_SIZE, Self::ERROR_TYPE)?;

        let mut new = vec![0u8; section.size];
        Self::serialize(params, &mut new).map_err(|msg| Error::new(Self::ERROR_TYPE, &msg))?;

        let mut old = vec![0u8; section.size];
        Self::serialize(prev, &mut old).map_err(|msg| Error::new(Self::ERROR_TYPE, &msg))?;

        (0..section.size).step_by(4).try_for_each(|pos| {
            if new[pos..(pos + 4)] != old[pos..(pos + 4)] {
                Self::write(
                    req,
                    node,
                    section.offset + pos,
                    &mut new[pos..(pos + 4)],
                    timeout_ms,
                )
            } else {
                Ok(())
            }
        })?;

        Self::deserialize(prev, &new).map_err(|msg| Error::new(Self::ERROR_TYPE, &msg))
    }
}

/// Operation for notified parameters in section of TCAT general protocol.
pub trait TcatNotifiedSectionOperation<T>: TcatSectionOperation<T>
where
    T: Default + Debug,
{
    /// Flag in message notified for any change in section.
    const NOTIFY_FLAG: u32;

    /// Check message to be notified or not.
    fn notified(_: &T, msg: u32) -> bool {
        msg & Self::NOTIFY_FLAG > 0
    }
}

/// Operation for fluctuated content in section of TCAT general protocol.
pub trait TcatFluctuatedSectionOperation<T>: TcatSectionOperation<T>
where
    T: Default + Debug,
{
    /// The set of address offsets in which any value is changed apart from software operation;
    /// e.g. hardware metering.
    const FLUCTUATED_OFFSETS: &'static [usize];

    /// Cache part of section for fluctuated values, then deserialize for parameters.
    fn partial_cache(
        req: &FwReq,
        node: &FwNode,
        section: &Section,
        params: &mut T,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        check_section_cache(section, Self::MIN_SIZE, Self::ERROR_TYPE)?;

        let mut raw = vec![0u8; section.size];
        Self::serialize(params, &mut raw).map_err(|msg| Error::new(Self::ERROR_TYPE, &msg))?;

        Self::FLUCTUATED_OFFSETS.iter().try_for_each(|&offset| {
            Self::read(
                req,
                node,
                section.offset + offset,
                &mut raw[offset..(offset + 4)],
                timeout_ms,
            )
        })?;
        Self::deserialize(params, &raw).map_err(|msg| Error::new(Self::ERROR_TYPE, &msg))
    }
}

const BASE_ADDR: u64 = 0xffffe0000000;

/// Parameter of stream format for IEC 60958.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Iec60958Param {
    /// The corresponding channel supports IEC 60958 bit stream.
    pub cap: bool,
    /// The corresponding channel is enabled for IEC 60958 bit stream.
    pub enable: bool,
}

/// The maximum number of IEC 60958 channels for stream format entry.
pub const IEC60958_CHANNELS: usize = 32;

fn serialize_iec60958_params(
    params: &[Iec60958Param; IEC60958_CHANNELS],
    raw: &mut [u8],
) -> Result<(), String> {
    assert!(raw.len() >= IEC60958_CHANNELS / 8 * 2);

    let (caps, enables) =
        params
            .iter()
            .enumerate()
            .fold((0u32, 0u32), |(mut caps, mut enables), (i, params)| {
                if params.cap {
                    caps |= 1 << i;
                }
                if params.enable {
                    enables |= 1 << i;
                }
                (caps, enables)
            });

    raw[..4].copy_from_slice(&caps.to_be_bytes());
    raw[4..8].copy_from_slice(&enables.to_be_bytes());

    Ok(())
}

fn deserialize_iec60958_params(
    params: &mut [Iec60958Param; IEC60958_CHANNELS],
    raw: &[u8],
) -> Result<(), String> {
    assert!(raw.len() >= IEC60958_CHANNELS / 8 * 2);

    let mut quadlet = [0; 4];

    quadlet.copy_from_slice(&raw[..4]);
    let caps = u32::from_be_bytes(quadlet);

    quadlet.copy_from_slice(&raw[4..8]);
    let enables = u32::from_be_bytes(quadlet);

    params.iter_mut().enumerate().for_each(|(i, param)| {
        param.cap = (1 << i) & caps > 0;
        param.enable = (1 << i) & enables > 0;
    });

    Ok(())
}

fn from_ne(raw: &mut [u8]) {
    let mut quadlet = [0; 4];
    (0..raw.len()).step_by(4).for_each(|pos| {
        quadlet.copy_from_slice(&raw[pos..(pos + 4)]);
        raw[pos..(pos + 4)].copy_from_slice(&u32::from_ne_bytes(quadlet).to_be_bytes());
    });
}

fn to_ne(raw: &mut [u8]) {
    let mut quadlet = [0; 4];
    (0..raw.len()).step_by(4).for_each(|pos| {
        quadlet.copy_from_slice(&raw[pos..(pos + 4)]);
        raw[pos..(pos + 4)].copy_from_slice(&u32::from_be_bytes(quadlet).to_ne_bytes());
    });
}

fn serialize_label<T: AsRef<str>>(name: T, raw: &mut [u8]) -> Result<(), String> {
    let r = name.as_ref().as_bytes();

    if r.len() >= raw.len() {
        Err(format!("Insufficient buffer size {} for label", raw.len()))
    } else {
        raw[..r.len()].copy_from_slice(r);
        from_ne(raw);

        Ok(())
    }
}

fn serialize_labels<T: AsRef<str>>(labels: &[T], raw: &mut [u8]) -> Result<(), String> {
    raw.fill(0x00);

    let mut pos = 0;
    labels.iter().try_for_each(|label| {
        let r = label.as_ref().as_bytes();

        if pos + r.len() + 1 >= raw.len() {
            Err(format!(
                "Insufficient buffer size {} for all of labels",
                raw.len()
            ))
        } else {
            let end = pos + r.len();
            raw[pos..end].copy_from_slice(r);

            raw[end] = '\\' as u8;
            pos = end + 1;

            Ok(())
        }
    })?;

    if pos + 1 >= raw.len() {
        Err(format!(
            "Insufficient buffer size {} for all of labels",
            raw.len()
        ))
    } else {
        raw[pos] = '\\' as u8;

        from_ne(raw);

        Ok(())
    }
}

fn deserialize_label(label: &mut String, raw: &[u8]) -> Result<(), String> {
    let mut data = raw.to_vec();
    to_ne(&mut data);

    data.push(0x00);
    std::str::from_utf8(&data)
        .map_err(|err| err.to_string())
        .and_then(|text| {
            text.find('\0')
                .ok_or_else(|| "String terminator not found".to_string())
                .map(|pos| *label = text[..pos].to_string())
        })
}

fn deserialize_labels(labels: &mut Vec<String>, raw: &[u8]) -> Result<(), String> {
    labels.truncate(0);

    let mut data = raw.to_vec();
    to_ne(&mut data);

    // The terminator for the sequence of labels is a pair of backslashes. It brings a vacant
    // element of array when being split by single slack.
    data.split(|&b| b == '\\' as u8)
        .take_while(|chunk| chunk.len() > 0)
        .try_for_each(|chunk| {
            std::str::from_utf8(&chunk)
                .map(|label| labels.push(label.to_string()))
                .map_err(|err| err.to_string())
        })
}

const NOTIFY_RX_CFG_CHG: u32 = 0x00000001;
const NOTIFY_TX_CFG_CHG: u32 = 0x00000002;
const NOTIFY_LOCK_CHG: u32 = 0x00000010;
const NOTIFY_CLOCK_ACCEPTED: u32 = 0x00000020;
const NOTIFY_EXT_STATUS: u32 = 0x00000040;

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn label_serdes() {
        let label = "label-0";

        let mut raw = vec![0u8; 20];
        serialize_label(&label, &mut raw).unwrap();

        let mut l = String::new();
        deserialize_label(&mut l, &raw).unwrap();

        assert_eq!(label, l);
    }

    #[test]
    fn labels_serdes() {
        let labels: Vec<String> = (0..10).map(|num| format!("label-{}", num)).collect();

        let mut raw = vec![0u8; 100];
        serialize_labels(&labels, &mut raw).unwrap();

        let mut l = Vec::new();
        deserialize_labels(&mut l, &raw).unwrap();

        assert_eq!(labels, l);
    }

    #[test]
    fn sections_serdes() {
        let raw = [
            0x00, 0x00, 0x00, 0x0a, 0x00, 0x00, 0x00, 0x5f, 0x00, 0x00, 0x00, 0x69, 0x00, 0x00,
            0x00, 0x8e, 0x00, 0x00, 0x00, 0xf7, 0x00, 0x00, 0x01, 0x1a, 0x00, 0x00, 0x02, 0x11,
            0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        let mut params = GeneralSections::default();
        deserialize_general_sections(&mut params, &raw).unwrap();

        assert_eq!(params.global.offset, 0x28);
        assert_eq!(params.global.size, 0x17c);
        assert_eq!(params.tx_stream_format.offset, 0x1a4);
        assert_eq!(params.tx_stream_format.size, 0x238);
        assert_eq!(params.rx_stream_format.offset, 0x3dc);
        assert_eq!(params.rx_stream_format.size, 0x468);
        assert_eq!(params.ext_sync.offset, 0x844);
        assert_eq!(params.ext_sync.size, 0x10);
        assert_eq!(params.reserved.offset, 0);
        assert_eq!(params.reserved.size, 0);

        let mut r = vec![0u8; raw.len()];
        serialize_general_sections(&params, &mut r).unwrap();

        assert_eq!(r, raw);
    }

    // For an issue filed at https://github.com/alsa-project/snd-firewire-ctl-services/issues/200.
    #[test]
    fn deserialize_issue_200() {
        let raw_0: &[u8; 256] = &[
            0x5c, 0x31, 0x4e, 0x49, 0x5c, 0x32, 0x4e, 0x49, 0x5c, 0x33, 0x4e, 0x49, 0x5c, 0x34,
            0x4e, 0x49, 0x5c, 0x35, 0x4e, 0x49, 0x5c, 0x36, 0x4e, 0x49, 0x5c, 0x37, 0x4e, 0x49,
            0x5c, 0x38, 0x4e, 0x49, 0x54, 0x41, 0x44, 0x41, 0x44, 0x41, 0x5c, 0x31, 0x5c, 0x32,
            0x54, 0x41, 0x54, 0x41, 0x44, 0x41, 0x44, 0x41, 0x5c, 0x33, 0x5c, 0x34, 0x54, 0x41,
            0x54, 0x41, 0x44, 0x41, 0x44, 0x41, 0x5c, 0x35, 0x5c, 0x36, 0x54, 0x41, 0x54, 0x41,
            0x44, 0x41, 0x44, 0x41, 0x5c, 0x37, 0x5c, 0x38, 0x54, 0x41, 0x00, 0xad, 0x00, 0x5c,
            0x7c, 0x91, 0x02, 0x02, 0x00, 0x00, 0x00, 0x05, 0x00, 0xad, 0x07, 0x78, 0x00, 0xad,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x12, 0xe4, 0x64, 0x00, 0xdc, 0x7d, 0x60,
            0x00, 0x12, 0xe6, 0xa8, 0x7c, 0x90, 0xe9, 0x00, 0x7c, 0x91, 0x02, 0x08, 0xff, 0xff,
            0xff, 0xff, 0x00, 0x00, 0x0f, 0xa0, 0x7c, 0x91, 0x10, 0x66, 0x7c, 0x91, 0x01, 0xbb,
            0x7c, 0x91, 0x00, 0xa4, 0x00, 0x00, 0x00, 0x1c, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x46, 0x15, 0xc3, 0x00, 0x00, 0x00, 0x00, 0x33, 0x5c,
            0x36, 0xee, 0x00, 0xdc, 0x7d, 0x60, 0x00, 0xd1, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x12, 0xe4, 0xe4, 0x00, 0x00, 0x00, 0x00, 0x7c, 0x91,
            0x00, 0x98, 0x00, 0x14, 0xbd, 0xa8, 0x00, 0xdc, 0xa2, 0x40, 0x7c, 0x91, 0x00, 0x21,
            0x00, 0x00, 0x00, 0x00, 0x7c, 0x91, 0x00, 0x3d, 0x00, 0xad, 0x03, 0xd0, 0x00, 0xd1,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];
        let labels_0: &[&str] = &[
            "IN1", "IN2", "IN3", "IN4", "IN5", "IN6", "IN7", "IN8", "ADAT1", "ADAT2", "ADAT3",
            "ADAT4", "ADAT5", "ADAT6", "ADAT7", "ADAT8",
        ];
        let raw_1: &[u8; 256] = &[
            0x31, 0x57, 0x41, 0x44, 0x57, 0x41, 0x44, 0x5c, 0x41, 0x44, 0x5c, 0x32, 0x44, 0x5c,
            0x33, 0x57, 0x5c, 0x34, 0x57, 0x41, 0x35, 0x57, 0x41, 0x44, 0x57, 0x41, 0x44, 0x5c,
            0x41, 0x44, 0x5c, 0x36, 0x44, 0x5c, 0x37, 0x57, 0x5c, 0x38, 0x57, 0x41, 0x39, 0x57,
            0x41, 0x44, 0x57, 0x41, 0x44, 0x5c, 0x44, 0x5c, 0x30, 0x31, 0x31, 0x31, 0x57, 0x41,
            0x57, 0x41, 0x44, 0x5c, 0x44, 0x5c, 0x32, 0x31, 0x33, 0x31, 0x57, 0x41, 0x57, 0x41,
            0x44, 0x5c, 0x44, 0x5c, 0x34, 0x31, 0x35, 0x31, 0x57, 0x41, 0x57, 0x41, 0x44, 0x5c,
            0x5c, 0x5c, 0x36, 0x31, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];
        let labels_1: &[&str] = &[
            "DAW1", "DAW2", "DAW3", "DAW4", "DAW5", "DAW6", "DAW7", "DAW8", "DAW9", "DAW10",
            "DAW11", "DAW12", "DAW13", "DAW14", "DAW15", "DAW16",
        ];
        let raw_2: &[u8; 256] = &[
            0x31, 0x57, 0x41, 0x44, 0x41, 0x44, 0x5c, 0x37, 0x5c, 0x38, 0x31, 0x57, 0x00, 0x00,
            0x00, 0x5c, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];
        let labels_2: &[&str] = &["DAW17", "DAW18"];
        let raw_3: &[u8; 256] = &[
            0x5c, 0x31, 0x4e, 0x49, 0x5c, 0x32, 0x4e, 0x49, 0x5c, 0x33, 0x4e, 0x49, 0x5c, 0x34,
            0x4e, 0x49, 0x5c, 0x35, 0x4e, 0x49, 0x5c, 0x36, 0x4e, 0x49, 0x5c, 0x37, 0x4e, 0x49,
            0x5c, 0x38, 0x4e, 0x49, 0x54, 0x41, 0x44, 0x41, 0x44, 0x41, 0x5c, 0x31, 0x5c, 0x32,
            0x54, 0x41, 0x54, 0x41, 0x44, 0x41, 0x44, 0x41, 0x5c, 0x33, 0x5c, 0x34, 0x54, 0x41,
            0x54, 0x41, 0x44, 0x41, 0x44, 0x41, 0x5c, 0x35, 0x5c, 0x36, 0x54, 0x41, 0x54, 0x41,
            0x44, 0x41, 0x44, 0x41, 0x5c, 0x37, 0x5c, 0x38, 0x54, 0x41, 0x00, 0xad, 0x00, 0x5c,
            0x7c, 0x91, 0x02, 0x02, 0x00, 0x00, 0x00, 0x05, 0x00, 0xad, 0x07, 0x78, 0x00, 0xad,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x12, 0xe4, 0x64, 0x00, 0xdc, 0x7d, 0x60,
            0x00, 0x12, 0xe6, 0xa8, 0x7c, 0x90, 0xe9, 0x00, 0x7c, 0x91, 0x02, 0x08, 0xff, 0xff,
            0xff, 0xff, 0x00, 0x00, 0x0f, 0xa0, 0x7c, 0x91, 0x10, 0x66, 0x7c, 0x91, 0x01, 0xbb,
            0x7c, 0x91, 0x00, 0xa4, 0x00, 0x00, 0x00, 0x1c, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x46, 0x15, 0xc3, 0x00, 0x00, 0x00, 0x00, 0x33, 0x5c,
            0x36, 0xee, 0x00, 0xdc, 0x7d, 0x60, 0x00, 0xd1, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x12, 0xe4, 0xe4, 0x00, 0x00, 0x00, 0x00, 0x7c, 0x91,
            0x00, 0x98, 0x00, 0x14, 0xbd, 0xa8, 0x00, 0xdc, 0xa2, 0x40, 0x7c, 0x91, 0x00, 0x21,
            0x00, 0x00, 0x00, 0x00, 0x7c, 0x91, 0x00, 0x3d, 0x00, 0xad, 0x03, 0xd0, 0x00, 0xd1,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];
        let labels_3: &[&str] = &[
            "IN1", "IN2", "IN3", "IN4", "IN5", "IN6", "IN7", "IN8", "ADAT1", "ADAT2", "ADAT3",
            "ADAT4", "ADAT5", "ADAT6", "ADAT7", "ADAT8",
        ];
        let raw_4: &[u8; 256] = &[
            0x5c, 0x31, 0x4e, 0x49, 0x5c, 0x32, 0x4e, 0x49, 0x5c, 0x33, 0x4e, 0x49, 0x5c, 0x34,
            0x4e, 0x49, 0x5c, 0x35, 0x4e, 0x49, 0x5c, 0x36, 0x4e, 0x49, 0x5c, 0x37, 0x4e, 0x49,
            0x5c, 0x38, 0x4e, 0x49, 0x54, 0x41, 0x44, 0x41, 0x44, 0x41, 0x5c, 0x31, 0x5c, 0x32,
            0x54, 0x41, 0x54, 0x41, 0x44, 0x41, 0x44, 0x41, 0x5c, 0x33, 0x5c, 0x34, 0x54, 0x41,
            0x54, 0x41, 0x44, 0x41, 0x44, 0x41, 0x5c, 0x35, 0x5c, 0x36, 0x54, 0x41, 0x54, 0x41,
            0x44, 0x41, 0x44, 0x41, 0x5c, 0x37, 0x5c, 0x38, 0x54, 0x41, 0x00, 0x00, 0x00, 0x5c,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];
        let labels_4: &[&str] = &[
            "IN1", "IN2", "IN3", "IN4", "IN5", "IN6", "IN7", "IN8", "ADAT1", "ADAT2", "ADAT3",
            "ADAT4", "ADAT5", "ADAT6", "ADAT7", "ADAT8",
        ];
        let raw_5: &[u8; 256] = &[
            0x33, 0x53, 0x45, 0x41, 0x45, 0x41, 0x5c, 0x34, 0x5c, 0x36, 0x35, 0x53, 0x37, 0x53,
            0x45, 0x41, 0x45, 0x41, 0x5c, 0x38, 0x5c, 0x32, 0x31, 0x53, 0x5f, 0x53, 0x45, 0x41,
            0x5c, 0x59, 0x4e, 0x41, 0x54, 0x41, 0x44, 0x41, 0x41, 0x44, 0x41, 0x5c, 0x55, 0x41,
            0x5f, 0x54, 0x6f, 0x57, 0x5c, 0x58, 0x43, 0x20, 0x64, 0x72, 0x6b, 0x63, 0x6f, 0x6c,
            0x75, 0x6e, 0x55, 0x5c, 0x5c, 0x64, 0x65, 0x73, 0x73, 0x75, 0x6e, 0x55, 0x55, 0x5c,
            0x64, 0x65, 0x65, 0x73, 0x75, 0x6e, 0x6e, 0x55, 0x5c, 0x64, 0x64, 0x65, 0x73, 0x75,
            0x74, 0x6e, 0x49, 0x5c, 0x61, 0x6e, 0x72, 0x65, 0x00, 0x5c, 0x5c, 0x6c, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x7c, 0x91, 0x00, 0x3d, 0x00, 0xad, 0x03, 0xd0, 0x00, 0xd1,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x7c, 0x91, 0x00, 0x3d, 0x00, 0xad, 0x03, 0xd0,
            0x00, 0xd1, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x7c, 0x91, 0x00, 0x3d, 0x00, 0xad,
            0x03, 0xd0, 0x00, 0xd1, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x7c, 0x91, 0x00, 0x3d,
            0x00, 0xad, 0x03, 0xd0, 0x00, 0xd1, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x7c, 0x91,
            0x00, 0x3d, 0x00, 0xad, 0x03, 0xd0, 0x00, 0xd1, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x7c, 0x91, 0x00, 0x3d, 0x00, 0xad, 0x03, 0xd0, 0x00, 0xd1, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x7c, 0x91, 0x00, 0x3d, 0x00, 0xad, 0x03, 0xd0, 0x00, 0xd1, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x7c, 0x91, 0x00, 0x3d, 0x00, 0xad, 0x03, 0xd0, 0x00, 0xd1,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x7c, 0x91, 0x00, 0x3d, 0x00, 0xad, 0x03, 0xd0,
            0x00, 0xd1, 0x00, 0x00,
        ];
        let labels_5: &[&str] = &[
            "AES34",
            "AES56",
            "AES78",
            "AES12",
            "AES_ANY",
            "ADAT",
            "ADAT_AUX",
            "Word Clock",
            "Unused",
            "Unused",
            "Unused",
            "Unused",
            "Internal",
        ];

        [
            (raw_0, labels_0),
            (raw_1, labels_1),
            (raw_2, labels_2),
            (raw_3, labels_3),
            (raw_4, labels_4),
            (raw_5, labels_5),
        ]
        .iter()
        .for_each(|&(raw, expected)| {
            let mut result = Vec::new();
            deserialize_labels(&mut result, raw).unwrap();
            assert_eq!(result, expected);
        });
    }
}
