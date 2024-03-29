// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol about port configuration.
//!
//! The module includes protocol about port configuration defined by Echo Audio Digital Corporation
//! for Fireworks board module.

use super::*;

const CATEGORY_PORT_CONF: u32 = 9;

const CMD_SET_MIRROR: u32 = 0;
const CMD_GET_MIRROR: u32 = 1;
const CMD_SET_DIG_MODE: u32 = 2;
const CMD_GET_DIG_MODE: u32 = 3;
const CMD_SET_PHANTOM: u32 = 4;
const CMD_GET_PHANTOM: u32 = 5;
const CMD_SET_STREAM_MAP: u32 = 6;
const CMD_GET_STREAM_MAP: u32 = 7;

/// The parameters for source of control room.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct EfwControlRoomSource(pub usize);

const CONTROL_ROOM_SOURCES: &[PhysGroupType] = &[
    PhysGroupType::Analog,
    PhysGroupType::Headphones,
    PhysGroupType::Spdif,
];

/// The specification of control room operation.
pub trait EfwControlRoomSpecification: EfwHardwareSpecification {
    fn control_room_source_pairs() -> Vec<(PhysGroupType, usize)> {
        Self::PHYS_OUTPUT_GROUPS
            .iter()
            .filter(|(group_type, _)| {
                CONTROL_ROOM_SOURCES
                    .iter()
                    .find(|t| group_type.eq(t))
                    .is_some()
            })
            .flat_map(|&(group_type, count)| {
                let entries: Vec<(PhysGroupType, usize)> =
                    (0..count).step_by(2).map(|i| (group_type, i)).collect();
                entries
            })
            .collect()
    }
}

fn phys_group_pairs(groups: &[(PhysGroupType, usize)]) -> Vec<(PhysGroupType, usize)> {
    groups
        .iter()
        .flat_map(|&(group_type, count)| {
            let entries: Vec<(PhysGroupType, usize)> =
                (0..count).step_by(2).map(|pos| (group_type, pos)).collect();
            entries
        })
        .collect()
}

impl<O, P> EfwWhollyCachableParamsOperation<P, EfwControlRoomSource> for O
where
    O: EfwControlRoomSpecification,
    P: EfwProtocolExtManual,
{
    fn cache_wholly(
        proto: &mut P,
        states: &mut EfwControlRoomSource,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let args = Vec::new();
        let mut params = vec![0];
        proto.transaction(
            CATEGORY_PORT_CONF,
            CMD_GET_MIRROR,
            &args,
            &mut params,
            timeout_ms,
        )?;
        let pos = (params[0] / 2) as usize;
        let entries = phys_group_pairs(Self::PHYS_OUTPUT_GROUPS);
        let entry = entries.iter().nth(pos).ok_or_else(|| {
            let msg = format!("Unexpected value {} for source of control room", pos);
            Error::new(FileError::Nxio, &msg)
        })?;

        states.0 = Self::control_room_source_pairs()
            .iter()
            .position(|e| entry.eq(e))
            .ok_or_else(|| {
                let msg = format!(
                    "Unexpected entry for source of control room: {:?},{}",
                    entry.0, entry.1
                );
                Error::new(FileError::Nxio, &msg)
            })?;

        Ok(())
    }
}

impl<O, P> EfwWhollyUpdatableParamsOperation<P, EfwControlRoomSource> for O
where
    O: EfwControlRoomSpecification,
    P: EfwProtocolExtManual,
{
    fn update_wholly(
        proto: &mut P,
        states: &EfwControlRoomSource,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let pairs = Self::control_room_source_pairs();
        let entry = pairs.iter().nth(states.0).ok_or_else(|| {
            let msg = format!("Invalid value for source of control room: {}", states.0);
            Error::new(FileError::Inval, &msg)
        })?;

        let pos = phys_group_pairs(Self::PHYS_OUTPUT_GROUPS)
            .iter()
            .position(|e| entry.eq(&e))
            .map(|pos| pos * 2)
            .ok_or_else(|| {
                let msg = format!(
                    "Invalid entry for source of control room: {:?},{}",
                    entry.0, entry.1
                );
                Error::new(FileError::Inval, &msg)
            })?;

        let args = [pos as u32];
        let mut params = Vec::new();
        proto.transaction(
            CATEGORY_PORT_CONF,
            CMD_SET_MIRROR,
            &args,
            &mut params,
            timeout_ms,
        )
    }
}

/// Type of audio signal for dignal input and output.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum EfwDigitalMode {
    /// Coaxial interface for S/PDIF signal.
    SpdifCoax,
    /// XLR interface for AES/EBU signal.
    AesebuXlr,
    /// Optical interface for S/PDIF signal.
    SpdifOpt,
    /// Optical interface for ADAT signal.
    AdatOpt,
    Unknown(u32),
}

impl Default for EfwDigitalMode {
    fn default() -> Self {
        Self::Unknown(u32::MAX)
    }
}

fn serialize_digital_mode(mode: &EfwDigitalMode, val: &mut u32) {
    *val = match *mode {
        EfwDigitalMode::SpdifCoax => 0,
        EfwDigitalMode::AesebuXlr => 1,
        EfwDigitalMode::SpdifOpt => 2,
        EfwDigitalMode::AdatOpt => 3,
        EfwDigitalMode::Unknown(val) => val,
    };
}

fn deserialize_digital_mode(val: u32) -> EfwDigitalMode {
    match val {
        0 => EfwDigitalMode::SpdifCoax,
        1 => EfwDigitalMode::AesebuXlr,
        2 => EfwDigitalMode::SpdifOpt,
        3 => EfwDigitalMode::AdatOpt,
        _ => EfwDigitalMode::Unknown(val),
    }
}

/// The specification for mode of digital input and output.
pub trait EfwDigitalModeSpecification: EfwHardwareSpecification {
    const DIG_MODES: &'static [(HwCap, EfwDigitalMode)] = &[
        (HwCap::OptionalSpdifCoax, EfwDigitalMode::SpdifCoax),
        (HwCap::OptionalAesebuXlr, EfwDigitalMode::AesebuXlr),
        (HwCap::OptionalSpdifOpt, EfwDigitalMode::SpdifOpt),
        (HwCap::OptionalAdatOpt, EfwDigitalMode::AdatOpt),
    ];

    fn create_digital_mode() -> EfwDigitalMode {
        Self::DIG_MODES
            .iter()
            .find(|(cap, _)| Self::CAPABILITIES.iter().find(|c| cap.eq(c)).is_some())
            .map(|(_, mode)| *mode)
            .unwrap()
    }

    fn create_digital_modes() -> Vec<EfwDigitalMode> {
        Self::DIG_MODES
            .iter()
            .filter(|(cap, _)| Self::CAPABILITIES.iter().find(|c| cap.eq(c)).is_some())
            .map(|(_, mode)| *mode)
            .collect()
    }
}

impl<O, P> EfwWhollyCachableParamsOperation<P, EfwDigitalMode> for O
where
    O: EfwDigitalModeSpecification,
    P: EfwProtocolExtManual,
{
    fn cache_wholly(
        proto: &mut P,
        states: &mut EfwDigitalMode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        assert!(Self::CAPABILITIES
            .iter()
            .find(|cap| Self::DIG_MODES.iter().find(|(c, _)| c.eq(cap)).is_some())
            .is_some());

        let args = Vec::new();
        let mut params = vec![0];
        proto
            .transaction(
                CATEGORY_PORT_CONF,
                CMD_GET_DIG_MODE,
                &args,
                &mut params,
                timeout_ms,
            )
            .map(|_| *states = deserialize_digital_mode(params[0]))
    }
}

impl<O, P> EfwWhollyUpdatableParamsOperation<P, EfwDigitalMode> for O
where
    O: EfwDigitalModeSpecification,
    P: EfwProtocolExtManual,
{
    fn update_wholly(proto: &mut P, states: &EfwDigitalMode, timeout_ms: u32) -> Result<(), Error> {
        assert!(Self::CAPABILITIES
            .iter()
            .find(|cap| Self::DIG_MODES.iter().find(|(c, _)| c.eq(cap)).is_some())
            .is_some());

        let mut args = [0];
        let mut params = Vec::new();
        serialize_digital_mode(&states, &mut args[0]);
        proto.transaction(
            CATEGORY_PORT_CONF,
            CMD_SET_DIG_MODE,
            &args,
            &mut params,
            timeout_ms,
        )
    }
}

/// The parameters for phantom powering.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct EfwPhantomPowering(pub bool);

/// The specification for mode of digital input and output.
pub trait EfwPhantomPoweringSpecification: EfwHardwareSpecification {}

impl<O, P> EfwWhollyCachableParamsOperation<P, EfwPhantomPowering> for O
where
    O: EfwPhantomPoweringSpecification,
    P: EfwProtocolExtManual,
{
    fn cache_wholly(
        proto: &mut P,
        states: &mut EfwPhantomPowering,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let args = Vec::new();
        let mut params = vec![0];
        proto
            .transaction(
                CATEGORY_PORT_CONF,
                CMD_GET_PHANTOM,
                &args,
                &mut params,
                timeout_ms,
            )
            .map(|_| states.0 = params[0] > 0)
    }
}

impl<O, P> EfwWhollyUpdatableParamsOperation<P, EfwPhantomPowering> for O
where
    O: EfwPhantomPoweringSpecification,
    P: EfwProtocolExtManual,
{
    fn update_wholly(
        proto: &mut P,
        states: &EfwPhantomPowering,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let args = [states.0 as u32];
        let mut params = Vec::new();
        proto.transaction(
            CATEGORY_PORT_CONF,
            CMD_SET_PHANTOM,
            &args,
            &mut params,
            timeout_ms,
        )
    }
}

/// Mapping between rx stream channel pairs and physical output channel pairs per mode of sampling
/// transfer frequency.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EfwRxStreamMaps(pub Vec<Vec<usize>>);

/// The specification of rx stream mapping.
pub trait EfwRxStreamMapsSpecification: EfwHardwareSpecification {
    const STREAM_MAPPING_RATE_TABLE: [&'static [u32]; 3] =
        [&[44100, 48000, 32000], &[88200, 96000], &[176400, 192000]];

    fn create_rx_stream_maps() -> EfwRxStreamMaps {
        let maps = Self::RX_CHANNEL_COUNTS
            .iter()
            .map(|&count| vec![Default::default(); count])
            .collect();
        EfwRxStreamMaps(maps)
    }
}

const MAP_SIZE: usize = 70;
const MAP_ENTRY_UNABAILABLE: u32 = 0xffffffff;

impl<O, P> EfwWhollyCachableParamsOperation<P, EfwRxStreamMaps> for O
where
    O: EfwRxStreamMapsSpecification,
    P: EfwProtocolExtManual,
{
    fn cache_wholly(
        proto: &mut P,
        states: &mut EfwRxStreamMaps,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        states
            .0
            .iter_mut()
            .zip(Self::STREAM_MAPPING_RATE_TABLE)
            .try_for_each(|(state, rates)| {
                let args = [rates[0] as u32];
                let mut params = vec![0; MAP_SIZE];

                proto
                    .transaction(
                        CATEGORY_PORT_CONF,
                        CMD_GET_STREAM_MAP,
                        &args,
                        &mut params,
                        timeout_ms,
                    )
                    .map(|_| {
                        params[4..]
                            .iter()
                            .zip(state.iter_mut())
                            .for_each(|(&quad, src)| *src = (quad / 2) as usize);
                    })
            })
    }
}

impl<O, P> EfwPartiallyUpdatableParamsOperation<P, EfwRxStreamMaps> for O
where
    O: EfwRxStreamMapsSpecification,
    P: EfwProtocolExtManual,
{
    fn update_partially(
        proto: &mut P,
        states: &mut EfwRxStreamMaps,
        updates: EfwRxStreamMaps,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        states
            .0
            .iter_mut()
            .zip(&updates.0)
            .zip(Self::STREAM_MAPPING_RATE_TABLE)
            .zip(Self::RX_CHANNEL_COUNTS)
            .zip(Self::TX_CHANNEL_COUNTS)
            .filter(|((((o, n), _), _), _)| !n.eq(o))
            .try_for_each(
                |((((curr, update), rates), rx_channel_count), tx_channel_count)| {
                    let mut args = [0; MAP_SIZE];

                    args[0] = rates[0];
                    // NOTE: This field is filled with clock source bits, however it's not used actually.
                    args[1] = 0;
                    args[2] = (rx_channel_count / 2) as u32;
                    args[3] = (Self::phys_output_count() / 2) as u32;
                    args[4..36].fill(MAP_ENTRY_UNABAILABLE);
                    args[36] = (tx_channel_count / 2) as u32;
                    args[37] = (Self::phys_input_count() / 2) as u32;
                    args[38..70].fill(MAP_ENTRY_UNABAILABLE);

                    args[4..]
                        .iter_mut()
                        .zip(update.iter())
                        .for_each(|(quad, &src)| *quad = (src * 2) as u32);

                    // MEMO: No hardware supports tx stream mapping.

                    proto
                        .transaction(
                            CATEGORY_PORT_CONF,
                            CMD_SET_STREAM_MAP,
                            &args,
                            &mut Vec::new(),
                            timeout_ms,
                        )
                        .map(|_| curr.copy_from_slice(&update))
                },
            )
    }
}
