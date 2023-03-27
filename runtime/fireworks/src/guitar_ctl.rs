// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use {super::*, protocols::robot_guitar::*};

#[derive(Default)]
pub struct GuitarCtl {
    params: GuitarChargeState,
}

const MANUAL_CHARGE_NAME: &str = "guitar-manual-chage";
const AUTO_CHARGE_NAME: &str = "guitar-auto-chage";
const SUSPEND_TO_CHARGE: &str = "guitar-suspend-to-charge";

impl GuitarCtl {
    const MIN_SEC: i32 = 0;
    const MAX_SEC: i32 = 60 * 60; // = One hour.
    const STEP_SEC: i32 = 1;

    fn cache(&mut self, hw_info: &HwInfo, unit: &mut SndEfw, timeout_ms: u32) -> Result<(), Error> {
        if hw_info
            .caps
            .iter()
            .find(|e| HwCap::GuitarCharging.eq(e))
            .is_some()
        {
            unit.get_charge_state(timeout_ms)
                .map(|state| self.params = state)?;
        }

        Ok(())
    }

    pub fn load(
        &mut self,
        hwinfo: &HwInfo,
        unit: &mut SndEfw,
        card_cntr: &mut CardCntr,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        self.cache(hwinfo, unit, timeout_ms)?;

        if hwinfo
            .caps
            .iter()
            .find(|&e| HwCap::GuitarCharging.eq(e))
            .is_some()
        {
            let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, MANUAL_CHARGE_NAME, 0);
            let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

            let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, AUTO_CHARGE_NAME, 0);
            let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

            let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, SUSPEND_TO_CHARGE, 0);
            let _ = card_cntr.add_int_elems(
                &elem_id,
                1,
                Self::MIN_SEC,
                Self::MAX_SEC,
                Self::STEP_SEC,
                1,
                None,
                true,
            )?;
        }

        Ok(())
    }

    pub fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
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

    pub fn write(
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
                unit.set_charge_state(&params, timeout_ms)
                    .map(|_| self.params = params)?;
                Ok(true)
            }
            AUTO_CHARGE_NAME => {
                let mut params = self.params.clone();
                params.auto_charge = elem_value.boolean()[0];
                unit.set_charge_state(&params, timeout_ms)
                    .map(|_| self.params = params)?;
                Ok(true)
            }
            SUSPEND_TO_CHARGE => {
                let mut params = self.params.clone();
                params.suspend_to_charge = elem_value.int()[0] as u32;
                unit.set_charge_state(&params, timeout_ms)
                    .map(|_| self.params = params)?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
