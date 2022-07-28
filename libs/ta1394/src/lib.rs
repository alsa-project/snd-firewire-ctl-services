// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
pub mod amdtp;
pub mod audio;
pub mod ccm;
pub mod config_rom;
pub mod general;
pub mod stream_format;

use glib::{error::ErrorDomain, Error, Quark};

/// The type of subunit for AV/C address defined by 1394 Trading Association.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AvcSubunitType {
    Monitor,
    Audio,
    Printer,
    Disc,
    Tape,
    Tuner,
    Ca,
    Camera,
    Panel,
    BulletinBoard,
    CameraStorage,
    Music,
    VendorUnique,
    Extended,
    Reserved(u8),
}

impl AvcSubunitType {
    const MONITOR: u8 = 0x00;
    const AUDIO: u8 = 0x01;
    const PRINTER: u8 = 0x02;
    const DISC: u8 = 0x03;
    const TAPE: u8 = 0x04;
    const TUNER: u8 = 0x05;
    const CA: u8 = 0x06;
    const CAMERA: u8 = 0x07;
    const PANEL: u8 = 0x09;
    const BULLETIN_BOARD: u8 = 0x0a;
    const CAMERA_STORAGE: u8 = 0x0b;
    const MUSIC: u8 = 0x0c;
    const VENDOR_UNIQUE: u8 = 0x1c;
    const EXTENDED: u8 = 0x1e;
}

impl From<u8> for AvcSubunitType {
    fn from(val: u8) -> Self {
        match val {
            AvcSubunitType::MONITOR => AvcSubunitType::Monitor,
            AvcSubunitType::AUDIO => AvcSubunitType::Audio,
            AvcSubunitType::PRINTER => AvcSubunitType::Printer,
            AvcSubunitType::DISC => AvcSubunitType::Disc,
            AvcSubunitType::TAPE => AvcSubunitType::Tape,
            AvcSubunitType::TUNER => AvcSubunitType::Tuner,
            AvcSubunitType::CA => AvcSubunitType::Ca,
            AvcSubunitType::CAMERA => AvcSubunitType::Camera,
            AvcSubunitType::PANEL => AvcSubunitType::Panel,
            AvcSubunitType::BULLETIN_BOARD => AvcSubunitType::BulletinBoard,
            AvcSubunitType::CAMERA_STORAGE => AvcSubunitType::CameraStorage,
            AvcSubunitType::MUSIC => AvcSubunitType::Music,
            AvcSubunitType::VENDOR_UNIQUE => AvcSubunitType::VendorUnique,
            AvcSubunitType::EXTENDED => AvcSubunitType::Extended,
            _ => AvcSubunitType::Reserved(val),
        }
    }
}

impl From<&AvcSubunitType> for u8 {
    fn from(subunit_type: &AvcSubunitType) -> Self {
        match subunit_type {
            AvcSubunitType::Monitor => AvcSubunitType::MONITOR,
            AvcSubunitType::Audio => AvcSubunitType::AUDIO,
            AvcSubunitType::Printer => AvcSubunitType::PRINTER,
            AvcSubunitType::Disc => AvcSubunitType::DISC,
            AvcSubunitType::Tape => AvcSubunitType::TAPE,
            AvcSubunitType::Tuner => AvcSubunitType::TUNER,
            AvcSubunitType::Ca => AvcSubunitType::CA,
            AvcSubunitType::Camera => AvcSubunitType::CAMERA,
            AvcSubunitType::Panel => AvcSubunitType::PANEL,
            AvcSubunitType::BulletinBoard => AvcSubunitType::BULLETIN_BOARD,
            AvcSubunitType::CameraStorage => AvcSubunitType::CAMERA_STORAGE,
            AvcSubunitType::Music => AvcSubunitType::MUSIC,
            AvcSubunitType::VendorUnique => AvcSubunitType::VENDOR_UNIQUE,
            AvcSubunitType::Extended => AvcSubunitType::EXTENDED,
            AvcSubunitType::Reserved(value) => *value,
        }
    }
}

impl From<AvcSubunitType> for u8 {
    fn from(subunit_type: AvcSubunitType) -> Self {
        Self::from(&subunit_type)
    }
}

/// The AV/C address of first music subunit for convenience.
pub const MUSIC_SUBUNIT_0: AvcAddrSubunit = AvcAddrSubunit {
    subunit_type: AvcSubunitType::Music,
    subunit_id: 0,
};

/// The AV/C address of first audio subunit for convenience.
pub const AUDIO_SUBUNIT_0: AvcAddrSubunit = AvcAddrSubunit {
    subunit_type: AvcSubunitType::Audio,
    subunit_id: 0,
};

/// The data of AV/C address in subunit case.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AvcAddrSubunit {
    pub subunit_type: AvcSubunitType,
    pub subunit_id: u8,
}

impl AvcAddrSubunit {
    const SUBUNIT_TYPE_SHIFT: usize = 3;
    const SUBUNIT_TYPE_MASK: u8 = 0x1f;
    const SUBUNIT_ID_SHIFT: usize = 0;
    const SUBUNIT_ID_MASK: u8 = 0x07;

    pub fn new(subunit_type: AvcSubunitType, mut subunit_id: u8) -> Self {
        subunit_id &= Self::SUBUNIT_ID_MASK;
        AvcAddrSubunit {
            subunit_type,
            subunit_id,
        }
    }
}

impl From<u8> for AvcAddrSubunit {
    fn from(val: u8) -> Self {
        let subunit_type =
            AvcSubunitType::from((val >> Self::SUBUNIT_TYPE_SHIFT) & Self::SUBUNIT_TYPE_MASK);
        let subunit_id = (val >> Self::SUBUNIT_ID_SHIFT) & Self::SUBUNIT_ID_MASK;
        AvcAddrSubunit {
            subunit_type,
            subunit_id,
        }
    }
}

impl From<&AvcAddrSubunit> for u8 {
    fn from(subunit: &AvcAddrSubunit) -> Self {
        let mut val = u8::from(subunit.subunit_type);
        val = (val & AvcAddrSubunit::SUBUNIT_TYPE_MASK) << AvcAddrSubunit::SUBUNIT_TYPE_SHIFT;
        val |= (subunit.subunit_id & AvcAddrSubunit::SUBUNIT_ID_MASK)
            << AvcAddrSubunit::SUBUNIT_ID_SHIFT;
        val
    }
}

impl From<AvcAddrSubunit> for u8 {
    fn from(subunit: AvcAddrSubunit) -> Self {
        Self::from(&subunit)
    }
}

/// For AV/C address in both unit and subunit cases.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AvcAddr {
    Unit,
    Subunit(AvcAddrSubunit),
}

impl AvcAddr {
    const UNIT_ADDR: u8 = 0xff;
}

impl From<u8> for AvcAddr {
    fn from(val: u8) -> Self {
        match val {
            Self::UNIT_ADDR => AvcAddr::Unit,
            _ => AvcAddr::Subunit(AvcAddrSubunit::from(val)),
        }
    }
}

impl From<&AvcAddr> for u8 {
    fn from(addr: &AvcAddr) -> Self {
        match addr {
            AvcAddr::Unit => AvcAddr::UNIT_ADDR,
            AvcAddr::Subunit(d) => u8::from(*d),
        }
    }
}

impl From<AvcAddr> for u8 {
    fn from(addr: AvcAddr) -> Self {
        Self::from(&addr)
    }
}

/// The type of command in AV/C transaction.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AvcCmdType {
    /// Perform an operation to the addressed target.
    Control,
    /// Check current status of the addressed target.
    Status,
    /// Check whether the addressed target supports a particular Control command including operands.
    SpecificInquiry,
    /// Schedule notification of a change in the addressed target.
    Notify,
    /// Check whether the addressed target supports a particular Control command just with opcode.
    GeneralInquiry,
    Reserved(u8),
}

impl AvcCmdType {
    const CONTROL: u8 = 0x00;
    const STATUS: u8 = 0x01;
    const SPECIFIC_INQUIRY: u8 = 0x02;
    const NOTIFY: u8 = 0x03;
    const GENERAL_INQUIRY: u8 = 0x04;
}

impl From<u8> for AvcCmdType {
    fn from(val: u8) -> Self {
        match val {
            AvcCmdType::CONTROL => AvcCmdType::Control,
            AvcCmdType::STATUS => AvcCmdType::Status,
            AvcCmdType::SPECIFIC_INQUIRY => AvcCmdType::SpecificInquiry,
            AvcCmdType::NOTIFY => AvcCmdType::Notify,
            AvcCmdType::GENERAL_INQUIRY => AvcCmdType::GeneralInquiry,
            _ => Self::Reserved(val),
        }
    }
}

impl From<AvcCmdType> for u8 {
    fn from(code: AvcCmdType) -> Self {
        match code {
            AvcCmdType::Control => AvcCmdType::CONTROL,
            AvcCmdType::Status => AvcCmdType::STATUS,
            AvcCmdType::SpecificInquiry => AvcCmdType::SPECIFIC_INQUIRY,
            AvcCmdType::Notify => AvcCmdType::NOTIFY,
            AvcCmdType::GeneralInquiry => AvcCmdType::GENERAL_INQUIRY,
            AvcCmdType::Reserved(val) => val,
        }
    }
}

/// The status of response in AV/C transaction.
#[derive(Debug, Eq, PartialEq)]
pub enum AvcRespCode {
    /// The target does not implement the requested command or the addressed subunit.
    NotImplemented,
    /// The requested CONTROL command has been processed or is scheduled to process.
    Accepted,
    /// The target refused to process the requested command due to some reasons.
    Rejected,
    /// The target is under transition state and can not process the requested STATUS command.
    InTransition,
    /// The target implements the inquired command or returns current status against the requested
    /// STATUS command.
    ImplementedStable,
    /// The actual notification scheduled by the NOTIFY command.
    Changed,
    /// The intermediate response during AV/C deferred transaction.
    Interim,
    Reserved(u8),
}

impl AvcRespCode {
    const NOT_IMPLEMENTED: u8 = 0x08;
    const ACCEPTED: u8 = 0x09;
    const REJECTED: u8 = 0x0a;
    const IN_TRANSITION: u8 = 0x0b;
    const IMPLEMENTED_STABLE: u8 = 0x0c;
    const CHANGED: u8 = 0x0d;
    const INTERIM: u8 = 0x0f;
}

impl From<u8> for AvcRespCode {
    fn from(val: u8) -> Self {
        match val {
            AvcRespCode::NOT_IMPLEMENTED => AvcRespCode::NotImplemented,
            AvcRespCode::ACCEPTED => AvcRespCode::Accepted,
            AvcRespCode::REJECTED => AvcRespCode::Rejected,
            AvcRespCode::IN_TRANSITION => AvcRespCode::InTransition,
            AvcRespCode::IMPLEMENTED_STABLE => AvcRespCode::ImplementedStable,
            AvcRespCode::CHANGED => AvcRespCode::Changed,
            AvcRespCode::INTERIM => AvcRespCode::Interim,
            _ => Self::Reserved(val),
        }
    }
}

impl From<AvcRespCode> for u8 {
    fn from(resp: AvcRespCode) -> Self {
        match resp {
            AvcRespCode::NotImplemented => AvcRespCode::NOT_IMPLEMENTED,
            AvcRespCode::Accepted => AvcRespCode::ACCEPTED,
            AvcRespCode::Rejected => AvcRespCode::REJECTED,
            AvcRespCode::InTransition => AvcRespCode::IN_TRANSITION,
            AvcRespCode::ImplementedStable => AvcRespCode::IMPLEMENTED_STABLE,
            AvcRespCode::Changed => AvcRespCode::CHANGED,
            AvcRespCode::Interim => AvcRespCode::INTERIM,
            AvcRespCode::Reserved(val) => val,
        }
    }
}

/// The error to build command frame for AV/C transaction.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum AvcCmdBuildError {
    /// Invalid address for the operation.
    InvalidAddress,
    /// Fail to prepare operands for the operation.
    InvalidOperands,
}

impl std::fmt::Display for AvcCmdBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidAddress => write!(f, "invalid address"),
            Self::InvalidOperands => write!(f, "invalid operands"),
        }
    }
}

/// For AV/C operation with opcode.
pub trait AvcOp {
    /// The code to specify operation.
    const OPCODE: u8;
}

/// The AV/C operation supporting control and inquiry command.
pub trait AvcControl {
    fn build_operands(
        &mut self,
        addr: &AvcAddr,
        operands: &mut Vec<u8>,
    ) -> Result<(), AvcCmdBuildError>;
    fn parse_operands(&mut self, addr: &AvcAddr, operands: &[u8]) -> Result<(), Error>;
}

/// The AV/C operation supporting status command.
pub trait AvcStatus {
    fn build_operands(
        &mut self,
        addr: &AvcAddr,
        operands: &mut Vec<u8>,
    ) -> Result<(), AvcCmdBuildError>;
    fn parse_operands(&mut self, addr: &AvcAddr, operands: &[u8]) -> Result<(), Error>;
}

/// The AV/C operation supporting notify command.
pub trait AvcNotify {
    fn build_operands(
        &mut self,
        addr: &AvcAddr,
        operands: &mut Vec<u8>,
    ) -> Result<(), AvcCmdBuildError>;
    fn parse_operands(&mut self, addr: &AvcAddr, operands: &[u8]) -> Result<(), Error>;
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Ta1394AvcError {
    TooShortResp,
    UnexpectedRespCode,
    UnexpectedRespOperands,
    /// Fail to build command frame.
    CmdBuild(AvcCmdBuildError),
    Invalid(i32),
}

impl ErrorDomain for Ta1394AvcError {
    fn domain() -> Quark {
        Quark::from_str("ta1394-avc-error-quark")
    }

    fn code(self) -> i32 {
        match self {
            Self::TooShortResp => 0,
            Self::UnexpectedRespCode => 1,
            Self::UnexpectedRespOperands => 2,
            Self::CmdBuild(_) => 3,
            Self::Invalid(val) => val,
        }
    }

    fn from(code: i32) -> Option<Self> {
        let enumeration = match code {
            0 => Self::TooShortResp,
            1 => Self::UnexpectedRespCode,
            2 => Self::UnexpectedRespOperands,
            3 => Self::CmdBuild(AvcCmdBuildError::InvalidAddress),
            _ => Self::Invalid(code),
        };

        Some(enumeration)
    }
}

/// For AV/C transaction defined by 1394 Trading Association.
///
/// AV/C transaction is defined to use Function Control Protocol (FCP) in IEC 61883-1 as
/// communication backend to target device. Additionally, two types of transaction are supported;
/// immediate and deferred transactions. `Ta1394Avc::transaction()` should support both of them
/// as blocking API to wait for response.
pub trait Ta1394Avc {
    const FRAME_SIZE: usize = 0x200;
    const RESP_CODE_MASK: u8 = 0x0f;

    fn transaction(
        &self,
        ctype: AvcCmdType,
        addr: &AvcAddr,
        opcode: u8,
        operands: &[u8],
        timeout_ms: u32,
    ) -> Result<(AvcRespCode, Vec<u8>), Error>;

    fn control<O: AvcOp + AvcControl>(
        &self,
        addr: &AvcAddr,
        op: &mut O,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut operands = Vec::new();
        AvcControl::build_operands(op, addr, &mut operands)
            .map_err(|err| Error::new(Ta1394AvcError::CmdBuild(err), ""))?;
        self.transaction(AvcCmdType::Control, addr, O::OPCODE, &operands, timeout_ms)
            .and_then(|(rcode, operands)| match rcode {
                AvcRespCode::Accepted => AvcControl::parse_operands(op, addr, &operands),
                _ => {
                    let label = format!(
                        "Unexpected response code for control opcode {}: {:?}",
                        O::OPCODE,
                        rcode
                    );
                    Err(Error::new(Ta1394AvcError::UnexpectedRespCode, &label))
                }
            })
    }

    fn status<O: AvcOp + AvcStatus>(
        &self,
        addr: &AvcAddr,
        op: &mut O,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut operands = Vec::new();
        AvcStatus::build_operands(op, addr, &mut operands)
            .map_err(|err| Error::new(Ta1394AvcError::CmdBuild(err), ""))?;
        self.transaction(AvcCmdType::Status, addr, O::OPCODE, &operands, timeout_ms)
            .and_then(|(rcode, operands)| match rcode {
                AvcRespCode::ImplementedStable => AvcStatus::parse_operands(op, addr, &operands),
                _ => {
                    let label = format!(
                        "Unexpected response code for status opcode {}: {:?}",
                        O::OPCODE,
                        rcode
                    );
                    Err(Error::new(Ta1394AvcError::UnexpectedRespCode, &label))
                }
            })
    }

    fn specific_inquiry<O: AvcOp + AvcControl>(
        &self,
        addr: &AvcAddr,
        op: &mut O,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut operands = Vec::new();
        AvcControl::build_operands(op, addr, &mut operands)
            .map_err(|err| Error::new(Ta1394AvcError::CmdBuild(err), ""))?;
        self.transaction(
            AvcCmdType::SpecificInquiry,
            addr,
            O::OPCODE,
            &operands,
            timeout_ms,
        )
        .and_then(|(rcode, operands)| match rcode {
            AvcRespCode::ImplementedStable => AvcControl::parse_operands(op, addr, &operands),
            _ => {
                let label = format!(
                    "Unexpected response code for specific inquiry opcode {}: {:?}",
                    O::OPCODE,
                    rcode
                );
                Err(Error::new(Ta1394AvcError::UnexpectedRespCode, &label))
            }
        })
    }

    fn notify<O: AvcOp + AvcNotify>(
        &self,
        addr: &AvcAddr,
        op: &mut O,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut operands = Vec::new();
        AvcNotify::build_operands(op, addr, &mut operands)
            .map_err(|err| Error::new(Ta1394AvcError::CmdBuild(err), ""))?;
        self.transaction(AvcCmdType::Notify, addr, O::OPCODE, &operands, timeout_ms)
            .and_then(|(rcode, operands)| match rcode {
                AvcRespCode::Changed => AvcNotify::parse_operands(op, addr, &operands),
                _ => {
                    let label = format!(
                        "Unexpected response code for notify opcode {}: {:?}",
                        O::OPCODE,
                        rcode
                    );
                    Err(Error::new(Ta1394AvcError::UnexpectedRespCode, &label))
                }
            })
    }
}

#[cfg(test)]
mod test {
    use crate::*;

    #[test]
    fn avcsubunittype_from() {
        assert_eq!(0x00, u8::from(AvcSubunitType::from(0x00)));
        assert_eq!(0x01, u8::from(AvcSubunitType::from(0x01)));
        assert_eq!(0x02, u8::from(AvcSubunitType::from(0x02)));
        assert_eq!(0x03, u8::from(AvcSubunitType::from(0x03)));
        assert_eq!(0x04, u8::from(AvcSubunitType::from(0x04)));
        assert_eq!(0x05, u8::from(AvcSubunitType::from(0x05)));
        assert_eq!(0x06, u8::from(AvcSubunitType::from(0x06)));
        assert_eq!(0x07, u8::from(AvcSubunitType::from(0x07)));
        assert_eq!(0x09, u8::from(AvcSubunitType::from(0x09)));
        assert_eq!(0x0a, u8::from(AvcSubunitType::from(0x0a)));
        assert_eq!(0x0b, u8::from(AvcSubunitType::from(0x0b)));
        assert_eq!(0x0c, u8::from(AvcSubunitType::from(0x0c)));
        assert_eq!(0x1c, u8::from(AvcSubunitType::from(0x1c)));
        assert_eq!(0x1e, u8::from(AvcSubunitType::from(0x1e)));
        assert_eq!(0xff, u8::from(AvcSubunitType::from(0xff)));
    }

    #[test]
    fn avcaddrsubunit_from() {
        // For audio subunit.
        assert_eq!(0x80, u8::from(AvcAddrSubunit::from(0x80)));
        assert_eq!(0x81, u8::from(AvcAddrSubunit::from(0x81)));
        assert_eq!(0x82, u8::from(AvcAddrSubunit::from(0x82)));
        // For music subunit.
        assert_eq!(0x60, u8::from(AvcAddrSubunit::from(0x60)));
        assert_eq!(0x61, u8::from(AvcAddrSubunit::from(0x61)));
        assert_eq!(0x62, u8::from(AvcAddrSubunit::from(0x62)));
    }

    #[test]
    fn avcaddr_from() {
        assert_eq!(AvcAddr::from(0xff), AvcAddr::Unit);
        assert_eq!(AvcAddr::from(0x09),
                   AvcAddr::Subunit(AvcAddrSubunit::new(AvcSubunitType::Audio, 0x01)));
        assert_eq!(AvcAddr::from(0x63),
                   AvcAddr::Subunit(AvcAddrSubunit::new(AvcSubunitType::Music, 0x03)));
        assert_eq!(AvcAddr::from(0x87),
                   AvcAddr::Subunit(AvcAddrSubunit::new(AvcSubunitType::Reserved(0x10), 0x07)));
    }

    #[test]
    fn avccmdtype_from() {
        assert_eq!(0x00, u8::from(AvcCmdType::from(0x00)));
        assert_eq!(0x01, u8::from(AvcCmdType::from(0x01)));
        assert_eq!(0x02, u8::from(AvcCmdType::from(0x02)));
        assert_eq!(0x03, u8::from(AvcCmdType::from(0x03)));
        assert_eq!(0x04, u8::from(AvcCmdType::from(0x04)));
    }

    #[test]
    fn avcrespcode_from() {
        assert_eq!(0x08, u8::from(AvcRespCode::from(0x08)));
        assert_eq!(0x09, u8::from(AvcRespCode::from(0x09)));
        assert_eq!(0x0a, u8::from(AvcRespCode::from(0x0a)));
        assert_eq!(0x0b, u8::from(AvcRespCode::from(0x0b)));
        assert_eq!(0x0c, u8::from(AvcRespCode::from(0x0c)));
        assert_eq!(0x0d, u8::from(AvcRespCode::from(0x0d)));
        assert_eq!(0x0e, u8::from(AvcRespCode::from(0x0e)));
        assert_eq!(0x0f, u8::from(AvcRespCode::from(0x0f)));
        assert_eq!(0xff, u8::from(AvcRespCode::from(0xff)));
    }

    #[test]
    fn ta1394avcerror_from() {
        assert_eq!(Some(Ta1394AvcError::TooShortResp), ErrorDomain::from(0));
        assert_eq!(
            Some(Ta1394AvcError::UnexpectedRespCode),
            ErrorDomain::from(1)
        );
        assert_eq!(
            Some(Ta1394AvcError::UnexpectedRespOperands),
            ErrorDomain::from(2)
        );
        assert_eq!(
            Some(Ta1394AvcError::CmdBuild(AvcCmdBuildError::InvalidAddress)),
            ErrorDomain::from(3)
        );
        assert_eq!(Some(Ta1394AvcError::Invalid(1234)), ErrorDomain::from(1234));
    }
}
