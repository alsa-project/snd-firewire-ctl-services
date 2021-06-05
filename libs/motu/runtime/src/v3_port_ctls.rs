// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use hinawa::FwReq;
use hinawa::SndMotu;

use core::card_cntr::CardCntr;
use core::elem_value_accessor::ElemValueAccessor;

use super::common_proto::CommonProto;

pub struct V3PortCtl<'a> {
    assign_labels: &'a [&'a str],
    assign_vals: &'a [u8],
    has_word_bnc: bool,

    pub notified_elems: Vec<alsactl::ElemId>,
}

impl<'a> V3PortCtl<'a> {
    const PHONE_ASSIGN_NAME: &'a str = "phone-assign";
    const WORD_OUT_MODE_NAME: &'a str = "word-out-mode";

    const WORD_OUT_MODE_LABELS: &'a [&'a str] = &[
        "Force 44.1/48.0 kHz",
        "Follow to system clock",
    ];
    const WORD_OUT_MODE_VALS: &'a [u8] = &[0x00, 0x01];

    pub fn new(assign_labels: &'a [&'a str], assign_vals: &'a [u8], _: bool,
               _: bool, _: bool, has_word_bnc: bool) -> Self {
        V3PortCtl{
            assign_labels,
            assign_vals,
            has_word_bnc,
            notified_elems: Vec::new(),
        }
    }

    pub fn load(&mut self, _: &SndMotu, card_cntr: &mut CardCntr)
        -> Result<(), Error>
    {
        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Mixer,
                                                   0, 0, Self::PHONE_ASSIGN_NAME, 0);
        let elem_id_list = card_cntr.add_enum_elems(&elem_id, 1, 1, &self.assign_labels, None, true)?;
        self.notified_elems.extend_from_slice(&elem_id_list);

        if self.has_word_bnc {
            let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Card,
                                                       0, 0, Self::WORD_OUT_MODE_NAME, 0);
            let elem_id_list = card_cntr.add_enum_elems(&elem_id, 1, 1,
                                                        Self::WORD_OUT_MODE_LABELS, None, true)?;
            self.notified_elems.extend_from_slice(&elem_id_list);
        }

        Ok(())
    }

    pub fn read<O>(&mut self, unit: &SndMotu, proto: &O, elem_id: &alsactl::ElemId,
                   elem_value: &mut alsactl::ElemValue)
        -> Result<bool, Error>
        where O: AsRef<FwReq>,
    {
        match elem_id.get_name().as_str() {
            Self::PHONE_ASSIGN_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let val = proto.as_ref().get_phone_assign(unit, &self.assign_vals)?;
                    Ok(val as u32)
                })?;
                Ok(true)
            }
            Self::WORD_OUT_MODE_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let val = proto.as_ref().get_word_out(unit, &Self::WORD_OUT_MODE_VALS)?;
                    Ok(val as u32)
                })?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub fn write<O>(&mut self, unit: &SndMotu, proto: &O, elem_id: &alsactl::ElemId,
                    _: &alsactl::ElemValue, new: &alsactl::ElemValue)
        -> Result<bool, Error>
        where O: AsRef<FwReq>,
    {
        match elem_id.get_name().as_str() {
            Self::PHONE_ASSIGN_NAME => {
                ElemValueAccessor::<u32>::get_val(new, |val| {
                    proto.as_ref().set_phone_assign(unit, &self.assign_vals, val as usize)
                })?;
                Ok(true)
            }
            Self::WORD_OUT_MODE_NAME => {
                ElemValueAccessor::<u32>::get_val(new, |val| {
                    proto.as_ref().set_word_out(unit, &Self::WORD_OUT_MODE_VALS, val as usize)
                })?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
