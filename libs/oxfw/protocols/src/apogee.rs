// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol defined by Apogee Electronics for Duet FireWire.
//!
//! The module includes protocol implementation defined by Apogee Electronics for Duet FireWire.

use glib::Error;

use hinawa::FwReqExtManual;

use ta1394::{Ta1394AvcError, AvcAddr};
use ta1394::{AvcOp, AvcStatus, AvcControl};
use ta1394::general::VendorDependent;

/// The enumeration to represent type of command for Apogee Duet FireWire.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
// Usually 5 params.
pub enum VendorCmd {
    MicPolarity(u8),
    PhoneInLine(u8),
    LineInLevel(u8),
    MicPhantom(u8),
    OutAttr,
    InGain(u8),
    HwState,        // 11 parameters.
    OutMute,
    MicIn(u8),
    MixerSrc(u8, u8),       // 7 params (0x0a, 0x10)
    UseMixerOut,
    DisplayOverhold,
    DisplayClear,
    OutVolume,
    MuteForLineOut,
    MuteForHpOut,
    UnmuteForLineOut,
    UnmuteForHpOut,
    DisplayInput,
    InClickless,
    DisplayFollow,
}

/// The structure to represent protocol of Apogee Duet FireWire.
pub struct ApogeeCmd{
    cmd: VendorCmd,
    vals: Vec<u8>,
    op: VendorDependent,
}

impl<'a> ApogeeCmd {
    const APOGEE_PREFIX: &'a [u8] = &[0x50, 0x43, 0x4d];    // 'P', 'C', 'M'

    const MIC_POLARITY: u8 = 0x00;
    const PHONE_IN_LEVEL: u8 = 0x01;
    const LINE_IN_LEVEL: u8 = 0x02;
    const MIC_PHANTOM: u8 = 0x03;
    const OUT_ATTR: u8 = 0x04;
    const IN_GAIN: u8 = 0x05;
    const HW_STATE: u8 = 0x07;
    const OUT_MUTE: u8 = 0x09;
    const USE_LINE_IN: u8 = 0x0c;
    const MIXER_SRC: u8 = 0x10;
    const USE_MIXER_OUT: u8 = 0x11;
    const DISPLAY_OVERHOLD: u8 = 0x13;
    const DISPLAY_CLEAR: u8 = 0x14;
    const OUT_VOLUME: u8 = 0x15;
    const MUTE_FOR_LINE_OUT: u8 = 0x16;
    const MUTE_FOR_HP_OUT: u8 = 0x17;
    const UNMUTE_FOR_LINE_OUT: u8 = 0x18;
    const UNMUTE_FOR_HP_OUT: u8 = 0x19;
    const DISPLAY_INPUT: u8 = 0x1b;
    const IN_CLICKLESS: u8 = 0x1e;
    const DISPLAY_FOLLOW: u8 = 0x22;

    const ON: u8 = 0x70;
    const OFF: u8 = 0x60;

    pub fn new(company_id: &[u8;3], cmd: VendorCmd) -> Self {
        ApogeeCmd{
            cmd,
            vals: Vec::new(),
            op: VendorDependent::new(company_id),
        }
    }

    fn build_args(&self) -> Vec<u8> {
        let mut args = Vec::with_capacity(6);
        args.extend_from_slice(&Self::APOGEE_PREFIX);
        args.extend_from_slice(&[0xff;3]);

        match &self.cmd {
            VendorCmd::MicPolarity(ch) => {
                args[3] = Self::MIC_POLARITY;
                args[4] = 0x80;
                args[5] = *ch;
            }
            VendorCmd::PhoneInLine(ch) => {
                args[3] = Self::PHONE_IN_LEVEL;
                args[4] = 0x80;
                args[5] = *ch;
            }
            VendorCmd::LineInLevel(ch) => {
                args[3] = Self::LINE_IN_LEVEL;
                args[4] = 0x80;
                args[5] = *ch;
            }
            VendorCmd::MicPhantom(ch) => {
                args[3] = Self::MIC_PHANTOM;
                args[4] = 0x80;
                args[5] = *ch;
            }
            VendorCmd::OutAttr => {
                args[3] = Self::OUT_ATTR;
                args[4] = 0x80;
            }
            VendorCmd::InGain(ch) => {
                args[3] = Self::IN_GAIN;
                args[4] = 0x80;
                args[5] = *ch;
            }
            VendorCmd::HwState => args[3] = Self::HW_STATE,
            VendorCmd::OutMute => {
                args[3] = Self::OUT_MUTE;
                args[4] = 0x80;
            }
            VendorCmd::MicIn(ch) => {
                args[3] = Self::USE_LINE_IN;
                args[4] = 0x80;
                args[5] = *ch;
            }
            VendorCmd::MixerSrc(src, dst) => {
                args[3] = Self::MIXER_SRC;
                args[4] = (((*src / 2) << 4) | (*src % 2)) as u8;
                args[5] = *dst;
            }
            VendorCmd::UseMixerOut => args[3] = Self::USE_MIXER_OUT,
            VendorCmd::DisplayOverhold => args[3] = Self::DISPLAY_OVERHOLD,
            VendorCmd::DisplayClear => args[3] = Self::DISPLAY_CLEAR,
            VendorCmd::OutVolume => {
                args[3] = Self::OUT_VOLUME;
                args[4] = 0x80;
            }
            VendorCmd::MuteForLineOut => {
                args[3] = Self::MUTE_FOR_LINE_OUT;
                args[4] = 0x80;
            }
            VendorCmd::MuteForHpOut => {
                args[3] = Self::MUTE_FOR_HP_OUT;
                args[4] = 0x80;
            }
            VendorCmd::UnmuteForLineOut => {
                args[3] = Self::UNMUTE_FOR_LINE_OUT;
                args[4] = 0x80;
            }
            VendorCmd::UnmuteForHpOut => {
                args[3] = Self::UNMUTE_FOR_HP_OUT;
                args[4] = 0x80;
            }
            VendorCmd::DisplayInput=> args[3] = Self::DISPLAY_INPUT,
            VendorCmd::InClickless => args[3] = Self::IN_CLICKLESS,
            VendorCmd::DisplayFollow => args[3] = Self::DISPLAY_FOLLOW,
        }

        args
    }

    fn build_data(&mut self) -> Result<(), Error> {
        self.op.data = self.build_args();
        self.op.data.extend_from_slice(&self.vals);
        Ok(())
    }

    fn parse_data(&mut self) -> Result<(), Error> {
        let args = self.build_args();
        if self.op.data[..6] != args[..6] {
            let label = format!("Unexpected arguments in response: {:?} but {:?}", args, self.op.data);
            Err(Error::new(Ta1394AvcError::UnexpectedRespOperands, &label))
        } else {
            self.vals = self.op.data.split_off(6);
            Ok(())
        }
    }

    pub fn get_enum(&self) -> u32 {
        assert!(self.vals.len() > 0, "Unexpected read operation as bool argument.");
        (self.vals[0] == Self::ON) as u32
    }

    pub fn put_enum(&mut self, val: u32) {
        assert!(self.vals.len() == 0, "Unexpected write operation as bool argument.");
        self.vals.push(if val > 0 { Self::ON } else { Self::OFF })
    }

    pub fn read_u16(&self) -> u16 {
        assert!(self.vals.len() > 0, "Unexpected read operation as bool argument.");
        let mut doublet = [0;2];
        doublet.copy_from_slice(&self.vals[..2]);
        u16::from_be_bytes(doublet)
    }

    pub fn write_u16(&mut self, val: u16) {
        assert!(self.vals.len() == 0, "Unexpected write operation as u16 argument.");
        self.vals.extend_from_slice(&val.to_be_bytes());
    }

    pub fn get_u8(&self) -> u8 {
        assert!(self.vals.len() > 0, "Unexpected read operation as u8 argument.");
        self.vals[0]
    }

    pub fn put_u8(&mut self, val: u8) {
        assert!(self.vals.len() == 0, "Unexpected write operation as u8 argument.");
        self.vals.push(val);
    }

    pub fn copy_block(&self, data: &mut [u8;8]) {
        data.copy_from_slice(&self.vals[..8]);
    }
}

impl AvcOp for ApogeeCmd {
    const OPCODE: u8 = VendorDependent::OPCODE;
}

impl AvcControl for ApogeeCmd {
    fn build_operands(&mut self, addr: &AvcAddr, operands: &mut Vec<u8>) -> Result<(), Error> {
        self.build_data()?;
        AvcControl::build_operands(&mut self.op, addr, operands)
    }

    fn parse_operands(&mut self, addr: &AvcAddr, operands: &[u8]) -> Result<(), Error> {
        AvcControl::parse_operands(&mut self.op, addr, operands)?;
        self.parse_data()
    }
}

impl AvcStatus for ApogeeCmd {
    fn build_operands(&mut self, addr: &AvcAddr, operands: &mut Vec<u8>) -> Result<(), Error> {
        self.build_data()?;
        AvcStatus::build_operands(&mut self.op, addr, operands)
    }

    fn parse_operands(&mut self, addr: &AvcAddr, operands: &[u8]) -> Result<(), Error> {
        AvcStatus::parse_operands(&mut self.op, addr, operands)?;
        self.parse_data()
    }
}

const METER_ADDR_BASE: u64 = 0xfffff0080000;

const METER_OFFSET_ANALOG_INPUT: u32 = 0x0004;
const METER_ANALOG_INPUT_SIZE: usize = 8;

const METER_OFFSET_MIXER_SRC: u32 = 0x0404;
const METER_MIXER_SRC_SIZE: usize = 16;

/// The trait to represent meter protocol of Apogee Duet FireWire.
pub trait ApogeeMeterProtocol : AsRef<hinawa::FwReq> {
    fn read_meters(&self, node: &hinawa::FwNode, meters: &mut [u32;6]) -> Result<(), Error> {
        let mut frame = [0;METER_ANALOG_INPUT_SIZE + METER_MIXER_SRC_SIZE];

        let addr = METER_ADDR_BASE + METER_OFFSET_ANALOG_INPUT as u64;
        self.as_ref().transaction_sync(node, hinawa::FwTcode::ReadBlockRequest, addr, METER_ANALOG_INPUT_SIZE,
                              &mut frame[..METER_ANALOG_INPUT_SIZE], 10)?;

        let addr = METER_ADDR_BASE + METER_OFFSET_MIXER_SRC as u64;
        self.as_ref().transaction_sync(node, hinawa::FwTcode::ReadBlockRequest, addr, METER_MIXER_SRC_SIZE,
            &mut frame[METER_ANALOG_INPUT_SIZE..(METER_ANALOG_INPUT_SIZE + METER_MIXER_SRC_SIZE)], 10)?;

        meters.iter_mut().enumerate().for_each(|(i, meter)| {
            let mut quadlet = [0;4];
            quadlet.copy_from_slice(&frame[(i * 4)..(i * 4 + 4)]);
            *meter = u32::from_be_bytes(quadlet);
        });

        Ok(())
    }
}

impl<O: AsRef<hinawa::FwReq>> ApogeeMeterProtocol for O {}

#[cfg(test)]
mod test {
    use ta1394::AvcAddr;
    use ta1394::{AvcStatus, AvcControl};
    use super::{ApogeeCmd, VendorCmd};

    #[test]
    fn apogee_cmd_proto_operands() {
        // No argument command.
        let mut op = ApogeeCmd::new(&[0x01, 0x23, 0x45], VendorCmd::UseMixerOut);
        let operands = [0x01, 0x23, 0x45, 0x50, 0x43, 0x4d, 0x11, 0xff, 0xff, 0xe3];
        AvcStatus::parse_operands(&mut op, &AvcAddr::Unit, &operands).unwrap();
        assert_eq!(op.vals, &[0xe3]);

        let mut o = Vec::new();
        AvcStatus::build_operands(&mut op, &AvcAddr::Unit, &mut o).unwrap();
        assert_eq!(o, operands);

        let mut op = ApogeeCmd::new(&[0x54, 0x32, 0x10], VendorCmd::UseMixerOut);
        let operands = [0x54, 0x32, 0x10, 0x50, 0x43, 0x4d, 0x11, 0xff, 0xff, 0xe3];
        AvcControl::parse_operands(&mut op, &AvcAddr::Unit, &operands).unwrap();
        assert_eq!(op.vals, &[0xe3]);

        let mut o = Vec::new();
        AvcControl::build_operands(&mut op, &AvcAddr::Unit, &mut o).unwrap();
        assert_eq!(o, operands);

        // One argument command.
        let mut op = ApogeeCmd::new(&[0x01, 0x23, 0x45], VendorCmd::LineInLevel(1));
        let operands = [0x01, 0x23, 0x45, 0x50, 0x43, 0x4d, 0x02, 0x80, 0x01, 0xb9];
        AvcStatus::parse_operands(&mut op, &AvcAddr::Unit, &operands).unwrap();
        assert_eq!(op.vals, &[0xb9]);

        let mut o = Vec::new();
        AvcStatus::build_operands(&mut op, &AvcAddr::Unit, &mut o).unwrap();
        assert_eq!(o, operands);

        let mut op = ApogeeCmd::new(&[0x54, 0x32, 0x10], VendorCmd::LineInLevel(1));
        let operands = [0x54, 0x32, 0x10, 0x50, 0x43, 0x4d, 0x02, 0x80, 0x01, 0xb9];
        AvcControl::parse_operands(&mut op, &AvcAddr::Unit, &operands).unwrap();
        assert_eq!(op.vals, &[0xb9]);

        let mut o = Vec::new();
        AvcControl::build_operands(&mut op, &AvcAddr::Unit, &mut o).unwrap();
        assert_eq!(o, operands);

        // Two arguments command.
        let mut op = ApogeeCmd::new(&[0x01, 0x23, 0x45], VendorCmd::MixerSrc(1, 0));
        let operands = [0x01, 0x23, 0x45, 0x50, 0x43, 0x4d, 0x10, 0x01, 0x00, 0xde, 0x00, 0xad, 0x01, 0xbe, 0x02, 0xef];
        AvcStatus::parse_operands(&mut op, &AvcAddr::Unit, &operands).unwrap();
        assert_eq!(op.vals, &[0xde, 0x00, 0xad, 0x01, 0xbe, 0x02, 0xef]);

        let mut o = Vec::new();
        AvcStatus::build_operands(&mut op, &AvcAddr::Unit, &mut o).unwrap();
        assert_eq!(o, operands);

        let mut op = ApogeeCmd::new(&[0x54, 0x32, 0x10], VendorCmd::MixerSrc(1, 0));
        let operands = [0x54, 0x32, 0x10, 0x50, 0x43, 0x4d, 0x10, 0x01, 0x00, 0xde, 0x00, 0xad, 0x01, 0xbe, 0x02, 0xef];
        AvcControl::parse_operands(&mut op, &AvcAddr::Unit, &operands).unwrap();
        assert_eq!(op.vals, &[0xde, 0x00, 0xad, 0x01, 0xbe, 0x02, 0xef]);

        let mut o = Vec::new();
        AvcControl::build_operands(&mut op, &AvcAddr::Unit, &mut o).unwrap();
        assert_eq!(o, operands);

        // Command for block request.
        let mut op = ApogeeCmd::new(&[0x01, 0x23, 0x45], VendorCmd::HwState);
        let operands = [0x01, 0x23, 0x45, 0x50, 0x43, 0x4d, 0x07, 0xff, 0xff,
                        0xde, 0x00, 0xad, 0x01, 0xbe, 0x02, 0xef, 0xde, 0xad, 0xbe, 0xef];
        AvcStatus::parse_operands(&mut op, &AvcAddr::Unit, &operands).unwrap();
        assert_eq!(op.vals, &[0xde, 0x00, 0xad, 0x01, 0xbe, 0x02, 0xef, 0xde, 0xad, 0xbe, 0xef]);

        let mut o = Vec::new();
        AvcStatus::build_operands(&mut op, &AvcAddr::Unit, &mut o).unwrap();
        assert_eq!(o, operands);

        let mut op = ApogeeCmd::new(&[0x54, 0x32, 0x10], VendorCmd::HwState);
        let operands = [0x54, 0x32, 0x10, 0x50, 0x43, 0x4d, 0x07, 0xff, 0xff,
                        0xde, 0x00, 0xad, 0x01, 0xbe, 0x02, 0xef, 0xde, 0xad, 0xbe, 0xef];
        AvcControl::parse_operands(&mut op, &AvcAddr::Unit, &operands).unwrap();
        assert_eq!(op.vals, &[0xde, 0x00, 0xad, 0x01, 0xbe, 0x02, 0xef, 0xde, 0xad, 0xbe, 0xef]);

        let mut o = Vec::new();
        AvcControl::build_operands(&mut op, &AvcAddr::Unit, &mut o).unwrap();
        assert_eq!(o, operands);

    }
}
