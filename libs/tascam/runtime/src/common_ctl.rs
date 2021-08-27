// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::{Error, FileError};

use hinawa::SndUnitExt;

use core::card_cntr;
use core::elem_value_accessor::ElemValueAccessor;

use super::protocol::{ClkSrc, ClkRate, CommonProtocol};

pub struct CommonCtl<'a> {
    clk_srcs: &'a [ClkSrc],
    clk_src_labels: &'a [&'a str],
}

impl<'a> CommonCtl<'a> {
    const CLK_SRC_NAME: &'a str = "clock-source";
    const CLK_RATE_NAME: &'a str = "clock-rate";
    const INPUT_THRESHOLD_NAME: &'a str = "input-threshold";
    const COAX_OUT_SRC_NAME: &'a str = "coax-output-source";

    const CLK_RATES: &'a [ClkRate] = &[
        ClkRate::R44100,
        ClkRate::R48000,
        ClkRate::R88200,
        ClkRate::R96000,
    ];

    pub const CLK_RATE_LABELS: &'a [&'a str] = &["44100", "48000", "88200", "96000"];

    const THRESHOLD_MIN: i32 = 0;
    const THRESHOLD_MAX: i32 = 32767;
    const THRESHOLD_STEP: i32 = 256;

    const COAX_OUT_SRC_LABELS: &'a [&'a str] = &["S/PDIF-1/2", "Mixer-1/2"];

    pub fn new(clk_srcs: &'a [ClkSrc], clk_src_labels: &'a [&'a str]) -> Self {
        CommonCtl {
            clk_srcs: clk_srcs,
            clk_src_labels: clk_src_labels,
        }
    }

    pub fn load(
        &mut self,
        _: &hinawa::SndTscm,
        _: &hinawa::FwReq,
        card_cntr: &mut card_cntr::CardCntr,
    ) -> Result<(), Error> {
        // For source of sampling clock.
        let elem_id = alsactl::ElemId::new_by_name(
            alsactl::ElemIfaceType::Mixer,
            0,
            0,
            Self::CLK_SRC_NAME,
            0,
        );
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, self.clk_src_labels, None, true)?;

        // For rate of sampling clock.
        let elem_id = alsactl::ElemId::new_by_name(
            alsactl::ElemIfaceType::Mixer,
            0,
            0,
            Self::CLK_RATE_NAME,
            0,
        );
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, Self::CLK_RATE_LABELS, None, true)?;

        // For threshold of input LED.
        let elem_id = alsactl::ElemId::new_by_name(
            alsactl::ElemIfaceType::Mixer,
            0,
            0,
            Self::INPUT_THRESHOLD_NAME,
            0,
        );
        let _ = card_cntr.add_int_elems(
            &elem_id,
            1,
            Self::THRESHOLD_MIN,
            Self::THRESHOLD_MAX,
            Self::THRESHOLD_STEP,
            1,
            None,
            true,
        )?;

        // For the source of output to coaxial interface.
        let elem_id = alsactl::ElemId::new_by_name(
            alsactl::ElemIfaceType::Mixer,
            0,
            0,
            Self::COAX_OUT_SRC_NAME,
            0,
        );
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, Self::COAX_OUT_SRC_LABELS, None, true)?;

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
            Self::CLK_SRC_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let src = req.get_clk_src(&node)?;
                    if let Some(index) = self.clk_srcs.iter().position(|&s| s == src) {
                        Ok(index as u32)
                    } else {
                        let label = "Unexpected value for source of clock";
                        Err(Error::new(FileError::Io, &label))
                    }
                })?;
                Ok(true)
            }
            Self::CLK_RATE_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let rate = req.get_clk_rate(&node)?;
                    if let Some(index) = Self::CLK_RATES.iter().position(|&r| r == rate) {
                        Ok(index as u32)
                    } else {
                        let label = "Unexpected value for rate of clock";
                        Err(Error::new(FileError::Io, &label))
                    }
                })?;
                Ok(true)
            }
            Self::INPUT_THRESHOLD_NAME => {
                ElemValueAccessor::<i32>::set_val(elem_value, || {
                    let val = req.get_input_threshold(&node)?;
                    Ok(val as i32)
                })?;
                Ok(true)
            }
            Self::COAX_OUT_SRC_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let index = req.get_coax_out_src(&node)?;
                    Ok(index as u32)
                })?;
                Ok(true)
            }
            _ => Ok(true),
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
            Self::CLK_SRC_NAME => {
                ElemValueAccessor::<u32>::get_val(new, |val| {
                    unit.lock()?;
                    let res = req.set_clk_src(&node, self.clk_srcs[val as usize]);
                    let _ = unit.unlock();
                    res
                })?;
                Ok(true)
            }
            Self::CLK_RATE_NAME => {
                ElemValueAccessor::<u32>::get_val(new, |val| {
                    unit.lock()?;
                    let res = req.set_clk_rate(&node, Self::CLK_RATES[val as usize]);
                    let _ = unit.unlock();
                    res
                })?;
                Ok(true)
            }
            Self::INPUT_THRESHOLD_NAME => {
                ElemValueAccessor::<i32>::get_val(new, |val| {
                    if val >= Self::THRESHOLD_MIN && val <= Self::THRESHOLD_MAX {
                        req.set_input_threshold(&node, val as i16)
                    } else {
                        let label = "Unexpected value for threshold";
                        Err(Error::new(FileError::Inval, &label))
                    }
                })?;
                Ok(true)
            }
            Self::COAX_OUT_SRC_NAME => {
                ElemValueAccessor::<u32>::get_val(new, |val| {
                    if val < Self::COAX_OUT_SRC_LABELS.len() as u32 {
                        req.set_coax_out_src(&node, val as usize)
                    } else {
                        let label = "Unexpected value for source of coaxial output";
                        Err(Error::new(FileError::Inval, &label))
                    }
                })?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
