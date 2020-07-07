// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use hinawa::FwReqExtManual;

pub trait CommonProtocol {
    const BASE_OFFSET: u64 = 0xffffe0000000;
    fn read_quadlet(&self, node: &hinawa::FwNode, offset: u64) -> Result<u32, Error>;
    fn write_quadlet(&self, node: &hinawa::FwNode, offset: u64, val: u32) -> Result<(), Error>;
}

impl CommonProtocol for hinawa::FwReq {
    fn read_quadlet(&self, node: &hinawa::FwNode, offset: u64) -> Result<u32, Error> {
        let mut quadlet = [0; 4];
        self.transaction(
            node,
            hinawa::FwTcode::ReadQuadletRequest,
            Self::BASE_OFFSET + offset,
            quadlet.len(),
            &mut quadlet,
        )?;
        Ok(u32::from_be_bytes(quadlet))
    }

    fn write_quadlet(&self, node: &hinawa::FwNode, offset: u64, val: u32) -> Result<(), Error> {
        let mut quadlet = [0; 4];
        quadlet.copy_from_slice(&val.to_be_bytes());
        self.transaction(
            node,
            hinawa::FwTcode::WriteQuadletRequest,
            Self::BASE_OFFSET + offset,
            quadlet.len(),
            &mut quadlet,
        )
    }
}
