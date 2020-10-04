// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
pub mod unit;

mod extensions;
mod common_ctls;

mod model;

mod apogee;


use glib::Error;
use crate::ta1394::{Ta1394Avc, Ta1394AvcError, AvcCmdType, AvcAddr, AvcRespCode};
use crate::ta1394::{AvcOp, AvcStatus, AvcControl, AvcNotify};
use crate::ta1394::general::{InputPlugSignalFormat, OutputPlugSignalFormat};
use crate::ta1394::ccm::SignalSource;

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
        let opcode = op.opcode();
        let (rcode, operands) = self.trx(AvcCmdType::Control, addr, opcode, &mut operands, timeout_ms)?;
        let unexpected = match opcode {
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
            let label = format!("Unexpected response code for control opcode {}: {:?}", opcode, rcode);
            Err(Error::new(Ta1394AvcError::UnexpectedRespCode, &label))
        } else {
            AvcControl::parse_operands(op, addr, &operands)
        }
    }

    fn status<O: AvcOp + AvcStatus>(&self, addr: &AvcAddr, op: &mut O, timeout_ms: u32) -> Result<(), Error> {
        self.fcp.status(addr, op, timeout_ms)
    }

    fn specific_inquiry<O: AvcOp + AvcControl>(&self, addr: &AvcAddr, op: &mut O, timeout_ms: u32) -> Result<(), Error> {
        self.fcp.specific_inquiry(addr, op, timeout_ms)
    }

    fn notify<O: AvcOp + AvcNotify>(&self, addr: &AvcAddr, op: &mut O, timeout_ms: u32) -> Result<(), Error> {
        self.fcp.notify(addr, op, timeout_ms)
    }
}
