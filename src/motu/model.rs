// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::{Error, FileError};

use crate::card_cntr::{CardCntr, CtlModel};

use super::ultralite_mk3::UltraLiteMk3;
use super::h4pre::H4pre;

pub struct MotuModel<'a> {
    firmware_version: u32,
    ctl_model: MotuCtlModel<'a>,
}

enum MotuCtlModel<'a> {
    UltraLiteMk3(UltraLiteMk3<'a>),
    H4pre(H4pre<'a>),
}

impl<'a> MotuModel<'a> {
    pub fn new(model_id: u32, version: u32) -> Result<Self, Error> {
        let ctl_model = match model_id {
            0x000019 => MotuCtlModel::UltraLiteMk3(UltraLiteMk3::new()),
            0x000045 => MotuCtlModel::H4pre(H4pre::new()),
            _ => {
                let label = format!("Unsupported model ID: 0x{:06x}", model_id);
                return Err(Error::new(FileError::Noent, &label));
            }
        };
        let model = MotuModel{
            firmware_version: version,
            ctl_model,
        };
        Ok(model)
    }

    pub fn load(&mut self, unit: &hinawa::SndMotu, card_cntr: &mut CardCntr)
        -> Result<(), Error>
    {
        match &mut self.ctl_model {
            MotuCtlModel::UltraLiteMk3(m) => m.load(unit, card_cntr),
            MotuCtlModel::H4pre(m) => m.load(unit, card_cntr),
        }
    }

    pub fn dispatch_elem_event(&mut self, unit: &hinawa::SndMotu, card_cntr: &mut CardCntr,
                               elem_id: &alsactl::ElemId, events: &alsactl::ElemEventMask)
        -> Result<(), Error>
    {
        match &mut self.ctl_model {
            MotuCtlModel::UltraLiteMk3(m) => card_cntr.dispatch_elem_event(unit, elem_id, events, m),
            MotuCtlModel::H4pre(m) => card_cntr.dispatch_elem_event(unit, elem_id, events, m),
        }
    }
}
