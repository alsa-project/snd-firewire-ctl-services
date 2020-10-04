// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use hinawa::SndUnitExt;

use alsactl::{ElemValueExt, ElemValueExtManual};

use crate::card_cntr;

use crate::ta1394::{AvcAddr, Ta1394Avc};

use crate::bebob::BebobAvc;
use crate::bebob::extensions::{BcoPlugAddr, BcoPlugDirection, BcoPlugAddrUnitType};
use crate::bebob::extensions::BcoCompoundAm824StreamFormat;
use crate::bebob::extensions::ExtendedStreamFormatSingle;
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
        let mut op = ApogeeCmd::new(&avc.company_id, VendorCmd::Hw(HwCmdOp::CdMode),
                                    &[self.cd as u8]);
        avc.control(&AvcAddr::Unit, &mut op, timeout_ms)?;

        let mut op = ApogeeCmd::new(&avc.company_id, VendorCmd::Downgrade, &[self.cd as u8]);
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
                elem_value.set_enum(&[self.stream]);
                Ok(true)
            }
            Self::CD_MODE_NAME => {
                elem_value.set_bool(&[self.cd]);
                Ok(true)
            }
            Self::SPDIF_OUT_BYPASS_NAME => {
                elem_value.set_enum(&[self.spdif_out_bypass]);
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
                let mut vals = [0];
                new.get_enum(&mut vals);
                let mut op = ApogeeCmd::new(&avc.company_id, VendorCmd::Hw(HwCmdOp::StreamMode),
                                            &[vals[0] as u8]);
                unit.lock()?;
                let res = avc.control(&AvcAddr::Unit, &mut op, timeout_ms);
                unit.unlock()?;
                res.and(Ok(true))
            }
            Self::CD_MODE_NAME => {
                let mut vals = [false];
                new.get_bool(&mut vals);
                let mut op = ApogeeCmd::new(&avc.company_id, VendorCmd::Hw(HwCmdOp::CdMode),
                                            &[vals[0] as u8]);
                avc.control(&AvcAddr::Unit, &mut op, timeout_ms)?;
                self.cd = vals[0];
                Ok(true)
            }
            Self::SPDIF_OUT_BYPASS_NAME => {
                let mut vals = [0];
                new.get_enum(&mut vals);
                let mut op = ApogeeCmd::new(&avc.company_id, VendorCmd::Downgrade,
                                            &[vals[0] as u8]);
                avc.control(&AvcAddr::Unit, &mut op, timeout_ms)?;
                self.spdif_out_bypass = vals[0] as u32;
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
        let mut op = ApogeeCmd::new(&avc.company_id, VendorCmd::Hw(HwCmdOp::DisplayIlluminate),
                                    &[self.illuminate as u8]);
        avc.control(&AvcAddr::Unit, &mut op, timeout_ms)?;

        let mut op = ApogeeCmd::new(&avc.company_id, VendorCmd::Hw(HwCmdOp::DisplayMode),
                                    &[self.mode as u8]);
        avc.control(&AvcAddr::Unit, &mut op, timeout_ms)?;

        let mut op = ApogeeCmd::new(&avc.company_id, VendorCmd::Hw(HwCmdOp::DisplayTarget),
                                    &[self.target as u8]);
        avc.control(&AvcAddr::Unit, &mut op, timeout_ms)?;

        let mut op = ApogeeCmd::new(&avc.company_id, VendorCmd::Hw(HwCmdOp::DisplayOverhold),
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
                elem_value.set_bool(&[self.illuminate]);
                Ok(true)
            }
            Self::MODE_NAME => {
                elem_value.set_bool(&[self.mode]);
                Ok(true)
            }
            Self::TARGET_NAME => {
                elem_value.set_enum(&[self.target]);
                Ok(true)
            }
            Self::OVERHOLD_NAME => {
                elem_value.set_bool(&[self.overhold]);
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
                let mut vals = [false];
                new.get_bool(&mut vals);
                let mut op = ApogeeCmd::new(&avc.company_id, VendorCmd::Hw(HwCmdOp::DisplayIlluminate),
                                            &[vals[0] as u8]);
                avc.control(&AvcAddr::Unit, &mut op, timeout_ms)?;
                self.illuminate = vals[0];
                Ok(true)
            }
            Self::MODE_NAME => {
                let mut vals = [false];
                new.get_bool(&mut vals);
                let mut op = ApogeeCmd::new(&avc.company_id, VendorCmd::Hw(HwCmdOp::DisplayMode),
                                            &[vals[0] as u8]);
                avc.control(&AvcAddr::Unit, &mut op, timeout_ms)?;
                self.mode = vals[0];
                Ok(true)
            }
            Self::TARGET_NAME => {
                let mut vals = [0];
                new.get_enum(&mut vals);
                let mut op = ApogeeCmd::new(&avc.company_id, VendorCmd::Hw(HwCmdOp::DisplayTarget),
                                            &[vals[0] as u8]);
                avc.control(&AvcAddr::Unit, &mut op, timeout_ms)?;
                self.target = vals[0];
                Ok(true)
            }
            Self::OVERHOLD_NAME => {
                let mut vals = [false];
                new.get_bool(&mut vals);
                let mut op = ApogeeCmd::new(&avc.company_id, VendorCmd::Hw(HwCmdOp::DisplayOverhold),
                                            &[vals[0] as u8]);
                avc.control(&AvcAddr::Unit, &mut op, timeout_ms)?;
                self.overhold = vals[0];
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
        let mut op = ApogeeCmd::new(&avc.company_id, VendorCmd::OptIfaceMode(0x00), &[self.output as u8]);
        avc.control(&AvcAddr::Unit, &mut op, timeout_ms)?;

        let mut op = ApogeeCmd::new(&avc.company_id, VendorCmd::OptIfaceMode(0x01), &[self.input as u8]);
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
                elem_value.set_enum(&[self.output]);
                Ok(true)
            }
            Self::IN_MODE_NAME => {
                elem_value.set_enum(&[self.input]);
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
                let mut vals = [0];
                new.get_enum(&mut vals);
                let mut op = ApogeeCmd::new(&avc.company_id, VendorCmd::OptIfaceMode(0x00), &[vals[0] as u8]);
                avc.control(&AvcAddr::Unit, &mut op, timeout_ms)?;
                self.output = vals[0];
                Ok(true)
            }
            Self::IN_MODE_NAME => {
                let mut vals = [0];
                new.get_enum(&mut vals);
                let mut op = ApogeeCmd::new(&avc.company_id, VendorCmd::OptIfaceMode(0x01), &[vals[0] as u8]);
                avc.control(&AvcAddr::Unit, &mut op, timeout_ms)?;
                self.input = vals[0];
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
            let mut op = ApogeeCmd::new(&avc.company_id, VendorCmd::InputLimit(i as u8),
                                        &[self.limits[i] as u8]);
            avc.control(&AvcAddr::Unit, &mut op, timeout_ms)?;

            let mut op = ApogeeCmd::new(&avc.company_id, VendorCmd::IoAttr(i as u8, 0x01),
                                        &[self.levels[i] as u8]);
            avc.control(&AvcAddr::Unit, &mut op, timeout_ms)?;

            Ok(())
        })?;

        (0..Self::MIC_LABELS.len()).try_for_each(|i| {
            let mut op = ApogeeCmd::new(&avc.company_id, VendorCmd::MicPower(i as u8),
                                        &[self.phantoms[i] as u8]);
            avc.control(&AvcAddr::Unit, &mut op, timeout_ms)?;

            let mut op = ApogeeCmd::new(&avc.company_id, VendorCmd::MicPolarity(i as u8),
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
                let len = Self::IN_LABELS.len();
                let mut vals = vec![false;len * 2];
                new.get_bool(&mut vals[..len]);
                old.get_bool(&mut vals[len..]);
                vals[..len].iter().zip(vals[len..].iter()).enumerate()
                    .filter(|(_, (n, o))| *n != *o)
                    .try_for_each(|(i, (n, _))| {
                        let mut op = ApogeeCmd::new(&avc.company_id, VendorCmd::InputLimit(i as u8),
                                                    &[*n as u8]);
                        avc.control(&AvcAddr::Unit, &mut op, timeout_ms)?;
                        self.limits[i] = *n;
                        Ok(())
                    })?;
                Ok(true)
            }
            Self::IN_LEVEL_NAME => {
                let len = Self::IN_LABELS.len();
                let mut vals = vec![0;len * 2];
                new.get_enum(&mut vals[..len]);
                old.get_enum(&mut vals[len..]);
                vals[..len].iter().zip(vals[len..].iter()).enumerate()
                    .filter(|(_, (n, o))| *n != *o)
                    .try_for_each(|(i, (n, _))| {
                        let mut op = ApogeeCmd::new(&avc.company_id, VendorCmd::IoAttr(i as u8, 0x01),
                                                    &[*n as u8]);
                        avc.control(&AvcAddr::Unit, &mut op, timeout_ms)?;
                        self.levels[i] = *n;
                        Ok(())
                    })?;
                Ok(true)
            }
            Self::MIC_PHANTOM_NAME => {
                let len = Self::MIC_LABELS.len();
                let mut vals = vec![false;len * 2];
                new.get_bool(&mut vals[..len]);
                old.get_bool(&mut vals[len..]);
                vals[..len].iter().zip(vals[len..].iter())
                    .enumerate()
                    .filter(|(_, (n, o))| *n != *o)
                    .try_for_each(|(i, (n, _))| {
                        let mut op = ApogeeCmd::new(&avc.company_id, VendorCmd::MicPower(i as u8),
                                                    &[*n as u8]);
                        avc.control(&AvcAddr::Unit, &mut op, timeout_ms)?;
                        self.phantoms[i] = *n;
                        Ok(())
                    })?;
                Ok(true)
            }
            Self::MIC_POLARITY_NAME => {
                let len = Self::MIC_LABELS.len();
                let mut vals = vec![false;len * 2];
                new.get_bool(&mut vals[0..len]);
                old.get_bool(&mut vals[len..]);
                vals[..len].iter().zip(vals[len..].iter())
                    .enumerate()
                    .filter(|(_, (n, o))| *n != *o)
                    .try_for_each(|(i, (n, _))| {
                        let mut op = ApogeeCmd::new(&avc.company_id, VendorCmd::MicPolarity(i as u8),
                                                    &[*n as u8]);
                        avc.control(&AvcAddr::Unit, &mut op, timeout_ms)?;
                        self.polarities[i] = *n;
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
                let mut op = ApogeeCmd::new(&avc.company_id, VendorCmd::IoAttr(i as u8, 0x00),
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
                let len = Self::OUT_LABELS.len();
                let mut vals = vec![0;len * 2];
                new.get_enum(&mut vals[..len]);
                old.get_enum(&mut vals[len..]);
                vals[..len].iter().zip(vals[len..].iter())
                    .enumerate()
                    .filter(|(_, (n, o))| *n != *o)
                    .try_for_each(|(i, (n, _))| {
                        let mut op = ApogeeCmd::new(&avc.company_id, VendorCmd::IoAttr(i as u8, 0x00),
                                                    &[*n as u8]);
                        avc.control(&AvcAddr::Unit, &mut op, timeout_ms)?;
                        self.levels[i] = *n;
                        Ok(())
                    })?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
