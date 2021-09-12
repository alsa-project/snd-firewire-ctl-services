// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Command section in protocol extension defined by TCAT for ASICs of DICE.
//!
//! The module includes structure, enumeration, and trait and its implementation for command
//! section in protocol extension defined by TCAT for ASICs of DICE.
use super::{*, caps_section::*};

use crate::tcat::global_section::ClockRate;

use std::convert::TryFrom;

/// The enumeration to represent the mode of sampling transfer frequency.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RateMode {
    Low,
    Middle,
    High,
}

impl Default for RateMode {
    fn default() -> Self {
        Self::Low
    }
}

impl std::fmt::Display for RateMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            RateMode::Low => "low",
            RateMode::Middle => "middle",
            RateMode::High => "high",
        };
        write!(f, "{}", label)
    }
}

impl From<ClockRate> for RateMode {
    fn from(rate: ClockRate) -> Self {
        match rate {
            ClockRate::R32000 |
            ClockRate::R44100 |
            ClockRate::R48000 |
            ClockRate::AnyLow |
            ClockRate::None |
            ClockRate::Reserved(_) => RateMode::Low,
            ClockRate::R88200 |
            ClockRate::R96000 |
            ClockRate::AnyMid => RateMode::Middle,
            ClockRate::R176400 |
            ClockRate::R192000 |
            ClockRate::AnyHigh => RateMode::High,
        }
    }
}

impl TryFrom<u32> for RateMode {
    type Error = Error;

    fn try_from(rate: u32) -> Result<Self, Self::Error> {
        ClockRate::try_from(rate)
            .map(|clock_rate| RateMode::from(clock_rate))
    }
}

/// The enumeration to represent opcode of command.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Opcode {
    NoOp,
    LoadRouter(RateMode),
    LoadStreamConfig(RateMode),
    LoadRouterStreamConfig(RateMode),
    LoadConfigFromFlash,
    StoreConfigToFlash,
}

impl From<Opcode> for u16 {
    fn from(code: Opcode) -> u16 {
        match code {
            Opcode::NoOp => 0x00,
            Opcode::LoadRouter(_) => 0x01,
            Opcode::LoadStreamConfig(_) => 0x02,
            Opcode::LoadRouterStreamConfig(_) => 0x03,
            Opcode::LoadConfigFromFlash => 0x04,
            Opcode::StoreConfigToFlash => 0x05,
        }
    }
}

/// The structure for protocol implementation of command section.
#[derive(Default)]
pub struct CmdSectionProtocol;

impl CmdSectionProtocol {
    const OPCODE_OFFSET: usize = 0x00;
    const RETURN_OFFSET: usize = 0x04;

    const EXECUTE: u8 = 0x80;

    pub fn initiate(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        caps: &ExtensionCaps,
        opcode: Opcode,
        timeout_ms: u32
    ) -> Result<u32, Error> {
        if let Opcode::LoadRouter(_) = opcode {
            if caps.mixer.is_readonly {
                Err(Error::new(ProtocolExtensionError::Cmd, "Router configuration is immutable"))?
            }
        } else if let Opcode::LoadStreamConfig(_) = opcode {
            if !caps.general.dynamic_stream_format {
                Err(Error::new(ProtocolExtensionError::Cmd, "Stream format configuration is immutable"))?
            }
        } else if let Opcode::LoadRouterStreamConfig(_) = opcode {
            if caps.mixer.is_readonly && !caps.general.dynamic_stream_format {
                Err(Error::new(ProtocolExtensionError::Cmd, "Any configuration is immutable"))?
            }
        } else if opcode == Opcode::LoadConfigFromFlash {
            if !caps.general.storage_avail {
                Err(Error::new(ProtocolExtensionError::Cmd, "Storage is not available"))?
            }
        } else if opcode == Opcode::StoreConfigToFlash {
            if !caps.general.storage_avail {
                Err(Error::new(ProtocolExtensionError::Cmd, "Storage is not available"))?
            }
        }

        let mut data = [0;4];
        data[2..4].copy_from_slice(&u16::from(opcode).to_be_bytes());
        data[1] = match opcode {
            Opcode::LoadRouter(r) |
            Opcode::LoadStreamConfig(r) |
            Opcode::LoadRouterStreamConfig(r) => match r {
                RateMode::Low => 1,
                RateMode::Middle => 2,
                RateMode::High => 4,
            }
            _ => 0,
        };
        data[0] = Self::EXECUTE;
        ProtocolExtension::write(
            req,
            node,
            sections.cmd.offset + Self::OPCODE_OFFSET,
            &mut data,
            timeout_ms
        )
            .map_err(|e| Error::new(ProtocolExtensionError::Cmd, &e.to_string()))?;

        let mut count = 0;
        while count < 10 {
            std::thread::sleep(std::time::Duration::from_millis(50));

            ProtocolExtension::read(
                req,
                node,
                sections.cmd.offset,
                &mut data,
                timeout_ms
            )
                .map_err(|e| Error::new(ProtocolExtensionError::Cmd, &e.to_string()))?;

            if (data[0] & Self::EXECUTE) != Self::EXECUTE {
                ProtocolExtension::read(
                    req,
                    node,
                    sections.cmd.offset + Self::RETURN_OFFSET,
                    &mut data,
                    timeout_ms
                )
                    .map_err(|e| Error::new(ProtocolExtensionError::Cmd, &e.to_string()))?;
                return Ok(u32::from_be_bytes(data));
            }
            count += 1;
        }

        Err(Error::new(ProtocolExtensionError::Cmd, "Operation timeout."))
    }
}
