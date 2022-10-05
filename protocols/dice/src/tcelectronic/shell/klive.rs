// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Protocol defined by TC Electronic for Konnekt Live.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by TC Electronic for Konnekt Live.

use super::*;

/// Protocol implementation of Konnekt Live.
#[derive(Default, Debug)]
pub struct KliveProtocol;

impl TcatOperation for KliveProtocol {}

impl TcatGlobalSectionSpecification for KliveProtocol {}

/// Segment for knob. 0x0000..0x0027 (36 quads).
pub type KliveKnobSegment = TcKonnektSegment<KliveKnob>;
impl SegmentOperation<KliveKnob> for KliveProtocol {}

/// Segment for configuration. 0x0028..0x00ab (33 quads).
pub type KliveConfigSegment = TcKonnektSegment<KliveConfig>;
impl SegmentOperation<KliveConfig> for KliveProtocol {}

/// Segment for state of mixer. 0x00ac..0x0217 (91 quads).
pub type KliveMixerStateSegment = TcKonnektSegment<KliveMixerState>;
impl SegmentOperation<KliveMixerState> for KliveProtocol {}

/// Segment for state of reverb effect. 0x0218..0x025b. (17 quads)
pub type KliveReverbStateSegment = TcKonnektSegment<KliveReverbState>;
impl SegmentOperation<KliveReverbState> for KliveProtocol {}

/// Segment for states of channel strip effect. 0x0260..0x037f (72 quads).
pub type KliveChStripStatesSegment = TcKonnektSegment<KliveChStripStates>;
impl SegmentOperation<KliveChStripStates> for KliveProtocol {}

// NOTE: Segment for tuner. 0x0384..0x039c (8 quads).

/// Segment for mixer meter. 0x1068..0x10c3 (23 quads).
pub type KliveMixerMeterSegment = TcKonnektSegment<KliveMixerMeter>;
impl SegmentOperation<KliveMixerMeter> for KliveProtocol {}

/// Segment for state of hardware. 0x1008..0x1023 (7 quads).
pub type KliveHwStateSegment = TcKonnektSegment<KliveHwState>;
impl SegmentOperation<KliveHwState> for KliveProtocol {}

/// Segment for meter of reverb effect. 0x10c4..0x010db (6 quads).
pub type KliveReverbMeterSegment = TcKonnektSegment<KliveReverbMeter>;
impl SegmentOperation<KliveReverbMeter> for KliveProtocol {}

/// Segment for meters of channel strip effect. 0x10dc..0x1117 (15 quads).
pub type KliveChStripMetersSegment = TcKonnektSegment<KliveChStripMeters>;
impl SegmentOperation<KliveChStripMeters> for KliveProtocol {}

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

segment_default!(KliveProtocol, KliveKnob);
segment_default!(KliveProtocol, KliveConfig);
segment_default!(KliveProtocol, KliveMixerState);
segment_default!(KliveProtocol, KliveReverbState);
segment_default!(KliveProtocol, KliveChStripStates);
segment_default!(KliveProtocol, KliveMixerMeter);
segment_default!(KliveProtocol, KliveHwState);
segment_default!(KliveProtocol, KliveReverbMeter);
segment_default!(KliveProtocol, KliveChStripMeters);

/// State of knob.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct KliveKnob {
    pub target: ShellKnobTarget,
    pub knob2_target: ShellKnob2Target,
    pub prog: TcKonnektLoadedProgram,
    pub out_impedance: [OutputImpedance; 2],
}

impl TcKonnektSegmentSerdes<KliveKnob> for KliveProtocol {
    const NAME: &'static str = "knob";
    const OFFSET: usize = 0x0004;
    const SIZE: usize = SHELL_KNOB_SIZE;

    fn serialize(params: &KliveKnob, raw: &mut [u8]) -> Result<(), String> {
        params.target.0.build_quadlet(&mut raw[..4]);
        params.knob2_target.0.build_quadlet(&mut raw[4..8]);
        params.prog.build(&mut raw[8..12]);
        params.out_impedance.build_quadlet_block(&mut raw[12..20]);
        Ok(())
    }

    fn deserialize(params: &mut KliveKnob, raw: &[u8]) -> Result<(), String> {
        params.target.0.parse_quadlet(&raw[..4]);
        params.knob2_target.0.parse_quadlet(&raw[4..8]);
        params.prog.parse(&raw[8..12]);
        params.out_impedance.parse_quadlet_block(&raw[12..20]);
        Ok(())
    }
}

impl TcKonnektMutableSegmentOperation<KliveKnob> for KliveProtocol {}

impl TcKonnektNotifiedSegmentOperation<KliveKnob> for KliveProtocol {
    const NOTIFY_FLAG: u32 = SHELL_KNOB_NOTIFY_FLAG;
}

impl TcKonnektSegmentData for KliveKnob {
    fn build(&self, raw: &mut [u8]) {
        let _ = <KliveProtocol as TcKonnektSegmentSerdes<Self>>::serialize(self, raw);
    }

    fn parse(&mut self, raw: &[u8]) {
        let _ = <KliveProtocol as TcKonnektSegmentSerdes<Self>>::deserialize(self, raw);
    }
}

impl ShellKnobTargetSpec for KliveKnob {
    const HAS_SPDIF: bool = false;
    const HAS_EFFECTS: bool = false;
}

impl ShellKnob2TargetSpec for KliveKnob {
    const KNOB2_TARGET_COUNT: usize = 9;
}

impl TcKonnektSegmentSpec for TcKonnektSegment<KliveKnob> {
    const OFFSET: usize = <KliveProtocol as TcKonnektSegmentSerdes<KliveKnob>>::OFFSET;
    const SIZE: usize = <KliveProtocol as TcKonnektSegmentSerdes<KliveKnob>>::SIZE;
}

impl TcKonnektNotifiedSegmentSpec for TcKonnektSegment<KliveKnob> {
    const NOTIFY_FLAG: u32 =
        <KliveProtocol as TcKonnektNotifiedSegmentOperation<KliveKnob>>::NOTIFY_FLAG;
}

/// Configuration.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct KliveConfig {
    pub opt: ShellOptIfaceConfig,
    pub coax_out_src: ShellCoaxOutPairSrc,
    pub out_01_src: ShellPhysOutSrc,
    pub out_23_src: ShellPhysOutSrc,
    pub mixer_stream_src_pair: ShellMixerStreamSrcPair,
    pub standalone_src: ShellStandaloneClkSrc,
    pub standalone_rate: TcKonnektStandaloneClkRate,
    pub midi_sender: TcKonnektMidiSender,
}

impl ShellMixerStreamSrcPairSpec for KliveConfig {
    const MAXIMUM_STREAM_SRC_PAIR_COUNT: usize = 6;
}

impl ShellStandaloneClkSpec for KliveConfig {
    const STANDALONE_CLOCK_SOURCES: &'static [ShellStandaloneClkSrc] = &[
        ShellStandaloneClkSrc::Optical,
        ShellStandaloneClkSrc::Coaxial,
        ShellStandaloneClkSrc::Internal,
    ];
}

impl TcKonnektSegmentSerdes<KliveConfig> for KliveProtocol {
    const NAME: &'static str = "configuration";
    const OFFSET: usize = 0x0028;
    const SIZE: usize = 132;

    fn serialize(params: &KliveConfig, raw: &mut [u8]) -> Result<(), String> {
        params.opt.build(&mut raw[..12]);
        params.coax_out_src.0.build_quadlet(&mut raw[12..16]);
        params.out_01_src.build_quadlet(&mut raw[16..20]);
        params.out_23_src.build_quadlet(&mut raw[20..24]);
        params.mixer_stream_src_pair.build_quadlet(&mut raw[24..28]);
        params.standalone_src.build_quadlet(&mut raw[28..32]);
        params.standalone_rate.build_quadlet(&mut raw[32..36]);
        params.midi_sender.build(&mut raw[84..120]);
        Ok(())
    }

    fn deserialize(params: &mut KliveConfig, raw: &[u8]) -> Result<(), String> {
        params.opt.parse(&raw[..12]);
        params.coax_out_src.0.parse_quadlet(&raw[12..16]);
        params.out_01_src.parse_quadlet(&raw[16..20]);
        params.out_23_src.parse_quadlet(&raw[20..24]);
        params.mixer_stream_src_pair.parse_quadlet(&raw[24..28]);
        params.standalone_src.parse_quadlet(&raw[28..32]);
        params.standalone_rate.parse_quadlet(&raw[32..36]);
        params.midi_sender.parse(&raw[84..120]);
        Ok(())
    }
}

impl TcKonnektMutableSegmentOperation<KliveConfig> for KliveProtocol {}

impl TcKonnektNotifiedSegmentOperation<KliveConfig> for KliveProtocol {
    const NOTIFY_FLAG: u32 = SHELL_CONFIG_NOTIFY_FLAG;
}

impl TcKonnektSegmentData for KliveConfig {
    fn build(&self, raw: &mut [u8]) {
        let _ = <KliveProtocol as TcKonnektSegmentSerdes<Self>>::serialize(self, raw);
    }

    fn parse(&mut self, raw: &[u8]) {
        let _ = <KliveProtocol as TcKonnektSegmentSerdes<Self>>::deserialize(self, raw);
    }
}

impl TcKonnektSegmentSpec for TcKonnektSegment<KliveConfig> {
    const OFFSET: usize = <KliveProtocol as TcKonnektSegmentSerdes<KliveConfig>>::OFFSET;
    const SIZE: usize = <KliveProtocol as TcKonnektSegmentSerdes<KliveConfig>>::SIZE;
}

impl TcKonnektNotifiedSegmentSpec for TcKonnektSegment<KliveConfig> {
    const NOTIFY_FLAG: u32 =
        <KliveProtocol as TcKonnektNotifiedSegmentOperation<KliveConfig>>::NOTIFY_FLAG;
}

/// The source of channel strip effect.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ChStripSrc {
    Stream01,
    Analog01,
    Analog23,
    Digital01,
    Digital23,
    Digital45,
    Digital67,
    MixerOutput,
    None,
}

impl Default for ChStripSrc {
    fn default() -> Self {
        ChStripSrc::None
    }
}

impl From<u32> for ChStripSrc {
    fn from(val: u32) -> Self {
        match val {
            0 => Self::Stream01,
            4 => Self::Analog01,
            5 => Self::Analog23,
            6 => Self::Digital01,
            7 => Self::Digital23,
            8 => Self::Digital45,
            9 => Self::Digital67,
            10 => Self::MixerOutput,
            _ => Self::None,
        }
    }
}

impl From<ChStripSrc> for u32 {
    fn from(src: ChStripSrc) -> Self {
        match src {
            ChStripSrc::Stream01 => 0,
            ChStripSrc::Analog01 => 4,
            ChStripSrc::Analog23 => 5,
            ChStripSrc::Digital01 => 6,
            ChStripSrc::Digital23 => 7,
            ChStripSrc::Digital45 => 8,
            ChStripSrc::Digital67 => 9,
            ChStripSrc::MixerOutput => 10,
            ChStripSrc::None => 11,
        }
    }
}

/// The type of channel strip effect.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ChStripMode {
    FabrikC,
    RIAA1964,
    RIAA1987,
}

impl Default for ChStripMode {
    fn default() -> Self {
        ChStripMode::FabrikC
    }
}

impl From<u32> for ChStripMode {
    fn from(val: u32) -> Self {
        match val {
            2 => Self::RIAA1987,
            1 => Self::RIAA1964,
            _ => Self::FabrikC,
        }
    }
}

impl From<ChStripMode> for u32 {
    fn from(effect: ChStripMode) -> Self {
        match effect {
            ChStripMode::FabrikC => 0,
            ChStripMode::RIAA1964 => 1,
            ChStripMode::RIAA1987 => 2,
        }
    }
}

/// State of mixer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KliveMixerState {
    /// The common structure for state of mixer.
    pub mixer: ShellMixerState,
    /// The parameter of return from reverb effect.
    pub reverb_return: ShellReverbReturn,
    /// Whether to use channel strip effect as plugin. It results in output of channel strip effect
    /// on tx stream.
    pub use_ch_strip_as_plugin: bool,
    /// The source of channel strip effect.
    pub ch_strip_src: ChStripSrc,
    /// The type of channel effect. Fabrik-C, RIAA-1964, and RIAA-1987.
    pub ch_strip_mode: ChStripMode,
    /// Whether to use channel strip effect at middle sampling rate (88.2/96.0 kHz).
    pub use_reverb_at_mid_rate: bool,
    /// Whether to use mixer function.
    pub enabled: bool,
}

impl Default for KliveMixerState {
    fn default() -> Self {
        KliveMixerState {
            mixer: Self::create_mixer_state(),
            reverb_return: Default::default(),
            use_ch_strip_as_plugin: Default::default(),
            ch_strip_src: Default::default(),
            ch_strip_mode: Default::default(),
            use_reverb_at_mid_rate: Default::default(),
            enabled: Default::default(),
        }
    }
}

impl ShellMixerStateConvert for KliveMixerState {
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

    fn state(&self) -> &ShellMixerState {
        &self.mixer
    }

    fn state_mut(&mut self) -> &mut ShellMixerState {
        &mut self.mixer
    }
}

impl TcKonnektSegmentSerdes<KliveMixerState> for KliveProtocol {
    const NAME: &'static str = "mixer-state";
    const OFFSET: usize = 0x0004;
    const SIZE: usize = ShellMixerState::SIZE + 48;

    fn serialize(params: &KliveMixerState, raw: &mut [u8]) -> Result<(), String> {
        ShellMixerStateConvert::build(params, raw);

        params.reverb_return.build(&mut raw[316..328]);
        params
            .use_ch_strip_as_plugin
            .build_quadlet(&mut raw[328..332]);
        params.ch_strip_src.build_quadlet(&mut raw[332..336]);
        params.ch_strip_mode.build_quadlet(&mut raw[336..340]);
        params
            .use_reverb_at_mid_rate
            .build_quadlet(&mut raw[340..344]);
        params.enabled.build_quadlet(&mut raw[344..348]);
        Ok(())
    }

    fn deserialize(params: &mut KliveMixerState, raw: &[u8]) -> Result<(), String> {
        ShellMixerStateConvert::parse(params, raw);

        params.reverb_return.parse(&raw[316..328]);
        params.use_ch_strip_as_plugin.parse_quadlet(&raw[328..332]);
        params.ch_strip_src.parse_quadlet(&raw[332..336]);
        params.ch_strip_mode.parse_quadlet(&raw[336..340]);
        params.use_reverb_at_mid_rate.parse_quadlet(&raw[340..344]);
        params.enabled.parse_quadlet(&raw[344..348]);
        Ok(())
    }
}

impl TcKonnektMutableSegmentOperation<KliveMixerState> for KliveProtocol {}

impl TcKonnektNotifiedSegmentOperation<KliveMixerState> for KliveProtocol {
    const NOTIFY_FLAG: u32 = SHELL_MIXER_NOTIFY_FLAG;
}

impl TcKonnektSegmentData for KliveMixerState {
    fn build(&self, raw: &mut [u8]) {
        let _ = <KliveProtocol as TcKonnektSegmentSerdes<Self>>::serialize(self, raw);
    }

    fn parse(&mut self, raw: &[u8]) {
        let _ = <KliveProtocol as TcKonnektSegmentSerdes<Self>>::deserialize(self, raw);
    }
}

impl TcKonnektSegmentSpec for TcKonnektSegment<KliveMixerState> {
    const OFFSET: usize = <KliveProtocol as TcKonnektSegmentSerdes<KliveMixerState>>::OFFSET;
    const SIZE: usize = <KliveProtocol as TcKonnektSegmentSerdes<KliveMixerState>>::SIZE;
}

impl TcKonnektNotifiedSegmentSpec for TcKonnektSegment<KliveMixerState> {
    const NOTIFY_FLAG: u32 =
        <KliveProtocol as TcKonnektNotifiedSegmentOperation<KliveMixerState>>::NOTIFY_FLAG;
}

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct KliveReverbState(pub ReverbState);

impl TcKonnektSegmentSerdes<KliveReverbState> for KliveProtocol {
    const NAME: &'static str = "reverb-state";
    const OFFSET: usize = 0x0218;
    const SIZE: usize = ReverbState::SIZE;

    fn serialize(params: &KliveReverbState, raw: &mut [u8]) -> Result<(), String> {
        params.0.build(raw);
        Ok(())
    }

    fn deserialize(params: &mut KliveReverbState, raw: &[u8]) -> Result<(), String> {
        params.0.parse(raw);
        Ok(())
    }
}

impl TcKonnektMutableSegmentOperation<KliveReverbState> for KliveProtocol {}

impl TcKonnektNotifiedSegmentOperation<KliveReverbState> for KliveProtocol {
    const NOTIFY_FLAG: u32 = SHELL_REVERB_NOTIFY_FLAG;
}

impl TcKonnektSegmentData for KliveReverbState {
    fn build(&self, raw: &mut [u8]) {
        let _ = <KliveProtocol as TcKonnektSegmentSerdes<Self>>::serialize(self, raw);
    }

    fn parse(&mut self, raw: &[u8]) {
        let _ = <KliveProtocol as TcKonnektSegmentSerdes<Self>>::deserialize(self, raw);
    }
}

impl TcKonnektSegmentSpec for TcKonnektSegment<KliveReverbState> {
    const OFFSET: usize = <KliveProtocol as TcKonnektSegmentSerdes<KliveReverbState>>::OFFSET;
    const SIZE: usize = <KliveProtocol as TcKonnektSegmentSerdes<KliveReverbState>>::SIZE;
}

impl TcKonnektNotifiedSegmentSpec for TcKonnektSegment<KliveReverbState> {
    const NOTIFY_FLAG: u32 =
        <KliveProtocol as TcKonnektNotifiedSegmentOperation<KliveReverbState>>::NOTIFY_FLAG;
}

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct KliveChStripStates(pub [ChStripState; SHELL_CH_STRIP_COUNT]);

impl TcKonnektSegmentSerdes<KliveChStripStates> for KliveProtocol {
    const NAME: &'static str = "channel-strip-state";
    const OFFSET: usize = 0x0260;
    const SIZE: usize = ChStripState::SIZE * SHELL_CH_STRIP_COUNT + 4;

    fn serialize(params: &KliveChStripStates, raw: &mut [u8]) -> Result<(), String> {
        params.0.build(raw);
        Ok(())
    }

    fn deserialize(params: &mut KliveChStripStates, raw: &[u8]) -> Result<(), String> {
        params.0.parse(raw);
        Ok(())
    }
}

impl TcKonnektMutableSegmentOperation<KliveChStripStates> for KliveProtocol {}

impl TcKonnektNotifiedSegmentOperation<KliveChStripStates> for KliveProtocol {
    const NOTIFY_FLAG: u32 = SHELL_CH_STRIP_NOTIFY_FLAG;
}

impl TcKonnektSegmentData for KliveChStripStates {
    fn build(&self, raw: &mut [u8]) {
        let _ = <KliveProtocol as TcKonnektSegmentSerdes<Self>>::serialize(self, raw);
    }

    fn parse(&mut self, raw: &[u8]) {
        let _ = <KliveProtocol as TcKonnektSegmentSerdes<Self>>::deserialize(self, raw);
    }
}

impl TcKonnektSegmentSpec for TcKonnektSegment<KliveChStripStates> {
    const OFFSET: usize = <KliveProtocol as TcKonnektSegmentSerdes<KliveChStripStates>>::OFFSET;
    const SIZE: usize = <KliveProtocol as TcKonnektSegmentSerdes<KliveChStripStates>>::SIZE;
}

impl TcKonnektNotifiedSegmentSpec for TcKonnektSegment<KliveChStripStates> {
    const NOTIFY_FLAG: u32 =
        <KliveProtocol as TcKonnektNotifiedSegmentOperation<KliveChStripStates>>::NOTIFY_FLAG;
}

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct KliveHwState(pub ShellHwState);

impl TcKonnektSegmentSerdes<KliveHwState> for KliveProtocol {
    const NAME: &'static str = "hardware-state";
    const OFFSET: usize = 0x1008;
    const SIZE: usize = ShellHwState::SIZE;

    fn serialize(params: &KliveHwState, raw: &mut [u8]) -> Result<(), String> {
        params.0.build(raw);
        Ok(())
    }

    fn deserialize(params: &mut KliveHwState, raw: &[u8]) -> Result<(), String> {
        params.0.parse(raw);
        Ok(())
    }
}

impl TcKonnektMutableSegmentOperation<KliveHwState> for KliveProtocol {}

impl TcKonnektNotifiedSegmentOperation<KliveHwState> for KliveProtocol {
    const NOTIFY_FLAG: u32 = SHELL_HW_STATE_NOTIFY_FLAG;
}

impl TcKonnektSegmentData for KliveHwState {
    fn build(&self, raw: &mut [u8]) {
        let _ = <KliveProtocol as TcKonnektSegmentSerdes<Self>>::serialize(self, raw);
    }

    fn parse(&mut self, raw: &[u8]) {
        let _ = <KliveProtocol as TcKonnektSegmentSerdes<Self>>::deserialize(self, raw);
    }
}

impl TcKonnektSegmentSpec for TcKonnektSegment<KliveHwState> {
    const OFFSET: usize = <KliveProtocol as TcKonnektSegmentSerdes<KliveHwState>>::OFFSET;
    const SIZE: usize = <KliveProtocol as TcKonnektSegmentSerdes<KliveHwState>>::SIZE;
}

impl TcKonnektNotifiedSegmentSpec for TcKonnektSegment<KliveHwState> {
    const NOTIFY_FLAG: u32 =
        <KliveProtocol as TcKonnektNotifiedSegmentOperation<KliveHwState>>::NOTIFY_FLAG;
}

const KLIVE_METER_ANALOG_INPUT_COUNT: usize = 4;
const KLIVE_METER_DIGITAL_INPUT_COUNT: usize = 8;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KliveMixerMeter(pub ShellMixerMeter);

impl Default for KliveMixerMeter {
    fn default() -> Self {
        KliveMixerMeter(Self::create_meter_state())
    }
}

impl ShellMixerMeterConvert for KliveMixerMeter {
    const ANALOG_INPUT_COUNT: usize = KLIVE_METER_ANALOG_INPUT_COUNT;
    const DIGITAL_INPUT_COUNT: usize = KLIVE_METER_DIGITAL_INPUT_COUNT;

    fn meter(&self) -> &ShellMixerMeter {
        &self.0
    }

    fn meter_mut(&mut self) -> &mut ShellMixerMeter {
        &mut self.0
    }
}

impl TcKonnektSegmentSerdes<KliveMixerMeter> for KliveProtocol {
    const NAME: &'static str = "mixer-meter";
    const OFFSET: usize = 0x1068;
    const SIZE: usize = ShellMixerMeter::SIZE;

    fn serialize(params: &KliveMixerMeter, raw: &mut [u8]) -> Result<(), String> {
        ShellMixerMeterConvert::build(params, raw);
        Ok(())
    }

    fn deserialize(params: &mut KliveMixerMeter, raw: &[u8]) -> Result<(), String> {
        ShellMixerMeterConvert::parse(params, raw);
        Ok(())
    }
}

impl TcKonnektSegmentData for KliveMixerMeter {
    fn build(&self, raw: &mut [u8]) {
        let _ = <KliveProtocol as TcKonnektSegmentSerdes<Self>>::serialize(self, raw);
    }

    fn parse(&mut self, raw: &[u8]) {
        let _ = <KliveProtocol as TcKonnektSegmentSerdes<Self>>::deserialize(self, raw);
    }
}

impl TcKonnektSegmentSpec for TcKonnektSegment<KliveMixerMeter> {
    const OFFSET: usize = <KliveProtocol as TcKonnektSegmentSerdes<KliveMixerMeter>>::OFFSET;
    const SIZE: usize = <KliveProtocol as TcKonnektSegmentSerdes<KliveMixerMeter>>::SIZE;
}

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct KliveReverbMeter(pub ReverbMeter);

impl TcKonnektSegmentSerdes<KliveReverbMeter> for KliveProtocol {
    const NAME: &'static str = "reverb-meter";
    const OFFSET: usize = 0x10c4;
    const SIZE: usize = ReverbMeter::SIZE;

    fn serialize(params: &KliveReverbMeter, raw: &mut [u8]) -> Result<(), String> {
        params.0.build(raw);
        Ok(())
    }

    fn deserialize(params: &mut KliveReverbMeter, raw: &[u8]) -> Result<(), String> {
        params.0.parse(raw);
        Ok(())
    }
}

impl TcKonnektSegmentData for KliveReverbMeter {
    fn build(&self, raw: &mut [u8]) {
        let _ = <KliveProtocol as TcKonnektSegmentSerdes<Self>>::serialize(self, raw);
    }

    fn parse(&mut self, raw: &[u8]) {
        let _ = <KliveProtocol as TcKonnektSegmentSerdes<Self>>::deserialize(self, raw);
    }
}

impl TcKonnektSegmentSpec for TcKonnektSegment<KliveReverbMeter> {
    const OFFSET: usize = <KliveProtocol as TcKonnektSegmentSerdes<KliveReverbMeter>>::OFFSET;
    const SIZE: usize = <KliveProtocol as TcKonnektSegmentSerdes<KliveReverbMeter>>::SIZE;
}

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct KliveChStripMeters(pub [ChStripMeter; SHELL_CH_STRIP_COUNT]);

impl TcKonnektSegmentSerdes<KliveChStripMeters> for KliveProtocol {
    const NAME: &'static str = "channel-strip-meter";
    const OFFSET: usize = 0x10dc;
    const SIZE: usize = ChStripMeter::SIZE * SHELL_CH_STRIP_COUNT + 4;

    fn serialize(params: &KliveChStripMeters, raw: &mut [u8]) -> Result<(), String> {
        params.0.build(raw);
        Ok(())
    }

    fn deserialize(params: &mut KliveChStripMeters, raw: &[u8]) -> Result<(), String> {
        params.0.parse(raw);
        Ok(())
    }
}

impl TcKonnektSegmentData for KliveChStripMeters {
    fn build(&self, raw: &mut [u8]) {
        let _ = <KliveProtocol as TcKonnektSegmentSerdes<Self>>::serialize(self, raw);
    }

    fn parse(&mut self, raw: &[u8]) {
        let _ = <KliveProtocol as TcKonnektSegmentSerdes<Self>>::deserialize(self, raw);
    }
}

impl TcKonnektSegmentSpec for TcKonnektSegment<KliveChStripMeters> {
    const OFFSET: usize = <KliveProtocol as TcKonnektSegmentSerdes<KliveChStripMeters>>::OFFSET;
    const SIZE: usize = <KliveProtocol as TcKonnektSegmentSerdes<KliveChStripMeters>>::SIZE;
}

/// Impedance of output.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum OutputImpedance {
    Unbalance,
    Balance,
}

impl Default for OutputImpedance {
    fn default() -> Self {
        Self::Unbalance
    }
}

impl From<u32> for OutputImpedance {
    fn from(val: u32) -> Self {
        match val {
            0 => Self::Unbalance,
            _ => Self::Balance,
        }
    }
}

impl From<OutputImpedance> for u32 {
    fn from(impedance: OutputImpedance) -> Self {
        match impedance {
            OutputImpedance::Unbalance => 0,
            OutputImpedance::Balance => 1,
        }
    }
}
