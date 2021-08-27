// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! The implementation for protocol defined by Tascam specific to FireWire series.
//!
//! The crate includes traits, structures, and enumerations for protocol defined by Tascam specific
//! to its FireWire series.

pub mod isoch;

use glib::Error;

use hinawa::{FwNode, FwReq, FwReqExtManual, FwTcode};

const BASE_OFFSET: u64 = 0xffff00000000;

fn read_quadlet(
    req: &mut FwReq,
    node: &mut FwNode,
    offset: u64,
    frames: &mut [u8],
    timeout_ms: u32,
) -> Result<(), Error> {
    req.transaction_sync(
        node,
        FwTcode::ReadQuadletRequest,
        BASE_OFFSET + offset,
        4,
        frames,
        timeout_ms,
    )
}

fn write_quadlet(
    req: &mut FwReq,
    node: &FwNode,
    offset: u64,
    frames: &mut [u8],
    timeout_ms: u32,
) -> Result<(), Error> {
    req.transaction_sync(
        node,
        FwTcode::WriteQuadletRequest,
        BASE_OFFSET + offset,
        4,
        frames,
        timeout_ms,
    )
}
