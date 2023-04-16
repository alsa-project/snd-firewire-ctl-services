// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol implementation defined by BridgeCo. AG for its BridgeCo. Enhanced Break Out Box
//! (BeBoB) solution.
//!
//! The module includes structure, enumeration, trait and its implementation for AV/C command
//! extensions defined by BridgeCo. AG for BeBoB solution.

use super::*;

//
// Bco Extended Plug Info command
//

/// Type of address to plug for unit.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BcoPlugAddrUnitType {
    /// Address to unit for isochronous input/output.
    Isoc,
    /// Address to unit for external input/output.
    Ext,
    /// Address to unit for asynchronous input/output.
    Async,
}

impl BcoPlugAddrUnitType {
    const ISOC: u8 = 0x00;
    const EXT: u8 = 0x01;
    const ASYNC: u8 = 0x02;

    fn from_val(val: u8) -> Result<Self, AvcRespParseError> {
        let unit_type = match val {
            Self::ISOC => Self::Isoc,
            Self::EXT => Self::Ext,
            Self::ASYNC => Self::Async,
            _ => Err(AvcRespParseError::UnexpectedOperands(0))?,
        };
        Ok(unit_type)
    }

    fn to_val(&self) -> u8 {
        match self {
            Self::Isoc => Self::ISOC,
            Self::Ext => Self::EXT,
            Self::Async => Self::ASYNC,
        }
    }
}

impl Default for BcoPlugAddrUnitType {
    fn default() -> Self {
        Self::Isoc
    }
}

/// Address to plug for unit.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct BcoPlugAddrUnit {
    /// The type of unit to address to.
    pub plug_type: BcoPlugAddrUnitType,
    /// The numeric identifier of plug in the unit.
    pub plug_id: u8,
}

impl BcoPlugAddrUnit {
    const LENGTH: usize = 3;

    fn from_raw(raw: &[u8]) -> Result<Self, AvcRespParseError> {
        if raw.len() < Self::LENGTH {
            Err(AvcRespParseError::TooShortResp(Self::LENGTH))?;
        }

        let plug_type = BcoPlugAddrUnitType::from_val(raw[0])?;
        let plug_id = raw[1];

        Ok(Self { plug_type, plug_id })
    }

    fn to_raw(&self) -> [u8; Self::LENGTH] {
        [self.plug_type.to_val(), self.plug_id.into(), 0xff]
    }
}

impl Default for BcoPlugAddrUnit {
    fn default() -> Self {
        Self {
            plug_type: Default::default(),
            plug_id: 0xff,
        }
    }
}

/// Address to plug for subunit.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct BcoPlugAddrSubunit {
    /// The numeric identifier of plug in the subunit.
    pub plug_id: u8,
}

impl BcoPlugAddrSubunit {
    const LENGTH: usize = 3;

    fn from_raw(raw: &[u8]) -> Result<Self, AvcRespParseError> {
        if raw.len() < Self::LENGTH {
            Err(AvcRespParseError::TooShortResp(Self::LENGTH))?;
        }

        Ok(BcoPlugAddrSubunit { plug_id: raw[0] })
    }

    fn to_raw(&self) -> [u8; Self::LENGTH] {
        [self.plug_id, 0xff, 0xff]
    }
}

impl Default for BcoPlugAddrSubunit {
    fn default() -> Self {
        Self { plug_id: 0xff }
    }
}

/// Address to plug for function block.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct BcoPlugAddrFuncBlk {
    /// The numeric type of function block.
    pub func_blk_type: u8,
    /// The numeric identifier of funtion block.
    pub func_blk_id: u8,
    /// The numneric identifier of plug in the function block.
    pub plug_id: u8,
}

impl BcoPlugAddrFuncBlk {
    const LENGTH: usize = 3;

    fn from_raw(raw: &[u8]) -> Result<Self, AvcRespParseError> {
        if raw.len() < Self::LENGTH {
            Err(AvcRespParseError::TooShortResp(Self::LENGTH))?;
        }

        Ok(Self {
            func_blk_type: raw[0],
            func_blk_id: raw[1],
            plug_id: raw[2],
        })
    }

    fn to_raw(&self) -> [u8; Self::LENGTH] {
        [self.func_blk_type, self.func_blk_id, self.plug_id]
    }
}

impl Default for BcoPlugAddrFuncBlk {
    fn default() -> Self {
        Self {
            func_blk_type: 0xff,
            func_blk_id: 0xff,
            plug_id: 0xff,
        }
    }
}

/// Mode of address to plug.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BcoPlugAddrMode {
    /// Address to unit.
    Unit(BcoPlugAddrUnit),
    /// Address to subunit.
    Subunit(BcoPlugAddrSubunit),
    /// Address to function block.
    FuncBlk(BcoPlugAddrFuncBlk),
}

impl BcoPlugAddrMode {
    const LENGTH: usize = 4;

    const UNIT: u8 = 0x00;
    const SUBUNIT: u8 = 0x01;
    const FUNCBLK: u8 = 0x02;

    fn from_raw(raw: &[u8]) -> Result<Self, AvcRespParseError> {
        if raw.len() < Self::LENGTH {
            Err(AvcRespParseError::TooShortResp(Self::LENGTH))?;
        }

        let mode = match raw[0] {
            Self::UNIT => {
                let data = BcoPlugAddrUnit::from_raw(&raw[1..]).map_err(|err| err.add_offset(1))?;
                Self::Unit(data)
            }
            Self::SUBUNIT => {
                let data =
                    BcoPlugAddrSubunit::from_raw(&raw[1..]).map_err(|err| err.add_offset(1))?;
                Self::Subunit(data)
            }
            Self::FUNCBLK => {
                let data =
                    BcoPlugAddrFuncBlk::from_raw(&raw[1..]).map_err(|err| err.add_offset(1))?;
                Self::FuncBlk(data)
            }
            _ => Err(AvcRespParseError::UnexpectedOperands(0))?,
        };

        Ok(mode)
    }

    fn to_raw(&self) -> [u8; Self::LENGTH] {
        let mut raw = [0; Self::LENGTH];
        match self {
            Self::Unit(d) => {
                raw[0] = Self::UNIT;
                raw[1..].copy_from_slice(&d.to_raw());
            }
            Self::Subunit(d) => {
                raw[0] = Self::SUBUNIT;
                raw[1..].copy_from_slice(&d.to_raw());
            }
            Self::FuncBlk(d) => {
                raw[0] = Self::FUNCBLK;
                raw[1..].copy_from_slice(&d.to_raw());
            }
        }
        raw
    }
}

impl Default for BcoPlugAddrMode {
    fn default() -> Self {
        Self::Unit(Default::default())
    }
}

/// Direction of plug.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BcoPlugDirection {
    /// For input plug.
    Input,
    /// For output plug.
    Output,
}

impl BcoPlugDirection {
    const INPUT: u8 = 0x00;
    const OUTPUT: u8 = 0x01;

    fn from_val(val: u8) -> Result<Self, AvcRespParseError> {
        let direction = match val {
            Self::INPUT => Self::Input,
            Self::OUTPUT => Self::Output,
            _ => Err(AvcRespParseError::UnexpectedOperands(0))?,
        };
        Ok(direction)
    }

    fn to_val(&self) -> u8 {
        match self {
            Self::Input => Self::INPUT,
            Self::Output => Self::OUTPUT,
        }
    }
}

impl Default for BcoPlugDirection {
    fn default() -> Self {
        Self::Input
    }
}

/// Address of plug.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub struct BcoPlugAddr {
    /// The direction of plug.
    pub direction: BcoPlugDirection,
    /// The mode to address for the plug.
    pub mode: BcoPlugAddrMode,
}

impl BcoPlugAddr {
    const LENGTH: usize = 5;

    /// Instantiate address structure to plug for unit.
    pub fn new_for_unit(
        direction: BcoPlugDirection,
        plug_type: BcoPlugAddrUnitType,
        plug_id: u8,
    ) -> Self {
        Self {
            direction,
            mode: BcoPlugAddrMode::Unit(BcoPlugAddrUnit { plug_type, plug_id }),
        }
    }

    /// Instantiate address structure to plug for subunit.
    pub fn new_for_subunit(direction: BcoPlugDirection, plug_id: u8) -> Self {
        Self {
            direction,
            mode: BcoPlugAddrMode::Subunit(BcoPlugAddrSubunit { plug_id }),
        }
    }

    /// Instantiate address structure to plug for function block.
    pub fn new_for_func_blk(
        direction: BcoPlugDirection,
        func_blk_type: u8,
        func_blk_id: u8,
        plug_id: u8,
    ) -> Self {
        Self {
            direction,
            mode: BcoPlugAddrMode::FuncBlk(BcoPlugAddrFuncBlk {
                func_blk_type,
                func_blk_id,
                plug_id,
            }),
        }
    }

    fn from_raw(raw: &[u8]) -> Result<Self, AvcRespParseError> {
        if raw.len() < Self::LENGTH {
            Err(AvcRespParseError::TooShortResp(Self::LENGTH))?;
        }

        let direction = BcoPlugDirection::from_val(raw[0])?;
        let mode = BcoPlugAddrMode::from_raw(&raw[1..]).map_err(|err| err.add_offset(1))?;

        Ok(Self { direction, mode })
    }

    fn to_raw(&self) -> [u8; Self::LENGTH] {
        let mut raw = [0; Self::LENGTH];
        raw[0] = self.direction.to_val();
        raw[1..].copy_from_slice(&self.mode.to_raw());
        raw
    }
}

/// Mode to address to plug for input and output direction.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BcoIoPlugAddrMode {
    /// Address to unit.
    Unit(BcoPlugAddrUnit),
    /// Address to subunit.
    Subunit(AvcAddrSubunit, BcoPlugAddrSubunit),
    /// Address to function block.
    FuncBlk(AvcAddrSubunit, BcoPlugAddrFuncBlk),
}

impl BcoIoPlugAddrMode {
    const LENGTH: usize = 6;

    fn from_raw(raw: &[u8]) -> Result<Self, AvcRespParseError> {
        if raw.len() < Self::LENGTH {
            Err(AvcRespParseError::TooShortResp(Self::LENGTH))?;
        }

        let mode = match raw[0] {
            BcoPlugAddrMode::UNIT => {
                let data = BcoPlugAddrUnit::from_raw(&raw[1..]).map_err(|err| err.add_offset(1))?;
                Self::Unit(data)
            }
            BcoPlugAddrMode::SUBUNIT => {
                let subunit = AvcAddrSubunit {
                    subunit_type: AvcSubunitType::from(raw[1]),
                    subunit_id: raw[2],
                };
                let data =
                    BcoPlugAddrSubunit::from_raw(&raw[3..]).map_err(|err| err.add_offset(3))?;
                Self::Subunit(subunit, data)
            }
            BcoPlugAddrMode::FUNCBLK => {
                let subunit = AvcAddrSubunit {
                    subunit_type: AvcSubunitType::from(raw[1]),
                    subunit_id: raw[2],
                };
                let data =
                    BcoPlugAddrFuncBlk::from_raw(&raw[3..]).map_err(|err| err.add_offset(3))?;
                Self::FuncBlk(subunit, data)
            }
            _ => Err(AvcRespParseError::UnexpectedOperands(0))?,
        };

        Ok(mode)
    }

    fn to_raw(&self) -> [u8; Self::LENGTH] {
        let mut raw = [0xff; Self::LENGTH];
        match self {
            Self::Unit(d) => {
                raw[0] = BcoPlugAddrMode::UNIT;
                raw[1..4].copy_from_slice(&d.to_raw());
            }
            Self::Subunit(s, d) => {
                raw[0] = BcoPlugAddrMode::SUBUNIT;
                raw[1] = s.subunit_type.into();
                raw[2] = s.subunit_id;
                raw[3..6].copy_from_slice(&d.to_raw());
            }
            Self::FuncBlk(s, d) => {
                raw[0] = BcoPlugAddrMode::FUNCBLK;
                raw[1] = s.subunit_type.into();
                raw[2] = s.subunit_id;
                raw[3..6].copy_from_slice(&d.to_raw());
            }
        }
        raw
    }
}

impl Default for BcoIoPlugAddrMode {
    fn default() -> Self {
        Self::Unit(Default::default())
    }
}

/// Address to plug for input and output direction.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub struct BcoIoPlugAddr {
    /// For direction of plug.
    pub direction: BcoPlugDirection,
    /// The mode to address for the plug.
    pub mode: BcoIoPlugAddrMode,
}

impl BcoIoPlugAddr {
    const LENGTH: usize = 7;

    fn from_raw(raw: &[u8]) -> Result<Self, AvcRespParseError> {
        if raw.len() < Self::LENGTH {
            Err(AvcRespParseError::TooShortResp(Self::LENGTH))?;
        }

        let direction = BcoPlugDirection::from_val(raw[0])?;
        let mode = BcoIoPlugAddrMode::from_raw(&raw[1..]).map_err(|err| err.add_offset(1))?;

        Ok(Self { direction, mode })
    }

    fn to_raw(&self) -> [u8; Self::LENGTH] {
        let mut raw = [0; Self::LENGTH];
        raw[0] = self.direction.to_val();
        raw[1..].copy_from_slice(&self.mode.to_raw());
        raw
    }
}

/// The type of plug.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BcoPlugType {
    /// For isochronous communication.
    Isoc,
    /// For asynchronous communication.
    Async,
    /// For MIDI messages.
    Midi,
    /// For synchronization.
    Sync,
    /// For analog signal.
    Analog,
    /// For digital signal.
    Digital,
}

impl BcoPlugType {
    const ISOC_STREAM: u8 = 0x00;
    const ASYNC_STREAM: u8 = 0x01;
    const MIDI: u8 = 0x02;
    const SYNC: u8 = 0x03;
    const ANALOG: u8 = 0x04;
    const DIGITAL: u8 = 0x05;

    fn from_val(val: u8) -> Result<Self, AvcRespParseError> {
        let plug_type = match val {
            Self::ISOC_STREAM => Self::Isoc,
            Self::ASYNC_STREAM => Self::Async,
            Self::MIDI => Self::Midi,
            Self::SYNC => Self::Sync,
            Self::ANALOG => Self::Analog,
            Self::DIGITAL => Self::Digital,
            _ => Err(AvcRespParseError::UnexpectedOperands(0))?,
        };
        Ok(plug_type)
    }

    fn to_val(&self) -> u8 {
        match self {
            Self::Isoc => Self::ISOC_STREAM,
            Self::Async => Self::ASYNC_STREAM,
            Self::Midi => Self::MIDI,
            Self::Sync => Self::SYNC,
            Self::Analog => Self::ANALOG,
            Self::Digital => Self::DIGITAL,
        }
    }
}

impl Default for BcoPlugType {
    fn default() -> Self {
        Self::Isoc
    }
}

/// Physical location of data channel for multi bit linear audio.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BcoLocation {
    LeftFront,
    RightFront,
    Center,
    LowFrequencyEffect,
    LeftSurround,
    RightSurround,
    LeftCenter,
    RightCenter,
    Surround,
    SideLeft,
    SideRight,
    Top,
    Bottom,
    LeftFrontEffect,
    RightFrontEffect,
    NoPosition,
}

impl BcoLocation {
    const L: u8 = 0x01;
    const R: u8 = 0x02;
    const C: u8 = 0x03;
    const LFE: u8 = 0x04;
    const LS: u8 = 0x05;
    const RS: u8 = 0x06;
    const LC: u8 = 0x07;
    const RC: u8 = 0x08;
    const S: u8 = 0x09;
    const SL: u8 = 0x0a;
    const SR: u8 = 0x0b;
    const T: u8 = 0x0c;
    const B: u8 = 0x0d;
    const FEL: u8 = 0x0e;
    const FER: u8 = 0x0f;
    const NO_POSITION: u8 = 0xff;

    fn from_val(val: u8) -> Result<Self, AvcRespParseError> {
        let loc = match val {
            Self::L => Self::LeftFront,
            Self::R => Self::RightFront,
            Self::C => Self::Center,
            Self::LFE => Self::LowFrequencyEffect,
            Self::LS => Self::LeftSurround,
            Self::RS => Self::RightSurround,
            Self::LC => Self::LeftCenter,
            Self::RC => Self::RightCenter,
            Self::S => Self::Surround,
            Self::SL => Self::SideLeft,
            Self::SR => Self::SideRight,
            Self::T => Self::Top,
            Self::B => Self::Bottom,
            Self::FEL => Self::LeftFrontEffect,
            Self::FER => Self::RightFrontEffect,
            Self::NO_POSITION => Self::NoPosition,
            _ => Err(AvcRespParseError::UnexpectedOperands(0))?,
        };
        Ok(loc)
    }

    fn to_val(&self) -> u8 {
        match self {
            Self::LeftFront => Self::L,
            Self::RightFront => Self::R,
            Self::Center => Self::C,
            Self::LowFrequencyEffect => Self::LFE,
            Self::LeftSurround => Self::LS,
            Self::RightSurround => Self::RS,
            Self::LeftCenter => Self::LC,
            Self::RightCenter => Self::RC,
            Self::Surround => Self::S,
            Self::SideLeft => Self::SL,
            Self::SideRight => Self::SR,
            Self::Top => Self::T,
            Self::Bottom => Self::B,
            Self::LeftFrontEffect => Self::FEL,
            Self::RightFrontEffect => Self::FER,
            Self::NoPosition => Self::NO_POSITION,
        }
    }
}

impl Default for BcoLocation {
    fn default() -> Self {
        Self::NoPosition
    }
}

/// Information about data channel for multi bit linear audio.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct BcoChannelInfo {
    /// The position of channel in data frame.
    pos: u8,
    /// The location of channel for playback or capture.
    loc: BcoLocation,
}

impl BcoChannelInfo {
    const LENGTH: usize = 2;

    fn to_raw(&self) -> [u8; Self::LENGTH] {
        let mut raw = [0; Self::LENGTH];
        raw[0] = self.pos;
        raw[1] = self.loc.to_val();
        raw
    }

    fn from_raw(raw: &[u8]) -> Result<Self, AvcRespParseError> {
        if raw.len() < Self::LENGTH {
            Err(AvcRespParseError::TooShortResp(Self::LENGTH))?;
        }

        let pos = raw[0];
        let loc = BcoLocation::from_val(raw[1]).map_err(|err| err.add_offset(1))?;

        Ok(Self { pos, loc })
    }
}

impl Default for BcoChannelInfo {
    fn default() -> Self {
        Self {
            pos: 0xff,
            loc: Default::default(),
        }
    }
}

/// Cluster with single or multiple data channels.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct BcoCluster {
    /// The entries of cluster.
    entries: Vec<BcoChannelInfo>,
}

impl BcoCluster {
    const LENGTH_MIN: usize = 1;

    fn from_raw(raw: &[u8]) -> Result<Self, AvcRespParseError> {
        if raw.len() < Self::LENGTH_MIN {
            Err(AvcRespParseError::TooShortResp(Self::LENGTH_MIN))?;
        }

        let count = raw[0] as usize;

        let mut entries = Vec::with_capacity(count);
        let mut pos = 1;
        while pos + BcoChannelInfo::LENGTH <= raw.len() {
            let entry = BcoChannelInfo::from_raw(&raw[pos..]).map_err(|err| err.add_offset(pos))?;
            entries.push(entry);
            pos += BcoChannelInfo::LENGTH;
        }

        if entries.len() != count {
            Err(AvcRespParseError::UnexpectedOperands(0))?;
        }

        Ok(Self { entries })
    }

    fn to_raw(&self) -> Vec<u8> {
        let mut raw = Vec::new();
        raw.push(self.entries.len() as u8);
        self.entries.iter().fold(raw, |mut raw, entry| {
            raw.extend_from_slice(&entry.to_raw());
            raw
        })
    }
}

/// Name of data channel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BcoChannelName {
    /// The channel in data frame.
    pub ch: u8,
    /// The name of channel.
    pub name: String,
}

impl Default for BcoChannelName {
    fn default() -> Self {
        Self {
            ch: 0xff,
            name: Default::default(),
        }
    }
}

/// Type of physical port.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BcoPortType {
    Speaker,
    Headphone,
    Microphone,
    Line,
    Spdif,
    Adat,
    Tdif,
    Madi,
    Analog,
    Digital,
    Midi,
    NoType,
}

impl BcoPortType {
    const SPEAKER: u8 = 0x00;
    const HEADPHONE: u8 = 0x01;
    const MICROPHONE: u8 = 0x02;
    const LINE: u8 = 0x03;
    const SPDIF: u8 = 0x04;
    const ADAT: u8 = 0x05;
    const TDIF: u8 = 0x06;
    const MADI: u8 = 0x07;
    const ANALOG: u8 = 0x08;
    const DIGITAL: u8 = 0x09;
    const MIDI: u8 = 0x0a;
    const NO_TYPE: u8 = 0xff;

    fn to_val(&self) -> u8 {
        match self {
            Self::Speaker => Self::SPEAKER,
            Self::Headphone => Self::HEADPHONE,
            Self::Microphone => Self::MICROPHONE,
            Self::Line => Self::LINE,
            Self::Spdif => Self::SPDIF,
            Self::Adat => Self::ADAT,
            Self::Tdif => Self::TDIF,
            Self::Madi => Self::MADI,
            Self::Analog => Self::ANALOG,
            Self::Digital => Self::DIGITAL,
            Self::Midi => Self::MIDI,
            Self::NoType => Self::NO_TYPE,
        }
    }

    fn from_val(val: u8) -> Result<Self, AvcRespParseError> {
        let port_type = match val {
            Self::SPEAKER => Self::Speaker,
            Self::HEADPHONE => Self::Headphone,
            Self::MICROPHONE => Self::Microphone,
            Self::LINE => Self::Line,
            Self::SPDIF => Self::Spdif,
            Self::ADAT => Self::Adat,
            Self::TDIF => Self::Tdif,
            Self::MADI => Self::Madi,
            Self::ANALOG => Self::Analog,
            Self::DIGITAL => Self::Digital,
            Self::MIDI => Self::Midi,
            Self::NO_TYPE => Self::NoType,
            _ => Err(AvcRespParseError::UnexpectedOperands(0))?,
        };
        Ok(port_type)
    }
}

impl Default for BcoPortType {
    fn default() -> Self {
        Self::NoType
    }
}

/// Information about cluster of data channels.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BcoClusterInfo {
    /// The index of cluster.
    pub index: u8,
    /// The type of port for the cluster.
    pub port_type: BcoPortType,
    /// The name of cluster.
    pub name: String,
}

impl BcoClusterInfo {
    const LENGTH_MIN: usize = 3;

    fn from_raw(raw: &[u8]) -> Result<Self, AvcRespParseError> {
        if raw.len() < Self::LENGTH_MIN {
            Err(AvcRespParseError::TooShortResp(Self::LENGTH_MIN))?;
        }

        let index = raw[0];
        let port_type = BcoPortType::from_val(raw[1]).map_err(|err| err.add_offset(1))?;
        let pos = Self::LENGTH_MIN + raw[2] as usize;
        let name = if pos > raw.len() {
            "".to_string()
        } else {
            String::from_utf8(raw[Self::LENGTH_MIN..pos].to_vec()).unwrap_or("".to_string())
        };
        Ok(Self {
            index,
            port_type,
            name,
        })
    }

    fn to_raw(&self) -> Vec<u8> {
        let mut raw = Vec::new();
        raw.push(self.index);
        raw.push(self.port_type.to_val());
        raw.push(self.name.len() as u8);
        raw.append(&mut self.name.clone().into_bytes());
        raw
    }
}

impl Default for BcoClusterInfo {
    fn default() -> Self {
        Self {
            index: 0xff,
            port_type: Default::default(),
            name: Default::default(),
        }
    }
}

/// Type of information about plug.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BcoPlugInfo {
    /// The type of plug.
    Type(BcoPlugType),
    /// The name of plug.
    Name(String),
    /// The number of channels in the plug.
    ChCount(u8),
    /// The position of channels in each cluster in the plug.
    ChPositions(Vec<BcoCluster>),
    /// The name of channel in the plug.
    ChName(BcoChannelName),
    /// The plug information as signal source to the plug.
    Input(BcoIoPlugAddr),
    /// The plug information as signal destination from the plug.
    Outputs(Vec<BcoIoPlugAddr>),
    /// The data of each cluster in the plug.
    ClusterInfo(BcoClusterInfo),
    Reserved(Vec<u8>),
}

impl BcoPlugInfo {
    const TYPE: u8 = 0x00;
    const NAME: u8 = 0x01;
    const CH_COUNT: u8 = 0x02;
    const CH_POSITIONS: u8 = 0x03;
    const CH_NAME: u8 = 0x04;
    const INPUT: u8 = 0x05;
    const OUTPUTS: u8 = 0x06;
    const CLUSTER_INFO: u8 = 0x07;

    const LENGTH_MIN: usize = 2;

    fn to_raw(&self) -> Vec<u8> {
        let mut raw = Vec::with_capacity(Self::LENGTH_MIN);
        match self {
            Self::Type(plug_type) => {
                raw.push(Self::TYPE);
                raw.push(plug_type.to_val());
            }
            Self::Name(n) => {
                raw.push(Self::NAME);
                raw.push(n.len() as u8);
                raw.append(&mut n.clone().into_bytes());
            }
            Self::ChCount(c) => {
                raw.push(Self::CH_COUNT);
                raw.push(*c);
            }
            Self::ChPositions(entries) => {
                raw.push(Self::CH_POSITIONS);
                raw.push(entries.len() as u8);
                entries
                    .iter()
                    .for_each(|entry| raw.append(&mut entry.to_raw()));
            }
            Self::ChName(d) => {
                raw.push(Self::CH_NAME);
                raw.push(d.ch);
                raw.push(d.name.len() as u8);
                raw.append(&mut d.name.clone().into_bytes());
            }
            Self::Input(plug_addr) => {
                raw.push(Self::INPUT);
                raw.extend_from_slice(&mut plug_addr.to_raw());
            }
            Self::Outputs(plug_addrs) => {
                raw.push(Self::OUTPUTS);
                raw.push(plug_addrs.len() as u8);
                plug_addrs
                    .iter()
                    .for_each(|plug_addr| raw.extend_from_slice(&plug_addr.to_raw()));
            }
            Self::ClusterInfo(d) => {
                raw.push(Self::CLUSTER_INFO);
                raw.append(&mut d.to_raw());
            }
            Self::Reserved(d) => raw.extend_from_slice(&d),
        }
        raw
    }

    fn from_raw(raw: &[u8]) -> Result<Self, AvcRespParseError> {
        if raw.len() < Self::LENGTH_MIN {
            Err(AvcRespParseError::TooShortResp(Self::LENGTH_MIN))?;
        }

        let info = match raw[0] {
            Self::TYPE => {
                let plug_type = BcoPlugType::from_val(raw[1])?;
                Self::Type(plug_type)
            }
            Self::NAME => {
                let pos = Self::LENGTH_MIN + raw[1] as usize;
                let name = if pos > raw.len() {
                    "".to_string()
                } else {
                    String::from_utf8(raw[2..pos].to_vec()).unwrap_or("".to_string())
                };
                Self::Name(name)
            }
            Self::CH_COUNT => Self::ChCount(raw[1]),
            Self::CH_POSITIONS => {
                let count = raw[1] as usize;
                let mut entries = Vec::with_capacity(count);
                let mut pos = 2;
                while pos < raw.len() && entries.len() < count {
                    let c = raw[pos] as usize;
                    let size = 1 + 2 * c;
                    let entry = BcoCluster::from_raw(&raw[pos..(pos + size)])
                        .map_err(|err| err.add_offset(pos))?;
                    entries.push(entry);
                    pos += size;
                }
                Self::ChPositions(entries)
            }
            Self::CH_NAME => {
                let ch = raw[1] as u8;
                let pos = 3 + raw[2] as usize;
                let name = if pos > raw.len() {
                    "".to_string()
                } else {
                    String::from_utf8(raw[3..pos].to_vec()).unwrap_or("".to_string())
                };
                Self::ChName(BcoChannelName { ch, name })
            }
            Self::INPUT => {
                let addr = BcoIoPlugAddr::from_raw(&raw[1..]).map_err(|err| err.add_offset(1))?;
                Self::Input(addr)
            }
            Self::OUTPUTS => {
                let count = raw[1] as usize;
                let mut entries = Vec::with_capacity(count);
                let mut pos = 2;
                while pos + BcoIoPlugAddr::LENGTH <= raw.len() && entries.len() != count {
                    let entry = BcoIoPlugAddr::from_raw(&raw[pos..(pos + BcoIoPlugAddr::LENGTH)])
                        .map_err(|err| err.add_offset(pos))?;
                    entries.push(entry);
                    pos += BcoIoPlugAddr::LENGTH;
                }
                Self::Outputs(entries)
            }
            Self::CLUSTER_INFO => {
                let info = BcoClusterInfo::from_raw(&raw[1..]).map_err(|err| err.add_offset(1))?;
                Self::ClusterInfo(info)
            }
            _ => Self::Reserved(raw.to_vec()),
        };

        Ok(info)
    }
}

impl Default for BcoPlugInfo {
    fn default() -> Self {
        Self::Type(Default::default())
    }
}

/// AV/C command for extend plug information.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct ExtendedPlugInfo {
    /// The address of plug.
    pub addr: BcoPlugAddr,
    /// The type of plug information
    pub info: BcoPlugInfo,
}

impl ExtendedPlugInfo {
    const SUBFUNC: u8 = 0xc0;

    /// Instantiate extended plug info structure with parameters.
    pub fn new(addr: &BcoPlugAddr, info: BcoPlugInfo) -> Self {
        Self { addr: *addr, info }
    }
}

impl AvcOp for ExtendedPlugInfo {
    const OPCODE: u8 = PlugInfo::OPCODE;
}

impl AvcStatus for ExtendedPlugInfo {
    fn build_operands(&mut self, _: &AvcAddr) -> Result<Vec<u8>, AvcCmdBuildError> {
        let mut operands = Vec::new();
        operands.push(Self::SUBFUNC);
        operands.extend_from_slice(&self.addr.to_raw());
        operands.append(&mut self.info.to_raw());
        Ok(operands)
    }

    fn parse_operands(&mut self, _: &AvcAddr, operands: &[u8]) -> Result<(), AvcRespParseError> {
        if operands.len() < 8 {
            Err(AvcRespParseError::TooShortResp(8))?;
        }

        if operands[0] != Self::SUBFUNC {
            Err(AvcRespParseError::UnexpectedOperands(0))?;
        }

        let addr = BcoPlugAddr::from_raw(&operands[1..]).map_err(|err| err.add_offset(1))?;
        if addr != self.addr {
            Err(AvcRespParseError::UnexpectedOperands(1))?;
        }

        let info_type = match &self.info {
            BcoPlugInfo::Type(_) => BcoPlugInfo::TYPE,
            BcoPlugInfo::Name(_) => BcoPlugInfo::NAME,
            BcoPlugInfo::ChCount(_) => BcoPlugInfo::CH_COUNT,
            BcoPlugInfo::ChPositions(_) => BcoPlugInfo::CH_POSITIONS,
            BcoPlugInfo::ChName(_) => BcoPlugInfo::CH_NAME,
            BcoPlugInfo::Input(_) => BcoPlugInfo::INPUT,
            BcoPlugInfo::Outputs(_) => BcoPlugInfo::OUTPUTS,
            BcoPlugInfo::ClusterInfo(_) => BcoPlugInfo::CLUSTER_INFO,
            BcoPlugInfo::Reserved(d) => d[0],
        };
        if info_type != operands[6] {
            Err(AvcRespParseError::UnexpectedOperands(6))?;
        }

        self.info = BcoPlugInfo::from_raw(&operands[6..]).map_err(|err| err.add_offset(6))?;

        Ok(())
    }
}

//
// Bco Extended Subunit Info command
//

/// Entry for information about plugs in subunit.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ExtendedSubunitInfoEntry {
    /// The type of function block.
    pub func_blk_type: u8,
    /// The numeric identifier of function block.
    pub func_blk_id: u8,
    /// The purpose of function block.
    pub func_blk_purpose: u8,
    /// The number of input plugs.
    pub input_plugs: u8,
    /// The number of output plugs.
    pub output_plugs: u8,
}

impl ExtendedSubunitInfoEntry {
    const LENGTH: usize = 5;

    fn from_raw(raw: &[u8]) -> Result<Self, AvcRespParseError> {
        if raw.len() < Self::LENGTH {
            Err(AvcRespParseError::TooShortResp(Self::LENGTH))?;
        }

        Ok(Self {
            func_blk_type: raw[0],
            func_blk_id: raw[1],
            func_blk_purpose: raw[2],
            input_plugs: raw[3],
            output_plugs: raw[4],
        })
    }

    #[allow(dead_code)]
    fn to_raw(&self) -> [u8; 5] {
        [
            self.func_blk_type,
            self.func_blk_id,
            self.func_blk_purpose,
            self.input_plugs,
            self.output_plugs,
        ]
    }
}

impl Default for ExtendedSubunitInfoEntry {
    fn default() -> Self {
        Self {
            func_blk_type: 0xff,
            func_blk_id: 0xff,
            func_blk_purpose: 0xff,
            input_plugs: 0xff,
            output_plugs: 0xff,
        }
    }
}

/// AV/C command for extended subunit information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtendedSubunitInfo {
    /// The numeric identifier of page.
    pub page: u8,
    /// The type of function block.
    pub func_blk_type: u8,
    /// The entries for subunit information.
    pub entries: Vec<ExtendedSubunitInfoEntry>,
}

impl ExtendedSubunitInfo {
    const LENGTH: usize = 27;

    /// Instantiate structure with the numeric identifier of page and the type of function block.
    pub fn new(page: u8, func_blk_type: u8) -> Self {
        Self {
            page,
            func_blk_type,
            ..Default::default()
        }
    }
}

impl Default for ExtendedSubunitInfo {
    fn default() -> Self {
        Self {
            page: 0xff,
            func_blk_type: 0xff,
            entries: Default::default(),
        }
    }
}

impl AvcOp for ExtendedSubunitInfo {
    const OPCODE: u8 = SubunitInfo::OPCODE;
}

impl AvcStatus for ExtendedSubunitInfo {
    fn build_operands(&mut self, _: &AvcAddr) -> Result<Vec<u8>, AvcCmdBuildError> {
        let mut operands = vec![0xff; Self::LENGTH];
        operands[0] = self.page;
        operands[1] = self.func_blk_type;
        Ok(operands)
    }

    fn parse_operands(&mut self, _: &AvcAddr, operands: &[u8]) -> Result<(), AvcRespParseError> {
        if operands.len() != 27 {
            Err(AvcRespParseError::TooShortResp(27))
        } else if self.page != operands[0] {
            Err(AvcRespParseError::UnexpectedOperands(0))
        } else if self.func_blk_type != operands[1] {
            Err(AvcRespParseError::UnexpectedOperands(1))
        } else {
            let mut entries = Vec::new();
            let mut pos = 2;
            while pos + ExtendedSubunitInfoEntry::LENGTH <= operands.len() {
                if operands[pos] != 0xff {
                    let entry = ExtendedSubunitInfoEntry::from_raw(&operands[pos..])
                        .map_err(|err| err.add_offset(pos))?;
                    entries.push(entry);
                }
                pos += ExtendedSubunitInfoEntry::LENGTH;
            }
            self.entries = entries;
            Ok(())
        }
    }
}

//
// Bco Extended Stream Format Info command
//

/// Format of compound AM824 stream.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BcoCompoundAm824StreamFormat {
    /// For IEC 60958-3.
    Iec60958_3,
    /// For IEC 61937-3.
    Iec61937_3,
    /// For IEC 61937-4.
    Iec61937_4,
    /// For IEC 61937-5.
    Iec61937_5,
    /// For IEC 61937-6.
    Iec61937_6,
    /// For IEC 61937-7.
    Iec61937_7,
    /// For multi bit linear audio (raw).
    MultiBitLinearAudioRaw,
    /// For multi bit linear audio (DVD-Audio).
    MultiBitLinearAudioDvd,
    /// For high precision multi bit linear audio.
    HighPrecisionMultiBitLinearAudio,
    /// For MIDI conformant (MMA/AMEI RP-027).
    MidiConformant,
    Reserved(u8),
}

impl BcoCompoundAm824StreamFormat {
    const IEC60958_3: u8 = 0x00;
    const IEC61937_3: u8 = 0x01;
    const IEC61937_4: u8 = 0x02;
    const IEC61937_5: u8 = 0x03;
    const IEC61937_6: u8 = 0x04;
    const IEC61937_7: u8 = 0x05;
    const MULTI_BIT_LINEAR_AUDIO_RAW: u8 = 0x06;
    const MULTI_BIT_LINEAR_AUDIO_DVD: u8 = 0x07;
    const HIGH_PRECISION_MULTI_BIT_LINEAR_AUDIO: u8 = 0x0c;
    const MIDI_CONFORMANT: u8 = 0x0d;

    fn from_val(val: u8) -> Self {
        match val {
            Self::IEC60958_3 => Self::Iec60958_3,
            Self::IEC61937_3 => Self::Iec61937_3,
            Self::IEC61937_4 => Self::Iec61937_4,
            Self::IEC61937_5 => Self::Iec61937_5,
            Self::IEC61937_6 => Self::Iec61937_6,
            Self::IEC61937_7 => Self::Iec61937_7,
            Self::MULTI_BIT_LINEAR_AUDIO_RAW => Self::MultiBitLinearAudioRaw,
            Self::MULTI_BIT_LINEAR_AUDIO_DVD => Self::MultiBitLinearAudioDvd,
            Self::HIGH_PRECISION_MULTI_BIT_LINEAR_AUDIO => Self::HighPrecisionMultiBitLinearAudio,
            Self::MIDI_CONFORMANT => Self::MidiConformant,
            _ => Self::Reserved(val),
        }
    }

    fn to_val(&self) -> u8 {
        match self {
            Self::Iec60958_3 => Self::IEC60958_3,
            Self::Iec61937_3 => Self::IEC61937_3,
            Self::Iec61937_4 => Self::IEC61937_4,
            Self::Iec61937_5 => Self::IEC61937_5,
            Self::Iec61937_6 => Self::IEC61937_6,
            Self::Iec61937_7 => Self::IEC61937_7,
            Self::MultiBitLinearAudioRaw => Self::MULTI_BIT_LINEAR_AUDIO_RAW,
            Self::MultiBitLinearAudioDvd => Self::MULTI_BIT_LINEAR_AUDIO_DVD,
            Self::HighPrecisionMultiBitLinearAudio => Self::HIGH_PRECISION_MULTI_BIT_LINEAR_AUDIO,
            Self::MidiConformant => Self::MIDI_CONFORMANT,
            Self::Reserved(val) => *val,
        }
    }
}

impl Default for BcoCompoundAm824StreamFormat {
    fn default() -> Self {
        Self::Reserved(0xff)
    }
}

/// Entry for compound AM824 stream.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BcoCompoundAm824StreamEntry {
    /// The number of data channels.
    pub count: u8,
    /// The format of data channel.
    pub format: BcoCompoundAm824StreamFormat,
}

impl BcoCompoundAm824StreamEntry {
    const LENGTH: usize = 2;

    fn from_raw(raw: &[u8]) -> Result<Self, AvcRespParseError> {
        if raw.len() < Self::LENGTH {
            Err(AvcRespParseError::TooShortResp(Self::LENGTH))?;
        }

        Ok(BcoCompoundAm824StreamEntry {
            count: raw[0],
            format: BcoCompoundAm824StreamFormat::from_val(raw[1]),
        })
    }

    fn to_raw(&self) -> [u8; Self::LENGTH] {
        [self.count, self.format.to_val()]
    }
}

impl Default for BcoCompoundAm824StreamEntry {
    fn default() -> Self {
        Self {
            count: 0xff,
            format: Default::default(),
        }
    }
}

/// Parameters for compound AM824 stream.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BcoCompoundAm824Stream {
    /// The nominal frequency of media clock.
    pub freq: u32,
    /// Whether to atopt Command-based rate control defined in 1394 Trading Association.
    pub rate_ctl: bool,
    /// The entries of available stream format.
    pub entries: Vec<BcoCompoundAm824StreamEntry>,
}

impl BcoCompoundAm824Stream {
    const FREQ_CODE_22050: u8 = 0x00;
    const FREQ_CODE_24000: u8 = 0x01;
    const FREQ_CODE_32000: u8 = 0x02;
    const FREQ_CODE_44100: u8 = 0x03;
    const FREQ_CODE_48000: u8 = 0x04;
    const FREQ_CODE_96000: u8 = 0x05;
    const FREQ_CODE_176400: u8 = 0x06;
    const FREQ_CODE_192000: u8 = 0x07;
    const FREQ_CODE_88200: u8 = 0x0a;

    const RATE_CTL_MASK: u8 = 0x03;
    const RATE_CTL_SHIFT: usize = 0;

    const LENGTH_MIN: usize = 3;

    fn from_raw(raw: &[u8]) -> Result<Self, AvcRespParseError> {
        if raw.len() < Self::LENGTH_MIN {
            Err(AvcRespParseError::TooShortResp(Self::LENGTH_MIN))?;
        }

        let freq = match raw[0] {
            Self::FREQ_CODE_22050 => 22050,
            Self::FREQ_CODE_24000 => 24000,
            Self::FREQ_CODE_32000 => 32000,
            Self::FREQ_CODE_44100 => 44100,
            Self::FREQ_CODE_48000 => 48000,
            Self::FREQ_CODE_96000 => 96000,
            Self::FREQ_CODE_176400 => 176400,
            Self::FREQ_CODE_192000 => 192000,
            Self::FREQ_CODE_88200 => 88200,
            _ => Err(AvcRespParseError::UnexpectedOperands(0))?,
        };
        let rate_ctl_code = (raw[1] >> Self::RATE_CTL_SHIFT) & Self::RATE_CTL_MASK;
        let rate_ctl = rate_ctl_code == 0;
        let entry_count = raw[2] as usize;

        let mut entries = Vec::with_capacity(entry_count);
        let mut pos = 3;
        while pos + BcoCompoundAm824StreamEntry::LENGTH <= raw.len() && entries.len() < entry_count
        {
            let entry = BcoCompoundAm824StreamEntry::from_raw(&raw[pos..])
                .map_err(|err| err.add_offset(pos))?;
            entries.push(entry);
            pos += BcoCompoundAm824StreamEntry::LENGTH;
        }

        Ok(Self {
            freq,
            rate_ctl,
            entries,
        })
    }

    fn to_raw(&self) -> Result<Vec<u8>, AvcCmdBuildError> {
        let mut raw = Vec::with_capacity(Self::LENGTH_MIN);
        let freq_code = match self.freq {
            22050 => Self::FREQ_CODE_22050,
            24000 => Self::FREQ_CODE_24000,
            32000 => Self::FREQ_CODE_32000,
            44100 => Self::FREQ_CODE_44100,
            48000 => Self::FREQ_CODE_48000,
            96000 => Self::FREQ_CODE_96000,
            176400 => Self::FREQ_CODE_176400,
            192000 => Self::FREQ_CODE_192000,
            88200 => Self::FREQ_CODE_88200,
            _ => Err(AvcCmdBuildError::InvalidOperands)?,
        };
        raw.push(freq_code);

        let rate_ctl_code = ((self.rate_ctl as u8) & Self::RATE_CTL_MASK) << Self::RATE_CTL_SHIFT;
        raw.push(rate_ctl_code);

        raw.push(self.entries.len() as u8);
        self.entries.iter().for_each(|entry| {
            raw.extend_from_slice(&entry.to_raw());
        });

        Ok(raw)
    }
}

impl Default for BcoCompoundAm824Stream {
    fn default() -> Self {
        Self {
            freq: 44100,
            rate_ctl: Default::default(),
            entries: Default::default(),
        }
    }
}

/// Format of isochronous packet stream for Audio and Music data transmission.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BcoAmStream {
    /// For format compliant to AM824 stream.
    AmStream(AmStream),
    /// For format compliant to Compound AM824 stream specific to BridgeCo.
    BcoStream(BcoCompoundAm824Stream),
}

impl BcoAmStream {
    const LENGTH_MIN: usize = 1;

    fn from_raw(raw: &[u8]) -> Result<Self, AvcRespParseError> {
        if raw.len() < Self::LENGTH_MIN {
            Err(AvcRespParseError::TooShortResp(Self::LENGTH_MIN))?;
        }

        let fmt = match raw[0] {
            // MEMO: overwrite compound AM824 stream.
            AmStream::HIER_LEVEL_1_COMPOUND_AM824 => {
                let s =
                    BcoCompoundAm824Stream::from_raw(&raw[1..]).map_err(|err| err.add_offset(1))?;
                Self::BcoStream(s)
            }
            _ => {
                let am = AmStream::from_raw(raw)?;
                Self::AmStream(am)
            }
        };
        Ok(fmt)
    }

    fn to_raw(&self) -> Result<Vec<u8>, AvcCmdBuildError> {
        match self {
            Self::BcoStream(s) => {
                let mut raw = Vec::with_capacity(Self::LENGTH_MIN);
                raw.push(AmStream::HIER_LEVEL_1_COMPOUND_AM824);
                let mut r = s.to_raw()?;
                raw.append(&mut r);
                Ok(raw)
            }
            _ => self.to_raw(),
        }
    }
}

impl Default for BcoAmStream {
    fn default() -> Self {
        Self::BcoStream(Default::default())
    }
}

/// Format of isochronous packet stream.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BcoStreamFormat {
    /// For Audio and music data stream.
    Am(BcoAmStream),
    /// DVCR is not supported in the implementation..
    Reserved(Vec<u8>),
}

impl BcoStreamFormat {
    const LENGTH_MIN: usize = 1;

    fn as_bco_am_stream(&self) -> Option<&BcoAmStream> {
        if let BcoStreamFormat::Am(s) = self {
            Some(s)
        } else {
            None
        }
    }

    /// Parse for AM824 stream.
    pub fn as_am_stream(&self) -> Option<&AmStream> {
        if let BcoAmStream::AmStream(s) = self.as_bco_am_stream()? {
            Some(s)
        } else {
            None
        }
    }

    /// Parse for Compound AM824 stream specific to BridgeCo.
    pub fn as_bco_compound_am824_stream(&self) -> Option<&BcoCompoundAm824Stream> {
        if let BcoAmStream::BcoStream(s) = self.as_bco_am_stream()? {
            Some(s)
        } else {
            None
        }
    }

    fn from_raw(raw: &[u8]) -> Result<Self, AvcRespParseError> {
        if raw.len() < Self::LENGTH_MIN {
            Err(AvcRespParseError::TooShortResp(Self::LENGTH_MIN))?;
        }

        let fmt = match raw[0] {
            StreamFormat::HIER_ROOT_AM => {
                let am = BcoAmStream::from_raw(&raw[1..]).map_err(|err| err.add_offset(1))?;
                Self::Am(am)
            }
            _ => Self::Reserved(raw.to_vec()),
        };
        Ok(fmt)
    }

    fn to_raw(&self) -> Result<Vec<u8>, AvcCmdBuildError> {
        let mut raw = Vec::with_capacity(Self::LENGTH_MIN);
        match self {
            Self::Am(am) => {
                raw.push(StreamFormat::HIER_ROOT_AM);
                let mut r = am.to_raw()?;
                raw.append(&mut r);
            }
            Self::Reserved(d) => raw.extend_from_slice(d),
        }
        Ok(raw)
    }
}

impl Default for BcoStreamFormat {
    fn default() -> Self {
        Self::Reserved(Default::default())
    }
}

/// The status to support the stream format.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BcoSupportStatus {
    /// The format is already set and stream is available.
    Active,
    /// The format is already set but stream is not available.
    Inactive,
    /// The format is not uset yet.
    NoStreamFormat,
    NotUsed,
}

impl Default for BcoSupportStatus {
    fn default() -> Self {
        Self::NotUsed
    }
}

impl BcoSupportStatus {
    const ACTIVE: u8 = 0x00;
    const INACTIVE: u8 = 0x01;
    const NO_STREAM_FORMAT: u8 = 0x02;
    const NOT_USED: u8 = 0xff;

    fn from_val(val: u8) -> Result<Self, AvcRespParseError> {
        let status = match val {
            Self::ACTIVE => Self::Active,
            Self::INACTIVE => Self::Inactive,
            Self::NO_STREAM_FORMAT => Self::NoStreamFormat,
            Self::NOT_USED => Self::NotUsed,
            _ => Err(AvcRespParseError::UnexpectedOperands(0))?,
        };
        Ok(status)
    }

    fn to_val(&self) -> u8 {
        match self {
            Self::Active => Self::ACTIVE,
            Self::Inactive => Self::INACTIVE,
            Self::NoStreamFormat => Self::NO_STREAM_FORMAT,
            Self::NotUsed => Self::NOT_USED,
        }
    }
}

/// AV/C command for extension of stream format.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct BcoExtendedStreamFormat {
    subfunc: u8,
    plug_addr: BcoPlugAddr,
    support_status: BcoSupportStatus,
}

impl BcoExtendedStreamFormat {
    const OPCODE: u8 = 0x2f;

    const LENGTH_MIN: usize = 7;

    fn new(subfunc: u8, plug_addr: &BcoPlugAddr) -> Self {
        Self {
            subfunc,
            plug_addr: *plug_addr,
            ..Default::default()
        }
    }
}

impl Default for BcoExtendedStreamFormat {
    fn default() -> Self {
        Self {
            subfunc: 0xff,
            plug_addr: Default::default(),
            support_status: Default::default(),
        }
    }
}

impl AvcStatus for BcoExtendedStreamFormat {
    fn build_operands(&mut self, _: &AvcAddr) -> Result<Vec<u8>, AvcCmdBuildError> {
        let mut operands = Vec::new();
        operands.push(self.subfunc);
        operands.extend_from_slice(&self.plug_addr.to_raw());
        operands.push(self.support_status.to_val());
        Ok(operands)
    }

    fn parse_operands(&mut self, _: &AvcAddr, operands: &[u8]) -> Result<(), AvcRespParseError> {
        if operands.len() < Self::LENGTH_MIN {
            Err(AvcRespParseError::TooShortResp(Self::LENGTH_MIN))?;
        }

        if operands[0] != self.subfunc {
            Err(AvcRespParseError::UnexpectedOperands(0))?;
        }

        let plug_addr = BcoPlugAddr::from_raw(&operands[1..]).map_err(|err| err.add_offset(1))?;
        if plug_addr != self.plug_addr {
            Err(AvcRespParseError::UnexpectedOperands(1))?;
        }

        self.support_status = BcoSupportStatus::from_val(operands[6])?;

        Ok(())
    }
}

/// AV/C command for single subfunction of extension of stream format.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtendedStreamFormatSingle {
    /// The status to support the stream format.
    pub support_status: BcoSupportStatus,
    /// The stream format.
    pub stream_format: BcoStreamFormat,
    op: BcoExtendedStreamFormat,
}

impl ExtendedStreamFormatSingle {
    const SUBFUNC: u8 = 0xc0;

    pub fn new(plug_addr: &BcoPlugAddr) -> Self {
        Self {
            op: BcoExtendedStreamFormat::new(Self::SUBFUNC, plug_addr),
            ..Default::default()
        }
    }
}

impl Default for ExtendedStreamFormatSingle {
    fn default() -> Self {
        Self {
            support_status: Default::default(),
            stream_format: Default::default(),
            op: BcoExtendedStreamFormat::new(Self::SUBFUNC, &BcoPlugAddr::default()),
        }
    }
}

impl AvcOp for ExtendedStreamFormatSingle {
    const OPCODE: u8 = BcoExtendedStreamFormat::OPCODE;
}

impl AvcStatus for ExtendedStreamFormatSingle {
    fn build_operands(&mut self, addr: &AvcAddr) -> Result<Vec<u8>, AvcCmdBuildError> {
        self.op.support_status = BcoSupportStatus::NotUsed;
        self.op.build_operands(addr)
    }

    fn parse_operands(&mut self, addr: &AvcAddr, operands: &[u8]) -> Result<(), AvcRespParseError> {
        self.op.parse_operands(addr, operands)?;

        self.support_status = self.op.support_status;
        self.stream_format =
            BcoStreamFormat::from_raw(&operands[7..]).map_err(|err| err.add_offset(7))?;

        Ok(())
    }
}

impl AvcControl for ExtendedStreamFormatSingle {
    fn build_operands(&mut self, addr: &AvcAddr) -> Result<Vec<u8>, AvcCmdBuildError> {
        self.op.support_status = BcoSupportStatus::Active;
        self.op.build_operands(addr).and_then(|mut operands| {
            self.stream_format.to_raw().map(|mut raw| {
                operands.append(&mut raw);
                operands
            })
        })
    }

    fn parse_operands(&mut self, addr: &AvcAddr, operands: &[u8]) -> Result<(), AvcRespParseError> {
        self.op.parse_operands(addr, operands)?;

        self.support_status = self.op.support_status;
        self.stream_format =
            BcoStreamFormat::from_raw(&operands[7..]).map_err(|err| err.add_offset(7))?;

        Ok(())
    }
}

/// AV/C command for list subfunction of extension of stream format.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtendedStreamFormatList {
    /// The status to support the stream format.
    pub support_status: BcoSupportStatus,
    /// The index of stream format.
    pub index: u8,
    /// The stream format.
    pub stream_format: BcoStreamFormat,
    op: BcoExtendedStreamFormat,
}

impl ExtendedStreamFormatList {
    const SUBFUNC: u8 = 0xc1;

    /// Instantiate extended stream format list structure with parameters.
    pub fn new(plug_addr: &BcoPlugAddr, index: u8) -> Self {
        Self {
            index,
            op: BcoExtendedStreamFormat::new(Self::SUBFUNC, plug_addr),
            ..Default::default()
        }
    }
}

impl Default for ExtendedStreamFormatList {
    fn default() -> Self {
        Self {
            support_status: Default::default(),
            index: 0xff,
            stream_format: Default::default(),
            op: BcoExtendedStreamFormat::new(Self::SUBFUNC, &BcoPlugAddr::default()),
        }
    }
}

impl AvcOp for ExtendedStreamFormatList {
    const OPCODE: u8 = BcoExtendedStreamFormat::OPCODE;
}

impl AvcStatus for ExtendedStreamFormatList {
    fn build_operands(&mut self, addr: &AvcAddr) -> Result<Vec<u8>, AvcCmdBuildError> {
        self.op.build_operands(addr).map(|mut operands| {
            operands.push(self.index);
            operands
        })
    }

    fn parse_operands(&mut self, addr: &AvcAddr, operands: &[u8]) -> Result<(), AvcRespParseError> {
        self.op.parse_operands(addr, operands)?;

        self.support_status = self.op.support_status;

        if operands[7] != self.index {
            Err(AvcRespParseError::UnexpectedOperands(7))?;
        }

        self.stream_format =
            BcoStreamFormat::from_raw(&operands[8..]).map_err(|err| err.add_offset(8))?;

        Ok(())
    }
}

/// Information about firmware image.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BcoImageInformation {
    pub timestamp: glib::DateTime,
    pub id: u32,
    pub version: u32,
}

impl Default for BcoImageInformation {
    fn default() -> Self {
        Self {
            timestamp: glib::DateTime::now_utc().unwrap(),
            id: Default::default(),
            version: Default::default(),
        }
    }
}

/// Information about boot loader.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BcoBootloaderInformation {
    /// The version of supported protocol.
    pub protocol_version: u32,
    /// The version of bootloader.
    pub bootloader_version: u32,
    /// GUID of the device.
    pub guid: u64,
    /// The numeric model identifier of the device.
    pub hardware_model_id: u32,
    /// The revision of the device.
    pub hardware_revision: u32,
    /// The information about installed software.
    pub software: BcoImageInformation,
    /// The base address of software image.
    pub image_base_address: usize,
    /// The maximum size of software image.
    pub image_maximum_size: usize,
    /// The time stamp of bootloader.
    pub bootloader_timestamp: glib::DateTime,
    /// The information about debugger.
    pub debugger: Option<BcoImageInformation>,
}

impl Default for BcoBootloaderInformation {
    fn default() -> Self {
        Self {
            protocol_version: Default::default(),
            bootloader_version: Default::default(),
            guid: Default::default(),
            hardware_model_id: Default::default(),
            hardware_revision: Default::default(),
            software: Default::default(),
            image_base_address: Default::default(),
            image_maximum_size: Default::default(),
            bootloader_timestamp: glib::DateTime::now_utc().unwrap(),
            debugger: Default::default(),
        }
    }
}

impl BcoBootloaderInformation {
    const SIZE_WITHOUT_DEBUGGER: usize = 80;
    const SIZE_WITH_DEBUGGER: usize = Self::SIZE_WITHOUT_DEBUGGER + 24;

    const INFO_MAGIC_BYTES: [u8; 8] = [
        0x62, // 'b'
        0x72, // 'r'
        0x69, // 'i'
        0x64, // 'd'
        0x67, // 'g'
        0x65, // 'e'
        0x43, // 'C'
        0x6f, // 'o'
    ];

    fn deserialize(&mut self, raw: &[u8]) -> Result<(), Error> {
        if &raw[..8] != &Self::INFO_MAGIC_BYTES {
            let msg = format!("Unexpected magic bytes: {:?}", &raw[..8]);
            Err(Error::new(FileError::Nxio, &msg))?;
        }

        let mut quadlet = [0; 4];
        quadlet.copy_from_slice(&raw[8..12]);
        self.protocol_version = u32::from_le_bytes(quadlet);

        quadlet.copy_from_slice(&raw[12..16]);
        self.bootloader_version = u32::from_le_bytes(quadlet);

        let mut octlet = [0; 8];
        octlet.copy_from_slice(&raw[16..24]);
        self.guid = u64::from_le_bytes(octlet);

        quadlet.copy_from_slice(&raw[24..28]);
        self.hardware_model_id = u32::from_le_bytes(quadlet);

        quadlet.copy_from_slice(&raw[28..32]);
        self.hardware_revision = u32::from_le_bytes(quadlet);

        self.software.timestamp = parse_tstamp(&raw[32..48])?;

        quadlet.copy_from_slice(&raw[48..52]);
        self.software.id = u32::from_le_bytes(quadlet);

        quadlet.copy_from_slice(&raw[52..56]);
        self.software.version = u32::from_le_bytes(quadlet);

        quadlet.copy_from_slice(&raw[56..60]);
        self.image_base_address = u32::from_le_bytes(quadlet) as usize;

        quadlet.copy_from_slice(&raw[60..64]);
        self.image_maximum_size = u32::from_le_bytes(quadlet) as usize;

        self.bootloader_timestamp = parse_tstamp(&raw[64..80])?;

        if raw.len() >= Self::SIZE_WITH_DEBUGGER && &raw[80..96] != &[0; 16] {
            let mut debugger = BcoImageInformation::default();

            debugger.timestamp = parse_tstamp(&raw[80..96])?;

            quadlet.copy_from_slice(&raw[96..100]);
            debugger.id = u32::from_le_bytes(quadlet);

            quadlet.copy_from_slice(&raw[100..104]);
            debugger.version = u32::from_le_bytes(quadlet);

            self.debugger = Some(debugger);
        }

        Ok(())
    }
}

/// The operation of bootloader in BridgeCo ASICs.
pub trait BcoBootloaderOperation {
    fn read_info(
        req: &FwReq,
        node: &FwNode,
        info: &mut BcoBootloaderInformation,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut raw = vec![0; BcoBootloaderInformation::SIZE_WITH_DEBUGGER];
        req.transaction_sync(
            node,
            FwTcode::ReadBlockRequest,
            DM_BCO_BOOTLOADER_INFO_OFFSET,
            raw.len(),
            &mut raw,
            timeout_ms,
        )
        .or_else(|_| {
            raw = vec![0; BcoBootloaderInformation::SIZE_WITHOUT_DEBUGGER];
            req.transaction_sync(
                node,
                FwTcode::ReadBlockRequest,
                DM_BCO_BOOTLOADER_INFO_OFFSET,
                raw.len(),
                &mut raw,
                timeout_ms,
            )
        })
        .and_then(|_| info.deserialize(&raw))
    }
}

fn parse_tstamp(buf: &[u8]) -> Result<glib::DateTime, Error> {
    assert_eq!(buf.len(), 16);

    let literal = std::str::from_utf8(&buf[..8]).map_err(|err| {
        let msg = format!("{}", err);
        Error::new(FileError::Nxio, &msg)
    })?;

    let year = u32::from_str_radix(&literal[..4], 10).map_err(|err| {
        let msg = format!("{}", err);
        Error::new(FileError::Nxio, &msg)
    })?;

    let month = u16::from_str_radix(&literal[4..6], 10).map_err(|err| {
        let msg = format!("{}", err);
        Error::new(FileError::Nxio, &msg)
    })?;

    let day = u16::from_str_radix(&literal[6..8], 10).map_err(|err| {
        let msg = format!("{}", err);
        Error::new(FileError::Nxio, &msg)
    })?;

    let (hour, minute, seconds) = if &buf[8..16] != &[0; 8] {
        let literal = std::str::from_utf8(&buf[8..16]).map_err(|err| {
            let msg = format!("{}", err);
            Error::new(FileError::Nxio, &msg)
        })?;

        let hour = u16::from_str_radix(&literal[..2], 10).map_err(|err| {
            let msg = format!("{}", err);
            Error::new(FileError::Nxio, &msg)
        })?;

        let minute = u16::from_str_radix(&literal[2..4], 10).map_err(|err| {
            let msg = format!("{}", err);
            Error::new(FileError::Nxio, &msg)
        })?;

        u16::from_str_radix(&literal[4..6], 10)
            .map_err(|err| {
                let msg = format!("{}", err);
                Error::new(FileError::Nxio, &msg)
            })
            .map(|seconds| (hour, minute, seconds))
    } else {
        Ok((0, 0, 0))
    }?;

    let tstamp = glib::DateTime::from_utc(
        year as i32,
        month as i32,
        day as i32,
        hour as i32,
        minute as i32,
        seconds as f64,
    )
    .unwrap();

    Ok(tstamp)
}

#[cfg(test)]
mod test {
    use super::BcoChannelName;
    use super::BcoPlugInfo;
    use super::BcoPlugType;
    use super::ExtendedPlugInfo;
    use super::ExtendedSubunitInfo;
    use super::{BcoChannelInfo, BcoLocation};
    use super::{BcoCluster, BcoClusterInfo, BcoPortType};
    use super::{BcoIoPlugAddr, BcoIoPlugAddrMode};
    use super::{BcoPlugAddr, BcoPlugAddrMode, BcoPlugDirection};
    use super::{BcoPlugAddrFuncBlk, BcoPlugAddrSubunit, BcoPlugAddrUnit, BcoPlugAddrUnitType};
    use ta1394_avc_general::*;

    #[test]
    fn bcoplugaddr_from() {
        // Input plug for Unit.
        let raw = [0x00, 0x00, 0x00, 0x03, 0xff];
        let addr = BcoPlugAddr::from_raw(&raw).unwrap();
        assert_eq!(addr.direction, BcoPlugDirection::Input);
        if let BcoPlugAddrMode::Unit(d) = &addr.mode {
            assert_eq!(d.plug_type, BcoPlugAddrUnitType::Isoc);
            assert_eq!(d.plug_id, 0x03);
        } else {
            unreachable!();
        }
        assert_eq!(raw, addr.to_raw());

        // Output plug for Subunit.
        let raw = [0x01, 0x01, 0x04, 0xff, 0xff];
        let addr = BcoPlugAddr::from_raw(&raw).unwrap();
        assert_eq!(addr.direction, BcoPlugDirection::Output);
        if let BcoPlugAddrMode::Subunit(d) = &addr.mode {
            assert_eq!(d.plug_id, 0x04);
        } else {
            unreachable!();
        }
        assert_eq!(raw, addr.to_raw());

        // Input plug for function block.
        let raw = [0x01, 0x02, 0x06, 0x03, 0x02];
        let addr = BcoPlugAddr::from_raw(&raw).unwrap();
        assert_eq!(addr.direction, BcoPlugDirection::Output);
        if let BcoPlugAddrMode::FuncBlk(d) = &addr.mode {
            assert_eq!(d.func_blk_type, 0x06);
            assert_eq!(d.func_blk_id, 0x03);
            assert_eq!(d.plug_id, 0x02);
        } else {
            unreachable!();
        }
        assert_eq!(raw, addr.to_raw());
    }

    #[test]
    fn bcochannelinfo_from() {
        let raw = [0x02, 0x0d];
        let info = BcoChannelInfo::from_raw(&raw).unwrap();
        assert_eq!(raw, info.to_raw());

        let raw = [0x3e, 0x0c];
        let info = BcoChannelInfo::from_raw(&raw).unwrap();
        assert_eq!(raw, info.to_raw());
    }

    #[test]
    fn bcocluster_from() {
        let raw: Vec<u8> = vec![0x03, 0x03, 0x0b, 0x09, 0x03, 0x2c, 0x01];
        let cluster = BcoCluster::from_raw(&raw).unwrap();
        assert_eq!(raw, cluster.to_raw());

        let raw: Vec<u8> = vec![
            0x05, 0x03, 0x0b, 0x09, 0x03, 0x2c, 0x01, 0x02, 0x0d, 0x3e, 0x0c,
        ];
        let cluster = BcoCluster::from_raw(&raw).unwrap();
        assert_eq!(raw, cluster.to_raw());
    }

    #[test]
    fn bcoclusterinfo_from() {
        let raw: Vec<u8> = vec![0x03, 0x0a, 0x03, 0x4c, 0x51, 0x33];
        let info = BcoClusterInfo::from_raw(&raw).unwrap();
        assert_eq!(raw, info.to_raw(),);
    }

    #[test]
    fn bcoioplugaddr_from() {
        // Unit.
        let raw = [0x00, 0x00, 0x02, 0x05, 0xff, 0xff, 0xff];
        let addr = BcoIoPlugAddr::from_raw(&raw).unwrap();
        assert_eq!(addr.direction, BcoPlugDirection::Input);
        if let BcoIoPlugAddrMode::Unit(d) = &addr.mode {
            assert_eq!(d.plug_type, BcoPlugAddrUnitType::Async);
            assert_eq!(d.plug_id, 0x05);
        } else {
            unreachable!();
        }
        assert_eq!(raw, addr.to_raw());

        // Subunit.
        let raw = [0x01, 0x01, 0x06, 0x05, 0x02, 0xff, 0xff];
        let addr = BcoIoPlugAddr::from_raw(&raw).unwrap();
        assert_eq!(addr.direction, BcoPlugDirection::Output);
        if let BcoIoPlugAddrMode::Subunit(s, d) = &addr.mode {
            assert_eq!(s.subunit_type, AvcSubunitType::Ca);
            assert_eq!(s.subunit_id, 0x05);
            assert_eq!(d.plug_id, 0x02);
        } else {
            unreachable!();
        }
        assert_eq!(raw, addr.to_raw());

        // Function block.
        let raw = [0x00, 0x02, 0x04, 0x09, 0x80, 0x12, 0x23];
        let addr = BcoIoPlugAddr::from_raw(&raw).unwrap();
        assert_eq!(addr.direction, BcoPlugDirection::Input);
        if let BcoIoPlugAddrMode::FuncBlk(s, d) = &addr.mode {
            assert_eq!(s.subunit_type, AvcSubunitType::Tape);
            assert_eq!(s.subunit_id, 0x09);
            assert_eq!(d.func_blk_type, 0x80);
            assert_eq!(d.func_blk_id, 0x12);
            assert_eq!(d.plug_id, 0x23);
        } else {
            unreachable!();
        }
        assert_eq!(raw, addr.to_raw());
    }

    #[test]
    fn bcopluginfo_type_from() {
        let raw = vec![0x00, 0x03];
        let info = BcoPlugInfo::from_raw(&raw).unwrap();
        if let BcoPlugInfo::Type(t) = &info {
            assert_eq!(*t, BcoPlugType::Sync);
        } else {
            unreachable!();
        }
        assert_eq!(raw, info.to_raw());
    }

    #[test]
    fn bcopluginfo_name_from() {
        let raw = vec![0x01, 0x03, 0x31, 0x32, 0x33];
        let info = BcoPlugInfo::from_raw(&raw).unwrap();
        if let BcoPlugInfo::Name(n) = &info {
            assert_eq!(n, "123");
        } else {
            unreachable!();
        }
        assert_eq!(raw, info.to_raw());
    }

    #[test]
    fn bcopluginfo_chcount_from() {
        let raw = vec![0x02, 0xc3];
        let info = BcoPlugInfo::from_raw(&raw).unwrap();
        if let BcoPlugInfo::ChCount(c) = &info {
            assert_eq!(*c, 0xc3);
        } else {
            unreachable!();
        }
        assert_eq!(raw, info.to_raw());
    }

    #[test]
    fn bcopluginfo_chpositions_from() {
        let raw = vec![
            0x03, 0x04, 0x01, 0x00, 0x04, 0x02, 0x03, 0x08, 0x00, 0x09, 0x03, 0x04, 0x08, 0x06,
            0x08, 0x05, 0x07, 0x01, 0x09, 0xb,
        ];
        let info = BcoPlugInfo::from_raw(&raw).unwrap();
        if let BcoPlugInfo::ChPositions(clusters) = &info {
            assert_eq!(clusters.len(), 4);
            let m = &clusters[0];
            assert_eq!(m.entries.len(), 1);
            assert_eq!(
                m.entries[0],
                BcoChannelInfo {
                    pos: 0x00,
                    loc: BcoLocation::LowFrequencyEffect
                }
            );
            let m = &clusters[1];
            assert_eq!(m.entries.len(), 2);
            assert_eq!(
                m.entries[0],
                BcoChannelInfo {
                    pos: 0x03,
                    loc: BcoLocation::RightCenter
                }
            );
            assert_eq!(
                m.entries[1],
                BcoChannelInfo {
                    pos: 0x00,
                    loc: BcoLocation::Surround
                }
            );
            let m = &clusters[2];
            assert_eq!(m.entries.len(), 3);
            assert_eq!(
                m.entries[0],
                BcoChannelInfo {
                    pos: 0x04,
                    loc: BcoLocation::RightCenter
                }
            );
            assert_eq!(
                m.entries[1],
                BcoChannelInfo {
                    pos: 0x06,
                    loc: BcoLocation::RightCenter
                }
            );
            assert_eq!(
                m.entries[2],
                BcoChannelInfo {
                    pos: 0x05,
                    loc: BcoLocation::LeftCenter
                }
            );
            let m = &clusters[3];
            assert_eq!(m.entries.len(), 1);
            assert_eq!(
                m.entries[0],
                BcoChannelInfo {
                    pos: 0x09,
                    loc: BcoLocation::SideRight
                }
            );
        } else {
            unreachable!();
        }
        assert_eq!(raw, info.to_raw());
    }

    #[test]
    fn bcopluginfo_chname_from() {
        let raw = vec![0x04, 0x9a, 0x01, 0x39];
        let info = BcoPlugInfo::from_raw(&raw).unwrap();
        if let BcoPlugInfo::ChName(d) = &info {
            assert_eq!(d.ch, 0x9a);
            assert_eq!(d.name, "9");
        } else {
            unreachable!();
        }
        assert_eq!(raw, info.to_raw());
    }

    #[test]
    fn bcopluginfo_input_from() {
        let raw = vec![0x05, 0x01, 0x01, 0x0b, 0x07, 0x42, 0xff, 0xff];
        let info = BcoPlugInfo::from_raw(&raw).unwrap();
        if let BcoPlugInfo::Input(plug_addr) = &info {
            assert_eq!(plug_addr.direction, BcoPlugDirection::Output);
            if let BcoIoPlugAddrMode::Subunit(s, d) = &plug_addr.mode {
                assert_eq!(s.subunit_type, AvcSubunitType::CameraStorage);
                assert_eq!(s.subunit_id, 0x07);
                assert_eq!(d.plug_id, 0x42);
            } else {
                unreachable!();
            }
        } else {
            unreachable!();
        }
        assert_eq!(raw, info.to_raw());
    }

    #[test]
    fn bcopluginfo_outputs_from() {
        let raw = vec![
            0x06, 0x02, 0x01, 0x01, 0x0b, 0x07, 0x42, 0xff, 0xff, 0x01, 0x01, 0x0b, 0x07, 0x42,
            0xff, 0xff,
        ];
        let info = BcoPlugInfo::from_raw(&raw).unwrap();
        if let BcoPlugInfo::Outputs(plug_addrs) = &info {
            let plug_addr = plug_addrs[0];
            assert_eq!(plug_addr.direction, BcoPlugDirection::Output);
            if let BcoIoPlugAddrMode::Subunit(s, d) = &plug_addr.mode {
                assert_eq!(s.subunit_type, AvcSubunitType::CameraStorage);
                assert_eq!(s.subunit_id, 0x07);
                assert_eq!(d.plug_id, 0x42);
            } else {
                unreachable!();
            }
            let plug_addr = plug_addrs[1];
            assert_eq!(plug_addr.direction, BcoPlugDirection::Output);
            if let BcoIoPlugAddrMode::Subunit(s, d) = &plug_addr.mode {
                assert_eq!(s.subunit_type, AvcSubunitType::CameraStorage);
                assert_eq!(s.subunit_id, 0x07);
                assert_eq!(d.plug_id, 0x42);
            } else {
                unreachable!();
            }
        } else {
            unreachable!();
        }
        assert_eq!(raw, info.to_raw());
    }

    #[test]
    fn bcopluginfo_clusterinfo_from() {
        let raw = vec![0x07, 0x01, 0x09, 0x05, 0x41, 0x42, 0x43, 0x44, 0x45];
        let info = BcoPlugInfo::from_raw(&raw).unwrap();
        if let BcoPlugInfo::ClusterInfo(d) = &info {
            assert_eq!(d.index, 0x01);
            assert_eq!(d.port_type, BcoPortType::Digital);
            assert_eq!(d.name, "ABCDE");
        } else {
            unreachable!();
        }
        assert_eq!(raw, info.to_raw());
    }

    #[test]
    fn extendedpluginfo_type_operands() {
        let raw = vec![0xc0, 0x01, 0x00, 0x00, 0x03, 0xff, 0x00, 0x00];
        let addr = BcoPlugAddr {
            direction: BcoPlugDirection::Output,
            mode: BcoPlugAddrMode::Unit(BcoPlugAddrUnit {
                plug_type: BcoPlugAddrUnitType::Isoc,
                plug_id: 0x03,
            }),
        };
        let info = BcoPlugInfo::Type(BcoPlugType::Isoc);
        let mut op = ExtendedPlugInfo::new(&addr, info);
        AvcStatus::parse_operands(&mut op, &AvcAddr::Unit, &raw).unwrap();
        assert_eq!(op.addr, addr);
        if let BcoPlugInfo::Type(plug_type) = &op.info {
            assert_eq!(plug_type, &BcoPlugType::Isoc);
        } else {
            unreachable!();
        }
    }

    #[test]
    fn extendedpluginfo_name_operands() {
        let raw = vec![
            0xc0, 0x00, 0x01, 0x17, 0xff, 0xff, 0x01, 0x05, 0x39, 0x38, 0x52, 0x36, 0x35,
        ];
        let addr = BcoPlugAddr {
            direction: BcoPlugDirection::Input,
            mode: BcoPlugAddrMode::Subunit(BcoPlugAddrSubunit { plug_id: 0x17 }),
        };
        let info = BcoPlugInfo::Name("".to_string());
        let mut op = ExtendedPlugInfo::new(&addr, info);
        AvcStatus::parse_operands(&mut op, &AvcAddr::Unit, &raw).unwrap();
        assert_eq!(op.addr, addr);
        if let BcoPlugInfo::Name(n) = &op.info {
            assert_eq!(&n, &"98R65");
        } else {
            unreachable!();
        }
    }

    #[test]
    fn extendedpluginfo_chcount_operands() {
        let raw = vec![0xc0, 0x01, 0x02, 0x3e, 0x9a, 0x77, 0x02, 0xe4];
        let addr = BcoPlugAddr {
            direction: BcoPlugDirection::Output,
            mode: BcoPlugAddrMode::FuncBlk(BcoPlugAddrFuncBlk {
                func_blk_type: 0x3e,
                func_blk_id: 0x9a,
                plug_id: 0x77,
            }),
        };
        let info = BcoPlugInfo::ChCount(0xff);
        let mut op = ExtendedPlugInfo::new(&addr, info);
        AvcStatus::parse_operands(&mut op, &AvcAddr::Unit, &raw).unwrap();
        assert_eq!(op.addr, addr);
        if let BcoPlugInfo::ChCount(c) = &op.info {
            assert_eq!(*c, 0xe4);
        } else {
            unreachable!();
        }
    }

    #[test]
    fn extendedpluginfo_chpositions_operands() {
        let raw = vec![
            0xc0, 0x00, 0x00, 0x01, 0x5c, 0xff, 0x03, 0x03, 0x01, 0x00, 0x0a, 0x02, 0x03, 0x04,
            0x02, 0x07, 0x03, 0x01, 0x0f, 0x04, 0x01, 0x05, 0x03,
        ];
        let addr = BcoPlugAddr {
            direction: BcoPlugDirection::Input,
            mode: BcoPlugAddrMode::Unit(BcoPlugAddrUnit {
                plug_type: BcoPlugAddrUnitType::Ext,
                plug_id: 0x5c,
            }),
        };
        let info = BcoPlugInfo::ChPositions(Vec::new());
        let mut op = ExtendedPlugInfo::new(&addr, info);
        AvcStatus::parse_operands(&mut op, &AvcAddr::Unit, &raw).unwrap();
        assert_eq!(op.addr, addr);
        if let BcoPlugInfo::ChPositions(entries) = &op.info {
            assert_eq!(entries.len(), 3);
            let e = &entries[0].entries;
            assert_eq!(e.len(), 1);
            assert_eq!(
                e[0],
                BcoChannelInfo {
                    pos: 0x00,
                    loc: BcoLocation::SideLeft
                }
            );
            let e = &entries[1].entries;
            assert_eq!(e.len(), 2);
            assert_eq!(
                e[0],
                BcoChannelInfo {
                    pos: 0x03,
                    loc: BcoLocation::LowFrequencyEffect
                }
            );
            assert_eq!(
                e[1],
                BcoChannelInfo {
                    pos: 0x02,
                    loc: BcoLocation::LeftCenter
                }
            );
            let e = &entries[2].entries;
            assert_eq!(e.len(), 3);
            assert_eq!(
                e[0],
                BcoChannelInfo {
                    pos: 0x01,
                    loc: BcoLocation::RightFrontEffect
                }
            );
            assert_eq!(
                e[1],
                BcoChannelInfo {
                    pos: 0x04,
                    loc: BcoLocation::LeftFront
                }
            );
            assert_eq!(
                e[2],
                BcoChannelInfo {
                    pos: 0x05,
                    loc: BcoLocation::Center
                }
            );
        } else {
            unreachable!();
        }
    }

    #[test]
    fn extendedpluginfo_chname_operands() {
        let raw = vec![
            0xc0, 0x00, 0x00, 0x02, 0x97, 0xff, 0x04, 0x9d, 0x02, 0x46, 0x54,
        ];
        let addr = BcoPlugAddr {
            direction: BcoPlugDirection::Input,
            mode: BcoPlugAddrMode::Unit(BcoPlugAddrUnit {
                plug_type: BcoPlugAddrUnitType::Async,
                plug_id: 0x97,
            }),
        };
        let info = BcoPlugInfo::ChName(BcoChannelName {
            ch: 0x9d,
            name: "".to_string(),
        });
        let mut op = ExtendedPlugInfo::new(&addr, info);
        AvcStatus::parse_operands(&mut op, &AvcAddr::Unit, &raw).unwrap();
        assert_eq!(op.addr, addr);
        if let BcoPlugInfo::ChName(d) = &op.info {
            assert_eq!(d.ch, 0x9d);
            assert_eq!(&d.name, &"FT");
        } else {
            unreachable!();
        }
    }

    #[test]
    fn extendedpluginfo_input_operands() {
        let raw = vec![
            0xc0, 0x01, 0x01, 0x5d, 0xff, 0xff, 0x05, 0x00, 0x02, 0x0c, 0x12, 0x80, 0xd9, 0x04,
        ];
        let addr = BcoPlugAddr {
            direction: BcoPlugDirection::Output,
            mode: BcoPlugAddrMode::Subunit(BcoPlugAddrSubunit { plug_id: 0x5d }),
        };
        let info = BcoPlugInfo::Input(BcoIoPlugAddr {
            direction: BcoPlugDirection::Input,
            mode: BcoIoPlugAddrMode::Unit(BcoPlugAddrUnit {
                plug_type: BcoPlugAddrUnitType::Isoc,
                plug_id: 0xff,
            }),
        });
        let mut op = ExtendedPlugInfo::new(&addr, info);
        AvcStatus::parse_operands(&mut op, &AvcAddr::Unit, &raw).unwrap();
        assert_eq!(op.addr, addr);
        if let BcoPlugInfo::Input(plug_addr) = op.info {
            assert_eq!(plug_addr.direction, BcoPlugDirection::Input);
            if let BcoIoPlugAddrMode::FuncBlk(s, d) = &plug_addr.mode {
                assert_eq!(s.subunit_type, AvcSubunitType::Music);
                assert_eq!(s.subunit_id, 0x12);
                assert_eq!(d.func_blk_type, 0x80);
                assert_eq!(d.func_blk_id, 0xd9);
                assert_eq!(d.plug_id, 0x04);
            } else {
                unreachable!();
            }
        } else {
            unreachable!();
        }
    }

    #[test]
    fn extendedpluginfo_outputs_operands() {
        let raw = vec![
            0xc0, 0x01, 0x00, 0x00, 0x11, 0xff, 0x06, 0x02, 0x00, 0x02, 0x0c, 0x12, 0x80, 0xd9,
            0x04, 0x00, 0x01, 0x0c, 0x03, 0x31, 0xff, 0xff,
        ];
        let addr = BcoPlugAddr {
            direction: BcoPlugDirection::Output,
            mode: BcoPlugAddrMode::Unit(BcoPlugAddrUnit {
                plug_type: BcoPlugAddrUnitType::Isoc,
                plug_id: 0x11,
            }),
        };
        let mut op = ExtendedPlugInfo::new(&addr, BcoPlugInfo::Outputs(Vec::new()));
        AvcStatus::parse_operands(&mut op, &AvcAddr::Unit, &raw).unwrap();
        assert_eq!(op.addr, addr);
        if let BcoPlugInfo::Outputs(plug_addrs) = op.info {
            assert_eq!(plug_addrs.len(), 2);

            assert_eq!(plug_addrs[0].direction, BcoPlugDirection::Input);
            if let BcoIoPlugAddrMode::FuncBlk(s, d) = &plug_addrs[0].mode {
                assert_eq!(s.subunit_type, AvcSubunitType::Music);
                assert_eq!(s.subunit_id, 0x12);
                assert_eq!(d.func_blk_type, 0x80);
                assert_eq!(d.func_blk_id, 0xd9);
                assert_eq!(d.plug_id, 0x04);
            } else {
                unreachable!();
            }

            assert_eq!(plug_addrs[1].direction, BcoPlugDirection::Input);
            if let BcoIoPlugAddrMode::Subunit(s, d) = &plug_addrs[1].mode {
                assert_eq!(s.subunit_type, AvcSubunitType::Music);
                assert_eq!(s.subunit_id, 0x03);
                assert_eq!(d.plug_id, 0x31);
            } else {
                unreachable!();
            }
        } else {
            unreachable!();
        }
    }

    #[test]
    fn extendedpluginfo_clusterinfo_operands() {
        let raw = vec![
            0xc0, 0x01, 0x00, 0x01, 0x1e, 0xff, 0x07, 0x02, 0x05, 0x03, 0x60, 0x50, 0x70,
        ];
        let addr = BcoPlugAddr {
            direction: BcoPlugDirection::Output,
            mode: BcoPlugAddrMode::Unit(BcoPlugAddrUnit {
                plug_type: BcoPlugAddrUnitType::Ext,
                plug_id: 0x1e,
            }),
        };
        let info = BcoClusterInfo {
            index: 0x02,
            port_type: BcoPortType::NoType,
            name: "".to_string(),
        };
        let mut op = ExtendedPlugInfo::new(&addr, BcoPlugInfo::ClusterInfo(info));
        AvcStatus::parse_operands(&mut op, &AvcAddr::Unit, &raw).unwrap();
        assert_eq!(op.addr, addr);
        if let BcoPlugInfo::ClusterInfo(i) = &op.info {
            assert_eq!(i.index, 0x02);
            assert_eq!(i.port_type, BcoPortType::Adat);
            assert_eq!(&i.name, &"`Pp");
        } else {
            unreachable!();
        }
    }

    #[test]
    fn extendedsubunitinfo_operands() {
        let operands = [
            0x00, 0xff, 0x81, 0x70, 0xd0, 0xe0, 0x03, 0x82, 0x60, 0xe0, 0xe0, 0x04, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        ];
        let mut op = ExtendedSubunitInfo::new(0, 0xff);
        AvcStatus::parse_operands(&mut op, &AvcAddr::Unit, &operands).unwrap();
        assert_eq!(op.page, 0x00);
        assert_eq!(op.func_blk_type, 0xff);
        assert_eq!(op.entries.len(), 2);
        let e = op.entries[0];
        assert_eq!(e.func_blk_type, 0x81);
        assert_eq!(e.func_blk_id, 0x70);
        assert_eq!(e.func_blk_purpose, 0xd0);
        assert_eq!(e.input_plugs, 0xe0);
        assert_eq!(e.output_plugs, 0x03);
        let e = op.entries[1];
        assert_eq!(e.func_blk_type, 0x82);
        assert_eq!(e.func_blk_id, 0x60);
        assert_eq!(e.func_blk_purpose, 0xe0);
        assert_eq!(e.input_plugs, 0xe0);
        assert_eq!(e.output_plugs, 0x04);
        let operands = AvcStatus::build_operands(&mut op, &AvcAddr::Unit).unwrap();
        assert_eq!(&operands[..2], &[0x00, 0xff]);
        assert_eq!(&operands[2..], &[0xff; 25]);
    }
}
