// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol defined by Oxford Semiconductor, Inc for its FW970/971 ASICs.

use super::*;

const CSR_REGISTER_BASE: u64 = 0xfffff0000000;
const FIRMWARE_ID_OFFSET: u64 = 0x50000;
const HARDWARE_ID_OFFSET: u64 = 0x90020;

/// Operation specific to Oxford ASICs.
pub trait OxfordOperation {
    /// The numeric identifier of hardware with FW970 ASIC.
    const HARDWARE_ID_IS_FW970: u32 = 0x39443841; // '9', 'D', '8', 'A'
                                                  //
    /// The numeric identifier of hardware with FW971 ASIC.
    const HARDWARE_ID_IS_FW971: u32 = 0x39373100; // '9', '7', '1', '\0'

    /// Read numeric identifier of firmware.
    fn read_firmware_id(
        req: &FwReq,
        node: &FwNode,
        firmware_id: &mut u32,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut quadlet = [0; 4];
        req.transaction(
            node,
            FwTcode::ReadQuadletRequest,
            CSR_REGISTER_BASE + FIRMWARE_ID_OFFSET,
            4,
            &mut quadlet,
            timeout_ms,
        )
        .map(|_| *firmware_id = u32::from_be_bytes(quadlet))
    }

    /// Read numeric identifier of hardware.
    fn read_hardware_id(
        req: &FwReq,
        node: &FwNode,
        hardware_id: &mut u32,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut quadlet = [0; 4];
        req.transaction(
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
