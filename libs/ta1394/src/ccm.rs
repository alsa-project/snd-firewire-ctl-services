// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use super::*;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SignalUnitAddr {
    Isoc(u8),
    Ext(u8),
}

impl SignalUnitAddr {
    const EXT_PLUG_FLAG: u8 = 0x80;
    const PLUG_ID_MASK: u8 = 0x7f;
}

impl From<&[u8; 2]> for SignalUnitAddr {
    fn from(data: &[u8; 2]) -> Self {
        let plug_id = data[1] & Self::PLUG_ID_MASK;
        if data[1] & Self::EXT_PLUG_FLAG > 0 {
            Self::Ext(plug_id)
        } else {
            Self::Isoc(plug_id)
        }
    }
}

impl From<SignalUnitAddr> for [u8; 2] {
    fn from(addr: SignalUnitAddr) -> Self {
        let mut data = [0; 2];
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

impl From<&[u8; 2]> for SignalSubunitAddr {
    fn from(data: &[u8; 2]) -> Self {
        let subunit = AvcAddrSubunit::from(data[0]);
        let plug_id = data[1];
        SignalSubunitAddr { subunit, plug_id }
    }
}

impl From<SignalSubunitAddr> for [u8; 2] {
    fn from(addr: SignalSubunitAddr) -> Self {
        let mut data = [0; 2];
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
        SignalAddr::Subunit(SignalSubunitAddr {
            subunit: AvcAddrSubunit {
                subunit_type,
                subunit_id,
            },
            plug_id,
        })
    }
}

impl From<&[u8; 2]> for SignalAddr {
    fn from(data: &[u8; 2]) -> Self {
        if data[0] == AvcAddr::UNIT_ADDR {
            SignalAddr::Unit(data.into())
        } else {
            SignalAddr::Subunit(data.into())
        }
    }
}

impl From<SignalAddr> for [u8; 2] {
    fn from(addr: SignalAddr) -> Self {
        match addr {
            SignalAddr::Unit(a) => a.into(),
            SignalAddr::Subunit(a) => a.into(),
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct SignalSource {
    pub src: SignalAddr,
    pub dst: SignalAddr,
}

impl SignalSource {
    pub fn new(dst: &SignalAddr) -> Self {
        SignalSource {
            src: SignalAddr::Unit(SignalUnitAddr::Isoc(SignalUnitAddr::PLUG_ID_MASK)),
            dst: *dst,
        }
    }

    fn parse_operands(&mut self, operands: &[u8]) -> Result<(), Error> {
        if operands.len() > 4 {
            let mut doublet = [0; 2];
            doublet.copy_from_slice(&operands[1..3]);
            self.src = SignalAddr::from(&doublet);
            doublet.copy_from_slice(&operands[3..5]);
            self.dst = SignalAddr::from(&doublet);
            Ok(())
        } else {
            let label = format!("Oprands too short for VendorDependent; {}", operands.len());
            Err(Error::new(Ta1394AvcError::TooShortResp, &label))
        }
    }
}

impl AvcOp for SignalSource {
    const OPCODE: u8 = 0x1a;
}

impl AvcControl for SignalSource {
    fn build_operands(
        &mut self,
        _: &AvcAddr,
        operands: &mut Vec<u8>,
    ) -> Result<(), AvcCmdBuildError> {
        operands.push(0xff);
        operands.extend_from_slice(&Into::<[u8; 2]>::into(self.src));
        operands.extend_from_slice(&Into::<[u8; 2]>::into(self.dst));
        Ok(())
    }

    fn parse_operands(&mut self, _: &AvcAddr, operands: &[u8]) -> Result<(), Error> {
        Self::parse_operands(self, operands)
    }
}

impl AvcStatus for SignalSource {
    fn build_operands(
        &mut self,
        _: &AvcAddr,
        operands: &mut Vec<u8>,
    ) -> Result<(), AvcCmdBuildError> {
        operands.push(0xff);
        operands.extend_from_slice(&[0xff, 0xfe]);
        operands.extend_from_slice(&Into::<[u8; 2]>::into(self.dst));
        Ok(())
    }

    fn parse_operands(&mut self, _: &AvcAddr, operands: &[u8]) -> Result<(), Error> {
        Self::parse_operands(self, operands)
    }
}

#[cfg(test)]
mod test {
    use crate::ccm::*;

    #[test]
    fn signaladdr_from() {
        assert_eq!(
            [0xff, 0x00],
            Into::<[u8; 2]>::into(SignalAddr::from(&[0xff, 0x00]))
        );
        assert_eq!(
            [0xff, 0x27],
            Into::<[u8; 2]>::into(SignalAddr::from(&[0xff, 0x27]))
        );
        assert_eq!(
            [0xff, 0x87],
            Into::<[u8; 2]>::into(SignalAddr::from(&[0xff, 0x87]))
        );
        assert_eq!(
            [0xff, 0xc7],
            Into::<[u8; 2]>::into(SignalAddr::from(&[0xff, 0xc7]))
        );
        assert_eq!(
            [0x63, 0x07],
            Into::<[u8; 2]>::into(SignalAddr::from(&[0x63, 0x07]))
        );
        assert_eq!(
            [0x09, 0x11],
            Into::<[u8; 2]>::into(SignalAddr::from(&[0x09, 0x11]))
        );
    }

    #[test]
    fn signalsource_operands() {
        let operands = [0x00, 0x2e, 0x1c, 0xff, 0x05];
        let dst = SignalAddr::Unit(SignalUnitAddr::Isoc(0x05));
        let src = SignalAddr::Subunit(SignalSubunitAddr {
            subunit: AvcAddrSubunit::new(AvcSubunitType::Tuner, 0x06),
            plug_id: 0x1c,
        });
        let mut op = SignalSource::new(&dst);
        AvcStatus::parse_operands(&mut op, &AvcAddr::Unit, &operands).unwrap();
        assert_eq!(op.src, src);
        assert_eq!(op.dst, dst);

        let mut targets = Vec::new();
        AvcStatus::build_operands(&mut op, &AvcAddr::Unit, &mut targets).unwrap();
        assert_eq!(targets, [0xff, 0xff, 0xfe, 0xff, 0x05]);

        let mut targets = Vec::new();
        let src = SignalAddr::Subunit(SignalSubunitAddr {
            subunit: AvcAddrSubunit::new(AvcSubunitType::Extended, 0x05),
            plug_id: 0x07,
        });
        let dst = SignalAddr::Unit(SignalUnitAddr::Ext(0x03));
        let mut op = SignalSource { src, dst };
        AvcControl::build_operands(&mut op, &AvcAddr::Unit, &mut targets).unwrap();
        assert_eq!(targets, [0xff, 0xf5, 0x07, 0xff, 0x83]);

        let mut op = SignalSource {
            src: SignalAddr::Unit(SignalUnitAddr::Isoc(0xf)),
            dst: SignalAddr::Unit(SignalUnitAddr::Isoc(0xf)),
        };
        AvcControl::parse_operands(&mut op, &AvcAddr::Unit, &targets).unwrap();
        assert_eq!(op.src, src);
        assert_eq!(op.dst, dst);
    }
}
