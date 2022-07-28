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
    glib::{Error, FileError},
    hinawa::{
        prelude::{FwFcpExtManual, FwReqExtManual},
        FwFcp, FwNode, FwReq, FwTcode,
    },
    ta1394::{audio::*, ccm::*, general::*, *},
};

/// The structure to implement AV/C transaction.
#[derive(Default, Debug)]
pub struct OxfwAvc(FwFcp);

impl AsRef<FwFcp> for OxfwAvc {
    fn as_ref(&self) -> &FwFcp {
        &self.0
    }
}

impl Ta1394Avc for OxfwAvc {
    fn transaction(
        &self,
        ctype: AvcCmdType,
        addr: &AvcAddr,
        opcode: u8,
        operands: &[u8],
        timeout_ms: u32,
    ) -> Result<(AvcRespCode, Vec<u8>), Error> {
        let mut cmd = Vec::new();
        cmd.push(ctype.into());
        cmd.push(addr.into());
        cmd.push(opcode);
        cmd.extend_from_slice(operands);

        let mut resp = vec![0; Self::FRAME_SIZE];
        self.0
            .avc_transaction(&cmd, &mut resp, timeout_ms)
            .and_then(|len| {
                if resp[1] != addr.into() {
                    let label = format!("Unexpected address in response: {}", resp[1]);
                    Err(Error::new(Ta1394AvcError::UnexpectedRespCode, &label))
                } else if resp[2] != opcode {
                    let label =
                        format!("Unexpected opcode in response: {} but {}", opcode, resp[2]);
                    Err(Error::new(Ta1394AvcError::UnexpectedRespCode, &label))
                } else {
                    let rcode = AvcRespCode::from(resp[0] & Self::RESP_CODE_MASK);

                    resp.truncate(len);
                    let operands = resp.split_off(3);

                    Ok((rcode, operands))
                }
            })
    }
}
