// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Protocol defined by TC Electronic for Desktop Konnekt 6.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by TC Electronic for Desktop Konnekt 6.

use crate::*;

use crate::tcelectronic::{*, fw_led::*};

const DESKTOP_PANEL_NOTIFY_FLAG: u32 = 0x00080000;

/// The structure to represent segments in memory space of Desktop Konnekt 6.
#[derive(Default, Debug)]
pub struct DesktopSegments{
    /// Segment for panel. 0x2008..0x2047 (15 quads).
    pub panel: TcKonnektSegment<DesktopPanel>,
    /// Segment for meter. 0x20e4..0x213f (23 quads).
    pub meter: TcKonnektSegment<DesktopMeter>,
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
