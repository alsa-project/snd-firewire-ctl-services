// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol implementation for Focusrite Saffire and Saffire LE.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by Focusrite Audio Engineering for Saffire and Saffire LE.
//!
//! DM1000E ASIC is used for Saffire, while DM1000 ASIC is used for Saffire LE.

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
}

/// The protocol implementation of metering in Saffire.
#[derive(Default)]
pub struct SaffireMeterProtocol;

impl SaffireMeterProtocol {
    /// The number of destionation pairs.
    pub const LEVEL_MIN: i32 = 0;
    pub const LEVEL_MAX: i32 = 0x7fffffff;
    pub const LEVEL_STEP: i32 = 1;

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
