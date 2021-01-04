// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Protocol defined by TC Electronic for Desktop Konnekt 6.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by TC Electronic for Desktop Konnekt 6.

use crate::*;

use crate::tcelectronic::{*, fw_led::*};

const DESKTOP_MIXER_STATE_NOTIFY_FLAG: u32 = 0x00040000;
const DESKTOP_PANEL_NOTIFY_FLAG: u32 = 0x00080000;

/// The structure to represent segments in memory space of Desktop Konnekt 6.
#[derive(Default, Debug)]
pub struct DesktopSegments{
    /// Segment for state of mixer. 0x00b8..0x0367 (172 quads).
    pub mixer: TcKonnektSegment<DesktopMixerState>,
    /// Segment for panel. 0x2008..0x2047 (15 quads).
    pub panel: TcKonnektSegment<DesktopPanel>,
    /// Segment for meter. 0x20e4..0x213f (23 quads).
    pub meter: TcKonnektSegment<DesktopMeter>,
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
pub struct DesktopMixerState{
    /// The input level for ch 0 and 1. -1000..0 (-94.0..0.0 dB)
    pub mic_inst_level: [i32;2],
    /// The LR balance for ch 0 and 1. -50..50.
    pub mic_inst_pan: [i32;2],
    /// The level to send for ch 0 and 1. -1000..0 (-94.0..0.0 dB)
    pub mic_inst_send: [i32;2],
    /// The input level for ch 0 and 1. -1000..0 (-94.0..0.0 dB)
    pub dual_inst_level: [i32;2],
    /// The LR balance for ch 0 and 1. -50..50.
    pub dual_inst_pan: [i32;2],
    /// The level to send for ch 0 and 1. -1000..0 (-94.0..0.0 dB)
    pub dual_inst_send: [i32;2],
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
pub struct DesktopPanel{
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

impl AsRef<FireWireLedState> for DesktopPanel {
    fn as_ref(&self) -> &FireWireLedState {
        &self.firewire_led
    }
}

impl AsMut<FireWireLedState> for DesktopPanel {
    fn as_mut(&mut self) -> &mut FireWireLedState {
        &mut self.firewire_led
    }
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
pub struct DesktopMeter{
    pub analog_inputs: [i32;2],
    pub mixer_outputs: [i32;2],
    pub stream_inputs: [i32;2],
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
