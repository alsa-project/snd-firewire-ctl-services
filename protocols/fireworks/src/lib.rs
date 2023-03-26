// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

#![doc = include_str!("../README.md")]

pub mod flash;
pub mod hw_ctl;
pub mod hw_info;
pub mod monitor;
pub mod phys_input;
pub mod phys_output;
pub mod playback;
pub mod port_conf;
pub mod robot_guitar;
pub mod transaction;
pub mod transport;

pub mod audiofire;
pub mod onyx_f;

pub mod rip;

use {
    glib::{Error, FileError},
    hitaki::{prelude::EfwProtocolExtManual, EfwProtocolError},
    hw_info::HwMeter,
    monitor::{EfwMonitorParameters, EfwMonitorSourceParameters},
    phys_output::EfwOutputParameters,
    playback::{EfwPlaybackParameters, EfwPlaybackSoloSpecification},
};

/// The specification of hardware.
pub trait EfwHardwareSpecification {
    /// The list of supported sampling transfer frequencies.
    const SUPPORTED_SAMPLING_RATES: &'static [u32];
    /// The list of supported sources of sampling clock.
    const SUPPORTED_SAMPLING_CLOCKS: &'static [ClkSrc];
    /// The list of hardware capabilities.
    const CAPABILITIES: &'static [HwCap];
    /// The number of audio channels in received isochronous stream at each rate mode.
    const RX_CHANNEL_COUNTS: [usize; 3];
    /// The number of audio channels in transmitted isochronous stream at each rate mode.
    const TX_CHANNEL_COUNTS: [usize; 3];
    /// The total number of monitor inputs.
    const MONITOR_SOURCE_COUNT: usize;
    /// The total number of monitor outputs.
    const MONITOR_DESTINATION_COUNT: usize;
    /// The number of MIDI input port.
    const MIDI_INPUT_COUNT: usize;
    /// The number of MIDI output port.
    const MIDI_OUTPUT_COUNT: usize;

    const PHYS_INPUT_GROUPS: &'static [(PhysGroupType, usize)];

    const PHYS_OUTPUT_GROUPS: &'static [(PhysGroupType, usize)];

    /// The total number of physical audio inputs.
    fn phys_input_count() -> usize {
        Self::PHYS_INPUT_GROUPS
            .iter()
            .fold(0, |count, entry| count + entry.1)
    }

    /// The total number of physical audio outputs.
    fn phys_output_count() -> usize {
        Self::PHYS_OUTPUT_GROUPS
            .iter()
            .fold(0, |count, entry| count + entry.1)
    }

    fn create_hardware_meter() -> HwMeter {
        HwMeter {
            detected_clk_srcs: Self::SUPPORTED_SAMPLING_CLOCKS
                .iter()
                .map(|&src| (src, Default::default()))
                .collect(),
            detected_midi_inputs: Default::default(),
            detected_midi_outputs: Default::default(),
            guitar_charging: Default::default(),
            guitar_stereo_connect: Default::default(),
            guitar_hex_signal: Default::default(),
            phys_output_meters: vec![Default::default(); Self::phys_output_count()],
            phys_input_meters: vec![Default::default(); Self::phys_input_count()],
        }
    }

    fn create_monitor_parameters() -> EfwMonitorParameters {
        EfwMonitorParameters(vec![
            EfwMonitorSourceParameters {
                gains: vec![Default::default(); Self::MONITOR_SOURCE_COUNT],
                mutes: vec![Default::default(); Self::MONITOR_SOURCE_COUNT],
                solos: vec![Default::default(); Self::MONITOR_SOURCE_COUNT],
                pans: vec![Default::default(); Self::MONITOR_SOURCE_COUNT],
            };
            Self::MONITOR_DESTINATION_COUNT
        ])
    }

    fn create_output_parameters() -> EfwOutputParameters {
        EfwOutputParameters {
            volumes: vec![Default::default(); Self::phys_output_count()],
            mutes: vec![Default::default(); Self::phys_output_count()],
        }
    }

    fn create_playback_parameters() -> EfwPlaybackParameters {
        EfwPlaybackParameters {
            volumes: vec![Default::default(); Self::RX_CHANNEL_COUNTS[0]],
            mutes: vec![Default::default(); Self::RX_CHANNEL_COUNTS[0]],
        }
    }
}

/// Cache whole parameters.
pub trait EfwWhollyCachableParamsOperation<P, T>
where
    P: EfwProtocolExtManual,
{
    fn cache_wholly(proto: &mut P, states: &mut T, timeout_ms: u32) -> Result<(), Error>;
}

/// Update the part of parameters.
pub trait EfwPartiallyUpdatableParamsOperation<P, T>
where
    P: EfwProtocolExtManual,
{
    fn update_partially(
        proto: &mut P,
        params: &mut T,
        update: T,
        timeout_ms: u32,
    ) -> Result<(), Error>;
}

/// Update whole parameters.
pub trait EfwWhollyUpdatableParamsOperation<P, T>
where
    P: EfwProtocolExtManual,
{
    fn update_wholly(proto: &mut P, states: &T, timeout_ms: u32) -> Result<(), Error>;
}

/// Signal source of sampling clock.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ClkSrc {
    Internal,
    WordClock,
    Spdif,
    Adat,
    Adat2,
    Continuous,
    Reserved(u32),
}

impl Default for ClkSrc {
    fn default() -> Self {
        Self::Reserved(u32::MAX)
    }
}

fn serialize_clock_source(src: &ClkSrc) -> u32 {
    match src {
        ClkSrc::Internal => 0,
        // blank.
        ClkSrc::WordClock => 2,
        ClkSrc::Spdif => 3,
        ClkSrc::Adat => 4,
        ClkSrc::Adat2 => 5,
        ClkSrc::Continuous => 6,
        ClkSrc::Reserved(val) => *val,
    }
}

fn deserialize_clock_source(src: &mut ClkSrc, val: u32) {
    *src = match val {
        0 => ClkSrc::Internal,
        // blank.
        2 => ClkSrc::WordClock,
        3 => ClkSrc::Spdif,
        4 => ClkSrc::Adat,
        5 => ClkSrc::Adat2,
        6 => ClkSrc::Continuous,
        _ => ClkSrc::Reserved(val),
    };
}

/// Hardware capability.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum HwCap {
    /// The address to which response of transaction is transmitted is configurable.
    ChangeableRespAddr,
    /// Has control room for output mirror.
    ControlRoom,
    /// S/PDIF signal is available for coaxial interface as option.
    OptionalSpdifCoax,
    /// S/PDIF signal is available for AES/EBU XLR interface as option.
    OptionalAesebuXlr,
    /// Has DSP (Texas Instrument TMS320C67).
    Dsp,
    /// Has FPGA (Xilinx Spartan XC35250E).
    Fpga,
    /// Support phantom powering for any mic input.
    PhantomPowering,
    /// Support mapping between playback stream and physical output.
    OutputMapping,
    /// The gain of physical input is adjustable.
    InputGain,
    /// S/PDIF signal is available for optical interface as option.
    OptionalSpdifOpt,
    /// ADAT signal is available for optical interface as option.
    OptionalAdatOpt,
    /// The nominal level of input audio signal is selectable.
    NominalInput,
    /// The nominal level of output audio signal is selectable.
    NominalOutput,
    /// Has software clipping for input audio signal.
    SoftClip,
    /// Is robot guitar.
    RobotGuitar,
    /// Support chaging for guitar.
    GuitarCharging,
    Reserved(usize),
    #[doc(hidden)]
    // For my purpose.
    InputMapping,
    PlaybackSoloUnsupported,
}

impl Default for HwCap {
    fn default() -> Self {
        Self::Reserved(usize::MAX)
    }
}

#[cfg(test)]
fn serialize_hw_cap(cap: &HwCap) -> usize {
    match cap {
        HwCap::ChangeableRespAddr => 0,
        HwCap::ControlRoom => 1,
        HwCap::OptionalSpdifCoax => 2,
        HwCap::OptionalAesebuXlr => 3,
        HwCap::Dsp => 4,
        HwCap::Fpga => 5,
        HwCap::PhantomPowering => 6,
        HwCap::OutputMapping => 7,
        HwCap::InputGain => 8,
        HwCap::OptionalSpdifOpt => 9,
        HwCap::OptionalAdatOpt => 10,
        HwCap::NominalInput => 11,
        HwCap::NominalOutput => 12,
        HwCap::SoftClip => 13,
        HwCap::RobotGuitar => 14,
        HwCap::GuitarCharging => 15,
        HwCap::InputMapping => 3000,
        HwCap::PlaybackSoloUnsupported => 3001,
        HwCap::Reserved(pos) => *pos,
    }
}

fn deserialize_hw_cap(cap: &mut HwCap, pos: usize) {
    *cap = match pos {
        0 => HwCap::ChangeableRespAddr,
        1 => HwCap::ControlRoom,
        2 => HwCap::OptionalSpdifCoax,
        3 => HwCap::OptionalAesebuXlr,
        4 => HwCap::Dsp,
        5 => HwCap::Fpga,
        6 => HwCap::PhantomPowering,
        7 => HwCap::OutputMapping,
        8 => HwCap::InputGain,
        9 => HwCap::OptionalSpdifOpt,
        10 => HwCap::OptionalAdatOpt,
        11 => HwCap::NominalInput,
        12 => HwCap::NominalOutput,
        13 => HwCap::SoftClip,
        14 => HwCap::RobotGuitar,
        15 => HwCap::GuitarCharging,
        3000 => HwCap::InputMapping,
        3001 => HwCap::PlaybackSoloUnsupported,
        _ => HwCap::Reserved(pos),
    };
}

/// Nominal level of audio signal.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum NominalSignalLevel {
    /// +4 dBu.
    Professional,
    Medium,
    /// -10 dBV.
    Consumer,
}

impl Default for NominalSignalLevel {
    fn default() -> Self {
        Self::Professional
    }
}

fn serialize_nominal_signal_level(level: &NominalSignalLevel) -> u32 {
    match level {
        NominalSignalLevel::Consumer => 2,
        NominalSignalLevel::Medium => 1,
        NominalSignalLevel::Professional => 0,
    }
}

fn deserialize_nominal_signal_level(level: &mut NominalSignalLevel, val: u32) {
    *level = match val {
        2 => NominalSignalLevel::Consumer,
        1 => NominalSignalLevel::Medium,
        _ => NominalSignalLevel::Professional,
    };
}

/// Type of physical group.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PhysGroupType {
    Analog,
    Spdif,
    Adat,
    SpdifOrAdat,
    AnalogMirror,
    Headphones,
    I2s,
    Guitar,
    PiezoGuitar,
    GuitarString,
    Unknown(u8),
}

#[cfg(test)]
fn serialize_phys_group_type(group_type: &PhysGroupType) -> u8 {
    match group_type {
        PhysGroupType::Analog => 0,
        PhysGroupType::Spdif => 1,
        PhysGroupType::Adat => 2,
        PhysGroupType::SpdifOrAdat => 3,
        PhysGroupType::AnalogMirror => 4,
        PhysGroupType::Headphones => 5,
        PhysGroupType::I2s => 6,
        PhysGroupType::Guitar => 7,
        PhysGroupType::PiezoGuitar => 8,
        PhysGroupType::GuitarString => 9,
        PhysGroupType::Unknown(val) => *val,
    }
}

fn deserialize_phys_group_type(group_type: &mut PhysGroupType, val: u8) {
    *group_type = match val {
        0 => PhysGroupType::Analog,
        1 => PhysGroupType::Spdif,
        2 => PhysGroupType::Adat,
        3 => PhysGroupType::SpdifOrAdat,
        4 => PhysGroupType::AnalogMirror,
        5 => PhysGroupType::Headphones,
        6 => PhysGroupType::I2s,
        7 => PhysGroupType::Guitar,
        8 => PhysGroupType::PiezoGuitar,
        9 => PhysGroupType::GuitarString,
        _ => PhysGroupType::Unknown(val),
    };
}

impl Default for PhysGroupType {
    fn default() -> Self {
        Self::Unknown(u8::MAX)
    }
}

/// Entry of physical group.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct PhysGroupEntry {
    pub group_type: PhysGroupType,
    pub group_count: usize,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn clock_source_serdes() {
        [
            ClkSrc::Internal,
            ClkSrc::WordClock,
            ClkSrc::Spdif,
            ClkSrc::Adat,
            ClkSrc::Adat2,
            ClkSrc::Continuous,
            ClkSrc::default(),
        ]
        .iter()
        .for_each(|src| {
            let val = serialize_clock_source(&src);
            let mut s = ClkSrc::default();
            deserialize_clock_source(&mut s, val);
            assert_eq!(*src, s);
        });
    }

    #[test]
    fn nominal_signal_level_serdes() {
        [
            NominalSignalLevel::Professional,
            NominalSignalLevel::Medium,
            NominalSignalLevel::Consumer,
        ]
        .iter()
        .for_each(|level| {
            let val = serialize_nominal_signal_level(&level);
            let mut l = NominalSignalLevel::default();
            deserialize_nominal_signal_level(&mut l, val);
            assert_eq!(*level, l);
        });
    }

    #[test]
    fn phys_group_type_serdes() {
        [
            PhysGroupType::Analog,
            PhysGroupType::Spdif,
            PhysGroupType::Adat,
            PhysGroupType::SpdifOrAdat,
            PhysGroupType::AnalogMirror,
            PhysGroupType::Headphones,
            PhysGroupType::I2s,
            PhysGroupType::Guitar,
            PhysGroupType::PiezoGuitar,
            PhysGroupType::GuitarString,
            PhysGroupType::default(),
        ]
        .iter()
        .for_each(|group_type| {
            let val = serialize_phys_group_type(&group_type);
            let mut t = PhysGroupType::default();
            deserialize_phys_group_type(&mut t, val);
            assert_eq!(*group_type, t);
        });
    }

    #[test]
    fn hw_cap_serdes() {
        [
            HwCap::ChangeableRespAddr,
            HwCap::ControlRoom,
            HwCap::OptionalSpdifCoax,
            HwCap::OptionalAesebuXlr,
            HwCap::Dsp,
            HwCap::Fpga,
            HwCap::PhantomPowering,
            HwCap::OutputMapping,
            HwCap::InputGain,
            HwCap::OptionalSpdifOpt,
            HwCap::OptionalAdatOpt,
            HwCap::NominalInput,
            HwCap::NominalOutput,
            HwCap::SoftClip,
            HwCap::RobotGuitar,
            HwCap::GuitarCharging,
            HwCap::InputMapping,
            HwCap::PlaybackSoloUnsupported,
            HwCap::default(),
        ]
        .iter()
        .for_each(|cap| {
            let val = serialize_hw_cap(&cap);
            let mut c = HwCap::default();
            deserialize_hw_cap(&mut c, val);
            assert_eq!(*cap, c);
        });
    }
}
