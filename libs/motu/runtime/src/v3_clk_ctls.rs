// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use hinawa::FwReq;
use hinawa::{SndUnitExt, SndMotu};

use core::card_cntr::CardCntr;
use core::elem_value_accessor::ElemValueAccessor;

use super::common_proto::CommonProto;
use super::v3_proto::V3Proto;

pub struct V3ClkCtl<'a> {
    rate_labels: &'a [&'a str],
    rate_vals: &'a [u8],
    src_labels: &'a [&'a str],
    src_vals: &'a [u8],
    has_lcd: bool,
}

impl<'a> V3ClkCtl<'a> {
    const RATE_NAME: &'a str = "sampling- rate";
    const SRC_NAME: &'a str = "clock-source";

    pub fn new(rate_labels: &'a [&'a str], rate_vals: &'a [u8],
               src_labels: &'a [&'a str], src_vals: &'a [u8], has_lcd: bool) -> Self {
        V3ClkCtl{rate_labels, rate_vals, src_labels, src_vals, has_lcd}
    }

    pub fn load(&mut self, _: &SndMotu, card_cntr: &mut CardCntr)
        -> Result<(), Error>
    {
        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Card,
                                                   0, 0, Self::RATE_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &self.rate_labels, None, true)?;

        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Card,
                                                   0, 0, Self::SRC_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &self.src_labels, None, true)?;

        Ok(())
    }

    pub fn read<O>(&mut self, unit: &SndMotu, proto: &O, elem_id: &alsactl::ElemId,
                   elem_value: &mut alsactl::ElemValue)
        -> Result<bool, Error>
        where O: AsRef<FwReq>,
    {
        match elem_id.get_name().as_str() {
            Self::RATE_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let val = proto.as_ref().get_clk_rate(unit, &self.rate_vals)?;
                    Ok(val as u32)
                })?;
                Ok(true)
            }
            Self::SRC_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let val = proto.as_ref().get_clk_src(unit, &self.src_vals)?;
                    if self.has_lcd {
                        proto.as_ref().update_clk_disaplay(unit, &self.src_labels[val])?;
                    }
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
            Self::RATE_NAME => {
                ElemValueAccessor::<u32>::get_val(new, |val| {
                    unit.lock()?;
                    let res = proto.as_ref().set_clk_rate(unit, &self.rate_vals, val as usize);
                    let _ = unit.unlock();
                    res
                })?;
                Ok(true)
            }
            Self::SRC_NAME => {
                ElemValueAccessor::<u32>::get_val(new, |val| {
                    let prev_src = proto.as_ref().get_clk_src(unit, &self.src_vals)?;
                    unit.lock()?;
                    let mut res = proto.as_ref().set_clk_src(unit, &self.src_vals, val as usize);
                    if res.is_ok() && self.has_lcd {
                        res = proto.as_ref().update_clk_disaplay(unit, self.src_labels[val as usize]);
                        if res.is_err() {
                            let _ = proto.as_ref().set_clk_src(unit, &self.src_vals, prev_src);
                        }
                    }
                    let _ = unit.unlock();
                    res
                })?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
