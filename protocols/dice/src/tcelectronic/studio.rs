// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Protocol defined by TC Electronic for Studio Konnekt 48.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by TC Electronic for Studio Konnekt 48.

use super::{ch_strip::*, midi_send::*, prog::*, reverb::*, *};

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

impl From<u32> for NominalSignalLevel {
    fn from(val: u32) -> Self {
        if val > 0 {
            Self::Professional
        } else {
            Self::Consumer
        }
    }
}

impl From<NominalSignalLevel> for u32 {
    fn from(level: NominalSignalLevel) -> Self {
        match level {
            NominalSignalLevel::Consumer => 0,
            NominalSignalLevel::Professional => 1,
        }
    }
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
        params.line_45.build_quadlet(&mut raw[4..8]);
        params.line_67.build_quadlet(&mut raw[8..12]);
        params.line_89.build_quadlet(&mut raw[12..16]);
        params.line_1011.build_quadlet(&mut raw[16..20]);
        Ok(())
    }

    fn deserialize(params: &mut StudioLineOutLevel, raw: &[u8]) -> Result<(), String> {
        params.line_45.parse_quadlet(&raw[4..8]);
        params.line_67.parse_quadlet(&raw[8..12]);
        params.line_89.parse_quadlet(&raw[12..16]);
        params.line_1011.parse_quadlet(&raw[16..20]);
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
    Reverb,
    Midi,
}

impl Default for RemoteEffectButtonMode {
    fn default() -> Self {
        Self::Reverb
    }
}

impl From<u32> for RemoteEffectButtonMode {
    fn from(val: u32) -> Self {
        if val > 0 {
            Self::Midi
        } else {
            Self::Reverb
        }
    }
}

impl From<RemoteEffectButtonMode> for u32 {
    fn from(mode: RemoteEffectButtonMode) -> Self {
        match mode {
            RemoteEffectButtonMode::Reverb => 0,
            RemoteEffectButtonMode::Midi => 1,
        }
    }
}

/// Mode of knob target at pushed state.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum KnobPushMode {
    Pan,
    GainToReverb,
    GainToAux0,
    GainToAux1,
}

impl Default for KnobPushMode {
    fn default() -> Self {
        Self::Pan
    }
}

impl From<u32> for KnobPushMode {
    fn from(val: u32) -> Self {
        match val {
            3 => Self::GainToAux1,
            2 => Self::GainToAux0,
            1 => Self::GainToReverb,
            _ => Self::Pan,
        }
    }
}

impl From<KnobPushMode> for u32 {
    fn from(mode: KnobPushMode) -> Self {
        match mode {
            KnobPushMode::Pan => 0,
            KnobPushMode::GainToReverb => 1,
            KnobPushMode::GainToAux0 => 2,
            KnobPushMode::GainToAux1 => 3,
        }
    }
}

/// The number of entries for user-assigned button.
pub const STUDIO_REMOTE_USER_ASSIGN_COUNT: usize = 6;

/// State of remote controller.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct StudioRemote {
    pub prog: TcKonnektLoadedProgram,
    pub user_assigns: [SrcEntry; STUDIO_REMOTE_USER_ASSIGN_COUNT],
    pub effect_button_mode: RemoteEffectButtonMode,
    pub fallback_to_master_enable: bool,
    pub fallback_to_master_duration: u32,
    pub knob_push_mode: KnobPushMode,
}

impl TcKonnektSegmentSerdes<StudioRemote> for Studiok48Protocol {
    const NAME: &'static str = "remote-controller";
    const OFFSET: usize = 0x0014;
    const SIZE: usize = 48;

    fn serialize(params: &StudioRemote, raw: &mut [u8]) -> Result<(), String> {
        params.prog.build(&mut raw[..4]);
        params.user_assigns.build_quadlet_block(&mut raw[4..28]);
        params.effect_button_mode.build_quadlet(&mut raw[28..32]);
        params
            .fallback_to_master_enable
            .build_quadlet(&mut raw[32..36]);
        params
            .fallback_to_master_duration
            .build_quadlet(&mut raw[36..40]);
        params.knob_push_mode.build_quadlet(&mut raw[40..44]);
        Ok(())
    }

    fn deserialize(params: &mut StudioRemote, raw: &[u8]) -> Result<(), String> {
        params.prog.parse(&raw[..4]);
        params.user_assigns.parse_quadlet_block(&raw[4..28]);
        params.effect_button_mode.parse_quadlet(&raw[28..32]);
        params.fallback_to_master_enable.parse_quadlet(&raw[32..36]);
        params
            .fallback_to_master_duration
            .parse_quadlet(&raw[36..40]);
        params.knob_push_mode.parse_quadlet(&raw[40..44]);
        Ok(())
    }
}

impl TcKonnektMutableSegmentOperation<StudioRemote> for Studiok48Protocol {}

impl TcKonnektNotifiedSegmentOperation<StudioRemote> for Studiok48Protocol {
    const NOTIFY_FLAG: u32 = STUDIO_REMOTE_NOTIFY_FLAG;
}

/// Mode of optical interface.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum OptIfaceMode {
    Adat,
    Spdif,
}

impl Default for OptIfaceMode {
    fn default() -> Self {
        Self::Adat
    }
}

impl From<u32> for OptIfaceMode {
    fn from(val: u32) -> Self {
        if val > 0 {
            Self::Spdif
        } else {
            Self::Adat
        }
    }
}

impl From<OptIfaceMode> for u32 {
    fn from(mode: OptIfaceMode) -> Self {
        (mode == OptIfaceMode::Spdif) as u32
    }
}

/// Source of standalone clock.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum StudioStandaloneClkSrc {
    Adat,
    SpdifOnOpt01,
    SpdifOnOpt23,
    SpdifOnCoax,
    WordClock,
    Internal,
}

impl Default for StudioStandaloneClkSrc {
    fn default() -> Self {
        Self::Internal
    }
}

impl From<u32> for StudioStandaloneClkSrc {
    fn from(val: u32) -> Self {
        match val {
            0 => Self::Adat,
            1 => Self::SpdifOnOpt01,
            2 => Self::SpdifOnOpt23,
            3 => Self::SpdifOnCoax,
            4 => Self::WordClock,
            _ => Self::Internal,
        }
    }
}

impl From<StudioStandaloneClkSrc> for u32 {
    fn from(src: StudioStandaloneClkSrc) -> Self {
        match src {
            StudioStandaloneClkSrc::Adat => 0,
            StudioStandaloneClkSrc::SpdifOnOpt01 => 1,
            StudioStandaloneClkSrc::SpdifOnOpt23 => 2,
            StudioStandaloneClkSrc::SpdifOnCoax => 3,
            StudioStandaloneClkSrc::WordClock => 4,
            StudioStandaloneClkSrc::Internal => 5,
        }
    }
}

/// Configuration.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct StudioConfig {
    pub opt_iface_mode: OptIfaceMode,
    pub standalone_src: StudioStandaloneClkSrc,
    pub standalone_rate: TcKonnektStandaloneClockRate,
    pub clock_recovery: bool,
    pub midi_send: TcKonnektMidiSender,
}

impl TcKonnektSegmentSerdes<StudioConfig> for Studiok48Protocol {
    const NAME: &'static str = "configuration";
    const OFFSET: usize = 0x0044;
    const SIZE: usize = 100;

    fn serialize(params: &StudioConfig, raw: &mut [u8]) -> Result<(), String> {
        params.opt_iface_mode.build_quadlet(&mut raw[..4]);
        params.standalone_src.build_quadlet(&mut raw[4..8]);
        serialize_standalone_clock_rate(&params.standalone_rate, &mut raw[8..12])?;
        params.clock_recovery.build_quadlet(&mut raw[16..20]);
        params.midi_send.build(&mut raw[52..88]);
        Ok(())
    }

    fn deserialize(params: &mut StudioConfig, raw: &[u8]) -> Result<(), String> {
        params.opt_iface_mode.parse_quadlet(&raw[..4]);
        params.standalone_src.parse_quadlet(&raw[4..8]);
        deserialize_standalone_clock_rate(&mut params.standalone_rate, &raw[8..12])?;
        params.clock_recovery.parse_quadlet(&raw[16..20]);
        params.midi_send.parse(&raw[52..88]);
        Ok(())
    }
}

impl TcKonnektMutableSegmentOperation<StudioConfig> for Studiok48Protocol {}

impl TcKonnektNotifiedSegmentOperation<StudioConfig> for Studiok48Protocol {
    const NOTIFY_FLAG: u32 = STUDIO_CONFIG_NOTIFY_FLAG;
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

impl From<u32> for SrcEntry {
    fn from(val: u32) -> Self {
        let v = val as usize;
        if v >= SrcEntry::ANALOG_OFFSET && v < SrcEntry::SPDIF_OFFSET {
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
        }
    }
}

impl From<SrcEntry> for u32 {
    fn from(src: SrcEntry) -> Self {
        (match src {
            SrcEntry::Unused => SrcEntry::UNUSED,
            SrcEntry::Analog(ch) => SrcEntry::ANALOG_OFFSET + ch,
            SrcEntry::Spdif(ch) => SrcEntry::SPDIF_OFFSET + ch,
            SrcEntry::Adat(ch) => SrcEntry::ADAT_OFFSET + ch,
            SrcEntry::StreamA(ch) => SrcEntry::STREAM_A_OFFSET + ch,
            SrcEntry::StreamB(ch) => SrcEntry::STREAM_B_OFFSET + ch,
            SrcEntry::Mixer(ch) => SrcEntry::MIXER_OFFSET + ch,
        }) as u32
    }
}

/// State of output pair.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct OutPair {
    pub dim_enabled: bool,
    pub vol: i32,
    pub dim_vol: i32,
}

impl OutPair {
    const SIZE: usize = 12;

    fn build(&self, raw: &mut [u8]) {
        assert_eq!(raw.len(), Self::SIZE, "Programming error...");

        self.dim_enabled.build_quadlet(&mut raw[..4]);
        self.vol.build_quadlet(&mut raw[4..8]);
        self.dim_vol.build_quadlet(&mut raw[8..12]);
    }

    fn parse(&mut self, raw: &[u8]) {
        assert_eq!(raw.len(), Self::SIZE, "Programming error...");

        self.dim_enabled.parse_quadlet(&raw[..4]);
        self.vol.parse_quadlet(&raw[4..8]);
        self.dim_vol.parse_quadlet(&raw[8..12]);
    }
}

/// The mode of entry for pair of source of monitor.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MonitorSrcPairMode {
    Inactive,
    Active,
    Fixed,
}

impl Default for MonitorSrcPairMode {
    fn default() -> Self {
        Self::Inactive
    }
}

impl From<u32> for MonitorSrcPairMode {
    fn from(val: u32) -> Self {
        match val {
            2 => Self::Fixed,
            1 => Self::Active,
            _ => Self::Inactive,
        }
    }
}

impl From<MonitorSrcPairMode> for u32 {
    fn from(mode: MonitorSrcPairMode) -> Self {
        match mode {
            MonitorSrcPairMode::Inactive => 0,
            MonitorSrcPairMode::Active => 1,
            MonitorSrcPairMode::Fixed => 2,
        }
    }
}

/// Parameters of source of monitor.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct MonitorSrcParam {
    pub src: SrcEntry,
    pub gain_to_main: i32,
    pub pan_to_main: i32,
    pub gain_to_reverb: i32,
    pub gain_to_aux0: i32,
    pub gain_to_aux1: i32,
}

impl MonitorSrcParam {
    const SIZE: usize = 24;

    fn build(&self, raw: &mut [u8]) {
        assert_eq!(raw.len(), Self::SIZE, "Programming error");

        self.src.build_quadlet(&mut raw[..4]);
        self.gain_to_main.build_quadlet(&mut raw[4..8]);
        self.pan_to_main.build_quadlet(&mut raw[8..12]);
        self.gain_to_reverb.build_quadlet(&mut raw[12..16]);
        self.gain_to_aux0.build_quadlet(&mut raw[16..20]);
        self.gain_to_aux1.build_quadlet(&mut raw[20..24]);
    }

    fn parse(&mut self, raw: &[u8]) {
        assert_eq!(raw.len(), Self::SIZE, "Programming error");

        self.src.parse_quadlet(&raw[..4]);
        self.gain_to_main.parse_quadlet(&raw[4..8]);
        self.pan_to_main.parse_quadlet(&raw[8..12]);
        self.gain_to_reverb.parse_quadlet(&raw[12..16]);
        self.gain_to_aux0.parse_quadlet(&raw[16..20]);
        self.gain_to_aux1.parse_quadlet(&raw[20..24]);
    }
}

/// Source of monitor.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct MonitorSrcPair {
    pub mode: MonitorSrcPairMode,
    pub stereo_link: bool,
    pub left: MonitorSrcParam,
    pub right: MonitorSrcParam,
}

impl MonitorSrcPair {
    const SIZE: usize = 56;

    fn build(&self, raw: &mut [u8]) {
        assert_eq!(raw.len(), Self::SIZE, "Programming error...");

        self.mode.build_quadlet(&mut raw[..4]);
        self.stereo_link.build_quadlet(&mut raw[4..8]);
        self.left.build(&mut raw[8..32]);
        self.right.build(&mut raw[32..56]);
    }

    fn parse(&mut self, raw: &[u8]) {
        assert_eq!(raw.len(), Self::SIZE, "Programming error...");

        self.mode.parse_quadlet(&raw[..4]);
        self.stereo_link.parse_quadlet(&raw[4..8]);
        self.left.parse(&raw[8..32]);
        self.right.parse(&raw[32..56]);
    }
}

/// The number of pairs for source of monitor.
pub const STUDIO_MIXER_SRC_PAIR_COUNT: usize = 12;

/// State of mixer.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct StudioMixerState {
    pub src_pairs: [MonitorSrcPair; STUDIO_MIXER_SRC_PAIR_COUNT],
    pub mutes: [bool; STUDIO_MIXER_SRC_PAIR_COUNT],
    pub reverb_return_mute: [bool; 3],
    pub reverb_return_gain: [i32; 3],
    pub ch_strip_as_plugin: [bool; 2],
    pub ch_strip_src: [SrcEntry; 4],
    pub ch_strip_23_at_mid_rate: bool,
    pub mixer_out: [OutPair; 3],
    pub post_fader: [bool; 3],
    pub enabled: bool,
}

impl TcKonnektSegmentSerdes<StudioMixerState> for Studiok48Protocol {
    const NAME: &'static str = "mixer-state";
    const OFFSET: usize = 0x00a8;
    const SIZE: usize = 820;

    fn serialize(params: &StudioMixerState, raw: &mut [u8]) -> Result<(), String> {
        params.src_pairs.iter().enumerate().for_each(|(i, p)| {
            let pos = i * MonitorSrcPair::SIZE;
            p.build(&mut raw[pos..(pos + MonitorSrcPair::SIZE)]);
        });
        let mut val = 0u32;
        params
            .mutes
            .iter()
            .enumerate()
            .filter(|(_, &m)| m)
            .for_each(|(i, _)| {
                val |= 1 << i;
            });
        val.build_quadlet(&mut raw[672..676]);
        params.reverb_return_mute[0].build_quadlet(&mut raw[712..716]);
        params.reverb_return_gain[0].build_quadlet(&mut raw[716..720]);
        params.reverb_return_mute[1].build_quadlet(&mut raw[720..724]);
        params.reverb_return_gain[1].build_quadlet(&mut raw[724..728]);
        params.reverb_return_mute[2].build_quadlet(&mut raw[728..732]);
        params.reverb_return_gain[2].build_quadlet(&mut raw[732..736]);
        params
            .ch_strip_as_plugin
            .build_quadlet_block(&mut raw[736..744]);
        params.ch_strip_src.build_quadlet_block(&mut raw[744..760]);
        params
            .ch_strip_23_at_mid_rate
            .build_quadlet(&mut raw[760..764]);
        params.mixer_out[0].build(&mut raw[764..776]);
        params.mixer_out[1].build(&mut raw[776..788]);
        params.mixer_out[2].build(&mut raw[788..800]);
        params.post_fader.build_quadlet_block(&mut raw[800..812]);
        params.enabled.build_quadlet(&mut raw[812..816]);
        Ok(())
    }

    fn deserialize(params: &mut StudioMixerState, raw: &[u8]) -> Result<(), String> {
        params.src_pairs.iter_mut().enumerate().for_each(|(i, p)| {
            let pos = i * MonitorSrcPair::SIZE;
            p.parse(&raw[pos..(pos + MonitorSrcPair::SIZE)]);
        });
        let mut val = 0u32;
        val.parse_quadlet(&raw[672..676]);
        params.mutes.iter_mut().enumerate().for_each(|(i, m)| {
            *m = (val & 1 << i) > 0;
        });
        params.reverb_return_mute[0].parse_quadlet(&raw[712..716]);
        params.reverb_return_gain[0].parse_quadlet(&raw[716..720]);
        params.reverb_return_mute[1].parse_quadlet(&raw[720..724]);
        params.reverb_return_gain[1].parse_quadlet(&raw[724..728]);
        params.reverb_return_mute[2].parse_quadlet(&raw[728..732]);
        params.reverb_return_gain[2].parse_quadlet(&raw[732..736]);
        params
            .ch_strip_as_plugin
            .parse_quadlet_block(&raw[736..744]);
        params.ch_strip_src.parse_quadlet_block(&raw[744..760]);
        params.ch_strip_23_at_mid_rate.parse_quadlet(&raw[760..764]);
        params.mixer_out[0].parse(&raw[764..776]);
        params.mixer_out[1].parse(&raw[776..788]);
        params.mixer_out[2].parse(&raw[788..800]);
        params.post_fader.parse_quadlet_block(&raw[800..812]);
        params.enabled.parse_quadlet(&raw[812..816]);
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
    pub src: SrcEntry,
    pub vol: i32,
    pub delay: i32,
}

impl PhysOutSrcParam {
    const SIZE: usize = 12;

    fn build(&self, raw: &mut [u8]) {
        assert_eq!(raw.len(), Self::SIZE, "Programming error...");

        self.src.build_quadlet(&mut raw[..4]);
        self.vol.build_quadlet(&mut raw[4..8]);
        self.delay.build_quadlet(&mut raw[8..12]);
    }

    fn parse(&mut self, raw: &[u8]) {
        assert_eq!(raw.len(), Self::SIZE, "Programming error...");

        self.src.parse_quadlet(&raw[..4]);
        self.vol.parse_quadlet(&raw[4..8]);
        self.delay.parse_quadlet(&raw[8..12]);
    }
}

/// Source of physical output.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct PhysOutPairSrc {
    pub stereo_link: bool,
    pub left: PhysOutSrcParam,
    pub right: PhysOutSrcParam,
}

impl PhysOutPairSrc {
    const SIZE: usize = 28;

    fn build(&self, raw: &mut [u8]) {
        assert_eq!(raw.len(), Self::SIZE, "Programming error...");

        self.stereo_link.build_quadlet(&mut raw[..4]);
        self.left.build(&mut raw[4..16]);
        self.right.build(&mut raw[16..28]);
    }

    fn parse(&mut self, raw: &[u8]) {
        assert_eq!(raw.len(), Self::SIZE, "Programming error...");

        self.stereo_link.parse_quadlet(&raw[..4]);
        self.left.parse(&raw[4..16]);
        self.right.parse(&raw[16..28]);
    }
}

/// The highest frequency to cross over into LFE channel.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CrossOverFreq {
    F50,
    F80,
    F95,
    F110,
    F115,
    F120,
    Reserved(u32),
}

impl Default for CrossOverFreq {
    fn default() -> Self {
        Self::Reserved(0xff)
    }
}

impl From<u32> for CrossOverFreq {
    fn from(val: u32) -> Self {
        match val {
            0 => Self::F50,
            1 => Self::F80,
            2 => Self::F95,
            3 => Self::F110,
            4 => Self::F115,
            5 => Self::F120,
            _ => Self::Reserved(val),
        }
    }
}

impl From<CrossOverFreq> for u32 {
    fn from(freq: CrossOverFreq) -> u32 {
        match freq {
            CrossOverFreq::F50 => 0,
            CrossOverFreq::F80 => 1,
            CrossOverFreq::F95 => 2,
            CrossOverFreq::F110 => 3,
            CrossOverFreq::F115 => 4,
            CrossOverFreq::F120 => 5,
            CrossOverFreq::Reserved(val) => val,
        }
    }
}

/// The frequency above cross over frequency into main channel.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum HighPassFreq {
    Off,
    Above12,
    Above24,
    Reserved(u32),
}

impl Default for HighPassFreq {
    fn default() -> Self {
        HighPassFreq::Reserved(0xff)
    }
}

impl From<u32> for HighPassFreq {
    fn from(val: u32) -> Self {
        match val {
            0 => Self::Off,
            1 => Self::Above12,
            2 => Self::Above24,
            _ => Self::Reserved(val),
        }
    }
}

impl From<HighPassFreq> for u32 {
    fn from(freq: HighPassFreq) -> Self {
        match freq {
            HighPassFreq::Off => 0,
            HighPassFreq::Above12 => 1,
            HighPassFreq::Above24 => 2,
            HighPassFreq::Reserved(val) => val,
        }
    }
}

/// The frequency below cross over frequency into LFE channel.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum LowPassFreq {
    Below12,
    Below24,
    Reserved(u32),
}

impl Default for LowPassFreq {
    fn default() -> Self {
        LowPassFreq::Reserved(0xff)
    }
}

impl From<u32> for LowPassFreq {
    fn from(val: u32) -> Self {
        match val {
            1 => Self::Below12,
            2 => Self::Below24,
            _ => Self::Reserved(val),
        }
    }
}

impl From<LowPassFreq> for u32 {
    fn from(freq: LowPassFreq) -> Self {
        match freq {
            LowPassFreq::Below12 => 1,
            LowPassFreq::Below24 => 2,
            LowPassFreq::Reserved(val) => val,
        }
    }
}

/// The maximum number of surround channel of which a output group consists.
pub const STUDIO_MAX_SURROUND_CHANNELS: usize = 8;

/// The group to aggregate several outputs for surround channels.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct OutGroup {
    pub assigned_phys_outs: [bool; STUDIO_PHYS_OUT_PAIR_COUNT * 2],
    pub bass_management: bool,
    pub sub_channel: Option<usize>,
    pub main_cross_over_freq: CrossOverFreq,
    pub main_level_to_sub: i32,
    pub sub_level_to_sub: i32,
    pub main_filter_for_main: HighPassFreq,
    pub main_filter_for_sub: LowPassFreq,
}

impl OutGroup {
    const SIZE: usize = 36;

    fn build(&self, raw: &mut [u8]) {
        // NOTE: when the value has bit flags more than 8, the ASIC to read the value is going to
        // freeze. The corruption can be recovered to recall the other program state (P1/P2/P3) by
        // the controller at standalone mode, then connect and factory reset by software.
        let mut val = 0u32;
        self.assigned_phys_outs
            .iter()
            .enumerate()
            .filter(|(_, &a)| a)
            .take(STUDIO_MAX_SURROUND_CHANNELS)
            .for_each(|(i, _)| {
                val |= 1 << i;
            });
        val.build_quadlet(&mut raw[..4]);
        self.bass_management.build_quadlet(&mut raw[4..8]);
        val = match self.sub_channel {
            Some(pos) => 1 << pos,
            None => 0,
        };
        val.build_quadlet(&mut raw[12..16]);
        self.main_cross_over_freq.build_quadlet(&mut raw[16..20]);
        self.main_level_to_sub.build_quadlet(&mut raw[20..24]);
        self.sub_level_to_sub.build_quadlet(&mut raw[24..28]);
        self.main_filter_for_main.build_quadlet(&mut raw[28..32]);
        self.main_filter_for_sub.build_quadlet(&mut raw[32..]);
    }

    fn parse(&mut self, raw: &[u8]) {
        let mut val = 0u32;
        val.parse_quadlet(&raw[..4]);
        self.assigned_phys_outs
            .iter_mut()
            .enumerate()
            .for_each(|(i, a)| {
                *a = val & (1 << i) > 0;
            });
        self.bass_management.parse_quadlet(&raw[4..8]);
        val.parse_quadlet(&raw[12..16]);
        self.sub_channel = (0..self.assigned_phys_outs.len())
            .position(|i| val & (1 << i) > 0)
            .map(|pos| pos as usize);
        self.main_cross_over_freq.parse_quadlet(&raw[16..20]);
        self.main_level_to_sub.parse_quadlet(&raw[20..24]);
        self.sub_level_to_sub.parse_quadlet(&raw[24..28]);
        self.main_filter_for_main.parse_quadlet(&raw[28..32]);
        self.main_filter_for_sub.parse_quadlet(&raw[32..]);
    }
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
        params.master_out.build(&mut raw[..12]);
        params.out_pair_srcs.iter().enumerate().for_each(|(i, p)| {
            let pos = 16 + i * PhysOutPairSrc::SIZE;
            p.build(&mut raw[pos..(pos + PhysOutPairSrc::SIZE)]);
        });
        (params.selected_out_grp as u32).build_quadlet(&mut raw[12..16]);
        let mut val = 0u32;
        params
            .out_assign_to_grp
            .iter()
            .enumerate()
            .filter(|(_, &m)| m)
            .for_each(|(i, _)| {
                val |= 1 << i;
            });
        val.build_quadlet(&mut raw[324..328]);
        let mut val = 0u32;
        params
            .out_mutes
            .iter()
            .enumerate()
            .filter(|(_, &d)| d)
            .for_each(|(i, _)| {
                val |= 1 << i;
            });
        val.build_quadlet(&mut raw[328..332]);
        params.out_grps.iter().enumerate().for_each(|(i, s)| {
            let pos = 332 + OutGroup::SIZE * i;
            s.build(&mut raw[pos..(pos + OutGroup::SIZE)]);
        });
        Ok(())
    }

    fn deserialize(params: &mut StudioPhysOut, raw: &[u8]) -> Result<(), String> {
        params.master_out.parse(&raw[..12]);
        params
            .out_pair_srcs
            .iter_mut()
            .enumerate()
            .for_each(|(i, p)| {
                let pos = 16 + i * PhysOutPairSrc::SIZE;
                p.parse(&raw[pos..(pos + PhysOutPairSrc::SIZE)]);
            });
        let mut val = 0u32;
        val.parse_quadlet(&raw[12..16]);
        params.selected_out_grp = val as usize;
        val.parse_quadlet(&raw[324..328]);
        params
            .out_assign_to_grp
            .iter_mut()
            .enumerate()
            .for_each(|(i, m)| {
                *m = val & (1 << i) > 0;
            });
        let mut val = 0u32;
        val.parse_quadlet(&raw[328..332]);
        params.out_mutes.iter_mut().enumerate().for_each(|(i, d)| {
            *d = val & (1 << i) > 0;
        });
        params.out_grps.iter_mut().enumerate().for_each(|(i, s)| {
            let pos = 332 + OutGroup::SIZE * i;
            s.parse(&raw[pos..(pos + OutGroup::SIZE)]);
        });
        Ok(())
    }
}

impl TcKonnektMutableSegmentOperation<StudioPhysOut> for Studiok48Protocol {}

impl TcKonnektNotifiedSegmentOperation<StudioPhysOut> for Studiok48Protocol {
    const NOTIFY_FLAG: u32 = STUDIO_PHYS_OUT_NOTIFY_FLAG;
}

const STUDIO_CH_STRIP_COUNT: usize = 4;

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct StudioReverbState(pub ReverbState);

impl TcKonnektSegmentSerdes<StudioReverbState> for Studiok48Protocol {
    const NAME: &'static str = "reverb-state";
    const OFFSET: usize = 0x0594;
    const SIZE: usize = ReverbState::SIZE;

    fn serialize(params: &StudioReverbState, raw: &mut [u8]) -> Result<(), String> {
        params.0.build(raw);
        Ok(())
    }

    fn deserialize(params: &mut StudioReverbState, raw: &[u8]) -> Result<(), String> {
        params.0.parse(raw);
        Ok(())
    }
}

impl TcKonnektMutableSegmentOperation<StudioReverbState> for Studiok48Protocol {}

impl TcKonnektNotifiedSegmentOperation<StudioReverbState> for Studiok48Protocol {
    const NOTIFY_FLAG: u32 = STUDIO_REVERB_NOTIFY_CHANGE;
}

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct StudioChStripStates(pub [ChStripState; STUDIO_CH_STRIP_COUNT]);

impl TcKonnektSegmentSerdes<StudioChStripStates> for Studiok48Protocol {
    const NAME: &'static str = "channel-strip-state";
    const OFFSET: usize = 0x05dc;
    const SIZE: usize = ChStripState::SIZE * STUDIO_CH_STRIP_COUNT + 8;

    fn serialize(params: &StudioChStripStates, raw: &mut [u8]) -> Result<(), String> {
        params.0.build(raw);
        Ok(())
    }

    fn deserialize(params: &mut StudioChStripStates, raw: &[u8]) -> Result<(), String> {
        params.0.parse(raw);
        Ok(())
    }
}

impl TcKonnektMutableSegmentOperation<StudioChStripStates> for Studiok48Protocol {}

impl TcKonnektNotifiedSegmentOperation<StudioChStripStates> for Studiok48Protocol {
    const NOTIFY_FLAG: u32 = STUDIO_CH_STRIP_NOTIFY_01_CHANGE | STUDIO_CH_STRIP_NOTIFY_23_CHANGE;
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
/// State of jack sense for analog input.
pub enum StudioAnalogJackState {
    FrontSelected,
    FrontInserted,
    RearSelected,
    RearInserted,
}

impl Default for StudioAnalogJackState {
    fn default() -> Self {
        Self::FrontSelected
    }
}

impl From<u32> for StudioAnalogJackState {
    fn from(val: u32) -> Self {
        match val {
            8 => Self::RearInserted,
            7 => Self::RearSelected,
            6 => Self::FrontInserted,
            _ => Self::FrontSelected,
        }
    }
}

impl From<StudioAnalogJackState> for u32 {
    fn from(state: StudioAnalogJackState) -> Self {
        match state {
            StudioAnalogJackState::FrontSelected => 5,
            StudioAnalogJackState::FrontInserted => 6,
            StudioAnalogJackState::RearSelected => 7,
            StudioAnalogJackState::RearInserted => 8,
        }
    }
}

/// The number of analog inputs which has jack sense.
pub const STUDIO_ANALOG_JACK_STATE_COUNT: usize = 12;

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
/// Hardware state.
pub struct StudioHwState {
    pub analog_jack_states: [StudioAnalogJackState; STUDIO_ANALOG_JACK_STATE_COUNT],
    pub hp_state: [bool; 2],
    pub firewire_led: FireWireLedState,
    pub valid_master_level: bool,
}

impl TcKonnektSegmentSerdes<StudioHwState> for Studiok48Protocol {
    const NAME: &'static str = "hardware-state";
    const OFFSET: usize = 0x2008;
    const SIZE: usize = 68;

    fn serialize(params: &StudioHwState, raw: &mut [u8]) -> Result<(), String> {
        params
            .analog_jack_states
            .build_quadlet_block(&mut raw[..48]);
        params.hp_state.build_quadlet_block(&mut raw[48..56]);
        serialize_fw_led_state(&params.firewire_led, &mut raw[56..60])?;
        params.valid_master_level.build_quadlet(&mut raw[60..64]);
        Ok(())
    }

    fn deserialize(params: &mut StudioHwState, raw: &[u8]) -> Result<(), String> {
        params.analog_jack_states.parse_quadlet_block(&raw[..48]);
        params.hp_state.parse_quadlet_block(&raw[48..56]);
        deserialize_fw_led_state(&mut params.firewire_led, &raw[56..60])?;
        params.valid_master_level.parse_quadlet(&raw[60..64]);
        Ok(())
    }
}

impl TcKonnektMutableSegmentOperation<StudioHwState> for Studiok48Protocol {}

impl TcKonnektNotifiedSegmentOperation<StudioHwState> for Studiok48Protocol {
    const NOTIFY_FLAG: u32 = STUDIO_HW_STATE_NOTIFY_FLAG;
}

/// Meter for input/output of mixer.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct StudioMixerMeter {
    pub src_inputs: [i32; 24],
    pub mixer_outputs: [i32; 2],
    pub aux_outputs: [i32; 4],
}

impl TcKonnektSegmentSerdes<StudioMixerMeter> for Studiok48Protocol {
    const NAME: &'static str = "mixer-meter";
    const OFFSET: usize = 0x20b8;
    const SIZE: usize = 128;

    fn serialize(params: &StudioMixerMeter, raw: &mut [u8]) -> Result<(), String> {
        params.src_inputs.build_quadlet_block(&mut raw[4..100]);
        params.mixer_outputs.build_quadlet_block(&mut raw[100..108]);
        params.aux_outputs.build_quadlet_block(&mut raw[108..124]);
        Ok(())
    }

    fn deserialize(params: &mut StudioMixerMeter, raw: &[u8]) -> Result<(), String> {
        params.src_inputs.parse_quadlet_block(&raw[4..100]);
        params.mixer_outputs.parse_quadlet_block(&raw[100..108]);
        params.aux_outputs.parse_quadlet_block(&raw[108..124]);
        Ok(())
    }
}

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct StudioReverbMeter(pub ReverbMeter);

impl TcKonnektSegmentSerdes<StudioReverbMeter> for Studiok48Protocol {
    const NAME: &'static str = "reverb-meter";
    const OFFSET: usize = 0x2164;
    const SIZE: usize = ReverbMeter::SIZE;

    fn serialize(params: &StudioReverbMeter, raw: &mut [u8]) -> Result<(), String> {
        params.0.build(raw);
        Ok(())
    }

    fn deserialize(params: &mut StudioReverbMeter, raw: &[u8]) -> Result<(), String> {
        params.0.parse(raw);
        Ok(())
    }
}

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct StudioChStripMeters(pub [ChStripMeter; STUDIO_CH_STRIP_COUNT]);

impl TcKonnektSegmentSerdes<StudioChStripMeters> for Studiok48Protocol {
    const NAME: &'static str = "channel-strip-meter";
    const OFFSET: usize = 0x217c;
    const SIZE: usize = ChStripMeter::SIZE * STUDIO_CH_STRIP_COUNT + 8;

    fn serialize(params: &StudioChStripMeters, raw: &mut [u8]) -> Result<(), String> {
        params.0.build(raw);
        Ok(())
    }

    fn deserialize(params: &mut StudioChStripMeters, raw: &[u8]) -> Result<(), String> {
        params.0.parse(raw);
        Ok(())
    }
}
