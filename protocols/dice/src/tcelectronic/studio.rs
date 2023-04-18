// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Protocol defined by TC Electronic for Studio Konnekt 48.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by TC Electronic for Studio Konnekt 48.
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
//! XLR input 3 ----------------or----+--------------------------> analog-input-3/4
//! Phone input 3 --------------+     |
//!                                   |
//! XLR input 4 ----------------or----+
//! Phone input 4 --------------+
//!
//! Phone input 5/6 ---------------------------------------------> analog-input-5/6
//! Phone input 7/8 ---------------------------------------------> analog-input-7/8
//! Phone input 9/10 --------------------------------------------> analog-input-9/10
//!
//! Phone input 11/12 ----------+--------------------------------> analog-input-11/12
//! Mic in remote controller ---+
//!
//! Coaxial input 1/2 -------------------------------------------> coaxial-input-1/2
//!
//! Optical input 1 ------------+--------------------------------> optical-input-1..8
//! Optical input 2 ------------+
//!
//!
//! analog-input-1/2 --------------------------------------------> stream-output-A-1/2
//! analog-input-3/4 --------------------------------------------> stream-output-A-3/4
//! analog-input-5/6 --------------------------------------------> stream-output-A-5/6
//! analog-input-7/8 --------------------------------------------> stream-output-A-7/8
//! analog-input-9/10 -------------------------------------------> stream-output-A-9/10
//! analog-input-11/12 ------------------------------------------> stream-output-A-11/12
//! (blank) -----------------------------------------------------> stream-output-A-13/14
//! coaxial-input-1/2 -------------------------------------------> stream-output-A-15/16
//! optical-input-1..8 ------------------------------------------> stream-output-B-1..8
//! channel-strip-effect-output-1/2 -----------------------------> stream-output-B-9/10
//! channel-strip-effect-output-3/4 -----------------------------> stream-output-B-11/12
//! reverb-effect-output-1/2 ------------------------------------> stream-output-B-13/14
//! aux-output-1/2 ----------------------------------------------> stream-output-B-15/16
//!
//!
//!                                            ++============++
//! analog-input-1/2 ----+                     ||            ||
//! analog-input-3/4 ----+                     ||            ||
//! analog-input-5/6 ----+                     ||            ||
//! analog-input-7/8 ----+                     ||            ||
//! analog-input-9/10 ---+                     ||  channel   ||
//! analog-input-11/12 --+-- (one of them) --> ||   strip    || --> channel-strip-effect-output-1/2
//! coaxial-input-1/2 ---+  (internal mode)    ||  effects   ||     (can replace original signal)
//! optical-input-1/2 ---+                     ||    1/2     ||
//! optical-input-3/4 ---+                     ||            ||
//! optical-input-5/6 ---+                     ||            ||
//! optical-input-7/8 ---+                     ||            ||
//! stream-input-B-9/10 ---- (plugin mode) --> ||            ||
//!                                            ++============++
//!
//!                                            ++============++
//! analog-input-1/2 ----+                     ||            ||
//! analog-input-3/4 ----+                     ||            ||
//! analog-input-5/6 ----+                     ||            ||
//! analog-input-7/8 ----+                     ||            ||
//! analog-input-9/10 ---+                     ||  channel   ||
//! analog-input-11/12 --+-- (one of them) --> ||   strip    || --> channel-strip-effect-output-3/4
//! coaxial-input-1/2 ---+  (internal mode)    ||  effects   ||     (can replace original signal)
//! optical-input-1/2 ---+                     ||    3/4     ||
//! optical-input-3/4 ---+                     ||            ||
//! optical-input-5/6 ---+                     ||            ||
//! optical-input-7/8 ---+                     ||            ||
//! stream-input-B-11/12 --- (plugin mode) --> ||            ||
//!                                            ++============++
//!
//!                                            ++============++
//! analog-input-1/2 ------------------------> ||            ||
//! analog-input-3/4 ------------------------> ||            ||
//! analog-input-5/6 ------------------------> ||            ||
//! analog-input-7/8 ------------------------> ||            ||
//! analog-input-9/10 -----------------------> ||            ||
//! analog-input-11/12 ----------------------> ||            ||
//! coaxial-input-1/2 -----------------------> ||            || --> mixer-source-1/2
//! optical-input-1/2 -----------------------> ||            || --> mixer-source-3/4
//! optical-input-3/4 -----------------------> ||            || --> mixer-source-5/6
//! optical-input-5/6 -----------------------> ||            || --> mixer-source-7/8
//! optical-input-7/8 -----------------------> ||  44 x 24   || --> mixer-source-9/10
//!                                            ||            || --> mixer-source-11/12
//! stream-input-A-1/2 ----------------------> ||            || --> mixer-source-13/14
//! stream-input-A-3/4 ----------------------> ||   router   || --> mixer-source-15/16
//! stream-input-A-5/6 ----------------------> ||            || --> mixer-source-17/18
//! stream-input-A-7/8 ----------------------> ||            || --> mixer-source-19/20
//! stream-input-A-9/10 ---------------------> ||            || --> mixer-source-21/22
//! stream-input-A-11/12 --------------------> ||            || --> mixer-source-23/24
//! stream-input-A-13/14 (unused)              ||            ||
//! stream-input-A-15/16 --------------------> ||            ||
//!                                            ||            ||
//! stream-input-B-1/2 ----------------------> ||            ||
//! stream-input-B-3/4 ----------------------> ||            ||
//! stream-input-B-5/6 ----------------------> ||            ||
//! stream-input-B-7/8 ----------------------> ||            ||
//!                                            ++============++
//!
//!                                            ++============++
//! mixer-source-1/2 ----- (internal mode) --> ||            ||
//! mixer-source-3/4 ----- (internal mode) --> ||            ||
//! mixer-source-5/6 ----- (internal mode) --> ||            ||
//! mixer-source-7/8 ----- (internal mode) --> ||            ||
//! mixer-source-9/10 ---- (internal mode) --> ||   24 x 2   ||
//! mixer-source-11/12 --- (internal mode) --> ||            ||
//! mixer-source-13/14 --- (internal mode) --> ||   reverb   || --> reverb-effect-output-1/2
//! mixer-source-15/16 --- (internal mode) --> ||            ||
//! mixer-source-17/18 --- (internal mode) --> ||   effect   ||
//! mixer-source-19/20 --- (internal mode) --> ||            ||
//! mixer-source-21/22 --- (internal mode) --> ||            ||
//! mixer-source-23/24 --- (internal mode) --> ||            ||
//! stream-input-B-13/14 --(plugin mode) ----> ||            ||
//!                                            ++============++
//!
//!                                            ++============++
//! mixer-source-1/2 ------------------------> ||            ||
//! mixer-source-3/4 ------------------------> ||            ||
//! mixer-source-5/6 ------------------------> ||            ||
//! mixer-source-7/8 ------------------------> ||   24 x 2   ||
//! mixer-source-9/10 -----------------------> ||            ||
//! mixer-source-11/12 ----------------------> ||   main     ||
//! mixer-source-13/14 ----------------------> ||            || --> aux-output-3/4
//! mixer-source-15/16 ----------------------> ||   mixer    ||
//! mixer-source-17/18 ----------------------> ||            ||
//! mixer-source-19/20 ----------------------> ||    3/4     ||
//! mixer-source-21/22 ----------------------> ||            ||
//! mixer-source-23/24 ----------------------> ||            ||
//! reverb-effect-output-1/2 ----------------> ||            ||
//!                                            ++============++
//!
//!                                            ++============++
//! mixer-source-1/2 ------------------------> ||            ||
//! mixer-source-3/4 ------------------------> ||            ||
//! mixer-source-5/6 ------------------------> ||            ||
//! mixer-source-7/8 ------------------------> ||   24 x 2   ||
//! mixer-source-9/10 -----------------------> ||            ||
//! mixer-source-11/12 ----------------------> || auxiliary  ||
//! mixer-source-13/14 ----------------------> ||            || --> aux-output-1/2
//! mixer-source-15/16 ----------------------> ||   mixer    ||
//! mixer-source-17/18 ----------------------> ||            ||
//! mixer-source-19/20 ----------------------> ||    1/2     ||
//! mixer-source-21/22 ----------------------> ||            ||
//! mixer-source-23/24 ----------------------> ||            ||
//! reverb-effect-output-1/2 ----------------> ||            ||
//!                                            ++============++
//!
//!                                            ++============++
//! mixer-source-1/2 ------------------------> ||            ||
//! mixer-source-3/4 ------------------------> ||            ||
//! mixer-source-5/6 ------------------------> ||            ||
//! mixer-source-7/8 ------------------------> ||   24 x 2   ||
//! mixer-source-9/10 -----------------------> ||            ||
//! mixer-source-11/12 ----------------------> || auxiliary  ||
//! mixer-source-13/14 ----------------------> ||            || --> aux-output-3/4
//! mixer-source-15/16 ----------------------> ||   mixer    ||
//! mixer-source-17/18 ----------------------> ||            ||
//! mixer-source-19/20 ----------------------> ||    3/4     ||
//! mixer-source-21/22 ----------------------> ||            ||
//! mixer-source-23/24 ----------------------> ||            ||
//! reverb-effect-output-1/2 ----------------> ||            ||
//!                                            ++============++
//!
//!                                            ++==========++
//! analog-input-1/2 ------------------------> ||          ||
//! analog-input-3/4 ------------------------> ||          ||
//! analog-input-5/6 ------------------------> ||          ||
//! analog-input-7/8 ------------------------> ||          ||
//! analog-input-9/10 -----------------------> ||          ||
//! analog-input-11/12 ----------------------> ||          ||
//! coaxial-input-1/2 -----------------------> ||          ||
//! optical-input-1/2 -----------------------> ||          ||
//! optical-input-3/4 -----------------------> ||          || --> analog-output-1/2
//! optical-input-5/6 -----------------------> ||          || --> headphone-output-1/2
//! optical-input-7/8 -----------------------> || 54 x 24  || --> analog-output-5/6
//!                                            ||          || --> analog-output-7/8
//! stream-input-A-1/2 ----------------------> ||          || --> analog-output-9/10
//! stream-input-A-3/4 ----------------------> ||  router  || --> analog-output-11/12
//! stream-input-A-5/6 ----------------------> ||          || --> headphone-output-3/4
//! stream-input-A-7/8 ----------------------> ||          || --> coaxial-output-1/2
//! stream-input-A-9/10 ---------------------> ||          || --> coaxial-output-1/2
//! stream-input-A-11/12 --------------------> ||          || --> optical-output-1..8
//! stream-input-A-13/14 (unused)              ||          ||
//! stream-input-A-15/16 --------------------> ||          ||
//!                                            ||          ||
//! stream-input-B-1/2 ----------------------> ||          ||
//! stream-input-B-3/4 ----------------------> ||          ||
//! stream-input-B-5/6 ----------------------> ||          ||
//! stream-input-B-7/8 ----------------------> ||          ||
//!                                            ||          ||
//! mixer-output-1/2 ------------------------> ||          ||
//! aux-output-1/2 --------------------------> ||          ||
//! aux-output-3/4 --------------------------> ||          ||
//! reverb-output-1/2 -----------------------> ||          ||
//!                                            ++==========++
//!
//! ```

use super::{ch_strip::*, reverb::*, *};

/// Protocol implementation of Studio Konnekt 48.
#[derive(Default, Debug)]
pub struct Studiok48Protocol;

impl TcatOperation for Studiok48Protocol {}

impl TcatGlobalSectionSpecification for Studiok48Protocol {}

/// Segment for output level. 0x0000..0x0013 (4 quads).
pub type Studiok48LineOutLevelSegment = TcKonnektSegment<StudioLineOutLevel>;

/// Segment for remote controller. 0x0014..0x0043 (12 quads).
pub type Studiok48RemoteSegment = TcKonnektSegment<StudioRemote>;

/// Segment for configuration. 0x0044..0x00a7 (25 quads).
pub type Studiok48ConfigSegment = TcKonnektSegment<StudioConfig>;

/// Segment for state of mixer. 0x00a8..0x03db (205 quads).
pub type Studiok48MixerStateSegment = TcKonnektSegment<StudioMixerState>;

/// Segment for physical output. 0x03dc..0x0593 (110 quads).
pub type Studiok48PhysOutSegment = TcKonnektSegment<StudioPhysOut>;

/// Segment for state of reverb effect. 0x0594..0x05d7. (17 quads)
pub type Studiok48ReverbStateSegment = TcKonnektSegment<StudioReverbState>;

/// Segment for states of channel strip effect. 0x05dc..0x081f (145 quads).
pub type Studiok48ChStripStatesSegment = TcKonnektSegment<StudioChStripStates>;

// NOTE: Segment for tuner. 0x0820..0x083f (8 quads).

/// Segment for state of hardware. 0x2008..0x204b (17 quads).
pub type Studiok48HwStateSegment = TcKonnektSegment<StudioHwState>;

// NOTE: Segment for meter of remote controller. 0x204c..0x205b (4 quads).

/// Segment for meter of mixer. 0x20b8..0x2137 (32 quads).
pub type Studiok48MixerMeterSegment = TcKonnektSegment<StudioMixerMeter>;

// NOTE: Segment for inidentified meter. 0x2138..0x2163 (11 quads).

/// Segment for meter of reverb effect. 0x2164..0x217b (6 quads).
pub type Studiok48ReverbMeterSegment = TcKonnektSegment<StudioReverbMeter>;

/// Segment for meters of channel strip effect. 0x217c..0x21b7 (30 quads).
pub type Studiok48ChStripMetersSegment = TcKonnektSegment<StudioChStripMeters>;

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

segment_default!(Studiok48Protocol, StudioLineOutLevel);
segment_default!(Studiok48Protocol, StudioRemote);
segment_default!(Studiok48Protocol, StudioConfig);
segment_default!(Studiok48Protocol, StudioMixerState);
segment_default!(Studiok48Protocol, StudioPhysOut);
segment_default!(Studiok48Protocol, StudioReverbState);
segment_default!(Studiok48Protocol, StudioChStripStates);
segment_default!(Studiok48Protocol, StudioHwState);
segment_default!(Studiok48Protocol, StudioMixerMeter);
segment_default!(Studiok48Protocol, StudioReverbMeter);
segment_default!(Studiok48Protocol, StudioChStripMeters);

const STUDIO_LINE_OUT_LEVEL_NOTIFY_FLAG: u32 = 0x00010000;
const STUDIO_REMOTE_NOTIFY_FLAG: u32 = 0x00020000;
const STUDIO_CONFIG_NOTIFY_FLAG: u32 = 0x00040000;
const STUDIO_MIXER_STATE_NOTIFY_FLAG: u32 = 0x00080000;
const STUDIO_PHYS_OUT_NOTIFY_FLAG: u32 = 0x00100000;
const STUDIO_REVERB_NOTIFY_CHANGE: u32 = 0x00200000;
const STUDIO_CH_STRIP_NOTIFY_01_CHANGE: u32 = 0x00400000;
const STUDIO_CH_STRIP_NOTIFY_23_CHANGE: u32 = 0x00800000;
// NOTE: 0x01000000 is for tuner.
// NOTE: 0x02000000 is unidentified.
const STUDIO_HW_STATE_NOTIFY_FLAG: u32 = 0x04000000;
// NOTE: 0x08000000 is for remote controller.

/// Line output level.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum NominalSignalLevel {
    /// +4dBu.
    Professional,
    /// -10dBV.
    Consumer,
}

impl Default for NominalSignalLevel {
    fn default() -> Self {
        NominalSignalLevel::Professional
    }
}

const NOMINAL_LEVELS: &[NominalSignalLevel] = &[
    NominalSignalLevel::Professional,
    NominalSignalLevel::Consumer,
];

const NOMINAL_LEVEL_LABEL: &str = "nominal level";

fn serialize_nominal_level(level: &NominalSignalLevel, raw: &mut [u8]) -> Result<(), String> {
    serialize_position(NOMINAL_LEVELS, level, raw, NOMINAL_LEVEL_LABEL)
}

fn deserialize_nominal_level(level: &mut NominalSignalLevel, raw: &[u8]) -> Result<(), String> {
    deserialize_position(NOMINAL_LEVELS, level, raw, NOMINAL_LEVEL_LABEL)
}

/// Line output levels.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct StudioLineOutLevel {
    pub line_45: NominalSignalLevel,
    pub line_67: NominalSignalLevel,
    pub line_89: NominalSignalLevel,
    pub line_1011: NominalSignalLevel,
}

impl TcKonnektSegmentSerdes<StudioLineOutLevel> for Studiok48Protocol {
    const NAME: &'static str = "line-output-level";
    const OFFSET: usize = 0x0000;
    const SIZE: usize = 20;

    fn serialize(params: &StudioLineOutLevel, raw: &mut [u8]) -> Result<(), String> {
        serialize_nominal_level(&params.line_45, &mut raw[4..8])?;
        serialize_nominal_level(&params.line_67, &mut raw[8..12])?;
        serialize_nominal_level(&params.line_89, &mut raw[12..16])?;
        serialize_nominal_level(&params.line_1011, &mut raw[16..20])?;
        Ok(())
    }

    fn deserialize(params: &mut StudioLineOutLevel, raw: &[u8]) -> Result<(), String> {
        deserialize_nominal_level(&mut params.line_45, &raw[4..8])?;
        deserialize_nominal_level(&mut params.line_67, &raw[8..12])?;
        deserialize_nominal_level(&mut params.line_89, &raw[12..16])?;
        deserialize_nominal_level(&mut params.line_1011, &raw[16..20])?;
        Ok(())
    }
}

impl TcKonnektMutableSegmentOperation<StudioLineOutLevel> for Studiok48Protocol {}

impl TcKonnektNotifiedSegmentOperation<StudioLineOutLevel> for Studiok48Protocol {
    const NOTIFY_FLAG: u32 = STUDIO_LINE_OUT_LEVEL_NOTIFY_FLAG;
}

/// Mode of remote effect button.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum RemoteEffectButtonMode {
    /// For reverb effect.
    Reverb,
    /// For MIDI message generator.
    Midi,
}

impl Default for RemoteEffectButtonMode {
    fn default() -> Self {
        Self::Reverb
    }
}

const REMOTE_EFFECT_BUTTON_MODES: &[RemoteEffectButtonMode] =
    &[RemoteEffectButtonMode::Reverb, RemoteEffectButtonMode::Midi];

const REMOTE_EFFECT_BUTTON_MODE_LABEL: &str = "remote effect button mode";

fn serialize_remote_effect_button_mode(
    mode: &RemoteEffectButtonMode,
    raw: &mut [u8],
) -> Result<(), String> {
    serialize_position(
        REMOTE_EFFECT_BUTTON_MODES,
        mode,
        raw,
        REMOTE_EFFECT_BUTTON_MODE_LABEL,
    )
}

fn deserialize_remote_effect_button_mode(
    mode: &mut RemoteEffectButtonMode,
    raw: &[u8],
) -> Result<(), String> {
    deserialize_position(
        REMOTE_EFFECT_BUTTON_MODES,
        mode,
        raw,
        REMOTE_EFFECT_BUTTON_MODE_LABEL,
    )
}

/// Mode of knob target at pushed state.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum KnobPushMode {
    /// Left/Right balance.
    Pan,
    /// Gain to reverb effect.
    GainToReverb,
    /// Gain to 1st auxiliary mixer.
    GainToAux0,
    /// Gain to 2nd auxiliary mixer.
    GainToAux1,
}

impl Default for KnobPushMode {
    fn default() -> Self {
        Self::Pan
    }
}

const KNOB_PUSH_MODES: &[KnobPushMode] = &[
    KnobPushMode::Pan,
    KnobPushMode::GainToReverb,
    KnobPushMode::GainToAux0,
    KnobPushMode::GainToAux1,
];

const KNOB_PUSH_MODE_LABEL: &str = "knob push mode";

fn serialize_knob_push_mode(mode: &KnobPushMode, raw: &mut [u8]) -> Result<(), String> {
    serialize_position(KNOB_PUSH_MODES, mode, raw, KNOB_PUSH_MODE_LABEL)
}

fn deserialize_knob_push_mode(mode: &mut KnobPushMode, raw: &[u8]) -> Result<(), String> {
    deserialize_position(KNOB_PUSH_MODES, mode, raw, KNOB_PUSH_MODE_LABEL)
}

/// The number of entries for user-assigned button.
pub const STUDIO_REMOTE_USER_ASSIGN_COUNT: usize = 6;

/// State of remote controller.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct StudioRemote {
    /// Loaded program number.
    pub prog: TcKonnektLoadedProgram,
    /// Assignment of each user button to control one of source.
    pub user_assigns: [SrcEntry; STUDIO_REMOTE_USER_ASSIGN_COUNT],
    /// The mode of effect button.
    pub effect_button_mode: RemoteEffectButtonMode,
    /// Enable mode of fallback to master.
    pub fallback_to_master_enable: bool,
    /// The duration for the fallback mode.
    pub fallback_to_master_duration: u32,
    /// The mode at pushing knob.
    pub knob_push_mode: KnobPushMode,
}

impl TcKonnektSegmentSerdes<StudioRemote> for Studiok48Protocol {
    const NAME: &'static str = "remote-controller";
    const OFFSET: usize = 0x0014;
    const SIZE: usize = 48;

    fn serialize(params: &StudioRemote, raw: &mut [u8]) -> Result<(), String> {
        serialize_loaded_program(&params.prog, &mut raw[..4])?;
        params
            .user_assigns
            .iter()
            .enumerate()
            .try_for_each(|(i, assign)| {
                let pos = 4 + i * 4;
                serialize_src_entry(assign, &mut raw[pos..(pos + 4)])
            })?;
        serialize_remote_effect_button_mode(&params.effect_button_mode, &mut raw[28..32])?;
        serialize_bool(&params.fallback_to_master_enable, &mut raw[32..36]);
        serialize_u32(&params.fallback_to_master_duration, &mut raw[36..40]);
        serialize_knob_push_mode(&params.knob_push_mode, &mut raw[40..44])?;
        Ok(())
    }

    fn deserialize(params: &mut StudioRemote, raw: &[u8]) -> Result<(), String> {
        deserialize_loaded_program(&mut params.prog, &raw[..4])?;
        params
            .user_assigns
            .iter_mut()
            .enumerate()
            .try_for_each(|(i, assign)| {
                let pos = 4 + i * 4;
                deserialize_src_entry(assign, &raw[pos..(pos + 4)])
            })?;
        deserialize_remote_effect_button_mode(&mut params.effect_button_mode, &raw[28..32])?;
        deserialize_bool(&mut params.fallback_to_master_enable, &raw[32..36]);
        deserialize_u32(&mut params.fallback_to_master_duration, &raw[36..40]);
        deserialize_knob_push_mode(&mut params.knob_push_mode, &raw[40..44])?;
        Ok(())
    }
}

impl TcKonnektMutableSegmentOperation<StudioRemote> for Studiok48Protocol {}

impl TcKonnektNotifiedSegmentOperation<StudioRemote> for Studiok48Protocol {
    const NOTIFY_FLAG: u32 = STUDIO_REMOTE_NOTIFY_FLAG;
}

impl AsRef<TcKonnektLoadedProgram> for StudioRemote {
    fn as_ref(&self) -> &TcKonnektLoadedProgram {
        &self.prog
    }
}

impl AsMut<TcKonnektLoadedProgram> for StudioRemote {
    fn as_mut(&mut self) -> &mut TcKonnektLoadedProgram {
        &mut self.prog
    }
}

/// Mode of optical interface.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum OptIfaceMode {
    /// For ADAT signal.
    Adat,
    /// For S/PDIF signal.
    Spdif,
}

impl Default for OptIfaceMode {
    fn default() -> Self {
        Self::Adat
    }
}

const OPT_IFACE_MODES: &[OptIfaceMode] = &[OptIfaceMode::Adat, OptIfaceMode::Spdif];

const OPT_IFACE_MODE_LABEL: &str = "optical interface mode";

fn serialize_opt_iface_mode(mode: &OptIfaceMode, raw: &mut [u8]) -> Result<(), String> {
    serialize_position(OPT_IFACE_MODES, mode, raw, OPT_IFACE_MODE_LABEL)
}

fn deserialize_opt_iface_mode(mode: &mut OptIfaceMode, raw: &[u8]) -> Result<(), String> {
    deserialize_position(OPT_IFACE_MODES, mode, raw, OPT_IFACE_MODE_LABEL)
}

/// Source of standalone clock.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum StudioStandaloneClkSrc {
    /// From ADAT in optical input interface.
    Adat,
    /// From S/PDIF in 1st optical input interface.
    SpdifOnOpt01,
    /// From S/PDIF in 2nd optical input interface.
    SpdifOnOpt23,
    /// From S/PDIF in coaxial input interface.
    SpdifOnCoax,
    /// Word clock in BNC input interface.
    WordClock,
    /// Internal oscillator.
    Internal,
}

impl Default for StudioStandaloneClkSrc {
    fn default() -> Self {
        Self::Internal
    }
}

const STANDALONE_CLOCK_SOURCES: &[StudioStandaloneClkSrc] = &[
    StudioStandaloneClkSrc::Adat,
    StudioStandaloneClkSrc::SpdifOnOpt01,
    StudioStandaloneClkSrc::SpdifOnOpt23,
    StudioStandaloneClkSrc::SpdifOnCoax,
    StudioStandaloneClkSrc::WordClock,
    StudioStandaloneClkSrc::Internal,
];

const STANDALONE_CLOCK_SOURCE_LABEL: &str = "standalone clock source";

fn serialize_standalone_clock_source(
    src: &StudioStandaloneClkSrc,
    raw: &mut [u8],
) -> Result<(), String> {
    serialize_position(
        STANDALONE_CLOCK_SOURCES,
        src,
        raw,
        STANDALONE_CLOCK_SOURCE_LABEL,
    )
}

fn deserialize_standalone_clock_source(
    src: &mut StudioStandaloneClkSrc,
    raw: &[u8],
) -> Result<(), String> {
    deserialize_position(
        STANDALONE_CLOCK_SOURCES,
        src,
        raw,
        STANDALONE_CLOCK_SOURCE_LABEL,
    )
}

/// Configuration.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct StudioConfig {
    /// The mode of optical input/output interfaces.
    pub opt_iface_mode: OptIfaceMode,
    /// Source of sampling clock at standalone mode.
    pub standalone_src: StudioStandaloneClkSrc,
    /// Rate of sampling clock at standalone mode.
    pub standalone_rate: TcKonnektStandaloneClockRate,
    /// Whether to recover sampling clock from any source jitter.
    pub clock_recovery: bool,
    /// Configuration for midi event generator.
    pub midi_send: TcKonnektMidiSender,
}

impl TcKonnektSegmentSerdes<StudioConfig> for Studiok48Protocol {
    const NAME: &'static str = "configuration";
    const OFFSET: usize = 0x0044;
    const SIZE: usize = 100;

    fn serialize(params: &StudioConfig, raw: &mut [u8]) -> Result<(), String> {
        serialize_opt_iface_mode(&params.opt_iface_mode, &mut raw[..4])?;
        serialize_standalone_clock_source(&params.standalone_src, &mut raw[4..8])?;
        serialize_standalone_clock_rate(&params.standalone_rate, &mut raw[8..12])?;
        serialize_bool(&params.clock_recovery, &mut raw[16..20]);
        serialize_midi_sender(&params.midi_send, &mut raw[52..88])?;
        Ok(())
    }

    fn deserialize(params: &mut StudioConfig, raw: &[u8]) -> Result<(), String> {
        deserialize_opt_iface_mode(&mut params.opt_iface_mode, &raw[..4])?;
        deserialize_standalone_clock_source(&mut params.standalone_src, &raw[4..8])?;
        deserialize_standalone_clock_rate(&mut params.standalone_rate, &raw[8..12])?;
        deserialize_bool(&mut params.clock_recovery, &raw[16..20]);
        deserialize_midi_sender(&mut params.midi_send, &raw[52..88])?;
        Ok(())
    }
}

impl TcKonnektMutableSegmentOperation<StudioConfig> for Studiok48Protocol {}

impl TcKonnektNotifiedSegmentOperation<StudioConfig> for Studiok48Protocol {
    const NOTIFY_FLAG: u32 = STUDIO_CONFIG_NOTIFY_FLAG;
}

impl AsRef<TcKonnektStandaloneClockRate> for StudioConfig {
    fn as_ref(&self) -> &TcKonnektStandaloneClockRate {
        &self.standalone_rate
    }
}

impl AsMut<TcKonnektStandaloneClockRate> for StudioConfig {
    fn as_mut(&mut self) -> &mut TcKonnektStandaloneClockRate {
        &mut self.standalone_rate
    }
}

/// Entry of signal source.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SrcEntry {
    /// For unused.
    Unused,
    /// For analog 0..11.
    Analog(usize), // 0x01..0x0c
    /// For S/PDIF 0..1
    Spdif(usize), // 0x0d..0x0e
    /// For ADAT 0..7.
    Adat(usize), // 0x0f..0x16
    /// For stream A 0..11, 14,15.
    StreamA(usize), // 0x37..0x46
    /// For stream B 0..8.
    StreamB(usize), // 0x47..0x58
    /// For mixer output (main/aux0/aux1/reverb)
    Mixer(usize), // 0x55..0x5c
}

impl SrcEntry {
    const UNUSED: usize = 0x00;
    const ANALOG_OFFSET: usize = 0x01;
    const SPDIF_OFFSET: usize = 0x0d;
    const ADAT_OFFSET: usize = 0x0f;
    const STREAM_A_OFFSET: usize = 0x37;
    const STREAM_B_OFFSET: usize = 0x47;
    const MIXER_OFFSET: usize = 0x55;
}

impl Default for SrcEntry {
    fn default() -> Self {
        SrcEntry::Unused
    }
}

fn serialize_src_entry(entry: &SrcEntry, raw: &mut [u8]) -> Result<(), String> {
    assert!(raw.len() >= 4);

    let val = (match entry {
        SrcEntry::Unused => SrcEntry::UNUSED,
        SrcEntry::Analog(ch) => SrcEntry::ANALOG_OFFSET + ch,
        SrcEntry::Spdif(ch) => SrcEntry::SPDIF_OFFSET + ch,
        SrcEntry::Adat(ch) => SrcEntry::ADAT_OFFSET + ch,
        SrcEntry::StreamA(ch) => SrcEntry::STREAM_A_OFFSET + ch,
        SrcEntry::StreamB(ch) => SrcEntry::STREAM_B_OFFSET + ch,
        SrcEntry::Mixer(ch) => SrcEntry::MIXER_OFFSET + ch,
    }) as u32;

    serialize_u32(&val, raw);

    Ok(())
}

fn deserialize_src_entry(entry: &mut SrcEntry, raw: &[u8]) -> Result<(), String> {
    assert!(raw.len() >= 4);

    let mut val = 0u32;
    deserialize_u32(&mut val, raw);

    let v = val as usize;
    *entry = if v >= SrcEntry::ANALOG_OFFSET && v < SrcEntry::SPDIF_OFFSET {
        SrcEntry::Analog(v - SrcEntry::ANALOG_OFFSET)
    } else if v >= SrcEntry::SPDIF_OFFSET && v < SrcEntry::ADAT_OFFSET {
        SrcEntry::Spdif(v - SrcEntry::SPDIF_OFFSET)
    } else if v >= SrcEntry::ADAT_OFFSET && v < 0x17 {
        SrcEntry::Adat(v - SrcEntry::ADAT_OFFSET)
    } else if v >= SrcEntry::STREAM_A_OFFSET && v < SrcEntry::STREAM_B_OFFSET {
        SrcEntry::StreamA(v - SrcEntry::STREAM_A_OFFSET)
    } else if v >= SrcEntry::STREAM_B_OFFSET && v < SrcEntry::MIXER_OFFSET {
        SrcEntry::StreamB(v - SrcEntry::STREAM_B_OFFSET)
    } else if v >= SrcEntry::MIXER_OFFSET && v < 0x5d {
        SrcEntry::Mixer(v - SrcEntry::MIXER_OFFSET)
    } else {
        SrcEntry::Unused
    };

    Ok(())
}

/// State of output pair.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct OutPair {
    /// Whether to enable dim or not.
    pub dim_enabled: bool,
    /// Volume of the pair.
    pub vol: i32,
    /// Dimmed volume of the pair.
    pub dim_vol: i32,
}

impl OutPair {
    const SIZE: usize = 12;
}

fn serialize_out_pair(pair: &OutPair, raw: &mut [u8]) -> Result<(), String> {
    assert!(raw.len() >= OutPair::SIZE);

    serialize_bool(&pair.dim_enabled, &mut raw[..4]);
    serialize_i32(&pair.vol, &mut raw[4..8]);
    serialize_i32(&pair.dim_vol, &mut raw[8..12]);

    Ok(())
}

fn deserialize_out_pair(pair: &mut OutPair, raw: &[u8]) -> Result<(), String> {
    assert!(raw.len() >= OutPair::SIZE);

    deserialize_bool(&mut pair.dim_enabled, &raw[..4]);
    deserialize_i32(&mut pair.vol, &raw[4..8]);
    deserialize_i32(&mut pair.dim_vol, &raw[8..12]);

    Ok(())
}

/// The mode of entry for pair of source of monitor.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MonitorSrcPairMode {
    /// Inactive.
    Inactive,
    /// Active.
    Active,
    /// Always available.
    Fixed,
}

impl Default for MonitorSrcPairMode {
    fn default() -> Self {
        Self::Inactive
    }
}

const MONITOR_SRC_PAIR_MODES: &[MonitorSrcPairMode] = &[
    MonitorSrcPairMode::Inactive,
    MonitorSrcPairMode::Active,
    MonitorSrcPairMode::Fixed,
];

const MONITOR_SRC_PAIR_MODE_LABEL: &str = "monitor source pair mode";

fn serialize_monitor_src_pair_mode(
    mode: &MonitorSrcPairMode,
    raw: &mut [u8],
) -> Result<(), String> {
    serialize_position(
        MONITOR_SRC_PAIR_MODES,
        mode,
        raw,
        MONITOR_SRC_PAIR_MODE_LABEL,
    )
}

fn deserialize_monitor_src_pair_mode(
    mode: &mut MonitorSrcPairMode,
    raw: &[u8],
) -> Result<(), String> {
    deserialize_position(
        MONITOR_SRC_PAIR_MODES,
        mode,
        raw,
        MONITOR_SRC_PAIR_MODE_LABEL,
    )
}

/// Parameters of source of monitor.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct MonitorSrcParam {
    /// Assigned mixer source.
    pub src: SrcEntry,
    /// Gain to main mixer, between -1000 and 0 (-72.00 and 0.00 dB).
    pub gain_to_main: i32,
    /// Left/Right balance to main mixer, between -50 and 50.
    pub pan_to_main: i32,
    /// Gain to reverb effect, between -1000 and 0 (-72.00 and 0.00 dB).
    pub gain_to_reverb: i32,
    /// Gain to 1st auxiliary mixer, between -1000 and 0 (-72.00 and 0.00 dB).
    pub gain_to_aux0: i32,
    /// Gain to 2nd auxiliary mixer, between -1000 and 0 (-72.00 and 0.00 dB).
    pub gain_to_aux1: i32,
}

impl MonitorSrcParam {
    const SIZE: usize = 24;
}

fn serialize_monitor_src_params(params: &MonitorSrcParam, raw: &mut [u8]) -> Result<(), String> {
    assert!(raw.len() >= MonitorSrcParam::SIZE);

    serialize_src_entry(&params.src, &mut raw[..4])?;
    serialize_i32(&params.gain_to_main, &mut raw[4..8]);
    serialize_i32(&params.pan_to_main, &mut raw[8..12]);
    serialize_i32(&params.gain_to_reverb, &mut raw[12..16]);
    serialize_i32(&params.gain_to_aux0, &mut raw[16..20]);
    serialize_i32(&params.gain_to_aux1, &mut raw[20..24]);

    Ok(())
}

fn deserialize_monitor_src_params(params: &mut MonitorSrcParam, raw: &[u8]) -> Result<(), String> {
    assert!(raw.len() >= MonitorSrcParam::SIZE);

    deserialize_src_entry(&mut params.src, &raw[..4])?;
    deserialize_i32(&mut params.gain_to_main, &raw[4..8]);
    deserialize_i32(&mut params.pan_to_main, &raw[8..12]);
    deserialize_i32(&mut params.gain_to_reverb, &raw[12..16]);
    deserialize_i32(&mut params.gain_to_aux0, &raw[16..20]);
    deserialize_i32(&mut params.gain_to_aux1, &raw[20..24]);

    Ok(())
}

/// Source of monitor.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct MonitorSrcPair {
    /// Mode of source pair of monitor.
    pub mode: MonitorSrcPairMode,
    ///  Stereo channel link for the pair.
    pub stereo_link: bool,
    /// Parameters of monitor source for left and right channels in its order.
    pub params: [MonitorSrcParam; 2],
}

impl MonitorSrcPair {
    const SIZE: usize = 56;
}

fn serialize_monitor_src_pair(pair: &MonitorSrcPair, raw: &mut [u8]) -> Result<(), String> {
    assert!(raw.len() >= MonitorSrcPair::SIZE);

    serialize_monitor_src_pair_mode(&pair.mode, &mut raw[..4])?;
    serialize_bool(&pair.stereo_link, &mut raw[4..8]);
    serialize_monitor_src_params(&pair.params[0], &mut raw[8..32])?;
    serialize_monitor_src_params(&pair.params[1], &mut raw[32..56])?;

    Ok(())
}

fn deserialize_monitor_src_pair(pair: &mut MonitorSrcPair, raw: &[u8]) -> Result<(), String> {
    assert!(raw.len() >= MonitorSrcPair::SIZE);

    deserialize_monitor_src_pair_mode(&mut pair.mode, &raw[..4])?;
    deserialize_bool(&mut pair.stereo_link, &raw[4..8]);
    deserialize_monitor_src_params(&mut pair.params[0], &raw[8..32])?;
    deserialize_monitor_src_params(&mut pair.params[1], &raw[32..56])?;

    Ok(())
}

/// The number of pairs for source of monitor.
pub const STUDIO_MIXER_SRC_PAIR_COUNT: usize = 12;

/// State of mixer.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct StudioMixerState {
    /// For mixer sources.
    pub src_pairs: [MonitorSrcPair; STUDIO_MIXER_SRC_PAIR_COUNT],
    /// Whethe to mute mixer sources.
    pub mutes: [bool; STUDIO_MIXER_SRC_PAIR_COUNT],
    /// Whether to mute reverb effect return.
    pub reverb_return_mute: [bool; 3],
    /// Gain of reverb effect return.
    pub reverb_return_gain: [i32; 3],
    /// Whether to use channel strip effects as plugin.
    pub ch_strip_as_plugin: [bool; 2],
    /// The source of channel strip effects.
    pub ch_strip_src: [SrcEntry; 4],
    /// Use 3rd and 4th channel strip effects at 88.2/96.0 kHz.
    pub ch_strip_23_at_mid_rate: bool,
    /// Settings for mixer outputs.
    pub mixer_out: [OutPair; 3],
    /// Control volume before/after mixing.
    pub post_fader: [bool; 3],
    /// Whether to enable mixer function or not.
    pub enabled: bool,
}

impl TcKonnektSegmentSerdes<StudioMixerState> for Studiok48Protocol {
    const NAME: &'static str = "mixer-state";
    const OFFSET: usize = 0x00a8;
    const SIZE: usize = 820;

    fn serialize(params: &StudioMixerState, raw: &mut [u8]) -> Result<(), String> {
        params.src_pairs.iter().enumerate().try_for_each(|(i, p)| {
            let pos = i * MonitorSrcPair::SIZE;
            serialize_monitor_src_pair(p, &mut raw[pos..(pos + MonitorSrcPair::SIZE)])
        })?;
        let mut val = 0u32;
        params
            .mutes
            .iter()
            .enumerate()
            .filter(|(_, &m)| m)
            .for_each(|(i, _)| {
                val |= 1 << i;
            });
        serialize_u32(&val, &mut raw[672..676]);
        serialize_bool(&params.reverb_return_mute[0], &mut raw[712..716]);
        serialize_i32(&params.reverb_return_gain[0], &mut raw[716..720]);
        serialize_bool(&params.reverb_return_mute[1], &mut raw[720..724]);
        serialize_i32(&params.reverb_return_gain[1], &mut raw[724..728]);
        serialize_bool(&params.reverb_return_mute[2], &mut raw[728..732]);
        serialize_i32(&params.reverb_return_gain[2], &mut raw[732..736]);
        serialize_bool(&params.ch_strip_as_plugin[0], &mut raw[736..740]);
        serialize_bool(&params.ch_strip_as_plugin[1], &mut raw[740..744]);
        params
            .ch_strip_src
            .iter()
            .enumerate()
            .try_for_each(|(i, entry)| {
                let pos = 744 + i * 4;
                serialize_src_entry(entry, &mut raw[pos..(pos + 4)])
            })?;
        serialize_bool(&params.ch_strip_23_at_mid_rate, &mut raw[760..764]);
        serialize_out_pair(&params.mixer_out[0], &mut raw[764..776])?;
        serialize_out_pair(&params.mixer_out[1], &mut raw[776..788])?;
        serialize_out_pair(&params.mixer_out[2], &mut raw[788..800])?;
        serialize_bool(&params.post_fader[0], &mut raw[800..804]);
        serialize_bool(&params.post_fader[1], &mut raw[804..808]);
        serialize_bool(&params.post_fader[2], &mut raw[808..812]);
        serialize_bool(&params.enabled, &mut raw[812..816]);
        Ok(())
    }

    fn deserialize(params: &mut StudioMixerState, raw: &[u8]) -> Result<(), String> {
        params
            .src_pairs
            .iter_mut()
            .enumerate()
            .try_for_each(|(i, p)| {
                let pos = i * MonitorSrcPair::SIZE;
                deserialize_monitor_src_pair(p, &raw[pos..(pos + MonitorSrcPair::SIZE)])
            })?;
        let mut val = 0u32;
        deserialize_u32(&mut val, &raw[672..676]);
        params.mutes.iter_mut().enumerate().for_each(|(i, m)| {
            *m = (val & 1 << i) > 0;
        });
        deserialize_bool(&mut params.reverb_return_mute[0], &raw[712..716]);
        deserialize_i32(&mut params.reverb_return_gain[0], &raw[716..720]);
        deserialize_bool(&mut params.reverb_return_mute[1], &raw[720..724]);
        deserialize_i32(&mut params.reverb_return_gain[1], &raw[724..728]);
        deserialize_bool(&mut params.reverb_return_mute[2], &raw[728..732]);
        deserialize_i32(&mut params.reverb_return_gain[2], &raw[732..736]);
        deserialize_bool(&mut params.ch_strip_as_plugin[0], &raw[736..740]);
        deserialize_bool(&mut params.ch_strip_as_plugin[1], &raw[740..744]);
        params
            .ch_strip_src
            .iter_mut()
            .enumerate()
            .try_for_each(|(i, entry)| {
                let pos = 744 + i * 4;
                deserialize_src_entry(entry, &raw[pos..(pos + 4)])
            })?;
        deserialize_bool(&mut params.ch_strip_23_at_mid_rate, &raw[760..764]);
        deserialize_out_pair(&mut params.mixer_out[0], &raw[764..776])?;
        deserialize_out_pair(&mut params.mixer_out[1], &raw[776..788])?;
        deserialize_out_pair(&mut params.mixer_out[2], &raw[788..800])?;
        deserialize_bool(&mut params.post_fader[0], &raw[800..804]);
        deserialize_bool(&mut params.post_fader[1], &raw[804..808]);
        deserialize_bool(&mut params.post_fader[2], &raw[800..812]);
        deserialize_bool(&mut params.enabled, &raw[812..816]);
        Ok(())
    }
}

impl TcKonnektMutableSegmentOperation<StudioMixerState> for Studiok48Protocol {}

impl TcKonnektNotifiedSegmentOperation<StudioMixerState> for Studiok48Protocol {
    const NOTIFY_FLAG: u32 = STUDIO_MIXER_STATE_NOTIFY_FLAG;
}

/// Parameter of each channel for source of physical output.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct PhysOutSrcParam {
    /// Source.
    pub src: SrcEntry,
    /// Volume.
    pub vol: i32,
    /// Delay.
    pub delay: i32,
}

impl PhysOutSrcParam {
    const SIZE: usize = 12;
}

fn serialize_phys_out_src_params(params: &PhysOutSrcParam, raw: &mut [u8]) -> Result<(), String> {
    assert!(raw.len() >= PhysOutSrcParam::SIZE);

    serialize_src_entry(&params.src, &mut raw[..4])?;
    serialize_i32(&params.vol, &mut raw[4..8]);
    serialize_i32(&params.delay, &mut raw[8..12]);

    Ok(())
}

fn deserialize_phys_out_src_params(params: &mut PhysOutSrcParam, raw: &[u8]) -> Result<(), String> {
    assert!(raw.len() >= PhysOutSrcParam::SIZE);

    deserialize_src_entry(&mut params.src, &raw[..4])?;
    deserialize_i32(&mut params.vol, &raw[4..8]);
    deserialize_i32(&mut params.delay, &raw[8..12]);

    Ok(())
}

/// Source of physical output.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct PhysOutPairSrc {
    /// Stereo channel link for the pair.
    pub stereo_link: bool,
    /// Parameters of sources for left and right channels.
    pub params: [PhysOutSrcParam; 2],
}

impl PhysOutPairSrc {
    const SIZE: usize = 28;
}

fn serialize_phys_out_pair_src(src: &PhysOutPairSrc, raw: &mut [u8]) -> Result<(), String> {
    assert!(raw.len() >= PhysOutPairSrc::SIZE);

    serialize_bool(&src.stereo_link, &mut raw[..4]);
    serialize_phys_out_src_params(&src.params[0], &mut raw[4..16])?;
    serialize_phys_out_src_params(&src.params[1], &mut raw[16..28])?;

    Ok(())
}

fn deserialize_phys_out_pair_src(src: &mut PhysOutPairSrc, raw: &[u8]) -> Result<(), String> {
    assert!(raw.len() >= PhysOutPairSrc::SIZE);

    deserialize_bool(&mut src.stereo_link, &raw[..4]);
    deserialize_phys_out_src_params(&mut src.params[0], &raw[4..16])?;
    deserialize_phys_out_src_params(&mut src.params[1], &raw[16..28])?;

    Ok(())
}

/// The highest frequency to cross over into LFE channel.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CrossOverFreq {
    /// 50 Hz.
    F50,
    /// 80 Hz.
    F80,
    /// 95 Hz.
    F95,
    /// 110 Hz.
    F110,
    /// 115 Hz.
    F115,
    /// 120 Hz.
    F120,
}

impl Default for CrossOverFreq {
    fn default() -> Self {
        Self::F50
    }
}

const CROSS_OVER_FREQS: &[CrossOverFreq] = &[
    CrossOverFreq::F50,
    CrossOverFreq::F80,
    CrossOverFreq::F95,
    CrossOverFreq::F110,
    CrossOverFreq::F115,
    CrossOverFreq::F120,
];

const CROSS_OVER_FREQ_LABEL: &str = "cross over frequency";

fn serialize_cross_over_freq(freq: &CrossOverFreq, raw: &mut [u8]) -> Result<(), String> {
    serialize_position(CROSS_OVER_FREQS, freq, raw, CROSS_OVER_FREQ_LABEL)
}

fn deserialize_cross_over_freq(freq: &mut CrossOverFreq, raw: &[u8]) -> Result<(), String> {
    deserialize_position(CROSS_OVER_FREQS, freq, raw, CROSS_OVER_FREQ_LABEL)
}

/// The frequency above cross over frequency into main channel.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum HighPassFreq {
    /// Off.
    Off,
    /// Above 12 Hz per octave.
    Above12,
    /// Above 24 Hz per octave.
    Above24,
}

impl Default for HighPassFreq {
    fn default() -> Self {
        HighPassFreq::Off
    }
}

const HIGH_PASS_FREQS: &[HighPassFreq] = &[
    HighPassFreq::Off,
    HighPassFreq::Above12,
    HighPassFreq::Above24,
];

const HIGH_PASS_FREQ_LABEL: &str = "high pass frequency";

fn serialize_high_pass_freq(freq: &HighPassFreq, raw: &mut [u8]) -> Result<(), String> {
    serialize_position(HIGH_PASS_FREQS, freq, raw, HIGH_PASS_FREQ_LABEL)
}

fn deserialize_high_pass_freq(freq: &mut HighPassFreq, raw: &[u8]) -> Result<(), String> {
    deserialize_position(HIGH_PASS_FREQS, freq, raw, HIGH_PASS_FREQ_LABEL)
}

/// The frequency below cross over frequency into LFE channel.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum LowPassFreq {
    /// Below 12 Hz per octave.
    Below12,
    /// Below 24 Hz per octave.
    Below24,
}

impl Default for LowPassFreq {
    fn default() -> Self {
        LowPassFreq::Below12
    }
}

fn serialize_low_pass_freq(freq: &LowPassFreq, raw: &mut [u8]) -> Result<(), String> {
    assert!(raw.len() >= 4);

    let val = match freq {
        LowPassFreq::Below12 => 1u32,
        LowPassFreq::Below24 => 2,
    };
    serialize_u32(&val, raw);

    Ok(())
}

fn deserialize_low_pass_freq(freq: &mut LowPassFreq, raw: &[u8]) -> Result<(), String> {
    assert!(raw.len() >= 4);

    let mut val = 0u32;
    deserialize_u32(&mut val, raw);

    *freq = match val {
        1 => LowPassFreq::Below12,
        2 => LowPassFreq::Below24,
        _ => Err(format!("low pass frequency not found for value {}", val))?,
    };

    Ok(())
}

/// The maximum number of surround channel of which a output group consists.
pub const STUDIO_MAX_SURROUND_CHANNELS: usize = 8;

/// The group to aggregate several outputs for surround channels.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct OutGroup {
    /// Assignment of physical outputs to the group.
    pub assigned_phys_outs: [bool; STUDIO_PHYS_OUT_PAIR_COUNT * 2],
    /// Whether to enable bass management.
    pub bass_management: bool,
    /// The sub channel to Low Frequency Effect (LFE).
    pub sub_channel: Option<usize>,
    /// The frequency above which signal is to main , below which signal is to Low Frequency Effect
    /// (LFE).
    pub main_cross_over_freq: CrossOverFreq,
    /// Gain for signal from main to Low Frequency Effect (LFE).
    pub main_level_to_sub: i32,
    /// Gain for signal from sub channel to Low Frequency Effect (LFE).
    pub sub_level_to_sub: i32,
    /// Frequency of high pass filter for the signal of main channel.
    pub main_filter_for_main: HighPassFreq,
    /// Frequency of low pass filter for the signal from main channel to sub channel.
    pub main_filter_for_sub: LowPassFreq,
}

impl OutGroup {
    const SIZE: usize = 36;
}

fn serialize_out_group(group: &OutGroup, raw: &mut [u8]) -> Result<(), String> {
    assert!(raw.len() >= OutGroup::SIZE);

    // NOTE: when the value has bit flags more than 8, the ASIC to read the value is going to
    // freeze. The corruption can be recovered to recall the other program state (P1/P2/P3) by
    // the controller at standalone mode, then connect and factory reset by software.
    let mut val = 0u32;
    group
        .assigned_phys_outs
        .iter()
        .enumerate()
        .filter(|(_, &a)| a)
        .take(STUDIO_MAX_SURROUND_CHANNELS)
        .for_each(|(i, _)| {
            val |= 1 << i;
        });
    serialize_u32(&val, &mut raw[..4]);
    serialize_bool(&group.bass_management, &mut raw[4..8]);
    val = match group.sub_channel {
        Some(pos) => 1 << pos,
        None => 0,
    };
    serialize_u32(&val, &mut raw[12..16]);
    serialize_cross_over_freq(&group.main_cross_over_freq, &mut raw[16..20])?;
    serialize_i32(&group.main_level_to_sub, &mut raw[20..24]);
    serialize_i32(&group.sub_level_to_sub, &mut raw[24..28]);
    serialize_high_pass_freq(&group.main_filter_for_main, &mut raw[28..32])?;
    serialize_low_pass_freq(&group.main_filter_for_sub, &mut raw[32..])?;

    Ok(())
}

fn deserialize_out_group(group: &mut OutGroup, raw: &[u8]) -> Result<(), String> {
    assert!(raw.len() >= OutGroup::SIZE);

    let mut val = 0u32;
    deserialize_u32(&mut val, &raw[..4]);
    group
        .assigned_phys_outs
        .iter_mut()
        .enumerate()
        .for_each(|(i, a)| *a = val & (1 << i) > 0);
    deserialize_bool(&mut group.bass_management, &raw[4..8]);
    deserialize_u32(&mut val, &raw[12..16]);
    group.sub_channel = (0..group.assigned_phys_outs.len())
        .position(|i| val & (1 << i) > 0)
        .map(|pos| pos as usize);
    deserialize_cross_over_freq(&mut group.main_cross_over_freq, &raw[16..20])?;
    deserialize_i32(&mut group.main_level_to_sub, &raw[20..24]);
    deserialize_i32(&mut group.sub_level_to_sub, &raw[24..28]);
    deserialize_high_pass_freq(&mut group.main_filter_for_main, &raw[28..32])?;
    deserialize_low_pass_freq(&mut group.main_filter_for_sub, &raw[32..])?;

    Ok(())
}

/// The number of pairs of physical output.
pub const STUDIO_PHYS_OUT_PAIR_COUNT: usize = 11;

/// The number of groups to aggregate several outputs for surround channels.
pub const STUDIO_OUTPUT_GROUP_COUNT: usize = 3;

/// Data of physical out segment.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct StudioPhysOut {
    /// The configuration for master output
    pub master_out: OutPair,
    /// The selected output group.
    pub selected_out_grp: usize,
    /// The source for pairs of physical output. It includes below pairs in
    /// the order:
    /// - main out 1/2
    /// - phone out 1/2
    /// - line out 5/6, 7/8, 9/10, 11/12,
    /// - S/PDIF out 1/2,
    /// - ADAT out 1/2, 3/4, 5/6, 7/8,
    pub out_pair_srcs: [PhysOutPairSrc; STUDIO_PHYS_OUT_PAIR_COUNT],
    /// The state of assignment to output group.
    pub out_assign_to_grp: [bool; STUDIO_PHYS_OUT_PAIR_COUNT * 2],
    /// Whether to mute any source to the physical output.
    pub out_mutes: [bool; STUDIO_PHYS_OUT_PAIR_COUNT * 2],
    /// The settings of each group for surround channels.
    pub out_grps: [OutGroup; STUDIO_OUTPUT_GROUP_COUNT],
}

impl TcKonnektSegmentSerdes<StudioPhysOut> for Studiok48Protocol {
    const NAME: &'static str = "physical-output";
    const OFFSET: usize = 0x03dc;
    const SIZE: usize = 440;

    fn serialize(params: &StudioPhysOut, raw: &mut [u8]) -> Result<(), String> {
        serialize_out_pair(&params.master_out, &mut raw[..12])?;
        params
            .out_pair_srcs
            .iter()
            .enumerate()
            .try_for_each(|(i, p)| {
                let pos = 16 + i * PhysOutPairSrc::SIZE;
                serialize_phys_out_pair_src(p, &mut raw[pos..(pos + PhysOutPairSrc::SIZE)])
            })?;
        serialize_usize(&params.selected_out_grp, &mut raw[12..16]);
        let mut val = 0u32;
        params
            .out_assign_to_grp
            .iter()
            .enumerate()
            .filter(|(_, &m)| m)
            .for_each(|(i, _)| {
                val |= 1 << i;
            });
        serialize_u32(&val, &mut raw[324..328]);
        let mut val = 0u32;
        params
            .out_mutes
            .iter()
            .enumerate()
            .filter(|(_, &d)| d)
            .for_each(|(i, _)| {
                val |= 1 << i;
            });
        serialize_u32(&val, &mut raw[328..332]);
        params.out_grps.iter().enumerate().try_for_each(|(i, s)| {
            let pos = 332 + OutGroup::SIZE * i;
            serialize_out_group(s, &mut raw[pos..(pos + OutGroup::SIZE)])
        })?;
        Ok(())
    }

    fn deserialize(params: &mut StudioPhysOut, raw: &[u8]) -> Result<(), String> {
        deserialize_out_pair(&mut params.master_out, &raw[..12])?;
        params
            .out_pair_srcs
            .iter_mut()
            .enumerate()
            .try_for_each(|(i, p)| {
                let pos = 16 + i * PhysOutPairSrc::SIZE;
                deserialize_phys_out_pair_src(p, &raw[pos..(pos + PhysOutPairSrc::SIZE)])
            })?;
        let mut val = 0u32;
        deserialize_u32(&mut val, &raw[12..16]);
        deserialize_usize(&mut params.selected_out_grp, &raw[324..328]);
        params
            .out_assign_to_grp
            .iter_mut()
            .enumerate()
            .for_each(|(i, m)| {
                *m = val & (1 << i) > 0;
            });
        let mut val = 0u32;
        deserialize_u32(&mut val, &raw[328..332]);
        params.out_mutes.iter_mut().enumerate().for_each(|(i, d)| {
            *d = val & (1 << i) > 0;
        });
        params
            .out_grps
            .iter_mut()
            .enumerate()
            .try_for_each(|(i, s)| {
                let pos = 332 + OutGroup::SIZE * i;
                deserialize_out_group(s, &raw[pos..(pos + OutGroup::SIZE)])
            })?;
        Ok(())
    }
}

impl TcKonnektMutableSegmentOperation<StudioPhysOut> for Studiok48Protocol {}

impl TcKonnektNotifiedSegmentOperation<StudioPhysOut> for Studiok48Protocol {
    const NOTIFY_FLAG: u32 = STUDIO_PHYS_OUT_NOTIFY_FLAG;
}

const STUDIO_CH_STRIP_COUNT: usize = 4;

/// Configuration for reverb effect.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct StudioReverbState(pub ReverbState);

impl TcKonnektSegmentSerdes<StudioReverbState> for Studiok48Protocol {
    const NAME: &'static str = "reverb-state";
    const OFFSET: usize = 0x0594;
    const SIZE: usize = ReverbState::SIZE;

    fn serialize(params: &StudioReverbState, raw: &mut [u8]) -> Result<(), String> {
        serialize_reverb_state(&params.0, raw)
    }

    fn deserialize(params: &mut StudioReverbState, raw: &[u8]) -> Result<(), String> {
        deserialize_reverb_state(&mut params.0, raw)
    }
}

impl TcKonnektMutableSegmentOperation<StudioReverbState> for Studiok48Protocol {}

impl TcKonnektNotifiedSegmentOperation<StudioReverbState> for Studiok48Protocol {
    const NOTIFY_FLAG: u32 = STUDIO_REVERB_NOTIFY_CHANGE;
}

impl AsRef<ReverbState> for StudioReverbState {
    fn as_ref(&self) -> &ReverbState {
        &self.0
    }
}

impl AsMut<ReverbState> for StudioReverbState {
    fn as_mut(&mut self) -> &mut ReverbState {
        &mut self.0
    }
}

/// Configuration for channel strip effect.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct StudioChStripStates(pub [ChStripState; STUDIO_CH_STRIP_COUNT]);

impl TcKonnektSegmentSerdes<StudioChStripStates> for Studiok48Protocol {
    const NAME: &'static str = "channel-strip-state";
    const OFFSET: usize = 0x05dc;
    const SIZE: usize = ChStripState::SIZE * STUDIO_CH_STRIP_COUNT + 8;

    fn serialize(params: &StudioChStripStates, raw: &mut [u8]) -> Result<(), String> {
        serialize_ch_strip_states(&params.0, raw)
    }

    fn deserialize(params: &mut StudioChStripStates, raw: &[u8]) -> Result<(), String> {
        deserialize_ch_strip_states(&mut params.0, raw)
    }
}

impl TcKonnektMutableSegmentOperation<StudioChStripStates> for Studiok48Protocol {}

impl TcKonnektNotifiedSegmentOperation<StudioChStripStates> for Studiok48Protocol {
    const NOTIFY_FLAG: u32 = STUDIO_CH_STRIP_NOTIFY_01_CHANGE | STUDIO_CH_STRIP_NOTIFY_23_CHANGE;
}

impl AsRef<[ChStripState]> for StudioChStripStates {
    fn as_ref(&self) -> &[ChStripState] {
        &self.0
    }
}

impl AsMut<[ChStripState]> for StudioChStripStates {
    fn as_mut(&mut self) -> &mut [ChStripState] {
        &mut self.0
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
/// State of jack sense for analog input.
pub enum StudioAnalogJackState {
    /// Select front jack instead of rear.
    FrontSelected,
    /// Detect plug insertion in front jack.
    FrontInserted,
    /// Select rear jack instead of front.
    RearSelected,
    /// Detect plug insertion in rear jack.
    RearInserted,
}

impl Default for StudioAnalogJackState {
    fn default() -> Self {
        Self::FrontSelected
    }
}

fn serialize_analog_jack_state(
    state: &StudioAnalogJackState,
    raw: &mut [u8],
) -> Result<(), String> {
    assert!(raw.len() >= 4);

    let val = match state {
        StudioAnalogJackState::FrontSelected => 5,
        StudioAnalogJackState::FrontInserted => 6,
        StudioAnalogJackState::RearSelected => 7,
        StudioAnalogJackState::RearInserted => 8,
    };

    serialize_u32(&val, raw);

    Ok(())
}

fn deserialize_analog_jack_state(
    state: &mut StudioAnalogJackState,
    raw: &[u8],
) -> Result<(), String> {
    assert!(raw.len() >= 4);

    let mut val = 0u32;
    deserialize_u32(&mut val, raw);

    *state = match val {
        8 => StudioAnalogJackState::RearInserted,
        7 => StudioAnalogJackState::RearSelected,
        6 => StudioAnalogJackState::FrontInserted,
        _ => StudioAnalogJackState::FrontSelected,
    };

    Ok(())
}

/// The number of analog inputs which has jack sense.
pub const STUDIO_ANALOG_JACK_STATE_COUNT: usize = 12;

/// Hardware state.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct StudioHwState {
    /// The state of analog jack with sense.
    pub analog_jack_states: [StudioAnalogJackState; STUDIO_ANALOG_JACK_STATE_COUNT],
    /// State of headphone.
    pub hp_state: [bool; 2],
    /// State of FireWire LED.
    pub firewire_led: FireWireLedState,
    /// Whether knob of master level is actually effective for volume of master output. This is
    /// needed since the volume is controlled by remote controller as well.
    pub valid_master_level: bool,
}

impl TcKonnektSegmentSerdes<StudioHwState> for Studiok48Protocol {
    const NAME: &'static str = "hardware-state";
    const OFFSET: usize = 0x2008;
    const SIZE: usize = 68;

    fn serialize(params: &StudioHwState, raw: &mut [u8]) -> Result<(), String> {
        params
            .analog_jack_states
            .iter()
            .enumerate()
            .try_for_each(|(i, state)| {
                let pos = 4 * i;
                serialize_analog_jack_state(state, &mut raw[pos..(pos + 4)])
            })?;
        serialize_bool(&params.hp_state[0], &mut raw[48..56]);
        serialize_bool(&params.hp_state[1], &mut raw[48..56]);
        serialize_fw_led_state(&params.firewire_led, &mut raw[56..60])?;
        serialize_bool(&params.valid_master_level, &mut raw[60..64]);
        Ok(())
    }

    fn deserialize(params: &mut StudioHwState, raw: &[u8]) -> Result<(), String> {
        params
            .analog_jack_states
            .iter_mut()
            .enumerate()
            .try_for_each(|(i, state)| {
                let pos = 4 * i;
                deserialize_analog_jack_state(state, &raw[pos..(pos + 4)])
            })?;
        deserialize_bool(&mut params.hp_state[0], &raw[48..52]);
        deserialize_bool(&mut params.hp_state[1], &raw[52..56]);
        deserialize_fw_led_state(&mut params.firewire_led, &raw[56..60])?;
        deserialize_bool(&mut params.valid_master_level, &raw[60..64]);
        Ok(())
    }
}

impl TcKonnektMutableSegmentOperation<StudioHwState> for Studiok48Protocol {}

impl TcKonnektNotifiedSegmentOperation<StudioHwState> for Studiok48Protocol {
    const NOTIFY_FLAG: u32 = STUDIO_HW_STATE_NOTIFY_FLAG;
}

impl AsRef<FireWireLedState> for StudioHwState {
    fn as_ref(&self) -> &FireWireLedState {
        &self.firewire_led
    }
}

impl AsMut<FireWireLedState> for StudioHwState {
    fn as_mut(&mut self) -> &mut FireWireLedState {
        &mut self.firewire_led
    }
}

/// Hardware metering for mixer function.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct StudioMixerMeter {
    /// Detected signal level of main mixer sources.
    pub src_inputs: [i32; 24],
    /// Detected signal level of main mixer outputs.
    pub mixer_outputs: [i32; 2],
    /// Detected signal level of aux mixer outputs.
    pub aux_outputs: [i32; 4],
}

impl TcKonnektSegmentSerdes<StudioMixerMeter> for Studiok48Protocol {
    const NAME: &'static str = "mixer-meter";
    const OFFSET: usize = 0x20b8;
    const SIZE: usize = 128;

    fn serialize(params: &StudioMixerMeter, raw: &mut [u8]) -> Result<(), String> {
        params
            .src_inputs
            .iter()
            .chain(&params.mixer_outputs)
            .chain(&params.aux_outputs)
            .enumerate()
            .for_each(|(i, level)| {
                let pos = i * 4;
                serialize_i32(level, &mut raw[pos..(pos + 4)])
            });

        Ok(())
    }

    fn deserialize(params: &mut StudioMixerMeter, raw: &[u8]) -> Result<(), String> {
        params
            .src_inputs
            .iter_mut()
            .chain(&mut params.mixer_outputs)
            .chain(&mut params.aux_outputs)
            .enumerate()
            .for_each(|(i, level)| {
                let pos = i * 4;
                deserialize_i32(level, &raw[pos..(pos + 4)])
            });

        Ok(())
    }
}

/// Hardware metering for reverb effect.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct StudioReverbMeter(pub ReverbMeter);

impl TcKonnektSegmentSerdes<StudioReverbMeter> for Studiok48Protocol {
    const NAME: &'static str = "reverb-meter";
    const OFFSET: usize = 0x2164;
    const SIZE: usize = ReverbMeter::SIZE;

    fn serialize(params: &StudioReverbMeter, raw: &mut [u8]) -> Result<(), String> {
        serialize_reverb_meter(&params.0, raw)
    }

    fn deserialize(params: &mut StudioReverbMeter, raw: &[u8]) -> Result<(), String> {
        deserialize_reverb_meter(&mut params.0, raw)
    }
}

impl AsRef<ReverbMeter> for StudioReverbMeter {
    fn as_ref(&self) -> &ReverbMeter {
        &self.0
    }
}

impl AsMut<ReverbMeter> for StudioReverbMeter {
    fn as_mut(&mut self) -> &mut ReverbMeter {
        &mut self.0
    }
}

/// Hardware metering for channel strip effect.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct StudioChStripMeters(pub [ChStripMeter; STUDIO_CH_STRIP_COUNT]);

impl TcKonnektSegmentSerdes<StudioChStripMeters> for Studiok48Protocol {
    const NAME: &'static str = "channel-strip-meter";
    const OFFSET: usize = 0x217c;
    const SIZE: usize = ChStripMeter::SIZE * STUDIO_CH_STRIP_COUNT + 8;

    fn serialize(params: &StudioChStripMeters, raw: &mut [u8]) -> Result<(), String> {
        serialize_ch_strip_meters(&params.0, raw)
    }

    fn deserialize(params: &mut StudioChStripMeters, raw: &[u8]) -> Result<(), String> {
        deserialize_ch_strip_meters(&mut params.0, raw)
    }
}

impl AsRef<[ChStripMeter]> for StudioChStripMeters {
    fn as_ref(&self) -> &[ChStripMeter] {
        &self.0
    }
}

impl AsMut<[ChStripMeter]> for StudioChStripMeters {
    fn as_mut(&mut self) -> &mut [ChStripMeter] {
        &mut self.0
    }
}
