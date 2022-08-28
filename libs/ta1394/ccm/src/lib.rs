// SPDX-License-Identifier: MIT
// Copyright (c) 2022 Takashi Sakamoto

#![doc = include_str!("../README.md")]

use ta1394_avc_general::*;

/// Address of plug in unit.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SignalUnitAddr {
    /// The plug for isochronous stream.
    Isoc(
        /// The numeric identifier of plug.
        u8,
    ),
    /// The plug for external signal.
    Ext(
        /// The numeric identifier of plug.
        u8,
    ),
}

impl SignalUnitAddr {
    const EXT_PLUG_FLAG: u8 = 0x80;
    const PLUG_ID_MASK: u8 = 0x7f;

    const LENGTH: usize = 2;

    fn from_raw(raw: &[u8]) -> Self {
        assert!(raw.len() >= Self::LENGTH);
        let plug_id = raw[1] & Self::PLUG_ID_MASK;
        if raw[1] & Self::EXT_PLUG_FLAG > 0 {
            Self::Ext(plug_id)
        } else {
            Self::Isoc(plug_id)
        }
    }

    fn to_raw(&self) -> [u8; Self::LENGTH] {
        let mut raw = [0; Self::LENGTH];
        raw[0] = AvcAddr::UNIT_ADDR;
        raw[1] = match self {
            SignalUnitAddr::Isoc(val) => *val,
            SignalUnitAddr::Ext(val) => SignalUnitAddr::EXT_PLUG_FLAG | *val,
        };
        raw
    }
}

/// Address of plug in subunit.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct SignalSubunitAddr {
    /// The address of subunit.
    pub subunit: AvcAddrSubunit,
    /// The numeric identifier of plug.
    pub plug_id: u8,
}

impl SignalSubunitAddr {
    const LENGTH: usize = 2;

    fn from_raw(raw: &[u8]) -> Self {
        assert!(raw.len() >= Self::LENGTH);
        let subunit = AvcAddrSubunit::from(raw[0]);
        let plug_id = raw[1];
        SignalSubunitAddr { subunit, plug_id }
    }

    fn to_raw(&self) -> [u8; Self::LENGTH] {
        let mut raw = [0; Self::LENGTH];
        raw[0] = u8::from(self.subunit);
        raw[1] = self.plug_id;
        raw
    }
}

/// Address of plug for signal source or destination.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SignalAddr {
    Unit(SignalUnitAddr),
    Subunit(SignalSubunitAddr),
}

impl SignalAddr {
    const LENGTH: usize = 2;

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

    fn from_raw(raw: &[u8]) -> Self {
        assert!(raw.len() >= Self::LENGTH);
        if raw[0] == AvcAddr::UNIT_ADDR {
            SignalAddr::Unit(SignalUnitAddr::from_raw(&raw))
        } else {
            SignalAddr::Subunit(SignalSubunitAddr::from_raw(&raw))
        }
    }

    fn to_raw(&self) -> [u8; Self::LENGTH] {
        match self {
            SignalAddr::Unit(a) => a.to_raw(),
            SignalAddr::Subunit(a) => a.to_raw(),
        }
    }
}

/// AV/C SIGNAL SOURCE command
///
/// Described in clause 7.1.1 SIGNAL SOURCE control command format.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct SignalSource {
    /// The source of signal.
    pub src: SignalAddr,
    /// The destination of signal.
    pub dst: SignalAddr,
}

impl SignalSource {
    pub fn new(dst: &SignalAddr) -> Self {
        SignalSource {
            src: SignalAddr::Unit(SignalUnitAddr::Isoc(SignalUnitAddr::PLUG_ID_MASK)),
            dst: *dst,
        }
    }

    fn parse_operands(&mut self, operands: &[u8]) -> Result<(), AvcRespParseError> {
        if operands.len() > 4 {
            self.src = SignalAddr::from_raw(&operands[1..3]);
            self.dst = SignalAddr::from_raw(&operands[3..5]);
            Ok(())
        } else {
            Err(AvcRespParseError::TooShortResp(4))
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
        operands.extend_from_slice(&self.src.to_raw());
        operands.extend_from_slice(&self.dst.to_raw());
        Ok(())
    }

    fn parse_operands(&mut self, _: &AvcAddr, operands: &[u8]) -> Result<(), AvcRespParseError> {
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
        operands.extend_from_slice(&self.dst.to_raw());
        Ok(())
    }

    fn parse_operands(&mut self, _: &AvcAddr, operands: &[u8]) -> Result<(), AvcRespParseError> {
        Self::parse_operands(self, operands)
    }
}

#[cfg(test)]
mod test {
    use crate::*;

    #[test]
    fn signaladdr_from() {
        assert_eq!([0xff, 0x00], SignalAddr::from_raw(&[0xff, 0x00]).to_raw());
        assert_eq!([0xff, 0x27], SignalAddr::from_raw(&[0xff, 0x27]).to_raw());
        assert_eq!([0xff, 0x87], SignalAddr::from_raw(&[0xff, 0x87]).to_raw());
        assert_eq!([0xff, 0xc7], SignalAddr::from_raw(&[0xff, 0xc7]).to_raw());
        assert_eq!([0x63, 0x07], SignalAddr::from_raw(&[0x63, 0x07]).to_raw());
        assert_eq!([0x09, 0x11], SignalAddr::from_raw(&[0x09, 0x11]).to_raw());
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
