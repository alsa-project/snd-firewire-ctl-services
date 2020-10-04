// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

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

#[cfg(test)]
mod test {
    use super::{BcoPlugAddr, BcoPlugAddrMode, BcoPlugDirection};
    use super::BcoPlugAddrUnitType;

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
}
