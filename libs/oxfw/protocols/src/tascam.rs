// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol defined by TASCAM for FireOne.
//!
//! The module includes protocol implementation defined by TASCAM for FireOne.

use glib::Error;

use hinawa::FwFcp;

use ta1394::general::VendorDependent;
use ta1394::{AvcAddr, AvcCmdType, AvcRespCode, Ta1394Avc, Ta1394AvcError};
use ta1394::{AvcControl, AvcOp, AvcStatus};

const TEAC_OUI: [u8; 3] = [0x00, 0x02, 0x2e];

/// The enumeration for mode of display.
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

/// The enumeration for mode of MIDI message.
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

/// The enumeration for mode of input.
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
        let mut op = TascamProto::new(VendorCmd::DisplayMode);
        avc.status(&AvcAddr::Unit, &mut op, timeout_ms).map(|_| {
            *mode = match op.val {
                7 => FireoneDisplayMode::JogTrack,
                6 => FireoneDisplayMode::JogSlowRotate,
                5 => FireoneDisplayMode::MidiClockFlash,
                4 => FireoneDisplayMode::MidiClockRotate,
                3 => FireoneDisplayMode::Metronome,
                2 => FireoneDisplayMode::Breathe,
                1 => FireoneDisplayMode::AlwaysOn,
                _ => FireoneDisplayMode::Off,
            };
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
        let mut op = TascamProto::new(VendorCmd::DisplayMode);
        op.val = val;
        avc.control(&AvcAddr::Unit, &mut op, timeout_ms)
    }

    pub fn read_midi_message_mode(
        avc: &TascamAvc,
        mode: &mut FireoneMidiMessageMode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut op = TascamProto::new(VendorCmd::MessageMode);
        avc.status(&AvcAddr::Unit, &mut op, timeout_ms).map(|_| {
            *mode = if op.val > 0 {
                FireoneMidiMessageMode::MackieHuiEmulation
            } else {
                FireoneMidiMessageMode::Native
            };
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
        let mut op = TascamProto::new(VendorCmd::MessageMode);
        op.val = val as u8;
        avc.control(&AvcAddr::Unit, &mut op, timeout_ms)
    }

    pub fn read_input_mode(
        avc: &TascamAvc,
        mode: &mut FireoneInputMode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut op = TascamProto::new(VendorCmd::MessageMode);
        avc.status(&AvcAddr::Unit, &mut op, timeout_ms).map(|_| {
            *mode = if op.val > 0 {
                FireoneInputMode::Monaural
            } else {
                FireoneInputMode::Stereo
            };
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
        let mut op = TascamProto::new(VendorCmd::InputMode);
        op.val = val as u8;
        avc.control(&AvcAddr::Unit, &mut op, timeout_ms)
    }

    pub fn read_firmware_version(
        avc: &TascamAvc,
        version: &mut u8,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut op = TascamProto::new(VendorCmd::FirmwareVersion);
        avc.status(&AvcAddr::Unit, &mut op, timeout_ms)
            .map(|_| *version = op.val as u8)
    }
}

/// The enumeration to represent type of command for TASCAM FireOne.
pub enum VendorCmd {
    DisplayMode,
    MessageMode,
    InputMode,
    FirmwareVersion,
}

impl VendorCmd {
    const DISPLAY_MODE: u8 = 0x10;
    const MESSAGE_MODE: u8 = 0x11;
    const INPUT_MODE: u8 = 0x12;
    const FIRMWARE_VERSION: u8 = 0x13;
}

impl From<&VendorCmd> for u8 {
    fn from(cmd: &VendorCmd) -> u8 {
        match cmd {
            VendorCmd::DisplayMode => VendorCmd::DISPLAY_MODE,
            VendorCmd::MessageMode => VendorCmd::MESSAGE_MODE,
            VendorCmd::InputMode => VendorCmd::INPUT_MODE,
            VendorCmd::FirmwareVersion => VendorCmd::FIRMWARE_VERSION,
        }
    }
}

/// The structure to represent protocol of TASCAM FireOne.
pub struct TascamProto {
    cmd: VendorCmd,
    pub val: u8,
    op: VendorDependent,
}

impl TascamProto {
    const TASCAM_PREFIX: [u8; 3] = [0x46, 0x49, 0x31]; // 'F', 'I', '1'

    pub fn new(cmd: VendorCmd) -> Self {
        TascamProto{
            cmd,
            val: 0xff,
            op: VendorDependent::new(&TEAC_OUI),
        }
    }

    fn build_op(&mut self) -> Result<(), Error> {
        self.op.data.clear();
        self.op.data.extend_from_slice(&Self::TASCAM_PREFIX);
        self.op.data.push(u8::from(&self.cmd));
        self.op.data.push(self.val);
        Ok(())
    }

    fn parse_op(&mut self) -> Result<(), Error> {
        if self.op.data.len() < 5 {
            let label = format!("Data too short for TascamProtocol; {}", self.op.data.len());
            return Err(Error::new(Ta1394AvcError::TooShortResp, &label));
        }

        if self.op.data[3] != u8::from(&self.cmd) {
            let label = format!("Invalid command for TascamProto; {:?}", self.op.data[3]);
            return Err(Error::new(Ta1394AvcError::UnexpectedRespOperands, &label));
        }

        self.val = self.op.data[4];

        Ok(())
    }
}

impl AvcOp for TascamProto {
    const OPCODE: u8 = VendorDependent::OPCODE;
}

impl AvcControl for TascamProto {
    fn build_operands(&mut self, addr: &AvcAddr, operands: &mut Vec<u8>) -> Result<(), Error> {
        Self::build_op(self)?;
        AvcControl::build_operands(&mut self.op, addr, operands)
    }

    fn parse_operands(&mut self, addr: &AvcAddr, operands: &[u8]) -> Result<(), Error> {
        AvcControl::parse_operands(&mut self.op, addr, operands)?;
        Self::parse_op(self)
    }
}

impl AvcStatus for TascamProto {
    fn build_operands(&mut self, addr: &AvcAddr, operands: &mut Vec<u8>) -> Result<(), Error> {
        Self::build_op(self)?;
        AvcStatus::build_operands(&mut self.op, addr, operands)
    }

    fn parse_operands(&mut self, addr: &AvcAddr, operands: &[u8]) -> Result<(), Error> {
        AvcStatus::parse_operands(&mut self.op, addr, operands)?;
        Self::parse_op(self)
    }
}

/// The structure to represent AV/C protocol for TASCAM FireOne.
#[derive(Default, Debug)]
pub struct TascamAvc(pub FwFcp);

impl AsRef<FwFcp> for TascamAvc {
    fn as_ref(&self) -> &FwFcp {
        &self.0
    }
}

impl Ta1394Avc for TascamAvc {
    fn control<O: AvcOp + AvcControl>(
        &self,
        addr: &AvcAddr,
        op: &mut O,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut operands = Vec::new();
        AvcControl::build_operands(op, addr, &mut operands)?;
        let (rcode, operands) =
            self.trx(AvcCmdType::Control, addr, O::OPCODE, &operands, timeout_ms)?;
        let expected = if O::OPCODE != VendorDependent::OPCODE {
            AvcRespCode::Accepted
        } else {
            // NOTE: quirk. Furthermore, company_id in response transaction is 0xffffff.
            AvcRespCode::ImplementedStable
        };
        if rcode != expected {
            let label = format!(
                "Unexpected response code for TascamAvc control: {:?}",
                rcode
            );
            return Err(Error::new(Ta1394AvcError::UnexpectedRespCode, &label));
        }
        AvcControl::parse_operands(op, addr, &operands)
    }
}

#[cfg(test)]
mod test {
    use super::{TascamProto, VendorCmd};
    use ta1394::{AvcAddr, AvcControl, AvcStatus};

    #[test]
    fn tascam_proto_operands() {
        let mut op = TascamProto::new(VendorCmd::DisplayMode);
        let operands = [0x00, 0x02, 0x2e, 0x46, 0x49, 0x31, 0x10, 0x01];
        AvcStatus::parse_operands(&mut op, &AvcAddr::Unit, &operands).unwrap();
        assert_eq!(op.val, 0x01);

        let mut o = Vec::new();
        AvcStatus::build_operands(&mut op, &AvcAddr::Unit, &mut o).unwrap();
        assert_eq!(o, operands);

        let mut op = TascamProto::new(VendorCmd::InputMode);
        let operands = [0x00, 0x02, 0x2e, 0x46, 0x49, 0x31, 0x12, 0x1c];
        AvcControl::parse_operands(&mut op, &AvcAddr::Unit, &operands).unwrap();
        assert_eq!(op.val, 0x1c);

        let mut o = Vec::new();
        AvcControl::build_operands(&mut op, &AvcAddr::Unit, &mut o).unwrap();
        assert_eq!(o, operands);
    }
}
