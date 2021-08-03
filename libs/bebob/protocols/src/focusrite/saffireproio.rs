// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol implementation for Focusrite Saffire Pro 10 i/o and Pro 26 i/o.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by Focusrite Audio Engineering for Saffire Pro 10 i/o and Pro 26 i/o.
//!
//! DM1500E ASIC is used for Saffire Pro 26 i/o, while DM1500 is used for Saffire Pro 10 i/o.
//!
//! ## Diagram of internal signal flow for Saffire Pro 26 i/o.
//!
//! ```text
//! analog-input-1/2 ------+-----------------------> stream-output-1/2
//! analog-input-3/4 ------|-+---------------------> stream-output-3/4
//! analog-input-5/6 ------|-|-+-------------------> stream-output-5/6
//! analog-input-7/8 ------|-|-|-+-----------------> stream-output-7/8
//! spdif-input-1/2  ------|-|-|-|-+---------------> stream-output-9/10
//! adat-input-1/2   ------|-|-|-|-|-+-------------> stream-output-11/12
//! adat-input-3/4   ------|-|-|-|-|-|-+-----------> stream-output-13/14
//! adat-input-5/6   ------|-|-|-|-|-|-|-+---------> stream-output-15/16
//! adat-input-7/8   ------|-|-|-|-|-|-|-|-+-------> stream-output-17/18
//!                        | | | | | | | | |
//!                        v v v v v v v v v
//!                      ++=================++
//!                      ||     monitor     ||
//!                      ||                 ||
//!                      ||     18 x 2      ||
//!                      ++=================++
//!                                 |
//!                                 v
//!                        monitor-output-1/2
//!                                 |
//! stream-input-1/2   ------+------|-------------->
//!                          |      +--------------> analog-output-1/2
//!                          |      |
//! stream-input-3/4   ------|------|-------------->
//!                          +------|--------------> analog-output-3/4
//!                          |      +-------------->
//!                          |      |
//! stream-input-5/6   ------|------|-------------->
//!                          +------|--------------> analog-output-5/6
//!                          |      +-------------->
//!                          |      |
//! stream-input-7/8   ------|------|-------------->
//!                          +---------------------> analog-output-7/8
//!                          |      +-------------->
//!                          |      |
//! stream-input-9/10  ------|------|-------------->
//!                          +---------------------> spdif-output-1/2
//!                          |      +-------------->
//!                          |      |
//! stream-input-11/12 ------|------|-------------->
//!                          +---------------------> adat-output-1/2
//!                          |      +-------------->
//!                          |      |
//! stream-input-13/14 ------|------|-------------->
//!                          +---------------------> adat-output-3/4
//!                          |      +-------------->
//!                          |      |
//! stream-input-15/16 ------|------|-------------->
//!                          +---------------------> adat-output-5/6
//!                          |      +-------------->
//!                          |      |
//! stream-input-17/18 ------|------|-------------->
//!                          +---------------------> adat-output-7/8
//!                                 +-------------->
//! ```
//!
//! The protocol implementation for Saffire Pro 26 i/o is done with firmware version below:
//!
//! ```sh
//! $ cargo run --bin bco-bootloader-info -- /dev/fw1
//! protocol:
//!   version: 3
//! bootloader:
//!   timestamp: 2006-05-30T02:56:34+0000
//!   version: 0.0.0
//! hardware:
//!   GUID: 0x00030cdd00130e01
//!   model ID: 0x000013
//!   revision: 0.0.0
//! software:
//!   timestamp: 2008-09-10T03:51:13+0000
//!   ID: 3
//!   revision: 2.1.8386
//! image:
//!   base address: 0x400c0080
//!   maximum size: 0x149334
//! ```
//!
//! The protocol implementation for Saffire Pro 10 i/o is done with firmware version below:
//!
//! ```sh
//! $ cargo run --bin bco-bootloader-info -- /dev/fw1
//! protocol:
//!   version: 3
//! bootloader:
//!   timestamp: 2006-11-03T11:54:44+0000
//!   version: 0.0.0
//! hardware:
//!   GUID: 0x000606e000130e01
//!   model ID: 0x000014
//!   revision: 0.0.0
//! software:
//!   timestamp: 2008-09-10T03:51:12+0000
//!   ID: 6
//!   revision: 2.1.8386
//! image:
//!   base address: 0x400c0080
//!   maximum size: 0x149174
//! ```

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

/// The protocol implementation of meter information in Saffire Pro 26 i/o.
#[derive(Default)]
pub struct SaffirePro26ioMeterProtocol;

impl SaffireProioMeterOperation for SaffirePro26ioMeterProtocol {
    const SRC_LIST: &'static [SaffireProioSamplingClockSource] = &[
        SaffireProioSamplingClockSource::Internal,
        SaffireProioSamplingClockSource::Spdif,
        SaffireProioSamplingClockSource::Adat0,
        SaffireProioSamplingClockSource::Adat1,
        SaffireProioSamplingClockSource::WordClock,
    ];
}

/// The protocol implementation of input monitor for Saffire Pro i/o 26.
#[derive(Default, Debug)]
pub struct SaffirePro26ioMonitorProtocol;

impl SaffireProioMonitorProtocol for SaffirePro26ioMonitorProtocol {
    const HAS_ADAT: bool = true;
}

/// The protocol implementaion of function specific to Saffire Pro 26 i/o
#[derive(Default)]
pub struct SaffirePro26ioSpecificProtocol;

impl SaffireProioSpecificOperation for SaffirePro26ioSpecificProtocol {
    const PHANTOM_POWERING_COUNT: usize = 2;
    const INSERT_SWAP_COUNT: usize = 2;
}

/// The protocol implementation of media and sampling clocks for Saffire Pro 10 i/o. Write
/// operation corresponding to any change takes the unit to disappear from the bus, then
/// appears again with new configurations.
#[derive(Default)]
pub struct SaffirePro10ioClkProtocol;

impl SaffireProioMediaClockFrequencyOperation for SaffirePro10ioClkProtocol {
    const FREQ_LIST: &'static [u32] = &[44100, 48000, 88200, 96000];
}

/// The protocol implementation of input monitor for Saffire Pro i/o 10.
#[derive(Default, Debug)]
pub struct SaffirePro10ioMonitorProtocol;

impl SaffireProioMonitorProtocol for SaffirePro10ioMonitorProtocol {
    const HAS_ADAT: bool = false;
}

/// The protocol implementaion of function specific to Saffire Pro 26 i/o
#[derive(Default)]
pub struct SaffirePro10ioSpecificProtocol;

impl SaffireProioSpecificOperation for SaffirePro10ioSpecificProtocol {
    const PHANTOM_POWERING_COUNT: usize = 0;
    const INSERT_SWAP_COUNT: usize = 0;
}

/// The protocol implementation for operation of output parameters in Saffire Pro i/o series.
#[derive(Default)]
pub struct SaffireProioOutputProtocol;

impl SaffireOutputOperation for SaffireProioOutputProtocol {
    // analog-output-1/2, 3/4, 5/6, 7/8.
    const OFFSETS: &'static [usize] = &[0x140, 0x144, 0x148, 0x14c];

    const MUTE_COUNT: usize = 4;
    const VOL_COUNT: usize = 4;
    const HWCTL_COUNT: usize = 4;
    const DIM_COUNT: usize = 4;
    const PAD_COUNT: usize = 4;
}

impl SaffireProioSamplingClockSourceOperation for SaffirePro10ioClkProtocol {
    const SRC_LIST: &'static [SaffireProioSamplingClockSource] = &[
        SaffireProioSamplingClockSource::Internal,
        SaffireProioSamplingClockSource::Spdif,
    ];
}

/// The protocol implementation of meter information in Saffire Pro 10 i/o.
#[derive(Default)]
pub struct SaffirePro10ioMeterProtocol;

impl SaffireProioMeterOperation for SaffirePro10ioMeterProtocol {
    const SRC_LIST: &'static [SaffireProioSamplingClockSource] = &[
        SaffireProioSamplingClockSource::Internal,
        SaffireProioSamplingClockSource::Spdif,
    ];
}

/// The protocol implementation for operation of mixer in Saffire Pro i/o series.
#[derive(Default, Debug)]
pub struct SaffireProioMixerProtocol;

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

const CLK_SRC_EFFECTIVE_MASK: u32 = 0x0000ff00;
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

/// The ptorocol implementation of AC3 and MIDI signal through.
#[derive(Default, Debug)]
pub struct SaffireProioThroughProtocol;

impl SaffireThroughOperation for SaffireProioThroughProtocol {
    const MIDI_THROUGH_OFFSET: usize = 0x019c;
    const AC3_THROUGH_OFFSET: usize = 0x01a0;
}

/// The structure for meter information in Saffire Pro i/o.
#[derive(Default, Debug)]
pub struct SaffireProioMeterState {
    pub monitor_knob: u8,
    pub mute_led: bool,
    pub dim_led: bool,
    pub effective_clk_srcs: SaffireProioSamplingClockSource,
}

const MONITOR_KNOB_OFFSET: usize = 0x0158;
const DIM_LED_OFFSET: usize = 0x015c;
const MUTE_LED_OFFSET: usize = 0x0160;

const CLK_SRC_OFFSET: usize = 0x0174;

/// The trait of operation for meter information. The value of monitor knob is available only when
/// any of hwctl in output parameter is enabled, else it's always 0x8f.
pub trait SaffireProioMeterOperation {
    const SRC_LIST: &'static [SaffireProioSamplingClockSource];

    fn read_state(
        req: &FwReq,
        node: &FwNode,
        state: &mut SaffireProioMeterState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let offsets = [
            MONITOR_KNOB_OFFSET,
            DIM_LED_OFFSET,
            MUTE_LED_OFFSET,
            CLK_SRC_OFFSET,
        ];
        let mut buf = vec![0; offsets.len() * 4];
        saffire_read_quadlets(req, node, &offsets, &mut buf, timeout_ms).and_then(|_| {
            let mut quadlet = [0; 4];
            let vals = (0..offsets.len()).fold(Vec::new(), |mut vals, i| {
                let pos = i * 4;
                quadlet.copy_from_slice(&buf[pos..(pos + 4)]);
                vals.push(u32::from_be_bytes(quadlet));
                vals
            });

            state.monitor_knob = (vals[0] & 0xff) as u8;
            state.mute_led = vals[1] > 0;
            state.dim_led = vals[2] > 0;

            state.effective_clk_srcs = match (vals[3] & CLK_SRC_EFFECTIVE_MASK) >> 8 {
                CLK_SRC_INTERNAL => Ok(SaffireProioSamplingClockSource::Internal),
                CLK_SRC_SPDIF => Ok(SaffireProioSamplingClockSource::Spdif),
                CLK_SRC_ADAT0 => Ok(SaffireProioSamplingClockSource::Adat0),
                CLK_SRC_ADAT1 => Ok(SaffireProioSamplingClockSource::Adat1),
                CLK_SRC_WORD_CLOCK => Ok(SaffireProioSamplingClockSource::WordClock),
                _ => {
                    let msg = format!("Unexpected value for source of sampling clock: {}", vals[0]);
                    Err(Error::new(FileError::Io, &msg))
                }
            }?;

            Ok(())
        })
    }
}

/// The parameters of input monitor in Saffire Pro i/o.
#[derive(Default, Debug)]
pub struct SaffireProioMonitorParameters {
    pub analog_inputs: [[i16; 8]; 2],
    pub spdif_inputs: [[i16; 2]; 2],
    pub adat_inputs: Option<[[i16; 16]; 2]>,
}

/// The trait for input monitor protocol in Saffire Pro i/o.
pub trait SaffireProioMonitorProtocol {
    const HAS_ADAT: bool;

    const LEVEL_MIN: i16 = 0;
    const LEVEL_MAX: i16 = 0x7fff;
    const LEVEL_STEP: i16 = 0x100;

    const ANALOG_INPUT_OFFSETS: [usize; 16] = [
        0x00, // level from analog-input-0 to monitor-output-0
        0x04, // level from analog-input-0 to monitor-output-1
        0x08, // level from analog-input-1 to monitor-output-0
        0x0c, // level from analog-input-1 to monitor-output-1
        0x10, // level from analog-input-2 to monitor-output-0
        0x14, // level from analog-input-2 to monitor-output-1
        0x18, // level from analog-input-3 to monitor-output-0
        0x1c, // level from analog-input-3 to monitor-output-1
        0x20, // level from analog-input-4 to monitor-output-0
        0x24, // level from analog-input-4 to monitor-output-1
        0x28, // level from analog-input-5 to monitor-output-0
        0x2c, // level from analog-input-5 to monitor-output-1
        0x30, // level from analog-input-6 to monitor-output-0
        0x34, // level from analog-input-6 to monitor-output-1
        0x38, // level from analog-input-7 to monitor-output-0
        0x3c, // level from analog-input-7 to monitor-output-1
    ];
    const SPDIF_INPUT_OFFSETS: [usize; 4] = [
        0x40, // level from spdif-input-0 to monitor-output-0
        0x44, // level from spdif-input-1 to monitor-output-0
        0x48, // level from spdif-input-0 to monitor-output-1
        0x4c, // level from spdif-input-1 to monitor-output-1
    ];
    const ADAT_INPUT_OFFSETS: [usize; 32] = [
        0x50, // level from adat-input-a-0 to monitor-output-0
        0x54, // level from adat-input-a-0 to monitor-output-1
        0x58, // level from adat-input-a-1 to monitor-output-0
        0x5c, // level from adat-input-a-1 to monitor-output-1
        0x60, // level from adat-input-a-2 to monitor-output-0
        0x64, // level from adat-input-a-2 to monitor-output-1
        0x68, // level from adat-input-a-3 to monitor-output-0
        0x6c, // level from adat-input-a-3 to monitor-output-1
        0x70, // level from adat-input-a-4 to monitor-output-0
        0x74, // level from adat-input-a-4 to monitor-output-1
        0x78, // level from adat-input-a-5 to monitor-output-0
        0x7c, // level from adat-input-a-5 to monitor-output-1
        0x80, // level from adat-input-a-6 to monitor-output-0
        0x84, // level from adat-input-a-6 to monitor-output-1
        0x88, // level from adat-input-a-7 to monitor-output-0
        0x8c, // level from adat-input-a-7 to monitor-output-1
        0x90, // level from adat-input-b-0 to monitor-output-0
        0x94, // level from adat-input-b-0 to monitor-output-1
        0x98, // level from adat-input-b-1 to monitor-output-0
        0x9c, // level from adat-input-b-1 to monitor-output-1
        0xa0, // level from adat-input-b-2 to monitor-output-0
        0xa4, // level from adat-input-b-2 to monitor-output-1
        0xa8, // level from adat-input-b-3 to monitor-output-0
        0xac, // level from adat-input-b-3 to monitor-output-1
        0xb0, // level from adat-input-b-4 to monitor-output-0
        0xb4, // level from adat-input-b-4 to monitor-output-1
        0xb8, // level from adat-input-b-5 to monitor-output-0
        0xbc, // level from adat-input-b-5 to monitor-output-1
        0xc0, // level from adat-input-b-6 to monitor-output-0
        0xc4, // level from adat-input-b-6 to monitor-output-1
        0xc8, // level from adat-input-b-7 to monitor-output-0
        0xcc, // level from adat-input-b-7 to monitor-output-1
    ];

    fn create_params() -> SaffireProioMonitorParameters {
        SaffireProioMonitorParameters {
            analog_inputs: Default::default(),
            spdif_inputs: Default::default(),
            adat_inputs: if Self::HAS_ADAT {
                Some(Default::default())
            } else {
                None
            },
        }
    }

    fn read_params(
        req: &FwReq,
        node: &FwNode,
        params: &mut SaffireProioMonitorParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        read_monitor_params(
            req,
            node,
            &Self::ANALOG_INPUT_OFFSETS,
            &mut params.analog_inputs,
            timeout_ms,
        )?;
        read_monitor_params(
            req,
            node,
            &Self::SPDIF_INPUT_OFFSETS,
            &mut params.spdif_inputs,
            timeout_ms,
        )?;
        if let Some(mut levels_list) = &mut params.adat_inputs {
            read_monitor_params(
                req,
                node,
                &Self::ADAT_INPUT_OFFSETS,
                &mut levels_list,
                timeout_ms,
            )?;
        }
        Ok(())
    }

    fn write_analog_inputs(
        req: &FwReq,
        node: &FwNode,
        idx: usize,
        levels: &[i16],
        params: &mut SaffireProioMonitorParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        write_monitor_params(
            req,
            node,
            idx,
            levels,
            &Self::ANALOG_INPUT_OFFSETS,
            &mut params.analog_inputs,
            timeout_ms,
        )
    }

    fn write_spdif_inputs(
        req: &FwReq,
        node: &FwNode,
        idx: usize,
        levels: &[i16],
        params: &mut SaffireProioMonitorParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        write_monitor_params(
            req,
            node,
            idx,
            levels,
            &Self::SPDIF_INPUT_OFFSETS,
            &mut params.spdif_inputs,
            timeout_ms,
        )
    }

    fn write_adat_inputs(
        req: &FwReq,
        node: &FwNode,
        idx: usize,
        levels: &[i16],
        params: &mut SaffireProioMonitorParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        if let Some(adat_inputs) = &mut params.adat_inputs {
            write_monitor_params(
                req,
                node,
                idx,
                levels,
                &Self::ADAT_INPUT_OFFSETS,
                adat_inputs,
                timeout_ms,
            )
        } else {
            Err(Error::new(FileError::Inval, "ADAT is not supported"))
        }
    }
}

fn read_monitor_params<T>(
    req: &FwReq,
    node: &FwNode,
    offsets: &[usize],
    levels_list: &mut [T],
    timeout_ms: u32,
) -> Result<(), Error>
where
    T: AsMut<[i16]>,
{
    let mut buf = vec![0; offsets.len() * 4];
    saffire_read_quadlets(req, node, &offsets, &mut buf, timeout_ms).map(|_| {
        let mut quadlet = [0; 4];
        let vals = (0..offsets.len()).fold(Vec::new(), |mut vals, i| {
            let pos = i * 4;
            quadlet.copy_from_slice(&buf[pos..(pos + 4)]);
            vals.push(i32::from_be_bytes(quadlet) as i16);
            vals
        });
        levels_list.iter_mut().enumerate().for_each(|(i, levels)| {
            levels
                .as_mut()
                .iter_mut()
                .enumerate()
                .for_each(|(j, level)| *level = vals[i + j * 2]);
        });
    })
}

fn write_monitor_params<T>(
    req: &FwReq,
    node: &FwNode,
    idx: usize,
    levels: &[i16],
    offset_list: &[usize],
    old_levels_list: &mut [T],
    timeout_ms: u32,
) -> Result<(), Error>
where
    T: AsMut<[i16]>,
{
    let old_levels = old_levels_list.iter_mut().nth(idx).ok_or_else(|| {
        let msg = format!("Invalid index for monitor: {}", idx);
        Error::new(FileError::Inval, &msg)
    })?;

    let (offsets, buf) = old_levels
        .as_mut()
        .iter()
        .zip(levels.iter())
        .enumerate()
        .filter(|(_, (old, new))| !old.eq(new))
        .fold(
            (Vec::new(), Vec::new()),
            |(mut offsets, mut buf), (j, (_, &level))| {
                offsets.push(offset_list[idx + j * 2]);
                buf.extend_from_slice(&(level as i32).to_be_bytes());
                (offsets, buf)
            },
        );

    saffire_write_quadlets(req, node, &offsets, &buf, timeout_ms)
        .map(|_| old_levels.as_mut().copy_from_slice(levels))
}

/// The structure for parameters of mixer in Saffire Pro i/o.
#[derive(Default, Debug)]
pub struct SaffireProioMixerParameters {
    pub monitor_sources: [i16; 10],
    pub stream_source_pair0: [i16; 10],
    pub stream_sources: [i16; 10],
}

const MIXER_OFFSETS: [usize; 28] = [
    // level to analog-output-0
    0x0d0, // from stream-input-0
    0x0d4, // from monitor-output-0

    // level to analog-output-1
    0x0d8, // from stream-input-1
    0x0dc, // from monitor-output-1

    // level to analog-out-2
    0x0e0, // from stream-input-0
    0x0e4, // from stream-input-2
    0x0e8, // from monitor-output-0

    // level to analog-out-3
    0x0ec, // from stream-input-1
    0x0f0, // from stream-input-3
    0x0f4, // from monitor-output-1

    // level to analog-out-4
    0x0f8, // from stream-input-0
    0x0fc, // from stream-input-4
    0x100, // from monitor-output-0

    // level to analog-out-5
    0x104, // from stream-input-1
    0x108, // from stream-input-5
    0x10c, // from monitor-output-1

    // level to analog-out-6
    0x110, // from stream-input-0
    0x114, // from stream-input-6
    0x118, // from monitor-output-0

    // level to analog-out-7
    0x11c, // from stream-input-1
    0x120, // from stream-input-7
    0x124, // from monitor-output-1

    // level to analog-out-8
    0x128, // from stream-input-0
    0x12c, // from stream-input-8
    0x130, // from monitor-output-0

    // level to analog-out-9
    0x134, // from stream-input-1
    0x138, // from stream-input-9
    0x13c, // from monitor-output-1
];

impl SaffireProioMixerProtocol {
    pub const LEVEL_MIN: i16 = 0;
    pub const LEVEL_MAX: i16 = 0x7fff;
    pub const LEVEL_STEP: i16 = 0x100;

    pub fn read_params(
        req: &FwReq,
        node: &FwNode,
        params: &mut SaffireProioMixerParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut buf = vec![0; MIXER_OFFSETS.len() * 4];
        saffire_read_quadlets(req, node, &MIXER_OFFSETS, &mut buf, timeout_ms)?;

        let mut quadlet = [0; 4];
        let vals = (0..MIXER_OFFSETS.len()).fold(Vec::new(), |mut vals, i| {
            let pos = i * 4;
            quadlet.copy_from_slice(&buf[pos..(pos + 4)]);
            vals.push(u32::from_be_bytes(quadlet) as i16);
            vals
        });

        params
            .monitor_sources
            .iter_mut()
            .enumerate()
            .for_each(|(i, level)| *level = vals[calc_monitor_source_pos(i)]);

        params
            .stream_source_pair0
            .iter_mut()
            .enumerate()
            .for_each(|(i, level)| *level = vals[calc_stream_source_pair0_pos(i)]);

        params
            .stream_sources
            .iter_mut()
            .enumerate()
            .for_each(|(i, level)| *level = vals[calc_stream_source_pos(i)]);

        Ok(())
    }

    pub fn write_monitor_sources(
        req: &FwReq,
        node: &FwNode,
        levels: &[i16],
        params: &mut SaffireProioMixerParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        write_source_levels(
            req,
            node,
            levels,
            &mut params.monitor_sources,
            calc_monitor_source_pos,
            timeout_ms,
        )
    }

    pub fn write_stream_source_pair0(
        req: &FwReq,
        node: &FwNode,
        levels: &[i16],
        params: &mut SaffireProioMixerParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        write_source_levels(
            req,
            node,
            levels,
            &mut params.stream_source_pair0,
            calc_stream_source_pair0_pos,
            timeout_ms,
        )
    }

    pub fn write_stream_sources(
        req: &FwReq,
        node: &FwNode,
        levels: &[i16],
        params: &mut SaffireProioMixerParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        write_source_levels(
            req,
            node,
            levels,
            &mut params.stream_sources,
            calc_stream_source_pos,
            timeout_ms,
        )
    }
}

fn write_source_levels<F>(
    req: &FwReq,
    node: &FwNode,
    levels: &[i16],
    old_levels: &mut [i16],
    calc_pos: F,
    timeout_ms: u32,
) -> Result<(), Error>
where
    F: Fn(usize) -> usize,
{
    let (offsets, buf) = old_levels
        .iter()
        .zip(levels.iter())
        .enumerate()
        .filter(|(_, (old, new))| !old.eq(new))
        .fold(
            (Vec::new(), Vec::new()),
            |(mut offsets, mut buf), (i, (_, &value))| {
                offsets.push(MIXER_OFFSETS[calc_pos(i)]);
                buf.extend_from_slice(&(value as i32).to_be_bytes());
                (offsets, buf)
            },
        );
    saffire_write_quadlets(req, node, &offsets, &buf, timeout_ms)
        .map(|_| old_levels.copy_from_slice(levels))
}

fn calc_monitor_source_pos(i: usize) -> usize {
    if i < 2 {
        1 + i * 2
    } else {
        6 + (i - 2) * 3
    }
}

fn calc_stream_source_pair0_pos(i: usize) -> usize {
    if i < 2 {
        i * 2
    } else {
        4 + (i - 2) * 3
    }
}

fn calc_stream_source_pos(i: usize) -> usize {
    if i < 2 {
        i * 2
    } else {
        5 + (i - 2) * 3
    }
}

/// The enumeration for mode of stand alone.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SaffireProioStandaloneMode {
    Mix,
    Track,
}

impl Default for SaffireProioStandaloneMode {
    fn default() -> Self {
        Self::Mix
    }
}

/// The structure for parameters specific to Saffire Pro i/o series.
#[derive(Default, Debug)]
pub struct SaffireProioSpecificParameters {
    pub head_room: bool,

    pub phantom_powerings: Vec<bool>,
    pub insert_swaps: Vec<bool>,

    pub standalone_mode: SaffireProioStandaloneMode,
    pub adat_enabled: bool,
    pub direct_monitoring: bool,
}

const HEAD_ROOM_OFFSET: usize = 0x016c;

const PHANTOM_POWERING4567_OFFSET: usize = 0x0188;
const PHANTOM_POWERING0123_OFFSET: usize = 0x018c;
const INSERT_SWAP_0_OFFSET: usize = 0x0190;
const INSERT_SWAP_1_OFFSET: usize = 0x0194;
#[allow(dead_code)]
const IDENTIFY_OFFSET: usize = 0x0198;

const STANDALONE_MODE_OFFSET: usize = 0x01bc;
const ADAT_DISABLE_OFFSET: usize = 0x01c0;
const DIRECT_MONITORING_OFFSET: usize = 0x01c8;

/// The protocol implementation for functions specific to Saffire Pro i/o series. The change
/// operation to enable/disable ADAT corresponds to bus reset.
pub trait SaffireProioSpecificOperation {
    const PHANTOM_POWERING_COUNT: usize;
    const INSERT_SWAP_COUNT: usize;

    fn create_params() -> SaffireProioSpecificParameters {
        SaffireProioSpecificParameters {
            head_room: Default::default(),
            phantom_powerings: vec![Default::default(); Self::PHANTOM_POWERING_COUNT],
            insert_swaps: vec![Default::default(); Self::INSERT_SWAP_COUNT],
            standalone_mode: Default::default(),
            adat_enabled: Default::default(),
            direct_monitoring: Default::default(),
        }
    }

    fn read_params(
        req: &FwReq,
        node: &FwNode,
        params: &mut SaffireProioSpecificParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let offsets = [
            HEAD_ROOM_OFFSET,
            PHANTOM_POWERING4567_OFFSET,
            PHANTOM_POWERING0123_OFFSET,
            INSERT_SWAP_0_OFFSET,
            INSERT_SWAP_1_OFFSET,
            STANDALONE_MODE_OFFSET,
            ADAT_DISABLE_OFFSET,
            DIRECT_MONITORING_OFFSET,
        ];
        let mut buf = vec![0; offsets.len() * 4];
        saffire_read_quadlets(req, node, &offsets, &mut buf, timeout_ms).map(|_| {
            let mut quadlet = [0; 4];
            let vals = (0..offsets.len()).fold(Vec::new(), |mut vals, i| {
                let pos = i * 4;
                quadlet.copy_from_slice(&buf[pos..(pos + 4)]);
                vals.push(u32::from_be_bytes(quadlet));
                vals
            });

            params.head_room = vals[0] > 0;
            if Self::PHANTOM_POWERING_COUNT > 0 {
                params.phantom_powerings[0] = vals[2] > 0;
                params.phantom_powerings[1] = vals[1] > 0;
            }
            if Self::INSERT_SWAP_COUNT > 0 {
                params.insert_swaps[0] = vals[3] > 0;
                params.insert_swaps[1] = vals[4] > 0;
            }

            params.standalone_mode = if vals[5] > 0 {
                SaffireProioStandaloneMode::Track
            } else {
                SaffireProioStandaloneMode::Mix
            };

            params.adat_enabled = vals[6] == 0;
            params.direct_monitoring = vals[7] > 0;
        })
    }

    fn write_head_room(
        req: &FwReq,
        node: &FwNode,
        head_room: bool,
        params: &mut SaffireProioSpecificParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let buf = (head_room as u32).to_be_bytes();
        saffire_write_quadlet(req, node, HEAD_ROOM_OFFSET, &buf, timeout_ms)
            .map(|_| params.head_room = head_room)
    }

    fn write_phantom_powerings(
        req: &FwReq,
        node: &FwNode,
        phantom_powerings: &[bool],
        params: &mut SaffireProioSpecificParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        if Self::PHANTOM_POWERING_COUNT > 0 {
            let (offsets, buf) = params
                .phantom_powerings
                .iter()
                .rev()
                .zip(phantom_powerings.iter().rev())
                .zip([PHANTOM_POWERING4567_OFFSET, PHANTOM_POWERING0123_OFFSET].iter())
                .filter(|((old, new), _)| !old.eq(new))
                .fold(
                    (Vec::new(), Vec::new()),
                    |(mut offsets, mut buf), ((_, &value), &offset)| {
                        offsets.push(offset);
                        buf.extend_from_slice(&(value as u32).to_be_bytes());
                        (offsets, buf)
                    },
                );
            saffire_write_quadlets(req, node, &offsets, &buf, timeout_ms)
                .map(|_| params.phantom_powerings.copy_from_slice(&phantom_powerings))
        } else {
            Err(Error::new(
                FileError::Nxio,
                "Phantom powering is not supported",
            ))
        }
    }

    fn write_insert_swaps(
        req: &FwReq,
        node: &FwNode,
        insert_swaps: &[bool],
        params: &mut SaffireProioSpecificParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        if Self::INSERT_SWAP_COUNT > 0 {
            let (offsets, buf) = params
                .insert_swaps
                .iter()
                .zip(insert_swaps.iter())
                .zip([INSERT_SWAP_0_OFFSET, INSERT_SWAP_1_OFFSET].iter())
                .filter(|((old, new), _)| !old.eq(new))
                .fold(
                    (Vec::new(), Vec::new()),
                    |(mut offsets, mut buf), ((_, &value), &offset)| {
                        offsets.push(offset);
                        buf.extend_from_slice(&(value as u32).to_be_bytes());
                        (offsets, buf)
                    },
                );
            saffire_write_quadlets(req, node, &offsets, &buf, timeout_ms)
                .map(|_| params.insert_swaps.copy_from_slice(&insert_swaps))
        } else {
            Err(Error::new(FileError::Nxio, "Insert swap is not supported"))
        }
    }

    fn write_standalone_mode(
        req: &FwReq,
        node: &FwNode,
        mode: SaffireProioStandaloneMode,
        params: &mut SaffireProioSpecificParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let buf = if mode == SaffireProioStandaloneMode::Track {
            1u32.to_be_bytes()
        } else {
            0u32.to_be_bytes()
        };
        saffire_write_quadlet(req, node, STANDALONE_MODE_OFFSET, &buf, timeout_ms)
            .map(|_| params.standalone_mode = mode)
    }

    fn write_adat_enable(
        req: &FwReq,
        node: &FwNode,
        enable: bool,
        params: &mut SaffireProioSpecificParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let buf = (!enable as u32).to_be_bytes();
        saffire_write_quadlet(req, node, ADAT_DISABLE_OFFSET, &buf, timeout_ms)
            .map(|_| params.adat_enabled = enable)
    }

    fn write_direct_monitoring(
        req: &FwReq,
        node: &FwNode,
        enable: bool,
        params: &mut SaffireProioSpecificParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let buf = if enable {
            0xffffffffu32.to_be_bytes()
        } else {
            0u32.to_be_bytes()
        };
        saffire_write_quadlet(req, node, DIRECT_MONITORING_OFFSET, &buf, timeout_ms)
            .map(|_| params.direct_monitoring = enable)
    }
}

/// The protocol implementation to store configuration in Saffire.
#[derive(Default)]
pub struct SaffireProioStoreConfigProtocol;

impl SaffireStoreConfigOperation for SaffireProioStoreConfigProtocol {
    const OFFSET: usize = 0x1b0;
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_mixer_offset_helpers() {
        assert_eq!(calc_monitor_source_pos(0), 1);
        assert_eq!(calc_monitor_source_pos(1), 3);
        assert_eq!(calc_monitor_source_pos(2), 6);
        assert_eq!(calc_monitor_source_pos(3), 9);
        assert_eq!(calc_monitor_source_pos(4), 12);
        assert_eq!(calc_monitor_source_pos(5), 15);
        assert_eq!(calc_monitor_source_pos(6), 18);
        assert_eq!(calc_monitor_source_pos(7), 21);
        assert_eq!(calc_monitor_source_pos(8), 24);
        assert_eq!(calc_monitor_source_pos(9), 27);

        assert_eq!(calc_stream_source_pair0_pos(0), 0);
        assert_eq!(calc_stream_source_pair0_pos(1), 2);
        assert_eq!(calc_stream_source_pair0_pos(2), 4);
        assert_eq!(calc_stream_source_pair0_pos(3), 7);
        assert_eq!(calc_stream_source_pair0_pos(4), 10);
        assert_eq!(calc_stream_source_pair0_pos(5), 13);
        assert_eq!(calc_stream_source_pair0_pos(6), 16);
        assert_eq!(calc_stream_source_pair0_pos(7), 19);
        assert_eq!(calc_stream_source_pair0_pos(8), 22);
        assert_eq!(calc_stream_source_pair0_pos(9), 25);

        assert_eq!(calc_stream_source_pos(0), 0);
        assert_eq!(calc_stream_source_pos(1), 2);
        assert_eq!(calc_stream_source_pos(2), 5);
        assert_eq!(calc_stream_source_pos(3), 8);
        assert_eq!(calc_stream_source_pos(4), 11);
        assert_eq!(calc_stream_source_pos(5), 14);
        assert_eq!(calc_stream_source_pos(6), 17);
        assert_eq!(calc_stream_source_pos(7), 20);
        assert_eq!(calc_stream_source_pos(8), 23);
        assert_eq!(calc_stream_source_pos(9), 26);
    }
}
