// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol about physical input.
//!
//! The module includes protocol about physical input defined by Echo Audio Digital Corporation for
//! Fireworks board module.

use super::*;

const CATEGORY_PHYS_INPUT: u32 = 5;

const CMD_SET_NOMINAL: u32 = 8;
const CMD_GET_NOMINAL: u32 = 9;

/// The parameters of physical inputs.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct EfwPhysInputParameters {
    /// The nominal signal level of physical input.
    pub nominals: Vec<NominalSignalLevel>,
}

/// The specification of physical input.
pub trait EfwPhysInputSpecification: EfwHardwareSpecification {
    fn phys_analog_input_count() -> usize {
        Self::PHYS_INPUT_GROUPS
            .iter()
            .filter(|(group_type, _)| PhysGroupType::Analog.eq(group_type))
            .fold(0, |total, (_, count)| total + count)
    }

    fn create_phys_input_parameters() -> EfwPhysInputParameters {
        EfwPhysInputParameters {
            nominals: vec![Default::default(); Self::phys_analog_input_count()],
        }
    }
}

impl<O, P> EfwWhollyCachableParamsOperation<P, EfwPhysInputParameters> for O
where
    O: EfwPhysInputSpecification,
    P: EfwProtocolExtManual,
{
    fn cache_wholly(
        proto: &mut P,
        states: &mut EfwPhysInputParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        assert_eq!(states.nominals.len(), Self::phys_analog_input_count());

        states
            .nominals
            .iter_mut()
            .enumerate()
            .try_for_each(|(ch, level)| {
                let args = [ch as u32, 0];
                let mut params = vec![0; 2];
                proto
                    .transaction(
                        CATEGORY_PHYS_INPUT,
                        CMD_GET_NOMINAL,
                        &args,
                        &mut params,
                        timeout_ms,
                    )
                    .map(|_| deserialize_nominal_signal_level(level, params[1]))
            })
    }
}

impl<O, P> EfwPartiallyUpdatableParamsOperation<P, EfwPhysInputParameters> for O
where
    O: EfwPhysInputSpecification,
    P: EfwProtocolExtManual,
{
    fn update_partially(
        proto: &mut P,
        states: &mut EfwPhysInputParameters,
        updates: EfwPhysInputParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        assert_eq!(states.nominals.len(), Self::phys_analog_input_count());
        assert_eq!(updates.nominals.len(), Self::phys_analog_input_count());

        states
            .nominals
            .iter_mut()
            .zip(&updates.nominals)
            .enumerate()
            .filter(|(_, (o, n))| !o.eq(n))
            .try_for_each(|(ch, (curr, level))| {
                let args = [ch as u32, serialize_nominal_signal_level(level)];
                proto
                    .transaction(
                        CATEGORY_PHYS_INPUT,
                        CMD_SET_NOMINAL,
                        &args,
                        &mut vec![0; 2],
                        timeout_ms,
                    )
                    .map(|_| *curr = *level)
            })
    }
}
