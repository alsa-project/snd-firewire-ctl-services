// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Protocol defined by TC Electronic for Impact Twin.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by TC Electronic for Impact Twin.
//!
//! ## Diagram of internal signal flow
//!
//! ```text
//!
//!                              ++=========++
//!                              ||         ||
//! XLR input 1 ----or---------> ||         ||
//! Phone input 1 --+            || channel ||
//!                              ||  strip  || --> analog-intput-1/2
//!                              || effects ||
//! XLR input 2 ----or---------> ||   1/2   ||
//! Phone input 2 --+            ||         ||
//!                              ++=========++
//! Phone input 3/4 -----------------------------> analog-input-3/4
//! S/PDIF input 1/2 ----------------------------> coaxial-input-1/2
//! ADAT input 1..8 or S/PDIF input 1/2 ---------> optical-input-1..8
//!
//!
//! stream-input-1/2 ----------- (one of them) --> stream-source-1/2
//! stream-input-3/4 -----------------+
//! stream-input-5/6 -----------------+
//! stream-input-7/8 -----------------+
//! stream-input-9/10 ----------------+
//! stream-input-11/12 ---------------+
//! stream-input-13/14 ---------------+
//!
//!                              ++=========++
//! analog-input-1/2 ----------> ||         ||
//! analog-input-3/4 ----------> ||         ||
//! coaxial-input-1/2 ---------> || 18 x 2  ||
//! optical-input-1/2 ---------> ||         || --> reverb-effect-output-1/2
//! optical-input-3/4 ---------> || reverb  ||
//! optical-input-5/6 ---------> || effect  ||
//! optical-input-7/8 ---------> ||         ||
//! reverb-effect-output-1/2 --> ||         ||
//! stream-source-1/2 ---------> ||         ||
//!                              ++=========++
//!
//!                              ++=========++
//! analog-input-1/2 ----------> ||         ||
//! analog-input-3/4 ----------> ||         ||
//! coaxial-input-1/2 ---------> || 18 x 2  ||
//! optical-input-1/2 ---------> ||         || --> mixer-output-1/2
//! optical-input-3/4 ---------> ||         ||
//! optical-input-5/6 ---------> ||  mixer  ||
//! optical-input-7/8 ---------> ||         ||
//! reverb-effect-output-1/2 --> ||         ||
//! stream-source-1/2 ---------> ||         ||
//!                              ++=========++
//!
//!                              ++=========++
//! analog-input-1/2 ----------> ||         ||
//! analog-input-3/4 ----------> ||         ||
//! coaxial-input-1/2 ---------> ||         ||
//! optical-input-1/2 ---------> ||         ||
//! optical-input-3/4 ---------> ||         || --> analog-output-1/2
//! optical-input-5/6 ---------> ||         || --> analog-output-3/4
//! optical-input-7/8 ---------> || 32 x 14 || --> coaxial-output-1/2
//! stream-input-1/2 ----------> ||         || --> optical-output-1/2
//! stream-input-3/4 ----------> || router  || --> optical-output-3/4
//! stream-input-5/6 ----------> ||         || --> optical-output-5/6
//! stream-input-7/8 ----------> ||         || --> optical-output-7/8
//! stream-input-9/10 ---------> ||         ||
//! stream-input-11/12 --------> ||         ||
//! stream-input-13/14 --------> ||         ||
//! mixer-output-1/2 ----------> ||         ||
//! reverb-effect-source-1/2 --> ||         ||
//!                              ++=========++
//!
//!
//! ```

use super::*;

/// Protocol implementation of Impact Twin.
#[derive(Default, Debug)]
pub struct ItwinProtocol;

impl TcatOperation for ItwinProtocol {}

impl TcatGlobalSectionSpecification for ItwinProtocol {}

/// Segment for knob. 0x0000..0x0027 (36 quads).
pub type ItwinKnobSegment = TcKonnektSegment<ItwinKnob>;

/// Segment for configuration. 0x0028..0x00cf (168 quads).
pub type ItwinConfigSegment = TcKonnektSegment<ItwinConfig>;

/// Segment for state of mixer. 0x00d0..0x0243 (93 quads).
pub type ItwinMixerStateSegment = TcKonnektSegment<ItwinMixerState>;

/// Segment for state of reverb effect. 0x0244..0x0287. (17 quads)
pub type ItwinReverbStateSegment = TcKonnektSegment<ItwinReverbState>;

/// Segment for states of channel strip effect. 0x028c..0x03ab (72 quads).
pub type ItwinChStripStatesSegment = TcKonnektSegment<ItwinChStripStates>;

// NOTE: Segment for tuner. 0x03ac..0x03c8 (8 quads).

/// Segment for mixer meter. 0x106c..0x10c7 (23 quads).
pub type ItwinMixerMeterSegment = TcKonnektSegment<ItwinMixerMeter>;

/// Segment for state of hardware. 0x1008..0x1023 (7 quads).
pub type ItwinHwStateSegment = TcKonnektSegment<ItwinHwState>;

/// Segment for meter of reverb effect. 0x10c8..0x010df (6 quads).
pub type ItwinReverbMeterSegment = TcKonnektSegment<ItwinReverbMeter>;

/// Segment for meters of channel strip effect. 0x10e0..0x111b (15 quads).
pub type ItwinChStripMetersSegment = TcKonnektSegment<ItwinChStripMeters>;

macro_rules! segment_default {
    ($p:ty, $t:ty) => {
        impl Default for TcKonnektSegment<$t> {
            fn default() -> Self {
                Self {
                    data: <$t>::default(),
                    raw: vec![0; <$p as TcKonnektSegmentSerdes<$t>>::SIZE],
                }
            }
        }
    };
}

segment_default!(ItwinProtocol, ItwinKnob);
segment_default!(ItwinProtocol, ItwinConfig);
segment_default!(ItwinProtocol, ItwinMixerState);
segment_default!(ItwinProtocol, ItwinReverbState);
segment_default!(ItwinProtocol, ItwinChStripStates);
segment_default!(ItwinProtocol, ItwinMixerMeter);
segment_default!(ItwinProtocol, ItwinHwState);
segment_default!(ItwinProtocol, ItwinReverbMeter);
segment_default!(ItwinProtocol, ItwinChStripMeters);

/// State of knob.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ItwinKnob {
    /// Target of 1st knob.
    pub target: ShellKnob0Target,
    /// Whether to recover sampling clock from any source jitter.
    pub clock_recovery: bool,
}

impl Default for ItwinKnob {
    fn default() -> Self {
        Self {
            target: <ItwinProtocol as ShellKnob0TargetSpecification>::KNOB0_TARGETS[0],
            clock_recovery: Default::default(),
        }
    }
}

impl ShellKnob0TargetSpecification for ItwinProtocol {
    const KNOB0_TARGETS: &'static [ShellKnob0Target] = &[
        ShellKnob0Target::ChannelStrip0,
        ShellKnob0Target::ChannelStrip1,
        ShellKnob0Target::Reverb,
        ShellKnob0Target::Mixer,
    ];
}

impl TcKonnektSegmentSerdes<ItwinKnob> for ItwinProtocol {
    const NAME: &'static str = "knob";
    const OFFSET: usize = 0x0004;
    const SIZE: usize = SHELL_KNOB_SEGMENT_SIZE;

    fn serialize(params: &ItwinKnob, raw: &mut [u8]) -> Result<(), String> {
        serialize_knob0_target::<ItwinProtocol>(&params.target, &mut raw[..4])?;
        serialize_bool(&params.clock_recovery, &mut raw[8..12]);
        Ok(())
    }

    fn deserialize(params: &mut ItwinKnob, raw: &[u8]) -> Result<(), String> {
        deserialize_knob0_target::<ItwinProtocol>(&mut params.target, &raw[..4])?;
        deserialize_bool(&mut params.clock_recovery, &raw[8..12]);
        Ok(())
    }
}

impl TcKonnektMutableSegmentOperation<ItwinKnob> for ItwinProtocol {}

impl TcKonnektNotifiedSegmentOperation<ItwinKnob> for ItwinProtocol {
    const NOTIFY_FLAG: u32 = SHELL_KNOB_NOTIFY_FLAG;
}

/// The number of pair for physical output.
pub const ITWIN_PHYS_OUT_PAIR_COUNT: usize = 7;

/// Source of stream for mixer.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ItwinOutputPairSrc {
    /// Mixer output 1/2.
    MixerOut01,
    /// Analog input 1/2.
    Analog01,
    /// Analog input 3/4.
    Analog23,
    /// S/PDIF input 1/2.
    Spdif01,
    /// ADAT input 1/2.
    Adat01,
    /// ADAT input 3/4.
    Adat23,
    /// ADAT input 5/6.
    Adat45,
    /// ADAT input 7/8.
    Adat67,
    /// Stream input 1/2.
    Stream01,
    /// Stream input 3/4.
    Stream23,
    /// Stream input 5/6.
    Stream45,
    /// Stream input 7/8.
    Stream67,
    /// Stream input 9/10.
    Stream89,
    /// Stream input 11/12.
    Stream1011,
    /// Stream input 13/14.
    Stream1213,
    /// Send source 1/2.
    MixerSend01,
}

impl Default for ItwinOutputPairSrc {
    fn default() -> Self {
        Self::MixerOut01
    }
}

const OUTPUT_PAIR_SOURCES: &[ItwinOutputPairSrc] = &[
    ItwinOutputPairSrc::MixerOut01,
    ItwinOutputPairSrc::Analog01,
    ItwinOutputPairSrc::Analog23,
    ItwinOutputPairSrc::Spdif01,
    ItwinOutputPairSrc::Adat01,
    ItwinOutputPairSrc::Adat23,
    ItwinOutputPairSrc::Adat45,
    ItwinOutputPairSrc::Adat67,
    ItwinOutputPairSrc::Stream01,
    ItwinOutputPairSrc::Stream23,
    ItwinOutputPairSrc::Stream45,
    ItwinOutputPairSrc::Stream67,
    ItwinOutputPairSrc::Stream89,
    ItwinOutputPairSrc::Stream1011,
    ItwinOutputPairSrc::Stream1213,
    ItwinOutputPairSrc::MixerSend01,
];

const OUTPUT_PAIR_SOURCE_LABEL: &str = "output pair source";

fn serialize_output_pair_src(src: &ItwinOutputPairSrc, raw: &mut [u8]) -> Result<(), String> {
    serialize_position(OUTPUT_PAIR_SOURCES, src, raw, OUTPUT_PAIR_SOURCE_LABEL)
}

fn deserialize_output_pair_src(src: &mut ItwinOutputPairSrc, raw: &[u8]) -> Result<(), String> {
    deserialize_position(OUTPUT_PAIR_SOURCES, src, raw, OUTPUT_PAIR_SOURCE_LABEL)
}

/// Configuration.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct ItwinConfig {
    /// Pair of stream source as mixer input.
    pub mixer_stream_src_pair: ShellMixerStreamSourcePair,
    /// Source of sampling clock at standalone mode.
    pub standalone_src: ShellStandaloneClockSource,
    /// Rate of sampling clock at standalone mode.
    pub standalone_rate: TcKonnektStandaloneClockRate,
    /// Pair of source for any type of physical output.
    pub output_pair_src: [ItwinOutputPairSrc; ITWIN_PHYS_OUT_PAIR_COUNT],
}

impl ShellMixerStreamSourcePairSpecification for ItwinProtocol {
    const MIXER_STREAM_SOURCE_PAIRS: &'static [ShellMixerStreamSourcePair] = &[
        ShellMixerStreamSourcePair::Stream0_1,
        ShellMixerStreamSourcePair::Stream2_3,
        ShellMixerStreamSourcePair::Stream4_5,
        ShellMixerStreamSourcePair::Stream6_7,
        ShellMixerStreamSourcePair::Stream8_9,
        ShellMixerStreamSourcePair::Stream10_11,
        ShellMixerStreamSourcePair::Stream12_13,
    ];
}

impl ShellStandaloneClockSpecification for ItwinProtocol {
    const STANDALONE_CLOCK_SOURCES: &'static [ShellStandaloneClockSource] = &[
        ShellStandaloneClockSource::Optical,
        ShellStandaloneClockSource::Coaxial,
        ShellStandaloneClockSource::Internal,
    ];
}

impl TcKonnektSegmentSerdes<ItwinConfig> for ItwinProtocol {
    const NAME: &'static str = "configuration";
    const OFFSET: usize = 0x0028;
    const SIZE: usize = 168;

    fn serialize(params: &ItwinConfig, raw: &mut [u8]) -> Result<(), String> {
        serialize_mixer_stream_source_pair::<ItwinProtocol>(
            &params.mixer_stream_src_pair,
            &mut raw[24..28],
        )?;
        serialize_standalone_clock_source::<ItwinProtocol>(
            &params.standalone_src,
            &mut raw[28..32],
        )?;
        serialize_standalone_clock_rate(&params.standalone_rate, &mut raw[32..36])?;
        params
            .output_pair_src
            .iter()
            .enumerate()
            .try_for_each(|(i, src)| {
                let pos = 120 + i * 4;
                serialize_output_pair_src(src, &mut raw[pos..(pos + 4)])
            })?;
        Ok(())
    }

    fn deserialize(params: &mut ItwinConfig, raw: &[u8]) -> Result<(), String> {
        deserialize_mixer_stream_source_pair::<ItwinProtocol>(
            &mut params.mixer_stream_src_pair,
            &raw[24..28],
        )?;
        deserialize_standalone_clock_source::<ItwinProtocol>(
            &mut params.standalone_src,
            &raw[28..32],
        )?;
        deserialize_standalone_clock_rate(&mut params.standalone_rate, &raw[32..36])?;
        params
            .output_pair_src
            .iter_mut()
            .enumerate()
            .try_for_each(|(i, src)| {
                let pos = 120 + i * 4;
                deserialize_output_pair_src(src, &raw[pos..(pos + 4)])
            })?;
        Ok(())
    }
}

impl TcKonnektMutableSegmentOperation<ItwinConfig> for ItwinProtocol {}

impl TcKonnektNotifiedSegmentOperation<ItwinConfig> for ItwinProtocol {
    const NOTIFY_FLAG: u32 = SHELL_CONFIG_NOTIFY_FLAG;
}

impl AsRef<TcKonnektStandaloneClockRate> for ItwinConfig {
    fn as_ref(&self) -> &TcKonnektStandaloneClockRate {
        &self.standalone_rate
    }
}

impl AsMut<TcKonnektStandaloneClockRate> for ItwinConfig {
    fn as_mut(&mut self) -> &mut TcKonnektStandaloneClockRate {
        &mut self.standalone_rate
    }
}

/// State of mixer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItwinMixerState {
    /// Configuration of internal mixer.
    pub mixer: ShellMixerState,
    /// The balance between analog and stream inputs to mix. 0..1000.
    pub stream_mix_balance: u32,
    /// Whether to enable mixer or not.
    pub enabled: bool,
}

impl Default for ItwinMixerState {
    fn default() -> Self {
        ItwinMixerState {
            mixer: ItwinProtocol::create_mixer_state(),
            enabled: Default::default(),
            stream_mix_balance: Default::default(),
        }
    }
}

impl ShellMixerStateSpecification for ItwinProtocol {
    const MONITOR_SRC_MAP: [Option<ShellMixerMonitorSrcType>; SHELL_MIXER_MONITOR_SRC_COUNT] = [
        Some(ShellMixerMonitorSrcType::Stream),
        None,
        None,
        Some(ShellMixerMonitorSrcType::Spdif),
        Some(ShellMixerMonitorSrcType::Analog),
        Some(ShellMixerMonitorSrcType::Analog),
        Some(ShellMixerMonitorSrcType::AdatSpdif),
        Some(ShellMixerMonitorSrcType::Adat),
        Some(ShellMixerMonitorSrcType::Adat),
        Some(ShellMixerMonitorSrcType::Adat),
    ];
}

impl TcKonnektSegmentSerdes<ItwinMixerState> for ItwinProtocol {
    const NAME: &'static str = "mixer-state";
    const OFFSET: usize = 0x00d0;
    const SIZE: usize = ShellMixerState::SIZE + 56;

    fn serialize(params: &ItwinMixerState, raw: &mut [u8]) -> Result<(), String> {
        serialize_mixer_state::<ItwinProtocol>(&params.mixer, raw)?;

        serialize_u32(&params.stream_mix_balance, &mut raw[348..352]);
        serialize_bool(&params.enabled, &mut raw[352..356]);
        Ok(())
    }

    fn deserialize(params: &mut ItwinMixerState, raw: &[u8]) -> Result<(), String> {
        deserialize_mixer_state::<ItwinProtocol>(&mut params.mixer, raw)?;

        deserialize_u32(&mut params.stream_mix_balance, &raw[348..352]);
        deserialize_bool(&mut params.enabled, &raw[352..356]);
        Ok(())
    }
}

impl TcKonnektMutableSegmentOperation<ItwinMixerState> for ItwinProtocol {}

impl TcKonnektNotifiedSegmentOperation<ItwinMixerState> for ItwinProtocol {
    const NOTIFY_FLAG: u32 = SHELL_MIXER_NOTIFY_FLAG;
}

impl AsRef<ShellMixerState> for ItwinMixerState {
    fn as_ref(&self) -> &ShellMixerState {
        &self.mixer
    }
}

impl AsMut<ShellMixerState> for ItwinMixerState {
    fn as_mut(&mut self) -> &mut ShellMixerState {
        &mut self.mixer
    }
}

/// Configuration for reverb effect.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct ItwinReverbState(pub ReverbState);

impl TcKonnektSegmentSerdes<ItwinReverbState> for ItwinProtocol {
    const NAME: &'static str = "reverb-state";
    const OFFSET: usize = 0x0244;
    const SIZE: usize = ReverbState::SIZE;

    fn serialize(params: &ItwinReverbState, raw: &mut [u8]) -> Result<(), String> {
        serialize_reverb_state(&params.0, raw)
    }

    fn deserialize(params: &mut ItwinReverbState, raw: &[u8]) -> Result<(), String> {
        deserialize_reverb_state(&mut params.0, raw)
    }
}

impl TcKonnektMutableSegmentOperation<ItwinReverbState> for ItwinProtocol {}

impl TcKonnektNotifiedSegmentOperation<ItwinReverbState> for ItwinProtocol {
    const NOTIFY_FLAG: u32 = SHELL_REVERB_NOTIFY_FLAG;
}

impl AsRef<ReverbState> for ItwinReverbState {
    fn as_ref(&self) -> &ReverbState {
        &self.0
    }
}

impl AsMut<ReverbState> for ItwinReverbState {
    fn as_mut(&mut self) -> &mut ReverbState {
        &mut self.0
    }
}

/// Configuration for channel strip effect.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct ItwinChStripStates(pub [ChStripState; SHELL_CH_STRIP_COUNT]);

impl TcKonnektSegmentSerdes<ItwinChStripStates> for ItwinProtocol {
    const NAME: &'static str = "channel-strip-state";
    const OFFSET: usize = 0x028c;
    const SIZE: usize = ChStripState::SIZE * SHELL_CH_STRIP_COUNT + 4;

    fn serialize(params: &ItwinChStripStates, raw: &mut [u8]) -> Result<(), String> {
        serialize_ch_strip_states(&params.0, raw)
    }

    fn deserialize(params: &mut ItwinChStripStates, raw: &[u8]) -> Result<(), String> {
        deserialize_ch_strip_states(&mut params.0, raw)
    }
}

impl TcKonnektMutableSegmentOperation<ItwinChStripStates> for ItwinProtocol {}

impl TcKonnektNotifiedSegmentOperation<ItwinChStripStates> for ItwinProtocol {
    const NOTIFY_FLAG: u32 = SHELL_CH_STRIP_NOTIFY_FLAG;
}

impl AsRef<[ChStripState]> for ItwinChStripStates {
    fn as_ref(&self) -> &[ChStripState] {
        &self.0
    }
}

impl AsMut<[ChStripState]> for ItwinChStripStates {
    fn as_mut(&mut self) -> &mut [ChStripState] {
        &mut self.0
    }
}

/// The mode to listen for analog outputs.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ListeningMode {
    /// Monaural.
    Monaural,
    /// Stereo.
    Stereo,
    /// Side channels only.
    Side,
}

impl Default for ListeningMode {
    fn default() -> Self {
        Self::Monaural
    }
}

const LISTENING_MODES: &[ListeningMode] = &[
    ListeningMode::Monaural,
    ListeningMode::Stereo,
    ListeningMode::Side,
];

const LISTENING_MODE_LABEL: &str = "listening mode";

fn serialize_listening_mode(mode: &ListeningMode, raw: &mut [u8]) -> Result<(), String> {
    serialize_position(LISTENING_MODES, mode, raw, LISTENING_MODE_LABEL)
}

fn deserialize_listening_mode(mode: &mut ListeningMode, raw: &[u8]) -> Result<(), String> {
    deserialize_position(LISTENING_MODES, mode, raw, LISTENING_MODE_LABEL)
}

/// Hardware state.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct ItwinHwState {
    /// State of hardware.
    pub hw_state: ShellHwState,
    /// Mode of listening.
    pub listening_mode: ListeningMode,
}

impl TcKonnektSegmentSerdes<ItwinHwState> for ItwinProtocol {
    const NAME: &'static str = "hardware-state";
    const OFFSET: usize = 0x1008;
    const SIZE: usize = ShellHwState::SIZE;

    fn serialize(params: &ItwinHwState, raw: &mut [u8]) -> Result<(), String> {
        serialize_hw_state(&params.hw_state, raw)?;
        serialize_listening_mode(&params.listening_mode, &mut raw[8..12])?;
        Ok(())
    }

    fn deserialize(params: &mut ItwinHwState, raw: &[u8]) -> Result<(), String> {
        deserialize_hw_state(&mut params.hw_state, raw)?;
        deserialize_listening_mode(&mut params.listening_mode, &raw[8..12])?;
        Ok(())
    }
}

impl TcKonnektMutableSegmentOperation<ItwinHwState> for ItwinProtocol {}

impl TcKonnektNotifiedSegmentOperation<ItwinHwState> for ItwinProtocol {
    const NOTIFY_FLAG: u32 = SHELL_HW_STATE_NOTIFY_FLAG;
}

impl AsRef<ShellHwState> for ItwinHwState {
    fn as_ref(&self) -> &ShellHwState {
        &self.hw_state
    }
}

impl AsMut<ShellHwState> for ItwinHwState {
    fn as_mut(&mut self) -> &mut ShellHwState {
        &mut self.hw_state
    }
}

impl AsRef<FireWireLedState> for ItwinHwState {
    fn as_ref(&self) -> &FireWireLedState {
        &self.hw_state.firewire_led
    }
}

impl AsMut<FireWireLedState> for ItwinHwState {
    fn as_mut(&mut self) -> &mut FireWireLedState {
        &mut self.hw_state.firewire_led
    }
}

/// Hardware metering for mixer function.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItwinMixerMeter(pub ShellMixerMeter);

impl Default for ItwinMixerMeter {
    fn default() -> Self {
        ItwinMixerMeter(ItwinProtocol::create_meter_state())
    }
}

impl ShellMixerMeterSpecification for ItwinProtocol {
    const ANALOG_INPUT_COUNT: usize = 4;
    const DIGITAL_INPUT_COUNT: usize = 8;
}

impl TcKonnektSegmentSerdes<ItwinMixerMeter> for ItwinProtocol {
    const NAME: &'static str = "mixer-meter";
    const OFFSET: usize = 0x106c;
    const SIZE: usize = ShellMixerMeter::SIZE;

    fn serialize(params: &ItwinMixerMeter, raw: &mut [u8]) -> Result<(), String> {
        serialize_mixer_meter::<ItwinProtocol>(&params.0, raw)
    }

    fn deserialize(params: &mut ItwinMixerMeter, raw: &[u8]) -> Result<(), String> {
        deserialize_mixer_meter::<ItwinProtocol>(&mut params.0, raw)
    }
}

impl AsRef<ShellMixerMeter> for ItwinMixerMeter {
    fn as_ref(&self) -> &ShellMixerMeter {
        &self.0
    }
}

impl AsMut<ShellMixerMeter> for ItwinMixerMeter {
    fn as_mut(&mut self) -> &mut ShellMixerMeter {
        &mut self.0
    }
}

/// Hardware metering for reverb effect.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct ItwinReverbMeter(pub ReverbMeter);

impl TcKonnektSegmentSerdes<ItwinReverbMeter> for ItwinProtocol {
    const NAME: &'static str = "reverb-meter";
    const OFFSET: usize = 0x10c8;
    const SIZE: usize = ReverbMeter::SIZE;

    fn serialize(params: &ItwinReverbMeter, raw: &mut [u8]) -> Result<(), String> {
        serialize_reverb_meter(&params.0, raw)
    }

    fn deserialize(params: &mut ItwinReverbMeter, raw: &[u8]) -> Result<(), String> {
        deserialize_reverb_meter(&mut params.0, raw)
    }
}

impl AsRef<ReverbMeter> for ItwinReverbMeter {
    fn as_ref(&self) -> &ReverbMeter {
        &self.0
    }
}

impl AsMut<ReverbMeter> for ItwinReverbMeter {
    fn as_mut(&mut self) -> &mut ReverbMeter {
        &mut self.0
    }
}

/// Hardware metering for channel strip effect.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct ItwinChStripMeters(pub [ChStripMeter; SHELL_CH_STRIP_COUNT]);

impl TcKonnektSegmentSerdes<ItwinChStripMeters> for ItwinProtocol {
    const NAME: &'static str = "channel-strip-meter";
    const OFFSET: usize = 0x10e0;
    const SIZE: usize = ChStripMeter::SIZE * SHELL_CH_STRIP_COUNT + 4;

    fn serialize(params: &ItwinChStripMeters, raw: &mut [u8]) -> Result<(), String> {
        serialize_ch_strip_meters(&params.0, raw)
    }

    fn deserialize(params: &mut ItwinChStripMeters, raw: &[u8]) -> Result<(), String> {
        deserialize_ch_strip_meters(&mut params.0, raw)
    }
}

impl AsRef<[ChStripMeter]> for ItwinChStripMeters {
    fn as_ref(&self) -> &[ChStripMeter] {
        &self.0
    }
}

impl AsMut<[ChStripMeter]> for ItwinChStripMeters {
    fn as_mut(&mut self) -> &mut [ChStripMeter] {
        &mut self.0
    }
}
