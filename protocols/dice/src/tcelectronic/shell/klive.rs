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

/// Segment for configuration. 0x0028..0x00ab (33 quads).
pub type KliveConfigSegment = TcKonnektSegment<KliveConfig>;

/// Segment for state of mixer. 0x00ac..0x0217 (91 quads).
pub type KliveMixerStateSegment = TcKonnektSegment<KliveMixerState>;

/// Segment for state of reverb effect. 0x0218..0x025b. (17 quads)
pub type KliveReverbStateSegment = TcKonnektSegment<KliveReverbState>;

/// Segment for states of channel strip effect. 0x0260..0x037f (72 quads).
pub type KliveChStripStatesSegment = TcKonnektSegment<KliveChStripStates>;

// NOTE: Segment for tuner. 0x0384..0x039c (8 quads).

/// Segment for mixer meter. 0x1068..0x10c3 (23 quads).
pub type KliveMixerMeterSegment = TcKonnektSegment<KliveMixerMeter>;

/// Segment for state of hardware. 0x1008..0x1023 (7 quads).
pub type KliveHwStateSegment = TcKonnektSegment<KliveHwState>;

/// Segment for meter of reverb effect. 0x10c4..0x010db (6 quads).
pub type KliveReverbMeterSegment = TcKonnektSegment<KliveReverbMeter>;

/// Segment for meters of channel strip effect. 0x10dc..0x1117 (15 quads).
pub type KliveChStripMetersSegment = TcKonnektSegment<KliveChStripMeters>;

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
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct KliveKnob {
    pub knob0_target: ShellKnob0Target,
    pub knob1_target: ShellKnob1Target,
    pub prog: TcKonnektLoadedProgram,
    pub out_impedance: [OutputImpedance; 2],
}

impl Default for KliveKnob {
    fn default() -> Self {
        Self {
            knob0_target: KliveProtocol::KNOB0_TARGETS[0],
            knob1_target: KliveProtocol::KNOB1_TARGETS[0],
            prog: Default::default(),
            out_impedance: Default::default(),
        }
    }
}

impl ShellKnob0TargetSpecification for KliveProtocol {
    const KNOB0_TARGETS: &'static [ShellKnob0Target] = &[
        ShellKnob0Target::Analog0,
        ShellKnob0Target::Analog1,
        ShellKnob0Target::Analog2_3,
        ShellKnob0Target::Configurable,
    ];
}

impl ShellKnob1TargetSpecification for KliveProtocol {
    const KNOB1_TARGETS: &'static [ShellKnob1Target] = &[
        ShellKnob1Target::Digital0_1,
        ShellKnob1Target::Digital2_3,
        ShellKnob1Target::Digital4_5,
        ShellKnob1Target::Digital6_7,
        ShellKnob1Target::Stream,
        ShellKnob1Target::Reverb,
        ShellKnob1Target::Mixer,
        ShellKnob1Target::TunerPitchTone,
        ShellKnob1Target::MidiSend,
    ];
}

impl TcKonnektSegmentSerdes<KliveKnob> for KliveProtocol {
    const NAME: &'static str = "knob";
    const OFFSET: usize = 0x0004;
    const SIZE: usize = SHELL_KNOB_SEGMENT_SIZE;

    fn serialize(params: &KliveKnob, raw: &mut [u8]) -> Result<(), String> {
        serialize_knob0_target::<KliveProtocol>(&params.knob0_target, &mut raw[..4])?;
        serialize_knob1_target::<KliveProtocol>(&params.knob1_target, &mut raw[4..8])?;
        serialize_loaded_program(&params.prog, &mut raw[8..12])?;
        params.out_impedance.build_quadlet_block(&mut raw[12..20]);
        Ok(())
    }

    fn deserialize(params: &mut KliveKnob, raw: &[u8]) -> Result<(), String> {
        deserialize_knob0_target::<KliveProtocol>(&mut params.knob0_target, &raw[..4])?;
        deserialize_knob1_target::<KliveProtocol>(&mut params.knob1_target, &raw[4..8])?;
        deserialize_loaded_program(&mut params.prog, &raw[8..12])?;
        params.out_impedance.parse_quadlet_block(&raw[12..20]);
        Ok(())
    }
}

impl TcKonnektMutableSegmentOperation<KliveKnob> for KliveProtocol {}

impl TcKonnektNotifiedSegmentOperation<KliveKnob> for KliveProtocol {
    const NOTIFY_FLAG: u32 = SHELL_KNOB_NOTIFY_FLAG;
}

/// Configuration.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct KliveConfig {
    pub opt: ShellOptIfaceConfig,
    pub coax_out_src: ShellCoaxOutPairSrc,
    pub out_01_src: ShellPhysOutSrc,
    pub out_23_src: ShellPhysOutSrc,
    pub mixer_stream_src_pair: ShellMixerStreamSourcePair,
    pub standalone_src: ShellStandaloneClockSource,
    pub standalone_rate: TcKonnektStandaloneClockRate,
    pub midi_sender: TcKonnektMidiSender,
}

impl ShellMixerStreamSourcePairSpecification for KliveProtocol {
    const MIXER_STREAM_SOURCE_PAIRS: &'static [ShellMixerStreamSourcePair] = &[
        ShellMixerStreamSourcePair::Stream0_1,
        ShellMixerStreamSourcePair::Stream2_3,
        ShellMixerStreamSourcePair::Stream4_5,
        ShellMixerStreamSourcePair::Stream6_7,
        ShellMixerStreamSourcePair::Stream8_9,
        ShellMixerStreamSourcePair::Stream10_11,
    ];
}

impl ShellStandaloneClockSpecification for KliveProtocol {
    const STANDALONE_CLOCK_SOURCES: &'static [ShellStandaloneClockSource] = &[
        ShellStandaloneClockSource::Optical,
        ShellStandaloneClockSource::Coaxial,
        ShellStandaloneClockSource::Internal,
    ];
}

impl TcKonnektSegmentSerdes<KliveConfig> for KliveProtocol {
    const NAME: &'static str = "configuration";
    const OFFSET: usize = 0x0028;
    const SIZE: usize = 132;

    fn serialize(params: &KliveConfig, raw: &mut [u8]) -> Result<(), String> {
        serialize_opt_iface_config(&params.opt, &mut raw[..12])?;
        serialize_coax_out_pair_source(&params.coax_out_src, &mut raw[12..16])?;
        serialize_phys_out_src(&params.out_01_src, &mut raw[16..20])?;
        serialize_phys_out_src(&params.out_23_src, &mut raw[20..24])?;
        serialize_mixer_stream_source_pair::<KliveProtocol>(
            &params.mixer_stream_src_pair,
            &mut raw[24..28],
        )?;
        serialize_standalone_clock_source::<KliveProtocol>(
            &params.standalone_src,
            &mut raw[28..32],
        )?;
        serialize_standalone_clock_rate(&params.standalone_rate, &mut raw[32..36])?;
        serialize_midi_sender(&params.midi_sender, &mut raw[84..120])?;
        Ok(())
    }

    fn deserialize(params: &mut KliveConfig, raw: &[u8]) -> Result<(), String> {
        deserialize_opt_iface_config(&mut params.opt, &raw[..12])?;
        deserialize_coax_out_pair_source(&mut params.coax_out_src, &raw[12..16])?;
        deserialize_phys_out_src(&mut params.out_01_src, &raw[16..20])?;
        deserialize_phys_out_src(&mut params.out_23_src, &raw[20..24])?;
        deserialize_mixer_stream_source_pair::<KliveProtocol>(
            &mut params.mixer_stream_src_pair,
            &raw[24..28],
        )?;
        deserialize_standalone_clock_source::<KliveProtocol>(
            &mut params.standalone_src,
            &raw[28..32],
        )?;
        deserialize_standalone_clock_rate(&mut params.standalone_rate, &raw[32..36])?;
        deserialize_midi_sender(&mut params.midi_sender, &raw[84..120])?;
        Ok(())
    }
}

impl TcKonnektMutableSegmentOperation<KliveConfig> for KliveProtocol {}

impl TcKonnektNotifiedSegmentOperation<KliveConfig> for KliveProtocol {
    const NOTIFY_FLAG: u32 = SHELL_CONFIG_NOTIFY_FLAG;
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
            mixer: KliveProtocol::create_mixer_state(),
            reverb_return: Default::default(),
            use_ch_strip_as_plugin: Default::default(),
            ch_strip_src: Default::default(),
            ch_strip_mode: Default::default(),
            use_reverb_at_mid_rate: Default::default(),
            enabled: Default::default(),
        }
    }
}

impl ShellMixerStateSpecification for KliveProtocol {
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

impl TcKonnektSegmentSerdes<KliveMixerState> for KliveProtocol {
    const NAME: &'static str = "mixer-state";
    const OFFSET: usize = 0x0004;
    const SIZE: usize = ShellMixerState::SIZE + 48;

    fn serialize(params: &KliveMixerState, raw: &mut [u8]) -> Result<(), String> {
        serialize_mixer_state::<KliveProtocol>(&params.mixer, raw)?;

        serialize_reverb_return(&params.reverb_return, &mut raw[316..328])?;
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
        deserialize_mixer_state::<KliveProtocol>(&mut params.mixer, raw)?;

        deserialize_reverb_return(&mut params.reverb_return, &raw[316..328])?;
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

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct KliveChStripStates(pub [ChStripState; SHELL_CH_STRIP_COUNT]);

impl TcKonnektSegmentSerdes<KliveChStripStates> for KliveProtocol {
    const NAME: &'static str = "channel-strip-state";
    const OFFSET: usize = 0x0260;
    const SIZE: usize = ChStripState::SIZE * SHELL_CH_STRIP_COUNT + 4;

    fn serialize(params: &KliveChStripStates, raw: &mut [u8]) -> Result<(), String> {
        serialize_ch_strip_states(&params.0, raw)
    }

    fn deserialize(params: &mut KliveChStripStates, raw: &[u8]) -> Result<(), String> {
        deserialize_ch_strip_states(&mut params.0, raw)
    }
}

impl TcKonnektMutableSegmentOperation<KliveChStripStates> for KliveProtocol {}

impl TcKonnektNotifiedSegmentOperation<KliveChStripStates> for KliveProtocol {
    const NOTIFY_FLAG: u32 = SHELL_CH_STRIP_NOTIFY_FLAG;
}

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct KliveHwState(pub ShellHwState);

impl TcKonnektSegmentSerdes<KliveHwState> for KliveProtocol {
    const NAME: &'static str = "hardware-state";
    const OFFSET: usize = 0x1008;
    const SIZE: usize = ShellHwState::SIZE;

    fn serialize(params: &KliveHwState, raw: &mut [u8]) -> Result<(), String> {
        serialize_hw_state(&params.0, raw)
    }

    fn deserialize(params: &mut KliveHwState, raw: &[u8]) -> Result<(), String> {
        deserialize_hw_state(&mut params.0, raw)
    }
}

impl TcKonnektMutableSegmentOperation<KliveHwState> for KliveProtocol {}

impl TcKonnektNotifiedSegmentOperation<KliveHwState> for KliveProtocol {
    const NOTIFY_FLAG: u32 = SHELL_HW_STATE_NOTIFY_FLAG;
}

const KLIVE_METER_ANALOG_INPUT_COUNT: usize = 4;
const KLIVE_METER_DIGITAL_INPUT_COUNT: usize = 8;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KliveMixerMeter(pub ShellMixerMeter);

impl Default for KliveMixerMeter {
    fn default() -> Self {
        KliveMixerMeter(KliveProtocol::create_meter_state())
    }
}

impl ShellMixerMeterSpecification for KliveProtocol {
    const ANALOG_INPUT_COUNT: usize = KLIVE_METER_ANALOG_INPUT_COUNT;
    const DIGITAL_INPUT_COUNT: usize = KLIVE_METER_DIGITAL_INPUT_COUNT;
}

impl TcKonnektSegmentSerdes<KliveMixerMeter> for KliveProtocol {
    const NAME: &'static str = "mixer-meter";
    const OFFSET: usize = 0x1068;
    const SIZE: usize = ShellMixerMeter::SIZE;

    fn serialize(params: &KliveMixerMeter, raw: &mut [u8]) -> Result<(), String> {
        serialize_mixer_meter::<KliveProtocol>(&params.0, raw)
    }

    fn deserialize(params: &mut KliveMixerMeter, raw: &[u8]) -> Result<(), String> {
        deserialize_mixer_meter::<KliveProtocol>(&mut params.0, raw)
    }
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

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct KliveChStripMeters(pub [ChStripMeter; SHELL_CH_STRIP_COUNT]);

impl TcKonnektSegmentSerdes<KliveChStripMeters> for KliveProtocol {
    const NAME: &'static str = "channel-strip-meter";
    const OFFSET: usize = 0x10dc;
    const SIZE: usize = ChStripMeter::SIZE * SHELL_CH_STRIP_COUNT + 4;

    fn serialize(params: &KliveChStripMeters, raw: &mut [u8]) -> Result<(), String> {
        serialize_ch_strip_meters(&params.0, raw)
    }

    fn deserialize(params: &mut KliveChStripMeters, raw: &[u8]) -> Result<(), String> {
        deserialize_ch_strip_meters(&mut params.0, raw)
    }
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
