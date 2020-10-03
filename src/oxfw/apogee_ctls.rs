// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use alsactl::{ElemValueExt, ElemValueExtManual};

use crate::card_cntr;

use crate::ta1394::{AvcAddr, Ta1394Avc};
use super::apogee_proto::{VendorCmd, ApogeeCmd};

const TIMEOUT_MS: u32 = 100;

pub struct OutputCtl;

impl<'a> OutputCtl {
    const SRC_NAME: &'a str = "output-source";
    const LEVEL_NAME: &'a str = "output-level";
    const MUTE_FOR_LINE_OUT: &'a str = "mute-for-line-out";
    const MUTE_FOR_HP_OUT: &'a str = "mute-for-hp-out";

    const SRC_LABELS: &'a [&'a str] = &["stream-1/2", "mixer-1/2"];
    const LEVEL_LABELS: &'a [&'a str] = &["instrument", "-10dB"];

    const MUTE_LABELS: &'a [&'a str] = &[
        "never",
        "normal",
        "swapped"
    ];

    pub fn new() -> Self {
        OutputCtl {}
    }

    pub fn load(&mut self, _: &hinawa::FwFcp, card_cntr: &mut card_cntr::CardCntr)
        -> Result<(), Error>
    {
        // For source of analog outputs.
        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Mixer, 0, 0,
                                                   Self::SRC_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, Self::SRC_LABELS, None, true)?;

        // For level of analog outputs.
        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Mixer, 0, 0,
                                                   Self::LEVEL_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, Self::LEVEL_LABELS, None, true)?;

        // For association of mute state to line output.
        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Mixer,
                                                   0, 0, Self::MUTE_FOR_LINE_OUT, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, Self::MUTE_LABELS, None, true)?;

        // For association of mute state to hp output.
        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Mixer,
                                                   0, 0, Self::MUTE_FOR_HP_OUT, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, Self::MUTE_LABELS, None, true)?;

        Ok(())
    }

    fn parse_mute_mode(mute_mode: bool, unmute_mode: bool) -> u32 {
        let mut idx = 0;
        if !mute_mode {
            idx |= 0x01;
        }
        if !unmute_mode {
            idx |= 0x02;
        }
        if idx > 2 {
            idx = 0
        }
        idx
    }

    pub fn read(&mut self, avc: &hinawa::FwFcp, company_id: &[u8;3], elem_id: &alsactl::ElemId,
                elem_value: &mut alsactl::ElemValue)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::SRC_NAME => {
                let mut op = ApogeeCmd::new(company_id, VendorCmd::UseMixerOut);
                avc.status(&AvcAddr::Unit, &mut op, TIMEOUT_MS)?;
                elem_value.set_enum(&[op.get_enum()]);
                Ok(true)
            }
            Self::LEVEL_NAME => {
                let mut op = ApogeeCmd::new(company_id, VendorCmd::OutAttr);
                avc.status(&AvcAddr::Unit, &mut op, TIMEOUT_MS)?;
                elem_value.set_enum(&[op.get_enum()]);
                Ok(true)
            }
            Self::MUTE_FOR_LINE_OUT => {
                let mut mute_op = ApogeeCmd::new(company_id, VendorCmd::MuteForLineOut);
                avc.status(&AvcAddr::Unit, &mut mute_op, TIMEOUT_MS)?;
                let mut unmute_op = ApogeeCmd::new(company_id, VendorCmd::UnmuteForLineOut);
                avc.status(&AvcAddr::Unit, &mut unmute_op, TIMEOUT_MS)?;

                let idx = Self::parse_mute_mode(mute_op.get_enum() > 0, unmute_op.get_enum() > 0);
                elem_value.set_enum(&[idx]);
                Ok(true)
            }
            Self::MUTE_FOR_HP_OUT => {
                let mut mute_op = ApogeeCmd::new(company_id, VendorCmd::MuteForHpOut);
                avc.status(&AvcAddr::Unit, &mut mute_op, TIMEOUT_MS)?;
                let mut unmute_op = ApogeeCmd::new(company_id, VendorCmd::UnmuteForHpOut);
                avc.status(&AvcAddr::Unit, &mut unmute_op, TIMEOUT_MS)?;

                let idx = Self::parse_mute_mode(mute_op.get_enum() > 0, unmute_op.get_enum() > 0);
                elem_value.set_enum(&[idx]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn build_mute_mode(idx: u32) -> (bool, bool) {
        (idx & 0x01 == 0, idx & 0x02 == 0)
    }

    pub fn write(&mut self, avc: &hinawa::FwFcp, company_id: &[u8;3], elem_id: &alsactl::ElemId,
                 _: &alsactl::ElemValue, new: &alsactl::ElemValue)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::SRC_NAME => {
                let mut vals = [0];
                new.get_enum(&mut vals);
                let mut op = ApogeeCmd::new(company_id, VendorCmd::UseMixerOut);
                op.put_enum(vals[0]);
                avc.control(&AvcAddr::Unit, &mut op, TIMEOUT_MS)?;
                Ok(true)
            }
            Self::LEVEL_NAME => {
                let mut vals = [0];
                new.get_enum(&mut vals);
                let mut op = ApogeeCmd::new(company_id, VendorCmd::OutAttr);
                op.put_enum(vals[0]);
                avc.control(&AvcAddr::Unit, &mut op, TIMEOUT_MS)?;
                Ok(true)
            }
            Self::MUTE_FOR_LINE_OUT => {
                let mut vals = [0];
                new.get_enum(&mut vals);
                let (mute_mode, unmute_mode) = Self::build_mute_mode(vals[0]);

                let mut op = ApogeeCmd::new(company_id, VendorCmd::MuteForLineOut);
                op.put_enum(mute_mode as u32);
                avc.control(&AvcAddr::Unit, &mut op, TIMEOUT_MS)?;

                let mut op = ApogeeCmd::new(company_id, VendorCmd::UnmuteForLineOut);
                op.put_enum(unmute_mode as u32);
                avc.control(&AvcAddr::Unit, &mut op, TIMEOUT_MS)?;

                Ok(true)
            }
            Self::MUTE_FOR_HP_OUT => {
                let mut vals = [0];
                new.get_enum(&mut vals);
                let (mute_mode, unmute_mode) = Self::build_mute_mode(vals[0]);

                let mut op = ApogeeCmd::new(company_id, VendorCmd::MuteForHpOut);
                op.put_enum(mute_mode as u32);
                avc.control(&AvcAddr::Unit, &mut op, TIMEOUT_MS)?;

                let mut op = ApogeeCmd::new(company_id, VendorCmd::UnmuteForHpOut);
                op.put_enum(unmute_mode as u32);
                avc.control(&AvcAddr::Unit, &mut op, TIMEOUT_MS)?;

                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
