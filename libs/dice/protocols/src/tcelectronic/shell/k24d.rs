// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Protocol defined by TC Electronic for Konnekt 24d.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by TC Electronic for Konnekt 24d.

use super::*;

/// The structure for protocol implementation of Konnekt 24d.
#[derive(Default)]
pub struct K24dProtocol;

/// Segment for knob. 0x0000..0x0027 (36 quads).
pub type K24dKnobSegment = TcKonnektSegment<K24dKnob>;
impl SegmentOperation<K24dKnob> for K24dProtocol {}

/// Segment for configuration. 0x0028..0x0073 (76 quads).
pub type K24dConfigSegment = TcKonnektSegment<K24dConfig>;
impl SegmentOperation<K24dConfig> for K24dProtocol {}

/// Segment for state of mixer. 0x0074..0x01cf (87 quads).
pub type K24dMixerStateSegment = TcKonnektSegment<K24dMixerState>;
impl SegmentOperation<K24dMixerState> for K24dProtocol {}

/// Segment for state of reverb effect. 0x01d0..0x0213. (17 quads)
pub type K24dReverbStateSegment = TcKonnektSegment<K24dReverbState>;
impl SegmentOperation<K24dReverbState> for K24dProtocol {}

/// Segment for states of channel strip effect. 0x0218..0x0337 (72 quads).
pub type K24dChStripStatesSegment = TcKonnektSegment<K24dChStripStates>;
impl SegmentOperation<K24dChStripStates> for K24dProtocol {}

// NOTE: Segment for tuner. 0x0338..0x033b (8 quads).

/// Segment for mixer meter. 0x105c..0x10b7 (23 quads).
pub type K24dMixerMeterSegment = TcKonnektSegment<K24dMixerMeter>;
impl SegmentOperation<K24dMixerMeter> for K24dProtocol {}

/// Segment for state of hardware. 0x100c..0x1027 (7 quads).
pub type K24dHwStateSegment = TcKonnektSegment<K24dHwState>;
impl SegmentOperation<K24dHwState> for K24dProtocol {}

/// Segment for meter of reverb effect. 0x10b8..0x010cf (6 quads).
pub type K24dReverbMeterSegment = TcKonnektSegment<K24dReverbMeter>;
impl SegmentOperation<K24dReverbMeter> for K24dProtocol {}

/// Segment for meters of channel strip effect. 0x10d0..0x110b (15 quads).
pub type K24dChStripMetersSegment = TcKonnektSegment<K24dChStripMeters>;
impl SegmentOperation<K24dChStripMeters> for K24dProtocol {}

/// The structure to represent state of knob.
#[derive(Default, Debug)]
pub struct K24dKnob {
    pub target: ShellKnobTarget,
    pub knob2_target: ShellKnob2Target,
    pub prog: TcKonnektLoadedProgram,
}

impl ShellKnobTargetSpec for K24dKnob {
    const HAS_SPDIF: bool = false;
    const HAS_EFFECTS: bool = false;
}

impl ShellKnob2TargetSpec for K24dKnob {
    const KNOB2_TARGET_COUNT: usize = 8;
}

impl TcKonnektSegmentData for K24dKnob {
    fn build(&self, raw: &mut [u8]) {
        self.target.0.build_quadlet(&mut raw[..4]);
        self.knob2_target.0.build_quadlet(&mut raw[4..8]);
        self.prog.build(&mut raw[8..12]);
    }

    fn parse(&mut self, raw: &[u8]) {
        self.target.0.parse_quadlet(&raw[..4]);
        self.knob2_target.0.parse_quadlet(&raw[4..8]);
        self.prog.parse(&raw[8..12]);
    }
}

impl TcKonnektSegmentSpec for TcKonnektSegment<K24dKnob> {
    const OFFSET: usize = 0x0004;
    const SIZE: usize = SHELL_KNOB_SIZE;
}

impl TcKonnektNotifiedSegmentSpec for TcKonnektSegment<K24dKnob> {
    const NOTIFY_FLAG: u32 = SHELL_KNOB_NOTIFY_FLAG;
}

#[derive(Default, Debug)]
pub struct K24dConfig {
    pub opt: ShellOptIfaceConfig,
    pub coax_out_src: ShellCoaxOutPairSrc,
    pub out_23_src: ShellPhysOutSrc,
    pub standalone_src: ShellStandaloneClkSrc,
    pub standalone_rate: TcKonnektStandaloneClkRate,
}

impl ShellStandaloneClkSpec for K24dConfig {
    const STANDALONE_CLOCK_SOURCES: &'static [ShellStandaloneClkSrc] = &[
        ShellStandaloneClkSrc::Optical,
        ShellStandaloneClkSrc::Coaxial,
        ShellStandaloneClkSrc::Internal,
    ];
}

impl TcKonnektSegmentData for K24dConfig {
    fn build(&self, raw: &mut [u8]) {
        self.opt.build(&mut raw[..12]);
        self.coax_out_src.0.build_quadlet(&mut raw[12..16]);
        self.out_23_src.build_quadlet(&mut raw[16..20]);
        self.standalone_src.build_quadlet(&mut raw[20..24]);
        self.standalone_rate.build_quadlet(&mut raw[24..28]);
    }

    fn parse(&mut self, raw: &[u8]) {
        self.opt.parse(&raw[..12]);
        self.coax_out_src.0.parse_quadlet(&raw[12..16]);
        self.out_23_src.parse_quadlet(&raw[16..20]);
        self.standalone_src.parse_quadlet(&raw[20..24]);
        self.standalone_rate.parse_quadlet(&raw[24..28]);
    }
}

impl TcKonnektSegmentSpec for TcKonnektSegment<K24dConfig> {
    const OFFSET: usize = 0x0028;
    const SIZE: usize = 76;
}

impl TcKonnektNotifiedSegmentSpec for TcKonnektSegment<K24dConfig> {
    const NOTIFY_FLAG: u32 = SHELL_CONFIG_NOTIFY_FLAG;
}

/// The structure to represent state of mixer.
#[derive(Debug)]
pub struct K24dMixerState {
    /// The common structure for state of mixer.
    pub mixer: ShellMixerState,
    /// The parameter of return from reverb effect.
    pub reverb_return: ShellReverbReturn,
    /// Whether to use channel strip effect as plugin. It results in output of channel strip effect
    /// on tx stream.
    pub use_ch_strip_as_plugin: bool,
    /// Whether to use reverb effect at middle sampling rate (88.2/96.0 kHz).
    pub use_reverb_at_mid_rate: bool,
    /// Whether to use mixer function.
    pub enabled: bool,
}

impl Default for K24dMixerState {
    fn default() -> Self {
        K24dMixerState {
            mixer: Self::create_mixer_state(),
            reverb_return: Default::default(),
            use_ch_strip_as_plugin: Default::default(),
            use_reverb_at_mid_rate: Default::default(),
            enabled: Default::default(),
        }
    }
}

impl ShellMixerStateConvert for K24dMixerState {
    const MONITOR_SRC_MAP: [Option<ShellMixerMonitorSrcType>; SHELL_MIXER_MONITOR_SRC_COUNT] = [
        Some(ShellMixerMonitorSrcType::Stream),
        None,
        None,
        None,
        Some(ShellMixerMonitorSrcType::Analog),
        Some(ShellMixerMonitorSrcType::Analog),
        Some(ShellMixerMonitorSrcType::AdatSpdif),
        Some(ShellMixerMonitorSrcType::Adat),
        Some(ShellMixerMonitorSrcType::Adat),
        Some(ShellMixerMonitorSrcType::AdatSpdif),
    ];

    fn state(&self) -> &ShellMixerState {
        &self.mixer
    }

    fn state_mut(&mut self) -> &mut ShellMixerState {
        &mut self.mixer
    }
}

impl TcKonnektSegmentData for K24dMixerState {
    fn build(&self, raw: &mut [u8]) {
        ShellMixerStateConvert::build(self, raw);

        self.reverb_return.build(&mut raw[316..328]);
        self.use_ch_strip_as_plugin
            .build_quadlet(&mut raw[328..332]);
        self.use_reverb_at_mid_rate
            .build_quadlet(&mut raw[332..336]);
        self.enabled.build_quadlet(&mut raw[340..344]);
    }

    fn parse(&mut self, raw: &[u8]) {
        ShellMixerStateConvert::parse(self, raw);

        self.reverb_return.parse(&raw[316..328]);
        self.use_ch_strip_as_plugin.parse_quadlet(&raw[328..332]);
        self.use_reverb_at_mid_rate.parse_quadlet(&raw[332..336]);
        self.enabled.parse_quadlet(&raw[340..344]);
    }
}

impl TcKonnektSegmentSpec for TcKonnektSegment<K24dMixerState> {
    const OFFSET: usize = 0x0074;
    const SIZE: usize = ShellMixerState::SIZE + 32;
}

impl TcKonnektNotifiedSegmentSpec for TcKonnektSegment<K24dMixerState> {
    const NOTIFY_FLAG: u32 = SHELL_MIXER_NOTIFY_FLAG;
}

#[derive(Default, Debug)]
pub struct K24dReverbState(pub ReverbState);

impl TcKonnektSegmentData for K24dReverbState {
    fn build(&self, raw: &mut [u8]) {
        self.0.build(raw)
    }

    fn parse(&mut self, raw: &[u8]) {
        self.0.parse(raw)
    }
}

impl TcKonnektSegmentSpec for TcKonnektSegment<K24dReverbState> {
    const OFFSET: usize = 0x01d0;
    const SIZE: usize = ReverbState::SIZE;
}

impl TcKonnektNotifiedSegmentSpec for TcKonnektSegment<K24dReverbState> {
    const NOTIFY_FLAG: u32 = SHELL_REVERB_NOTIFY_FLAG;
}

#[derive(Default, Debug)]
pub struct K24dChStripStates(pub [ChStripState; SHELL_CH_STRIP_COUNT]);

impl TcKonnektSegmentData for K24dChStripStates {
    fn build(&self, raw: &mut [u8]) {
        self.0.build(raw)
    }

    fn parse(&mut self, raw: &[u8]) {
        self.0.parse(raw)
    }
}

impl TcKonnektSegmentSpec for TcKonnektSegment<K24dChStripStates> {
    const OFFSET: usize = 0x0218;
    const SIZE: usize = ChStripState::SIZE * SHELL_CH_STRIP_COUNT + 4;
}

impl TcKonnektNotifiedSegmentSpec for TcKonnektSegment<K24dChStripStates> {
    const NOTIFY_FLAG: u32 = SHELL_CH_STRIP_NOTIFY_FLAG;
}

#[derive(Default, Debug)]
pub struct K24dHwState(pub ShellHwState);

impl TcKonnektSegmentData for K24dHwState {
    fn build(&self, raw: &mut [u8]) {
        self.0.build(raw);
    }

    fn parse(&mut self, raw: &[u8]) {
        self.0.parse(raw);
    }
}

impl TcKonnektSegmentSpec for TcKonnektSegment<K24dHwState> {
    const OFFSET: usize = 0x100c;
    const SIZE: usize = ShellHwState::SIZE;
}

impl TcKonnektNotifiedSegmentSpec for TcKonnektSegment<K24dHwState> {
    const NOTIFY_FLAG: u32 = SHELL_HW_STATE_NOTIFY_FLAG;
}

const K24D_METER_ANALOG_INPUT_COUNT: usize = 2;
const K24D_METER_DIGITAL_INPUT_COUNT: usize = 2;

#[derive(Debug)]
pub struct K24dMixerMeter(pub ShellMixerMeter);

impl Default for K24dMixerMeter {
    fn default() -> Self {
        K24dMixerMeter(Self::create_meter_state())
    }
}

impl ShellMixerMeterConvert for K24dMixerMeter {
    const ANALOG_INPUT_COUNT: usize = K24D_METER_ANALOG_INPUT_COUNT;
    const DIGITAL_INPUT_COUNT: usize = K24D_METER_DIGITAL_INPUT_COUNT;

    fn meter(&self) -> &ShellMixerMeter {
        &self.0
    }

    fn meter_mut(&mut self) -> &mut ShellMixerMeter {
        &mut self.0
    }
}

impl TcKonnektSegmentData for K24dMixerMeter {
    fn build(&self, raw: &mut [u8]) {
        ShellMixerMeterConvert::build(self, raw)
    }

    fn parse(&mut self, raw: &[u8]) {
        ShellMixerMeterConvert::parse(self, raw)
    }
}

impl TcKonnektSegmentSpec for TcKonnektSegment<K24dMixerMeter> {
    const OFFSET: usize = 0x105c;
    const SIZE: usize = ShellMixerMeter::SIZE;
}

#[derive(Default, Debug)]
pub struct K24dReverbMeter(pub ReverbMeter);

impl TcKonnektSegmentData for K24dReverbMeter {
    fn build(&self, raw: &mut [u8]) {
        self.0.build(raw)
    }

    fn parse(&mut self, raw: &[u8]) {
        self.0.parse(raw)
    }
}

impl TcKonnektSegmentSpec for TcKonnektSegment<K24dReverbMeter> {
    const OFFSET: usize = 0x10b8;
    const SIZE: usize = ReverbMeter::SIZE;
}

#[derive(Default, Debug)]
pub struct K24dChStripMeters(pub [ChStripMeter; SHELL_CH_STRIP_COUNT]);

impl TcKonnektSegmentData for K24dChStripMeters {
    fn build(&self, raw: &mut [u8]) {
        self.0.build(raw)
    }

    fn parse(&mut self, raw: &[u8]) {
        self.0.parse(raw)
    }
}

impl TcKonnektSegmentSpec for TcKonnektSegment<K24dChStripMeters> {
    const OFFSET: usize = 0x10d0;
    const SIZE: usize = ChStripMeter::SIZE * SHELL_CH_STRIP_COUNT + 4;
}
