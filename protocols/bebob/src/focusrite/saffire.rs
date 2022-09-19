// SPDX-License-Identifier: LGPL-3.0-or-later
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

        let mut op = InputPlugSignalFormat(PlugSignalFormat {
            plug_id: 0,
            fmt: FMT_IS_AMDTP,
            fdf: fdf.into(),
        });
        avc.control(&AvcAddr::Unit, &mut op, timeout_ms)?;

        let mut op = OutputPlugSignalFormat(PlugSignalFormat {
            plug_id: 0,
            fmt: FMT_IS_AMDTP,
            fdf: fdf.into(),
        });
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

/// Information of hardware metering in Saffire.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct SaffireMeter {
    pub phys_inputs: [i32; 4],
    pub dig_input_detect: bool,
    pub monitor_knob_value: u16,
}

/// The protocol implementation of metering in Saffire.
#[derive(Default, Debug)]
pub struct SaffireMeterProtocol;

impl SaffireMeterProtocol {
    pub const MONITOR_KNOB_MIN: u16 = 0;
    pub const MONITOR_KNOB_MAX: u16 = 0x1ff0;
    pub const MONITOR_KNOB_STEP: u16 = 0x10;

    pub const LEVEL_MIN: i32 = 0;
    pub const LEVEL_MAX: i32 = 0x7fffffff;
    pub const LEVEL_STEP: i32 = 1;

    const OFFSETS: &'static [usize] = &[
        0x00f4, // The value of monitor knob.
        0x0100, // The signal level of analog-input-0.
        0x0104, // The signal level of analog-input-2.
        0x0108, // The signal level of analog-input-1.
        0x010c, // The signal level of analog-input-3.
        0x013c, // Whether to detect digital input.
    ];

    pub fn cache(
        req: &FwReq,
        node: &FwNode,
        meter: &mut SaffireMeter,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut raw = [0u8; Self::OFFSETS.len() * 4];

        saffire_read_quadlets(req, node, Self::OFFSETS, &mut raw, timeout_ms).map(|_| {
            let mut quadlet = [0u8; 4];
            quadlet.copy_from_slice(&raw[..4]);
            meter.monitor_knob_value = (u32::from_be_bytes(quadlet) & 0x0000ffff) as u16;

            meter
                .phys_inputs
                .iter_mut()
                .enumerate()
                .for_each(|(i, level)| {
                    let pos = 4 + i * 4;
                    quadlet.copy_from_slice(&raw[pos..(pos + 4)]);
                    *level = i32::from_be_bytes(quadlet);
                });

            quadlet.copy_from_slice(&raw[20..]);
            meter.dig_input_detect = u32::from_be_bytes(quadlet) > 0;
        })
    }
}

/// The protocol implementation for operation of output parameters in Saffire.
#[derive(Default, Debug)]
pub struct SaffireOutputProtocol;

impl SaffireOutputSpecification for SaffireOutputProtocol {
    // physical-output-1/2, 3/4, 5/6, 7/8, and digital-output-1/2.
    const OUTPUT_OFFSETS: &'static [usize] = &[0xdc, 0xe0, 0xe4, 0xe8, 0xec];

    const MUTE_COUNT: usize = 5;
    const VOL_COUNT: usize = 4;
    const HWCTL_COUNT: usize = 4;
    const DIM_COUNT: usize = 1;
    const PAD_COUNT: usize = 0;
}

/// The signal source of input 2/3.
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

/// The mode of signal multiplexer.
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

/// Parameters specific to Saffire.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct SaffireSpecificParameters {
    pub mode_192khz: bool,
    pub input_pair_1_src: SaffireInputPair1Source,
    pub mixer_mode: SaffireMixerMode,
}

/// The protocol implementation for operation specific to Saffire.
#[derive(Default, Debug)]
pub struct SaffireSpecificProtocol;

impl SaffireParametersSerdes<SaffireSpecificParameters> for SaffireSpecificProtocol {
    const OFFSETS: &'static [usize] = &[
        SAFFIRE_MODE_192KHZ_OFFSET,
        SAFFIRE_INPUT_PAIR1_SRC_OFFSET,
        SAFFIRE_MIXER_MODE_OFFSET,
    ];

    fn serialize(params: &SaffireSpecificParameters, raw: &mut [u8]) {
        raw[..4].copy_from_slice(&(params.mode_192khz as u32).to_be_bytes());

        let val: u32 = if params.input_pair_1_src == SaffireInputPair1Source::DigitalInputPair0 {
            1
        } else {
            0
        };
        raw[4..8].copy_from_slice(&val.to_be_bytes());

        let val: u32 = if params.mixer_mode == SaffireMixerMode::StereoSeparated {
            1
        } else {
            0
        };
        raw[8..12].copy_from_slice(&val.to_be_bytes());
    }

    fn deserialize(params: &mut SaffireSpecificParameters, raw: &[u8]) {
        let mut quadlet = [0u8; 4];

        let quads: Vec<u32> = (0..raw.len())
            .step_by(4)
            .map(|pos| {
                quadlet.copy_from_slice(&raw[pos..(pos + 4)]);
                u32::from_be_bytes(quadlet)
            })
            .collect();

        params.mode_192khz = quads[0] > 0;

        params.input_pair_1_src = if quads[1] > 0 {
            SaffireInputPair1Source::DigitalInputPair0
        } else {
            SaffireInputPair1Source::AnalogInputPair0
        };

        params.mixer_mode = if quads[2] > 0 {
            SaffireMixerMode::StereoSeparated
        } else {
            SaffireMixerMode::StereoPaired
        };
    }
}

const SAFFIRE_INPUT_PAIR1_SRC_OFFSET: usize = 0xf8;
const SAFFIRE_MIXER_MODE_OFFSET: usize = 0xfc;

/// The protocol implementation for operation of mixer at stereo separated mode in Saffire.
#[derive(Default, Debug)]
pub struct SaffireSeparatedMixerProtocol;

impl SaffireMixerSpecification for SaffireSeparatedMixerProtocol {
    const MIXER_OFFSETS: &'static [usize] = &[
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
    fn phys_src_pos(dst_idx: usize, src_idx: usize) -> usize {
        ((dst_idx + 1) % 5) + ((src_idx % 2 * 15) + (src_idx / 2 * 5))
    }

    #[inline(always)]
    fn reverb_return_pos(dst_idx: usize, src_idx: usize) -> usize {
        10 + ((dst_idx + 1) % 5) + ((src_idx % 2 * 15) + (src_idx / 2 * 5))
    }

    #[inline(always)]
    fn stream_src_pos(dst_idx: usize, src_idx: usize) -> usize {
        30 + ((dst_idx + 1) % 5) + ((src_idx + 1) % 5) * 5
    }
}

/// The protocol implementation for operation of mixer at stereo paired mode in Saffire.
#[derive(Default, Debug)]
pub struct SaffirePairedMixerProtocol;

impl SaffireMixerSpecification for SaffirePairedMixerProtocol {
    const MIXER_OFFSETS: &'static [usize] = &[
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
    fn stream_src_pos(dst_idx: usize, src_idx: usize) -> usize {
        ((src_idx + 1) % 5 * 5) + ((dst_idx + 1) % 5)
    }

    #[inline(always)]
    fn phys_src_pos(dst_idx: usize, src_idx: usize) -> usize {
        25 + src_idx * 5 + (dst_idx + 1) % 5
    }

    #[inline(always)]
    fn reverb_return_pos(dst_idx: usize, _: usize) -> usize {
        35 + (dst_idx + 1) % 5
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
#[derive(Default, Debug)]
pub struct SaffireStoreConfigProtocol;

impl SaffireStoreConfigSpecification for SaffireStoreConfigProtocol {
    const STORE_CONFIG_OFFSETS: &'static [usize] = &[0x148];
}

/// Information of hardware metering in Saffire LE.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct SaffireLeMeter {
    pub phys_inputs: [i32; 6],
    pub phys_outputs: [i32; 8],
    pub stream_inputs: [i32; 4],
    pub dig_input_detect: bool,
}

/// The protocol implementation of metering in Saffire LE.
#[derive(Default, Debug)]
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

    /// Cache the state of hardware to the parameter.
    pub fn cache(
        req: &FwReq,
        node: &FwNode,
        meter: &mut SaffireLeMeter,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let offsets: Vec<usize> = Self::PHYS_INPUT_OFFSETS
            .iter()
            .chain(&Self::PHYS_OUTPUT_OFFSETS)
            .chain(&Self::STREAM_INPUT_OFFSETS)
            .chain(&[Self::DIG_INPUT_DETECT_OFFSET])
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
#[derive(Default, Debug)]
pub struct SaffireLeOutputProtocol;

impl SaffireOutputSpecification for SaffireLeOutputProtocol {
    // physical-output-1/2, 3/4, 5/6, 7/8, and digital-output-1/2.
    const OUTPUT_OFFSETS: &'static [usize] = &[0x15c, 0x160, 0x164];

    const MUTE_COUNT: usize = 3;
    const VOL_COUNT: usize = 3;
    const HWCTL_COUNT: usize = 0;
    const DIM_COUNT: usize = 0;
    const PAD_COUNT: usize = 0;
}

/// The parameters specific to Saffire.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct SaffireLeSpecificParameters {
    pub analog_input_2_3_high_gains: [bool; 2],
}

/// The protocol implementation for operation specific to Saffire.
#[derive(Default, Debug)]
pub struct SaffireLeSpecificProtocol;

impl SaffireParametersSerdes<SaffireLeSpecificParameters> for SaffireLeSpecificProtocol {
    const OFFSETS: &'static [usize] = &[
        0x154, // High gain mode at 3rd analog input.
        0x158, // High gain mode at 4th analog input.
    ];

    fn serialize(params: &SaffireLeSpecificParameters, raw: &mut [u8]) {
        raw[..4].copy_from_slice(&(params.analog_input_2_3_high_gains[0] as u32).to_be_bytes());
        raw[4..8].copy_from_slice(&(params.analog_input_2_3_high_gains[1] as u32).to_be_bytes());
    }

    fn deserialize(params: &mut SaffireLeSpecificParameters, raw: &[u8]) {
        let mut quadlet = [0u8; 4];

        let quads: Vec<u32> = (0..raw.len())
            .step_by(4)
            .map(|pos| {
                quadlet.copy_from_slice(&raw[pos..(pos + 4)]);
                u32::from_be_bytes(quadlet)
            })
            .collect();

        params
            .analog_input_2_3_high_gains
            .iter_mut()
            .enumerate()
            .for_each(|(i, enabled)| *enabled = quads[i] > 0);
    }
}

/// Signal source of S/PDIF output.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SaffireLeSpdifOutputSource {
    MixerOutputPair01,
    MixerOutputPair67,
}

impl Default for SaffireLeSpdifOutputSource {
    fn default() -> Self {
        Self::MixerOutputPair01
    }
}

/// State of signal multiplexer in Saffire LE at 44.1/48.0 kHz.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct SaffireLeMixerLowRateState {
    pub phys_src_gains: [[i16; 6]; 4],
    pub stream_src_gains: [[i16; 8]; 4],
    pub spdif_out_src: SaffireLeSpdifOutputSource,
}

/// The protocol implementation for operation of mixer at 44.1/48.0 kHz in Saffire LE.
#[derive(Default, Debug)]
pub struct SaffireLeMixerLowRateProtocol;

impl SaffireParametersSerdes<SaffireLeMixerLowRateState> for SaffireLeMixerLowRateProtocol {
    const OFFSETS: &'static [usize] = &[
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

    fn serialize(params: &SaffireLeMixerLowRateState, raw: &mut [u8]) {
        params
            .stream_src_gains
            .iter()
            .enumerate()
            .for_each(|(dst_idx, gains)| {
                gains.iter().enumerate().for_each(|(src_idx, &gain)| {
                    let pos = SaffireLeMixerLowRateProtocol::stream_src_pos(dst_idx, src_idx) * 4;
                    let gain = gain as i32;
                    raw[pos..(pos + 4)].copy_from_slice(&gain.to_be_bytes());
                });
            });

        params
            .phys_src_gains
            .iter()
            .enumerate()
            .for_each(|(dst_idx, gains)| {
                gains.iter().enumerate().for_each(|(src_idx, &gain)| {
                    let pos = SaffireLeMixerLowRateProtocol::phys_src_pos(dst_idx, src_idx) * 4;
                    let gain = gain as i32;
                    raw[pos..(pos + 4)].copy_from_slice(&gain.to_be_bytes());
                });
            });

        let val = match params.spdif_out_src {
            SaffireLeSpdifOutputSource::MixerOutputPair67 => 1u32,
            SaffireLeSpdifOutputSource::MixerOutputPair01 => 0,
        };
        raw[224..228].copy_from_slice(&val.to_be_bytes());
    }

    fn deserialize(params: &mut SaffireLeMixerLowRateState, raw: &[u8]) {
        let mut quadlet = [0u8; 4];

        let quads: Vec<u32> = (0..raw.len())
            .step_by(4)
            .map(|pos| {
                quadlet.copy_from_slice(&raw[pos..(pos + 4)]);
                u32::from_be_bytes(quadlet)
            })
            .collect();

        params
            .stream_src_gains
            .iter_mut()
            .enumerate()
            .for_each(|(dst_idx, gains)| {
                gains.iter_mut().enumerate().for_each(|(src_idx, gain)| {
                    let pos = SaffireLeMixerLowRateProtocol::stream_src_pos(dst_idx, src_idx);
                    *gain = quads[pos] as i16;
                })
            });

        params
            .phys_src_gains
            .iter_mut()
            .enumerate()
            .for_each(|(dst_idx, gains)| {
                gains.iter_mut().enumerate().for_each(|(src_idx, gain)| {
                    let pos = SaffireLeMixerLowRateProtocol::phys_src_pos(dst_idx, src_idx);
                    *gain = quads[pos] as i16;
                })
            });

        params.spdif_out_src = if quads[56] != 0 {
            SaffireLeSpdifOutputSource::MixerOutputPair67
        } else {
            SaffireLeSpdifOutputSource::MixerOutputPair01
        };
    }
}

impl SaffireLeMixerLowRateProtocol {
    /// The number of destionation pairs.
    pub const LEVEL_MIN: i16 = 0;
    pub const LEVEL_MAX: i16 = 0x7fff;
    pub const LEVEL_STEP: i16 = 0x100;

    #[inline(always)]
    fn stream_src_pos(dst_idx: usize, src_idx: usize) -> usize {
        assert!(dst_idx < 4);
        assert!(src_idx < 8);

        Self::compute_pos(dst_idx, src_idx, 4, 8)
    }

    #[inline(always)]
    fn phys_src_pos(dst_idx: usize, src_idx: usize) -> usize {
        assert!(dst_idx < 4);
        assert!(src_idx < 6);

        32 + Self::compute_pos(dst_idx, src_idx, 4, 6)
    }

    #[inline(always)]
    fn compute_pos(dst_idx: usize, src_idx: usize, dst_count: usize, src_count: usize) -> usize {
        let pair_gap = dst_count * src_count / 2;
        ((1 - dst_idx % 2) * 2 + dst_idx / 2 % 2) + (src_idx % 2 * pair_gap + src_idx / 2 * 4)
    }
}

/// State of signal multiplexer in Saffire LE at 88.2/96.0 kHz.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct SaffireLeMixerMiddleRateState {
    pub monitor_src_phys_input_gains: [i16; 6],
    pub monitor_out_src_pair_gains: [[i16; 1]; 4],
    pub stream_src_pair_gains: [[i16; 2]; 4],
    pub spdif_out_src: SaffireLeSpdifOutputSource,
}

impl SaffireParametersSerdes<SaffireLeMixerMiddleRateState> for SaffireLeMixerMiddleRateProtocol {
    const OFFSETS: &'static [usize] = &[
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

    fn serialize(params: &SaffireLeMixerMiddleRateState, raw: &mut [u8]) {
        params
            .monitor_src_phys_input_gains
            .iter()
            .enumerate()
            .for_each(|(src_idx, &gain)| {
                let gain = gain as i32;
                let pos =
                    SaffireLeMixerMiddleRateProtocol::monitor_analog_input_pos(0, src_idx) * 4;
                raw[pos..(pos + 4)].copy_from_slice(&gain.to_be_bytes());
            });

        params
            .monitor_out_src_pair_gains
            .iter()
            .enumerate()
            .for_each(|(dst_idx, gains)| {
                gains.iter().enumerate().for_each(|(src_idx, &gain)| {
                    let gain = gain as i32;
                    let pos =
                        SaffireLeMixerMiddleRateProtocol::mixer_monitor_src_pos(dst_idx, src_idx)
                            * 4;
                    raw[pos..(pos + 4)].copy_from_slice(&gain.to_be_bytes());
                });
            });

        params
            .stream_src_pair_gains
            .iter()
            .enumerate()
            .for_each(|(dst_idx, gains)| {
                gains.iter().enumerate().for_each(|(src_idx, &gain)| {
                    let gain = gain as i32;
                    let pos =
                        SaffireLeMixerMiddleRateProtocol::mixer_stream_input_pos(dst_idx, src_idx)
                            * 4;
                    raw[pos..(pos + 4)].copy_from_slice(&gain.to_be_bytes());
                });
            });

        let val = if params.spdif_out_src == SaffireLeSpdifOutputSource::MixerOutputPair67 {
            1u32
        } else {
            0
        };
        raw[72..76].copy_from_slice(&val.to_be_bytes());
    }

    fn deserialize(params: &mut SaffireLeMixerMiddleRateState, raw: &[u8]) {
        let mut quadlet = [0; 4];

        let quads: Vec<i16> = (0..raw.len())
            .step_by(4)
            .map(|pos| {
                quadlet.copy_from_slice(&raw[pos..(pos + 4)]);
                i32::from_be_bytes(quadlet) as i16
            })
            .collect();

        params
            .monitor_src_phys_input_gains
            .iter_mut()
            .enumerate()
            .for_each(|(src_idx, gain)| {
                let pos = SaffireLeMixerMiddleRateProtocol::monitor_analog_input_pos(0, src_idx);
                *gain = quads[pos];
            });

        params
            .monitor_out_src_pair_gains
            .iter_mut()
            .enumerate()
            .for_each(|(dst_idx, gains)| {
                gains.iter_mut().enumerate().for_each(|(src_idx, gain)| {
                    let pos =
                        SaffireLeMixerMiddleRateProtocol::mixer_monitor_src_pos(dst_idx, src_idx);
                    *gain = quads[pos];
                });
            });

        params
            .stream_src_pair_gains
            .iter_mut()
            .enumerate()
            .for_each(|(dst_idx, gains)| {
                gains.iter_mut().enumerate().for_each(|(src_idx, gain)| {
                    let pos =
                        SaffireLeMixerMiddleRateProtocol::mixer_stream_input_pos(dst_idx, src_idx);
                    *gain = quads[pos];
                });
            });

        params.spdif_out_src = if quads[18] > 0 {
            SaffireLeSpdifOutputSource::MixerOutputPair67
        } else {
            SaffireLeSpdifOutputSource::MixerOutputPair01
        };
    }
}

/// The protocol implementation for operation of mixer at 88.2/96.0 kHz in Saffire LE.
#[derive(Default, Debug)]
pub struct SaffireLeMixerMiddleRateProtocol;

impl SaffireLeMixerMiddleRateProtocol {
    pub const LEVEL_MIN: i16 = 0;
    pub const LEVEL_MAX: i16 = 0x7fff;
    pub const LEVEL_STEP: i16 = 0x100;

    #[inline(always)]
    fn monitor_analog_input_pos(_: usize, src_idx: usize) -> usize {
        src_idx % 2 * 3 + src_idx / 2
    }

    #[inline(always)]
    fn mixer_monitor_src_pos(dst_idx: usize, src_idx: usize) -> usize {
        6 + (dst_idx * 3) + src_idx
    }

    #[inline(always)]
    fn mixer_stream_input_pos(dst_idx: usize, src_idx: usize) -> usize {
        6 + (dst_idx * 3) + src_idx + 1
    }
}

/// The ptorocol implementation of AC3 and MIDI signal through.
#[derive(Default, Debug)]
pub struct SaffireLeThroughProtocol;

impl SaffireThroughSpecification for SaffireLeThroughProtocol {
    const THROUGH_OFFSETS: &'static [usize] = &[0x01bc, 0x01c0];
}

/// The protocol implementation to store configuration in Saffire.
#[derive(Default, Debug)]
pub struct SaffireLeStoreConfigProtocol;

impl SaffireStoreConfigSpecification for SaffireLeStoreConfigProtocol {
    const STORE_CONFIG_OFFSETS: &'static [usize] = &[0x1b8];
}

/// State of signal multiplexer in Saffire.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaffireMixerState {
    pub phys_inputs: Vec<Vec<i16>>,
    pub reverb_returns: Vec<Vec<i16>>,
    pub stream_inputs: Vec<Vec<i16>>,
}

/// The specification of protocol for mixer function.
pub trait SaffireMixerSpecification {
    const MIXER_OFFSETS: &'static [usize];

    const PHYS_INPUT_COUNT: usize;
    const REVERB_RETURN_COUNT: usize;

    fn stream_src_pos(dst_idx: usize, src_idx: usize) -> usize;
    fn phys_src_pos(dst_idx: usize, src_idx: usize) -> usize;
    fn reverb_return_pos(dst_idx: usize, src_idx: usize) -> usize;
}

impl<O: SaffireMixerSpecification> SaffireParametersSerdes<SaffireMixerState> for O {
    const OFFSETS: &'static [usize] = O::MIXER_OFFSETS;

    fn serialize(params: &SaffireMixerState, raw: &mut [u8]) {
        params
            .phys_inputs
            .iter()
            .enumerate()
            .for_each(|(dst_idx, gains)| {
                gains.iter().enumerate().for_each(|(src_idx, &gain)| {
                    let pos = O::phys_src_pos(dst_idx, src_idx) * 4;
                    let gain = gain as i32;
                    raw[pos..(pos + 4)].copy_from_slice(&gain.to_be_bytes());
                });
            });

        params
            .reverb_returns
            .iter()
            .enumerate()
            .for_each(|(dst_idx, gains)| {
                gains.iter().enumerate().for_each(|(src_idx, &gain)| {
                    let pos = O::reverb_return_pos(dst_idx, src_idx) * 4;
                    let gain = gain as i32;
                    raw[pos..(pos + 4)].copy_from_slice(&gain.to_be_bytes());
                });
            });

        params
            .stream_inputs
            .iter()
            .enumerate()
            .for_each(|(dst_idx, gains)| {
                gains.iter().enumerate().for_each(|(src_idx, &gain)| {
                    let pos = O::stream_src_pos(dst_idx, src_idx) * 4;
                    let gain = gain as i32;
                    raw[pos..(pos + 4)].copy_from_slice(&gain.to_be_bytes());
                });
            });
    }

    fn deserialize(params: &mut SaffireMixerState, raw: &[u8]) {
        let mut quadlet = [0; 4];

        let quads: Vec<i16> = (0..raw.len())
            .step_by(4)
            .map(|pos| {
                quadlet.copy_from_slice(&raw[pos..(pos + 4)]);
                i32::from_be_bytes(quadlet) as i16
            })
            .collect();

        params
            .phys_inputs
            .iter_mut()
            .enumerate()
            .for_each(|(dst_idx, gains)| {
                gains.iter_mut().enumerate().for_each(|(src_idx, gain)| {
                    let pos = O::phys_src_pos(dst_idx, src_idx);
                    *gain = quads[pos];
                });
            });

        params
            .reverb_returns
            .iter_mut()
            .enumerate()
            .for_each(|(dst_idx, gains)| {
                gains.iter_mut().enumerate().for_each(|(src_idx, gain)| {
                    let pos = O::reverb_return_pos(dst_idx, src_idx);
                    *gain = quads[pos];
                });
            });

        params
            .stream_inputs
            .iter_mut()
            .enumerate()
            .for_each(|(dst_idx, gains)| {
                gains.iter_mut().enumerate().for_each(|(src_idx, gain)| {
                    let pos = O::stream_src_pos(dst_idx, src_idx);
                    *gain = quads[pos];
                });
            });
    }
}

/// The trait for mixer operation in Saffire.
pub trait SaffireMixerOperation: SaffireMixerSpecification {
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
}

impl<O: SaffireMixerSpecification> SaffireMixerOperation for O {}

/// State of stereo-separated reverb effect in Saffire.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct SaffireReverbParameters {
    pub amounts: [i32; 2],
    pub room_sizes: [i32; 2],
    pub diffusions: [i32; 2],
    pub tones: [i32; 2],
}

/// The protocol implementation to operate parameters of reverb effect.
///
/// parameters | ch0    | ch1    | minimum    | maximum    | min val  | max val
/// ---------- | ------ | ------ | ---------- | ---------- | -------- | --------
/// amount     | 0x1004 | 0x1018 | 0x00000000 | 0x7fffffff |    -     |    -
/// room_size  | 0x1008 | 0x101c | 0x00000000 | 0x7fffffff |    -     |    -
/// diffusion  | 0x100c | 0x1020 | 0x00000000 | 0x7fffffff |    -     |    -
/// tone sign  | 0x1010 | 0x1024 | 0x00000000 | 0x00000001 | positive | negative
/// tone value | 0x1014 | 0x1028 | 0x00000000 | 0x7fffffff |    -     |    -
#[derive(Default, Debug)]
pub struct SaffireReverbProtocol;

impl SaffireParametersSerdes<SaffireReverbParameters> for SaffireReverbProtocol {
    const OFFSETS: &'static [usize] = &[
        // ch 0
        0x1004, // amount
        0x1008, // room-size
        0x100c, // diffusion
        0x1010, // tone-negative
        0x1014, // tone-value
        // ch 1
        0x1018, // amount
        0x101c, // room-size
        0x1020, // diffusion
        0x1024, // tone-negative
        0x1028, // tone-value
    ];

    fn serialize(params: &SaffireReverbParameters, raw: &mut [u8]) {
        raw[..4].copy_from_slice(&params.amounts[0].to_be_bytes());
        raw[4..8].copy_from_slice(&params.room_sizes[0].to_be_bytes());
        raw[8..12].copy_from_slice(&params.diffusions[0].to_be_bytes());
        Self::build_tone(&mut raw[12..20], params.tones[0]);

        raw[20..24].copy_from_slice(&params.amounts[1].to_be_bytes());
        raw[24..28].copy_from_slice(&params.room_sizes[1].to_be_bytes());
        raw[28..32].copy_from_slice(&params.diffusions[1].to_be_bytes());
        Self::build_tone(&mut raw[32..40], params.tones[1]);
    }

    fn deserialize(params: &mut SaffireReverbParameters, raw: &[u8]) {
        let mut quadlet = [0u8; 4];

        let quads: Vec<i32> = (0..raw.len())
            .step_by(4)
            .map(|pos| {
                quadlet.copy_from_slice(&raw[pos..(pos + 4)]);
                i32::from_be_bytes(quadlet)
            })
            .collect();

        params.amounts[0] = quads[0];
        params.room_sizes[0] = quads[1];
        params.diffusions[0] = quads[2];
        params.tones[0] = Self::parse_tone(&quads[3..5]);
        params.amounts[1] = quads[5];
        params.room_sizes[1] = quads[6];
        params.diffusions[1] = quads[7];
        params.tones[1] = Self::parse_tone(&quads[8..10]);
    }
}

impl SaffireReverbProtocol {
    pub const AMOUNT_MIN: i32 = 0x00000000;
    pub const AMOUNT_MAX: i32 = 0x7fffffff;
    pub const AMOUNT_STEP: i32 = 0x00000001;

    pub const ROOM_SIZE_MIN: i32 = 0x00000000;
    pub const ROOM_SIZE_MAX: i32 = 0x7fffffff;
    pub const ROOM_SIZE_STEP: i32 = 0x00000001;

    pub const DIFFUSION_MIN: i32 = 0x00000000;
    pub const DIFFUSION_MAX: i32 = 0x7fffffff;
    pub const DIFFUSION_STEP: i32 = 0x00000001;

    pub const TONE_MIN: i32 = i32::MIN + 1;
    pub const TONE_MAX: i32 = i32::MAX;
    pub const TONE_STEP: i32 = 0x00000001;

    fn parse_tone(vals: &[i32]) -> i32 {
        assert_eq!(vals.len(), 2);
        let mut tone = vals[1];
        if vals[0] > 0 {
            tone *= -1;
        }
        tone
    }

    fn build_tone(vals: &mut [u8], tone: i32) {
        assert_eq!(vals.len(), 8);
        vals[..4].copy_from_slice(&((tone < 0) as u32).to_be_bytes());
        vals[4..].copy_from_slice(&(tone.abs() as u32).to_be_bytes());
    }
}

/// Parameters of compressor effect in Saffire.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct SaffireCompressorParameters {
    pub input_gains: [i32; 2],
    pub enables: [bool; 2],
    pub output_volumes: [i32; 2],
}

/// The protocol implementation to operate compressor parameters.
///
/// parameters    | ch 0  | ch 1  | minimum    | maximum    | min val | max val
/// ------------- | ----- | ----- | ---------- | ---------- | ------- | -------
/// attack        | 0xc00 | 0xc28 | 0x81539001 | 0x8006d381 | 2 ms    | 100 ms
/// threshold     | 0xc04 | 0xc2c | 0x9c000001 | 0x00000000 | -50     | 0
/// release       | 0xc08 | 0xc30 | 0x7ff92c7f | 0x7fffc57f | 0.1 s   | 3.0 s
/// ratio         | 0xc0c | 0xc34 | 0x0666665f | 0x7f5c28ff | 1.1:1   | 100 ms
/// gain          | 0xc10 | 0xc38 | 0x0fffffff | 0x7f17aeff | 0       | +18
/// enable        | 0xc14 | 0xc3c | 0x00000000 | 0x7fffffff | disable | enable
/// input gain    | 0xc18 | 0xc40 | 0x0203a7e7 | 0x7f17aeff | -18     | +18
/// output volume | 0xc1c | 0xc44 | 0x0203a7e7 | 0x7f17aeff | -18     | +18
#[derive(Default, Debug)]
pub struct SaffireCompressorProtocol;

impl SaffireParametersSerdes<SaffireCompressorParameters> for SaffireCompressorProtocol {
    const OFFSETS: &'static [usize] = &[
        // ch 0.
        0x0c00, 0x0c04, 0x0c08, 0x0c0c, 0x0c10, 0x0c14, 0x0c18, 0x0c1c, // ch 1.
        0x0c28, 0x0c2c, 0x0c30, 0x0c34, 0x0c38, 0x0c3c, 0x0c40, 0x0c48,
    ];

    fn serialize(params: &SaffireCompressorParameters, raw: &mut [u8]) {
        raw[20..24].copy_from_slice(&(params.enables[0] as u32).to_be_bytes());
        raw[24..28].copy_from_slice(&params.input_gains[0].to_be_bytes());
        raw[28..32].copy_from_slice(&params.output_volumes[0].to_be_bytes());
        raw[52..56].copy_from_slice(&(params.enables[1] as u32).to_be_bytes());
        raw[56..60].copy_from_slice(&params.input_gains[1].to_be_bytes());
        raw[60..64].copy_from_slice(&params.output_volumes[1].to_be_bytes());
    }

    fn deserialize(params: &mut SaffireCompressorParameters, raw: &[u8]) {
        let mut quadlet = [0u8; 4];

        let quads: Vec<i32> = (0..raw.len())
            .step_by(4)
            .map(|pos| {
                quadlet.copy_from_slice(&raw[pos..(pos + 4)]);
                i32::from_be_bytes(quadlet)
            })
            .collect();

        params.enables[0] = quads[5] > 0;
        params.enables[1] = quads[13] > 0;
        params.input_gains[0] = quads[6];
        params.input_gains[1] = quads[14];
        params.output_volumes[0] = quads[7];
        params.output_volumes[1] = quads[15];
    }
}

impl SaffireCompressorProtocol {
    pub const GAIN_MIN: i32 = 0x0fffffff;
    pub const GAIN_MAX: i32 = 0x7fffffff;
    pub const GAIN_STEP: i32 = 1;

    pub const VOLUME_MIN: i32 = 0x0fffffff;
    pub const VOLUME_MAX: i32 = 0x7fffffff;
    pub const VOLUME_STEP: i32 = 1;
}

/// Parameters of equalizer effect in Saffire.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct SaffireEqualizerParameters {
    pub enables: [bool; 2],
    pub input_gains: [i32; 2],
    pub output_volumes: [i32; 2],
}

/// The protocol implementation to operate equalizer parameters.
///
/// parameters    | ch0    | ch1    | minimum    | maximum    | min val | max val
/// ------------- | ------ | ------ | ---------- | ---------- | ------- | -------
/// enable        | 0x0800 | 0x0804 | 0x00000000 | 0x7fffffff | disable | enable
/// input gain    | 0x0808 | 0x0810 | 0x0203a7e7 | 0x7f17afff | -18     | +18
/// output volume | 0x080c | 0x0814 | 0x0203a7e7 | 0x7f17afff | -18     | +18
/// ------------- | ------ | ------ | ---------- | ---------- | ------- | -------
/// band 0        | 0x0828 | 0x0878 |      -     |      -     |    -    |    -
/// band 0        | 0x082c | 0x087c |      -     |      -     |    -    |    -
/// band 0        | 0x0830 | 0x0880 |      -     |      -     |    -    |    -
/// band 0        | 0x0834 | 0x0884 |      -     |      -     |    -    |    -
/// band 0        | 0x0840 | 0x0890 |      -     |      -     |    -    |    -
/// band 0        | 0x0844 | 0x0894 |      -     |      -     |    -    |    -
/// band 0        | 0x0850 | 0x08a0 |      -     |      -     |    -    |    -
/// ------------- | ------ |------- | ---------- | ---------- | ------- | -------
/// band 1        | 0x08c8 | 0x0918 |      -     |      -     |    -    |    -
/// band 1        | 0x08cc | 0x091c |      -     |      -     |    -    |    -
/// band 1        | 0x08d0 | 0x0920 |      -     |      -     |    -    |    -
/// band 1        | 0x08d4 | 0x0924 |      -     |      -     |    -    |    -
/// band 1        | 0x08e0 | 0x0930 |      -     |      -     |    -    |    -
/// band 1        | 0x08e4 | 0x0934 |      -     |      -     |    -    |    -
/// band 1        | 0x08f0 | 0x0940 |      -     |      -     |    -    |    -
/// ------------- | ------ |------- | ---------- | ---------- | ------- | -------
/// band 2        | 0x0968 | 0x09b8 |      -     |      -     |    -    |    -
/// band 2        | 0x096c | 0x09bc |      -     |      -     |    -    |    -
/// band 2        | 0x0970 | 0x09c0 |      -     |      -     |    -    |    -
/// band 2        | 0x0974 | 0x09c4 |      -     |      -     |    -    |    -
/// band 2        | 0x0980 | 0x09d0 |      -     |      -     |    -    |    -
/// band 2        | 0x0984 | 0x09d4 |      -     |      -     |    -    |    -
/// band 2        | 0x0990 | 0x09e0 |      -     |      -     |    -    |    -
/// ------------- | ------ |------- | ---------- | ---------- | ------- | -------
/// band 3        | 0x0a08 | 0x0a58 |      -     |      -     |    -    |    -
/// band 3        | 0x0a0c | 0x0a5c |      -     |      -     |    -    |    -
/// band 3        | 0x0a10 | 0x0a60 |      -     |      -     |    -    |    -
/// band 3        | 0x0a14 | 0x0a64 |      -     |      -     |    -    |    -
/// band 3        | 0x0a20 | 0x0a70 |      -     |      -     |    -    |    -
/// band 3        | 0x0a24 | 0x0a74 |      -     |      -     |    -    |    -
/// band 3        | 0x0a30 | 0x0a80 |      -     |      -     |    -    |    -
#[derive(Default, Debug)]
pub struct SaffireEqualizerProtocol;

impl SaffireParametersSerdes<SaffireEqualizerParameters> for SaffireEqualizerProtocol {
    const OFFSETS: &'static [usize] = &[
        0x0800, 0x0804, 0x0808, 0x080c, 0x0810, 0x0814, // ch 0 band 0.
        0x0828, 0x082c, 0x0830, 0x0834, 0x0840, 0x0844, 0x0850, // ch 0 band 1.
        0x08c8, 0x08cc, 0x08d0, 0x08d4, 0x08e0, 0x08e4, 0x08f0, // ch 0 band 2.
        0x0968, 0x096c, 0x0970, 0x0974, 0x0980, 0x0984, 0x0990, // ch 0 band 3.
        0x0a08, 0x0a0c, 0x0a10, 0x0a14, 0x0a20, 0x0a24, 0x0a30, // ch 1 band 0.
        0x0878, 0x087c, 0x0880, 0x0884, 0x0890, 0x0894, 0x08a0, // ch 1 band 1.
        0x0918, 0x091c, 0x0920, 0x0924, 0x0930, 0x0934, 0x0940, // ch 1 band 2.
        0x09b8, 0x09bc, 0x09c0, 0x09c4, 0x09d0, 0x09d4, 0x09e0, // ch 1 band 3.
        0x0a58, 0x0a5c, 0x0a60, 0x0a64, 0x0a70, 0x0a74, 0x0a80,
    ];

    fn serialize(params: &SaffireEqualizerParameters, raw: &mut [u8]) {
        raw[20..24].copy_from_slice(&(params.enables[0] as u32).to_be_bytes());
        raw[24..28].copy_from_slice(&params.input_gains[0].to_be_bytes());
        raw[28..32].copy_from_slice(&params.output_volumes[0].to_be_bytes());
        raw[52..56].copy_from_slice(&(params.enables[1] as u32).to_be_bytes());
        raw[56..60].copy_from_slice(&params.input_gains[1].to_be_bytes());
        raw[60..64].copy_from_slice(&params.output_volumes[1].to_be_bytes());
    }

    fn deserialize(params: &mut SaffireEqualizerParameters, raw: &[u8]) {
        let mut quadlet = [0u8; 4];

        let quads: Vec<i32> = (0..raw.len())
            .step_by(4)
            .map(|pos| {
                quadlet.copy_from_slice(&raw[pos..(pos + 4)]);
                i32::from_be_bytes(quadlet)
            })
            .collect();

        params.enables[0] = quads[5] > 0;
        params.enables[1] = quads[13] > 0;
        params.input_gains[0] = quads[6];
        params.input_gains[1] = quads[14];
        params.output_volumes[0] = quads[7];
        params.output_volumes[1] = quads[15];
    }
}

/// Parameters of amplifier effect in Saffire.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct SaffireAmplifierParameters {
    pub enables: [bool; 2],
    pub output_volumes: [i32; 2],
}

/// parameter     | ch0           | ch1           | minimum    | maximum    | min val | max val
/// ------------- | ------------- | ------------- | ---------- | ---------- | ------- | -------
/// enable        | 0x1400        | 0x17ec        | 0x00000000 | 0x7fffffff | disable | enable
/// coefficients  | 0x1454-0x17e4 | 0x1840-0x1bd0 |      -     |      -     |    -    |    -
/// output volume | 0x17e8        | 0x1bd4        | 0x080e9f9f | 0x3fffffff | -18     | +18
#[derive(Default, Debug)]
pub struct SaffireAmplifierProtocol;

impl SaffireParametersSerdes<SaffireAmplifierParameters> for SaffireAmplifierProtocol {
    const OFFSETS: &'static [usize] = &[0x1400, 0x17e8, 0x17ec, 0x1bd4];

    fn serialize(params: &SaffireAmplifierParameters, raw: &mut [u8]) {
        raw[..4].copy_from_slice(&(params.enables[0] as u32).to_be_bytes());
        raw[4..8].copy_from_slice(&(params.enables[1] as u32).to_be_bytes());
        raw[8..12].copy_from_slice(&params.output_volumes[0].to_be_bytes());
        raw[12..16].copy_from_slice(&params.output_volumes[1].to_be_bytes());
    }

    fn deserialize(params: &mut SaffireAmplifierParameters, raw: &[u8]) {
        let mut quadlet = [0u8; 4];

        let quads: Vec<i32> = (0..raw.len())
            .step_by(4)
            .map(|pos| {
                quadlet.copy_from_slice(&raw[pos..(pos + 4)]);
                i32::from_be_bytes(quadlet)
            })
            .collect();

        params.enables[0] = quads[0] > 0;
        params.enables[1] = quads[1] > 0;
        params.output_volumes[0] = quads[2];
        params.output_volumes[1] = quads[3];
    }
}

/// Order of compressor effect against equalizer/amplifier effect.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SaffireChStripCompOrder {
    Pre,
    Post,
}

impl Default for SaffireChStripCompOrder {
    fn default() -> Self {
        Self::Pre
    }
}

/// General parameters for channel strip effects.
#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
pub struct SaffireChStripParameters {
    pub paired_mode: SaffireMixerMode,
    pub comp_orders: [SaffireChStripCompOrder; 2],
}

impl SaffireParametersSerdes<SaffireChStripParameters> for SaffireChStripProtocol {
    const OFFSETS: &'static [usize] = &[0x7d0, 0x7d4, 0x7d8];

    fn serialize(params: &SaffireChStripParameters, raw: &mut [u8]) {
        raw[..4].copy_from_slice(
            &(match params.paired_mode {
                SaffireMixerMode::StereoSeparated => 1u32,
                SaffireMixerMode::StereoPaired => 0,
            })
            .to_be_bytes(),
        );

        params
            .comp_orders
            .iter()
            .enumerate()
            .for_each(|(i, comp_order)| {
                let pos = 4 + i * 4;
                raw[pos..(pos + 4)].copy_from_slice(
                    &(match comp_order {
                        SaffireChStripCompOrder::Pre => 1u32,
                        SaffireChStripCompOrder::Post => 0,
                    })
                    .to_be_bytes(),
                );
            });
    }

    fn deserialize(params: &mut SaffireChStripParameters, raw: &[u8]) {
        let mut quadlet = [0u8; 4];

        let quads: Vec<u32> = (0..raw.len())
            .step_by(4)
            .map(|pos| {
                quadlet.copy_from_slice(&raw[pos..(pos + 4)]);
                u32::from_be_bytes(quadlet)
            })
            .collect();

        params.paired_mode = if quads[0] > 0 {
            SaffireMixerMode::StereoSeparated
        } else {
            SaffireMixerMode::StereoPaired
        };
        params
            .comp_orders
            .iter_mut()
            .enumerate()
            .for_each(|(i, comp_order)| {
                *comp_order = if quads[1 + i] > 0 {
                    SaffireChStripCompOrder::Pre
                } else {
                    SaffireChStripCompOrder::Post
                };
            });
    }
}

/// The protocol implementation to operate channel strip effects.
#[derive(Default, Debug)]
pub struct SaffireChStripProtocol;

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn saffire_output_protocol_serdes() {
        let mut params = SaffireOutputProtocol::create_output_parameters();

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

        let mut raw = vec![0u8; SaffireOutputProtocol::OFFSETS.len() * 4];
        SaffireOutputProtocol::serialize(&params, &mut raw);
        let mut p = SaffireOutputProtocol::create_output_parameters();
        SaffireOutputProtocol::deserialize(&mut p, &raw);

        assert_eq!(params, p);
    }

    #[test]
    fn saffire_le_output_protocol_serdes() {
        let mut params = SaffireLeOutputProtocol::create_output_parameters();

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

        let mut raw = vec![0u8; SaffireLeOutputProtocol::OFFSETS.len() * 4];
        SaffireLeOutputProtocol::serialize(&params, &mut raw);
        let mut p = SaffireLeOutputProtocol::create_output_parameters();
        SaffireLeOutputProtocol::deserialize(&mut p, &raw);

        assert_eq!(params, p);
    }

    #[test]
    fn saffire_le_through_protocol_serdes() {
        let params = SaffireThroughParameters {
            midi: true,
            ac3: true,
        };
        let mut raw = vec![0u8; SaffireLeThroughProtocol::OFFSETS.len() * 4];
        SaffireLeThroughProtocol::serialize(&params, &mut raw);
        let mut p = SaffireThroughParameters::default();
        SaffireLeThroughProtocol::deserialize(&mut p, &raw);
        assert_eq!(params, p);
    }

    #[test]
    fn saffire_specific_protocols_serdes() {
        let params = SaffireSpecificParameters {
            mode_192khz: true,
            input_pair_1_src: SaffireInputPair1Source::DigitalInputPair0,
            mixer_mode: SaffireMixerMode::StereoSeparated,
        };
        let mut raw = vec![0u8; SaffireSpecificProtocol::OFFSETS.len() * 4];
        SaffireSpecificProtocol::serialize(&params, &mut raw);
        let mut p = SaffireSpecificParameters::default();
        SaffireSpecificProtocol::deserialize(&mut p, &raw);
    }

    #[test]
    fn saffire_separated_mixer_phys_src_pos() {
        [
            (0, 0, 1),
            (0, 1, 2),
            (0, 2, 3),
            (0, 3, 4),
            (0, 4, 0),
            (1, 0, 16),
            (1, 1, 17),
            (1, 2, 18),
            (1, 3, 19),
            (1, 4, 15),
            (2, 0, 6),
            (2, 1, 7),
            (2, 2, 8),
            (2, 3, 9),
            (2, 4, 5),
            (3, 0, 21),
            (3, 1, 22),
            (3, 2, 23),
            (3, 3, 24),
            (3, 4, 20),
        ]
        .iter()
        .for_each(|&(src_idx, dst_idx, expected)| {
            let pos = SaffireSeparatedMixerProtocol::phys_src_pos(dst_idx, src_idx);
            assert_eq!(pos, expected);
        });
    }

    #[test]
    fn saffire_separated_mixer_reverb_return_pos() {
        [
            (0, 0, 11),
            (0, 1, 12),
            (0, 2, 13),
            (0, 3, 14),
            (0, 4, 10),
            (1, 0, 26),
            (1, 1, 27),
            (1, 2, 28),
            (1, 3, 29),
            (1, 4, 25),
        ]
        .iter()
        .for_each(|&(src_idx, dst_idx, expected)| {
            let pos = SaffireSeparatedMixerProtocol::reverb_return_pos(dst_idx, src_idx);
            assert_eq!(pos, expected);
        });
    }

    #[test]
    fn saffire_separated_mixer_stream_src_pos() {
        [
            (0, 0, 36),
            (0, 1, 37),
            (0, 2, 38),
            (0, 3, 39),
            (0, 4, 35),
            (1, 0, 41),
            (1, 1, 42),
            (1, 2, 43),
            (1, 3, 44),
            (1, 4, 40),
            (2, 0, 46),
            (2, 1, 47),
            (2, 2, 48),
            (2, 3, 49),
            (2, 4, 45),
            (3, 0, 51),
            (3, 1, 52),
            (3, 2, 53),
            (3, 3, 54),
            (3, 4, 50),
            (4, 0, 31),
            (4, 1, 32),
            (4, 2, 33),
            (4, 3, 34),
            (4, 4, 30),
        ]
        .iter()
        .for_each(|&(src_idx, dst_idx, expected)| {
            let pos = SaffireSeparatedMixerProtocol::stream_src_pos(dst_idx, src_idx);
            assert_eq!(pos, expected);
        });
    }

    #[test]
    fn saffire_separated_mixer_protocol_serdes() {
        let mut params = SaffireSeparatedMixerProtocol::create_mixer_state();
        params
            .phys_inputs
            .iter_mut()
            .enumerate()
            .for_each(|(i, gains)| {
                gains
                    .iter_mut()
                    .enumerate()
                    .for_each(|(j, gain)| *gain = (i * 100 + j) as i16)
            });
        params
            .reverb_returns
            .iter_mut()
            .enumerate()
            .for_each(|(i, gains)| {
                gains
                    .iter_mut()
                    .enumerate()
                    .for_each(|(j, gain)| *gain = (i * 100 + j) as i16)
            });
        params
            .stream_inputs
            .iter_mut()
            .enumerate()
            .for_each(|(i, gains)| {
                gains
                    .iter_mut()
                    .enumerate()
                    .for_each(|(j, gain)| *gain = (i * 100 + j) as i16)
            });
        let mut raw = vec![0u8; SaffireSeparatedMixerProtocol::OFFSETS.len() * 4];
        SaffireSeparatedMixerProtocol::serialize(&params, &mut raw);
        let mut p = SaffireSeparatedMixerProtocol::create_mixer_state();
        SaffireSeparatedMixerProtocol::deserialize(&mut p, &raw);

        assert_eq!(params, p);
    }

    #[test]
    fn saffire_paired_mixer_stream_src_pos() {
        [
            (0, 0, 6),
            (0, 1, 7),
            (0, 2, 8),
            (0, 3, 9),
            (0, 4, 5),
            (1, 0, 11),
            (1, 1, 12),
            (1, 2, 13),
            (1, 3, 14),
            (1, 4, 10),
            (2, 0, 16),
            (2, 1, 17),
            (2, 2, 18),
            (2, 3, 19),
            (2, 4, 15),
            (3, 0, 21),
            (3, 1, 22),
            (3, 2, 23),
            (3, 3, 24),
            (3, 4, 20),
            (4, 0, 1),
            (4, 1, 2),
            (4, 2, 3),
            (4, 3, 4),
            (4, 4, 0),
        ]
        .iter()
        .for_each(|&(src_idx, dst_idx, expected)| {
            let pos = SaffirePairedMixerProtocol::stream_src_pos(dst_idx, src_idx);
            assert_eq!(pos, expected);
        });
    }

    #[test]
    fn saffire_paired_mixer_phys_src_pos() {
        [
            (0, 0, 26),
            (0, 1, 27),
            (0, 2, 28),
            (0, 3, 29),
            (0, 4, 25),
            (1, 0, 31),
            (1, 1, 32),
            (1, 2, 33),
            (1, 3, 34),
            (1, 4, 30),
        ]
        .iter()
        .for_each(|&(src_idx, dst_idx, expected)| {
            let pos = SaffirePairedMixerProtocol::phys_src_pos(dst_idx, src_idx);
            assert_eq!(pos, expected);
        });
    }

    #[test]
    fn saffire_paired_mixer_reverb_return_pos() {
        [(0, 36), (1, 37), (2, 38), (3, 39), (4, 35)]
            .iter()
            .for_each(|&(dst_idx, expected)| {
                let pos = SaffirePairedMixerProtocol::reverb_return_pos(dst_idx, 0);
                assert_eq!(pos, expected);
            });
    }

    #[test]
    fn saffire_paired_mixer_protocol_serdes() {
        let mut params = SaffirePairedMixerProtocol::create_mixer_state();
        params
            .phys_inputs
            .iter_mut()
            .enumerate()
            .for_each(|(i, gains)| {
                gains
                    .iter_mut()
                    .enumerate()
                    .for_each(|(j, gain)| *gain = (i * 100 + j) as i16)
            });
        params
            .reverb_returns
            .iter_mut()
            .enumerate()
            .for_each(|(i, gains)| {
                gains
                    .iter_mut()
                    .enumerate()
                    .for_each(|(j, gain)| *gain = (i * 100 + j) as i16)
            });
        params
            .stream_inputs
            .iter_mut()
            .enumerate()
            .for_each(|(i, gains)| {
                gains
                    .iter_mut()
                    .enumerate()
                    .for_each(|(j, gain)| *gain = (i * 100 + j) as i16)
            });
        let mut raw = vec![0u8; SaffirePairedMixerProtocol::OFFSETS.len() * 4];
        SaffirePairedMixerProtocol::serialize(&params, &mut raw);
        let mut p = SaffirePairedMixerProtocol::create_mixer_state();
        SaffirePairedMixerProtocol::deserialize(&mut p, &raw);

        assert_eq!(params, p);
    }

    #[test]
    fn saffire_reverb_protocol_serdes() {
        let params = SaffireReverbParameters {
            amounts: [-101, 102],
            room_sizes: [111, -112],
            diffusions: [-113, 113],
            tones: [114, -114],
        };
        let mut raw = vec![0u8; SaffireReverbProtocol::OFFSETS.len() * 4];
        SaffireReverbProtocol::serialize(&params, &mut raw);
        let mut p = SaffireReverbParameters::default();
        SaffireReverbProtocol::deserialize(&mut p, &raw);

        assert_eq!(params, p);
    }

    #[test]
    fn saffire_compressor_protocol_serdes() {
        let params = SaffireCompressorParameters {
            input_gains: [-200, 200],
            enables: [true, false],
            output_volumes: [201, -201],
        };
        let mut raw = vec![0u8; SaffireCompressorProtocol::OFFSETS.len() * 4];
        SaffireCompressorProtocol::serialize(&params, &mut raw);
        let mut p = SaffireCompressorParameters::default();
        SaffireCompressorProtocol::deserialize(&mut p, &raw);

        assert_eq!(params, p);
    }

    #[test]
    fn saffire_equalizer_protocol_serdes() {
        let params = SaffireEqualizerParameters {
            input_gains: [-200, 200],
            enables: [true, false],
            output_volumes: [201, -201],
        };
        let mut raw = vec![0u8; SaffireEqualizerProtocol::OFFSETS.len() * 4];
        SaffireEqualizerProtocol::serialize(&params, &mut raw);
        let mut p = SaffireEqualizerParameters::default();
        SaffireEqualizerProtocol::deserialize(&mut p, &raw);

        assert_eq!(params, p);
    }

    #[test]
    fn saffire_amplifier_protocol_serdes() {
        let params = SaffireAmplifierParameters {
            enables: [false, true],
            output_volumes: [400, -401],
        };
        let mut raw = vec![0u8; SaffireAmplifierProtocol::OFFSETS.len() * 4];
        SaffireAmplifierProtocol::serialize(&params, &mut raw);
        let mut p = SaffireAmplifierParameters::default();
        SaffireAmplifierProtocol::deserialize(&mut p, &raw);

        assert_eq!(params, p);
    }

    #[test]
    fn saffire_ch_strip_protocol_serdes() {
        let params = SaffireChStripParameters {
            paired_mode: SaffireMixerMode::StereoSeparated,
            comp_orders: [SaffireChStripCompOrder::Pre, SaffireChStripCompOrder::Post],
        };
        let mut raw = vec![0u8; SaffireChStripProtocol::OFFSETS.len() * 4];
        SaffireChStripProtocol::serialize(&params, &mut raw);
        let mut p = SaffireChStripParameters::default();
        SaffireChStripProtocol::deserialize(&mut p, &raw);

        assert_eq!(params, p);
    }

    #[test]
    fn saffire_le_specific_protocol_serdes() {
        let params = SaffireLeSpecificParameters {
            analog_input_2_3_high_gains: [true, false],
        };
        let mut raw = vec![0u8; SaffireLeSpecificProtocol::OFFSETS.len() * 4];
        SaffireLeSpecificProtocol::serialize(&params, &mut raw);
        let mut p = SaffireLeSpecificParameters::default();
        SaffireLeSpecificProtocol::deserialize(&mut p, &raw);

        assert_eq!(params, p);
    }

    #[test]
    fn saffire_le_mixer_low_rate_stream_src_pos() {
        // NOTE: src_idx, dst_idx, expected offsets index
        [
            (0, 0, 2),
            (0, 1, 0),
            (0, 2, 3),
            (0, 3, 1),
            (1, 0, 18),
            (1, 1, 16),
            (1, 2, 19),
            (1, 3, 17),
            (2, 0, 6),
            (2, 1, 4),
            (2, 2, 7),
            (2, 3, 5),
            (3, 0, 22),
            (3, 1, 20),
            (3, 2, 23),
            (3, 3, 21),
            (4, 0, 10),
            (4, 1, 8),
            (4, 2, 11),
            (4, 3, 9),
            (5, 0, 26),
            (5, 1, 24),
            (5, 2, 27),
            (5, 3, 25),
            (6, 0, 14),
            (6, 1, 12),
            (6, 2, 15),
            (6, 3, 13),
            (7, 0, 30),
            (7, 1, 28),
            (7, 2, 31),
            (7, 3, 29),
        ]
        .iter()
        .for_each(|&(src_idx, dst_idx, expected)| {
            let idx = SaffireLeMixerLowRateProtocol::stream_src_pos(dst_idx, src_idx);
            assert_eq!(idx, expected, "{}-{}", dst_idx, src_idx);
        })
    }

    #[test]
    fn saffire_le_mixer_low_rate_phys_src_pos() {
        // NOTE: src_idx, dst_idx, expected offsets index
        [
            (0, 0, 34),
            (0, 1, 32),
            (0, 2, 35),
            (0, 3, 33),
            (1, 0, 46),
            (1, 1, 44),
            (1, 2, 47),
            (1, 3, 45),
            (2, 0, 38),
            (2, 1, 36),
            (2, 2, 39),
            (2, 3, 37),
            (3, 0, 50),
            (3, 1, 48),
            (3, 2, 51),
            (3, 3, 49),
            (4, 0, 42),
            (4, 1, 40),
            (4, 2, 43),
            (4, 3, 41),
            (5, 0, 54),
            (5, 1, 52),
            (5, 2, 55),
            (5, 3, 53),
        ]
        .iter()
        .for_each(|&(src_idx, dst_idx, expected)| {
            let idx = SaffireLeMixerLowRateProtocol::phys_src_pos(dst_idx, src_idx);
            assert_eq!(idx, expected, "{}-{}", dst_idx, src_idx);
        })
    }

    #[test]
    fn saffire_le_mixer_middle_rate_monitor_pair_from_analog_input_pos() {
        // NOTE: src_idx, expected offset idx.
        [(0, 0), (1, 3), (2, 1), (3, 4), (4, 2), (5, 5)]
            .iter()
            .for_each(|&(src_idx, expected)| {
                let idx = SaffireLeMixerMiddleRateProtocol::monitor_analog_input_pos(0, src_idx);
                assert_eq!(idx, expected);
            });
    }

    #[test]
    fn saffire_le_mixer_middle_rate_monitor_src_pos() {
        // NOTE: dst_idx, expected.
        [(0, 6), (1, 9), (2, 12), (3, 15)]
            .iter()
            .for_each(|&(dst_idx, expected)| {
                let idx = SaffireLeMixerMiddleRateProtocol::mixer_monitor_src_pos(dst_idx, 0);
                assert_eq!(idx, expected);
            });
    }

    #[test]
    fn saffire_le_mixer_middle_rate_mixer_stream_input_pos() {
        // NOTE: dst_idx, src_idx, expected.
        [
            (0, 0, 7),
            (0, 1, 8),
            (1, 0, 10),
            (1, 1, 11),
            (2, 0, 13),
            (2, 1, 14),
            (3, 0, 16),
            (3, 1, 17),
        ]
        .iter()
        .for_each(|&(dst_idx, src_idx, expected)| {
            let idx = SaffireLeMixerMiddleRateProtocol::mixer_stream_input_pos(dst_idx, src_idx);
            assert_eq!(idx, expected);
        });
    }

    #[test]
    fn saffire_le_mixer_low_rate_protocol_serdes() {
        let mut params = SaffireLeMixerLowRateState::default();

        params
            .phys_src_gains
            .iter_mut()
            .enumerate()
            .for_each(|(i, gains)| {
                gains
                    .iter_mut()
                    .enumerate()
                    .for_each(|(j, gain)| *gain = (i * 100 + j) as i16);
            });

        params
            .stream_src_gains
            .iter_mut()
            .enumerate()
            .for_each(|(i, gains)| {
                gains
                    .iter_mut()
                    .enumerate()
                    .for_each(|(j, gain)| *gain = (i * 100 + j) as i16);
            });

        params.spdif_out_src = SaffireLeSpdifOutputSource::MixerOutputPair67;

        let mut raw = vec![0u8; SaffireLeMixerLowRateProtocol::OFFSETS.len() * 4];
        SaffireLeMixerLowRateProtocol::serialize(&params, &mut raw);
        let mut p = SaffireLeMixerLowRateState::default();
        SaffireLeMixerLowRateProtocol::deserialize(&mut p, &raw);

        assert_eq!(params, p);
    }

    #[test]
    fn saffire_le_mixer_middle_rate_protocol_serdes() {
        let mut params = SaffireLeMixerMiddleRateState::default();

        params
            .monitor_src_phys_input_gains
            .iter_mut()
            .enumerate()
            .for_each(|(i, gain)| *gain = i as i16);

        params
            .monitor_out_src_pair_gains
            .iter_mut()
            .enumerate()
            .for_each(|(i, gains)| {
                gains
                    .iter_mut()
                    .enumerate()
                    .for_each(|(j, gain)| *gain = (i * 100 + j) as i16);
            });

        params
            .stream_src_pair_gains
            .iter_mut()
            .enumerate()
            .for_each(|(i, gains)| {
                gains
                    .iter_mut()
                    .enumerate()
                    .for_each(|(j, gain)| *gain = (i * 100 + j) as i16);
            });

        params.spdif_out_src = SaffireLeSpdifOutputSource::MixerOutputPair67;

        let mut raw = vec![0u8; SaffireLeMixerMiddleRateProtocol::OFFSETS.len() * 4];
        SaffireLeMixerMiddleRateProtocol::serialize(&params, &mut raw);
        let mut p = SaffireLeMixerMiddleRateState::default();
        SaffireLeMixerMiddleRateProtocol::deserialize(&mut p, &raw);

        assert_eq!(params, p);
    }
}
