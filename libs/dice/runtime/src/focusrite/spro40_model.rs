// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto
use glib::{Error, FileError};

use alsactl::{ElemId, ElemIfaceType, ElemValue, ElemValueExt, ElemValueExtManual};

use hinawa::FwReq;
use hinawa::{SndDice, SndUnitExt};

use core::card_cntr::*;

use dice_protocols::tcat::{*, global_section::*};
use dice_protocols::tcat::extension::*;
use dice_protocols::focusrite::spro40::*;

use crate::common_ctl::*;
use crate::tcd22xx_ctl::*;

use super::out_grp_ctl::*;

#[derive(Default)]
pub struct SPro40Model {
    proto: FwReq,
    sections: GeneralSections,
    extension_sections: ExtensionSections,
    ctl: CommonCtl,
    tcd22xx_ctl: Tcd22xxCtl<SPro40State>,
    out_grp_ctl: OutGroupCtl,
    specific_ctl: SpecificCtl,
}

const TIMEOUT_MS: u32 = 20;

impl CtlModel<SndDice> for SPro40Model {
    fn load(&mut self, unit: &SndDice, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let node = unit.get_node();

        self.sections = self.proto.read_general_sections(&node, TIMEOUT_MS)?;
        let caps = self.proto.read_clock_caps(&node, &self.sections, TIMEOUT_MS)?;
        let src_labels = self.proto.read_clock_source_labels(&node, &self.sections, TIMEOUT_MS)?;
        self.ctl.load(card_cntr, &caps, &src_labels)?;

        self.extension_sections = self.proto.read_extension_sections(&node, TIMEOUT_MS)?;
        self.tcd22xx_ctl.load(unit, &self.proto, &self.extension_sections, &caps, &src_labels,
                          TIMEOUT_MS, card_cntr)?;

        self.tcd22xx_ctl.cache(unit, &self.proto, &self.sections, &self.extension_sections, TIMEOUT_MS)?;

        self.out_grp_ctl.load(card_cntr, unit, &self.proto, &self.extension_sections,
                              &mut self.tcd22xx_ctl.state, TIMEOUT_MS)?;
        self.specific_ctl.load(card_cntr)?;

        Ok(())
    }

    fn read(&mut self, unit: &SndDice, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        if self.ctl.read(unit, &self.proto, &self.sections, elem_id, elem_value, TIMEOUT_MS)? {
            Ok(true)
        } else if self.tcd22xx_ctl.read(unit, &self.proto, &self.extension_sections, elem_id,
                                    elem_value, TIMEOUT_MS)? {
            Ok(true)
        } else if self.out_grp_ctl.read(&self.tcd22xx_ctl.state, elem_id, elem_value)? {
            Ok(true)
        } else if self.specific_ctl.read(unit, &self.proto, &self.extension_sections, elem_id, elem_value,
                                         TIMEOUT_MS)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(&mut self, unit: &SndDice, elem_id: &ElemId, old: &ElemValue, new: &ElemValue)
        -> Result<bool, Error>
    {
        if self.ctl.write(unit, &self.proto, &self.sections, elem_id, old, new, TIMEOUT_MS)? {
            Ok(true)
        } else if self.tcd22xx_ctl.write(unit, &self.proto, &self.extension_sections, elem_id,
                                     old, new, TIMEOUT_MS)? {
            Ok(true)
        } else if self.out_grp_ctl.write(unit, &self.proto, &self.extension_sections,
                                         &mut self.tcd22xx_ctl.state, elem_id, new, TIMEOUT_MS)? {
            Ok(true)
        } else if self.specific_ctl.write(unit, &self.proto, &self.extension_sections, elem_id,
                                          new, TIMEOUT_MS)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl NotifyModel<SndDice, u32> for SPro40Model {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.ctl.notified_elem_list);
        self.tcd22xx_ctl.get_notified_elem_list(elem_id_list);
        self.out_grp_ctl.get_notified_elem_list(elem_id_list);
    }

    fn parse_notification(&mut self, unit: &SndDice, msg: &u32) -> Result<(), Error> {
        self.ctl.parse_notification(unit, &self.proto, &self.sections, *msg, TIMEOUT_MS)?;
        self.tcd22xx_ctl.parse_notification(unit, &self.proto, &self.sections,
                                        &self.extension_sections, TIMEOUT_MS, *msg)?;
        self.out_grp_ctl.parse_notification(unit, &self.proto, &self.extension_sections,
                                            &mut self.tcd22xx_ctl.state, *msg, TIMEOUT_MS)?;
        Ok(())
    }

    fn read_notified_elem(&mut self, _: &SndDice, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        if self.ctl.read_notified_elem(elem_id, elem_value)? {
            Ok(true)
        } else if self.tcd22xx_ctl.read_notified_elem(elem_id, elem_value)? {
            Ok(true)
        } else if self.out_grp_ctl.read_notified_elem(&self.tcd22xx_ctl.state, elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl MeasureModel<hinawa::SndDice> for SPro40Model {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.ctl.measured_elem_list);
        self.tcd22xx_ctl.get_measured_elem_list(elem_id_list);
    }

    fn measure_states(&mut self, unit: &SndDice) -> Result<(), Error> {
        self.ctl.measure_states(unit, &self.proto, &self.sections, TIMEOUT_MS)?;
        self.tcd22xx_ctl.measure_states(unit, &self.proto, &self.extension_sections, TIMEOUT_MS)?;
        Ok(())
    }

    fn measure_elem(&mut self, _: &SndDice, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        if self.ctl.measure_elem(elem_id, elem_value)? {
            Ok(true)
        } else if self.tcd22xx_ctl.measure_elem(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

fn opt_out_iface_mode_to_string(mode: &OptOutIfaceMode) -> String {
    match mode {
        OptOutIfaceMode::Adat => "ADAT",
        OptOutIfaceMode::Spdif => "S/PDIF",
    }.to_string()
}

#[derive(Default, Debug)]
struct SpecificCtl;

impl<'a> SpecificCtl {
    const ANALOG_OUT_0_1_PAD_NAME: &'a str = "analog-output-1/2-pad";
    const OPT_OUT_IFACE_MODE_NAME: &'a str = "optical-output-interface-mode";

    const OPT_OUT_IFACE_MODES: [OptOutIfaceMode;2] = [
        OptOutIfaceMode::Adat,
        OptOutIfaceMode::Spdif,
    ];

    pub fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::ANALOG_OUT_0_1_PAD_NAME, 0);
        card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        let labels: Vec<String> = Self::OPT_OUT_IFACE_MODES.iter()
            .map(|mode| opt_out_iface_mode_to_string(mode))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::OPT_OUT_IFACE_MODE_NAME, 0);
        card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        Ok(())
    }

    pub fn read(&mut self, unit: &SndDice, proto: &FwReq, sections: &ExtensionSections, elem_id: &ElemId,
                elem_value: &mut ElemValue, timeout_ms: u32)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::ANALOG_OUT_0_1_PAD_NAME => {
                let enabled = proto.read_analog_out_0_1_pad_offset(&unit.get_node(), sections, timeout_ms)?;
                elem_value.set_bool(&[enabled]);
                Ok(true)
            }
            Self::OPT_OUT_IFACE_MODE_NAME => {
                let mode = proto.read_opt_out_iface_mode(&unit.get_node(), sections, timeout_ms)?;
                let pos = Self::OPT_OUT_IFACE_MODES.iter()
                    .position(|m| m.eq(&mode))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub fn write(&mut self, unit: &SndDice, proto: &FwReq, sections: &ExtensionSections, elem_id: &ElemId,
                 elem_value: &ElemValue, timeout_ms: u32)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::ANALOG_OUT_0_1_PAD_NAME => {
                let mut vals = [false];
                elem_value.get_bool(&mut vals);
                proto.write_analog_out_0_1_pad_offset(&unit.get_node(), sections, vals[0], timeout_ms)
                    .map(|_| true)
            }
            Self::OPT_OUT_IFACE_MODE_NAME => {
                let mut vals = [0];
                elem_value.get_enum(&mut vals);
                let mode = Self::OPT_OUT_IFACE_MODES.iter()
                    .nth(vals[0] as usize)
                    .ok_or_else(|| {
                        let msg = format!("Invalid value for index of optical output interface mode: {}", vals[0]);
                        Error::new(FileError::Inval, &msg)
                    })?;
                proto.write_opt_out_iface_mode(&unit.get_node(), sections, *mode, timeout_ms)
                    .map(|_| true)
            }
            _ => Ok(false),
        }
    }
}
