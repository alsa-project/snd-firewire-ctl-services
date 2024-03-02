// SPDX-License-Identifier: LGPL-3.0-or-later
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

use {super::*, rand::Rng};

/// OUI registerd to IEEE for Focusrite Audio Engineering Ltd.
pub const FOCUSRITE_OUI: [u8; 3] = [0x00, 0x13, 0x0e];

/// The interface to serialize/deserialize parameters.
pub trait SaffireParametersSerdes<T: Clone> {
    /// The set of offsets for the parameters.
    const OFFSETS: &'static [usize];

    /// Change the content of raw data by the parameters.
    fn serialize(params: &T, raw: &mut [u8]);

    /// Decode the cache to change the parameter.
    fn deserialize(params: &mut T, raw: &[u8]);
}

/// The interface to operate hardware by parameters.
pub trait SaffireParametersOperation<T: Clone>: SaffireParametersSerdes<T> {
    /// Cache the state of hardware to the parameters.
    fn cache(req: &FwReq, node: &FwNode, params: &mut T, timeout_ms: u32) -> Result<(), Error> {
        let mut raw = vec![0u8; Self::OFFSETS.len() * 4];

        saffire_read_quadlets(req, node, Self::OFFSETS, &mut raw, timeout_ms)
            .map(|_| Self::deserialize(params, &raw))
    }

    /// Update the hardware for the parameters.
    fn update(
        req: &FwReq,
        node: &FwNode,
        params: &T,
        prev: &mut T,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut new = vec![0u8; Self::OFFSETS.len() * 4];
        Self::serialize(&params, &mut new);

        let mut old = vec![0u8; Self::OFFSETS.len() * 4];
        Self::serialize(&prev, &mut old);

        let mut peek_iter = Self::OFFSETS.iter().enumerate().peekable();
        let mut prev_idx = 0;

        // Detect range of continuous offsets, then read.
        while let Some((curr_idx, curr_offset)) = peek_iter.next() {
            let begin_idx = prev_idx;
            let mut next = peek_iter.peek();

            if let Some(&(next_idx, &next_offset)) = next {
                if curr_offset + 4 != next_offset || next_idx - prev_idx >= MAXIMUM_OFFSET_COUNT {
                    prev_idx = next_idx;
                    next = None;
                }
            }

            if next.is_none() {
                let end_idx = curr_idx + 1;

                let begin_pos = begin_idx * 4;
                let end_pos = end_idx * 4;

                let n = &mut new[begin_pos..end_pos];
                let o = &mut old[begin_pos..end_pos];

                if n != o {
                    saffire_write_quadlets(
                        req,
                        node,
                        &Self::OFFSETS[begin_idx..end_idx],
                        n,
                        timeout_ms,
                    )?;
                }
            }
        }

        Self::deserialize(prev, &new);

        Ok(())
    }
}

impl<O: SaffireParametersSerdes<T>, T: Clone> SaffireParametersOperation<T> for O {}

/// The structure for output parameters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaffireOutputParameters {
    /// Mute the output or not.
    pub mutes: Vec<bool>,
    /// The volume of outputs.
    pub vols: Vec<u8>,
    /// Enable hardware control for the output.
    pub hwctls: Vec<bool>,
    /// Dim the output or not.
    pub dims: Vec<bool>,
    /// Pad the output or not.
    pub pads: Vec<bool>,
}

const PAD_FLAG: u32 = 0x10000000;
const HWCTL_FLAG: u32 = 0x08000000;
//TODO: const DIRECT_FLAG: u32 = 0x04000000;
const MUTE_FLAG: u32 = 0x02000000;
const DIM_FLAG: u32 = 0x01000000;
const VOL_MASK: u32 = 0x000000ff;

/// The specification of protocol for output parameters.
pub trait SaffireOutputSpecification {
    /// The address offsets to operate for the parameters.
    const OUTPUT_OFFSETS: &'static [usize];

    /// The number of outputs accepting mute operation.
    const MUTE_COUNT: usize;

    /// The number of outputs accepting volume operation.
    const VOL_COUNT: usize;

    /// The number of outputs accepting hardware control operation.
    const HWCTL_COUNT: usize;

    /// The number of outputs accepting dim operation.
    const DIM_COUNT: usize;

    /// The number of outputs accepting pad operation.
    const PAD_COUNT: usize;
}

impl<O: SaffireOutputSpecification> SaffireParametersSerdes<SaffireOutputParameters> for O {
    const OFFSETS: &'static [usize] = O::OUTPUT_OFFSETS;

    fn serialize(params: &SaffireOutputParameters, raw: &mut [u8]) {
        (0..(raw.len() / 4)).for_each(|i| {
            let mut quadlet = 0u32;

            params
                .mutes
                .iter()
                .nth(i)
                .filter(|&mute| *mute)
                .map(|_| quadlet |= MUTE_FLAG);

            params
                .vols
                .iter()
                .nth(i)
                .map(|&vol| quadlet |= (0xff - vol) as u32);

            params
                .hwctls
                .iter()
                .nth(i)
                .filter(|&hwctl| *hwctl)
                .map(|_| quadlet |= HWCTL_FLAG);

            params
                .dims
                .iter()
                .nth(i)
                .filter(|&dim| *dim)
                .map(|_| quadlet |= DIM_FLAG);

            params
                .pads
                .iter()
                .nth(i)
                .filter(|&pad| *pad)
                .map(|_| quadlet |= PAD_FLAG);

            let pos = i * 4;
            raw[pos..(pos + 4)].copy_from_slice(&quadlet.to_be_bytes());
        })
    }

    fn deserialize(params: &mut SaffireOutputParameters, raw: &[u8]) {
        let mut quadlet = [0u8; 4];

        let quads: Vec<u32> = (0..raw.len())
            .step_by(4)
            .map(|pos| {
                quadlet.copy_from_slice(&raw[pos..(pos + 4)]);
                u32::from_be_bytes(quadlet)
            })
            .collect();

        params
            .mutes
            .iter_mut()
            .zip(&quads)
            .for_each(|(mute, &quad)| *mute = quad & MUTE_FLAG > 0);

        params
            .vols
            .iter_mut()
            .zip(&quads)
            .for_each(|(vol, &quad)| *vol = 0xff - (quad & VOL_MASK) as u8);

        params
            .hwctls
            .iter_mut()
            .zip(&quads)
            .for_each(|(hwctl, &quad)| *hwctl = quad & HWCTL_FLAG > 0);

        params
            .dims
            .iter_mut()
            .zip(&quads)
            .for_each(|(dim, &quad)| *dim = quad & DIM_FLAG > 0);

        params
            .pads
            .iter_mut()
            .zip(&quads)
            .for_each(|(pad, quad)| *pad = quad & PAD_FLAG > 0);
    }
}

/// The trait for operations of output parameters. It takes a bit time to read changed flag of
/// hwctl after finishing write transaction to change it. Use parse_hwctl argument to suppress
/// parsing the flag.
pub trait SaffireOutputOperation: SaffireOutputSpecification {
    /// The maximum value of signal level.
    const LEVEL_MIN: u8 = 0x00;
    /// The minimum value of signal level.
    const LEVEL_MAX: u8 = 0xff;
    /// The step value of signal level.
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
}

impl<O: SaffireOutputSpecification> SaffireOutputOperation for O {}

/// The structure for signal through parameters.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct SaffireThroughParameters {
    /// For MIDI inputs.
    pub midi: bool,
    /// For AC3 inputs.
    pub ac3: bool,
}

/// The specification of protocol for signal through function.
pub trait SaffireThroughSpecification {
    const THROUGH_OFFSETS: &'static [usize];
}

impl<O: SaffireThroughSpecification> SaffireParametersSerdes<SaffireThroughParameters> for O {
    const OFFSETS: &'static [usize] = O::THROUGH_OFFSETS;

    fn serialize(params: &SaffireThroughParameters, raw: &mut [u8]) {
        raw[..4].copy_from_slice(&(params.midi as u32).to_be_bytes());
        raw[4..8].copy_from_slice(&(params.ac3 as u32).to_be_bytes());
    }

    fn deserialize(params: &mut SaffireThroughParameters, raw: &[u8]) {
        let mut quadlet = [0u8; 4];

        quadlet.copy_from_slice(&raw[..4]);
        params.midi = u32::from_be_bytes(quadlet) > 0;

        quadlet.copy_from_slice(&raw[4..8]);
        params.ac3 = u32::from_be_bytes(quadlet) > 0;
    }
}

/// The parameters of configuration save.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct SaffireStoreConfigParameters;

/// The specification of configuration save.
pub trait SaffireStoreConfigSpecification {
    const STORE_CONFIG_OFFSETS: &'static [usize];
}

impl<O: SaffireStoreConfigSpecification> SaffireParametersSerdes<SaffireStoreConfigParameters>
    for O
{
    const OFFSETS: &'static [usize] = O::STORE_CONFIG_OFFSETS;

    fn serialize(_: &SaffireStoreConfigParameters, raw: &mut [u8]) {
        // NOTE: ensure to update the hardware every time to be requested.
        let mut rng = rand::thread_rng();
        raw.copy_from_slice(&rng.gen::<u32>().to_be_bytes());
    }

    fn deserialize(_: &mut SaffireStoreConfigParameters, _: &[u8]) {
        // Nothing to do.
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
    fn build_operands(&mut self, addr: &AvcAddr) -> Result<Vec<u8>, AvcCmdBuildError> {
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
        AvcControl::build_operands(&mut self.op, addr)
    }

    fn parse_operands(&mut self, addr: &AvcAddr, operands: &[u8]) -> Result<(), AvcRespParseError> {
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
    fn build_operands(&mut self, addr: &AvcAddr) -> Result<Vec<u8>, AvcCmdBuildError> {
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
        AvcStatus::build_operands(&mut self.op, addr)
    }

    fn parse_operands(&mut self, addr: &AvcAddr, operands: &[u8]) -> Result<(), AvcRespParseError> {
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

/// Read batch of quadlets.
pub fn saffire_read_quadlets(
    req: &FwReq,
    node: &FwNode,
    offsets: &[usize],
    raw: &mut [u8],
    timeout_ms: u32,
) -> Result<(), Error> {
    assert_eq!(offsets.len() * 4, raw.len());

    let mut peek_iter = offsets.iter().enumerate().peekable();
    let mut prev_idx = 0;

    // Detect range of continuous offsets, then read.
    while let Some((curr_idx, curr_offset)) = peek_iter.next() {
        let begin_idx = prev_idx;
        let mut next = peek_iter.peek();

        if let Some(&(next_idx, &next_offset)) = next {
            if curr_offset + 4 != next_offset {
                prev_idx = next_idx;
                next = None;
            }
        }

        if next.is_none() {
            let end_idx = curr_idx + 1;

            let begin_pos = begin_idx * 4;
            let end_pos = end_idx * 4;
            let r = &mut raw[begin_pos..end_pos];

            let tcode = if r.len() == 4 {
                FwTcode::ReadQuadletRequest
            } else {
                FwTcode::ReadBlockRequest
            };

            req.transaction(
                node,
                tcode,
                READ_OFFSET + offsets[begin_idx] as u64,
                r.len(),
                r,
                timeout_ms,
            )?;
        }
    }

    Ok(())
}

fn build_frame_for_write(offsets: &[usize], raw: &[u8]) -> Vec<u8> {
    assert_eq!(offsets.len() * 4, raw.len());

    offsets
        .iter()
        .enumerate()
        .fold(Vec::new(), |mut frame, (i, &offset)| {
            frame.extend_from_slice(&((offset / 4) as u32).to_be_bytes());
            let pos = i * 4;
            frame.extend_from_slice(&raw[pos..(pos + 4)]);
            frame
        })
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
    req.transaction(
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

    let mut frame = build_frame_for_write(offsets, buf);

    req.transaction(
        node,
        FwTcode::WriteBlockRequest,
        WRITE_OFFSET,
        frame.len(),
        &mut frame,
        timeout_ms,
    )
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn frame_for_write() {
        let offsets = [4, 64, 100, 256];
        let raw = [
            193, 55, 77, 70, 113, 5, 39, 141, 62, 68, 38, 80, 87, 130, 82, 133,
        ];

        let frame = build_frame_for_write(&offsets, &raw);

        offsets.iter().enumerate().for_each(|(i, &offset)| {
            assert_eq!(
                &frame[(i * 8)..(i * 8 + 4)],
                &((offset / 4) as u32).to_be_bytes()
            );
            assert_eq!(&frame[(i * 8 + 4)..(i * 8 + 8)], &raw[(i * 4)..(i * 4 + 4)]);
        });
    }

    #[test]
    fn vendor_dependent_control_operands() {
        let mut op = SaffireAvcOperation {
            offsets: vec![0x40, 0x400],
            buf: vec![0x01, 0x23, 0x45, 0x67, 0x76, 0x54, 0x32, 0x10],
            ..Default::default()
        };
        let generated = AvcControl::build_operands(&mut op, &AvcAddr::Unit).unwrap();

        let expected = [
            0x00, 0x13, 0x0e, 0x01, 0x02, 0x00, 0x00, 0x00, 0x10, 0x01, 0x23, 0x45, 0x67, 0x00,
            0x00, 0x01, 0x00, 0x76, 0x54, 0x32, 0x10,
        ];
        assert_eq!(&generated, &expected);

        let resp = [
            0x09, 0xff, 0x00, 0x00, 0x13, 0x0e, 0x01, 0x02, 0x00, 0x00, 0x00, 0x10, 0x76, 0x54,
            0x32, 0x10, 0x00, 0x00, 0x01, 0x00, 0x01, 0x23, 0x45, 0x67,
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
            0x00, 0x13, 0x0e, 0x03, 0x02, 0x00, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00, 0xff, 0x00,
            0x00, 0x01, 0x00, 0x00, 0xff, 0x00, 0xff,
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
        let generated = AvcStatus::build_operands(&mut op, &AvcAddr::Unit).unwrap();

        let expected = [
            0x00, 0x13, 0x0e, 0x03, 0x02, 0x00, 0x00, 0x00, 0x10, 0xff, 0xff, 0xff, 0xff, 0x00,
            0x00, 0x01, 0x00, 0xff, 0xff, 0xff, 0xff,
        ];
        assert_eq!(&generated, &expected);
    }
}
