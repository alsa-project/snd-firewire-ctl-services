// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::{Error, FileError};

use hinawa::SndUnitExt;

use core::card_cntr;
use core::elem_value_accessor::ElemValueAccessor;

use super::protocol::OpticalProtocol;

pub struct OpticalCtl<'a> {
    out_src_labels: &'a [&'a str],
}

impl<'a> OpticalCtl<'a> {
    const OPT_OUT_SRC_NAME: &'a str = "opt-output-source";

    const SPDIF_IN_SRC_NAME: &'a str = "spdif-input-source";

    const SPDIF_IN_SRC_LABELS: &'a [&'a str] = &["Coaxial", "Optical"];

    pub fn new(out_src_labels: &'a [&'a str]) -> Self {
        OpticalCtl {
            out_src_labels: out_src_labels,
        }
    }

    pub fn load(
        &mut self,
        _: &hinawa::SndTscm,
        _: &hinawa::FwReq,
        card_cntr: &mut card_cntr::CardCntr,
    ) -> Result<(), Error> {
        // For the source of output to optical interface.
        let elem_id = alsactl::ElemId::new_by_name(
            alsactl::ElemIfaceType::Mixer,
            0,
            0,
            Self::OPT_OUT_SRC_NAME,
            0,
        );
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, self.out_src_labels, None, true)?;

        // For interface of S/PDIF input.
        let elem_id = alsactl::ElemId::new_by_name(
            alsactl::ElemIfaceType::Mixer,
            0,
            0,
            Self::SPDIF_IN_SRC_NAME,
            0,
        );
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, Self::SPDIF_IN_SRC_LABELS, None, true)?;

        Ok(())
    }

    pub fn read(
        &mut self,
        unit: &hinawa::SndTscm,
        req: &hinawa::FwReq,
        elem_id: &alsactl::ElemId,
        elem_value: &mut alsactl::ElemValue,
    ) -> Result<bool, Error> {
        let node = unit.get_node();

        match elem_id.get_name().as_str() {
            Self::OPT_OUT_SRC_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let index = req.get_opt_out_src(&node)?;
                    Ok(index as u32)
                })?;
                Ok(true)
            }
            Self::SPDIF_IN_SRC_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let index = req.get_spdif_in_src(&node)?;
                    Ok(index as u32)
                })?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub fn write(
        &mut self,
        unit: &hinawa::SndTscm,
        req: &hinawa::FwReq,
        elem_id: &alsactl::ElemId,
        _: &alsactl::ElemValue,
        new: &alsactl::ElemValue,
    ) -> Result<bool, Error> {
        let node = unit.get_node();

        match elem_id.get_name().as_str() {
            Self::OPT_OUT_SRC_NAME => {
                ElemValueAccessor::<u32>::get_val(new, |val| {
                    let index = val as usize;
                    if index <= self.out_src_labels.len() {
                        req.set_opt_out_src(&node, index)
                    } else {
                        let label = "Invalid value of source of opticao output";
                        Err(Error::new(FileError::Inval, &label))
                    }
                })?;
                Ok(true)
            }
            Self::SPDIF_IN_SRC_NAME => {
                ElemValueAccessor::<u32>::get_val(new, |val| {
                    let index = val as usize;
                    if index <= Self::SPDIF_IN_SRC_LABELS.len() {
                        req.set_spdif_in_src(&node, index)
                    } else {
                        let label = "Invalid value for source of S/PDIF input";
                        Err(Error::new(FileError::Inval, &label))
                    }
                })?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
