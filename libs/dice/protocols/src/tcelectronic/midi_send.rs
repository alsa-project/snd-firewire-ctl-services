// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Data of MIDI sender in protocol defined by TC Electronic for Konnekt series.
//!
//! The module includes structure, trait and its implementation for data of MIDI sender in protocol
//! defined by TC Electronic for Konnekt series.

use super::*;

/// The structure to represent channel and control code of MIDI event.
#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
pub struct TcKonnektMidiMsgParams {
    /// The channel for MIDI message.
    pub ch: u8,
    /// The control code for MIDI message.
    pub cc: u8,
}

/// The structure to represent MIDI sender.
#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
pub struct TcKonnektMidiSender {
    /// The parameter of MIDI message generated normally.
    pub normal: TcKonnektMidiMsgParams,
    /// The parameter of MIDI message generated when knob is pushed.
    pub pushed: TcKonnektMidiMsgParams,
    /// Whether to send MIDI message to physical MIDI port.
    pub send_to_port: bool,
    /// Whether to deliver MIDI message by tx stream.
    pub send_to_stream: bool,
}

impl TcKonnektMidiSender {
    pub const SIZE: usize = 36;

    pub fn build(&self, raw: &mut [u8]) {
        self.normal.ch.build_quadlet(&mut raw[..4]);
        self.normal.cc.build_quadlet(&mut raw[4..8]);
        self.pushed.ch.build_quadlet(&mut raw[12..16]);
        self.pushed.cc.build_quadlet(&mut raw[16..20]);
        self.send_to_port.build_quadlet(&mut raw[24..28]);
        self.send_to_stream.build_quadlet(&mut raw[28..32]);
    }

    pub fn parse(&mut self, raw: &[u8]) {
        self.normal.ch.parse_quadlet(&raw[..4]);
        self.normal.cc.parse_quadlet(&raw[4..8]);
        self.pushed.ch.parse_quadlet(&raw[12..16]);
        self.pushed.cc.parse_quadlet(&raw[16..20]);
        self.send_to_port.parse_quadlet(&raw[24..28]);
        self.send_to_stream.parse_quadlet(&raw[28..32]);
    }
}
