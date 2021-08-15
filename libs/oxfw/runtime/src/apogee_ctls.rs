// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use hinawa::FwFcp;
use alsactl::{ElemId, ElemIfaceType, ElemValue};

use core::card_cntr::*;
use core::elem_value_accessor::ElemValueAccessor;

use ta1394::{AvcAddr, Ta1394Avc};

use oxfw_protocols::apogee::{VendorCmd, ApogeeCmd};

const TIMEOUT_MS: u32 = 100;

#[derive(Default, Debug)]
pub struct DisplayCtl;

impl DisplayCtl {
    const TARGET_NAME: &'static str = "display-target";
    const FOLLOWED_NAME: &'static str = "meter-followed";
    const OVERHOLDS_NAME: &'static str = "overholds-duration";

    const TARGET_LABELS: [&'static str; 2] = ["output", "input"];
    const OVERHOLDS_LABELS: [&'static str; 2] = ["infinite", "2 sec"];

    pub fn load(&mut self, _: &FwFcp, card_cntr: &mut CardCntr) -> Result<(), Error> {
        // For target of display.
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::TARGET_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &Self::TARGET_LABELS, None, true)?;

        // For switch to force meters followed to selected item.
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::FOLLOWED_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        // For overholds duration.
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::OVERHOLDS_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &Self::OVERHOLDS_LABELS, None, true)?;

        Ok(())
    }

    pub fn read(&mut self, avc: &FwFcp, company_id: &[u8;3], elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::TARGET_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let mut op = ApogeeCmd::new(company_id, VendorCmd::DisplayInput);
                    avc.status(&AvcAddr::Unit, &mut op, TIMEOUT_MS)?;
                    Ok(op.get_enum())
                })?;
                Ok(true)
            }
            Self::FOLLOWED_NAME => {
                ElemValueAccessor::<bool>::set_val(elem_value, || {
                    let mut op = ApogeeCmd::new(company_id, VendorCmd::DisplayFollow);
                    avc.status(&AvcAddr::Unit, &mut op, TIMEOUT_MS)?;
                    Ok(op.get_enum() > 0)
                })?;
                Ok(true)
            }
            Self::OVERHOLDS_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let mut op = ApogeeCmd::new(company_id, VendorCmd::DisplayOverhold);
                    avc.status(&AvcAddr::Unit, &mut op, TIMEOUT_MS)?;
                    Ok(op.get_enum())
                })?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub fn write(&mut self, avc: &FwFcp, company_id: &[u8;3], elem_id: &ElemId, _: &ElemValue,
                 new: &ElemValue)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::TARGET_NAME => {
                ElemValueAccessor::<u32>::get_val(new, |val| {
                    let mut op = ApogeeCmd::new(company_id, VendorCmd::DisplayInput);
                    op.put_enum(val);
                    avc.control(&AvcAddr::Unit, &mut op, TIMEOUT_MS)
                })?;
                Ok(true)
            }
            Self::FOLLOWED_NAME => {
                ElemValueAccessor::<bool>::get_val(new, |val| {
                    let mut op = ApogeeCmd::new(company_id, VendorCmd::DisplayFollow);
                    op.put_enum(val as u32);
                    avc.control(&AvcAddr::Unit, &mut op, TIMEOUT_MS)
                })?;
                Ok(true)
            }
            Self::OVERHOLDS_NAME => {
                ElemValueAccessor::<u32>::get_val(new, |val| {
                    let mut op = ApogeeCmd::new(company_id, VendorCmd::DisplayOverhold);
                    op.put_enum(val);
                    avc.control(&AvcAddr::Unit, &mut op, TIMEOUT_MS)
                })?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
