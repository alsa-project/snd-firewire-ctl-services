// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

#![doc = include_str!("../README.md")]

use glib::{Error, FileError};

use hinawa::{prelude::FwReqExtManual, FwNode, FwReq, FwTcode};

/// The protocol implementation for Digi 002.
#[derive(Default, Debug)]
pub struct Digi002Protocol;

impl Dg00xHardwareSpecification for Digi002Protocol {
    const SAMPLING_CLOCK_SOURCES: &'static [ClockSource] =
        &[ClockSource::Internal, ClockSource::Spdif, ClockSource::Adat];
}

/// The protocol implementation for Digi 003.
#[derive(Default, Debug)]
pub struct Digi003Protocol;

impl Dg00xHardwareSpecification for Digi003Protocol {
    const SAMPLING_CLOCK_SOURCES: &'static [ClockSource] = &[
        ClockSource::Internal,
        ClockSource::Spdif,
        ClockSource::Adat,
        ClockSource::WordClock,
    ];
}

const BASE_OFFSET: u64 = 0xffffe0000000;

fn read_quadlet(
    req: &mut FwReq,
    node: &mut FwNode,
    offset: u64,
    timeout_ms: u32,
) -> Result<u32, Error> {
    let mut quadlet = [0; 4];
    req.transaction(
        node,
        FwTcode::ReadQuadletRequest,
        BASE_OFFSET + offset,
        quadlet.len(),
        &mut quadlet,
        timeout_ms,
    )
    .map(|_| u32::from_be_bytes(quadlet))
}

fn write_quadlet(
    req: &mut FwReq,
    node: &mut FwNode,
    offset: u64,
    val: u32,
    timeout_ms: u32,
) -> Result<(), Error> {
    let mut quadlet = [0; 4];
    quadlet.copy_from_slice(&val.to_be_bytes());
    req.transaction(
        node,
        FwTcode::WriteQuadletRequest,
        BASE_OFFSET + offset,
        quadlet.len(),
        &mut quadlet,
        timeout_ms,
    )
}

/// The specification of hardware.
pub trait Dg00xHardwareSpecification {
    const SAMPLING_CLOCK_SOURCES: &'static [ClockSource];
    const SAMPLING_CLOCK_RATES: &'static [u32] = &[44100, 48000, 88200, 96000];

    const MONITOR_SOURCE_GAIN_MIN: u8 = 0;
    const MONITOR_SOURCE_GAIN_MAX: u8 = 0x80;
    const MONITOR_SOURCE_GAIN_STEP: u8 = 1;
}

/// Cache whole parameters.
pub trait Dg00xWhollyCachableParamsOperation<T>: Dg00xHardwareSpecification {
    fn cache_wholly(
        req: &mut FwReq,
        node: &mut FwNode,
        states: &mut T,
        timeout_ms: u32,
    ) -> Result<(), Error>;
}

/// Update the part of parameters.
pub trait Dg00xPartiallyUpdatableParamsOperation<T>: Dg00xHardwareSpecification {
    fn update_partially(
        req: &mut FwReq,
        node: &mut FwNode,
        params: &mut T,
        update: T,
        timeout_ms: u32,
    ) -> Result<(), Error>;
}

/// Update whole parameters.
pub trait Dg00xWhollyUpdatableParamsOperation<T>: Dg00xHardwareSpecification {
    fn update_wholly(
        req: &mut FwReq,
        node: &mut FwNode,
        states: &T,
        timeout_ms: u32,
    ) -> Result<(), Error>;
}

/// Nominal frequency of media clock.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ClockRate {
    R44100,
    R48000,
    R88200,
    R96000,
}

impl Default for ClockRate {
    fn default() -> Self {
        Self::R44100
    }
}

/// Signal source of sampling clock.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ClockSource {
    Internal,
    Spdif,
    Adat,
    WordClock,
}

impl Default for ClockSource {
    fn default() -> Self {
        Self::Internal
    }
}

const SAMPLING_CLOCK_SOURCE_OFFSET: u64 = 0x0118;
const MEDIA_CLOCK_RATE_OFFSET: u64 = 0x0110;
const EXTERNAL_CLOCK_RATE_OFFSET: u64 = 0x0114;
const OPTICAL_INTERFACE_MODE_OFFSET: u64 = 0x011c;
const EXTERNAL_CLOCK_SOURCE_DETECTION_OFFSET: u64 = 0x012c;

/// The parameters for sampling clock.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Dg00xSamplingClockParameters {
    /// The source.
    pub source: ClockSource,
}

impl<O> Dg00xWhollyCachableParamsOperation<Dg00xSamplingClockParameters> for O
where
    O: Dg00xHardwareSpecification,
{
    fn cache_wholly(
        req: &mut FwReq,
        node: &mut FwNode,
        states: &mut Dg00xSamplingClockParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        read_quadlet(req, node, SAMPLING_CLOCK_SOURCE_OFFSET, timeout_ms).and_then(|val| {
            let pos = val as usize;
            Self::SAMPLING_CLOCK_SOURCES
                .iter()
                .nth(pos)
                .ok_or_else(|| {
                    let msg = format!("Unexpected clock source: {}", pos);
                    Error::new(FileError::Io, &msg)
                })
                .map(|&s| states.source = s)
        })
    }
}

impl<O> Dg00xWhollyUpdatableParamsOperation<Dg00xSamplingClockParameters> for O
where
    O: Dg00xHardwareSpecification,
{
    fn update_wholly(
        req: &mut FwReq,
        node: &mut FwNode,
        params: &Dg00xSamplingClockParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let pos = Self::SAMPLING_CLOCK_SOURCES
            .iter()
            .position(|&s| s.eq(&params.source))
            .ok_or_else(|| {
                let msg = format!("Invalid argument for clock source: {:?}", params.source);
                Error::new(FileError::Inval, &msg)
            })?;
        let val = pos as u32;
        write_quadlet(req, node, SAMPLING_CLOCK_SOURCE_OFFSET, val, timeout_ms)
    }
}

/// The parameters for media clock.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Dg00xMediaClockParameters {
    /// The rate.
    pub rate: ClockRate,
}

const CLOCK_RATE_TABLE: &[ClockRate] = &[
    ClockRate::R44100,
    ClockRate::R48000,
    ClockRate::R88200,
    ClockRate::R96000,
];

fn serialize_clock_rate(rate: &ClockRate) -> u32 {
    CLOCK_RATE_TABLE
        .iter()
        .position(|r| rate.eq(r))
        .map(|pos| pos as u32)
        .unwrap()
}

fn deserialize_clock_rate(rate: &mut ClockRate, val: u32) -> Result<(), Error> {
    *rate = match val {
        0 => Ok(ClockRate::R44100),
        1 => Ok(ClockRate::R48000),
        2 => Ok(ClockRate::R88200),
        3 => Ok(ClockRate::R96000),
        _ => {
            let msg = format!("Unexpected value for clock rate: {}", val);
            Err(Error::new(FileError::Inval, &msg))
        }
    }?;
    Ok(())
}

impl<O> Dg00xWhollyCachableParamsOperation<Dg00xMediaClockParameters> for O
where
    O: Dg00xHardwareSpecification,
{
    fn cache_wholly(
        req: &mut FwReq,
        node: &mut FwNode,
        states: &mut Dg00xMediaClockParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        read_quadlet(req, node, MEDIA_CLOCK_RATE_OFFSET, timeout_ms)
            .and_then(|val| deserialize_clock_rate(&mut states.rate, val))
    }
}

impl<O> Dg00xWhollyUpdatableParamsOperation<Dg00xMediaClockParameters> for O
where
    O: Dg00xHardwareSpecification,
{
    fn update_wholly(
        req: &mut FwReq,
        node: &mut FwNode,
        params: &Dg00xMediaClockParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let val = serialize_clock_rate(&params.rate);
        write_quadlet(req, node, SAMPLING_CLOCK_SOURCE_OFFSET, val, timeout_ms)
    }
}

/// Mode of optical interface.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum OpticalInterfaceMode {
    Adat,
    Spdif,
}

impl Default for OpticalInterfaceMode {
    fn default() -> Self {
        Self::Adat
    }
}

impl Dg00xWhollyCachableParamsOperation<OpticalInterfaceMode> for Digi003Protocol {
    fn cache_wholly(
        req: &mut FwReq,
        node: &mut FwNode,
        states: &mut OpticalInterfaceMode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        read_quadlet(req, node, OPTICAL_INTERFACE_MODE_OFFSET, timeout_ms).map(|val| {
            *states = if val > 0 {
                OpticalInterfaceMode::Spdif
            } else {
                OpticalInterfaceMode::Adat
            };
        })
    }
}

impl Dg00xWhollyUpdatableParamsOperation<OpticalInterfaceMode> for Digi003Protocol {
    fn update_wholly(
        req: &mut FwReq,
        node: &mut FwNode,
        params: &OpticalInterfaceMode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let val = match params {
            OpticalInterfaceMode::Adat => 0,
            OpticalInterfaceMode::Spdif => 1,
        };
        write_quadlet(req, node, OPTICAL_INTERFACE_MODE_OFFSET, val, timeout_ms)
    }
}

/// The parameters for media clock.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Dg00xExternalClockParameters {
    /// The rate detected in input of external source. Once the source of sampling clock is
    /// configured to any external source, the function can return detected frequency. Once losing
    /// the input, it returns None.
    pub rate: Option<ClockRate>,
}

impl<O> Dg00xWhollyCachableParamsOperation<Dg00xExternalClockParameters> for O
where
    O: Dg00xHardwareSpecification,
{
    fn cache_wholly(
        req: &mut FwReq,
        node: &mut FwNode,
        states: &mut Dg00xExternalClockParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let detected = read_quadlet(
            req,
            node,
            EXTERNAL_CLOCK_SOURCE_DETECTION_OFFSET,
            timeout_ms,
        )?;

        if detected > 0 {
            let val = read_quadlet(req, node, EXTERNAL_CLOCK_RATE_OFFSET, timeout_ms)?;
            let mut rate = ClockRate::default();
            deserialize_clock_rate(&mut rate, val).map(|_| states.rate = Some(rate))
        } else {
            states.rate = None;
            Ok(())
        }
    }
}

const MONITOR_DST_COUNT: usize = 2;
const MONITOR_SRC_COUNT: usize = 18;

/// State of monitor. At offline mode (no packet streaming runs), the monitor function is disabled
/// and is not configurable. When packet streaming starts, the monitor function becomes
/// configurable with reset state.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Dg00xMonitorState {
    /// Whether to enable monitor mixer or not.
    pub enabled: bool,
    /// The gain of monitor inputs. The value is between 0x00 and 0x80 for -48.0 and 0.0 dB.
    pub src_gains: [[u8; MONITOR_SRC_COUNT]; MONITOR_DST_COUNT],
}

const ENABLE_OFFSET: u64 = 0x0124;
const MONITOR_SRC_GAIN_OFFSET: u64 = 0x0300;

const DST_STEP: usize = 4;
const SRC_STEP: usize = 8;

fn compute_source_offset(dst: usize, src: usize) -> u64 {
    (dst * DST_STEP + src * SRC_STEP) as u64
}

impl<O> Dg00xWhollyCachableParamsOperation<Dg00xMonitorState> for O
where
    O: Dg00xHardwareSpecification,
{
    fn cache_wholly(
        req: &mut FwReq,
        node: &mut FwNode,
        states: &mut Dg00xMonitorState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        read_quadlet(req, node, ENABLE_OFFSET, timeout_ms).map(|val| states.enabled = val > 0)?;
        states
            .src_gains
            .iter_mut()
            .enumerate()
            .try_for_each(|(dst, gains)| {
                gains.iter_mut().enumerate().try_for_each(|(src, gain)| {
                    let offset = MONITOR_SRC_GAIN_OFFSET + compute_source_offset(dst, src);
                    read_quadlet(req, node, offset, timeout_ms).map(|val| *gain = (val >> 24) as u8)
                })
            })
    }
}

impl<O> Dg00xWhollyUpdatableParamsOperation<Dg00xMonitorState> for O
where
    O: Dg00xHardwareSpecification,
{
    fn update_wholly(
        req: &mut FwReq,
        node: &mut FwNode,
        states: &Dg00xMonitorState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        write_quadlet(req, node, ENABLE_OFFSET, states.enabled as u32, timeout_ms)?;
        states
            .src_gains
            .iter()
            .enumerate()
            .try_for_each(|(dst, gains)| {
                gains.iter().enumerate().try_for_each(|(src, &gain)| {
                    let offset = MONITOR_SRC_GAIN_OFFSET + compute_source_offset(dst, src);
                    let val = (gain as u32) << 24;
                    write_quadlet(req, node, offset, val, timeout_ms)
                })
            })
    }
}

impl<O> Dg00xPartiallyUpdatableParamsOperation<Dg00xMonitorState> for O
where
    O: Dg00xHardwareSpecification,
{
    fn update_partially(
        req: &mut FwReq,
        node: &mut FwNode,
        states: &mut Dg00xMonitorState,
        updates: Dg00xMonitorState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        if states.enabled != updates.enabled {
            write_quadlet(req, node, ENABLE_OFFSET, updates.enabled as u32, timeout_ms)
                .map(|_| states.enabled = updates.enabled)?;
        }

        states
            .src_gains
            .iter_mut()
            .zip(updates.src_gains.iter())
            .enumerate()
            .try_for_each(|(dst, (state, update))| {
                state
                    .iter_mut()
                    .zip(update.iter())
                    .enumerate()
                    .filter(|(_, (o, n))| !o.eq(n))
                    .try_for_each(|(src, (g, &gain))| {
                        let offset = MONITOR_SRC_GAIN_OFFSET + compute_source_offset(dst, src);
                        let val = (gain as u32) << 24;
                        write_quadlet(req, node, offset, val, timeout_ms).map(|_| *g = gain)
                    })
            })
    }
}
