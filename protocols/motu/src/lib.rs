// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

#![doc = include_str!("../README.md")]

pub mod command_dsp;
pub mod config_rom;
pub mod register_dsp;
pub mod version_1;
pub mod version_2;
pub mod version_3;

use {
    glib::{Error, FileError},
    hinawa::{
        prelude::{FwNodeExt, FwReqExtManual},
        FwNode, FwReq, FwTcode,
    },
    std::{thread, time},
};

/// The trait to operate cacheable parameters at once.
pub trait MotuWhollyCacheableParamsOperation<T> {
    /// Cache whole parameters.
    fn cache_wholly(
        req: &mut FwReq,
        node: &mut FwNode,
        params: &mut T,
        timeout_ms: u32,
    ) -> Result<(), Error>;
}

/// The trait to operate updatable parameters at once.
pub trait MotuWhollyUpdatableParamsOperation<T> {
    /// Update whole parameters.
    fn update_wholly(
        req: &mut FwReq,
        node: &mut FwNode,
        params: &T,
        timeout_ms: u32,
    ) -> Result<(), Error>;
}

/// The trait to operate updatable parameters partially.
pub trait MotuPartiallyUpdatableParamsOperation<T> {
    /// Update the part of parameters.
    fn update_partially(
        req: &mut FwReq,
        node: &mut FwNode,
        params: &mut T,
        updates: T,
        timeout_ms: u32,
    ) -> Result<(), Error>;
}

const BASE_OFFSET: u64 = 0xfffff0000000;
const OFFSET_CLK: u32 = 0x0b14;
const OFFSET_PORT: u32 = 0x0c04;
const OFFSET_CLK_DISPLAY: u32 = 0x0c60;

fn read_quad(req: &FwReq, node: &mut FwNode, offset: u32, timeout_ms: u32) -> Result<u32, Error> {
    let mut frame = [0; 4];
    req.transaction_sync(
        node,
        FwTcode::ReadQuadletRequest,
        BASE_OFFSET + offset as u64,
        4,
        &mut frame,
        timeout_ms,
    )
    .map(|_| u32::from_be_bytes(frame))
}

// AudioExpress sometimes transfers response subaction with non-standard rcode. This causes
// Linux firewire subsystem to report 'unsolicited response' error. In the case, send error
// is reported to userspace applications. As a workaround, the change of register is ensured
// by following read transaction in failure of write transaction.
fn write_quad(
    req: &FwReq,
    node: &mut FwNode,
    offset: u32,
    quad: u32,
    timeout_ms: u32,
) -> Result<(), Error> {
    let mut frame = [0; 4];
    frame.copy_from_slice(&quad.to_be_bytes());
    req.transaction_sync(
        node,
        FwTcode::WriteQuadletRequest,
        BASE_OFFSET + offset as u64,
        4,
        &mut frame,
        timeout_ms,
    )
    .or_else(|err| {
        // For prevention of RCODE_BUSY.
        thread::sleep(time::Duration::from_millis(BUSY_DURATION));
        req.transaction_sync(
            node,
            FwTcode::WriteQuadletRequest,
            BASE_OFFSET + offset as u64,
            4,
            &mut frame,
            timeout_ms,
        )
        .and_then(|_| {
            if u32::from_be_bytes(frame) == quad {
                Ok(())
            } else {
                Err(err)
            }
        })
    })
}

fn serialize_flag<T: Copy + Eq>(
    flag: &T,
    quad: &mut u32,
    mask: u32,
    shift: usize,
    flags: &[T],
    vals: &[u8],
    label: &str,
) -> Result<(), Error> {
    flags
        .iter()
        .zip(vals)
        .find(|(f, _)| flag.eq(f))
        .ok_or_else(|| {
            let label = format!(
                "Invalid argument for {}, 0x{:08x}, 0x{:08x}",
                label, *quad, mask
            );
            Error::new(FileError::Io, &label)
        })
        .map(|(_, &val)| {
            *quad &= !mask;
            *quad |= (val as u32) << shift;
        })
}

fn deserialize_flag<T: Copy + Eq>(
    flag: &mut T,
    quad: &u32,
    mask: u32,
    shift: usize,
    flags: &[T],
    vals: &[u8],
    label: &str,
) -> Result<(), Error> {
    let val = ((*quad & mask) >> shift) as u8;
    flags
        .iter()
        .zip(vals)
        .find(|(_, v)| val.eq(v))
        .ok_or_else(|| {
            let label = format!(
                "Invalid value for {}, 0x{:08x}, 0x{:08x}",
                label, quad, mask
            );
            Error::new(FileError::Io, &label)
        })
        .map(|(&f, _)| *flag = f)
}

/// Nominal rate of sampling clock.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ClkRate {
    /// 44.1 kHx.
    R44100,
    /// 48.0 kHx.
    R48000,
    /// 88.2 kHx.
    R88200,
    /// 96.0 kHx.
    R96000,
    /// 176.4 kHx.
    R176400,
    /// 192.2 kHx.
    R192000,
}

impl Default for ClkRate {
    fn default() -> Self {
        Self::R44100
    }
}

const BUSY_DURATION: u64 = 150;
const DISPLAY_CHARS: usize = 4 * 4;

/// Parameters of clock name in LCD display.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct ClockNameDisplayParameters(pub String);

/// The trait for specification of LCD to display clock name.
pub trait MotuClockNameDisplaySpecification {}

impl<O> MotuWhollyUpdatableParamsOperation<ClockNameDisplayParameters> for O
where
    O: MotuClockNameDisplaySpecification,
{
    fn update_wholly(
        req: &mut FwReq,
        node: &mut FwNode,
        params: &ClockNameDisplayParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut chars = [0x20; DISPLAY_CHARS];
        chars
            .iter_mut()
            .zip(params.0.bytes())
            .for_each(|(c, l)| *c = l);

        (0..(DISPLAY_CHARS / 4)).try_for_each(|i| {
            let mut frame = [0; 4];
            frame.copy_from_slice(&chars[(i * 4)..(i * 4 + 4)]);
            frame.reverse();
            let quad = u32::from_ne_bytes(frame);
            let offset = OFFSET_CLK_DISPLAY + 4 * i as u32;
            write_quad(req, node, offset, quad, timeout_ms)
        })
    }
}

/// The trait for specification of port assignment.
pub trait MotuPortAssignSpecification {
    const ASSIGN_PORT_TARGETS: &'static [TargetPort];
    const ASSIGN_PORT_VALS: &'static [u8];
}

const PORT_PHONE_LABEL: &str = "phone-assign";
const PORT_PHONE_MASK: u32 = 0x0000000f;
const PORT_PHONE_SHIFT: usize = 0;

/// The parameters of phone assignments.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct PhoneAssignParameters(pub TargetPort);

impl<O> MotuWhollyCacheableParamsOperation<PhoneAssignParameters> for O
where
    O: MotuPortAssignSpecification,
{
    fn cache_wholly(
        req: &mut FwReq,
        node: &mut FwNode,
        params: &mut PhoneAssignParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let quad = read_quad(req, node, OFFSET_PORT, timeout_ms)?;

        deserialize_flag(
            &mut params.0,
            &quad,
            PORT_PHONE_MASK,
            PORT_PHONE_SHIFT,
            Self::ASSIGN_PORT_TARGETS,
            Self::ASSIGN_PORT_VALS,
            PORT_PHONE_LABEL,
        )
    }
}

impl<O> MotuWhollyUpdatableParamsOperation<PhoneAssignParameters> for O
where
    O: MotuPortAssignSpecification,
{
    fn update_wholly(
        req: &mut FwReq,
        node: &mut FwNode,
        params: &PhoneAssignParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut quad = read_quad(req, node, OFFSET_PORT, timeout_ms)?;

        serialize_flag(
            &params.0,
            &mut quad,
            PORT_PHONE_MASK,
            PORT_PHONE_SHIFT,
            Self::ASSIGN_PORT_TARGETS,
            Self::ASSIGN_PORT_VALS,
            PORT_PHONE_LABEL,
        )?;

        write_quad(req, node, OFFSET_PORT, quad, timeout_ms)
    }
}

/// Mode of speed for output signal of word clock on BNC interface.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum WordClkSpeedMode {
    /// The speed is forced to be 44.1/48.0 kHz.
    ForceLowRate,
    /// The speed is following to system clock.
    FollowSystemClk,
}

impl Default for WordClkSpeedMode {
    fn default() -> Self {
        Self::FollowSystemClk
    }
}

const WORD_OUT_LABEL: &str = "word-out";
const WORD_OUT_MASK: u32 = 0x08000000;
const WORD_OUT_SHIFT: usize = 27;

/// The trait for specification of speed of word clock signal in XLR output interface.
pub trait MotuWordClockOutputSpecification {
    const WORD_CLOCK_OUTPUT_SPEED_MODES: &'static [WordClkSpeedMode] = &[
        WordClkSpeedMode::ForceLowRate,
        WordClkSpeedMode::FollowSystemClk,
    ];
}

const WORD_CLOCK_OUTPUT_SPEED_MODE_VALS: &[u8] = &[0x00, 0x01];

impl<O: MotuWordClockOutputSpecification> MotuWhollyCacheableParamsOperation<WordClkSpeedMode>
    for O
{
    fn cache_wholly(
        req: &mut FwReq,
        node: &mut FwNode,
        params: &mut WordClkSpeedMode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let quad = read_quad(req, node, OFFSET_CLK, timeout_ms)?;

        deserialize_flag(
            params,
            &quad,
            WORD_OUT_MASK,
            WORD_OUT_SHIFT,
            Self::WORD_CLOCK_OUTPUT_SPEED_MODES,
            WORD_CLOCK_OUTPUT_SPEED_MODE_VALS,
            WORD_OUT_LABEL,
        )
    }
}

impl<O: MotuWordClockOutputSpecification> MotuWhollyUpdatableParamsOperation<WordClkSpeedMode>
    for O
{
    fn update_wholly(
        req: &mut FwReq,
        node: &mut FwNode,
        params: &WordClkSpeedMode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut quad = read_quad(req, node, OFFSET_CLK, timeout_ms)?;

        serialize_flag(
            params,
            &mut quad,
            WORD_OUT_MASK,
            WORD_OUT_SHIFT,
            Self::WORD_CLOCK_OUTPUT_SPEED_MODES,
            WORD_CLOCK_OUTPUT_SPEED_MODE_VALS,
            WORD_OUT_LABEL,
        )?;

        write_quad(req, node, OFFSET_CLK, quad, timeout_ms)
    }
}

/// Mode of rate convert for AES/EBU input/output signals.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum AesebuRateConvertMode {
    /// Not available.
    None,
    /// The rate of input signal is converted to system rate.
    InputToSystem,
    /// The rate of output signal is slave to input, ignoring system rate.
    OutputDependsInput,
    /// The rate of output signal is double rate than system rate.
    OutputDoubleSystem,
}

impl Default for AesebuRateConvertMode {
    fn default() -> Self {
        Self::None
    }
}

const AESEBU_RATE_CONVERT_LABEL: &str = "aesebu-rate-convert";

/// The trait for specification of rate convert specific to AES/EBU input/output signals.
pub trait MotuAesebuRateConvertSpecification {
    const AESEBU_RATE_CONVERT_MASK: u32;
    const AESEBU_RATE_CONVERT_SHIFT: usize;

    const AESEBU_RATE_CONVERT_MODES: &'static [AesebuRateConvertMode] = &[
        AesebuRateConvertMode::None,
        AesebuRateConvertMode::InputToSystem,
        AesebuRateConvertMode::OutputDependsInput,
        AesebuRateConvertMode::OutputDoubleSystem,
    ];
}

const AESEBU_RATE_CONVERT_VALS: &[u8] = &[0x00, 0x01, 0x02, 0x03];

impl<O> MotuWhollyCacheableParamsOperation<AesebuRateConvertMode> for O
where
    O: MotuAesebuRateConvertSpecification,
{
    fn cache_wholly(
        req: &mut FwReq,
        node: &mut FwNode,
        params: &mut AesebuRateConvertMode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let quad = read_quad(req, node, OFFSET_CLK, timeout_ms)?;

        deserialize_flag(
            params,
            &quad,
            Self::AESEBU_RATE_CONVERT_MASK,
            Self::AESEBU_RATE_CONVERT_SHIFT,
            Self::AESEBU_RATE_CONVERT_MODES,
            AESEBU_RATE_CONVERT_VALS,
            AESEBU_RATE_CONVERT_LABEL,
        )
    }
}

impl<O> MotuWhollyUpdatableParamsOperation<AesebuRateConvertMode> for O
where
    O: MotuAesebuRateConvertSpecification,
{
    fn update_wholly(
        req: &mut FwReq,
        node: &mut FwNode,
        params: &AesebuRateConvertMode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut quad = read_quad(req, node, OFFSET_CLK, timeout_ms)?;

        serialize_flag(
            params,
            &mut quad,
            Self::AESEBU_RATE_CONVERT_MASK,
            Self::AESEBU_RATE_CONVERT_SHIFT,
            Self::AESEBU_RATE_CONVERT_MODES,
            AESEBU_RATE_CONVERT_VALS,
            AESEBU_RATE_CONVERT_LABEL,
        )?;

        write_quad(req, node, OFFSET_CLK, quad, timeout_ms)
    }
}

/// Mode of hold time for clip and peak LEDs.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum LevelMetersHoldTimeMode {
    /// off.
    Off,
    /// 2 seconds.
    Sec2,
    /// 4 seconds.
    Sec4,
    /// 10 seconds.
    Sec10,
    /// 1 minute.
    Sec60,
    /// 5 minutes.
    Sec300,
    /// 8 minutes.
    Sec480,
    /// Infinite.
    Infinite,
}

impl Default for LevelMetersHoldTimeMode {
    fn default() -> Self {
        Self::Off
    }
}

/// Mode of programmable meter display.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum LevelMetersProgrammableMode {
    /// For analog outputs.
    AnalogOutput,
    /// For ADAT A inputs.
    AdatAInput,
    /// For ADAT A outputs.
    AdatAOutput,
    /// For ADAT B inputs.
    AdatBInput,
    /// For ADAT B outputs.
    AdatBOutput,
    /// For AES/EBU inputs and outputs.
    AesEbuInputOutput,
}

impl Default for LevelMetersProgrammableMode {
    fn default() -> Self {
        Self::AnalogOutput
    }
}

/// Mode of AES/EBU meter display.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum LevelMetersAesebuMode {
    /// For AES/EBU inputs.
    Input,
    /// For AES/EBU outputs.
    Output,
}

impl Default for LevelMetersAesebuMode {
    fn default() -> Self {
        Self::Input
    }
}

/// The parameters of level meters.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct LevelMetersParameters {
    /// The duration to hold peak.
    pub peak_hold_time: LevelMetersHoldTimeMode,
    /// The duration to hold clip.
    pub clip_hold_time: LevelMetersHoldTimeMode,
    /// The mode to display AES/EBU signal.
    pub aesebu_mode: LevelMetersAesebuMode,
    /// The mode of programmable meter.
    pub programmable_mode: LevelMetersProgrammableMode,
}

const LEVEL_METERS_OFFSET: u32 = 0x0b24;

const LEVEL_METERS_PEAK_HOLD_TIME_MASK: u32 = 0x00003800;
const LEVEL_METERS_PEAK_HOLD_TIME_SHIFT: usize = 11;

const LEVEL_METERS_CLIP_HOLD_TIME_MASK: u32 = 0x00000700;
const LEVEL_METERS_CLIP_HOLD_TIME_SHIFT: usize = 8;

const LEVEL_METERS_HOLD_TIME_VALS: &[u8] = &[0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07];

const LEVEL_METERS_AESEBU_MASK: u32 = 0x00000004;
const LEVEL_METERS_AESEBU_SHIFT: usize = 2;

const LEVEL_METERS_AESEBU_VALS: &[u8] = &[0x00, 0x01];

const LEVEL_METERS_PROGRAMMABLE_MASK: u32 = 0x00000003;
const LEVEL_METERS_PROGRAMMABLE_SHIFT: usize = 0;
const LEVEL_METERS_PROGRAMMABLE_VALS: &[u8] = &[0x00, 0x01, 0x02, 0x03, 0x04, 0x05];

const LEVEL_METERS_PEAK_HOLD_TIME_LABEL: &str = "level-meters-peak-hold-time";
const LEVEL_METERS_CLIP_HOLD_TIME_LABEL: &str = "level-meters-clip-hold-time";
const LEVEL_METERS_PROGRAMMABLE_LABEL: &str = "level-meters-programmable";
const LEVEL_METERS_AESEBU_LABEL: &str = "level-meters-aesebu";

/// The trait for specification of level meter.
pub trait MotuLevelMetersSpecification {
    const LEVEL_METERS_HOLD_TIME_MODES: &'static [LevelMetersHoldTimeMode] = &[
        LevelMetersHoldTimeMode::Off,
        LevelMetersHoldTimeMode::Sec2,
        LevelMetersHoldTimeMode::Sec4,
        LevelMetersHoldTimeMode::Sec10,
        LevelMetersHoldTimeMode::Sec60,
        LevelMetersHoldTimeMode::Sec300,
        LevelMetersHoldTimeMode::Sec480,
        LevelMetersHoldTimeMode::Infinite,
    ];

    const LEVEL_METERS_AESEBU_MODES: &'static [LevelMetersAesebuMode] =
        &[LevelMetersAesebuMode::Output, LevelMetersAesebuMode::Input];

    const LEVEL_METERS_PROGRAMMABLE_MODES: &'static [LevelMetersProgrammableMode];
}

impl<O> MotuWhollyCacheableParamsOperation<LevelMetersParameters> for O
where
    O: MotuLevelMetersSpecification,
{
    fn cache_wholly(
        req: &mut FwReq,
        node: &mut FwNode,
        params: &mut LevelMetersParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let quad = read_quad(req, node, LEVEL_METERS_OFFSET, timeout_ms)?;

        deserialize_flag(
            &mut params.peak_hold_time,
            &quad,
            LEVEL_METERS_PEAK_HOLD_TIME_MASK,
            LEVEL_METERS_PEAK_HOLD_TIME_SHIFT,
            Self::LEVEL_METERS_HOLD_TIME_MODES,
            LEVEL_METERS_HOLD_TIME_VALS,
            LEVEL_METERS_PEAK_HOLD_TIME_LABEL,
        )?;

        deserialize_flag(
            &mut params.clip_hold_time,
            &quad,
            LEVEL_METERS_CLIP_HOLD_TIME_MASK,
            LEVEL_METERS_CLIP_HOLD_TIME_SHIFT,
            Self::LEVEL_METERS_HOLD_TIME_MODES,
            LEVEL_METERS_HOLD_TIME_VALS,
            LEVEL_METERS_CLIP_HOLD_TIME_LABEL,
        )?;

        deserialize_flag(
            &mut params.aesebu_mode,
            &quad,
            LEVEL_METERS_AESEBU_MASK,
            LEVEL_METERS_AESEBU_SHIFT,
            Self::LEVEL_METERS_AESEBU_MODES,
            LEVEL_METERS_AESEBU_VALS,
            LEVEL_METERS_AESEBU_LABEL,
        )?;

        deserialize_flag(
            &mut params.programmable_mode,
            &quad,
            LEVEL_METERS_PROGRAMMABLE_MASK,
            LEVEL_METERS_PROGRAMMABLE_SHIFT,
            Self::LEVEL_METERS_PROGRAMMABLE_MODES,
            LEVEL_METERS_PROGRAMMABLE_VALS,
            LEVEL_METERS_PROGRAMMABLE_LABEL,
        )?;

        Ok(())
    }
}

impl<O> MotuWhollyUpdatableParamsOperation<LevelMetersParameters> for O
where
    O: MotuLevelMetersSpecification,
{
    fn update_wholly(
        req: &mut FwReq,
        node: &mut FwNode,
        params: &LevelMetersParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut quad = read_quad(req, node, LEVEL_METERS_OFFSET, timeout_ms)?;

        serialize_flag(
            &params.peak_hold_time,
            &mut quad,
            LEVEL_METERS_PEAK_HOLD_TIME_MASK,
            LEVEL_METERS_PEAK_HOLD_TIME_SHIFT,
            Self::LEVEL_METERS_HOLD_TIME_MODES,
            LEVEL_METERS_HOLD_TIME_VALS,
            LEVEL_METERS_PEAK_HOLD_TIME_LABEL,
        )?;

        serialize_flag(
            &params.clip_hold_time,
            &mut quad,
            LEVEL_METERS_CLIP_HOLD_TIME_MASK,
            LEVEL_METERS_CLIP_HOLD_TIME_SHIFT,
            Self::LEVEL_METERS_HOLD_TIME_MODES,
            LEVEL_METERS_HOLD_TIME_VALS,
            LEVEL_METERS_CLIP_HOLD_TIME_LABEL,
        )?;

        serialize_flag(
            &params.aesebu_mode,
            &mut quad,
            LEVEL_METERS_AESEBU_MASK,
            LEVEL_METERS_AESEBU_SHIFT,
            Self::LEVEL_METERS_AESEBU_MODES,
            LEVEL_METERS_AESEBU_VALS,
            LEVEL_METERS_AESEBU_LABEL,
        )?;

        serialize_flag(
            &params.programmable_mode,
            &mut quad,
            LEVEL_METERS_PROGRAMMABLE_MASK,
            LEVEL_METERS_PROGRAMMABLE_SHIFT,
            Self::LEVEL_METERS_PROGRAMMABLE_MODES,
            LEVEL_METERS_PROGRAMMABLE_VALS,
            LEVEL_METERS_PROGRAMMABLE_LABEL,
        )?;

        write_quad(req, node, LEVEL_METERS_OFFSET, quad, timeout_ms)
    }
}

/// Port to assign.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TargetPort {
    Disabled,
    AnalogPair(usize),
    AesEbuPair,
    PhonePair,
    MainPair,
    SpdifPair,
    AdatPair(usize),
    Analog6Pairs,
    Analog8Pairs,
    OpticalAPair(usize),
    OpticalBPair(usize),
    Analog(usize),
    AesEbu(usize),
    Phone(usize),
    Main(usize),
    Spdif(usize),
    Adat(usize),
    OpticalA(usize),
    OpticalB(usize),
}

impl Default for TargetPort {
    fn default() -> Self {
        Self::Disabled
    }
}

/// Nominal level of audio signal.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum NominalSignalLevel {
    /// -10 dBV.
    Consumer,
    /// +4 dBu.
    Professional,
}

impl Default for NominalSignalLevel {
    fn default() -> Self {
        Self::Consumer
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn flag_serdes() {
        const TEST0_MASK: u32 = 0x000000ff;
        const TEST0_SHIFT: usize = 0;
        const TEST0_LABEL: &'static str = "test0";
        const TEST0_TARGETS: &[TargetPort] = &[
            TargetPort::AesEbuPair,
            TargetPort::PhonePair,
            TargetPort::MainPair,
            TargetPort::SpdifPair,
        ];
        const TEST0_VALS: &[u8] = &[0x01, 0x02, 0x04, 0x08];

        const TEST1_MASK: u32 = 0x0000ff00;
        const TEST1_SHIFT: usize = 8;
        const TEST1_LABEL: &'static str = "test1";
        const TEST1_TARGETS: &[LevelMetersHoldTimeMode] = &[
            LevelMetersHoldTimeMode::Off,
            LevelMetersHoldTimeMode::Sec2,
            LevelMetersHoldTimeMode::Sec4,
            LevelMetersHoldTimeMode::Sec10,
            LevelMetersHoldTimeMode::Sec60,
            LevelMetersHoldTimeMode::Sec300,
            LevelMetersHoldTimeMode::Sec480,
        ];
        const TEST1_VALS: &[u8] = &[0x01, 0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x80];

        let orig0 = TargetPort::SpdifPair;
        let mut quad = 0;
        serialize_flag(
            &orig0,
            &mut quad,
            TEST0_MASK,
            TEST0_SHIFT,
            TEST0_TARGETS,
            TEST0_VALS,
            TEST0_LABEL,
        )
        .unwrap();
        assert_eq!(quad, 0x00000008);

        let orig1 = LevelMetersHoldTimeMode::Off;
        serialize_flag(
            &orig1,
            &mut quad,
            TEST1_MASK,
            TEST1_SHIFT,
            TEST1_TARGETS,
            TEST1_VALS,
            TEST1_LABEL,
        )
        .unwrap();
        assert_eq!(quad, 0x00000108);

        let mut target0 = TargetPort::default();
        deserialize_flag(
            &mut target0,
            &quad,
            TEST0_MASK,
            TEST0_SHIFT,
            TEST0_TARGETS,
            TEST0_VALS,
            TEST0_LABEL,
        )
        .unwrap();
        assert_eq!(target0, orig0);

        let mut target1 = LevelMetersHoldTimeMode::default();
        deserialize_flag(
            &mut target1,
            &quad,
            TEST1_MASK,
            TEST1_SHIFT,
            TEST1_TARGETS,
            TEST1_VALS,
            TEST1_LABEL,
        )
        .unwrap();
        assert_eq!(target1, orig1);
    }
}
