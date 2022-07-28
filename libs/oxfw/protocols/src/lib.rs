// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocols defined for Oxford Semiconductor FW970/971 chipset.
//!
//! The crate includes various kind of protocols defined by Oxford Semiconductor as well as
//! hardware vendors for FW970/971 ASICs.

pub mod apogee;
pub mod griffin;
pub mod lacie;
pub mod loud;
pub mod oxford;
pub mod tascam;

use {
    glib::{Error, FileError, IsA},
    hinawa::{
        prelude::{FwFcpExtManual, FwFcpExt, FwReqExtManual},
        FwFcp, FwNode, FwReq, FwTcode,
    },
    ta1394::{audio::*, ccm::*, general::*, *},
};

/// The structure to implement AV/C transaction.
#[derive(Default, Debug)]
pub struct OxfwAvc(FwFcp);

impl Ta1394Avc for OxfwAvc {
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
}
