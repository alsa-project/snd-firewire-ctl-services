// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocols defined for Tascam for FireWire series with isochronous communication.
//!
//! The module includes protocol implementation defined by Tascam for FireWire series with
//! isochronous communication.

pub mod fw1082;
pub mod fw1804;
pub mod fw1884;

use super::*;

/// Operation to cache whole parameters at once.
pub trait TascamIsochWhollyCachableParamsOperation<T> {
    /// Cache whole parameters.
    fn cache_wholly(
        req: &mut FwReq,
        node: &mut FwNode,
        states: &mut T,
        timeout_ms: u32,
    ) -> Result<(), Error>;
}

/// Operation to update whole parameters at once.
pub trait TascamIsochWhollyUpdatableParamsOperation<T> {
    /// Update whole parameters.
    fn update_wholly(
        req: &mut FwReq,
        node: &mut FwNode,
        states: &T,
        timeout_ms: u32,
    ) -> Result<(), Error>;
}

/// Operation to update the part of parameters.
pub trait TascamIsochPartiallyUpdatableParamsOperation<T> {
    /// Update the part of parameters.
    fn update_partially(
        req: &mut FwReq,
        node: &mut FwNode,
        params: &mut T,
        update: T,
        timeout_ms: u32,
    ) -> Result<(), Error>;
}

/// Signal source of sampling clock.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ClkSrc {
    /// Internal oscillator.
    Internal,
    /// Word clock signal from BNC input interface.
    Wordclock,
    /// S/PDIF signal from coaxial input interface.
    Spdif,
    /// ADAT signal from optical input interface.
    Adat,
}

impl Default for ClkSrc {
    fn default() -> Self {
        Self::Internal
    }
}

/// Nominal frequency of media clock.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ClkRate {
    /// At 44.1 kHz.
    R44100,
    /// At 48.0 kHz.
    R48000,
    /// At 88.2 kHz.
    R88200,
    /// At 96.0 kHz.
    R96000,
}

impl Default for ClkRate {
    fn default() -> Self {
        Self::R44100
    }
}

/// The parameters of sampling and media clock.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct TascamClockParameters {
    /// The source of sampling clock.
    pub sampling_clock_source: ClkSrc,
    /// The rate of media clock.
    pub media_clock_rate: ClkRate,
}

/// The parameters of threshold to detect input signal. The value is between 1 and 0x7fff. The dB
/// level can be calculated by below formula.
///
/// ```text
/// level = 20 * log10(value / 0x7fff)
/// ```
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct TascamInputDetectionThreshold {
    /// For signal detection itself.
    pub signal: u16,
    /// For over-level detection.
    pub over_level: u16,
}

/// Mode of monitor.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum MonitorMode {
    Computer,
    Inputs,
    Both,
}

impl Default for MonitorMode {
    fn default() -> Self {
        Self::Computer
    }
}

/// State of meter.
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct IsochMeterState {
    pub monitor: i16,
    pub solo: Option<i16>,
    pub inputs: Vec<i32>,
    pub outputs: Vec<i32>,
    pub rate: Option<ClkRate>,
    pub src: Option<ClkSrc>,
    pub monitor_meters: [i32; 2],
    pub analog_mixer_meters: [i32; 2],
    pub monitor_mode: MonitorMode,
}

/// The trait for meter operation.
pub trait IsochMeterOperation {
    const INPUT_COUNT: usize;
    const OUTPUT_COUNT: usize;
    const HAS_SOLO: bool;

    const ROTARY_MIN: i16 = 0;
    const ROTARY_MAX: i16 = 1023;
    const ROTARY_STEP: i16 = 2;

    const LEVEL_MIN: i32 = 0;
    const LEVEL_MAX: i32 = 0x7fffff00;
    const LEVEL_STEP: i32 = 0x100;

    fn create_meter_state() -> IsochMeterState {
        IsochMeterState {
            monitor: Default::default(),
            solo: if Self::HAS_SOLO {
                Some(Default::default())
            } else {
                None
            },
            inputs: vec![Default::default(); Self::INPUT_COUNT],
            outputs: vec![Default::default(); Self::OUTPUT_COUNT],
            rate: Default::default(),
            src: Default::default(),
            monitor_meters: Default::default(),
            analog_mixer_meters: Default::default(),
            monitor_mode: Default::default(),
        }
    }

    fn parse_meter_state(state: &mut IsochMeterState, image: &[u32]) -> Result<(), Error> {
        let monitor = (image[5] & 0x0000ffff) as i16;
        if (state.monitor - monitor).abs() > Self::ROTARY_STEP {
            state.monitor = monitor;
        }

        if let Some(solo) = &mut state.solo {
            let val = ((image[4] >> 16) & 0x0000ffff) as i16;
            if (*solo - val).abs() > Self::ROTARY_STEP {
                *solo = val;
            }
        }

        state
            .inputs
            .iter_mut()
            .take(Self::INPUT_COUNT)
            .enumerate()
            .for_each(|(i, input)| {
                let pos = if Self::INPUT_COUNT == 10 && i >= 8 {
                    i + 16
                } else {
                    i
                } + 16;
                *input = image[pos] as i32;
            });

        state
            .outputs
            .iter_mut()
            .take(Self::OUTPUT_COUNT)
            .enumerate()
            .for_each(|(i, output)| {
                let pos = if Self::OUTPUT_COUNT == 4 && i >= 2 {
                    i + 16
                } else {
                    i
                } + 34;
                *output = image[pos] as i32;
            });

        let bits = (image[52] & 0x0000000f) as u8;
        state.src = match bits {
            0x04 => Some(ClkSrc::Adat),
            0x03 => Some(ClkSrc::Spdif),
            0x02 => Some(ClkSrc::Wordclock),
            0x01 => Some(ClkSrc::Internal),
            _ => None,
        };

        let bits = ((image[52] >> 8) & 0x000000ff) as u8;
        state.rate = match bits {
            0x82 => Some(ClkRate::R96000),
            0x81 => Some(ClkRate::R88200),
            0x02 => Some(ClkRate::R48000),
            0x01 => Some(ClkRate::R44100),
            _ => None,
        };

        state
            .monitor_meters
            .iter_mut()
            .enumerate()
            .for_each(|(i, m)| {
                *m = image[i + 54] as i32;
            });

        state
            .analog_mixer_meters
            .iter_mut()
            .enumerate()
            .for_each(|(i, m)| {
                *m = image[i + 57] as i32;
            });

        if image[59] > 0 && image[59] < 4 {
            state.monitor_mode = match image[59] {
                2 => MonitorMode::Both,
                1 => MonitorMode::Inputs,
                _ => MonitorMode::Computer,
            };
        }

        Ok(())
    }
}

fn read_config_flag<T: Copy + Eq>(
    req: &mut FwReq,
    node: &mut FwNode,
    flags: &[(T, u32, u32)],
    timeout_ms: u32,
) -> Result<T, Error> {
    let mut quads = [0; 4];
    read_quadlet(req, node, CONFIG_FLAG_OFFSET, &mut quads, timeout_ms)?;
    let val = u32::from_be_bytes(quads);
    let mask = flags.iter().fold(0, |mask, (_, flag, _)| mask | flag);
    flags
        .iter()
        .find(|&(_, flag, _)| val & mask == *flag)
        .ok_or_else(|| {
            let msg = format!("No flag detected: val: 0x{:08x} mask: 0x{:08x}", val, mask);
            Error::new(FileError::Nxio, &msg)
        })
        .map(|&(option, _, _)| option)
}

fn write_config_flag<T: Copy + Eq>(
    req: &mut FwReq,
    node: &mut FwNode,
    flags: &[(T, u32, u32)],
    option: T,
    timeout_ms: u32,
) -> Result<(), Error> {
    let (_, _, flag) = flags.iter().find(|(o, _, _)| option.eq(o)).unwrap();
    write_quadlet(
        req,
        node,
        CONFIG_FLAG_OFFSET,
        &mut flag.to_be_bytes(),
        timeout_ms,
    )
}

/// Source of output coaxial interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoaxialOutputSource {
    /// A pair in stream inputs.
    StreamInputPair,
    /// Mirror of analog output 0 and 1.
    AnalogOutputPair0,
}

impl Default for CoaxialOutputSource {
    fn default() -> Self {
        Self::StreamInputPair
    }
}

const CLOCK_STATUS_OFFSET: u64 = 0x0228;
const CONFIG_FLAG_OFFSET: u64 = 0x022c;
const INPUT_THRESHOLD_OFFSET: u64 = 0x0230;

const CLOCK_SOURCES: [(ClkSrc, u8); 4] = [
    (ClkSrc::Internal, 0x01),
    (ClkSrc::Wordclock, 0x02),
    (ClkSrc::Spdif, 0x03),
    (ClkSrc::Adat, 0x04),
];

const CLOCK_RATES: [(ClkRate, u8); 4] = [
    (ClkRate::R44100, 0x01),
    (ClkRate::R48000, 0x02),
    (ClkRate::R88200, 0x03),
    (ClkRate::R96000, 0x04),
];

const COAXIAL_OUTPUT_SOURCES: [(CoaxialOutputSource, u32, u32); 2] = [
    (CoaxialOutputSource::StreamInputPair, 0x00000002, 0x00020000),
    (
        CoaxialOutputSource::AnalogOutputPair0,
        0x00000000,
        0x00000200,
    ),
];

/// The trait for common operation of isochronous models {
pub trait IsochCommonOperation {
    const SAMPLING_CLOCK_SOURCES: &'static [ClkSrc];

    const THRESHOLD_MIN: u16 = 1;
    const THRESHOLD_MAX: u16 = 0x7fff;

    fn get_sampling_clock_source(
        req: &mut FwReq,
        node: &mut FwNode,
        timeout_ms: u32,
    ) -> Result<ClkSrc, Error> {
        let mut frame = [0; 4];
        read_quadlet(req, node, CLOCK_STATUS_OFFSET, &mut frame, timeout_ms)?;
        let src = CLOCK_SOURCES
            .iter()
            .find_map(|(src, val)| if *val == frame[3] { Some(*src) } else { None })
            .ok_or_else(|| {
                let msg = format!("Unexpected value for source of clock: {}", frame[3]);
                Error::new(FileError::Io, &msg)
            })?;
        Self::SAMPLING_CLOCK_SOURCES
            .iter()
            .find_map(|&s| if s == src { Some(src) } else { None })
            .ok_or_else(|| {
                let msg = "Unsupported source of sampling clock";
                Error::new(FileError::Inval, &msg)
            })
    }

    fn set_sampling_clock_source(
        req: &mut FwReq,
        node: &mut FwNode,
        src: ClkSrc,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let _ = Self::SAMPLING_CLOCK_SOURCES
            .iter()
            .find(|&s| *s == src)
            .ok_or_else(|| {
                let msg = "Unsupported source of sampling clock";
                Error::new(FileError::Inval, &msg)
            })?;
        let val = CLOCK_SOURCES
            .iter()
            .find_map(|(s, val)| if *s == src { Some(*val) } else { None })
            .unwrap();
        let mut frame = [0; 4];
        read_quadlet(req, node, CLOCK_STATUS_OFFSET, &mut frame, timeout_ms)?;
        frame[0] = 0x00;
        frame[1] = 0x00;
        frame[3] = val;
        write_quadlet(req, node, CLOCK_STATUS_OFFSET, &mut frame, timeout_ms)
    }

    fn get_media_clock_rate(
        req: &mut FwReq,
        node: &mut FwNode,
        timeout_ms: u32,
    ) -> Result<ClkRate, Error> {
        let mut frames = [0; 4];
        read_quadlet(req, node, CLOCK_STATUS_OFFSET, &mut frames, timeout_ms)?;
        CLOCK_RATES
            .iter()
            .find_map(|(src, val)| if *val == frames[1] { Some(*src) } else { None })
            .ok_or_else(|| {
                let label = format!("Unexpected value for rate of clock: {}", frames[1]);
                Error::new(FileError::Io, &label)
            })
    }

    fn set_media_clock_rate(
        req: &mut FwReq,
        node: &mut FwNode,
        rate: ClkRate,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let val = CLOCK_RATES
            .iter()
            .find_map(|(r, val)| if *r == rate { Some(*val) } else { None })
            .unwrap();
        let mut frames = [0; 4];
        read_quadlet(req, node, CLOCK_STATUS_OFFSET, &mut frames, timeout_ms)?;
        frames[3] = val;
        write_quadlet(req, node, CLOCK_STATUS_OFFSET, &mut frames, timeout_ms)
    }

    fn get_coaxial_output_source(
        req: &mut FwReq,
        node: &mut FwNode,
        timeout_ms: u32,
    ) -> Result<CoaxialOutputSource, Error> {
        read_config_flag(req, node, &COAXIAL_OUTPUT_SOURCES, timeout_ms)
    }

    fn set_coaxial_output_source(
        req: &mut FwReq,
        node: &mut FwNode,
        src: CoaxialOutputSource,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        write_config_flag(req, node, &COAXIAL_OUTPUT_SOURCES, src, timeout_ms)
    }

    /// Get threshold of input gain for signal detection. The value between 1 and 0x7fff returns.
    /// The dB level can be calculated by below formula:
    ///
    /// ```text
    /// level = 20 * log10(value / 0x7fff)
    /// ```
    fn get_analog_input_threshold_for_signal_detection(
        req: &mut FwReq,
        node: &mut FwNode,
        timeout_ms: u32,
    ) -> Result<u16, Error> {
        let mut quads = [0; 4];
        read_quadlet(req, node, INPUT_THRESHOLD_OFFSET, &mut quads, timeout_ms).map(|_| {
            let val = (u32::from_be_bytes(quads) & 0x0000ffff) as u16;
            val.clamp(Self::THRESHOLD_MIN, Self::THRESHOLD_MAX)
        })
    }

    /// Set threshold of input gain for signal detection. The value should be between 1 and 0x7fff.
    /// The value can be calculated by below formula:
    ///
    /// ```text
    /// value = 0x7fff * pow(10, level / 20)
    /// ```
    fn set_analog_input_threshold_for_signal_detection(
        req: &mut FwReq,
        node: &mut FwNode,
        value: u16,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        if value > Self::THRESHOLD_MAX || value < Self::THRESHOLD_MIN {
            let msg = format!(
                "Argument should be greater than {} and less than {}, but {}",
                Self::THRESHOLD_MAX,
                Self::THRESHOLD_MIN,
                value
            );
            Err(Error::new(FileError::Inval, &msg))?;
        }

        let mut quads = [0; 4];
        read_quadlet(req, node, INPUT_THRESHOLD_OFFSET, &mut quads, timeout_ms)?;

        let val = (u32::from_be_bytes(quads) & 0xffff0000) | (value as u32);

        let mut quads = val.to_be_bytes();
        write_quadlet(req, node, INPUT_THRESHOLD_OFFSET, &mut quads, timeout_ms)
    }

    /// Get threshold of input gain for over-level detection. The value between 1 and 0x7fff
    /// returns. The dB level can be calculated by below formula:
    ///
    /// ```text
    /// level = 20 * log10(value / 0x7fff)
    /// ```
    fn get_analog_input_threshold_for_over_level_detection(
        req: &mut FwReq,
        node: &mut FwNode,
        timeout_ms: u32,
    ) -> Result<u16, Error> {
        let mut quads = [0; 4];
        read_quadlet(req, node, INPUT_THRESHOLD_OFFSET, &mut quads, timeout_ms).map(|_| {
            let val = (u32::from_be_bytes(quads) >> 16) as u16;
            val.clamp(Self::THRESHOLD_MIN, Self::THRESHOLD_MAX)
        })
    }

    /// Set threshold of input gain for over-level detection. The value should be between 1 and
    /// 0x7fff. The value can be calculated by below formula:
    ///
    /// ```text
    /// value = 0x7fff * pow(10, level / 20)
    /// ```
    fn set_analog_input_threshold_for_over_level_detection(
        req: &mut FwReq,
        node: &mut FwNode,
        value: u16,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        if value > Self::THRESHOLD_MAX || value < Self::THRESHOLD_MIN {
            let msg = format!(
                "Argument should be greater than {} and less than {}, but {}",
                Self::THRESHOLD_MAX,
                Self::THRESHOLD_MIN,
                value
            );
            Err(Error::new(FileError::Inval, &msg))?;
        }

        let mut quads = [0; 4];
        read_quadlet(req, node, INPUT_THRESHOLD_OFFSET, &mut quads, timeout_ms)?;

        let val = (u32::from_be_bytes(quads) & 0x0000ffff) | ((value as u32) << 16);

        let mut quads = val.to_be_bytes();
        write_quadlet(req, node, INPUT_THRESHOLD_OFFSET, &mut quads, timeout_ms)
    }
}

/// Source of S/PDIF input.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SpdifCaptureSource {
    /// To coaxial interface.
    Coaxial,
    /// To optical interface.
    Optical,
}

impl Default for SpdifCaptureSource {
    fn default() -> Self {
        Self::Coaxial
    }
}

/// Source of optical output.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum OpticalOutputSource {
    /// 4 pairs in stream inputs.
    StreamInputPairs,
    /// Mirror of coaxial output 0 and 1.
    CoaxialOutputPair0,
    /// Analog input 0 and 1.
    AnalogInputPair0,
    /// Mirror of analog output 0, 1, 2, 3, 4, 5, 6, 7, and 8.
    AnalogOutputPairs,
}

impl Default for OpticalOutputSource {
    fn default() -> Self {
        Self::StreamInputPairs
    }
}

/// The parameters of digital input and output interfaces.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct TascamOpticalIfaceParameters {
    /// The input interface from which the S/PDIF signal is captured.
    pub capture_source: SpdifCaptureSource,
    /// The source signal of optical output interface.
    pub output_source: OpticalOutputSource,
}

const SPDIF_CAPTURE_SOURCES: &[(SpdifCaptureSource, u32, u32)] = &[
    (SpdifCaptureSource::Coaxial, 0x00000000, 0x00010000),
    (SpdifCaptureSource::Optical, 0x00000001, 0x00000100),
];

/// The trait for operation of optical input/output interface.
pub trait IsochOpticalOperation {
    const OPTICAL_OUTPUT_SOURCES: &'static [(OpticalOutputSource, u32, u32)];

    fn get_spdif_capture_source(
        req: &mut FwReq,
        node: &mut FwNode,
        timeout_ms: u32,
    ) -> Result<SpdifCaptureSource, Error> {
        read_config_flag(req, node, &SPDIF_CAPTURE_SOURCES, timeout_ms)
    }

    fn set_spdif_capture_source(
        req: &mut FwReq,
        node: &mut FwNode,
        src: SpdifCaptureSource,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        write_config_flag(req, node, &SPDIF_CAPTURE_SOURCES, src, timeout_ms)
    }

    fn get_opt_output_source(
        req: &mut FwReq,
        node: &mut FwNode,
        timeout_ms: u32,
    ) -> Result<OpticalOutputSource, Error> {
        read_config_flag(req, node, &Self::OPTICAL_OUTPUT_SOURCES, timeout_ms)
    }

    fn set_opt_output_source(
        req: &mut FwReq,
        node: &mut FwNode,
        src: OpticalOutputSource,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        write_config_flag(req, node, &Self::OPTICAL_OUTPUT_SOURCES, src, timeout_ms)
    }
}

/// State of console.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct IsochConsoleState {
    /// Whether to enable host mode.
    pub host_mode: bool,

    /// Whether to assign master fader to analog output 1/2.
    pub master_fader_assign: bool,
}

const MASTER_FADER_ASSIGNS: [(bool, u32, u32); 2] = [
    (false, 0x00000040, 0x00400000),
    (true, 0x00000000, 0x00004000),
];

/// The trait for operation of console model.
pub trait IsochConsoleOperation {
    fn parse_console_state(state: &mut IsochConsoleState, image: &[u32]) -> Result<(), Error> {
        state.host_mode = (image[5] & 0xff000000) != 0xff000000;
        Ok(())
    }

    fn get_master_fader_assign(
        req: &mut FwReq,
        node: &mut FwNode,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        read_config_flag(req, node, &MASTER_FADER_ASSIGNS, timeout_ms)
    }

    fn set_master_fader_assign(
        req: &mut FwReq,
        node: &mut FwNode,
        enable: bool,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        write_config_flag(req, node, &MASTER_FADER_ASSIGNS, enable, timeout_ms)
    }
}

const RACK_STATE_SIZE: usize = 72;

/// State of rack.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct IsochRackState([u8; RACK_STATE_SIZE]);

impl Default for IsochRackState {
    fn default() -> Self {
        Self([0; RACK_STATE_SIZE])
    }
}

/// The parameters of input.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct IsochRackInputParameters {
    /// Gain of signal to stereo monitor. The value is between 0 and 0x7fff.
    pub gains: [i16; 18],
    /// L/R balance to stereo monitor. The value is between 0 and 255.
    pub balances: [u8; 18],
    /// Whether to mute.
    pub mutes: [bool; 18],
    /// The raw state of parameters.
    pub state: IsochRackState,
}

const INPUT_OFFSET: u64 = 0x0408;

fn write_input_quadlet(
    req: &mut FwReq,
    node: &mut FwNode,
    state: &mut IsochRackState,
    pos: usize,
    timeout_ms: u32,
) -> Result<(), Error> {
    write_quadlet(
        req,
        node,
        INPUT_OFFSET,
        &mut state.0[pos..(pos + 4)],
        timeout_ms,
    )
}

/// The trait for operation of rack model.
pub trait IsochRackOperation {
    const CHANNEL_COUNT: usize = 18;

    const GAIN_MIN: i16 = 0;
    const GAIN_MAX: i16 = 0x7fff;
    const GAIN_STEP: i16 = 0x100;

    const BALANCE_MIN: u8 = 0;
    const BALANCE_MAX: u8 = 255;
    const BALANCE_STEP: u8 = 1;

    fn init_input_state(
        req: &mut FwReq,
        node: &mut FwNode,
        state: &mut IsochRackState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let val: i16 = 0x7fff;

        (0..18).try_for_each(|i| {
            let pos = i * 4;

            // Channel index field.
            state.0[pos] = i as u8;

            // L/R balance field.
            state.0[pos + 1] = if i % 2 > 0 { 0xff } else { 0x00 };

            // Level gain field.
            state.0[(pos + 2)..(pos + 4)].copy_from_slice(&val.to_le_bytes());

            write_input_quadlet(req, node, state, pos, timeout_ms)
        })
    }

    fn get_input_gain(state: &IsochRackState, index: usize) -> i16 {
        let pos = index * 4;
        i16::from_be_bytes([state.0[pos + 2], state.0[pos + 3]])
    }

    fn set_input_gain(
        req: &mut FwReq,
        node: &mut FwNode,
        index: usize,
        gain: i16,
        state: &mut IsochRackState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let pos = index * 4;
        state.0[(pos + 2)..(pos + 4)].copy_from_slice(&gain.to_be_bytes());
        write_input_quadlet(req, node, state, pos, timeout_ms)
    }

    fn get_input_balance(state: &IsochRackState, index: usize) -> u8 {
        let pos = index * 4;
        state.0[pos + 1]
    }

    fn set_input_balance(
        req: &mut FwReq,
        node: &mut FwNode,
        index: usize,
        pan: u8,
        state: &mut IsochRackState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let pos = index * 4;
        state.0[pos + 1] = pan;
        write_input_quadlet(req, node, state, pos, timeout_ms)
    }

    fn get_input_mute(state: &IsochRackState, index: usize) -> bool {
        let pos = index * 4;
        state.0[pos] & 0x80 > 0
    }

    fn set_input_mute(
        req: &mut FwReq,
        node: &mut FwNode,
        index: usize,
        mute: bool,
        state: &mut IsochRackState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let pos = index * 4;
        state.0[pos] &= !0x80;
        if mute {
            state.0[pos] |= 0x80;
        }
        write_input_quadlet(req, node, state, pos, timeout_ms)
    }
}

/// State of surface in isoch models.
#[derive(Default, Debug)]
struct SurfaceIsochState {
    shifted: bool,
    shifted_items: Vec<bool>,
    bank: u16,
}

/// The trait for operation specific to isoch models.
trait SurfaceImageIsochOperation {
    const SHIFT_ITEM: SurfaceBoolValue;
    const SHIFTED_ITEMS: &'static [(SurfaceBoolValue, [MachineItem; 2])];
    const BANK_CURSORS: [SurfaceBoolValue; 2];

    fn initialize_surface_isoch_state(state: &mut SurfaceIsochState) {
        state.shifted = false;
        state.shifted_items = vec![false; Self::SHIFTED_ITEMS.len()];
        state.bank = 0;
    }

    fn decode_surface_image_isoch(
        machine_values: &mut Vec<(MachineItem, ItemValue)>,
        state: &SurfaceIsochState,
        index: u32,
        before: u32,
        after: u32,
    ) {
        let shifted = if detect_bool_action(&Self::SHIFT_ITEM, index, before, after) {
            let shifted = detect_bool_value(&Self::SHIFT_ITEM, before);
            machine_values.push((MachineItem::Shift, ItemValue::Bool(shifted)));
            shifted
        } else {
            state.shifted
        };

        if shifted != state.shifted {
            let prev_idx = state.shifted as usize;
            let curr_idx = shifted as usize;

            Self::SHIFTED_ITEMS
                .iter()
                .zip(&state.shifted_items)
                .filter(|(_, &s)| s)
                .for_each(|((_, pairs), _)| {
                    machine_values.push((pairs[prev_idx], ItemValue::Bool(false)));
                    machine_values.push((pairs[curr_idx], ItemValue::Bool(true)));
                });
        }

        Self::SHIFTED_ITEMS
            .iter()
            .filter(|(bool_val, _)| detect_bool_action(bool_val, index, before, after))
            .for_each(|(bool_val, pairs)| {
                let value = detect_bool_value(bool_val, before);
                machine_values.push((pairs[shifted as usize], ItemValue::Bool(value)));
            });

        Self::BANK_CURSORS
            .iter()
            .enumerate()
            .filter(|(_, bool_val)| detect_bool_action(bool_val, index, before, after))
            .for_each(|(idx, bool_val)| {
                let is_right = idx > 0;
                let push_event = detect_bool_value(bool_val, before);
                if push_event {
                    let mut bank = state.bank;

                    if !is_right && bank > BANK_MIN {
                        bank -= 1;
                    } else if is_right && bank < BANK_MAX {
                        bank += 1;
                    }

                    if bank != state.bank {
                        machine_values.push((MachineItem::Bank, ItemValue::U16(bank)));
                    }
                }
            });
    }

    fn feedback_to_surface_isoch(
        state: &mut SurfaceIsochState,
        machine_value: &(MachineItem, ItemValue),
    ) {
        match machine_value {
            &(MachineItem::Shift, ItemValue::Bool(value)) => state.shifted = value,
            &(MachineItem::Bank, ItemValue::U16(value)) => state.bank = value,
            _ => (),
        }
    }
}

/// The trait for operation of bank LEDs in surface.
trait SurfaceBankLedOperation {
    const BANK_LEDS: [&'static [u16]; 4];

    fn operate_bank_leds(
        state: &mut LedState,
        req: &mut FwReq,
        node: &mut FwNode,
        bank: u16,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        Self::BANK_LEDS
            .iter()
            .enumerate()
            .try_for_each(|(i, positions)| {
                let enable = bank == i as u16;
                let pos = positions[0];
                operate_led_cached(state, req, node, pos, enable, timeout_ms)
            })
    }
}
