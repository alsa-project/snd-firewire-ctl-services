// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

use glib::Error;

use hinawa::SndMotu;

use alsactl::{ElemId, ElemIfaceType, ElemValue};

use core::card_cntr::CardCntr;
use core::elem_value_accessor::ElemValueAccessor;

use motu_protocols::*;

#[derive(Default)]
pub struct CommonPhoneCtl(pub Vec<ElemId>);

impl<'a> CommonPhoneCtl {
    const PHONE_ASSIGN_NAME: &'a str = "phone-assign";

    pub fn load<O>(&mut self, _: &O, card_cntr: &mut CardCntr) -> Result<(), Error>
    where
        for<'b> O: AssignProtocol<'b>,
    {
        let labels: Vec<String> = O::ASSIGN_PORTS.iter().map(|e| e.0.to_string()).collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::PHONE_ASSIGN_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, true)
            .map(|elem_id_list| self.0.extend_from_slice(&elem_id_list))?;

        Ok(())
    }

    pub fn read<O>(
        &mut self,
        unit: &SndMotu,
        proto: &O,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error>
    where
        for<'b> O: AssignProtocol<'b>,
    {
        match elem_id.get_name().as_str() {
            Self::PHONE_ASSIGN_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    proto
                        .get_phone_assign(unit, timeout_ms)
                        .map(|val| val as u32)
                })?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub fn write<O>(
        &mut self,
        unit: &SndMotu,
        proto: &O,
        elem_id: &ElemId,
        _: &ElemValue,
        new: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error>
    where
        for<'b> O: AssignProtocol<'b>,
    {
        match elem_id.get_name().as_str() {
            Self::PHONE_ASSIGN_NAME => {
                ElemValueAccessor::<u32>::get_val(new, |val| {
                    proto.set_phone_assign(unit, val as usize, timeout_ms)
                })?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
