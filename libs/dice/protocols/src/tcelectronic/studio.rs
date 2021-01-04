// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Protocol defined by TC Electronic for Studio Konnekt 48.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by TC Electronic for Studio Konnekt 48.

use super::{*, ch_strip::*, reverb::*, fw_led::*};
use crate::*;

/// The structure to represent segments in memory space of Studio Konnekt 48.
#[derive(Default, Debug)]
pub struct StudioSegments{
    /// Segment for state of mixer. 0x00a8..0x03db (205 quads).
    pub mixer_state: TcKonnektSegment<StudioMixerState>,
    /// Segment for physical output. 0x03dc..0x0593 (110 quads).
    pub phys_out: TcKonnektSegment<StudioPhysOut>,
    /// Segment for state of reverb effect. 0x0594..0x05d7. (17 quads)
    pub reverb_state: TcKonnektSegment<StudioReverbState>,
    /// Segment for states of channel strip effect. 0x05d8..0x081f (146 quads).
    pub ch_strip_state: TcKonnektSegment<StudioChStripStates>,
    /// Segment for state of hardware. 0x2008..0x204b (17 quads).
    pub hw_state: TcKonnektSegment<StudioHwState>,
    /// Segment for meter of mixer. 0x20b8..0x2137 (32 quads).
    pub mixer_meter: TcKonnektSegment<StudioMixerMeter>,
    /// Segment for meter of reverb effect. 0x2164..0x217b (6 quads).
    pub reverb_meter: TcKonnektSegment<StudioReverbMeter>,
    /// Segment for meters of channel strip effect. 0x217c..0x21b7 (30 quads).
    pub ch_strip_meter: TcKonnektSegment<StudioChStripMeters>,
}

const STUDIO_MIXER_STATE_NOTIFY_FLAG: u32 = 0x00080000;
const STUDIO_PHYS_OUT_NOTIFY_FLAG: u32 = 0x00100000;
const STUDIO_REVERB_NOTIFY_CHANGE: u32 = 0x00200000;
const STUDIO_CH_STRIP_NOTIFY_01_CHANGE: u32 = 0x00400000;
const STUDIO_CH_STRIP_NOTIFY_23_CHANGE: u32 = 0x00800000;
const STUDIO_HW_STATE_NOTIFY_FLAG: u32 = 0x04000000;

/// The enumeration to represent entry of signal source.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SrcEntry {
    /// For unused.
    Unused,
    /// For analog 0..11.
    Analog(usize),  // 0x01..0x0c
    /// For S/PDIF 0..1
    Spdif(usize),   // 0x0d..0x0e
    /// For ADAT 0..7.
    Adat(usize),    // 0x0f..0x16
    /// For stream A 0..11, 14,15.
    StreamA(usize), // 0x37..0x46
    /// For stream B 0..8.
    StreamB(usize), // 0x47..0x58
    /// For mixer output (main/aux0/aux1/reverb)
    Mixer(usize),   // 0x55..0x5c
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

/// The structure to represent state of output pair.
#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
pub struct OutPair{
    pub dim_enabled: bool,
    pub vol: i32,
    pub dim_vol: i32
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
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
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

/// The structure to represent parameters of source of monitor.
#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
pub struct MonitorSrcParam{
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

/// The structure to represent source of monitor.
#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
pub struct MonitorSrcPair{
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

/// The structure to represent state of mixer.
#[derive(Default, Debug)]
pub struct StudioMixerState{
    pub src_pairs: [MonitorSrcPair;STUDIO_MIXER_SRC_PAIR_COUNT],
    pub mutes: [bool;STUDIO_MIXER_SRC_PAIR_COUNT],
    pub reverb_return_mute: [bool;3],
    pub reverb_return_gain: [i32;3],
    pub ch_strip_as_plugin: [bool;2],
    pub ch_strip_src: [SrcEntry;4],
    pub ch_strip_23_at_mid_rate: bool,
    pub mixer_out: [OutPair;3],
    pub post_fader: [bool;3],
    pub enabled: bool,
}

impl StudioMixerState {
    const SIZE: usize = 820;
}

impl TcKonnektSegmentData for StudioMixerState {
    fn build(&self, raw: &mut [u8]) {
        self.src_pairs.iter()
            .enumerate()
            .for_each(|(i, p)| {
                let pos = i * MonitorSrcPair::SIZE;
                p.build(&mut raw[pos..(pos + MonitorSrcPair::SIZE)]);
            });
        let mut val = 0u32;
        self.mutes.iter()
            .enumerate()
            .filter(|(_, &m)| m)
            .for_each(|(i, _)| {
                val |= 1 << i;
            });
        val.build_quadlet(&mut raw[672..676]);
        self.reverb_return_mute[0].build_quadlet(&mut raw[712..716]);
        self.reverb_return_gain[0].build_quadlet(&mut raw[716..720]);
        self.reverb_return_mute[1].build_quadlet(&mut raw[720..724]);
        self.reverb_return_gain[1].build_quadlet(&mut raw[724..728]);
        self.reverb_return_mute[2].build_quadlet(&mut raw[728..732]);
        self.reverb_return_gain[2].build_quadlet(&mut raw[732..736]);
        self.ch_strip_as_plugin.build_quadlet_block(&mut raw[736..744]);
        self.ch_strip_src.build_quadlet_block(&mut raw[744..760]);
        self.ch_strip_23_at_mid_rate.build_quadlet(&mut raw[760..764]);
        self.mixer_out[0].build(&mut raw[764..776]);
        self.mixer_out[1].build(&mut raw[776..788]);
        self.mixer_out[2].build(&mut raw[788..800]);
        self.post_fader.build_quadlet_block(&mut raw[800..812]);
        self.enabled.build_quadlet(&mut raw[812..816]);
    }

    fn parse(&mut self, raw: &[u8]) {
        self.src_pairs.iter_mut()
            .enumerate()
            .for_each(|(i, p)| {
                let pos = i * MonitorSrcPair::SIZE;
                p.parse(&raw[pos..(pos + MonitorSrcPair::SIZE)]);
            });
        let mut val = 0u32;
        val.parse_quadlet(&raw[672..676]);
        self.mutes.iter_mut()
            .enumerate()
            .for_each(|(i, m)| {
                *m = (val & 1 << i) > 0;
            });
        self.reverb_return_mute[0].parse_quadlet(&raw[712..716]);
        self.reverb_return_gain[0].parse_quadlet(&raw[716..720]);
        self.reverb_return_mute[1].parse_quadlet(&raw[720..724]);
        self.reverb_return_gain[1].parse_quadlet(&raw[724..728]);
        self.reverb_return_mute[2].parse_quadlet(&raw[728..732]);
        self.reverb_return_gain[2].parse_quadlet(&raw[732..736]);
        self.ch_strip_as_plugin.parse_quadlet_block(&raw[736..744]);
        self.ch_strip_src.parse_quadlet_block(&raw[744..760]);
        self.ch_strip_23_at_mid_rate.parse_quadlet(&raw[760..764]);
        self.mixer_out[0].parse(&raw[764..776]);
        self.mixer_out[1].parse(&raw[776..788]);
        self.mixer_out[2].parse(&raw[788..800]);
        self.post_fader.parse_quadlet_block(&raw[800..812]);
        self.enabled.parse_quadlet(&raw[812..816]);
    }
}

impl TcKonnektSegmentSpec for TcKonnektSegment<StudioMixerState> {
    const OFFSET: usize = 0x00a8;
    const SIZE: usize = StudioMixerState::SIZE;
}

impl TcKonnektNotifiedSegmentSpec for TcKonnektSegment<StudioMixerState> {
    const NOTIFY_FLAG: u32 = STUDIO_MIXER_STATE_NOTIFY_FLAG;
}

/// The structure to represent parameter of each channel for source of physical output.
#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
pub struct PhysOutSrcParam{
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
 
/// The structure to represent source of physical output.
#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
pub struct PhysOutPairSrc{
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

/// The virtual speaker to consists of several physical outputs.
#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
pub struct VirtualSpeaker{
    pub phys_out_pair_assigns: [bool;STUDIO_PHYS_OUT_PAIR_COUNT],
    pub bass_management: bool,
}

impl VirtualSpeaker {
    const SIZE: usize = 36;

    fn build(&self, raw: &mut [u8]) {
        let mut val = 0u32;
        self.phys_out_pair_assigns.iter()
            .enumerate()
            .filter(|(_, &a)| a)
            .for_each(|(i, _)| {
                val |= 1 << i;
            });
        val.build_quadlet(&mut raw[..4]);
        self.bass_management.build_quadlet(&mut raw[4..8]);
    }

    fn parse(&mut self, raw: &[u8]) {
        let mut val = 0u32;
        val.parse_quadlet(&raw[..4]);
        self.phys_out_pair_assigns.iter_mut()
            .enumerate()
            .for_each(|(i, a)| {
                *a = val & (1 << i) > 0;
            });
        self.bass_management.parse_quadlet(&raw[4..8]);
    }
}

/// The number of pairs of physical output.
pub const STUDIO_PHYS_OUT_PAIR_COUNT: usize = 11;

/// The number of speaker sets to consists of several physical outputs.
pub const STUDIO_VIRTUAL_SPEAKER_COUNT: usize = 3;

/// The structure to represent data of physical out segment.
#[derive(Default, Debug)]
pub struct StudioPhysOut{
    /// The configuration for master output
    pub master_out: OutPair,
    /// The source for pairs of physical output. It includes below pairs in
    /// the order:
    /// - main out 1/2
    /// - phone out 1/2
    /// - line out 5/6, 7/8, 9/10, 11/12,
    /// - S/PDIF out 1/2,
    /// - ADAT out 1/2, 3/4, 5/6, 7/8,
    pub out_pair_srcs: [PhysOutPairSrc;STUDIO_PHYS_OUT_PAIR_COUNT],
    /// The state of assignment to speakers.
    pub spkr_assigns: [bool;STUDIO_PHYS_OUT_PAIR_COUNT],
    /// Whether to mute any source to the physical output.
    pub muted: [bool;STUDIO_PHYS_OUT_PAIR_COUNT],
    /// The settings of each virtual speaker.
    pub speakers: [VirtualSpeaker;STUDIO_VIRTUAL_SPEAKER_COUNT],
}

impl StudioPhysOut {
    const SIZE: usize = 440;
}

impl TcKonnektSegmentData for StudioPhysOut {
    fn build(&self, raw: &mut [u8]) {
        self.master_out.build(&mut raw[..12]);
        self.out_pair_srcs.iter()
            .enumerate()
            .for_each(|(i, p)| {
                let pos = 16 + i * PhysOutPairSrc::SIZE;
                p.build(&mut raw[pos..(pos + PhysOutPairSrc::SIZE)]);
            });
        let mut val = 0u32;
        self.spkr_assigns.iter()
            .enumerate()
            .filter(|(_, &m)| m)
            .for_each(|(i, _)| {
                val |= 1 << i;
            });
        val.build_quadlet(&mut raw[324..328]);
        let mut val = 0u32;
        self.muted.iter()
            .enumerate()
            .filter(|(_, &d)| d)
            .for_each(|(i, _)| {
                val |= 1 << i;
            });
        val.build_quadlet(&mut raw[328..332]);
        self.speakers.iter()
            .enumerate()
            .for_each(|(i, s)| {
                let pos = 332 + VirtualSpeaker::SIZE * i;
                s.build(&mut raw[pos..(pos + VirtualSpeaker::SIZE)]);
            });
    }

    fn parse(&mut self, raw: &[u8]) {
        self.master_out.parse(&raw[..12]);
        self.out_pair_srcs.iter_mut()
            .enumerate()
            .for_each(|(i, p)| {
                let pos = 16 + i * PhysOutPairSrc::SIZE;
                p.parse(&raw[pos..(pos + PhysOutPairSrc::SIZE)]);
            });
        let mut val = 0u32;
        val.parse_quadlet(&raw[324..328]);
        self.spkr_assigns.iter_mut()
            .enumerate()
            .for_each(|(i, m)| {
                *m = val & (1 << i) > 0;
            });
        let mut val = 0u32;
        val.parse_quadlet(&raw[328..332]);
        self.muted.iter_mut()
            .enumerate()
            .for_each(|(i, d)| {
                *d = val & (1 << i) > 0;
            });
        self.speakers.iter_mut()
            .enumerate()
            .for_each(|(i, s)| {
                let pos = 332 + VirtualSpeaker::SIZE * i;
                s.parse(&raw[pos..(pos + VirtualSpeaker::SIZE)]);
            });
    }
}

impl TcKonnektSegmentSpec for TcKonnektSegment<StudioPhysOut> {
    const OFFSET: usize = 0x03dc;
    const SIZE: usize = StudioPhysOut::SIZE;
}

impl TcKonnektNotifiedSegmentSpec for TcKonnektSegment<StudioPhysOut> {
    const NOTIFY_FLAG: u32 = STUDIO_PHYS_OUT_NOTIFY_FLAG;
}

const STUDIO_CH_STRIP_COUNT: usize = 4;

#[derive(Default, Debug)]
pub struct StudioReverbState(ReverbState);

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

impl TcKonnektSegmentData for StudioReverbState {
    fn build(&self, raw: &mut [u8]) {
        self.0.build(raw)
    }

    fn parse(&mut self, raw: &[u8]) {
        self.0.parse(raw)
    }
}

impl TcKonnektSegmentSpec for TcKonnektSegment<StudioReverbState> {
    const OFFSET: usize = 0x0594;
    const SIZE: usize = ReverbState::SIZE;
}

impl TcKonnektNotifiedSegmentSpec for TcKonnektSegment<StudioReverbState> {
    const NOTIFY_FLAG: u32 = STUDIO_REVERB_NOTIFY_CHANGE;
}

#[derive(Default, Debug)]
pub struct StudioChStripStates([ChStripState;STUDIO_CH_STRIP_COUNT]);

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

impl TcKonnektSegmentData for StudioChStripStates {
    fn build(&self, raw: &mut [u8]) {
        self.0.build(raw)
    }

    fn parse(&mut self, raw: &[u8]) {
        self.0.parse(raw)
    }
}

impl TcKonnektSegmentSpec for TcKonnektSegment<StudioChStripStates> {
    const OFFSET: usize = 0x05d8;
    const SIZE: usize = ChStripState::SIZE * STUDIO_CH_STRIP_COUNT + 8;
}

impl TcKonnektNotifiedSegmentSpec for TcKonnektSegment<StudioChStripStates> {
    const NOTIFY_FLAG: u32 = STUDIO_CH_STRIP_NOTIFY_01_CHANGE | STUDIO_CH_STRIP_NOTIFY_23_CHANGE;
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
/// The enumeration to represent state of jack sense for analog input.
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

#[derive(Default, Debug)]
/// The structure to represent hardware state.
pub struct StudioHwState{
    pub analog_jack_states: [StudioAnalogJackState;STUDIO_ANALOG_JACK_STATE_COUNT],
    pub hp_state: [bool;2],
    pub firewire_led: FireWireLedState,
    pub valid_master_level: bool,
}

impl StudioHwState {
    const SIZE: usize = 68;
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

impl TcKonnektSegmentData for StudioHwState {
    fn build(&self, raw: &mut [u8]) {
        self.analog_jack_states.build_quadlet_block(&mut raw[..48]);
        self.hp_state.build_quadlet_block(&mut raw[48..56]);
        self.firewire_led.build_quadlet(&mut raw[56..60]);
        self.valid_master_level.build_quadlet(&mut raw[60..64]);
    }

    fn parse(&mut self, raw: &[u8]) {
        self.analog_jack_states.parse_quadlet_block(&raw[..48]);
        self.hp_state.parse_quadlet_block(&raw[48..56]);
        self.firewire_led.parse_quadlet(&raw[56..60]);
        self.valid_master_level.parse_quadlet(&raw[60..64]);
    }
}

impl TcKonnektSegmentSpec for TcKonnektSegment<StudioHwState> {
    const OFFSET: usize = 0x2008;
    const SIZE: usize = StudioHwState::SIZE;
}

impl TcKonnektNotifiedSegmentSpec for TcKonnektSegment<StudioHwState> {
    const NOTIFY_FLAG: u32 = STUDIO_HW_STATE_NOTIFY_FLAG;
}

/// The structure to represent meter for input/output of mixer.
#[derive(Default, Debug)]
pub struct StudioMixerMeter{
    pub src_inputs: [i32;24],
    pub mixer_outputs: [i32;2],
    pub aux_outputs: [i32;4],
}

impl StudioMixerMeter {
    const SIZE: usize = 128;
}

impl TcKonnektSegmentData for StudioMixerMeter {
    fn build(&self, raw: &mut [u8]) {
        self.src_inputs.build_quadlet_block(&mut raw[4..100]);
        self.mixer_outputs.build_quadlet_block(&mut raw[100..108]);
        self.aux_outputs.build_quadlet_block(&mut raw[108..124]);
    }

    fn parse(&mut self, raw: &[u8]) {
        self.src_inputs.parse_quadlet_block(&raw[4..100]);
        self.mixer_outputs.parse_quadlet_block(&raw[100..108]);
        self.aux_outputs.parse_quadlet_block(&raw[108..124]);
    }
}

impl TcKonnektSegmentSpec for TcKonnektSegment<StudioMixerMeter> {
    const OFFSET: usize = 0x20b8;
    const SIZE: usize = StudioMixerMeter::SIZE;
}

#[derive(Default, Debug)]
pub struct StudioReverbMeter(ReverbMeter);

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

impl TcKonnektSegmentData for StudioReverbMeter {
    fn build(&self, raw: &mut [u8]) {
        self.0.build(raw)
    }

    fn parse(&mut self, raw: &[u8]) {
        self.0.parse(raw)
    }
}

impl TcKonnektSegmentSpec for TcKonnektSegment<StudioReverbMeter> {
    const OFFSET: usize = 0x2164;
    const SIZE: usize = ReverbMeter::SIZE;
}

#[derive(Default, Debug)]
pub struct StudioChStripMeters([ChStripMeter;STUDIO_CH_STRIP_COUNT]);

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

impl TcKonnektSegmentData for StudioChStripMeters {
    fn build(&self, raw: &mut [u8]) {
        self.0.build(raw)
    }

    fn parse(&mut self, raw: &[u8]) {
        self.0.parse(raw)
    }
}

impl TcKonnektSegmentSpec for TcKonnektSegment<StudioChStripMeters> {
    const OFFSET: usize = 0x217c;
    const SIZE: usize = ChStripMeter::SIZE * STUDIO_CH_STRIP_COUNT + 8;
}
