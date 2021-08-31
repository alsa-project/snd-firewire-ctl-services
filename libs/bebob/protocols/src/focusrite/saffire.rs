// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol implementation for Focusrite Saffire and Saffire LE.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by Focusrite Audio Engineering for Saffire and Saffire LE.
//!
//! DM1000E ASIC is used for Saffire, while DM1000 ASIC is used for Saffire LE.
//!
//! ## Diagram of internal signal flow for Saffire.
//!
//! ```text
//!                              ++=========++
//! analog-input-1/2 -------+--> || effects || --+---------+-------------> stream-output-1/2
//!                         v    ++=========++   |         |
//! spdif-input-1/2  ----> or -------------------|--+------|-------------> stream-output-3/4
//!                                              |  |      v
//!                                              |  | ++=========+
//!                                              |  | || reverb ||
//!                                              |  | ++========++
//!                                              |  |      |
//!                                              v  v      v
//!                                         ++===============++
//! stream-input-1/2 ---------------------> ||               || ---------> analog-output-1/2
//! stream-input-3/4 ---------------------> ||     mixer     || ---------> analog-output-3/4
//! stream-input-5/6 ---------------------> ||               || ---------> analog-output-5/6
//! stream-input-7/8 ---------------------> ||    16 x 10    || ---------> analog-output-7/8
//! stream-input-9/10 --------------------> ||               || ---------> analog-output-9/10
//!                                         ++===============++
//! ```
//!
//! The protocol implementation for Saffire is done with firmware version below:
//!
//! ```sh
//! $ cargo run --bin bco-bootloader-info -- /dev/fw1
//! protocol:
//!   version: 1
//! bootloader:
//!   timestamp: 2005-05-17T02:19:34+0000
//!   version: 0.0.0
//! hardware:
//!   GUID: 0x0001007200130e01
//!   model ID: 0x000002
//!   revision: 0.0.1
//! software:
//!   timestamp: 2009-02-16T08:52:00+0000
//!   ID: 0
//!   revision: 2.2.7869
//! image:
//!   base address: 0x20080000
//!   maximum size: 0x180000
//! ```
//!
//! ## Diagram of internal signal flow at 44.1/48.0 kHz for Saffire LE.
//!
//! ```text
//! analog-input-1/2 ------------+------------------> stream-output-1/2
//! analog-input-3/4 ------------|-+----------------> stream-output-3/4
//! spdif-input-1/2  ------------|-|-+--------------> stream-output-5/6
//!                              | | |
//!                              v v v
//!                         ++===========++
//! stream-input-1/2 -----> ||           || ----+---> analog-output-1/2
//! stream-input-3/4 -----> ||   mixer   || ----|---> analog-output-3/4
//! stream-input-5/6 -----> ||  14 x 8   || ----|---> analog-output-5/6
//! stream-input-7/8 -----> ||           || ----|---> digital-output-1/2
//!                         ++===========++     |             v
//!                                             +----------> or ----> spdif-output-1/2
//! ```
//!
//! ## Diagram of internal signal flow at 88.2/96.0 kHz for Saffire LE.
//!
//! ```text
//! analog-input-1/2 ------------+------------------> stream-output-1/2
//! analog-input-3/4 ------------|-+----------------> stream-output-3/4
//! spdif-input-1/2  ------------|-|-+--------------> stream-output-5/6
//!                              | | |
//!                              v v v
//!                         ++===========++
//!                         ||  monitor  ||
//!                         ||   6 x 2   ||
//!                         ++===========++
//!                                |
//!                                v
//!                         ++===========++
//! stream-input-1/2 -----> ||   mixer   || ----+---> analog-output-1/2
//! stream-input-3/4 -----> ||   6 x 4   || ----|---> analog-output-3/4
//!                         ++===========++     |
//! stream-input-5/6 ---------------------------|---> analog-output-5/6
//!                                             v
//! stream-input-7/8 --------------------------or---> digital-output-1/2
//! ```
//!
//! The protocol implementation for Saffire LE is done with firmware version below:
//!
//! ```sh
//! $ cargo run --bin bco-bootloader-info -- /dev/fw1
//! protocol:
//!   version: 1
//! bootloader:
//!   timestamp: 2005-10-19T09:49:52+0000
//!   version: 0.0.0
//! hardware:
//!   GUID: 0x00040e1a00130e01
//!   model ID: 0x000002
//!   revision: 0.0.1
//! software:
//!   timestamp: 2006-12-07T02:08:26+0000
//!   ID: 0
//!   revision: 1.1.7442
//! image:
//!   base address: 0x20080000
//!   maximum size: 0x180000
//! ```

use super::*;
use crate::*;

/// The protocol implementation of media and sampling clocks for Saffire. 192.0 kHz is available
/// when configured by the other operation.
#[derive(Default)]
pub struct SaffireClkProtocol;

const SAFFIRE_MODE_192KHZ_OFFSET: usize = 0x138;

impl MediaClockFrequencyOperation for SaffireClkProtocol {
    const FREQ_LIST: &'static [u32] = &[44100, 48000, 88200, 96000, 192000];

    fn write_clk_freq(avc: &BebobAvc, idx: usize, timeout_ms: u32) -> Result<(), Error> {
        // 192 kHz is just available when enabled.
        let mut op = SaffireAvcOperation {
            offsets: vec![SAFFIRE_MODE_192KHZ_OFFSET],
            buf: vec![0; 4],
            ..Default::default()
        };
        avc.status(&AvcAddr::Unit, &mut op, timeout_ms)?;

        let mut quadlet = [0; 4];
        quadlet.copy_from_slice(&mut op.buf);
        let val = u32::from_be_bytes(quadlet);
        if (val > 0 && idx < 4) || (val == 0 && idx == 4) {
            let msg = format!("Invalid frequency of media clock: {}", Self::FREQ_LIST[idx]);
            Err(Error::new(FileError::Inval, &msg))?;
        }

        let fdf = Self::FREQ_LIST
            .iter()
            .nth(idx)
            .ok_or_else(|| {
                let msg = format!("Invalid argument for index of frequency: {}", idx);
                Error::new(FileError::Inval, &msg)
            })
            .map(|&freq| AmdtpFdf::new(AmdtpEventType::Am824, false, freq))?;

        let mut op = InputPlugSignalFormat {
            plug_id: 0,
            fmt: FMT_IS_AMDTP,
            fdf: fdf.into(),
        };
        avc.control(&AvcAddr::Unit, &mut op, timeout_ms)?;

        let mut op = OutputPlugSignalFormat {
            plug_id: 0,
            fmt: FMT_IS_AMDTP,
            fdf: fdf.into(),
        };
        avc.control(&AvcAddr::Unit, &mut op, timeout_ms)
    }
}

impl SamplingClockSourceOperation for SaffireClkProtocol {
    const DST: SignalAddr = SignalAddr::Subunit(SignalSubunitAddr {
        subunit: MUSIC_SUBUNIT_0,
        plug_id: 0x04,
    });

    const SRC_LIST: &'static [SignalAddr] = &[
        // Internal.
        SignalAddr::Subunit(SignalSubunitAddr {
            subunit: MUSIC_SUBUNIT_0,
            plug_id: 0x07,
        }),
        // S/PDIF in coaxial interface.
        SignalAddr::Unit(SignalUnitAddr::Ext(0x03)),
    ];
}

/// The structure for meter in Saffire.
#[derive(Debug, Default)]
pub struct SaffireMeter {
    pub phys_inputs: [i32; 4],
    pub dig_input_detect: bool,
    pub monitor_knob_value: u16,
}

/// The protocol implementation of metering in Saffire.
#[derive(Default)]
pub struct SaffireMeterProtocol;

impl SaffireMeterProtocol {
    pub const MONITOR_KNOB_MIN: u16 = 0;
    pub const MONITOR_KNOB_MAX: u16 = 0x1ff0;
    pub const MONITOR_KNOB_STEP: u16 = 0x10;

    pub const LEVEL_MIN: i32 = 0;
    pub const LEVEL_MAX: i32 = 0x7fffffff;
    pub const LEVEL_STEP: i32 = 1;

    const MONITOR_KNOB_VALUE_OFFSET: usize = 0x00f4;

    const PHYS_INPUT_OFFSETS: [usize; 4] = [
        0x100, // analog-input-0
        0x104, // analog-input-2
        0x108, // analog-input-1
        0x10c, // analog-input-3
    ];

    const DIG_INPUT_DETECT_OFFSET: usize = 0x13c;

    // Read meter information.
    pub fn read_meter(
        req: &FwReq,
        node: &FwNode,
        meter: &mut SaffireMeter,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut buf = [0; 4];
        saffire_read_quadlets(
            req,
            node,
            &[Self::MONITOR_KNOB_VALUE_OFFSET],
            &mut buf,
            timeout_ms,
        )
        .map(|_| {
            let vals = u32::from_be_bytes(buf);
            meter.monitor_knob_value = (vals & 0x0000ffff) as u16;
        })?;

        let mut buf = [0; 16];
        saffire_read_quadlets(req, node, &Self::PHYS_INPUT_OFFSETS, &mut buf, timeout_ms).map(
            |_| {
                let mut quadlet = [0; 4];
                let vals: Vec<i32> =
                    (0..Self::PHYS_INPUT_OFFSETS.len()).fold(Vec::new(), |mut vals, i| {
                        let pos = i * 4;
                        quadlet.copy_from_slice(&buf[pos..(pos + 4)]);
                        vals.push(i32::from_be_bytes(quadlet));
                        vals
                    });

                meter.phys_inputs[0] = vals[0];
                meter.phys_inputs[2] = vals[1];
                meter.phys_inputs[1] = vals[2];
                meter.phys_inputs[3] = vals[3];
            },
        )?;

        let mut buf = [0; 4];
        saffire_read_quadlets(
            req,
            node,
            &[Self::DIG_INPUT_DETECT_OFFSET],
            &mut buf,
            timeout_ms,
        )
        .map(|_| {
            let val = u32::from_be_bytes(buf);
            meter.dig_input_detect = val > 0;
        })
    }
}

/// The protocol implementation for operation of output parameters in Saffire.
#[derive(Default)]
pub struct SaffireOutputProtocol;

impl SaffireOutputOperation for SaffireOutputProtocol {
    // physical-output-1/2, 3/4, 5/6, 7/8, and digital-output-1/2.
    const OFFSETS: &'static [usize] = &[0xdc, 0xe0, 0xe4, 0xe8, 0xec];

    const MUTE_COUNT: usize = 5;
    const VOL_COUNT: usize = 4;
    const HWCTL_COUNT: usize = 4;
    const DIM_COUNT: usize = 1;
    const PAD_COUNT: usize = 0;
}

/// The enumeration for source of input-2/3.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SaffireInputPair1Source {
    AnalogInputPair0,
    DigitalInputPair0,
}

impl Default for SaffireInputPair1Source {
    fn default() -> Self {
        Self::AnalogInputPair0
    }
}

/// The enumeration for mode of mixer.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SaffireMixerMode {
    StereoPaired,
    StereoSeparated,
}

impl Default for SaffireMixerMode {
    fn default() -> Self {
        Self::StereoPaired
    }
}

/// The structure for parameters specific to Saffire.
#[derive(Default)]
pub struct SaffireSpecificParameters {
    pub mode_192khz: bool,
    pub input_pair_1_src: SaffireInputPair1Source,
    pub mixer_mode: SaffireMixerMode,
}

/// The protocol implementation for operation specific to Saffire.
#[derive(Default)]
pub struct SaffireSpecificProtocol;

const SAFFIRE_INPUT_PAIR1_SRC_OFFSET: usize = 0xf8;
const SAFFIRE_MIXER_MODE_OFFSET: usize = 0xfc;

impl SaffireSpecificProtocol {
    /// Read parameters.
    pub fn read_params(
        req: &FwReq,
        node: &FwNode,
        params: &mut SaffireSpecificParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let offsets = [
            SAFFIRE_MODE_192KHZ_OFFSET,
            SAFFIRE_INPUT_PAIR1_SRC_OFFSET,
            SAFFIRE_MIXER_MODE_OFFSET,
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
            params.mode_192khz = vals[0] > 0;
            params.input_pair_1_src = if vals[1] > 0 {
                SaffireInputPair1Source::DigitalInputPair0
            } else {
                SaffireInputPair1Source::AnalogInputPair0
            };
            params.mixer_mode = if vals[2] > 0 {
                SaffireMixerMode::StereoSeparated
            } else {
                SaffireMixerMode::StereoPaired
            };
        })
    }

    /// Use mode of 192.0 khz.
    pub fn write_192khz_mode(
        req: &FwReq,
        node: &FwNode,
        enable: bool,
        params: &mut SaffireSpecificParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let offsets = [SAFFIRE_MODE_192KHZ_OFFSET];
        let buf = (enable as u32).to_be_bytes();
        saffire_write_quadlets(req, node, &offsets, &buf, timeout_ms)
            .map(|_| params.mode_192khz = enable)
    }

    /// Write the source of input 2/3.
    pub fn write_input_pair_1_src(
        req: &FwReq,
        node: &FwNode,
        src: SaffireInputPair1Source,
        params: &mut SaffireSpecificParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let offsets = [SAFFIRE_INPUT_PAIR1_SRC_OFFSET];
        let val = match src {
            SaffireInputPair1Source::AnalogInputPair0 => 0,
            SaffireInputPair1Source::DigitalInputPair0 => 1,
        };
        let buf = u32::to_be_bytes(val);
        saffire_write_quadlets(req, node, &offsets, &buf, timeout_ms)
            .map(|_| params.input_pair_1_src = src)
    }

    /// Write the mode of mixer.
    pub fn write_mixer_mode(
        req: &FwReq,
        node: &FwNode,
        mode: SaffireMixerMode,
        params: &mut SaffireSpecificParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let offsets = [SAFFIRE_MIXER_MODE_OFFSET];
        let val = match mode {
            SaffireMixerMode::StereoPaired => 0,
            SaffireMixerMode::StereoSeparated => 1,
        };
        let buf = u32::to_be_bytes(val);
        saffire_write_quadlets(req, node, &offsets, &buf, timeout_ms)
            .map(|_| params.mixer_mode = mode)
    }
}

/// The protocol implementation for operation of mixer at stereo separated mode in Saffire.
#[derive(Default)]
pub struct SaffireSeparatedMixerProtocol;

impl SaffireMixerOperation for SaffireSeparatedMixerProtocol {
    const OFFSETS: &'static [usize] = &[
        // level from phys-input-0
        0x00, // to phys-output-8
        0x04, // to phys-output-0
        0x08, // to phys-output-2
        0x0c, // to phys-output-4
        0x10, // to phys-output-6
        // level from phys-input-2
        0x14, // to phys-output-8
        0x18, // to phys-output-0
        0x1c, // to phys-output-2
        0x20, // to phys-output-4
        0x24, // to phys-output-6
        // level from reverb-output-0
        0x28, // to phys-output-8
        0x2c, // to phys-output-0
        0x30, // to phys-output-2
        0x34, // to phys-output-4
        0x38, // to phys-output-6
        // level from phys-input-1
        0x3c, // to phys-output-9
        0x40, // to phys-output-1
        0x44, // to phys-output-3
        0x48, // to phys-output-5
        0x4c, // to phys-output-7
        // level from phys-input-3
        0x50, // to phys-output-9
        0x54, // to phys-output-1
        0x58, // to phys-output-3
        0x5c, // to phys-output-5
        0x60, // to phys-output-7
        // level from reverb-output-1
        0x64, // to phys-output-9
        0x68, // to phys-output-1
        0x6c, // to phys-output-3
        0x70, // to phys-output-5
        0x74, // to phys-output-7
        // level from stream-input-8/9
        0x78, // to phys-output-8/9
        0x7c, // to phys-output-0/1
        0x80, // to phys-output-2/3
        0x84, // to phys-output-4/5
        0x88, // to phys-output-6/7
        // level from stream-input-0/1
        0x8c, // to phys-output-8/9
        0x90, // to phys-output-0/1
        0x94, // to phys-output-2/3
        0x98, // to phys-output-4/5
        0x9c, // to phys-output-6/7
        // level from stream-input-2/3
        0xa0, // to phys-output-8/9
        0xa4, // to phys-output-0/1
        0xa8, // to phys-output-2/3
        0xac, // to phys-output-4/5
        0xb0, // to phys-output-6/7
        // level from stream-input-4/5
        0xb4, // to phys-output-8/9
        0xb8, // to phys-output-0/1
        0xbc, // to phys-output-2/3
        0xc0, // to phys-output-4/5
        0xc4, // to phys-output-6/7
        // level from stream-input-6/7
        0xc8, // to phys-output-8/9
        0xcc, // to phys-output-0/1
        0xd0, // to phys-output-2/3
        0xd4, // to phys-output-4/5
        0xd8, // to phys-output-6/7
    ];

    const PHYS_INPUT_COUNT: usize = 4;
    const REVERB_RETURN_COUNT: usize = 2;

    #[inline(always)]
    fn stream_src_pos(mut dst_idx: usize, mut src_idx: usize) -> usize {
        if dst_idx > 3 {
            dst_idx = 0
        } else {
            dst_idx += 1;
        }
        if src_idx > 3 {
            src_idx = 0;
        } else {
            src_idx += 1;
        }
        30 + src_idx * 5 + dst_idx
    }

    #[inline(always)]
    fn phys_src_pos(mut dst_idx: usize, src_idx: usize) -> usize {
        if dst_idx > 3 {
            dst_idx = 0;
        } else {
            dst_idx += 1;
        }
        (src_idx / 2 * 5) + (src_idx % 2 * 15) + dst_idx
    }

    #[inline(always)]
    fn reverb_return_pos(mut dst_idx: usize, src_idx: usize) -> usize {
        if dst_idx > 3 {
            dst_idx = 0
        } else {
            dst_idx += 1;
        }
        10 + src_idx * 15 + dst_idx
    }
}

/// The protocol implementation for operation of mixer at stereo paired mode in Saffire.
#[derive(Default)]
pub struct SaffirePairedMixerProtocol;

impl SaffireMixerOperation for SaffirePairedMixerProtocol {
    const OFFSETS: &'static [usize] = &[
        // level from stream-input-8/9
        0x00, // to phys-output-8/9
        0x04, // to phys-output-0/1
        0x08, // to phys-output-2/3
        0x0c, // to phys-output-4/5
        0x10, // to phys-output-6/7
        // level from stream-input-0/1
        0x14, // to phys-output-8/9
        0x18, // to phys-output-0/1
        0x1c, // to phys-output-2/3
        0x20, // to phys-output-4/5
        0x24, // to phys-output-6/7
        // level from stream-input-2/3
        0x28, // to phys-output-8/9
        0x2c, // to phys-output-0/1
        0x30, // to phys-output-2/3
        0x34, // to phys-output-4/5
        0x38, // to phys-output-6/7
        // level from stream-input-4/5
        0x3c, // to phys-output-8/9
        0x40, // to phys-output-0/1
        0x44, // to phys-output-2/3
        0x48, // to phys-output-4/5
        0x4c, // to phys-output-6/7
        // level from stream-input-6/7
        0x50, // to phys-output-8/9
        0x54, // to phys-output-0/1
        0x58, // to phys-output-2/3
        0x5c, // to phys-output-4/5
        0x60, // to phys-output-6/7

        // level from phys-input-0/1
        0x64, // to phys-output-8/9
        0x68, // to phys-output-0/1
        0x6c, // to phys-output-2/3
        0x70, // to phys-output-4/5
        0x74, // to phys-output-6/7
        // level from phys-input-2/3
        0x78, // to phys-output-8/9
        0x7c, // to phys-output-0/1
        0x80, // to phys-output-2/3
        0x84, // to phys-output-4/5
        0x88, // to phys-output-6/7

        // level from reverb-output-0/1
        0x8c, // to phys-output-8/9
        0x90, // to phys-output-0/1
        0x94, // to phys-output-2/3
        0x98, // to phys-output-4/5
        0x9c, // to phys-output-6/7
    ];

    const PHYS_INPUT_COUNT: usize = 2;
    const REVERB_RETURN_COUNT: usize = 1;

    #[inline(always)]
    fn stream_src_pos(mut dst_idx: usize, mut src_idx: usize) -> usize {
        if dst_idx > 3 {
            dst_idx = 0
        } else {
            dst_idx += 1;
        }
        if src_idx > 3 {
            src_idx = 0;
        } else {
            src_idx += 1;
        }
        dst_idx + src_idx * 5
    }

    #[inline(always)]
    fn phys_src_pos(mut dst_idx: usize, src_idx: usize) -> usize {
        if dst_idx > 3 {
            dst_idx = 0
        } else {
            dst_idx += 1;
        }
        20 + dst_idx + src_idx * 5
    }

    #[inline(always)]
    fn reverb_return_pos(mut dst_idx: usize, _: usize) -> usize {
        if dst_idx > 3 {
            dst_idx = 0
        } else {
            dst_idx += 1;
        }
        35 + dst_idx
    }
}

/// The protocol implementation of media and sampling clocks for Saffire LE.
#[derive(Default)]
pub struct SaffireLeClkProtocol;

impl MediaClockFrequencyOperation for SaffireLeClkProtocol {
    const FREQ_LIST: &'static [u32] = &[44100, 48000, 88200, 96000];
}

impl SamplingClockSourceOperation for SaffireLeClkProtocol {
    const DST: SignalAddr = SignalAddr::Subunit(SignalSubunitAddr {
        subunit: MUSIC_SUBUNIT_0,
        plug_id: 0x05,
    });

    const SRC_LIST: &'static [SignalAddr] = &[
        // Internal.
        SignalAddr::Subunit(SignalSubunitAddr {
            subunit: MUSIC_SUBUNIT_0,
            plug_id: 0x06,
        }),
        // S/PDIF in coaxial interface.
        SignalAddr::Unit(SignalUnitAddr::Ext(0x04)),
    ];
}

/// The protocol implementation to store configuration in Saffire.
#[derive(Default)]
pub struct SaffireStoreConfigProtocol;

impl SaffireStoreConfigOperation for SaffireStoreConfigProtocol {
    const OFFSET: usize = 0x148;
}

/// The structure for meter information in Saffire LE.
#[derive(Default)]
pub struct SaffireLeMeter {
    pub phys_inputs: [i32; 6],
    pub phys_outputs: [i32; 8],
    pub stream_inputs: [i32; 4],
    pub dig_input_detect: bool,
}

/// The protocol implementation of metering in Saffire LE.
#[derive(Default)]
pub struct SaffireLeMeterProtocol;

// NOTE: range 0x168-1b0.
impl SaffireLeMeterProtocol {
    /// The number of destionation pairs.
    pub const LEVEL_MIN: i32 = 0;
    pub const LEVEL_MAX: i32 = 0x7fffffff;
    pub const LEVEL_STEP: i32 = 1;

    const PHYS_INPUT_OFFSETS: [usize; 6] = [
        0x168, // analog-input-0
        0x16c, // analog-input-2
        0x170, // digital-input-0
        0x174, // analog-input-1
        0x178, // analog-input-3
        0x17c, // digital-input-1
    ];
    const PHYS_OUTPUT_OFFSETS: [usize; 8] = [
        0x180, // analog-output-0
        0x184, // analog-output-2
        0x188, // analog-output-1
        0x18c, // analog-output-3
        0x190, // analog-output-4
        0x194, // digital-output-0
        0x198, // analog-output-5
        0x19c, // digital-output-1
    ];
    const STREAM_INPUT_OFFSETS: [usize; 4] = [
        0x1a0, // stream-input-0
        0x1a4, // stream-input-2
        0x1a8, // stream-input-1
        0x1ac, // stream-input-3
    ];
    const DIG_INPUT_DETECT_OFFSET: usize = 0x1b0;

    // Read meter information.
    pub fn read_meter(
        req: &FwReq,
        node: &FwNode,
        meter: &mut SaffireLeMeter,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let offsets: Vec<usize> = Self::PHYS_INPUT_OFFSETS
            .iter()
            .chain(Self::PHYS_OUTPUT_OFFSETS.iter())
            .chain(Self::STREAM_INPUT_OFFSETS.iter())
            .chain([Self::DIG_INPUT_DETECT_OFFSET].iter())
            .copied()
            .collect();
        let mut buf = vec![0; offsets.len() * 4];
        saffire_read_quadlets(req, node, &offsets, &mut buf, timeout_ms).map(|_| {
            let mut quadlet = [0; 4];
            let vals = (0..offsets.len()).fold(Vec::new(), |mut vals, i| {
                let pos = i * 4;
                quadlet.copy_from_slice(&buf[pos..(pos + 4)]);
                vals.push(i32::from_be_bytes(quadlet));
                vals
            });

            meter.phys_inputs[0] = vals[0];
            meter.phys_inputs[2] = vals[1];
            meter.phys_inputs[4] = vals[2];
            meter.phys_inputs[1] = vals[3];
            meter.phys_inputs[3] = vals[4];
            meter.phys_inputs[5] = vals[5];

            meter.phys_outputs[0] = vals[6];
            meter.phys_outputs[2] = vals[7];
            meter.phys_outputs[1] = vals[8];
            meter.phys_outputs[3] = vals[9];
            meter.phys_outputs[4] = vals[10];
            meter.phys_outputs[6] = vals[11];
            meter.phys_outputs[5] = vals[12];
            meter.phys_outputs[7] = vals[13];

            meter.stream_inputs[0] = vals[14];
            meter.stream_inputs[2] = vals[15];
            meter.stream_inputs[1] = vals[16];
            meter.stream_inputs[3] = vals[17];

            meter.dig_input_detect = vals[18] > 0;
        })
    }
}

/// The protocol implementation for operation of output parameters in Saffire.
#[derive(Default)]
pub struct SaffireLeOutputProtocol;

impl SaffireOutputOperation for SaffireLeOutputProtocol {
    // physical-output-1/2, 3/4, 5/6, 7/8, and digital-output-1/2.
    const OFFSETS: &'static [usize] = &[0x15c, 0x160, 0x164];

    const MUTE_COUNT: usize = 3;
    const VOL_COUNT: usize = 3;
    const HWCTL_COUNT: usize = 0;
    const DIM_COUNT: usize = 0;
    const PAD_COUNT: usize = 0;
}

/// The structure for parameters specific to Saffire.
#[derive(Default)]
pub struct SaffireLeSpecificParameters {
    pub analog_input_2_3_high_gains: [bool; 2],
}

/// The protocol implementation for operation specific to Saffire.
#[derive(Default)]
pub struct SaffireLeSpecificProtocol;

const LE_ANALOG_INTPUT_2_HIGH_GAIN_OFFSET: usize = 0x154;
const LE_ANALOG_INTPUT_3_HIGH_GAIN_OFFSET: usize = 0x158;

impl SaffireLeSpecificProtocol {
    /// Read parameters.
    pub fn read_params(
        req: &FwReq,
        node: &FwNode,
        params: &mut SaffireLeSpecificParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let offsets = [
            LE_ANALOG_INTPUT_2_HIGH_GAIN_OFFSET,
            LE_ANALOG_INTPUT_3_HIGH_GAIN_OFFSET,
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
            params.analog_input_2_3_high_gains[0] = vals[0] > 0;
            params.analog_input_2_3_high_gains[1] = vals[1] > 0;
        })
    }

    /// Enable/disable high gain of analog input 2 or 3.
    pub fn write_analog_input_high_gains(
        req: &FwReq,
        node: &FwNode,
        enables: &[bool],
        params: &mut SaffireLeSpecificParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let (offsets, buf) = params
            .analog_input_2_3_high_gains
            .iter()
            .zip(enables.iter())
            .zip(
                [
                    LE_ANALOG_INTPUT_2_HIGH_GAIN_OFFSET,
                    LE_ANALOG_INTPUT_3_HIGH_GAIN_OFFSET,
                ]
                .iter(),
            )
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
            .map(|_| params.analog_input_2_3_high_gains.copy_from_slice(enables))
    }
}

/// The enumeration for source of S/PDIF output.
#[derive(Debug)]
pub enum SaffireLeSpdifOutputSource {
    MixerOutputPair01,
    MixerOutputPair67,
}

impl Default for SaffireLeSpdifOutputSource {
    fn default() -> Self {
        Self::MixerOutputPair01
    }
}

/// The structure for mixer state at 44.1/48.0 kHz in Saffire LE.
#[derive(Default, Debug)]
pub struct SaffireLeMixerLowRateState {
    pub phys_src_gains: [[i16; 6]; 4],
    pub stream_src_gains: [[i16; 8]; 4],
    pub spdif_out_src: SaffireLeSpdifOutputSource,
}

/// The protocol implementation for operation of mixer at 44.1/48.0 kHz in Saffire LE.
#[derive(Default)]
pub struct SaffireLeMixerLowRateProtocol;

impl SaffireLeMixerLowRateProtocol {
    /// The number of destionation pairs.
    pub const LEVEL_MIN: i16 = 0;
    pub const LEVEL_MAX: i16 = 0x7fff;
    pub const LEVEL_STEP: i16 = 0x100;

    const OFFSETS: [usize; 57] = [
        // level from stream-input-0
        0x000, // output-1
        0x004, // output-3
        0x008, // output-0
        0x00c, // output-2
        // level from stream-input-2
        0x010, // output-1
        0x014, // output-3
        0x018, // output-0
        0x01c, // output-2
        // level from stream-input-4
        0x020, // output-1
        0x024, // output-3
        0x028, // output-0
        0x02c, // output-2
        // level from stream-input-6
        0x030, // output-1
        0x034, // output-3
        0x038, // output-0
        0x03c, // output-2
        // level from stream-input-1
        0x040, // output-1
        0x044, // output-3
        0x048, // output-0
        0x04c, // output-2
        // level from stream-input-3
        0x050, // output-1
        0x054, // output-3
        0x058, // output-0
        0x05c, // output-2
        // level from stream-input-5
        0x060, // output-1
        0x064, // output-3
        0x068, // output-0
        0x06c, // output-2
        // level from stream-input-7
        0x070, // output-1
        0x074, // output-3
        0x078, // output-0
        0x07c, // output-2
        // level from analog-input-0
        0x080, // to output-1
        0x084, // to output-3
        0x088, // to output-0
        0x08c, // to output-2
        // level from analog-input-2
        0x090, // to output-1
        0x094, // to output-3
        0x098, // to output-0
        0x09c, // to output-2
        // level from analog-input-4
        0x0a0, // to output-1
        0x0a4, // to output-3
        0x0a8, // to output-0
        0x0ac, // to output-2
        // level from analog-input-1
        0x0b0, // to output-1
        0x0b4, // to output-3
        0x0b8, // to output-0
        0x0bc, // to output-2
        // level from analog-input-3
        0x0c0, // to output-1
        0x0c4, // to output-3
        0x0c8, // to output-0
        0x0cc, // to output-2
        // level from analog-input-5
        0x0d0, // to output-1
        0x0d4, // to output-3
        0x0d8, // to output-0
        0x0dc, // to output-2
        // source of S/PDIF output 0/1
        0x100,
    ];

    #[inline(always)]
    fn stream_src_pos(dst_idx: usize, src_idx: usize) -> usize {
        (src_idx % 2 * 4 + src_idx / 2) * 4 + dst_idx % 2 * 2 + dst_idx / 2
    }

    #[inline(always)]
    fn phys_src_pos(dst_idx: usize, src_idx: usize) -> usize {
        32 + (src_idx % 2 * 3 + src_idx / 2) * 4 + dst_idx % 2 * 2 + dst_idx / 2
    }

    /// Read levels of physical source for indicated destination.
    pub fn read_src_gains(
        req: &FwReq,
        node: &FwNode,
        state: &mut SaffireLeMixerLowRateState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut buf = vec![0; Self::OFFSETS.len() * 4];
        saffire_read_quadlets(req, node, &Self::OFFSETS, &mut buf, timeout_ms).map(|_| {
            let mut quadlet = [0; 4];
            let vals = (0..Self::OFFSETS.len()).fold(Vec::new(), |mut vals, i| {
                let pos = i * 4;
                quadlet.copy_from_slice(&buf[pos..(pos + 4)]);
                vals.push(i32::from_be_bytes(quadlet) as i16);
                vals
            });

            state
                .stream_src_gains
                .iter_mut()
                .enumerate()
                .for_each(|(i, gains)| {
                    gains.iter_mut().enumerate().for_each(|(j, gain)| {
                        *gain = vals[Self::stream_src_pos(i, j)];
                    });
                });

            state
                .phys_src_gains
                .iter_mut()
                .enumerate()
                .for_each(|(i, gains)| {
                    gains.iter_mut().enumerate().for_each(|(j, gain)| {
                        *gain = vals[Self::phys_src_pos(i, j)];
                    });
                });

            state.spdif_out_src = if vals[56] > 0 {
                SaffireLeSpdifOutputSource::MixerOutputPair67
            } else {
                SaffireLeSpdifOutputSource::MixerOutputPair01
            };
        })
    }

    /// Write levels of stream source for indicated destination.
    pub fn write_phys_src_gains(
        req: &FwReq,
        node: &FwNode,
        dst_idx: usize,
        src_gains: &[i16],
        state: &mut SaffireLeMixerLowRateState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        write_src_levels(
            req,
            node,
            dst_idx,
            src_gains,
            &Self::OFFSETS,
            &mut state.phys_src_gains,
            timeout_ms,
            Self::phys_src_pos,
        )
    }

    /// Write levels of stream source for indicated destination.
    pub fn write_stream_src_gains(
        req: &FwReq,
        node: &FwNode,
        dst_idx: usize,
        src_gains: &[i16],
        state: &mut SaffireLeMixerLowRateState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        write_src_levels(
            req,
            node,
            dst_idx,
            src_gains,
            &Self::OFFSETS,
            &mut state.stream_src_gains,
            timeout_ms,
            Self::stream_src_pos,
        )
    }

    /// Write source of S/PDIF output.
    pub fn write_spdif_out_src(
        req: &FwReq,
        node: &FwNode,
        src: SaffireLeSpdifOutputSource,
        state: &mut SaffireLeMixerLowRateState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let val = match src {
            SaffireLeSpdifOutputSource::MixerOutputPair01 => 0,
            SaffireLeSpdifOutputSource::MixerOutputPair67 => 1,
        };
        let buf = u32::to_be_bytes(val);
        saffire_write_quadlets(req, node, &Self::OFFSETS[56..], &buf, timeout_ms)
            .map(|_| state.spdif_out_src = src)
    }
}

/// The structure for mixer state at 88.2/96.0 kHz in Saffire LE.
#[derive(Default, Debug)]
pub struct SaffireLeMixerMiddleRateState {
    pub monitor_src_phys_input_gains: [i16; 6],
    pub monitor_out_src_pair_gains: [[i16; 1]; 4],
    pub stream_src_pair_gains: [[i16; 2]; 4],
    pub spdif_out_src: SaffireLeSpdifOutputSource,
}

/// The protocol implementation for operation of mixer at 88.2/96.0 kHz in Saffire LE.
#[derive(Default)]
pub struct SaffireLeMixerMiddleRateProtocol;

impl SaffireLeMixerMiddleRateProtocol {
    pub const LEVEL_MIN: i16 = 0;
    pub const LEVEL_MAX: i16 = 0x7fff;
    pub const LEVEL_STEP: i16 = 0x100;

    const OFFSETS: [usize; 19] = [
        // level to monitor-output-0/1
        0x108, // from analog-input-0
        0x10c, // from analog-input-2
        0x110, // from analog-input-4
        0x114, // from analog-input-1
        0x118, // from analog-input-3
        0x11c, // from analog-input-5
        // level to output-0
        0x120, // from monitor-output-0/1
        0x124, // from stream-input-0/1
        0x128, // from stream-input-2/3
        // level to output-1
        0x12c, // from monitor-output-0/1
        0x130, // from stream-input-0/1
        0x134, // from stream-input-2/3
        // level to output-2
        0x138, // from monitor-output-0/1
        0x13c, // from stream-input-0/1
        0x140, // from stream-input-2/3
        // level to output-3
        0x144, // from monitor-output-0/1
        0x148, // from stream-input-0/1
        0x14c, // from stream-input-2/3
        // source of S/PDIF output 0/1.
        0x150,
    ];

    #[inline(always)]
    fn monitor_analog_input_pos(_: usize, src_idx: usize) -> usize {
        src_idx % 2 * 2 + src_idx / 2
    }

    #[inline(always)]
    fn mixer_monitor_src_pos(dst_idx: usize, src_idx: usize) -> usize {
        6 + (dst_idx * 3) + src_idx
    }

    #[inline(always)]
    fn mixer_stream_input_pos(dst_idx: usize, src_idx: usize) -> usize {
        6 + (dst_idx * 3) + src_idx + 1
    }

    /// Read levels of physical source for indicated destination.
    pub fn read_state(
        req: &FwReq,
        node: &FwNode,
        state: &mut SaffireLeMixerMiddleRateState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut buf = vec![0; Self::OFFSETS.len() * 4];
        saffire_read_quadlets(req, node, &Self::OFFSETS, &mut buf, timeout_ms).map(|_| {
            let mut quadlet = [0; 4];
            let vals = (0..Self::OFFSETS.len()).fold(Vec::new(), |mut vals, i| {
                let pos = i * 4;
                quadlet.copy_from_slice(&buf[pos..(pos + 4)]);
                vals.push(i32::from_be_bytes(quadlet) as i16);
                vals
            });

            state
                .monitor_src_phys_input_gains
                .iter_mut()
                .enumerate()
                .for_each(|(i, gain)| *gain = vals[Self::monitor_analog_input_pos(0, i)]);

            state
                .monitor_out_src_pair_gains
                .iter_mut()
                .enumerate()
                .for_each(|(i, gains)| {
                    gains
                        .iter_mut()
                        .enumerate()
                        .for_each(|(j, gain)| *gain = vals[Self::mixer_monitor_src_pos(i, j)]);
                });

            state
                .stream_src_pair_gains
                .iter_mut()
                .enumerate()
                .for_each(|(i, gains)| {
                    gains
                        .iter_mut()
                        .enumerate()
                        .for_each(|(j, gain)| *gain = vals[Self::mixer_stream_input_pos(i, j)]);
                });

            state.spdif_out_src = if vals[18] > 0 {
                SaffireLeSpdifOutputSource::MixerOutputPair67
            } else {
                SaffireLeSpdifOutputSource::MixerOutputPair01
            };
        })
    }

    /// Write levels of input source to monitor for indicated destination.
    pub fn write_monitor_src_phys_input_gains(
        req: &FwReq,
        node: &FwNode,
        src_gains: &[i16],
        state: &mut SaffireLeMixerMiddleRateState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        write_src_levels(
            req,
            node,
            0,
            src_gains,
            &Self::OFFSETS,
            &mut [&mut state.monitor_src_phys_input_gains],
            timeout_ms,
            Self::monitor_analog_input_pos,
        )
    }

    /// Write levels of monitor output source to mixer for indicated destination.
    pub fn write_monitor_out_src_pair_gains(
        req: &FwReq,
        node: &FwNode,
        dst_idx: usize,
        src_gains: &[i16],
        state: &mut SaffireLeMixerMiddleRateState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        write_src_levels(
            req,
            node,
            dst_idx,
            src_gains,
            &Self::OFFSETS,
            &mut state.monitor_out_src_pair_gains,
            timeout_ms,
            Self::mixer_monitor_src_pos,
        )
    }

    /// Write levels of stream source to monitor for indicated destination.
    pub fn write_stream_src_pair_gains(
        req: &FwReq,
        node: &FwNode,
        dst_idx: usize,
        src_gains: &[i16],
        state: &mut SaffireLeMixerMiddleRateState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        write_src_levels(
            req,
            node,
            dst_idx,
            src_gains,
            &Self::OFFSETS,
            &mut state.stream_src_pair_gains,
            timeout_ms,
            Self::mixer_stream_input_pos,
        )
    }

    /// Write source of S/PDIF output.
    pub fn write_spdif_out_src(
        req: &FwReq,
        node: &FwNode,
        src: SaffireLeSpdifOutputSource,
        state: &mut SaffireLeMixerMiddleRateState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let val = match src {
            SaffireLeSpdifOutputSource::MixerOutputPair01 => 0,
            SaffireLeSpdifOutputSource::MixerOutputPair67 => 1,
        };
        let buf = u32::to_be_bytes(val);
        saffire_write_quadlets(req, node, &Self::OFFSETS[18..], &buf, timeout_ms)
            .map(|_| state.spdif_out_src = src)
    }
}

/// The ptorocol implementation of AC3 and MIDI signal through.
#[derive(Default, Debug)]
pub struct SaffireLeThroughProtocol;

impl SaffireThroughOperation for SaffireLeThroughProtocol {
    const MIDI_THROUGH_OFFSET: usize = 0x01bc;
    const AC3_THROUGH_OFFSET: usize = 0x01c0;
}

/// The protocol implementation to store configuration in Saffire.
#[derive(Default)]
pub struct SaffireLeStoreConfigProtocol;

impl SaffireStoreConfigOperation for SaffireLeStoreConfigProtocol {
    const OFFSET: usize = 0x1b8;
}

/// The structure of mixer coefficiencies in Saffire and Saffire LE.
#[derive(Default, Debug)]
pub struct SaffireMixerState {
    pub phys_inputs: Vec<Vec<i16>>,
    pub reverb_returns: Vec<Vec<i16>>,
    pub stream_inputs: Vec<Vec<i16>>,
}

fn write_src_levels<T, F>(
    req: &FwReq,
    node: &FwNode,
    idx: usize,
    new_vals: &[i16],
    offset_list: &[usize],
    old_val_list: &mut [T],
    timeout_ms: u32,
    src_pos: F,
) -> Result<(), Error>
where
    T: AsMut<[i16]>,
    F: Fn(usize, usize) -> usize,
{
    let old_vals = old_val_list
        .iter_mut()
        .nth(idx)
        .ok_or_else(|| {
            let msg = format!("Invalid index of destination {}", idx);
            Error::new(FileError::Inval, &msg)
        })
        .map(|old_vals| old_vals.as_mut())?;

    if new_vals.len() != old_vals.len() {
        let msg = format!(
            "Invalid length of value {}, but expected {}",
            new_vals.len(),
            old_vals.len()
        );
        Err(Error::new(FileError::Inval, &msg))?;
    }

    let (offsets, buf) = old_vals
        .iter()
        .zip(new_vals.iter())
        .enumerate()
        .filter(|(_, (old, new))| !new.eq(old))
        .fold(
            (Vec::new(), Vec::new()),
            |(mut offsets, mut buf), (i, (_, &new))| {
                offsets.push(offset_list[src_pos(idx, i)]);
                buf.extend_from_slice(&(new as i32).to_be_bytes());
                (offsets, buf)
            },
        );

    saffire_write_quadlets(req, node, &offsets, &buf, timeout_ms)
        .map(|_| old_vals.copy_from_slice(new_vals))
}

/// The trait for mixer operation in Saffire.
pub trait SaffireMixerOperation {
    const OFFSETS: &'static [usize];

    const PHYS_INPUT_COUNT: usize;
    const REVERB_RETURN_COUNT: usize;

    fn stream_src_pos(dst_idx: usize, src_idx: usize) -> usize;
    fn phys_src_pos(dst_idx: usize, src_idx: usize) -> usize;
    fn reverb_return_pos(dst_idx: usize, src_idx: usize) -> usize;

    const STREAM_INPUT_COUNT: usize = 5;
    const OUTPUT_PAIR_COUNT: usize = 5;

    const LEVEL_MIN: i16 = 0x0000;
    const LEVEL_MAX: i16 = 0x7fff;
    const LEVEL_STEP: i16 = 0x100;

    fn create_mixer_state() -> SaffireMixerState {
        SaffireMixerState {
            phys_inputs: vec![vec![0; Self::PHYS_INPUT_COUNT]; Self::OUTPUT_PAIR_COUNT],
            reverb_returns: vec![vec![0; Self::REVERB_RETURN_COUNT]; Self::OUTPUT_PAIR_COUNT],
            stream_inputs: vec![vec![0; Self::STREAM_INPUT_COUNT]; Self::OUTPUT_PAIR_COUNT],
        }
    }

    fn read_mixer_state(
        req: &FwReq,
        node: &FwNode,
        state: &mut SaffireMixerState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut buf = vec![0; Self::OFFSETS.len() * 4];
        saffire_read_quadlets(req, node, &Self::OFFSETS, &mut buf, timeout_ms).map(|_| {
            let mut quadlet = [0; 4];
            let vals = (0..Self::OFFSETS.len()).fold(Vec::new(), |mut vals, i| {
                let pos = i * 4;
                quadlet.copy_from_slice(&buf[pos..(pos + 4)]);
                vals.push(i32::from_be_bytes(quadlet) as i16);
                vals
            });

            state
                .phys_inputs
                .iter_mut()
                .enumerate()
                .for_each(|(dst_idx, gains)| {
                    gains.iter_mut().enumerate().for_each(|(src_idx, gain)| {
                        let pos = Self::phys_src_pos(dst_idx, src_idx);
                        *gain = vals[pos];
                    });
                });

            state
                .reverb_returns
                .iter_mut()
                .enumerate()
                .for_each(|(dst_idx, gains)| {
                    gains.iter_mut().enumerate().for_each(|(src_idx, gain)| {
                        let pos = Self::reverb_return_pos(dst_idx, src_idx);
                        *gain = vals[pos];
                    });
                });

            state
                .stream_inputs
                .iter_mut()
                .enumerate()
                .for_each(|(dst_idx, gains)| {
                    gains.iter_mut().enumerate().for_each(|(src_idx, gain)| {
                        let pos = Self::stream_src_pos(dst_idx, src_idx);
                        *gain = vals[pos];
                    });
                });
        })
    }

    fn write_mixer_state(
        req: &FwReq,
        node: &FwNode,
        state: &mut SaffireMixerState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut offsets = Vec::new();
        let mut buf = Vec::new();

        state
            .phys_inputs
            .iter()
            .enumerate()
            .for_each(|(dst_idx, gains)| {
                gains.iter().enumerate().for_each(|(src_idx, &gain)| {
                    let pos = Self::phys_src_pos(dst_idx, src_idx);
                    offsets.push(Self::OFFSETS[pos]);
                    buf.extend_from_slice(&(gain as i32).to_be_bytes());
                });
            });

        state
            .reverb_returns
            .iter()
            .enumerate()
            .for_each(|(dst_idx, gains)| {
                gains.iter().enumerate().for_each(|(src_idx, &gain)| {
                    let pos = Self::reverb_return_pos(dst_idx, src_idx);
                    offsets.push(Self::OFFSETS[pos]);
                    buf.extend_from_slice(&(gain as i32).to_be_bytes());
                });
            });

        state
            .stream_inputs
            .iter()
            .enumerate()
            .for_each(|(dst_idx, gains)| {
                gains.iter().enumerate().for_each(|(src_idx, &gain)| {
                    let pos = Self::stream_src_pos(dst_idx, src_idx);
                    offsets.push(Self::OFFSETS[pos]);
                    buf.extend_from_slice(&(gain as i32).to_be_bytes());
                });
            });

        saffire_write_quadlets(req, node, &offsets, &mut buf, timeout_ms)
    }

    fn write_phys_inputs(
        req: &FwReq,
        node: &FwNode,
        idx: usize,
        levels: &[i16],
        state: &mut SaffireMixerState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        write_src_levels(
            req,
            node,
            idx,
            levels,
            &Self::OFFSETS,
            &mut state.phys_inputs,
            timeout_ms,
            Self::phys_src_pos,
        )
    }

    fn write_reverb_returns(
        req: &FwReq,
        node: &FwNode,
        idx: usize,
        levels: &[i16],
        state: &mut SaffireMixerState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        write_src_levels(
            req,
            node,
            idx,
            levels,
            &Self::OFFSETS,
            &mut state.reverb_returns,
            timeout_ms,
            Self::reverb_return_pos,
        )
    }

    fn write_stream_inputs(
        req: &FwReq,
        node: &FwNode,
        idx: usize,
        levels: &[i16],
        state: &mut SaffireMixerState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        write_src_levels(
            req,
            node,
            idx,
            levels,
            &Self::OFFSETS,
            &mut state.stream_inputs,
            timeout_ms,
            Self::stream_src_pos,
        )
    }
}
