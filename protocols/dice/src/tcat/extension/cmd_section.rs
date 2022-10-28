// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Command section in protocol extension defined by TCAT for ASICs of DICE.
//!
//! The module includes structure, enumeration, and trait and its implementation for command
//! section in protocol extension defined by TCAT for ASICs of DICE.
use super::{caps_section::*, *};

/// Mode of sampling transfer frequency.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum RateMode {
    /// up to 48.0 kHz.
    Low,
    /// up to 96.0 kHz.
    Middle,
    /// up to 192.0 kHz.
    High,
}

impl Default for RateMode {
    fn default() -> Self {
        Self::Low
    }
}

impl RateMode {
    const LOW_FLAG: u32 = 0x00010000;
    const MIDDLE_FLAG: u32 = 0x00020000;
    const HIGH_FLAG: u32 = 0x00040000;

    /// Conversion from sampling transfer frequency.
    pub fn from_sampling_transfer_frequency(freq: u32) -> Self {
        match freq {
            0..=48000 => Self::Low,
            48001..=96000 => Self::Middle,
            96001.. => Self::High,
        }
    }

    /// Conversion from clock rate.
    pub fn from_clock_rate(rate: ClockRate) -> Self {
        match rate {
            ClockRate::R32000
            | ClockRate::R44100
            | ClockRate::R48000
            | ClockRate::AnyLow
            | ClockRate::None
            | ClockRate::Reserved(_) => RateMode::Low,
            ClockRate::R88200 | ClockRate::R96000 | ClockRate::AnyMid => RateMode::Middle,
            ClockRate::R176400 | ClockRate::R192000 | ClockRate::AnyHigh => RateMode::High,
        }
    }
}

/// Operation code of command.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Opcode {
    /// No operation.
    NoOp,
    /// Load router configuration to router section at given rate.
    LoadRouter(RateMode),
    /// Load stream format configuration to stream format section at given rate.
    LoadStreamConfig(RateMode),
    /// Load both router and stream format configurations at given rate.
    LoadRouterStreamConfig(RateMode),
    /// Load all configurations from on-board flash memory.
    LoadConfigFromFlash,
    /// Store all configurations to on-board flash memory.
    StoreConfigToFlash,
}

impl Opcode {
    const NOOP_VALUE: u16 = 0x0000;
    const LOAD_ROUTER_VALUE: u16 = 0x0001;
    const LOAD_STREAM_CONFIG_VALUE: u16 = 0x0002;
    const LOAD_ROUTER_STREAM_CONFIG_VALUE: u16 = 0x0003;
    const LOAD_FLASH_CONFIG_VALUE: u16 = 0x0004;
    const STORE_FLASH_CONFIG_VALUE: u16 = 0x0005;
}

const EXECUTE_FLAG: u32 = 0x80000000;

fn serialize_opcode(code: &Opcode, raw: &mut [u8]) {
    assert!(raw.len() >= 4);

    let mut val = match code {
        Opcode::NoOp => Opcode::NOOP_VALUE as u32,
        Opcode::LoadRouter(rate_mode)
        | Opcode::LoadStreamConfig(rate_mode)
        | Opcode::LoadRouterStreamConfig(rate_mode) => {
            let val = match code {
                Opcode::LoadRouter(_) => Opcode::LOAD_ROUTER_VALUE,
                Opcode::LoadStreamConfig(_) => Opcode::LOAD_STREAM_CONFIG_VALUE,
                Opcode::LoadRouterStreamConfig(_) => Opcode::LOAD_ROUTER_STREAM_CONFIG_VALUE,
                _ => unreachable!(),
            } as u32;

            let flag = match rate_mode {
                RateMode::Low => RateMode::LOW_FLAG,
                RateMode::Middle => RateMode::MIDDLE_FLAG,
                RateMode::High => RateMode::HIGH_FLAG,
            };
            flag | val
        }
        Opcode::LoadConfigFromFlash => Opcode::LOAD_FLASH_CONFIG_VALUE as u32,
        Opcode::StoreConfigToFlash => Opcode::STORE_FLASH_CONFIG_VALUE as u32,
    };

    val |= EXECUTE_FLAG;

    serialize_u32(&val, raw);
}

const OPCODE_OFFSET: usize = 0x00;
const RETURN_OFFSET: usize = 0x04;

/// Operation in command section of TCAT protocol extension.
pub trait TcatExtensionCommandSectionOperation: TcatExtensionOperation {
    /// Initiate command and wait for its completion.
    fn initiate(
        req: &FwReq,
        node: &FwNode,
        sections: &ExtensionSections,
        caps: &ExtensionCaps,
        opcode: Opcode,
        timeout_ms: u32,
    ) -> Result<u32, Error> {
        if let Opcode::LoadRouter(_) = opcode {
            if caps.mixer.is_readonly {
                Err(Error::new(
                    ProtocolExtensionError::Cmd,
                    "Router configuration is immutable",
                ))?
            }
        } else if let Opcode::LoadStreamConfig(_) = opcode {
            if !caps.general.dynamic_stream_format {
                Err(Error::new(
                    ProtocolExtensionError::Cmd,
                    "Stream format configuration is immutable",
                ))?
            }
        } else if let Opcode::LoadRouterStreamConfig(_) = opcode {
            if caps.mixer.is_readonly && !caps.general.dynamic_stream_format {
                Err(Error::new(
                    ProtocolExtensionError::Cmd,
                    "Any configuration is immutable",
                ))?
            }
        } else if opcode == Opcode::LoadConfigFromFlash {
            if !caps.general.storage_avail {
                Err(Error::new(
                    ProtocolExtensionError::Cmd,
                    "Storage is not available",
                ))?
            }
        } else if opcode == Opcode::StoreConfigToFlash {
            if !caps.general.storage_avail {
                Err(Error::new(
                    ProtocolExtensionError::Cmd,
                    "Storage is not available",
                ))?
            }
        }

        let mut raw = [0; 4];
        serialize_opcode(&opcode, &mut raw);
        Self::write_extension(
            req,
            node,
            &sections.cmd,
            OPCODE_OFFSET,
            &mut raw,
            timeout_ms,
        )?;

        let mut count = 0;
        while count < 10 {
            std::thread::sleep(std::time::Duration::from_millis(50));

            Self::read_extension(
                req,
                node,
                &sections.cmd,
                OPCODE_OFFSET,
                &mut raw,
                timeout_ms,
            )
            .map_err(|e| Error::new(ProtocolExtensionError::Cmd, &e.to_string()))?;

            let mut val = 0u32;
            deserialize_u32(&mut val, &raw);

            if val & EXECUTE_FLAG == 0 {
                Self::read_extension(
                    req,
                    node,
                    &sections.cmd,
                    RETURN_OFFSET,
                    &mut raw,
                    timeout_ms,
                )
                .map_err(|e| Error::new(ProtocolExtensionError::Cmd, &e.to_string()))?;
                return Ok(u32::from_be_bytes(raw));
            }
            count += 1;
        }

        Err(Error::new(
            ProtocolExtensionError::Cmd,
            "Operation timeout.",
        ))
    }
}

impl<O: TcatExtensionOperation> TcatExtensionCommandSectionOperation for O {}

/// Protocol implementation of command section.
#[derive(Default)]
pub struct CmdSectionProtocol;

impl CmdSectionProtocol {
    const OPCODE_OFFSET: usize = 0x00;
    const RETURN_OFFSET: usize = 0x04;

    pub fn initiate(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        caps: &ExtensionCaps,
        opcode: Opcode,
        timeout_ms: u32,
    ) -> Result<u32, Error> {
        if let Opcode::LoadRouter(_) = opcode {
            if caps.mixer.is_readonly {
                Err(Error::new(
                    ProtocolExtensionError::Cmd,
                    "Router configuration is immutable",
                ))?
            }
        } else if let Opcode::LoadStreamConfig(_) = opcode {
            if !caps.general.dynamic_stream_format {
                Err(Error::new(
                    ProtocolExtensionError::Cmd,
                    "Stream format configuration is immutable",
                ))?
            }
        } else if let Opcode::LoadRouterStreamConfig(_) = opcode {
            if caps.mixer.is_readonly && !caps.general.dynamic_stream_format {
                Err(Error::new(
                    ProtocolExtensionError::Cmd,
                    "Any configuration is immutable",
                ))?
            }
        } else if opcode == Opcode::LoadConfigFromFlash {
            if !caps.general.storage_avail {
                Err(Error::new(
                    ProtocolExtensionError::Cmd,
                    "Storage is not available",
                ))?
            }
        } else if opcode == Opcode::StoreConfigToFlash {
            if !caps.general.storage_avail {
                Err(Error::new(
                    ProtocolExtensionError::Cmd,
                    "Storage is not available",
                ))?
            }
        }

        let mut raw = [0; 4];
        serialize_opcode(&opcode, &mut raw);
        extension_write(
            req,
            node,
            sections.cmd.offset + Self::OPCODE_OFFSET,
            &mut raw,
            timeout_ms,
        )
        .map_err(|e| Error::new(ProtocolExtensionError::Cmd, &e.to_string()))?;

        let mut count = 0;
        while count < 10 {
            std::thread::sleep(std::time::Duration::from_millis(50));

            extension_read(req, node, sections.cmd.offset, &mut raw, timeout_ms)
                .map_err(|e| Error::new(ProtocolExtensionError::Cmd, &e.to_string()))?;

            let mut val = 0u32;
            deserialize_u32(&mut val, &raw);

            if val & EXECUTE_FLAG == 0 {
                extension_read(
                    req,
                    node,
                    sections.cmd.offset + Self::RETURN_OFFSET,
                    &mut raw,
                    timeout_ms,
                )
                .map_err(|e| Error::new(ProtocolExtensionError::Cmd, &e.to_string()))?;
                return Ok(u32::from_be_bytes(raw));
            }
            count += 1;
        }

        Err(Error::new(
            ProtocolExtensionError::Cmd,
            "Operation timeout.",
        ))
    }
}
