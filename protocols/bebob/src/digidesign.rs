// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol implementation for Digidesign Mbox 2 Pro.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by Digidesign for Mbox 2 pro.
//!
//! ## Diagram of internal signal flow
//!
//! ```text
//! digital-input-1/2 -------------------------> stream-output-1/2
//! analog-input-1/2 ------------+-------------> stream-output-3/4
//! analog-input-3/4 ------------|-+-----------> stream-output-5/6
//!                              | |
//!                              v v
//!                          ++=======++
//!                     +--> || 6 x 2 ||-------> monitor-output-1/2
//!                     |    || mixer ||
//!                     |    ++=======++
//! stream-input-1/2 ---+----------------------> digital-output-1/2
//! stream-input-3/4 --------------------------> analog-output-1/2
//! stream-input-5/6 --------------------------> analog-output-3/4
//! ```
//!
//! None of the above audio signals is configurable by software.
//!
//! The protocol implementation for M-Audio FireWire 1814 was written with firmware version
//! below:
//!
//! ```sh
//! $ cargo run --bin bco-bootloader-info -- /dev/fw1
//! protocol:
//!   version: 1
//! bootloader:
//!   timestamp: 2005-12-07T08:55:54+0000
//!   version: 0.0.0
//! hardware:
//!   GUID: 0x00a9000000a07e01
//!   model ID: 0x000001
//!   revision: 0.0.1
//! software:
//!   timestamp: 2007-10-31T03:44:02+0000
//!   ID: 0x000000a9
//!   revision: 0.255.65535
//! image:
//!   base address: 0x20080000
//!   maximum size: 0x180000
//! ```

use super::*;

/// The protocol implementation of operation for media clock and sampling clock.
#[derive(Default, Debug)]
pub struct Mbox2proClkProtocol;

impl MediaClockFrequencyOperation for Mbox2proClkProtocol {
    const FREQ_LIST: &'static [u32] = &[44100, 48000, 88200, 96000];
}

impl SamplingClockSourceOperation for Mbox2proClkProtocol {
    const DST: SignalAddr = SignalAddr::Subunit(SignalSubunitAddr {
        subunit: MUSIC_SUBUNIT_0,
        plug_id: 0x01,
    });

    const SRC_LIST: &'static [SignalAddr] = &[
        // Internal.
        SignalAddr::Subunit(SignalSubunitAddr {
            subunit: MUSIC_SUBUNIT_0,
            plug_id: 0x01,
        }),
        // Internal with S/PDIF output.
        SignalAddr::Subunit(SignalSubunitAddr {
            subunit: MUSIC_SUBUNIT_0,
            plug_id: 0x07,
        }),
        // S/PDIF input in coaxial interface.
        SignalAddr::Unit(SignalUnitAddr::Ext(0x03)),
        // Word clock input in BNC interface.
        SignalAddr::Unit(SignalUnitAddr::Ext(0x04)),
        // Word clock input or S/PDIF input.
        SignalAddr::Unit(SignalUnitAddr::Ext(0x05)),
    ];
}

/// The protocol implementation to initialize input/output.
#[derive(Default, Debug)]
pub struct Mbox2proIoProtocol;

impl Mbox2proIoProtocol {
    // This takes the unit to process audio signal from stream-input-1/2.
    pub fn init(req: &FwReq, node: &FwNode, timeout_ms: u32) -> Result<(), Error> {
        let mut frame = [0; 12];
        frame[0] = 1;
        req.transaction(
            node,
            FwTcode::WriteBlockRequest,
            DM_APPL_PARAM_OFFSET,
            frame.len(),
            &mut frame,
            timeout_ms,
        )
    }
}
