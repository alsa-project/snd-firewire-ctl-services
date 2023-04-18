// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Protocol defined by TC Electronic for Konnekt 24d.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by TC Electronic for Konnekt 24d.
//!
//! ## Diagram of internal signal flow
//!
//! ```text
//!
//! XLR input 1 ----or--+
//! Phone input 1 --+   |
//!                     +--> analog-input-1/2 --------------------------> stream-output-1/2
//!                     |        |
//! XLR input 2 ----or--+        |
//! Phone input 2 --+            |
//!                              |
//! Phone input 3/4  -----------------------> analog-input-3/4 ---+-----> stream-output-3/4
//!                              |                  |             |
//!                        (internal mode)          |             |
//!                              v                  |             |
//!                         ++=========++           |             |
//!                         || channel ||           |             |
//!                    +--> || strip   || -------+-------------+--------> stream-output-5/6
//!                    |    || effects ||        |  |          |  |
//!           (plugin mode) ||   1/2   ||        |  |          |  |
//!                    |    ++=========++        |  |          |  |
//!                    |                         |  |          |  |
//!                    |        ++=========++    |  |          |  |
//! Coaxial input -----|------> || digital ||    |  |          |  |
//!                    |        || input   || ---------+-------------+--> stream-output-9..16
//! Optical input -----|------> || select  ||    |  |  |       |  |  |
//!                    |        ++=========++    |  |  |       |  |  |
//!                    |                     (internal mode)   |  |  |
//!                    |                         v  v  v       |  |  |
//!                    |                     ++============++  |  |  |
//!                    |                     ||   12 x 2   ||  |  |  |
//!                    |  +--(plugin mode)-> ||            || ----------> stream-output-7/8
//!                    |  |                  ||   reverb   ||  |  |  |
//!                    |  |                  ++============++  |  |  |
//!                    |  |                         v          v  v  v
//!                    |  |                     ++=====================++
//!                    |  |                     ||       16 x 2        ||
//! stream-input-1/2 --|--|-------------------> ||                     ||
//!                    |  |                     ||       mixer         ||
//!                    |  |                     ++=====================++
//!                    |  |                                v
//!                    |  |                         mixer-output-1/2 ---> analog-output-1/2
//!                    |  |
//! stream-input-3/4 --|--|---------------------------------------------> analog-output-3/4
//!                    |  |
//! stream-input-5/6 --+  |
//!                       |
//! stream-input-7/8 -----+
//!
//! stream-input-9..16 -------------------------------------------------> digital-output-1..8
//!
//! ```

use super::*;

/// Protocol implementation of Konnekt 24d.
#[derive(Default, Debug)]
pub struct K24dProtocol;

impl TcatOperation for K24dProtocol {}

impl TcatGlobalSectionSpecification for K24dProtocol {}

/// Segment for knob. 0x0000..0x0027 (36 quads).
pub type K24dKnobSegment = TcKonnektSegment<K24dKnob>;

/// Segment for configuration. 0x0028..0x0073 (76 quads).
pub type K24dConfigSegment = TcKonnektSegment<K24dConfig>;

/// Segment for state of mixer. 0x0074..0x01cf (87 quads).
pub type K24dMixerStateSegment = TcKonnektSegment<K24dMixerState>;

/// Segment for state of reverb effect. 0x01d0..0x0213. (17 quads)
pub type K24dReverbStateSegment = TcKonnektSegment<K24dReverbState>;

/// Segment for states of channel strip effect. 0x0218..0x0337 (72 quads).
pub type K24dChStripStatesSegment = TcKonnektSegment<K24dChStripStates>;

// NOTE: Segment for tuner. 0x0338..0x033b (8 quads).

/// Segment for mixer meter. 0x105c..0x10b7 (23 quads).
pub type K24dMixerMeterSegment = TcKonnektSegment<K24dMixerMeter>;

/// Segment for state of hardware. 0x100c..0x1027 (7 quads).
pub type K24dHwStateSegment = TcKonnektSegment<K24dHwState>;

/// Segment for meter of reverb effect. 0x10b8..0x010cf (6 quads).
pub type K24dReverbMeterSegment = TcKonnektSegment<K24dReverbMeter>;

/// Segment for meters of channel strip effect. 0x10d0..0x110b (15 quads).
pub type K24dChStripMetersSegment = TcKonnektSegment<K24dChStripMeters>;

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

segment_default!(K24dProtocol, K24dKnob);
segment_default!(K24dProtocol, K24dConfig);
segment_default!(K24dProtocol, K24dMixerState);
segment_default!(K24dProtocol, K24dReverbState);
segment_default!(K24dProtocol, K24dChStripStates);
segment_default!(K24dProtocol, K24dMixerMeter);
segment_default!(K24dProtocol, K24dHwState);
segment_default!(K24dProtocol, K24dReverbMeter);
segment_default!(K24dProtocol, K24dChStripMeters);

/// State of knob.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct K24dKnob {
    /// Target of 1st knob.
    pub knob0_target: ShellKnob0Target,
    /// Target of 2nd knob.
    pub knob1_target: ShellKnob1Target,
    /// Loaded program number.
    pub prog: TcKonnektLoadedProgram,
}

impl Default for K24dKnob {
    fn default() -> Self {
        Self {
            knob0_target: K24dProtocol::KNOB0_TARGETS[0],
            knob1_target: K24dProtocol::KNOB1_TARGETS[0],
            prog: Default::default(),
        }
    }
}

impl ShellKnob0TargetSpecification for K24dProtocol {
    const KNOB0_TARGETS: &'static [ShellKnob0Target] = &[
        ShellKnob0Target::Analog0,
        ShellKnob0Target::Analog1,
        ShellKnob0Target::Analog2_3,
        ShellKnob0Target::Configurable,
    ];
}

impl ShellKnob1TargetSpecification for K24dProtocol {
    const KNOB1_TARGETS: &'static [ShellKnob1Target] = &[
        ShellKnob1Target::Digital0_1,
        ShellKnob1Target::Digital2_3,
        ShellKnob1Target::Digital4_5,
        ShellKnob1Target::Digital6_7,
        ShellKnob1Target::Stream,
        ShellKnob1Target::Reverb,
        ShellKnob1Target::Mixer,
        ShellKnob1Target::TunerPitchTone,
    ];
}

impl TcKonnektSegmentSerdes<K24dKnob> for K24dProtocol {
    const NAME: &'static str = "knob";
    const OFFSET: usize = 0x0004;
    const SIZE: usize = SHELL_KNOB_SEGMENT_SIZE;

    fn serialize(params: &K24dKnob, raw: &mut [u8]) -> Result<(), String> {
        serialize_knob0_target::<K24dProtocol>(&params.knob0_target, &mut raw[..4])?;
        serialize_knob1_target::<K24dProtocol>(&params.knob1_target, &mut raw[4..8])?;
        serialize_loaded_program(&params.prog, &mut raw[8..12])?;
        Ok(())
    }

    fn deserialize(params: &mut K24dKnob, raw: &[u8]) -> Result<(), String> {
        deserialize_knob0_target::<K24dProtocol>(&mut params.knob0_target, &raw[..4])?;
        deserialize_knob1_target::<K24dProtocol>(&mut params.knob1_target, &raw[4..8])?;
        deserialize_loaded_program(&mut params.prog, &raw[8..12])?;
        Ok(())
    }
}

impl TcKonnektMutableSegmentOperation<K24dKnob> for K24dProtocol {}

impl TcKonnektNotifiedSegmentOperation<K24dKnob> for K24dProtocol {
    const NOTIFY_FLAG: u32 = SHELL_KNOB_NOTIFY_FLAG;
}

/// Configuration.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct K24dConfig {
    /// Configuration for optical interface.
    pub opt: ShellOptIfaceConfig,
    /// Source of coaxial output.
    pub coax_out_src: ShellCoaxOutPairSrc,
    /// Source of analog output 3/4.
    pub out_23_src: ShellPhysOutSrc,
    /// Source of sampling clock at standalone mode.
    pub standalone_src: ShellStandaloneClockSource,
    /// Rate of sampling clock at standalone mode.
    pub standalone_rate: TcKonnektStandaloneClockRate,
}

impl ShellStandaloneClockSpecification for K24dProtocol {
    const STANDALONE_CLOCK_SOURCES: &'static [ShellStandaloneClockSource] = &[
        ShellStandaloneClockSource::Optical,
        ShellStandaloneClockSource::Coaxial,
        ShellStandaloneClockSource::Internal,
    ];
}

impl TcKonnektSegmentSerdes<K24dConfig> for K24dProtocol {
    const NAME: &'static str = "configuration";
    const OFFSET: usize = 0x0028;
    const SIZE: usize = 76;

    fn serialize(params: &K24dConfig, raw: &mut [u8]) -> Result<(), String> {
        serialize_opt_iface_config(&params.opt, &mut raw[..12])?;
        serialize_coax_out_pair_source(&params.coax_out_src, &mut raw[12..16])?;
        serialize_phys_out_src(&params.out_23_src, &mut raw[16..20])?;
        serialize_standalone_clock_source::<K24dProtocol>(
            &params.standalone_src,
            &mut raw[20..24],
        )?;
        serialize_standalone_clock_rate(&params.standalone_rate, &mut raw[24..28])?;
        Ok(())
    }

    fn deserialize(params: &mut K24dConfig, raw: &[u8]) -> Result<(), String> {
        deserialize_opt_iface_config(&mut params.opt, &raw[..12])?;
        deserialize_coax_out_pair_source(&mut params.coax_out_src, &raw[12..16])?;
        deserialize_phys_out_src(&mut params.out_23_src, &raw[16..20])?;
        deserialize_standalone_clock_source::<K24dProtocol>(
            &mut params.standalone_src,
            &raw[20..24],
        )?;
        deserialize_standalone_clock_rate(&mut params.standalone_rate, &raw[24..28])?;
        Ok(())
    }
}

impl TcKonnektMutableSegmentOperation<K24dConfig> for K24dProtocol {}

impl TcKonnektNotifiedSegmentOperation<K24dConfig> for K24dProtocol {
    const NOTIFY_FLAG: u32 = SHELL_CONFIG_NOTIFY_FLAG;
}

impl AsRef<ShellStandaloneClockSource> for K24dConfig {
    fn as_ref(&self) -> &ShellStandaloneClockSource {
        &self.standalone_src
    }
}

impl AsMut<ShellStandaloneClockSource> for K24dConfig {
    fn as_mut(&mut self) -> &mut ShellStandaloneClockSource {
        &mut self.standalone_src
    }
}

impl AsRef<TcKonnektStandaloneClockRate> for K24dConfig {
    fn as_ref(&self) -> &TcKonnektStandaloneClockRate {
        &self.standalone_rate
    }
}

impl AsMut<TcKonnektStandaloneClockRate> for K24dConfig {
    fn as_mut(&mut self) -> &mut TcKonnektStandaloneClockRate {
        &mut self.standalone_rate
    }
}

/// State of mixer.
#[derive(Debug, Clone, PartialEq, Eq)]
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
            mixer: K24dProtocol::create_mixer_state(),
            reverb_return: Default::default(),
            use_ch_strip_as_plugin: Default::default(),
            use_reverb_at_mid_rate: Default::default(),
            enabled: Default::default(),
        }
    }
}

impl ShellMixerStateSpecification for K24dProtocol {
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
}

impl TcKonnektSegmentSerdes<K24dMixerState> for K24dProtocol {
    const NAME: &'static str = "mixer-state";
    const OFFSET: usize = 0x0074;
    const SIZE: usize = ShellMixerState::SIZE + 32;

    fn serialize(params: &K24dMixerState, raw: &mut [u8]) -> Result<(), String> {
        serialize_mixer_state::<K24dProtocol>(&params.mixer, raw)?;
        serialize_reverb_return(&params.reverb_return, &mut raw[316..328])?;
        serialize_bool(&params.use_ch_strip_as_plugin, &mut raw[328..332]);
        serialize_bool(&params.use_reverb_at_mid_rate, &mut raw[332..336]);
        serialize_bool(&params.enabled, &mut raw[340..344]);
        Ok(())
    }

    fn deserialize(params: &mut K24dMixerState, raw: &[u8]) -> Result<(), String> {
        deserialize_mixer_state::<K24dProtocol>(&mut params.mixer, raw)?;
        deserialize_reverb_return(&mut params.reverb_return, &raw[316..328])?;
        deserialize_bool(&mut params.use_ch_strip_as_plugin, &raw[328..332]);
        deserialize_bool(&mut params.use_reverb_at_mid_rate, &raw[332..336]);
        deserialize_bool(&mut params.enabled, &raw[340..344]);
        Ok(())
    }
}

impl TcKonnektMutableSegmentOperation<K24dMixerState> for K24dProtocol {}

impl TcKonnektNotifiedSegmentOperation<K24dMixerState> for K24dProtocol {
    const NOTIFY_FLAG: u32 = SHELL_MIXER_NOTIFY_FLAG;
}

impl AsRef<ShellMixerState> for K24dMixerState {
    fn as_ref(&self) -> &ShellMixerState {
        &self.mixer
    }
}

impl AsMut<ShellMixerState> for K24dMixerState {
    fn as_mut(&mut self) -> &mut ShellMixerState {
        &mut self.mixer
    }
}

/// Configuration for reverb effect.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct K24dReverbState(pub ReverbState);

impl TcKonnektSegmentSerdes<K24dReverbState> for K24dProtocol {
    const NAME: &'static str = "reverb-state";
    const OFFSET: usize = 0x01d0;
    const SIZE: usize = ReverbState::SIZE;

    fn serialize(params: &K24dReverbState, raw: &mut [u8]) -> Result<(), String> {
        serialize_reverb_state(&params.0, raw)
    }

    fn deserialize(params: &mut K24dReverbState, raw: &[u8]) -> Result<(), String> {
        deserialize_reverb_state(&mut params.0, raw)
    }
}

impl TcKonnektMutableSegmentOperation<K24dReverbState> for K24dProtocol {}

impl TcKonnektNotifiedSegmentOperation<K24dReverbState> for K24dProtocol {
    const NOTIFY_FLAG: u32 = SHELL_REVERB_NOTIFY_FLAG;
}

impl AsRef<ReverbState> for K24dReverbState {
    fn as_ref(&self) -> &ReverbState {
        &self.0
    }
}

impl AsMut<ReverbState> for K24dReverbState {
    fn as_mut(&mut self) -> &mut ReverbState {
        &mut self.0
    }
}

/// Configuration for channel strip effect.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct K24dChStripStates(pub [ChStripState; SHELL_CH_STRIP_COUNT]);

impl TcKonnektSegmentSerdes<K24dChStripStates> for K24dProtocol {
    const NAME: &'static str = "channel-strip-state";
    const OFFSET: usize = 0x0218;
    const SIZE: usize = ChStripState::SIZE * SHELL_CH_STRIP_COUNT + 4;

    fn serialize(params: &K24dChStripStates, raw: &mut [u8]) -> Result<(), String> {
        serialize_ch_strip_states(&params.0, raw)
    }

    fn deserialize(params: &mut K24dChStripStates, raw: &[u8]) -> Result<(), String> {
        deserialize_ch_strip_states(&mut params.0, raw)
    }
}

impl TcKonnektMutableSegmentOperation<K24dChStripStates> for K24dProtocol {}

impl TcKonnektNotifiedSegmentOperation<K24dChStripStates> for K24dProtocol {
    const NOTIFY_FLAG: u32 = SHELL_CH_STRIP_NOTIFY_FLAG;
}

impl AsRef<[ChStripState]> for K24dChStripStates {
    fn as_ref(&self) -> &[ChStripState] {
        &self.0
    }
}

impl AsMut<[ChStripState]> for K24dChStripStates {
    fn as_mut(&mut self) -> &mut [ChStripState] {
        &mut self.0
    }
}

/// Hardware state.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct K24dHwState(pub ShellHwState);

impl TcKonnektSegmentSerdes<K24dHwState> for K24dProtocol {
    const NAME: &'static str = "hardware-state";
    const OFFSET: usize = 0x100c;
    const SIZE: usize = ShellHwState::SIZE;

    fn serialize(params: &K24dHwState, raw: &mut [u8]) -> Result<(), String> {
        serialize_hw_state(&params.0, raw)
    }

    fn deserialize(params: &mut K24dHwState, raw: &[u8]) -> Result<(), String> {
        deserialize_hw_state(&mut params.0, raw)
    }
}

impl TcKonnektMutableSegmentOperation<K24dHwState> for K24dProtocol {}

impl TcKonnektNotifiedSegmentOperation<K24dHwState> for K24dProtocol {
    const NOTIFY_FLAG: u32 = SHELL_HW_STATE_NOTIFY_FLAG;
}

impl AsRef<ShellHwState> for K24dHwState {
    fn as_ref(&self) -> &ShellHwState {
        &self.0
    }
}

impl AsMut<ShellHwState> for K24dHwState {
    fn as_mut(&mut self) -> &mut ShellHwState {
        &mut self.0
    }
}

impl AsRef<FireWireLedState> for K24dHwState {
    fn as_ref(&self) -> &FireWireLedState {
        &self.0.firewire_led
    }
}

impl AsMut<FireWireLedState> for K24dHwState {
    fn as_mut(&mut self) -> &mut FireWireLedState {
        &mut self.0.firewire_led
    }
}

const K24D_METER_ANALOG_INPUT_COUNT: usize = 2;
const K24D_METER_DIGITAL_INPUT_COUNT: usize = 2;

/// Hardware metering for mixer function.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct K24dMixerMeter(pub ShellMixerMeter);

impl Default for K24dMixerMeter {
    fn default() -> Self {
        K24dMixerMeter(K24dProtocol::create_meter_state())
    }
}

impl ShellMixerMeterSpecification for K24dProtocol {
    const ANALOG_INPUT_COUNT: usize = K24D_METER_ANALOG_INPUT_COUNT;
    const DIGITAL_INPUT_COUNT: usize = K24D_METER_DIGITAL_INPUT_COUNT;
}

impl TcKonnektSegmentSerdes<K24dMixerMeter> for K24dProtocol {
    const NAME: &'static str = "mixer-meter";
    const OFFSET: usize = 0x105c;
    const SIZE: usize = ShellMixerMeter::SIZE;

    fn serialize(params: &K24dMixerMeter, raw: &mut [u8]) -> Result<(), String> {
        serialize_mixer_meter::<K24dProtocol>(&params.0, raw)
    }

    fn deserialize(params: &mut K24dMixerMeter, raw: &[u8]) -> Result<(), String> {
        deserialize_mixer_meter::<K24dProtocol>(&mut params.0, raw)
    }
}

impl AsRef<ShellMixerMeter> for K24dMixerMeter {
    fn as_ref(&self) -> &ShellMixerMeter {
        &self.0
    }
}

impl AsMut<ShellMixerMeter> for K24dMixerMeter {
    fn as_mut(&mut self) -> &mut ShellMixerMeter {
        &mut self.0
    }
}

/// Hardware metering for reverb effect.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct K24dReverbMeter(pub ReverbMeter);

impl TcKonnektSegmentSerdes<K24dReverbMeter> for K24dProtocol {
    const NAME: &'static str = "reverb-meter";
    const OFFSET: usize = 0x10b8;
    const SIZE: usize = ReverbMeter::SIZE;

    fn serialize(params: &K24dReverbMeter, raw: &mut [u8]) -> Result<(), String> {
        serialize_reverb_meter(&params.0, raw)
    }

    fn deserialize(params: &mut K24dReverbMeter, raw: &[u8]) -> Result<(), String> {
        deserialize_reverb_meter(&mut params.0, raw)
    }
}

impl AsRef<ReverbMeter> for K24dReverbMeter {
    fn as_ref(&self) -> &ReverbMeter {
        &self.0
    }
}

impl AsMut<ReverbMeter> for K24dReverbMeter {
    fn as_mut(&mut self) -> &mut ReverbMeter {
        &mut self.0
    }
}

/// Hardware metering for channel strip effect.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct K24dChStripMeters(pub [ChStripMeter; SHELL_CH_STRIP_COUNT]);

impl TcKonnektSegmentSerdes<K24dChStripMeters> for K24dProtocol {
    const NAME: &'static str = "channel-strip-meter";
    const OFFSET: usize = 0x10d0;
    const SIZE: usize = ChStripMeter::SIZE * SHELL_CH_STRIP_COUNT + 4;

    fn serialize(params: &K24dChStripMeters, raw: &mut [u8]) -> Result<(), String> {
        serialize_ch_strip_meters(&params.0, raw)
    }

    fn deserialize(params: &mut K24dChStripMeters, raw: &[u8]) -> Result<(), String> {
        deserialize_ch_strip_meters(&mut params.0, raw)
    }
}

impl AsRef<[ChStripMeter]> for K24dChStripMeters {
    fn as_ref(&self) -> &[ChStripMeter] {
        &self.0
    }
}

impl AsMut<[ChStripMeter]> for K24dChStripMeters {
    fn as_mut(&mut self) -> &mut [ChStripMeter] {
        &mut self.0
    }
}
