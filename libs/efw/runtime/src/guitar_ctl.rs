// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use core::card_cntr;
use core::elem_value_accessor::ElemValueAccessor;

use efw_protocols::hw_info::*;
use efw_protocols::robot_guitar::*;

pub struct GuitarCtl {}

impl GuitarCtl {
    const MANUAL_CHARGE_NAME: &'static str = "guitar-manual-chage";
    const AUTO_CHARGE_NAME: &'static str = "guitar-auto-chage";
    const SUSPEND_TO_CHARGE: &'static str = "guitar-suspend-to-charge";

    const MIN_SEC: i32 = 0;
    const MAX_SEC: i32 = 60 * 60;   // = One hour.
    const STEP_SEC: i32 = 1;

    pub fn new() -> Self {
        GuitarCtl{}
    }

    pub fn load(&mut self, hwinfo: &HwInfo, card_cntr: &mut card_cntr::CardCntr)
        -> Result<(), Error>
    {
        let has_guitar_charge = hwinfo.caps.iter().find(|&e| *e == HwCap::GuitarCharging).is_some();

        if has_guitar_charge {
            let elem_id = alsactl::ElemId::new_by_name(
                alsactl::ElemIfaceType::Card, 0, 0, Self::MANUAL_CHARGE_NAME, 0);
            let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

            let elem_id = alsactl::ElemId::new_by_name(
                alsactl::ElemIfaceType::Card, 0, 0, Self::AUTO_CHARGE_NAME, 0);
            let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

            let elem_id = alsactl::ElemId::new_by_name(
                alsactl::ElemIfaceType::Card, 0, 0, Self::SUSPEND_TO_CHARGE, 0);
            let _ = card_cntr.add_int_elems(&elem_id, 1,
                Self::MIN_SEC, Self::MAX_SEC, Self::STEP_SEC, 1, None, true)?;
        }

        Ok(())
    }

    pub fn read(
        &mut self,
        unit: &mut hinawa::SndEfw,
        elem_id: &alsactl::ElemId,
        elem_value: &mut alsactl::ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            Self::MANUAL_CHARGE_NAME => {
                ElemValueAccessor::<bool>::set_val(elem_value, || {
                    unit.get_charge_state(timeout_ms)
                        .map(|s| s.manual_charge)
                })?;
                Ok(true)
            }
            Self::AUTO_CHARGE_NAME => {
                ElemValueAccessor::<bool>::set_val(elem_value, || {
                    unit.get_charge_state(timeout_ms)
                        .map(|s| s.auto_charge)
                })?;
                Ok(true)
            }
            Self::SUSPEND_TO_CHARGE => {
                ElemValueAccessor::<i32>::set_val(elem_value, || {
                    unit.get_charge_state(timeout_ms)
                        .map(|s| s.suspend_to_charge as i32)
                })?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub fn write(
        &mut self,
        unit: &mut hinawa::SndEfw,
        elem_id: &alsactl::ElemId,
        _: &alsactl::ElemValue,
        new: &alsactl::ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            Self::MANUAL_CHARGE_NAME => {
                ElemValueAccessor::<bool>::get_val(new, |val| {
                    let mut state = unit.get_charge_state(timeout_ms)?;
                    state.manual_charge = val;
                    unit.set_charge_state(&state, timeout_ms)
                })?;
                Ok(true)
            }
            Self::AUTO_CHARGE_NAME => {
                ElemValueAccessor::<bool>::get_val(new, |val| {
                    let mut state = unit.get_charge_state(timeout_ms)?;
                    state.auto_charge = val;
                    unit.set_charge_state(&state, timeout_ms)
                })?;
                Ok(true)
            }
            Self::SUSPEND_TO_CHARGE => {
                ElemValueAccessor::<i32>::get_val(new, |val| {
                    let mut state = unit.get_charge_state(timeout_ms)?;
                    state.suspend_to_charge = val as u32;
                    unit.set_charge_state(&state, timeout_ms)
                })?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
