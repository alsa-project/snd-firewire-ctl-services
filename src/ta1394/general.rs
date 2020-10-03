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

//
// AV/C PLUG INFO command.
//
#[derive(Debug)]
pub struct PlugInfoUnitIsocExtData {
    pub isoc_input_plugs: u8,
    pub isoc_output_plugs: u8,
    pub external_input_plugs: u8,
    pub external_output_plugs: u8,
}

#[derive(Debug)]
pub struct PlugInfoUnitAsyncData {
    pub async_input_plugs: u8,
    pub async_output_plugs: u8,
}

#[derive(Debug)]
pub struct PlugInfoUnitOtherData {
    pub subfunction: u8,
    pub first_input_plug: u8,
    pub input_plugs: u8,
    pub first_output_plug: u8,
    pub output_plugs: u8,
}

#[derive(Debug)]
pub enum PlugInfoUnitData {
    IsocExt(PlugInfoUnitIsocExtData),
    Async(PlugInfoUnitAsyncData),
    Other(PlugInfoUnitOtherData),
}

#[derive(Debug)]
pub struct PlugInfoSubunitData {
    pub dst_plugs: u8,
    pub src_plugs: u8,
}

#[derive(Debug)]
pub enum PlugInfo {
    Unit(PlugInfoUnitData),
    Subunit(PlugInfoSubunitData),
}

impl PlugInfo {
    const SUBFUNC_UNIT_ISOC_EXT: u8 = 0x00;
    const SUBFUNC_UNIT_ASYNC: u8 = 0x01;
    const SUBFUNC_SUBUNIT: u8 = 0x00;

    pub fn new_for_unit_isoc_ext_plugs() -> Self {
        PlugInfo::Unit(PlugInfoUnitData::IsocExt(PlugInfoUnitIsocExtData{
            isoc_input_plugs: 0xff,
            isoc_output_plugs: 0xff,
            external_input_plugs: 0xff,
            external_output_plugs: 0xff,
        }))
    }

    pub fn new_for_unit_async_plugs() -> Self {
        PlugInfo::Unit(PlugInfoUnitData::Async(PlugInfoUnitAsyncData{
            async_input_plugs: 0xff,
            async_output_plugs: 0xff,
        }))
    }

    pub fn new_for_unit_other_plugs(subfunction: u8) -> Self {
        PlugInfo::Unit(PlugInfoUnitData::Other(PlugInfoUnitOtherData{
            subfunction,
            first_input_plug: 0xff,
            input_plugs: 0xff,
            first_output_plug: 0xff,
            output_plugs: 0xff,
        }))
    }

    pub fn new_for_subunit_plugs() -> Self {
        PlugInfo::Subunit(PlugInfoSubunitData{
            dst_plugs: 0xff,
            src_plugs: 0xff,
        })
    }
}

impl AvcOp for PlugInfo {
    const OPCODE: u8 = 0x02;
}

impl AvcStatus for PlugInfo {
    fn build_operands(&mut self, addr: &AvcAddr, operands: &mut Vec<u8>) -> Result<(), Error> {
        let subfunction = match &self {
            PlugInfo::Unit(u) => {
                if let AvcAddr::Subunit(_) = addr {
                    let label = "Subunit address is not supported for unit plug data by PlugInfo";
                    return Err(Error::new(Ta1394AvcError::InvalidCmdOperands, &label));
                }
                match u {
                    PlugInfoUnitData::IsocExt(_) => Self::SUBFUNC_UNIT_ISOC_EXT,
                    PlugInfoUnitData::Async(_) => Self::SUBFUNC_UNIT_ASYNC,
                    PlugInfoUnitData::Other(d) => d.subfunction,
                }
            }
            PlugInfo::Subunit(_) => {
                if let AvcAddr::Unit = addr {
                    let label = "Unit address is not supported for subunit plug data by PlugInfo";
                    return Err(Error::new(Ta1394AvcError::InvalidCmdOperands, &label));
                }
                Self::SUBFUNC_SUBUNIT
            }
        };
        operands.push(subfunction);
        operands.extend_from_slice(&[0xff;4]);
        Ok(())
    }

    fn parse_operands(&mut self, _: &AvcAddr, operands: &[u8]) -> Result<(), Error> {
        if operands.len() < 5 {
            let label = format!("Oprands too short for PlugInfo; {}", operands.len());
            return Err(Error::new(Ta1394AvcError::TooShortResp, &label));
        }

        let subfunction = operands[0];
        match self {
            PlugInfo::Unit(u) => {
                match u {
                    PlugInfoUnitData::IsocExt(d) => {
                        if subfunction != Self::SUBFUNC_UNIT_ISOC_EXT {
                            let label = format!("Invalid subfunction for unit by PlugInfo: {}", subfunction);
                            return Err(Error::new(Ta1394AvcError::UnexpectedRespOperands, &label));
                        }
                        d.isoc_input_plugs = operands[1];
                        d.isoc_output_plugs = operands[2];
                        d.external_input_plugs = operands[3];
                        d.external_output_plugs = operands[4];
                    }
                    PlugInfoUnitData::Async(d) => {
                        if subfunction != Self::SUBFUNC_UNIT_ASYNC {
                            let label = format!("Invalid subfunction for unit by PlugInfo: {}", subfunction);
                            return Err(Error::new(Ta1394AvcError::UnexpectedRespOperands, &label));
                        }
                        d.async_input_plugs = operands[1];
                        d.async_output_plugs = operands[2];
                    }
                    PlugInfoUnitData::Other(d) => {
                        if subfunction != d.subfunction {
                            let label = format!("Invalid subfunction for unit by PlugInfo: {}", subfunction);
                            return Err(Error::new(Ta1394AvcError::UnexpectedRespOperands, &label));
                        }
                        d.first_input_plug = operands[1];
                        d.input_plugs = operands[2];
                        d.first_output_plug = operands[3];
                        d.output_plugs = operands[4];
                    }
                }
            }
            PlugInfo::Subunit(s) => {
                if subfunction != Self::SUBFUNC_SUBUNIT {
                    let label = format!("Invalid subfunction for subunit by PlugInfo: {}", subfunction);
                    return Err(Error::new(Ta1394AvcError::UnexpectedRespOperands, &label));
                }
                s.dst_plugs = operands[1];
                s.src_plugs = operands[2];
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::{AvcSubunitType, AvcAddr, AvcAddrSubunit};
    use super::{AvcStatus, AvcControl};
    use super::{UnitInfo, SubunitInfo, SubunitInfoEntry, VendorDependent};
    use super::{PlugInfo, PlugInfoUnitData};

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

    #[test]
    fn op_operands() {
        let operands = [0x00, 0xde, 0xad, 0xbe, 0xef];
        let mut op = PlugInfo::new_for_unit_isoc_ext_plugs();
        AvcStatus::parse_operands(&mut op, &AvcAddr::Unit, &operands).unwrap();
        match &op {
            PlugInfo::Unit(u) => match u {
                PlugInfoUnitData::IsocExt(d) => {
                    assert_eq!(d.isoc_input_plugs, 0xde);
                    assert_eq!(d.isoc_output_plugs, 0xad);
                    assert_eq!(d.external_input_plugs, 0xbe);
                    assert_eq!(d.external_output_plugs, 0xef);
                },
                _ => unreachable!(),
            }
            _ => unreachable!(),
        }

        let mut target = Vec::new();
        AvcStatus::build_operands(&mut op, &AvcAddr::Unit, &mut target).unwrap();
        assert_eq!(&target, &[0x00, 0xff, 0xff, 0xff, 0xff]);

        let operands = [0x01, 0xde, 0xad, 0xff, 0xff];
        let mut op = PlugInfo::new_for_unit_async_plugs();
        AvcStatus::parse_operands(&mut op, &AvcAddr::Unit, &operands).unwrap();
        match &op {
            PlugInfo::Unit(u) => match u {
                PlugInfoUnitData::Async(d) => {
                    assert_eq!(d.async_input_plugs, 0xde);
                    assert_eq!(d.async_output_plugs, 0xad);
                }
                _ => unreachable!(),
            }
            _ => unreachable!(),
        }

        let mut target = Vec::new();
        AvcStatus::build_operands(&mut op, &AvcAddr::Unit, &mut target).unwrap();
        assert_eq!(&target, &[0x01, 0xff, 0xff, 0xff, 0xff]);

        let operands = [0x53, 0xde, 0xad, 0xbe, 0xef];
        let mut op = PlugInfo::new_for_unit_other_plugs(0x53);
        AvcStatus::parse_operands(&mut op, &AvcAddr::Unit, &operands).unwrap();
        match &op {
            PlugInfo::Unit(u) => match u {
                PlugInfoUnitData::Other(d) => {
                    assert_eq!(d.subfunction, 0x53);
                    assert_eq!(d.first_input_plug, 0xde);
                    assert_eq!(d.input_plugs, 0xad);
                    assert_eq!(d.first_output_plug, 0xbe);
                    assert_eq!(d.output_plugs, 0xef);
                }
                _ => unreachable!(),
            }
            _ => unreachable!(),
        }

        let mut target = Vec::new();
        AvcStatus::build_operands(&mut op, &AvcAddr::Unit, &mut target).unwrap();
        assert_eq!(&target, &[0x53, 0xff, 0xff, 0xff, 0xff]);

        let operands = [0x00, 0xde, 0xad, 0xff, 0xff];
        let mut op = PlugInfo::new_for_subunit_plugs();
        AvcStatus::parse_operands(&mut op, &AvcAddr::Unit, &operands).unwrap();
        match &op {
            PlugInfo::Subunit(s) => {
                assert_eq!(s.dst_plugs, 0xde);
                assert_eq!(s.src_plugs, 0xad);
            }
            _ => unreachable!(),
        }

        let mut target = Vec::new();
        let addr = AvcAddr::Subunit(AvcAddrSubunit{
            subunit_type: AvcSubunitType::Audio,
            subunit_id: 0x4,
        });
        AvcStatus::build_operands(&mut op, &addr, &mut target).unwrap();
        assert_eq!(&target, &[0x00, 0xff, 0xff, 0xff, 0xff]);
    }
}
