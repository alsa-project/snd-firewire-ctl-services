// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

pub const FMT_IS_AMDTP: u8 = 0x90;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum AmdtpEventType {
    Am824,
    AudioPack,
    FloatingPoint,
    Reserved(u8),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct AmdtpFdf {
    pub ev_type: AmdtpEventType,
    pub cmd_rate_ctl: bool,
    pub freq: u32,
}

impl AmdtpFdf {
    const EVT_MASK: u8 = 0x03;
    const EVT_SHIFT: usize = 4;

    const NFLAG_MASK: u8 = 0x01;
    const NFLAG_SHIFT: usize = 3;

    const SFC_MASK: u8 = 0x07;
    const SFC_SHIFT: usize = 0;

    const AM824: u8 = 0x00;
    const AUDIOPACK: u8 = 0x01;
    const FLOATINGPOINT: u8 = 0x02;

    const SFC_32000: u8 = 0x00;
    const SFC_44100: u8 = 0x01;
    const SFC_48000: u8 = 0x02;
    const SFC_88200: u8 = 0x03;
    const SFC_96000: u8 = 0x04;
    const SFC_176400: u8 = 0x05;
    const SFC_192000: u8 = 0x06;
    const SFC_RESERVED: u8 = 0x07;

    pub fn new(ev_type: AmdtpEventType, cmd_rate_ctl: bool, freq: u32) -> Self {
        AmdtpFdf {
            ev_type,
            cmd_rate_ctl,
            freq,
        }
    }
}

impl From<&[u8]> for AmdtpFdf {
    fn from(data: &[u8]) -> Self {
        let ev_type = (data[0] >> Self::EVT_SHIFT) & Self::EVT_MASK;
        let nflag = (data[0] >> Self::NFLAG_SHIFT) & Self::NFLAG_MASK;
        let sfc = (data[0] >> Self::SFC_SHIFT) & Self::SFC_MASK;

        let ev_type = match ev_type {
            Self::AM824 => AmdtpEventType::Am824,
            Self::AUDIOPACK => AmdtpEventType::AudioPack,
            Self::FLOATINGPOINT => AmdtpEventType::FloatingPoint,
            _ => AmdtpEventType::Reserved(data[0]),
        };

        let freq = match sfc {
            Self::SFC_32000 => 32000,
            Self::SFC_44100 => 44100,
            Self::SFC_48000 => 48000,
            Self::SFC_88200 => 88200,
            Self::SFC_96000 => 96000,
            Self::SFC_176400 => 176400,
            Self::SFC_192000 => 192000,
            _ => 0,
        };

        AmdtpFdf {
            ev_type,
            cmd_rate_ctl: nflag > 0,
            freq,
        }
    }
}

impl From<AmdtpFdf> for [u8; 3] {
    fn from(fdf: AmdtpFdf) -> Self {
        let mut data = [0xff; 3];

        let ev_type = match fdf.ev_type {
            AmdtpEventType::Am824 => AmdtpFdf::AM824,
            AmdtpEventType::AudioPack => AmdtpFdf::AUDIOPACK,
            AmdtpEventType::FloatingPoint => AmdtpFdf::FLOATINGPOINT,
            AmdtpEventType::Reserved(val) => val,
        };

        let nflag = fdf.cmd_rate_ctl;

        let sfc = match fdf.freq {
            32000 => AmdtpFdf::SFC_32000,
            44100 => AmdtpFdf::SFC_44100,
            48000 => AmdtpFdf::SFC_48000,
            88200 => AmdtpFdf::SFC_88200,
            96000 => AmdtpFdf::SFC_96000,
            176400 => AmdtpFdf::SFC_176400,
            192000 => AmdtpFdf::SFC_192000,
            _ => AmdtpFdf::SFC_RESERVED,
        };

        data[0] = ((ev_type & AmdtpFdf::EVT_MASK) << AmdtpFdf::EVT_SHIFT)
            | ((nflag as u8 & AmdtpFdf::NFLAG_MASK) << AmdtpFdf::NFLAG_SHIFT)
            | ((sfc & AmdtpFdf::SFC_MASK) << AmdtpFdf::SFC_SHIFT);

        data
    }
}
