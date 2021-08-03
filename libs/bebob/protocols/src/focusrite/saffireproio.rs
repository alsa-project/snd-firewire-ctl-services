// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol implementation for Focusrite Saffire Pro 10 i/o and Pro 26 i/o.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by Focusrite Audio Engineering for Saffire Pro 10 i/o and Pro 26 i/o.
//!
//! DM1500E ASIC is used for Saffire Pro 26 i/o, while DM1500 is used for Saffire Pro 10 i/o.

use glib::FileError;

use super::*;

/// The protocol implementation of media and sampling clocks for Saffire Pro 26 i/o. Write
/// operation corresponding to any change takes the unit to disappear from the bus, then
/// appears again with new configurations.
#[derive(Default)]
pub struct SaffirePro26ioClkProtocol;

impl SaffireProioMediaClockFrequencyOperation for SaffirePro26ioClkProtocol {
    const FREQ_LIST: &'static [u32] = &[44100, 48000, 88200, 96000, 176400, 192000];
}

impl SaffireProioSamplingClockSourceOperation for SaffirePro26ioClkProtocol {
    const SRC_LIST: &'static [SaffireProioSamplingClockSource] = &[
        SaffireProioSamplingClockSource::Internal,
        SaffireProioSamplingClockSource::Spdif,
        SaffireProioSamplingClockSource::Adat0,
        SaffireProioSamplingClockSource::Adat1,
        SaffireProioSamplingClockSource::WordClock,
    ];
}

/// The protocol implementation of media and sampling clocks for Saffire Pro 10 i/o. Write
/// operation corresponding to any change takes the unit to disappear from the bus, then
/// appears again with new configurations.
#[derive(Default)]
pub struct SaffirePro10ioClkProtocol;

impl SaffireProioMediaClockFrequencyOperation for SaffirePro10ioClkProtocol {
    const FREQ_LIST: &'static [u32] = &[44100, 48000, 88200, 96000];
}

impl SaffireProioSamplingClockSourceOperation for SaffirePro10ioClkProtocol {
    const SRC_LIST: &'static [SaffireProioSamplingClockSource] = &[
        SaffireProioSamplingClockSource::Internal,
        SaffireProioSamplingClockSource::Spdif,
    ];
}

const MEDIA_CLOCK_FREQ_OFFSET: usize = 0x0150;

/// The trait of frequency operation for media clock in Saffire Pro series.
pub trait SaffireProioMediaClockFrequencyOperation {
    const FREQ_LIST: &'static [u32];

    fn read_clk_freq(req: &FwReq, node: &FwNode, timeout_ms: u32) -> Result<usize, Error> {
        let mut buf = [0; 4];
        saffire_read_quadlet(req, node, MEDIA_CLOCK_FREQ_OFFSET, &mut buf, timeout_ms).and_then(
            |_| {
                let val = u32::from_be_bytes(buf) as usize;
                if val > 0 || val < 1 + Self::FREQ_LIST.len() {
                    Ok(val - 1)
                } else {
                    let msg = format!("Unexpected value for frequency of media clock: {}", val);
                    Err(Error::new(FileError::Io, &msg))
                }
            },
        )
    }

    fn write_clk_freq(
        req: &FwReq,
        node: &FwNode,
        idx: usize,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let buf = u32::to_be_bytes((idx + 1) as u32);
        saffire_write_quadlet(req, node, MEDIA_CLOCK_FREQ_OFFSET, &buf, timeout_ms)
    }
}

/// The enumeration for source of sampling clock in Saffire Pro series.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SaffireProioSamplingClockSource {
    Internal,
    Spdif,
    Adat0,
    Adat1,
    WordClock,
}

impl Default for SaffireProioSamplingClockSource {
    fn default() -> Self {
        Self::Internal
    }
}

const SAMPLING_CLOCK_SRC_OFFSET: usize = 0x0174;

const CLK_SRC_CONF_MASK: u32 = 0x000000ff;
const CLK_SRC_INTERNAL: u32 = 0x00;
const CLK_SRC_SPDIF: u32 = 0x02;
const CLK_SRC_ADAT0: u32 = 0x03;
const CLK_SRC_ADAT1: u32 = 0x04;
const CLK_SRC_WORD_CLOCK: u32 = 0x05;

/// The trait of source operation for sampling clock in Saffire Pro series.
pub trait SaffireProioSamplingClockSourceOperation {
    const SRC_LIST: &'static [SaffireProioSamplingClockSource];

    fn read_clk_src(req: &FwReq, node: &FwNode, timeout_ms: u32) -> Result<usize, Error> {
        let mut buf = [0; 4];
        saffire_read_quadlet(req, node, SAMPLING_CLOCK_SRC_OFFSET, &mut buf, timeout_ms)?;

        let val = u32::from_be_bytes(buf) & CLK_SRC_CONF_MASK;
        let src = match val {
            CLK_SRC_INTERNAL => Ok(SaffireProioSamplingClockSource::Internal),
            CLK_SRC_SPDIF => Ok(SaffireProioSamplingClockSource::Spdif),
            CLK_SRC_ADAT0 => Ok(SaffireProioSamplingClockSource::Adat0),
            CLK_SRC_ADAT1 => Ok(SaffireProioSamplingClockSource::Adat1),
            CLK_SRC_WORD_CLOCK => Ok(SaffireProioSamplingClockSource::WordClock),
            _ => {
                let msg = format!("Unexpected value for source of sampling clock: {}", val);
                Err(Error::new(FileError::Io, &msg))
            }
        }?;

        Self::SRC_LIST
            .iter()
            .position(|s| s.eq(&src))
            .ok_or_else(|| {
                let msg = format!("Detecting unexpected source of sampling clock: {:?}", src);
                Error::new(FileError::Io, &msg)
            })
    }

    fn write_clk_src(req: &FwReq, node: &FwNode, idx: usize, timeout_ms: u32) -> Result<(), Error> {
        let &src = Self::SRC_LIST.iter().nth(idx).ok_or_else(|| {
            let msg = format!("Invalid index for source of sampling clock: {}", idx);
            Error::new(FileError::Inval, &msg)
        })?;

        let value = match src {
            SaffireProioSamplingClockSource::Internal => CLK_SRC_INTERNAL,
            SaffireProioSamplingClockSource::Spdif => CLK_SRC_SPDIF,
            SaffireProioSamplingClockSource::Adat0 => CLK_SRC_ADAT0,
            SaffireProioSamplingClockSource::Adat1 => CLK_SRC_ADAT1,
            SaffireProioSamplingClockSource::WordClock => CLK_SRC_WORD_CLOCK,
        };

        let buf = value.to_be_bytes();
        saffire_write_quadlet(req, node, SAMPLING_CLOCK_SRC_OFFSET, &buf, timeout_ms)
    }
}
