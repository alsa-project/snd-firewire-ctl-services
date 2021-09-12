// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Meter protocol specific to Alesis iO FireWire series.
//!
//! The module includes structure, enumeration, and trait and its implementation for meter
//! protocol defined by Alesis for iO FireWire series.

use glib::Error;
use hinawa::FwNode;

use super::*;

/// The structure to represent hardware meters. 0..0x007fff00 with 0x100 step (-90.0..0.0 dB)
#[derive(Debug)]
pub struct IoMeter{
    pub analog_inputs: Vec<i32>,
    pub stream_inputs: [i32;8],
    pub digital_a_inputs: [i32;8],
    pub digital_b_inputs: Vec<i32>,
    pub mixer_outputs: [i32;8],
}

const STREAM_INPUT_COUNT: usize = 8;
const DIGITAL_A_INPUT_COUNT: usize = 8;
const MIXER_OUTPUT_COUNT: usize = 8;

trait IoMeterSpec {
    const ANALOG_INPUT_COUNT: usize;
    const DIGITAL_B_INPUT_COUNT: usize;

    fn create_meter() -> IoMeter {
        IoMeter{
            analog_inputs: vec![0;Self::ANALOG_INPUT_COUNT],
            stream_inputs: [0;STREAM_INPUT_COUNT],
            digital_a_inputs: [0;DIGITAL_A_INPUT_COUNT],
            digital_b_inputs: vec![0;Self::DIGITAL_B_INPUT_COUNT],
            mixer_outputs: [0;MIXER_OUTPUT_COUNT],
        }
    }
}

/// The structure to represent hardware meter for iO 14 FireWire.
#[derive(Debug)]
pub struct Io14Meter(IoMeter);

impl IoMeterSpec for Io14Meter {
    const ANALOG_INPUT_COUNT: usize = 4;
    const DIGITAL_B_INPUT_COUNT: usize = 2;
}

impl Default for Io14Meter {
    fn default() -> Self {
        Io14Meter(Self::create_meter())
    }
}

impl AsRef<IoMeter> for Io14Meter {
    fn as_ref(&self) -> &IoMeter {
        &self.0
    }
}

impl AsMut<IoMeter> for Io14Meter {
    fn as_mut(&mut self) -> &mut IoMeter {
        &mut self.0
    }
}

/// The structure to represent hardware meter for iO 26 FireWire.
#[derive(Debug)]
pub struct Io26Meter(IoMeter);

impl IoMeterSpec for Io26Meter {
    const ANALOG_INPUT_COUNT: usize = 8;
    const DIGITAL_B_INPUT_COUNT: usize = 8;
}

impl Default for Io26Meter {
    fn default() -> Self {
        Io26Meter(Self::create_meter())
    }
}

impl AsRef<IoMeter> for Io26Meter {
    fn as_ref(&self) -> &IoMeter {
        &self.0
    }
}

impl AsMut<IoMeter> for Io26Meter {
    fn as_mut(&mut self) -> &mut IoMeter {
        &mut self.0
    }
}

/// The trait to represent protofol of hardware meter.
pub trait IoMeterProtocol: AlesisIoProtocol {
    const METER_OFFSET: usize = 0x04c0;
    const SIZE: usize = 160;

    fn read_meter<M: AsMut<IoMeter>>(
        &self,
        node: &mut FwNode,
        meter: &mut M,
        timeout_ms: u32
    ) -> Result<(), Error> {
        let mut raw = vec![0;Self::SIZE];
        self.read_block(node, Self::METER_OFFSET, &mut raw, timeout_ms)
            .map(|_| {
                let m = meter.as_mut();
                let count = m.analog_inputs.len();
                m.analog_inputs.parse_quadlet_block(&raw[..(count * 4)]);
                m.stream_inputs.parse_quadlet_block(&raw[32..64]);
                m.digital_a_inputs.parse_quadlet_block(&raw[64..96]);
                let length = m.digital_b_inputs.len() * 4;
                m.digital_b_inputs.parse_quadlet_block(&raw[(128 - length)..128]);
                m.mixer_outputs.parse_quadlet_block(&raw[128..160]);
            })
    }
}

impl<O: AlesisIoProtocol> IoMeterProtocol for O {}
