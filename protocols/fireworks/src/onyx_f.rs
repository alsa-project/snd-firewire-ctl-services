// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2023 Takashi Sakamoto

//! Protocol implementations for Mackie Onyx-F series.
//!
//! The module includes protocol about port configuration defined by Echo Audio Digital Corporation
//! for Mackie Onyx-F series.

use super::{port_conf::*, *};

/// Protocol implementation for former model of Onyx 1200F.
#[derive(Default, Debug)]
pub struct Onyx1200fProtocol;

impl EfwHardwareSpecification for Onyx1200fProtocol {
    const SUPPORTED_SAMPLING_RATES: &'static [u32] = &[44100, 48000, 88200, 96000, 176400, 192000];
    const SUPPORTED_SAMPLING_CLOCKS: &'static [ClkSrc] = &[
        ClkSrc::Internal,
        ClkSrc::WordClock,
        ClkSrc::Spdif,
        ClkSrc::Adat,
        ClkSrc::Adat2,
    ];
    const CAPABILITIES: &'static [HwCap] = &[
        HwCap::ChangeableRespAddr,
        HwCap::OptionalSpdifCoax,
        HwCap::OptionalAesebuXlr,
        HwCap::Dsp,
        HwCap::Fpga,
        // Fixup.
        HwCap::ControlRoom,
    ];
    const TX_CHANNEL_COUNTS: [usize; 3] = [30, 16, 8];
    const RX_CHANNEL_COUNTS: [usize; 3] = [34, 16, 8];
    const MONITOR_SOURCE_COUNT: usize = 30;
    const MONITOR_DESTINATION_COUNT: usize = 34;
    const MIDI_INPUT_COUNT: usize = 2;
    const MIDI_OUTPUT_COUNT: usize = 2;

    const PHYS_INPUT_GROUPS: &'static [(PhysGroupType, usize)] = &[
        (PhysGroupType::Analog, 12),
        (PhysGroupType::Adat, 8),
        (PhysGroupType::Adat, 8),
        (PhysGroupType::Spdif, 2),
    ];

    const PHYS_OUTPUT_GROUPS: &'static [(PhysGroupType, usize)] = &[
        (PhysGroupType::Analog, 8),
        (PhysGroupType::Adat, 8),
        (PhysGroupType::Adat, 8),
        (PhysGroupType::Headphones, 4),
        (PhysGroupType::Spdif, 2),
        //(PhysGroupType::AnalogMirror, 2), This is not operable from software.
    ];
}

impl EfwControlRoomSpecification for Onyx1200fProtocol {}

/// Protocol implementation for Mackie Onyx 400F. The higher sampling rates are available only with
/// firmware version 4 and former.
#[derive(Default, Debug)]
pub struct Onyx400fProtocol;

impl EfwHardwareSpecification for Onyx400fProtocol {
    const SUPPORTED_SAMPLING_RATES: &'static [u32] = &[44100, 48000, 88200, 96000, 176400, 192000];
    const SUPPORTED_SAMPLING_CLOCKS: &'static [ClkSrc] =
        &[ClkSrc::Internal, ClkSrc::WordClock, ClkSrc::Spdif];
    const CAPABILITIES: &'static [HwCap] =
        &[HwCap::ChangeableRespAddr, HwCap::ControlRoom, HwCap::Dsp];
    const TX_CHANNEL_COUNTS: [usize; 3] = [10, 10, 10];
    const RX_CHANNEL_COUNTS: [usize; 3] = [10, 10, 10];
    const MONITOR_SOURCE_COUNT: usize = 10;
    const MONITOR_DESTINATION_COUNT: usize = 10;
    const MIDI_INPUT_COUNT: usize = 2;
    const MIDI_OUTPUT_COUNT: usize = 2;

    const PHYS_INPUT_GROUPS: &'static [(PhysGroupType, usize)] =
        &[(PhysGroupType::Analog, 8), (PhysGroupType::Spdif, 2)];

    const PHYS_OUTPUT_GROUPS: &'static [(PhysGroupType, usize)] = &[
        (PhysGroupType::Analog, 8),
        (PhysGroupType::Spdif, 2),
        //(PhysGroupType::AnalogMirror, 2), This is not operable from software.
    ];
}

impl EfwControlRoomSpecification for Onyx400fProtocol {}
