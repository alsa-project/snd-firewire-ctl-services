// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2023 Takashi Sakamoto

//! Protocol implementations for Echo Audio Audiofire series.
//!
//! The module includes protocol about port configuration defined by Echo Audio Digital Corporation
//! for Audiofire.

use super::{phys_input::*, phys_output::*, port_conf::*, *};

/// Protocol implementation for former model of AudioFire 12. The higher sampling rates are
/// available only with firmware version 4 and former.
///
/// Diagram of internal signal flow
///
/// ```text
/// analog-input-1/2 ---------+----------------> stream-output-1/2
/// analog-input-3/4 ---------|-+--------------> stream-output-3/4
/// analog-input-5/6 ---------|-|-+------------> stream-output-5/6
/// analog-input-7/8 ---------|-|-|-+----------> stream-output-7/8
/// analog-input-9/10 --------|-|-|-|----------> stream-output-9/10
/// analog-input-11/12 -------|-|-|-|-+--------> stream-output-11/12
///                           | | | | |
///                           v v v v v
///                        ++===========++
/// stream-input-1/2 ----> ||           || ----> analog-output-1/2
/// stream-input-3/4 ----> ||   mixer   || ----> analog-output-3/4
/// stream-input-5/6 ----> ||           || ----> analog-output-5/6
/// stream-input-7/8 ----> ||  24 x 12  || ----> analog-output-7/8
/// stream-input-9/10 ---> ||           || ----> analog-output-9/10
/// stream-input-11/12 --> ||           || ----> analog-output-11/12
///                        ++===========++
/// ```
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

impl EfwPhysInputSpecification for Audiofire12FormerProtocol {}

impl EfwPhysOutputSpecification for Audiofire12FormerProtocol {}

impl EfwPlaybackSoloSpecification for Audiofire12FormerProtocol {}

/// Protocol implementation for later model of AudioFire 12. The higher sampling rates are
/// available only with firmware version 4 and former.
///
/// Diagram of internal signal flow
///
/// ```text
/// analog-input-1/2 ---------+----------------> stream-output-1/2
/// analog-input-3/4 ---------|-+--------------> stream-output-3/4
/// analog-input-5/6 ---------|-|-+------------> stream-output-5/6
/// analog-input-7/8 ---------|-|-|-+----------> stream-output-7/8
/// analog-input-9/10 --------|-|-|-|----------> stream-output-9/10
/// analog-input-11/12 -------|-|-|-|-+--------> stream-output-11/12
///                           | | | | |
///                           v v v v v
///                        ++===========++
/// stream-input-1/2 ----> ||           || ----> analog-output-1/2
/// stream-input-3/4 ----> ||   mixer   || ----> analog-output-3/4
/// stream-input-5/6 ----> ||           || ----> analog-output-5/6
/// stream-input-7/8 ----> ||  24 x 12  || ----> analog-output-7/8
/// stream-input-9/10 ---> ||           || ----> analog-output-9/10
/// stream-input-11/12 --> ||           || ----> analog-output-11/12
///                        ++===========++
/// ```
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

impl EfwPhysInputSpecification for Audiofire12LaterProtocol {}

impl EfwPhysOutputSpecification for Audiofire12LaterProtocol {}

impl EfwPlaybackSoloSpecification for Audiofire12LaterProtocol {}

/// Protocol implementation for former model of AudioFire 8.
///
/// Diagram of internal signal flow
///
/// ```text
/// analog-input-1/2 ---------+----------------> stream-output-1/2
/// analog-input-3/4 ---------|-+--------------> stream-output-3/4
/// analog-input-5/6 ---------|-|-+------------> stream-output-5/6
/// analog-input-7/8 ---------|-|-|-+----------> stream-output-7/8
///                           | | | |
/// coaxial-input-1/2 --------|-|-|-|-+--------> stream-output-9/10
///                           | | | | |
///                           v v v v v
///                        ++===========++
/// stream-input-1/2 ----> ||           || ----> analog-output-1/2
/// stream-input-3/4 ----> ||   mixer   || ----> analog-output-3/4
/// stream-input-5/6 ----> ||           || ----> analog-output-5/6
/// stream-input-7/8 ----> ||  20 x 10  || ----> analog-output-7/8
/// stream-input-9/10 ---> ||           || ----> coaxial-output-1/2
///                        ++===========++
/// ```
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

impl EfwPhysInputSpecification for Audiofire8Protocol {}

impl EfwPhysOutputSpecification for Audiofire8Protocol {}

impl EfwPlaybackSoloSpecification for Audiofire8Protocol {}

/// Protocol implementation for latter model of AudioFire 8 and AudioFirePre 8
///
/// Diagram of internal signal flow
///
/// ```text
///
/// analog-input-1/2 ---------+----------------------------------------------> stream-output-1/2
/// analog-input-3/4 ---------|-+--------------------------------------------> stream-output-3/4
/// analog-input-5/6 ---------|-|-+------------------------------------------> stream-output-5/6
/// analog-input-7/8 ---------|-|-|-+----------------------------------------> stream-output-7/8
///                           | | | |
/// coaxial-input-1/2 --+     | | | |
/// optical-input-1/2 --or----|-|-|-|-+--------------------------------------> stream-output-9/10
/// optical-input-3/4 --------|-|-|-|-|-+------------------------------------> stream-output-11/12
/// optical-input-5/6 --------|-|-|-|-|-|-+----------------------------------> stream-output-13/14
/// optical-input-7/8 --------|-|-|-|-|-|-|-+--------------------------------> stream-output-15/16
///                           | | | | | | | |
///                           v v v v v v v v
///                        ++=================++
/// stream-input-1/2 ----> ||                 || ----------------------------> analog-output-1/2
/// stream-input-3/4 ----> ||                 || ----------------------------> analog-output-3/4
/// stream-input-5/6 ----> ||      mixer      || ----------------------------> analog-output-5/6
/// stream-input-7/8 ----> ||                 || ----------------------------> analog-output-7/8
///                        ||                 ||                          +--> coaxial-output-1/2
/// stream-input-9/10 ---> ||                 || --> digital-output-1/2 --or-> optical-output-1/2
/// stream-input-11/12 --> ||     32 x 16     || --> digital-output-3/4 -----> optical-output-3/4
/// stream-input-13/14 --> ||                 || --> digital-output-5/6 -----> optical-output-5/6
/// stream-input-15/16 --> ||                 || --> digital-output-7/8 -----> optical-output-7/8
///                        ++=================++
/// ```
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

impl EfwPhysInputSpecification for Audiofire9Protocol {}

impl EfwPhysOutputSpecification for Audiofire9Protocol {}

impl EfwPlaybackSoloSpecification for Audiofire9Protocol {}

/// Protocol implementation for Audiofire 4.
///
/// Diagram of internal signal flow
///
/// ```text
///
/// analog-input-1/2 --------+--------------------------------> stream-output-1/2
/// analog-input-3/4 --------|--+-----------------------------> stream-output-3/4
///                          |  |
/// spdif-input-1/2 ---------|--|--+--------------------------> stream-output-5/6
///                          |  |  |
///                          v  v  v
///                       ++==========++      ++========++
/// stream-input-1/2 ---> ||  mixer   || ---> || router || ---> analog-output-1/2
/// stream-input-3/4 ---> ||          || ---> ||        || ---> analog-output-3/4
/// stream-input-5/6 ---> ||  12 x 6  || ---> || 6 x 6  || ---> spdif-output-1/2
///                       ++==========++      ++========++
/// ```
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

impl EfwRxStreamMapsSpecification for Audiofire4Protocol {}

impl EfwPhysInputSpecification for Audiofire4Protocol {}

impl EfwPhysOutputSpecification for Audiofire4Protocol {}

impl EfwPlaybackSoloSpecification for Audiofire4Protocol {}

/// Protocol implementation for Audiofire 2.
///
/// Diagram of internal signal flow
///
/// ```text
///
/// analog-input-1/2 ---------+-------------------------------> stream-output-1/2
///                           |
/// spdif-input-1/2 ----------|----+--------------------------> stream-output-3/4
///                           |    |
///                           v    v
///                       ++==========++      ++========++
/// stream-input-1/2 ---> ||  mixer   || ---> || router || ---> analog-output-1/2
/// stream-input-3/4 ---> ||          || ---> ||        || ---> headphone-output-1/2
/// stream-input-5/6 ---> ||  10 x 6  || ---> || 6 x 6  || ---> spdif-output-1/2
///                       ++==========++      ++========++
/// ```
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

impl EfwRxStreamMapsSpecification for Audiofire2Protocol {}

impl EfwPhysInputSpecification for Audiofire2Protocol {}

impl EfwPhysOutputSpecification for Audiofire2Protocol {}

impl EfwPlaybackSoloSpecification for Audiofire2Protocol {}
