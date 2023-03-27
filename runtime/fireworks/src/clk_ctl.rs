// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use super::*;

const SRC_NAME: &str = "clock-source";
const RATE_NAME: &str = "clock-rate";

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
