// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Standalone section in protocol extension defined by TCAT for ASICs of DICE.
//!
//! The module includes structure, enumeration, and trait and its implementation for standalone
//! section in protocol extension defined by TCAT for ASICs of DICE.

use super::{global_section::*, *};

// const CLK_SRC_OFFSET: usize = 0x00;
// const AES_CFG_OFFSET: usize = 0x04;
// const ADAT_CFG_OFFSET: usize = 0x08;
// const WC_CFG_OFFSET: usize = 0x0c;
// const INTERNAL_CFG_OFFSET: usize = 0x10;

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

    let mut val = 0u8;
    serialize_clock_source(&params.clock_source, &mut val)?;
    serialize_u8(&val, &mut raw[..4]);

    serialize_bool(&params.aes_high_rate, &mut raw[4..8]);

    let val = match params.adat_mode {
        AdatParam::Normal => 0x00u32,
        AdatParam::SMUX2 => 0x01,
        AdatParam::SMUX4 => 0x02,
        AdatParam::Auto => 0x03,
    };
    serialize_u32(&val, &mut raw[8..12]);

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
    serialize_u32(&val, &mut raw[12..16]);

    let mut val = 0u8;
    serialize_clock_rate(&params.internal_rate, &mut val)?;
    serialize_u8(&val, &mut raw[16..20]);

    Ok(())
}

fn deserialize(params: &mut StandaloneParameters, raw: &[u8]) -> Result<(), String> {
    assert!(raw.len() >= MIN_SIZE);

    let mut val = 0u8;
    deserialize_u8(&mut val, &raw[..4]);
    deserialize_clock_source(&mut params.clock_source, &val)?;

    deserialize_bool(&mut params.aes_high_rate, &raw[4..8]);

    let mut val = 0u32;
    deserialize_u32(&mut val, &raw[8..12]);
    params.adat_mode = match val {
        0x01 => AdatParam::SMUX2,
        0x02 => AdatParam::SMUX4,
        0x03 => AdatParam::Auto,
        _ => AdatParam::Normal,
    };

    deserialize_u32(&mut val, &raw[12..16]);
    params.word_clock_param.mode = match val & 0x03 {
        0x01 => WordClockMode::Low,
        0x02 => WordClockMode::Middle,
        0x03 => WordClockMode::High,
        _ => WordClockMode::Normal,
    };
    params.word_clock_param.rate.numerator = 1 + ((val >> 4) & 0x0fff) as u16;
    params.word_clock_param.rate.denominator = 1 + ((val >> 16) & 0xffff) as u16;

    let mut val = 0u8;
    deserialize_u8(&mut val, &raw[16..20]);
    deserialize_clock_rate(&mut params.internal_rate, &val)?;

    Ok(())
}

impl StandaloneSectionProtocol {
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
                    extension_write(
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
