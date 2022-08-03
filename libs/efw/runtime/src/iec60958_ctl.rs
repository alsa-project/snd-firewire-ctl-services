// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use {
    super::*,
    protocols::{hw_ctl::*, hw_info::*, *},
};

#[derive(Default)]
pub struct Iec60958Ctl;

const DEFAULT_NAME: &str = "IEC958 Playback Default";
const MASK_NAME: &str = "IEC958 Playback Mask";

impl Iec60958Ctl {
    const AES0_PROFESSIONAL: u8 = 0x1;
    const AES0_NONAUDIO: u8 = 0x2;

    pub fn load(&mut self, hwinfo: &HwInfo, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let has_spdif = hwinfo
            .clk_srcs
            .iter()
            .find(|src| ClkSrc::Spdif.eq(src))
            .is_some();

        if has_spdif {
            let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, DEFAULT_NAME, 0);
            let _ = card_cntr.add_iec60958_elem(&elem_id, 1, true)?;

            let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MASK_NAME, 0);
            let _ = card_cntr.add_iec60958_elem(&elem_id, 1, false)?;
        }

        Ok(())
    }

    pub fn read(
        &mut self,
        unit: &mut SndEfw,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            DEFAULT_NAME => {
                let mut val = [0; 24];
                let flags = unit.get_flags(timeout_ms)?;
                if flags
                    .iter()
                    .find(|&flag| *flag == HwCtlFlag::SpdifPro)
                    .is_some()
                {
                    val[0] |= Self::AES0_PROFESSIONAL;
                }
                if flags
                    .iter()
                    .find(|&flag| *flag == HwCtlFlag::SpdifNoneAudio)
                    .is_some()
                {
                    val[0] |= Self::AES0_NONAUDIO;
                }
                elem_value.set_iec60958_channel_status(&val);
                Ok(true)
            }
            MASK_NAME => {
                let mut val = [0; 24];
                val[0] = Self::AES0_PROFESSIONAL | Self::AES0_NONAUDIO;
                elem_value.set_iec60958_channel_status(&val);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub fn write(
        &mut self,
        unit: &mut SndEfw,
        elem_id: &ElemId,
        _: &ElemValue,
        new: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            DEFAULT_NAME => {
                let vals = new.iec60958_channel_status();

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

                unit.set_flags(Some(&enable), Some(&disable), timeout_ms)?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
