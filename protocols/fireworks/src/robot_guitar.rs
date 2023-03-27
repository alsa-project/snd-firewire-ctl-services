// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol about robot guitar.
//!
//! The module includes protocol about robot guitar defined by Echo Audio Digital Corporation for
//! Fireworks board module.

use super::*;

const CATEGORY_ROBOT_GUITAR: u32 = 10;

const CMD_SET_CHARGE_STATE: u32 = 7;
const CMD_GET_CHARGE_STATE: u32 = 8;

/// State of charging for Robot Guitar.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct GuitarChargeState {
    pub manual_charge: bool,
    pub auto_charge: bool,
    pub suspend_to_charge: u32,
}

/// The specification of robot guitar.
pub trait EfwRobotGuitarSpecification {}

impl<O, P> EfwWhollyCachableParamsOperation<P, GuitarChargeState> for O
where
    O: EfwRobotGuitarSpecification,
    P: EfwProtocolExtManual,
{
    fn cache_wholly(
        proto: &mut P,
        states: &mut GuitarChargeState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut params = vec![0; 3];
        proto
            .transaction(
                CATEGORY_ROBOT_GUITAR,
                CMD_GET_CHARGE_STATE,
                &[],
                &mut params,
                timeout_ms,
            )
            .map(|_| {
                states.manual_charge = params[0] > 0;
                states.auto_charge = params[1] > 0;
                states.suspend_to_charge = params[2];
            })
    }
}

impl<O, P> EfwWhollyUpdatableParamsOperation<P, GuitarChargeState> for O
where
    O: EfwRobotGuitarSpecification,
    P: EfwProtocolExtManual,
{
    fn update_wholly(
        proto: &mut P,
        states: &GuitarChargeState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let args = [
            states.manual_charge as u32,
            states.auto_charge as u32,
            states.suspend_to_charge,
        ];
        let mut params = vec![0; 3];
        proto.transaction(
            CATEGORY_ROBOT_GUITAR,
            CMD_SET_CHARGE_STATE,
            &args,
            &mut params,
            timeout_ms,
        )
    }
}
