// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2023 Takashi Sakamoto

//! Protocol implementations for Robot Interface Pack.
//!
//! The module includes protocol about port configuration defined by Echo Audio Digital Corporation
//! for Gibson Robot Interface Pack.

use super::{robot_guitar::*, *};

/// Protocol implementation for former model of Robot Interface Pack (RIP).
///
/// Diagram of internal signal flow
///
/// ```text
///
/// guitar-input-1 ------+
/// piezo-input-2 -------+---+-------------> stream-output-1/2
/// string-input-1/2 --------|--+----------> stream-output-5/6
/// string-input-3/4 --------|--|----------> stream-output-7/8
/// string-input-5/6 --------|--|--+-------> stream-output-7/8
///                          |  |  |
///                          v  v  v
///                       ++=========++
///                       ||  mixer  ||
/// stream-input-1/2 ---> ||         || ---> analog-output-1/2
///                       ||  8 x 6  ||
///                       ++=========++
/// ```
#[derive(Default, Debug)]
pub struct RipProtocol;

impl EfwHardwareSpecification for RipProtocol {
    const SUPPORTED_SAMPLING_RATES: &'static [u32] = &[44100, 48000, 88200, 96000, 176400, 192000];
    const SUPPORTED_SAMPLING_CLOCKS: &'static [ClkSrc] =
        &[ClkSrc::Internal, ClkSrc::WordClock, ClkSrc::Spdif];
    const CAPABILITIES: &'static [HwCap] = &[
        HwCap::ChangeableRespAddr,
        HwCap::Fpga,
        HwCap::RobotGuitar,
        HwCap::GuitarCharging,
    ];
    const TX_CHANNEL_COUNTS: [usize; 3] = [8, 8, 8];
    const RX_CHANNEL_COUNTS: [usize; 3] = [2, 2, 2];
    const MONITOR_SOURCE_COUNT: usize = 8;
    const MONITOR_DESTINATION_COUNT: usize = 2;
    const MIDI_INPUT_COUNT: usize = 0;
    const MIDI_OUTPUT_COUNT: usize = 0;

    const PHYS_INPUT_GROUPS: &'static [(PhysGroupType, usize)] = &[
        (PhysGroupType::Guitar, 1),
        (PhysGroupType::PiezoGuitar, 1),
        (PhysGroupType::GuitarString, 6),
    ];

    const PHYS_OUTPUT_GROUPS: &'static [(PhysGroupType, usize)] = &[(PhysGroupType::Analog, 2)];
}

impl EfwPlaybackSoloSpecification for RipProtocol {}

impl EfwRobotGuitarSpecification for RipProtocol {}
