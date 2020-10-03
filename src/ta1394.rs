// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
pub mod config_rom;

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

#[cfg(test)]
mod test {
    use super::AvcSubunitType;

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
}
