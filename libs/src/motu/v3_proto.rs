// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use super::common_proto::CommonProto;

pub trait V3Proto<'a> : CommonProto<'a> {
    const OFFSET_OPT: u32 = 0x0c94;

    const CLK_RATE_LABEL: &'a str = "sampling rate";
    const CLK_RATE_MASK: u32 = 0x0000ff00;
    const CLK_RATE_SHIFT: usize = 8;

    const CLK_SRC_LABEL: &'a str = "clock source";
    const CLK_SRC_MASK: u32 = 0x000000ff;
    const CLK_SRC_SHIFT: usize = 0;

    const PORT_MAIN_LABEL: &'a str = "main out assign";
    const PORT_MAIN_MASK: u32 = 0x000000f0;
    const PORT_MAIN_SHIFT: usize = 4;

    const PORT_RETURN_LABEL: &'a str = "return assign";
    const PORT_RETURN_MASK: u32 = 0x00000f00;
    const PORT_RETURN_SHIFT: usize = 8;

    fn get_clk_rate(&self, unit: &hinawa::SndMotu, vals: &[u8]) -> Result<usize, Error>;
    fn set_clk_rate(&self, unit: &hinawa::SndMotu, vals: &[u8], idx: usize) -> Result<(), Error>;

    fn get_clk_src(&self, unit: &hinawa::SndMotu, vals: &[u8]) -> Result<usize, Error>;
    fn set_clk_src(&self, unit: &hinawa::SndMotu, vals: &[u8], idx: usize) -> Result<(), Error>;

    fn get_main_assign(&self, unit: &hinawa::SndMotu, vals: &[u8]) -> Result<usize, Error>;
    fn set_main_assign(&self, unit: &hinawa::SndMotu, vals: &[u8], idx: usize) -> Result<(), Error>;

    fn get_return_assign(&self, unit: &hinawa::SndMotu, vals: &[u8]) -> Result<usize, Error>;
    fn set_return_assign(&self, unit: &hinawa::SndMotu, vals: &[u8], idx: usize) -> Result<(), Error>;

    fn get_opt_iface_masks(&self, is_out: bool, is_b: bool) -> (u32, u32) {
        let mut enabled_mask = 0x00000001;
        if is_out {
            enabled_mask <<= 8;
        }
        if is_b {
            enabled_mask <<= 1;
        }

        let mut no_adat_mask = 0x00010000;
        if is_out {
            no_adat_mask <<= 2;
        }
        if is_b {
            no_adat_mask <<= 4;
        }

        (enabled_mask, no_adat_mask)
    }

    fn set_opt_iface_mode(&self, unit: &hinawa::SndMotu, is_out: bool, is_b: bool,
                          enable: bool, no_adat: bool)
        -> Result<(), Error>;

    fn get_opt_iface_mode(&self, unit: &hinawa::SndMotu, is_out: bool, is_b: bool)
        -> Result<(bool, bool), Error>;
}

impl<'a> V3Proto<'a> for hinawa::FwReq {
    fn get_clk_rate(&self, unit: &hinawa::SndMotu, vals: &[u8]) -> Result<usize, Error>
    {
        self.get_idx_from_val(Self::OFFSET_CLK, Self::CLK_RATE_MASK, Self::CLK_RATE_SHIFT,
                              Self::CLK_RATE_LABEL, unit, vals)
    }

    fn set_clk_rate(&self, unit: &hinawa::SndMotu, vals: &[u8], idx: usize) -> Result<(), Error>
    {
        self.set_idx_to_val(Self::OFFSET_CLK, Self::CLK_RATE_MASK, Self::CLK_RATE_SHIFT,
                            Self::CLK_RATE_LABEL, unit, vals, idx)
    }

    fn get_clk_src(&self, unit: &hinawa::SndMotu, vals: &[u8]) -> Result<usize, Error>
    {
        self.get_idx_from_val(Self::OFFSET_CLK, Self::CLK_SRC_MASK, Self::CLK_SRC_SHIFT,
                              Self::CLK_SRC_LABEL, unit, vals)
    }

    fn set_clk_src(&self, unit: &hinawa::SndMotu, vals: &[u8], idx: usize) -> Result<(), Error>
    {
        self.set_idx_to_val(Self::OFFSET_CLK, Self::CLK_SRC_MASK, Self::CLK_SRC_SHIFT,
                            Self::CLK_SRC_LABEL, unit, vals, idx)
    }

    fn get_main_assign(&self, unit: &hinawa::SndMotu, vals: &[u8]) -> Result<usize, Error> {
        self.get_idx_from_val(Self::OFFSET_PORT, Self::PORT_MAIN_MASK, Self::PORT_MAIN_SHIFT,
                              Self::PORT_MAIN_LABEL, unit, vals)
    }

    fn set_main_assign(&self, unit: &hinawa::SndMotu, vals: &[u8], idx: usize) -> Result<(), Error> {
        self.set_idx_to_val(Self::OFFSET_PORT, Self::PORT_MAIN_MASK, Self::PORT_MAIN_SHIFT,
                            Self::PORT_MAIN_LABEL, unit, vals, idx)
    }

    fn get_return_assign(&self, unit: &hinawa::SndMotu, vals: &[u8]) -> Result<usize, Error> {
        self.get_idx_from_val(Self::OFFSET_PORT, Self::PORT_RETURN_MASK, Self::PORT_RETURN_SHIFT,
                              Self::PORT_RETURN_LABEL, unit, vals)
    }

    fn set_return_assign(&self, unit: &hinawa::SndMotu, vals: &[u8], idx: usize) -> Result<(), Error> {
        self.set_idx_to_val(Self::OFFSET_PORT, Self::PORT_RETURN_MASK, Self::PORT_RETURN_SHIFT,
                            Self::PORT_RETURN_LABEL, unit, vals, idx)
    }

    fn set_opt_iface_mode(&self, unit: &hinawa::SndMotu, is_out: bool, is_b: bool,
                          enable: bool, no_adat: bool)
        -> Result<(), Error>
    {
        let (enabled_mask, no_adat_mask) = self.get_opt_iface_masks(is_out, is_b);
        let mut quad = self.read_quad(unit, Self::OFFSET_OPT)?;
        quad &= !enabled_mask;
        quad &= !no_adat_mask;
        if enable {
            quad |= enabled_mask;
        }
        if no_adat {
            quad |= no_adat_mask;
        }
        self.write_quad(unit, Self::OFFSET_OPT, quad)
    }

    fn get_opt_iface_mode(&self, unit: &hinawa::SndMotu, is_out: bool, is_b: bool)
        -> Result<(bool, bool), Error>
    {
        let quad = self.read_quad(unit, Self::OFFSET_OPT)?;

        let (enabled_mask, no_adat_mask) = self.get_opt_iface_masks(is_out, is_b);
        let enabled = (quad & enabled_mask) > 0;
        let no_adat = (quad & no_adat_mask) > 0;

        Ok((enabled, no_adat))
    }
}
