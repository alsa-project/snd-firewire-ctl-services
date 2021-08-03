// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

pub mod saffirepro26io_model;
pub mod saffirepro10io_model;

pub mod saffire_model;
pub mod saffirele_model;

use glib::Error;

use hinawa::FwReq;
use hinawa::{SndUnit, SndUnitExt};

use alsactl::{ElemId, ElemIfaceType, ElemValue};

use core::card_cntr::CardCntr;
use core::elem_value_accessor::ElemValueAccessor;

use bebob_protocols::focusrite::saffireproio::*;

use crate::model::{CLK_RATE_NAME, CLK_SRC_NAME};

trait SaffireProMediaClkFreqCtlOperation<T: SaffireProioMediaClockFrequencyOperation> {
    fn load_freq(&mut self, card_cntr: &mut CardCntr) -> Result<Vec<ElemId>, Error> {
        let labels: Vec<String> = T::FREQ_LIST.iter().map(|&r| r.to_string()).collect();

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, CLK_RATE_NAME, 0);
        card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)
    }

    fn read_freq(
        &self,
        unit: &SndUnit,
        req: &FwReq,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            CLK_RATE_NAME => ElemValueAccessor::<u32>::set_val(elem_value, || {
                T::read_clk_freq(req, &unit.get_node(), timeout_ms).map(|idx| idx as u32)
            })
            .map(|_| true),
            _ => Ok(false),
        }
    }

    fn write_freq(
        &self,
        unit: &mut SndUnit,
        req: &FwReq,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            CLK_RATE_NAME => {
                unit.lock()?;
                let res = ElemValueAccessor::<u32>::get_val(elem_value, |val| {
                    T::write_clk_freq(req, &unit.get_node(), val as usize, timeout_ms)
                })
                .map(|_| true);
                let _ = unit.unlock();
                res
            }
            _ => Ok(false),
        }
    }
}

fn sampling_clk_src_to_str(src: &SaffireProioSamplingClockSource) -> &str {
    match src {
        SaffireProioSamplingClockSource::Internal => "Internal",
        SaffireProioSamplingClockSource::Spdif => "S/PDIF",
        SaffireProioSamplingClockSource::Adat0 => "ADAT-A",
        SaffireProioSamplingClockSource::Adat1 => "ADAT-B",
        SaffireProioSamplingClockSource::WordClock => "Word-clock",
    }
}

trait SaffireProSamplingClkSrcCtlOperation<T: SaffireProioSamplingClockSourceOperation> {
    fn load_src(&mut self, card_cntr: &mut CardCntr) -> Result<Vec<ElemId>, Error> {
        let mut elem_id_list = Vec::new();

        let labels: Vec<&str> = T::SRC_LIST.iter()
            .map(|s| sampling_clk_src_to_str(s))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, CLK_SRC_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, true)
            .map(|mut elem_id| elem_id_list.append(&mut elem_id))?;

        Ok(elem_id_list)
    }

    fn read_src(
        &self,
        unit: &mut SndUnit,
        req: &FwReq,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            CLK_SRC_NAME => ElemValueAccessor::<u32>::set_val(elem_value, || {
                T::read_clk_src(req, &unit.get_node(), timeout_ms).map(|idx| idx as u32)
            })
            .map(|_| true),
            _ => Ok(false),
        }
    }

    fn write_src(
        &self,
        unit: &mut SndUnit,
        req: &FwReq,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            CLK_SRC_NAME => {
                unit.lock()?;
                let res = ElemValueAccessor::<u32>::get_val(elem_value, |val| {
                    T::write_clk_src(req, &unit.get_node(), val as usize, timeout_ms)
                })
                .map(|_| true);
                let _ = unit.unlock();
                res
            }
            _ => Ok(false),
        }
    }
}
