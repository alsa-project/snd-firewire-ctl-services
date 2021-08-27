// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocols defined for Tascam for FireWire series only with asynchronous communication.
//!
//! The module includes protocol implementation defined by Tascam for FireWire series only with
//! asynchronous communication.

pub mod fe8;

use hinawa::{FwRcode, FwTcode};

use crate::*;

const ENABLE_NOTIFICATION: u64 = 0x0310;
const ADDR_HIGH_OFFSET: u64 = 0x0314;
const ADDR_LOW_OFFSET: u64 = 0x0318;

/// The trait for protocol of expander model.
pub trait ExpanderOperation {
    fn register_notification_address(
        req: &mut FwReq,
        node: &mut FwNode,
        address: u64,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut addr_hi = ((address >> 32) as u32).to_be_bytes();
        write_quadlet(req, node, ADDR_HIGH_OFFSET, &mut addr_hi, timeout_ms)?;

        let mut addr_lo = ((address & 0xffffffff) as u32).to_be_bytes();
        write_quadlet(req, node, ADDR_LOW_OFFSET, &mut addr_lo, timeout_ms)
    }

    fn enable_notification(
        req: &mut FwReq,
        node: &mut FwNode,
        enable: bool,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut frames = (enable as u32).to_be_bytes();
        write_quadlet(req, node, ENABLE_NOTIFICATION, &mut frames, timeout_ms)
    }
}

/// The structure for surface image of asynchronous model.
#[derive(Default, Debug)]
pub struct AsynchSurfaceImage(pub [u32; 32]);

impl AsynchSurfaceImage {
    /// Parse notification into surface event.
    pub fn parse_notification(
        &mut self,
        events: &mut Vec<(u32, u32, u32)>,
        tcode: FwTcode,
        frame: &[u8],
    ) -> FwRcode {
        if tcode == FwTcode::WriteQuadletRequest || tcode == FwTcode::WriteBlockRequest {
            let mut quadlet = [0; 4];
            (0..frame.len()).step_by(4).for_each(|pos| {
                quadlet.copy_from_slice(&frame[pos..(pos + 4)]);
                let value = u32::from_be_bytes(quadlet);
                let index = ((value & 0x00ff0000) >> 16) as usize;
                let state = value & 0x0000ffff;
                if self.0[index] != state {
                    events.push((index as u32, self.0[index], state));
                    self.0[index] = state;
                }
            });
            FwRcode::Complete
        } else {
            FwRcode::TypeError
        }
    }
}
