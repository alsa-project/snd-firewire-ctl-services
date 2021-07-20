// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol implementation for Inspire 1394.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by PreSonus for Inspire 1394.

use hinawa::{FwNode, FwReq, FwReqExtManual, FwTcode};

use crate::*;

/// The protocol implementation of clock operation.
#[derive(Default)]
pub struct Inspire1394ClkProtocol;

impl MediaClockFrequencyOperation for Inspire1394ClkProtocol {
    const FREQ_LIST: &'static [u32] = &[44100, 48000, 88200, 96000];
}

impl SamplingClockSourceOperation for Inspire1394ClkProtocol {
    const DST: SignalAddr = SignalAddr::Subunit(SignalSubunitAddr{
        subunit: MUSIC_SUBUNIT_0,
        plug_id: 0x03,
    });

    const SRC_LIST: &'static [SignalAddr] = &[
        // Internal.
        SignalAddr::Subunit(SignalSubunitAddr{subunit: MUSIC_SUBUNIT_0, plug_id: 0x02}),
    ];
}

/// The protocol implementation of meter information.
#[derive(Default)]
pub struct Inspire1394MeterProtocol;

impl Inspire1394MeterOperation for Inspire1394MeterProtocol {}

const METER_FRAME_SIZE: usize = 32;

/// The structure of meter information for Inspire 1394.
#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
pub struct Inspire1394Meter {
    pub phys_inputs: [i32; 4],
    pub stream_inputs: [i32; 2],
    pub phys_outputs: [i32; 2],
    frame: [u8; METER_FRAME_SIZE],
}

const BASE_OFFSET: u64 = 0xffc700000000;
const METER_OFFSET: u64 = 0x00600000;

/// The trait for meter information operation.
pub trait Inspire1394MeterOperation {
    const LEVEL_MIN: i32 = 0;
    const LEVEL_MAX: i32 = 0x07ffffff;
    const LEVEL_STEP: i32 = 0x100;

    fn read_meter(
        req: &FwReq,
        node: &FwNode,
        meter: &mut Inspire1394Meter,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let frame = &mut meter.frame;
        req.transaction_sync(node, FwTcode::ReadBlockRequest, BASE_OFFSET + METER_OFFSET,
                             METER_FRAME_SIZE, frame, timeout_ms)?;

        let mut quadlet = [0u8;4];
        meter.phys_inputs.iter_mut()
            .chain(meter.stream_inputs.iter_mut())
            .chain(meter.phys_outputs.iter_mut())
            .enumerate()
            .for_each(|(i, m)| {
                let pos = i * 4;
                quadlet.copy_from_slice(&frame[pos..(pos + 4)]);
                *m = i32::from_be_bytes(quadlet);
            });

        Ok(())
    }
}
