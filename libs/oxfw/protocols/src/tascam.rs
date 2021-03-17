// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol defined by TASCAM for FireOne.
//!
//! The module includes protocol implementation defined by TASCAM for FireOne.

use glib::Error;

use ta1394::{Ta1394Avc, Ta1394AvcError, AvcCmdType, AvcAddr, AvcRespCode};
use ta1394::{AvcOp, AvcControl, AvcStatus, AvcNotify};
use ta1394::general::VendorDependent;

pub enum VendorCmd {
    DisplayMode,
    MessageMode,
    InputMode,
    FirmwareVersion,
}

impl VendorCmd {
    const DISPLAY_MODE: u8 = 0x10;
    const MESSAGE_MODE: u8 = 0x11;
    const INPUT_MODE: u8 = 0x12;
    const FIRMWARE_VERSION: u8 = 0x13;
}

impl From<&VendorCmd> for u8 {
    fn from(cmd: &VendorCmd) -> u8 {
        match cmd {
            VendorCmd::DisplayMode => VendorCmd::DISPLAY_MODE,
            VendorCmd::MessageMode => VendorCmd::MESSAGE_MODE,
            VendorCmd::InputMode => VendorCmd::INPUT_MODE,
            VendorCmd::FirmwareVersion => VendorCmd::FIRMWARE_VERSION,
        }
    }
}

pub struct TascamProto{
    cmd: VendorCmd,
    pub val: u8,
    op: VendorDependent,
}

impl<'a> TascamProto {
    const TASCAM_PREFIX: &'a [u8] = &[0x46, 0x49, 0x31];    // 'F', 'I', '1'

    pub fn new(company_id: &[u8;3], cmd: VendorCmd) -> Self {
        TascamProto{
            cmd,
            val: 0xff,
            op: VendorDependent::new(company_id),
        }
    }

    fn build_op(&mut self) -> Result<(), Error> {
        self.op.data.clear();
        self.op.data.extend_from_slice(&Self::TASCAM_PREFIX);
        self.op.data.push(u8::from(&self.cmd));
        self.op.data.push(self.val);
        Ok(())
    }

    fn parse_op(&mut self) -> Result<(), Error> {
        if self.op.data.len() < 5 {
            let label = format!("Data too short for TascamProtocol; {}", self.op.data.len());
            return Err(Error::new(Ta1394AvcError::TooShortResp, &label));
        }

        if self.op.data[3] != u8::from(&self.cmd) {
            let label = format!("Invalid command for TascamProto; {:?}", self.op.data[3]);
            return Err(Error::new(Ta1394AvcError::UnexpectedRespOperands, &label));
        }

        self.val = self.op.data[4];

        Ok(())
    }
}

impl AvcOp for TascamProto{
    const OPCODE: u8 = VendorDependent::OPCODE;
}

impl AvcControl for TascamProto {
    fn build_operands(&mut self, addr: &AvcAddr, operands: &mut Vec<u8>) -> Result<(), Error> {
        Self::build_op(self)?;
        AvcControl::build_operands(&mut self.op, addr, operands)
    }

    fn parse_operands(&mut self, addr: &AvcAddr, operands: &[u8]) -> Result<(), Error> {
        AvcControl::parse_operands(&mut self.op, addr, operands)?;
        Self::parse_op(self)
    }
}

impl AvcStatus for TascamProto {
    fn build_operands(&mut self, addr: &AvcAddr, operands: &mut Vec<u8>) -> Result<(), Error> {
        Self::build_op(self)?;
        AvcStatus::build_operands(&mut self.op, addr, operands)
    }

    fn parse_operands(&mut self, addr: &AvcAddr, operands: &[u8]) -> Result<(), Error> {
        AvcStatus::parse_operands(&mut self.op, addr, operands)?;
        Self::parse_op(self)
    }
}

pub struct TascamAvc{
    pub fcp: hinawa::FwFcp,
    pub company_id: [u8;3],
}

impl TascamAvc {
    pub fn new() -> Self {
        TascamAvc {
            fcp: hinawa::FwFcp::new(),
            company_id: [0;3],
        }
    }
}

impl AsRef<hinawa::FwFcp> for TascamAvc {
    fn as_ref(&self) -> &hinawa::FwFcp {
        &self.fcp
    }
}

impl Ta1394Avc for TascamAvc {
    fn control<O: AvcOp + AvcControl>(&self, addr: &AvcAddr, op: &mut O, timeout_ms: u32) -> Result<(), Error> {
        let mut operands = Vec::new();
        AvcControl::build_operands(op, addr, &mut operands)?;
        let opcode = op.opcode();
        let (rcode, operands) = self.trx(AvcCmdType::Control, addr, opcode, &mut operands, timeout_ms)?;
        let expected = if opcode != VendorDependent::OPCODE {
            AvcRespCode::Accepted
        } else {
            // NOTE: quirk. Furthermore, company_id in response transaction is 0xffffff.
            AvcRespCode::ImplementedStable
        };
        if rcode != expected {
            let label = format!("Unexpected response code for TascamAvc control: {:?}", rcode);
            return Err(Error::new(Ta1394AvcError::UnexpectedRespCode, &label));
        }
        AvcControl::parse_operands(op, addr, &operands)
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

#[cfg(test)]
mod test {
    use ta1394::{AvcAddr, AvcStatus, AvcControl};
    use super::{TascamProto, VendorCmd};

    #[test]
    fn tascam_proto_operands() {
        let mut op = TascamProto::new(&[0x01, 0x23, 0x45], VendorCmd::DisplayMode);
        let operands = [0x01, 0x23, 0x45, 0x46, 0x49, 0x31, 0x10, 0x01];
        AvcStatus::parse_operands(&mut op, &AvcAddr::Unit, &operands).unwrap();
        assert_eq!(op.val, 0x01);

        let mut o = Vec::new();
        AvcStatus::build_operands(&mut op, &AvcAddr::Unit, &mut o).unwrap();
        assert_eq!(o, operands);

        let mut op = TascamProto::new(&[0x54, 0x32, 0x10], VendorCmd::InputMode);
        let operands = [0x54, 0x32, 0x10, 0x46, 0x49, 0x31, 0x12, 0x1c];
        AvcControl::parse_operands(&mut op, &AvcAddr::Unit, &operands).unwrap();
        assert_eq!(op.val, 0x1c);

        let mut o = Vec::new();
        AvcControl::build_operands(&mut op, &AvcAddr::Unit, &mut o).unwrap();
        assert_eq!(o, operands);

    }
}
