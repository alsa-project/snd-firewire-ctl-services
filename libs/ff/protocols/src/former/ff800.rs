// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol defined by RME GmbH for Fireface 800.
use hinawa::{FwNode, FwReq};

use super::*;

/// The structure to represent unique protocol for Fireface 800.
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct Ff800Protocol(FwReq);

impl AsRef<FwReq> for Ff800Protocol {
    fn as_ref(&self) -> &FwReq {
        &self.0
    }
}


const OUTPUT_OFFSET: usize      = 0x000080081f80;
const METER_OFFSET: usize       = 0x000080100000;

const ANALOG_INPUT_COUNT: usize = 10;
const SPDIF_INPUT_COUNT: usize = 2;
const ADAT_INPUT_COUNT: usize = 16;
const STREAM_INPUT_COUNT: usize = 28;

const ANALOG_OUTPUT_COUNT: usize = 10;
const SPDIF_OUTPUT_COUNT: usize = 2;
const ADAT_OUTPUT_COUNT: usize = 16;

/// The structure to represent state of hardware meter for Fireface 400.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Ff800MeterState(FormerMeterState);

impl AsRef<FormerMeterState> for Ff800MeterState {
    fn as_ref(&self) -> &FormerMeterState {
        &self.0
    }
}

impl AsMut<FormerMeterState> for Ff800MeterState {
    fn as_mut(&mut self) -> &mut FormerMeterState {
        &mut self.0
    }
}

impl FormerMeterSpec for Ff800MeterState {
    const ANALOG_INPUT_COUNT: usize = ANALOG_INPUT_COUNT;
    const SPDIF_INPUT_COUNT: usize = SPDIF_INPUT_COUNT;
    const ADAT_INPUT_COUNT: usize = ADAT_INPUT_COUNT;
    const STREAM_INPUT_COUNT: usize = STREAM_INPUT_COUNT;

    const ANALOG_OUTPUT_COUNT: usize = ANALOG_OUTPUT_COUNT;
    const SPDIF_OUTPUT_COUNT: usize = SPDIF_OUTPUT_COUNT;
    const ADAT_OUTPUT_COUNT: usize = ADAT_OUTPUT_COUNT;
}

impl Default for Ff800MeterState {
    fn default() -> Self {
        Self(Self::create_meter_state())
    }
}

impl<T, O> RmeFfFormerMeterProtocol<T, Ff800MeterState> for O
    where T: AsRef<FwNode>,
          O: AsRef<FwReq>,
{
    const METER_OFFSET: usize = METER_OFFSET;
}

/// The structure to represent volume of outputs for Fireface 800.
///
/// The value for volume is between 0x00000000 and 0x00010000 through 0x00000001 and 0x00080000 to
/// represent the range from negative infinite to 6.00 dB through -90.30 dB and 0.00 dB.
#[derive(Default, Debug, Clone, Copy, Eq, PartialEq)]
pub struct Ff800OutputVolumeState([i32;ANALOG_OUTPUT_COUNT + SPDIF_OUTPUT_COUNT + ADAT_OUTPUT_COUNT]);

impl AsRef<[i32]> for Ff800OutputVolumeState {
    fn as_ref(&self) -> &[i32] {
        &self.0
    }
}

impl AsMut<[i32]> for Ff800OutputVolumeState {
    fn as_mut(&mut self) -> &mut [i32] {
        &mut self.0
    }
}

impl<T: AsRef<FwNode>> RmeFormerOutputProtocol<T, Ff800OutputVolumeState> for Ff800Protocol {
    fn write_output_vol(&self, node: &T, ch: usize, vol: i32, timeout_ms: u32) -> Result<(), Error> {
        let mut raw = [0;4];
        raw.copy_from_slice(&vol.to_le_bytes());
        self.as_ref().transaction_sync(node.as_ref(), FwTcode::WriteBlockRequest,
                                       (OUTPUT_OFFSET + ch * 4) as u64, raw.len(), &mut raw,
                                       timeout_ms)
    }
}
