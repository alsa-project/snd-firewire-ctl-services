// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::{Error, FileError};

use alsactl::{ElemId, ElemIfaceType, ElemValue, ElemValueExtManual};

use hinawa::FwReq;
use hinawa::{SndDice, SndUnitExt};

use core::card_cntr::*;
use core::elem_value_accessor::*;

use dice_protocols::tcat::{*, global_section::*};
use dice_protocols::tcat::extension::*;
use dice_protocols::tcat::tcd22xx_spec::*;
use dice_protocols::maudio::*;

use super::common_ctl::*;
use super::tcd22xx_ctl::*;

#[derive(Default)]
pub struct PfireModel<S>
    where for<'a> S: AsRef<Tcd22xxState> + AsMut<Tcd22xxState> + Tcd22xxSpec<'a> + PfireClkSpec<'a>,
{
    proto: FwReq,
    sections: GeneralSections,
    extension_sections: ExtensionSections,
    ctl: CommonCtl,
    tcd22xx_ctl: Tcd22xxCtl<S>,
    specific_ctl: SpecificCtl,
}

pub type Pfire2626Model = PfireModel<Pfire2626State>;
pub type Pfire610Model = PfireModel<Pfire610State>;

const TIMEOUT_MS: u32 = 20;

impl<S> CtlModel<SndDice> for PfireModel<S>
    where for<'a> S: AsRef<Tcd22xxState> + AsMut<Tcd22xxState> + Tcd22xxSpec<'a> + PfireClkSpec<'a>,
{
    fn load(&mut self, unit: &mut SndDice, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let node = unit.get_node();

        self.sections = self.proto.read_general_sections(&node, TIMEOUT_MS)?;
        let caps = ClockCaps::new(S::AVAIL_CLK_RATES, S::AVAIL_CLK_SRCS);
        let src_labels = self.proto.read_clock_source_labels(&node, &self.sections, TIMEOUT_MS)?;
        self.ctl.load(card_cntr, &caps, &src_labels)?;

        self.extension_sections = self.proto.read_extension_sections(&node, TIMEOUT_MS)?;
        self.tcd22xx_ctl.load(unit, &self.proto, &self.extension_sections, &caps, &src_labels,
                          TIMEOUT_MS, card_cntr)?;
        self.specific_ctl.load(&caps, &src_labels, card_cntr)?;

        self.tcd22xx_ctl.cache(unit, &self.proto, &self.sections, &self.extension_sections, TIMEOUT_MS)?;

        Ok(())
    }

    fn read(&mut self, unit: &mut SndDice, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        if self.ctl.read(unit, &self.proto, &self.sections, elem_id, elem_value, TIMEOUT_MS)? {
            Ok(true)
        } else if self.tcd22xx_ctl.read(unit, &self.proto, &self.extension_sections, elem_id,
                                    elem_value, TIMEOUT_MS)? {
            Ok(true)
        } else if self.specific_ctl.read(unit, &self.proto, &self.extension_sections, elem_id,
                                         elem_value, TIMEOUT_MS)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(&mut self, unit: &mut SndDice, elem_id: &ElemId, old: &ElemValue, new: &ElemValue)
        -> Result<bool, Error>
    {
        if self.ctl.write(unit, &self.proto, &self.sections, elem_id, old, new, TIMEOUT_MS)? {
            Ok(true)
        } else if self.tcd22xx_ctl.write(unit, &self.proto, &self.extension_sections, elem_id,
                                     old, new, TIMEOUT_MS)? {
            Ok(true)
        } else if self.specific_ctl.write(unit, &self.proto, &self.extension_sections, elem_id,
                                          old, new, TIMEOUT_MS)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl<S> NotifyModel<SndDice, u32> for PfireModel<S>
    where for<'a> S: AsRef<Tcd22xxState> + AsMut<Tcd22xxState> + Tcd22xxSpec<'a> + PfireClkSpec<'a>,
{
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.ctl.notified_elem_list);
        self.tcd22xx_ctl.get_notified_elem_list(elem_id_list);
    }

    fn parse_notification(&mut self, unit: &mut SndDice, msg: &u32) -> Result<(), Error> {
        self.ctl.parse_notification(unit, &self.proto, &self.sections, *msg, TIMEOUT_MS)?;
        self.tcd22xx_ctl.parse_notification(unit, &self.proto, &self.sections,
                                        &self.extension_sections, TIMEOUT_MS, *msg)?;
        Ok(())
    }

    fn read_notified_elem(&mut self, _: &SndDice, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        if self.ctl.read_notified_elem(elem_id, elem_value)? {
            Ok(true)
        } else if self.tcd22xx_ctl.read_notified_elem(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl<S> MeasureModel<hinawa::SndDice> for PfireModel<S>
    where for<'a> S: AsRef<Tcd22xxState> + AsMut<Tcd22xxState> + Tcd22xxSpec<'a> + PfireClkSpec<'a>,
{
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.ctl.measured_elem_list);
        self.tcd22xx_ctl.get_measured_elem_list(elem_id_list);
    }

    fn measure_states(&mut self, unit: &mut SndDice) -> Result<(), Error> {
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

#[derive(Default, Debug)]
struct SpecificCtl{
    targets: [bool;KNOB_COUNT],
}

impl<'a> SpecificCtl {
    const MASTER_KNOB_NAME: &'a str = "master-knob-target";
    const OPT_IFACE_B_MODE_NAME: &'a str = "optical-iface-b-mode";
    const STANDALONE_CONVERTER_MODE_NAME: &'a str = "standalone-converter-mode";

    // MEMO: Both models support 'Output{id: DstBlkId::Ins0, count: 8}'.
    const MASTER_KNOB_TARGET_LABELS: &'a [&'a str] = &[
        "analog-out-1/2",
        "analog-out-3/4",
        "analog-out-5/6",
        "analog-out-7/8",
    ];
    const OPT_IFACE_B_MODE_LABELS: &'a [&'a str] = &["ADAT", "S/PDIF"];
    const STANDALONE_CONVERTER_MODE_LABELS: &'a [&'a str] = &["A/D-D/A", "A/D-only"];

    fn load(&self, caps: &ClockCaps, src_labels: &ClockSourceLabels, card_cntr: &mut CardCntr)
        -> Result<(), Error>
    {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::MASTER_KNOB_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, Self::MASTER_KNOB_TARGET_LABELS.len(), true)?;

        // NOTE: ClockSource::Tdif is used for second optical interface as 'ADAT_AUX'.
        if ClockSource::Tdif.is_supported(caps, src_labels) {
            let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::OPT_IFACE_B_MODE_NAME, 0);
            let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, Self::OPT_IFACE_B_MODE_LABELS, None, true)?;

            let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::STANDALONE_CONVERTER_MODE_NAME, 0);
            let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, Self::STANDALONE_CONVERTER_MODE_LABELS, None, true)?;
        }

        Ok(())
    }

    fn read(&self, unit: &SndDice, proto: &FwReq, sections: &ExtensionSections, elem_id: &ElemId,
            elem_value: &ElemValue, timeout_ms: u32)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::MASTER_KNOB_NAME => {
                let mut assigns = [false;KNOB_COUNT];
                proto.read_knob_assign(&unit.get_node(), sections, &mut assigns, timeout_ms)?;
                elem_value.set_bool(&self.targets);
                Ok(true)
            }
            Self::OPT_IFACE_B_MODE_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    proto.read_opt_iface_b_mode(&unit.get_node(), sections, timeout_ms)
                        .map(|mode| {
                            match mode {
                                OptIfaceMode::Adat => 0,
                                OptIfaceMode::Spdif => 1,
                            }
                        })
                })
                .map(|_| true)
            }
            Self::STANDALONE_CONVERTER_MODE_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    proto.read_standalone_converter_mode(&unit.get_node(), sections, timeout_ms)
                        .map(|mode| {
                            match mode {
                                StandaloneConerterMode::AdDa => 0,
                                StandaloneConerterMode::AdOnly => 1,
                            }
                        })
                })
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }

    fn write(&mut self, unit: &SndDice, proto: &FwReq, sections: &ExtensionSections,
             elem_id: &ElemId, old: &ElemValue, new: &ElemValue, timeout_ms: u32)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::MASTER_KNOB_NAME => {
                ElemValueAccessor::<bool>::get_vals(new, old, self.targets.len(), |idx, val| {
                    self.targets[idx] = val;
                    Ok(())
                })?;
                let node = unit.get_node();
                proto.write_knob_assign(&node, sections, &self.targets, timeout_ms)?;
                Ok(true)
            }
            Self::OPT_IFACE_B_MODE_NAME => {
                ElemValueAccessor::<u32>::get_val(new, |val| {
                    let mode = match val {
                        0 => OptIfaceMode::Adat,
                        1 => OptIfaceMode::Spdif,
                        _ => {
                            let msg = format!("Invalid value for index of optical interface mode: {}", val);
                            Err(Error::new(FileError::Inval, &msg))?
                        }
                    };
                    proto.write_opt_iface_b_mode(&unit.get_node(), sections, mode, timeout_ms)
                })
                .map(|_| true)
            }
            Self::STANDALONE_CONVERTER_MODE_NAME => {
                ElemValueAccessor::<u32>::get_val(new, |val| {
                    let mode = match val {
                        0 => StandaloneConerterMode::AdDa,
                        1 => StandaloneConerterMode::AdOnly,
                        _ => {
                            let msg = format!("Invalid value for index of standalone converter mode: {}", val);
                            Err(Error::new(FileError::Inval, &msg))?
                        }
                    };
                    proto.write_standalone_converter_mode(&unit.get_node(), sections, mode, timeout_ms)
                })
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }
}
