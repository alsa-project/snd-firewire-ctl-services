// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use hinawa::FwReqExtManual;

pub trait BaseProtocol {
    const BASE_ADDR: u64 = 0xffff00000000;
    const LED_OFFSET: u64 = 0x0404;

    fn read_transaction(
        &self,
        fw_node: &hinawa::FwNode,
        offset: u64,
        frames: &mut [u8],
    ) -> Result<(), Error>;
    fn write_transaction(
        &self,
        fw_node: &hinawa::FwNode,
        offset: u64,
        frames: &mut [u8],
    ) -> Result<(), Error>;
    fn bright_led(&self, fw_node: &hinawa::FwNode, pos: u16, state: bool) -> Result<(), Error>;
}

impl BaseProtocol for hinawa::FwReq {
    fn read_transaction(
        &self,
        fw_node: &hinawa::FwNode,
        offset: u64,
        frames: &mut [u8],
    ) -> Result<(), Error> {
        self.transaction(
            fw_node,
            hinawa::FwTcode::ReadQuadletRequest,
            Self::BASE_ADDR + offset,
            4,
            frames,
        )
    }

    fn write_transaction(
        &self,
        fw_node: &hinawa::FwNode,
        offset: u64,
        frames: &mut [u8],
    ) -> Result<(), Error> {
        self.transaction(
            fw_node,
            hinawa::FwTcode::WriteQuadletRequest,
            Self::BASE_ADDR + offset,
            4,
            frames,
        )
    }

    fn bright_led(&self, fw_node: &hinawa::FwNode, pos: u16, state: bool) -> Result<(), Error> {
        let mut frame = [0; 4];
        frame[0..2].copy_from_slice(&(state as u16).to_be_bytes());
        frame[2..4].copy_from_slice(&pos.to_be_bytes());

        self.write_transaction(fw_node, Self::LED_OFFSET, &mut frame)
    }
}
