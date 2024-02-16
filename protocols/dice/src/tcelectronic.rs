// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Protocol defined by TC Electronic.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by TC Electronic.
//!
//! In Konnekt series, the accessible memory space is separated to several segments.

pub mod desktop;
pub mod shell;
pub mod studio;

pub mod ch_strip;
pub mod reverb;

use {
    super::{tcat::*, *},
    ta1394_avc_general::{general::*, *},
};

// The base offset to operate functions in Konnekt series.
const BASE_OFFSET: usize = 0x00a01000;

/// The generic structure for segment. In Konnekt series, the accessible memory space is separated
/// to several segments. This structure expresses them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TcKonnektSegment<U> {
    /// Intermediate structured data for parameters.
    pub data: U,
    /// Raw byte data for memory layout in hardware.
    raw: Vec<u8>,
}

/// Serialize and deserialize for segment in TC Konnekt protocol.
pub trait TcKonnektSegmentSerdes<T> {
    /// The name of segment.
    const NAME: &'static str;

    /// The offset of segment.
    const OFFSET: usize;

    /// The size of segment.
    const SIZE: usize;

    /// Serialize for parameter.
    fn serialize(params: &T, raw: &mut [u8]) -> Result<(), String>;

    /// Deserialize for parameter.
    fn deserialize(params: &mut T, raw: &[u8]) -> Result<(), String>;
}

fn generate_error(segment_name: &str, cause: &str, raw: &[u8]) -> Error {
    let msg = format!(
        "segment: {}, cause: '{}', raw: {:02x?}",
        segment_name, cause, raw
    );
    Error::new(GeneralProtocolError::VendorDependent, &msg)
}

/// Operation to cache content of segment in TC Electronic Konnekt series.
pub trait TcKonnektSegmentOperation<T>: TcatOperation + TcKonnektSegmentSerdes<T> {
    /// Cache whole segment and deserialize for parameters.
    fn cache_whole_segment(
        req: &FwReq,
        node: &FwNode,
        segment: &mut TcKonnektSegment<T>,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        // NOTE: Something wrong to implement Default trait.
        assert_eq!(segment.raw.len(), Self::SIZE);

        Self::read(
            req,
            node,
            BASE_OFFSET + Self::OFFSET,
            &mut segment.raw,
            timeout_ms,
        )?;

        Self::deserialize(&mut segment.data, &segment.raw)
            .map_err(|cause| generate_error(Self::NAME, &cause, &segment.raw))
    }
}

impl<O: TcatOperation + TcKonnektSegmentSerdes<T>, T> TcKonnektSegmentOperation<T> for O {}

/// Operation to update content of segment in TC Electronic Konnekt series.
pub trait TcKonnektMutableSegmentOperation<T>: TcatOperation + TcKonnektSegmentSerdes<T> {
    /// Update part of segment for any change at the parameters.
    fn update_partial_segment(
        req: &FwReq,
        node: &FwNode,
        params: &T,
        segment: &mut TcKonnektSegment<T>,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        // NOTE: Something wrong to implement Default trait.
        assert_eq!(segment.raw.len(), Self::SIZE);

        let mut raw = segment.raw.clone();
        Self::serialize(params, &mut raw)
            .map_err(|cause| generate_error(Self::NAME, &cause, &segment.raw))?;

        (0..Self::SIZE).step_by(4).try_for_each(|pos| {
            let new = &mut raw[pos..(pos + 4)];
            if new != &segment.raw[pos..(pos + 4)] {
                Self::write(req, node, BASE_OFFSET + Self::OFFSET + pos, new, timeout_ms)
                    .map(|_| segment.raw[pos..(pos + 4)].copy_from_slice(new))
            } else {
                Ok(())
            }
        })?;

        Self::deserialize(&mut segment.data, &raw)
            .map_err(|cause| generate_error(Self::NAME, &cause, &segment.raw))
    }

    /// Update whole segment by the parameters.
    fn update_whole_segment(
        req: &FwReq,
        node: &FwNode,
        params: &T,
        segment: &mut TcKonnektSegment<T>,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        // NOTE: Something wrong to implement Default trait.
        assert_eq!(segment.raw.len(), Self::SIZE);

        let mut raw = segment.raw.clone();
        Self::serialize(&params, &mut raw)
            .map_err(|cause| generate_error(Self::NAME, &cause, &segment.raw))?;

        Self::write(req, node, BASE_OFFSET + Self::OFFSET, &mut raw, timeout_ms)?;

        segment.raw.copy_from_slice(&raw);
        Self::deserialize(&mut segment.data, &segment.raw)
            .map_err(|cause| generate_error(Self::NAME, &cause, &segment.raw))
    }
}

/// Operation for segment in which any change is notified in TC Electronic Konnekt series.
pub trait TcKonnektNotifiedSegmentOperation<T> {
    const NOTIFY_FLAG: u32;

    /// Check message to be notified or not.
    fn is_notified_segment(_: &TcKonnektSegment<T>, msg: u32) -> bool {
        msg & Self::NOTIFY_FLAG > 0
    }
}

fn serialize_position<T: Eq + std::fmt::Debug>(
    entries: &[T],
    entry: &T,
    raw: &mut [u8],
    label: &str,
) -> Result<(), String> {
    assert!(raw.len() >= 4);

    entries
        .iter()
        .position(|t| entry.eq(t))
        .ok_or_else(|| format!("{} {:?} is not supported", label, entry))
        .map(|pos| serialize_usize(&pos, raw))
}

fn deserialize_position<T: Copy + Eq + std::fmt::Debug>(
    entries: &[T],
    entry: &mut T,
    raw: &[u8],
    label: &str,
) -> Result<(), String> {
    assert!(raw.len() >= 4);

    let mut val = 0usize;
    deserialize_usize(&mut val, raw);

    entries
        .iter()
        .nth(val as usize)
        .ok_or_else(|| format!("{} not found for index {}", label, val))
        .map(|&e| *entry = e)
}

/// The state of FireWire LED in TC Konnekt Protocol.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum FireWireLedState {
    /// Off.
    Off,
    /// On.
    On,
    /// Blinking fastly.
    BlinkFast,
    /// Blinking slowly.
    BlinkSlow,
}

impl Default for FireWireLedState {
    fn default() -> Self {
        Self::Off
    }
}

const FW_LED_STATES: &[FireWireLedState] = &[
    FireWireLedState::Off,
    FireWireLedState::On,
    FireWireLedState::BlinkSlow,
    FireWireLedState::BlinkFast,
];

const FW_LED_STATE_LABEL: &str = "FireWire LED state";

fn serialize_fw_led_state(state: &FireWireLedState, raw: &mut [u8]) -> Result<(), String> {
    serialize_position(FW_LED_STATES, state, raw, FW_LED_STATE_LABEL)
}

fn deserialize_fw_led_state(state: &mut FireWireLedState, raw: &[u8]) -> Result<(), String> {
    deserialize_position(FW_LED_STATES, state, raw, FW_LED_STATE_LABEL)
}

/// Available rate for sampling clock in standalone mode in TC Konnekt protocol.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TcKonnektStandaloneClockRate {
    /// At 44.1 kHz.
    R44100,
    /// At 48.0 kHz.
    R48000,
    /// At 88.2 kHz.
    R88200,
    /// At 96.0 kHz.
    R96000,
}

impl Default for TcKonnektStandaloneClockRate {
    fn default() -> Self {
        Self::R44100
    }
}

fn serialize_standalone_clock_rate(
    rate: &TcKonnektStandaloneClockRate,
    raw: &mut [u8],
) -> Result<(), String> {
    assert!(raw.len() >= 4);

    let val = match rate {
        TcKonnektStandaloneClockRate::R96000 => 4,
        TcKonnektStandaloneClockRate::R88200 => 3,
        TcKonnektStandaloneClockRate::R48000 => 2,
        TcKonnektStandaloneClockRate::R44100 => 1,
    };

    serialize_u32(&val, raw);

    Ok(())
}

fn deserialize_standalone_clock_rate(
    rate: &mut TcKonnektStandaloneClockRate,
    raw: &[u8],
) -> Result<(), String> {
    assert!(raw.len() >= 4);

    let mut val = 0u32;
    deserialize_u32(&mut val, raw);

    *rate = match val {
        4 => TcKonnektStandaloneClockRate::R96000,
        3 => TcKonnektStandaloneClockRate::R88200,
        2 => TcKonnektStandaloneClockRate::R48000,
        1 => TcKonnektStandaloneClockRate::R44100,
        _ => Err(format!(
            "Unexpected value for standalone clock rate: {}",
            val
        ))?,
    };

    Ok(())
}

/// Channel and control code of MIDI event in TC Konnekt protocol.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct TcKonnektMidiMsgParams {
    /// The channel for MIDI message.
    pub ch: u8,
    /// The control code for MIDI message.
    pub cc: u8,
}

/// MIDI sender settings in TC Konnekt protocol.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
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
    pub(crate) const SIZE: usize = 36;
}

fn serialize_midi_sender(sender: &TcKonnektMidiSender, raw: &mut [u8]) -> Result<(), String> {
    assert!(raw.len() >= TcKonnektMidiSender::SIZE);

    serialize_u8(&sender.normal.ch, &mut raw[..4]);
    serialize_u8(&sender.normal.cc, &mut raw[4..8]);
    serialize_u8(&sender.pushed.ch, &mut raw[12..16]);
    serialize_u8(&sender.pushed.cc, &mut raw[16..20]);
    serialize_bool(&sender.send_to_port, &mut raw[24..28]);
    serialize_bool(&sender.send_to_stream, &mut raw[28..32]);

    Ok(())
}

fn deserialize_midi_sender(sender: &mut TcKonnektMidiSender, raw: &[u8]) -> Result<(), String> {
    assert!(raw.len() >= TcKonnektMidiSender::SIZE);

    deserialize_u8(&mut sender.normal.ch, &raw[..4]);
    deserialize_u8(&mut sender.normal.cc, &raw[4..8]);
    deserialize_u8(&mut sender.pushed.ch, &raw[12..16]);
    deserialize_u8(&mut sender.pushed.cc, &raw[16..20]);
    deserialize_bool(&mut sender.send_to_port, &raw[24..28]);
    deserialize_bool(&mut sender.send_to_stream, &raw[28..32]);

    Ok(())
}

/// Loaded program in TC Konnekt series.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TcKonnektLoadedProgram {
    /// Program 1.
    P0,
    /// Program 2.
    P1,
    /// Program 3.
    P2,
}

impl Default for TcKonnektLoadedProgram {
    fn default() -> Self {
        Self::P0
    }
}

const LOADED_PROGRAMS: &[TcKonnektLoadedProgram] = &[
    TcKonnektLoadedProgram::P0,
    TcKonnektLoadedProgram::P1,
    TcKonnektLoadedProgram::P2,
];

const LOADED_PROGRAM_LABEL: &str = "loaded program";

fn serialize_loaded_program(prog: &TcKonnektLoadedProgram, raw: &mut [u8]) -> Result<(), String> {
    serialize_position(LOADED_PROGRAMS, prog, raw, LOADED_PROGRAM_LABEL)
}

fn deserialize_loaded_program(prog: &mut TcKonnektLoadedProgram, raw: &[u8]) -> Result<(), String> {
    deserialize_position(LOADED_PROGRAMS, prog, raw, LOADED_PROGRAM_LABEL)
}

/// AV/C operation defined by TC Electronic. This is not used in TC Konnekt series.
#[derive(Default, Debug)]
pub struct TcAvcCmd {
    pub class_id: u8,
    pub sequence_id: u8,
    pub command_id: u16,
    pub arguments: Vec<u8>,
    op: VendorDependent,
}

// From open source by Weiss engineering for ALSA dice driver.
// class_id: 0 -> common
//   command_id: 6 -> squawk
//   command_id: 7 -> self identify
//   command_id: 8 -> codeload
// class_id: 1 -> general
//   command_id: 1 -> program identify
//   command_id: 2 -> tuner frequency
//   command_id: 3 -> tuner preset
//   command_id: 4 -> tuner scan mode
//   command_id: 5 -> tuner tuner output
//   command_id: 10 -> tuner raw serial

fn tc_avc_cmd_prepare_vendor_dependent_data(cmd: &mut TcAvcCmd) {
    cmd.op.data.resize(4 + cmd.arguments.len(), 0);

    cmd.op.data[0] = cmd.class_id;
    cmd.op.data[1] = 0xff;
    cmd.op.data[2] = (0xff & (cmd.command_id >> 8)) as u8;
    cmd.op.data[3] = (0xff & cmd.command_id) as u8;
    cmd.op.data[4..].copy_from_slice(&cmd.arguments);
}

fn tc_avc_cmd_parse_vendor_dependent_data(cmd: &mut TcAvcCmd) {
    cmd.class_id = cmd.op.data[0];
    cmd.sequence_id = cmd.op.data[1];
    cmd.command_id = ((cmd.op.data[2] as u16) << 8) | (cmd.op.data[3] as u16);
    cmd.arguments = cmd.op.data[4..].to_owned();
}

impl TcAvcCmd {
    pub fn new(company_id: &[u8; 3]) -> Self {
        Self {
            class_id: Default::default(),
            sequence_id: Default::default(),
            command_id: Default::default(),
            arguments: Default::default(),
            op: VendorDependent {
                company_id: company_id.clone(),
                // 4 elements at least.
                data: vec![0; 4],
            },
        }
    }
}

impl AvcOp for TcAvcCmd {
    const OPCODE: u8 = VendorDependent::OPCODE;
}

impl AvcStatus for TcAvcCmd {
    fn build_operands(&mut self, addr: &AvcAddr) -> Result<Vec<u8>, AvcCmdBuildError> {
        tc_avc_cmd_prepare_vendor_dependent_data(self);
        AvcStatus::build_operands(&mut self.op, addr)
    }

    fn parse_operands(&mut self, addr: &AvcAddr, operands: &[u8]) -> Result<(), AvcRespParseError> {
        AvcStatus::parse_operands(&mut self.op, addr, operands)
            .map(|_| tc_avc_cmd_parse_vendor_dependent_data(self))
    }
}

impl AvcControl for TcAvcCmd {
    fn build_operands(&mut self, addr: &AvcAddr) -> Result<Vec<u8>, AvcCmdBuildError> {
        tc_avc_cmd_prepare_vendor_dependent_data(self);
        AvcControl::build_operands(&mut self.op, addr)
    }

    fn parse_operands(&mut self, addr: &AvcAddr, operands: &[u8]) -> Result<(), AvcRespParseError> {
        AvcControl::parse_operands(&mut self.op, addr, operands)
            .map(|_| tc_avc_cmd_parse_vendor_dependent_data(self))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn tc_avc_operation_operands() {
        let company_id = [0xfe, 0xdc, 0xba];
        let operands = [0xfe, 0xdc, 0xba, 0x98, 0x76, 0x54, 0x32, 0x10];
        let mut op = TcAvcCmd::new(&company_id);
        AvcStatus::parse_operands(&mut op, &AvcAddr::Unit, &operands).unwrap();
        assert_eq!(op.op.company_id, company_id);
        assert_eq!(op.class_id, operands[3]);
        assert_eq!(op.sequence_id, operands[4]);
        assert_eq!(
            op.command_id,
            ((operands[5] as u16) << 8) | (operands[6] as u16)
        );
        assert_eq!(op.arguments, operands[7..]);

        let target = AvcStatus::build_operands(&mut op, &AvcAddr::Unit).unwrap();
        assert_eq!(&target[..4], &operands[..4]);
        // The value of sequence_id field is never matched.
        assert_eq!(&target[5..], &operands[5..]);

        let target = AvcControl::build_operands(&mut op, &AvcAddr::Unit).unwrap();
        assert_eq!(&target[..4], &operands[..4]);
        // The value of sequence_id field is never matched.
        assert_eq!(&target[5..], &operands[5..]);

        let mut op = TcAvcCmd::new(&company_id);
        AvcControl::parse_operands(&mut op, &AvcAddr::Unit, &operands).unwrap();
        assert_eq!(op.op.company_id, company_id);
        assert_eq!(op.class_id, operands[3]);
        assert_eq!(op.sequence_id, operands[4]);
        assert_eq!(
            op.command_id,
            ((operands[5] as u16) << 8) | (operands[6] as u16)
        );
        assert_eq!(op.arguments, operands[7..]);
    }
}
