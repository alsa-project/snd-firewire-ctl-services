// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Protocol defined by TC Electronic for Konnekt Live.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by TC Electronic for Konnekt Live.

use super::*;

/// Protocol implementation of Konnekt Live.
#[derive(Default)]
pub struct KliveProtocol;

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

/// State of knob.
#[derive(Default, Debug)]
pub struct KliveKnob {
    pub target: ShellKnobTarget,
    pub knob2_target: ShellKnob2Target,
    pub prog: TcKonnektLoadedProgram,
    pub out_impedance: [OutputImpedance; 2],
}

impl TcKonnektSegmentData for KliveKnob {
    fn build(&self, raw: &mut [u8]) {
        self.target.0.build_quadlet(&mut raw[..4]);
        self.knob2_target.0.build_quadlet(&mut raw[4..8]);
        self.prog.build(&mut raw[8..12]);
        self.out_impedance.build_quadlet_block(&mut raw[12..20]);
    }

    fn parse(&mut self, raw: &[u8]) {
        self.target.0.parse_quadlet(&raw[..4]);
        self.knob2_target.0.parse_quadlet(&raw[4..8]);
        self.prog.parse(&raw[8..12]);
        self.out_impedance.parse_quadlet_block(&raw[12..20]);
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
    const OFFSET: usize = 0x0004;
    const SIZE: usize = SHELL_KNOB_SIZE;
}

impl TcKonnektNotifiedSegmentSpec for TcKonnektSegment<KliveKnob> {
    const NOTIFY_FLAG: u32 = SHELL_KNOB_NOTIFY_FLAG;
}

/// Configuration.
#[derive(Default, Debug)]
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

impl TcKonnektSegmentData for KliveConfig {
    fn build(&self, raw: &mut [u8]) {
        self.opt.build(&mut raw[..12]);
        self.coax_out_src.0.build_quadlet(&mut raw[12..16]);
        self.out_01_src.build_quadlet(&mut raw[16..20]);
        self.out_23_src.build_quadlet(&mut raw[20..24]);
        self.mixer_stream_src_pair.build_quadlet(&mut raw[24..28]);
        self.standalone_src.build_quadlet(&mut raw[28..32]);
        self.standalone_rate.build_quadlet(&mut raw[32..36]);
        self.midi_sender.build(&mut raw[84..120]);
    }

    fn parse(&mut self, raw: &[u8]) {
        self.opt.parse(&raw[..12]);
        self.coax_out_src.0.parse_quadlet(&raw[12..16]);
        self.out_01_src.parse_quadlet(&raw[16..20]);
        self.out_23_src.parse_quadlet(&raw[20..24]);
        self.mixer_stream_src_pair.parse_quadlet(&raw[24..28]);
        self.standalone_src.parse_quadlet(&raw[28..32]);
        self.standalone_rate.parse_quadlet(&raw[32..36]);
        self.midi_sender.parse(&raw[84..120]);
    }
}

impl TcKonnektSegmentSpec for TcKonnektSegment<KliveConfig> {
    const OFFSET: usize = 0x0028;
    const SIZE: usize = 132;
}

impl TcKonnektNotifiedSegmentSpec for TcKonnektSegment<KliveConfig> {
    const NOTIFY_FLAG: u32 = SHELL_CONFIG_NOTIFY_FLAG;
}

/// The source of channel strip effect.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
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
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
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
#[derive(Debug)]
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

impl TcKonnektSegmentData for KliveMixerState {
    fn build(&self, raw: &mut [u8]) {
        ShellMixerStateConvert::build(self, raw);

        self.reverb_return.build(&mut raw[316..328]);
        self.use_ch_strip_as_plugin
            .build_quadlet(&mut raw[328..332]);
        self.ch_strip_src.build_quadlet(&mut raw[332..336]);
        self.ch_strip_mode.build_quadlet(&mut raw[336..340]);
        self.use_reverb_at_mid_rate
            .build_quadlet(&mut raw[340..344]);
        self.enabled.build_quadlet(&mut raw[344..348]);
    }

    fn parse(&mut self, raw: &[u8]) {
        ShellMixerStateConvert::parse(self, raw);

        self.reverb_return.parse(&raw[316..328]);
        self.use_ch_strip_as_plugin.parse_quadlet(&raw[328..332]);
        self.ch_strip_src.parse_quadlet(&raw[332..336]);
        self.ch_strip_mode.parse_quadlet(&raw[336..340]);
        self.use_reverb_at_mid_rate.parse_quadlet(&raw[340..344]);
        self.enabled.parse_quadlet(&raw[344..348]);
    }
}

impl TcKonnektSegmentSpec for TcKonnektSegment<KliveMixerState> {
    const OFFSET: usize = 0x00ac;
    const SIZE: usize = ShellMixerState::SIZE + 48;
}

impl TcKonnektNotifiedSegmentSpec for TcKonnektSegment<KliveMixerState> {
    const NOTIFY_FLAG: u32 = SHELL_MIXER_NOTIFY_FLAG;
}

#[derive(Default, Debug)]
pub struct KliveReverbState(pub ReverbState);

impl TcKonnektSegmentData for KliveReverbState {
    fn build(&self, raw: &mut [u8]) {
        self.0.build(raw)
    }

    fn parse(&mut self, raw: &[u8]) {
        self.0.parse(raw)
    }
}

impl TcKonnektSegmentSpec for TcKonnektSegment<KliveReverbState> {
    const OFFSET: usize = 0x0218;
    const SIZE: usize = ReverbState::SIZE;
}

impl TcKonnektNotifiedSegmentSpec for TcKonnektSegment<KliveReverbState> {
    const NOTIFY_FLAG: u32 = SHELL_REVERB_NOTIFY_FLAG;
}

#[derive(Default, Debug)]
pub struct KliveChStripStates(pub [ChStripState; SHELL_CH_STRIP_COUNT]);

impl TcKonnektSegmentData for KliveChStripStates {
    fn build(&self, raw: &mut [u8]) {
        self.0.build(raw)
    }

    fn parse(&mut self, raw: &[u8]) {
        self.0.parse(raw)
    }
}

impl TcKonnektSegmentSpec for TcKonnektSegment<KliveChStripStates> {
    const OFFSET: usize = 0x0260;
    const SIZE: usize = ChStripState::SIZE * SHELL_CH_STRIP_COUNT + 4;
}

impl TcKonnektNotifiedSegmentSpec for TcKonnektSegment<KliveChStripStates> {
    const NOTIFY_FLAG: u32 = SHELL_CH_STRIP_NOTIFY_FLAG;
}

#[derive(Default, Debug)]
pub struct KliveHwState(pub ShellHwState);

impl TcKonnektSegmentData for KliveHwState {
    fn build(&self, raw: &mut [u8]) {
        self.0.build(raw)
    }

    fn parse(&mut self, raw: &[u8]) {
        self.0.parse(raw)
    }
}

impl TcKonnektSegmentSpec for TcKonnektSegment<KliveHwState> {
    const OFFSET: usize = 0x1008;
    const SIZE: usize = ShellHwState::SIZE;
}

impl TcKonnektNotifiedSegmentSpec for TcKonnektSegment<KliveHwState> {
    const NOTIFY_FLAG: u32 = SHELL_HW_STATE_NOTIFY_FLAG;
}

const KLIVE_METER_ANALOG_INPUT_COUNT: usize = 4;
const KLIVE_METER_DIGITAL_INPUT_COUNT: usize = 8;

#[derive(Debug)]
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

impl TcKonnektSegmentData for KliveMixerMeter {
    fn build(&self, raw: &mut [u8]) {
        ShellMixerMeterConvert::build(self, raw)
    }

    fn parse(&mut self, raw: &[u8]) {
        ShellMixerMeterConvert::parse(self, raw)
    }
}

impl TcKonnektSegmentSpec for TcKonnektSegment<KliveMixerMeter> {
    const OFFSET: usize = 0x1068;
    const SIZE: usize = ShellMixerMeter::SIZE;
}

#[derive(Default, Debug)]
pub struct KliveReverbMeter(pub ReverbMeter);

impl TcKonnektSegmentData for KliveReverbMeter {
    fn build(&self, raw: &mut [u8]) {
        self.0.build(raw)
    }

    fn parse(&mut self, raw: &[u8]) {
        self.0.parse(raw)
    }
}

impl TcKonnektSegmentSpec for TcKonnektSegment<KliveReverbMeter> {
    const OFFSET: usize = 0x10c4;
    const SIZE: usize = ReverbMeter::SIZE;
}

#[derive(Default, Debug)]
pub struct KliveChStripMeters(pub [ChStripMeter; SHELL_CH_STRIP_COUNT]);

impl TcKonnektSegmentData for KliveChStripMeters {
    fn build(&self, raw: &mut [u8]) {
        self.0.build(raw)
    }

    fn parse(&mut self, raw: &[u8]) {
        self.0.parse(raw)
    }
}

impl TcKonnektSegmentSpec for TcKonnektSegment<KliveChStripMeters> {
    const OFFSET: usize = 0x10dc;
    const SIZE: usize = ChStripMeter::SIZE * SHELL_CH_STRIP_COUNT + 4;
}

/// Impedance of output.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
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
