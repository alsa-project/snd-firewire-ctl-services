// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::{Error, FileError};

use core::card_cntr;
use card_cntr::{CtlModel, MeasureModel};

use ieee1212_config_rom::ConfigRom;
use ta1394::config_rom::Ta1394ConfigRom;

use super::transactions::EfwInfo;
use super::clk_ctl;
use super::mixer_ctl;
use super::output_ctl;
use super::input_ctl;
use super::port_ctl;
use super::meter_ctl;
use super::guitar_ctl;
use super::iec60958_ctl;

use std::convert::TryFrom;

pub struct EfwModel {
    clk_ctl: clk_ctl::ClkCtl,
    mixer_ctl: mixer_ctl::MixerCtl,
    output_ctl: output_ctl::OutputCtl,
    input_ctl: input_ctl::InputCtl,
    port_ctl: port_ctl::PortCtl,
    meter_ctl: meter_ctl::MeterCtl,
    guitar_ctl: guitar_ctl::GuitarCtl,
    iec60958_ctl: iec60958_ctl::Iec60958Ctl,
}

impl EfwModel {
    pub fn new(raw: &[u8]) -> Result<Self, Error> {
        let config_rom = ConfigRom::try_from(raw)
            .map_err(|e| {
                let label = format!("Malformed configuration ROM detected: {}", e);
                Error::new(FileError::Nxio, &label)
            })?;

        let (vendor, model) = config_rom.get_vendor()
            .and_then(|vendor| {
                config_rom.get_model()
                    .map(|model| (vendor, model))
            })
            .ok_or(Error::new(FileError::Nxio, "Configuration ROM is not for 1394TA standard"))?;

        match (vendor.vendor_id, model.model_id) {
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
                    output_ctl: output_ctl::OutputCtl::new(),
                    input_ctl: input_ctl::InputCtl::new(),
                    port_ctl: port_ctl::PortCtl::new(),
                    meter_ctl: meter_ctl::MeterCtl::new(),
                    guitar_ctl: guitar_ctl::GuitarCtl::new(),
                    iec60958_ctl: iec60958_ctl::Iec60958Ctl::new(),
                };
                Ok(model)
            },
            _ => {
                let label = "Not supported.";
                Err(Error::new(FileError::Noent, label))
            },
        }
    }
}

impl CtlModel<hinawa::SndEfw> for EfwModel {
    fn load(&mut self, unit: &hinawa::SndEfw, card_cntr: &mut card_cntr::CardCntr)
        -> Result<(), Error> {
        let hwinfo = EfwInfo::get_hwinfo(unit)?;
        self.clk_ctl.load(&hwinfo, card_cntr)?;
        self.mixer_ctl.load(&hwinfo, card_cntr)?;
        self.output_ctl.load(&hwinfo, card_cntr)?;
        self.input_ctl.load(unit, &hwinfo, card_cntr)?;
        self.port_ctl.load(&hwinfo, card_cntr)?;
        self.meter_ctl.load(&hwinfo, card_cntr)?;
        self.guitar_ctl.load(&hwinfo, card_cntr)?;
        self.iec60958_ctl.load(&hwinfo, card_cntr)?;
        Ok(())
    }

    fn read(&mut self, unit: &hinawa::SndEfw, elem_id: &alsactl::ElemId,
            elem_value: &mut alsactl::ElemValue)
        -> Result<bool, Error> {
        if self.clk_ctl.read(unit, elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_ctl.read(unit, elem_id, elem_value)? {
            Ok(true)
        } else if self.output_ctl.read(unit, elem_id, elem_value)? {
            Ok(true)
        } else if self.input_ctl.read(unit, elem_id, elem_value)? {
            Ok(true)
        } else if self.port_ctl.read(unit, elem_id, elem_value)? {
            Ok(true)
        } else if self.guitar_ctl.read(unit, elem_id, elem_value)? {
            Ok(true)
        } else if self.iec60958_ctl.read(unit, elem_id, elem_value)? {
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
        } else if self.output_ctl.write(unit, elem_id, old, new)? {
            Ok(true)
        } else if self.input_ctl.write(unit, elem_id, old, new)? {
            Ok(true)
        } else if self.port_ctl.write(unit, elem_id, old, new)? {
            Ok(true)
        } else if self.guitar_ctl.write(unit, elem_id, old, new)? {
            Ok(true)
        } else if self.iec60958_ctl.write(unit, elem_id, old, new)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl MeasureModel<hinawa::SndEfw> for EfwModel {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<alsactl::ElemId>) {
        elem_id_list.extend_from_slice(&self.meter_ctl.measure_elems);
    }

    fn measure_states(&mut self, unit: &hinawa::SndEfw) -> Result<(), Error> {
        self.meter_ctl.measure_states(unit)
    }

    fn measure_elem(&mut self, _: &hinawa::SndEfw, elem_id: &alsactl::ElemId,
                    elem_value: &mut alsactl::ElemValue)
        -> Result<bool, Error>
    {
        self.meter_ctl.measure_elem(elem_id, elem_value)
    }
}
