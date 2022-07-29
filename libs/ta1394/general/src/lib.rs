// SPDX-License-Identifier: MIT
// Copyright (c) 2022 Takashi Sakamoto

#![doc = include_str!("../README.md")]

pub mod config_rom;
pub mod general;

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
    pub const UNIT_ADDR: u8 = 0xff;
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

/// The error to build command frame for AV/C transaction.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum AvcRespParseError {
    /// The length of response frame is shorter than expected.
    TooShortResp(
        /// The expected length at least.
        usize,
    ),
    /// The status code in response frame is not expected.
    UnexpectedStatus,
    /// Any of operand in response frame is not expected.
    UnexpectedOperands(
        /// The first offset for unexpected operand.
        usize,
    ),
}

impl std::fmt::Display for AvcRespParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TooShortResp(expected) => write!(f, "response frame too short {}", expected),
            Self::UnexpectedStatus => write!(f, "unexpected response status"),
            Self::UnexpectedOperands(offset) => {
                write!(f, "unexpected response operands at {}", offset)
            }
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
    fn parse_operands(&mut self, addr: &AvcAddr, operands: &[u8]) -> Result<(), AvcRespParseError>;
}

/// The AV/C operation supporting status command.
pub trait AvcStatus {
    fn build_operands(
        &mut self,
        addr: &AvcAddr,
        operands: &mut Vec<u8>,
    ) -> Result<(), AvcCmdBuildError>;
    fn parse_operands(&mut self, addr: &AvcAddr, operands: &[u8]) -> Result<(), AvcRespParseError>;
}

/// The AV/C operation supporting notify command.
pub trait AvcNotify {
    fn build_operands(
        &mut self,
        addr: &AvcAddr,
        operands: &mut Vec<u8>,
    ) -> Result<(), AvcCmdBuildError>;
    fn parse_operands(&mut self, addr: &AvcAddr, operands: &[u8]) -> Result<(), AvcRespParseError>;
}

/// For error reporting of AV/C transaction.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Ta1394AvcError<T: std::fmt::Display + Clone> {
    /// Fail to build command frame.
    CmdBuild(AvcCmdBuildError),
    /// Fail to initiate and finish AV/C transaction by Function Control Protocol.
    CommunicationFailure(T),
    /// Fail to parse response frame.
    RespParse(AvcRespParseError),
}

impl<T: std::fmt::Display + Clone> std::fmt::Display for Ta1394AvcError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CmdBuild(cause) => write!(f, "Fail to build command frame: {}", cause),
            Self::CommunicationFailure(cause) => write!(f, "Fail to communicate: {}", cause),
            Self::RespParse(cause) => write!(f, "Fail to parse response frame: {}", cause),
        }
    }
}

/// For AV/C transaction defined by 1394 Trading Association.
pub trait Ta1394Avc<T: std::fmt::Display + Clone> {
    /// The maximum size of frame in both command and response.
    const FRAME_SIZE: usize = 0x200;

    /// The mask for first byte of response frame to detect status code. The rest bits express
    /// Command/transaction set (CTS) but appears not to be used actually.
    const RESP_CODE_MASK: u8 = 0x0f;

    /// Transmit given command frame and return received response frame.
    ///
    /// Initiate request transaction and wait for response transaction by Function Control
    /// Protocol (FCP) in IEC 61883-1. When detecting `AvcRespCode::INTERIM` in received response
    /// frame, wait for further response transaction as final result, according to "deferred
    /// transaction" in AV/C general specification.
    ///
    /// The call of method is expected to yield running processor to wait for the response.
    fn transaction(&self, command_frame: &[u8], timeout_ms: u32) -> Result<Vec<u8>, T>;

    fn compose_command_frame(
        ctype: AvcCmdType,
        addr: &AvcAddr,
        opcode: u8,
        operands: &[u8],
    ) -> Vec<u8> {
        let mut frame = Vec::new();
        frame.push(ctype.into());
        frame.push(addr.into());
        frame.push(opcode);
        frame.extend_from_slice(operands);
        frame
    }

    fn detect_response_operands<'a>(
        frame: &'a [u8],
        addr: &AvcAddr,
        opcode: u8,
    ) -> Result<(AvcRespCode, &'a [u8]), AvcRespParseError> {
        if frame[1] != addr.into() {
            Err(AvcRespParseError::UnexpectedStatus)
        } else if frame[2] != opcode {
            Err(AvcRespParseError::UnexpectedStatus)
        } else {
            let rcode = AvcRespCode::from(frame[0] & Self::RESP_CODE_MASK);
            let operands = &frame[3..];
            Ok((rcode, operands))
        }
    }

    fn control<O: AvcOp + AvcControl>(
        &self,
        addr: &AvcAddr,
        op: &mut O,
        timeout_ms: u32,
    ) -> Result<(), Ta1394AvcError<T>> {
        let mut operands = Vec::new();
        let command_frame = AvcControl::build_operands(op, addr, &mut operands)
            .map_err(|err| Ta1394AvcError::CmdBuild(err))
            .map(|_| {
                Self::compose_command_frame(AvcCmdType::Control, addr, O::OPCODE, &operands)
            })?;
        self.transaction(&command_frame, timeout_ms)
            .map_err(|cause| Ta1394AvcError::CommunicationFailure(cause))
            .and_then(|response_frame| {
                Self::detect_response_operands(&response_frame, addr, O::OPCODE)
                    .and_then(|(rcode, operands)| match rcode {
                        AvcRespCode::Accepted => AvcControl::parse_operands(op, addr, &operands),
                        _ => Err(AvcRespParseError::UnexpectedStatus),
                    })
                    .map_err(|err| Ta1394AvcError::RespParse(err))
            })
    }

    fn status<O: AvcOp + AvcStatus>(
        &self,
        addr: &AvcAddr,
        op: &mut O,
        timeout_ms: u32,
    ) -> Result<(), Ta1394AvcError<T>> {
        let mut operands = Vec::new();
        let command_frame = AvcStatus::build_operands(op, addr, &mut operands)
            .map_err(|err| Ta1394AvcError::CmdBuild(err))
            .map(|_| Self::compose_command_frame(AvcCmdType::Status, addr, O::OPCODE, &operands))?;
        self.transaction(&command_frame, timeout_ms)
            .map_err(|cause| Ta1394AvcError::CommunicationFailure(cause))
            .and_then(|response_frame| {
                Self::detect_response_operands(&response_frame, addr, O::OPCODE)
                    .and_then(|(rcode, operands)| match rcode {
                        AvcRespCode::ImplementedStable => {
                            AvcStatus::parse_operands(op, addr, &operands)
                        }
                        _ => Err(AvcRespParseError::UnexpectedStatus),
                    })
                    .map_err(|err| Ta1394AvcError::RespParse(err))
            })
    }

    fn specific_inquiry<O: AvcOp + AvcControl>(
        &self,
        addr: &AvcAddr,
        op: &mut O,
        timeout_ms: u32,
    ) -> Result<(), Ta1394AvcError<T>> {
        let mut operands = Vec::new();
        let command_frame = AvcControl::build_operands(op, addr, &mut operands)
            .map_err(|err| Ta1394AvcError::CmdBuild(err))
            .map(|_| {
                Self::compose_command_frame(AvcCmdType::SpecificInquiry, addr, O::OPCODE, &operands)
            })?;
        self.transaction(&command_frame, timeout_ms)
            .map_err(|cause| Ta1394AvcError::CommunicationFailure(cause))
            .and_then(|response_frame| {
                Self::detect_response_operands(&response_frame, addr, O::OPCODE)
                    .and_then(|(rcode, operands)| match rcode {
                        AvcRespCode::ImplementedStable => {
                            AvcControl::parse_operands(op, addr, &operands)
                        }
                        _ => Err(AvcRespParseError::UnexpectedStatus),
                    })
                    .map_err(|err| Ta1394AvcError::RespParse(err))
            })
    }

    fn notify<O: AvcOp + AvcNotify>(
        &self,
        addr: &AvcAddr,
        op: &mut O,
        timeout_ms: u32,
    ) -> Result<(), Ta1394AvcError<T>> {
        let mut operands = Vec::new();
        let command_frame = AvcNotify::build_operands(op, addr, &mut operands)
            .map_err(|err| Ta1394AvcError::CmdBuild(err))
            .map(|_| Self::compose_command_frame(AvcCmdType::Notify, addr, O::OPCODE, &operands))?;
        self.transaction(&command_frame, timeout_ms)
            .map_err(|cause| Ta1394AvcError::CommunicationFailure(cause))
            .and_then(|response_frame| {
                Self::detect_response_operands(&response_frame, addr, O::OPCODE)
                    .and_then(|(rcode, operands)| match rcode {
                        AvcRespCode::Changed => AvcNotify::parse_operands(op, addr, &operands),
                        _ => Err(AvcRespParseError::UnexpectedStatus),
                    })
                    .map_err(|err| Ta1394AvcError::RespParse(err))
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
        assert_eq!(
            AvcAddr::from(0x09),
            AvcAddr::Subunit(AvcAddrSubunit::new(AvcSubunitType::Audio, 0x01))
        );
        assert_eq!(
            AvcAddr::from(0x63),
            AvcAddr::Subunit(AvcAddrSubunit::new(AvcSubunitType::Music, 0x03))
        );
        assert_eq!(
            AvcAddr::from(0x87),
            AvcAddr::Subunit(AvcAddrSubunit::new(AvcSubunitType::Reserved(0x10), 0x07))
        );
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
}
