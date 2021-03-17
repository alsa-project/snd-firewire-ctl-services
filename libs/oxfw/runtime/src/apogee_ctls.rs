// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use core::card_cntr;
use core::elem_value_accessor::ElemValueAccessor;

use ta1394::{AvcAddr, Ta1394Avc};

use super::apogee_proto::{VendorCmd, ApogeeCmd, ApogeeMeterProtocol};

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
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let mut op = ApogeeCmd::new(company_id, VendorCmd::UseMixerOut);
                    avc.status(&AvcAddr::Unit, &mut op, TIMEOUT_MS)?;
                    Ok(op.get_enum())
                })?;
                Ok(true)
            }
            Self::LEVEL_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let mut op = ApogeeCmd::new(company_id, VendorCmd::OutAttr);
                    avc.status(&AvcAddr::Unit, &mut op, TIMEOUT_MS)?;
                    Ok(op.get_enum())
                })?;
                Ok(true)
            }
            Self::MUTE_FOR_LINE_OUT => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let mut mute_op = ApogeeCmd::new(company_id, VendorCmd::MuteForLineOut);
                    avc.status(&AvcAddr::Unit, &mut mute_op, TIMEOUT_MS)?;

                    let mut unmute_op = ApogeeCmd::new(company_id, VendorCmd::UnmuteForLineOut);
                    avc.status(&AvcAddr::Unit, &mut unmute_op, TIMEOUT_MS)?;

                    Ok(Self::parse_mute_mode(mute_op.get_enum() > 0, unmute_op.get_enum() > 0))
                })?;
                Ok(true)
            }
            Self::MUTE_FOR_HP_OUT => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let mut mute_op = ApogeeCmd::new(company_id, VendorCmd::MuteForHpOut);
                    avc.status(&AvcAddr::Unit, &mut mute_op, TIMEOUT_MS)?;
                    let mut unmute_op = ApogeeCmd::new(company_id, VendorCmd::UnmuteForHpOut);
                    avc.status(&AvcAddr::Unit, &mut unmute_op, TIMEOUT_MS)?;

                    Ok(Self::parse_mute_mode(mute_op.get_enum() > 0, unmute_op.get_enum() > 0))
                })?;
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
                ElemValueAccessor::<u32>::get_val(new, |val| {
                    let mut op = ApogeeCmd::new(company_id, VendorCmd::UseMixerOut);
                    op.put_enum(val);
                    avc.control(&AvcAddr::Unit, &mut op, TIMEOUT_MS)
                })?;
                Ok(true)
            }
            Self::LEVEL_NAME => {
                ElemValueAccessor::<u32>::get_val(new, |val| {
                    let mut op = ApogeeCmd::new(company_id, VendorCmd::OutAttr);
                    op.put_enum(val);
                    avc.control(&AvcAddr::Unit, &mut op, TIMEOUT_MS)
                })?;
                Ok(true)
            }
            Self::MUTE_FOR_LINE_OUT => {
                ElemValueAccessor::<u32>::get_val(new, |val| {
                    let (mute_mode, unmute_mode) = Self::build_mute_mode(val);

                    let mut op = ApogeeCmd::new(company_id, VendorCmd::MuteForLineOut);
                    op.put_enum(mute_mode as u32);
                    avc.control(&AvcAddr::Unit, &mut op, TIMEOUT_MS)?;

                    let mut op = ApogeeCmd::new(company_id, VendorCmd::UnmuteForLineOut);
                    op.put_enum(unmute_mode as u32);
                    avc.control(&AvcAddr::Unit, &mut op, TIMEOUT_MS)
                })?;
                Ok(true)
            }
            Self::MUTE_FOR_HP_OUT => {
                ElemValueAccessor::<u32>::get_val(new, |val| {
                    let (mute_mode, unmute_mode) = Self::build_mute_mode(val);

                    let mut op = ApogeeCmd::new(company_id, VendorCmd::MuteForHpOut);
                    op.put_enum(mute_mode as u32);
                    avc.control(&AvcAddr::Unit, &mut op, TIMEOUT_MS)?;

                    let mut op = ApogeeCmd::new(company_id, VendorCmd::UnmuteForHpOut);
                    op.put_enum(unmute_mode as u32);
                    avc.control(&AvcAddr::Unit, &mut op, TIMEOUT_MS)
                })?;
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
                ElemValueAccessor::<i32>::set_vals(elem_value, Self::SRC_LABELS.len(), |src| {
                    let mut op = ApogeeCmd::new(company_id, VendorCmd::MixerSrc(src as u8, dst as u8));
                    avc.status(&AvcAddr::Unit, &mut op, TIMEOUT_MS)?;
                    Ok(op.read_u16() as i32)
                })?;
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
                ElemValueAccessor::<i32>::get_vals(new, old, 4, |src, val| {
                    let mut op = ApogeeCmd::new(company_id, VendorCmd::MixerSrc(src as u8, dst as u8));
                    op.write_u16(val as u16);
                    avc.control(&AvcAddr::Unit, &mut op, TIMEOUT_MS)
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
                ElemValueAccessor::<bool>::set_vals(elem_value, 2, |idx| {
                    let mut op = ApogeeCmd::new(company_id, VendorCmd::MicPolarity(idx as u8));
                    avc.status(&AvcAddr::Unit, &mut op, TIMEOUT_MS)?;
                    Ok(op.get_enum() > 0)
                })?;
                Ok(true)
            }
            Self::PHONE_LEVEL_NAME => {
                ElemValueAccessor::<u32>::set_vals(elem_value, 2, |idx| {
                    let mut op = ApogeeCmd::new(company_id, VendorCmd::PhoneInLine(idx as u8));
                    avc.status(&AvcAddr::Unit, &mut op, TIMEOUT_MS)?;
                    Ok(op.get_enum())
                })?;
                Ok(true)
            }
            Self::LINE_LEVEL_NAME => {
                ElemValueAccessor::<u32>::set_vals(elem_value, 2, |idx| {
                    let mut op = ApogeeCmd::new(company_id, VendorCmd::LineInLevel(idx as u8));
                    avc.status(&AvcAddr::Unit, &mut op, TIMEOUT_MS)?;
                    Ok(op.get_enum())
                })?;
                Ok(true)
            }
            Self::PHANTOM_NAME => {
                ElemValueAccessor::<bool>::set_vals(elem_value, 2, |idx| {
                    let mut op = ApogeeCmd::new(company_id, VendorCmd::MicPhantom(idx as u8));
                    avc.status(&AvcAddr::Unit, &mut op, TIMEOUT_MS)?;
                    Ok(op.get_enum() > 0)
                })?;
                Ok(true)
            }
            Self::SRC_NAME => {
                ElemValueAccessor::<u32>::set_vals(elem_value, 2, |idx| {
                    let mut op = ApogeeCmd::new(company_id, VendorCmd::MicIn(idx as u8));
                    avc.status(&AvcAddr::Unit, &mut op, TIMEOUT_MS)?;
                    Ok(op.get_enum())
                })?;
                Ok(true)
            }
            Self::CLICKLESS_NAME => {
                ElemValueAccessor::<bool>::set_val(elem_value, || {
                    let mut op = ApogeeCmd::new(company_id, VendorCmd::InClickless);
                    avc.status(&AvcAddr::Unit, &mut op, TIMEOUT_MS)?;
                    Ok(op.get_enum() > 0)
                })?;
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
                ElemValueAccessor::<bool>::get_vals(new, old, 2, |idx, val| {
                    let mut op = ApogeeCmd::new(company_id, VendorCmd::MicPolarity(idx as u8));
                    op.put_enum(val as u32);
                    avc.control(&AvcAddr::Unit, &mut op, TIMEOUT_MS)
                })?;
                Ok(true)
            }
            Self::PHONE_LEVEL_NAME => {
                ElemValueAccessor::<u32>::get_vals(new, old, 2, |idx, val| {
                    let mut op = ApogeeCmd::new(company_id, VendorCmd::PhoneInLine(idx as u8));
                    op.put_enum(val);
                    avc.control(&AvcAddr::Unit, &mut op, TIMEOUT_MS)
                })?;
                Ok(true)
            }
            Self::LINE_LEVEL_NAME => {
                ElemValueAccessor::<u32>::get_vals(new, old, 2, |idx, val| {
                    let mut op = ApogeeCmd::new(company_id, VendorCmd::LineInLevel(idx as u8));
                    op.put_enum(val);
                    avc.control(&AvcAddr::Unit, &mut op, TIMEOUT_MS)
                })?;
                Ok(true)
            }
            Self::PHANTOM_NAME => {
                ElemValueAccessor::<bool>::get_vals(new, old, 2, |idx, val| {
                    let mut op = ApogeeCmd::new(company_id, VendorCmd::MicPhantom(idx as u8));
                    op.put_enum(val as u32);
                    avc.control(&AvcAddr::Unit, &mut op, TIMEOUT_MS)
                })?;
                Ok(true)
            }
            Self::SRC_NAME => {
                ElemValueAccessor::<u32>::get_vals(new, old, 2, |idx, val| {
                    let mut op = ApogeeCmd::new(company_id, VendorCmd::MicIn(idx as u8));
                    op.put_enum(val);
                    avc.control(&AvcAddr::Unit, &mut op, TIMEOUT_MS)
                })?;
                Ok(true)
            }
            Self::CLICKLESS_NAME => {
                ElemValueAccessor::<bool>::get_val(new, |val| {
                    let mut op = ApogeeCmd::new(company_id, VendorCmd::InClickless);
                    op.put_enum(val as u32);
                    avc.control(&AvcAddr::Unit, &mut op, TIMEOUT_MS)
                })?;
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
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let mut op = ApogeeCmd::new(company_id, VendorCmd::DisplayInput);
                    avc.status(&AvcAddr::Unit, &mut op, TIMEOUT_MS)?;
                    Ok(op.get_enum())
                })?;
                Ok(true)
            }
            Self::FOLLOWED_NAME => {
                ElemValueAccessor::<bool>::set_val(elem_value, || {
                    let mut op = ApogeeCmd::new(company_id, VendorCmd::DisplayFollow);
                    avc.status(&AvcAddr::Unit, &mut op, TIMEOUT_MS)?;
                    Ok(op.get_enum() > 0)
                })?;
                Ok(true)
            }
            Self::OVERHOLDS_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let mut op = ApogeeCmd::new(company_id, VendorCmd::DisplayOverhold);
                    avc.status(&AvcAddr::Unit, &mut op, TIMEOUT_MS)?;
                    Ok(op.get_enum())
                })?;
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
                ElemValueAccessor::<u32>::get_val(new, |val| {
                    let mut op = ApogeeCmd::new(company_id, VendorCmd::DisplayInput);
                    op.put_enum(val);
                    avc.control(&AvcAddr::Unit, &mut op, TIMEOUT_MS)
                })?;
                Ok(true)
            }
            Self::FOLLOWED_NAME => {
                ElemValueAccessor::<bool>::get_val(new, |val| {
                    let mut op = ApogeeCmd::new(company_id, VendorCmd::DisplayFollow);
                    op.put_enum(val as u32);
                    avc.control(&AvcAddr::Unit, &mut op, TIMEOUT_MS)
                })?;
                Ok(true)
            }
            Self::OVERHOLDS_NAME => {
                ElemValueAccessor::<u32>::get_val(new, |val| {
                    let mut op = ApogeeCmd::new(company_id, VendorCmd::DisplayOverhold);
                    op.put_enum(val);
                    avc.control(&AvcAddr::Unit, &mut op, TIMEOUT_MS)
                })?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

pub struct HwState {
    pub measure_elems: Vec<alsactl::ElemId>,

    req: hinawa::FwReq,
    meters: [i32;6],
    states: [u8;8],
}

impl<'a> HwState {
    const OUT_MUTE_NAME: &'a str = "output-mute";
    const SELECTED_KNOB_NAME: &'a str = "selected-knob";
    const OUT_VOLUME_NAME: &'a str = "output-volume";
    const IN_GAIN_NAME: &'a str = "input-gain";

    const ANALOG_IN_METER_NAME: &'a str = "analog-input-meters";
    const MIXER_SRC_METER_NAME: &'a str = "mixer-source-meters";
    const MIXER_OUT_METER_NAME: &'a str = "mixer-output-meters";

    const KNOB_LABELS: &'a [&'a str] = &["Out", "In-1", "In-2"];

    const INPUT_LABELS: &'a [&'a str] = &[
        "analog-input-1",
        "analog-input-2",
    ];

    const DIAL_OUT_MIN: i32 = 0;
    const DIAL_OUT_MAX: i32 = 75;
    const DIAL_OUT_STEP: i32 = 1;

    const DIAL_IN_MIN: i32 = 10;
    const DIAL_IN_MAX: i32 = 75;
    const DIAL_IN_STEP: i32 = 1;

    const METER_MIN: i32 = 0;
    const METER_MAX: i32 = i32::MAX;
    const METER_STEP: i32 = 256;

    pub fn new() -> Self {
        HwState{
            measure_elems: Vec::new(),
            req: hinawa::FwReq::new(),
            meters: [0;6],
            states: [0;8],
        }
    }

    pub fn load(&mut self, _: &hinawa::FwFcp, card_cntr: &mut card_cntr::CardCntr)
        -> Result<(), Error>
    {
        // For mute of analog outputs.
        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Mixer,
                                                   0, 0, Self::OUT_MUTE_NAME, 0);
        let elem_id_list = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;
        self.measure_elems.extend_from_slice(&elem_id_list);

        // For selection of knob.
        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Card,
                                                   0, 0, Self::SELECTED_KNOB_NAME, 0);
        let elem_id_list = card_cntr.add_enum_elems(&elem_id, 1, 1, Self::KNOB_LABELS,
                                                    None, false)?;
        self.measure_elems.extend_from_slice(&elem_id_list);

        // For output volume.
        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Mixer,
                                                   0, 0, Self::OUT_VOLUME_NAME, 0);
        let elem_id_list = card_cntr.add_int_elems(&elem_id, 1,
                                    Self::DIAL_OUT_MIN, Self::DIAL_OUT_MAX, Self::DIAL_OUT_STEP,
                                    1, None, true)?;
        self.measure_elems.extend_from_slice(&elem_id_list);

        // For input gain.
        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Mixer,
                                                   0, 0, Self::IN_GAIN_NAME, 0);
        let elem_id_list = card_cntr.add_int_elems(&elem_id, 1,
                                    Self::DIAL_IN_MIN, Self::DIAL_IN_MAX, Self::DIAL_IN_STEP,
                                    Self::INPUT_LABELS.len(), None, true)?;
        self.measure_elems.extend_from_slice(&elem_id_list);

        // For meter of inputs.
        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Mixer,
                                                   0, 0, Self::ANALOG_IN_METER_NAME, 0);
        let elem_id_list = card_cntr.add_int_elems(&elem_id, 1,
                                                   Self::METER_MIN, Self::METER_MAX, Self::METER_STEP,
                                                   Self::INPUT_LABELS.len(), None, false)?;
        self.measure_elems.extend_from_slice(&elem_id_list);

        // For meters of mixer sources.
        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Mixer,
                                                   0, 0, Self::MIXER_SRC_METER_NAME, 0);
        let elem_id_list = card_cntr.add_int_elems(&elem_id, 1,
                                                   Self::METER_MIN, Self::METER_MAX, Self::METER_STEP,
                                                   MixerCtl::SRC_LABELS.len(), None, false)?;
        self.measure_elems.extend_from_slice(&elem_id_list);

        // For meters of mixer sources.
        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Mixer,
                                                   0, 0, Self::MIXER_OUT_METER_NAME, 0);
        let elem_id_list = card_cntr.add_int_elems(&elem_id, 1,
                                                   Self::METER_MIN, Self::METER_MAX, Self::METER_STEP,
                                                   MixerCtl::TARGET_LABELS.len(), None, false)?;
        self.measure_elems.extend_from_slice(&elem_id_list);

        Ok(())
    }

    pub fn read(&mut self, avc: &hinawa::FwFcp, company_id: &[u8;3], elem_id: &alsactl::ElemId,
            elem_value: &mut alsactl::ElemValue)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::OUT_MUTE_NAME => {
                ElemValueAccessor::<bool>::set_val(elem_value, || {
                    let mut op = ApogeeCmd::new(company_id, VendorCmd::OutMute);
                    avc.status(&AvcAddr::Unit, &mut op, TIMEOUT_MS)?;
                    Ok(op.get_enum() > 0)
                })?;
                Ok(true)
            }
            Self::OUT_VOLUME_NAME => {
                ElemValueAccessor::<i32>::set_val(elem_value, || {
                    let mut op = ApogeeCmd::new(company_id, VendorCmd::OutVolume);
                    avc.status(&AvcAddr::Unit, &mut op, TIMEOUT_MS)?;
                    Ok(Self::DIAL_OUT_MAX - (op.get_u8() as i32))
                })?;
                Ok(true)
            }
            Self::IN_GAIN_NAME => {
                ElemValueAccessor::<i32>::set_vals(elem_value, 2, |idx| {
                    let mut op = ApogeeCmd::new(company_id, VendorCmd::InGain(idx as u8));
                    avc.status(&AvcAddr::Unit, &mut op, TIMEOUT_MS)?;
                    Ok(op.get_u8() as i32)
                })?;
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
            Self::OUT_MUTE_NAME => {
                ElemValueAccessor::<bool>::get_val(new, |val| {
                    let mut op = ApogeeCmd::new(company_id, VendorCmd::OutMute);
                    op.put_enum(val as u32);
                    avc.control(&AvcAddr::Unit, &mut op, TIMEOUT_MS)
                })?;
                Ok(true)
            }
            Self::OUT_VOLUME_NAME => {
                ElemValueAccessor::<i32>::get_val(new, |val| {
                    let mut op = ApogeeCmd::new(company_id, VendorCmd::OutVolume);
                    op.put_u8((Self::DIAL_OUT_MAX - val) as u8);
                    avc.control(&AvcAddr::Unit, &mut op, TIMEOUT_MS)
                })?;
                Ok(true)
            }
            Self::IN_GAIN_NAME => {
                ElemValueAccessor::<i32>::get_vals(new, old, 2, |idx, val| {
                    let mut op = ApogeeCmd::new(company_id, VendorCmd::InGain(idx as u8));
                    op.put_u8(val as u8);
                    avc.control(&AvcAddr::Unit, &mut op, TIMEOUT_MS)
                })?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub fn measure_states(&mut self, node: &hinawa::FwNode, avc: &hinawa::FwFcp, company_id: &[u8;3])
        -> Result<(), Error>
    {
        let mut meters = [0;6];
        self.req.read_meters(node, &mut meters)?;
        self.meters.iter_mut().zip(meters.iter()).for_each(|(d, s)| *d = *s as i32);

        let mut op = ApogeeCmd::new(company_id, VendorCmd::HwState);
        avc.status(&AvcAddr::Unit, &mut op, TIMEOUT_MS)?;
        op.copy_block(&mut self.states);

        Ok(())
    }

    pub fn measure_elems(&mut self, elem_id: &alsactl::ElemId, elem_value: &mut alsactl::ElemValue)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::OUT_MUTE_NAME => {
                ElemValueAccessor::<bool>::set_val(elem_value, || Ok(self.states[0] > 0))?;
                Ok(true)
            }
            Self::SELECTED_KNOB_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || Ok(self.states[1] as u32))?;
                Ok(true)
            }
            Self::OUT_VOLUME_NAME => {
                ElemValueAccessor::<i32>::set_val(elem_value, || Ok(self.states[3] as i32))?;
                Ok(true)
            }
            Self::IN_GAIN_NAME => {
                ElemValueAccessor::<i32>::set_vals(elem_value, 2, |idx| Ok(self.states[4 + idx] as i32))?;
                Ok(true)
            }
            Self::ANALOG_IN_METER_NAME => {
                ElemValueAccessor::<i32>::set_vals(elem_value, 2, |idx| Ok(self.meters[idx]))?;
                Ok(true)
            }
            Self::MIXER_SRC_METER_NAME => {
                ElemValueAccessor::<i32>::set_vals(elem_value, 2, |idx| Ok(self.meters[idx + 2]))?;
                Ok(true)
            }
            Self::MIXER_OUT_METER_NAME => {
                ElemValueAccessor::<i32>::set_val(elem_value, || Ok(self.meters[4]))?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
