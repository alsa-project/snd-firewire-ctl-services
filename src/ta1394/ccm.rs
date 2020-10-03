// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use super::{AvcSubunitType, AvcAddr, AvcAddrSubunit};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SignalUnitAddr {
    Isoc(u8),
    Ext(u8),
}

impl SignalUnitAddr {
    const EXT_PLUG_FLAG: u8 = 0x80;
    const PLUG_ID_MASK: u8 = 0x7f;
}

impl From<&[u8;2]> for SignalUnitAddr {
    fn from(data: &[u8;2]) -> Self {
        let plug_id = data[1] & Self::PLUG_ID_MASK;
        if data[1] & Self::EXT_PLUG_FLAG > 0 {
            Self::Ext(plug_id)
        } else {
            Self::Isoc(plug_id)
        }
    }
}

impl From<SignalUnitAddr> for [u8;2] {
    fn from(addr: SignalUnitAddr) -> Self {
        let mut data = [0;2];
        data[0] = AvcAddr::UNIT_ADDR;
        data[1] = match addr {
            SignalUnitAddr::Isoc(val) => val,
            SignalUnitAddr::Ext(val) => SignalUnitAddr::EXT_PLUG_FLAG | val,
        };
        data
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct SignalSubunitAddr {
    pub subunit: AvcAddrSubunit,
    pub plug_id: u8,
}

impl From<&[u8;2]> for SignalSubunitAddr {
    fn from(data: &[u8;2]) -> Self {
        let subunit = AvcAddrSubunit::from(data[0]);
        let plug_id = data[1];
        SignalSubunitAddr{subunit, plug_id}
    }
}

impl From<SignalSubunitAddr> for [u8;2] {
    fn from(addr: SignalSubunitAddr) -> Self {
        let mut data = [0;2];
        data[0] = u8::from(addr.subunit);
        data[1] = addr.plug_id;
        data
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SignalAddr {
    Unit(SignalUnitAddr),
    Subunit(SignalSubunitAddr),
}

impl SignalAddr {
    pub fn new_for_isoc_unit(plug_id: u8) -> Self {
        SignalAddr::Unit(SignalUnitAddr::Isoc(plug_id & SignalUnitAddr::PLUG_ID_MASK))
    }

    pub fn new_for_ext_unit(plug_id: u8) -> Self {
        SignalAddr::Unit(SignalUnitAddr::Ext(plug_id & SignalUnitAddr::PLUG_ID_MASK))
    }

    pub fn new_for_subunit(subunit_type: AvcSubunitType, subunit_id: u8, plug_id: u8) -> Self {
        SignalAddr::Subunit(SignalSubunitAddr{
            subunit: AvcAddrSubunit{subunit_type, subunit_id},
            plug_id,
        })
    }
}

impl From<&[u8;2]> for SignalAddr {
    fn from(data: &[u8;2]) -> Self {
        if data[0] == AvcAddr::UNIT_ADDR {
            SignalAddr::Unit(data.into())
        } else {
            SignalAddr::Subunit(data.into())
        }
    }
}

impl From<SignalAddr> for [u8;2] {
    fn from(addr: SignalAddr) -> Self {
        match addr {
            SignalAddr::Unit(a) => a.into(),
            SignalAddr::Subunit(a) => a.into(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::SignalAddr;

    #[test]
    fn signaladdr_from() {
        assert_eq!([0xff, 0x00], Into::<[u8;2]>::into(SignalAddr::from(&[0xff, 0x00])));
        assert_eq!([0xff, 0x27], Into::<[u8;2]>::into(SignalAddr::from(&[0xff, 0x27])));
        assert_eq!([0xff, 0x87], Into::<[u8;2]>::into(SignalAddr::from(&[0xff, 0x87])));
        assert_eq!([0xff, 0xc7], Into::<[u8;2]>::into(SignalAddr::from(&[0xff, 0xc7])));
        assert_eq!([0x63, 0x07], Into::<[u8;2]>::into(SignalAddr::from(&[0x63, 0x07])));
        assert_eq!([0x09, 0x11], Into::<[u8;2]>::into(SignalAddr::from(&[0x09, 0x11])));
    }
}
