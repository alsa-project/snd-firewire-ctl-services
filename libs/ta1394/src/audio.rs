// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use super::*;

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

    fn build_operands(
        &self,
        addr: &AvcAddr,
        operands: &mut Vec<u8>,
    ) -> Result<(), AvcCmdBuildError> {
        match addr {
            AvcAddr::Unit => Err(AvcCmdBuildError::InvalidAddress),
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
                    Err(AvcCmdBuildError::InvalidAddress)
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
    fn build_operands(
        &mut self,
        addr: &AvcAddr,
        operands: &mut Vec<u8>,
    ) -> Result<(), AvcCmdBuildError> {
        AudioFuncBlk::build_operands(self, addr, operands)
    }

    fn parse_operands(&mut self, _: &AvcAddr, operands: &[u8]) -> Result<(), Error> {
        AudioFuncBlk::parse_operands(self, operands)
    }
}

impl AvcControl for AudioFuncBlk {
    fn build_operands(
        &mut self,
        addr: &AvcAddr,
        operands: &mut Vec<u8>,
    ) -> Result<(), AvcCmdBuildError> {
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

    fn build_func_blk(&mut self) -> Result<(), AvcCmdBuildError> {
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
    fn build_operands(
        &mut self,
        addr: &AvcAddr,
        operands: &mut Vec<u8>,
    ) -> Result<(), AvcCmdBuildError> {
        self.build_func_blk()?;
        AvcStatus::build_operands(&mut self.func_blk, addr, operands)
    }

    fn parse_operands(&mut self, addr: &AvcAddr, operands: &[u8]) -> Result<(), Error> {
        AvcStatus::parse_operands(&mut self.func_blk, addr, operands)?;
        self.parse_func_blk()
    }
}

impl AvcControl for AudioSelector {
    fn build_operands(
        &mut self,
        addr: &AvcAddr,
        operands: &mut Vec<u8>,
    ) -> Result<(), AvcCmdBuildError> {
        self.build_func_blk()?;
        AvcControl::build_operands(&mut self.func_blk, addr, operands)
    }

    fn parse_operands(&mut self, addr: &AvcAddr, operands: &[u8]) -> Result<(), Error> {
        AvcControl::parse_operands(&mut self.func_blk, addr, operands)?;
        self.parse_func_blk()
    }
}

//
// AV/C Audio Subunit FUNCTION_BLOCK command for Feature function block
//
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GraphicEqualizerData {
    pub bands_present: [u8;4],
    pub ex_bands_present: [u8;4],
    pub gain: Vec<i8>,
}

impl From<&[u8]> for GraphicEqualizerData {
    fn from(raw: &[u8]) -> Self {
        let mut data = GraphicEqualizerData{
            bands_present: [0;4],
            ex_bands_present: [0;4],
            gain: Vec::new(),
        };
        data.bands_present.copy_from_slice(&raw[0..4]);
        data.ex_bands_present.copy_from_slice(&raw[4..8]);
        raw[8..].iter().for_each(|val| data.gain.push(*val as i8));
        data
    }
}

impl From<&GraphicEqualizerData> for Vec<u8> {
    fn from(data: &GraphicEqualizerData) -> Self {
        let mut raw = Vec::new();
        raw.extend_from_slice(&data.bands_present);
        raw.extend_from_slice(&data.ex_bands_present);
        data.gain.iter().for_each(|val| raw.push(*val as u8));
        raw
    }
}

fn i16_vector_to_raw(data: &[i16]) -> Vec<u8> {
    data.iter().fold(Vec::new(), |mut raw, d| { raw.extend_from_slice(&d.to_be_bytes()); raw})
}

fn u16_vector_to_raw(data: &[u16]) -> Vec<u8> {
    data.iter().fold(Vec::new(), |mut raw, d| { raw.extend_from_slice(&d.to_be_bytes()); raw})
}

fn bool_vector_to_raw(data: &[bool]) -> Vec<u8> {
    data.iter().map(|&d| if d { FeatureCtl::TRUE } else { FeatureCtl::FALSE }).collect()
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FeatureCtl {
    Mute(Vec<bool>),
    Volume(Vec<i16>),
    LrBalance(i16),
    FrBalance(i16),
    Bass(Vec<i8>),
    Mid(Vec<i8>),
    Treble(Vec<i8>),
    GraphicEqualizer(GraphicEqualizerData),
    AutomaticGain(Vec<bool>),
    Delay(Vec<u16>),
    BassBoost(Vec<bool>),
    Loudness(Vec<bool>),
    Reserved(Vec<u8>),
}

impl FeatureCtl {
    const MUTE: u8 = 0x01;
    const VOLUME: u8 = 0x02;
    const LR_BALANCE: u8 = 0x03;
    const FR_BALANCE: u8 = 0x04;
    const BASS: u8 = 0x05;
    const MID: u8 = 0x06;
    const TREBLE: u8 = 0x07;
    const GRAPHIC_EQUALIZER: u8 = 0x08;
    const AUTOMATIC_GAIN: u8 = 0x09;
    const DELAY: u8 = 0x0a;
    const BASS_BOOST: u8 = 0x0b;
    const LOUDNESS: u8 = 0x0c;

    const TRUE: u8 = 0x70;
    const FALSE: u8 = 0x60;

    pub const INFINITY: i16 = 0x7ffeu16 as i16;
    pub const NEG_INFINITY: i16 = 0x8000u16 as i16;
}

impl From<&FeatureCtl> for AudioFuncBlkCtl {
    fn from(ctl: &FeatureCtl) -> Self {
        match &ctl {
            FeatureCtl::Mute(data) => {
                AudioFuncBlkCtl{
                    selector: FeatureCtl::MUTE,
                    data: bool_vector_to_raw(data),
                }
            }
            FeatureCtl::Volume(data) => {
                AudioFuncBlkCtl{
                    selector: FeatureCtl::VOLUME,
                    data: i16_vector_to_raw(data),
                }
            }
            FeatureCtl::LrBalance(data) => {
                AudioFuncBlkCtl{
                    selector: FeatureCtl::LR_BALANCE,
                    data: data.to_be_bytes().to_vec(),
                }
            }
            FeatureCtl::FrBalance(data) => {
                AudioFuncBlkCtl{
                    selector: FeatureCtl::FR_BALANCE,
                    data: data.to_be_bytes().to_vec(),
                }
            }
            FeatureCtl::Bass(data) => {
                AudioFuncBlkCtl{
                    selector: FeatureCtl::BASS,
                    data: data.iter().map(|v| *v as u8).collect(),
                }
            }
            FeatureCtl::Mid(data) => {
                AudioFuncBlkCtl{
                    selector: FeatureCtl::MID,
                    data: data.iter().map(|v| *v as u8).collect(),
                }
            }
            FeatureCtl::Treble(data) => {
                AudioFuncBlkCtl{
                    selector: FeatureCtl::TREBLE,
                    data: data.iter().map(|v| *v as u8).collect(),
                }
            }
            FeatureCtl::GraphicEqualizer(data) => {
                AudioFuncBlkCtl{
                    selector: FeatureCtl::GRAPHIC_EQUALIZER,
                    data: data.into(),
                }
            }
            FeatureCtl::AutomaticGain(data) => {
                AudioFuncBlkCtl{
                    selector: FeatureCtl::AUTOMATIC_GAIN,
                    data: bool_vector_to_raw(data),
                }
            }
            FeatureCtl::Delay(data) => {
                AudioFuncBlkCtl{
                    selector: FeatureCtl::DELAY,
                    data: u16_vector_to_raw(data),
                }
            }
            FeatureCtl::BassBoost(data) => {
                AudioFuncBlkCtl{
                    selector: FeatureCtl::BASS_BOOST,
                    data: bool_vector_to_raw(data),
                }
            }
            FeatureCtl::Loudness(data) => {
                AudioFuncBlkCtl{
                    selector: FeatureCtl::LOUDNESS,
                    data: bool_vector_to_raw(data),
                }
            }
            FeatureCtl::Reserved(data) => {
                AudioFuncBlkCtl{
                    selector: data[0],
                    data: data[2..].to_vec(),
                }
            }
        }
    }
}

fn i16_vector_from_raw(raw: &[u8]) -> Vec<i16> {
    (0..(raw.len() / 2)).map(|i| {
        let mut doublet = [0;2];
        doublet.copy_from_slice(&raw[(i * 2)..(i * 2 + 2)]);
        i16::from_be_bytes(doublet)
    }).collect()
}

fn u16_vector_from_raw(raw: &[u8]) -> Vec<u16> {
    (0..(raw.len() / 2)).map(|i| {
        let mut doublet = [0;2];
        doublet.copy_from_slice(&raw[(i * 2)..(i * 2 + 2)]);
        u16::from_be_bytes(doublet)
    }).collect()
}

fn bool_vector_from_raw(raw: &[u8]) -> Vec<bool> {
    raw.iter().map(|&b| b == FeatureCtl::TRUE).collect()
}

fn i8_vector_from_raw(raw: &[u8]) -> Vec<i8> {
    raw.iter().map(|&b| b as i8).collect()
}

fn i16_from_raw(data: &[u8]) -> i16 {
    let mut doublet = [0;2];
    doublet.copy_from_slice(&data);
    i16::from_be_bytes(doublet)
}

impl From<&AudioFuncBlkCtl> for FeatureCtl {
    fn from(ctl: &AudioFuncBlkCtl) -> Self {
        match ctl.selector {
            Self::MUTE => FeatureCtl::Mute(bool_vector_from_raw(&ctl.data)),
            Self::VOLUME => FeatureCtl::Volume(i16_vector_from_raw(&ctl.data)),
            Self::LR_BALANCE => FeatureCtl::LrBalance(i16_from_raw(&ctl.data)),
            Self::FR_BALANCE => FeatureCtl::FrBalance(i16_from_raw(&ctl.data)),
            Self::BASS => FeatureCtl::Bass(i8_vector_from_raw(&ctl.data)),
            Self::MID => FeatureCtl::Mid(i8_vector_from_raw(&ctl.data)),
            Self::TREBLE => FeatureCtl::Treble(i8_vector_from_raw(&ctl.data)),
            Self::GRAPHIC_EQUALIZER => FeatureCtl::GraphicEqualizer(ctl.data.as_slice().into()),
            Self::AUTOMATIC_GAIN => FeatureCtl::AutomaticGain(bool_vector_from_raw(&ctl.data)),
            Self::DELAY => FeatureCtl::Delay(u16_vector_from_raw(&ctl.data)),
            Self::BASS_BOOST => FeatureCtl::BassBoost(bool_vector_from_raw(&ctl.data)),
            Self::LOUDNESS => FeatureCtl::Loudness(bool_vector_from_raw(&ctl.data)),
            _ => {
                let mut data = Vec::new();
                data.push(ctl.selector);
                data.push(1 + ctl.data.len() as u8);
                data.extend_from_slice(&ctl.data);
                FeatureCtl::Reserved(data)
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AudioCh {
    All,
    Each(u8),
}

impl AudioCh {
    const MASTER: u8 = 0x00;
}

impl From<u8> for AudioCh {
    fn from(val: u8) -> Self {
        match val {
            AudioCh::MASTER => AudioCh::All,
            _ => AudioCh::Each(val - 1),
        }
    }
}

impl From<AudioCh> for u8 {
    fn from(num: AudioCh) -> u8 {
        match num {
            AudioCh::All => AudioCh::MASTER,
            AudioCh::Each(val) => val + 1,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AudioFeature {
    pub audio_ch_num: AudioCh,
    pub ctl: FeatureCtl,

    func_blk: AudioFuncBlk,
}

impl AudioFeature {
    pub fn new(func_blk_id: u8, ctl_attr: CtlAttr, audio_ch_num: AudioCh, ctl: FeatureCtl) -> Self {
        AudioFeature{
            audio_ch_num,
            ctl,
            func_blk: AudioFuncBlk::new(AudioFuncBlkType::Feature, func_blk_id, ctl_attr),
        }
    }

    fn build_func_blk(&mut self) -> Result<(), AvcCmdBuildError> {
        self.func_blk.audio_selector_data.clear();
        self.func_blk.audio_selector_data.push(u8::from(self.audio_ch_num));
        self.func_blk.ctl = AudioFuncBlkCtl::from(&self.ctl);
        Ok(())
    }

    fn parse_func_blk(&mut self) -> Result<(), Error> {
        let audio_ch_num = AudioCh::from(self.func_blk.audio_selector_data[0]);
        if audio_ch_num != self.audio_ch_num {
            let label = format!("Unexpected channel number for AudioFeature: {:?} but {:?}",
                                self.audio_ch_num, audio_ch_num);
            Err(Error::new(Ta1394AvcError::UnexpectedRespOperands, &label))
        } else {
            self.ctl = FeatureCtl::from(&self.func_blk.ctl);
            Ok(())
        }
    }
}

impl AvcOp for AudioFeature {
    const OPCODE: u8 = AudioFuncBlk::OPCODE;
}

impl AvcStatus for AudioFeature {
    fn build_operands(
        &mut self,
        addr: &AvcAddr,
        operands: &mut Vec<u8>,
    ) -> Result<(), AvcCmdBuildError> {
        self.build_func_blk()?;
        AvcStatus::build_operands(&mut self.func_blk, addr, operands)
    }

    fn parse_operands(&mut self, addr: &AvcAddr, operands: &[u8]) -> Result<(), Error> {
        AvcStatus::parse_operands(&mut self.func_blk, addr, operands)?;
        self.parse_func_blk()
    }
}

impl AvcControl for AudioFeature {
    fn build_operands(
        &mut self,
        addr: &AvcAddr,
        operands: &mut Vec<u8>,
    ) -> Result<(), AvcCmdBuildError> {
        self.build_func_blk()?;
        AvcControl::build_operands(&mut self.func_blk, addr, operands)
    }

    fn parse_operands(&mut self, addr: &AvcAddr, operands: &[u8]) -> Result<(), Error> {
        AvcControl::parse_operands(&mut self.func_blk, addr, operands)?;
        self.parse_func_blk()
    }
}

//
// AV/C Audio Subunit FUNCTION_BLOCK command for processing function block
//
//
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProcessingCtl {
    Enable(bool),
    Mode(Vec<u8>),
    Mixer(Vec<i16>),
    Reserved(Vec<u8>),
}

impl ProcessingCtl {
    const ENABLE: u8 = 0x01;
    const MODE: u8 = 0x02;
    const MIXER: u8 = 0x03;

    const TRUE: u8 = 0x70;
    const FALSE: u8 = 0x60;

    pub const INFINITY: i16 = 0x7ffeu16 as i16;
    pub const NEG_INFINITY: i16 = 0x8000u16 as i16;
}

impl From<&ProcessingCtl> for AudioFuncBlkCtl {
    fn from(ctl: &ProcessingCtl) -> Self {
        match ctl {
            ProcessingCtl::Enable(data) => {
                AudioFuncBlkCtl{
                    selector: ProcessingCtl::ENABLE,
                    data: vec![if *data { ProcessingCtl::TRUE } else { ProcessingCtl::FALSE }],
                }
            }
            ProcessingCtl::Mode(data) => {
                AudioFuncBlkCtl{
                    selector: ProcessingCtl::MODE,
                    data: data.to_vec(),
                }
            }
            ProcessingCtl::Mixer(data) => {
                AudioFuncBlkCtl{
                    selector: ProcessingCtl::MIXER,
                    data: i16_vector_to_raw(data),
                }
            }
            ProcessingCtl::Reserved(data) => {
                AudioFuncBlkCtl{
                    selector: data[0],
                    data: data[2..].to_vec(),
                }
            }
        }
    }
}

impl From<&AudioFuncBlkCtl> for ProcessingCtl {
    fn from(ctl_blk: &AudioFuncBlkCtl) -> Self {
        match ctl_blk.selector {
            Self::ENABLE => ProcessingCtl::Enable(ctl_blk.data[0] == ProcessingCtl::TRUE),
            Self::MODE => ProcessingCtl::Mode(ctl_blk.data.to_vec()),
            Self::MIXER => ProcessingCtl::Mixer(i16_vector_from_raw(&ctl_blk.data)),
            _ => {
                let mut data = Vec::new();
                data.push(ctl_blk.selector);
                data.push(1 + ctl_blk.data.len() as u8);
                data.extend_from_slice(&ctl_blk.data);
                ProcessingCtl::Reserved(data)
            }
        }
    }
}

pub struct AudioProcessing {
    pub input_plug_id: u8,
    pub input_ch: AudioCh,
    pub output_ch: AudioCh,
    pub ctl: ProcessingCtl,

    func_blk: AudioFuncBlk,
}

impl AudioProcessing {
    pub fn new(func_blk_id: u8, ctl_attr: CtlAttr, input_plug_id: u8, input_ch: AudioCh,
               output_ch: AudioCh, ctl: ProcessingCtl) -> Self {
        AudioProcessing{
            input_plug_id,
            input_ch,
            output_ch,
            ctl,
            func_blk: AudioFuncBlk::new(AudioFuncBlkType::Processing, func_blk_id, ctl_attr),
        }
    }

    fn build_func_blk(&mut self) -> Result<(), AvcCmdBuildError> {
        self.func_blk.audio_selector_data.clear();
        self.func_blk.audio_selector_data.push(self.input_plug_id);
        self.func_blk.audio_selector_data.push(u8::from(self.input_ch));
        self.func_blk.audio_selector_data.push(u8::from(self.output_ch));
        self.func_blk.ctl = AudioFuncBlkCtl::from(&self.ctl);
        Ok(())
    }

    fn parse_func_blk(&mut self) -> Result<(), Error> {
        if self.func_blk.audio_selector_data[0] != self.input_plug_id {
            let label = format!("Unexpected input plug ID for AudioProcessing: {} but {}",
                                self.input_plug_id, self.func_blk.func_blk_id);
            return Err(Error::new(Ta1394AvcError::UnexpectedRespOperands, &label));
        }

        let input_ch = AudioCh::from(self.func_blk.audio_selector_data[1]);
        if input_ch != self.input_ch {
            let label = format!("Unexpected input audio channel number for AudioProcessing: {:?} but {:?}",
                                self.input_ch, input_ch);
            return Err(Error::new(Ta1394AvcError::UnexpectedRespOperands, &label));
        }

        let output_ch = AudioCh::from(self.func_blk.audio_selector_data[2]);
        if output_ch != self.output_ch {
            let label = format!("Unexpected output audio channel number for AudioProcessing: {:?} but {:?}",
                                self.output_ch, output_ch);
            return Err(Error::new(Ta1394AvcError::UnexpectedRespOperands, &label));
        }

        self.ctl = ProcessingCtl::from(&self.func_blk.ctl);
        Ok(())
    }
}

impl AvcOp for AudioProcessing {
    const OPCODE: u8 = AudioFuncBlk::OPCODE;
}

impl AvcStatus for AudioProcessing {
    fn build_operands(
        &mut self,
        addr: &AvcAddr,
        operands: &mut Vec<u8>,
    ) -> Result<(), AvcCmdBuildError> {
        self.build_func_blk()?;
        AvcStatus::build_operands(&mut self.func_blk, addr, operands)
    }

    fn parse_operands(&mut self, addr: &AvcAddr, operands: &[u8]) -> Result<(), Error> {
        AvcStatus::parse_operands(&mut self.func_blk, addr, operands)?;
        self.parse_func_blk()
    }
}

impl AvcControl for AudioProcessing {
    fn build_operands(
        &mut self,
        addr: &AvcAddr,
        operands: &mut Vec<u8>,
    ) -> Result<(), AvcCmdBuildError> {
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
    use crate::audio::*;

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

    #[test]
    fn featurectl_from() {
        let ctl = FeatureCtl::Mute(vec![false, true, false]);
        assert_eq!(ctl, FeatureCtl::from(&AudioFuncBlkCtl::from(&ctl)));

        let ctl = FeatureCtl::Volume(vec![0x1234, 0x3456, 0x789a]);
        assert_eq!(ctl, FeatureCtl::from(&AudioFuncBlkCtl::from(&ctl)));

        let ctl = FeatureCtl::LrBalance(-123);
        assert_eq!(ctl, FeatureCtl::from(&AudioFuncBlkCtl::from(&ctl)));

        let ctl = FeatureCtl::FrBalance(321);
        assert_eq!(ctl, FeatureCtl::from(&AudioFuncBlkCtl::from(&ctl)));

        let ctl = FeatureCtl::Bass(vec![10, -10, 20, -20]);
        assert_eq!(ctl, FeatureCtl::from(&AudioFuncBlkCtl::from(&ctl)));

        let ctl = FeatureCtl::Mid(vec![30, -30, -40, 40]);
        assert_eq!(ctl, FeatureCtl::from(&AudioFuncBlkCtl::from(&ctl)));

        let data = GraphicEqualizerData{
            bands_present: [0x00, 0x01, 0x02, 0x03],
            ex_bands_present: [0x04, 0x05, 0x06, 0x07],
            gain: vec![-1, -2, -3, 10, 14, -40, -100, 33, 87, 99, -123, 100, -76, -97, 18, 21],
        };
        let ctl = FeatureCtl::GraphicEqualizer(data);
        assert_eq!(ctl, FeatureCtl::from(&AudioFuncBlkCtl::from(&ctl)));

        let ctl = FeatureCtl::Treble(vec![50, 60, -70, -80]);
        assert_eq!(ctl, FeatureCtl::from(&AudioFuncBlkCtl::from(&ctl)));

        let ctl = FeatureCtl::AutomaticGain(vec![false, true, false]);
        assert_eq!(ctl, FeatureCtl::from(&AudioFuncBlkCtl::from(&ctl)));

        let ctl = FeatureCtl::Delay(vec![0x1234, 0x3456, 0x789a]);
        assert_eq!(ctl, FeatureCtl::from(&AudioFuncBlkCtl::from(&ctl)));

        let ctl = FeatureCtl::BassBoost(vec![true, false, true]);
        assert_eq!(ctl, FeatureCtl::from(&AudioFuncBlkCtl::from(&ctl)));

        let ctl = FeatureCtl::Loudness(vec![false, true, false]);
        assert_eq!(ctl, FeatureCtl::from(&AudioFuncBlkCtl::from(&ctl)));

        let ctl = FeatureCtl::Reserved(vec![0xff, 0x04, 0xad, 0xbe, 0xef]);
        assert_eq!(ctl, FeatureCtl::from(&AudioFuncBlkCtl::from(&ctl)));
    }

    #[test]
    fn avcaudiofeature_operands() {
        let ctl = FeatureCtl::Volume(vec![-1234, 5678, 3210]);
        let mut op = AudioFeature::new(0x03, CtlAttr::Minimum, AudioCh::Each(0x1b), ctl.clone());
        let mut operands = Vec::new();
        AvcStatus::build_operands(&mut op, &AUDIO_SUBUNIT_0_ADDR, &mut operands).unwrap();
        assert_eq!(&operands, &[0x81, 0x03, 0x02, 0x02, 0x1c, 0x02, 0x06, 0xfb, 0x2e, 0x16, 0x2e, 0x0c, 0x8a]);

        AvcStatus::parse_operands(&mut op, &AvcAddr::Unit, &operands).unwrap();
        assert_eq!(AudioCh::Each(0x1b), op.audio_ch_num);
        assert_eq!(ctl, op.ctl);

        let ctl = FeatureCtl::Treble(vec![40, -33, 123, -96]);
        let mut op = AudioFeature::new(0x33, CtlAttr::Resolution, AudioCh::Each(0xd8), ctl.clone());
        let mut operands = Vec::new();
        AvcControl::build_operands(&mut op, &AUDIO_SUBUNIT_0_ADDR, &mut operands).unwrap();
        assert_eq!(&operands, &[0x81, 0x33, 0x01, 0x2, 0xd9, 0x07, 0x04, 0x28, 0xdf, 0x7b, 0xa0]);

        AvcControl::parse_operands(&mut op, &AvcAddr::Unit, &operands).unwrap();
        assert_eq!(AudioCh::Each(0xd8), op.audio_ch_num);
        assert_eq!(ctl, op.ctl);
    }

    #[test]
    fn processingctl_from() {
        let ctl = ProcessingCtl::Enable(true);
        assert_eq!(ctl, ProcessingCtl::from(&AudioFuncBlkCtl::from(&ctl)));

        let ctl = ProcessingCtl::Mode(vec![0xde, 0xad, 0xbe, 0xef]);
        assert_eq!(ctl, ProcessingCtl::from(&AudioFuncBlkCtl::from(&ctl)));

        let ctl = ProcessingCtl::Mixer(vec![-73, -157]);
        assert_eq!(ctl, ProcessingCtl::from(&AudioFuncBlkCtl::from(&ctl)));
    }

    #[test]
    fn avcaudioprocessing_operands() {
        let ctl = ProcessingCtl::Enable(true);
        let mut op = AudioProcessing::new(0xf5, CtlAttr::Default, 0x71, AudioCh::Each(0xa8),
                                          AudioCh::Each(0x3e), ctl.clone());
        let mut operands = Vec::new();
        AvcStatus::build_operands(&mut op, &AUDIO_SUBUNIT_0_ADDR, &mut operands).unwrap();
        assert_eq!(&operands, &[0x82, 0xf5, 0x04, 0x04, 0x71, 0xa9, 0x3f, 0x01, 0x01, 0x70]);

        AvcStatus::parse_operands(&mut op, &AvcAddr::Unit, &operands).unwrap();
        assert_eq!(0x71, op.input_plug_id);
        assert_eq!(AudioCh::Each(0xa8), op.input_ch);
        assert_eq!(AudioCh::Each(0x3e), op.output_ch);
        assert_eq!(ctl, op.ctl);

        let ctl = ProcessingCtl::Mixer(vec![10, -10]);
        let mut op = AudioProcessing::new(0x11, CtlAttr::Minimum, 0x22, AudioCh::Each(0x32),
                                          AudioCh::Each(0x43), ctl.clone());
        let mut operands = Vec::new();
        AvcControl::build_operands(&mut op, &AUDIO_SUBUNIT_0_ADDR, &mut operands).unwrap();
        assert_eq!(&operands, &[0x82, 0x11, 0x02, 0x04, 0x22, 0x33, 0x44, 0x03, 0x04, 0x00, 0x0a, 0xff, 0xf6]);

        AvcControl::parse_operands(&mut op, &AvcAddr::Unit, &operands).unwrap();
        assert_eq!(0x22, op.input_plug_id);
        assert_eq!(AudioCh::Each(0x32), op.input_ch);
        assert_eq!(AudioCh::Each(0x43), op.output_ch);
        assert_eq!(ctl, op.ctl);
    }
}
