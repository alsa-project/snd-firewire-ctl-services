// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::{Error, FileError};

use hinawa::SndUnitExt;

use alsactl::{ElemValueExt, ElemValueExtManual};

use alsa_ctl_tlv_codec::items::DbInterval;

use core::card_cntr;
use core::elem_value_accessor::ElemValueAccessor;

use ta1394::{AvcAddr, Ta1394Avc};

use super::super::BebobAvc;
use super::super::extensions::{BcoPlugAddr, BcoPlugDirection, BcoPlugAddrUnitType};
use super::super::extensions::BcoCompoundAm824StreamFormat;
use super::super::extensions::ExtendedStreamFormatSingle;
use super::super::model::{HP_SRC_NAME, OUT_SRC_NAME, OUT_VOL_NAME, IN_METER_NAME, OUT_METER_NAME};
use super::apogee_proto::{ApogeeCmd, VendorCmd, HwCmdOp};

pub struct HwCtl{
    stream: u32,
    cd: bool,
    spdif_out_bypass: u32,
}

impl<'a> HwCtl {
    const STREAM_MODE_NAME: &'a str = "stream-mode";
    const CD_MODE_NAME: &'a str = "cd-mode";
    const SPDIF_OUT_BYPASS_NAME: &'a str = "S/PDIF-out-bypass";

    const STREAM_MODE_LABELS: &'a [&'a str] = &["16x16", "10x10", "8x8"];

    const SPDIF_OUT_BYPASS_LABELS: &'a [&'a str] = &[
        "none",
        "analog-in-1/2",
        "analog-in-3/4",
        "analog-in-5/6",
        "analog-in-7/8",
        "spdif-opt-in-1/2",
        "spdif-coax-in-1/2",
        "spdif-coax-out-1/2",
        "spdif-opt-out-1/2",
    ];

    pub fn new() -> Self {
        HwCtl {
            stream: 0,
            cd: false,
            spdif_out_bypass: 0,
        }
    }

    pub fn load(&mut self, avc: &BebobAvc, card_cntr: &mut card_cntr::CardCntr, timeout_ms: u32)
        -> Result<(), Error>
    {
        let plug_addr = BcoPlugAddr::new_for_unit(BcoPlugDirection::Output, BcoPlugAddrUnitType::Isoc,
                                                  0);
        let mut op = ExtendedStreamFormatSingle::new(&plug_addr);
        avc.status(&AvcAddr::Unit, &mut op, timeout_ms)?;
        let info = op.stream_format.as_bco_compound_am824_stream()?;
        let count = info.entries.iter()
            .filter(|entry| entry.format == BcoCompoundAm824StreamFormat::MultiBitLinearAudioRaw)
            .fold(0, |count, entry| count + entry.count as usize);
        self.stream = match count {
            18 => 0,
            10 => 1,
            _ => 2,
        };

        // Transfer initialized data.
        let mut op = ApogeeCmd::new(VendorCmd::Hw(HwCmdOp::CdMode),
                                    &[self.cd as u8]);
        avc.control(&AvcAddr::Unit, &mut op, timeout_ms)?;

        let mut op = ApogeeCmd::new(VendorCmd::Downgrade, &[self.cd as u8]);
        avc.control(&AvcAddr::Unit, &mut op, timeout_ms)?;

        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Card,
                                                   0, 0, Self::STREAM_MODE_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, Self::STREAM_MODE_LABELS, None, true)?;

        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Card,
                                                   0, 0, Self::CD_MODE_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Card,
                                                   0, 0, Self::SPDIF_OUT_BYPASS_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, Self::SPDIF_OUT_BYPASS_LABELS, None, true)?;

        Ok(())
    }

    pub fn read(&mut self, elem_id: &alsactl::ElemId, elem_value: &mut alsactl::ElemValue)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::STREAM_MODE_NAME => {
                ElemValueAccessor::set_val(elem_value, || Ok(self.stream))?;
                Ok(true)
            }
            Self::CD_MODE_NAME => {
                ElemValueAccessor::set_val(elem_value, || Ok(self.cd))?;
                Ok(true)
            }
            Self::SPDIF_OUT_BYPASS_NAME => {
                ElemValueAccessor::set_val(elem_value, || Ok(self.spdif_out_bypass))?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub fn write(&mut self, unit: &hinawa::SndUnit, avc: &BebobAvc, elem_id: &alsactl::ElemId,
                 _: &alsactl::ElemValue, new: &alsactl::ElemValue, timeout_ms: u32)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::STREAM_MODE_NAME => {
                ElemValueAccessor::<u32>::get_val(new, |val| {
                    let mut op = ApogeeCmd::new(VendorCmd::Hw(HwCmdOp::StreamMode),
                                                &[val as u8]);
                    unit.lock()?;
                    let res = avc.control(&AvcAddr::Unit, &mut op, timeout_ms);
                    unit.unlock()?;
                    res
                })?;
                Ok(true)
            }
            Self::CD_MODE_NAME => {
                ElemValueAccessor::<bool>::get_val(new, |val| {
                    let mut op = ApogeeCmd::new(VendorCmd::Hw(HwCmdOp::CdMode),
                                                &[val as u8]);
                    avc.control(&AvcAddr::Unit, &mut op, timeout_ms)?;
                    self.cd = val;
                    Ok(())
                })?;
                Ok(true)
            }
            Self::SPDIF_OUT_BYPASS_NAME => {
                ElemValueAccessor::<u32>::get_val(new, |val| {
                    let mut op = ApogeeCmd::new(VendorCmd::Downgrade,
                                                &[val as u8]);
                    avc.control(&AvcAddr::Unit, &mut op, timeout_ms)?;
                    self.spdif_out_bypass = val as u32;
                    Ok(())
                })?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

pub struct DisplayCtl{
    illuminate: bool,
    mode: bool,
    target: u32,
    overhold: bool,
}

impl<'a> DisplayCtl {
    const ILLUMINATE_NAME: &'a str = "display-illuminate";
    const MODE_NAME: &'a str = "display-mode";
    const TARGET_NAME: &'a str = "display-target";
    const OVERHOLD_NAME: &'a str = "display-overhold";

    const TARGET_LABELS: &'a [&'a str] = &["output", "input"];

    pub fn new() -> Self {
        DisplayCtl {
            illuminate: false,
            mode: false,
            target: 0,
            overhold: false,
        }
    }

    pub fn load(&mut self, avc: &BebobAvc, card_cntr: &mut card_cntr::CardCntr, timeout_ms: u32)
        -> Result<(), Error>
    {
        // Transfer initialized data.
        let mut op = ApogeeCmd::new(VendorCmd::Hw(HwCmdOp::DisplayIlluminate),
                                    &[self.illuminate as u8]);
        avc.control(&AvcAddr::Unit, &mut op, timeout_ms)?;

        let mut op = ApogeeCmd::new(VendorCmd::Hw(HwCmdOp::DisplayMode),
                                    &[self.mode as u8]);
        avc.control(&AvcAddr::Unit, &mut op, timeout_ms)?;

        let mut op = ApogeeCmd::new(VendorCmd::Hw(HwCmdOp::DisplayTarget),
                                    &[self.target as u8]);
        avc.control(&AvcAddr::Unit, &mut op, timeout_ms)?;

        let mut op = ApogeeCmd::new(VendorCmd::Hw(HwCmdOp::DisplayOverhold),
                                    &[self.overhold as u8]);
        avc.control(&AvcAddr::Unit, &mut op, timeout_ms)?;

        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Card,
                                                   0, 0, Self::ILLUMINATE_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Card,
                                                   0, 0, Self::MODE_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Card,
                                                   0, 0, Self::TARGET_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, Self::TARGET_LABELS, None, true)?;

        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Card,
                                                   0, 0, Self::OVERHOLD_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        Ok(())
    }

    pub fn read(&mut self, elem_id: &alsactl::ElemId, elem_value: &mut alsactl::ElemValue)
                -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::ILLUMINATE_NAME => {
                ElemValueAccessor::<bool>::set_val(elem_value, || Ok(self.illuminate))?;
                Ok(true)
            }
            Self::MODE_NAME => {
                ElemValueAccessor::<bool>::set_val(elem_value, || Ok(self.mode))?;
                Ok(true)
            }
            Self::TARGET_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || Ok(self.target))?;
                Ok(true)
            }
            Self::OVERHOLD_NAME => {
                ElemValueAccessor::<bool>::set_val(elem_value, || Ok(self.overhold))?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub fn write(&mut self, avc: &BebobAvc, elem_id: &alsactl::ElemId,
                 _: &alsactl::ElemValue, new: &alsactl::ElemValue, timeout_ms: u32)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::ILLUMINATE_NAME => {
                ElemValueAccessor::<bool>::get_val(new, |val| {
                    let mut op = ApogeeCmd::new(VendorCmd::Hw(HwCmdOp::DisplayIlluminate),
                                                &[val as u8]);
                    avc.control(&AvcAddr::Unit, &mut op, timeout_ms)?;
                    self.illuminate = val;
                    Ok(())
                })?;
                Ok(true)
            }
            Self::MODE_NAME => {
                ElemValueAccessor::<bool>::get_val(new, |val| {
                    let mut op = ApogeeCmd::new(VendorCmd::Hw(HwCmdOp::DisplayMode),
                                                &[val as u8]);
                    avc.control(&AvcAddr::Unit, &mut op, timeout_ms)?;
                    self.mode = val;
                    Ok(())
                })?;
                Ok(true)
            }
            Self::TARGET_NAME => {
                ElemValueAccessor::<u32>::get_val(new, |val| {
                    let mut op = ApogeeCmd::new(VendorCmd::Hw(HwCmdOp::DisplayTarget),
                                                &[val as u8]);
                    avc.control(&AvcAddr::Unit, &mut op, timeout_ms)?;
                    self.target = val;
                    Ok(())
                })?;
                Ok(true)
            }
            Self::OVERHOLD_NAME => {
                ElemValueAccessor::<bool>::get_val(new, |val| {
                    let mut op = ApogeeCmd::new(VendorCmd::Hw(HwCmdOp::DisplayOverhold),
                                                &[val as u8]);
                    avc.control(&AvcAddr::Unit, &mut op, timeout_ms)?;
                    self.overhold = val;
                    Ok(())
                })?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

pub struct OpticalCtl{
    output: u32,
    input: u32,
}

impl<'a> OpticalCtl {
    const OUT_MODE_NAME: &'a str = "output-optical-mode";
    const IN_MODE_NAME: &'a str = "input-optical-mode";

    const MODE_LABELS: &'a [&'a str] = &["S/PDIF", "ADAT/SMUX"];

    pub fn new() -> Self {
        OpticalCtl {
            output: 0,
            input: 0,
        }
    }

    pub fn load(&mut self, avc: &BebobAvc, card_cntr: &mut card_cntr::CardCntr, timeout_ms: u32)
        -> Result<(), Error>
    {
        // Transfer initialized data.
        let mut op = ApogeeCmd::new(VendorCmd::OptIfaceMode(0x00), &[self.output as u8]);
        avc.control(&AvcAddr::Unit, &mut op, timeout_ms)?;

        let mut op = ApogeeCmd::new(VendorCmd::OptIfaceMode(0x01), &[self.input as u8]);
        avc.control(&AvcAddr::Unit, &mut op, timeout_ms)?;

        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Mixer,
                                                   0, 0, Self::OUT_MODE_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, Self::MODE_LABELS, None, true)?;

        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Mixer,
                                                   0, 0, Self::IN_MODE_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, Self::MODE_LABELS, None, true)?;

        Ok(())
    }

    pub fn read(&mut self, elem_id: &alsactl::ElemId, elem_value: &mut alsactl::ElemValue)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::OUT_MODE_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || Ok(self.output))?;
                Ok(true)
            }
            Self::IN_MODE_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || Ok(self.input))?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub fn write(&mut self, avc: &BebobAvc, elem_id: &alsactl::ElemId,
                 _: &alsactl::ElemValue, new: &alsactl::ElemValue, timeout_ms: u32)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::OUT_MODE_NAME => {
                ElemValueAccessor::<u32>::get_val(new, |val| {
                    let mut op = ApogeeCmd::new(VendorCmd::OptIfaceMode(0x00),
                                                &[val as u8]);
                    avc.control(&AvcAddr::Unit, &mut op, timeout_ms)?;
                    self.output = val;
                    Ok(())
                })?;
                Ok(true)
            }
            Self::IN_MODE_NAME => {
                ElemValueAccessor::<u32>::get_val(new, |val| {
                    let mut op = ApogeeCmd::new(VendorCmd::OptIfaceMode(0x01),
                                                &[val as u8]);
                    avc.control(&AvcAddr::Unit, &mut op, timeout_ms)?;
                    self.input = val;
                    Ok(())
                })?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

pub struct InputCtl{
    limits: [bool; 8],
    levels: [u32; 8],

    phantoms: [bool; 4],
    polarities: [bool; 4],
}

impl<'a> InputCtl {
    const IN_LIMIT_NAME: &'a str = "input-limit";
    const IN_LEVEL_NAME: &'a str = "input-level";
    const MIC_PHANTOM_NAME: &'a str = "mic-phantom";
    const MIC_POLARITY_NAME: &'a str = "mic-polarity";

    const IN_LABELS: &'a [&'a str] = &[
        "analog-1", "analog-2", "analog-3", "analog-4", "analog-5", "analog-6", "analog-7",
        "analog-8",
    ];

    const IN_LEVEL_LABELS: &'a [&'a str] = &["+4dB", "-10dB", "Mic"];

    const MIC_LABELS: &'a [&'a str] = &["mci-1", "mic-2", "mic-3", "mic-4"];

    pub fn new() -> Self {
        InputCtl {
            limits: [false;8],
            levels: [0;8],
            phantoms: [false;4],
            polarities: [false;4],
        }
    }

    pub fn load(&mut self, avc: &BebobAvc, card_cntr: &mut card_cntr::CardCntr, timeout_ms: u32)
        -> Result<(), Error>
    {
        // Transfer initialized data.
        (0..Self::IN_LABELS.len()).try_for_each(|i| {
            let mut op = ApogeeCmd::new(VendorCmd::InputLimit(i as u8),
                                        &[self.limits[i] as u8]);
            avc.control(&AvcAddr::Unit, &mut op, timeout_ms)?;

            let mut op = ApogeeCmd::new(VendorCmd::IoAttr(i as u8, 0x01),
                                        &[self.levels[i] as u8]);
            avc.control(&AvcAddr::Unit, &mut op, timeout_ms)?;

            Ok(())
        })?;

        (0..Self::MIC_LABELS.len()).try_for_each(|i| {
            let mut op = ApogeeCmd::new(VendorCmd::MicPower(i as u8),
                                        &[self.phantoms[i] as u8]);
            avc.control(&AvcAddr::Unit, &mut op, timeout_ms)?;

            let mut op = ApogeeCmd::new(VendorCmd::MicPolarity(i as u8),
                                        &[self.polarities[i] as u8]);
            avc.control(&AvcAddr::Unit, &mut op, timeout_ms)?;

            Ok(())
        })?;

        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Mixer,
                                                   0, 0, Self::IN_LIMIT_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, Self::IN_LABELS.len(), true)?;

        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Mixer,
                                                   0, 0, Self::IN_LEVEL_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, Self::IN_LABELS.len(),
                                         Self::IN_LEVEL_LABELS, None, true)?;

        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Mixer,
                                                   0, 0, Self::MIC_PHANTOM_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, Self::MIC_LABELS.len(), true)?;

        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Mixer,
                                                   0, 0, Self::MIC_POLARITY_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, Self::MIC_LABELS.len(), true)?;

        Ok(())
    }

    pub fn read(&mut self, elem_id: &alsactl::ElemId, elem_value: &mut alsactl::ElemValue) -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::IN_LIMIT_NAME => {
                elem_value.set_bool(&self.limits);
                Ok(true)
            }
            Self::IN_LEVEL_NAME => {
                elem_value.set_enum(&self.levels);
                Ok(true)
            }
            Self::MIC_PHANTOM_NAME => {
                elem_value.set_bool(&self.phantoms);
                Ok(true)
            }
            Self::MIC_POLARITY_NAME => {
                elem_value.set_bool(&self.polarities);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub fn write(&mut self, avc: &BebobAvc, elem_id: &alsactl::ElemId,
                 old: &alsactl::ElemValue, new: &alsactl::ElemValue, timeout_ms: u32)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::IN_LIMIT_NAME => {
                ElemValueAccessor::<bool>::get_vals(new, old, Self::IN_LABELS.len(), |idx, val| {
                    let mut op = ApogeeCmd::new(VendorCmd::InputLimit(idx as u8),
                                                &[val as u8]);
                    avc.control(&AvcAddr::Unit, &mut op, timeout_ms)?;
                    self.limits[idx] = val;
                    Ok(())
                })?;
                Ok(true)
            }
            Self::IN_LEVEL_NAME => {
                ElemValueAccessor::<u32>::get_vals(new, old, Self::IN_LABELS.len(), |idx, val| {
                    let mut op = ApogeeCmd::new(VendorCmd::IoAttr(idx as u8, 0x01),
                                                &[val as u8]);
                    avc.control(&AvcAddr::Unit, &mut op, timeout_ms)?;
                    self.levels[idx] = val;
                    Ok(())
                })?;
                Ok(true)
            }
            Self::MIC_PHANTOM_NAME => {
                ElemValueAccessor::<bool>::get_vals(new, old, Self::MIC_LABELS.len(), |idx, val| {
                    let mut op = ApogeeCmd::new(VendorCmd::MicPower(idx as u8),
                                                &[val as u8]);
                    avc.control(&AvcAddr::Unit, &mut op, timeout_ms)?;
                    self.phantoms[idx] = val;
                    Ok(())
                })?;
                Ok(true)
            }
            Self::MIC_POLARITY_NAME => {
                ElemValueAccessor::<bool>::get_vals(new, old, Self::MIC_LABELS.len(), |idx, val| {
                    let mut op = ApogeeCmd::new(VendorCmd::MicPolarity(idx as u8),
                                                &[val as u8]);
                    avc.control(&AvcAddr::Unit, &mut op, timeout_ms)?;
                    self.polarities[idx] = val;
                    Ok(())
                })?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

pub struct OutputCtl {
    levels: [u32; 8],
}

impl<'a> OutputCtl {
    const OUT_LEVEL_LABELS: &'a [&'a str] = &["+4dB", "-10dB"];

    const OUT_LABELS: &'a [&'a str] = &[
        "analog-1", "analog-2", "analog-3", "analog-4", "analog-5", "analog-6", "analog-7",
        "analog-8",
    ];

    const OUT_LEVEL_NAME: &'a str = "output-level";

    pub fn new() -> Self {
        OutputCtl { levels: [1; 8] }
    }

    pub fn load(&mut self, avc: &BebobAvc, card_cntr: &mut card_cntr::CardCntr, timeout_ms: u32)
        -> Result<(), Error>
    {
        // Transfer initialized data.
        self.levels.iter()
            .enumerate()
            .try_for_each(|(i, l)| {
                let mut op = ApogeeCmd::new(VendorCmd::IoAttr(i as u8, 0x00),
                                            &[*l as u8]);
                avc.control(&AvcAddr::Unit, &mut op, timeout_ms)?;
                Ok(())
            })?;

        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Mixer, 0, 0, Self::OUT_LEVEL_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, Self::OUT_LABELS.len(),
                                         Self::OUT_LEVEL_LABELS, None, true)?;

        Ok(())
    }

    pub fn read(&mut self, elem_id: &alsactl::ElemId, elem_value: &mut alsactl::ElemValue)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::OUT_LEVEL_NAME => {
                elem_value.set_enum(&self.levels);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub fn write(&mut self, avc: &BebobAvc, elem_id: &alsactl::ElemId,
                 old: &alsactl::ElemValue, new: &alsactl::ElemValue, timeout_ms: u32)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::OUT_LEVEL_NAME => {
                ElemValueAccessor::<u32>::get_vals(new, old, Self::OUT_LABELS.len(), |idx, val| {
                    let mut op = ApogeeCmd::new(VendorCmd::IoAttr(idx as u8, 0x00),
                                                &[val as u8]);
                    avc.control(&AvcAddr::Unit, &mut op, timeout_ms)?;
                    self.levels[idx] = val;
                    Ok(())
                })?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

pub struct MixerCtl {
    mixers: [[i32; 36]; 4],
}

impl<'a> MixerCtl {
    const MIXER_LABELS: &'a [&'a str] = &["mixer-1", "mixer-2", "mixer-3", "mixer-4"];

    const MIXER_SRC_LABELS: &'a [&'a str] = &[
        // = VendorCmd::MixerSrc0.
        "analog-1", "analog-2", "analog-3", "analog-4",
        "analog-5", "analog-6", "analog-7", "analog-8",
        "stream-1",
        // = VendorCmd::MixerSrc1.
        "stream-2", "stream-3", "stream-4",
        "stream-5", "stream-6", "stream-7", "stream-8",
        "stream-9", "stream-10",
        // = VendorCmd::MixerSrc2.
        "stream-11", "stream-12",
        "stream-13", "stream-14", "stream-15", "stream-16",
        "stream-17", "stream-18",
        "adat-1",
        // = VendorCmd::MixerSrc3.
        "adat-2", "adat-3", "adat-4",
        "adat-5", "adat-6", "adat-7", "adat-8",
        "spdif-1", "spdif-2",
    ];

    const MIXER_SRC_GAIN_NAME: &'a str = "mixer-source-gain";

    const GAIN_MIN: i32 = 0;
    const GAIN_MAX: i32 = 0x7fff;
    const GAIN_STEP: i32 = 0xff;
    const GAIN_TLV: DbInterval = DbInterval{min: -4800, max: 0, linear: false, mute_avail: true};

    pub fn new() -> Self {
        let mut mixers = [[0; 36]; 4];

        mixers.iter_mut()
            .enumerate()
            .for_each(|(i, mixer)| {
                mixer.iter_mut()
                    .enumerate()
                    .filter(|(j, _)| i % 2 == j % 2)
                    .for_each(|(_, v)| {
                        *v = Self::GAIN_MAX;
                    });
            });

        MixerCtl{mixers}
    }

    fn write_pair(&mut self, avc: &BebobAvc, index: usize, vals: &[i32], pos: usize, timeout_ms: u32)
        -> Result<(), Error>
    {
        let mut args = Vec::new();
        args.push((index / 2) as u8);

        let params = (pos..(pos + 9)).fold(Vec::new(), |mut params, i| {
            let (l, r) = match index % 2 {
                0 => (vals[i] as i16, self.mixers[index + 1][i] as i16),
                _ => (self.mixers[index - 1][i] as i16, vals[i] as i16),
            };
            params.extend_from_slice(&l.to_le_bytes());
            params.extend_from_slice(&r.to_le_bytes());
            params
        });

        let p = (index / 2) as u8;
        let cmd = match pos / 9 {
            3 => VendorCmd::MixerSrc3(p),
            2 => VendorCmd::MixerSrc2(p),
            1 => VendorCmd::MixerSrc1(p),
            _ => VendorCmd::MixerSrc0(p),
        };

        let mut op = ApogeeCmd::new(cmd, &params);
        avc.control(&AvcAddr::Unit, &mut op, timeout_ms)?;

        self.mixers[index].copy_from_slice(&vals[0..Self::MIXER_SRC_LABELS.len()]);

        Ok(())
    }

    pub fn load(&mut self, avc: &BebobAvc, card_cntr: &mut card_cntr::CardCntr, timeout_ms: u32)
        -> Result<(), Error>
    {
        // Transfer initialized data.
        let mixers = self.mixers;
        (0..4).try_for_each(|i| {
            mixers.iter().enumerate().try_for_each(|(j, vals)| {
                self.write_pair(avc, j, vals, i * 9, timeout_ms)
            })
        })?;

        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Mixer,
                                                   0, 0, Self::MIXER_SRC_GAIN_NAME, 0);
        let _ = card_cntr.add_int_elems(&elem_id, Self::MIXER_LABELS.len(),
                                        Self::GAIN_MIN, Self::GAIN_MAX, Self::GAIN_STEP,
                                        Self::MIXER_SRC_LABELS.len(),
                                        Some(&Into::<Vec<u32>>::into(Self::GAIN_TLV)), true)?;

        Ok(())
    }

    pub fn read(&mut self, elem_id: &alsactl::ElemId, elem_value: &mut alsactl::ElemValue)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::MIXER_SRC_GAIN_NAME => {
                let vals = &self.mixers[elem_id.get_index() as usize];
                elem_value.set_int(vals);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub fn write(&mut self, avc: &BebobAvc, elem_id: &alsactl::ElemId,
                 old: &alsactl::ElemValue, new: &alsactl::ElemValue, timeout_ms: u32)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::MIXER_SRC_GAIN_NAME => {
                let len = Self::MIXER_SRC_LABELS.len();
                let mut vals = vec![0;len * 2];
                new.get_int(&mut vals[0..len]);
                old.get_int(&mut vals[len..]);
                let index = elem_id.get_index() as usize;
                for i in 0..4 {
                    let p = i * 9;
                    if vals[p..(p + 9)] != vals[(len + p)..(len + p + 9)] {
                        self.write_pair(avc, index, &vals, p, timeout_ms)?;
                    }
                }
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

pub struct RouteCtl {
    out: [u32; 18],
    cap: [u32; 18],
    hp: [u32; 2],
}

impl<'a> RouteCtl {
    const PORT_LABELS: &'a [&'a str] = &[
        // From external interfaces.
        "analog-1", "analog-2", "analog-3", "analog-4",
        "analog-5", "analog-6", "analog-7", "analog-8",
        // For host computer.
        "stream-1", "stream-2", "stream-3", "stream-4",
        "stream-5", "stream-6", "stream-7", "stream-8",
        "stream-9", "stream-10", "stream-11", "stream-12",
        "stream-13", "stream-14", "stream-15", "stream-16",
        "stream-17", "stream-18",
        // From external interfaces.
        "spdif-1", "spdif-2",
        "adat-1", "adat-2", "adat-3", "adat-4",
        "adat-5", "adat-6", "adat-7", "adat-8",
        // From internal multiplexers.
        "mixer-1", "mixer-2", "mixer-3", "mixer-4",
    ];

    const OUT_LABELS: &'a [&'a str] = &[
        "analog-1", "analog-2", "analog-3", "analog-4",
        "analog-5", "analog-6", "analog-7", "analog-8",
        "spdif-1", "spdif-2",
        "adat-1", "adat-2", "adat-3", "adat-4",
        "adat-5", "adat-6", "adat-7", "adat-8",
    ];

    const OUT_SRC_LABELS: &'a [&'a str] = &[
        "analog-1", "analog-2", "analog-3", "analog-4",
        "analog-5", "analog-6", "analog-7", "analog-8",
        "stream-1", "stream-2", "stream-3", "stream-4",
        "stream-5", "stream-6", "stream-7", "stream-8",
        "stream-9", "stream-10", "stream-11", "stream-12",
        "stream-13", "stream-14", "stream-15", "stream-16",
        "stream-17", "stream-18",
        "spdif-1", "spdif-2",
        "adat-1", "adat-2", "adat-3", "adat-4",
        "adat-5", "adat-6", "adat-7", "adat-8",
        "mixer-1", "mixer-2", "mixer-3", "mixer-4",
    ];

    const CAP_LABELS: &'a [&'a str] = &[
        "stream-1", "stream-2", "stream-3", "stream-4",
        "stream-5", "stream-6", "stream-7", "stream-8",
        "stream-9", "stream-10", "stream-11", "stream-12",
        "stream-13", "stream-14", "stream-15", "stream-16",
        "stream-17", "stream-18",
    ];

    const CAP_SRC_LABELS: &'a [&'a str] = &[
        "analog-1", "analog-2", "analog-3", "analog-4", "analog-5", "analog-6", "analog-7",
        "analog-8", "spdif-1", "spdif-2", "adat-1", "adat-2", "adat-3", "adat-4", "adat-5",
        "adat-6", "adat-7", "adat-8",
    ];

    const HP_LABELS: &'a [&'a str] = &["hp-2", "hp-1"];

    const HP_SRC_LABELS: &'a [&'a str] = &[
        "analog-1/2",
        "analog-3/4",
        "analog-5/6",
        "analog-7/8",
        "spdif-1/2",
        "none",
    ];

    const CAP_SRC_NAME: &'a str = "capture-source";

    pub fn new() -> Self {
        let mut out = [0; 18];
        for (i, v) in out.iter_mut().enumerate() {
            *v = (i + 8) as u32;
        }

        let mut cap = [0; 18];
        for (i, v) in cap.iter_mut().enumerate() {
            *v = i as u32;
        }

        let hp = [1, 0];

        RouteCtl { out, cap, hp }
    }

    fn update_route(&mut self, avc: &BebobAvc, dst: &str, src: &str, timeout_ms: u32)
        -> Result<(), Error>
    {
        if let Some(d) = Self::PORT_LABELS.iter().position(|&x| x == dst) {
            if let Some(s) = Self::PORT_LABELS.iter().position(|&x| x == src) {
                let mut op = ApogeeCmd::new(VendorCmd::IoRouting(d as u8),
                                            &[s as u8]);
                avc.control(&AvcAddr::Unit, &mut op, timeout_ms)?;
                Ok(())
            } else {
                unreachable!();
            }
        } else {
            unreachable!();
        }
    }

    fn update_hp_source(&mut self, avc: &BebobAvc, dst: usize, src: usize, timeout_ms: u32)
        -> Result<(), Error>
    {
        let val = src * 2 + 1;
        let mut op = ApogeeCmd::new(VendorCmd::HpSrc(dst as u8),
                                    &[val as u8]);
        avc.control(&AvcAddr::Unit, &mut op, timeout_ms)?;
        self.hp[dst] = src as u32;
        Ok(())
    }

    pub fn load(&mut self, avc: &BebobAvc, card_cntr: &mut card_cntr::CardCntr, timeout_ms: u32)
        -> Result<(), Error>
    {
        // Transfer initialized data.
        Self::OUT_LABELS.iter().enumerate().try_for_each(|(i, dst)| {
            let src = Self::OUT_SRC_LABELS[8 + i];
            self.update_route(avc, dst, src, timeout_ms)
        })?;

        Self::CAP_LABELS.iter().enumerate().try_for_each(|(i, dst)| {
            let src = Self::CAP_SRC_LABELS[i];
            self.update_route(avc, dst, src, timeout_ms)
        })?;

        (0..Self::HP_LABELS.len()).try_for_each(|i| {
            self.update_hp_source(avc, i, i, timeout_ms)
        })?;

        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Mixer, 0, 0, OUT_SRC_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, Self::OUT_LABELS.len(),
                                         Self::OUT_SRC_LABELS, None, true)?;

        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Mixer,
                                                   0, 0, Self::CAP_SRC_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, Self::CAP_LABELS.len(),
                                         Self::CAP_SRC_LABELS, None, true)?;

        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Mixer, 0, 0, HP_SRC_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, Self::HP_LABELS.len(),
                                         Self::HP_SRC_LABELS, None, true)?;

        Ok(())
    }

    pub fn read(&mut self, elem_id: &alsactl::ElemId, elem_value: &mut alsactl::ElemValue)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            OUT_SRC_NAME => {
                elem_value.set_enum(&self.out);
                Ok(true)
            }
            Self::CAP_SRC_NAME => {
                elem_value.set_enum(&self.cap);
                Ok(true)
            }
            HP_SRC_NAME => {
                elem_value.set_enum(&self.hp);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub fn write(&mut self, avc: &BebobAvc, elem_id: &alsactl::ElemId,
                 old: &alsactl::ElemValue, new: &alsactl::ElemValue, timeout_ms: u32)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            OUT_SRC_NAME => {
                ElemValueAccessor::<u32>::get_vals(new, old, Self::OUT_LABELS.len(), |idx, val| {
                    let dst = Self::OUT_LABELS[idx];
                    let src = Self::OUT_SRC_LABELS[val as usize];
                    self.update_route(avc, dst, src, timeout_ms)?;
                    self.out[idx] = val;
                    Ok(())
                })?;
                Ok(true)
            }
            Self::CAP_SRC_NAME => {
                ElemValueAccessor::<u32>::get_vals(new, old, Self::CAP_LABELS.len(), |idx, val| {
                    let dst = Self::CAP_LABELS[idx];
                    let src = Self::CAP_SRC_LABELS[val as usize];
                    self.update_route(avc, dst, src, timeout_ms)?;
                    self.cap[idx] = val;
                    Ok(())
                })?;
                Ok(true)
            }
            HP_SRC_NAME => {
                ElemValueAccessor::<u32>::get_vals(new, old, Self::HP_LABELS.len(), |idx, val| {
                    self.update_hp_source(avc, idx, val as usize, timeout_ms)?;
                    Ok(())
                })?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

pub struct ResamplerCtl {
    enabled: bool,
    iface: u8,
    direction: u8,
    rate: u8,
}

impl<'a> ResamplerCtl {
    const ENABLE_NAME: &'a str = "resampler-enable";
    const IFACE_NAME: &'a str = "resampler-interface";
    const DIRECTION_NAME: &'a str = "resampler-direction";
    const RATE_NAME: &'a str = "resampler-rate";

    const IFACE_LABELS: &'a [&'a str] = &["optical", "coaxial"];
    const DIRECTION_LABELS: &'a [&'a str] = &["output", "input"];
    const RATE_LABELS: &'a [&'a str] = &["44100", "48000", "88200", "96000", "176400", "192000"];

    pub fn new() -> Self {
        ResamplerCtl {
            enabled: false,
            iface: 0,
            direction: 0,
            rate: 0,
        }
    }

    fn send(&mut self, avc: &BebobAvc, timeout_ms: u32) -> Result<(), Error> {
        let mut op = ApogeeCmd::new(VendorCmd::SpdifResample,
                                    &[self.enabled as u8, self.iface, self.direction, self.rate]);
        avc.control(&AvcAddr::Unit, &mut op, timeout_ms)
    }

    pub fn load(&mut self, avc: &BebobAvc, card_cntr: &mut card_cntr::CardCntr, timeout_ms: u32)
        -> Result<(), Error>
    {
        // Transfer initialized data.
        self.send(avc, timeout_ms)?;

        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Card,
                                                   0, 0, Self::ENABLE_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Card,
                                                   0, 0, Self::IFACE_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, Self::IFACE_LABELS, None, true)?;

        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Card,
                                                   0, 0, Self::DIRECTION_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, Self::DIRECTION_LABELS, None, true)?;

        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Card,
                                                   0, 0, Self::RATE_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, Self::RATE_LABELS, None, true)?;

        Ok(())
    }

    pub fn read(&mut self, elem_id: &alsactl::ElemId, elem_value: &mut alsactl::ElemValue)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::ENABLE_NAME => {
                ElemValueAccessor::<bool>::set_val(elem_value, || Ok(self.enabled))?;
                Ok(true)
            }
            Self::IFACE_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || Ok(self.iface as u32))?;
                Ok(true)
            }
            Self::DIRECTION_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || Ok(self.direction as u32))?;
                Ok(true)
            }
            Self::RATE_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || Ok(self.rate as u32))?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub fn write(&mut self, avc: &BebobAvc, elem_id: &alsactl::ElemId,
                 _: &alsactl::ElemValue, new: &alsactl::ElemValue, timeout_ms: u32)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::ENABLE_NAME => {
                ElemValueAccessor::<bool>::get_val(new, |val| {
                    self.enabled = val;
                    self.send(avc, timeout_ms)
                })?;
                Ok(true)
            }
            Self::IFACE_NAME => {
                ElemValueAccessor::<u32>::get_val(new, |val| {
                    self.iface = val as u8;
                    self.send(avc, timeout_ms)
                })?;
                Ok(true)
            }
            Self::DIRECTION_NAME => {
                ElemValueAccessor::<u32>::get_val(new, |val| {
                    self.direction = val as u8;
                    self.send(avc, timeout_ms)
                })?;
                Ok(true)
            }
            Self::RATE_NAME => {
                ElemValueAccessor::<u32>::get_val(new, |val| {
                    self.rate = val as u8;
                    self.send(avc, timeout_ms)
                })?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

// 33-34: mixer-out-3/4
// 35: unknown
// 36-52: stream-in-0..16, missing 17
pub struct MeterCtl {
    cache: [u8;Self::FRAME_SIZE],
    pub measure_elem_list: Vec<alsactl::ElemId>,
}

impl<'a> MeterCtl {
    const IN_KNOB_TARGET_NAME: &'a str = "input-knob-target";
    const IN_GAIN_NAME: &'a str = "input-gain";

    const OUT_KNOB_TARGET_NAME: &'a str = "output-knob-target";

    const IN_METER_LABELS: &'a [&'a str] = &[
        "analog-1", "analog-2", "analog-3", "analog-4", "analog-5", "analog-6", "analog-7", "analog-8",
        "spdif-1", "spdif-2", "adat-1", "adat-2", "adat-3", "adat-4", "adat-5", "adat-6",
        "adat-7", "adat-8",
    ];

    const OUT_METER_LABELS: &'a [&'a str] = &[
        "analog-1", "analog-2", "analog-3", "analog-4", "analog-5", "analog-6", "analog-7", "analog-8",
        "spdif-1", "spdif-2", "adat-1", "adat-2", "adat-3", "adat-4", "adat-5", "adat-6",
        //"adat-7", "adat-8",
    ];

    const IN_SELECT_LABELS: &'a [&'a str] = &["mic-1", "mic-2", "mic-3", "mic-4"];
    const OUT_SELECT_LABELS: &'a [&'a str] = &["main", "headphone-1/2", "headphone-3/4"];

    const IN_KNOB_TARGET_MASK: u8 = 0x03;
    const IN_KNOB_TARGET_SHIFT: usize = 3;
    const IN_KNOB_TARGET_VALS: &'a [u8] = &[0x00, 0x01, 0x02, 0x03];
    const IN_KNOB_VAL_TARGETS: &'a [u8] = &[0, 1, 2, 3];

    const OUT_KNOB_TARGET_MASK: u8 = 0x07;
    const OUT_KNOB_TARGET_SHIFT: usize = 0;
    const OUT_KNOB_TARGET_VALS: &'a [u8] = &[0x01, 0x02, 0x04];
    const OUT_KNOB_VAL_TARGETS: &'a [u8] = &[0, 1, 2];

    const GAIN_MIN: i32 = 10;
    const GAIN_MAX: i32 = 75;
    const GAIN_STEP: i32 = 1;
    const GAIN_TLV: DbInterval = DbInterval{min: 1000, max: 7500, linear: false, mute_avail: false};

    // NOTE: actually inverted value.
    const VOL_MIN: i32 = -127;
    const VOL_MAX: i32 = 0;
    const VOL_STEP: i32 = 1;
    const VOL_TLV: DbInterval = DbInterval{min: -12700, max: 0, linear: false, mute_avail: false};

    const SELECT_POS: usize = 4;
    const IN_GAIN_POS: &'a [usize] = &[0, 1, 2, 3];
    const OUT_VOL_POS: &'a [usize] = &[7, 6, 5];
    const IN_METER_POS: &'a [usize] = &[12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29];
    const OUT_METER_POS: &'a [usize] = &[35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50];

    const METER_MIN: i32 = 0x00;
    const METER_MAX: i32 = 0xff;
    const METER_STEP: i32 = 0x01;
    const METER_TLV: DbInterval = DbInterval{min: -4800, max: 0, linear: false, mute_avail: false};

    const FRAME_SIZE: usize = 56;

    pub fn new() -> Self {
        MeterCtl {
            cache: [0;Self::FRAME_SIZE],
            measure_elem_list: Vec::new(),
        }
    }

    pub fn load(&mut self, avc: &BebobAvc, card_cntr: &mut card_cntr::CardCntr, timeout_ms: u32)
        -> Result<(), Error>
    {
        self.measure_states(avc, timeout_ms)?;

        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Mixer,
                                                   0, 0, Self::IN_KNOB_TARGET_NAME, 0);
        let mut elem_id_list = card_cntr.add_enum_elems(&elem_id, 1, 1,
                                                        Self::IN_SELECT_LABELS, None, true)?;
        self.measure_elem_list.append(&mut elem_id_list);

        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Mixer,
                                                   0, 0, Self::IN_GAIN_NAME, 0);
        let mut elem_id_list = card_cntr.add_int_elems(&elem_id, 1,
                                        Self::GAIN_MIN, Self::GAIN_MAX, Self::GAIN_STEP,
                                        Self::IN_SELECT_LABELS.len(),
                                        Some(&Into::<Vec<u32>>::into(Self::GAIN_TLV)), true)?;
        self.measure_elem_list.append(&mut elem_id_list);

        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Mixer,
                                                    0, 0, Self::OUT_KNOB_TARGET_NAME, 0);
        let mut elem_id_list = card_cntr.add_enum_elems(&elem_id, 1, 1,
                                                        Self::OUT_SELECT_LABELS, None, true)?;
        self.measure_elem_list.append(&mut elem_id_list);

        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Mixer, 0, 0, OUT_VOL_NAME, 0);
        let mut elem_id_list = card_cntr.add_int_elems(&elem_id, 1,
                                        Self::VOL_MIN, Self::VOL_MAX, Self::VOL_STEP,
                                        Self::OUT_SELECT_LABELS.len(),
                                        Some(&Into::<Vec<u32>>::into(Self::VOL_TLV)), true)?;
        self.measure_elem_list.append(&mut elem_id_list);

        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Mixer, 0, 0, IN_METER_NAME, 0);
        let mut elem_id_list = card_cntr.add_int_elems(&elem_id, 1,
                                                       Self::METER_MIN, Self::METER_MAX, Self::METER_STEP,
                                                       Self::IN_METER_LABELS.len(),
                                                       Some(&Into::<Vec<u32>>::into(Self::METER_TLV)), false)?;
        self.measure_elem_list.append(&mut elem_id_list);

        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Mixer, 0, 0, OUT_METER_NAME, 0);
        let mut elem_id_list = card_cntr.add_int_elems(&elem_id, 1,
                                                       Self::METER_MIN, Self::METER_MAX, Self::METER_STEP,
                                                       Self::OUT_METER_LABELS.len(),
                                                       Some(&Into::<Vec<u32>>::into(Self::VOL_TLV)), false)?;
        self.measure_elem_list.append(&mut elem_id_list);


        Ok(())
    }

    pub fn read(&mut self, elem_id: &alsactl::ElemId, elem_value: &mut alsactl::ElemValue)
        -> Result<bool, Error>
    {
        self.measure_elem(elem_id, elem_value)
    }

    pub fn write(&mut self, avc: &BebobAvc, elem_id: &alsactl::ElemId,
                 old: &alsactl::ElemValue, new: &alsactl::ElemValue, timeout_ms: u32)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::IN_KNOB_TARGET_NAME => {
                ElemValueAccessor::<u32>::get_val(new, |val| {
                    let idx = val as usize;
                    let target = Self::IN_KNOB_VAL_TARGETS[idx];
                    let pos = Self::IN_GAIN_POS[idx];
                    let mut op = ApogeeCmd::new(VendorCmd::MicGain(target),
                                                &self.cache[pos..(pos + 1)]);
                    avc.control(&AvcAddr::Unit, &mut op, timeout_ms)
                })?;
                Ok(true)
            }
            Self::IN_GAIN_NAME => {
                ElemValueAccessor::<i32>::get_vals(new, old, Self::IN_KNOB_VAL_TARGETS.len(), |idx, val| {
                    let target = Self::IN_KNOB_VAL_TARGETS[idx];
                    let mut op = ApogeeCmd::new(VendorCmd::MicGain(target), &[val as u8]);
                    avc.control(&AvcAddr::Unit, &mut op, timeout_ms)
                })?;
                Ok(true)
            }
            Self::OUT_KNOB_TARGET_NAME => {
                ElemValueAccessor::<u32>::get_val(new, |val| {
                    let idx = val as usize;
                    let target = Self::OUT_KNOB_VAL_TARGETS[idx];
                    let pos = Self::OUT_VOL_POS[idx];
                    let mut op = ApogeeCmd::new(VendorCmd::OutVol(target),
                                                &self.cache[pos..(pos + 1)]);
                    avc.control(&AvcAddr::Unit, &mut op, timeout_ms)
                })?;
                Ok(true)
            }
            OUT_VOL_NAME => {
                ElemValueAccessor::<i32>::get_vals(new, old, Self::OUT_KNOB_VAL_TARGETS.len(), |idx, val| {
                    let target = Self::OUT_KNOB_VAL_TARGETS[idx];
                    let val = -val;
                    let mut op = ApogeeCmd::new(VendorCmd::OutVol(target), &[val as u8]);
                    avc.control(&AvcAddr::Unit, &mut op, timeout_ms)?;
                    Ok(())
                })?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub fn measure_states(&mut self, avc: &BebobAvc, timeout_ms: u32) -> Result<(), Error> {
        let mut op = ApogeeCmd::new(VendorCmd::HwStatus(true), &[0x01]);
        avc.control(&AvcAddr::Unit, &mut op, timeout_ms)?;
        self.cache.copy_from_slice(&op.params);
        Ok(())
    }

    pub fn measure_elem(&mut self, elem_id: &alsactl::ElemId, elem_value: &mut alsactl::ElemValue)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::IN_KNOB_TARGET_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let pos = Self::SELECT_POS;
                    let val = (self.cache[pos] >> Self::IN_KNOB_TARGET_SHIFT) & Self::IN_KNOB_TARGET_MASK;
                    match Self::IN_KNOB_TARGET_VALS.iter().position(|v| *v == val) {
                        Some(pos) => Ok(pos as u32),
                        None => {
                            let label = format!("Unexpected value for flag of input knob: {}", val);
                            Err(Error::new(FileError::Io, &label))
                        }
                    }
                })?;
                Ok(true)
            }
            Self::IN_GAIN_NAME => {
                ElemValueAccessor::<i32>::set_vals(elem_value, Self::IN_GAIN_POS.len(), |idx| {
                    let pos = Self::IN_GAIN_POS[idx];
                    Ok(self.cache[pos] as i32)
                })?;
                Ok(true)
            }
            Self::OUT_KNOB_TARGET_NAME => {
                let val = (self.cache[Self::SELECT_POS] >> Self::OUT_KNOB_TARGET_SHIFT) & Self::OUT_KNOB_TARGET_MASK;
                match Self::OUT_KNOB_TARGET_VALS.iter().position(|v| *v == val) {
                    Some(pos) => {
                        elem_value.set_enum(&[pos as u32]);
                        Ok(true)
                    }
                    None => {
                        let label = format!("Unexpected value for flag of output knob: {}", val);
                        Err(Error::new(FileError::Io, &label))
                    }
                }
            }
            OUT_VOL_NAME => {
                ElemValueAccessor::<i32>::set_vals(elem_value, Self::OUT_VOL_POS.len(), |idx| {
                    let pos = Self::OUT_VOL_POS[idx];
                    Ok(self.cache[pos] as i32)
                })?;
                Ok(true)
            }
            IN_METER_NAME => {
                ElemValueAccessor::<i32>::set_vals(elem_value, Self::IN_METER_POS.len(), |idx| {
                    let pos = Self::IN_METER_POS[idx];
                    Ok(self.cache[pos] as i32)
                })?;
                Ok(true)
            }
            OUT_METER_NAME => {
                ElemValueAccessor::<i32>::set_vals(elem_value, Self::OUT_METER_POS.len(), |idx| {
                    let pos = Self::OUT_METER_POS[idx];
                    Ok(self.cache[pos] as i32)
                })?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
