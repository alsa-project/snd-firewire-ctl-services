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

pub struct MixerCtl;

impl<'a> MixerCtl {
    const TARGET_LABELS: &'a [&'a str] = &["mixer-1", "mixer-2"];

    const SRC_LABELS: &'a [&'a str] = &["stream-1", "stream-2", "analog-1", "analog-2"];

    const MIXER_NAME: &'a str = "mixer-source-gain";

    const GAIN_MIN: i32 = 0;
    const GAIN_MAX: i32 = 0x3fff;
    const GAIN_STEP: i32 = 0xff;

    pub fn new() -> Self {
        MixerCtl {}
    }

    pub fn load(&mut self, _: &hinawa::FwFcp, card_cntr: &mut card_cntr::CardCntr)
        -> Result<(), Error>
    {
        // For gain of mixer sources.
        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Mixer,
                                                   0, 0, Self::MIXER_NAME, 0);
        let _ = card_cntr.add_int_elems(&elem_id, Self::TARGET_LABELS.len(),
                                        Self::GAIN_MIN, Self::GAIN_MAX, Self::GAIN_STEP,
                                        Self::SRC_LABELS.len(),
                                        None, true)?;

        Ok(())
    }

    pub fn read(&mut self, avc: &hinawa::FwFcp, company_id: &[u8;3], elem_id: &alsactl::ElemId,
                elem_value: &mut alsactl::ElemValue)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::MIXER_NAME => {
                let dst = elem_id.get_index();
                let mut vals = vec![0;Self::SRC_LABELS.len()];
                vals.iter_mut().enumerate().try_for_each(|(src, val)| {
                    let mut op = ApogeeCmd::new(company_id, VendorCmd::MixerSrc(src as u8, dst as u8));
                    avc.status(&AvcAddr::Unit, &mut op, TIMEOUT_MS)?;
                    *val = op.read_u16() as i32;
                    Ok(())
                })?;
                elem_value.set_int(&vals);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub fn write(&mut self, avc: &hinawa::FwFcp, company_id: &[u8;3], elem_id: &alsactl::ElemId,
                 old: &alsactl::ElemValue, new: &alsactl::ElemValue)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::MIXER_NAME => {
                let dst = elem_id.get_index();
                let mut vals = vec![0;8];
                new.get_int(&mut vals[..4]);
                old.get_int(&mut vals[4..]);

                vals[..4].iter().zip(&vals[4..]).enumerate().try_for_each(|(src, (n, o))| {
                    if n != o {
                        let mut op = ApogeeCmd::new(company_id, VendorCmd::MixerSrc(src as u8, dst as u8));
                        op.write_u16(*n as u16);
                        avc.control(&AvcAddr::Unit, &mut op, TIMEOUT_MS)
                    } else {
                        Ok::<(), Error>(())
                    }
                })?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

pub struct InputCtl;

impl<'a> InputCtl {
    const SRC_NAME: &'a str = "input-source";
    const PHONE_LEVEL_NAME: &'a str = "input-phone-level";
    const LINE_LEVEL_NAME: &'a str = "input-line-level";
    const POLARITY_NAME: &'a str = "mic-polarity";
    const PHANTOM_NAME: &'a str = "mic-phantom";
    const CLICKLESS_NAME: &'a str = "input-clickless";

    const TARGET_LABELS: &'a [&'a str] = &["Analog-in-1", "Analog-in-2"];
    const MIC_LABELS: &'a [&'a str] = &["Mic-1", "Mic-2"];
    const PHONE_LABELS: &'a [&'a str] = &["Phone-1", "Phone-2"];
    const SRC_LABELS: &'a [&'a str] = &["Mic", "Phone"];
    const PHONE_LEVEL_LABELS: &'a [&'a str] = &["Instrument", "Line"];
    const LINE_LEVEL_LABELS: &'a [&'a str] = &["+4dB", "-10dB"];

    pub fn new() -> Self {
        InputCtl{}
    }

    pub fn load(&mut self, _: &hinawa::FwFcp, card_cntr: &mut card_cntr::CardCntr)
        -> Result<(), Error>
    {
        // For polarity of microphone.
        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Mixer,
                                                   0, 0, Self::POLARITY_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, Self::MIC_LABELS.len(), true)?;

        // For level of input in phone jack.
        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Mixer,
                                                   0, 0, Self::PHONE_LEVEL_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1,
                                         Self::PHONE_LABELS.len(), Self::PHONE_LEVEL_LABELS,
                                         None, true)?;

        // For level of line input in phone jack.
        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Mixer,
                                                   0, 0, Self::LINE_LEVEL_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1,
                                         Self::PHONE_LABELS.len(), Self::LINE_LEVEL_LABELS,
                                         None, true)?;

        // For phantom powering of microphone.
        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Mixer,
                                                   0, 0, Self::PHANTOM_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, Self::MIC_LABELS.len(), true)?;

        // For source of analog inputs.
        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Mixer,
                                                   0, 0, Self::SRC_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, Self::TARGET_LABELS.len(),
                                         Self::SRC_LABELS, None, true)?;

        // For input clickless.
        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Mixer,
                                                   0, 0, Self::CLICKLESS_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        Ok(())
    }

    pub fn read(&mut self, avc: &hinawa::FwFcp, company_id: &[u8;3], elem_id: &alsactl::ElemId,
                elem_value: &mut alsactl::ElemValue)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::POLARITY_NAME => {
                let mut vals = [false;2];
                vals.iter_mut().enumerate().try_for_each(|(i, v)| {
                    let mut op = ApogeeCmd::new(company_id, VendorCmd::MicPolarity(i as u8));
                    avc.status(&AvcAddr::Unit, &mut op, TIMEOUT_MS)?;
                    *v = op.get_enum() > 0;
                    Ok(())
                })?;
                elem_value.set_bool(&vals);
                Ok(true)
            }
            Self::PHONE_LEVEL_NAME => {
                let mut vals = [0;2];
                vals.iter_mut().enumerate().try_for_each(|(i, v)| {
                    let mut op = ApogeeCmd::new(company_id, VendorCmd::PhoneInLine(i as u8));
                    avc.status(&AvcAddr::Unit, &mut op, TIMEOUT_MS)?;
                    *v = op.get_enum();
                    Ok(())
                })?;
                elem_value.set_enum(&vals);
                Ok(true)
            }
            Self::LINE_LEVEL_NAME => {
                let mut vals = [0;2];
                vals.iter_mut().enumerate().try_for_each(|(i, v)| {
                    let mut op = ApogeeCmd::new(company_id, VendorCmd::LineInLevel(i as u8));
                    avc.status(&AvcAddr::Unit, &mut op, TIMEOUT_MS)?;
                    *v = op.get_enum();
                    Ok(())
                })?;
                elem_value.set_enum(&vals);
                Ok(true)
            }
            Self::PHANTOM_NAME => {
                let mut vals = [false;2];
                vals.iter_mut().enumerate().try_for_each(|(i, v)| {
                    let mut op = ApogeeCmd::new(company_id, VendorCmd::MicPhantom(i as u8));
                    avc.status(&AvcAddr::Unit, &mut op, TIMEOUT_MS)?;
                    *v = op.get_enum() > 0;
                    Ok(())
                })?;
                elem_value.set_bool(&vals);
                Ok(true)
            }
            Self::SRC_NAME => {
                let mut vals = [0;2];
                vals.iter_mut().enumerate().try_for_each(|(i, v)| {
                    let mut op = ApogeeCmd::new(company_id, VendorCmd::MicIn(i as u8));
                    avc.status(&AvcAddr::Unit, &mut op, TIMEOUT_MS)?;
                    *v = op.get_enum();
                    Ok(())
                })?;
                elem_value.set_enum(&vals);
                Ok(true)
            }
            Self::CLICKLESS_NAME => {
                let mut op = ApogeeCmd::new(company_id, VendorCmd::InClickless);
                avc.status(&AvcAddr::Unit, &mut op, TIMEOUT_MS)?;
                elem_value.set_bool(&[op.get_enum() > 0]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub fn write(&mut self, avc: &hinawa::FwFcp, company_id: &[u8;3], elem_id: &alsactl::ElemId,
                 old: &alsactl::ElemValue, new: &alsactl::ElemValue)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::POLARITY_NAME => {
                let mut vals = [false;4];
                new.get_bool(&mut vals[..2]);
                old.get_bool(&mut vals[2..]);
                vals[..2].iter().zip(vals[2..].iter()).enumerate()
                    .filter(|(_, (n, o))| n != o)
                    .try_for_each(|(i, (n, _))| {
                        let mut op = ApogeeCmd::new(company_id, VendorCmd::MicPolarity(i as u8));
                        op.put_enum(*n as u32);
                        avc.control(&AvcAddr::Unit, &mut op, TIMEOUT_MS)
                    })?;
                Ok(true)
            }
            Self::PHONE_LEVEL_NAME => {
                let mut vals = [0;4];
                new.get_enum(&mut vals[..2]);
                old.get_enum(&mut vals[2..]);
                vals[..2].iter().zip(vals[2..].iter()).enumerate()
                    .filter(|(_, (n, o))| n != o)
                    .try_for_each(|(i, (n, _))| {
                        let mut op = ApogeeCmd::new(company_id, VendorCmd::PhoneInLine(i as u8));
                        op.put_enum(*n);
                        avc.control(&AvcAddr::Unit, &mut op, TIMEOUT_MS)
                    })?;
                Ok(true)
            }
            Self::LINE_LEVEL_NAME => {
                let mut vals = [0;4];
                new.get_enum(&mut vals[..2]);
                old.get_enum(&mut vals[2..]);
                vals[..2].iter().zip(vals[2..].iter()).enumerate()
                    .filter(|(_, (n, o))| n != o)
                    .try_for_each(|(i, (n, _))| {
                        let mut op = ApogeeCmd::new(company_id, VendorCmd::LineInLevel(i as u8));
                        op.put_enum(*n);
                        avc.control(&AvcAddr::Unit, &mut op, TIMEOUT_MS)
                    })?;
                Ok(true)
            }
            Self::PHANTOM_NAME => {
                let mut vals = [false;4];
                new.get_bool(&mut vals[..2]);
                old.get_bool(&mut vals[2..]);
                vals[..2].iter().zip(vals[2..].iter()).enumerate()
                    .filter(|(_, (n, o))| n != o)
                    .try_for_each(|(i, (n, _))| {
                        let mut op = ApogeeCmd::new(company_id, VendorCmd::MicPhantom(i as u8));
                        op.put_enum(*n as u32);
                        avc.control(&AvcAddr::Unit, &mut op, TIMEOUT_MS)
                    })?;
                Ok(true)
            }
            Self::SRC_NAME => {
                let mut vals = [0;4];
                new.get_enum(&mut vals[..2]);
                old.get_enum(&mut vals[2..]);
                vals[..2].iter().zip(vals[2..].iter()).enumerate()
                    .filter(|(_, (n, o))| n != o)
                    .try_for_each(|(i, (n, _))| {
                        let mut op = ApogeeCmd::new(company_id, VendorCmd::MicIn(i as u8));
                        op.put_enum(*n);
                        avc.control(&AvcAddr::Unit, &mut op, TIMEOUT_MS)
                    })?;
                Ok(true)
            }
            Self::CLICKLESS_NAME => {
                let mut vals = [false];
                new.get_bool(&mut vals);
                let mut op = ApogeeCmd::new(company_id, VendorCmd::InClickless);
                op.put_enum(vals[0] as u32);
                avc.control(&AvcAddr::Unit, &mut op, TIMEOUT_MS)?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

pub struct DisplayCtl;

impl<'a> DisplayCtl {
    const TARGET_NAME: &'a str = "display-target";
    const FOLLOWED_NAME: &'a str = "meter-followed";
    const OVERHOLDS_NAME: &'a str = "overholds-duration";

    const TARGET_LABELS: &'a [&'a str] = &["output", "input"];
    const OVERHOLDS_LABELS: &'a [&'a str] = &["infinite", "2 sec"];

    pub fn new() -> Self {
        DisplayCtl{}
    }

    pub fn load(&mut self, _: &hinawa::FwFcp, card_cntr: &mut card_cntr::CardCntr)
        -> Result<(), Error>
    {
        // For target of display.
        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Card,
                                                   0, 0, Self::TARGET_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, Self::TARGET_LABELS, None, true)?;

        // For switch to force meters followed to selected item.
        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Card,
                                                   0, 0, Self::FOLLOWED_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        // For overholds duration.
        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Card,
                                                   0, 0, Self::OVERHOLDS_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, Self::OVERHOLDS_LABELS, None, true)?;

        Ok(())
    }

    pub fn read(&mut self, avc: &hinawa::FwFcp, company_id: &[u8;3], elem_id: &alsactl::ElemId,
            elem_value: &mut alsactl::ElemValue)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::TARGET_NAME => {
                let mut op = ApogeeCmd::new(company_id, VendorCmd::DisplayInput);
                avc.status(&AvcAddr::Unit, &mut op, TIMEOUT_MS)?;
                elem_value.set_enum(&[op.get_enum()]);
                Ok(true)
            }
            Self::FOLLOWED_NAME => {
                let mut op = ApogeeCmd::new(company_id, VendorCmd::DisplayFollow);
                avc.status(&AvcAddr::Unit, &mut op, TIMEOUT_MS)?;
                elem_value.set_bool(&[op.get_enum() > 0]);
                Ok(true)
            }
            Self::OVERHOLDS_NAME => {
                let mut op = ApogeeCmd::new(company_id, VendorCmd::DisplayOverhold);
                avc.status(&AvcAddr::Unit, &mut op, TIMEOUT_MS)?;
                elem_value.set_enum(&[op.get_enum()]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub fn write(&mut self, avc: &hinawa::FwFcp, company_id: &[u8;3], elem_id: &alsactl::ElemId,
                 _: &alsactl::ElemValue, new: &alsactl::ElemValue)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::TARGET_NAME => {
                let mut vals = [0];
                new.get_enum(&mut vals);
                let mut op = ApogeeCmd::new(company_id, VendorCmd::DisplayInput);
                op.put_enum(vals[0]);
                avc.control(&AvcAddr::Unit, &mut op, TIMEOUT_MS)?;
                //avc.write_bool(company_id, VendorCmd::DisplayInput, vals[0] > 0)?;
                Ok(true)
            }
            Self::FOLLOWED_NAME => {
                let mut vals = [false];
                new.get_bool(&mut vals);
                let mut op = ApogeeCmd::new(company_id, VendorCmd::DisplayFollow);
                op.put_enum(vals[0] as u32);
                avc.control(&AvcAddr::Unit, &mut op, TIMEOUT_MS)?;
                Ok(true)
            }
            Self::OVERHOLDS_NAME => {
                let mut vals = [0];
                new.get_enum(&mut vals);
                let mut op = ApogeeCmd::new(company_id, VendorCmd::DisplayOverhold);
                op.put_enum(vals[0]);
                avc.control(&AvcAddr::Unit, &mut op, TIMEOUT_MS)?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
