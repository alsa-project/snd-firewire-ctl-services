// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use hinawa::SndUnitExt;

use alsactl::{ElemValueExt, ElemValueExtManual};

use alsa_ctl_tlv_codec::items::DbInterval;

use core::card_cntr;
use core::elem_value_accessor::ElemValueAccessor;

use ta1394::{AvcAddr, Ta1394Avc};

use bebob_protocols::*;
use bebob_protocols::bridgeco::{BcoPlugAddr, BcoPlugDirection, BcoPlugAddrUnitType};
use bebob_protocols::bridgeco::BcoCompoundAm824StreamFormat;
use bebob_protocols::bridgeco::ExtendedStreamFormatSingle;

use crate::model::{HP_SRC_NAME, OUT_SRC_NAME};

use bebob_protocols::apogee::ensemble::{EnsembleOperation, EnsembleCmd, HwCmd};

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
        let mut op = EnsembleOperation::new(EnsembleCmd::Hw(HwCmd::CdMode),
                                    &[self.cd as u8]);
        avc.control(&AvcAddr::Unit, &mut op, timeout_ms)?;

        let mut op = EnsembleOperation::new(EnsembleCmd::Downgrade, &[self.cd as u8]);
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
                    let mut op = EnsembleOperation::new(EnsembleCmd::Hw(HwCmd::StreamMode),
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
                    let mut op = EnsembleOperation::new(EnsembleCmd::Hw(HwCmd::CdMode),
                                                &[val as u8]);
                    avc.control(&AvcAddr::Unit, &mut op, timeout_ms)?;
                    self.cd = val;
                    Ok(())
                })?;
                Ok(true)
            }
            Self::SPDIF_OUT_BYPASS_NAME => {
                ElemValueAccessor::<u32>::get_val(new, |val| {
                    let mut op = EnsembleOperation::new(EnsembleCmd::Downgrade,
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
        let mut op = EnsembleOperation::new(EnsembleCmd::Hw(HwCmd::DisplayIlluminate),
                                    &[self.illuminate as u8]);
        avc.control(&AvcAddr::Unit, &mut op, timeout_ms)?;

        let mut op = EnsembleOperation::new(EnsembleCmd::Hw(HwCmd::DisplayMode),
                                    &[self.mode as u8]);
        avc.control(&AvcAddr::Unit, &mut op, timeout_ms)?;

        let mut op = EnsembleOperation::new(EnsembleCmd::Hw(HwCmd::DisplayTarget),
                                    &[self.target as u8]);
        avc.control(&AvcAddr::Unit, &mut op, timeout_ms)?;

        let mut op = EnsembleOperation::new(EnsembleCmd::Hw(HwCmd::DisplayOverhold),
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
                    let mut op = EnsembleOperation::new(EnsembleCmd::Hw(HwCmd::DisplayIlluminate),
                                                &[val as u8]);
                    avc.control(&AvcAddr::Unit, &mut op, timeout_ms)?;
                    self.illuminate = val;
                    Ok(())
                })?;
                Ok(true)
            }
            Self::MODE_NAME => {
                ElemValueAccessor::<bool>::get_val(new, |val| {
                    let mut op = EnsembleOperation::new(EnsembleCmd::Hw(HwCmd::DisplayMode),
                                                &[val as u8]);
                    avc.control(&AvcAddr::Unit, &mut op, timeout_ms)?;
                    self.mode = val;
                    Ok(())
                })?;
                Ok(true)
            }
            Self::TARGET_NAME => {
                ElemValueAccessor::<u32>::get_val(new, |val| {
                    let mut op = EnsembleOperation::new(EnsembleCmd::Hw(HwCmd::DisplayTarget),
                                                &[val as u8]);
                    avc.control(&AvcAddr::Unit, &mut op, timeout_ms)?;
                    self.target = val;
                    Ok(())
                })?;
                Ok(true)
            }
            Self::OVERHOLD_NAME => {
                ElemValueAccessor::<bool>::get_val(new, |val| {
                    let mut op = EnsembleOperation::new(EnsembleCmd::Hw(HwCmd::DisplayOverhold),
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
        let mut op = EnsembleOperation::new(EnsembleCmd::OptIfaceMode(0x00), &[self.output as u8]);
        avc.control(&AvcAddr::Unit, &mut op, timeout_ms)?;

        let mut op = EnsembleOperation::new(EnsembleCmd::OptIfaceMode(0x01), &[self.input as u8]);
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
                    let mut op = EnsembleOperation::new(EnsembleCmd::OptIfaceMode(0x00),
                                                &[val as u8]);
                    avc.control(&AvcAddr::Unit, &mut op, timeout_ms)?;
                    self.output = val;
                    Ok(())
                })?;
                Ok(true)
            }
            Self::IN_MODE_NAME => {
                ElemValueAccessor::<u32>::get_val(new, |val| {
                    let mut op = EnsembleOperation::new(EnsembleCmd::OptIfaceMode(0x01),
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
            let mut op = EnsembleOperation::new(EnsembleCmd::InputLimit(i as u8),
                                        &[self.limits[i] as u8]);
            avc.control(&AvcAddr::Unit, &mut op, timeout_ms)?;

            let mut op = EnsembleOperation::new(EnsembleCmd::IoAttr(i as u8, 0x01),
                                        &[self.levels[i] as u8]);
            avc.control(&AvcAddr::Unit, &mut op, timeout_ms)?;

            Ok(())
        })?;

        (0..Self::MIC_LABELS.len()).try_for_each(|i| {
            let mut op = EnsembleOperation::new(EnsembleCmd::MicPower(i as u8),
                                        &[self.phantoms[i] as u8]);
            avc.control(&AvcAddr::Unit, &mut op, timeout_ms)?;

            let mut op = EnsembleOperation::new(EnsembleCmd::MicPolarity(i as u8),
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
                    let mut op = EnsembleOperation::new(EnsembleCmd::InputLimit(idx as u8),
                                                &[val as u8]);
                    avc.control(&AvcAddr::Unit, &mut op, timeout_ms)?;
                    self.limits[idx] = val;
                    Ok(())
                })?;
                Ok(true)
            }
            Self::IN_LEVEL_NAME => {
                ElemValueAccessor::<u32>::get_vals(new, old, Self::IN_LABELS.len(), |idx, val| {
                    let mut op = EnsembleOperation::new(EnsembleCmd::IoAttr(idx as u8, 0x01),
                                                &[val as u8]);
                    avc.control(&AvcAddr::Unit, &mut op, timeout_ms)?;
                    self.levels[idx] = val;
                    Ok(())
                })?;
                Ok(true)
            }
            Self::MIC_PHANTOM_NAME => {
                ElemValueAccessor::<bool>::get_vals(new, old, Self::MIC_LABELS.len(), |idx, val| {
                    let mut op = EnsembleOperation::new(EnsembleCmd::MicPower(idx as u8),
                                                &[val as u8]);
                    avc.control(&AvcAddr::Unit, &mut op, timeout_ms)?;
                    self.phantoms[idx] = val;
                    Ok(())
                })?;
                Ok(true)
            }
            Self::MIC_POLARITY_NAME => {
                ElemValueAccessor::<bool>::get_vals(new, old, Self::MIC_LABELS.len(), |idx, val| {
                    let mut op = EnsembleOperation::new(EnsembleCmd::MicPolarity(idx as u8),
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
                let mut op = EnsembleOperation::new(EnsembleCmd::IoAttr(i as u8, 0x00),
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
                    let mut op = EnsembleOperation::new(EnsembleCmd::IoAttr(idx as u8, 0x00),
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
        // = EnsembleCmd::MixerSrc0.
        "analog-1", "analog-2", "analog-3", "analog-4",
        "analog-5", "analog-6", "analog-7", "analog-8",
        "stream-1",
        // = EnsembleCmd::MixerSrc1.
        "stream-2", "stream-3", "stream-4",
        "stream-5", "stream-6", "stream-7", "stream-8",
        "stream-9", "stream-10",
        // = EnsembleCmd::MixerSrc2.
        "stream-11", "stream-12",
        "stream-13", "stream-14", "stream-15", "stream-16",
        "stream-17", "stream-18",
        "adat-1",
        // = EnsembleCmd::MixerSrc3.
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
            3 => EnsembleCmd::MixerSrc3(p),
            2 => EnsembleCmd::MixerSrc2(p),
            1 => EnsembleCmd::MixerSrc1(p),
            _ => EnsembleCmd::MixerSrc0(p),
        };

        let mut op = EnsembleOperation::new(cmd, &params);
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
                let mut op = EnsembleOperation::new(EnsembleCmd::IoRouting(d as u8),
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
        let mut op = EnsembleOperation::new(EnsembleCmd::HpSrc(dst as u8),
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
        let mut op = EnsembleOperation::new(EnsembleCmd::SpdifResample,
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
