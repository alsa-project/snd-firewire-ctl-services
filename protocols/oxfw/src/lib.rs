// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

#![doc = include_str!("../README.md")]

pub mod apogee;
pub mod griffin;
pub mod lacie;
pub mod loud;
pub mod oxford;
pub mod tascam;

use {
    glib::{Error, FileError, IsA},
    hinawa::{
        prelude::{FwFcpExt, FwFcpExtManual, FwReqExtManual},
        FwFcp, FwNode, FwReq, FwTcode,
    },
    ta1394_avc_audio::*,
    ta1394_avc_ccm::*,
    ta1394_avc_general::{general::*, *},
};

/// The implementation of AV/C transaction.
#[derive(Default, Debug)]
pub struct OxfwAvc(FwFcp);

impl Ta1394Avc<Error> for OxfwAvc {
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

impl OxfwAvc {
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

fn from_avc_err(err: Ta1394AvcError<Error>) -> Error {
    match err {
        Ta1394AvcError::CmdBuild(cause) => Error::new(FileError::Inval, &cause.to_string()),
        Ta1394AvcError::CommunicationFailure(cause) => cause,
        Ta1394AvcError::RespParse(cause) => Error::new(FileError::Io, &cause.to_string()),
    }
}

/// Operation for read-only parameters by AV/C command in FCP.
pub trait OxfwFcpParamsOperation<P, T>
where
    P: Ta1394Avc<Error>,
{
    /// Cache state of hardware for the parameter.
    fn cache(avc: &mut P, params: &mut T, timeout_ms: u32) -> Result<(), Error>;
}

/// Operation for mutable parameters by AV/C command in FCP.
pub trait OxfwFcpMutableParamsOperation<P, T>
where
    P: Ta1394Avc<Error>,
{
    /// Update state of hardware when detecting any change between the given parameters.
    fn update(avc: &mut P, params: &T, prev: &mut T, timeout_ms: u32) -> Result<(), Error>;
}
