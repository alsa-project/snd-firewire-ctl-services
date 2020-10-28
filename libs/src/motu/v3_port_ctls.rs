// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::{Error, FileError};

use hinawa::{SndUnitExt, SndMotu};
use alsactl::{ElemValueExt, ElemValueExtManual};

use crate::card_cntr::CardCntr;

use super::common_proto::CommonProto;
use super::v3_proto::V3Proto;

pub struct V3PortCtl<'a> {
    assign_labels: &'a [&'a str],
    assign_vals: &'a [u8],
    has_main_assign: bool,
    has_return_assign: bool,
    has_word_bnc: bool,
    has_opt_ifaces: bool,

    pub notified_elems: Vec<alsactl::ElemId>,
}

impl<'a> V3PortCtl<'a> {
    const PHONE_ASSIGN_NAME: &'a str = "phone-assign";
    const MAIN_ASSIGN_NAME: &'a str = "main-assign";
    const RETURN_ASSIGN_NAME: &'a str = "return-assign";
    const WORD_OUT_MODE_NAME: &'a str = "word-out-mode";
    const OPT_IFACE_IN_MODE_NAME: &'a str = "optical-iface-in-mode";
    const OPT_IFACE_OUT_MODE_NAME: &'a str = "optical-iface-out-mode";

    const WORD_OUT_MODE_LABELS: &'a [&'a str] = &[
        "Force 44.1/48.0 kHz",
        "Follow to system clock",
    ];
    const WORD_OUT_MODE_VALS: &'a [u8] = &[0x00, 0x01];

    const OPT_IFACE_MODE_LABELS: &'a [&'a str] = &[
        "None",
        "ADAT",
        "S/PDIF",
    ];

    pub fn new(assign_labels: &'a [&'a str], assign_vals: &'a [u8], has_main_assign: bool,
               has_return_assign: bool, has_opt_ifaces: bool, has_word_bnc: bool) -> Self {
        V3PortCtl{
            assign_labels,
            assign_vals,
            has_main_assign,
            has_return_assign,
            has_word_bnc,
            has_opt_ifaces,
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

        if self.has_main_assign {
            let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Mixer,
                                                       0, 0, Self::MAIN_ASSIGN_NAME, 0);
            let elem_id_list = card_cntr.add_enum_elems(&elem_id, 1, 1, &self.assign_labels, None, true)?;
            self.notified_elems.extend_from_slice(&elem_id_list);
        }

        if self.has_return_assign {
            let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Mixer,
                                                       0, 0, Self::RETURN_ASSIGN_NAME, 0);
            let elem_id_list = card_cntr.add_enum_elems(&elem_id, 1, 1, &self.assign_labels, None, true)?;
            self.notified_elems.extend_from_slice(&elem_id_list);
        }

        if self.has_word_bnc {
            let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Card,
                                                       0, 0, Self::WORD_OUT_MODE_NAME, 0);
            let elem_id_list = card_cntr.add_enum_elems(&elem_id, 1, 1,
                                                        Self::WORD_OUT_MODE_LABELS, None, true)?;
            self.notified_elems.extend_from_slice(&elem_id_list);
        }

        if self.has_opt_ifaces {
            let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Mixer,
                                                       0, 0, Self::OPT_IFACE_IN_MODE_NAME, 0);
            let _ = card_cntr.add_enum_elems(&elem_id, 1, 2, Self::OPT_IFACE_MODE_LABELS, None, true)?;

            let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Mixer,
                                                       0, 0, Self::OPT_IFACE_OUT_MODE_NAME, 0);
            let _ = card_cntr.add_enum_elems(&elem_id, 1, 2, Self::OPT_IFACE_MODE_LABELS, None, true)?;
        }

        Ok(())
    }

    fn get_opt_iface_mode(&mut self, unit: &SndMotu, req: &hinawa::FwReq, is_out: bool, is_b: bool)
        -> Result<u32, Error>
    {
        let (enabled, no_adat) = req.get_opt_iface_mode(unit, is_out, is_b)?;

        let idx = match enabled {
            false => 0,
            true => {
                match no_adat {
                    false => 1,
                    true => 2,
                }
            }
        };
        Ok(idx)
    }

    fn set_opt_iface_mode(&mut self, unit: &SndMotu, req: &hinawa::FwReq, is_out: bool, is_b: bool,
                          mode: u32)
        -> Result<(), Error>
    {
        let (enabled, no_adat) = match mode {
            0 => (false, false),
            1 => (true, false),
            2 => (true, true),
            _ => {
                let label = format!("Invalid argument for optical interface: {}", mode);
                return Err(Error::new(FileError::Nxio, &label));
            }
        };
        req.set_opt_iface_mode(unit, is_out, is_b, enabled, no_adat)
    }

    pub fn read(&mut self, unit: &SndMotu, req: &hinawa::FwReq, elem_id: &alsactl::ElemId,
            elem_value: &mut alsactl::ElemValue)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::PHONE_ASSIGN_NAME => {
                let val = req.get_phone_assign(unit, &self.assign_vals)?;
                elem_value.set_enum(&[val as u32]);
                Ok(true)
            }
            Self::MAIN_ASSIGN_NAME => {
                let val = req.get_main_assign(unit, &self.assign_vals)?;
                elem_value.set_enum(&[val as u32]);
                Ok(true)
            }
            Self::RETURN_ASSIGN_NAME => {
                let val = req.get_return_assign(unit, &self.assign_vals)?;
                elem_value.set_enum(&[val as u32]);
                Ok(true)
            }
            Self::WORD_OUT_MODE_NAME => {
                let val = req.get_word_out(unit, &Self::WORD_OUT_MODE_VALS)?;
                elem_value.set_enum(&[val as u32]);
                Ok(true)
            }
            Self::OPT_IFACE_IN_MODE_NAME => {
                let mut vals = [0;2];
                vals.iter_mut().enumerate().try_for_each(|(i, val)| {
                    *val = self.get_opt_iface_mode(unit, req, false, i > 0)?;
                    Ok(())
                })?;
                elem_value.set_enum(&vals);
                Ok(true)
            }
            Self::OPT_IFACE_OUT_MODE_NAME => {
                let mut vals = [0;2];
                vals.iter_mut().enumerate().try_for_each(|(i, val)| {
                    *val = self.get_opt_iface_mode(unit, req, true, i > 0)?;
                    Ok(())
                })?;
                elem_value.set_enum(&vals);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub fn write(&mut self, unit: &SndMotu, req: &hinawa::FwReq, elem_id: &alsactl::ElemId,
                 old: &alsactl::ElemValue, new: &alsactl::ElemValue)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::PHONE_ASSIGN_NAME => {
                let mut vals = [0];
                new.get_enum(&mut vals);
                req.set_phone_assign(unit, &self.assign_vals, vals[0] as usize)?;
                Ok(true)
            }
            Self::MAIN_ASSIGN_NAME => {
                let mut vals = [0];
                new.get_enum(&mut vals);
                req.set_main_assign(unit, &self.assign_vals, vals[0] as usize)?;
                Ok(true)
            }
            Self::RETURN_ASSIGN_NAME => {
                let mut vals = [0];
                new.get_enum(&mut vals);
                req.set_return_assign(unit, &self.assign_vals, vals[0] as usize)?;
                Ok(true)
            }
            Self::WORD_OUT_MODE_NAME => {
                let mut vals = [0];
                new.get_enum(&mut vals);
                req.set_word_out(unit, &Self::WORD_OUT_MODE_VALS, vals[0] as usize)?;
                Ok(true)
            }
            Self::OPT_IFACE_IN_MODE_NAME => {
                let mut vals = [0;4];
                old.get_enum(&mut vals[2..]);
                new.get_enum(&mut vals[..2]);
                unit.lock()?;
                let res = vals[..2].iter().zip(vals[2..].iter()).enumerate()
                    .filter(|(_, (new, old))| new != old)
                    .try_for_each(|(i, (v, _))| {
                        self.set_opt_iface_mode(unit, req, false, i > 0, *v)
                    });
                let _ = unit.unlock();
                match res {
                    Ok(()) => Ok(true),
                    Err(err) => Err(err)
                }
            }
            Self::OPT_IFACE_OUT_MODE_NAME => {
                let mut vals = [0;4];
                old.get_enum(&mut vals[2..]);
                new.get_enum(&mut vals[..2]);
                unit.lock()?;
                let res = vals[..2].iter().zip(vals[2..].iter()).enumerate()
                    .filter(|(_, (new, old))| new != old)
                    .try_for_each(|(i, (v, _))| {
                        self.set_opt_iface_mode(unit, req, true, i > 0, *v)
                    });
                let _ = unit.unlock();
                match res {
                    Ok(()) => Ok(true),
                    Err(err) => Err(err)
                }
            }
            _ => Ok(false),
        }
    }
}
