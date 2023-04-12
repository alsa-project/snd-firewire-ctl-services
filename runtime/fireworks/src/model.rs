// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use {
    super::{
        audiofire12_former::*, audiofire12_later::*, audiofire2::*, audiofire4::*, audiofire8::*,
        audiofire9::*, onyx1200f::*, onyx400f::*, rip::*, *,
    },
    ieee1212_config_rom::ConfigRom,
    std::convert::TryFrom,
    ta1394_avc_general::config_rom::Ta1394ConfigRom,
};

pub(crate) enum EfwModel {
    Onyx1200f(Onyx1200f),
    Onyx400f(Onyx400f),
    Audiofire8(Audiofire8),
    Audiofire12Former(Audiofire12Former),
    Audiofire12Later(Audiofire12Later),
    Audiofire2(Audiofire2),
    Audiofire4(Audiofire4),
    Audiofire9(Audiofire9),
    Rip(Rip),
}

impl EfwModel {
    pub(crate) fn new(data: &[u8]) -> Result<Self, Error> {
        let config_rom = ConfigRom::try_from(data).map_err(|e| {
            let msg = format!("Malformed configuration ROM detected: {}", e);
            Error::new(FileError::Nxio, &msg)
        })?;

        let (vendor, model) = config_rom
            .get_vendor()
            .and_then(|vendor| config_rom.get_model().map(|model| (vendor, model)))
            .ok_or_else(|| {
                let msg = "Configuration ROM is not for 1394TA standard";
                Error::new(FileError::Nxio, &msg)
            })?;

        match (vendor.vendor_id, model.model_id) {
            (0x000ff2, 0x00400f) => Ok(Self::Onyx400f(Default::default())),
            (0x000ff2, 0x01200f) => Ok(Self::Onyx1200f(Default::default())),
            (0x001486, 0x00af12) | (0x001486, 0x0af12d) | (0x001486, 0x0af12a) => {
                // Later model is detected later.
                Ok(Self::Audiofire12Former(Default::default()))
            }
            (0x001486, 0x000af8) => Ok(Self::Audiofire8(Default::default())),
            (0x001486, 0x000af2) => Ok(Self::Audiofire2(Default::default())),
            (0x001486, 0x000af4) => Ok(Self::Audiofire4(Default::default())),
            (0x001486, 0x000af9) => Ok(Self::Audiofire9(Default::default())),
            (0x00075b, 0x00afb2) | (0x00075b, 0x00afb9) => Ok(Self::Rip(Default::default())),
            _ => Err(Error::new(FileError::Inval, "Not supported"))?,
        }
    }

    pub(crate) fn cache(&mut self, unit: &mut SndEfw) -> Result<(), Error> {
        match self {
            Self::Onyx1200f(model) => model.cache(unit),
            Self::Onyx400f(model) => model.cache(unit),
            Self::Audiofire8(model) => model.cache(unit),
            Self::Audiofire12Former(model) => {
                // Swap model instance for later model if detected.
                if model.is_later_model(unit)? {
                    let mut model = Audiofire12Later::default();
                    model
                        .cache(unit)
                        .map(|_| *self = Self::Audiofire12Later(model))
                } else {
                    model.cache(unit)
                }
            }
            // To exhaust entries, while not used actually.
            Self::Audiofire12Later(model) => model.cache(unit),
            Self::Audiofire2(model) => model.cache(unit),
            Self::Audiofire4(model) => model.cache(unit),
            Self::Audiofire9(model) => model.cache(unit),
            Self::Rip(model) => model.cache(unit),
        }
    }

    pub(crate) fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        match self {
            Self::Onyx1200f(model) => model.load(card_cntr),
            Self::Onyx400f(model) => model.load(card_cntr),
            Self::Audiofire8(model) => model.load(card_cntr),
            Self::Audiofire12Former(model) => model.load(card_cntr),
            Self::Audiofire12Later(model) => model.load(card_cntr),
            Self::Audiofire2(model) => model.load(card_cntr),
            Self::Audiofire4(model) => model.load(card_cntr),
            Self::Audiofire9(model) => model.load(card_cntr),
            Self::Rip(model) => model.load(card_cntr),
        }
    }

    pub(crate) fn get_measured_elem_id_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        match self {
            Self::Onyx1200f(model) => model.get_measure_elem_list(elem_id_list),
            Self::Onyx400f(model) => model.get_measure_elem_list(elem_id_list),
            Self::Audiofire8(model) => model.get_measure_elem_list(elem_id_list),
            Self::Audiofire12Former(model) => model.get_measure_elem_list(elem_id_list),
            Self::Audiofire12Later(model) => model.get_measure_elem_list(elem_id_list),
            Self::Audiofire2(model) => model.get_measure_elem_list(elem_id_list),
            Self::Audiofire4(model) => model.get_measure_elem_list(elem_id_list),
            Self::Audiofire9(model) => model.get_measure_elem_list(elem_id_list),
            Self::Rip(model) => model.get_measure_elem_list(elem_id_list),
        }
    }

    pub(crate) fn get_notified_elem_id_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        match self {
            Self::Onyx1200f(model) => model.get_notified_elem_list(elem_id_list),
            Self::Onyx400f(model) => model.get_notified_elem_list(elem_id_list),
            Self::Audiofire8(model) => model.get_notified_elem_list(elem_id_list),
            Self::Audiofire12Former(model) => model.get_notified_elem_list(elem_id_list),
            Self::Audiofire12Later(model) => model.get_notified_elem_list(elem_id_list),
            Self::Audiofire2(model) => model.get_notified_elem_list(elem_id_list),
            Self::Audiofire4(model) => model.get_notified_elem_list(elem_id_list),
            Self::Audiofire9(model) => model.get_notified_elem_list(elem_id_list),
            Self::Rip(model) => model.get_notified_elem_list(elem_id_list),
        }
    }

    pub(crate) fn dispatch_elem_event(
        &mut self,
        card_cntr: &mut CardCntr,
        unit: &mut SndEfw,
        elem_id: &ElemId,
        events: &ElemEventMask,
    ) -> Result<(), Error> {
        match self {
            Self::Onyx1200f(model) => card_cntr.dispatch_elem_event(unit, elem_id, events, model),
            Self::Onyx400f(model) => card_cntr.dispatch_elem_event(unit, elem_id, events, model),
            Self::Audiofire8(model) => card_cntr.dispatch_elem_event(unit, elem_id, events, model),
            Self::Audiofire12Former(model) => {
                card_cntr.dispatch_elem_event(unit, elem_id, events, model)
            }
            Self::Audiofire12Later(model) => {
                card_cntr.dispatch_elem_event(unit, elem_id, events, model)
            }
            Self::Audiofire2(model) => card_cntr.dispatch_elem_event(unit, elem_id, events, model),
            Self::Audiofire4(model) => card_cntr.dispatch_elem_event(unit, elem_id, events, model),
            Self::Audiofire9(model) => card_cntr.dispatch_elem_event(unit, elem_id, events, model),
            Self::Rip(model) => card_cntr.dispatch_elem_event(unit, elem_id, events, model),
        }
    }

    pub(crate) fn measure_elems(
        &mut self,
        card_cntr: &mut CardCntr,
        unit: &mut SndEfw,
        elem_id_list: &Vec<ElemId>,
    ) -> Result<(), Error> {
        match self {
            Self::Onyx1200f(model) => card_cntr.measure_elems(unit, elem_id_list, model),
            Self::Onyx400f(model) => card_cntr.measure_elems(unit, elem_id_list, model),
            Self::Audiofire8(model) => card_cntr.measure_elems(unit, elem_id_list, model),
            Self::Audiofire12Former(model) => card_cntr.measure_elems(unit, elem_id_list, model),
            Self::Audiofire12Later(model) => card_cntr.measure_elems(unit, elem_id_list, model),
            Self::Audiofire2(model) => card_cntr.measure_elems(unit, elem_id_list, model),
            Self::Audiofire4(model) => card_cntr.measure_elems(unit, elem_id_list, model),
            Self::Audiofire9(model) => card_cntr.measure_elems(unit, elem_id_list, model),
            Self::Rip(model) => card_cntr.measure_elems(unit, elem_id_list, model),
        }
    }

    pub(crate) fn dispatch_notification(
        &mut self,
        card_cntr: &mut CardCntr,
        unit: &mut SndEfw,
        locked: bool,
        elem_id_list: &Vec<ElemId>,
    ) -> Result<(), Error> {
        match self {
            Self::Onyx1200f(model) => {
                card_cntr.dispatch_notification(unit, &locked, elem_id_list, model)
            }
            Self::Onyx400f(model) => {
                card_cntr.dispatch_notification(unit, &locked, elem_id_list, model)
            }
            Self::Audiofire8(model) => {
                card_cntr.dispatch_notification(unit, &locked, elem_id_list, model)
            }
            Self::Audiofire12Former(model) => {
                card_cntr.dispatch_notification(unit, &locked, elem_id_list, model)
            }
            Self::Audiofire12Later(model) => {
                card_cntr.dispatch_notification(unit, &locked, elem_id_list, model)
            }
            Self::Audiofire2(model) => {
                card_cntr.dispatch_notification(unit, &locked, elem_id_list, model)
            }
            Self::Audiofire4(model) => {
                card_cntr.dispatch_notification(unit, &locked, elem_id_list, model)
            }
            Self::Audiofire9(model) => {
                card_cntr.dispatch_notification(unit, &locked, elem_id_list, model)
            }
            Self::Rip(model) => card_cntr.dispatch_notification(unit, &locked, elem_id_list, model),
        }
    }
}
