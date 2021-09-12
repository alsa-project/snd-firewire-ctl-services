// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Protocol defined by TC Electronic for Konnekt 24d, Konnekt 8, Konnekt Live, and Impact Twin.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by TC Electronic for Konnekt 24d, Konnekt 8, Konnekt Live, and Impact Twin.

pub mod itwin;
pub mod k24d;
pub mod k8;
pub mod klive;

use super::fw_led::*;
use super::*;

use crate::*;

const SHELL_KNOB_NOTIFY_FLAG: u32 = 0x00010000;
const SHELL_CONFIG_NOTIFY_FLAG: u32 = 0x00020000;
const SHELL_MIXER_NOTIFY_FLAG: u32 = 0x00040000;
const SHELL_REVERB_NOTIFY_FLAG: u32 = 0x00080000;
const SHELL_CH_STRIP_NOTIFY_FLAG: u32 = 0x00100000;
// NOTE: 0x00200000 is for tuner.
// NOTE: 0x00400000 is unidentified.
const SHELL_HW_STATE_NOTIFY_FLAG: u32 = 0x01000000;

const SHELL_CH_STRIP_COUNT: usize = 2;

/// The enumeration to represent state of jack sense for analog input.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ShellAnalogJackState {
    FrontSelected,
    FrontInserted,
    FrontInsertedAttenuated,
    RearSelected,
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

impl From<u32> for ShellAnalogJackState {
    fn from(val: u32) -> Self {
        match val & 0xff {
            Self::FRONT_INSERTED => Self::FrontInserted,
            Self::FRONT_INSERTED_ATTENUATED => Self::FrontInsertedAttenuated,
            Self::REAR_SELECTED => Self::RearSelected,
            Self::REAR_INSERTED => Self::RearInserted,
            _ => Self::FrontSelected,
        }
    }
}

impl From<ShellAnalogJackState> for u32 {
    fn from(state: ShellAnalogJackState) -> Self {
        match state {
            ShellAnalogJackState::FrontSelected => ShellAnalogJackState::FRONT_SELECTED,
            ShellAnalogJackState::FrontInserted => ShellAnalogJackState::FRONT_INSERTED,
            ShellAnalogJackState::FrontInsertedAttenuated => {
                ShellAnalogJackState::FRONT_INSERTED_ATTENUATED
            }
            ShellAnalogJackState::RearSelected => ShellAnalogJackState::REAR_SELECTED,
            ShellAnalogJackState::RearInserted => ShellAnalogJackState::REAR_INSERTED,
        }
    }
}

/// The number of analog inputs which has jack sense.
pub const SHELL_ANALOG_JACK_STATE_COUNT: usize = 2;

/// The structure to represent hardware state.
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct ShellHwState {
    pub analog_jack_states: [ShellAnalogJackState; SHELL_ANALOG_JACK_STATE_COUNT],
    pub firewire_led: FireWireLedState,
}

impl ShellHwState {
    pub const SIZE: usize = 28;

    pub fn build(&self, raw: &mut [u8]) {
        assert_eq!(raw.len(), Self::SIZE, "Programming error...");

        self.analog_jack_states.build_quadlet_block(&mut raw[..8]);
        self.firewire_led.build_quadlet(&mut raw[20..24]);
    }

    pub fn parse(&mut self, raw: &[u8]) {
        assert_eq!(raw.len(), Self::SIZE, "Programming error...");

        self.analog_jack_states.parse_quadlet_block(&raw[..8]);
        self.firewire_led.parse_quadlet(&raw[20..24]);
    }
}

/// The structure to represent parameter of monitor source.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
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

    pub fn build(&self, raw: &mut [u8]) {
        assert_eq!(
            raw.len(),
            Self::SIZE,
            "Programming error for the length of monitor source parameter."
        );

        self.gain_to_mixer.build_quadlet(&mut raw[..4]);
        self.pan_to_mixer.build_quadlet(&mut raw[4..8]);
        self.gain_to_send.build_quadlet(&mut raw[8..12]);
    }

    pub fn parse(&mut self, raw: &[u8]) {
        assert_eq!(
            raw.len(),
            Self::SIZE,
            "Programming error for the length of monitor source parameter."
        );

        self.gain_to_mixer.parse_quadlet(&raw[..4]);
        self.pan_to_mixer.parse_quadlet(&raw[4..8]);
        self.gain_to_send.parse_quadlet(&raw[8..12]);
    }
}

/// The structure to represent monitor source.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
pub struct ShellMonitorSrcPair {
    ///  ch 1/2 stereo link (0 or 1)
    pub stereo_link: bool,
    /// Left channel.
    pub left: MonitorSrcParam,
    /// Right channel.
    pub right: MonitorSrcParam,
}

impl ShellMonitorSrcPair {
    const SIZE: usize = 28;

    pub fn build(&self, raw: &mut [u8]) {
        self.stereo_link.build_quadlet(&mut raw[..4]);
        self.left.build(&mut raw[4..16]);
        self.right.build(&mut raw[16..28]);
    }

    pub fn parse(&mut self, raw: &[u8]) {
        self.stereo_link.parse_quadlet(&raw[..4]);
        self.left.parse(&raw[4..16]);
        self.right.parse(&raw[16..28]);
    }
}

/// The structure to represent mute state for monitor sources.
#[derive(Debug)]
pub struct ShellMonitorSrcMute {
    pub stream: bool,
    pub analog: Vec<bool>,
    pub digital: Vec<bool>,
}

/// The structure to represent state of mixer.
#[derive(Debug)]
pub struct ShellMixerState {
    pub stream: ShellMonitorSrcPair,
    pub analog: Vec<ShellMonitorSrcPair>,
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
    pub const SIZE: usize = ShellMonitorSrcPair::SIZE * SHELL_MIXER_MONITOR_SRC_COUNT + 36;
}

/// The type of monitor source.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ShellMixerMonitorSrcType {
    Stream,
    Analog,
    Spdif,
    Adat,
    AdatSpdif,
}

pub trait ShellMixerStateConvert {
    const MONITOR_SRC_MAP: [Option<ShellMixerMonitorSrcType>; SHELL_MIXER_MONITOR_SRC_COUNT];

    fn state(&self) -> &ShellMixerState;
    fn state_mut(&mut self) -> &mut ShellMixerState;

    fn create_mixer_state() -> ShellMixerState {
        let analog_input_pair_count = Self::MONITOR_SRC_MAP
            .iter()
            .filter(|&&m| m == Some(ShellMixerMonitorSrcType::Analog))
            .count();
        let digital_input_pair_count = Self::MONITOR_SRC_MAP
            .iter()
            .filter(|&&m| {
                m != Some(ShellMixerMonitorSrcType::Analog)
                    && m != Some(ShellMixerMonitorSrcType::Stream)
                    && m.is_some()
            })
            .count();

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

    fn build(&self, raw: &mut [u8]) {
        let state = self.state();

        state.stream.build(&mut raw[..ShellMonitorSrcPair::SIZE]);

        // For analog inputs.
        Self::MONITOR_SRC_MAP
            .iter()
            .enumerate()
            .filter(|(_, &m)| m == Some(ShellMixerMonitorSrcType::Analog))
            .zip(state.analog.iter())
            .for_each(|((i, _), src)| {
                let pos = i * ShellMonitorSrcPair::SIZE;
                src.build(&mut raw[pos..(pos + ShellMonitorSrcPair::SIZE)]);
            });

        // For digital inputs.
        Self::MONITOR_SRC_MAP
            .iter()
            .enumerate()
            .filter(|(_, &m)| {
                m.is_some()
                    && m != Some(ShellMixerMonitorSrcType::Analog)
                    && m != Some(ShellMixerMonitorSrcType::Stream)
            })
            .zip(state.digital.iter())
            .for_each(|((i, _), src)| {
                let pos = i * ShellMonitorSrcPair::SIZE;
                src.build(&mut raw[pos..(pos + ShellMonitorSrcPair::SIZE)]);
            });

        // For mixer output.
        state.output_dim_enable.build_quadlet(&mut raw[280..284]);
        state.output_volume.build_quadlet(&mut raw[284..288]);
        state.output_dim_volume.build_quadlet(&mut raw[296..300]);

        // For mute of sources.
        let mut mutes = 0u32;
        if state.mutes.stream {
            mutes |= 0x00000001;
        }
        state
            .mutes
            .analog
            .iter()
            .chain(state.mutes.digital.iter())
            .enumerate()
            .filter(|(_, &muted)| muted)
            .for_each(|(i, _)| {
                mutes |= 1 << (8 + i);
            });
        mutes.build_quadlet(&mut raw[308..312]);
    }

    fn parse(&mut self, raw: &[u8]) {
        let state = self.state_mut();

        state.stream.parse(&raw[..ShellMonitorSrcPair::SIZE]);

        // For analog inputs.
        Self::MONITOR_SRC_MAP
            .iter()
            .enumerate()
            .filter(|(_, &m)| m == Some(ShellMixerMonitorSrcType::Analog))
            .zip(state.analog.iter_mut())
            .for_each(|((i, _), src)| {
                let pos = i * ShellMonitorSrcPair::SIZE;
                src.parse(&raw[pos..(pos + ShellMonitorSrcPair::SIZE)]);
            });

        // For digital inputs.
        Self::MONITOR_SRC_MAP
            .iter()
            .enumerate()
            .filter(|(_, &m)| m.is_some() && m != Some(ShellMixerMonitorSrcType::Analog))
            .zip(state.digital.iter_mut())
            .for_each(|((i, _), src)| {
                let pos = i * ShellMonitorSrcPair::SIZE;
                src.parse(&raw[pos..(pos + ShellMonitorSrcPair::SIZE)]);
            });

        // For mixer output.
        state.output_dim_enable.parse_quadlet(&raw[280..284]);
        state.output_volume.parse_quadlet(&raw[284..288]);
        state.output_dim_volume.parse_quadlet(&raw[296..300]);

        // For mute of sources.
        let mut mutes = 0u32;
        mutes.parse_quadlet(&raw[308..312]);
        state.mutes.stream = mutes & 0x00000001 > 0;
        state
            .mutes
            .analog
            .iter_mut()
            .chain(state.mutes.digital.iter_mut())
            .enumerate()
            .for_each(|(i, muted)| {
                *muted = mutes & (1 << (8 + i)) > 0;
            });
    }
}

/// The structure to represent return configuration of reverb effect.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
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
    pub const SIZE: usize = 12;

    pub fn build(&self, raw: &mut [u8]) {
        assert_eq!(raw.len(), Self::SIZE, "Programming error");

        self.plugin_mode.build_quadlet(&mut raw[..4]);
        self.return_gain.build_quadlet(&mut raw[4..8]);
        self.return_mute.build_quadlet(&mut raw[8..12]);
    }

    pub fn parse(&mut self, raw: &[u8]) {
        assert_eq!(raw.len(), Self::SIZE, "Programming error");

        self.plugin_mode.parse_quadlet(&raw[..4]);
        self.return_gain.parse_quadlet(&raw[4..8]);
        self.return_mute.parse_quadlet(&raw[8..12]);
    }
}

/// The structure to represent meter information. -1000..0 (-94.0..0 dB).
#[derive(Default, Debug)]
pub struct ShellMixerMeter {
    pub stream_inputs: [i32; Self::STREAM_INPUT_COUNT],
    pub analog_inputs: Vec<i32>,
    pub digital_inputs: Vec<i32>,
    pub main_outputs: [i32; Self::MAIN_OUTPUT_COUNT],
}

impl ShellMixerMeter {
    pub const SIZE: usize = 0x5c;

    const STREAM_INPUT_COUNT: usize = 2;
    const MAIN_OUTPUT_COUNT: usize = 2;
    const MAX_STREAM_INPUT_COUNT: usize = 8;
    const MAX_ANALOG_INPUT_COUNT: usize = 4;
    const MAX_DIGITAL_INPUT_COUNT: usize = 8;
}

pub trait ShellMixerMeterConvert {
    const ANALOG_INPUT_COUNT: usize;
    const DIGITAL_INPUT_COUNT: usize;

    fn meter(&self) -> &ShellMixerMeter;
    fn meter_mut(&mut self) -> &mut ShellMixerMeter;

    fn create_meter_state() -> ShellMixerMeter {
        ShellMixerMeter {
            stream_inputs: [Default::default(); ShellMixerMeter::STREAM_INPUT_COUNT],
            analog_inputs: vec![Default::default(); Self::ANALOG_INPUT_COUNT],
            digital_inputs: vec![Default::default(); Self::DIGITAL_INPUT_COUNT],
            main_outputs: [Default::default(); ShellMixerMeter::MAIN_OUTPUT_COUNT],
        }
    }

    fn build(&self, raw: &mut [u8]) {
        assert_eq!(raw.len(), ShellMixerMeter::SIZE, "Programming error...");

        let state = self.meter();

        let mut offset = 0;
        state.stream_inputs.iter().enumerate().for_each(|(i, m)| {
            let pos = offset + i * 4;
            m.build_quadlet(&mut raw[pos..(pos + 4)]);
        });

        offset += ShellMixerMeter::MAX_STREAM_INPUT_COUNT * 4;
        state
            .analog_inputs
            .iter()
            .take(Self::ANALOG_INPUT_COUNT)
            .take(ShellMixerMeter::MAX_ANALOG_INPUT_COUNT)
            .enumerate()
            .for_each(|(i, m)| {
                let pos = offset + i * 4;
                m.build_quadlet(&mut raw[pos..(pos + 4)]);
            });

        offset += ShellMixerMeter::MAX_ANALOG_INPUT_COUNT * 4;
        state
            .digital_inputs
            .iter()
            .take(Self::DIGITAL_INPUT_COUNT)
            .take(ShellMixerMeter::MAX_DIGITAL_INPUT_COUNT)
            .enumerate()
            .for_each(|(i, m)| {
                let pos = offset + i * 4;
                m.build_quadlet(&mut raw[pos..(pos + 4)]);
            });

        offset += ShellMixerMeter::MAX_DIGITAL_INPUT_COUNT * 4;
        state.main_outputs.iter().enumerate().for_each(|(i, m)| {
            let pos = offset + i * 4;
            m.build_quadlet(&mut raw[pos..(pos + 4)]);
        });
    }

    fn parse(&mut self, raw: &[u8]) {
        assert_eq!(raw.len(), ShellMixerMeter::SIZE, "Programming error...");

        let state = self.meter_mut();

        let mut offset = 0;
        state
            .stream_inputs
            .iter_mut()
            .enumerate()
            .for_each(|(i, m)| {
                let pos = offset + i * 4;
                m.parse_quadlet(&raw[pos..(pos + 4)]);
            });

        offset += ShellMixerMeter::MAX_STREAM_INPUT_COUNT * 4;
        state
            .analog_inputs
            .iter_mut()
            .take(Self::ANALOG_INPUT_COUNT)
            .take(ShellMixerMeter::MAX_ANALOG_INPUT_COUNT)
            .enumerate()
            .for_each(|(i, m)| {
                let pos = offset + i * 4;
                m.parse_quadlet(&raw[pos..(pos + 4)]);
            });

        offset += ShellMixerMeter::MAX_ANALOG_INPUT_COUNT * 4;
        state
            .digital_inputs
            .iter_mut()
            .take(Self::DIGITAL_INPUT_COUNT)
            .take(ShellMixerMeter::MAX_DIGITAL_INPUT_COUNT)
            .enumerate()
            .for_each(|(i, m)| {
                let pos = offset + i * 4;
                m.parse_quadlet(&raw[pos..(pos + 4)]);
            });

        offset += ShellMixerMeter::MAX_DIGITAL_INPUT_COUNT * 4;
        state
            .main_outputs
            .iter_mut()
            .enumerate()
            .for_each(|(i, m)| {
                let pos = offset + i * 4;
                m.parse_quadlet(&raw[pos..(pos + 4)]);
            });
    }
}

/// The enumeration to represent available source for sampling clock.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ShellPhysOutSrc {
    Stream,
    Analog01,
    MixerOut01,
    MixerSend01,
}

impl Default for ShellPhysOutSrc {
    fn default() -> Self {
        Self::Stream
    }
}

impl From<u32> for ShellPhysOutSrc {
    fn from(val: u32) -> Self {
        match val {
            3 => Self::MixerSend01,
            2 => Self::MixerOut01,
            1 => Self::Analog01,
            _ => Self::Stream,
        }
    }
}

impl From<ShellPhysOutSrc> for u32 {
    fn from(src: ShellPhysOutSrc) -> Self {
        match src {
            ShellPhysOutSrc::MixerSend01 => 3,
            ShellPhysOutSrc::MixerOut01 => 2,
            ShellPhysOutSrc::Analog01 => 1,
            ShellPhysOutSrc::Stream => 0,
        }
    }
}

/// The enumeration to represent format of optical input interface.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ShellOptInputIfaceFormat {
    Adat0to7,
    Adat0to5Spdif01,
    Toslink01Spdif01,
}

impl Default for ShellOptInputIfaceFormat {
    fn default() -> Self {
        ShellOptInputIfaceFormat::Adat0to7
    }
}

impl From<u32> for ShellOptInputIfaceFormat {
    fn from(val: u32) -> Self {
        match val {
            2 => Self::Toslink01Spdif01,
            1 => Self::Adat0to5Spdif01,
            _ => Self::Adat0to7,
        }
    }
}

impl From<ShellOptInputIfaceFormat> for u32 {
    fn from(fmt: ShellOptInputIfaceFormat) -> Self {
        match fmt {
            ShellOptInputIfaceFormat::Toslink01Spdif01 => 2,
            ShellOptInputIfaceFormat::Adat0to5Spdif01 => 1,
            ShellOptInputIfaceFormat::Adat0to7 => 0,
        }
    }
}

/// The enumeration to represent format of optical output interface.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ShellOptOutputIfaceFormat {
    Adat,
    Spdif,
}

impl Default for ShellOptOutputIfaceFormat {
    fn default() -> Self {
        Self::Adat
    }
}

impl From<u32> for ShellOptOutputIfaceFormat {
    fn from(val: u32) -> Self {
        match val {
            1 => Self::Spdif,
            _ => Self::Adat,
        }
    }
}

impl From<ShellOptOutputIfaceFormat> for u32 {
    fn from(fmt: ShellOptOutputIfaceFormat) -> Self {
        match fmt {
            ShellOptOutputIfaceFormat::Spdif => 1,
            ShellOptOutputIfaceFormat::Adat => 0,
        }
    }
}

/// The enumeration to represent source for optical output interface.
#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
pub struct ShellOptOutputSrc(pub ShellPhysOutSrc);

/// The structure to represent configuration for optical interface.
#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
pub struct ShellOptIfaceConfig {
    pub input_format: ShellOptInputIfaceFormat,
    pub output_format: ShellOptOutputIfaceFormat,
    pub output_source: ShellOptOutputSrc,
}

impl ShellOptIfaceConfig {
    const SIZE: usize = 12;

    pub fn build(&self, raw: &mut [u8]) {
        assert_eq!(raw.len(), Self::SIZE, "Programming error...");

        self.input_format.build_quadlet(&mut raw[..4]);
        self.output_format.build_quadlet(&mut raw[4..8]);
        self.output_source.0.build_quadlet(&mut raw[8..]);
    }

    pub fn parse(&mut self, raw: &[u8]) {
        assert_eq!(raw.len(), Self::SIZE, "Programming error...");

        self.input_format.parse_quadlet(&raw[..4]);
        self.output_format.parse_quadlet(&raw[4..8]);
        self.output_source.0.parse_quadlet(&raw[8..]);
    }
}

/// The structure to represent source of coaxial output interface.
#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
pub struct ShellCoaxOutPairSrc(pub ShellPhysOutSrc);

/// The enumeration to represent available source for sampling clock.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ShellStandaloneClkSrc {
    Optical,
    Coaxial,
    Internal,
}

impl Default for ShellStandaloneClkSrc {
    fn default() -> Self {
        Self::Internal
    }
}

impl From<ShellStandaloneClkSrc> for u32 {
    fn from(src: ShellStandaloneClkSrc) -> Self {
        match src {
            ShellStandaloneClkSrc::Optical => 0,
            ShellStandaloneClkSrc::Coaxial => 1,
            ShellStandaloneClkSrc::Internal => 2,
        }
    }
}

impl From<u32> for ShellStandaloneClkSrc {
    fn from(val: u32) -> Self {
        match val {
            2 => ShellStandaloneClkSrc::Internal,
            1 => ShellStandaloneClkSrc::Coaxial,
            _ => ShellStandaloneClkSrc::Optical,
        }
    }
}

pub trait ShellStandaloneClkSpec: TcKonnektSegmentData {
    const STANDALONE_CLOCK_SOURCES: &'static [ShellStandaloneClkSrc];
}

/// The structure to represent source pair of stream to mixer.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ShellMixerStreamSrcPair {
    Stream01,
    Stream23,
    Stream45,
    Stream67,
    Stream89,
    Stream1011,
    Stream1213,
}

impl Default for ShellMixerStreamSrcPair {
    fn default() -> Self {
        ShellMixerStreamSrcPair::Stream01
    }
}

impl From<u32> for ShellMixerStreamSrcPair {
    fn from(val: u32) -> Self {
        match val {
            6 => Self::Stream1213,
            5 => Self::Stream1011,
            4 => Self::Stream89,
            3 => Self::Stream67,
            2 => Self::Stream45,
            1 => Self::Stream23,
            _ => Self::Stream01,
        }
    }
}

impl From<ShellMixerStreamSrcPair> for u32 {
    fn from(pair: ShellMixerStreamSrcPair) -> Self {
        match pair {
            ShellMixerStreamSrcPair::Stream01 => 0,
            ShellMixerStreamSrcPair::Stream23 => 1,
            ShellMixerStreamSrcPair::Stream45 => 2,
            ShellMixerStreamSrcPair::Stream67 => 3,
            ShellMixerStreamSrcPair::Stream89 => 4,
            ShellMixerStreamSrcPair::Stream1011 => 5,
            ShellMixerStreamSrcPair::Stream1213 => 6,
        }
    }
}

/// The trait to represent specification for source pair of stream to mixer.
pub trait ShellMixerStreamSrcPairSpec {
    const MAXIMUM_STREAM_SRC_PAIR_COUNT: usize;
}

/// The wrapper to represent target of knob.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
pub struct ShellKnobTarget(pub u32);

/// The trait to represent maximum value of knob 2.
pub trait ShellKnobTargetSpec {
    const HAS_SPDIF: bool;
    const HAS_EFFECTS: bool;
}

/// The wrapper to represent target of knob 2.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
pub struct ShellKnob2Target(pub u32);

/// The trait to represent maximum value of knob 2.
pub trait ShellKnob2TargetSpec {
    const KNOB2_TARGET_COUNT: usize;
}

pub const SHELL_KNOB_SIZE: usize = 36;
