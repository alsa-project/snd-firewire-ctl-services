// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Protocol defined by TC Electronic for Konnekt Live.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by TC Electronic for Konnekt Live.
//!
//! ## Diagram of internal signal flow
//!
//! ```text
//!
//! XLR input 1 ----------------or----+--------------------------> analog-input-1/2
//! Phone input 1 --------------+     |
//!                                   |
//! XLR input 2 ----------------or----+
//! Phone input 2 --------------+
//!
//! Phone input 3/4  --------------------------------------------> analog-input-3/4
//!                               ++=========++
//! Coaxial input --------------> || digital ||
//!                               || input   || -----------------> digital-input-1..8
//! Optical input --------------> || select  ||
//!                               ++=========++
//!
//! analog-input-1/2 --------------------------------------------> stream-output-1/2
//! analog-input-3/4 --------------------------------------------> stream-output-3/4
//! channel-strip-effects-output-1/2 ----------------------------> stream-output-5/6
//! reverb-effect-output-1/2 ------------------------------------> stream-output-7/8
//! digital-input-1..8 ------------------------------------------> stream-output-9..16
//!
//!                                           ++============++
//! analog-input-1/2 ---+                     ||            ||
//! analog-input-3/4 ---+                     ||  channel   ||
//! digital-input-1/2 --+-- (one of them) --> ||   strip    || --> channel-strip-effect-output-1/2
//! digital-input-3/4 --+  (internal mode)    ||  effects   ||
//! digital-input-5/6 --+                     ||    1/2     ||
//! digital-input-7/8 --+                     ||            ||
//! stream-input-5/6  ----- (plugin mode) --> ++============++
//!
//!                                           ++============++
//! analog-input-1/2 --- (internal mode) ---> ||            ||
//! analog-input-3/4 --- (internal mode) ---> ||   reverb   ||
//! digital-input-1/2 -- (internal mode) ---> ||            ||
//! digital-input-3/4 -- (internal mode) ---> ||   effect   || --> reverb-effect-output-1/2
//! digital-input-5/6 -- (internal mode) ---> ||            ||
//! digital-input-7/8 -- (internal mode) ---> ||   12 x 2   ||
//! stream-input-7/8 --- (plugin mode) -----> ||            ||
//!                                           ++============++
//!
//!
//! stream-input-1/2 --------------(one of them) ----------------> stream-source-input-1/2
//! stream-input-3/4 ------------------+
//! stream-input-5/6 ------------------+
//! stream-input-7/8 ------------------+
//! stream-input-9/10 -----------------+
//! stream-input-11/12 ----------------+
//! stream-input-13/14 ----------------+
//! stream-input-15/16 ----------------+
//!                                      ++==========++
//! analog-input-1/2 -----------or-----> ||          ||
//! channel-effect-output-1/2 --+        ||          ||
//!                                      ||          ||
//! analog-input-3/4 -----------or-----> ||          ||
//! channel-effect-output-1/2 --+        ||          ||
//!                                      ||          ||
//! digital-input-1/2 ----------or-----> ||          ||
//! channel-effect-output-1/2 --+        ||  16 x 2  ||
//!                                      ||          ||
//! digital-input-3/4 ----------or-----> ||          || ---------> mixer-output-1/2
//! channel-effect-output-1/2 --+        ||  mixer   ||
//!                                      ||          ||
//! digital-input-5/6 ----------or-----> ||          ||
//! channel-effect-output-1/2 --+        ||          ||
//!                                      ||          ||
//! digital-input-7/8 ----------or-----> ||          ||
//! channel-effect-output-1/2 --+        ||          ||
//!                                      ||          ||
//! reverb-effect-output-1/2 ----------> ||          ||
//! stream-source-input-1/2 -----------> ||          ||
//!                                      ++==========++
//!
//!
//! stream-input-1/2 -------------(one of them) -----------------> analog-output-1/2
//! analog-input-1/2 ------------------+
//! mixer-output-1/2 ------------------+
//! reverb-source-1/2 -----------------+
//!
//! stream-input-1/2 -------------(one of them) -----------------> analog-output-3/4
//! analog-input-1/2 ------------------+
//! mixer-output-1/2 ------------------+
//! reverb-source-1/2 -----------------+
//!
//! stream-input-15/16 -----------(one of them) -----------------> coaxial-output-1/2
//! analog-input-1/2 ------------------+
//! mixer-output-1/2 ------------------+
//! reverb-source-1/2 -----------------+
//!
//! stream-input-9..16 -----------(one of them) -----------------> optical-output-9..16
//! analog-input-1/2 ------------------+
//! mixer-output-1/2 ------------------+
//! reverb-source-1/2 -----------------+
//!
//! ```

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
    /// Target of 1st knob.
    pub knob0_target: ShellKnob0Target,
    /// Target of 2nd knob.
    pub knob1_target: ShellKnob1Target,
    /// Loaded program number.
    pub prog: TcKonnektLoadedProgram,
    /// Impedance of outputs.
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
        serialize_output_impedance(&params.out_impedance[0], &mut raw[12..16])?;
        serialize_output_impedance(&params.out_impedance[1], &mut raw[16..20])?;
        Ok(())
    }

    fn deserialize(params: &mut KliveKnob, raw: &[u8]) -> Result<(), String> {
        deserialize_knob0_target::<KliveProtocol>(&mut params.knob0_target, &raw[..4])?;
        deserialize_knob1_target::<KliveProtocol>(&mut params.knob1_target, &raw[4..8])?;
        deserialize_loaded_program(&mut params.prog, &raw[8..12])?;
        deserialize_output_impedance(&mut params.out_impedance[0], &raw[12..16])?;
        deserialize_output_impedance(&mut params.out_impedance[1], &raw[16..20])?;
        Ok(())
    }
}

impl TcKonnektMutableSegmentOperation<KliveKnob> for KliveProtocol {}

impl TcKonnektNotifiedSegmentOperation<KliveKnob> for KliveProtocol {
    const NOTIFY_FLAG: u32 = SHELL_KNOB_NOTIFY_FLAG;
}

impl AsRef<TcKonnektLoadedProgram> for KliveKnob {
    fn as_ref(&self) -> &TcKonnektLoadedProgram {
        &self.prog
    }
}

impl AsMut<TcKonnektLoadedProgram> for KliveKnob {
    fn as_mut(&mut self) -> &mut TcKonnektLoadedProgram {
        &mut self.prog
    }
}

/// Configuration.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct KliveConfig {
    /// Configuration for optical interface.
    pub opt: ShellOptIfaceConfig,
    /// Source of coaxial output.
    pub coax_out_src: ShellCoaxOutPairSrc,
    /// Source of analog output 1/2.
    pub out_01_src: ShellPhysOutSrc,
    /// Source of analog output 3/4.
    pub out_23_src: ShellPhysOutSrc,
    /// Source of stream input pair as mixer source.
    pub mixer_stream_src_pair: ShellMixerStreamSourcePair,
    /// Source of sampling clock at standalone mode.
    pub standalone_src: ShellStandaloneClockSource,
    /// Rate of sampling clock at standalone mode.
    pub standalone_rate: TcKonnektStandaloneClockRate,
    /// Configuration for midi event generator.
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

impl AsRef<ShellStandaloneClockSource> for KliveConfig {
    fn as_ref(&self) -> &ShellStandaloneClockSource {
        &self.standalone_src
    }
}

impl AsMut<ShellStandaloneClockSource> for KliveConfig {
    fn as_mut(&mut self) -> &mut ShellStandaloneClockSource {
        &mut self.standalone_src
    }
}

impl AsRef<TcKonnektStandaloneClockRate> for KliveConfig {
    fn as_ref(&self) -> &TcKonnektStandaloneClockRate {
        &self.standalone_rate
    }
}

impl AsMut<TcKonnektStandaloneClockRate> for KliveConfig {
    fn as_mut(&mut self) -> &mut TcKonnektStandaloneClockRate {
        &mut self.standalone_rate
    }
}

impl AsRef<TcKonnektMidiSender> for KliveConfig {
    fn as_ref(&self) -> &TcKonnektMidiSender {
        &self.midi_sender
    }
}

impl AsMut<TcKonnektMidiSender> for KliveConfig {
    fn as_mut(&mut self) -> &mut TcKonnektMidiSender {
        &mut self.midi_sender
    }
}

/// The source of channel strip effect.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ChStripSrc {
    /// Stream input 1/2.
    Stream01,
    /// Analog input 1/2.
    Analog01,
    /// Analog input 3/4.
    Analog23,
    /// Digital input 1/2.
    Digital01,
    /// Digital input 3/4.
    Digital23,
    /// Digital input 5/6.
    Digital45,
    /// Digital input 7/8.
    Digital67,
    /// Mixer output 1/2.
    MixerOutput,
    /// Nothing.
    None,
}

impl Default for ChStripSrc {
    fn default() -> Self {
        ChStripSrc::None
    }
}

fn serialize_ch_strip_src(src: &ChStripSrc, raw: &mut [u8]) -> Result<(), String> {
    assert!(raw.len() >= 4);

    let val = match src {
        ChStripSrc::Stream01 => 0,
        ChStripSrc::Analog01 => 4,
        ChStripSrc::Analog23 => 5,
        ChStripSrc::Digital01 => 6,
        ChStripSrc::Digital23 => 7,
        ChStripSrc::Digital45 => 8,
        ChStripSrc::Digital67 => 9,
        ChStripSrc::MixerOutput => 10,
        ChStripSrc::None => 11,
    };

    serialize_u32(&val, raw);

    Ok(())
}

fn deserialize_ch_strip_src(src: &mut ChStripSrc, raw: &[u8]) -> Result<(), String> {
    assert!(raw.len() >= 4);

    let mut val = 0u32;
    deserialize_u32(&mut val, raw);

    *src = match val {
        0 => ChStripSrc::Stream01,
        4 => ChStripSrc::Analog01,
        5 => ChStripSrc::Analog23,
        6 => ChStripSrc::Digital01,
        7 => ChStripSrc::Digital23,
        8 => ChStripSrc::Digital45,
        9 => ChStripSrc::Digital67,
        10 => ChStripSrc::MixerOutput,
        _ => ChStripSrc::None,
    };

    Ok(())
}

/// The type of channel strip effect.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ChStripMode {
    /// Fablik C.
    FabrikC,
    /// RIAA 1964.
    RIAA1964,
    /// RIAA 1987.
    RIAA1987,
}

impl Default for ChStripMode {
    fn default() -> Self {
        ChStripMode::FabrikC
    }
}

const CH_STRIP_MODES: &[ChStripMode] = &[
    ChStripMode::FabrikC,
    ChStripMode::RIAA1964,
    ChStripMode::RIAA1987,
];

const CH_STRIP_MODE_LABEL: &str = "channel strip mode";

fn serialize_ch_strip_mode(mode: &ChStripMode, raw: &mut [u8]) -> Result<(), String> {
    serialize_position(CH_STRIP_MODES, mode, raw, CH_STRIP_MODE_LABEL)
}

fn deserialize_ch_strip_mode(mode: &mut ChStripMode, raw: &[u8]) -> Result<(), String> {
    deserialize_position(CH_STRIP_MODES, mode, raw, CH_STRIP_MODE_LABEL)
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
        serialize_bool(&params.use_ch_strip_as_plugin, &mut raw[328..332]);
        serialize_ch_strip_src(&params.ch_strip_src, &mut raw[332..336])?;
        serialize_ch_strip_mode(&params.ch_strip_mode, &mut raw[336..340])?;
        serialize_bool(&params.use_reverb_at_mid_rate, &mut raw[340..344]);
        serialize_bool(&params.enabled, &mut raw[344..348]);
        Ok(())
    }

    fn deserialize(params: &mut KliveMixerState, raw: &[u8]) -> Result<(), String> {
        deserialize_mixer_state::<KliveProtocol>(&mut params.mixer, raw)?;
        deserialize_reverb_return(&mut params.reverb_return, &raw[316..328])?;
        deserialize_bool(&mut params.use_ch_strip_as_plugin, &raw[328..332]);
        deserialize_ch_strip_src(&mut params.ch_strip_src, &raw[332..336])?;
        deserialize_ch_strip_mode(&mut params.ch_strip_mode, &raw[336..340])?;
        deserialize_bool(&mut params.use_reverb_at_mid_rate, &raw[340..344]);
        deserialize_bool(&mut params.enabled, &raw[344..348]);
        Ok(())
    }
}

impl TcKonnektMutableSegmentOperation<KliveMixerState> for KliveProtocol {}

impl TcKonnektNotifiedSegmentOperation<KliveMixerState> for KliveProtocol {
    const NOTIFY_FLAG: u32 = SHELL_MIXER_NOTIFY_FLAG;
}

impl AsRef<ShellMixerState> for KliveMixerState {
    fn as_ref(&self) -> &ShellMixerState {
        &self.mixer
    }
}

impl AsMut<ShellMixerState> for KliveMixerState {
    fn as_mut(&mut self) -> &mut ShellMixerState {
        &mut self.mixer
    }
}

impl AsRef<ShellReverbReturn> for KliveMixerState {
    fn as_ref(&self) -> &ShellReverbReturn {
        &self.reverb_return
    }
}

impl AsMut<ShellReverbReturn> for KliveMixerState {
    fn as_mut(&mut self) -> &mut ShellReverbReturn {
        &mut self.reverb_return
    }
}

/// Configuration for reverb effect.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct KliveReverbState(pub ReverbState);

impl TcKonnektSegmentSerdes<KliveReverbState> for KliveProtocol {
    const NAME: &'static str = "reverb-state";
    const OFFSET: usize = 0x0218;
    const SIZE: usize = ReverbState::SIZE;

    fn serialize(params: &KliveReverbState, raw: &mut [u8]) -> Result<(), String> {
        serialize_reverb_state(&params.0, raw)
    }

    fn deserialize(params: &mut KliveReverbState, raw: &[u8]) -> Result<(), String> {
        deserialize_reverb_state(&mut params.0, raw)
    }
}

impl TcKonnektMutableSegmentOperation<KliveReverbState> for KliveProtocol {}

impl TcKonnektNotifiedSegmentOperation<KliveReverbState> for KliveProtocol {
    const NOTIFY_FLAG: u32 = SHELL_REVERB_NOTIFY_FLAG;
}

impl AsRef<ReverbState> for KliveReverbState {
    fn as_ref(&self) -> &ReverbState {
        &self.0
    }
}

impl AsMut<ReverbState> for KliveReverbState {
    fn as_mut(&mut self) -> &mut ReverbState {
        &mut self.0
    }
}

/// Configuration for channel strip effect.
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

impl AsRef<[ChStripState]> for KliveChStripStates {
    fn as_ref(&self) -> &[ChStripState] {
        &self.0
    }
}

impl AsMut<[ChStripState]> for KliveChStripStates {
    fn as_mut(&mut self) -> &mut [ChStripState] {
        &mut self.0
    }
}

/// Hardware state.
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

impl AsRef<ShellHwState> for KliveHwState {
    fn as_ref(&self) -> &ShellHwState {
        &self.0
    }
}

impl AsMut<ShellHwState> for KliveHwState {
    fn as_mut(&mut self) -> &mut ShellHwState {
        &mut self.0
    }
}

impl AsRef<FireWireLedState> for KliveHwState {
    fn as_ref(&self) -> &FireWireLedState {
        &self.0.firewire_led
    }
}

impl AsMut<FireWireLedState> for KliveHwState {
    fn as_mut(&mut self) -> &mut FireWireLedState {
        &mut self.0.firewire_led
    }
}

const KLIVE_METER_ANALOG_INPUT_COUNT: usize = 4;
const KLIVE_METER_DIGITAL_INPUT_COUNT: usize = 8;

/// Hardware metering for mixer function.
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

impl AsRef<ShellMixerMeter> for KliveMixerMeter {
    fn as_ref(&self) -> &ShellMixerMeter {
        &self.0
    }
}

impl AsMut<ShellMixerMeter> for KliveMixerMeter {
    fn as_mut(&mut self) -> &mut ShellMixerMeter {
        &mut self.0
    }
}

/// Hardware metering for reverb effect.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct KliveReverbMeter(pub ReverbMeter);

impl TcKonnektSegmentSerdes<KliveReverbMeter> for KliveProtocol {
    const NAME: &'static str = "reverb-meter";
    const OFFSET: usize = 0x10c4;
    const SIZE: usize = ReverbMeter::SIZE;

    fn serialize(params: &KliveReverbMeter, raw: &mut [u8]) -> Result<(), String> {
        serialize_reverb_meter(&params.0, raw)
    }

    fn deserialize(params: &mut KliveReverbMeter, raw: &[u8]) -> Result<(), String> {
        deserialize_reverb_meter(&mut params.0, raw)
    }
}

impl AsRef<ReverbMeter> for KliveReverbMeter {
    fn as_ref(&self) -> &ReverbMeter {
        &self.0
    }
}

impl AsMut<ReverbMeter> for KliveReverbMeter {
    fn as_mut(&mut self) -> &mut ReverbMeter {
        &mut self.0
    }
}

/// Hardware metering for channel strip effect.
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

impl AsRef<[ChStripMeter]> for KliveChStripMeters {
    fn as_ref(&self) -> &[ChStripMeter] {
        &self.0
    }
}

impl AsMut<[ChStripMeter]> for KliveChStripMeters {
    fn as_mut(&mut self) -> &mut [ChStripMeter] {
        &mut self.0
    }
}

/// Impedance of output.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum OutputImpedance {
    /// Unbalance.
    Unbalance,
    /// Balance.
    Balance,
}

impl Default for OutputImpedance {
    fn default() -> Self {
        Self::Unbalance
    }
}

const OUTPUT_IMPEDANCES: &[OutputImpedance] =
    &[OutputImpedance::Unbalance, OutputImpedance::Balance];

const OUTPUT_IMPEDANCE_LABEL: &str = "output impedance";

fn serialize_output_impedance(impedance: &OutputImpedance, raw: &mut [u8]) -> Result<(), String> {
    serialize_position(OUTPUT_IMPEDANCES, impedance, raw, OUTPUT_IMPEDANCE_LABEL)
}

fn deserialize_output_impedance(impedance: &mut OutputImpedance, raw: &[u8]) -> Result<(), String> {
    deserialize_position(OUTPUT_IMPEDANCES, impedance, raw, OUTPUT_IMPEDANCE_LABEL)
}
