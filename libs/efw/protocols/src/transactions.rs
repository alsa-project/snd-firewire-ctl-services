// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use hinawa::SndEfwExtManual;

const TIMEOUT: u32 = 200;

enum Category {
    Guitar,
}

impl From<Category> for u32 {
    fn from(cat: Category) -> Self {
        match cat {
            Category::Guitar => 0x0a,
        }
    }
}

#[derive(Debug)]
pub struct GuitarChargeState {
    pub manual_charge: bool,
    pub auto_charge: bool,
    pub suspend_to_charge: u32,
}

pub struct EfwGuitar {}

impl EfwGuitar {
    const CMD_SET_CHARGE_STATE: u32 = 7;
    const CMD_GET_CHARGE_STATE: u32 = 8;

    pub fn get_charge_state(unit: &hinawa::SndEfw) -> Result<GuitarChargeState, Error> {
        let mut params = [0; 3];
        let _ = unit.transaction_sync(
            u32::from(Category::Guitar),
            Self::CMD_GET_CHARGE_STATE,
            None,
            Some(&mut params),
            TIMEOUT,
        )?;
        let state = GuitarChargeState {
            manual_charge: params[0] > 0,
            auto_charge: params[1] > 0,
            suspend_to_charge: params[2],
        };
        Ok(state)
    }

    pub fn set_charge_state(unit: &hinawa::SndEfw, state: &GuitarChargeState) -> Result<(), Error> {
        let args = [
            state.manual_charge as u32,
            state.auto_charge as u32,
            state.suspend_to_charge,
        ];
        let mut params = [0; 3];
        let _ = unit.transaction_sync(
            u32::from(Category::Guitar),
            Self::CMD_SET_CHARGE_STATE,
            Some(&args),
            Some(&mut params),
            TIMEOUT,
        )?;
        Ok(())
    }
}
