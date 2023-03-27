// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use super::*;

fn clk_src_to_str(src: &ClkSrc) -> &'static str {
    match src {
        ClkSrc::Internal => "Internal",
        ClkSrc::WordClock => "WordClock",
        ClkSrc::Spdif => "S/PDIF",
        ClkSrc::Adat => "ADAT",
        ClkSrc::Adat2 => "ADAT2",
        ClkSrc::Continuous => "Continuous",
        ClkSrc::Reserved(_) => "Reserved",
    }
}

#[derive(Default, Debug)]
pub(crate) struct SamplingClockCtl<T>
where
    T: EfwHardwareSpecification
        + EfwWhollyCachableParamsOperation<SndEfw, EfwSamplingClockParameters>
        + EfwWhollyUpdatableParamsOperation<SndEfw, EfwSamplingClockParameters>,
{
    pub elem_id_list: Vec<ElemId>,
    pub params: EfwSamplingClockParameters,
    _phantom: PhantomData<T>,
}

impl<T> SamplingClockCtl<T>
where
    T: EfwHardwareSpecification
        + EfwWhollyCachableParamsOperation<SndEfw, EfwSamplingClockParameters>
        + EfwWhollyUpdatableParamsOperation<SndEfw, EfwSamplingClockParameters>,
{
    pub(crate) fn cache(&mut self, unit: &mut SndEfw, timeout_ms: u32) -> Result<(), Error> {
        T::cache_wholly(unit, &mut self.params, timeout_ms)
    }

    pub(crate) fn load(
        &mut self,
        card_cntr: &mut CardCntr,
        higher_rate_is_supported: bool,
    ) -> Result<(), Error> {
        let labels: Vec<&str> = T::SUPPORTED_SAMPLING_CLOCKS
            .iter()
            .map(|src| clk_src_to_str(src))
            .collect();

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, SRC_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, true)
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let labels: Vec<String> = T::SUPPORTED_SAMPLING_RATES
            .iter()
            .filter_map(|&rate| {
                if higher_rate_is_supported && rate >= 176400 {
                    None
                } else {
                    Some(rate)
                }
            })
            .map(|rate| rate.to_string())
            .collect();

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, RATE_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, true)
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        Ok(())
    }

    pub(crate) fn read(
        &mut self,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            SRC_NAME => {
                let pos = T::SUPPORTED_SAMPLING_CLOCKS
                    .iter()
                    .position(|s| self.params.source.eq(s))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            RATE_NAME => {
                let pos = T::SUPPORTED_SAMPLING_RATES
                    .iter()
                    .position(|r| self.params.rate.eq(r))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub(crate) fn write(
        &mut self,
        unit: &mut SndEfw,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            SRC_NAME => {
                let pos = elem_value.enumerated()[0] as usize;
                let mut params = self.params.clone();
                params.source = T::SUPPORTED_SAMPLING_CLOCKS
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| {
                        let label = format!("Invalid value for source of clock: {}", pos);
                        Error::new(FileError::Io, &label)
                    })
                    .copied()?;
                unit.lock()?;
                let res = T::update_wholly(unit, &params, timeout_ms).map(|_| self.params = params);
                let _ = unit.unlock();
                res.map(|_| true)
            }
            RATE_NAME => {
                let pos = elem_value.enumerated()[0] as usize;
                let mut params = self.params.clone();
                params.rate = T::SUPPORTED_SAMPLING_RATES
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| {
                        let label = format!("Invalid value for rate of clock: {}", pos);
                        Error::new(FileError::Io, &label)
                    })
                    .copied()?;
                unit.lock()?;
                let res = T::update_wholly(unit, &params, timeout_ms).map(|_| self.params = params);
                let _ = unit.unlock();
                res.map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

#[derive(Default)]
pub struct ClkCtl {
    pub notified_elem_id_list: Vec<ElemId>,
    pub params: EfwSamplingClockParameters,
    srcs: Vec<ClkSrc>,
    rates: Vec<u32>,
}

const SRC_NAME: &str = "clock-source";
const RATE_NAME: &str = "clock-rate";

impl ClkCtl {
    pub fn load(
        &mut self,
        hwinfo: &HwInfo,
        card_cntr: &mut CardCntr,
        unit: &mut SndEfw,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        self.srcs.extend_from_slice(&hwinfo.clk_srcs);
        self.rates.extend_from_slice(&hwinfo.clk_rates);

        self.cache(unit, timeout_ms)?;

        let labels: Vec<&str> = self.srcs.iter().map(|src| clk_src_to_str(src)).collect();

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, SRC_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, true)
            .map(|mut elem_id_list| self.notified_elem_id_list.append(&mut elem_id_list))?;

        let labels: Vec<String> = hwinfo
            .clk_rates
            .iter()
            .map(|rate| rate.to_string())
            .collect();

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, RATE_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, true)
            .map(|mut elem_id_list| self.notified_elem_id_list.append(&mut elem_id_list))?;

        Ok(())
    }

    pub fn cache(&mut self, unit: &mut SndEfw, timeout_ms: u32) -> Result<(), Error> {
        let state = unit.get_clock(timeout_ms)?;

        if self.srcs.iter().find(|s| state.0.eq(s)).is_none() {
            let name = clk_src_to_str(&state.0);
            let label = format!("Unexpected value for source of clock: {}", name);
            Err(Error::new(FileError::Io, &label))?;
        } else {
            self.params.source = state.0;
        }

        if self.rates.iter().find(|r| state.1.eq(r)).is_none() {
            let label = format!("Unexpected value for rate of clock: {}", state.1);
            Err(Error::new(FileError::Io, &label))?;
        } else {
            self.params.rate = state.1;
        }

        Ok(())
    }

    pub fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            SRC_NAME => {
                let pos = self
                    .srcs
                    .iter()
                    .position(|s| self.params.source.eq(s))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            RATE_NAME => {
                let pos = self
                    .rates
                    .iter()
                    .position(|r| self.params.rate.eq(r))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub fn write(
        &mut self,
        unit: &mut SndEfw,
        elem_id: &ElemId,
        _: &ElemValue,
        new: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            SRC_NAME => {
                let pos = new.enumerated()[0] as usize;
                let src = self.srcs.iter().nth(pos).copied().ok_or_else(|| {
                    let label = "Invalid value for source of clock";
                    Error::new(FileError::Io, &label)
                })?;
                unit.lock()?;
                let res = unit.set_clock(Some(src), None, timeout_ms);
                let _ = unit.unlock();
                res.map(|_| true)
            }
            RATE_NAME => {
                let pos = new.enumerated()[0] as usize;
                let rate = self.rates.iter().nth(pos).copied().ok_or_else(|| {
                    let label = "Invalid value for rate of clock";
                    Error::new(FileError::Io, &label)
                })?;
                unit.lock()?;
                let res = unit.set_clock(None, Some(rate), timeout_ms);
                let _ = unit.unlock();
                res.map(|_| true)
            }
            _ => Ok(false),
        }
    }
}
