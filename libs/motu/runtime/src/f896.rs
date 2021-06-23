// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto
use glib::Error;

use hinawa::SndMotu;

use alsactl::{ElemId, ElemValue};

use core::card_cntr::{CardCntr, CtlModel};

use motu_protocols::version_1::*;

const TIMEOUT_MS: u32 = 100;

#[derive(Default)]
pub struct F896 {
    proto: F896Protocol,
}

impl CtlModel<SndMotu> for F896 {
    fn load(&mut self, _: &SndMotu, _: &mut CardCntr) -> Result<(), Error> {
        Ok(())
    }

    fn read(&mut self, _: &SndMotu, _: &ElemId, _: &mut ElemValue) -> Result<bool, Error> {
        Ok(false)
    }

    fn write(
        &mut self,
        _: &SndMotu,
        _: &ElemId,
        _: &ElemValue,
        _: &ElemValue,
    ) -> Result<bool, Error> {
        Ok(false)
    }
}
