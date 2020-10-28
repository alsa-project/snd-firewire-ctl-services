// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::{Error, FileError};
use crate::ta1394::{AvcAddr, AvcAddrSubunit, AvcSubunitType, Ta1394AvcError};
use crate::ta1394::{AvcOp, AvcStatus, AvcControl};
use crate::ta1394::general::{PlugInfo, SubunitInfo};
use crate::ta1394::stream_format::{StreamFormat, AmStream, SupportStatus};

//
// Bco Extended Plug Info command
//
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum BcoPlugAddrUnitType {
    Isoc,
    Ext,
    Async,
    Reserved(u8),
}

impl BcoPlugAddrUnitType {
    const ISOC: u8 = 0x00;
    const EXT: u8 = 0x01;
    const ASYNC: u8 = 0x02;
}

impl From<u8> for BcoPlugAddrUnitType {
    fn from(val: u8) -> Self {
        match val {
            BcoPlugAddrUnitType::ISOC => BcoPlugAddrUnitType::Isoc,
            BcoPlugAddrUnitType::EXT => BcoPlugAddrUnitType::Ext,
            BcoPlugAddrUnitType::ASYNC => BcoPlugAddrUnitType::Async,
            _ => BcoPlugAddrUnitType::Reserved(val),
        }
    }
}

impl From<BcoPlugAddrUnitType> for u8 {
    fn from(unit_type: BcoPlugAddrUnitType) -> u8 {
        match unit_type {
            BcoPlugAddrUnitType::Isoc => BcoPlugAddrUnitType::ISOC,
            BcoPlugAddrUnitType::Ext => BcoPlugAddrUnitType::EXT,
            BcoPlugAddrUnitType::Async => BcoPlugAddrUnitType::ASYNC,
            BcoPlugAddrUnitType::Reserved(val) => val,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct BcoPlugAddrUnit{
    pub plug_type: BcoPlugAddrUnitType,
    pub plug_id: u8,
}

impl From<&[u8;3]> for BcoPlugAddrUnit {
    fn from(raw: &[u8;3]) -> Self {
        BcoPlugAddrUnit{
            plug_type: BcoPlugAddrUnitType::from(raw[0]),
            plug_id: raw[1],
        }
    }
}

impl From<&BcoPlugAddrUnit> for [u8;3] {
    fn from(data: &BcoPlugAddrUnit) -> Self {
        [data.plug_type.into(), data.plug_id.into(), 0xff]
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct BcoPlugAddrSubunit{
    pub plug_id: u8,
}

impl From<&[u8;3]> for BcoPlugAddrSubunit {
    fn from(raw: &[u8;3]) -> Self {
        BcoPlugAddrSubunit{
            plug_id: raw[0],
        }
    }
}

impl From<&BcoPlugAddrSubunit> for [u8;3] {
    fn from(data: &BcoPlugAddrSubunit) -> Self {
        [data.plug_id, 0xff, 0xff]
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct BcoPlugAddrFuncBlk{
    pub func_blk_type: u8,
    pub func_blk_id: u8,
    pub plug_id: u8,
}

impl From<&[u8;3]> for BcoPlugAddrFuncBlk {
    fn from(raw: &[u8;3]) -> Self {
        BcoPlugAddrFuncBlk{
            func_blk_type: raw[0],
            func_blk_id: raw[1],
            plug_id: raw[2],
        }
    }
}

impl From<&BcoPlugAddrFuncBlk> for [u8;3] {
    fn from(data: &BcoPlugAddrFuncBlk) -> Self {
        [data.func_blk_type, data.func_blk_id, data.plug_id]
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum BcoPlugAddrMode {
    Unit(BcoPlugAddrUnit),
    Subunit(BcoPlugAddrSubunit),
    FuncBlk(BcoPlugAddrFuncBlk),
    Reserved([u8;4]),
}

impl BcoPlugAddrMode {
    const UNIT: u8 = 0x00;
    const SUBUNIT: u8 = 0x01;
    const FUNCBLK: u8 = 0x02;
}

impl From<&[u8;4]> for BcoPlugAddrMode {
    fn from(raw: &[u8;4]) -> Self {
        let mut r = [0;3];
        r.copy_from_slice(&raw[1..]);
        match raw[0] {
            BcoPlugAddrMode::UNIT => BcoPlugAddrMode::Unit(BcoPlugAddrUnit::from(&r)),
            BcoPlugAddrMode::SUBUNIT => BcoPlugAddrMode::Subunit(BcoPlugAddrSubunit::from(&r)),
            BcoPlugAddrMode::FUNCBLK => BcoPlugAddrMode::FuncBlk(BcoPlugAddrFuncBlk::from(&r)),
            _ => BcoPlugAddrMode::Reserved(*raw),
        }
    }
}

impl From<&BcoPlugAddrMode> for [u8;4] {
    fn from(data: &BcoPlugAddrMode) -> Self {
        let mut raw = [0;4];
        match data {
            BcoPlugAddrMode::Unit(d) => {
                raw[0] = BcoPlugAddrMode::UNIT;
                raw[1..].copy_from_slice(&Into::<[u8;3]>::into(d));
            }
            BcoPlugAddrMode::Subunit(d) => {
                raw[0] = BcoPlugAddrMode::SUBUNIT;
                raw[1..].copy_from_slice(&Into::<[u8;3]>::into(d));
            }
            BcoPlugAddrMode::FuncBlk(d) => {
                raw[0] = BcoPlugAddrMode::FUNCBLK;
                raw[1..].copy_from_slice(&Into::<[u8;3]>::into(d));
            }
            BcoPlugAddrMode::Reserved(d) => {
                raw.copy_from_slice(d);
            }
        }
        raw
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum BcoPlugDirection {
    Input,
    Output,
    Reserved(u8),
}

impl BcoPlugDirection {
    const INPUT: u8 = 0x00;
    const OUTPUT: u8 = 0x01;
}

impl From<u8> for BcoPlugDirection {
    fn from(val: u8) -> Self {
        match val {
            BcoPlugDirection::INPUT => BcoPlugDirection::Input,
            BcoPlugDirection::OUTPUT => BcoPlugDirection::Output,
            _ => BcoPlugDirection::Reserved(val),
        }
    }
}

impl From<BcoPlugDirection> for u8 {
    fn from(direction: BcoPlugDirection) -> u8 {
        match direction {
            BcoPlugDirection::Input => BcoPlugDirection::INPUT,
            BcoPlugDirection::Output => BcoPlugDirection::OUTPUT,
            BcoPlugDirection::Reserved(val) => val,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct BcoPlugAddr{
    pub direction: BcoPlugDirection,
    pub mode: BcoPlugAddrMode,
}

impl BcoPlugAddr {
    pub fn new_for_unit(direction: BcoPlugDirection, plug_type: BcoPlugAddrUnitType,
                        plug_id: u8) -> Self {
        BcoPlugAddr{
            direction,
            mode: BcoPlugAddrMode::Unit(BcoPlugAddrUnit{
                plug_type,
                plug_id,
            }),
        }
    }

    pub fn new_for_subunit(direction: BcoPlugDirection, plug_id: u8) -> Self {
        BcoPlugAddr{
            direction,
            mode: BcoPlugAddrMode::Subunit(BcoPlugAddrSubunit{
                plug_id,
            }),
        }
    }

    pub fn new_for_func_blk(direction: BcoPlugDirection, func_blk_type: u8,
                            func_blk_id: u8, plug_id: u8) -> Self {
        BcoPlugAddr{
            direction,
            mode: BcoPlugAddrMode::FuncBlk(BcoPlugAddrFuncBlk{
                func_blk_type,
                func_blk_id,
                plug_id,
            }),
        }
    }
}

impl From<&[u8;5]> for BcoPlugAddr {
    fn from(raw: &[u8;5]) -> Self {
        let mut r = [0;4];
        r.copy_from_slice(&raw[1..]);
        BcoPlugAddr{
            direction: BcoPlugDirection::from(raw[0]),
            mode: BcoPlugAddrMode::from(&r),
        }
    }
}

impl From<&BcoPlugAddr> for [u8;5] {
    fn from(data: &BcoPlugAddr) -> [u8;5] {
        let mut raw = [0;5];
        raw[0] = data.direction.into();
        raw[1..].copy_from_slice(&Into::<[u8;4]>::into(&data.mode));
        raw
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum BcoIoPlugAddrMode {
    Unit(BcoPlugAddrUnit),
    Subunit(AvcAddrSubunit, BcoPlugAddrSubunit),
    FuncBlk(AvcAddrSubunit, BcoPlugAddrFuncBlk),
    Reserved([u8;6]),
}

impl From<&[u8;6]> for BcoIoPlugAddrMode {
    fn from(raw: &[u8;6]) -> Self {
        let mut r = [0;3];
        match raw[0] {
            BcoPlugAddrMode::UNIT => {
                r.copy_from_slice(&raw[1..4]);
                BcoIoPlugAddrMode::Unit(BcoPlugAddrUnit::from(&r))
            }
            BcoPlugAddrMode::SUBUNIT => {
                let subunit = AvcAddrSubunit{
                    subunit_type: AvcSubunitType::from(raw[1]),
                    subunit_id: raw[2],
                };
                r.copy_from_slice(&raw[3..6]);
                BcoIoPlugAddrMode::Subunit(subunit, BcoPlugAddrSubunit::from(&r))
            }
            BcoPlugAddrMode::FUNCBLK => {
                let subunit = AvcAddrSubunit{
                    subunit_type: AvcSubunitType::from(raw[1]),
                    subunit_id: raw[2],
                };
                r.copy_from_slice(&raw[3..6]);
                BcoIoPlugAddrMode::FuncBlk(subunit, BcoPlugAddrFuncBlk::from(&r))
            }
            _ => BcoIoPlugAddrMode::Reserved(*raw),
        }
    }
}

impl From<&BcoIoPlugAddrMode> for [u8;6] {
    fn from(data: &BcoIoPlugAddrMode) -> Self {
        let mut raw = [0xff;6];
        match data {
            BcoIoPlugAddrMode::Unit(d) => {
                raw[0] = BcoPlugAddrMode::UNIT;
                raw[1..4].copy_from_slice(&Into::<[u8;3]>::into(d));
            }
            BcoIoPlugAddrMode::Subunit(s, d) => {
                raw[0] = BcoPlugAddrMode::SUBUNIT;
                raw[1] = s.subunit_type.into();
                raw[2] = s.subunit_id;
                raw[3..6].copy_from_slice(&Into::<[u8;3]>::into(d));
            }
            BcoIoPlugAddrMode::FuncBlk(s, d) => {
                raw[0] = BcoPlugAddrMode::FUNCBLK;
                raw[1] = s.subunit_type.into();
                raw[2] = s.subunit_id;
                raw[3..6].copy_from_slice(&Into::<[u8;3]>::into(d));
            }
            BcoIoPlugAddrMode::Reserved(d) => {
                raw.copy_from_slice(d);
            }
        }
        raw
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct BcoIoPlugAddr{
    pub direction: BcoPlugDirection,
    pub mode: BcoIoPlugAddrMode,
}

impl From<&[u8;7]> for BcoIoPlugAddr {
    fn from(raw: &[u8;7]) -> Self {
        let mut r = [0;6];
        r.copy_from_slice(&raw[1..]);
        BcoIoPlugAddr{
            direction: BcoPlugDirection::from(raw[0]),
            mode: BcoIoPlugAddrMode::from(&r),
        }
    }
}

impl From<&BcoIoPlugAddr> for [u8;7] {
    fn from(data: &BcoIoPlugAddr) -> [u8;7] {
        let mut raw = [0;7];
        raw[0] = data.direction.into();
        raw[1..].copy_from_slice(&Into::<[u8;6]>::into(&data.mode));
        raw
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum BcoPlugType {
    Isoc,
    Async,
    Midi,
    Sync,
    Analog,
    Digital,
    Reserved(u8),
}

impl BcoPlugType {
    const ISOC_STREAM: u8 = 0x00;
    const ASYNC_STREAM: u8 = 0x01;
    const MIDI: u8 = 0x02;
    const SYNC: u8 = 0x03;
    const ANALOG: u8 = 0x04;
    const DIGITAL: u8 = 0x05;
}

impl From<u8> for BcoPlugType {
    fn from(val: u8) -> Self {
        match val {
            BcoPlugType::ISOC_STREAM => BcoPlugType::Isoc,
            BcoPlugType::ASYNC_STREAM => BcoPlugType::Async,
            BcoPlugType::MIDI => BcoPlugType::Midi,
            BcoPlugType::SYNC => BcoPlugType::Sync,
            BcoPlugType::ANALOG => BcoPlugType::Analog,
            BcoPlugType::DIGITAL => BcoPlugType::Digital,
            _ => BcoPlugType::Reserved(val),
        }
    }
}

impl From<BcoPlugType> for u8 {
    fn from(plug_type: BcoPlugType) -> u8 {
        match plug_type {
            BcoPlugType::Isoc => BcoPlugType::ISOC_STREAM,
            BcoPlugType::Async => BcoPlugType::ASYNC_STREAM,
            BcoPlugType::Midi => BcoPlugType::MIDI,
            BcoPlugType::Sync => BcoPlugType::SYNC,
            BcoPlugType::Analog => BcoPlugType::ANALOG,
            BcoPlugType::Digital => BcoPlugType::DIGITAL,
            BcoPlugType::Reserved(val) => val,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum BcoLocation{
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
    Reserved(u8),
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
}

impl From<u8> for BcoLocation {
    fn from(val: u8) -> Self {
        match val {
            BcoLocation::L => BcoLocation::LeftFront,
            BcoLocation::R => BcoLocation::RightFront,
            BcoLocation::C => BcoLocation::Center,
            BcoLocation::LFE => BcoLocation::LowFrequencyEffect,
            BcoLocation::LS => BcoLocation::LeftSurround,
            BcoLocation::RS => BcoLocation::RightSurround,
            BcoLocation::LC => BcoLocation::LeftCenter,
            BcoLocation::RC => BcoLocation::RightCenter,
            BcoLocation::S => BcoLocation::Surround,
            BcoLocation::SL => BcoLocation::SideLeft,
            BcoLocation::SR => BcoLocation::SideRight,
            BcoLocation::T => BcoLocation::Top,
            BcoLocation::B => BcoLocation::Bottom,
            BcoLocation::FEL => BcoLocation::LeftFrontEffect,
            BcoLocation::FER => BcoLocation::RightFrontEffect,
            _ => BcoLocation::Reserved(val),
        }
    }
}

impl From<BcoLocation> for u8 {
    fn from(loc: BcoLocation) -> Self {
        match loc {
            BcoLocation::LeftFront => BcoLocation::L,
            BcoLocation::RightFront => BcoLocation::R,
            BcoLocation::Center => BcoLocation::C,
            BcoLocation::LowFrequencyEffect => BcoLocation::LFE,
            BcoLocation::LeftSurround => BcoLocation::LS,
            BcoLocation::RightSurround => BcoLocation::RS,
            BcoLocation::LeftCenter => BcoLocation::LC,
            BcoLocation::RightCenter => BcoLocation::RC,
            BcoLocation::Surround => BcoLocation::S,
            BcoLocation::SideLeft => BcoLocation::SL,
            BcoLocation::SideRight => BcoLocation::SR,
            BcoLocation::Top => BcoLocation::T,
            BcoLocation::Bottom => BcoLocation::B,
            BcoLocation::LeftFrontEffect => BcoLocation::FEL,
            BcoLocation::RightFrontEffect => BcoLocation::FER,
            BcoLocation::Reserved(val) => val,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct BcoChannelInfo{
    pos: u8,
    loc: BcoLocation,
}

impl From<&BcoChannelInfo> for [u8;2] {
    fn from(data: &BcoChannelInfo) -> Self {
        let mut raw = [0;2];
        raw[0] = data.pos;
        raw[1] = data.loc.into();
        raw
    }
}

impl From<&[u8;2]> for BcoChannelInfo {
    fn from(raw: &[u8;2]) -> Self {
        BcoChannelInfo{
            pos: raw[0],
            loc: BcoLocation::from(raw[1])
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct BcoCluster{
    entries: Vec<BcoChannelInfo>,
}

impl From<&[u8]> for BcoCluster{
    fn from(raw: &[u8]) -> Self {
        let count = raw[0] as usize;
        BcoCluster{
            entries: (0..count).map(|i| {
                let mut r = [0;2];
                let pos = 1 + i * 2;
                r.copy_from_slice(&raw[pos..(pos + 2)]);
                BcoChannelInfo::from(&r)
            }).collect(),
        }
    }
}

impl From<&BcoCluster> for Vec<u8> {
    fn from(data: &BcoCluster) -> Self {
        let mut raw = Vec::new();
        raw.push(data.entries.len() as u8);
        data.entries.iter().fold(raw, |mut raw, entry| {
            raw.extend_from_slice(&Into::<[u8;2]>::into(entry));
            raw
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct BcoChannelName{
    pub ch: u8,
    pub name: String,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
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
    Reserved(u8),
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
}

impl From<BcoPortType> for u8 {
    fn from(port_type: BcoPortType) -> Self {
        match port_type {
            BcoPortType::Speaker => BcoPortType::SPEAKER,
            BcoPortType::Headphone => BcoPortType::HEADPHONE,
            BcoPortType::Microphone => BcoPortType::MICROPHONE,
            BcoPortType::Line => BcoPortType::LINE,
            BcoPortType::Spdif => BcoPortType::SPDIF,
            BcoPortType::Adat => BcoPortType::ADAT,
            BcoPortType::Tdif => BcoPortType::TDIF,
            BcoPortType::Madi => BcoPortType::MADI,
            BcoPortType::Analog => BcoPortType::ANALOG,
            BcoPortType::Digital => BcoPortType::DIGITAL,
            BcoPortType::Midi => BcoPortType::MIDI,
            BcoPortType::Reserved(val) => val,
        }
    }
}

impl From<u8> for BcoPortType {
    fn from(val: u8) -> Self {
        match val {
            BcoPortType::SPEAKER => BcoPortType::Speaker,
            BcoPortType::HEADPHONE => BcoPortType::Headphone,
            BcoPortType::MICROPHONE => BcoPortType::Microphone,
            BcoPortType::LINE => BcoPortType::Line,
            BcoPortType::SPDIF => BcoPortType::Spdif,
            BcoPortType::ADAT => BcoPortType::Adat,
            BcoPortType::TDIF => BcoPortType::Tdif,
            BcoPortType::MADI => BcoPortType::Madi,
            BcoPortType::ANALOG => BcoPortType::Analog,
            BcoPortType::DIGITAL => BcoPortType::Digital,
            BcoPortType::MIDI => BcoPortType::Midi,
            _ => BcoPortType::Reserved(val),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct BcoClusterInfo{
    pub index: u8,
    pub port_type: BcoPortType,
    pub name: String,
}

impl From<&[u8]> for BcoClusterInfo {
    fn from(raw: &[u8]) -> Self {
        let pos = 3 + raw[2] as usize;
        let name = if pos > raw.len() {
            "".to_string()
        } else {
            String::from_utf8(raw[3..pos].to_vec()).unwrap_or("".to_string())
        };
        BcoClusterInfo{
            index: raw[0],
            port_type: BcoPortType::from(raw[1]),
            name,
        }
    }
}

impl From<&BcoClusterInfo> for Vec<u8> {
    fn from(data: &BcoClusterInfo) -> Vec<u8> {
        let mut raw = Vec::new();
        raw.push(data.index);
        raw.push(data.port_type.into());
        raw.push(data.name.len() as u8);
        raw.append(&mut data.name.clone().into_bytes());
        raw
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum BcoPlugInfo {
    Type(BcoPlugType),
    Name(String),
    ChCount(u8),
    ChPositions(Vec<BcoCluster>),
    ChName(BcoChannelName),
    Input(BcoIoPlugAddr),
    Outputs(Vec<BcoIoPlugAddr>),
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
}

impl From<&BcoPlugInfo> for Vec<u8> {
    fn from(data: &BcoPlugInfo) -> Self {
        let mut raw = Vec::new();
        match data {
            BcoPlugInfo::Type(plug_type) => {
                raw.push(BcoPlugInfo::TYPE);
                raw.push(u8::from(*plug_type));
            }
            BcoPlugInfo::Name(n) => {
                raw.push(BcoPlugInfo::NAME);
                raw.push(n.len() as u8);
                raw.append(&mut n.clone().into_bytes());
            }
            BcoPlugInfo::ChCount(c) => {
                raw.push(BcoPlugInfo::CH_COUNT);
                raw.push(*c);
            }
            BcoPlugInfo::ChPositions(entries) => {
                raw.push(BcoPlugInfo::CH_POSITIONS);
                raw.push(entries.len() as u8);
                entries.iter().for_each(|entry| raw.append(&mut entry.into()));
            }
            BcoPlugInfo::ChName(d) => {
                raw.push(BcoPlugInfo::CH_NAME);
                raw.push(d.ch);
                raw.push(d.name.len() as u8);
                raw.append(&mut d.name.clone().into_bytes());
            }
            BcoPlugInfo::Input(plug_addr) => {
                raw.push(BcoPlugInfo::INPUT);
                raw.extend_from_slice(&mut Into::<[u8;7]>::into(plug_addr));
            }
            BcoPlugInfo::Outputs(plug_addrs) => {
                raw.push(BcoPlugInfo::OUTPUTS);
                raw.push(plug_addrs.len() as u8);
                plug_addrs.iter().for_each(|plug_addr| raw.extend_from_slice(&Into::<[u8;7]>::into(plug_addr)));
            }
            BcoPlugInfo::ClusterInfo(d) => {
                raw.push(BcoPlugInfo::CLUSTER_INFO);
                raw.append(&mut d.into());
            }
            BcoPlugInfo::Reserved(d) => raw.extend_from_slice(&d),
        }
        raw
    }
}

impl From<&[u8]> for BcoPlugInfo{
    fn from(raw: &[u8]) -> Self {
        match raw[0] {
            BcoPlugInfo::TYPE => {
                BcoPlugInfo::Type(BcoPlugType::from(raw[1]))
            }
            BcoPlugInfo::NAME => {
                let pos = 2 + raw[1] as usize;
                let name = if pos > raw.len() {
                    "".to_string()
                } else {
                    String::from_utf8(raw[2..pos].to_vec()).unwrap_or("".to_string())
                };
                BcoPlugInfo::Name(name)
            }
            BcoPlugInfo::CH_COUNT => {
                BcoPlugInfo::ChCount(raw[1])
            }
            BcoPlugInfo::CH_POSITIONS => {
                let count = raw[1] as usize;
                let mut entries = Vec::with_capacity(count);
                let mut pos = 2;
                while pos < raw.len() && entries.len() < count {
                    let c = raw[pos] as usize;
                    let size = 1 + 2 * c;
                    entries.push(BcoCluster::from(&raw[pos..(pos + size)]));
                    pos += size;
                }
                BcoPlugInfo::ChPositions(entries)
            }
            BcoPlugInfo::CH_NAME => {
                let ch = raw[1] as u8;
                let pos = 3 + raw[2] as usize;
                let name = if pos > raw.len() {
                    "".to_string()
                } else {
                    String::from_utf8(raw[3..pos].to_vec()).unwrap_or("".to_string())
                };
                BcoPlugInfo::ChName(BcoChannelName{ch, name})
            }
            BcoPlugInfo::INPUT => {
                let mut r = [0;7];
                r.copy_from_slice(&raw[1..8]);
                BcoPlugInfo::Input(BcoIoPlugAddr::from(&r))
            }
            BcoPlugInfo::OUTPUTS => {
                let count = raw[1] as usize;
                let mut pos = 2;
                let mut entries = Vec::new();
                while pos < raw.len() && entries.len() < count {
                    let mut r = [0;7];
                    r.copy_from_slice(&raw[pos..(pos + 7)]);
                    entries.push(BcoIoPlugAddr::from(&r));
                    pos += 7;
                }
                BcoPlugInfo::Outputs(entries)
            }
            BcoPlugInfo::CLUSTER_INFO => {
                BcoPlugInfo::ClusterInfo(BcoClusterInfo::from(&raw[1..]))
            }
            _ => BcoPlugInfo::Reserved(raw.to_vec()),
        }
    }
}

pub struct ExtendedPlugInfo{
    pub addr: BcoPlugAddr,
    pub info: BcoPlugInfo,
}

impl ExtendedPlugInfo {
    const SUBFUNC: u8 = 0xc0;

    #[allow(dead_code)]
    pub fn new(addr: &BcoPlugAddr, info: BcoPlugInfo) -> Self {
        ExtendedPlugInfo{
            addr: *addr,
            info,
        }
    }
}

impl AvcOp for ExtendedPlugInfo {
    const OPCODE: u8 = PlugInfo::OPCODE;
}

impl AvcStatus for ExtendedPlugInfo {
    fn build_operands(&mut self, _: &AvcAddr, operands: &mut Vec<u8>) -> Result<(), Error> {
        operands.push(Self::SUBFUNC);
        operands.extend_from_slice(&Into::<[u8;5]>::into(&self.addr));
        operands.append(&mut Into::<Vec<u8>>::into(&self.info));
        Ok(())
    }

    fn parse_operands(&mut self, _: &AvcAddr, operands: &[u8]) -> Result<(), Error> {
        if operands.len() < 8 {
            let label = format!("Oprands too short for ExtendedPlugInfo; {}", operands.len());
            return Err(Error::new(Ta1394AvcError::TooShortResp, &label));
        }

        if operands[0] != Self::SUBFUNC {
            let label = format!("Unexpected subfunction for ExtendedPlugInfo; {}", operands[0]);
            return Err(Error::new(Ta1394AvcError::TooShortResp, &label));
        }

        let mut a = [0;5];
        a.copy_from_slice(&operands[1..6]);
        let addr = BcoPlugAddr::from(&a);
        if addr != self.addr {
            let label = format!("Unexpected address for ExtendedPlugInfo; {:?}", addr);
            return Err(Error::new(Ta1394AvcError::TooShortResp, &label));
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
            let label = format!("Unexpected type of information for ExtendedPlugInfo; {}", operands[6]);
            return Err(Error::new(Ta1394AvcError::TooShortResp, &label));
        }

        let info = BcoPlugInfo::from(&operands[6..]);
        if let BcoPlugInfo::Input(d) = &info {
            if let BcoPlugDirection::Reserved(val) = &d.direction {
                let label = format!("Unexpected value for direction of ExtendedPlugInfo: {}", val);
                return Err(Error::new(Ta1394AvcError::TooShortResp, &label));
            }
        }

        self.info = info;

        Ok(())
    }
}

//
// Bco Extended Subunit Info command
//
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ExtendedSubunitInfoEntry{
    pub func_blk_type: u8,
    pub func_blk_id: u8,
    pub func_blk_purpose: u8,
    pub input_plugs: u8,
    pub output_plugs: u8,
}

impl From<&[u8;5]> for ExtendedSubunitInfoEntry {
    fn from(raw: &[u8;5]) -> Self {
        ExtendedSubunitInfoEntry{
            func_blk_type: raw[0],
            func_blk_id: raw[1],
            func_blk_purpose: raw[2],
            input_plugs: raw[3],
            output_plugs: raw[4],
        }
    }
}

impl From<&ExtendedSubunitInfoEntry> for [u8;5] {
    fn from(data: &ExtendedSubunitInfoEntry) -> Self {
        [
            data.func_blk_type,
            data.func_blk_id,
            data.func_blk_purpose,
            data.input_plugs,
            data.output_plugs,
        ]
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ExtendedSubunitInfo{
    pub page: u8,
    pub func_blk_type: u8,
    pub entries: Vec<ExtendedSubunitInfoEntry>,
}

impl ExtendedSubunitInfo {
    #[allow(dead_code)]
    pub fn new(page: u8, func_blk_type: u8) -> Self {
        ExtendedSubunitInfo{
            page,
            func_blk_type,
            entries: Vec::new(),
        }
    }
}

impl AvcOp for ExtendedSubunitInfo {
    const OPCODE: u8 = SubunitInfo::OPCODE;
}

impl AvcStatus for ExtendedSubunitInfo {
    fn build_operands(&mut self, _: &AvcAddr, operands: &mut Vec<u8>) -> Result<(), Error> {
        operands.push(self.page);
        operands.push(self.func_blk_type);
        operands.extend_from_slice(&[0xff;25]);
        Ok(())
    }

    fn parse_operands(&mut self, _: &AvcAddr, operands: &[u8]) -> Result<(), Error> {
        if operands.len() != 27 {
            let label = format!("Unexpected length of operands for ExtendedSubunitInfo: {}",
                                operands.len());
            Err(Error::new(Ta1394AvcError::UnexpectedRespOperands, &label))
        } else if self.page != operands[0] {
            let label = format!("Unexpected value of page for ExtendedSubunitInfo: {} but {}",
                                self.page, operands[0]);
            Err(Error::new(Ta1394AvcError::UnexpectedRespOperands, &label))
        } else if self.func_blk_type != operands[1] {
            let label = format!("Unexpected value of function block type for ExtendedSubunitInfo: {} but {}",
                                self.func_blk_type, operands[2]);
            Err(Error::new(Ta1394AvcError::UnexpectedRespOperands, &label))
        } else {
            self.entries = (0..5).filter(|i| operands[2 + i * 5] != 0xff).map(|i| {
                let pos = 2 + i * 5;
                let mut raw = [0;5];
                raw.copy_from_slice(&operands[pos..(pos + 5)]);
                ExtendedSubunitInfoEntry::from(&raw)
            }).collect();
            Ok(())
        }
    }
}

//
// Bco Extended Stream Format Info command
//
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum BcoCompoundAm824StreamFormat{
    Iec60958_3,
    Iec61937_3,
    Iec61937_4,
    Iec61937_5,
    Iec61937_6,
    Iec61937_7,
    MultiBitLinearAudioRaw,
    MultiBitLinearAudioDvd,
    HighPrecisionMultiBitLinearAudio,
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
}

impl From<u8> for BcoCompoundAm824StreamFormat {
    fn from(val: u8) -> Self {
        match val {
            BcoCompoundAm824StreamFormat::IEC60958_3 => BcoCompoundAm824StreamFormat::Iec60958_3,
            BcoCompoundAm824StreamFormat::IEC61937_3 => BcoCompoundAm824StreamFormat::Iec61937_3,
            BcoCompoundAm824StreamFormat::IEC61937_4 => BcoCompoundAm824StreamFormat::Iec61937_4,
            BcoCompoundAm824StreamFormat::IEC61937_5 => BcoCompoundAm824StreamFormat::Iec61937_5,
            BcoCompoundAm824StreamFormat::IEC61937_6 => BcoCompoundAm824StreamFormat::Iec61937_6,
            BcoCompoundAm824StreamFormat::IEC61937_7 => BcoCompoundAm824StreamFormat::Iec61937_7,
            BcoCompoundAm824StreamFormat::MULTI_BIT_LINEAR_AUDIO_RAW => BcoCompoundAm824StreamFormat::MultiBitLinearAudioRaw,
            BcoCompoundAm824StreamFormat::MULTI_BIT_LINEAR_AUDIO_DVD => BcoCompoundAm824StreamFormat::MultiBitLinearAudioDvd,
            BcoCompoundAm824StreamFormat::HIGH_PRECISION_MULTI_BIT_LINEAR_AUDIO => BcoCompoundAm824StreamFormat::HighPrecisionMultiBitLinearAudio,
            BcoCompoundAm824StreamFormat::MIDI_CONFORMANT => BcoCompoundAm824StreamFormat::MidiConformant,
            _ => BcoCompoundAm824StreamFormat::Reserved(val),
        }
    }
}

impl From<BcoCompoundAm824StreamFormat> for u8 {
    fn from(fmt: BcoCompoundAm824StreamFormat) -> u8 {
        match fmt {
            BcoCompoundAm824StreamFormat::Iec60958_3 => BcoCompoundAm824StreamFormat::IEC60958_3,
            BcoCompoundAm824StreamFormat::Iec61937_3 => BcoCompoundAm824StreamFormat::IEC61937_3,
            BcoCompoundAm824StreamFormat::Iec61937_4 => BcoCompoundAm824StreamFormat::IEC61937_4,
            BcoCompoundAm824StreamFormat::Iec61937_5 => BcoCompoundAm824StreamFormat::IEC61937_5,
            BcoCompoundAm824StreamFormat::Iec61937_6 => BcoCompoundAm824StreamFormat::IEC61937_6,
            BcoCompoundAm824StreamFormat::Iec61937_7 => BcoCompoundAm824StreamFormat::IEC61937_7,
            BcoCompoundAm824StreamFormat::MultiBitLinearAudioRaw => BcoCompoundAm824StreamFormat::MULTI_BIT_LINEAR_AUDIO_RAW,
            BcoCompoundAm824StreamFormat::MultiBitLinearAudioDvd => BcoCompoundAm824StreamFormat::MULTI_BIT_LINEAR_AUDIO_DVD,
            BcoCompoundAm824StreamFormat::HighPrecisionMultiBitLinearAudio => BcoCompoundAm824StreamFormat::HIGH_PRECISION_MULTI_BIT_LINEAR_AUDIO,
            BcoCompoundAm824StreamFormat::MidiConformant => BcoCompoundAm824StreamFormat::MIDI_CONFORMANT,
            BcoCompoundAm824StreamFormat::Reserved(val) => val,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BcoCompoundAm824StreamEntry{
    pub count: u8,
    pub format: BcoCompoundAm824StreamFormat,
}

impl From<&[u8;2]> for BcoCompoundAm824StreamEntry {
    fn from(raw: &[u8;2]) -> Self {
        BcoCompoundAm824StreamEntry{
            count: raw[0],
            format: BcoCompoundAm824StreamFormat::from(raw[1]),
        }
    }
}

impl From<&BcoCompoundAm824StreamEntry> for [u8;2] {
    fn from(data: &BcoCompoundAm824StreamEntry) -> Self {
        [data.count, data.format.into()]
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BcoCompoundAm824Stream{
    pub freq: u32,
    pub sync_src: bool,
    pub rate_ctl: bool,
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

    const SYNC_SRC_MASK: u8 = 0x01;
    const SYNC_SRC_SHIFT: usize = 2;

    const RATE_CTL_MASK: u8 = 0x03;
    const RATE_CTL_SHIFT: usize = 0;
}

impl From<&[u8]> for BcoCompoundAm824Stream {
    fn from(raw: &[u8]) -> Self {
        let freq = match raw[0] {
            BcoCompoundAm824Stream::FREQ_CODE_22050 => 22050,
            BcoCompoundAm824Stream::FREQ_CODE_24000 => 24000,
            BcoCompoundAm824Stream::FREQ_CODE_32000 => 32000,
            BcoCompoundAm824Stream::FREQ_CODE_44100 => 44100,
            BcoCompoundAm824Stream::FREQ_CODE_48000 => 48000,
            BcoCompoundAm824Stream::FREQ_CODE_96000 => 96000,
            BcoCompoundAm824Stream::FREQ_CODE_176400 => 176400,
            BcoCompoundAm824Stream::FREQ_CODE_192000 => 192000,
            BcoCompoundAm824Stream::FREQ_CODE_88200 => 88200,
            _ => u32::MAX,
        };
        let sync_src_code =
            (raw[1] >> BcoCompoundAm824Stream::SYNC_SRC_SHIFT) & BcoCompoundAm824Stream::SYNC_SRC_MASK;
        let sync_src = sync_src_code > 0;
        let rate_ctl_code =
            (raw[1] >> BcoCompoundAm824Stream::RATE_CTL_SHIFT) & BcoCompoundAm824Stream::RATE_CTL_MASK;
        let rate_ctl = rate_ctl_code == 0;
        let entry_count = raw[2] as usize;
        let entries = (0..entry_count).filter_map(|i| {
            if 3 + i * 2 + 2 > raw.len() {
                None
            } else {
                let mut doublet = [0;2];
                doublet.copy_from_slice(&raw[(3 + i * 2)..(3 + i * 2 + 2)]);
                Some(BcoCompoundAm824StreamEntry::from(&doublet))
            }
        }).collect();
        BcoCompoundAm824Stream{freq, sync_src, rate_ctl, entries}
    }
}

impl From<&BcoCompoundAm824Stream> for Vec<u8> {
    fn from(data: &BcoCompoundAm824Stream) -> Self {
        let mut raw = Vec::new();
        let freq_code = match data.freq {
            22050 => BcoCompoundAm824Stream::FREQ_CODE_22050,
            24000 => BcoCompoundAm824Stream::FREQ_CODE_24000,
            32000 => BcoCompoundAm824Stream::FREQ_CODE_32000,
            44100 => BcoCompoundAm824Stream::FREQ_CODE_44100,
            48000 => BcoCompoundAm824Stream::FREQ_CODE_48000,
            96000 => BcoCompoundAm824Stream::FREQ_CODE_96000,
            176400 => BcoCompoundAm824Stream::FREQ_CODE_176400,
            192000 => BcoCompoundAm824Stream::FREQ_CODE_192000,
            88200 => BcoCompoundAm824Stream::FREQ_CODE_88200,
            _ => u8::MAX,
        };
        raw.push(freq_code);

        let sync_src_code = ((data.sync_src as u8) & BcoCompoundAm824Stream::SYNC_SRC_MASK) <<
                            BcoCompoundAm824Stream::SYNC_SRC_SHIFT;
        let rate_ctl_code = ((data.rate_ctl as u8) & BcoCompoundAm824Stream::RATE_CTL_MASK) <<
                            BcoCompoundAm824Stream::RATE_CTL_SHIFT;
        raw.push(sync_src_code | rate_ctl_code);

        raw.push(data.entries.len() as u8);
        data.entries.iter().for_each(|entry|{
            raw.extend_from_slice(&Into::<[u8;2]>::into(entry));
        });

        raw
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BcoAmStream{
    AmStream(AmStream),
    BcoStream(BcoCompoundAm824Stream),
}

impl From<&[u8]> for BcoAmStream {
    fn from(raw: &[u8]) -> Self {
        match raw[0] {
            AmStream::HIER_LEVEL_1_COMPOUND_AM824 => {
                let s = BcoCompoundAm824Stream::from(&raw[1..]);
                BcoAmStream::BcoStream(s)
            }
            _ => BcoAmStream::AmStream(AmStream::from(raw)),
        }
    }
}

impl From<&BcoAmStream> for Vec<u8> {
    fn from(data: &BcoAmStream) -> Self {
        match data {
            BcoAmStream::BcoStream(s) => {
                let mut raw = Vec::new();
                raw.push(AmStream::HIER_LEVEL_1_COMPOUND_AM824);
                raw.append(&mut Into::<Vec<u8>>::into(s));
                raw
            }
            _ => {
                Into::<Vec<u8>>::into(data)
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BcoStreamFormat{
    // Dvcr is not supported currently.
    Am(BcoAmStream),
    Reserved(Vec<u8>),
}

impl BcoStreamFormat {
    fn as_bco_am_stream(&self) -> Result<&BcoAmStream, Error> {
        if let BcoStreamFormat::Am(s) = self {
            Ok(s)
        } else {
            let label = "Bco Audio & Music stream is not available for the unit";
            Err(Error::new(FileError::Nxio, &label))
        }
    }

    pub fn as_am_stream(&self) -> Result<&AmStream, Error> {
        if let BcoAmStream::AmStream(s) = self.as_bco_am_stream()? {
            Ok(s)
        } else {
            let label = "Audio & Music stream is not available for the unit";
            Err(Error::new(FileError::Nxio, &label))
        }
    }

    pub fn as_bco_compound_am824_stream(&self) -> Result<&BcoCompoundAm824Stream, Error> {
        if let BcoAmStream::BcoStream(s) = self.as_bco_am_stream()? {
            Ok(s)
        } else {
            let label = "Bco Compound AM824 stream is not available for the unit";
            Err(Error::new(FileError::Nxio, &label))
        }
    }
}

impl From<&[u8]> for BcoStreamFormat {
    fn from(raw: &[u8]) -> Self {
        match raw[0] {
            StreamFormat::HIER_ROOT_AM => BcoStreamFormat::Am(BcoAmStream::from(&raw[1..])),
            _ => BcoStreamFormat::Reserved(raw.to_vec()),
        }
    }
}

impl From<&BcoStreamFormat> for Vec<u8> {
    fn from(data: &BcoStreamFormat) -> Self {
        let mut raw = Vec::new();
        match data {
            BcoStreamFormat::Am(i) => {
                raw.push(StreamFormat::HIER_ROOT_AM);
                raw.append(&mut i.into());
            }
            BcoStreamFormat::Reserved(d) => raw.extend_from_slice(d),
        }
        raw
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct BcoExtendedStreamFormat{
    subfunc: u8,
    plug_addr: BcoPlugAddr,
    support_status: SupportStatus,
}

impl BcoExtendedStreamFormat {
    const OPCODE: u8 = 0x2f;

    fn new(subfunc: u8, plug_addr: &BcoPlugAddr) -> Self {
        BcoExtendedStreamFormat{
            subfunc,
            plug_addr: *plug_addr,
            support_status: SupportStatus::Reserved(0xff),
        }
    }
}

impl AvcStatus for BcoExtendedStreamFormat {
    fn build_operands(&mut self, _: &AvcAddr, operands: &mut Vec<u8>) -> Result<(), Error> {
        operands.push(self.subfunc);
        operands.extend_from_slice(&Into::<[u8;5]>::into(&self.plug_addr));
        operands.push(self.support_status.into());
        Ok(())
    }

    fn parse_operands(&mut self, _: &AvcAddr, operands: &[u8]) -> Result<(), Error> {
        if operands.len() < 7 {
            let label = format!("Unexpected length of data for BcoExtendedStreamFormat: {}", operands.len());
            return Err(Error::new(Ta1394AvcError::UnexpectedRespOperands, &label));
        }

        if operands[0] != self.subfunc {
            let label = format!("Unexpected subfunction: {} but {}", self.subfunc, operands[0]);
            return Err(Error::new(Ta1394AvcError::UnexpectedRespOperands, &label));
        }

        let mut r = [0;5];
        r.copy_from_slice(&operands[1..6]);
        let plug_addr = BcoPlugAddr::from(&r);
        if plug_addr != self.plug_addr {
            let label = format!("Unexpected address for plug: {:?} but {:?}", self.plug_addr, plug_addr);
            return Err(Error::new(Ta1394AvcError::UnexpectedRespOperands, &label));
        }

        self.support_status = SupportStatus::from(operands[6]);

        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExtendedStreamFormatSingle{
    pub support_status: SupportStatus,
    pub stream_format: BcoStreamFormat,
    op: BcoExtendedStreamFormat,
}

impl ExtendedStreamFormatSingle {
    const SUBFUNC: u8 = 0xc0;

    pub fn new(plug_addr: &BcoPlugAddr) -> Self {
        ExtendedStreamFormatSingle{
            support_status: SupportStatus::Reserved(0xff),
            stream_format: BcoStreamFormat::Reserved(Vec::new()),
            op: BcoExtendedStreamFormat::new(Self::SUBFUNC, plug_addr),
        }
    }
}

impl AvcOp for ExtendedStreamFormatSingle {
    const OPCODE: u8 = BcoExtendedStreamFormat::OPCODE;
}

impl AvcStatus for ExtendedStreamFormatSingle {
    fn build_operands(&mut self, addr: &AvcAddr, operands: &mut Vec<u8>) -> Result<(), Error> {
        self.op.support_status = SupportStatus::Reserved(0xff);
        self.op.build_operands(addr, operands)?;
        Ok(())
    }

    fn parse_operands(&mut self, addr: &AvcAddr, operands: &[u8]) -> Result<(), Error> {
        self.op.parse_operands(addr, operands)?;

        self.support_status = self.op.support_status;
        self.stream_format = BcoStreamFormat::from(&operands[7..]);

        Ok(())
    }
}

impl AvcControl for ExtendedStreamFormatSingle {
    fn build_operands(&mut self, addr: &AvcAddr, operands: &mut Vec<u8>) -> Result<(), Error> {
        self.op.support_status = SupportStatus::Active;
        self.op.build_operands(addr, operands)?;
        operands.append(&mut Into::<Vec<u8>>::into(&self.stream_format));
        Ok(())
    }

    fn parse_operands(&mut self, addr: &AvcAddr, operands: &[u8]) -> Result<(), Error> {
        self.op.parse_operands(addr, operands)?;

        self.support_status = self.op.support_status;
        self.stream_format = BcoStreamFormat::from(&operands[7..]);

        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExtendedStreamFormatList{
    pub support_status: SupportStatus,
    pub index: u8,
    pub stream_format: BcoStreamFormat,
    op: BcoExtendedStreamFormat,
}

impl ExtendedStreamFormatList {
    const SUBFUNC: u8 = 0xc1;

    pub fn new(plug_addr: &BcoPlugAddr, index: u8) -> Self {
        ExtendedStreamFormatList{
            support_status: SupportStatus::NoInfo,
            index,
            stream_format: BcoStreamFormat::Reserved(Vec::new()),
            op: BcoExtendedStreamFormat::new(Self::SUBFUNC, plug_addr),
        }
    }
}

impl AvcOp for ExtendedStreamFormatList {
    const OPCODE: u8 = BcoExtendedStreamFormat::OPCODE;
}

impl AvcStatus for ExtendedStreamFormatList {
    fn build_operands(&mut self, addr: &AvcAddr, operands: &mut Vec<u8>) -> Result<(), Error> {
        self.op.build_operands(addr, operands)?;
        operands.push(self.index);
        Ok(())
    }

    fn parse_operands(&mut self, addr: &AvcAddr, operands: &[u8]) -> Result<(), Error> {
        self.op.parse_operands(addr, operands)?;

        self.support_status = self.op.support_status;

        if operands[7] != self.index {
            let label = format!("Unexpected index to stream entry: {} but {}",
                                self.index, operands[7]);
            return Err(Error::new(Ta1394AvcError::UnexpectedRespOperands, &label));
        }

        self.stream_format = BcoStreamFormat::from(&operands[8..]);

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::ta1394::{AvcSubunitType, AvcAddr};
    use crate::ta1394::AvcStatus;
    use super::{BcoPlugAddr, BcoPlugAddrMode, BcoPlugDirection};
    use super::{BcoPlugAddrUnit, BcoPlugAddrUnitType, BcoPlugAddrSubunit, BcoPlugAddrFuncBlk};
    use super::BcoPlugType;
    use super::{BcoLocation, BcoChannelInfo};
    use super::BcoChannelName;
    use super::{BcoClusterInfo, BcoPortType, BcoCluster};
    use super::{BcoIoPlugAddr, BcoIoPlugAddrMode};
    use super::BcoPlugInfo;
    use super::ExtendedPlugInfo;
    use super::ExtendedSubunitInfo;

    #[test]
    fn bcoplugaddr_from() {
        // Input plug for Unit.
        let raw = [0x00, 0x00, 0x00, 0x03, 0xff];
        let addr = BcoPlugAddr::from(&raw);
        assert_eq!(addr.direction, BcoPlugDirection::Input);
        if let BcoPlugAddrMode::Unit(d) = &addr.mode {
            assert_eq!(d.plug_type, BcoPlugAddrUnitType::Isoc);
            assert_eq!(d.plug_id, 0x03);
        } else {
            unreachable!();
        }
        assert_eq!(raw, Into::<[u8;5]>::into(&addr));

        // Output plug for Subunit.
        let raw = [0x01, 0x01, 0x04, 0xff, 0xff];
        let addr = BcoPlugAddr::from(&raw);
        assert_eq!(addr.direction, BcoPlugDirection::Output);
        if let BcoPlugAddrMode::Subunit(d) = &addr.mode {
            assert_eq!(d.plug_id, 0x04);
        } else {
            unreachable!();
        }
        assert_eq!(raw, Into::<[u8;5]>::into(&addr));

        // Input plug for function block.
        let raw = [0x02, 0x02, 0x06, 0x03, 0x02];
        let addr = BcoPlugAddr::from(&raw);
        assert_eq!(addr.direction, BcoPlugDirection::Reserved(0x02));
        if let BcoPlugAddrMode::FuncBlk(d) = &addr.mode {
            assert_eq!(d.func_blk_type, 0x06);
            assert_eq!(d.func_blk_id, 0x03);
            assert_eq!(d.plug_id, 0x02);
        } else {
            unreachable!();
        }
        assert_eq!(raw, Into::<[u8;5]>::into(&addr));
    }

    #[test]
    fn bcochannelinfo_from() {
        let raw = [0x02, 0x0d];
        assert_eq!(raw, Into::<[u8;2]>::into(&BcoChannelInfo::from(&raw)));
        let raw = [0x3e, 0x0c];
        assert_eq!(raw, Into::<[u8;2]>::into(&BcoChannelInfo::from(&raw)));
    }

    #[test]
    fn bcocluster_from() {
        let raw: Vec<u8> = vec![0x03, 0x03, 0x0b, 0x09, 0x03, 0x2c, 0x01];
        assert_eq!(raw, Into::<Vec<u8>>::into(&BcoCluster::from(raw.as_slice())));

        let raw: Vec<u8> = vec![0x05, 0x03, 0x0b, 0x09, 0x03, 0x2c, 0x01, 0x02, 0x0d, 0x3e, 0x0c];
        assert_eq!(raw, Into::<Vec<u8>>::into(&BcoCluster::from(raw.as_slice())));
    }

    #[test]
    fn bcoclusterinfo_from() {
        let raw: Vec<u8> = vec![0x03, 0x0a, 0x03, 0x4c, 0x51, 0x33];
        assert_eq!(raw, Into::<Vec<u8>>::into(&BcoClusterInfo::from(raw.as_slice())));
    }

    #[test]
    fn bcoioplugaddr_from() {
        // Unit.
        let raw: [u8;7] = [0x00, 0x00, 0x02, 0x05, 0xff, 0xff, 0xff];
        let addr = BcoIoPlugAddr::from(&raw);
        assert_eq!(addr.direction, BcoPlugDirection::Input);
        if let BcoIoPlugAddrMode::Unit(d) = &addr.mode {
            assert_eq!(d.plug_type, BcoPlugAddrUnitType::Async);
            assert_eq!(d.plug_id, 0x05);
        } else {
            unreachable!();
        }
        assert_eq!(raw, Into::<[u8;7]>::into(&addr));

        // Subunit.
        let raw: [u8;7] = [0x01, 0x01, 0x06, 0x05, 0x02, 0xff, 0xff];
        let addr = BcoIoPlugAddr::from(&raw);
        assert_eq!(addr.direction, BcoPlugDirection::Output);
        if let BcoIoPlugAddrMode::Subunit(s, d) = &addr.mode {
            assert_eq!(s.subunit_type, AvcSubunitType::Ca);
            assert_eq!(s.subunit_id, 0x05);
            assert_eq!(d.plug_id, 0x02);
        } else {
            unreachable!();
        }
        assert_eq!(raw, Into::<[u8;7]>::into(&addr));

        // Function block.
        let raw: [u8;7] = [0x00, 0x02, 0x04, 0x09, 0x80, 0x12, 0x23];
        let addr = BcoIoPlugAddr::from(&raw);
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
        assert_eq!(raw, Into::<[u8;7]>::into(&addr));
    }

    #[test]
    fn bcopluginfo_type_from() {
        let raw: Vec<u8> = vec![0x00, 0x03];
        let info = BcoPlugInfo::from(raw.as_slice());
        if let BcoPlugInfo::Type(t) = &info {
            assert_eq!(*t, BcoPlugType::Sync);
        } else {
            unreachable!();
        }
        assert_eq!(raw, Into::<Vec<u8>>::into(&info));

    }

    #[test]
    fn bcopluginfo_name_from() {
        let raw: Vec<u8> = vec![0x01, 0x03, 0x31, 0x32, 0x33];
        let info = BcoPlugInfo::from(raw.as_slice());
        if let BcoPlugInfo::Name(n) = &info {
            assert_eq!(n, "123");
        } else {
            unreachable!();
        }
        assert_eq!(raw, Into::<Vec<u8>>::into(&info));

    }

    #[test]
    fn bcopluginfo_chcount_from() {
        let raw: Vec<u8> = vec![0x02, 0xc3];
        let info = BcoPlugInfo::from(raw.as_slice());
        if let BcoPlugInfo::ChCount(c) = &info {
            assert_eq!(*c, 0xc3);
        } else {
            unreachable!();
        }
        assert_eq!(raw, Into::<Vec<u8>>::into(&info));
    }

    #[test]
    fn bcopluginfo_chpositions_from() {
        let raw: Vec<u8> = vec![0x03, 0x04,
                                0x01, 0x00, 0x04,
                                0x02, 0x03, 0x08, 0x00, 0x09,
                                0x03, 0x04, 0x08, 0x06, 0x08, 0x05, 0x07,
                                0x01, 0x09, 0xb];
        let info = BcoPlugInfo::from(raw.as_slice());
        if let BcoPlugInfo::ChPositions(clusters) = &info {
            assert_eq!(clusters.len(), 4);
            let m = &clusters[0];
            assert_eq!(m.entries.len(), 1);
            assert_eq!(m.entries[0], BcoChannelInfo{pos: 0x00, loc: BcoLocation::LowFrequencyEffect});
            let m = &clusters[1];
            assert_eq!(m.entries.len(), 2);
            assert_eq!(m.entries[0], BcoChannelInfo{pos: 0x03, loc: BcoLocation::RightCenter});
            assert_eq!(m.entries[1], BcoChannelInfo{pos: 0x00, loc: BcoLocation::Surround});
            let m = &clusters[2];
            assert_eq!(m.entries.len(), 3);
            assert_eq!(m.entries[0], BcoChannelInfo{pos: 0x04, loc: BcoLocation::RightCenter});
            assert_eq!(m.entries[1], BcoChannelInfo{pos: 0x06, loc: BcoLocation::RightCenter});
            assert_eq!(m.entries[2], BcoChannelInfo{pos: 0x05, loc: BcoLocation::LeftCenter});
            let m = &clusters[3];
            assert_eq!(m.entries.len(), 1);
            assert_eq!(m.entries[0], BcoChannelInfo{pos: 0x09, loc: BcoLocation::SideRight});
        } else {
            unreachable!();
        }
        assert_eq!(raw, Into::<Vec<u8>>::into(&info));
    }

    #[test]
    fn bcopluginfo_chname_from() {
        let raw: Vec<u8> = vec![0x04, 0x9a, 0x01, 0x39];
        let info = BcoPlugInfo::from(raw.as_slice());
        if let BcoPlugInfo::ChName(d) = &info {
            assert_eq!(d.ch, 0x9a);
            assert_eq!(d.name, "9");
        } else {
            unreachable!();
        }
        assert_eq!(raw, Into::<Vec<u8>>::into(&info));
    }

    #[test]
    fn bcopluginfo_input_from() {
        let raw: Vec<u8> = vec![0x05, 0xa9, 0x01, 0x0b, 0x07, 0x42, 0xff, 0xff];
        let info = BcoPlugInfo::from(raw.as_slice());
        if let BcoPlugInfo::Input(plug_addr) = &info {
            assert_eq!(plug_addr.direction, BcoPlugDirection::Reserved(0xa9));
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
        assert_eq!(raw, Into::<Vec<u8>>::into(&info));
    }

    #[test]
    fn bcopluginfo_outputs_from() {
        let raw: Vec<u8> = vec![0x06, 0x02,
                                0xa9, 0x01, 0x0b, 0x07, 0x42, 0xff, 0xff,
                                0xa9, 0x01, 0x0b, 0x07, 0x42, 0xff, 0xff];
        let info = BcoPlugInfo::from(raw.as_slice());
        if let BcoPlugInfo::Outputs(plug_addrs) = &info {
            let plug_addr = plug_addrs[0];
            assert_eq!(plug_addr.direction, BcoPlugDirection::Reserved(0xa9));
            if let BcoIoPlugAddrMode::Subunit(s, d) = &plug_addr.mode {
                assert_eq!(s.subunit_type, AvcSubunitType::CameraStorage);
                assert_eq!(s.subunit_id, 0x07);
                assert_eq!(d.plug_id, 0x42);
            } else {
                unreachable!();
            }
            let plug_addr = plug_addrs[1];
            assert_eq!(plug_addr.direction, BcoPlugDirection::Reserved(0xa9));
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
        assert_eq!(raw, Into::<Vec<u8>>::into(&info));
    }

    #[test]
    fn bcopluginfo_clusterinfo_from() {
        let raw: Vec<u8> = vec![0x07, 0x01, 0x09, 0x05, 0x41, 0x42, 0x43, 0x44, 0x45];
        let info = BcoPlugInfo::from(raw.as_slice());
        if let BcoPlugInfo::ClusterInfo(d) = &info {
            assert_eq!(d.index, 0x01);
            assert_eq!(d.port_type, BcoPortType::Digital);
            assert_eq!(d.name, "ABCDE");
        } else {
            unreachable!();
        }
        assert_eq!(raw, Into::<Vec<u8>>::into(&info));
    }

    #[test]
    fn extendedpluginfo_type_operands() {
        let raw: Vec<u8> = vec![0xc0, 0x01, 0x00, 0x00, 0x03, 0xff,
                                0x00, 0x00];
        let addr = BcoPlugAddr{
            direction: BcoPlugDirection::Output,
            mode: BcoPlugAddrMode::Unit(BcoPlugAddrUnit{
                plug_type: BcoPlugAddrUnitType::Isoc,
                plug_id: 0x03,
            }),
        };
        let info = BcoPlugInfo::Type(BcoPlugType::Reserved(0xff));
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
        let raw: Vec<u8> = vec![0xc0, 0x00, 0x01, 0x17, 0xff, 0xff,
                                0x01, 0x05, 0x39, 0x38, 0x52, 0x36, 0x35];
        let addr = BcoPlugAddr{
            direction: BcoPlugDirection::Input,
            mode: BcoPlugAddrMode::Subunit(BcoPlugAddrSubunit{
                plug_id: 0x17,
            }),
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
        let raw: Vec<u8> = vec![0xc0, 0x01, 0x02, 0x3e, 0x9a, 0x77,
                                0x02, 0xe4];
        let addr = BcoPlugAddr{
            direction: BcoPlugDirection::Output,
            mode: BcoPlugAddrMode::FuncBlk(BcoPlugAddrFuncBlk{
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
        let raw: Vec<u8> = vec![0xc0, 0x00, 0x00, 0x01, 0x5c, 0xff,
                                0x03, 0x03,
                                0x01, 0x00, 0x0a,
                                0x02, 0x03, 0x04, 0x02, 0x07,
                                0x03, 0x01, 0x0f, 0x04, 0x01, 0x05, 0x03];
        let addr = BcoPlugAddr{
            direction: BcoPlugDirection::Input,
            mode: BcoPlugAddrMode::Unit(BcoPlugAddrUnit{
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
            assert_eq!(e[0], BcoChannelInfo{pos: 0x00, loc: BcoLocation::SideLeft});
            let e = &entries[1].entries;
            assert_eq!(e.len(), 2);
            assert_eq!(e[0], BcoChannelInfo{pos: 0x03, loc: BcoLocation::LowFrequencyEffect});
            assert_eq!(e[1], BcoChannelInfo{pos: 0x02, loc: BcoLocation::LeftCenter});
            let e = &entries[2].entries;
            assert_eq!(e.len(), 3);
            assert_eq!(e[0], BcoChannelInfo{pos: 0x01, loc: BcoLocation::RightFrontEffect});
            assert_eq!(e[1], BcoChannelInfo{pos: 0x04, loc: BcoLocation::LeftFront});
            assert_eq!(e[2], BcoChannelInfo{pos: 0x05, loc: BcoLocation::Center});
        } else {
            unreachable!();
        }
    }

    #[test]
    fn extendedpluginfo_chname_operands() {
        let raw: Vec<u8> = vec![0xc0, 0x00, 0x00, 0x02, 0x97, 0xff,
                                0x04, 0x9d, 0x02, 0x46, 0x54];
        let addr = BcoPlugAddr{
            direction: BcoPlugDirection::Input,
            mode: BcoPlugAddrMode::Unit(BcoPlugAddrUnit{
                plug_type: BcoPlugAddrUnitType::Async,
                plug_id: 0x97,
            }),
        };
        let info = BcoPlugInfo::ChName(BcoChannelName{
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
        let raw: Vec<u8> = vec![0xc0, 0x01, 0x01, 0x5d, 0xff, 0xff,
                                0x05, 0x00, 0x02, 0x0c, 0x12, 0x80, 0xd9, 0x04];
        let addr = BcoPlugAddr{
            direction: BcoPlugDirection::Output,
            mode: BcoPlugAddrMode::Subunit(BcoPlugAddrSubunit{
                plug_id: 0x5d
            }),
        };
        let info = BcoPlugInfo::Input(BcoIoPlugAddr{
            direction: BcoPlugDirection::Reserved(0xff),
            mode: BcoIoPlugAddrMode::Reserved([0;6]),
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
        let raw: Vec<u8> = vec![0xc0, 0x01, 0x00, 0x00, 0x11, 0xff,
                                0x06, 0x02,
                                0x00, 0x02, 0x0c, 0x12, 0x80, 0xd9, 0x04,
                                0x00, 0x01, 0x0c, 0x03, 0x31, 0xff, 0xff];
        let addr = BcoPlugAddr{
            direction: BcoPlugDirection::Output,
            mode: BcoPlugAddrMode::Unit(BcoPlugAddrUnit{
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
        let raw: Vec<u8> = vec![0xc0, 0x01, 0x00, 0x01, 0x1e, 0xff,
                                0x07, 0x02, 0x05, 0x03, 0x60, 0x50, 0x70];
        let addr = BcoPlugAddr{
            direction: BcoPlugDirection::Output,
            mode: BcoPlugAddrMode::Unit(BcoPlugAddrUnit{
                plug_type: BcoPlugAddrUnitType::Ext,
                plug_id: 0x1e,
            }),
        };
        let info = BcoClusterInfo{
            index: 0x02,
            port_type: BcoPortType::Reserved(0xff),
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
        let operands = [0x00, 0xff,
                        0x81, 0x70, 0xd0, 0xe0, 0x03,
                        0x82, 0x60, 0xe0, 0xe0, 0x04,
                        0xff, 0xff, 0xff, 0xff, 0xff,
                        0xff, 0xff, 0xff, 0xff, 0xff,
                        0xff, 0xff, 0xff, 0xff, 0xff];
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
        let mut operands = Vec::new();
        AvcStatus::build_operands(&mut op, &AvcAddr::Unit, &mut operands).unwrap();
        assert_eq!(&operands[..2], &[0x00, 0xff]);
        assert_eq!(&operands[2..], &[0xff;25]);
    }
}
