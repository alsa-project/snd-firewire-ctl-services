// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use core::card_cntr;

use alsactl::{ElemValueExt, ElemValueExtManual};

use super::transactions::{HwInfo, HwCap, EfwHwCtl, HwCtlFlag};

pub struct Iec60958Ctl {
}

impl<'a> Iec60958Ctl {
    const DEFAULT: &'a str = "IEC958 Playback Default";
    const MASK: &'a str = "IEC958 Playback Mask";

    const AES0_PROFESSIONAL: u8 = 0x1;
    const AES0_NONAUDIO: u8 = 0x2;

    pub fn new() -> Self {
        Iec60958Ctl{}
    }

    pub fn load(&mut self, hwinfo: &HwInfo, card_cntr: &mut card_cntr::CardCntr)
        -> Result<(), Error>
    {
        if hwinfo.caps.iter().find(|&cap| *cap == HwCap::SpdifCoax).is_some() {
            let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Mixer, 0, 0, Self::DEFAULT, 0);
            let _ = card_cntr.add_iec60958_elem(&elem_id, 1, true)?;

            let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Mixer, 0, 0, Self::MASK, 0);
            let _ = card_cntr.add_iec60958_elem(&elem_id, 1, false)?;
        }

        Ok(())
    }

    pub fn read(
        &mut self,
        unit: &hinawa::SndEfw,
        elem_id: &alsactl::ElemId,
        elem_value: &mut alsactl::ElemValue,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            Self::DEFAULT => {
                let mut val = [0;24];
                let flags = EfwHwCtl::get_flags(unit)?;
                if flags.iter().find(|&flag| *flag == HwCtlFlag::SpdifPro).is_some() {
                    val[0] |= Self::AES0_PROFESSIONAL;
                }
                if flags.iter().find(|&flag| *flag == HwCtlFlag::SpdifNoneAudio).is_some() {
                    val[0] |= Self::AES0_NONAUDIO;
                }
                elem_value.set_iec60958_channel_status(&val);
                Ok(true)
            }
            Self::MASK => {
                let mut val = [0;24];
                val[0] = Self::AES0_PROFESSIONAL | Self::AES0_NONAUDIO;
                elem_value.set_iec60958_channel_status(&val);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub fn write(
        &mut self,
        unit: &hinawa::SndEfw,
        elem_id: &alsactl::ElemId,
        _: &alsactl::ElemValue,
        new: &alsactl::ElemValue,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            Self::DEFAULT => {
                let mut vals = [0;24];
                new.get_iec60958_channel_status(&mut vals);

                let mut enable = vec![];
                let mut disable = vec![];
                if vals[0] & Self::AES0_PROFESSIONAL > 0 {
                    enable.push(HwCtlFlag::SpdifPro);
                } else {
                    disable.push(HwCtlFlag::SpdifPro);
                }
                if vals[0] & Self::AES0_NONAUDIO > 0 {
                    enable.push(HwCtlFlag::SpdifNoneAudio);
                } else {
                    disable.push(HwCtlFlag::SpdifNoneAudio);
                }

                EfwHwCtl::set_flags(unit, &enable, &disable)?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
