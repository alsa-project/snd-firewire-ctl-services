// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use {super::*, protocols::robot_guitar::*};

const MANUAL_CHARGE_NAME: &str = "guitar-manual-chage";
const AUTO_CHARGE_NAME: &str = "guitar-auto-chage";
const SUSPEND_TO_CHARGE: &str = "guitar-suspend-to-charge";

#[derive(Default, Debug)]
pub(crate) struct RobotGuitarCtl<T>
where
    T: EfwRobotGuitarSpecification
        + EfwWhollyCachableParamsOperation<SndEfw, GuitarChargeState>
        + EfwWhollyUpdatableParamsOperation<SndEfw, GuitarChargeState>,
{
    pub elem_id_list: Vec<ElemId>,
    params: GuitarChargeState,
    _phantom: PhantomData<T>,
}

impl<T> RobotGuitarCtl<T>
where
    T: EfwRobotGuitarSpecification
        + EfwWhollyCachableParamsOperation<SndEfw, GuitarChargeState>
        + EfwWhollyUpdatableParamsOperation<SndEfw, GuitarChargeState>,
{
    const MIN_SEC: i32 = 0;
    const MAX_SEC: i32 = 60 * 60; // = One hour.
    const STEP_SEC: i32 = 1;

    pub(crate) fn cache(&mut self, unit: &mut SndEfw, timeout_ms: u32) -> Result<(), Error> {
        let res = T::cache_wholly(unit, &mut self.params, timeout_ms);
        debug!(params = ?self.params, ?res);
        res
    }

    pub(crate) fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, MANUAL_CHARGE_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, 1, 1, true)
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, AUTO_CHARGE_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, 1, 1, true)
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, SUSPEND_TO_CHARGE, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                Self::MIN_SEC,
                Self::MAX_SEC,
                Self::STEP_SEC,
                1,
                None,
                true,
            )
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        Ok(())
    }

    pub(crate) fn read(
        &mut self,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            MANUAL_CHARGE_NAME => {
                elem_value.set_bool(&[self.params.manual_charge]);
                Ok(true)
            }
            AUTO_CHARGE_NAME => {
                elem_value.set_bool(&[self.params.auto_charge]);
                Ok(true)
            }
            SUSPEND_TO_CHARGE => {
                elem_value.set_int(&[self.params.suspend_to_charge as i32]);
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
            MANUAL_CHARGE_NAME => {
                let mut params = self.params.clone();
                params.manual_charge = elem_value.boolean()[0];
                let res = T::update_wholly(unit, &params, timeout_ms).map(|_| self.params = params);
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            AUTO_CHARGE_NAME => {
                let mut params = self.params.clone();
                params.auto_charge = elem_value.boolean()[0];
                let res = T::update_wholly(unit, &params, timeout_ms).map(|_| self.params = params);
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            SUSPEND_TO_CHARGE => {
                let mut params = self.params.clone();
                params.suspend_to_charge = elem_value.int()[0] as u32;
                let res = T::update_wholly(unit, &params, timeout_ms).map(|_| self.params = params);
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            _ => Ok(false),
        }
    }
}
