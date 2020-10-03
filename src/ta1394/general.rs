// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use super::{AvcAddr, AvcAddrSubunit, AvcSubunitType, Ta1394AvcError};
use super::{AvcOp, AvcStatus};

//
// AV/C UNIT INFO command.
//
#[derive(Debug)]
pub struct UnitInfo {
    pub unit_type: AvcSubunitType,
    pub unit_id: u8,
    pub company_id: [u8;3],
}

impl UnitInfo {
    const FIRST_OPERAND: u8 = 0x07;

    pub fn new() -> Self {
        UnitInfo{
            unit_type: AvcSubunitType::Reserved(AvcAddrSubunit::SUBUNIT_TYPE_MASK),
            unit_id: AvcAddrSubunit::SUBUNIT_ID_MASK,
            company_id: [0xff;3],
        }
    }
}

impl AvcOp for UnitInfo {
    const OPCODE: u8 = 0x30;
}

impl AvcStatus for UnitInfo {
    fn build_operands(&mut self, addr: &AvcAddr, operands: &mut Vec<u8>) -> Result<(), Error> {
        if let AvcAddr::Subunit(_) = addr {
            let label = "Subunit address is not supported by UnitInfo";
            Err(Error::new(Ta1394AvcError::InvalidCmdOperands, &label))
        } else {
            operands.push(Self::FIRST_OPERAND);
            operands.extend_from_slice(&[0xff;4]);
            Ok(())
        }
    }

    fn parse_operands(&mut self, _: &AvcAddr, operands: &[u8]) -> Result<(), Error> {
        if operands.len() < 5 {
            let label = format!("Oprands too short for UnitInfo; {}", operands.len());
            Err(Error::new(Ta1394AvcError::TooShortResp, &label))
        } else {
            let unit_type = (operands[1] >> AvcAddrSubunit::SUBUNIT_TYPE_SHIFT) & AvcAddrSubunit::SUBUNIT_TYPE_MASK;
            let unit_id = (operands[1] >> AvcAddrSubunit::SUBUNIT_ID_SHIFT) & AvcAddrSubunit::SUBUNIT_ID_MASK;

            self.unit_type = AvcSubunitType::from(unit_type);
            self.unit_id = unit_id;
            self.company_id.copy_from_slice(&operands[2..5]);
            Ok(())
        }
    }
}

#[cfg(test)]
mod test {
    use super::{AvcSubunitType, AvcAddr};
    use super::AvcStatus;
    use super::UnitInfo;

    #[test]
    fn unitinfo_operands() {
        let operands = [0x07, 0xde, 0xad, 0xbe, 0xef];
        let mut op = UnitInfo::new();
        AvcStatus::parse_operands(&mut op, &AvcAddr::Unit, &operands).unwrap();
        assert_eq!(op.unit_type, AvcSubunitType::Reserved(0x1b));
        assert_eq!(op.unit_id, 0x06);
        assert_eq!(op.company_id, [0xad, 0xbe, 0xef]);

        let mut operands = Vec::new();
        AvcStatus::build_operands(&mut op, &AvcAddr::Unit, &mut operands).unwrap();
        assert_eq!(&operands, &[0x07, 0xff, 0xff, 0xff, 0xff]);
    }
}
