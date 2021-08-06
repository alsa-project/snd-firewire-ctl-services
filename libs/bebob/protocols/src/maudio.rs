// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol implementation for M-Audio FireWire series.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by M-Audio FireWire series.

pub mod normal;
pub mod pfl;
pub mod special;

use glib::Error;

use hinawa::{FwNode, FwReq, FwReqExtManual};

const MAUDIO_OUI: [u8; 3] = [0x00, 0x0d, 0x6c];

const BASE_OFFSET: u64 = 0xffc700000000;
const METER_OFFSET: u64 = 0x00600000;

fn read_block(
    req: &FwReq,
    node: &FwNode,
    offset: u64,
    frames: &mut [u8],
    timeout_ms: u32,
) -> Result<(), Error> {
    req.transaction_sync(
        node,
        hinawa::FwTcode::ReadBlockRequest,
        BASE_OFFSET + offset,
        frames.len(),
        frames,
        timeout_ms,
    )
}
