// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocols defined for BridgeCo. Enhanced Break Out Box (BeBoB) solution.
//!
//! The crate includes various kind of protocols defined by BridgeCo. AG and application vendors
//! for DM1000, DM1100, and DM1500 ASICs with its BridgeCo. Enhanced Break Out Box (BeBoB) solution.

pub mod bridgeco;

use glib::Error;

use hinawa::FwFcp;

use ta1394::{ccm::*, general::*, *};

/// The structure for AV/C transaction helper with quirks specific to BeBoB solution.
#[derive(Default, Debug)]
pub struct BebobAvc(FwFcp);

impl AsRef<FwFcp> for BebobAvc {
    fn as_ref(&self) -> &FwFcp {
        &self.0
    }
}

impl Ta1394Avc for BebobAvc {
    fn control<O: AvcOp + AvcControl>(
        &self,
        addr: &AvcAddr,
        op: &mut O,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut operands = Vec::new();
        AvcControl::build_operands(op, addr, &mut operands)?;
        let (rcode, operands) = self.trx(
            AvcCmdType::Control,
            addr,
            O::OPCODE,
            &mut operands,
            timeout_ms,
        )?;
        let expected = match O::OPCODE {
            InputPlugSignalFormat::OPCODE
            | OutputPlugSignalFormat::OPCODE
            | SignalSource::OPCODE => {
                // NOTE: quirk.
                rcode == AvcRespCode::Accepted || rcode == AvcRespCode::Reserved(0x00)
            }
            _ => rcode == AvcRespCode::Accepted,
        };
        if !expected {
            let label = format!(
                "Unexpected response code for control opcode {}: {:?}",
                O::OPCODE,
                rcode
            );
            Err(Error::new(Ta1394AvcError::UnexpectedRespCode, &label))
        } else {
            AvcControl::parse_operands(op, addr, &operands)
        }
    }
}
