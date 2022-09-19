// SPDX-License-Identifier: LGPL-3.0-or-later
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

use super::*;

/// The protocol implementation of media and sampling clocks for Saffire Pro 26 i/o. Write
/// operation corresponding to any change takes the unit to disappear from the bus, then
/// appears again with new configurations.
#[derive(Default, Debug)]
pub struct SaffirePro26ioClkProtocol;

impl SaffireProioMediaClockSpecification for SaffirePro26ioClkProtocol {
    const FREQ_LIST: &'static [u32] = &[44100, 48000, 88200, 96000, 176400, 192000];
}

impl SaffireProioSamplingClockSpecification for SaffirePro26ioClkProtocol {
    const SRC_LIST: &'static [SaffireProioSamplingClockSource] = &[
        SaffireProioSamplingClockSource::Internal,
        SaffireProioSamplingClockSource::Spdif,
        SaffireProioSamplingClockSource::Adat0,
        SaffireProioSamplingClockSource::Adat1,
        SaffireProioSamplingClockSource::WordClock,
    ];
}

/// The protocol implementation of meter information in Saffire Pro 26 i/o.
#[derive(Default, Debug)]
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

impl SaffireProioMonitorSpecification for SaffirePro26ioMonitorProtocol {
    const HAS_ADAT: bool = true;
}

/// The protocol implementaion of function specific to Saffire Pro 26 i/o
#[derive(Default, Debug)]
pub struct SaffirePro26ioSpecificProtocol;

impl SaffireProioSpecificSpecification for SaffirePro26ioSpecificProtocol {
    const PHANTOM_POWERING_COUNT: usize = 2;
    const INSERT_SWAP_COUNT: usize = 2;
}

/// The protocol implementation of media and sampling clocks for Saffire Pro 10 i/o. Write
/// operation corresponding to any change takes the unit to disappear from the bus, then
/// appears again with new configurations.
#[derive(Default, Debug)]
pub struct SaffirePro10ioClkProtocol;

impl SaffireProioMediaClockSpecification for SaffirePro10ioClkProtocol {
    const FREQ_LIST: &'static [u32] = &[44100, 48000, 88200, 96000];
}

/// The protocol implementation of input monitor for Saffire Pro i/o 10.
#[derive(Default, Debug)]
pub struct SaffirePro10ioMonitorProtocol;

impl SaffireProioMonitorSpecification for SaffirePro10ioMonitorProtocol {
    const HAS_ADAT: bool = false;
}

/// The protocol implementaion of function specific to Saffire Pro 26 i/o
#[derive(Default, Debug)]
pub struct SaffirePro10ioSpecificProtocol;

impl SaffireProioSpecificSpecification for SaffirePro10ioSpecificProtocol {
    const PHANTOM_POWERING_COUNT: usize = 0;
    const INSERT_SWAP_COUNT: usize = 0;
}

/// The protocol implementation for operation of output parameters in Saffire Pro i/o series.
#[derive(Default, Debug)]
pub struct SaffireProioOutputProtocol;

impl SaffireOutputSpecification for SaffireProioOutputProtocol {
    // analog-output-1/2, 3/4, 5/6, 7/8.
    const OUTPUT_OFFSETS: &'static [usize] = &[0x140, 0x144, 0x148, 0x14c];

    const MUTE_COUNT: usize = 4;
    const VOL_COUNT: usize = 4;
    const HWCTL_COUNT: usize = 4;
    const DIM_COUNT: usize = 4;
    const PAD_COUNT: usize = 4;
}

impl SaffireProioSamplingClockSpecification for SaffirePro10ioClkProtocol {
    const SRC_LIST: &'static [SaffireProioSamplingClockSource] = &[
        SaffireProioSamplingClockSource::Internal,
        SaffireProioSamplingClockSource::Spdif,
    ];
}

/// The protocol implementation of meter information in Saffire Pro 10 i/o.
#[derive(Default, Debug)]
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

/// The specification of media clock.
pub trait SaffireProioMediaClockSpecification {
    /// The list of supported frequency.
    const FREQ_LIST: &'static [u32];
}

impl<O: SaffireProioMediaClockSpecification> SaffireParametersSerdes<MediaClockParameters> for O {
    const OFFSETS: &'static [usize] = &[0x0150];

    fn serialize(params: &MediaClockParameters, raw: &mut [u8]) {
        raw.copy_from_slice(&(params.freq_idx as u32).to_be_bytes());
    }

    fn deserialize(params: &mut MediaClockParameters, raw: &[u8]) {
        let mut quadlet = [9u8; 4];
        quadlet.copy_from_slice(&raw[..4]);
        let idx = u32::from_be_bytes(quadlet) as usize;
        if idx < Self::FREQ_LIST.len() {
            params.freq_idx = idx;
        }
    }
}

/// Signal source of sampling clock in Saffire Pro series.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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

/// The specification of sampling clock.
pub trait SaffireProioSamplingClockSpecification {
    /// The list of supported sources.
    const SRC_LIST: &'static [SaffireProioSamplingClockSource];
}

impl<O: SaffireProioSamplingClockSpecification> SaffireParametersSerdes<SamplingClockParameters>
    for O
{
    const OFFSETS: &'static [usize] = &[0x0174];

    fn serialize(params: &SamplingClockParameters, raw: &mut [u8]) {
        if let Some(val) = Self::SRC_LIST
            .iter()
            .nth(params.src_idx)
            .and_then(|&src| match src {
                SaffireProioSamplingClockSource::Internal => Some(CLK_SRC_INTERNAL),
                SaffireProioSamplingClockSource::Spdif => Some(CLK_SRC_SPDIF),
                SaffireProioSamplingClockSource::Adat0 => Some(CLK_SRC_ADAT0),
                SaffireProioSamplingClockSource::Adat1 => Some(CLK_SRC_ADAT1),
                SaffireProioSamplingClockSource::WordClock => Some(CLK_SRC_WORD_CLOCK),
            })
        {
            raw.copy_from_slice(&val.to_be_bytes());
        }
    }

    fn deserialize(params: &mut SamplingClockParameters, raw: &[u8]) {
        let mut quadlet = [0u8; 4];
        quadlet.copy_from_slice(&raw[..4]);
        let val = u32::from_be_bytes(quadlet) & CLK_SRC_CONF_MASK;
        if let Some(src_idx) = match val {
            CLK_SRC_INTERNAL => Some(SaffireProioSamplingClockSource::Internal),
            CLK_SRC_SPDIF => Some(SaffireProioSamplingClockSource::Spdif),
            CLK_SRC_ADAT0 => Some(SaffireProioSamplingClockSource::Adat0),
            CLK_SRC_ADAT1 => Some(SaffireProioSamplingClockSource::Adat1),
            CLK_SRC_WORD_CLOCK => Some(SaffireProioSamplingClockSource::WordClock),
            _ => None,
        }
        .and_then(|src| Self::SRC_LIST.iter().position(|s| s.eq(&src)))
        {
            params.src_idx = src_idx;
        }
    }
}

const CLK_SRC_EFFECTIVE_MASK: u32 = 0x0000ff00;
const CLK_SRC_CONF_MASK: u32 = 0x000000ff;
const CLK_SRC_INTERNAL: u32 = 0x00;
const CLK_SRC_SPDIF: u32 = 0x02;
const CLK_SRC_ADAT0: u32 = 0x03;
const CLK_SRC_ADAT1: u32 = 0x04;
const CLK_SRC_WORD_CLOCK: u32 = 0x05;

/// The prorocol implementation of AC3 and MIDI signal through.
#[derive(Default, Debug)]
pub struct SaffireProioThroughProtocol;

impl SaffireThroughSpecification for SaffireProioThroughProtocol {
    const THROUGH_OFFSETS: &'static [usize] = &[0x019c, 0x01a0];
}

/// Information of hardware metering in Saffire Pro i/o.
#[derive(Default, Debug)]
pub struct SaffireProioMeterState {
    pub monitor_knob: u8,
    pub mute_led: bool,
    pub dim_led: bool,
    pub effective_clk_srcs: SaffireProioSamplingClockSource,
}

/// The trait of operation for meter information. The value of monitor knob is available only when
/// any of hwctl in output parameter is enabled, else it's always 0x8f.
pub trait SaffireProioMeterOperation {
    const SRC_LIST: &'static [SaffireProioSamplingClockSource];

    const OFFSETS: &'static [usize] = &[
        0x0158, // The value of hardware knob.
        0x015c, // The state of dim LED.
        0x0160, // The state of mute LED.
        0x0174, // The effective source of sampling clock.
    ];

    /// Cache the state of hardware to the parameter.
    fn cache(
        req: &FwReq,
        node: &FwNode,
        state: &mut SaffireProioMeterState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut buf = vec![0; Self::OFFSETS.len() * 4];
        saffire_read_quadlets(req, node, &Self::OFFSETS, &mut buf, timeout_ms).and_then(|_| {
            let mut quadlet = [0; 4];
            let vals = (0..Self::OFFSETS.len()).fold(Vec::new(), |mut vals, i| {
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
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct SaffireProioMonitorParameters {
    pub analog_inputs: [[i16; 8]; 2],
    pub spdif_inputs: [[i16; 2]; 2],
    pub adat_inputs: Option<[[i16; 16]; 2]>,
}

/// The specification of protocol for hardware metering.
pub trait SaffireProioMonitorSpecification {
    /// Whether to have a pair of optical interface for ADAT signal.
    const HAS_ADAT: bool;

    /// The address offsets to operate for the parameters.
    const MONITOR_OFFSETS: &'static [usize] = &[
        // From analog inputs, at 16 address offsets.
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
        // From S/PDIF inputs, at 4 address offsets.
        0x40, // level from spdif-input-0 to monitor-output-0
        0x44, // level from spdif-input-1 to monitor-output-0
        0x48, // level from spdif-input-0 to monitor-output-1
        0x4c, // level from spdif-input-1 to monitor-output-1
        // From ADAT inputs, at 32 address offsets.
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
}

impl<O: SaffireProioMonitorSpecification> SaffireParametersSerdes<SaffireProioMonitorParameters>
    for O
{
    const OFFSETS: &'static [usize] = Self::MONITOR_OFFSETS;

    fn serialize(params: &SaffireProioMonitorParameters, raw: &mut [u8]) {
        params
            .analog_inputs
            .iter()
            .enumerate()
            .for_each(|(i, gains)| {
                gains.iter().enumerate().for_each(|(j, &gain)| {
                    let pos = (i + j * 2) * 4;
                    let gain = gain as i32;
                    raw[pos..(pos + 4)].copy_from_slice(&gain.to_be_bytes());
                });
            });

        params
            .spdif_inputs
            .iter()
            .enumerate()
            .for_each(|(i, gains)| {
                gains.iter().enumerate().for_each(|(j, &gain)| {
                    let pos = (16 + i + j * 2) * 4;
                    let gain = gain as i32;
                    raw[pos..(pos + 4)].copy_from_slice(&gain.to_be_bytes());
                });
            });

        if let Some(adat_inputs) = &params.adat_inputs {
            adat_inputs.iter().enumerate().for_each(|(i, gains)| {
                gains.iter().enumerate().for_each(|(j, &gain)| {
                    let pos = (20 + i + j * 2) * 4;
                    let gain = gain as i32;
                    raw[pos..(pos + 4)].copy_from_slice(&gain.to_be_bytes());
                });
            });
        }
    }

    fn deserialize(params: &mut SaffireProioMonitorParameters, raw: &[u8]) {
        let mut quadlet = [0; 4];

        let quads: Vec<i16> = (0..raw.len())
            .step_by(4)
            .map(|pos| {
                quadlet.copy_from_slice(&raw[pos..(pos + 4)]);
                i32::from_be_bytes(quadlet) as i16
            })
            .collect();

        params
            .analog_inputs
            .iter_mut()
            .enumerate()
            .for_each(|(i, gains)| {
                gains.iter_mut().enumerate().for_each(|(j, gain)| {
                    let pos = i + j * 2;
                    *gain = quads[pos];
                });
            });

        params
            .spdif_inputs
            .iter_mut()
            .enumerate()
            .for_each(|(i, gains)| {
                gains.iter_mut().enumerate().for_each(|(j, gain)| {
                    let pos = 16 + i + j * 2;
                    *gain = quads[pos];
                });
            });

        if let Some(adat_inputs) = &mut params.adat_inputs {
            adat_inputs.iter_mut().enumerate().for_each(|(i, gains)| {
                gains.iter_mut().enumerate().for_each(|(j, gain)| {
                    let pos = 20 + i + j * 2;
                    *gain = quads[pos];
                });
            });
        }
    }
}

/// The trait for input monitor protocol in Saffire Pro i/o.
pub trait SaffireProioMonitorProtocol: SaffireProioMonitorSpecification {
    const LEVEL_MIN: i16 = 0;
    const LEVEL_MAX: i16 = 0x7fff;
    const LEVEL_STEP: i16 = 0x100;

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
}

impl<O: SaffireProioMonitorSpecification> SaffireProioMonitorProtocol for O {}

/// The parameters of signal multiplexer in Saffire Pro i/o.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct SaffireProioMixerParameters {
    pub monitor_sources: [i16; 10],
    pub stream_source_pair0: [i16; 10],
    pub stream_sources: [i16; 10],
}

impl SaffireParametersSerdes<SaffireProioMixerParameters> for SaffireProioMixerProtocol {
    const OFFSETS: &'static [usize] = &[
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

    fn serialize(params: &SaffireProioMixerParameters, raw: &mut [u8]) {
        params
            .monitor_sources
            .iter()
            .enumerate()
            .for_each(|(i, &level)| {
                let pos = calc_monitor_source_pos(i) * 4;
                let level = level as i32;
                raw[pos..(pos + 4)].copy_from_slice(&level.to_be_bytes());
            });

        params
            .stream_source_pair0
            .iter()
            .enumerate()
            .for_each(|(i, &level)| {
                let pos = calc_stream_source_pair0_pos(i) * 4;
                let level = level as i32;
                raw[pos..(pos + 4)].copy_from_slice(&level.to_be_bytes());
            });

        params
            .stream_sources
            .iter()
            .enumerate()
            .for_each(|(i, &level)| {
                let pos = calc_stream_source_pos(i) * 4;
                let level = level as i32;
                raw[pos..(pos + 4)].copy_from_slice(&level.to_be_bytes());
            });
    }

    fn deserialize(params: &mut SaffireProioMixerParameters, raw: &[u8]) {
        let mut quadlet = [0; 4];

        let quads: Vec<i16> = (0..raw.len())
            .step_by(4)
            .map(|pos| {
                quadlet.copy_from_slice(&raw[pos..(pos + 4)]);
                i32::from_be_bytes(quadlet) as i16
            })
            .collect();

        params
            .monitor_sources
            .iter_mut()
            .enumerate()
            .for_each(|(i, level)| {
                let pos = calc_monitor_source_pos(i);
                *level = quads[pos];
            });

        params
            .stream_source_pair0
            .iter_mut()
            .enumerate()
            .for_each(|(i, level)| {
                let pos = calc_stream_source_pair0_pos(i);
                *level = quads[pos];
            });

        params
            .stream_sources
            .iter_mut()
            .enumerate()
            .for_each(|(i, level)| {
                let pos = calc_stream_source_pos(i);
                *level = quads[pos];
            });
    }
}

impl SaffireProioMixerProtocol {
    pub const LEVEL_MIN: i16 = 0;
    pub const LEVEL_MAX: i16 = 0x7fff;
    pub const LEVEL_STEP: i16 = 0x100;
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

/// Working mode at standalone mode.
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

/// Parameters specific to Saffire Pro i/o series.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaffireProioSpecificParameters {
    pub head_room: bool,

    pub phantom_powerings: Vec<bool>,
    pub insert_swaps: Vec<bool>,

    pub standalone_mode: SaffireProioStandaloneMode,
    pub adat_enabled: bool,
    pub direct_monitoring: bool,
}

/// The specification of protocol for function specific to Pro i/o.
pub trait SaffireProioSpecificSpecification {
    /// The address offsets to operate for the parameters.
    const SPECIFIC_OFFSETS: &'static [usize] = &[
        0x016c, 0x0188, // Phantom powering for microphone input 5-8.
        0x018c, // Phantom powering for microphone input 1-4.
        0x0190, // Ese 5th line input as insert.
        0x0194, // Ese 6th line input as insert.
        // 0x0198, any write operation blights LED.
        0x01bc, // The mode at standalone.
        0x01c0, // Enable/Disable ADAT inputs/outputs.
        0x01c8, // Enable/Disable direct monitoring.
    ];

    /// The number of microphone inputs supporting phantom powering.
    const PHANTOM_POWERING_COUNT: usize;

    /// The number of line inputs supporting polarity.
    const INSERT_SWAP_COUNT: usize;
}

// MEMO: The write transaction to enable/disable ADAT inputs/outputs generates bus reset.
impl<O: SaffireProioSpecificSpecification> SaffireParametersSerdes<SaffireProioSpecificParameters>
    for O
{
    const OFFSETS: &'static [usize] = Self::SPECIFIC_OFFSETS;

    fn serialize(params: &SaffireProioSpecificParameters, raw: &mut [u8]) {
        raw[..4].copy_from_slice(&(params.head_room as u32).to_be_bytes());

        params
            .phantom_powerings
            .iter()
            .rev()
            .enumerate()
            .for_each(|(i, &enabled)| {
                let pos = 4 + i * 4;
                raw[pos..(pos + 4)].copy_from_slice(&(enabled as u32).to_be_bytes());
            });

        params
            .insert_swaps
            .iter()
            .enumerate()
            .for_each(|(i, &enabled)| {
                let pos = 12 + i * 4;
                raw[pos..(pos + 4)].copy_from_slice(&(enabled as u32).to_be_bytes());
            });

        let val = (params.standalone_mode == SaffireProioStandaloneMode::Track) as u32;
        raw[20..24].copy_from_slice(&val.to_be_bytes());

        raw[24..28].copy_from_slice(&(!params.adat_enabled as u32).to_be_bytes());
        raw[28..32].copy_from_slice(&(params.direct_monitoring as u32).to_be_bytes());
    }

    fn deserialize(params: &mut SaffireProioSpecificParameters, raw: &[u8]) {
        let mut quadlet = [0; 4];

        let quads: Vec<i16> = (0..raw.len())
            .step_by(4)
            .map(|pos| {
                quadlet.copy_from_slice(&raw[pos..(pos + 4)]);
                i32::from_be_bytes(quadlet) as i16
            })
            .collect();

        params.head_room = quads[0] > 0;

        params
            .phantom_powerings
            .iter_mut()
            .rev()
            .enumerate()
            .for_each(|(i, enabled)| *enabled = quads[1 + i] > 0);

        params
            .insert_swaps
            .iter_mut()
            .enumerate()
            .for_each(|(i, enabled)| *enabled = quads[3 + i] > 0);

        params.standalone_mode = if quads[5] > 0 {
            SaffireProioStandaloneMode::Track
        } else {
            SaffireProioStandaloneMode::Mix
        };

        params.adat_enabled = quads[6] == 0;
        params.direct_monitoring = quads[7] > 0;
    }
}

/// The protocol implementation for functions specific to Saffire Pro i/o series. The change
/// operation to enable/disable ADAT corresponds to bus reset.
pub trait SaffireProioSpecificOperation: SaffireProioSpecificSpecification {
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
}

impl<O: SaffireProioSpecificSpecification> SaffireProioSpecificOperation for O {}

/// The protocol implementation to store configuration in Saffire.
#[derive(Default, Debug)]
pub struct SaffireProioStoreConfigProtocol;

impl SaffireStoreConfigSpecification for SaffireProioStoreConfigProtocol {
    const STORE_CONFIG_OFFSETS: &'static [usize] = &[0x1b0];
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn saffireproio_output_protocol_serdes() {
        let mut params = SaffireProioOutputProtocol::create_output_parameters();

        params
            .mutes
            .iter_mut()
            .step_by(2)
            .for_each(|mute| *mute = true);

        params
            .vols
            .iter_mut()
            .enumerate()
            .for_each(|(i, vol)| *vol = i as u8);

        params
            .hwctls
            .iter_mut()
            .step_by(2)
            .for_each(|hwctl| *hwctl = true);

        params
            .dims
            .iter_mut()
            .step_by(2)
            .for_each(|dim| *dim = true);

        params
            .pads
            .iter_mut()
            .step_by(2)
            .for_each(|pad| *pad = true);

        let mut raw = vec![0u8; SaffireProioOutputProtocol::OFFSETS.len() * 4];
        SaffireProioOutputProtocol::serialize(&params, &mut raw);
        let mut p = SaffireProioOutputProtocol::create_output_parameters();
        SaffireProioOutputProtocol::deserialize(&mut p, &raw);

        assert_eq!(params, p);
    }

    #[test]
    fn saffirepro10io_monitor_protocol_serdes() {
        let params = SaffireProioMonitorParameters {
            analog_inputs: [
                [45, 89, -17, -90, -28, 95, 32, -51],
                [-2, -43, -34, 69, 27, 14, 37, 10],
            ],
            spdif_inputs: [[84, 46], [42, -20]],
            adat_inputs: None,
        };
        let mut raw = vec![0u8; SaffirePro10ioMonitorProtocol::OFFSETS.len() * 4];
        SaffirePro10ioMonitorProtocol::serialize(&params, &mut raw);
        let mut p = SaffirePro10ioMonitorProtocol::create_params();
        SaffirePro10ioMonitorProtocol::deserialize(&mut p, &raw);

        assert_eq!(params, p);
    }

    #[test]
    fn saffirepro26io_monitor_protocol_serdes() {
        let params = SaffireProioMonitorParameters {
            analog_inputs: [
                [45, 89, -17, -90, -28, 95, 32, -51],
                [-2, -43, -34, 69, 27, 14, 37, 10],
            ],
            spdif_inputs: [[84, 46], [42, -20]],
            adat_inputs: Some([
                [
                    38, 9, 20, 4, -9, 82, -41, -14, 88, -18, 58, 1, -98, -26, 54, 21,
                ],
                [
                    58, 66, 42, -36, -50, 36, 50, -77, -99, -49, 52, -78, -51, -80, -40, 94,
                ],
            ]),
        };
        let mut raw = vec![0u8; SaffirePro26ioMonitorProtocol::OFFSETS.len() * 4];
        SaffirePro26ioMonitorProtocol::serialize(&params, &mut raw);
        let mut p = SaffirePro26ioMonitorProtocol::create_params();
        SaffirePro26ioMonitorProtocol::deserialize(&mut p, &raw);

        assert_eq!(params, p);
    }

    #[test]
    fn saffirepro10io_specific_protocol_serdes() {
        let mut params = SaffirePro10ioSpecificProtocol::create_params();
        params.standalone_mode = SaffireProioStandaloneMode::Track;
        params.adat_enabled = true;
        params.direct_monitoring = true;
        let mut raw = vec![0u8; SaffirePro10ioSpecificProtocol::OFFSETS.len() * 4];
        SaffirePro10ioSpecificProtocol::serialize(&params, &mut raw);
        let mut p = SaffirePro10ioSpecificProtocol::create_params();
        SaffirePro10ioSpecificProtocol::deserialize(&mut p, &raw);

        assert_eq!(params, p);
    }

    #[test]
    fn saffirepro26io_specific_protocol_serdes() {
        let mut params = SaffirePro26ioSpecificProtocol::create_params();
        params.phantom_powerings[0] = true;
        params.phantom_powerings[1] = false;
        params.insert_swaps[0] = false;
        params.insert_swaps[1] = true;
        params.standalone_mode = SaffireProioStandaloneMode::Track;
        params.adat_enabled = true;
        params.direct_monitoring = true;
        let mut raw = vec![0u8; SaffirePro26ioSpecificProtocol::OFFSETS.len() * 4];
        SaffirePro26ioSpecificProtocol::serialize(&params, &mut raw);
        let mut p = SaffirePro26ioSpecificProtocol::create_params();
        SaffirePro26ioSpecificProtocol::deserialize(&mut p, &raw);

        assert_eq!(params, p);
    }

    #[test]
    #[should_panic(expected = "expected to fail")]
    fn saffireproio_mixer_protocol_serdes() {
        let params = SaffireProioMixerParameters {
            monitor_sources: [-6, 25, 32, 76, 91, 57, -21, 88, 9, -87],
            stream_source_pair0: [84, -65, 59, 2, -21, 96, 40, 67, 72, 30],
            stream_sources: [-78, -75, -58, 86, 16, 59, 41, 88, 57, 24],
        };
        let mut raw = vec![0u8; SaffireProioMixerProtocol::OFFSETS.len() * 4];
        SaffireProioMixerProtocol::serialize(&params, &mut raw);
        let mut p = SaffireProioMixerParameters::default();
        SaffireProioMixerProtocol::deserialize(&mut p, &raw);

        assert_eq!(params, p, "expected to fail");
    }

    #[test]
    fn saffireproio_through_protocol_serdes() {
        let params = SaffireThroughParameters {
            midi: true,
            ac3: true,
        };
        let mut raw = vec![0u8; SaffireProioThroughProtocol::OFFSETS.len() * 4];
        SaffireProioThroughProtocol::serialize(&params, &mut raw);
        let mut p = SaffireThroughParameters::default();
        SaffireProioThroughProtocol::deserialize(&mut p, &raw);

        assert_eq!(params, p);
    }

    #[test]
    fn saffirepro10io_media_clock_serdes() {
        let mut raw = [0u8; 4];
        (0..SaffirePro10ioClkProtocol::FREQ_LIST.len()).for_each(|freq_idx| {
            let params = MediaClockParameters { freq_idx };
            SaffirePro10ioClkProtocol::serialize(&params, &mut raw);
            let mut p = MediaClockParameters::default();
            SaffirePro10ioClkProtocol::deserialize(&mut p, &raw);
            assert_eq!(params, p);
        });
    }

    #[test]
    fn saffirepro26io_media_clock_serdes() {
        let mut raw = [0u8; 4];
        (0..SaffirePro26ioClkProtocol::FREQ_LIST.len()).for_each(|freq_idx| {
            let params = MediaClockParameters { freq_idx };
            SaffirePro26ioClkProtocol::serialize(&params, &mut raw);
            let mut p = MediaClockParameters::default();
            SaffirePro26ioClkProtocol::deserialize(&mut p, &raw);
            assert_eq!(params, p);
        });
    }

    #[test]
    fn saffirepro10io_sampling_clock_serdes() {
        let mut raw = [0u8; 4];
        (0..SaffirePro10ioClkProtocol::SRC_LIST.len()).for_each(|src_idx| {
            let params = SamplingClockParameters { src_idx };
            SaffirePro10ioClkProtocol::serialize(&params, &mut raw);
            let mut p = SamplingClockParameters::default();
            SaffirePro10ioClkProtocol::deserialize(&mut p, &raw);
            assert_eq!(params, p);
        });
    }

    #[test]
    fn saffirepro26io_sampling_clock_serdes() {
        let mut raw = [0u8; 4];
        (0..SaffirePro26ioClkProtocol::SRC_LIST.len()).for_each(|src_idx| {
            let params = SamplingClockParameters { src_idx };
            SaffirePro26ioClkProtocol::serialize(&params, &mut raw);
            let mut p = SamplingClockParameters::default();
            SaffirePro26ioClkProtocol::deserialize(&mut p, &raw);
            assert_eq!(params, p);
        });
    }

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
