// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use glib::Error;

use hinawa::{FwReq, FwReqExtManual};
use hinawa::SndUnitExt;

pub trait CommonProto: FwReqExtManual {
    const BASE_ADDR: u64 = 0xffc700700000;
    const METER_ADDR: u64 = 0xffc700600000;

    const TIMEOUT: u32 = 100;

    fn read_meters(&self, unit: &hinawa::SndUnit, frames: &mut [u8]) -> Result<(), Error> {
        self.transaction_sync(&unit.get_node(), hinawa::FwTcode::ReadBlockRequest,
                              Self::METER_ADDR, frames.len(), frames, Self::TIMEOUT)
    }

    fn write_block(&self, unit: &hinawa::SndUnit, offset: u64, frames: &mut [u8]) -> Result<(), Error> {
        self.transaction_sync(&unit.get_node(), hinawa::FwTcode::WriteBlockRequest,
                              Self::BASE_ADDR + offset, frames.len(), frames, Self::TIMEOUT)
    }
}

impl CommonProto for FwReq {}
