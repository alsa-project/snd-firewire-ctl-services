// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Standalone section in protocol extension defined by TCAT for ASICs of DICE.
//!
//! The module includes structure, enumeration, and trait and its implementation for standalone
//! section in protocol extension defined by TCAT for ASICs of DICE.

use super::{global_section::*, *};

/// Parameter of ADAT input/output.
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

/// Mode of word clock input/output.
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

/// Rate of word clock input/output by enumerator and denominator.
#[derive(Default, Debug, Clone, Copy, Eq, PartialEq)]
pub struct WordClockRate {
    pub numerator: u16,
    pub denominator: u16,
}

/// Parameter of word clock input/output.
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

/// Parameters in standalone configuration section.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct StandaloneParameters {
    /// Source of sampling clock.
    pub clock_source: ClockSource,
    /// Mode of AES input at high rate.
    pub aes_high_rate: bool,
    /// Mode of ADAT input for supported rates.
    pub adat_mode: AdatParam,
    /// Mode of word clock input.
    pub word_clock_param: WordClockParam,
    /// Internally generated sampling clock.
    pub internal_rate: ClockRate,
}

/// Protocol implementation of standalone section.
#[derive(Default)]
pub struct StandaloneSectionProtocol;

const MIN_SIZE: usize = 20;

fn serialize(params: &StandaloneParameters, raw: &mut [u8]) -> Result<(), String> {
    assert!(raw.len() >= MIN_SIZE);

    (u8::from(params.clock_source) as u32).build_quadlet(&mut raw[..4]);

    (params.aes_high_rate as u32).build_quadlet(&mut raw[4..8]);

    (match params.adat_mode {
        AdatParam::Normal => 0x00u32,
        AdatParam::SMUX2 => 0x01,
        AdatParam::SMUX4 => 0x02,
        AdatParam::Auto => 0x03,
    })
    .build_quadlet(&mut raw[8..12]);

    let mut val = match params.word_clock_param.mode {
        WordClockMode::Normal => 0x00u32,
        WordClockMode::Low => 0x01,
        WordClockMode::Middle => 0x02,
        WordClockMode::High => 0x03,
    };
    if params.word_clock_param.rate.numerator < 1 || params.word_clock_param.rate.denominator < 1 {
        let msg = format!(
            "Invalid parameters for rate of word clock: {} / {}",
            params.word_clock_param.rate.numerator, params.word_clock_param.rate.denominator
        );
        Err(msg)?;
    }
    val |= ((params.word_clock_param.rate.numerator as u32) - 1) << 4;
    val |= ((params.word_clock_param.rate.denominator as u32) - 1) << 16;
    val.build_quadlet(&mut raw[12..16]);

    (u8::from(params.internal_rate) as u32).build_quadlet(&mut raw[16..20]);

    Ok(())
}

fn deserialize(params: &mut StandaloneParameters, raw: &[u8]) -> Result<(), String> {
    assert!(raw.len() >= MIN_SIZE);

    let mut val = 0u32;

    val.parse_quadlet(&raw[..4]);
    params.clock_source = ClockSource::from(val as u8);

    val.parse_quadlet(&raw[4..8]);
    params.aes_high_rate = val > 0;

    val.parse_quadlet(&raw[8..12]);
    params.adat_mode = match val {
        0x01 => AdatParam::SMUX2,
        0x02 => AdatParam::SMUX4,
        0x03 => AdatParam::Auto,
        _ => AdatParam::Normal,
    };

    val.parse_quadlet(&raw[12..16]);
    params.word_clock_param.mode = match val & 0x03 {
        0x01 => WordClockMode::Low,
        0x02 => WordClockMode::Middle,
        0x03 => WordClockMode::High,
        _ => WordClockMode::Normal,
    };
    params.word_clock_param.rate.numerator = 1 + ((val >> 4) & 0x0fff) as u16;
    params.word_clock_param.rate.denominator = 1 + ((val >> 16) & 0xffff) as u16;

    val.parse_quadlet(&raw[16..20]);
    params.internal_rate = ClockRate::from(val as u8);

    Ok(())
}

impl StandaloneSectionProtocol {
    const CLK_SRC_OFFSET: usize = 0x00;
    const AES_CFG_OFFSET: usize = 0x04;
    const ADAT_CFG_OFFSET: usize = 0x08;
    const WC_CFG_OFFSET: usize = 0x0c;
    const INTERNAL_CFG_OFFSET: usize = 0x10;

    pub fn cache_standalone_params(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        params: &mut StandaloneParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut raw = vec![0; sections.standalone.size];
        extension_read(req, node, sections.standalone.offset, &mut raw, timeout_ms)
            .map_err(|e| Error::new(ProtocolExtensionError::Standalone, &e.to_string()))?;

        deserialize(params, &raw)
            .map_err(|msg| Error::new(ProtocolExtensionError::Standalone, &msg))
    }

    pub fn update_standalone_params(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        params: &StandaloneParameters,
        prev: &mut StandaloneParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut new = vec![0; sections.standalone.size];
        serialize(params, &mut new)
            .map_err(|e| Error::new(ProtocolExtensionError::Standalone, &e.to_string()))?;

        let mut old = vec![0; sections.standalone.size];
        serialize(params, &mut old)
            .map_err(|e| Error::new(ProtocolExtensionError::Standalone, &e.to_string()))?;

        (0..sections.standalone.size)
            .step_by(4)
            .try_for_each(|pos| {
                if new[pos] != old[pos] {
                    extension_read(
                        req,
                        node,
                        sections.standalone.offset + pos,
                        &mut new[pos..(pos + 4)],
                        timeout_ms,
                    )
                } else {
                    Ok(())
                }
            })?;

        deserialize(prev, &new)
            .map_err(|e| Error::new(ProtocolExtensionError::Standalone, &e.to_string()))
    }

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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn standalone_params_serdes() {
        let params = StandaloneParameters {
            clock_source: ClockSource::Tdif,
            aes_high_rate: true,
            adat_mode: AdatParam::SMUX4,
            word_clock_param: WordClockParam {
                mode: WordClockMode::Middle,
                rate: WordClockRate {
                    numerator: 12,
                    denominator: 7,
                },
            },
            internal_rate: ClockRate::R88200,
        };

        let mut raw = [0u8; MIN_SIZE];
        assert!(serialize(&params, &mut raw).is_ok());

        let mut p = StandaloneParameters::default();
        assert!(deserialize(&mut p, &raw).is_ok());

        assert_eq!(params, p);
    }
}
