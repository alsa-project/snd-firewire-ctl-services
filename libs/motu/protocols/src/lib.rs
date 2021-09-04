// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocols defined for Mark of the Unicorn FireWire series.
//!
//! The crate includes protocols defined by Mark of the Unicorn for its FireWire series.

pub mod config_rom;
pub mod version_1;
pub mod version_2;
pub mod version_3;

use glib::{Error, FileError};
use hinawa::{FwNode, FwReq, FwReqExtManual, FwTcode};
use hinawa::{SndMotu, SndUnitExt};

use std::{thread, time};

const BASE_OFFSET: u64 = 0xfffff0000000;

const BUSY_DURATION: u64 = 150;

const DISPLAY_CHARS: usize = 4 * 4;

fn read_quad(
    req: &FwReq,
    node: &mut FwNode,
    offset: u32,
    timeout_ms: u32
) -> Result<u32, Error> {
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

fn get_idx_from_val(
    offset: u32,
    mask: u32,
    shift: usize,
    label: &str,
    req: &FwReq,
    node: &mut FwNode,
    vals: &[u8],
    timeout_ms: u32,
) -> Result<usize, Error> {
    let quad = read_quad(req, node, offset, timeout_ms)?;
    let val = ((quad & mask) >> shift) as u8;
    vals.iter().position(|&v| v == val).ok_or_else(|| {
        let label = format!("Detect invalid value for {}: {:02x}", label, val);
        Error::new(FileError::Io, &label)
    })
}

fn set_idx_to_val(
    offset: u32,
    mask: u32,
    shift: usize,
    label: &str,
    req: &FwReq,
    node: &mut FwNode,
    vals: &[u8],
    idx: usize,
    timeout_ms: u32,
) -> Result<(), Error> {
    if idx >= vals.len() {
        let label = format!("Invalid argument for {}: {} {}", label, vals.len(), idx);
        return Err(Error::new(FileError::Inval, &label));
    }
    let mut quad = read_quad(req, node, offset, timeout_ms)?;
    quad &= !mask;
    quad |= (vals[idx] as u32) << shift;
    write_quad(req, node, offset, quad, timeout_ms)
}

/// The enumeration to express rate of sampling clock.
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

/// The trait for common protocol.
pub trait CommonProtocol: AsRef<FwReq> {
    const OFFSET_CLK: u32 = 0x0b14;
    const OFFSET_PORT: u32 = 0x0c04;
    const OFFSET_CLK_DISPLAY: u32 = 0x0c60;

    fn update_clk_display(
        &self,
        unit: &SndMotu,
        label: &str,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut chars = [0; DISPLAY_CHARS];
        chars
            .iter_mut()
            .zip(label.bytes())
            .for_each(|(c, l)| *c = l);

        (0..(DISPLAY_CHARS / 4)).try_for_each(|i| {
            let mut frame = [0; 4];
            frame.copy_from_slice(&chars[(i * 4)..(i * 4 + 4)]);
            frame.reverse();
            let quad = u32::from_ne_bytes(frame);
            let offset = Self::OFFSET_CLK_DISPLAY + 4 * i as u32;
            write_quad(self.as_ref(), &mut unit.get_node(), offset, quad, timeout_ms)
        })
    }
}

const PORT_PHONE_LABEL: &str = "phone-assign";
const PORT_PHONE_MASK: u32 = 0x0000000f;
const PORT_PHONE_SHIFT: usize = 0;

/// The trait for headphone assignment protocol.
pub trait AssignProtocol: CommonProtocol {
    const ASSIGN_PORTS: &'static [(&'static str, u8)];

    fn get_phone_assign(&self, unit: &SndMotu, timeout_ms: u32) -> Result<usize, Error> {
        let vals: Vec<u8> = Self::ASSIGN_PORTS.iter().map(|e| e.1).collect();
        get_idx_from_val(
            Self::OFFSET_PORT,
            PORT_PHONE_MASK,
            PORT_PHONE_SHIFT,
            PORT_PHONE_LABEL,
            self.as_ref(),
            &mut unit.get_node(),
            &vals,
            timeout_ms,
        )
    }

    fn set_phone_assign(&self, unit: &SndMotu, idx: usize, timeout_ms: u32) -> Result<(), Error> {
        let vals: Vec<u8> = Self::ASSIGN_PORTS.iter().map(|e| e.1).collect();
        set_idx_to_val(
            Self::OFFSET_PORT,
            PORT_PHONE_MASK,
            PORT_PHONE_SHIFT,
            PORT_PHONE_LABEL,
            self.as_ref(),
            &mut unit.get_node(),
            &vals,
            idx,
            timeout_ms,
        )
    }
}

/// The enumeration to express mode of speed for output signal of word clock on BNC interface.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum WordClkSpeedMode {
    /// The speed is forced to be 44.1/48.0 kHz.
    ForceLowRate,
    /// The speed is following to system clock.
    FollowSystemClk,
}

const WORD_OUT_LABEL: &str = "word-out";
const WORD_OUT_MASK: u32 = 0x08000000;
const WORD_OUT_SHIFT: usize = 27;

const WORD_OUT_VALS: [u8; 2] = [0x00, 0x01];

/// The trait for word-clock protocol.
pub trait WordClkProtocol: CommonProtocol {
    fn get_word_out(&self, unit: &SndMotu, timeout_ms: u32) -> Result<WordClkSpeedMode, Error> {
        get_idx_from_val(
            Self::OFFSET_CLK,
            WORD_OUT_MASK,
            WORD_OUT_SHIFT,
            WORD_OUT_LABEL,
            self.as_ref(),
            &mut unit.get_node(),
            &WORD_OUT_VALS,
            timeout_ms,
        )
        .map(|val| {
            if val == 0 {
                WordClkSpeedMode::ForceLowRate
            } else {
                WordClkSpeedMode::FollowSystemClk
            }
        })
    }

    fn set_word_out(
        &self,
        unit: &SndMotu,
        mode: WordClkSpeedMode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let idx = match mode {
            WordClkSpeedMode::ForceLowRate => 0,
            WordClkSpeedMode::FollowSystemClk => 1,
        };
        set_idx_to_val(
            Self::OFFSET_CLK,
            WORD_OUT_MASK,
            WORD_OUT_SHIFT,
            WORD_OUT_LABEL,
            self.as_ref(),
            &mut unit.get_node(),
            &WORD_OUT_VALS,
            idx,
            timeout_ms,
        )
    }
}

/// The enumeration to express the mode of rate convert for AES/EBU input/output signals.
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

const AESEBU_RATE_CONVERT_LABEL: &str = "aesebu-rate-convert";

/// The trait for protocol of rate convert specific to AES/EBU input/output signals.
pub trait AesebuRateConvertProtocol: CommonProtocol {
    const AESEBU_RATE_CONVERT_MASK: u32;
    const AESEBU_RATE_CONVERT_SHIFT: usize;

    const AESEBU_RATE_CONVERT_VALS: [u8; 4] = [0x00, 0x01, 0x02, 0x03];

    const AESEBU_RATE_CONVERT_MODES: [AesebuRateConvertMode; 4] = [
        AesebuRateConvertMode::None,
        AesebuRateConvertMode::InputToSystem,
        AesebuRateConvertMode::OutputDependsInput,
        AesebuRateConvertMode::OutputDoubleSystem,
    ];

    fn get_aesebu_rate_convert_mode(
        &self,
        unit: &SndMotu,
        timeout_ms: u32,
    ) -> Result<usize, Error> {
        get_idx_from_val(
            Self::OFFSET_CLK,
            Self::AESEBU_RATE_CONVERT_MASK,
            Self::AESEBU_RATE_CONVERT_SHIFT,
            AESEBU_RATE_CONVERT_LABEL,
            self.as_ref(),
            &mut unit.get_node(),
            &Self::AESEBU_RATE_CONVERT_VALS,
            timeout_ms,
        )
    }

    fn set_aesebu_rate_convert_mode(
        &self,
        unit: &SndMotu,
        idx: usize,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        set_idx_to_val(
            Self::OFFSET_CLK,
            Self::AESEBU_RATE_CONVERT_MASK,
            Self::AESEBU_RATE_CONVERT_SHIFT,
            AESEBU_RATE_CONVERT_LABEL,
            self.as_ref(),
            &mut unit.get_node(),
            &Self::AESEBU_RATE_CONVERT_VALS,
            idx,
            timeout_ms,
        )
    }
}

/// The enumeration to express the mode of hold time for clip and peak LEDs.
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

/// The enumeration to express the mode of programmable meter display.
pub enum LevelMetersProgrammableMode {
    AnalogOutput,
    AdatInput,
    AdatOutput,
}

/// The enumeration to express the mode of AES/EBU meter display.
pub enum LevelMetersAesebuMode {
    Input,
    Output,
}

const LEVEL_METERS_OFFSET: u32 = 0x0b24;

const LEVEL_METERS_PEAK_HOLD_TIME_MASK: u32 = 0x00003800;
const LEVEL_METERS_PEAK_HOLD_TIME_SHIFT: usize = 11;

const LEVEL_METERS_CLIP_HOLD_TIME_MASK: u32 = 0x00000700;
const LEVEL_METERS_CLIP_HOLD_TIME_SHIFT: usize = 8;

const LEVEL_METERS_HOLD_TIME_VALS: [u8; 8] = [0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07];

const LEVEL_METERS_AESEBU_MASK: u32 = 0x00000004;
const LEVEL_METERS_AESEBU_SHIFT: usize = 2;

const LEVEL_METERS_AESEBU_VALS: [u8; 2] = [0x00, 0x01];

const LEVEL_METERS_PROGRAMMABLE_MASK: u32 = 0x00000003;
const LEVEL_METERS_PROGRAMMABLE_SHIFT: usize = 0;
const LEVEL_METERS_PROGRAMMABLE_VALS: [u8; 3] = [0x00, 0x01, 0x02];

const LEVEL_METERS_PEAK_HOLD_TIME_LABEL: &str = "level-meters-peak-hold-time";
const LEVEL_METERS_CLIP_HOLD_TIME_LABEL: &str = "level-meters-clip-hold-time";
const LEVEL_METERS_PROGRAMMABLE_LABEL: &str = "level-meters-programmable";
const LEVEL_METERS_AESEBU_LABEL: &str = "level-meters-aesebu";

/// The trait for protocol of level meter.
pub trait LevelMetersProtocol: CommonProtocol {
    const LEVEL_METERS_HOLD_TIME_MODES: [LevelMetersHoldTimeMode; 8] = [
        LevelMetersHoldTimeMode::Off,
        LevelMetersHoldTimeMode::Sec2,
        LevelMetersHoldTimeMode::Sec4,
        LevelMetersHoldTimeMode::Sec10,
        LevelMetersHoldTimeMode::Sec60,
        LevelMetersHoldTimeMode::Sec300,
        LevelMetersHoldTimeMode::Sec480,
        LevelMetersHoldTimeMode::Infinite,
    ];

    const LEVEL_METERS_AESEBU_MODES: [LevelMetersAesebuMode; 2] =
        [LevelMetersAesebuMode::Output, LevelMetersAesebuMode::Input];

    const LEVEL_METERS_PROGRAMMABLE_MODES: [LevelMetersProgrammableMode; 3] = [
        LevelMetersProgrammableMode::AnalogOutput,
        LevelMetersProgrammableMode::AdatInput,
        LevelMetersProgrammableMode::AdatOutput,
    ];

    fn get_level_meters_peak_hold_time_mode(
        &self,
        unit: &SndMotu,
        timeout_ms: u32,
    ) -> Result<usize, Error> {
        get_idx_from_val(
            LEVEL_METERS_OFFSET,
            LEVEL_METERS_PEAK_HOLD_TIME_MASK,
            LEVEL_METERS_PEAK_HOLD_TIME_SHIFT,
            LEVEL_METERS_PEAK_HOLD_TIME_LABEL,
            self.as_ref(),
            &mut unit.get_node(),
            &LEVEL_METERS_HOLD_TIME_VALS,
            timeout_ms,
        )
    }

    fn set_level_meters_peak_hold_time_mode(
        &self,
        unit: &SndMotu,
        idx: usize,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        set_idx_to_val(
            LEVEL_METERS_OFFSET,
            LEVEL_METERS_PEAK_HOLD_TIME_MASK,
            LEVEL_METERS_PEAK_HOLD_TIME_SHIFT,
            LEVEL_METERS_PEAK_HOLD_TIME_LABEL,
            self.as_ref(),
            &mut unit.get_node(),
            &LEVEL_METERS_HOLD_TIME_VALS,
            idx,
            timeout_ms,
        )
    }

    fn get_level_meters_clip_hold_time_mode(
        &self,
        unit: &SndMotu,
        timeout_ms: u32,
    ) -> Result<usize, Error> {
        get_idx_from_val(
            LEVEL_METERS_OFFSET,
            LEVEL_METERS_CLIP_HOLD_TIME_MASK,
            LEVEL_METERS_CLIP_HOLD_TIME_SHIFT,
            LEVEL_METERS_CLIP_HOLD_TIME_LABEL,
            self.as_ref(),
            &mut unit.get_node(),
            &LEVEL_METERS_HOLD_TIME_VALS,
            timeout_ms,
        )
    }

    fn set_level_meters_clip_hold_time_mode(
        &self,
        unit: &SndMotu,
        idx: usize,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        set_idx_to_val(
            LEVEL_METERS_OFFSET,
            LEVEL_METERS_CLIP_HOLD_TIME_MASK,
            LEVEL_METERS_CLIP_HOLD_TIME_SHIFT,
            LEVEL_METERS_CLIP_HOLD_TIME_LABEL,
            self.as_ref(),
            &mut unit.get_node(),
            &LEVEL_METERS_HOLD_TIME_VALS,
            idx,
            timeout_ms,
        )
    }

    fn get_level_meters_aesebu_mode(
        &self,
        unit: &SndMotu,
        timeout_ms: u32,
    ) -> Result<usize, Error> {
        get_idx_from_val(
            LEVEL_METERS_OFFSET,
            LEVEL_METERS_AESEBU_MASK,
            LEVEL_METERS_AESEBU_SHIFT,
            LEVEL_METERS_AESEBU_LABEL,
            self.as_ref(),
            &mut unit.get_node(),
            &LEVEL_METERS_AESEBU_VALS,
            timeout_ms,
        )
    }

    fn set_level_meters_aesebu_mode(
        &self,
        unit: &SndMotu,
        idx: usize,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        set_idx_to_val(
            LEVEL_METERS_OFFSET,
            LEVEL_METERS_AESEBU_MASK,
            LEVEL_METERS_AESEBU_SHIFT,
            LEVEL_METERS_AESEBU_LABEL,
            self.as_ref(),
            &mut unit.get_node(),
            &LEVEL_METERS_AESEBU_VALS,
            idx,
            timeout_ms,
        )
    }

    fn get_level_meters_programmable_mode(
        &self,
        unit: &SndMotu,
        timeout_ms: u32,
    ) -> Result<usize, Error> {
        get_idx_from_val(
            LEVEL_METERS_OFFSET,
            LEVEL_METERS_PROGRAMMABLE_MASK,
            LEVEL_METERS_PROGRAMMABLE_SHIFT,
            LEVEL_METERS_PROGRAMMABLE_LABEL,
            self.as_ref(),
            &mut unit.get_node(),
            &LEVEL_METERS_PROGRAMMABLE_VALS,
            timeout_ms,
        )
    }

    fn set_level_meters_programmable_mode(
        &self,
        unit: &SndMotu,
        idx: usize,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        set_idx_to_val(
            LEVEL_METERS_OFFSET,
            LEVEL_METERS_PROGRAMMABLE_MASK,
            LEVEL_METERS_PROGRAMMABLE_SHIFT,
            LEVEL_METERS_PROGRAMMABLE_LABEL,
            self.as_ref(),
            &mut unit.get_node(),
            &LEVEL_METERS_PROGRAMMABLE_VALS,
            idx,
            timeout_ms,
        )
    }
}
