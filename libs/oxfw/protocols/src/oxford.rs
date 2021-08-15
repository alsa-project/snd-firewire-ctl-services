// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol defined by Oxford Semiconductor, Inc for its FW970/971 ASICs.

use glib::Error;

use hinawa::{FwNode, FwReq, FwReqExtManual, FwTcode};

#[derive(Default, Debug)]
pub struct OxfordProtocol;

const CSR_REGISTER_BASE: u64 = 0xfffff0000000;
const FIRMWARE_ID_OFFSET: u64 = 0x50000;
const HARDWARE_ID_OFFSET: u64 = 0x90020;

impl OxfordProtocol {
    pub const HARDWARE_ID_IS_FW970: u32 = 0x39443841; // '9', 'D', '8', 'A'
    pub const HARDWARE_ID_IS_FW971: u32 = 0x39373100; // '9', '7', '1', '\0'

    pub fn read_firmware_id(
        req: &mut FwReq,
        node: &mut FwNode,
        firmware_id: &mut u32,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut quadlet = [0; 4];
        req.transaction_sync(
            node,
            FwTcode::ReadQuadletRequest,
            CSR_REGISTER_BASE + FIRMWARE_ID_OFFSET,
            4,
            &mut quadlet,
            timeout_ms,
        )
        .map(|_| *firmware_id = u32::from_be_bytes(quadlet))
    }

    pub fn read_hardware_id(
        req: &mut FwReq,
        node: &mut FwNode,
        hardware_id: &mut u32,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut quadlet = [0; 4];
        req.transaction_sync(
            node,
            FwTcode::ReadQuadletRequest,
            CSR_REGISTER_BASE + HARDWARE_ID_OFFSET,
            4,
            &mut quadlet,
            timeout_ms,
        )
        .map(|_| *hardware_id = u32::from_be_bytes(quadlet))
    }
}
