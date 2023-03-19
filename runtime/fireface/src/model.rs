// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

use {
    super::{ff400_model::*, ff800_model::*, ff802_model::*, ucx_model::*, *},
    ieee1212_config_rom::*,
    std::convert::TryFrom,
};

pub enum Model {
    Ff800(Ff800Model),
    Ff400(Ff400Model),
    Ucx(UcxModel),
    Ff802(Ff802Model),
}

pub struct FfModel {
    model: Model,
    pub measured_elem_list: Vec<alsactl::ElemId>,
}

impl FfModel {
    pub fn new(raw: &[u8]) -> Result<FfModel, Error> {
        let config_rom = ConfigRom::try_from(&raw[..]).map_err(|e| {
            let msg = format!("Malformed configuration ROM detected: {}", e);
            Error::new(FileError::Nxio, &msg)
        })?;
        let model_id = config_rom
            .get_model_id()
            .ok_or_else(|| Error::new(FileError::Nxio, "Unexpected format of configuration ROM"))?;

        let model = match model_id {
            0x00000001 => Model::Ff800(Ff800Model::default()),
            0x00000002 => Model::Ff400(Ff400Model::default()),
            0x00000004 => Model::Ucx(UcxModel::default()),
            0x00000005 => Model::Ff802(Ff802Model::default()),
            _ => Err(Error::new(FileError::Nxio, "Not supported."))?,
        };

        let measured_elem_list = Vec::new();

        Ok(FfModel {
            model,
            measured_elem_list,
        })
    }

    pub fn cache(&mut self, unit: &mut (SndUnit, FwNode)) -> Result<(), Error> {
        match &mut self.model {
            Model::Ff800(m) => m.cache(unit),
            Model::Ff400(m) => m.cache(unit),
            Model::Ucx(m) => m.cache(unit),
            Model::Ff802(m) => m.cache(unit),
        }
    }

    pub fn load(
        &mut self,
        unit: &mut (SndUnit, FwNode),
        card_cntr: &mut CardCntr,
    ) -> Result<(), Error> {
        match &mut self.model {
            Model::Ff800(m) => m.load(unit, card_cntr),
            Model::Ff400(m) => m.load(unit, card_cntr),
            Model::Ucx(m) => m.load(unit, card_cntr),
            Model::Ff802(m) => m.load(unit, card_cntr),
        }?;

        match &mut self.model {
            Model::Ff800(m) => m.get_measure_elem_list(&mut self.measured_elem_list),
            Model::Ff400(m) => m.get_measure_elem_list(&mut self.measured_elem_list),
            Model::Ucx(m) => m.get_measure_elem_list(&mut self.measured_elem_list),
            Model::Ff802(m) => m.get_measure_elem_list(&mut self.measured_elem_list),
        }

        Ok(())
    }

    pub fn dispatch_elem_event(
        &mut self,
        unit: &mut (SndUnit, FwNode),
        card_cntr: &mut CardCntr,
        elem_id: &alsactl::ElemId,
        events: &alsactl::ElemEventMask,
    ) -> Result<(), Error> {
        match &mut self.model {
            Model::Ff800(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
            Model::Ff400(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
            Model::Ucx(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
            Model::Ff802(m) => card_cntr.dispatch_elem_event(unit, &elem_id, &events, m),
        }
    }

    pub fn measure_elems(
        &mut self,
        unit: &mut (SndUnit, FwNode),
        card_cntr: &mut CardCntr,
    ) -> Result<(), Error> {
        match &mut self.model {
            Model::Ff800(m) => card_cntr.measure_elems(unit, &self.measured_elem_list, m),
            Model::Ff400(m) => card_cntr.measure_elems(unit, &self.measured_elem_list, m),
            Model::Ucx(m) => card_cntr.measure_elems(unit, &self.measured_elem_list, m),
            Model::Ff802(m) => card_cntr.measure_elems(unit, &self.measured_elem_list, m),
        }
    }
}
