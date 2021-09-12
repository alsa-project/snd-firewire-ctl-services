// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Protocol defined by TC Electronic for Desktop Konnekt 6.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by TC Electronic for Desktop Konnekt 6.

use crate::*;

use crate::tcelectronic::{fw_led::*, standalone::*, *};

const DESKTOP_HW_STATE_NOTIFY_FLAG: u32 = 0x00010000;
const DESKTOP_CONFIG_NOTIFY_FLAG: u32 = 0x00020000;
const DESKTOP_MIXER_STATE_NOTIFY_FLAG: u32 = 0x00040000;
const DESKTOP_PANEL_NOTIFY_FLAG: u32 = 0x00080000;

/// The structure for protocol implementation of Desktop Konnekt 6.
#[derive(Default)]
pub struct Desktopk6Protocol;

/// Segment for panel. 0x0008..0x0097 (36 quads).
pub type Desktopk6HwStateSegment = TcKonnektSegment<DesktopHwState>;
impl SegmentOperation<DesktopHwState> for Desktopk6Protocol {}

/// Segment for configuration. 0x0098..0x00b7 (8 quads).
pub type Desktopk6ConfigSegment = TcKonnektSegment<DesktopConfig>;
impl SegmentOperation<DesktopConfig> for Desktopk6Protocol {}

/// Segment for state of mixer. 0x00b8..0x0367 (172 quads).
pub type Desktopk6MixerStateSegment = TcKonnektSegment<DesktopMixerState>;
impl SegmentOperation<DesktopMixerState> for Desktopk6Protocol {}

/// Segment for panel. 0x2008..0x2047 (15 quads).
pub type Desktopk6PanelSegment = TcKonnektSegment<DesktopPanel>;
impl SegmentOperation<DesktopPanel> for Desktopk6Protocol {}

/// Segment for meter. 0x20e4..0x213f (23 quads).
pub type Desktopk6MeterSegment = TcKonnektSegment<DesktopMeter>;
impl SegmentOperation<DesktopMeter> for Desktopk6Protocol {}

/// The enumeration to represent target of meter.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum MeterTarget {
    Input,
    Pre,
    Post,
}

impl Default for MeterTarget {
    fn default() -> Self {
        Self::Input
    }
}

impl From<u32> for MeterTarget {
    fn from(val: u32) -> Self {
        match val {
            2 => Self::Post,
            1 => Self::Pre,
            _ => Self::Input,
        }
    }
}

impl From<MeterTarget> for u32 {
    fn from(target: MeterTarget) -> Self {
        match target {
            MeterTarget::Post => 2,
            MeterTarget::Pre => 1,
            MeterTarget::Input => 0,
        }
    }
}

/// The enumeration to represent current scene.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum InputScene {
    MicInst,
    DualInst,
    StereoIn,
}

impl Default for InputScene {
    fn default() -> Self {
        Self::MicInst
    }
}

impl From<u32> for InputScene {
    fn from(val: u32) -> Self {
        match val {
            2 => Self::StereoIn,
            1 => Self::DualInst,
            _ => Self::MicInst,
        }
    }
}

impl From<InputScene> for u32 {
    fn from(target: InputScene) -> Self {
        match target {
            InputScene::StereoIn => 2,
            InputScene::DualInst => 1,
            InputScene::MicInst => 0,
        }
    }
}

/// The structure to represent state of panel.
#[derive(Default, Debug)]
pub struct DesktopHwState {
    pub meter_target: MeterTarget,
    pub mixer_output_monaural: bool,
    /// Whether to assign main knob to hp.
    pub knob_assign_to_hp: bool,
    /// Whether to dim output.
    pub mixer_output_dim_enabled: bool,
    /// The volume of dimmed output. -1000..-60. (-94.0..-6.0)
    pub mixer_output_dim_volume: i32,
    pub input_scene: InputScene,
    pub reverb_to_master: bool,
    pub reverb_to_hp: bool,
    /// Turn on backlight in master knob.
    pub master_knob_backlight: bool,
    /// Phantom powering in mic 0.
    pub mic_0_phantom: bool,
    /// Signal boost in mic 0 by 12 dB.
    pub mic_0_boost: bool,
}

impl DesktopHwState {
    const SIZE: usize = 144;

    const REVERB_TO_MAIN_MASK: u32 = 0x00000001;
    const REVERB_TO_HP_MASK: u32 = 0x00000002;
}

impl TcKonnektSegmentData for DesktopHwState {
    fn build(&self, raw: &mut [u8]) {
        self.meter_target.build_quadlet(&mut raw[..4]);
        self.mixer_output_monaural.build_quadlet(&mut raw[4..8]);
        self.knob_assign_to_hp.build_quadlet(&mut raw[8..12]);
        self.mixer_output_dim_enabled
            .build_quadlet(&mut raw[12..16]);
        self.mixer_output_dim_volume.build_quadlet(&mut raw[16..20]);
        self.input_scene.build_quadlet(&mut raw[20..24]);

        let mut val = 0;
        if self.reverb_to_master {
            val |= Self::REVERB_TO_MAIN_MASK;
        }
        if self.reverb_to_hp {
            val |= Self::REVERB_TO_HP_MASK;
        }
        val.build_quadlet(&mut raw[28..32]);

        self.master_knob_backlight.build_quadlet(&mut raw[32..36]);
        self.mic_0_phantom.build_quadlet(&mut raw[52..56]);
        self.mic_0_boost.build_quadlet(&mut raw[56..60]);
    }

    fn parse(&mut self, raw: &[u8]) {
        self.meter_target.parse_quadlet(&raw[..4]);
        self.mixer_output_monaural.parse_quadlet(&raw[4..8]);
        self.knob_assign_to_hp.parse_quadlet(&raw[8..12]);
        self.mixer_output_dim_enabled.parse_quadlet(&raw[12..16]);
        self.mixer_output_dim_volume.parse_quadlet(&raw[16..20]);
        self.input_scene.parse_quadlet(&raw[20..24]);

        let mut val = 0;
        val.parse_quadlet(&raw[28..32]);
        self.reverb_to_master = val & Self::REVERB_TO_MAIN_MASK > 0;
        self.reverb_to_hp = val & Self::REVERB_TO_HP_MASK > 0;

        self.master_knob_backlight.parse_quadlet(&raw[32..36]);
        self.mic_0_phantom.parse_quadlet(&raw[52..56]);
        self.mic_0_boost.parse_quadlet(&raw[56..60]);
    }
}

impl TcKonnektSegmentSpec for TcKonnektSegment<DesktopHwState> {
    const OFFSET: usize = 0x0008;
    const SIZE: usize = DesktopHwState::SIZE;
}

impl TcKonnektNotifiedSegmentSpec for TcKonnektSegment<DesktopHwState> {
    const NOTIFY_FLAG: u32 = DESKTOP_HW_STATE_NOTIFY_FLAG;
}

/// The structure to represent configuration.
#[derive(Default, Debug)]
pub struct DesktopConfig {
    pub standalone_rate: TcKonnektStandaloneClkRate,
}

impl DesktopConfig {
    const SIZE: usize = 32;
}

impl TcKonnektSegmentData for DesktopConfig {
    fn build(&self, raw: &mut [u8]) {
        self.standalone_rate.build_quadlet(&mut raw[4..8]);
    }

    fn parse(&mut self, raw: &[u8]) {
        self.standalone_rate.parse_quadlet(&raw[4..8]);
    }
}

impl TcKonnektSegmentSpec for TcKonnektSegment<DesktopConfig> {
    const OFFSET: usize = 0x0098;
    const SIZE: usize = DesktopConfig::SIZE;
}

impl TcKonnektNotifiedSegmentSpec for TcKonnektSegment<DesktopConfig> {
    const NOTIFY_FLAG: u32 = DESKTOP_CONFIG_NOTIFY_FLAG;
}

/// The enumeration to represent source of headphone.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum DesktopHpSrc {
    Stream23,
    Mixer01,
}

impl Default for DesktopHpSrc {
    fn default() -> Self {
        Self::Stream23
    }
}

impl From<u32> for DesktopHpSrc {
    fn from(val: u32) -> Self {
        match val {
            0x05 => Self::Stream23,
            _ => Self::Mixer01,
        }
    }
}

impl From<DesktopHpSrc> for u32 {
    fn from(src: DesktopHpSrc) -> Self {
        match src {
            DesktopHpSrc::Stream23 => 0x05,
            DesktopHpSrc::Mixer01 => 0x0b,
        }
    }
}

/// The structure to represent state of mixer.
#[derive(Default, Debug)]
pub struct DesktopMixerState {
    /// The input level for ch 0 and 1. -1000..0 (-94.0..0.0 dB)
    pub mic_inst_level: [i32; 2],
    /// The LR balance for ch 0 and 1. -50..50.
    pub mic_inst_pan: [i32; 2],
    /// The level to send for ch 0 and 1. -1000..0 (-94.0..0.0 dB)
    pub mic_inst_send: [i32; 2],
    /// The input level for ch 0 and 1. -1000..0 (-94.0..0.0 dB)
    pub dual_inst_level: [i32; 2],
    /// The LR balance for ch 0 and 1. -50..50.
    pub dual_inst_pan: [i32; 2],
    /// The level to send for ch 0 and 1. -1000..0 (-94.0..0.0 dB)
    pub dual_inst_send: [i32; 2],
    /// The input level for both channels. -1000..0 (-94.0..0.0 dB)
    pub stereo_in_level: i32,
    /// The LR balance for both channels. -50..50.
    pub stereo_in_pan: i32,
    /// The level to send for both channels. -1000..0 (-94.0..0.0 dB)
    pub stereo_in_send: i32,
    /// The source of headphone.
    pub hp_src: DesktopHpSrc,
}

impl DesktopMixerState {
    const SIZE: usize = 688;
}

impl TcKonnektSegmentData for DesktopMixerState {
    fn build(&self, raw: &mut [u8]) {
        self.mic_inst_level[0].build_quadlet(&mut raw[12..16]);
        self.mic_inst_pan[0].build_quadlet(&mut raw[16..20]);
        self.mic_inst_send[0].build_quadlet(&mut raw[20..24]);
        self.mic_inst_level[1].build_quadlet(&mut raw[28..32]);
        self.mic_inst_pan[1].build_quadlet(&mut raw[32..36]);
        self.mic_inst_send[1].build_quadlet(&mut raw[40..44]);

        self.dual_inst_level[0].build_quadlet(&mut raw[228..232]);
        self.dual_inst_pan[0].build_quadlet(&mut raw[232..236]);
        self.dual_inst_send[0].build_quadlet(&mut raw[240..244]);
        self.dual_inst_level[1].build_quadlet(&mut raw[248..252]);
        self.dual_inst_pan[1].build_quadlet(&mut raw[252..256]);
        self.dual_inst_send[1].build_quadlet(&mut raw[260..264]);

        self.stereo_in_level.build_quadlet(&mut raw[444..448]);
        self.stereo_in_pan.build_quadlet(&mut raw[448..452]);
        self.stereo_in_send.build_quadlet(&mut raw[452..456]);

        self.hp_src.build_quadlet(&mut raw[648..652]);
    }

    fn parse(&mut self, raw: &[u8]) {
        self.mic_inst_level[0].parse_quadlet(&raw[12..16]);
        self.mic_inst_pan[0].parse_quadlet(&raw[16..20]);
        self.mic_inst_send[0].parse_quadlet(&raw[20..24]);
        self.mic_inst_level[1].parse_quadlet(&raw[28..32]);
        self.mic_inst_pan[1].parse_quadlet(&raw[32..36]);
        self.mic_inst_send[1].parse_quadlet(&raw[40..44]);

        self.dual_inst_level[0].parse_quadlet(&raw[228..232]);
        self.dual_inst_pan[0].parse_quadlet(&raw[232..236]);
        self.dual_inst_send[0].parse_quadlet(&raw[240..244]);
        self.dual_inst_level[1].parse_quadlet(&raw[248..252]);
        self.dual_inst_pan[1].parse_quadlet(&raw[252..256]);
        self.dual_inst_send[1].parse_quadlet(&raw[260..264]);

        self.stereo_in_level.parse_quadlet(&raw[444..448]);
        self.stereo_in_pan.parse_quadlet(&raw[448..452]);
        self.stereo_in_send.parse_quadlet(&raw[452..456]);

        self.hp_src.parse_quadlet(&raw[648..652]);
    }
}

impl TcKonnektSegmentSpec for TcKonnektSegment<DesktopMixerState> {
    const OFFSET: usize = 0x00b8;
    const SIZE: usize = DesktopMixerState::SIZE;
}

impl TcKonnektNotifiedSegmentSpec for TcKonnektSegment<DesktopMixerState> {
    const NOTIFY_FLAG: u32 = DESKTOP_MIXER_STATE_NOTIFY_FLAG;
}

#[derive(Default, Debug)]
pub struct DesktopPanel {
    /// The count of panel button to push.
    pub panel_button_count: u32,
    /// The value of main knob. -1000..0
    pub main_knob_value: i32,
    /// The value of phone knob. -1000..0
    pub phone_knob_value: i32,
    /// The value of mix knob. 0..1000.
    pub mix_knob_value: u32,
    /// The state of reverb LED.
    pub reverb_led_on: bool,
    /// The value of reverb knob. -1000..0
    pub reverb_knob_value: i32,
    /// The state of FireWire LED.
    pub firewire_led: FireWireLedState,
}

impl DesktopPanel {
    const SIZE: usize = 64;
}

impl TcKonnektSegmentData for DesktopPanel {
    fn build(&self, raw: &mut [u8]) {
        self.panel_button_count.build_quadlet(&mut raw[..4]);
        self.main_knob_value.build_quadlet(&mut raw[4..8]);
        self.phone_knob_value.build_quadlet(&mut raw[8..12]);
        self.mix_knob_value.build_quadlet(&mut raw[12..16]);
        self.reverb_led_on.build_quadlet(&mut raw[16..20]);
        self.reverb_knob_value.build_quadlet(&mut raw[24..28]);
        self.firewire_led.build_quadlet(&mut raw[36..40]);
    }

    fn parse(&mut self, raw: &[u8]) {
        self.panel_button_count.parse_quadlet(&raw[..4]);
        self.main_knob_value.parse_quadlet(&raw[4..8]);
        self.phone_knob_value.parse_quadlet(&raw[8..12]);
        self.mix_knob_value.parse_quadlet(&raw[12..16]);
        self.reverb_led_on.parse_quadlet(&raw[16..20]);
        self.reverb_knob_value.parse_quadlet(&raw[24..28]);
        self.firewire_led.parse_quadlet(&raw[36..40]);
    }
}

impl TcKonnektSegmentSpec for TcKonnektSegment<DesktopPanel> {
    const OFFSET: usize = 0x2008;
    const SIZE: usize = DesktopPanel::SIZE;
}

impl TcKonnektNotifiedSegmentSpec for TcKonnektSegment<DesktopPanel> {
    const NOTIFY_FLAG: u32 = DESKTOP_PANEL_NOTIFY_FLAG;
}

#[derive(Default, Debug)]
pub struct DesktopMeter {
    pub analog_inputs: [i32; 2],
    pub mixer_outputs: [i32; 2],
    pub stream_inputs: [i32; 2],
}

impl DesktopMeter {
    const SIZE: usize = 92;
}

impl TcKonnektSegmentData for DesktopMeter {
    fn build(&self, raw: &mut [u8]) {
        self.analog_inputs.build_quadlet_block(&mut raw[..8]);
        self.mixer_outputs.build_quadlet_block(&mut raw[40..48]);
        self.stream_inputs.build_quadlet_block(&mut raw[48..56]);
    }

    fn parse(&mut self, raw: &[u8]) {
        self.analog_inputs.parse_quadlet_block(&raw[..8]);
        self.mixer_outputs.parse_quadlet_block(&raw[40..48]);
        self.stream_inputs.parse_quadlet_block(&raw[48..56]);
    }
}

impl TcKonnektSegmentSpec for TcKonnektSegment<DesktopMeter> {
    const OFFSET: usize = 0x20e4;
    const SIZE: usize = DesktopMeter::SIZE;
}
