// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use super::{AvcAddr, AvcAddrSubunit, AvcSubunitType, Ta1394AvcError};
use super::{AvcOp, AvcStatus, AvcControl};

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

//
// AV/C SUBUNIT INFO command.
//
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct SubunitInfoEntry{
    pub subunit_type: AvcSubunitType,
    pub maximum_id: u8,
}

impl SubunitInfoEntry {
    pub fn new(subunit_type: AvcSubunitType, maximum_id: u8) -> Self {
        SubunitInfoEntry{subunit_type, maximum_id}
    }
}

#[derive(Debug)]
pub struct SubunitInfo{
    pub page: u8,
    pub extension_code: u8,
    pub entries: Vec<SubunitInfoEntry>,
}

impl SubunitInfo {
    const PAGE_SHIFT: usize = 4;
    const PAGE_MASK: u8 = 0x07;
    const EXTENSION_CODE_SHIFT: usize = 0;
    const EXTENSION_CODE_MASK: u8 = 0x07;

    pub fn new(page: u8, extension_code: u8) -> Self {
        SubunitInfo{
            page,
            extension_code,
            entries: Vec::new(),
        }
    }
}

impl AvcOp for SubunitInfo {
    const OPCODE: u8 = 0x31;
}

impl AvcStatus for SubunitInfo {
    fn build_operands(&mut self, addr: &AvcAddr, operands: &mut Vec<u8>) -> Result<(), Error> {
        if let AvcAddr::Subunit(_) = addr {
            let label = "Subunit address is not supported by SubunitInfo";
            return Err(Error::new(Ta1394AvcError::InvalidCmdOperands, &label));
        } else {
            operands.push(((self.page & Self::PAGE_MASK) << Self::PAGE_SHIFT) |
                          ((self.extension_code & Self::EXTENSION_CODE_MASK) << Self::EXTENSION_CODE_SHIFT));
            operands.extend_from_slice(&[0xff;4]);
            Ok(())
        }
    }

    fn parse_operands(&mut self, _: &AvcAddr, operands: &[u8]) -> Result<(), Error> {
        if operands.len() < 4 {
            let label = format!("Oprands too short for SubunitInfo; {}", operands.len());
            Err(Error::new(Ta1394AvcError::TooShortResp, &label))
        } else {
            self.page = (operands[0] >> Self::PAGE_SHIFT) & Self::PAGE_MASK;
            self.extension_code = (operands[0] >> Self::EXTENSION_CODE_SHIFT) & Self::EXTENSION_CODE_MASK;

            self.entries = operands[1..5].iter()
                .filter(|&operand| *operand != 0xff)
                .map(|operand| {
                    let subunit_type = (operand >> AvcAddrSubunit::SUBUNIT_TYPE_SHIFT) & AvcAddrSubunit::SUBUNIT_TYPE_MASK;
                    let maximum_id = (operand >> AvcAddrSubunit::SUBUNIT_ID_SHIFT) & AvcAddrSubunit::SUBUNIT_ID_MASK;
                    SubunitInfoEntry{
                        subunit_type: AvcSubunitType::from(subunit_type),
                        maximum_id,
                    }
                }).collect();

            Ok(())
        }
    }
}

//
// AV/C VENDOR-DEPENDENT command.
//
#[derive(Debug)]
pub struct VendorDependent {
    pub company_id: [u8;3],
    pub data: Vec<u8>,
}

impl VendorDependent {
    pub fn new(company_id: &[u8;3]) -> Self {
        VendorDependent{
            company_id: *company_id,
            data: Vec::new(),
        }
    }

    fn build_operands(&self, operands: &mut Vec<u8>) -> Result<(), Error> {
        if self.data.len() > 0 {
            operands.extend_from_slice(&self.company_id);
            operands.extend_from_slice(&self.data);
            Ok(())
        } else {
            let label = format!("No data for VendorDependent");
            Err(Error::new(Ta1394AvcError::InvalidCmdOperands, &label))
        }
    }

    fn parse_operands(&mut self, operands: &[u8]) -> Result<(), Error> {
        if operands.len() > 3 {
            self.company_id.copy_from_slice(&operands[0..3]);
            self.data = operands[3..].to_vec();
            Ok(())
        } else {
            let label = format!("Oprands too short for VendorDependent; {}", operands.len());
            Err(Error::new(Ta1394AvcError::TooShortResp, &label))
        }
    }
}

impl AvcOp for VendorDependent {
    const OPCODE: u8 = 0x00;
}

impl AvcControl for VendorDependent {
    fn build_operands(&mut self, _: &AvcAddr, operands: &mut Vec<u8>) -> Result<(), Error> {
        Self::build_operands(self, operands)
    }

    fn parse_operands(&mut self, _: &AvcAddr, operands: &[u8]) -> Result<(), Error> {
        Self::parse_operands(self, operands)
    }
}

impl AvcStatus for VendorDependent {
    fn build_operands(&mut self, _: &AvcAddr, operands: &mut Vec<u8>) -> Result<(), Error> {
        Self::build_operands(self, operands)
    }

    fn parse_operands(&mut self, _: &AvcAddr, operands: &[u8]) -> Result<(), Error> {
        Self::parse_operands(self, operands)
    }
}

#[cfg(test)]
mod test {
    use super::{AvcSubunitType, AvcAddr};
    use super::{AvcStatus, AvcControl};
    use super::{UnitInfo, SubunitInfo, SubunitInfoEntry, VendorDependent};

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

    #[test]
    fn subunitinfo_operands() {
        let operands = [0xde, 0xad, 0xbe, 0xef, 0x3a];
        let mut op = SubunitInfo::new(0, 0);
        AvcStatus::parse_operands(&mut op, &AvcAddr::Unit, &operands).unwrap();
        assert_eq!(op.page, 0x05);
        assert_eq!(op.extension_code, 0x06);
        assert_eq!(op.entries[0], SubunitInfoEntry::new(AvcSubunitType::Reserved(0x15), 0x05));
        assert_eq!(op.entries[1], SubunitInfoEntry::new(AvcSubunitType::Reserved(0x17), 0x06));
        assert_eq!(op.entries[2], SubunitInfoEntry::new(AvcSubunitType::Reserved(0x1d), 0x07));
        assert_eq!(op.entries[3], SubunitInfoEntry::new(AvcSubunitType::Camera, 0x02));
    }

    #[test]
    fn vendor_dependent_operands() {
        let company_id = [0x00, 0x01, 0x02];
        let operands = [0x00, 0x01, 0x02, 0xde, 0xad, 0xbe, 0xef];
        let mut op = VendorDependent::new(&company_id);
        AvcStatus::parse_operands(&mut op, &AvcAddr::Unit, &operands).unwrap();
        assert_eq!(op.company_id, company_id);
        assert_eq!(&op.data, &[0xde, 0xad, 0xbe, 0xef]);

        let mut target = Vec::new();
        AvcStatus::build_operands(&mut op, &AvcAddr::Unit, &mut target).unwrap();
        assert_eq!(&target, &operands);

        let mut target = Vec::new();
        AvcControl::build_operands(&mut op, &AvcAddr::Unit, &mut target).unwrap();
        assert_eq!(&target, &operands);

        AvcControl::parse_operands(&mut op, &AvcAddr::Unit, &target).unwrap();
        assert_eq!(op.company_id, company_id);
        assert_eq!(&op.data, &[0xde, 0xad, 0xbe, 0xef]);
    }
}
