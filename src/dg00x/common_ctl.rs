// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use hinawa::SndUnitExt;

use alsactl::{ElemValueExt, ElemValueExtManual};

use crate::card_cntr;

use super::protocol::CommonProtocol;

pub struct CommonCtl {
    has_word_bnc: bool,
}

impl<'a> CommonCtl {
    const CLK_LOCAL_RATE_NAME: &'a str = "local-clock-rate";
    const CLK_EXT_RATE_NAME: &'a str = "external-clock-rate";
    const CLK_SRC_NAME: &'a str = "clock-source";
    const OPT_IFACE_NAME: &'a str = "optical-interface";
    const CLK_EXT_DETECT_NAME: &'a str = "external-clock-detect";

    const CLK_RATE_LABELS: &'a [&'a str] = &["44100", "48000", "88200", "96000"];
    const CLK_SRC_LABELS: &'a [&'a str] = &["Internal", "S/PDIF", "ADAT", "Word-clock"];
    const OPT_IFACE_LABELS: &'a [&'a str] = &["ADAT", "S/PDIF"];

    const CLK_LOCAL_RATE_OFFSET: u64 = 0x0110;
    const CLK_EXT_RATE_OFFSET: u64 = 0x0114;
    const CLK_SRC_OFFSET: u64 = 0x0118;
    const OPT_IFACE_OFFSET: u64 = 0x011c;
    const CLK_EXT_DETECT_OFFSET: u64 = 0x012c;

    pub fn new(has_word_bnc: bool) -> Self {
        CommonCtl { has_word_bnc }
    }

    pub fn load(
        &mut self,
        _: &hinawa::SndDg00x,
        _: &hinawa::FwReq,
        card_cntr: &mut card_cntr::CardCntr,
    ) -> Result<(), Error> {
        let mut len = Self::CLK_SRC_LABELS.len();
        if !self.has_word_bnc {
            len -= 1;
        }
        let elem_id = alsactl::ElemId::new_by_name(
            alsactl::ElemIfaceType::Mixer,
            0,
            0,
            Self::CLK_SRC_NAME,
            0,
        );
        let _ =
            card_cntr.add_enum_elems(&elem_id, 1, 1, &Self::CLK_SRC_LABELS[0..len], None, true)?;

        let elem_id = alsactl::ElemId::new_by_name(
            alsactl::ElemIfaceType::Mixer,
            0,
            0,
            Self::CLK_LOCAL_RATE_NAME,
            0,
        );
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, Self::CLK_RATE_LABELS, None, true)?;

        let elem_id = alsactl::ElemId::new_by_name(
            alsactl::ElemIfaceType::Mixer,
            0,
            0,
            Self::CLK_EXT_RATE_NAME,
            0,
        );
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, Self::CLK_RATE_LABELS, None, false)?;

        let elem_id = alsactl::ElemId::new_by_name(
            alsactl::ElemIfaceType::Mixer,
            0,
            0,
            Self::OPT_IFACE_NAME,
            0,
        );
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, Self::OPT_IFACE_LABELS, None, true)?;

        let elem_id = alsactl::ElemId::new_by_name(
            alsactl::ElemIfaceType::Mixer,
            0,
            0,
            Self::CLK_EXT_DETECT_NAME,
            0,
        );
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, false)?;

        Ok(())
    }

    pub fn read(
        &mut self,
        unit: &hinawa::SndDg00x,
        req: &hinawa::FwReq,
        elem_id: &alsactl::ElemId,
        elem_value: &mut alsactl::ElemValue,
    ) -> Result<bool, Error> {
        let node = unit.get_node();

        match elem_id.get_name().as_str() {
            Self::CLK_SRC_NAME => {
                let val = req.read_quadlet(&node, Self::CLK_SRC_OFFSET)?;
                elem_value.set_enum(&[val]);
                Ok(true)
            }
            Self::CLK_LOCAL_RATE_NAME => {
                let val = req.read_quadlet(&node, Self::CLK_LOCAL_RATE_OFFSET)?;
                elem_value.set_enum(&[val]);
                Ok(true)
            }
            Self::CLK_EXT_RATE_NAME => {
                let val = req.read_quadlet(&node, Self::CLK_EXT_RATE_OFFSET)?;
                elem_value.set_enum(&[val]);
                Ok(true)
            }
            Self::OPT_IFACE_NAME => {
                let val = req.read_quadlet(&node, Self::OPT_IFACE_OFFSET)?;
                elem_value.set_enum(&[val]);
                Ok(true)
            }
            Self::CLK_EXT_DETECT_NAME => {
                let val = req.read_quadlet(&node, Self::CLK_EXT_DETECT_OFFSET)?;
                elem_value.set_enum(&[val]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub fn write(
        &mut self,
        unit: &hinawa::SndDg00x,
        req: &hinawa::FwReq,
        elem_id: &alsactl::ElemId,
        _: &alsactl::ElemValue,
        new: &alsactl::ElemValue,
    ) -> Result<bool, Error> {
        if unit.get_property_streaming() {
            return Ok(false);
        }

        let node = unit.get_node();

        match elem_id.get_name().as_str() {
            Self::CLK_SRC_NAME => {
                let mut vals = [0];
                new.get_enum(&mut vals);
                req.write_quadlet(&node, Self::CLK_SRC_OFFSET, vals[0])?;
                Ok(true)
            }
            Self::CLK_LOCAL_RATE_NAME => {
                let mut vals = [0];
                new.get_enum(&mut vals);
                req.write_quadlet(&node, Self::CLK_LOCAL_RATE_OFFSET, vals[0])?;
                Ok(true)
            }
            Self::OPT_IFACE_NAME => {
                let mut vals = [0];
                new.get_enum(&mut vals);
                req.write_quadlet(&node, Self::OPT_IFACE_OFFSET, vals[0])?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
