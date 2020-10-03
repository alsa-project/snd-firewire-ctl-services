// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
pub mod config_rom;

use glib::Error;

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

impl From<AvcSubunitType> for u8 {
    fn from(subunit_type: AvcSubunitType) -> Self {
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
            AvcSubunitType::Reserved(value) => value,
        }
    }
}

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
        AvcAddrSubunit{subunit_type, subunit_id}
    }
}

impl From<u8> for AvcAddrSubunit {
    fn from(val: u8) -> Self {
        let subunit_type = AvcSubunitType::from((val >> Self::SUBUNIT_TYPE_SHIFT) & Self::SUBUNIT_TYPE_MASK);
        let subunit_id = (val >> Self::SUBUNIT_ID_SHIFT) & Self::SUBUNIT_ID_MASK;
        AvcAddrSubunit{subunit_type, subunit_id}
    }
}

impl From<AvcAddrSubunit> for u8 {
    fn from(subunit: AvcAddrSubunit) -> u8 {
        let mut val = u8::from(subunit.subunit_type);
        val = (val & AvcAddrSubunit::SUBUNIT_TYPE_MASK) << AvcAddrSubunit::SUBUNIT_TYPE_SHIFT;
        val |= (subunit.subunit_id & AvcAddrSubunit::SUBUNIT_ID_MASK) << AvcAddrSubunit::SUBUNIT_ID_SHIFT;
        val
    }
}

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

impl From<AvcAddr> for u8 {
    fn from(addr: AvcAddr) -> Self {
        match addr {
            AvcAddr::Unit => AvcAddr::UNIT_ADDR,
            AvcAddr::Subunit(d) => u8::from(d),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AvcCmdType {
    Control,
    Status,
    SpecificInquiry,
    Notify,
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

#[derive(Debug, Eq, PartialEq)]
pub enum AvcRespCode {
    NotImplemented,
    Accepted,
    Rejected,
    InTransition,
    ImplementedStable,
    Changed,
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
    fn from(resp: AvcRespCode) -> u8 {
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

pub trait AvcOp {
    const OPCODE: u8;

    fn opcode(&self) -> u8 {
        Self::OPCODE
    }
}

pub trait AvcControl {
    fn build_operands(&mut self, addr: &AvcAddr, operands: &mut Vec<u8>) -> Result<(), Error>;
    fn parse_operands(&mut self, addr: &AvcAddr, operands: &[u8]) -> Result<(), Error>;
}

pub trait AvcStatus {
    fn build_operands(&mut self, addr: &AvcAddr, operands: &mut Vec<u8>) -> Result<(), Error>;
    fn parse_operands(&mut self, addr: &AvcAddr, operands: &[u8]) -> Result<(), Error>;
}

pub trait AvcNotify {
    fn build_operands(&mut self, addr: &AvcAddr, operands: &mut Vec<u8>) -> Result<(), Error>;
    fn parse_operands(&mut self, addr: &AvcAddr, operands: &[u8]) -> Result<(), Error>;
}

#[cfg(test)]
mod test {
    use super::{AvcSubunitType, AvcAddrSubunit, AvcAddr};
    use super::{AvcCmdType, AvcRespCode};

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
}
