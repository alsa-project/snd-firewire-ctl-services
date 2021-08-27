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
