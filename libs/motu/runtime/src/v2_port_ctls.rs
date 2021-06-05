// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use hinawa::FwReq;
use hinawa::SndMotu;

use core::card_cntr::CardCntr;
use core::elem_value_accessor::ElemValueAccessor;

use super::common_proto::CommonProto;

pub struct V2PortCtl {
    has_word_bnc: bool,

    pub notified_elems: Vec<alsactl::ElemId>,
}

impl<'a> V2PortCtl {
    const WORD_OUT_MODE_NAME: &'a str = "word-out-mode";

    const WORD_OUT_MODE_LABELS: &'a [&'a str] = &[
        "Force 44.1/48.0 kHz",
        "Follow to system clock",
    ];
    const WORD_OUT_MODE_VALS: &'a [u8] = &[0x00, 0x01];

    pub fn new(_: &'a [&str], _: &'a [u8], _: bool,
               has_word_bnc: bool, _: bool, _: bool) -> Self {
        V2PortCtl{
            has_word_bnc,
            notified_elems: Vec::new(),
        }
    }

    pub fn load(&mut self, _: &SndMotu, card_cntr: &mut CardCntr)
        -> Result<(), Error>
    {
        if self.has_word_bnc {
            let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Card,
                                                       0, 0, Self::WORD_OUT_MODE_NAME, 0);
            let elem_id_list = card_cntr.add_enum_elems(&elem_id, 1, 1, Self::WORD_OUT_MODE_LABELS,
                                                        None, true)?;
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
