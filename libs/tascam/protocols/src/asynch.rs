// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocols defined for Tascam for FireWire series only with asynchronous communication.
//!
//! The module includes protocol implementation defined by Tascam for FireWire series only with
//! asynchronous communication.

pub mod fe8;

use super::*;

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
