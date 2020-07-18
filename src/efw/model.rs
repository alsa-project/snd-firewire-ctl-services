// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::{Error, FileError};

use crate::ta1394;

use crate::card_cntr;
use card_cntr::CtlModel;

use super::transactions::EfwInfo;
use super::clk_ctl;
use super::mixer_ctl;

pub struct EfwModel {
    clk_ctl: clk_ctl::ClkCtl,
    mixer_ctl: mixer_ctl::MixerCtl,
}

impl EfwModel {
    pub fn new(data: &[u8]) -> Result<Self, Error> {
        match ta1394::config_rom::parse_entries(&data) {
            Some((v, m)) => match (v.vendor_id, m.model_id) {
                // Mackie/Loud Onyx 400F.
                (0x000ff2, 0x00400f) |
                // Mackie/Loud Onyx 1200F.
                (0x000ff2, 0x01200f) |
                // Echo Digital Audio, AudioFire 12.
                (0x001486, 0x00af12) |
                // Echo Digital Audio, AudioFire 12.
                (0x001486, 0x0af12d) |
                // Echo Digital Audio, AudioFire 12.
                (0x001486, 0x0af12a) |
                // Echo Digital Audio, AudioFire 8.
                (0x001486, 0x000af8) |
                // Echo Digital Audio, AudioFire 2.
                (0x001486, 0x000af2) |
                // Echo Digital Audio, AudioFire 4.
                (0x001486, 0x000af4) |
                // Echo Digital Audio, AudioFire 8/Pre8.
                (0x001486, 0x000af9) |
                // Gibson, Robot Interface Pack (RIP) for Robot Guitar series.
                (0x00075b, 0x00afb2) |
                // Gibson, Robot Interface Pack (RIP) for Dark Fire series.
                (0x00075b, 0x00afb9) => {
                    let model = EfwModel {
                        clk_ctl: clk_ctl::ClkCtl::new(),
                        mixer_ctl: mixer_ctl::MixerCtl::new(),
                    };
                    Ok(model)
                },
                _ => {
                    let label = "Not supported.";
                    Err(Error::new(FileError::Noent, label))
                },
            },
            None => {
                let label = "Fail to detect information of unit";
                Err(Error::new(FileError::Noent, label))
            }
        }
    }
}

impl CtlModel<hinawa::SndEfw> for EfwModel {
    fn load(&mut self, unit: &hinawa::SndEfw, card_cntr: &mut card_cntr::CardCntr)
        -> Result<(), Error> {
        let hwinfo = EfwInfo::get_hwinfo(unit)?;
        self.clk_ctl.load(&hwinfo, card_cntr)?;
        self.mixer_ctl.load(&hwinfo, card_cntr)?;
        Ok(())
    }

    fn read(&mut self, unit: &hinawa::SndEfw, elem_id: &alsactl::ElemId,
            elem_value: &mut alsactl::ElemValue)
        -> Result<bool, Error> {
        if self.clk_ctl.read(unit, elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_ctl.read(unit, elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(
        &mut self,
        unit: &hinawa::SndEfw,
        elem_id: &alsactl::ElemId,
        old: &alsactl::ElemValue,
        new: &alsactl::ElemValue,
    ) -> Result<bool, Error> {
        if self.clk_ctl.write(unit, elem_id, old, new)? {
            Ok(true)
        } else if self.mixer_ctl.write(unit, elem_id, old, new)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
