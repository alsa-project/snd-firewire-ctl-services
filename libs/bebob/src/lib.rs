// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
pub mod runtime;

mod extensions;
mod common_ctls;

mod model;

mod apogee;
mod maudio;
mod behringer;
mod stanton;
mod esi;

use glib::Error;
use ta1394::{Ta1394Avc, Ta1394AvcError, AvcCmdType, AvcAddr, AvcRespCode};
use ta1394::{AvcOp, AvcControl};
use ta1394::general::{InputPlugSignalFormat, OutputPlugSignalFormat};
use ta1394::ccm::SignalSource;

#[derive(Default, Debug)]
pub struct BebobAvc{
    pub fcp: hinawa::FwFcp,
    pub company_id: [u8;3],
}

impl BebobAvc {
    pub fn new() -> Self {
        BebobAvc{
            fcp: hinawa::FwFcp::new(),
            company_id: [0;3],
        }
    }
}

impl AsRef<hinawa::FwFcp> for BebobAvc {
    fn as_ref(&self) -> &hinawa::FwFcp {
        &self.fcp
    }
}

impl Ta1394Avc for BebobAvc {
    fn control<O: AvcOp + AvcControl>(&self, addr: &AvcAddr, op: &mut O, timeout_ms: u32) -> Result<(), Error> {
        let mut operands = Vec::new();
        AvcControl::build_operands(op, addr, &mut operands)?;
        let (rcode, operands) = self.trx(AvcCmdType::Control, addr, O::OPCODE, &operands, timeout_ms)?;
        let unexpected = match O::OPCODE {
            InputPlugSignalFormat::OPCODE |
            OutputPlugSignalFormat::OPCODE |
            SignalSource::OPCODE => {
                // NOTE: quirk.
                rcode == AvcRespCode::Accepted || rcode == AvcRespCode::Reserved(0x00)
            }
            _ => {
                rcode == AvcRespCode::Accepted
            }
        };
        if !unexpected {
            let label = format!("Unexpected response code for control opcode {}: {:?}", O::OPCODE, rcode);
            Err(Error::new(Ta1394AvcError::UnexpectedRespCode, &label))
        } else {
            AvcControl::parse_operands(op, addr, &operands)
        }
    }
}
