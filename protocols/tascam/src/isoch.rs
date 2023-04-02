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

/// Operation to parse parameters in the image of hardware state.
pub trait TascamIsochImageParamsOperation<T> {
    /// Parse the image of hardware state.
    fn parse_image(params: &mut T, image: &[u32]);
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

/// The specification of sampling and media clocks.
pub trait TascamIsochClockSpecification {
    const SAMPLING_CLOCK_SOURCES: &'static [ClkSrc];
}

const CLOCK_STATUS_OFFSET: u64 = 0x0228;

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

impl<O> TascamIsochWhollyCachableParamsOperation<TascamClockParameters> for O
where
    O: TascamIsochClockSpecification,
{
    fn cache_wholly(
        req: &mut FwReq,
        node: &mut FwNode,
        states: &mut TascamClockParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut frame = [0; 4];
        read_quadlet(req, node, CLOCK_STATUS_OFFSET, &mut frame, timeout_ms)?;
        let src = CLOCK_SOURCES
            .iter()
            .find_map(|&(src, val)| if val == frame[3] { Some(src) } else { None })
            .ok_or_else(|| {
                let msg = format!("Unexpected value for source of clock: {}", frame[3]);
                Error::new(FileError::Io, &msg)
            })?;
        Self::SAMPLING_CLOCK_SOURCES
            .iter()
            .find_map(|s| if src.eq(s) { Some(src) } else { None })
            .ok_or_else(|| {
                let msg = "Unsupported source of sampling clock";
                Error::new(FileError::Inval, &msg)
            })
            .map(|src| states.sampling_clock_source = src)?;
        CLOCK_RATES
            .iter()
            .find_map(|&(rate, val)| if val == frame[1] { Some(rate) } else { None })
            .ok_or_else(|| {
                let label = format!("Unexpected value for rate of clock: {}", frame[1]);
                Error::new(FileError::Io, &label)
            })
            .map(|rate| states.media_clock_rate = rate)
    }
}

impl<O> TascamIsochWhollyUpdatableParamsOperation<TascamClockParameters> for O
where
    O: TascamIsochClockSpecification,
{
    /// Update whole parameters.
    fn update_wholly(
        req: &mut FwReq,
        node: &mut FwNode,
        params: &TascamClockParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let _ = Self::SAMPLING_CLOCK_SOURCES
            .iter()
            .find(|s| params.sampling_clock_source.eq(s))
            .ok_or_else(|| {
                let msg = "Unsupported source of sampling clock";
                Error::new(FileError::Inval, &msg)
            })?;
        let mut frame = [0; 4];
        frame[3] = CLOCK_SOURCES
            .iter()
            .find_map(|&(s, val)| {
                if params.sampling_clock_source.eq(&s) {
                    Some(val)
                } else {
                    None
                }
            })
            .unwrap();
        frame[2] = CLOCK_RATES
            .iter()
            .find_map(|&(r, val)| {
                if params.media_clock_rate.eq(&r) {
                    Some(val)
                } else {
                    None
                }
            })
            .unwrap();
        write_quadlet(req, node, CLOCK_STATUS_OFFSET, &mut frame, timeout_ms)
    }
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

/// The specification of input detection.
pub trait TascamIsochInputDetectionSpecification {
    /// The minimum value of threshold to detect input signal.
    const INPUT_SIGNAL_THRESHOLD_MIN: u16 = 1;
    /// The maximum value of threshold to detect input signal.
    const INPUT_SIGNAL_THRESHOLD_MAX: u16 = 0x7fff;
}

const INPUT_THRESHOLD_OFFSET: u64 = 0x0230;

impl<O> TascamIsochWhollyCachableParamsOperation<TascamInputDetectionThreshold> for O
where
    O: TascamIsochInputDetectionSpecification,
{
    fn cache_wholly(
        req: &mut FwReq,
        node: &mut FwNode,
        states: &mut TascamInputDetectionThreshold,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut quads = [0; 4];
        read_quadlet(req, node, INPUT_THRESHOLD_OFFSET, &mut quads, timeout_ms).map(|_| {
            let quad = u32::from_be_bytes(quads);
            let val = (quad & 0x0000ffff) as u16;
            states.signal = val.clamp(
                Self::INPUT_SIGNAL_THRESHOLD_MIN,
                Self::INPUT_SIGNAL_THRESHOLD_MAX,
            );

            let val = (quad >> 16) as u16;
            states.over_level = val.clamp(
                Self::INPUT_SIGNAL_THRESHOLD_MIN,
                Self::INPUT_SIGNAL_THRESHOLD_MAX,
            );
        })
    }
}

impl<O> TascamIsochWhollyUpdatableParamsOperation<TascamInputDetectionThreshold> for O
where
    O: TascamIsochInputDetectionSpecification,
{
    fn update_wholly(
        req: &mut FwReq,
        node: &mut FwNode,
        params: &TascamInputDetectionThreshold,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        if params.signal > Self::INPUT_SIGNAL_THRESHOLD_MAX
            || params.signal < Self::INPUT_SIGNAL_THRESHOLD_MIN
        {
            let msg = format!(
                "Argument should be greater than {} and less than {}, but {}",
                Self::INPUT_SIGNAL_THRESHOLD_MIN,
                Self::INPUT_SIGNAL_THRESHOLD_MAX,
                params.signal
            );
            Err(Error::new(FileError::Inval, &msg))?;
        }

        if params.over_level > Self::INPUT_SIGNAL_THRESHOLD_MAX
            || params.over_level < Self::INPUT_SIGNAL_THRESHOLD_MIN
        {
            let msg = format!(
                "Argument should be greater than {} and less than {}, but {}",
                Self::INPUT_SIGNAL_THRESHOLD_MIN,
                Self::INPUT_SIGNAL_THRESHOLD_MAX,
                params.over_level
            );
            Err(Error::new(FileError::Inval, &msg))?;
        }

        let quad = ((params.over_level as u32) << 16) | (params.signal as u32);

        write_quadlet(
            req,
            node,
            INPUT_THRESHOLD_OFFSET,
            &mut quad.to_be_bytes(),
            timeout_ms,
        )
    }
}

const ISOCH_IMAGE_QUADLET_COUNT: usize = 64;

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

/// The specification of metering.
pub trait TascamIsochMeterSpecification: TascamHardwareImageSpecification {
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
}

impl<O> TascamIsochImageParamsOperation<IsochMeterState> for O
where
    O: TascamIsochMeterSpecification,
{
    fn parse_image(state: &mut IsochMeterState, image: &[u32]) {
        assert_eq!(image.len(), Self::IMAGE_QUADLET_COUNT);

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
                2 => MonitorMode::Inputs,
                1 => MonitorMode::Computer,
                _ => MonitorMode::Both,
            };
        }
    }
}

fn serialize_config_flag<T: Copy + Eq>(
    option: &T,
    flags: &[(T, u32, u32)],
    val: &mut u32,
) -> Result<(), Error> {
    let mask = flags.iter().fold(0, |mask, (_, _, flag)| mask | flag);
    *val &= !mask;
    let (_, _, flag) = flags.iter().find(|(o, _, _)| option.eq(o)).unwrap();
    *val |= flag;
    Ok(())
}

fn deserialize_config_flag<T: Copy + Eq>(
    option: &mut T,
    flags: &[(T, u32, u32)],
    val: u32,
) -> Result<(), Error> {
    let mask = flags.iter().fold(0, |mask, (_, flag, _)| mask | flag);
    flags
        .iter()
        .find(|&(_, flag, _)| val & mask == *flag)
        .ok_or_else(|| {
            let msg = format!("No flag detected: val: 0x{:08x} mask: 0x{:08x}", val, mask);
            Error::new(FileError::Nxio, &msg)
        })
        .map(|&(opt, _, _)| *option = opt)
}

fn read_config(
    req: &mut FwReq,
    node: &mut FwNode,
    config: &mut u32,
    timeout_ms: u32,
) -> Result<(), Error> {
    let mut quads = [0; 4];
    read_quadlet(req, node, CONFIG_FLAG_OFFSET, &mut quads, timeout_ms).map(|_| {
        *config = u32::from_be_bytes(quads);
    })
}

fn write_config(
    req: &mut FwReq,
    node: &mut FwNode,
    config: u32,
    timeout_ms: u32,
) -> Result<(), Error> {
    write_quadlet(
        req,
        node,
        CONFIG_FLAG_OFFSET,
        &mut config.to_be_bytes(),
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

/// The specification of coaxial output interface.
pub trait TascamIsochCoaxialOutputSpecification {}

const COAXIAL_OUTPUT_SOURCES: [(CoaxialOutputSource, u32, u32); 2] = [
    (CoaxialOutputSource::StreamInputPair, 0x00000002, 0x00020000),
    (
        CoaxialOutputSource::AnalogOutputPair0,
        0x00000000,
        0x00000200,
    ),
];

impl<O> TascamIsochWhollyCachableParamsOperation<CoaxialOutputSource> for O
where
    O: TascamIsochCoaxialOutputSpecification,
{
    fn cache_wholly(
        req: &mut FwReq,
        node: &mut FwNode,
        states: &mut CoaxialOutputSource,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut config = 0;
        read_config(req, node, &mut config, timeout_ms)?;
        deserialize_config_flag(states, &COAXIAL_OUTPUT_SOURCES, config)
    }
}

impl<O> TascamIsochWhollyUpdatableParamsOperation<CoaxialOutputSource> for O
where
    O: TascamIsochCoaxialOutputSpecification,
{
    fn update_wholly(
        req: &mut FwReq,
        node: &mut FwNode,
        states: &CoaxialOutputSource,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut config = 0;
        serialize_config_flag(states, &COAXIAL_OUTPUT_SOURCES, &mut config)?;
        write_config(req, node, config, timeout_ms)
    }
}

const CONFIG_FLAG_OFFSET: u64 = 0x022c;

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

/// The specification of digital interfaces.
pub trait TascamIsochOpticalIfaceSpecification {
    const OPTICAL_OUTPUT_SOURCES: &'static [(OpticalOutputSource, u32, u32)];
}

const SPDIF_CAPTURE_SOURCES: &[(SpdifCaptureSource, u32, u32)] = &[
    (SpdifCaptureSource::Coaxial, 0x00000000, 0x00010000),
    (SpdifCaptureSource::Optical, 0x00000001, 0x00000100),
];

impl<O> TascamIsochWhollyCachableParamsOperation<TascamOpticalIfaceParameters> for O
where
    O: TascamIsochOpticalIfaceSpecification,
{
    fn cache_wholly(
        req: &mut FwReq,
        node: &mut FwNode,
        states: &mut TascamOpticalIfaceParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut config = 0;
        read_config(req, node, &mut config, timeout_ms)?;
        deserialize_config_flag(&mut states.capture_source, &SPDIF_CAPTURE_SOURCES, config)?;
        deserialize_config_flag(
            &mut states.output_source,
            &Self::OPTICAL_OUTPUT_SOURCES,
            config,
        )?;
        Ok(())
    }
}

impl<O> TascamIsochWhollyUpdatableParamsOperation<TascamOpticalIfaceParameters> for O
where
    O: TascamIsochOpticalIfaceSpecification,
{
    /// Update whole parameters.
    fn update_wholly(
        req: &mut FwReq,
        node: &mut FwNode,
        states: &TascamOpticalIfaceParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut config = 0;
        serialize_config_flag(&states.capture_source, &SPDIF_CAPTURE_SOURCES, &mut config)?;
        serialize_config_flag(
            &states.output_source,
            &Self::OPTICAL_OUTPUT_SOURCES,
            &mut config,
        )?;
        write_config(req, node, config, timeout_ms)
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

/// The specification of console.
pub trait TascamIsochConsoleSpecification: TascamHardwareImageSpecification {}

const MASTER_FADER_ASSIGNS: [(bool, u32, u32); 2] = [
    (false, 0x00000040, 0x00400000),
    (true, 0x00000000, 0x00004000),
];

impl<O> TascamIsochWhollyCachableParamsOperation<IsochConsoleState> for O
where
    O: TascamIsochConsoleSpecification,
{
    fn cache_wholly(
        req: &mut FwReq,
        node: &mut FwNode,
        states: &mut IsochConsoleState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut config = 0;
        read_config(req, node, &mut config, timeout_ms)?;
        deserialize_config_flag(
            &mut states.master_fader_assign,
            &MASTER_FADER_ASSIGNS,
            config,
        )
    }
}

impl<O> TascamIsochWhollyUpdatableParamsOperation<IsochConsoleState> for O
where
    O: TascamIsochConsoleSpecification,
{
    fn update_wholly(
        req: &mut FwReq,
        node: &mut FwNode,
        states: &IsochConsoleState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut config = 0;
        serialize_config_flag(
            &states.master_fader_assign,
            &MASTER_FADER_ASSIGNS,
            &mut config,
        )?;
        write_config(req, node, config, timeout_ms)
    }
}

impl<O> TascamIsochImageParamsOperation<IsochConsoleState> for O
where
    O: TascamIsochConsoleSpecification + TascamHardwareImageSpecification,
{
    fn parse_image(params: &mut IsochConsoleState, image: &[u32]) {
        assert_eq!(image.len(), Self::IMAGE_QUADLET_COUNT);
        params.host_mode = (image[5] & 0xff000000) != 0xff000000;
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
}

const RACK_STATE_SIZE: usize = 72;

fn serialize_input_params(params: &IsochRackInputParameters, raw: &mut [u8; RACK_STATE_SIZE]) {
    (0..18).for_each(|i| {
        let val = ((i as u32) << 24) | ((params.mutes[i] as u32) << 16) | (params.gains[i] as u32);
        let pos = i * 4;
        raw[pos..(pos + 4)].copy_from_slice(&val.to_be_bytes());
    });
}

#[cfg(test)]
fn deserialize_input_params(params: &mut IsochRackInputParameters, raw: &[u8; RACK_STATE_SIZE]) {
    (0..RACK_STATE_SIZE).step_by(4).for_each(|pos| {
        let mut quad = [0; 4];
        quad.copy_from_slice(&raw[pos..(pos + 4)]);
        let val = u32::from_be_bytes(quad);
        let i = (val >> 24) as usize;
        let muted = (val & 0x00ff0000) > 0;
        let gain = (val & 0x0000ffff) as i16;
        if i < params.gains.len() {
            params.gains[i] = gain;
            params.mutes[i] = muted;
        }
    });
}

/// The specification of rack input.
pub trait TascamIsochRackInputSpecification {
    /// The number of input channels.
    const CHANNEL_COUNT: usize = 18;

    /// The minimum value of gain.
    const INPUT_GAIN_MIN: i16 = 0;
    /// The maximum value of gain.
    const INPUT_GAIN_MAX: i16 = 0x7fff;
    /// The step value of gain.
    const INPUT_GAIN_STEP: i16 = 0x100;

    /// The minimum value of L/R balance.
    const INPUT_BALANCE_MIN: u8 = 0;
    /// The maximum value of L/R balance.
    const INPUT_BALANCE_MAX: u8 = 255;
    /// The step value of L/R balance.
    const INPUT_BALANCE_STEP: u8 = 1;

    fn create_input_parameters() -> IsochRackInputParameters {
        let mut params = IsochRackInputParameters::default();
        params.gains.fill(Self::INPUT_GAIN_MAX);
        params
            .balances
            .iter_mut()
            .enumerate()
            .for_each(|(i, balance)| {
                *balance = if i % 2 > 0 { u8::MAX } else { u8::MIN };
            });
        params.mutes.fill(false);
        params
    }
}

const INPUT_OFFSET: u64 = 0x0408;

fn write_input_quadlet(
    req: &mut FwReq,
    node: &mut FwNode,
    quad: &mut [u8],
    timeout_ms: u32,
) -> Result<(), Error> {
    assert_eq!(quad.len(), 4);

    write_quadlet(req, node, INPUT_OFFSET, quad, timeout_ms)
}

impl<O> TascamIsochWhollyUpdatableParamsOperation<IsochRackInputParameters> for O
where
    O: TascamIsochRackInputSpecification,
{
    fn update_wholly(
        req: &mut FwReq,
        node: &mut FwNode,
        states: &IsochRackInputParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut raw = [0; RACK_STATE_SIZE];
        serialize_input_params(states, &mut raw);
        (0..RACK_STATE_SIZE).step_by(4).try_for_each(|pos| {
            write_input_quadlet(req, node, &mut raw[pos..(pos + 4)], timeout_ms)
        })
    }
}

impl<O> TascamIsochPartiallyUpdatableParamsOperation<IsochRackInputParameters> for O
where
    O: TascamIsochRackInputSpecification,
{
    fn update_partially(
        req: &mut FwReq,
        node: &mut FwNode,
        params: &mut IsochRackInputParameters,
        update: IsochRackInputParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut old = [0; RACK_STATE_SIZE];
        serialize_input_params(params, &mut old);

        let mut new = [0; RACK_STATE_SIZE];
        serialize_input_params(&update, &mut new);

        (0..RACK_STATE_SIZE)
            .step_by(4)
            .try_for_each(|pos| {
                if old[pos..(pos + 4)] != new[pos..(pos + 4)] {
                    write_input_quadlet(req, node, &mut new[pos..(pos + 4)], timeout_ms)
                } else {
                    Ok(())
                }
            })
            .map(|_| *params = update)
    }
}

/// State of surface in isoch models.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct TascamSurfaceIsochState {
    shifted: bool,
    shifted_items: Vec<bool>,
    bank: u16,
    enabled_leds: LedState,
}

/// The trait to express specification of LEDS for isochronous models.
pub trait TascamSurfaceLedIsochSpecification {
    const BANK_LEDS: [&'static [u16]; 4];
}

impl<O> TascamSurfaceLedOperation<TascamSurfaceIsochState> for O
where
    O: TascamSurfaceLedIsochSpecification,
{
    fn operate_leds(
        state: &mut TascamSurfaceIsochState,
        machine_value: &(MachineItem, ItemValue),
        req: &mut FwReq,
        node: &mut FwNode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        if let (MachineItem::Bank, ItemValue::U16(value)) = machine_value {
            Self::BANK_LEDS
                .iter()
                .enumerate()
                .try_for_each(|(i, positions)| {
                    let enable = *value == i as u16;
                    operate_led_cached(
                        &mut state.enabled_leds,
                        req,
                        node,
                        positions[0],
                        enable,
                        timeout_ms,
                    )
                })?;
        }

        Ok(())
    }

    fn clear_leds(
        state: &mut TascamSurfaceIsochState,
        req: &mut FwReq,
        node: &mut FwNode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        clear_leds(&mut state.enabled_leds, req, node, timeout_ms)
    }
}

/// The trait for operation specific to isoch models.
trait SurfaceImageIsochOperation {
    const SHIFT_ITEM: SurfaceBoolValue;
    const SHIFTED_ITEMS: &'static [(SurfaceBoolValue, [MachineItem; 2])];
    const BANK_CURSORS: [SurfaceBoolValue; 2];

    fn initialize_surface_isoch_state(state: &mut TascamSurfaceIsochState) {
        state.shifted = false;
        state.shifted_items = vec![false; Self::SHIFTED_ITEMS.len()];
        state.bank = 0;
    }

    fn decode_surface_image_isoch(
        machine_values: &mut Vec<(MachineItem, ItemValue)>,
        state: &TascamSurfaceIsochState,
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
        state: &mut TascamSurfaceIsochState,
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn config_flag_serdes() {
        #[derive(Debug, Copy, Clone, PartialEq, Eq)]
        enum Flag {
            A,
            B,
            C,
            N,
        }
        const TABLE: &[(Flag, u32, u32)] = &[
            (Flag::A, 0x00000001, 0x10000000),
            (Flag::B, 0x00000020, 0x20000000),
            (Flag::C, 0x00000004, 0x00040000),
        ];

        let mut param = Flag::N;
        deserialize_config_flag(&mut param, &TABLE, 0x00000001).unwrap();
        assert_eq!(param, Flag::A);

        deserialize_config_flag(&mut param, &TABLE, 0x00000000).unwrap_err();

        let mut val = 0;
        serialize_config_flag(&param, &TABLE, &mut val).unwrap();
        assert_eq!(val, 0x10000000);
    }

    #[test]
    fn rack_input_params_serdes() {
        let orig = IsochRackInputParameters::default();
        let mut raw = [0; RACK_STATE_SIZE];
        serialize_input_params(&orig, &mut raw);

        let mut target = IsochRackInputParameters::default();
        deserialize_input_params(&mut target, &raw);

        assert_eq!(target, orig);
    }
}
