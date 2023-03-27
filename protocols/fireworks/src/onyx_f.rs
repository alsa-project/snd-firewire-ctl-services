// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2023 Takashi Sakamoto

//! Protocol implementations for Mackie Onyx-F series.
//!
//! The module includes protocol about port configuration defined by Echo Audio Digital Corporation
//! for Mackie Onyx-F series.

use super::{port_conf::*, *};

/// Protocol implementation for former model of Onyx 1200F.
///
/// Diagram of internal signal flow
///
/// ```text
///
/// analog-input-1/2 ---------+------------------------------------------> stream-output-1/2
/// analog-input-3/4 ---------|-+----------------------------------------> stream-output-3/4
/// analog-input-5/6 ---------|-|-+--------------------------------------> stream-output-5/6
/// analog-input-7/8 ---------|-|-|-+------------------------------------> stream-output-7/8
/// analog-input-9/10 --------|-|-|-|-+----------------------------------> stream-output-9/10
/// analog-input-11/12 -------|-|-|-|-|-+--------------------------------> stream-output-11/12
///                           | | | | | |
/// optical-input-A-1/2 ------|-|-|-|-|-|-+------------------------------> stream-output-13/14
/// optical-input-A-3/4 ------|-|-|-|-|-|-|-+----------------------------> stream-output-15/16
/// optical-input-A-5/6 ------|-|-|-|-|-|-|-|-+--------------------------> stream-output-17/18
/// optical-input-A-7/8 ------|-|-|-|-|-|-|-|-|-+------------------------> stream-output-19/20
///                           | | | | | | | | | |
/// optical-input-B-1/2 ------|-|-|-|-|-|-|-|-|-|-+----------------------> stream-output-21/22
/// optical-input-B-3/4 ------|-|-|-|-|-|-|-|-|-|-|-+--------------------> stream-output-23/24
/// optical-input-B-5/6 ------|-|-|-|-|-|-|-|-|-|-|-|-+------------------> stream-output-25/26
/// optical-input-B-7/8 ------|-|-|-|-|-|-|-|-|-|-|-|-|-+----------------> stream-output-27/28
///                           | | | | | | | | | | | | | |
/// coaxial-input-1/2 ---or---|-|-|-|-|-|-|-|-|-|-|-|-|-|-+--------------> stream-output-29/30
/// XLR-input-1/2 -------+    | | | | | | | | | | | | | | |
///                           | | | | | | | | | | | | | | |
///                           v v v v v v v v v v v v v v v
///                        ++===============================++
/// stream-input-1/2 ----> ||                               || --+-------> analog-output-1/2
/// stream-input-3/4 ----> ||                               || --+-------> analog-output-3/4
/// stream-input-5/6 ----> ||             mixer             || --+-------> analog-output-5/6
/// stream-input-7/8 ----> ||                               || --+-------> analog-output-7/8
///                        ||            64 x 34            ||   |
/// stream-input-9/10 ---> ||                               || --|-------> adat-output-A-1/2
/// stream-input-11/12 --> ||                               || --|-------> adat-output-A-3/4
/// stream-input-13/14 --> ||                               || --|-------> adat-output-A-5/6
/// stream-input-15/16 --> ||                               || --|-------> adat-output-A-7/8
///                        ||                               ||   |
/// stream-input-17/18 --> ||                               || --|-------> adat-output-A-1/2
/// stream-input-19/20 --> ||                               || --|-------> adat-output-A-3/4
/// stream-input-21/22 --> ||                               || --|-------> adat-output-A-5/6
/// stream-input-23/24 --> ||                               || --|-------> adat-output-A-7/8
///                        ||                               ||   |
/// stream-input-25/26 --> ||                               || --+-------> headphone-output-1/2
/// stream-input-27/28 --> ||                               || --+-------> headphone-output-3/4
/// stream-input-29/30 --> ||                               || --+-------> headphone-output-5/6
/// stream-input-31/32 --> ||                               || --+-------> headphone-output-7/8
///                        ||                               ||   |
/// stream-input-33/34 --> ||                               || --+---or--> coaxial-output-7/8
///                        ++===============================++   |    +--> XLR-output-1/2
///                                                           (one of)
///                                                              +-------> control-room-1/2
/// ```
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

impl EfwDigitalModeSpecification for Onyx1200fProtocol {}

/// Protocol implementation for Mackie Onyx 400F. The higher sampling rates are available only with
/// firmware version 4 and former.
///
/// Diagram of internal signal flow
///
/// ```text
///
/// analog-input-1/2 ---------+----------------------> stream-output-1/2
/// analog-input-3/4 ---------|-+--------------------> stream-output-3/4
/// analog-input-5/6 ---------|-|-+------------------> stream-output-5/6
/// analog-input-7/8 ---------|-|-|-+----------------> stream-output-7/8
///                           | | | |
/// coaxial-input-1/2 --------|-|-|-|-+--------------> stream-output-9/10
///                           | | | | |
///                           v v v v v
///                        ++===========++
/// stream-input-1/2 ----> ||           || -----+-> analog-output-1/2
/// stream-input-3/4 ----> ||           || -----+-> analog-output-3/4
/// stream-input-5/6 ----> ||   mixer   || -----+-> analog-output-5/6
/// stream-input-7/8 ----> ||  20 x 20  || -----+-> analog-output-7/8
///                        ||           ||      |
/// stream-input-9/10 ---> ||           || -----+-> coaxial-output-1/2
///                        ++===========++      |
///                                         (one of)
///                                             +-> control-room-1/2
/// ```
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
