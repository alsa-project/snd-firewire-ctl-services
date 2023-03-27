// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol about physical output.
//!
//! The module includes protocol about physical output defined by Echo Audio Digital Corporation for
//! Fireworks board module.

use super::*;

const CATEGORY_PHYS_OUTPUT: u32 = 4;

const CMD_SET_VOL: u32 = 0;
const CMD_GET_VOL: u32 = 1;
const CMD_SET_MUTE: u32 = 2;
const CMD_GET_MUTE: u32 = 3;
const CMD_SET_NOMINAL: u32 = 8;
const CMD_GET_NOMINAL: u32 = 9;

/// The parameters of all outputs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EfwOutputParameters {
    /// The volume of physical output. The value is unsigned fixed-point number of 8.24 format;
    /// i.e. Q24. It is 0x00000000..0x02000000 for -144.0..+6.0 dB.
    pub volumes: Vec<i32>,
    /// Whether to mute the physical output.
    pub mutes: Vec<bool>,
}

impl<O, P> EfwWhollyCachableParamsOperation<P, EfwOutputParameters> for O
where
    O: EfwHardwareSpecification,
    P: EfwProtocolExtManual,
{
    fn cache_wholly(
        proto: &mut P,
        states: &mut EfwOutputParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        assert_eq!(states.volumes.len(), Self::phys_output_count());
        assert_eq!(states.mutes.len(), Self::phys_output_count());

        states
            .volumes
            .iter_mut()
            .enumerate()
            .try_for_each(|(ch, volume)| {
                let args = [ch as u32, 0];
                let mut params = vec![0; 2];
                proto
                    .transaction(
                        CATEGORY_PHYS_OUTPUT,
                        CMD_GET_VOL,
                        &args,
                        &mut params,
                        timeout_ms,
                    )
                    .map(|_| *volume = params[1] as i32)
            })?;

        states
            .mutes
            .iter_mut()
            .enumerate()
            .try_for_each(|(ch, mute)| {
                let args = [ch as u32, 0];
                let mut params = vec![0; 2];
                proto
                    .transaction(
                        CATEGORY_PHYS_OUTPUT,
                        CMD_GET_MUTE,
                        &args,
                        &mut params,
                        timeout_ms,
                    )
                    .map(|_| *mute = params[1] > 0)
            })
    }
}

impl<O, P> EfwPartiallyUpdatableParamsOperation<P, EfwOutputParameters> for O
where
    O: EfwHardwareSpecification,
    P: EfwProtocolExtManual,
{
    fn update_partially(
        proto: &mut P,
        states: &mut EfwOutputParameters,
        updates: EfwOutputParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        assert_eq!(states.volumes.len(), Self::phys_output_count());
        assert_eq!(states.mutes.len(), Self::phys_output_count());

        states
            .volumes
            .iter_mut()
            .zip(updates.volumes.iter())
            .enumerate()
            .filter(|(_, (o, n))| !o.eq(n))
            .try_for_each(|(ch, (curr, &vol))| {
                let args = [ch as u32, vol as u32];
                let mut params = vec![0; 2];
                proto
                    .transaction(
                        CATEGORY_PHYS_OUTPUT,
                        CMD_SET_VOL,
                        &args,
                        &mut params,
                        timeout_ms,
                    )
                    .map(|_| *curr = vol)
            })?;

        states
            .mutes
            .iter_mut()
            .zip(updates.mutes.iter())
            .enumerate()
            .filter(|(_, (o, n))| !o.eq(n))
            .try_for_each(|(ch, (curr, &mute))| {
                let args = [ch as u32, mute as u32];
                let mut params = vec![0; 2];
                proto
                    .transaction(
                        CATEGORY_PHYS_OUTPUT,
                        CMD_SET_MUTE,
                        &args,
                        &mut params,
                        timeout_ms,
                    )
                    .map(|_| *curr = mute)
            })
    }
}

/// The parameters of physical outputs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EfwPhysOutputParameters {
    /// The nominal signal level of physical output.
    pub nominals: Vec<NominalSignalLevel>,
}

/// The specification of physical output.
pub trait EfwPhysOutputSpecification: EfwHardwareSpecification {
    fn phys_analog_output_count() -> usize {
        Self::PHYS_INPUT_GROUPS
            .iter()
            .filter(|(group_type, _)| PhysGroupType::Analog.eq(group_type))
            .fold(0, |total, (_, count)| total + count)
    }

    fn create_phys_output_parameters() -> EfwPhysOutputParameters {
        EfwPhysOutputParameters {
            nominals: vec![Default::default(); Self::phys_analog_output_count()],
        }
    }
}

impl<O, P> EfwWhollyCachableParamsOperation<P, EfwPhysOutputParameters> for O
where
    O: EfwPhysOutputSpecification,
    P: EfwProtocolExtManual,
{
    fn cache_wholly(
        proto: &mut P,
        states: &mut EfwPhysOutputParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        assert_eq!(states.nominals.len(), Self::phys_analog_output_count());

        states
            .nominals
            .iter_mut()
            .enumerate()
            .try_for_each(|(ch, level)| {
                let args = [ch as u32, 0];
                let mut params = vec![0; 2];
                proto
                    .transaction(
                        CATEGORY_PHYS_OUTPUT,
                        CMD_GET_NOMINAL,
                        &args,
                        &mut params,
                        timeout_ms,
                    )
                    .map(|_| deserialize_nominal_signal_level(level, params[1]))
            })
    }
}

impl<O, P> EfwPartiallyUpdatableParamsOperation<P, EfwPhysOutputParameters> for O
where
    O: EfwPhysOutputSpecification,
    P: EfwProtocolExtManual,
{
    fn update_partially(
        proto: &mut P,
        states: &mut EfwPhysOutputParameters,
        updates: EfwPhysOutputParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        assert_eq!(states.nominals.len(), Self::phys_analog_output_count());
        assert_eq!(updates.nominals.len(), Self::phys_analog_output_count());

        states
            .nominals
            .iter_mut()
            .zip(updates.nominals.iter())
            .enumerate()
            .filter(|(_, (o, n))| !o.eq(n))
            .try_for_each(|(ch, (curr, &level))| {
                let args = [ch as u32, serialize_nominal_signal_level(&level)];
                let mut params = vec![0; 2];
                proto
                    .transaction(
                        CATEGORY_PHYS_OUTPUT,
                        CMD_SET_NOMINAL,
                        &args,
                        &mut params,
                        timeout_ms,
                    )
                    .map(|_| *curr = level)
            })
    }
}
