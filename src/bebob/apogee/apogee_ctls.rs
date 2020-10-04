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
