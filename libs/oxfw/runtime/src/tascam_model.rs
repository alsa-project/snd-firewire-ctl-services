// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use hinawa::{FwFcpExt, SndUnitExt};

use core::card_cntr;
use core::elem_value_accessor::ElemValueAccessor;

use ta1394::{Ta1394Avc, AvcAddr};
use ta1394::general::UnitInfo;

use super::tascam_proto::{TascamProto, VendorCmd, TascamAvc};

use super::common_ctl::CommonCtl;

pub struct TascamModel{
    avc: TascamAvc,
    common_ctl: CommonCtl,
}

impl<'a> TascamModel {
    const FCP_TIMEOUT_MS: u32 = 100;

    const DISPLAY_MODE_NAME: &'a str = "display-mode";
    const MESSAGE_MODE_NAME: &'a str = "message-mode";
    const INPUT_MODE_NAME: &'a str = "input-mode";
    const FIRMWARE_VERSION_NAME: &'a str = "firmware-version";

    const DISPLAY_MODE_LABELS: &'a [&'a str] = &[
        "always-off",
        "always-on",
        "breathe",
        "metronome",
        "midi-clock-rotate",
        "midi-clock-flash",
        "jog-slow-rotate",
        "jog-track",
    ];
    const MESSAGE_MODE_LABELS: &'a [&'a str] = &["native", "mackie-hui-emulation"];
    const INPUT_MODE_LABELS: &'a [&'a str] = &["stereo", "monaural"];

    pub fn new() -> Self {
        TascamModel{
            avc: TascamAvc::new(),
            common_ctl: CommonCtl::new(),
        }
    }
}

impl card_cntr::CtlModel<hinawa::SndUnit> for TascamModel {
    fn load(&mut self, unit: &hinawa::SndUnit, card_cntr: &mut card_cntr::CardCntr) -> Result<(), Error> {
        self.avc.fcp.bind(&unit.get_node())?;

        let mut op = UnitInfo::new();
        self.avc.status(&AvcAddr::Unit, &mut op, Self::FCP_TIMEOUT_MS)?;
        self.avc.company_id.copy_from_slice(&op.company_id);

        self.common_ctl.load(&self.avc, card_cntr, Self::FCP_TIMEOUT_MS)?;

        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Card, 0, 0, Self::DISPLAY_MODE_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, Self::DISPLAY_MODE_LABELS, None, true)?;

        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Card, 0, 0, Self::MESSAGE_MODE_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, Self::MESSAGE_MODE_LABELS, None, true)?;

        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Card, 0, 0, Self::INPUT_MODE_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, Self::INPUT_MODE_LABELS, None, true)?;

        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Card,
                                                   0, 0, Self::FIRMWARE_VERSION_NAME, 0);
        let _ = card_cntr.add_bytes_elems(&elem_id, 1, 1, None, false)?;

        Ok(())
    }

    fn read(&mut self, _: &hinawa::SndUnit, elem_id: &alsactl::ElemId,
            elem_value: &mut alsactl::ElemValue)
        -> Result<bool, Error>
    {
        if self.common_ctl.read(&self.avc, elem_id, elem_value, Self::FCP_TIMEOUT_MS)? {
            return Ok(true);
        } else {
            match elem_id.get_name().as_str() {
                Self::DISPLAY_MODE_NAME => {
                    ElemValueAccessor::<u32>::set_val(elem_value, || {
                        let mut op = TascamProto::new(&self.avc.company_id, VendorCmd::DisplayMode);
                        self.avc.status(&AvcAddr::Unit, &mut op, Self::FCP_TIMEOUT_MS)?;
                        Ok(op.val as u32)
                    })?;
                    Ok(true)
                }
                Self::MESSAGE_MODE_NAME => {
                    ElemValueAccessor::<u32>::set_val(elem_value, || {
                        let mut op = TascamProto::new(&self.avc.company_id, VendorCmd::MessageMode);
                        self.avc.status(&AvcAddr::Unit, &mut op, Self::FCP_TIMEOUT_MS)?;
                        Ok(op.val as u32)
                    })?;
                    Ok(true)
                }
                Self::INPUT_MODE_NAME => {
                    ElemValueAccessor::<u32>::set_val(elem_value, || {
                        let mut op = TascamProto::new(&self.avc.company_id, VendorCmd::InputMode);
                        self.avc.status(&AvcAddr::Unit, &mut op, Self::FCP_TIMEOUT_MS)?;
                        Ok(op.val as u32)
                    })?;
                    Ok(true)
                }
                Self::FIRMWARE_VERSION_NAME => {
                    ElemValueAccessor::<u8>::set_val(elem_value, || {
                        let mut op = TascamProto::new(&self.avc.company_id, VendorCmd::FirmwareVersion);
                        self.avc.status(&AvcAddr::Unit, &mut op, Self::FCP_TIMEOUT_MS)?;
                        Ok(op.val as u8)
                    })?;
                    Ok(true)
                }
                _ => Ok(false),
            }
        }
    }

    fn write(&mut self, unit: &hinawa::SndUnit, elem_id: &alsactl::ElemId, _: &alsactl::ElemValue,
             new: &alsactl::ElemValue) -> Result<bool, Error>
    {
        if self.common_ctl.write(unit, &self.avc, elem_id, new, Self::FCP_TIMEOUT_MS)? {
            return Ok(true);
        } else {
            match elem_id.get_name().as_str() {
                Self::DISPLAY_MODE_NAME => {
                    ElemValueAccessor::<u32>::get_val(new, |val| {
                        let mut op = TascamProto::new(&self.avc.company_id, VendorCmd::DisplayMode);
                        op.val = val as u8;
                        self.avc.control(&AvcAddr::Unit, &mut op, Self::FCP_TIMEOUT_MS)
                    })?;
                    Ok(true)
                }
                Self::MESSAGE_MODE_NAME => {
                    ElemValueAccessor::<u32>::get_val(new, |val| {
                        let mut op = TascamProto::new(&self.avc.company_id, VendorCmd::MessageMode);
                        op.val = val as u8;
                        self.avc.control(&AvcAddr::Unit, &mut op, Self::FCP_TIMEOUT_MS)
                    })?;
                    Ok(true)
                }
                Self::INPUT_MODE_NAME => {
                    ElemValueAccessor::<u32>::get_val(new, |val| {
                        let mut op = TascamProto::new(&self.avc.company_id, VendorCmd::InputMode);
                        op.val = val as u8;
                        self.avc.control(&AvcAddr::Unit, &mut op, Self::FCP_TIMEOUT_MS)
                    })?;
                    Ok(true)
                }
                _ => Ok(false),
            }
        }
    }
}

impl card_cntr::NotifyModel<hinawa::SndUnit, bool> for TascamModel {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<alsactl::ElemId>) {
        elem_id_list.extend_from_slice(&self.common_ctl.notified_elem_list);
    }

    fn parse_notification(&mut self, _: &hinawa::SndUnit, _: &bool) -> Result<(), Error> {
        Ok(())
    }

    fn read_notified_elem(&mut self, _: &hinawa::SndUnit, elem_id: &alsactl::ElemId, elem_value: &mut alsactl::ElemValue)
        -> Result<bool, Error>
    {
        self.common_ctl.read(&self.avc, elem_id, elem_value, Self::FCP_TIMEOUT_MS)
    }
}
