// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::{Error, FileError};

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

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum ClkSrc {
    Internal,
    Wordclock,
    Spdif,
    Adat,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum ClkRate {
    R44100,
    R48000,
    R88200,
    R96000,
}

pub trait CommonProtocol: BaseProtocol {
    const CLOCK_STATUS_OFFSET: u64 = 0x0228;
    const ROUTE_FLAG_OFFSET: u64 = 0x022c;
    const INPUT_THRESHOLD_OFFSET: u64 = 0x0230;

    fn get_clk_src(&self, fw_node: &hinawa::FwNode) -> Result<ClkSrc, Error>;
    fn set_clk_src(&self, fw_node: &hinawa::FwNode, src: ClkSrc) -> Result<(), Error>;
    fn get_clk_rate(&self, fw_node: &hinawa::FwNode) -> Result<ClkRate, Error>;
    fn set_clk_rate(&self, fw_node: &hinawa::FwNode, rate: ClkRate) -> Result<(), Error>;
    fn get_routing_flag(
        &self,
        fw_node: &hinawa::FwNode,
        shift: u8,
        mask: u8,
    ) -> Result<usize, Error>;
    fn set_routing_flag(
        &self,
        fw_node: &hinawa::FwNode,
        shift: u8,
        mask: u8,
        val: usize,
    ) -> Result<(), Error>;
    fn get_coax_out_src(&self, fw_node: &hinawa::FwNode) -> Result<usize, Error>;
    fn set_coax_out_src(&self, fw_node: &hinawa::FwNode, index: usize) -> Result<(), Error>;
    fn get_input_threshold(&self, fw_node: &hinawa::FwNode) -> Result<i16, Error>;
    fn set_input_threshold(&self, fw_node: &hinawa::FwNode, th: i16) -> Result<(), Error>;
}

impl CommonProtocol for hinawa::FwReq {
    fn get_clk_src(&self, fw_node: &hinawa::FwNode) -> Result<ClkSrc, Error> {
        let mut frames = [0; 4];
        self.read_transaction(fw_node, Self::CLOCK_STATUS_OFFSET, &mut frames)?;
        let src = match frames[3] {
            0x01 => ClkSrc::Internal,
            0x02 => ClkSrc::Wordclock,
            0x03 => ClkSrc::Spdif,
            0x04 => ClkSrc::Adat,
            _ => {
                let label = format!("Unexpected value for source of clock: {}", frames[1]);
                return Err(Error::new(FileError::Io, &label));
            }
        };
        Ok(src)
    }

    fn set_clk_src(&self, fw_node: &hinawa::FwNode, src: ClkSrc) -> Result<(), Error> {
        let mut frames = [0; 4];
        self.read_transaction(fw_node, Self::CLOCK_STATUS_OFFSET, &mut frames)?;
        frames[0] = 0x00;
        frames[1] = 0x00;
        frames[3] = match src {
            ClkSrc::Internal => 0x01,
            ClkSrc::Wordclock => 0x02,
            ClkSrc::Spdif => 0x03,
            ClkSrc::Adat => 0x04,
        };
        self.write_transaction(fw_node, Self::CLOCK_STATUS_OFFSET, &mut frames)
    }

    fn get_clk_rate(&self, fw_node: &hinawa::FwNode) -> Result<ClkRate, Error> {
        let mut frames = [0; 4];
        self.read_transaction(fw_node, Self::CLOCK_STATUS_OFFSET, &mut frames)?;
        let rate = match frames[1] {
            0x01 => ClkRate::R44100,
            0x02 => ClkRate::R48000,
            0x81 => ClkRate::R88200,
            0x82 => ClkRate::R96000,
            _ => {
                let label = format!("Unexpected value for rate of clock: {}", frames[1]);
                return Err(Error::new(FileError::Io, &label));
            }
        };
        Ok(rate)
    }

    fn set_clk_rate(&self, fw_node: &hinawa::FwNode, rate: ClkRate) -> Result<(), Error> {
        let mut frames = [0; 4];
        self.read_transaction(fw_node, Self::CLOCK_STATUS_OFFSET, &mut frames)?;
        frames[3] = match rate {
            ClkRate::R44100 => 0x01,
            ClkRate::R48000 => 0x02,
            ClkRate::R88200 => 0x81,
            ClkRate::R96000 => 0x82,
        };
        self.write_transaction(fw_node, Self::CLOCK_STATUS_OFFSET, &mut frames)
    }

    fn get_routing_flag(
        &self,
        fw_node: &hinawa::FwNode,
        shift: u8,
        mask: u8,
    ) -> Result<usize, Error> {
        let mut frames = [0; 4];
        self.read_transaction(fw_node, Self::ROUTE_FLAG_OFFSET, &mut frames)?;
        Ok(((frames[3] & mask) >> shift) as usize)
    }

    fn set_routing_flag(
        &self,
        fw_node: &hinawa::FwNode,
        shift: u8,
        mask: u8,
        val: usize,
    ) -> Result<(), Error> {
        let mut frames = [0; 4];
        self.read_transaction(fw_node, Self::ROUTE_FLAG_OFFSET, &mut frames)?;
        frames[0] = 0x00;
        frames[1] = (frames[3] & !((val as u8) << shift)) & (mask << shift);
        frames[2] = (!frames[3] & ((val as u8) << shift)) & (mask << shift);
        self.write_transaction(fw_node, Self::ROUTE_FLAG_OFFSET, &mut frames)
    }

    fn get_coax_out_src(&self, fw_node: &hinawa::FwNode) -> Result<usize, Error> {
        self.get_routing_flag(fw_node, 1, 0x01)
    }

    fn set_coax_out_src(&self, fw_node: &hinawa::FwNode, index: usize) -> Result<(), Error> {
        self.set_routing_flag(fw_node, 1, 0x01, index)
    }

    fn get_input_threshold(&self, fw_node: &hinawa::FwNode) -> Result<i16, Error> {
        let mut frames = [0; 4];
        self.read_transaction(fw_node, Self::INPUT_THRESHOLD_OFFSET, &mut frames)?;
        let val = i16::from_be_bytes([frames[0], frames[1]]);
        Ok(val)
    }

    fn set_input_threshold(&self, fw_node: &hinawa::FwNode, th: i16) -> Result<(), Error> {
        let mut frames = [0; 4];
        frames[0..2].copy_from_slice(&th.to_be_bytes());
        self.write_transaction(fw_node, Self::INPUT_THRESHOLD_OFFSET, &mut frames)
    }
}
