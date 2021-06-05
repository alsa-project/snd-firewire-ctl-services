// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use super::common_proto::CommonProto;

pub trait V2Proto<'a> : CommonProto<'a> {
    const CLK_RATE_LABEL: &'a str = "sampling rate";
    const CLK_RATE_MASK: u32 = 0x00000038;
    const CLK_RATE_SHIFT: usize = 3;

    const CLK_SRC_LABEL: &'a str = "clock source";
    const CLK_SRC_MASK: u32 = 0x00000007;
    const CLK_SRC_SHIFT: usize = 0;

    const MAIN_VOL_LABEL: &'a str = "main vol target";
    const MAIN_VOL_MASK: u32 = 0x000f0000;
    const MAIN_VOL_SHIFT: usize = 16;

    const OPT_IN_IFACE_LABEL: &'a str = "optical input interface";
    const OPT_IN_IFACE_MASK: u32 = 0x00000300;
    const OPT_IN_IFACE_SHIFT: usize = 8;

    const OPT_OUT_IFACE_LABEL: &'a str = "optical output interface";
    const OPT_OUT_IFACE_MASK: u32 = 0x00000c00;
    const OPT_OUT_IFACE_SHIFT: usize = 10;

    fn get_clk_rate(&self, unit: &hinawa::SndMotu, vals: &[u8]) -> Result<usize, Error>;
    fn set_clk_rate(&self, unit: &hinawa::SndMotu, vals: &[u8], idx: usize) -> Result<(), Error>;

    fn get_clk_src(&self, unit: &hinawa::SndMotu, vals: &[u8]) -> Result<usize, Error>;
    fn set_clk_src(&self, unit: &hinawa::SndMotu, vals: &[u8], idx: usize) -> Result<(), Error>;

    fn get_main_vol_assign(&self, unit: &hinawa::SndMotu, vals: &[u8]) -> Result<usize, Error>;
    fn set_main_vol_assign(&self, unit: &hinawa::SndMotu, vals: &[u8], idx: usize) -> Result<(), Error>;

    fn get_opt_in_iface_mode(&self, unit: &hinawa::SndMotu, vals: &[u8]) -> Result<usize, Error>;
    fn set_opt_in_iface_mode(&self, unit: &hinawa::SndMotu, vals: &[u8], idx: usize) -> Result<(), Error>;

    fn get_opt_out_iface_mode(&self, unit: &hinawa::SndMotu, vals: &[u8]) -> Result<usize, Error>;
    fn set_opt_out_iface_mode(&self, unit: &hinawa::SndMotu, vals: &[u8], idx: usize) -> Result<(), Error>;
}

impl<'a> V2Proto<'a> for hinawa::FwReq {
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

    fn get_main_vol_assign(&self, unit: &hinawa::SndMotu, vals: &[u8]) -> Result<usize, Error> {
        self.get_idx_from_val(Self::OFFSET_PORT, Self::MAIN_VOL_MASK, Self::MAIN_VOL_SHIFT,
                              Self::MAIN_VOL_LABEL, unit, vals)
    }

    fn set_main_vol_assign(&self, unit: &hinawa::SndMotu, vals: &[u8], idx: usize) -> Result<(), Error> {
        self.set_idx_to_val(Self::OFFSET_PORT, Self::MAIN_VOL_MASK, Self::MAIN_VOL_SHIFT,
                            Self::MAIN_VOL_LABEL, unit, vals, idx)
    }

    fn get_opt_in_iface_mode(&self, unit: &hinawa::SndMotu, vals: &[u8]) -> Result<usize, Error>
    {
        self.get_idx_from_val(Self::OFFSET_PORT, Self::OPT_IN_IFACE_MASK, Self::OPT_IN_IFACE_SHIFT,
                              Self::OPT_IN_IFACE_LABEL, unit, vals)
    }

    fn set_opt_in_iface_mode(&self, unit: &hinawa::SndMotu, vals: &[u8], idx: usize) -> Result<(), Error>
    {
        self.set_idx_to_val(Self::OFFSET_PORT, Self::OPT_IN_IFACE_MASK, Self::OPT_IN_IFACE_SHIFT,
                            Self::OPT_IN_IFACE_LABEL, unit, vals, idx)
    }

    fn get_opt_out_iface_mode(&self, unit: &hinawa::SndMotu, vals: &[u8]) -> Result<usize, Error>
    {
        self.get_idx_from_val(Self::OFFSET_PORT, Self::OPT_OUT_IFACE_MASK, Self::OPT_OUT_IFACE_SHIFT,
                              Self::OPT_OUT_IFACE_LABEL, unit, vals)
    }

    fn set_opt_out_iface_mode(&self, unit: &hinawa::SndMotu, vals: &[u8], idx: usize) -> Result<(), Error>
    {
        self.set_idx_to_val(Self::OFFSET_PORT, Self::OPT_OUT_IFACE_MASK, Self::OPT_OUT_IFACE_SHIFT,
                            Self::OPT_OUT_IFACE_LABEL, unit, vals, idx)
    }
}
