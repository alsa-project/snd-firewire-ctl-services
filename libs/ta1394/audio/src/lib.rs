// SPDX-License-Identifier: MIT
// Copyright (c) 2022 Takashi Sakamoto

#![doc = include_str!("../README.md")]

/// Encoder and decoder of FDF field in Audio and Music Data Transmission Protocol.
pub mod amdtp;

use ta1394_avc_general::*;

/// The AV/C address of first music subunit for convenience.
pub const AUDIO_SUBUNIT_0_ADDR: AvcAddr = AvcAddr::Subunit(AUDIO_SUBUNIT_0);

/// The type of function block in audio subunit.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AudioFuncBlkType {
    /// Selector function block.
    Selector,
    /// Feature function block.
    Feature,
    /// Processing function block.
    Processing,
    Reserved(u8),
}

impl Default for AudioFuncBlkType {
    fn default() -> Self {
        Self::Reserved(0xff)
    }
}

impl AudioFuncBlkType {
    fn from_val(val: u8) -> Self {
        match val {
            0x80 => Self::Selector,
            0x81 => Self::Feature,
            0x82 => Self::Processing,
            _ => Self::Reserved(val),
        }
    }

    fn to_val(&self) -> u8 {
        match self {
            Self::Selector => 0x80,
            Self::Feature => 0x81,
            Self::Processing => 0x82,
            Self::Reserved(val) => *val,
        }
    }
}

/// For attributes of control (clause "4.8 Control Attributes").
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CtlAttr {
    /// Minimum scale.
    Resolution,
    /// Minimum setting.
    Minimum,
    /// Maximum setting.
    Maximum,
    /// Default setting.
    Default,
    /// Minimum moving time.
    Duration,
    /// Current setting.
    Current,
    /// Request to change the value during a period equalds to a number of Duration.
    Move,
    /// Relative setting in unit steps.
    Delta,
    Reserved(u8),
}

impl Default for CtlAttr {
    fn default() -> Self {
        Self::Reserved(0xff)
    }
}

impl CtlAttr {
    fn from_val(val: u8) -> Self {
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

    fn to_val(&self) -> u8 {
        match self {
            Self::Resolution => 0x01,
            Self::Minimum => 0x02,
            Self::Maximum => 0x03,
            Self::Default => 0x04,
            Self::Duration => 0x08,
            Self::Current => 0x10,
            Self::Move => 0x18,
            Self::Delta => 0x19,
            Self::Reserved(val) => *val,
        }
    }
}

impl std::fmt::Display for CtlAttr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            Self::Resolution => "resolution".to_owned(),
            Self::Minimum => "minimum".to_owned(),
            Self::Maximum => "maximum".to_owned(),
            Self::Default => "default".to_owned(),
            Self::Duration => "duration".to_owned(),
            Self::Current => "current".to_owned(),
            Self::Move => "move".to_owned(),
            Self::Delta => "delta".to_owned(),
            Self::Reserved(val) => format!("reserved: {}", val),
        };
        write!(f, "{}", &label)
    }
}

/// For control information in frame of function block command.
#[derive(Clone, Debug, Eq, PartialEq)]
struct AudioFuncBlkCtl {
    /// The value of control_selector field for the type of control.
    selector: u8,
    /// The data in control_data field according to the type.
    data: Vec<u8>,
}

impl Default for AudioFuncBlkCtl {
    fn default() -> Self {
        Self {
            selector: 0xff,
            data: Default::default(),
        }
    }
}

impl AudioFuncBlkCtl {
    const LENGTH_MIN: usize = 1;

    fn from_raw(raw: &[u8]) -> Self {
        assert!(raw.len() >= Self::LENGTH_MIN);
        let mut ctl = Self {
            selector: raw[0],
            data: Default::default(),
        };
        if raw.len() > 1 {
            let length = raw[1] as usize;
            if raw.len() >= 2 + length {
                ctl.data.extend_from_slice(&raw[2..(2 + length)]);
            }
        }
        ctl
    }

    fn to_raw(&self) -> Vec<u8> {
        let mut raw = Vec::with_capacity(Self::LENGTH_MIN);
        raw.push(self.selector);
        if self.data.len() > 0 {
            raw.push(self.data.len() as u8);
            raw.extend_from_slice(&self.data);
        }
        raw
    }
}

/// For operands of frame in function block command (clause "10. Audio Subunit FUNCTION_BLOCK
/// command")
#[derive(Clone, Debug, Eq, PartialEq)]
struct AudioFuncBlk {
    func_blk_type: AudioFuncBlkType,
    func_blk_id: u8,
    ctl_attr: CtlAttr,
    audio_selector_data: Vec<u8>,
    ctl: AudioFuncBlkCtl,
}

impl Default for AudioFuncBlk {
    fn default() -> Self {
        Self {
            func_blk_type: Default::default(),
            func_blk_id: 0xff,
            ctl_attr: Default::default(),
            audio_selector_data: Default::default(),
            ctl: Default::default(),
        }
    }
}

impl AudioFuncBlk {
    fn build_operands(
        &self,
        addr: &AvcAddr,
        operands: &mut Vec<u8>,
    ) -> Result<(), AvcCmdBuildError> {
        if let AvcAddr::Subunit(AvcAddrSubunit {
            subunit_type: AvcSubunitType::Audio,
            subunit_id: _,
        }) = addr
        {
            operands.push(self.func_blk_type.to_val());
            operands.push(self.func_blk_id);
            operands.push(self.ctl_attr.to_val());
            operands.push(1 + self.audio_selector_data.len() as u8);
            operands.extend_from_slice(&self.audio_selector_data);
            operands.append(&mut self.ctl.to_raw());
            Ok(())
        } else {
            Err(AvcCmdBuildError::InvalidAddress)
        }
    }

    fn parse_operands(&mut self, operands: &[u8]) -> Result<(), AvcRespParseError> {
        if operands.len() < 4 {
            Err(AvcRespParseError::TooShortResp(4))?;
        }
        let func_blk_type = AudioFuncBlkType::from_val(operands[0]);
        if func_blk_type != self.func_blk_type {
            Err(AvcRespParseError::UnexpectedOperands(0))?;
        }

        let func_blk_id = operands[1];
        if func_blk_id != self.func_blk_id {
            Err(AvcRespParseError::UnexpectedOperands(1))?;
        }

        let ctl_attr = CtlAttr::from_val(operands[2]);
        if ctl_attr != self.ctl_attr {
            Err(AvcRespParseError::UnexpectedOperands(2))?;
        }

        let mut audio_selector_length = operands[3] as usize;
        if operands.len() < 3 + audio_selector_length {
            Err(AvcRespParseError::TooShortResp(3 + audio_selector_length))?;
        } else if audio_selector_length < 1 {
            Err(AvcRespParseError::UnexpectedOperands(3))?;
        }
        audio_selector_length -= 1;
        self.audio_selector_data = operands[4..(4 + audio_selector_length)].to_vec();

        self.ctl = AudioFuncBlkCtl::from_raw(&operands[(4 + audio_selector_length)..]);

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

    fn parse_operands(&mut self, _: &AvcAddr, operands: &[u8]) -> Result<(), AvcRespParseError> {
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

    fn parse_operands(&mut self, _: &AvcAddr, operands: &[u8]) -> Result<(), AvcRespParseError> {
        AudioFuncBlk::parse_operands(self, operands)
    }
}

///
/// AV/C Audio Subunit FUNCTION_BLOCK command for Selector function block
///
/// Described in clause "10.2 Selector function block".
pub struct AudioSelector {
    pub input_plug_id: u8,
    func_blk: AudioFuncBlk,
}

impl AudioSelector {
    const SELECTOR_CONTROL: u8 = 0x01;

    pub fn new(func_blk_id: u8, ctl_attr: CtlAttr, input_plug_id: u8) -> Self {
        Self {
            input_plug_id,
            func_blk: AudioFuncBlk {
                func_blk_type: AudioFuncBlkType::Selector,
                func_blk_id,
                ctl_attr,
                ..Default::default()
            },
        }
    }

    fn build_func_blk(&mut self) -> Result<(), AvcCmdBuildError> {
        self.func_blk.audio_selector_data.clear();
        self.func_blk.audio_selector_data.push(self.input_plug_id);
        self.func_blk.ctl.selector = Self::SELECTOR_CONTROL;
        self.func_blk.ctl.data.clear();
        Ok(())
    }

    fn parse_func_blk(&mut self) -> Result<(), AvcRespParseError> {
        if self.func_blk.ctl.selector != Self::SELECTOR_CONTROL {
            Err(AvcRespParseError::UnexpectedOperands(5))
        } else if self.func_blk.ctl.data.len() > 0 {
            Err(AvcRespParseError::UnexpectedOperands(7))
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

    fn parse_operands(&mut self, addr: &AvcAddr, operands: &[u8]) -> Result<(), AvcRespParseError> {
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

    fn parse_operands(&mut self, addr: &AvcAddr, operands: &[u8]) -> Result<(), AvcRespParseError> {
        AvcControl::parse_operands(&mut self.func_blk, addr, operands)?;
        self.parse_func_blk()
    }
}

/// Figure 10.30 – First Form of the Graphic Equalizer Control Parameters.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GraphicEqualizerData {
    pub bands_present: [u8; 4],
    pub ex_bands_present: [u8; 4],
    pub gain: Vec<i8>,
}

impl GraphicEqualizerData {
    fn from_raw<T: AsRef<[u8]>>(raw: T) -> Self {
        let mut data = GraphicEqualizerData {
            bands_present: [0; 4],
            ex_bands_present: [0; 4],
            gain: Vec::new(),
        };
        let r = &raw.as_ref();
        data.bands_present.copy_from_slice(&r[0..4]);
        data.ex_bands_present.copy_from_slice(&r[4..8]);
        r[8..].iter().for_each(|val| data.gain.push(*val as i8));
        data
    }

    fn to_raw(&self) -> Vec<u8> {
        let mut raw = Vec::new();
        raw.extend_from_slice(&self.bands_present);
        raw.extend_from_slice(&self.ex_bands_present);
        self.gain.iter().for_each(|val| raw.push(*val as u8));
        raw
    }
}

fn i16_vector_to_raw(data: &[i16]) -> Vec<u8> {
    data.iter().fold(Vec::new(), |mut raw, d| {
        raw.extend_from_slice(&d.to_be_bytes());
        raw
    })
}

fn u16_vector_to_raw(data: &[u16]) -> Vec<u8> {
    data.iter().fold(Vec::new(), |mut raw, d| {
        raw.extend_from_slice(&d.to_be_bytes());
        raw
    })
}

fn bool_vector_to_raw(data: &[bool]) -> Vec<u8> {
    data.iter()
        .map(|&d| {
            if d {
                FeatureCtl::TRUE
            } else {
                FeatureCtl::FALSE
            }
        })
        .collect()
}

/// The type of Feature Control.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FeatureCtl {
    /// Clause 10.3.1 Mute Control.
    Mute(Vec<bool>),
    /// Clause 10.3.2 Volume Control.
    Volume(Vec<i16>),
    /// Clause 10.3.3 LR Balance Control.
    LrBalance(i16),
    /// Clause 10.3.4 FR Balance Control.
    FrBalance(i16),
    /// Clause 10.3.5 Bass Control.
    Bass(Vec<i8>),
    /// Clause 10.3.6 Mid Control.
    Mid(Vec<i8>),
    /// Clause 10.3.7 Treble Control.
    Treble(Vec<i8>),
    /// Clause 10.3.8 Graphic Equalizer Control.
    GraphicEqualizer(GraphicEqualizerData),
    /// Clause 10.3.9 Automatic Gain Control.
    AutomaticGain(Vec<bool>),
    /// Clause 10.3.10 Delay Control.
    Delay(Vec<u16>),
    /// Clause 10.3.11 Bass Boost Control.
    BassBoost(Vec<bool>),
    /// Clause 10.3.12 Loudness Control.
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

impl FeatureCtl {
    fn to_ctl(&self) -> AudioFuncBlkCtl {
        match self {
            Self::Mute(data) => AudioFuncBlkCtl {
                selector: Self::MUTE,
                data: bool_vector_to_raw(data),
            },
            Self::Volume(data) => AudioFuncBlkCtl {
                selector: Self::VOLUME,
                data: i16_vector_to_raw(data),
            },
            Self::LrBalance(data) => AudioFuncBlkCtl {
                selector: Self::LR_BALANCE,
                data: data.to_be_bytes().to_vec(),
            },
            Self::FrBalance(data) => AudioFuncBlkCtl {
                selector: Self::FR_BALANCE,
                data: data.to_be_bytes().to_vec(),
            },
            Self::Bass(data) => AudioFuncBlkCtl {
                selector: Self::BASS,
                data: data.iter().map(|v| *v as u8).collect(),
            },
            Self::Mid(data) => AudioFuncBlkCtl {
                selector: Self::MID,
                data: data.iter().map(|v| *v as u8).collect(),
            },
            Self::Treble(data) => AudioFuncBlkCtl {
                selector: Self::TREBLE,
                data: data.iter().map(|v| *v as u8).collect(),
            },
            Self::GraphicEqualizer(data) => AudioFuncBlkCtl {
                selector: Self::GRAPHIC_EQUALIZER,
                data: data.to_raw(),
            },
            Self::AutomaticGain(data) => AudioFuncBlkCtl {
                selector: Self::AUTOMATIC_GAIN,
                data: bool_vector_to_raw(data),
            },
            Self::Delay(data) => AudioFuncBlkCtl {
                selector: Self::DELAY,
                data: u16_vector_to_raw(data),
            },
            Self::BassBoost(data) => AudioFuncBlkCtl {
                selector: Self::BASS_BOOST,
                data: bool_vector_to_raw(data),
            },
            Self::Loudness(data) => AudioFuncBlkCtl {
                selector: Self::LOUDNESS,
                data: bool_vector_to_raw(data),
            },
            Self::Reserved(data) => AudioFuncBlkCtl {
                selector: data[0],
                data: data[2..].to_vec(),
            },
        }
    }
}

fn i16_vector_from_raw(raw: &[u8]) -> Vec<i16> {
    (0..(raw.len() / 2))
        .map(|i| {
            let mut doublet = [0; 2];
            doublet.copy_from_slice(&raw[(i * 2)..(i * 2 + 2)]);
            i16::from_be_bytes(doublet)
        })
        .collect()
}

fn u16_vector_from_raw(raw: &[u8]) -> Vec<u16> {
    (0..(raw.len() / 2))
        .map(|i| {
            let mut doublet = [0; 2];
            doublet.copy_from_slice(&raw[(i * 2)..(i * 2 + 2)]);
            u16::from_be_bytes(doublet)
        })
        .collect()
}

fn bool_vector_from_raw(raw: &[u8]) -> Vec<bool> {
    raw.iter().map(|&b| b == FeatureCtl::TRUE).collect()
}

fn i8_vector_from_raw(raw: &[u8]) -> Vec<i8> {
    raw.iter().map(|&b| b as i8).collect()
}

fn i16_from_raw(data: &[u8]) -> i16 {
    let mut doublet = [0; 2];
    doublet.copy_from_slice(&data);
    i16::from_be_bytes(doublet)
}

impl FeatureCtl {
    fn from_ctl(ctl: &AudioFuncBlkCtl) -> Self {
        match ctl.selector {
            Self::MUTE => Self::Mute(bool_vector_from_raw(&ctl.data)),
            Self::VOLUME => Self::Volume(i16_vector_from_raw(&ctl.data)),
            Self::LR_BALANCE => Self::LrBalance(i16_from_raw(&ctl.data)),
            Self::FR_BALANCE => Self::FrBalance(i16_from_raw(&ctl.data)),
            Self::BASS => Self::Bass(i8_vector_from_raw(&ctl.data)),
            Self::MID => Self::Mid(i8_vector_from_raw(&ctl.data)),
            Self::TREBLE => Self::Treble(i8_vector_from_raw(&ctl.data)),
            Self::GRAPHIC_EQUALIZER => {
                Self::GraphicEqualizer(GraphicEqualizerData::from_raw(&ctl.data))
            }
            Self::AUTOMATIC_GAIN => Self::AutomaticGain(bool_vector_from_raw(&ctl.data)),
            Self::DELAY => Self::Delay(u16_vector_from_raw(&ctl.data)),
            Self::BASS_BOOST => Self::BassBoost(bool_vector_from_raw(&ctl.data)),
            Self::LOUDNESS => Self::Loudness(bool_vector_from_raw(&ctl.data)),
            _ => {
                let mut data = Vec::new();
                data.push(ctl.selector);
                data.push(1 + ctl.data.len() as u8);
                data.extend_from_slice(&ctl.data);
                Self::Reserved(data)
            }
        }
    }
}

/// For the value of audio_channel_number field described in clause "10.3 Feature function
/// block".
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AudioCh {
    /// Master channel.
    Master,
    /// Each of channel.
    Each(u8),
    /// Void channel.
    Void,
    /// All channels.
    All,
}

impl Default for AudioCh {
    fn default() -> Self {
        Self::All
    }
}

impl AudioCh {
    const MASTER: u8 = 0x00;
    const VOID: u8 = 0xfe;
    const ALL: u8 = 0xff;
}

impl AudioCh {
    fn from_val(val: u8) -> Self {
        match val {
            Self::MASTER => Self::Master,
            Self::ALL => Self::All,
            Self::VOID => Self::Void,
            // MEMO: It should be greater than 0 and less than 0xfe, however it' loosely handled
            // here.
            _ => Self::Each(val - 1),
        }
    }

    fn to_val(&self) -> u8 {
        match self {
            Self::Master => Self::MASTER,
            Self::All => Self::ALL,
            Self::Void => Self::VOID,
            Self::Each(val) => val + 1,
        }
    }
}

///
/// AV/C Audio Subunit FUNCTION_BLOCK command for Feature function block
///
/// Described in 10.3 Feature function block.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AudioFeature {
    /// The channels to address.
    pub audio_ch_num: AudioCh,
    /// The control to manipulate.
    pub ctl: FeatureCtl,

    func_blk: AudioFuncBlk,
}

impl AudioFeature {
    pub fn new(func_blk_id: u8, ctl_attr: CtlAttr, audio_ch_num: AudioCh, ctl: FeatureCtl) -> Self {
        Self {
            audio_ch_num,
            ctl,
            func_blk: AudioFuncBlk {
                func_blk_type: AudioFuncBlkType::Feature,
                func_blk_id,
                ctl_attr,
                ..Default::default()
            },
        }
    }

    fn build_func_blk(&mut self) -> Result<(), AvcCmdBuildError> {
        self.func_blk.audio_selector_data.clear();
        self.func_blk
            .audio_selector_data
            .push(self.audio_ch_num.to_val());
        self.func_blk.ctl = self.ctl.to_ctl();
        Ok(())
    }

    fn parse_func_blk(&mut self) -> Result<(), AvcRespParseError> {
        let audio_ch_num = AudioCh::from_val(self.func_blk.audio_selector_data[0]);
        if audio_ch_num != self.audio_ch_num {
            Err(AvcRespParseError::UnexpectedOperands(7))
        } else {
            self.ctl = FeatureCtl::from_ctl(&self.func_blk.ctl);
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

    fn parse_operands(&mut self, addr: &AvcAddr, operands: &[u8]) -> Result<(), AvcRespParseError> {
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

    fn parse_operands(&mut self, addr: &AvcAddr, operands: &[u8]) -> Result<(), AvcRespParseError> {
        AvcControl::parse_operands(&mut self.func_blk, addr, operands)?;
        self.parse_func_blk()
    }
}

/// The type of processing control.
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

impl ProcessingCtl {
    fn to_ctl(&self) -> AudioFuncBlkCtl {
        match self {
            Self::Enable(data) => AudioFuncBlkCtl {
                selector: Self::ENABLE,
                data: vec![if *data { Self::TRUE } else { Self::FALSE }],
            },
            Self::Mode(data) => AudioFuncBlkCtl {
                selector: Self::MODE,
                data: data.to_vec(),
            },
            Self::Mixer(data) => AudioFuncBlkCtl {
                selector: Self::MIXER,
                data: i16_vector_to_raw(data),
            },
            Self::Reserved(data) => AudioFuncBlkCtl {
                selector: data[0],
                data: data[2..].to_vec(),
            },
        }
    }

    fn from_ctl(ctl_blk: &AudioFuncBlkCtl) -> Self {
        match ctl_blk.selector {
            Self::ENABLE => Self::Enable(ctl_blk.data[0] == Self::TRUE),
            Self::MODE => Self::Mode(ctl_blk.data.to_vec()),
            Self::MIXER => Self::Mixer(i16_vector_from_raw(&ctl_blk.data)),
            _ => {
                let mut data = Vec::new();
                data.push(ctl_blk.selector);
                data.push(1 + ctl_blk.data.len() as u8);
                data.extend_from_slice(&ctl_blk.data);
                Self::Reserved(data)
            }
        }
    }
}

///
/// AV/C Audio Subunit FUNCTION_BLOCK command for processing function block
///
/// Described in 10.4 Processing function block.
pub struct AudioProcessing {
    /// Function block input plug number (FBPN).
    pub input_plug_id: u8,
    /// Input audio channel (ICN).
    pub input_ch: AudioCh,
    /// Output audio channel (OCN).
    pub output_ch: AudioCh,
    /// Processing function block type dependent parameters.
    pub ctl: ProcessingCtl,

    func_blk: AudioFuncBlk,
}

impl AudioProcessing {
    pub fn new(
        func_blk_id: u8,
        ctl_attr: CtlAttr,
        input_plug_id: u8,
        input_ch: AudioCh,
        output_ch: AudioCh,
        ctl: ProcessingCtl,
    ) -> Self {
        Self {
            input_plug_id,
            input_ch,
            output_ch,
            ctl,
            func_blk: AudioFuncBlk {
                func_blk_type: AudioFuncBlkType::Processing,
                func_blk_id,
                ctl_attr,
                ..Default::default()
            },
        }
    }

    fn build_func_blk(&mut self) -> Result<(), AvcCmdBuildError> {
        self.func_blk.audio_selector_data.clear();
        self.func_blk.audio_selector_data.push(self.input_plug_id);
        self.func_blk
            .audio_selector_data
            .push(self.input_ch.to_val());
        self.func_blk
            .audio_selector_data
            .push(self.output_ch.to_val());
        self.func_blk.ctl = self.ctl.to_ctl();
        Ok(())
    }

    fn parse_func_blk(&mut self) -> Result<(), AvcRespParseError> {
        if self.func_blk.audio_selector_data[0] != self.input_plug_id {
            Err(AvcRespParseError::UnexpectedOperands(7))?;
        }

        let input_ch = AudioCh::from_val(self.func_blk.audio_selector_data[1]);
        if input_ch != self.input_ch {
            Err(AvcRespParseError::UnexpectedOperands(8))?;
        }

        let output_ch = AudioCh::from_val(self.func_blk.audio_selector_data[2]);
        if output_ch != self.output_ch {
            Err(AvcRespParseError::UnexpectedOperands(9))?;
        }

        self.ctl = ProcessingCtl::from_ctl(&self.func_blk.ctl);
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

    fn parse_operands(&mut self, addr: &AvcAddr, operands: &[u8]) -> Result<(), AvcRespParseError> {
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

    fn parse_operands(&mut self, addr: &AvcAddr, operands: &[u8]) -> Result<(), AvcRespParseError> {
        AvcControl::parse_operands(&mut self.func_blk, addr, operands)?;
        self.parse_func_blk()
    }
}

#[cfg(test)]
mod test {
    use crate::*;

    #[test]
    fn func_blk_operands() {
        let mut op = AudioFuncBlk {
            func_blk_type: AudioFuncBlkType::Selector,
            func_blk_id: 0xfe,
            ctl_attr: CtlAttr::Resolution,
            ..Default::default()
        };
        op.audio_selector_data
            .extend_from_slice(&[0xde, 0xad, 0xbe, 0xef]);
        op.ctl.selector = 0x11;
        op.ctl.data.extend_from_slice(&[0xbe, 0xef]);

        let mut operands = Vec::new();
        AvcStatus::build_operands(&mut op, &AUDIO_SUBUNIT_0_ADDR, &mut operands).unwrap();
        assert_eq!(
            &operands,
            &[0x80, 0xfe, 0x01, 0x05, 0xde, 0xad, 0xbe, 0xef, 0x11, 0x02, 0xbe, 0xef]
        );

        AvcStatus::parse_operands(&mut op, &AvcAddr::Unit, &operands).unwrap();
        assert_eq!(op.func_blk_type, AudioFuncBlkType::Selector);
        assert_eq!(op.func_blk_id, 0xfe);
        assert_eq!(op.ctl_attr, CtlAttr::Resolution);
        assert_eq!(&op.audio_selector_data, &[0xde, 0xad, 0xbe, 0xef]);
        assert_eq!(op.ctl.selector, 0x11);
        assert_eq!(&op.ctl.data, &[0xbe, 0xef]);

        let mut op = AudioFuncBlk {
            func_blk_type: AudioFuncBlkType::Selector,
            func_blk_id: 0xfd,
            ctl_attr: CtlAttr::Minimum,
            ..Default::default()
        };
        op.audio_selector_data
            .extend_from_slice(&[0xde, 0xad, 0xbe, 0xef]);
        op.ctl.selector = 0x12;
        op.ctl.data.extend_from_slice(&[0xbe, 0xef]);

        let mut operands = Vec::new();
        AvcControl::build_operands(&mut op, &AUDIO_SUBUNIT_0_ADDR, &mut operands).unwrap();
        assert_eq!(
            &operands,
            &[0x80, 0xfd, 0x02, 0x05, 0xde, 0xad, 0xbe, 0xef, 0x12, 0x02, 0xbe, 0xef]
        );

        AvcControl::parse_operands(&mut op, &AvcAddr::Unit, &operands).unwrap();
        assert_eq!(op.func_blk_type, AudioFuncBlkType::Selector);
        assert_eq!(op.func_blk_id, 0xfd);
        assert_eq!(op.ctl_attr, CtlAttr::Minimum);
        assert_eq!(&op.audio_selector_data, &[0xde, 0xad, 0xbe, 0xef]);
        assert_eq!(op.ctl.selector, 0x12);
        assert_eq!(&op.ctl.data, &[0xbe, 0xef]);

        // For the case that audio_selector_data is empty.
        let mut op = AudioFuncBlk {
            func_blk_type: AudioFuncBlkType::Feature,
            func_blk_id: 0xfc,
            ctl_attr: CtlAttr::Maximum,
            ..Default::default()
        };
        op.ctl.selector = 0x13;
        op.ctl.data.extend_from_slice(&[0xfe, 0xeb, 0xda, 0xed]);

        let mut operands = Vec::new();
        AvcStatus::build_operands(&mut op, &AUDIO_SUBUNIT_0_ADDR, &mut operands).unwrap();
        assert_eq!(
            &operands,
            &[0x81, 0xfc, 0x03, 0x01, 0x13, 0x04, 0xfe, 0xeb, 0xda, 0xed]
        );

        AvcStatus::parse_operands(&mut op, &AvcAddr::Unit, &operands).unwrap();
        assert_eq!(op.func_blk_type, AudioFuncBlkType::Feature);
        assert_eq!(op.func_blk_id, 0xfc);
        assert_eq!(op.ctl_attr, CtlAttr::Maximum);
        assert_eq!(&op.audio_selector_data, &[]);
        assert_eq!(op.ctl.selector, 0x13);
        assert_eq!(&op.ctl.data, &[0xfe, 0xeb, 0xda, 0xed]);

        let mut op = AudioFuncBlk {
            func_blk_type: AudioFuncBlkType::Feature,
            func_blk_id: 0xfb,
            ctl_attr: CtlAttr::Default,
            ctl: AudioFuncBlkCtl {
                selector: 0x14,
                data: vec![0xfe, 0xeb, 0xda, 0xed],
            },
            ..Default::default()
        };

        let mut operands = Vec::new();
        AvcControl::build_operands(&mut op, &AUDIO_SUBUNIT_0_ADDR, &mut operands).unwrap();
        assert_eq!(
            &operands,
            &[0x81, 0xfb, 0x04, 0x01, 0x14, 0x04, 0xfe, 0xeb, 0xda, 0xed]
        );

        AvcControl::parse_operands(&mut op, &AvcAddr::Unit, &operands).unwrap();
        assert_eq!(op.func_blk_type, AudioFuncBlkType::Feature);
        assert_eq!(op.func_blk_id, 0xfb);
        assert_eq!(op.ctl_attr, CtlAttr::Default);
        assert_eq!(&op.audio_selector_data, &[]);
        assert_eq!(op.ctl.selector, 0x14);
        assert_eq!(&op.ctl.data, &[0xfe, 0xeb, 0xda, 0xed]);

        // For the case that ctl_data is empty.
        let mut op = AudioFuncBlk {
            func_blk_type: AudioFuncBlkType::Processing,
            func_blk_id: 0xfa,
            ctl_attr: CtlAttr::Duration,
            audio_selector_data: vec![0xda, 0xed],
            ctl: AudioFuncBlkCtl {
                selector: 0x15,
                ..Default::default()
            },
        };

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

        let mut op = AudioFuncBlk {
            func_blk_type: AudioFuncBlkType::Processing,
            func_blk_id: 0xf9,
            ctl_attr: CtlAttr::Current,
            audio_selector_data: vec![0xda, 0xed],
            ctl: AudioFuncBlkCtl {
                selector: 0x16,
                ..Default::default()
            },
        };

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
        assert_eq!(ctl, FeatureCtl::from_ctl(&ctl.to_ctl()));

        let ctl = FeatureCtl::Volume(vec![0x1234, 0x3456, 0x789a]);
        assert_eq!(ctl, FeatureCtl::from_ctl(&ctl.to_ctl()));

        let ctl = FeatureCtl::LrBalance(-123);
        assert_eq!(ctl, FeatureCtl::from_ctl(&ctl.to_ctl()));

        let ctl = FeatureCtl::FrBalance(321);
        assert_eq!(ctl, FeatureCtl::from_ctl(&ctl.to_ctl()));

        let ctl = FeatureCtl::Bass(vec![10, -10, 20, -20]);
        assert_eq!(ctl, FeatureCtl::from_ctl(&ctl.to_ctl()));

        let ctl = FeatureCtl::Mid(vec![30, -30, -40, 40]);
        assert_eq!(ctl, FeatureCtl::from_ctl(&ctl.to_ctl()));

        let data = GraphicEqualizerData {
            bands_present: [0x00, 0x01, 0x02, 0x03],
            ex_bands_present: [0x04, 0x05, 0x06, 0x07],
            gain: vec![
                -1, -2, -3, 10, 14, -40, -100, 33, 87, 99, -123, 100, -76, -97, 18, 21,
            ],
        };
        let ctl = FeatureCtl::GraphicEqualizer(data);
        assert_eq!(ctl, FeatureCtl::from_ctl(&ctl.to_ctl()));

        let ctl = FeatureCtl::Treble(vec![50, 60, -70, -80]);
        assert_eq!(ctl, FeatureCtl::from_ctl(&ctl.to_ctl()));

        let ctl = FeatureCtl::AutomaticGain(vec![false, true, false]);
        assert_eq!(ctl, FeatureCtl::from_ctl(&ctl.to_ctl()));

        let ctl = FeatureCtl::Delay(vec![0x1234, 0x3456, 0x789a]);
        assert_eq!(ctl, FeatureCtl::from_ctl(&ctl.to_ctl()));

        let ctl = FeatureCtl::BassBoost(vec![true, false, true]);
        assert_eq!(ctl, FeatureCtl::from_ctl(&ctl.to_ctl()));

        let ctl = FeatureCtl::Loudness(vec![false, true, false]);
        assert_eq!(ctl, FeatureCtl::from_ctl(&ctl.to_ctl()));

        let ctl = FeatureCtl::Reserved(vec![0xff, 0x04, 0xad, 0xbe, 0xef]);
        assert_eq!(ctl, FeatureCtl::from_ctl(&ctl.to_ctl()));
    }

    #[test]
    fn avcaudiofeature_operands() {
        let ctl = FeatureCtl::Volume(vec![-1234, 5678, 3210]);
        let mut op = AudioFeature::new(0x03, CtlAttr::Minimum, AudioCh::Each(0x1b), ctl.clone());
        let mut operands = Vec::new();
        AvcStatus::build_operands(&mut op, &AUDIO_SUBUNIT_0_ADDR, &mut operands).unwrap();
        assert_eq!(
            &operands,
            &[0x81, 0x03, 0x02, 0x02, 0x1c, 0x02, 0x06, 0xfb, 0x2e, 0x16, 0x2e, 0x0c, 0x8a]
        );

        AvcStatus::parse_operands(&mut op, &AvcAddr::Unit, &operands).unwrap();
        assert_eq!(AudioCh::Each(0x1b), op.audio_ch_num);
        assert_eq!(ctl, op.ctl);

        let ctl = FeatureCtl::Treble(vec![40, -33, 123, -96]);
        let mut op = AudioFeature::new(0x33, CtlAttr::Resolution, AudioCh::Each(0xd8), ctl.clone());
        let mut operands = Vec::new();
        AvcControl::build_operands(&mut op, &AUDIO_SUBUNIT_0_ADDR, &mut operands).unwrap();
        assert_eq!(
            &operands,
            &[0x81, 0x33, 0x01, 0x2, 0xd9, 0x07, 0x04, 0x28, 0xdf, 0x7b, 0xa0]
        );

        AvcControl::parse_operands(&mut op, &AvcAddr::Unit, &operands).unwrap();
        assert_eq!(AudioCh::Each(0xd8), op.audio_ch_num);
        assert_eq!(ctl, op.ctl);
    }

    #[test]
    fn processingctl_from() {
        let ctl = ProcessingCtl::Enable(true);
        assert_eq!(ctl, ProcessingCtl::from_ctl(&ctl.to_ctl()));

        let ctl = ProcessingCtl::Mode(vec![0xde, 0xad, 0xbe, 0xef]);
        assert_eq!(ctl, ProcessingCtl::from_ctl(&ctl.to_ctl()));

        let ctl = ProcessingCtl::Mixer(vec![-73, -157]);
        assert_eq!(ctl, ProcessingCtl::from_ctl(&ctl.to_ctl()));
    }

    #[test]
    fn avcaudioprocessing_operands() {
        let ctl = ProcessingCtl::Enable(true);
        let mut op = AudioProcessing::new(
            0xf5,
            CtlAttr::Default,
            0x71,
            AudioCh::Each(0xa8),
            AudioCh::Each(0x3e),
            ctl.clone(),
        );
        let mut operands = Vec::new();
        AvcStatus::build_operands(&mut op, &AUDIO_SUBUNIT_0_ADDR, &mut operands).unwrap();
        assert_eq!(
            &operands,
            &[0x82, 0xf5, 0x04, 0x04, 0x71, 0xa9, 0x3f, 0x01, 0x01, 0x70]
        );

        AvcStatus::parse_operands(&mut op, &AvcAddr::Unit, &operands).unwrap();
        assert_eq!(0x71, op.input_plug_id);
        assert_eq!(AudioCh::Each(0xa8), op.input_ch);
        assert_eq!(AudioCh::Each(0x3e), op.output_ch);
        assert_eq!(ctl, op.ctl);

        let ctl = ProcessingCtl::Mixer(vec![10, -10]);
        let mut op = AudioProcessing::new(
            0x11,
            CtlAttr::Minimum,
            0x22,
            AudioCh::Each(0x32),
            AudioCh::Each(0x43),
            ctl.clone(),
        );
        let mut operands = Vec::new();
        AvcControl::build_operands(&mut op, &AUDIO_SUBUNIT_0_ADDR, &mut operands).unwrap();
        assert_eq!(
            &operands,
            &[0x82, 0x11, 0x02, 0x04, 0x22, 0x33, 0x44, 0x03, 0x04, 0x00, 0x0a, 0xff, 0xf6]
        );

        AvcControl::parse_operands(&mut op, &AvcAddr::Unit, &operands).unwrap();
        assert_eq!(0x22, op.input_plug_id);
        assert_eq!(AudioCh::Each(0x32), op.input_ch);
        assert_eq!(AudioCh::Each(0x43), op.output_ch);
        assert_eq!(ctl, op.ctl);
    }
}
