// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use super::{AvcSubunitType, AUDIO_SUBUNIT_0, AvcAddr, Ta1394AvcError};
use super::{AvcOp, AvcStatus, AvcControl};

pub const AUDIO_SUBUNIT_0_ADDR: AvcAddr = AvcAddr::Subunit(AUDIO_SUBUNIT_0);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AudioFuncBlkType {
    Selector,
    Feature,
    Processing,
    Reserved(u8),
}

impl From<u8> for AudioFuncBlkType {
    fn from(val: u8) -> Self {
        match val {
            0x80 => AudioFuncBlkType::Selector,
            0x81 => AudioFuncBlkType::Feature,
            0x82 => AudioFuncBlkType::Processing,
            _ => AudioFuncBlkType::Reserved(val),
        }
    }
}

impl From<AudioFuncBlkType> for u8 {
    fn from(func_blk_type: AudioFuncBlkType) -> Self {
        match func_blk_type {
            AudioFuncBlkType::Selector => 0x80,
            AudioFuncBlkType::Feature => 0x81,
            AudioFuncBlkType::Processing => 0x82,
            AudioFuncBlkType::Reserved(val) => val,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CtlAttr {
    Resolution,
    Minimum,
    Maximum,
    Default,
    Duration,
    Current,
    Move,
    Delta,
    Reserved(u8),
}

impl From<u8> for CtlAttr {
    fn from(val: u8) -> Self {
        match val {
            0x01 => Self::Resolution,
            0x02 => Self::Minimum,
            0x03 => Self::Maximum,
            0x04 => Self::Default,
            0x08 => Self::Duration,
            0x10 => Self::Current,
            0x18 => Self::Move,
            0x19 => Self::Delta,
            _ => Self::Reserved(val),
        }
    }
}

impl From<CtlAttr> for u8 {
    fn from(attr_type: CtlAttr) -> Self {
        match attr_type {
            CtlAttr::Resolution => 0x01,
            CtlAttr::Minimum => 0x02,
            CtlAttr::Maximum => 0x03,
            CtlAttr::Default => 0x04,
            CtlAttr::Duration => 0x08,
            CtlAttr::Current => 0x10,
            CtlAttr::Move => 0x18,
            CtlAttr::Delta => 0x19,
            CtlAttr::Reserved(val) => val,
        }
    }
}

impl std::fmt::Display for CtlAttr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            CtlAttr::Resolution => "resolution".to_owned(),
            CtlAttr::Minimum => "minimum".to_owned(),
            CtlAttr::Maximum => "maximum".to_owned(),
            CtlAttr::Default => "default".to_owned(),
            CtlAttr::Duration => "duration".to_owned(),
            CtlAttr::Current => "current".to_owned(),
            CtlAttr::Move => "move".to_owned(),
            CtlAttr::Delta => "delta".to_owned(),
            CtlAttr::Reserved(val) => format!("reserved: {}", val),
        };
        write!(f, "{}", &label)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct AudioFuncBlkCtl {
    selector: u8,
    data: Vec<u8>,
}

impl AudioFuncBlkCtl {
    fn new() -> Self {
        AudioFuncBlkCtl{
            selector: 0xff,
            data: Vec::new(),
        }
    }

    fn build_raw(&self, raw: &mut Vec<u8>) {
        raw.push(self.selector);
        if self.data.len() > 0 {
            raw.push(self.data.len() as u8);
            raw.extend_from_slice(&self.data);
        }
    }

    fn parse_raw(&mut self, raw: &[u8]) {
        self.selector = raw[0];
        self.data.clear();
        if raw.len() > 1 {
            let length = raw[1] as usize;
            if raw.len() >= 2 + length {
                self.data.extend_from_slice(&raw[2..(2 + length)]);
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct AudioFuncBlk {
    func_blk_type: AudioFuncBlkType,
    func_blk_id: u8,
    ctl_attr: CtlAttr,
    audio_selector_data: Vec<u8>,
    ctl: AudioFuncBlkCtl,
}

impl AudioFuncBlk {
    fn new(func_blk_type: AudioFuncBlkType, func_blk_id: u8, ctl_attr: CtlAttr) -> Self {
        AudioFuncBlk{
            func_blk_type,
            func_blk_id,
            ctl_attr,
            audio_selector_data: Vec::new(),
            ctl: AudioFuncBlkCtl::new(),
        }
    }

    fn build_operands(&self, addr: &AvcAddr, operands: &mut Vec<u8>) -> Result<(), Error> {
        match addr {
            AvcAddr::Unit => {
                let label = "Unit address is not supported by AudioFuncBlk";
                Err(Error::new(Ta1394AvcError::InvalidCmdOperands, &label))
            }
            AvcAddr::Subunit(s) => {
                if s.subunit_type == AvcSubunitType::Audio {
                    operands.push(self.func_blk_type.into());
                    operands.push(self.func_blk_id);
                    operands.push(self.ctl_attr.into());
                    operands.push(1 + self.audio_selector_data.len() as u8);
                    operands.extend_from_slice(&self.audio_selector_data);
                    self.ctl.build_raw(operands);
                    Ok(())
                } else {
                    let label = "SubUnit address except for audio is not supported by AudioFuncBlk";
                    Err(Error::new(Ta1394AvcError::InvalidCmdOperands, &label))
                }
            }
        }
    }

    fn parse_operands(&mut self, operands: &[u8]) -> Result<(), Error> {
        if operands.len() < 4 {
            let label = format!("Oprands too short for AudioFuncBlk; {}", operands.len());
            return Err(Error::new(Ta1394AvcError::TooShortResp, &label));
        }
        let func_blk_type = AudioFuncBlkType::from(operands[0]);
        if func_blk_type != self.func_blk_type {
            let label = format!("Unexpected function block type: {}", operands[0]);
            return Err(Error::new(Ta1394AvcError::UnexpectedRespOperands, &label));
        }

        let func_blk_id = operands[1];
        if func_blk_id != self.func_blk_id {
            let label = format!("Unexpected function block ID: {} but {}",
                                self.func_blk_id, func_blk_id);
            return Err(Error::new(Ta1394AvcError::UnexpectedRespOperands, &label));
        }

        let ctl_attr = CtlAttr::from(operands[2]);
        if ctl_attr != self.ctl_attr {
            let label = format!("Unexpected control attribute: {} but {}",
                                self.ctl_attr, ctl_attr);
            return Err(Error::new(Ta1394AvcError::UnexpectedRespOperands, &label));
        }

        let mut audio_selector_length = operands[3] as usize;
        if operands.len() < 3 + audio_selector_length {
            let label = format!("Oprands too short for selector of AudioFuncBlk; {}", operands.len());
            return Err(Error::new(Ta1394AvcError::TooShortResp, &label));
        } else if audio_selector_length < 1 {
            let label = "The length of audio selector is less thant 1:";
            return Err(Error::new(Ta1394AvcError::UnexpectedRespOperands, &label));
        }
        audio_selector_length -= 1;
        self.audio_selector_data = operands[4..(4 + audio_selector_length)].to_vec();

        self.ctl.parse_raw(&operands[(4 + audio_selector_length)..]);

        Ok(())
    }
}

impl AvcOp for AudioFuncBlk {
    const OPCODE: u8 = 0xb8;
}

impl AvcStatus for AudioFuncBlk {
    fn build_operands(&mut self, addr: &AvcAddr, operands: &mut Vec<u8>) -> Result<(), Error> {
        AudioFuncBlk::build_operands(self, addr, operands)
    }

    fn parse_operands(&mut self, _: &AvcAddr, operands: &[u8]) -> Result<(), Error> {
        AudioFuncBlk::parse_operands(self, operands)
    }
}

impl AvcControl for AudioFuncBlk {
    fn build_operands(&mut self, addr: &AvcAddr, operands: &mut Vec<u8>) -> Result<(), Error> {
        AudioFuncBlk::build_operands(self, addr, operands)
    }

    fn parse_operands(&mut self, _: &AvcAddr, operands: &[u8]) -> Result<(), Error> {
        AudioFuncBlk::parse_operands(self, operands)
    }
}

//
// AV/C Audio Subunit FUNCTION_BLOCK command for Selector function block
//
pub struct AudioSelector {
    pub input_plug_id: u8,
    func_blk: AudioFuncBlk,
}

impl AudioSelector {
    const SELECTOR_CONTROL: u8 = 0x01;

    pub fn new(func_blk_id: u8, ctl_attr: CtlAttr, input_plug_id: u8) -> Self {
        AudioSelector{
            input_plug_id,
            func_blk: AudioFuncBlk::new(AudioFuncBlkType::Selector, func_blk_id, ctl_attr),
        }
    }

    fn build_func_blk(&mut self) -> Result<(), Error> {
        self.func_blk.audio_selector_data.clear();
        self.func_blk.audio_selector_data.push(self.input_plug_id);
        self.func_blk.ctl.selector = Self::SELECTOR_CONTROL;
        self.func_blk.ctl.data.clear();
        Ok(())
    }

    fn parse_func_blk(&mut self) -> Result<(), Error> {
        if self.func_blk.ctl.selector != Self::SELECTOR_CONTROL {
            let label = format!("Unexpected control selector: {} but {}",
                                Self::SELECTOR_CONTROL, self.func_blk.ctl.selector);
            Err(Error::new(Ta1394AvcError::UnexpectedRespOperands, &label))
        } else if self.func_blk.ctl.data.len() > 0 {
            let label = format!("Unexpected length of control data: {} but {}",
                                0, self.func_blk.ctl.data.len());
            Err(Error::new(Ta1394AvcError::UnexpectedRespOperands, &label))
        } else {
            self.input_plug_id = self.func_blk.audio_selector_data[0];
            Ok(())
        }
    }
}

impl AvcOp for AudioSelector {
    const OPCODE: u8 = AudioFuncBlk::OPCODE;
}

impl AvcStatus for AudioSelector {
    fn build_operands(&mut self, addr: &AvcAddr, operands: &mut Vec<u8>) -> Result<(), Error> {
        self.build_func_blk()?;
        AvcStatus::build_operands(&mut self.func_blk, addr, operands)
    }

    fn parse_operands(&mut self, addr: &AvcAddr, operands: &[u8]) -> Result<(), Error> {
        AvcStatus::parse_operands(&mut self.func_blk, addr, operands)?;
        self.parse_func_blk()
    }
}

impl AvcControl for AudioSelector {
    fn build_operands(&mut self, addr: &AvcAddr, operands: &mut Vec<u8>) -> Result<(), Error> {
        self.build_func_blk()?;
        AvcControl::build_operands(&mut self.func_blk, addr, operands)
    }

    fn parse_operands(&mut self, addr: &AvcAddr, operands: &[u8]) -> Result<(), Error> {
        AvcControl::parse_operands(&mut self.func_blk, addr, operands)?;
        self.parse_func_blk()
    }
}

#[cfg(test)]
mod test {
    use crate::ta1394::AvcAddr;
    use crate::ta1394::{AvcStatus, AvcControl};
    use super::{AUDIO_SUBUNIT_0_ADDR, AudioFuncBlk, AudioFuncBlkType, CtlAttr};
    use super::AudioSelector;

    #[test]
    fn func_blk_operands() {
        let mut op = AudioFuncBlk::new(AudioFuncBlkType::Selector, 0xfe, CtlAttr::Resolution);
        op.audio_selector_data.extend_from_slice(&[0xde, 0xad, 0xbe, 0xef]);
        op.ctl.selector = 0x11;
        op.ctl.data.extend_from_slice(&[0xbe, 0xef]);

        let mut operands = Vec::new();
        AvcStatus::build_operands(&mut op, &AUDIO_SUBUNIT_0_ADDR, &mut operands).unwrap();
        assert_eq!(&operands, &[0x80, 0xfe, 0x01, 0x05, 0xde, 0xad, 0xbe, 0xef, 0x11, 0x02, 0xbe, 0xef]);

        AvcStatus::parse_operands(&mut op, &AvcAddr::Unit, &operands).unwrap();
        assert_eq!(op.func_blk_type, AudioFuncBlkType::Selector);
        assert_eq!(op.func_blk_id, 0xfe);
        assert_eq!(op.ctl_attr, CtlAttr::Resolution);
        assert_eq!(&op.audio_selector_data, &[0xde, 0xad, 0xbe, 0xef]);
        assert_eq!(op.ctl.selector, 0x11);
        assert_eq!(&op.ctl.data, &[0xbe, 0xef]);

        let mut op = AudioFuncBlk::new(AudioFuncBlkType::Selector, 0xfd, CtlAttr::Minimum);
        op.audio_selector_data.extend_from_slice(&[0xde, 0xad, 0xbe, 0xef]);
        op.ctl.selector = 0x12;
        op.ctl.data.extend_from_slice(&[0xbe, 0xef]);

        let mut operands = Vec::new();
        AvcControl::build_operands(&mut op, &AUDIO_SUBUNIT_0_ADDR, &mut operands).unwrap();
        assert_eq!(&operands, &[0x80, 0xfd, 0x02, 0x05, 0xde, 0xad, 0xbe, 0xef, 0x12, 0x02, 0xbe, 0xef]);

        AvcControl::parse_operands(&mut op, &AvcAddr::Unit, &operands).unwrap();
        assert_eq!(op.func_blk_type, AudioFuncBlkType::Selector);
        assert_eq!(op.func_blk_id, 0xfd);
        assert_eq!(op.ctl_attr, CtlAttr::Minimum);
        assert_eq!(&op.audio_selector_data, &[0xde, 0xad, 0xbe, 0xef]);
        assert_eq!(op.ctl.selector, 0x12);
        assert_eq!(&op.ctl.data, &[0xbe, 0xef]);

        // For the case that audio_selector_data is empty.
        let mut op = AudioFuncBlk::new(AudioFuncBlkType::Feature, 0xfc, CtlAttr::Maximum);
        op.ctl.selector = 0x13;
        op.ctl.data.extend_from_slice(&[0xfe, 0xeb, 0xda, 0xed]);

        let mut operands = Vec::new();
        AvcStatus::build_operands(&mut op, &AUDIO_SUBUNIT_0_ADDR, &mut operands).unwrap();
        assert_eq!(&operands, &[0x81, 0xfc, 0x03, 0x01, 0x13, 0x04, 0xfe, 0xeb, 0xda, 0xed]);

        AvcStatus::parse_operands(&mut op, &AvcAddr::Unit, &operands).unwrap();
        assert_eq!(op.func_blk_type, AudioFuncBlkType::Feature);
        assert_eq!(op.func_blk_id, 0xfc);
        assert_eq!(op.ctl_attr, CtlAttr::Maximum);
        assert_eq!(&op.audio_selector_data, &[]);
        assert_eq!(op.ctl.selector, 0x13);
        assert_eq!(&op.ctl.data, &[0xfe, 0xeb, 0xda, 0xed]);

        let mut op = AudioFuncBlk::new(AudioFuncBlkType::Feature, 0xfb, CtlAttr::Default);
        op.ctl.selector = 0x14;
        op.ctl.data.extend_from_slice(&[0xfe, 0xeb, 0xda, 0xed]);

        let mut operands = Vec::new();
        AvcControl::build_operands(&mut op, &AUDIO_SUBUNIT_0_ADDR, &mut operands).unwrap();
        assert_eq!(&operands, &[0x81, 0xfb, 0x04, 0x01, 0x14, 0x04, 0xfe, 0xeb, 0xda, 0xed]);

        AvcControl::parse_operands(&mut op, &AvcAddr::Unit, &operands).unwrap();
        assert_eq!(op.func_blk_type, AudioFuncBlkType::Feature);
        assert_eq!(op.func_blk_id, 0xfb);
        assert_eq!(op.ctl_attr, CtlAttr::Default);
        assert_eq!(&op.audio_selector_data, &[]);
        assert_eq!(op.ctl.selector, 0x14);
        assert_eq!(&op.ctl.data, &[0xfe, 0xeb, 0xda, 0xed]);

        // For the case that ctl_data is empty.
        let mut op = AudioFuncBlk::new(AudioFuncBlkType::Processing, 0xfa, CtlAttr::Duration);
        op.audio_selector_data.extend_from_slice(&[0xda, 0xed]);
        op.ctl.selector = 0x15;

        let mut operands = Vec::new();
        AvcStatus::build_operands(&mut op, &AUDIO_SUBUNIT_0_ADDR, &mut operands).unwrap();
        assert_eq!(&operands, &[0x82, 0xfa, 0x08, 0x03, 0xda, 0xed, 0x15]);

        AvcStatus::parse_operands(&mut op, &AvcAddr::Unit, &operands).unwrap();
        assert_eq!(op.func_blk_type, AudioFuncBlkType::Processing);
        assert_eq!(op.func_blk_id, 0xfa);
        assert_eq!(op.ctl_attr, CtlAttr::Duration);
        assert_eq!(&op.audio_selector_data, &[0xda, 0xed]);
        assert_eq!(op.ctl.selector, 0x15);
        assert_eq!(&op.ctl.data, &[]);

        let mut op = AudioFuncBlk::new(AudioFuncBlkType::Processing, 0xf9, CtlAttr::Current);
        op.audio_selector_data.extend_from_slice(&[0xda, 0xed]);
        op.ctl.selector = 0x16;

        let mut operands = Vec::new();
        AvcControl::build_operands(&mut op, &AUDIO_SUBUNIT_0_ADDR, &mut operands).unwrap();
        assert_eq!(&operands, &[0x82, 0xf9, 0x10, 0x03, 0xda, 0xed, 0x16]);

        AvcControl::parse_operands(&mut op, &AvcAddr::Unit, &operands).unwrap();
        assert_eq!(op.func_blk_type, AudioFuncBlkType::Processing);
        assert_eq!(op.func_blk_id, 0xf9);
        assert_eq!(op.ctl_attr, CtlAttr::Current);
        assert_eq!(&op.audio_selector_data, &[0xda, 0xed]);
        assert_eq!(op.ctl.selector, 0x16);
        assert_eq!(&op.ctl.data, &[]);
    }

    #[test]
    fn avcaudioselector_operands() {
        let mut op = AudioSelector::new(0xe5, CtlAttr::Duration, 0x28);
        let mut operands = Vec::new();
        AvcStatus::build_operands(&mut op, &AUDIO_SUBUNIT_0_ADDR, &mut operands).unwrap();
        assert_eq!(&operands, &[0x80, 0xe5, 0x08, 0x02, 0x28, 0x01]);

        AvcStatus::parse_operands(&mut op, &AvcAddr::Unit, &operands).unwrap();
        assert_eq!(op.input_plug_id, 0x28);

        let mut op = AudioSelector::new(0x1e, CtlAttr::Move, 0x96);
        let mut operands = Vec::new();
        AvcControl::build_operands(&mut op, &AUDIO_SUBUNIT_0_ADDR, &mut operands).unwrap();
        assert_eq!(&operands, &[0x80, 0x1e, 0x18, 0x02, 0x96, 0x01]);

        AvcControl::parse_operands(&mut op, &AvcAddr::Unit, &operands).unwrap();
        assert_eq!(op.input_plug_id, 0x96);
    }
}
