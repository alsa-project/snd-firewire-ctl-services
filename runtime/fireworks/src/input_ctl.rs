// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use {super::*, protocols::phys_input::*};

const IN_NOMINAL_NAME: &str = "input-nominal";

#[derive(Debug)]
pub(crate) struct PhysInputCtl<T>
where
    T: EfwPhysInputSpecification
        + EfwWhollyCachableParamsOperation<SndEfw, EfwPhysInputParameters>
        + EfwPartiallyUpdatableParamsOperation<SndEfw, EfwPhysInputParameters>,
{
    pub elem_id_list: Vec<ElemId>,
    params: EfwPhysInputParameters,
    _phantom: PhantomData<T>,
}

impl<T> Default for PhysInputCtl<T>
where
    T: EfwPhysInputSpecification
        + EfwWhollyCachableParamsOperation<SndEfw, EfwPhysInputParameters>
        + EfwPartiallyUpdatableParamsOperation<SndEfw, EfwPhysInputParameters>,
{
    fn default() -> Self {
        Self {
            elem_id_list: Default::default(),
            params: T::create_phys_input_parameters(),
            _phantom: Default::default(),
        }
    }
}

impl<T> PhysInputCtl<T>
where
    T: EfwPhysInputSpecification
        + EfwWhollyCachableParamsOperation<SndEfw, EfwPhysInputParameters>
        + EfwPartiallyUpdatableParamsOperation<SndEfw, EfwPhysInputParameters>,
{
    const IN_NOMINAL_LABELS: [&'static str; 2] = ["+4dBu", "-10dBV"];
    const IN_NOMINAL_LEVELS: [NominalSignalLevel; 2] = [
        NominalSignalLevel::Professional,
        NominalSignalLevel::Consumer,
    ];

    pub(crate) fn cache(&mut self, unit: &mut SndEfw, timeout_ms: u32) -> Result<(), Error> {
        let res = T::cache_wholly(unit, &mut self.params, timeout_ms);
        debug!(params = ?self.params, ?res);
        res
    }

    pub(crate) fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, IN_NOMINAL_NAME, 0);
        card_cntr
            .add_enum_elems(
                &elem_id,
                1,
                self.params.nominals.len(),
                &Self::IN_NOMINAL_LABELS,
                None,
                true,
            )
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))
    }

    pub(crate) fn read(
        &mut self,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            IN_NOMINAL_NAME => {
                let vals: Vec<u32> = self
                    .params
                    .nominals
                    .iter()
                    .map(|level| {
                        Self::IN_NOMINAL_LEVELS
                            .iter()
                            .position(|l| level.eq(l))
                            .map(|pos| pos as u32)
                            .unwrap()
                    })
                    .collect();
                elem_value.set_enum(&vals);
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
            IN_NOMINAL_NAME => {
                let mut params = self.params.clone();
                params
                    .nominals
                    .iter_mut()
                    .zip(elem_value.enumerated())
                    .try_for_each(|(level, &val)| {
                        let pos = val as usize;
                        Self::IN_NOMINAL_LEVELS
                            .iter()
                            .nth(pos)
                            .ok_or_else(|| {
                                let label =
                                    format!("Invalid value for nominal level of input: {}", pos);
                                Error::new(FileError::Inval, &label)
                            })
                            .map(|&l| *level = l)
                    })?;
                let res = T::update_partially(unit, &mut self.params, params, timeout_ms);
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            _ => Ok(false),
        }
    }
}
