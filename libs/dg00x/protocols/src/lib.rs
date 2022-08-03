// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocols defined by Digidesign for Digi 00x family
//!
//! The crate includes implementation of protocol defined by Digidesign for Digi 00x family.

use glib::{Error, FileError};

use hinawa::{prelude::FwReqExtManual, FwNode, FwReq, FwTcode};

/// The protocol implementation for Digi 002.
#[derive(Default)]
pub struct Digi002Protocol;

impl Dg00xCommonOperation for Digi002Protocol {
    const SAMPLING_CLOCK_SOURCES: &'static [ClockSource] =
        &[ClockSource::Internal, ClockSource::Spdif, ClockSource::Adat];
}

impl Dg00xMonitorOperation for Digi002Protocol {}

/// The protocol implementation for Digi 003.
#[derive(Default)]
pub struct Digi003Protocol;

impl Dg00xCommonOperation for Digi003Protocol {
    const SAMPLING_CLOCK_SOURCES: &'static [ClockSource] = &[
        ClockSource::Internal,
        ClockSource::Spdif,
        ClockSource::Adat,
        ClockSource::WordClock,
    ];
}

impl Dg00xMonitorOperation for Digi003Protocol {}

const BASE_OFFSET: u64 = 0xffffe0000000;

fn read_quadlet(
    req: &mut FwReq,
    node: &mut FwNode,
    offset: u64,
    timeout_ms: u32,
) -> Result<u32, Error> {
    let mut quadlet = [0; 4];
    req.transaction_sync(
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
    req.transaction_sync(
        node,
        FwTcode::WriteQuadletRequest,
        BASE_OFFSET + offset,
        quadlet.len(),
        &mut quadlet,
        timeout_ms,
    )
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

/// Mode of optical interface.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum OpticalInterfaceMode {
    Adat,
    Spdif,
}

impl Default for OpticalInterfaceMode {
    fn default() -> Self {
        Self::Adat
    }
}

const SAMPLING_CLOCK_SOURCE_OFFSET: u64 = 0x0118;
const MEDIA_CLOCK_RATE_OFFSET: u64 = 0x0110;
const EXTERNAL_CLOCK_RATE_OFFSET: u64 = 0x0114;
const OPTICAL_INTERFACE_MODE_OFFSET: u64 = 0x011c;
const EXTERNAL_CLOCK_SOURCE_DETECTION_OFFSET: u64 = 0x012c;

fn read_clock_rate(
    req: &mut FwReq,
    node: &mut FwNode,
    offset: u64,
    timeout_ms: u32,
) -> Result<ClockRate, Error> {
    read_quadlet(req, node, offset, timeout_ms).map(|val| match val {
        3 => ClockRate::R96000,
        2 => ClockRate::R88200,
        1 => ClockRate::R48000,
        _ => ClockRate::R44100,
    })
}

/// The trait for common operation.
pub trait Dg00xCommonOperation {
    const SAMPLING_CLOCK_SOURCES: &'static [ClockSource];

    fn read_sampling_clock_source(
        req: &mut FwReq,
        node: &mut FwNode,
        timeout_ms: u32,
    ) -> Result<ClockSource, Error> {
        read_quadlet(req, node, SAMPLING_CLOCK_SOURCE_OFFSET, timeout_ms).and_then(|val| {
            let src = match val {
                3 => ClockSource::WordClock,
                2 => ClockSource::Adat,
                1 => ClockSource::Spdif,
                _ => ClockSource::Internal,
            };
            Self::SAMPLING_CLOCK_SOURCES
                .iter()
                .find(|&s| s.eq(&src))
                .map(|&s| s)
                .ok_or_else(|| {
                    let msg = format!("Unexpected clock source: {}", val);
                    Error::new(FileError::Io, &msg)
                })
        })
    }

    /// The change has effect to stop processing audio data during packet streaming.
    fn write_sampling_clock_source(
        req: &mut FwReq,
        node: &mut FwNode,
        src: ClockSource,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let val = match src {
            ClockSource::Internal => 0,
            ClockSource::Spdif => 1,
            ClockSource::Adat => 2,
            ClockSource::WordClock => 3,
        };
        if Self::SAMPLING_CLOCK_SOURCES
            .iter()
            .find(|&s| s.eq(&src))
            .is_none()
        {
            let msg = format!("Invalid argument for clock source: {}", val);
            Err(Error::new(FileError::Inval, &msg))?;
        }
        write_quadlet(req, node, SAMPLING_CLOCK_SOURCE_OFFSET, val, timeout_ms)
    }

    fn read_media_clock_rate(
        req: &mut FwReq,
        node: &mut FwNode,
        timeout_ms: u32,
    ) -> Result<ClockRate, Error> {
        read_clock_rate(req, node, MEDIA_CLOCK_RATE_OFFSET, timeout_ms)
    }

    /// The change has effect to stop processing audio data during packet streaming.
    fn write_media_clock_rate(
        req: &mut FwReq,
        node: &mut FwNode,
        rate: ClockRate,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let val = match rate {
            ClockRate::R44100 => 0,
            ClockRate::R48000 => 1,
            ClockRate::R88200 => 2,
            ClockRate::R96000 => 3,
        };
        write_quadlet(req, node, MEDIA_CLOCK_RATE_OFFSET, val, timeout_ms)
    }

    fn read_optical_interface_mode(
        req: &mut FwReq,
        node: &mut FwNode,
        timeout_ms: u32,
    ) -> Result<OpticalInterfaceMode, Error> {
        read_quadlet(req, node, OPTICAL_INTERFACE_MODE_OFFSET, timeout_ms).map(|val| {
            if val > 0 {
                OpticalInterfaceMode::Spdif
            } else {
                OpticalInterfaceMode::Adat
            }
        })
    }

    /// The change has effect to stop processing audio data during packet streaming.
    fn write_optical_interface_mode(
        req: &mut FwReq,
        node: &mut FwNode,
        mode: OpticalInterfaceMode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let val = match mode {
            OpticalInterfaceMode::Adat => 0,
            OpticalInterfaceMode::Spdif => 1,
        };
        write_quadlet(req, node, OPTICAL_INTERFACE_MODE_OFFSET, val, timeout_ms)
    }

    /// Read frequency of media clock detected in input of external source. Once the source of
    /// sampling clock is configured to any external source, the function can return detected
    /// frequency. Once losing the input, it returns None.
    fn read_external_clock_source_rate(
        req: &mut FwReq,
        node: &mut FwNode,
        timeout_ms: u32,
    ) -> Result<Option<ClockRate>, Error> {
        read_quadlet(
            req,
            node,
            EXTERNAL_CLOCK_SOURCE_DETECTION_OFFSET,
            timeout_ms,
        )
        .and_then(|val| {
            if val > 0 {
                read_clock_rate(req, node, EXTERNAL_CLOCK_RATE_OFFSET, timeout_ms)
                    .map(|rate| Some(rate))
            } else {
                Ok(None)
            }
        })
    }
}

const MONITOR_DST_COUNT: usize = 2;
const MONITOR_SRC_COUNT: usize = 18;

/// State of monitor. The gain is between 0x00 and 0x80 for -48.0 and 0.0 dB.
#[derive(Default, Debug)]
pub struct Dg00xMonitorState {
    pub enabled: bool,
    pub src_gains: [[u8; MONITOR_SRC_COUNT]; MONITOR_DST_COUNT],
}

const ENABLE_OFFSET: u64 = 0x0124;
const MONITOR_SRC_GAIN_OFFSET: u64 = 0x0300;

const DST_STEP: usize = 4;
const SRC_STEP: usize = 8;

/// The trait for monitor operation. At offline mode (no packet streaming runs), the monitor
/// function is disabled and is not configurable. When packet streaming starts, the monitor
/// function becomes configurable with reset state.
pub trait Dg00xMonitorOperation {
    // 'monitor-output-0', 'monitor-output-1'.
    const MONITOR_DST_COUNT: usize = 2;

    // 'analog-input-0' .. 'analog-input-7', 'spdif-input-0', 'spdif-input-1',
    // 'adat-input-0' .. 'adat-input-7'.
    const MONITOR_SRC_COUNT: usize = 18;

    const GAIN_MIN: u8 = 0;
    const GAIN_MAX: u8 = 0x80;
    const GAIN_STEP: u8 = 1;

    fn read_monitor_state(
        req: &mut FwReq,
        node: &mut FwNode,
        state: &mut Dg00xMonitorState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        read_quadlet(req, node, ENABLE_OFFSET, timeout_ms).map(|val| state.enabled = val > 0)?;
        state
            .src_gains
            .iter_mut()
            .enumerate()
            .try_for_each(|(dst, gains)| {
                gains.iter_mut().enumerate().try_for_each(|(src, gain)| {
                    let offset = (dst * DST_STEP + src * SRC_STEP) as u64;
                    read_quadlet(req, node, MONITOR_SRC_GAIN_OFFSET + offset, timeout_ms)
                        .map(|val| *gain = (val >> 24) as u8)
                })
            })?;

        Ok(())
    }

    fn write_monitor_state(
        req: &mut FwReq,
        node: &mut FwNode,
        state: &Dg00xMonitorState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        Self::write_monitor_enable(req, node, state.enabled, timeout_ms)?;

        state
            .src_gains
            .iter()
            .enumerate()
            .try_for_each(|(dst, gains)| {
                gains.iter().enumerate().try_for_each(|(src, gain)| {
                    Self::write_monitor_source_gain(req, node, dst, src, *gain, timeout_ms)
                })
            })?;

        Ok(())
    }

    fn write_monitor_enable(
        req: &mut FwReq,
        node: &mut FwNode,
        enable: bool,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        write_quadlet(req, node, ENABLE_OFFSET, enable as u32, timeout_ms)
    }

    fn write_monitor_source_gain(
        req: &mut FwReq,
        node: &mut FwNode,
        dst: usize,
        src: usize,
        gain: u8,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        if dst >= Self::MONITOR_DST_COUNT {
            let msg = format!("Invalid argument for monitor destination: {}", dst);
            Err(Error::new(FileError::Inval, &msg))?;
        }
        if src >= Self::MONITOR_SRC_COUNT {
            let msg = format!("Invalid argument for monitor source: {}", src);
            Err(Error::new(FileError::Inval, &msg))?;
        }
        let offset = MONITOR_SRC_GAIN_OFFSET + (dst * DST_STEP + src * SRC_STEP) as u64;
        let val = (gain as u32) << 24;
        write_quadlet(req, node, offset, val, timeout_ms)
    }
}
