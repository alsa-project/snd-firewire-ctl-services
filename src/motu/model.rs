// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::{Error, FileError};

use crate::card_cntr;

pub struct MotuModel {
}

impl MotuModel {
    pub fn new(model_id: u32, version: u32) -> Result<Self, Error> {
        match model_id {
            _ => {
                let label = format!("Unsupported model ID: 0x{:06x}", model_id);
                return Err(Error::new(FileError::Noent, &label));
            },
        }
    }

    pub fn load(&mut self, unit: &hinawa::SndMotu, card_cntr: &mut card_cntr::CardCntr)
        -> Result<(), Error>
    {
        Ok(())
    }

    pub fn dispatch_elem_event(&mut self, unit: &hinawa::SndMotu, card_cntr: &mut card_cntr::CardCntr,
                               elem_id: &alsactl::ElemId, events: &alsactl::ElemEventMask)
        -> Result<(), Error>
    {
        Ok(())
    }
}
