// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use {
    super::*, ieee1212_config_rom::ConfigRom, protocols::hw_info::*, std::convert::TryFrom,
    ta1394_avc_general::config_rom::Ta1394ConfigRom,
};

const TIMEOUT_MS: u32 = 100;

#[derive(Default)]
pub struct EfwModel {
    hw_info: HwInfo,
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
        let config_rom = ConfigRom::try_from(raw).map_err(|e| {
            let label = format!("Malformed configuration ROM detected: {}", e);
            Error::new(FileError::Nxio, &label)
        })?;

        let (vendor, model) = config_rom
            .get_vendor()
            .and_then(|vendor| config_rom.get_model().map(|model| (vendor, model)))
            .ok_or(Error::new(
                FileError::Nxio,
                "Configuration ROM is not for 1394TA standard",
            ))?;

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
                Ok(Default::default())
            },
            _ => {
                let label = "Not supported.";
                Err(Error::new(FileError::Noent, label))
            },
        }
    }
}

impl CtlModel<SndEfw> for EfwModel {
    fn load(&mut self, unit: &mut SndEfw, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.hw_info = HwInfo::default();
        unit.get_hw_info(&mut self.hw_info, TIMEOUT_MS)?;
        self.clk_ctl
            .load(&self.hw_info, card_cntr, unit, TIMEOUT_MS)?;
        self.mixer_ctl
            .load(&self.hw_info, unit, card_cntr, TIMEOUT_MS)?;
        self.output_ctl
            .load(&self.hw_info, unit, card_cntr, TIMEOUT_MS)?;
        self.input_ctl
            .load(unit, &self.hw_info, card_cntr, TIMEOUT_MS)?;
        self.port_ctl.load(
            &self.hw_info,
            card_cntr,
            unit,
            self.clk_ctl.params.rate,
            TIMEOUT_MS,
        )?;
        self.meter_ctl.load(&self.hw_info, card_cntr)?;
        self.guitar_ctl.load(&self.hw_info, card_cntr)?;
        self.iec60958_ctl.load(&self.hw_info, card_cntr)?;
        Ok(())
    }

    fn read(
        &mut self,
        unit: &mut SndEfw,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.clk_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_ctl.read(unit, elem_id, elem_value, TIMEOUT_MS)? {
            Ok(true)
        } else if self.output_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.input_ctl.read(unit, elem_id, elem_value, TIMEOUT_MS)? {
            Ok(true)
        } else if self.port_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self
            .guitar_ctl
            .read(unit, elem_id, elem_value, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .iec60958_ctl
            .read(unit, elem_id, elem_value, TIMEOUT_MS)?
        {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(
        &mut self,
        unit: &mut SndEfw,
        elem_id: &ElemId,
        old: &ElemValue,
        new: &ElemValue,
    ) -> Result<bool, Error> {
        if self.clk_ctl.write(unit, elem_id, old, new, TIMEOUT_MS)? {
            Ok(true)
        } else if self.mixer_ctl.write(unit, elem_id, new, TIMEOUT_MS)? {
            Ok(true)
        } else if self.output_ctl.write(unit, elem_id, new, TIMEOUT_MS)? {
            Ok(true)
        } else if self.input_ctl.write(unit, elem_id, old, new, TIMEOUT_MS)? {
            Ok(true)
        } else if self.port_ctl.write(unit, elem_id, old, new, TIMEOUT_MS)? {
            Ok(true)
        } else if self.guitar_ctl.write(unit, elem_id, old, new, TIMEOUT_MS)? {
            Ok(true)
        } else if self
            .iec60958_ctl
            .write(unit, elem_id, old, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl MeasureModel<SndEfw> for EfwModel {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.meter_ctl.measure_elems);
    }

    fn measure_states(&mut self, unit: &mut SndEfw) -> Result<(), Error> {
        self.meter_ctl.measure_states(unit, TIMEOUT_MS)
    }

    fn measure_elem(
        &mut self,
        _: &SndEfw,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        self.meter_ctl.measure_elem(elem_id, elem_value)
    }
}

impl NotifyModel<SndEfw, bool> for EfwModel {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.clk_ctl.notified_elem_id_list);
        elem_id_list.extend_from_slice(&self.port_ctl.notified_elem_id_list);
    }

    fn parse_notification(&mut self, unit: &mut SndEfw, &locked: &bool) -> Result<(), Error> {
        if locked {
            self.clk_ctl.cache(unit, TIMEOUT_MS)?;
            self.port_ctl
                .cache(&self.hw_info, unit, self.clk_ctl.params.rate, TIMEOUT_MS)?;
        }
        Ok(())
    }

    fn read_notified_elem(
        &mut self,
        _: &SndEfw,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.clk_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.port_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
