// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Application section in protocol extension defined by TCAT for ASICs of DICE.
//!
//! The module includes structure, enumeration, and trait and its implementation for application
//! section in protocol extension defined by TCAT for ASICs of DICE.

use super::*;

/// The structure for protocol implementation of application section.
#[derive(Default)]
pub struct ApplSectionProtocol;

impl ApplSectionProtocol {
    pub fn read_appl_data(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        offset: usize,
        frames: &mut [u8],
        timeout_ms: u32
    ) -> Result<(), Error> {
        ProtocolExtension::read(
            req,
            node,
            sections.application.offset + offset,
            frames,
            timeout_ms,
        )
        .map_err(|e| Error::new(ProtocolExtensionError::Appl, &e.to_string()))
    }

    pub fn write_appl_data(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        offset: usize,
        frames: &mut [u8],
        timeout_ms: u32
    ) -> Result<(), Error> {
        ProtocolExtension::write(
            req,
            node,
            sections.application.offset + offset,
            frames,
            timeout_ms,
        )
        .map_err(|e| Error::new(ProtocolExtensionError::Appl, &e.to_string()))
    }
}
