// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Meter protocol specific to Alesis iO FireWire series.
//!
//! The module includes structure, enumeration, and trait and its implementation for meter
//! protocol defined by Alesis for iO FireWire series.

use super::*;

/// For hardware meters. 0..0x007fff00 with 0x100 step (-90.0..0.0 dB)
#[derive(Default, Debug)]
pub struct IofwMeterState {
    pub analog_inputs: Vec<i32>,
    pub stream_inputs: [i32; 8],
    pub digital_a_inputs: [i32; 8],
    pub digital_b_inputs: Vec<i32>,
    pub mixer_outputs: [i32; 8],
}

const STREAM_INPUT_COUNT: usize = 8;
const DIGITAL_A_INPUT_COUNT: usize = 8;
const MIXER_OUTPUT_COUNT: usize = 8;

/// Protofol of hardware meter.
pub trait IofwMeterOperation {
    const ANALOG_INPUT_COUNT: usize;
    const DIGITAL_B_INPUT_COUNT: usize;

    const STREAM_INPUT_COUNT: usize = 8;
    const DIGITAL_A_INPUT_COUNT: usize = 8;
    const MIXER_COUNT: usize = 8;

    const LEVEL_MIN: i32 = 0;
    const LEVEL_MAX: i32 = 0x007fff00;
    const LEVEL_STEP: i32 = 0x100;

    fn create_meter_state() -> IofwMeterState {
        IofwMeterState {
            analog_inputs: vec![0; Self::ANALOG_INPUT_COUNT],
            stream_inputs: [0; STREAM_INPUT_COUNT],
            digital_a_inputs: [0; DIGITAL_A_INPUT_COUNT],
            digital_b_inputs: vec![0; Self::DIGITAL_B_INPUT_COUNT],
            mixer_outputs: [0; MIXER_OUTPUT_COUNT],
        }
    }
    fn read_meter(
        req: &mut FwReq,
        node: &mut FwNode,
        state: &mut IofwMeterState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut raw = vec![0; METER_SIZE];
        alesis_read_block(req, node, METER_OFFSET, &mut raw, timeout_ms).map(|_| {
            let count = state.analog_inputs.len();
            state.analog_inputs.parse_quadlet_block(&raw[..(count * 4)]);
            state.stream_inputs.parse_quadlet_block(&raw[32..64]);
            state.digital_a_inputs.parse_quadlet_block(&raw[64..96]);
            let length = state.digital_b_inputs.len() * 4;
            state
                .digital_b_inputs
                .parse_quadlet_block(&raw[(128 - length)..128]);
            state.mixer_outputs.parse_quadlet_block(&raw[128..160]);
        })
    }
}
