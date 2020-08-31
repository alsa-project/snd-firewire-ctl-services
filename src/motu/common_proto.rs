// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::{Error, FileError};
use hinawa::FwTcode;
use hinawa::FwReqExtManual;
use hinawa::{SndUnitExt};

pub trait CommonProto<'a> {
    const BASE_OFFSET: u64 = 0xfffff0000000;

    const TIMEOUT: u32 = 100;

    const OFFSET_CLK: u32 = 0x0b14;
    const OFFSET_PORT: u32 = 0x0c04;
    const OFFSET_CLK_DISPLAY: u32 = 0x0c60;

    const WORD_OUT_LABEL: &'a str = "word out";
    const WORD_OUT_MASK: u32 = 0x08000000;
    const WORD_OUT_SHIFT: usize = 27;

    const PORT_PHONE_LABEL: &'a str = "phone assign";
    const PORT_PHONE_MASK: u32 = 0x0000000f;
    const PORT_PHONE_SHIFT: usize = 0;

    const DISPLAY_CHARS: usize = 4 * 4;

    fn read_quad(&self, unit: &hinawa::SndMotu, offset: u32) -> Result<u32, Error>;
    fn write_quad(&self, unit: &hinawa::SndMotu, offset: u32, quad: u32) -> Result<(), Error>;

    fn get_idx_from_val(&self, offset: u32, mask: u32, shift: usize, label: &str,
                        unit: &hinawa::SndMotu, vals: &[u8])
        -> Result<usize, Error>;
    fn set_idx_to_val(&self, offset: u32, mask: u32, shift: usize, label: &str,
                      unit: &hinawa::SndMotu, vals: &[u8], idx: usize)
        -> Result<(), Error>;

    fn get_word_out(&self, unit: &hinawa::SndMotu, vals: &[u8]) -> Result<usize, Error>;
    fn set_word_out(&self, unit: &hinawa::SndMotu, vals: &[u8], idx: usize) -> Result<(), Error>;

    fn get_phone_assign(&self, unit: &hinawa::SndMotu, vals: &[u8]) -> Result<usize, Error>;
    fn set_phone_assign(&self, unit: &hinawa::SndMotu, vals: &[u8], idx: usize) -> Result<(), Error>;

    fn update_clk_disaplay(&self, unit: &hinawa::SndMotu, label: &str) -> Result<(), Error>;
}

impl<'a> CommonProto<'a> for hinawa::FwReq {
    fn read_quad(&self, unit: &hinawa::SndMotu, offset: u32) -> Result<u32, Error> {
        let mut frame = [0;4];
        self.transaction_sync(&unit.get_node(), FwTcode::ReadQuadletRequest,
                              Self::BASE_OFFSET + offset as u64, 4, &mut frame, Self::TIMEOUT)?;
        Ok(u32::from_be_bytes(frame))
    }

    fn write_quad(&self, unit: &hinawa::SndMotu, offset: u32, quad: u32) -> Result<(), Error> {
        let mut frame = [0;4];
        frame.copy_from_slice(&quad.to_be_bytes());
        self.transaction_sync(&unit.get_node(), FwTcode::WriteQuadletRequest,
                              Self::BASE_OFFSET + offset as u64, 4, &mut frame, Self::TIMEOUT)
    }

    fn get_idx_from_val(&self, offset: u32, mask: u32, shift: usize, label: &str, unit: &hinawa::SndMotu,
                        vals: &[u8])
        -> Result<usize, Error>
    {
        let quad = self.read_quad(unit, offset)?;
        let val = ((quad & mask) >> shift) as u8;
        vals.iter().position(|&v| v == val).ok_or_else(|| {
            let label = format!("Detect invalid value for {}: {:02x}", label, val);
            Error::new(FileError::Io, &label)
        })
    }

    fn set_idx_to_val(&self, offset: u32, mask: u32, shift: usize, label: &str, unit: &hinawa::SndMotu,
                      vals: &[u8], idx: usize)
        -> Result<(), Error>
    {
        if idx >= vals.len() {
            let label = format!("Invalid argument for {}: {} {}", label, vals.len(), idx);
            return Err(Error::new(FileError::Inval, &label));
        }
        let mut quad = self.read_quad(unit, offset)?;
        quad &= !mask;
        quad |= (vals[idx] as u32) << shift;
        self.write_quad(unit, offset, quad)
    }

    fn get_word_out(&self, unit: &hinawa::SndMotu, vals: &[u8]) -> Result<usize, Error> {
        self.get_idx_from_val(Self::OFFSET_CLK, Self::WORD_OUT_MASK, Self::WORD_OUT_SHIFT,
                              Self::WORD_OUT_LABEL, unit, vals)
    }

    fn set_word_out(&self, unit: &hinawa::SndMotu, vals: &[u8], idx: usize) -> Result<(), Error> {
        self.set_idx_to_val(Self::OFFSET_CLK, Self::WORD_OUT_MASK, Self::WORD_OUT_SHIFT,
                            Self::WORD_OUT_LABEL, unit, vals, idx)
    }

    fn get_phone_assign(&self, unit: &hinawa::SndMotu, vals: &[u8]) -> Result<usize, Error> {
        self.get_idx_from_val(Self::OFFSET_PORT, Self::PORT_PHONE_MASK, Self::PORT_PHONE_SHIFT,
                              Self::PORT_PHONE_LABEL, unit, vals)
    }

    fn set_phone_assign(&self, unit: &hinawa::SndMotu, vals: &[u8], idx: usize) -> Result<(), Error> {
        self.set_idx_to_val(Self::OFFSET_PORT, Self::PORT_PHONE_MASK, Self::PORT_PHONE_SHIFT,
                            Self::PORT_PHONE_LABEL, unit, vals, idx)
    }

    fn update_clk_disaplay(&self, unit: &hinawa::SndMotu, label: &str) -> Result<(), Error> {
        let mut chars = [0;Self::DISPLAY_CHARS];
        chars.iter_mut().zip(label.bytes()).for_each(|(c, l)| *c = l);

        (0..(Self::DISPLAY_CHARS / 4)).try_for_each(|i| {
            let mut frame = [0;4];
            frame.copy_from_slice(&chars[(i * 4)..(i * 4 + 4)]);
            frame.reverse();
            let quad = u32::from_ne_bytes(frame);
            let offset = Self::OFFSET_CLK_DISPLAY + 4 * i as u32;
            self.write_quad(unit, offset, quad)
        })
    }
}
