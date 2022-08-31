// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol defined by TASCAM for FireOne.
//!
//! The module includes protocol implementation defined by TASCAM for FireOne.

use super::*;

const TEAC_OUI: [u8; 3] = [0x00, 0x02, 0x2e];

/// Mode of display.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum FireoneDisplayMode {
    Off,
    AlwaysOn,
    Breathe,
    Metronome,
    MidiClockRotate,
    MidiClockFlash,
    JogSlowRotate,
    JogTrack,
}

impl Default for FireoneDisplayMode {
    fn default() -> Self {
        Self::Off
    }
}

/// Mode of MIDI message.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum FireoneMidiMessageMode {
    Native,
    MackieHuiEmulation,
}

impl Default for FireoneMidiMessageMode {
    fn default() -> Self {
        Self::Native
    }
}

/// Mode of input.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum FireoneInputMode {
    Stereo,
    Monaural,
}

impl Default for FireoneInputMode {
    fn default() -> Self {
        Self::Stereo
    }
}

/// The protocol implementation of protocol for Tascam FireOne.
#[derive(Default, Debug)]
pub struct FireoneProtocol;

impl FireoneProtocol {
    pub fn read_display_mode(
        avc: &TascamAvc,
        mode: &mut FireoneDisplayMode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut op = TascamProto::new(VendorCmd::DisplayMode(Default::default()));
        avc.status(&AvcAddr::Unit, &mut op, timeout_ms).map(|_| {
            if let VendorCmd::DisplayMode(val) = &op.cmd {
                *mode = match val {
                    7 => FireoneDisplayMode::JogTrack,
                    6 => FireoneDisplayMode::JogSlowRotate,
                    5 => FireoneDisplayMode::MidiClockFlash,
                    4 => FireoneDisplayMode::MidiClockRotate,
                    3 => FireoneDisplayMode::Metronome,
                    2 => FireoneDisplayMode::Breathe,
                    1 => FireoneDisplayMode::AlwaysOn,
                    _ => FireoneDisplayMode::Off,
                };
            }
        })
    }

    pub fn write_display_mode(
        avc: &TascamAvc,
        mode: FireoneDisplayMode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let val = match mode {
            FireoneDisplayMode::JogTrack => 7,
            FireoneDisplayMode::JogSlowRotate => 6,
            FireoneDisplayMode::MidiClockFlash => 5,
            FireoneDisplayMode::MidiClockRotate => 4,
            FireoneDisplayMode::Metronome => 3,
            FireoneDisplayMode::Breathe => 2,
            FireoneDisplayMode::AlwaysOn => 1,
            FireoneDisplayMode::Off => 0,
        } as u8;
        let mut op = TascamProto::new(VendorCmd::DisplayMode(val));
        avc.control(&AvcAddr::Unit, &mut op, timeout_ms)
    }

    pub fn read_midi_message_mode(
        avc: &TascamAvc,
        mode: &mut FireoneMidiMessageMode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut op = TascamProto::new(VendorCmd::MessageMode(Default::default()));
        avc.status(&AvcAddr::Unit, &mut op, timeout_ms).map(|_| {
            if let VendorCmd::MessageMode(val) = &op.cmd {
                *mode = if *val > 0 {
                    FireoneMidiMessageMode::MackieHuiEmulation
                } else {
                    FireoneMidiMessageMode::Native
                };
            }
        })
    }

    pub fn write_midi_message_mode(
        avc: &TascamAvc,
        mode: FireoneMidiMessageMode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let val = match mode {
            FireoneMidiMessageMode::Native => 0,
            FireoneMidiMessageMode::MackieHuiEmulation => 1,
        };
        let mut op = TascamProto::new(VendorCmd::MessageMode(val));
        avc.control(&AvcAddr::Unit, &mut op, timeout_ms)
    }

    pub fn read_input_mode(
        avc: &TascamAvc,
        mode: &mut FireoneInputMode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut op = TascamProto::new(VendorCmd::MessageMode(Default::default()));
        avc.status(&AvcAddr::Unit, &mut op, timeout_ms).map(|_| {
            if let VendorCmd::MessageMode(val) = &op.cmd {
                *mode = if *val > 0 {
                    FireoneInputMode::Monaural
                } else {
                    FireoneInputMode::Stereo
                };
            }
        })
    }

    pub fn write_input_mode(
        avc: &TascamAvc,
        mode: FireoneInputMode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let val = match mode {
            FireoneInputMode::Stereo => 0,
            FireoneInputMode::Monaural => 1,
        };
        let mut op = TascamProto::new(VendorCmd::InputMode(val));
        avc.control(&AvcAddr::Unit, &mut op, timeout_ms)
    }

    pub fn read_firmware_version(
        avc: &TascamAvc,
        version: &mut u8,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut op = TascamProto::new(VendorCmd::FirmwareVersion(Default::default()));
        avc.status(&AvcAddr::Unit, &mut op, timeout_ms).map(|_| {
            if let VendorCmd::FirmwareVersion(val) = &op.cmd {
                *version = *val as u8;
            }
        })
    }
}

/// Type of command for TASCAM FireOne.
enum VendorCmd {
    DisplayMode(u8),
    MessageMode(u8),
    InputMode(u8),
    FirmwareVersion(u8),
}

impl VendorCmd {
    const TASCAM_PREFIX: [u8; 3] = [0x46, 0x49, 0x31]; // 'F', 'I', '1'

    const DISPLAY_MODE: u8 = 0x10;
    const MESSAGE_MODE: u8 = 0x11;
    const INPUT_MODE: u8 = 0x12;
    const FIRMWARE_VERSION: u8 = 0x13;

    fn build_data(&self) -> Vec<u8> {
        let mut data = Self::TASCAM_PREFIX.to_vec();

        match self {
            VendorCmd::DisplayMode(_) => data.push(Self::DISPLAY_MODE),
            VendorCmd::MessageMode(_) => data.push(Self::MESSAGE_MODE),
            VendorCmd::InputMode(_) => data.push(Self::INPUT_MODE),
            VendorCmd::FirmwareVersion(_) => data.push(Self::FIRMWARE_VERSION),
        }

        data
    }

    fn append_variable(&self, data: &mut Vec<u8>) {
        match self {
            VendorCmd::DisplayMode(val) => data.push(*val),
            VendorCmd::MessageMode(val) => data.push(*val),
            VendorCmd::InputMode(val) => data.push(*val),
            _ => (),
        }
    }

    fn parse_variable(&mut self, data: &[u8]) -> Result<(), AvcRespParseError> {
        if data.len() < 5 {
            Err(AvcRespParseError::TooShortResp(5))?;
        }

        match self {
            VendorCmd::DisplayMode(val) => {
                if data[3] != Self::DISPLAY_MODE {
                    Err(AvcRespParseError::UnexpectedOperands(3))
                } else {
                    *val = data[4];
                    Ok(())
                }
            }
            VendorCmd::MessageMode(val) => {
                if data[3] != Self::MESSAGE_MODE {
                    Err(AvcRespParseError::UnexpectedOperands(3))
                } else {
                    *val = data[4];
                    Ok(())
                }
            }
            VendorCmd::InputMode(val) => {
                if data[3] != Self::INPUT_MODE {
                    Err(AvcRespParseError::UnexpectedOperands(3))
                } else {
                    *val = data[4];
                    Ok(())
                }
            }
            VendorCmd::FirmwareVersion(val) => {
                if data[3] != Self::FIRMWARE_VERSION {
                    Err(AvcRespParseError::UnexpectedOperands(3))
                } else {
                    *val = data[4];
                    Ok(())
                }
            }
        }
    }
}

impl From<&VendorCmd> for u8 {
    fn from(cmd: &VendorCmd) -> u8 {
        match cmd {
            VendorCmd::DisplayMode(_) => VendorCmd::DISPLAY_MODE,
            VendorCmd::MessageMode(_) => VendorCmd::MESSAGE_MODE,
            VendorCmd::InputMode(_) => VendorCmd::INPUT_MODE,
            VendorCmd::FirmwareVersion(_) => VendorCmd::FIRMWARE_VERSION,
        }
    }
}

/// AV/C vendor-dependent command specialized by TASCAM.
struct TascamProto {
    cmd: VendorCmd,
    op: VendorDependent,
}

impl TascamProto {
    fn new(cmd: VendorCmd) -> Self {
        TascamProto {
            cmd,
            op: VendorDependent::new(&TEAC_OUI),
        }
    }
}

impl AvcOp for TascamProto {
    const OPCODE: u8 = VendorDependent::OPCODE;
}

impl AvcControl for TascamProto {
    fn build_operands(&mut self, addr: &AvcAddr) -> Result<Vec<u8>, AvcCmdBuildError> {
        let mut data = self.cmd.build_data();
        self.cmd.append_variable(&mut data);
        self.op.data = data;
        AvcControl::build_operands(&mut self.op, addr)
    }

    fn parse_operands(&mut self, addr: &AvcAddr, operands: &[u8]) -> Result<(), AvcRespParseError> {
        AvcControl::parse_operands(&mut self.op, addr, operands)
    }
}

impl AvcStatus for TascamProto {
    fn build_operands(&mut self, addr: &AvcAddr) -> Result<Vec<u8>, AvcCmdBuildError> {
        self.op.data = self.cmd.build_data();
        AvcStatus::build_operands(&mut self.op, addr)
    }

    fn parse_operands(&mut self, addr: &AvcAddr, operands: &[u8]) -> Result<(), AvcRespParseError> {
        AvcStatus::parse_operands(&mut self.op, addr, operands)?;
        self.cmd.parse_variable(&self.op.data)
    }
}

/// The implementation of AV/C transaction with quirk specific to Tascam FireOne.
///
/// It seems a unique quirk that the status code in response frame for AV/C vendor-dependent
/// command is against AV/C general specification in control operation.
#[derive(Default, Debug)]
pub struct TascamAvc(OxfwAvc);

impl Ta1394Avc<Error> for TascamAvc {
    fn transaction(&self, command_frame: &[u8], timeout_ms: u32) -> Result<Vec<u8>, Error> {
        self.0.transaction(command_frame, timeout_ms)
    }

    fn control<O: AvcOp + AvcControl>(
        &self,
        addr: &AvcAddr,
        op: &mut O,
        timeout_ms: u32,
    ) -> Result<(), Ta1394AvcError<Error>> {
        let operands =
            AvcControl::build_operands(op, addr).map_err(|err| Ta1394AvcError::CmdBuild(err))?;
        let command_frame =
            Self::compose_command_frame(AvcCmdType::Control, addr, O::OPCODE, &operands);
        let response_frame = self
            .transaction(&command_frame, timeout_ms)
            .map_err(|cause| Ta1394AvcError::CommunicationFailure(cause))?;
        Self::detect_response_operands(&response_frame, addr, O::OPCODE)
            .and_then(|(rcode, operands)| {
                let expected = if O::OPCODE != VendorDependent::OPCODE {
                    AvcRespCode::Accepted
                } else {
                    // NOTE: quirk. Furthermore, company_id in response transaction is 0xffffff.
                    AvcRespCode::ImplementedStable
                };
                if rcode != expected {
                    Err(AvcRespParseError::UnexpectedStatus)
                } else {
                    AvcControl::parse_operands(op, addr, &operands)
                }
            })
            .map_err(|err| Ta1394AvcError::RespParse(err))
    }
}

impl TascamAvc {
    pub fn bind(&self, node: &impl IsA<FwNode>) -> Result<(), Error> {
        self.0.bind(node)
    }

    pub fn control<O: AvcOp + AvcControl>(
        &self,
        addr: &AvcAddr,
        op: &mut O,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        Ta1394Avc::<Error>::control(self, addr, op, timeout_ms).map_err(|err| from_avc_err(err))
    }

    pub fn status<O: AvcOp + AvcStatus>(
        &self,
        addr: &AvcAddr,
        op: &mut O,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        Ta1394Avc::<Error>::status(self, addr, op, timeout_ms).map_err(|err| from_avc_err(err))
    }
}

#[cfg(test)]
mod test {
    use super::{TascamProto, VendorCmd};
    use ta1394_avc_general::*;

    #[test]
    fn tascam_proto_operands() {
        let mut op = TascamProto::new(VendorCmd::DisplayMode(Default::default()));
        let operands = [0x00, 0x02, 0x2e, 0x46, 0x49, 0x31, 0x10, 0x01];
        AvcStatus::parse_operands(&mut op, &AvcAddr::Unit, &operands).unwrap();
        if let VendorCmd::DisplayMode(val) = &op.cmd {
            assert_eq!(*val, 0x01)
        } else {
            unreachable!();
        }

        let o = AvcStatus::build_operands(&mut op, &AvcAddr::Unit).unwrap();
        assert_eq!(o, operands[..7]);

        let mut op = TascamProto::new(VendorCmd::InputMode(0x01));
        let operands = [0x00, 0x02, 0x2e, 0x46, 0x49, 0x31, 0x12, 0x01];
        AvcControl::parse_operands(&mut op, &AvcAddr::Unit, &operands).unwrap();

        let o = AvcControl::build_operands(&mut op, &AvcAddr::Unit).unwrap();
        assert_eq!(o, operands);
    }
}
