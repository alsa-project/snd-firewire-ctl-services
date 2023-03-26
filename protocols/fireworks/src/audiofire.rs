// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2023 Takashi Sakamoto

//! Protocol implementations for Echo Audio Audiofire series.
//!
//! The module includes protocol about port configuration defined by Echo Audio Digital Corporation
//! for Audiofire.

use super::{port_conf::*, *};

/// Protocol implementation for former model of AudioFire 12. The higher sampling rates are
/// available only with firmware version 4 and former.
#[derive(Default, Debug)]
pub struct Audiofire12FormerProtocol;

impl EfwHardwareSpecification for Audiofire12FormerProtocol {
    const SUPPORTED_SAMPLING_RATES: &'static [u32] =
        &[32000, 44100, 48000, 88200, 96000, 176400, 192000];
    const SUPPORTED_SAMPLING_CLOCKS: &'static [ClkSrc] =
        &[ClkSrc::Internal, ClkSrc::WordClock, ClkSrc::Spdif];
    const CAPABILITIES: &'static [HwCap] = &[
        HwCap::ChangeableRespAddr,
        HwCap::Dsp,
        // Fixup.
        HwCap::NominalInput,
        HwCap::NominalOutput,
    ];
    const TX_CHANNEL_COUNTS: [usize; 3] = [12, 12, 12];
    const RX_CHANNEL_COUNTS: [usize; 3] = [12, 12, 12];
    const MONITOR_SOURCE_COUNT: usize = 12;
    const MONITOR_DESTINATION_COUNT: usize = 12;
    const MIDI_INPUT_COUNT: usize = 1;
    const MIDI_OUTPUT_COUNT: usize = 1;

    const PHYS_INPUT_GROUPS: &'static [(PhysGroupType, usize)] = &[(PhysGroupType::Analog, 12)];

    const PHYS_OUTPUT_GROUPS: &'static [(PhysGroupType, usize)] = &[(PhysGroupType::Analog, 12)];
}

/// Protocol implementation for later model of AudioFire 12. The higher sampling rates are
/// available only with firmware version 4 and former.
#[derive(Default, Debug)]
pub struct Audiofire12LaterProtocol;

impl EfwHardwareSpecification for Audiofire12LaterProtocol {
    const SUPPORTED_SAMPLING_RATES: &'static [u32] =
        &[32000, 44100, 48000, 88200, 96000, 176400, 192000];
    const SUPPORTED_SAMPLING_CLOCKS: &'static [ClkSrc] =
        &[ClkSrc::Internal, ClkSrc::WordClock, ClkSrc::Spdif];
    const CAPABILITIES: &'static [HwCap] = &[
        HwCap::ChangeableRespAddr,
        HwCap::Dsp,
        HwCap::InputGain,
        // Fixup.
        HwCap::NominalInput,
        HwCap::NominalOutput,
    ];
    const TX_CHANNEL_COUNTS: [usize; 3] = [12, 12, 12];
    const RX_CHANNEL_COUNTS: [usize; 3] = [12, 12, 12];
    const MONITOR_SOURCE_COUNT: usize = 12;
    const MONITOR_DESTINATION_COUNT: usize = 12;
    const MIDI_INPUT_COUNT: usize = 1;
    const MIDI_OUTPUT_COUNT: usize = 1;

    const PHYS_INPUT_GROUPS: &'static [(PhysGroupType, usize)] = &[(PhysGroupType::Analog, 12)];

    const PHYS_OUTPUT_GROUPS: &'static [(PhysGroupType, usize)] = &[(PhysGroupType::Analog, 12)];
}

/// Protocol implementation for former model of AudioFire 8.
#[derive(Default, Debug)]
pub struct Audiofire8Protocol;

impl EfwHardwareSpecification for Audiofire8Protocol {
    const SUPPORTED_SAMPLING_RATES: &'static [u32] = &[32000, 44100, 48000, 88200, 96000];
    const SUPPORTED_SAMPLING_CLOCKS: &'static [ClkSrc] =
        &[ClkSrc::Internal, ClkSrc::WordClock, ClkSrc::Spdif];
    const CAPABILITIES: &'static [HwCap] = &[
        HwCap::ChangeableRespAddr,
        HwCap::Dsp,
        // Fixup.
        HwCap::NominalInput,
        HwCap::NominalOutput,
    ];
    const TX_CHANNEL_COUNTS: [usize; 3] = [10, 10, 10];
    const RX_CHANNEL_COUNTS: [usize; 3] = [10, 10, 10];
    const MONITOR_SOURCE_COUNT: usize = 10;
    const MONITOR_DESTINATION_COUNT: usize = 10;
    const MIDI_INPUT_COUNT: usize = 1;
    const MIDI_OUTPUT_COUNT: usize = 1;

    const PHYS_INPUT_GROUPS: &'static [(PhysGroupType, usize)] =
        &[(PhysGroupType::Analog, 8), (PhysGroupType::Spdif, 2)];

    const PHYS_OUTPUT_GROUPS: &'static [(PhysGroupType, usize)] =
        &[(PhysGroupType::Analog, 8), (PhysGroupType::Spdif, 2)];
}

/// Protocol implementation for latter model of AudioFire 8 and AudioFirePre 8
#[derive(Default, Debug)]
pub struct Audiofire9Protocol;

impl EfwHardwareSpecification for Audiofire9Protocol {
    const SUPPORTED_SAMPLING_RATES: &'static [u32] = &[32000, 44100, 48000, 88200, 96000];
    const SUPPORTED_SAMPLING_CLOCKS: &'static [ClkSrc] = &[
        ClkSrc::Internal,
        ClkSrc::WordClock,
        ClkSrc::Spdif,
        ClkSrc::Adat,
    ];
    const CAPABILITIES: &'static [HwCap] = &[
        HwCap::ChangeableRespAddr,
        HwCap::OptionalSpdifCoax,
        HwCap::Fpga,
        HwCap::OptionalSpdifOpt,
        HwCap::OptionalAdatOpt,
        // Fixup.
        HwCap::NominalInput,
        HwCap::NominalOutput,
    ];
    const TX_CHANNEL_COUNTS: [usize; 3] = [16, 12, 10];
    const RX_CHANNEL_COUNTS: [usize; 3] = [16, 12, 10];
    const MONITOR_SOURCE_COUNT: usize = 16;
    const MONITOR_DESTINATION_COUNT: usize = 16;
    const MIDI_INPUT_COUNT: usize = 1;
    const MIDI_OUTPUT_COUNT: usize = 1;

    const PHYS_INPUT_GROUPS: &'static [(PhysGroupType, usize)] =
        &[(PhysGroupType::Analog, 8), (PhysGroupType::SpdifOrAdat, 8)];

    const PHYS_OUTPUT_GROUPS: &'static [(PhysGroupType, usize)] =
        &[(PhysGroupType::Analog, 8), (PhysGroupType::SpdifOrAdat, 8)];
}

impl EfwDigitalModeSpecification for Audiofire9Protocol {}

/// Protocol implementation for Audiofire 4.
#[derive(Default, Debug)]
pub struct Audiofire4Protocol;

impl EfwHardwareSpecification for Audiofire4Protocol {
    const SUPPORTED_SAMPLING_RATES: &'static [u32] = &[32000, 44100, 48000, 88200, 96000];
    const SUPPORTED_SAMPLING_CLOCKS: &'static [ClkSrc] = &[ClkSrc::Internal, ClkSrc::Spdif];
    const CAPABILITIES: &'static [HwCap] = &[
        HwCap::ChangeableRespAddr,
        HwCap::PhantomPowering,
        HwCap::OutputMapping,
        // Fixup.
        HwCap::NominalInput,
        HwCap::NominalOutput,
    ];
    const TX_CHANNEL_COUNTS: [usize; 3] = [6, 6, 6];
    const RX_CHANNEL_COUNTS: [usize; 3] = [6, 6, 6];
    const MONITOR_SOURCE_COUNT: usize = 6;
    const MONITOR_DESTINATION_COUNT: usize = 6;
    const MIDI_INPUT_COUNT: usize = 1;
    const MIDI_OUTPUT_COUNT: usize = 1;

    const PHYS_INPUT_GROUPS: &'static [(PhysGroupType, usize)] =
        &[(PhysGroupType::Analog, 4), (PhysGroupType::Spdif, 2)];

    const PHYS_OUTPUT_GROUPS: &'static [(PhysGroupType, usize)] =
        &[(PhysGroupType::Analog, 4), (PhysGroupType::Spdif, 2)];
}

impl EfwPhantomPoweringSpecification for Audiofire4Protocol {}

/// Protocol implementation for Audiofire 2.
#[derive(Default, Debug)]
pub struct Audiofire2Protocol;

impl EfwHardwareSpecification for Audiofire2Protocol {
    const SUPPORTED_SAMPLING_RATES: &'static [u32] = &[32000, 44100, 48000, 88200, 96000];
    const SUPPORTED_SAMPLING_CLOCKS: &'static [ClkSrc] = &[ClkSrc::Internal, ClkSrc::Spdif];
    const CAPABILITIES: &'static [HwCap] = &[
        HwCap::ChangeableRespAddr,
        HwCap::Fpga,
        HwCap::PhantomPowering,
        HwCap::OutputMapping,
        // Fixup.
        HwCap::NominalInput,
        HwCap::NominalOutput,
    ];
    const TX_CHANNEL_COUNTS: [usize; 3] = [4, 4, 4];
    const RX_CHANNEL_COUNTS: [usize; 3] = [6, 6, 6];
    const MONITOR_SOURCE_COUNT: usize = 4;
    const MONITOR_DESTINATION_COUNT: usize = 6;
    const MIDI_INPUT_COUNT: usize = 1;
    const MIDI_OUTPUT_COUNT: usize = 1;

    const PHYS_INPUT_GROUPS: &'static [(PhysGroupType, usize)] =
        &[(PhysGroupType::Analog, 2), (PhysGroupType::Spdif, 2)];

    const PHYS_OUTPUT_GROUPS: &'static [(PhysGroupType, usize)] = &[
        (PhysGroupType::Analog, 2),
        (PhysGroupType::Headphones, 2),
        (PhysGroupType::Spdif, 2),
    ];
}
