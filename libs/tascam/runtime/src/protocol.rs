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

pub trait RackProtocol: CommonProtocol {
    const INPUT_OFFSET: u64 = 0x0408;

    fn init_states(&self, node: &hinawa::FwNode, cache: &mut [u8; 72]) -> Result<(), Error>;

    fn write_input_quadlet(
        &self,
        fw_node: &hinawa::FwNode,
        cache: &mut [u8],
        pos: usize,
    ) -> Result<(), Error>;
    fn get_gain(&self, cache: &[u8; 72], index: usize) -> Result<i16, Error>;
    fn set_gain(
        &self,
        fw_node: &hinawa::FwNode,
        cache: &mut [u8; 72],
        index: usize,
        gain: i16,
    ) -> Result<(), Error>;
    fn get_balance(&self, cache: &[u8; 72], index: usize) -> Result<u8, Error>;
    fn set_balance(
        &self,
        fw_node: &hinawa::FwNode,
        cache: &mut [u8; 72],
        index: usize,
        pan: u8,
    ) -> Result<(), Error>;
    fn get_mute(&self, cache: &[u8; 72], index: usize) -> Result<bool, Error>;
    fn set_mute(
        &self,
        fw_node: &hinawa::FwNode,
        cache: &mut [u8; 72],
        index: usize,
        mute: bool,
    ) -> Result<(), Error>;
}

impl RackProtocol for hinawa::FwReq {
    fn init_states(&self, node: &hinawa::FwNode, cache: &mut [u8; 72]) -> Result<(), Error> {
        let val: i16 = 0x7fff;

        (0..18).try_for_each(|i| {
            let pos = i * std::mem::size_of::<u32>();
            cache[pos] = i as u8;
            if i % 2 > 0 {
                cache[pos + 1] = 0xff;
            }
            cache[(pos + 2)..(pos + 4)].copy_from_slice(&val.to_le_bytes());

            self.write_input_quadlet(&node, cache, pos)?;

            Ok(())
        })?;

        Ok(())
    }

    fn write_input_quadlet(
        &self,
        fw_node: &hinawa::FwNode,
        cache: &mut [u8],
        pos: usize,
    ) -> Result<(), Error> {
        self.write_transaction(fw_node, Self::INPUT_OFFSET, &mut cache[pos..(pos + 4)])
    }

    fn get_gain(&self, cache: &[u8; 72], index: usize) -> Result<i16, Error> {
        let pos = index * std::mem::size_of::<u32>();
        Ok(i16::from_be_bytes([cache[pos + 2], cache[pos + 3]]))
    }

    fn set_gain(
        &self,
        fw_node: &hinawa::FwNode,
        cache: &mut [u8; 72],
        index: usize,
        gain: i16,
    ) -> Result<(), Error> {
        let pos = index * std::mem::size_of::<u32>();
        cache[(pos + 2)..(pos + 4)].copy_from_slice(&gain.to_be_bytes());
        self.write_input_quadlet(fw_node, cache, pos)
    }

    fn get_balance(&self, cache: &[u8; 72], index: usize) -> Result<u8, Error> {
        let pos = index * std::mem::size_of::<u32>();
        Ok(cache[pos + 1])
    }

    fn set_balance(
        &self,
        fw_node: &hinawa::FwNode,
        cache: &mut [u8; 72],
        index: usize,
        pan: u8,
    ) -> Result<(), Error> {
        let pos = index * std::mem::size_of::<u32>();
        cache[pos + 1] = pan;
        self.write_input_quadlet(fw_node, cache, pos)
    }

    fn get_mute(&self, cache: &[u8; 72], index: usize) -> Result<bool, Error> {
        let pos = index * std::mem::size_of::<u32>();
        Ok(cache[pos] & 0x80 > 0)
    }

    fn set_mute(
        &self,
        fw_node: &hinawa::FwNode,
        cache: &mut [u8; 72],
        index: usize,
        mute: bool,
    ) -> Result<(), Error> {
        let pos = index * std::mem::size_of::<u32>();
        cache[pos] &= !0x80;
        if mute {
            cache[pos] |= 0x80;
        }
        self.write_input_quadlet(fw_node, cache, pos)
    }
}

pub trait ExpanderProtocol: BaseProtocol {
    const ENABLE_NOTIFICATION: u64 = 0x0310;
    const ADDR_HIGH_OFFSET: u64 = 0x0314;
    const ADDR_LOW_OFFSET: u64 = 0x0318;

    fn register_notification_addr(&self, node: &hinawa::FwNode, addr: u64) -> Result<(), Error>;
    fn enable_notification(&self, node: &hinawa::FwNode, state: bool) -> Result<(), Error>;
}

impl ExpanderProtocol for hinawa::FwReq {
    fn register_notification_addr(&self, node: &hinawa::FwNode, addr: u64) -> Result<(), Error> {
        let mut addr_hi = ((addr >> 32) as u32).to_be_bytes();
        self.write_transaction(node, Self::ADDR_HIGH_OFFSET, &mut addr_hi)?;

        let mut addr_lo = ((addr & 0xffffffff) as u32).to_be_bytes();
        self.write_transaction(node, Self::ADDR_LOW_OFFSET, &mut addr_lo)
    }

    fn enable_notification(&self, node: &hinawa::FwNode, enable: bool) -> Result<(), Error> {
        let mut frames = (enable as u32).to_be_bytes();
        self.write_transaction(node, Self::ENABLE_NOTIFICATION, &mut frames)
    }
}

pub trait GetPosition<H> {
    fn get_position(&self, handle: H) -> Result<(), Error>;
}

impl <H> GetPosition<H> for &[u16]
    where H: FnMut(u16) -> Result<(), Error>
{

    fn get_position(&self, mut handle: H) -> Result<(), Error> {
        match self.iter().nth(0) {
            Some(&pos) => handle(pos),
            None => {
                let label = "Program mistake for table of LED position.";
                Err(Error::new(FileError::Nxio, &label))
            }
        }
    }
}

pub trait DetectPosition<H> {
    fn detect_position(&self, index: usize, handle: H) -> Result<(), Error>;
}

impl<H> DetectPosition<H> for &[&[u16]]
    where H: FnMut(u16) -> Result<(), Error> {

    fn detect_position(&self, index: usize, handle: H) -> Result<(), Error> {
        match self.iter().nth(index) {
            Some(entries) => entries.get_position(handle),
            None => {
                let label = "Invalid argument for index to table of LED position.";
                Err(Error::new(FileError::Inval, &label))
            }
        }
    }
}

pub trait DetectAction<H> {
    fn detect_action(&self, index: u32, before: u32, after: u32, handle: H)
        -> Result<(), Error>;
}

impl<H> DetectAction<H> for &[((u32, u32), &[u16])]
    where H: FnMut(&(u32, u32), u16, bool) -> Result<(), Error>
{
    fn detect_action(&self, index: u32, before: u32, after: u32, mut handle: H)
        -> Result<(), Error> {
        self.iter().filter(|((idx, mask), _)| {
            *idx == index && (before ^ after) & *mask > 0
        }).try_for_each(|(key, entries)| {
            match entries.iter().nth(0) {
                Some(&pos) => {
                    let state = after & key.1 == 0;
                    handle(key, pos, state)
                }
                None => {
                    let label = "Program mistake for table of LED position.";
                    Err(Error::new(FileError::Nxio, &label))
                }
            }
        })
    }
}

impl<H> DetectAction<H> for &[(u32, u32)]
    where H: FnMut(usize, &(u32, u32), bool) -> Result<(), Error>
{
    fn detect_action(&self, index: u32, before: u32, after: u32, mut handle: H)
        -> Result<(), Error> {
        self.iter().enumerate().filter(|(_, &(idx, mask))| {
            idx == index && (before ^ after) & mask > 0
        }).try_for_each(|(i, key)| {
            handle(i, key, after & key.1 == 0)
        })
    }
}

impl<H> DetectAction<H> for &[((u32, u32), u8)]
    where H: FnMut(&(u32, u32), u16) -> Result<(), Error>
{
    fn detect_action(&self, index: u32, before: u32, after: u32, mut handle: H)
        -> Result<(), Error>
    {
        self.iter().filter(|((idx, mask), _)| {
            *idx == index && (before ^ after) & *mask > 0
        }).try_for_each(|(key, shift)| {
            let val = ((after & key.1) >> shift) as u16;
            handle(&key, val)
        })
    }
}

pub trait ChooseSingle<H> {
    fn choose_single(&self, index: usize, handle: H) -> Result<(), Error>;
}

impl<H> ChooseSingle<H> for &[&[u16]]
    where H: FnMut(u16, bool) -> Result<(), Error>
{
    fn choose_single(&self, index: usize, mut handle: H) -> Result<(), Error> {
        self.iter().enumerate().try_for_each(|(i, &entries)| {
            if let Some(&pos) = entries.iter().nth(0) {
                handle(pos, i == index)
            } else {
                Ok(())
            }
        })
    }
}

pub trait GetValue<T> {
    fn get_value(&self, states: &T, idx: usize) -> ((u32, u32), u16);
}

impl GetValue<[u32;64]> for &[((u32, u32), u8)] {
    fn get_value(&self, states: &[u32;64], idx: usize) -> ((u32, u32), u16) {
        let (key, shift) = self[idx];
        let val = ((states[key.0 as usize] & key.1) >> shift) as u16;
        (key, val)
    }
}

impl GetValue<[u32;32]> for &[((u32, u32), u8)] {
    fn get_value(&self, states: &[u32;32], idx: usize) -> ((u32, u32), u16) {
        let (key, shift) = self[idx];
        let val = ((states[key.0 as usize] & key.1) >> shift) as u16;
        (key, val)
    }
}

pub trait ComputeValue<H> {
    fn compute_value(&self) -> H;
}

impl ComputeValue<i32> for bool {
    fn compute_value(&self) -> i32 {
        match self {
            true => 127,
            false => 0,
        }
    }
}
