// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2024 Takashi Sakamoto

//! Protocol specific to Weiss Engineering AV/C models.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by Weiss Engineering.
//!
//! MAN301 includes two units in the root directory of its configuration ROM. The first unit
//! expresses AV/C protocol, and the second unit expresses TCAT protocol.

use {
    super::*,
    crate::{tcelectronic::*, *},
    glib::{Error, FileError, IsA},
    hinawa::{
        prelude::{FwFcpExt, FwFcpExtManual},
        FwFcp, FwNode,
    },
    ta1394_avc_general::{general::*, *},
};

/// Protocol implementation specific to MAN301.
#[derive(Default, Debug)]
pub struct WeissMan301Protocol;

impl TcatOperation for WeissMan301Protocol {}

// clock caps: 44100 48000 88200 96000 176400 192000
// clock source names: AES/EBU (XLR)\S/PDIF (RCA)\S/PDIF (TOS)\Unused\Unused\Unused\Unused\Word Clock (BNC)\Unused\Unused\Unused\Unused\Internal\\
impl TcatGlobalSectionSpecification for WeissMan301Protocol {}

/// The implementation of AV/C transaction.
#[derive(Default, Debug)]
pub struct WeissAvc(FwFcp);

impl Ta1394Avc<Error> for WeissAvc {
    fn transaction(&self, command_frame: &[u8], timeout_ms: u32) -> Result<Vec<u8>, Error> {
        let mut resp = vec![0; Self::FRAME_SIZE];
        self.0
            .avc_transaction(&command_frame, &mut resp, timeout_ms)
            .map(|len| {
                resp.truncate(len);
                resp
            })
    }
}

fn from_avc_err(err: Ta1394AvcError<Error>) -> Error {
    match err {
        Ta1394AvcError::CmdBuild(cause) => Error::new(FileError::Inval, &cause.to_string()),
        Ta1394AvcError::CommunicationFailure(cause) => cause,
        Ta1394AvcError::RespParse(cause) => Error::new(FileError::Io, &cause.to_string()),
    }
}

impl WeissAvc {
    /// Bind FCP protocol to the given node for AV/C operation.
    pub fn bind(&self, node: &impl IsA<FwNode>) -> Result<(), Error> {
        self.0.bind(node)
    }

    /// Request AV/C control operation and wait for response.
    pub fn control<O: AvcOp + AvcControl>(
        &self,
        addr: &AvcAddr,
        op: &mut O,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        Ta1394Avc::<Error>::control(self, addr, op, timeout_ms).map_err(|err| from_avc_err(err))
    }

    /// Request AV/C status operation and wait for response.
    pub fn status<O: AvcOp + AvcStatus>(
        &self,
        addr: &AvcAddr,
        op: &mut O,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        Ta1394Avc::<Error>::status(self, addr, op, timeout_ms).map_err(|err| from_avc_err(err))
    }
}

/// The content of command to operate parameter.
#[derive(Debug)]
pub struct WeissAvcParamCmd {
    /// The numeric identifier of parameter.
    pub numeric_id: u32,
    /// The value of parameter.
    pub value: u32,
    /// For future use.
    pub reserved: [u32; 4],
    op: TcAvcCmd,
}

impl Default for WeissAvcParamCmd {
    fn default() -> Self {
        let mut op = TcAvcCmd::new(&WEISS_OUI);
        op.class_id = 1;
        op.sequence_id = u8::MAX;
        op.command_id = 0x8002;
        Self {
            numeric_id: u32::MAX,
            value: u32::MAX,
            reserved: [u32::MAX; 4],
            op: TcAvcCmd::new(&WEISS_OUI),
        }
    }
}

impl AvcOp for WeissAvcParamCmd {
    const OPCODE: u8 = VendorDependent::OPCODE;
}

fn build_param_command_status_data(cmd: &mut WeissAvcParamCmd) -> Result<(), AvcCmdBuildError> {
    cmd.op.arguments.resize(24, u8::MAX);
    serialize_u32(&cmd.numeric_id, &mut cmd.op.arguments[..4]);
    (0..4).for_each(|i| {
        let pos = 8 + i * 4;
        serialize_u32(&cmd.reserved[i], &mut cmd.op.arguments[pos..(pos + 4)]);
    });
    Ok(())
}

fn build_param_command_control_data(cmd: &mut WeissAvcParamCmd) -> Result<(), AvcCmdBuildError> {
    cmd.op.arguments.resize(24, u8::MIN);
    serialize_u32(&cmd.numeric_id, &mut cmd.op.arguments[..4]);
    serialize_u32(&cmd.value, &mut cmd.op.arguments[4..8]);
    (0..4).for_each(|i| {
        let pos = 8 + i * 4;
        serialize_u32(&cmd.reserved[i], &mut cmd.op.arguments[pos..(pos + 4)]);
    });
    Ok(())
}

fn parse_param_command_response_data(cmd: &mut WeissAvcParamCmd) -> Result<(), AvcRespParseError> {
    if cmd.op.arguments.len() < 24 {
        Err(AvcRespParseError::TooShortResp(24))?
    }

    deserialize_u32(&mut cmd.numeric_id, &cmd.op.arguments[..4]);
    deserialize_u32(&mut cmd.value, &cmd.op.arguments[4..8]);
    (0..4).for_each(|i| {
        let pos = 8 + i * 4;
        deserialize_u32(&mut cmd.reserved[i], &cmd.op.arguments[pos..(pos + 4)]);
    });

    Ok(())
}

impl AvcStatus for WeissAvcParamCmd {
    fn build_operands(&mut self, addr: &AvcAddr) -> Result<Vec<u8>, AvcCmdBuildError> {
        build_param_command_status_data(self)?;
        AvcStatus::build_operands(&mut self.op, addr)
    }

    fn parse_operands(&mut self, addr: &AvcAddr, operands: &[u8]) -> Result<(), AvcRespParseError> {
        AvcStatus::parse_operands(&mut self.op, addr, operands)?;
        parse_param_command_response_data(self)
    }
}

impl AvcControl for WeissAvcParamCmd {
    fn build_operands(&mut self, addr: &AvcAddr) -> Result<Vec<u8>, AvcCmdBuildError> {
        build_param_command_control_data(self)?;
        AvcControl::build_operands(&mut self.op, addr)
    }

    fn parse_operands(&mut self, addr: &AvcAddr, operands: &[u8]) -> Result<(), AvcRespParseError> {
        AvcControl::parse_operands(&mut self.op, addr, operands)?;
        parse_param_command_response_data(self)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn weiss_avc_param_command_operands() {
        let operands = [
            0x00, 0x1c, 0x6a, 0x01, 0x7f, 0x80, 0x02, 0x76, 0x54, 0x32, 0x10, 0xfe, 0xdc, 0xba,
            0x98, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff,
        ];
        let mut op = WeissAvcParamCmd::default();
        AvcStatus::parse_operands(&mut op, &AvcAddr::Unit, &operands).unwrap();
        assert_eq!(op.op.class_id, 0x01);
        assert_eq!(op.op.sequence_id, 0x7f);
        assert_eq!(op.op.command_id, 0x8002);
        assert_eq!(op.op.arguments, &operands[7..]);
        assert_eq!(op.numeric_id, 0x76543210);
        assert_eq!(op.value, 0xfedcba98);

        let target = AvcStatus::build_operands(&mut op, &AvcAddr::Unit).unwrap();
        assert_eq!(&target[..4], &operands[..4]);
        // The value of sequence_id field is never matched.
        assert_eq!(&target[5..], &operands[5..]);

        let target = AvcControl::build_operands(&mut op, &AvcAddr::Unit).unwrap();
        assert_eq!(&target[..4], &operands[..4]);
        // The value of sequence_id field is never matched.
        assert_eq!(&target[5..], &operands[5..]);

        let mut op = WeissAvcParamCmd::default();
        AvcControl::parse_operands(&mut op, &AvcAddr::Unit, &operands).unwrap();
        assert_eq!(op.op.class_id, 0x01);
        assert_eq!(op.op.sequence_id, 0x7f);
        assert_eq!(op.op.command_id, 0x8002);
        assert_eq!(op.op.arguments, &operands[7..]);
        assert_eq!(op.numeric_id, 0x76543210);
        assert_eq!(op.value, 0xfedcba98);
    }
}
