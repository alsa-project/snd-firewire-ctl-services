// SPDX-License-Identifier: MIT
// Copyright (c) 2022 Takashi Sakamoto

//! A set of AV/C commands described in general specification.

use super::*;

/// AV/C UNIT INFO command.
///
/// Described in clause "9.2 UNIT INFO command".
#[derive(Debug)]
pub struct UnitInfo {
    pub unit_type: AvcSubunitType,
    pub unit_id: u8,
    pub company_id: [u8; 3],
}

impl Default for UnitInfo {
    fn default() -> Self {
        Self {
            unit_type: AvcSubunitType::Reserved(AvcAddrSubunit::SUBUNIT_TYPE_MASK),
            unit_id: AvcAddrSubunit::SUBUNIT_ID_MASK,
            company_id: [0xff; 3],
        }
    }
}

impl UnitInfo {
    const FIRST_OPERAND: u8 = 0x07;

    pub fn new() -> Self {
        Default::default()
    }
}

impl AvcOp for UnitInfo {
    const OPCODE: u8 = 0x30;
}

impl AvcStatus for UnitInfo {
    fn build_operands(
        &mut self,
        addr: &AvcAddr,
        operands: &mut Vec<u8>,
    ) -> Result<(), AvcCmdBuildError> {
        if let AvcAddr::Subunit(_) = addr {
            Err(AvcCmdBuildError::InvalidAddress)
        } else {
            operands.push(Self::FIRST_OPERAND);
            operands.extend_from_slice(&[0xff; 4]);
            Ok(())
        }
    }

    fn parse_operands(&mut self, _: &AvcAddr, operands: &[u8]) -> Result<(), AvcRespParseError> {
        if operands.len() < 5 {
            Err(AvcRespParseError::TooShortResp(5))
        } else {
            let unit_type = (operands[1] >> AvcAddrSubunit::SUBUNIT_TYPE_SHIFT)
                & AvcAddrSubunit::SUBUNIT_TYPE_MASK;
            let unit_id =
                (operands[1] >> AvcAddrSubunit::SUBUNIT_ID_SHIFT) & AvcAddrSubunit::SUBUNIT_ID_MASK;

            self.unit_type = AvcSubunitType::from(unit_type);
            self.unit_id = unit_id;
            self.company_id.copy_from_slice(&operands[2..5]);
            Ok(())
        }
    }
}

/// The data for each entry of subunit information.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct SubunitInfoEntry {
    pub subunit_type: AvcSubunitType,
    pub maximum_id: u8,
}

impl SubunitInfoEntry {
    pub fn new(subunit_type: AvcSubunitType, maximum_id: u8) -> Self {
        SubunitInfoEntry {
            subunit_type,
            maximum_id,
        }
    }
}

/// AV/C SUBUNIT INFO command.
///
/// Described in clause "9.3 SUBUNIT INFO command".
#[derive(Debug)]
pub struct SubunitInfo {
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
        SubunitInfo {
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
    fn build_operands(
        &mut self,
        addr: &AvcAddr,
        operands: &mut Vec<u8>,
    ) -> Result<(), AvcCmdBuildError> {
        if let AvcAddr::Subunit(_) = addr {
            Err(AvcCmdBuildError::InvalidAddress)
        } else {
            operands.push(
                ((self.page & Self::PAGE_MASK) << Self::PAGE_SHIFT)
                    | ((self.extension_code & Self::EXTENSION_CODE_MASK)
                        << Self::EXTENSION_CODE_SHIFT),
            );
            operands.extend_from_slice(&[0xff; 4]);
            Ok(())
        }
    }

    fn parse_operands(&mut self, _: &AvcAddr, operands: &[u8]) -> Result<(), AvcRespParseError> {
        if operands.len() < 4 {
            Err(AvcRespParseError::TooShortResp(4))
        } else {
            self.page = (operands[0] >> Self::PAGE_SHIFT) & Self::PAGE_MASK;
            self.extension_code =
                (operands[0] >> Self::EXTENSION_CODE_SHIFT) & Self::EXTENSION_CODE_MASK;

            self.entries = operands[1..5]
                .iter()
                .filter(|&operand| *operand != 0xff)
                .map(|operand| {
                    let subunit_type = (operand >> AvcAddrSubunit::SUBUNIT_TYPE_SHIFT)
                        & AvcAddrSubunit::SUBUNIT_TYPE_MASK;
                    let maximum_id = (operand >> AvcAddrSubunit::SUBUNIT_ID_SHIFT)
                        & AvcAddrSubunit::SUBUNIT_ID_MASK;
                    SubunitInfoEntry {
                        subunit_type: AvcSubunitType::from(subunit_type),
                        maximum_id,
                    }
                })
                .collect();

            Ok(())
        }
    }
}

/// AV/C VENDOR-DEPENDENT command.
///
/// Described in clause "9.6 VENDOR-DEPENDENT commands".
#[derive(Debug)]
pub struct VendorDependent {
    pub company_id: [u8; 3],
    pub data: Vec<u8>,
}

impl VendorDependent {
    pub fn new(company_id: &[u8; 3]) -> Self {
        VendorDependent {
            company_id: *company_id,
            data: Vec::new(),
        }
    }

    fn build_operands(&self, operands: &mut Vec<u8>) -> Result<(), AvcCmdBuildError> {
        if self.data.len() > 0 {
            operands.extend_from_slice(&self.company_id);
            operands.extend_from_slice(&self.data);
            Ok(())
        } else {
            Err(AvcCmdBuildError::InvalidOperands)
        }
    }

    fn parse_operands(&mut self, operands: &[u8]) -> Result<(), AvcRespParseError> {
        if operands.len() > 3 {
            self.company_id.copy_from_slice(&operands[0..3]);
            self.data = operands[3..].to_vec();
            Ok(())
        } else {
            Err(AvcRespParseError::TooShortResp(3))
        }
    }
}

impl AvcOp for VendorDependent {
    const OPCODE: u8 = 0x00;
}

impl AvcControl for VendorDependent {
    fn build_operands(
        &mut self,
        _: &AvcAddr,
        operands: &mut Vec<u8>,
    ) -> Result<(), AvcCmdBuildError> {
        Self::build_operands(self, operands)
    }

    fn parse_operands(&mut self, _: &AvcAddr, operands: &[u8]) -> Result<(), AvcRespParseError> {
        Self::parse_operands(self, operands)
    }
}

impl AvcStatus for VendorDependent {
    fn build_operands(
        &mut self,
        _: &AvcAddr,
        operands: &mut Vec<u8>,
    ) -> Result<(), AvcCmdBuildError> {
        Self::build_operands(self, operands)
    }

    fn parse_operands(&mut self, _: &AvcAddr, operands: &[u8]) -> Result<(), AvcRespParseError> {
        Self::parse_operands(self, operands)
    }
}

/// The data of unit plugs for isochronous and external inputs/outputs.
#[derive(Debug)]
pub struct PlugInfoUnitIsocExtData {
    pub isoc_input_plugs: u8,
    pub isoc_output_plugs: u8,
    pub external_input_plugs: u8,
    pub external_output_plugs: u8,
}

/// The data of unit plugs for asynchronous inputs/outputs.
#[derive(Debug)]
pub struct PlugInfoUnitAsyncData {
    pub async_input_plugs: u8,
    pub async_output_plugs: u8,
}

/// The data of the number of plugs for inputs/outputs.
#[derive(Debug)]
pub struct PlugInfoUnitOtherData {
    pub subfunction: u8,
    pub first_input_plug: u8,
    pub input_plugs: u8,
    pub first_output_plug: u8,
    pub output_plugs: u8,
}

/// Plug information for unit.
#[derive(Debug)]
pub enum PlugInfoUnitData {
    IsocExt(PlugInfoUnitIsocExtData),
    Async(PlugInfoUnitAsyncData),
    Other(PlugInfoUnitOtherData),
}

/// Plug information for subunit.
#[derive(Debug)]
pub struct PlugInfoSubunitData {
    pub dst_plugs: u8,
    pub src_plugs: u8,
}

/// AV/C PLUG INFO command.
///
/// Described in clause "10.1 PLUG INFO command".
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
        PlugInfo::Unit(PlugInfoUnitData::IsocExt(PlugInfoUnitIsocExtData {
            isoc_input_plugs: 0xff,
            isoc_output_plugs: 0xff,
            external_input_plugs: 0xff,
            external_output_plugs: 0xff,
        }))
    }

    pub fn new_for_unit_async_plugs() -> Self {
        PlugInfo::Unit(PlugInfoUnitData::Async(PlugInfoUnitAsyncData {
            async_input_plugs: 0xff,
            async_output_plugs: 0xff,
        }))
    }

    pub fn new_for_unit_other_plugs(subfunction: u8) -> Self {
        PlugInfo::Unit(PlugInfoUnitData::Other(PlugInfoUnitOtherData {
            subfunction,
            first_input_plug: 0xff,
            input_plugs: 0xff,
            first_output_plug: 0xff,
            output_plugs: 0xff,
        }))
    }

    pub fn new_for_subunit_plugs() -> Self {
        PlugInfo::Subunit(PlugInfoSubunitData {
            dst_plugs: 0xff,
            src_plugs: 0xff,
        })
    }
}

impl AvcOp for PlugInfo {
    const OPCODE: u8 = 0x02;
}

impl AvcStatus for PlugInfo {
    fn build_operands(
        &mut self,
        addr: &AvcAddr,
        operands: &mut Vec<u8>,
    ) -> Result<(), AvcCmdBuildError> {
        let subfunction = match &self {
            PlugInfo::Unit(u) => {
                if let AvcAddr::Subunit(_) = addr {
                    Err(AvcCmdBuildError::InvalidAddress)?;
                }
                match u {
                    PlugInfoUnitData::IsocExt(_) => Self::SUBFUNC_UNIT_ISOC_EXT,
                    PlugInfoUnitData::Async(_) => Self::SUBFUNC_UNIT_ASYNC,
                    PlugInfoUnitData::Other(d) => d.subfunction,
                }
            }
            PlugInfo::Subunit(_) => {
                if let AvcAddr::Unit = addr {
                    Err(AvcCmdBuildError::InvalidAddress)?;
                }
                Self::SUBFUNC_SUBUNIT
            }
        };
        operands.push(subfunction);
        operands.extend_from_slice(&[0xff; 4]);
        Ok(())
    }

    fn parse_operands(&mut self, _: &AvcAddr, operands: &[u8]) -> Result<(), AvcRespParseError> {
        if operands.len() < 5 {
            Err(AvcRespParseError::TooShortResp(5))?;
        }

        let subfunction = operands[0];
        match self {
            PlugInfo::Unit(u) => match u {
                PlugInfoUnitData::IsocExt(d) => {
                    if subfunction != Self::SUBFUNC_UNIT_ISOC_EXT {
                        Err(AvcRespParseError::UnexpectedOperands(0))?;
                    }
                    d.isoc_input_plugs = operands[1];
                    d.isoc_output_plugs = operands[2];
                    d.external_input_plugs = operands[3];
                    d.external_output_plugs = operands[4];
                }
                PlugInfoUnitData::Async(d) => {
                    if subfunction != Self::SUBFUNC_UNIT_ASYNC {
                        Err(AvcRespParseError::UnexpectedOperands(0))?;
                    }
                    d.async_input_plugs = operands[1];
                    d.async_output_plugs = operands[2];
                }
                PlugInfoUnitData::Other(d) => {
                    if subfunction != d.subfunction {
                        Err(AvcRespParseError::UnexpectedOperands(0))?;
                    }
                    d.first_input_plug = operands[1];
                    d.input_plugs = operands[2];
                    d.first_output_plug = operands[3];
                    d.output_plugs = operands[4];
                }
            },
            PlugInfo::Subunit(s) => {
                if subfunction != Self::SUBFUNC_SUBUNIT {
                    Err(AvcRespParseError::UnexpectedOperands(0))?;
                }
                s.dst_plugs = operands[1];
                s.src_plugs = operands[2];
            }
        }

        Ok(())
    }
}

/// The common data for plug signal format.
#[derive(Debug)]
pub struct PlugSignalFormat {
    pub plug_id: u8,
    pub fmt: u8,
    pub fdf: [u8; 3],
}

impl PlugSignalFormat {
    fn build_operands(
        &mut self,
        addr: &AvcAddr,
        operands: &mut Vec<u8>,
        for_status: bool,
    ) -> Result<(), AvcCmdBuildError> {
        if *addr == AvcAddr::Unit {
            operands.push(self.plug_id);
            if for_status {
                operands.extend_from_slice(&[0xff; 4]);
            } else {
                operands.push(self.fmt);
                operands.extend_from_slice(&self.fdf);
            }
            Ok(())
        } else {
            Err(AvcCmdBuildError::InvalidAddress)
        }
    }

    fn parse_operands(&mut self, _: &AvcAddr, operands: &[u8]) -> Result<(), AvcRespParseError> {
        if operands.len() > 4 {
            self.plug_id = operands[0];
            self.fmt = operands[1];
            self.fdf.copy_from_slice(&operands[2..5]);
            Ok(())
        } else {
            Err(AvcRespParseError::TooShortResp(4))
        }
    }
}

impl Default for PlugSignalFormat {
    fn default() -> Self {
        Self {
            plug_id: 0xff,
            fmt: 0xff,
            fdf: [0xff; 3],
        }
    }
}

/// AV/C INPUT PLUG SIGNAL FORMAT command.
///
/// Described in 10.10 INPUT PLUG SIGNAL FORMAT command.
#[derive(Debug, Default)]
pub struct InputPlugSignalFormat(pub PlugSignalFormat);

impl InputPlugSignalFormat {
    pub fn new(plug_id: u8) -> Self {
        InputPlugSignalFormat(PlugSignalFormat {
            plug_id,
            ..Default::default()
        })
    }
}

impl AvcOp for InputPlugSignalFormat {
    const OPCODE: u8 = 0x19;
}

impl AvcControl for InputPlugSignalFormat {
    fn build_operands(
        &mut self,
        addr: &AvcAddr,
        operands: &mut Vec<u8>,
    ) -> Result<(), AvcCmdBuildError> {
        self.0.build_operands(addr, operands, false)
    }

    fn parse_operands(&mut self, addr: &AvcAddr, operands: &[u8]) -> Result<(), AvcRespParseError> {
        self.0.parse_operands(addr, operands)
    }
}

impl AvcStatus for InputPlugSignalFormat {
    fn build_operands(
        &mut self,
        addr: &AvcAddr,
        operands: &mut Vec<u8>,
    ) -> Result<(), AvcCmdBuildError> {
        self.0.build_operands(addr, operands, true)
    }

    fn parse_operands(&mut self, addr: &AvcAddr, operands: &[u8]) -> Result<(), AvcRespParseError> {
        self.0.parse_operands(addr, operands)
    }
}

/// AV/C OUTPUT PLUG SIGNAL FORMAT command.
///
/// Described in 10.10 OUTPUT PLUG SIGNAL FORMAT command.
#[derive(Debug, Default)]
pub struct OutputPlugSignalFormat(pub PlugSignalFormat);

impl OutputPlugSignalFormat {
    pub fn new(plug_id: u8) -> Self {
        OutputPlugSignalFormat(PlugSignalFormat {
            plug_id,
            ..Default::default()
        })
    }
}

impl AvcOp for OutputPlugSignalFormat {
    const OPCODE: u8 = 0x18;
}

impl AvcControl for OutputPlugSignalFormat {
    fn build_operands(
        &mut self,
        addr: &AvcAddr,
        operands: &mut Vec<u8>,
    ) -> Result<(), AvcCmdBuildError> {
        self.0.build_operands(addr, operands, false)
    }

    fn parse_operands(&mut self, addr: &AvcAddr, operands: &[u8]) -> Result<(), AvcRespParseError> {
        self.0.parse_operands(addr, operands)
    }
}

impl AvcStatus for OutputPlugSignalFormat {
    fn build_operands(
        &mut self,
        addr: &AvcAddr,
        operands: &mut Vec<u8>,
    ) -> Result<(), AvcCmdBuildError> {
        self.0.build_operands(addr, operands, true)
    }

    fn parse_operands(&mut self, addr: &AvcAddr, operands: &[u8]) -> Result<(), AvcRespParseError> {
        self.0.parse_operands(addr, operands)
    }
}

#[cfg(test)]
mod test {
    use crate::general::*;

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
        assert_eq!(
            op.entries[0],
            SubunitInfoEntry::new(AvcSubunitType::Reserved(0x15), 0x05)
        );
        assert_eq!(
            op.entries[1],
            SubunitInfoEntry::new(AvcSubunitType::Reserved(0x17), 0x06)
        );
        assert_eq!(
            op.entries[2],
            SubunitInfoEntry::new(AvcSubunitType::Reserved(0x1d), 0x07)
        );
        assert_eq!(
            op.entries[3],
            SubunitInfoEntry::new(AvcSubunitType::Camera, 0x02)
        );
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
                }
                _ => unreachable!(),
            },
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
            },
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
            },
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
        let addr = AvcAddr::Subunit(AvcAddrSubunit {
            subunit_type: AvcSubunitType::Audio,
            subunit_id: 0x4,
        });
        AvcStatus::build_operands(&mut op, &addr, &mut target).unwrap();
        assert_eq!(&target, &[0x00, 0xff, 0xff, 0xff, 0xff]);
    }

    #[test]
    fn inputplugsignalformat_from() {
        let operands = [0x1e, 0xde, 0xad, 0xbe, 0xef];
        let mut op = InputPlugSignalFormat::new(0x1e);
        AvcStatus::parse_operands(&mut op, &AvcAddr::Unit, &operands).unwrap();
        assert_eq!(op.0.plug_id, 0x1e);
        assert_eq!(op.0.fmt, 0xde);
        assert_eq!(op.0.fdf, [0xad, 0xbe, 0xef]);

        let mut target = Vec::new();
        AvcStatus::build_operands(&mut op, &AvcAddr::Unit, &mut target).unwrap();
        assert_eq!(target, &[0x1e, 0xff, 0xff, 0xff, 0xff]);

        let mut target = Vec::new();
        AvcControl::build_operands(&mut op, &AvcAddr::Unit, &mut target).unwrap();
        assert_eq!(target, operands);

        let mut op = InputPlugSignalFormat::new(0x1e);
        AvcControl::parse_operands(&mut op, &AvcAddr::Unit, &operands).unwrap();
        assert_eq!(op.0.plug_id, 0x1e);
        assert_eq!(op.0.fmt, 0xde);
        assert_eq!(op.0.fdf, [0xad, 0xbe, 0xef]);
    }

    #[test]
    fn outputplugsignalformat_from() {
        let operands = [0x1e, 0xde, 0xad, 0xbe, 0xef];
        let mut op = OutputPlugSignalFormat::new(0x1e);
        AvcStatus::parse_operands(&mut op, &AvcAddr::Unit, &operands).unwrap();
        assert_eq!(op.0.plug_id, 0x1e);
        assert_eq!(op.0.fmt, 0xde);
        assert_eq!(op.0.fdf, [0xad, 0xbe, 0xef]);

        let mut target = Vec::new();
        AvcStatus::build_operands(&mut op, &AvcAddr::Unit, &mut target).unwrap();
        assert_eq!(target, &[0x1e, 0xff, 0xff, 0xff, 0xff]);

        let mut target = Vec::new();
        AvcControl::build_operands(&mut op, &AvcAddr::Unit, &mut target).unwrap();
        assert_eq!(target, operands);

        let mut op = OutputPlugSignalFormat::new(0x1e);
        AvcControl::parse_operands(&mut op, &AvcAddr::Unit, &operands).unwrap();
        assert_eq!(op.0.plug_id, 0x1e);
        assert_eq!(op.0.fmt, 0xde);
        assert_eq!(op.0.fdf, [0xad, 0xbe, 0xef]);
    }
}
