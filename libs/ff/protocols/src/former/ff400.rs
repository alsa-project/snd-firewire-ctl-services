// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol defined by RME GmbH for Fireface 400.

use hinawa::FwReq;

use super::*;

/// The structure to represent unique protocol for Fireface 400.
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct Ff400Protocol(FwReq);

impl AsRef<FwReq> for Ff400Protocol {
    fn as_ref(&self) -> &FwReq {
        &self.0
    }
}

const METER_OFFSET: usize       = 0x000080100000;

const ANALOG_INPUT_COUNT: usize = 8;
const SPDIF_INPUT_COUNT: usize = 2;
const ADAT_INPUT_COUNT: usize = 8;
const STREAM_INPUT_COUNT: usize = 18;

const ANALOG_OUTPUT_COUNT: usize = 8;
const SPDIF_OUTPUT_COUNT: usize = 2;
const ADAT_OUTPUT_COUNT: usize = 8;

/// The structure to represent state of hardware meter for Fireface 400.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Ff400MeterState(FormerMeterState);

impl AsRef<FormerMeterState> for Ff400MeterState {
    fn as_ref(&self) -> &FormerMeterState {
        &self.0
    }
}

impl AsMut<FormerMeterState> for Ff400MeterState {
    fn as_mut(&mut self) -> &mut FormerMeterState {
        &mut self.0
    }
}

impl FormerMeterSpec for Ff400MeterState {
    const ANALOG_INPUT_COUNT: usize = ANALOG_INPUT_COUNT;
    const SPDIF_INPUT_COUNT: usize = SPDIF_INPUT_COUNT;
    const ADAT_INPUT_COUNT: usize = ADAT_INPUT_COUNT;
    const STREAM_INPUT_COUNT: usize = STREAM_INPUT_COUNT;

    const ANALOG_OUTPUT_COUNT: usize = ANALOG_OUTPUT_COUNT;
    const SPDIF_OUTPUT_COUNT: usize = SPDIF_OUTPUT_COUNT;
    const ADAT_OUTPUT_COUNT: usize = ADAT_OUTPUT_COUNT;
}

impl Default for Ff400MeterState {
    fn default() -> Self {
        Self(Self::create_meter_state())
    }
}

impl<T, O> RmeFfFormerMeterProtocol<T, Ff400MeterState> for O
    where T: AsRef<FwNode>,
          O: AsRef<FwReq>,
{
    const METER_OFFSET: usize = METER_OFFSET;
}
