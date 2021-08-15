// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use hinawa::FwFcp;
use alsactl::{ElemId, ElemIfaceType, ElemValue};

use core::card_cntr::*;
use core::elem_value_accessor::ElemValueAccessor;

use ta1394::{AvcAddr, Ta1394Avc};

use oxfw_protocols::apogee::{VendorCmd, ApogeeCmd};

const TIMEOUT_MS: u32 = 100;

#[derive(Default, Debug)]
pub struct MixerCtl;

impl MixerCtl {
    const TARGET_LABELS: [&'static str; 2] = ["mixer-1", "mixer-2"];

    const SRC_LABELS: [&'static str; 4] = ["stream-1", "stream-2", "analog-1", "analog-2"];

    const MIXER_NAME: &'static str = "mixer-source-gain";

    const GAIN_MIN: i32 = 0;
    const GAIN_MAX: i32 = 0x3fff;
    const GAIN_STEP: i32 = 0xff;

    pub fn load(&mut self, _: &FwFcp, card_cntr: &mut CardCntr) -> Result<(), Error> {
        // For gain of mixer sources.
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::MIXER_NAME, 0);
        let _ = card_cntr.add_int_elems(&elem_id, Self::TARGET_LABELS.len(),
                                        Self::GAIN_MIN, Self::GAIN_MAX, Self::GAIN_STEP,
                                        Self::SRC_LABELS.len(),
                                        None, true)?;

        Ok(())
    }

    pub fn read(&mut self, avc: &FwFcp, company_id: &[u8;3], elem_id: &ElemId, elem_value: &mut ElemValue)
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

    pub fn write(&mut self, avc: &FwFcp, company_id: &[u8;3], elem_id: &ElemId, old: &ElemValue,
                 new: &ElemValue)
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

#[derive(Default, Debug)]
pub struct InputCtl;

impl InputCtl {
    const SRC_NAME: &'static str = "input-source";
    const PHONE_LEVEL_NAME: &'static str = "input-phone-level";
    const LINE_LEVEL_NAME: &'static str = "input-line-level";
    const POLARITY_NAME: &'static str = "mic-polarity";
    const PHANTOM_NAME: &'static str = "mic-phantom";
    const CLICKLESS_NAME: &'static str = "input-clickless";

    const TARGET_LABELS: [&'static str; 2] = ["Analog-in-1", "Analog-in-2"];
    const MIC_LABELS: [&'static str; 2] = ["Mic-1", "Mic-2"];
    const PHONE_LABELS: [&'static str; 2] = ["Phone-1", "Phone-2"];
    const SRC_LABELS: [&'static str; 2] = ["Mic", "Phone"];
    const PHONE_LEVEL_LABELS: [&'static str; 2] = ["Instrument", "Line"];
    const LINE_LEVEL_LABELS: [&'static str; 2] = ["+4dB", "-10dB"];

    pub fn load(&mut self, _: &FwFcp, card_cntr: &mut CardCntr) -> Result<(), Error> {
        // For polarity of microphone.
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::POLARITY_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, Self::MIC_LABELS.len(), true)?;

        // For level of input in phone jack.
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::PHONE_LEVEL_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1,
                                         Self::PHONE_LABELS.len(), &Self::PHONE_LEVEL_LABELS,
                                         None, true)?;

        // For level of line input in phone jack.
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::LINE_LEVEL_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1,
                                         Self::PHONE_LABELS.len(), &Self::LINE_LEVEL_LABELS,
                                         None, true)?;

        // For phantom powering of microphone.
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::PHANTOM_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, Self::MIC_LABELS.len(), true)?;

        // For source of analog inputs.
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::SRC_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, Self::TARGET_LABELS.len(),
                                         &Self::SRC_LABELS, None, true)?;

        // For input clickless.
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::CLICKLESS_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        Ok(())
    }

    pub fn read(&mut self, avc: &FwFcp, company_id: &[u8;3], elem_id: &ElemId, elem_value: &mut ElemValue)
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

    pub fn write(&mut self, avc: &FwFcp, company_id: &[u8;3], elem_id: &ElemId, old: &ElemValue,
                 new: &ElemValue)
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

#[derive(Default, Debug)]
pub struct DisplayCtl;

impl DisplayCtl {
    const TARGET_NAME: &'static str = "display-target";
    const FOLLOWED_NAME: &'static str = "meter-followed";
    const OVERHOLDS_NAME: &'static str = "overholds-duration";

    const TARGET_LABELS: [&'static str; 2] = ["output", "input"];
    const OVERHOLDS_LABELS: [&'static str; 2] = ["infinite", "2 sec"];

    pub fn load(&mut self, _: &FwFcp, card_cntr: &mut CardCntr) -> Result<(), Error> {
        // For target of display.
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::TARGET_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &Self::TARGET_LABELS, None, true)?;

        // For switch to force meters followed to selected item.
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::FOLLOWED_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        // For overholds duration.
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::OVERHOLDS_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &Self::OVERHOLDS_LABELS, None, true)?;

        Ok(())
    }

    pub fn read(&mut self, avc: &FwFcp, company_id: &[u8;3], elem_id: &ElemId, elem_value: &mut ElemValue)
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

    pub fn write(&mut self, avc: &FwFcp, company_id: &[u8;3], elem_id: &ElemId, _: &ElemValue,
                 new: &ElemValue)
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

#[derive(Default, Debug)]
pub struct HwState {
    pub measure_elems: Vec<ElemId>,

    pub states: [u8;8],
}

impl HwState {
    const IN_GAIN_NAME: &'static str = "input-gain";

    const INPUT_LABELS: [&'static str; 2] = [
        "analog-input-1",
        "analog-input-2",
    ];

    const DIAL_IN_MIN: i32 = 10;
    const DIAL_IN_MAX: i32 = 75;
    const DIAL_IN_STEP: i32 = 1;

    pub fn load(&mut self, _: &FwFcp, card_cntr: &mut CardCntr) -> Result<(), Error> {
        // For input gain.
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::IN_GAIN_NAME, 0);
        let elem_id_list = card_cntr.add_int_elems(&elem_id, 1,
                                    Self::DIAL_IN_MIN, Self::DIAL_IN_MAX, Self::DIAL_IN_STEP,
                                    Self::INPUT_LABELS.len(), None, true)?;
        self.measure_elems.extend_from_slice(&elem_id_list);

        Ok(())
    }

    pub fn read(&mut self, avc: &FwFcp, company_id: &[u8;3], elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
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

    pub fn write(&mut self, avc: &FwFcp, company_id: &[u8;3], elem_id: &ElemId, old: &ElemValue,
                 new: &ElemValue)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
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

    pub fn measure_elems(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::IN_GAIN_NAME => {
                ElemValueAccessor::<i32>::set_vals(elem_value, 2, |idx| Ok(self.states[4 + idx] as i32))?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
