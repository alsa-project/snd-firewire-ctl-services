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

impl Default for SignalUnitAddr {
    fn default() -> Self {
        Self::Isoc(Self::PLUG_ID_MASK)
    }
}

impl SignalUnitAddr {
    const EXT_PLUG_FLAG: u8 = 0x80;
    const PLUG_ID_MASK: u8 = 0x7f;

    const LENGTH: usize = 2;

    fn from_raw(raw: &[u8]) -> Result<Self, AvcRespParseError> {
        if raw.len() < Self::LENGTH {
            Err(AvcRespParseError::TooShortResp(Self::LENGTH))?;
        }

        let plug_id = raw[1] & Self::PLUG_ID_MASK;
        let plug = if raw[1] & Self::EXT_PLUG_FLAG > 0 {
            Self::Ext(plug_id)
        } else {
            Self::Isoc(plug_id)
        };
        Ok(plug)
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

impl Default for SignalSubunitAddr {
    fn default() -> Self {
        Self {
            subunit: Default::default(),
            plug_id: 0xff,
        }
    }
}

impl SignalSubunitAddr {
    const LENGTH: usize = 2;

    fn from_raw(raw: &[u8]) -> Result<Self, AvcRespParseError> {
        if raw.len() < Self::LENGTH {
            Err(AvcRespParseError::TooShortResp(Self::LENGTH))?;
        }

        let subunit = AvcAddrSubunit::from(raw[0]);
        let plug_id = raw[1];
        Ok(SignalSubunitAddr { subunit, plug_id })
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

impl Default for SignalAddr {
    fn default() -> Self {
        Self::Unit(Default::default())
    }
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

    fn from_raw(raw: &[u8]) -> Result<Self, AvcRespParseError> {
        if raw.len() < Self::LENGTH {
            Err(AvcRespParseError::TooShortResp(Self::LENGTH))?;
        }

        let addr = if raw[0] == AvcAddr::UNIT_ADDR {
            let data = SignalUnitAddr::from_raw(&raw)?;
            SignalAddr::Unit(data)
        } else {
            let data = SignalSubunitAddr::from_raw(&raw)?;
            SignalAddr::Subunit(data)
        };

        Ok(addr)
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
    const LENGTH_MIN: usize = 5;

    pub fn new(dst: &SignalAddr) -> Self {
        SignalSource {
            dst: *dst,
            ..Default::default()
        }
    }

    fn build_operands(&self, for_status: bool) -> Result<Vec<u8>, AvcCmdBuildError> {
        let mut operands = Vec::new();
        operands.push(0xff);

        if for_status {
            operands.extend_from_slice(&[0xff, 0xfe]);
        } else {
            operands.extend_from_slice(&self.src.to_raw());
        }

        operands.extend_from_slice(&self.dst.to_raw());
        Ok(operands)
    }

    fn parse_operands(&mut self, operands: &[u8]) -> Result<(), AvcRespParseError> {
        if operands.len() < Self::LENGTH_MIN {
            Err(AvcRespParseError::TooShortResp(4))?;
        }

        self.src = SignalAddr::from_raw(&operands[1..3]).map_err(|err| err.add_offset(1))?;
        self.dst = SignalAddr::from_raw(&operands[3..5]).map_err(|err| err.add_offset(3))?;
        Ok(())
    }
}

impl Default for SignalSource {
    fn default() -> Self {
        Self {
            src: Default::default(),
            dst: Default::default(),
        }
    }
}

impl AvcOp for SignalSource {
    const OPCODE: u8 = 0x1a;
}

impl AvcControl for SignalSource {
    fn build_operands(&mut self, _: &AvcAddr) -> Result<Vec<u8>, AvcCmdBuildError> {
        Self::build_operands(&self, false)
    }

    fn parse_operands(&mut self, _: &AvcAddr, operands: &[u8]) -> Result<(), AvcRespParseError> {
        Self::parse_operands(self, operands)
    }
}

impl AvcStatus for SignalSource {
    fn build_operands(&mut self, _: &AvcAddr) -> Result<Vec<u8>, AvcCmdBuildError> {
        Self::build_operands(&self, true)
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
        let raw = [0xff, 0x00];
        let addr = SignalAddr::from_raw(&raw).unwrap();
        assert_eq!(raw, addr.to_raw());

        let raw = [0xff, 0x27];
        let addr = SignalAddr::from_raw(&raw).unwrap();
        assert_eq!(raw, addr.to_raw());

        let raw = [0xff, 0x87];
        let addr = SignalAddr::from_raw(&raw).unwrap();
        assert_eq!(raw, addr.to_raw());

        let raw = [0xff, 0xc7];
        let addr = SignalAddr::from_raw(&raw).unwrap();
        assert_eq!(raw, addr.to_raw());

        let raw = [0x63, 0x07];
        let addr = SignalAddr::from_raw(&raw).unwrap();
        assert_eq!(raw, addr.to_raw());

        let raw = [0x09, 0x11];
        let addr = SignalAddr::from_raw(&raw).unwrap();
        assert_eq!(raw, addr.to_raw());
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

        let targets = AvcStatus::build_operands(&mut op, &AvcAddr::Unit).unwrap();
        assert_eq!(targets, [0xff, 0xff, 0xfe, 0xff, 0x05]);

        let src = SignalAddr::Subunit(SignalSubunitAddr {
            subunit: AvcAddrSubunit::new(AvcSubunitType::Extended, 0x05),
            plug_id: 0x07,
        });
        let dst = SignalAddr::Unit(SignalUnitAddr::Ext(0x03));
        let mut op = SignalSource { src, dst };
        let targets = AvcControl::build_operands(&mut op, &AvcAddr::Unit).unwrap();
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
