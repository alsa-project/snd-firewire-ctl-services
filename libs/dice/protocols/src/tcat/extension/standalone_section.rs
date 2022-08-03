// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Standalone section in protocol extension defined by TCAT for ASICs of DICE.
//!
//! The module includes structure, enumeration, and trait and its implementation for standalone
//! section in protocol extension defined by TCAT for ASICs of DICE.

use super::{global_section::*, *};

/// The enumeration to represent parameter of ADAT input/output.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum AdatParam {
    Normal,
    SMUX2,
    SMUX4,
    Auto,
}

impl Default for AdatParam {
    fn default() -> Self {
        AdatParam::Auto
    }
}

impl std::fmt::Display for AdatParam {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            AdatParam::Normal => "normal",
            AdatParam::SMUX2 => "S/MUX-2",
            AdatParam::SMUX4 => "S/MUX-4",
            AdatParam::Auto => "auto",
        };
        write!(f, "{}", label)
    }
}

impl From<[u8; 4]> for AdatParam {
    fn from(raw: [u8; 4]) -> Self {
        match u32::from_be_bytes(raw) & 0x03 {
            0x01 => AdatParam::SMUX2,
            0x02 => AdatParam::SMUX4,
            0x03 => AdatParam::Auto,
            _ => AdatParam::Normal,
        }
    }
}

impl From<AdatParam> for [u8; 4] {
    fn from(param: AdatParam) -> Self {
        let val = match param {
            AdatParam::Normal => 0x00,
            AdatParam::SMUX2 => 0x01,
            AdatParam::SMUX4 => 0x02,
            AdatParam::Auto => 0x03,
        };
        (val as u32).to_be_bytes()
    }
}

/// The enumeration to represent mode of word clock input/output.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum WordClockMode {
    Normal,
    Low,
    Middle,
    High,
}

impl Default for WordClockMode {
    fn default() -> Self {
        WordClockMode::Normal
    }
}

impl std::fmt::Display for WordClockMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            WordClockMode::Normal => "normal",
            WordClockMode::Low => "low",
            WordClockMode::Middle => "middle",
            WordClockMode::High => "high",
        };
        write!(f, "{}", label)
    }
}

/// The structure to represent rate of word clock input/output by enumerator and denominator.
#[derive(Default, Debug, Clone, Copy, Eq, PartialEq)]
pub struct WordClockRate {
    pub numerator: u16,
    pub denominator: u16,
}

/// The structure to represent parameter of word clock input/output.
#[derive(Default, Debug, Clone, Copy, Eq, PartialEq)]
pub struct WordClockParam {
    pub mode: WordClockMode,
    pub rate: WordClockRate,
}

impl From<[u8; 4]> for WordClockParam {
    fn from(raw: [u8; 4]) -> Self {
        let val = u32::from_be_bytes(raw);

        let mode = match val & 0x03 {
            0x01 => WordClockMode::Low,
            0x02 => WordClockMode::Middle,
            0x03 => WordClockMode::High,
            _ => WordClockMode::Normal,
        };

        let numerator = 1 + ((val >> 4) & 0x0fff) as u16;
        let denominator = 1 + ((val >> 16) & 0xffff) as u16;

        WordClockParam {
            mode,
            rate: WordClockRate {
                numerator,
                denominator,
            },
        }
    }
}

impl From<WordClockParam> for [u8; 4] {
    fn from(param: WordClockParam) -> Self {
        let mut val = match param.mode {
            WordClockMode::Normal => 0x00,
            WordClockMode::Low => 0x01,
            WordClockMode::Middle => 0x02,
            WordClockMode::High => 0x03,
        };
        val |= ((param.rate.numerator as u32) - 1) << 4;
        val |= ((param.rate.denominator as u32) - 1) << 16;
        val.to_be_bytes()
    }
}

/// The structure for protocol implementation of standalone section.
#[derive(Default)]
pub struct StandaloneSectionProtocol;

impl StandaloneSectionProtocol {
    const CLK_SRC_OFFSET: usize = 0x00;
    const AES_CFG_OFFSET: usize = 0x04;
    const ADAT_CFG_OFFSET: usize = 0x08;
    const WC_CFG_OFFSET: usize = 0x0c;
    const INTERNAL_CFG_OFFSET: usize = 0x10;

    pub fn read_standalone_clock_source(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        timeout_ms: u32,
    ) -> Result<ClockSource, Error> {
        let mut quadlet = [0; 4];
        extension_read(
            req,
            node,
            sections.standalone.offset + Self::CLK_SRC_OFFSET,
            &mut quadlet,
            timeout_ms,
        )
        .map_err(|e| Error::new(ProtocolExtensionError::Standalone, &e.to_string()))
        .map(|_| ClockSource::from(u32::from_be_bytes(quadlet) as u8))
    }

    pub fn write_standalone_clock_source(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        src: ClockSource,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut quadlet = [0; 4];
        quadlet.copy_from_slice(&(u8::from(src) as u32).to_be_bytes());
        extension_write(
            req,
            node,
            sections.standalone.offset + Self::CLK_SRC_OFFSET,
            &mut quadlet,
            timeout_ms,
        )
        .map_err(|e| Error::new(ProtocolExtensionError::Standalone, &e.to_string()))
    }

    pub fn read_standalone_aes_high_rate(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        let mut quadlet = [0; 4];
        extension_read(
            req,
            node,
            sections.standalone.offset + Self::AES_CFG_OFFSET,
            &mut quadlet,
            timeout_ms,
        )
        .map_err(|e| Error::new(ProtocolExtensionError::Standalone, &e.to_string()))
        .map(|_| u32::from_be_bytes(quadlet) > 0)
    }

    pub fn write_standalone_aes_high_rate(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        enable: bool,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut quadlet = [0; 4];
        quadlet.copy_from_slice(&(enable as u32).to_be_bytes());
        extension_write(
            req,
            node,
            sections.standalone.offset + Self::AES_CFG_OFFSET,
            &mut quadlet,
            timeout_ms,
        )
        .map_err(|e| Error::new(ProtocolExtensionError::Standalone, &e.to_string()))
    }

    pub fn read_standalone_adat_mode(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        timeout_ms: u32,
    ) -> Result<AdatParam, Error> {
        let mut quadlet = [0; 4];
        extension_read(
            req,
            node,
            sections.standalone.offset + Self::ADAT_CFG_OFFSET,
            &mut quadlet,
            timeout_ms,
        )
        .map_err(|e| Error::new(ProtocolExtensionError::Standalone, &e.to_string()))
        .map(|_| AdatParam::from(quadlet))
    }

    pub fn write_standalone_adat_mode(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        param: AdatParam,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        extension_write(
            req,
            node,
            sections.standalone.offset + Self::ADAT_CFG_OFFSET,
            &mut Into::<[u8; 4]>::into(param),
            timeout_ms,
        )
        .map_err(|e| Error::new(ProtocolExtensionError::Standalone, &e.to_string()))
    }

    pub fn read_standalone_word_clock_param(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        timeout_ms: u32,
    ) -> Result<WordClockParam, Error> {
        let mut quadlet = [0; 4];
        extension_read(
            req,
            node,
            sections.standalone.offset + Self::WC_CFG_OFFSET,
            &mut quadlet,
            timeout_ms,
        )
        .map_err(|e| Error::new(ProtocolExtensionError::Standalone, &e.to_string()))
        .map(|_| WordClockParam::from(quadlet))
    }

    pub fn write_standalone_word_clock_param(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        param: WordClockParam,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut quadlet = [0; 4];
        quadlet.copy_from_slice(&Into::<[u8; 4]>::into(param)[..4]);
        extension_write(
            req,
            node,
            sections.standalone.offset + Self::WC_CFG_OFFSET,
            &mut quadlet,
            timeout_ms,
        )
        .map_err(|e| Error::new(ProtocolExtensionError::Standalone, &e.to_string()))
    }

    pub fn read_standalone_internal_rate(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        timeout_ms: u32,
    ) -> Result<ClockRate, Error> {
        let mut quadlet = [0; 4];
        extension_read(
            req,
            node,
            sections.standalone.offset + Self::INTERNAL_CFG_OFFSET,
            &mut quadlet,
            timeout_ms,
        )
        .map_err(|e| Error::new(ProtocolExtensionError::Standalone, &e.to_string()))
        .map(|_| ClockRate::from(u32::from_be_bytes(quadlet) as u8))
    }

    pub fn write_standalone_internal_rate(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        rate: ClockRate,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut quadlet = [0; 4];
        quadlet.copy_from_slice(&(u8::from(rate) as u32).to_be_bytes());
        extension_write(
            req,
            node,
            sections.standalone.offset + Self::INTERNAL_CFG_OFFSET,
            &mut quadlet,
            timeout_ms,
        )
        .map_err(|e| Error::new(ProtocolExtensionError::Standalone, &e.to_string()))
    }
}
