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
    glib::{Error, FileError, IsA},
    hinawa::{
        prelude::{FwFcpExt, FwFcpExtManual},
        FwFcp, FwNode,
    },
    ta1394_avc_general::*,
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
