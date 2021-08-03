// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol implementation for Focusrite Saffire series based on BeBoB solution.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by Focusrite Audio Engineering for Saffire series based on BeBoB solution.
//!
//! ## The way to refer to or change content of address space
//!
//! The models in Saffire and Saffire Pro i/o series based on BeBoB solution allows the other node
//! in IEEE 1394 bus to refer to or change content of address space by three ways:
//!
//!  1. AV/C vendor dependent command with specific data fields by IEC 61883-1 FCP transaction
//!  2. Quadlet read or write operation by asynchronous transaction
//!  3. Block read or write operation by asynchronous transaction
//!
//! In the way 1, the data consists of three fields:
//!
//!   * action code
//!   * the number of quadlets to operate
//!   * successive quadlet-aligned data with one-quarter of offset, and value
//!
//! However, due to the heavy load of FCP transaction layer in ASIC side, the AV/C transaction can
//! not be operated so frequently.
//!
//! In the way 2, the transaction for read operation is sent to offset plus 0x'0001'0000'0000. The
//! transaction for write operation is sent to offset plus 0x'0001'0001'0000.
//!
//! When operating batch of quadlets, the way 3 is available. As well as quadlet operation, the
//! transaction for read operation is sent to offset plus 0x'0001'0000'0000. On the other hand,
//! the transaction for write operation is sent to 0x'0001'0001'0000 with the same quadlet-aligned
//! data with one-quarter of offset, and value, as the case of AV/C vendor dependent command.

pub mod saffire;
pub mod saffireproio;

use glib::Error;

use hinawa::{FwNode, FwReq, FwReqExtManual, FwTcode};

use ta1394::{general::*, *};

/// OUI registerd to IEEE for Focusrite Audio Engineering Ltd.
pub const FOCUSRITE_OUI: [u8; 3] = [0x00, 0x13, 0x0e];

/// The structure for output parameters.
#[derive(Default)]
pub struct SaffireOutputParameters {
    pub mutes: Vec<bool>,
    pub vols: Vec<u8>,
    pub hwctls: Vec<bool>,
    pub dims: Vec<bool>,
    pub pads: Vec<bool>,
}

const PAD_FLAG: u32 = 0x10000000;
const HWCTL_FLAG: u32 = 0x08000000;
//TODO: const DIRECT_FLAG: u32 = 0x04000000;
const MUTE_FLAG: u32 = 0x02000000;
const DIM_FLAG: u32 = 0x01000000;
const VOL_MASK: u32 = 0x000000ff;

/// The trait for operations of output parameters.
pub trait SaffireOutputOperation {
    // NOTE: the series of offset should be continuous.
    const OFFSETS: &'static [usize];

    const MUTE_COUNT: usize;
    const VOL_COUNT: usize;
    const HWCTL_COUNT: usize;
    const DIM_COUNT: usize;
    const PAD_COUNT: usize;

    const LEVEL_MIN: u8 = 0x00;
    const LEVEL_MAX: u8 = 0xff;
    const LEVEL_STEP: u8 = 0x01;

    fn create_output_parameters() -> SaffireOutputParameters {
        SaffireOutputParameters {
            mutes: vec![Default::default(); Self::MUTE_COUNT],
            vols: vec![Default::default(); Self::VOL_COUNT],
            hwctls: vec![Default::default(); Self::HWCTL_COUNT],
            dims: vec![Default::default(); Self::DIM_COUNT],
            pads: vec![Default::default(); Self::PAD_COUNT],
        }
    }

    fn read_output_parameters(
        req: &FwReq,
        node: &FwNode,
        params: &mut SaffireOutputParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut buf = vec![0; Self::OFFSETS.len() * 4];
        saffire_read_quadlets(req, node, &Self::OFFSETS, &mut buf, timeout_ms)?;

        let mut quadlet = [0; 4];
        let vals = (0..Self::OFFSETS.len()).fold(Vec::new(), |mut vals, i| {
            let pos = i * 4;
            quadlet.copy_from_slice(&buf[pos..(pos + 4)]);
            vals.push(u32::from_be_bytes(quadlet));
            vals
        });

        params
            .mutes
            .iter_mut()
            .zip(vals.iter())
            .for_each(|(mute, &val)| *mute = val & MUTE_FLAG > 0);

        params
            .vols
            .iter_mut()
            .zip(vals.iter())
            .for_each(|(vol, &val)| *vol = 0xff - (val & VOL_MASK) as u8);

        params
            .hwctls
            .iter_mut()
            .zip(vals.iter())
            .for_each(|(hwctl, &val)| *hwctl = val & HWCTL_FLAG > 0);

        params
            .dims
            .iter_mut()
            .zip(vals.iter())
            .for_each(|(dim, &val)| *dim = val & DIM_FLAG > 0);

        params
            .pads
            .iter_mut()
            .zip(vals.iter())
            .for_each(|(pad, val)| *pad = val & PAD_FLAG > 0);

        Ok(())
    }

    fn write_mutes(
        req: &FwReq,
        node: &FwNode,
        mutes: &[bool],
        params: &mut SaffireOutputParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let old_mutes = &mut params.mutes;
        let vols = &params.vols;
        let hwctls = &params.hwctls;
        let dims = &params.dims;
        let pads = &params.pads;

        let (offsets, buf) = old_mutes
            .iter()
            .zip(mutes.iter())
            .zip(Self::OFFSETS.iter())
            .enumerate()
            .filter(|(_, ((old, new), _))| !old.eq(new))
            .fold(
                (Vec::new(), Vec::new()),
                |(mut offsets, mut buf), (i, (_, &offset))| {
                    offsets.push(offset);
                    let val = build_output_parameter(mutes, vols, hwctls, dims, pads, i);
                    buf.extend_from_slice(&val.to_be_bytes());
                    (offsets, buf)
                },
            );

        if offsets.len() > 0 {
            saffire_write_quadlets(req, node, &offsets, &buf, timeout_ms)
                .map(|_| old_mutes.copy_from_slice(mutes))
        } else {
            Ok(())
        }
    }

    fn write_vols(
        req: &FwReq,
        node: &FwNode,
        vols: &[u8],
        params: &mut SaffireOutputParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mutes = &params.mutes;
        let old_vols = &mut params.vols;
        let hwctls = &params.hwctls;
        let dims = &params.dims;
        let pads = &params.pads;

        let (offsets, buf) = old_vols
            .iter()
            .zip(vols.iter())
            .zip(Self::OFFSETS.iter())
            .enumerate()
            .filter(|(_, ((old, new), _))| !old.eq(new))
            .fold(
                (Vec::new(), Vec::new()),
                |(mut offsets, mut buf), (i, (_, &offset))| {
                    offsets.push(offset);
                    let val = build_output_parameter(mutes, vols, hwctls, dims, pads, i);
                    buf.extend_from_slice(&val.to_be_bytes());
                    (offsets, buf)
                },
            );

        if offsets.len() > 0 {
            saffire_write_quadlets(req, node, &offsets, &buf, timeout_ms)
                .map(|_| old_vols.copy_from_slice(vols))
        } else {
            Ok(())
        }
    }

    fn write_hwctls(
        req: &FwReq,
        node: &FwNode,
        hwctls: &[bool],
        params: &mut SaffireOutputParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mutes = &params.mutes;
        let vols = &params.vols;
        let old_hwctls = &mut params.hwctls;
        let dims = &params.dims;
        let pads = &params.pads;

        let (offsets, buf) = old_hwctls
            .iter()
            .zip(hwctls.iter())
            .zip(Self::OFFSETS.iter())
            .enumerate()
            .filter(|(_, ((old, new), _))| !old.eq(new))
            .fold(
                (Vec::new(), Vec::new()),
                |(mut offsets, mut buf), (i, (_, &offset))| {
                    offsets.push(offset);
                    let val = build_output_parameter(mutes, vols, hwctls, dims, pads, i);
                    buf.extend_from_slice(&val.to_be_bytes());
                    (offsets, buf)
                },
            );

        if offsets.len() > 0 {
            saffire_write_quadlets(req, node, &offsets, &buf, timeout_ms)
                .map(|_| old_hwctls.copy_from_slice(hwctls))
        } else {
            Ok(())
        }
    }

    fn write_dims(
        req: &FwReq,
        node: &FwNode,
        dims: &[bool],
        params: &mut SaffireOutputParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mutes = &params.mutes;
        let vols = &params.vols;
        let hwctls = &params.hwctls;
        let old_dims = &mut params.dims;
        let pads = &params.pads;

        let (offsets, buf) = old_dims
            .iter()
            .zip(dims.iter())
            .zip(Self::OFFSETS.iter())
            .enumerate()
            .filter(|(_, ((old, new), _))| !old.eq(new))
            .fold(
                (Vec::new(), Vec::new()),
                |(mut offsets, mut buf), (i, (_, &offset))| {
                    offsets.push(offset);
                    let val = build_output_parameter(mutes, vols, hwctls, dims, pads, i);
                    buf.extend_from_slice(&val.to_be_bytes());
                    (offsets, buf)
                },
            );

        if offsets.len() > 0 {
            saffire_write_quadlets(req, node, &offsets, &buf, timeout_ms)
                .map(|_| old_dims.copy_from_slice(dims))
        } else {
            Ok(())
        }
    }

    fn write_pads(
        req: &FwReq,
        node: &FwNode,
        pads: &[bool],
        params: &mut SaffireOutputParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mutes = &params.mutes;
        let vols = &params.vols;
        let hwctls = &params.hwctls;
        let dims = &params.dims;
        let old_pads = &mut params.pads;

        let (offsets, buf) = old_pads
            .iter()
            .zip(pads.iter())
            .zip(Self::OFFSETS.iter())
            .enumerate()
            .filter(|(_, ((old, new), _))| !old.eq(new))
            .fold(
                (Vec::new(), Vec::new()),
                |(mut offsets, mut buf), (i, (_, &offset))| {
                    offsets.push(offset);
                    let val = build_output_parameter(mutes, vols, hwctls, dims, pads, i);
                    buf.extend_from_slice(&val.to_be_bytes());
                    (offsets, buf)
                },
            );

        if offsets.len() > 0 {
            saffire_write_quadlets(req, node, &offsets, &buf, timeout_ms)
                .map(|_| old_pads.copy_from_slice(pads))
        } else {
            Ok(())
        }
    }
}

fn build_output_parameter(
    mutes: &[bool],
    vols: &[u8],
    hwctls: &[bool],
    dims: &[bool],
    pads: &[bool],
    index: usize,
) -> u32 {
    let mut val = 0u32;
    mutes
        .iter()
        .nth(index)
        .filter(|&mute| *mute)
        .map(|_| val |= MUTE_FLAG);
    vols.iter()
        .nth(index)
        .map(|&vol| val |= (0xff - vol) as u32);
    hwctls
        .iter()
        .nth(index)
        .filter(|&hwctl| *hwctl)
        .map(|_| val |= HWCTL_FLAG);
    dims.iter()
        .nth(index)
        .filter(|&dim| *dim)
        .map(|_| val |= DIM_FLAG);
    pads.iter()
        .nth(index)
        .filter(|&pad| *pad)
        .map(|_| val |= PAD_FLAG);
    val
}

pub trait SaffireStoreConfigOperation {
    const OFFSET: usize;

    fn store_config(req: &FwReq, node: &FwNode, timeout_ms: u32) -> Result<(), Error> {
        saffire_write_quadlets(req, node, &[Self::OFFSET], &1u32.to_be_bytes(), timeout_ms)
    }
}

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

const READ_OFFSET: u64 = 0x000100000000;
const WRITE_OFFSET: u64 = 0x000100010000;

/// Read single quadlet.
pub fn saffire_read_quadlet(
    req: &FwReq,
    node: &FwNode,
    offset: usize,
    buf: &mut [u8],
    timeout_ms: u32,
) -> Result<(), Error> {
    assert_eq!(buf.len(), 4);

    req.transaction_sync(
        node,
        FwTcode::ReadQuadletRequest,
        READ_OFFSET + offset as u64,
        4,
        buf,
        timeout_ms,
    )
}

/// Read batch of quadlets.
pub fn saffire_read_quadlets(
    req: &FwReq,
    node: &FwNode,
    offsets: &[usize],
    buf: &mut [u8],
    timeout_ms: u32,
) -> Result<(), Error> {
    assert_eq!(offsets.len() * 4, buf.len());

    let mut prev_offset = offsets[0];
    let mut prev_index = 0;
    let mut count = 1;
    let mut peekable = offsets.iter().peekable();

    while let Some(&offset) = peekable.next() {
        let next_offset = if let Some(&next_offset) = peekable.peek() {
            if *next_offset == offset + 4 && count < MAXIMUM_OFFSET_COUNT {
                count += 1;
                continue;
            }
            *next_offset
        } else {
            // NOTE: Just for safe.
            offsets[0]
        };

        let frame = &mut buf[prev_index..(prev_index + count * 4)];

        if count == 1 {
            saffire_read_quadlet(req, node, prev_offset, frame, timeout_ms)
        } else {
            req.transaction_sync(
                node,
                FwTcode::ReadBlockRequest,
                READ_OFFSET + prev_offset as u64,
                frame.len(),
                frame,
                timeout_ms,
            )
        }
        .map(|_| {
            prev_index += count * 4;
            prev_offset = next_offset;
            count = 1;
        })?;
    }

    Ok(())
}

/// Write single quadlet.
pub fn saffire_write_quadlet(
    req: &FwReq,
    node: &FwNode,
    offset: usize,
    buf: &[u8],
    timeout_ms: u32,
) -> Result<(), Error> {
    assert_eq!(buf.len(), 4);

    let mut frame = buf.to_vec();
    req.transaction_sync(
        node,
        FwTcode::WriteQuadletRequest,
        WRITE_OFFSET + offset as u64,
        4,
        &mut frame,
        timeout_ms,
    )
}

/// Write batch of coefficieints.
pub fn saffire_write_quadlets(
    req: &FwReq,
    node: &FwNode,
    offsets: &[usize],
    buf: &[u8],
    timeout_ms: u32,
) -> Result<(), Error> {
    assert_eq!(offsets.len() * 4, buf.len());

    if offsets.len() == 1 {
        return saffire_write_quadlet(req, node, offsets[0], buf, timeout_ms);
    }

    let mut frame = offsets
        .iter()
        .enumerate()
        .fold(Vec::new(), |mut frame, (i, &offset)| {
            frame.extend_from_slice(&((offset / 4) as u32).to_be_bytes());
            let pos = i * 4;
            frame.extend_from_slice(&buf[pos..(pos + 4)]);
            frame
        });

    req.transaction_sync(
        node,
        FwTcode::WriteBlockRequest,
        WRITE_OFFSET,
        frame.len(),
        &mut frame,
        timeout_ms,
    )
}

/// The trait for operations of AC3 and MIDI signal through.
pub trait SaffireThroughOperation {
    const MIDI_THROUGH_OFFSET: usize;
    const AC3_THROUGH_OFFSET: usize;

    fn read_midi_through(
        req: &FwReq,
        node: &FwNode,
        enable: &mut bool,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut buf = [0; 4];
        saffire_read_quadlet(req, node, Self::MIDI_THROUGH_OFFSET, &mut buf, timeout_ms)
            .map(|_| *enable = u32::from_be_bytes(buf) > 0)
    }

    fn read_ac3_through(
        req: &FwReq,
        node: &FwNode,
        enable: &mut bool,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut buf = [0; 4];
        saffire_read_quadlet(req, node, Self::AC3_THROUGH_OFFSET, &mut buf, timeout_ms)
            .map(|_| *enable = u32::from_be_bytes(buf) > 0)
    }

    fn write_midi_through(
        req: &FwReq,
        node: &FwNode,
        enable: bool,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        saffire_write_quadlet(
            req,
            node,
            Self::MIDI_THROUGH_OFFSET,
            &(enable as u32).to_be_bytes(),
            timeout_ms,
        )
    }

    fn write_ac3_through(
        req: &FwReq,
        node: &FwNode,
        enable: bool,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        saffire_write_quadlet(
            req,
            node,
            Self::AC3_THROUGH_OFFSET,
            &(enable as u32).to_be_bytes(),
            timeout_ms,
        )
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
