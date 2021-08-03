// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol implementation for Focusrite Saffire series based on BeBoB solution.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by Focusrite Audio Engineering for Saffire series based on BeBoB solution.

use glib::Error;

use ta1394::{general::*, *};

/// OUI registerd to IEEE for Focusrite Audio Engineering Ltd.
pub const FOCUSRITE_OUI: [u8; 3] = [0x00, 0x13, 0x0e];

/// The maximum number of offsets read/written at once.
pub const MAXIMUM_OFFSET_COUNT: usize = 20;

/// The structure of AV/C vendor-dependent command for configuration operation. The number of
/// offsets read/written at once is 20.
#[derive(Debug)]
pub struct SaffireAvcOperation {
    pub offsets: Vec<usize>,
    pub buf: Vec<u8>,
    op: VendorDependent,
}

impl Default for SaffireAvcOperation {
    fn default() -> Self {
        Self {
            offsets: Default::default(),
            buf: Default::default(),
            op: VendorDependent {
                company_id: FOCUSRITE_OUI,
                data: Default::default(),
            },
        }
    }
}

impl AvcOp for SaffireAvcOperation {
    const OPCODE: u8 = VendorDependent::OPCODE;
}

// NOTE: IEC 61883 transaction layer in ASIC is a bit heavy load, thus it's preferable not to use
// them so often.
const FOCUSRITE_CONTROL_ACTION: u8 = 0x01;
const FOCUSRITE_STATUS_ACTION: u8 = 0x03;

impl AvcControl for SaffireAvcOperation {
    fn build_operands(&mut self, addr: &AvcAddr, operands: &mut Vec<u8>) -> Result<(), Error> {
        assert!(self.offsets.len() <= MAXIMUM_OFFSET_COUNT);
        assert_eq!(self.offsets.len() * 4, self.buf.len());

        let data = &mut self.op.data;
        let buf = &self.buf;
        data.clear();
        data.push(FOCUSRITE_CONTROL_ACTION);
        data.push(self.offsets.len() as u8);
        self.offsets.iter().enumerate().for_each(|(i, &offset)| {
            let idx = (offset / 4) as u32;
            let pos = i * 4;
            data.extend_from_slice(&idx.to_be_bytes());
            data.extend_from_slice(&buf[pos..(pos + 4)]);
        });
        AvcControl::build_operands(&mut self.op, addr, operands)
    }

    fn parse_operands(&mut self, addr: &AvcAddr, operands: &[u8]) -> Result<(), Error> {
        AvcControl::parse_operands(&mut self.op, addr, operands)?;
        (0..self.offsets.len()).for_each(|i| {
            let data = &self.op.data[(5 + i * 8 + 4)..(5 + i * 8 + 8)];
            let buf = &mut self.buf[(i * 4)..(i * 4 + 4)];
            buf.copy_from_slice(data);
        });
        Ok(())
    }
}

impl AvcStatus for SaffireAvcOperation {
    fn build_operands(&mut self, addr: &AvcAddr, operands: &mut Vec<u8>) -> Result<(), Error> {
        assert!(self.offsets.len() <= MAXIMUM_OFFSET_COUNT);
        assert_eq!(self.offsets.len() * 4, self.buf.len());

        let data = &mut self.op.data;
        data.clear();
        data.push(FOCUSRITE_STATUS_ACTION);
        data.push(self.offsets.len() as u8);
        self.offsets.iter().for_each(|&offset| {
            let idx = (offset / 4) as u32;
            data.extend_from_slice(&idx.to_be_bytes());
            data.extend_from_slice(&[0xff; 4]);
        });
        AvcStatus::build_operands(&mut self.op, addr, operands)
    }

    fn parse_operands(&mut self, addr: &AvcAddr, operands: &[u8]) -> Result<(), Error> {
        AvcStatus::parse_operands(&mut self.op, addr, operands)?;
        (0..self.offsets.len()).for_each(|i| {
            let data = &self.op.data[(2 + i * 8 + 4)..(2 + i * 8 + 8)];
            let buf = &mut self.buf[(i * 4)..(i * 4 + 4)];
            buf.copy_from_slice(data);
        });
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn vendor_dependent_control_operands() {
        let mut op = SaffireAvcOperation {
            offsets: vec![0x40, 0x400],
            buf: vec![0x01, 0x23, 0x45, 0x67, 0x76, 0x54, 0x32, 0x10],
            ..Default::default()
        };
        let mut generated = Vec::new();
        AvcControl::build_operands(&mut op, &AvcAddr::Unit, &mut generated).unwrap();

        let expected = [
            0x00, 0x13, 0x0e,
            0x01, 0x02,
            0x00, 0x00, 0x00, 0x10,
            0x01, 0x23, 0x45, 0x67,
            0x00, 0x00, 0x01, 0x00,
            0x76, 0x54, 0x32, 0x10,
        ];
        assert_eq!(&generated, &expected);

        let resp = [
            0x09, 0xff, 0x00,
            0x00, 0x13, 0x0e,
            0x01, 0x02,
            0x00, 0x00, 0x00, 0x10,
            0x76, 0x54, 0x32, 0x10,
            0x00, 0x00, 0x01, 0x00,
            0x01, 0x23, 0x45, 0x67,
        ];
        let mut op = SaffireAvcOperation {
            offsets: vec![0x40, 0x400],
            buf: vec![0; 8],
            ..Default::default()
        };
        AvcControl::parse_operands(&mut op, &AvcAddr::Unit, &resp).unwrap();
        assert_eq!(op.offsets[0], 0x40);
        assert_eq!(&op.buf[..4], &[0x76, 0x54, 0x32, 0x10]);
        assert_eq!(op.offsets[1], 0x400);
        assert_eq!(&op.buf[4..], &[0x01, 0x23, 0x45, 0x67]);
    }

    #[test]
    fn vendor_dependent_status_operands() {
        let resp = [
            0x00, 0x13, 0x0e,
            0x03, 0x02,
            0x00, 0x00, 0x00, 0x10,
            0x00, 0x00, 0x00, 0xff,
            0x00, 0x00, 0x01, 0x00,
            0x00, 0xff, 0x00, 0xff,
        ];
        let mut op = SaffireAvcOperation {
            offsets: vec![0x40, 0x400],
            buf: vec![0; 8],
            ..Default::default()
        };
        AvcStatus::parse_operands(&mut op, &AvcAddr::Unit, &resp).unwrap();
        assert_eq!(op.offsets[0], 0x40);
        assert_eq!(&op.buf[..4], &[0x00, 0x00, 0x00, 0xff]);
        assert_eq!(op.offsets[1], 0x400);
        assert_eq!(&op.buf[4..], &[0x00, 0xff, 0x00, 0xff]);

        let mut op = SaffireAvcOperation {
            offsets: vec![0x40, 0x400],
            buf: vec![0; 8],
            ..Default::default()
        };
        let mut generated = Vec::new();
        AvcStatus::build_operands(&mut op, &AvcAddr::Unit, &mut generated).unwrap();

        let expected = [
            0x00, 0x13, 0x0e,
            0x03, 0x02,
            0x00, 0x00, 0x00, 0x10,
            0xff, 0xff, 0xff, 0xff,
            0x00, 0x00, 0x01, 0x00,
            0xff, 0xff, 0xff, 0xff,
        ];
        assert_eq!(&generated, &expected);
    }
}
