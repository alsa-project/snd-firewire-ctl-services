// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Protocol defined by TC Electronic for Konnekt 24d, Konnekt 8, Konnekt Live, and Impact Twin.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by TC Electronic for Konnekt 24d, Konnekt 8, Konnekt Live, and Impact Twin.

pub mod itwin;
pub mod k24d;
pub mod k8;
pub mod klive;

use super::{ch_strip::*, reverb::*, *};

const SHELL_KNOB_NOTIFY_FLAG: u32 = 0x00010000;
const SHELL_CONFIG_NOTIFY_FLAG: u32 = 0x00020000;
const SHELL_MIXER_NOTIFY_FLAG: u32 = 0x00040000;
const SHELL_REVERB_NOTIFY_FLAG: u32 = 0x00080000;
const SHELL_CH_STRIP_NOTIFY_FLAG: u32 = 0x00100000;
// NOTE: 0x00200000 is for tuner.
// NOTE: 0x00400000 is unidentified.
const SHELL_HW_STATE_NOTIFY_FLAG: u32 = 0x01000000;

const SHELL_CH_STRIP_COUNT: usize = 2;

/// State of jack sense for analog input.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ShellAnalogJackState {
    /// Select front jack instead of rear.
    FrontSelected,
    /// Detect plug insertion in front jack.
    FrontInserted,
    /// Detect plug insertion in front jack with attenuation.
    FrontInsertedAttenuated,
    /// Select rear jack instead of front.
    RearSelected,
    /// Detect plug insertion in rear jack.
    RearInserted,
}

impl Default for ShellAnalogJackState {
    fn default() -> Self {
        Self::FrontSelected
    }
}

impl ShellAnalogJackState {
    const FRONT_SELECTED: u32 = 0x00;
    const FRONT_INSERTED: u32 = 0x05;
    const FRONT_INSERTED_ATTENUATED: u32 = 0x06;
    const REAR_SELECTED: u32 = 0x07;
    const REAR_INSERTED: u32 = 0x08;
}

fn serialize_analog_jack_state(state: &ShellAnalogJackState, raw: &mut [u8]) -> Result<(), String> {
    assert!(raw.len() >= 4);

    let val = match state {
        ShellAnalogJackState::FrontSelected => ShellAnalogJackState::FRONT_SELECTED,
        ShellAnalogJackState::FrontInserted => ShellAnalogJackState::FRONT_INSERTED,
        ShellAnalogJackState::FrontInsertedAttenuated => {
            ShellAnalogJackState::FRONT_INSERTED_ATTENUATED
        }
        ShellAnalogJackState::RearSelected => ShellAnalogJackState::REAR_SELECTED,
        ShellAnalogJackState::RearInserted => ShellAnalogJackState::REAR_INSERTED,
    };

    serialize_u32(&val, raw);

    Ok(())
}

fn deserialize_analog_jack_state(
    state: &mut ShellAnalogJackState,
    raw: &[u8],
) -> Result<(), String> {
    assert!(raw.len() >= 4);

    let mut val = 0u32;
    deserialize_u32(&mut val, raw);

    *state = match val & 0xff {
        ShellAnalogJackState::FRONT_INSERTED => ShellAnalogJackState::FrontInserted,
        ShellAnalogJackState::FRONT_INSERTED_ATTENUATED => {
            ShellAnalogJackState::FrontInsertedAttenuated
        }
        ShellAnalogJackState::REAR_SELECTED => ShellAnalogJackState::RearSelected,
        ShellAnalogJackState::REAR_INSERTED => ShellAnalogJackState::RearInserted,
        ShellAnalogJackState::FRONT_SELECTED => ShellAnalogJackState::FrontSelected,
        _ => Err(format!("Invalid value of analog jack state: {}", val))?,
    };
    Ok(())
}

/// The number of analog inputs which has jack sense.
pub const SHELL_ANALOG_JACK_STATE_COUNT: usize = 2;

/// Hardware state.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct ShellHwState {
    /// The state of analog jack with sense.
    pub analog_jack_states: [ShellAnalogJackState; SHELL_ANALOG_JACK_STATE_COUNT],
    /// The state of FireWire LED.
    pub firewire_led: FireWireLedState,
}

impl ShellHwState {
    pub(crate) const SIZE: usize = 28;
}

fn serialize_hw_state(state: &ShellHwState, raw: &mut [u8]) -> Result<(), String> {
    assert!(raw.len() >= ShellHwState::SIZE);

    serialize_analog_jack_state(&state.analog_jack_states[0], &mut raw[..4])?;
    serialize_analog_jack_state(&state.analog_jack_states[1], &mut raw[4..8])?;
    serialize_fw_led_state(&state.firewire_led, &mut raw[20..24])?;

    Ok(())
}

fn deserialize_hw_state(state: &mut ShellHwState, raw: &[u8]) -> Result<(), String> {
    assert!(raw.len() >= ShellHwState::SIZE);

    deserialize_analog_jack_state(&mut state.analog_jack_states[0], &raw[..4])?;
    deserialize_analog_jack_state(&mut state.analog_jack_states[1], &raw[4..8])?;
    deserialize_fw_led_state(&mut state.firewire_led, &raw[20..24])?;

    Ok(())
}

/// Parameter of monitor source.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub struct MonitorSrcParam {
    ///  ch 1 gain to mixer ch 1/2 (0xfffffc18..0x00000000, -90.0..0.00 dB)
    pub gain_to_mixer: i32,
    ///  ch 1 pan to mixer ch 1/2 (0xffffffce..0x00000032, -50.0..+50.0 dB)
    pub pan_to_mixer: i32,
    ///  ch 1 gain to send ch 1/2 (0xfffffc18..0x00000000, -90.0..0.00 dB)
    pub gain_to_send: i32,
}

impl MonitorSrcParam {
    const SIZE: usize = 12;
}

fn serialize_monitor_source_param(param: &MonitorSrcParam, raw: &mut [u8]) -> Result<(), String> {
    assert!(raw.len() >= MonitorSrcParam::SIZE);

    serialize_i32(&param.gain_to_mixer, &mut raw[..4]);
    serialize_i32(&param.pan_to_mixer, &mut raw[4..8]);
    serialize_i32(&param.gain_to_send, &mut raw[8..12]);

    Ok(())
}

fn deserialize_monitor_source_param(param: &mut MonitorSrcParam, raw: &[u8]) -> Result<(), String> {
    assert!(raw.len() >= MonitorSrcParam::SIZE);

    deserialize_i32(&mut param.gain_to_mixer, &raw[..4]);
    deserialize_i32(&mut param.pan_to_mixer, &raw[4..8]);
    deserialize_i32(&mut param.gain_to_send, &raw[8..12]);

    Ok(())
}

/// Monitor source.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub struct ShellMonitorSrcPair {
    ///  Stereo channel link for the pair.
    pub stereo_link: bool,
    /// Parameters of monitor source for left and right channels in its order.
    pub params: [MonitorSrcParam; 2],
}

impl ShellMonitorSrcPair {
    const SIZE: usize = 28;
}

fn serialize_monitor_source_pair(pair: &ShellMonitorSrcPair, raw: &mut [u8]) -> Result<(), String> {
    assert!(raw.len() >= ShellMonitorSrcPair::SIZE);

    serialize_bool(&pair.stereo_link, &mut raw[..4]);
    serialize_monitor_source_param(&pair.params[0], &mut raw[4..16])?;
    serialize_monitor_source_param(&pair.params[1], &mut raw[16..28])?;
    Ok(())
}

fn deserialize_monitor_source_pair(
    pair: &mut ShellMonitorSrcPair,
    raw: &[u8],
) -> Result<(), String> {
    assert!(raw.len() >= ShellMonitorSrcPair::SIZE);

    deserialize_bool(&mut pair.stereo_link, &raw[..4]);
    deserialize_monitor_source_param(&mut pair.params[0], &raw[4..16])?;
    deserialize_monitor_source_param(&mut pair.params[1], &raw[16..28])?;
    Ok(())
}

/// Mute state for monitor sources.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellMonitorSrcMute {
    /// For stream inputs.
    pub stream: bool,
    /// For analog inputs.
    pub analog: Vec<bool>,
    /// For digital inputs.
    pub digital: Vec<bool>,
}

/// State of mixer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellMixerState {
    /// For stream inputs.
    pub stream: ShellMonitorSrcPair,
    /// For analog inputs.
    pub analog: Vec<ShellMonitorSrcPair>,
    /// For digital inputs.
    pub digital: Vec<ShellMonitorSrcPair>,
    pub mutes: ShellMonitorSrcMute,
    /// The level of output volume.
    pub output_volume: i32,
    /// Whether to dim level of output volume
    pub output_dim_enable: bool,
    /// The level of output volume at dimmed.
    pub output_dim_volume: i32,
}

const SHELL_MIXER_MONITOR_SRC_COUNT: usize = 10;

impl ShellMixerState {
    pub(crate) const SIZE: usize = ShellMonitorSrcPair::SIZE * SHELL_MIXER_MONITOR_SRC_COUNT + 36;
}

/// The type of monitor source.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ShellMixerMonitorSrcType {
    /// Stream input.
    Stream,
    /// Analog input.
    Analog,
    /// S/PDIF input.
    Spdif,
    /// ADAT input.
    Adat,
    /// ADAT input and S/PDIF input.
    AdatSpdif,
}

/// The trait for specification of mixer.
pub trait ShellMixerStateSpecification {
    /// The sources of monitor.
    const MONITOR_SRC_MAP: [Option<ShellMixerMonitorSrcType>; SHELL_MIXER_MONITOR_SRC_COUNT];

    /// The number of analog input pairs.
    fn analog_input_pair_count() -> usize {
        Self::MONITOR_SRC_MAP
            .iter()
            .filter(|&&m| m == Some(ShellMixerMonitorSrcType::Analog))
            .count()
    }

    /// The number of digital input pairs.
    fn digital_input_pair_count() -> usize {
        Self::MONITOR_SRC_MAP
            .iter()
            .filter(|&&m| {
                m != Some(ShellMixerMonitorSrcType::Analog)
                    && m != Some(ShellMixerMonitorSrcType::Stream)
                    && m.is_some()
            })
            .count()
    }

    /// Instantiate state of mixer.
    fn create_mixer_state() -> ShellMixerState {
        let analog_input_pair_count = Self::analog_input_pair_count();
        let digital_input_pair_count = Self::digital_input_pair_count();

        ShellMixerState {
            stream: Default::default(),
            analog: vec![Default::default(); analog_input_pair_count],
            digital: vec![Default::default(); digital_input_pair_count],
            mutes: ShellMonitorSrcMute {
                stream: Default::default(),
                analog: vec![Default::default(); analog_input_pair_count * 2],
                digital: vec![Default::default(); digital_input_pair_count * 2],
            },
            output_volume: Default::default(),
            output_dim_enable: Default::default(),
            output_dim_volume: Default::default(),
        }
    }
}

fn serialize_mixer_state<T: ShellMixerStateSpecification>(
    state: &ShellMixerState,
    raw: &mut [u8],
) -> Result<(), String> {
    serialize_monitor_source_pair(&state.stream, &mut raw[..ShellMonitorSrcPair::SIZE])?;

    // For analog inputs.
    T::MONITOR_SRC_MAP
        .iter()
        .enumerate()
        .filter(|(_, &m)| m == Some(ShellMixerMonitorSrcType::Analog))
        .zip(&state.analog)
        .try_for_each(|((i, _), src)| {
            let pos = i * ShellMonitorSrcPair::SIZE;
            serialize_monitor_source_pair(src, &mut raw[pos..(pos + ShellMonitorSrcPair::SIZE)])
        })?;

    // For digital inputs.
    T::MONITOR_SRC_MAP
        .iter()
        .enumerate()
        .filter(|(_, &m)| {
            m.is_some()
                && m != Some(ShellMixerMonitorSrcType::Analog)
                && m != Some(ShellMixerMonitorSrcType::Stream)
        })
        .zip(&state.digital)
        .try_for_each(|((i, _), src)| {
            let pos = i * ShellMonitorSrcPair::SIZE;
            serialize_monitor_source_pair(src, &mut raw[pos..(pos + ShellMonitorSrcPair::SIZE)])
        })?;

    // For mixer output.
    serialize_bool(&state.output_dim_enable, &mut raw[280..284]);
    serialize_i32(&state.output_volume, &mut raw[284..288]);
    serialize_i32(&state.output_dim_volume, &mut raw[296..300]);

    // For mute of sources.
    let mut mutes = 0u32;
    if state.mutes.stream {
        mutes |= 0x00000001;
    }
    state
        .mutes
        .analog
        .iter()
        .chain(&state.mutes.digital)
        .enumerate()
        .filter(|(_, &muted)| muted)
        .for_each(|(i, _)| {
            mutes |= 1 << (8 + i);
        });
    serialize_u32(&mutes, &mut raw[308..312]);

    Ok(())
}

fn deserialize_mixer_state<T: ShellMixerStateSpecification>(
    state: &mut ShellMixerState,
    raw: &[u8],
) -> Result<(), String> {
    deserialize_monitor_source_pair(&mut state.stream, &raw[..ShellMonitorSrcPair::SIZE])?;

    // For analog inputs.
    T::MONITOR_SRC_MAP
        .iter()
        .enumerate()
        .filter(|(_, &m)| m == Some(ShellMixerMonitorSrcType::Analog))
        .zip(&mut state.analog)
        .try_for_each(|((i, _), src)| {
            let pos = i * ShellMonitorSrcPair::SIZE;
            deserialize_monitor_source_pair(src, &raw[pos..(pos + ShellMonitorSrcPair::SIZE)])
        })?;

    // For digital inputs.
    T::MONITOR_SRC_MAP
        .iter()
        .enumerate()
        .filter(|(_, &m)| m.is_some() && m != Some(ShellMixerMonitorSrcType::Analog))
        .zip(&mut state.digital)
        .try_for_each(|((i, _), src)| {
            let pos = i * ShellMonitorSrcPair::SIZE;
            deserialize_monitor_source_pair(src, &raw[pos..(pos + ShellMonitorSrcPair::SIZE)])
        })?;

    // For mixer output.
    deserialize_bool(&mut state.output_dim_enable, &raw[280..284]);
    deserialize_i32(&mut state.output_volume, &raw[284..288]);
    deserialize_i32(&mut state.output_dim_volume, &raw[296..300]);

    // For mute of sources.
    let mut mutes = 0u32;
    deserialize_u32(&mut mutes, &raw[308..312]);
    state.mutes.stream = mutes & 0x00000001 > 0;
    state
        .mutes
        .analog
        .iter_mut()
        .chain(&mut state.mutes.digital)
        .enumerate()
        .for_each(|(i, muted)| {
            *muted = mutes & (1 << (8 + i)) > 0;
        });

    Ok(())
}

/// Return configuration of reverb effect.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub struct ShellReverbReturn {
    /// Whether to use reverb effect as plugin. When enabled, return of reverb effect is delivered
    /// by rx stream.
    pub plugin_mode: bool,
    /// The gain to return reverb effect to mixer output.
    pub return_gain: i32,
    /// Whether to mute return reverb effect to mixer output.
    pub return_mute: bool,
}

impl ShellReverbReturn {
    pub(crate) const SIZE: usize = 12;
}

fn serialize_reverb_return(state: &ShellReverbReturn, raw: &mut [u8]) -> Result<(), String> {
    assert!(raw.len() >= ShellReverbReturn::SIZE);

    serialize_bool(&state.plugin_mode, &mut raw[..4]);
    serialize_i32(&state.return_gain, &mut raw[4..8]);
    serialize_bool(&state.return_mute, &mut raw[8..12]);

    Ok(())
}

fn deserialize_reverb_return(state: &mut ShellReverbReturn, raw: &[u8]) -> Result<(), String> {
    assert!(raw.len() >= ShellReverbReturn::SIZE);

    deserialize_bool(&mut state.plugin_mode, &raw[..4]);
    deserialize_i32(&mut state.return_gain, &raw[4..8]);
    deserialize_bool(&mut state.return_mute, &raw[8..12]);

    Ok(())
}

/// Meter information. -1000..0 (-94.0..0 dB).
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct ShellMixerMeter {
    /// Detected signal level of stream inputs.
    pub stream_inputs: Vec<i32>,
    /// Detected signal level of analog inputs.
    pub analog_inputs: Vec<i32>,
    /// Detected signal level of digital inputs.
    pub digital_inputs: Vec<i32>,
    /// Detected signal level of main outputs.
    pub main_outputs: Vec<i32>,
}

impl ShellMixerMeter {
    pub(crate) const SIZE: usize = 0x5c;
}

/// Specification for meter function of mixer.
pub trait ShellMixerMeterSpecification {
    const ANALOG_INPUT_COUNT: usize;
    const DIGITAL_INPUT_COUNT: usize;

    const STREAM_INPUT_COUNT: usize = 2;
    const MAIN_OUTPUT_COUNT: usize = 2;
    const MAX_STREAM_INPUT_COUNT: usize = 8;
    const MAX_ANALOG_INPUT_COUNT: usize = 4;
    const MAX_DIGITAL_INPUT_COUNT: usize = 8;

    fn create_meter_state() -> ShellMixerMeter {
        ShellMixerMeter {
            stream_inputs: vec![Default::default(); Self::STREAM_INPUT_COUNT],
            analog_inputs: vec![Default::default(); Self::ANALOG_INPUT_COUNT],
            digital_inputs: vec![Default::default(); Self::DIGITAL_INPUT_COUNT],
            main_outputs: vec![Default::default(); Self::MAIN_OUTPUT_COUNT],
        }
    }
}

fn serialize_mixer_meter<T: ShellMixerMeterSpecification>(
    state: &ShellMixerMeter,
    raw: &mut [u8],
) -> Result<(), String> {
    assert!(raw.len() >= ShellMixerMeter::SIZE);

    let mut offset = 0;
    state.stream_inputs.iter().enumerate().for_each(|(i, m)| {
        let pos = offset + i * 4;
        serialize_i32(m, &mut raw[pos..(pos + 4)]);
    });

    offset += T::MAX_STREAM_INPUT_COUNT * 4;
    state
        .analog_inputs
        .iter()
        .take(T::ANALOG_INPUT_COUNT)
        .take(T::MAX_ANALOG_INPUT_COUNT)
        .enumerate()
        .for_each(|(i, m)| {
            let pos = offset + i * 4;
            serialize_i32(m, &mut raw[pos..(pos + 4)]);
        });

    offset += T::MAX_ANALOG_INPUT_COUNT * 4;
    state
        .digital_inputs
        .iter()
        .take(T::DIGITAL_INPUT_COUNT)
        .take(T::MAX_DIGITAL_INPUT_COUNT)
        .enumerate()
        .for_each(|(i, m)| {
            let pos = offset + i * 4;
            serialize_i32(m, &mut raw[pos..(pos + 4)]);
        });

    offset += T::MAX_DIGITAL_INPUT_COUNT * 4;
    state.main_outputs.iter().enumerate().for_each(|(i, m)| {
        let pos = offset + i * 4;
        serialize_i32(m, &mut raw[pos..(pos + 4)]);
    });

    Ok(())
}

fn deserialize_mixer_meter<T: ShellMixerMeterSpecification>(
    state: &mut ShellMixerMeter,
    raw: &[u8],
) -> Result<(), String> {
    assert!(raw.len() >= ShellMixerMeter::SIZE);

    let mut offset = 0;
    state
        .stream_inputs
        .iter_mut()
        .enumerate()
        .for_each(|(i, m)| {
            let pos = offset + i * 4;
            deserialize_i32(m, &raw[pos..(pos + 4)]);
        });

    offset += T::MAX_STREAM_INPUT_COUNT * 4;
    state
        .analog_inputs
        .iter_mut()
        .take(T::ANALOG_INPUT_COUNT)
        .take(T::MAX_ANALOG_INPUT_COUNT)
        .enumerate()
        .for_each(|(i, m)| {
            let pos = offset + i * 4;
            deserialize_i32(m, &raw[pos..(pos + 4)]);
        });

    offset += T::MAX_ANALOG_INPUT_COUNT * 4;
    state
        .digital_inputs
        .iter_mut()
        .take(T::DIGITAL_INPUT_COUNT)
        .take(T::MAX_DIGITAL_INPUT_COUNT)
        .enumerate()
        .for_each(|(i, m)| {
            let pos = offset + i * 4;
            deserialize_i32(m, &raw[pos..(pos + 4)]);
        });

    offset += T::MAX_DIGITAL_INPUT_COUNT * 4;
    state
        .main_outputs
        .iter_mut()
        .enumerate()
        .for_each(|(i, m)| {
            let pos = offset + i * 4;
            deserialize_i32(m, &raw[pos..(pos + 4)]);
        });

    Ok(())
}

/// Available source for physical output.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ShellPhysOutSrc {
    /// Stream input.
    Stream,
    /// Analog input 1/2.
    Analog01,
    /// Mixer output 1/2.
    MixerOut01,
    /// Send 1/2.
    MixerSend01,
}

impl Default for ShellPhysOutSrc {
    fn default() -> Self {
        Self::Stream
    }
}

const PHYS_OUT_SRCS: &[ShellPhysOutSrc] = &[
    ShellPhysOutSrc::Stream,
    ShellPhysOutSrc::Analog01,
    ShellPhysOutSrc::MixerOut01,
    ShellPhysOutSrc::MixerSend01,
];

const PHYS_OUT_SRC_LABEL: &str = "physical output source";

fn serialize_phys_out_src(src: &ShellPhysOutSrc, raw: &mut [u8]) -> Result<(), String> {
    serialize_position(PHYS_OUT_SRCS, src, raw, PHYS_OUT_SRC_LABEL)
}

fn deserialize_phys_out_src(src: &mut ShellPhysOutSrc, raw: &[u8]) -> Result<(), String> {
    deserialize_position(PHYS_OUT_SRCS, src, raw, PHYS_OUT_SRC_LABEL)
}

/// Format of optical input interface.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ShellOptInputIfaceFormat {
    /// ADAT 1/2/3/4/5/6/7/8.
    Adat0to7,
    /// ADAT 1/2/3/4/5/6 and S/PDIF 1/2.
    Adat0to5Spdif01,
    /// S/PDIF 1/2 in both coaxial and optical interfaces.
    Toslink01Spdif01,
}

const OPT_INPUT_IFACE_FMT_LABELS: &str = "optical input format";

const OPT_INPUT_IFACE_FMTS: &[ShellOptInputIfaceFormat] = &[
    ShellOptInputIfaceFormat::Adat0to7,
    ShellOptInputIfaceFormat::Adat0to5Spdif01,
    ShellOptInputIfaceFormat::Toslink01Spdif01,
];

impl Default for ShellOptInputIfaceFormat {
    fn default() -> Self {
        ShellOptInputIfaceFormat::Adat0to7
    }
}

/// Format of optical output interface.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ShellOptOutputIfaceFormat {
    Adat,
    Spdif,
}

impl Default for ShellOptOutputIfaceFormat {
    fn default() -> Self {
        Self::Adat
    }
}

const OPT_OUTPUT_IFACE_FMT_LABELS: &str = "optical output format";

const OPT_OUTPUT_IFACE_FMTS: &[ShellOptOutputIfaceFormat] = &[
    ShellOptOutputIfaceFormat::Adat,
    ShellOptOutputIfaceFormat::Spdif,
];

/// Source for optical output interface.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct ShellOptOutputSrc(pub ShellPhysOutSrc);

const OPT_OUT_SRC_LABEL: &str = "optical output source";

/// Configuration for optical interface.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct ShellOptIfaceConfig {
    pub input_format: ShellOptInputIfaceFormat,
    pub output_format: ShellOptOutputIfaceFormat,
    pub output_source: ShellOptOutputSrc,
}

impl ShellOptIfaceConfig {
    const SIZE: usize = 12;
}

fn serialize_opt_iface_config(config: &ShellOptIfaceConfig, raw: &mut [u8]) -> Result<(), String> {
    assert!(raw.len() >= ShellOptIfaceConfig::SIZE);

    serialize_position(
        OPT_INPUT_IFACE_FMTS,
        &config.input_format,
        &mut raw[..4],
        OPT_INPUT_IFACE_FMT_LABELS,
    )?;
    serialize_position(
        OPT_OUTPUT_IFACE_FMTS,
        &config.output_format,
        &mut raw[4..8],
        OPT_OUTPUT_IFACE_FMT_LABELS,
    )?;
    serialize_position(
        PHYS_OUT_SRCS,
        &config.output_source.0,
        &mut raw[8..],
        OPT_OUT_SRC_LABEL,
    )?;
    Ok(())
}

fn deserialize_opt_iface_config(
    config: &mut ShellOptIfaceConfig,
    raw: &[u8],
) -> Result<(), String> {
    assert!(raw.len() >= ShellOptIfaceConfig::SIZE);

    deserialize_position(
        OPT_INPUT_IFACE_FMTS,
        &mut config.input_format,
        &raw[..4],
        OPT_INPUT_IFACE_FMT_LABELS,
    )?;
    deserialize_position(
        OPT_OUTPUT_IFACE_FMTS,
        &mut config.output_format,
        &raw[4..8],
        OPT_OUTPUT_IFACE_FMT_LABELS,
    )?;
    deserialize_position(
        PHYS_OUT_SRCS,
        &mut config.output_source.0,
        &raw[8..],
        OPT_OUT_SRC_LABEL,
    )?;
    Ok(())
}

/// Source of coaxial output interface.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct ShellCoaxOutPairSrc(pub ShellPhysOutSrc);

const COAX_OUT_PAIR_SRC_LABEL: &str = "coaxial output pair source";

fn serialize_coax_out_pair_source(src: &ShellCoaxOutPairSrc, raw: &mut [u8]) -> Result<(), String> {
    serialize_position(PHYS_OUT_SRCS, &src.0, raw, COAX_OUT_PAIR_SRC_LABEL)
}

fn deserialize_coax_out_pair_source(
    src: &mut ShellCoaxOutPairSrc,
    raw: &[u8],
) -> Result<(), String> {
    deserialize_position(PHYS_OUT_SRCS, &mut src.0, raw, COAX_OUT_PAIR_SRC_LABEL)
}

/// Available source for sampling clock.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ShellStandaloneClockSource {
    /// Signal from optical input interface.
    Optical,
    /// Signal from coaxial input interface.
    Coaxial,
    /// Internal oscillator.
    Internal,
}

impl Default for ShellStandaloneClockSource {
    fn default() -> Self {
        Self::Internal
    }
}

const STANDALONE_CLOCK_SOURCE_LABEL: &str = "Standalone clock source";

/// Function specification for standalone clock source.
pub trait ShellStandaloneClockSpecification {
    /// The list of available sources.
    const STANDALONE_CLOCK_SOURCES: &'static [ShellStandaloneClockSource];
}

/// Serialize for segment layout.
fn serialize_standalone_clock_source<T: ShellStandaloneClockSpecification>(
    src: &ShellStandaloneClockSource,
    raw: &mut [u8],
) -> Result<(), String> {
    serialize_position(
        T::STANDALONE_CLOCK_SOURCES,
        src,
        raw,
        STANDALONE_CLOCK_SOURCE_LABEL,
    )
}

/// Deserialize for segment layout.
fn deserialize_standalone_clock_source<T: ShellStandaloneClockSpecification>(
    src: &mut ShellStandaloneClockSource,
    raw: &[u8],
) -> Result<(), String> {
    deserialize_position(
        T::STANDALONE_CLOCK_SOURCES,
        src,
        raw,
        STANDALONE_CLOCK_SOURCE_LABEL,
    )
}

/// Stereo pair of audio data channels in isochronous packet stream available as single source of
/// mixer.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ShellMixerStreamSourcePair {
    /// 1st pair of audio data channels.
    Stream0_1,
    /// 2nd pair of audio data channels.
    Stream2_3,
    /// 3rd pair of audio data channels.
    Stream4_5,
    /// 4th pair of audio data channels.
    Stream6_7,
    /// 5th pair of audio data channels.
    Stream8_9,
    /// 6th pair of audio data channels.
    Stream10_11,
    /// 7th pair of audio data channels.
    Stream12_13,
}

impl Default for ShellMixerStreamSourcePair {
    fn default() -> Self {
        ShellMixerStreamSourcePair::Stream0_1
    }
}

const MIXER_STREAM_SOURCE_PAIR_LABEL: &str = "Mixer stream source pair";

/// Specification for source pair of stream to mixer.
pub trait ShellMixerStreamSourcePairSpecification {
    const MIXER_STREAM_SOURCE_PAIRS: &'static [ShellMixerStreamSourcePair];
}

/// Serialize for source pair.
fn serialize_mixer_stream_source_pair<T: ShellMixerStreamSourcePairSpecification>(
    pair: &ShellMixerStreamSourcePair,
    raw: &mut [u8],
) -> Result<(), String> {
    serialize_position(
        T::MIXER_STREAM_SOURCE_PAIRS,
        pair,
        raw,
        MIXER_STREAM_SOURCE_PAIR_LABEL,
    )
}

/// Deserialize for source pair.
fn deserialize_mixer_stream_source_pair<T: ShellMixerStreamSourcePairSpecification>(
    pair: &mut ShellMixerStreamSourcePair,
    raw: &[u8],
) -> Result<(), String> {
    deserialize_position(
        T::MIXER_STREAM_SOURCE_PAIRS,
        pair,
        raw,
        MIXER_STREAM_SOURCE_PAIR_LABEL,
    )
}

/// Target of 1st knob.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ShellKnob0Target {
    /// Analog input 1.
    Analog0,
    /// Analog input 2.
    Analog1,
    /// Analog input 3 and 4.
    Analog2_3,
    /// S/PDIF input 1 and 2.
    Spdif0_1,
    /// Compression ratio of channel strip effect to analog input 1.
    ChannelStrip0,
    /// Compression ratio of channel strip effect to analog input 2.
    ChannelStrip1,
    /// Reverb ratio or decay time of reverb effect.
    Reverb,
    /// Ratio to multiplex stream inputs in mixer.
    Mixer,
    /// Configured by mixer settings.
    Configurable,
}

const KNOB0_TARGET_LABEL: &str = "Knob 0 target";

/// Function specification of 1st knob.
pub trait ShellKnob0TargetSpecification {
    /// The list of targets supported for 1st knob.
    const KNOB0_TARGETS: &'static [ShellKnob0Target];
}

/// Serialize for 1st knob.
fn serialize_knob0_target<T: ShellKnob0TargetSpecification>(
    target: &ShellKnob0Target,
    raw: &mut [u8],
) -> Result<(), String> {
    serialize_position(T::KNOB0_TARGETS, target, raw, KNOB0_TARGET_LABEL)
}

/// Deserialize for 1st knob.
fn deserialize_knob0_target<T: ShellKnob0TargetSpecification>(
    target: &mut ShellKnob0Target,
    raw: &[u8],
) -> Result<(), String> {
    deserialize_position(T::KNOB0_TARGETS, target, raw, KNOB0_TARGET_LABEL)
}

/// Target of 2nd knob.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ShellKnob1Target {
    /// ADAT input 1/2 or S/PDIF input 1/2 in optical interface.
    Digital0_1,
    /// ADAT input 3/4.
    Digital2_3,
    /// ADAT input 5/6.
    Digital4_5,
    /// ADAT input 7/8 or S/PDIF input 1/2 in coaxial interface.
    Digital6_7,
    /// Stream input to mixer.
    Stream,
    /// Reverb ratio or decay time of reverb return 1/2.
    Reverb,
    /// Normal/Dim Level of mixer output 1/2.
    Mixer,
    /// Pitch or tone of tuner.
    TunerPitchTone,
    /// Generate MIDI event.
    MidiSend,
}

const KNOB1_TARGET_LABEL: &str = "Knob 1 target";

/// Function specification of 2nd knob.
pub trait ShellKnob1TargetSpecification {
    /// The list of targets supported for 2nd knob.
    const KNOB1_TARGETS: &'static [ShellKnob1Target];
}

/// Serialize for 1st knob.
fn serialize_knob1_target<T: ShellKnob1TargetSpecification>(
    target: &ShellKnob1Target,
    raw: &mut [u8],
) -> Result<(), String> {
    serialize_position(T::KNOB1_TARGETS, target, raw, KNOB1_TARGET_LABEL)
}

/// Deserialize for 1st knob.
fn deserialize_knob1_target<T: ShellKnob1TargetSpecification>(
    target: &mut ShellKnob1Target,
    raw: &[u8],
) -> Result<(), String> {
    deserialize_position(T::KNOB1_TARGETS, target, raw, KNOB1_TARGET_LABEL)
}

/// The size of segment for knob settings.
pub(crate) const SHELL_KNOB_SEGMENT_SIZE: usize = 36;
