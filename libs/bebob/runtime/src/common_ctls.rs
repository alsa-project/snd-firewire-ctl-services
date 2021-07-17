// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use hinawa::{SndUnit, SndUnitExt};

use alsactl::{ElemId, ElemIfaceType, ElemValue};

use core::card_cntr::CardCntr;
use core::elem_value_accessor::ElemValueAccessor;

use bebob_protocols::*;

use super::model::{CLK_RATE_NAME, CLK_SRC_NAME};

pub trait MediaClkFreqCtlOperation<T: MediaClockFrequencyOperation> {
    fn load_freq(&mut self, card_cntr: &mut CardCntr) -> Result<Vec<ElemId>, Error> {
        let labels: Vec<String> = T::FREQ_LIST.iter().map(|&r| r.to_string()).collect();

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, CLK_RATE_NAME, 0);
        card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)
    }

    fn read_freq(
        &self,
        avc: &BebobAvc,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            CLK_RATE_NAME => ElemValueAccessor::<u32>::set_val(elem_value, || {
                T::read_clk_freq(avc, timeout_ms).map(|idx| idx as u32)
            })
            .map(|_| true),
            _ => Ok(false),
        }
    }

    fn write_freq(
        &self,
        unit: &mut SndUnit,
        avc: &BebobAvc,
        elem_id: &ElemId,
        _: &ElemValue,
        new: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            CLK_RATE_NAME => {
                unit.lock()?;
                let res = ElemValueAccessor::<u32>::get_val(new, |val| {
                    T::write_clk_freq(avc, val as usize, timeout_ms)
                })
                .map(|_| true);
                let _ = unit.unlock();
                res
            }
            _ => Ok(false),
        }
    }
}

pub trait SamplingClkSrcCtlOperation<T: SamplingClockSourceOperation> {
    const SRC_LABELS: &'static [&'static str];

    fn load_src(&mut self, card_cntr: &mut CardCntr) -> Result<Vec<ElemId>, Error> {
        assert_eq!(
            Self::SRC_LABELS.len(),
            T::SRC_LIST.len(),
            "Programming error for count of clock source"
        );

        let mut elem_id_list = Vec::new();

        if T::SRC_LIST.len() > 1 {
            let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, CLK_SRC_NAME, 0);
            card_cntr
                .add_enum_elems(&elem_id, 1, 1, &Self::SRC_LABELS, None, true)
                .map(|mut elem_id| elem_id_list.append(&mut elem_id))?;
        }

        Ok(elem_id_list)
    }

    fn read_src(
        &self,
        avc: &BebobAvc,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            CLK_SRC_NAME => ElemValueAccessor::<u32>::set_val(elem_value, || {
                T::read_clk_src(avc, timeout_ms).map(|idx| idx as u32)
            })
            .map(|_| true),
            _ => Ok(false),
        }
    }

    fn write_src(
        &self,
        unit: &mut SndUnit,
        avc: &BebobAvc,
        elem_id: &ElemId,
        _: &ElemValue,
        new: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            CLK_SRC_NAME => {
                unit.lock()?;
                let res = ElemValueAccessor::<u32>::get_val(new, |val| {
                    T::write_clk_src(avc, val as usize, timeout_ms)
                })
                .map(|_| true);
                let _ = unit.unlock();
                res
            }
            _ => Ok(false),
        }
    }
}
