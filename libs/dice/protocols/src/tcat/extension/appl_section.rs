// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Application section in protocol extension defined by TCAT for ASICs of DICE.
//!
//! The module includes structure, enumeration, and trait and its implementation for application
//! section in protocol extension defined by TCAT for ASICs of DICE.

use super::*;

pub trait ApplSectionProtocol: ProtocolExtension {
    fn read_appl_data(
        &self,
        node: &mut FwNode,
        sections: &ExtensionSections,
        offset: usize,
        frames: &mut [u8],
        timeout_ms: u32
    ) -> Result<(), Error> {
        ProtocolExtension::read(self, node, sections.application.offset + offset, frames, timeout_ms)
            .map_err(|e| Error::new(ProtocolExtensionError::Appl, &e.to_string()))
    }

    fn write_appl_data(
        &self,
        node: &mut FwNode,
        sections: &ExtensionSections,
        offset: usize,
        frames: &mut [u8],
        timeout_ms: u32
    ) -> Result<(), Error> {
        ProtocolExtension::write(self, node, sections.application.offset + offset, frames, timeout_ms)
            .map_err(|e| Error::new(ProtocolExtensionError::Appl, &e.to_string()))
    }
}

impl<O: AsRef<FwReq>> ApplSectionProtocol for O {}
