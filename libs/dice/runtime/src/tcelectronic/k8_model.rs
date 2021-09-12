// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use alsactl::{ElemId, ElemIfaceType, ElemValue};

use hinawa::FwReq;
use hinawa::{SndDice, SndUnitExt};

use core::card_cntr::*;
use core::elem_value_accessor::*;

use dice_protocols::tcat::{*, global_section::*};
use dice_protocols::tcelectronic::*;
use dice_protocols::tcelectronic::shell::k8::*;

use crate::common_ctl::*;
use super::shell_ctl::*;

#[derive(Default)]
pub struct K8Model{
    req: FwReq,
    sections: GeneralSections,
    segments: K8Segments,
    ctl: CommonCtl,
    hw_state_ctl: HwStateCtl,
    mixer_ctl: ShellMixerCtl,
    standalone_ctl: ShellStandaloneCtl,
    coax_iface_ctl: ShellCoaxIfaceCtl,
    knob_ctl: ShellKnobCtl,
    knob2_ctl: ShellKnob2Ctl,
    specific_ctl: K8SpecificCtl,
}

const TIMEOUT_MS: u32 = 20;

impl CtlModel<SndDice> for K8Model {
    fn load(&mut self, unit: &mut SndDice, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let mut node = unit.get_node();

        self.sections = self.req.read_general_sections(&mut node, TIMEOUT_MS)?;
        let caps = self.req.read_clock_caps(&mut node, &self.sections, TIMEOUT_MS)?;
        let src_labels = self.req.read_clock_source_labels(&mut node, &self.sections, TIMEOUT_MS)?;
        self.ctl.load(card_cntr, &caps, &src_labels)?;

        self.req.read_segment(&mut node, &mut self.segments.hw_state, TIMEOUT_MS)?;
        self.req.read_segment(&mut node, &mut self.segments.mixer_state, TIMEOUT_MS)?;
        self.req.read_segment(&mut node, &mut self.segments.config, TIMEOUT_MS)?;
        self.req.read_segment(&mut node, &mut self.segments.knob, TIMEOUT_MS)?;

        self.hw_state_ctl.load(card_cntr)?;
        self.mixer_ctl.load(&self.segments.mixer_state, &self.segments.mixer_meter, card_cntr)?;
        self.standalone_ctl.load(&self.segments.config, card_cntr)?;
        self.coax_iface_ctl.load(card_cntr)?;
        self.knob_ctl.load(&self.segments.knob, card_cntr)?;
        self.knob2_ctl.load(&self.segments.knob, card_cntr)?;
        self.specific_ctl.load(card_cntr)?;

        Ok(())
    }

    fn read(&mut self, unit: &mut SndDice, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        if self.ctl.read(unit, &mut self.req, &self.sections, elem_id, elem_value, TIMEOUT_MS)? {
            Ok(true)
        } else if self.hw_state_ctl.read(&self.segments.hw_state, elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_ctl.read(&self.segments.mixer_state, &self.segments.mixer_meter, elem_id,
                                      elem_value)? {
            Ok(true)
        } else if self.standalone_ctl.read(&self.segments.config, elem_id, elem_value)? {
            Ok(true)
        } else if self.coax_iface_ctl.read(&self.segments.config, elem_id, elem_value)? {
            Ok(true)
        } else if self.knob_ctl.read(&self.segments.knob, elem_id, elem_value)? {
            Ok(true)
        } else if self.knob2_ctl.read(&self.segments.knob, elem_id, elem_value)? {
            Ok(true)
        } else if self.specific_ctl.read(&self.segments, elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(&mut self, unit: &mut SndDice, elem_id: &ElemId, old: &ElemValue, new: &ElemValue)
        -> Result<bool, Error>
    {
        if self.ctl.write(unit, &mut self.req, &self.sections, elem_id, old, new, TIMEOUT_MS)? {
            Ok(true)
        } else if self.hw_state_ctl.write(unit, &mut self.req, &mut self.segments.hw_state, elem_id,
                                          new, TIMEOUT_MS)? {
            Ok(true)
        } else if self.mixer_ctl.write(unit, &mut self.req, &mut self.segments.mixer_state, elem_id,
                                       old, new, TIMEOUT_MS)? {
            Ok(true)
        } else if self.standalone_ctl.write(unit, &mut self.req, &mut self.segments.config, elem_id, new,
                                            TIMEOUT_MS)? {
            Ok(true)
        } else if self.coax_iface_ctl.write(unit, &mut self.req, &mut self.segments.config, elem_id, new,
                                            TIMEOUT_MS)? {
            Ok(true)
        } else if self.knob_ctl.write(unit, &mut self.req, &mut self.segments.knob, elem_id, new,
                                      TIMEOUT_MS)? {
            Ok(true)
        } else if self.knob2_ctl.write(unit, &mut self.req, &mut self.segments.knob, elem_id, new,
                                       TIMEOUT_MS)? {
            Ok(true)
        } else if self.specific_ctl.write(unit, &mut self.req, &mut self.segments, elem_id, new,
                                          TIMEOUT_MS)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl NotifyModel<SndDice, u32> for K8Model {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.ctl.notified_elem_list);
        elem_id_list.extend_from_slice(&self.hw_state_ctl.notified_elem_list);
        elem_id_list.extend_from_slice(&self.mixer_ctl.notified_elem_list);
        elem_id_list.extend_from_slice(&self.knob_ctl.notified_elem_list);
        elem_id_list.extend_from_slice(&self.specific_ctl.0);
    }

    fn parse_notification(&mut self, unit: &mut SndDice, msg: &u32) -> Result<(), Error> {
        self.ctl.parse_notification(unit, &mut self.req, &self.sections, *msg, TIMEOUT_MS)?;

        let mut node = unit.get_node();
        self.req.parse_notification(&mut node, &mut self.segments.hw_state, TIMEOUT_MS, *msg)?;
        self.req.parse_notification(&mut node, &mut self.segments.mixer_state, TIMEOUT_MS, *msg)?;
        self.req.parse_notification(&mut node, &mut self.segments.config, TIMEOUT_MS, *msg)?;
        self.req.parse_notification(&mut node, &mut self.segments.knob, TIMEOUT_MS, *msg)?;
        Ok(())
    }

    fn read_notified_elem(&mut self, _: &SndDice, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        if self.ctl.read_notified_elem(elem_id, elem_value)? {
            Ok(true)
        } else if self.hw_state_ctl.read(&self.segments.hw_state, elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_ctl.read_notified_elem(&self.segments.mixer_state, elem_id, elem_value)? {
            Ok(true)
        } else if self.knob_ctl.read(&self.segments.knob, elem_id, elem_value)? {
            Ok(true)
        } else if self.specific_ctl.read_notified_elem(&self.segments, elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl MeasureModel<SndDice> for K8Model {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.ctl.measured_elem_list);
        elem_id_list.extend_from_slice(&self.mixer_ctl.measured_elem_list);
    }

    fn measure_states(&mut self, unit: &mut SndDice) -> Result<(), Error> {
        self.ctl.measure_states(unit, &mut self.req, &self.sections, TIMEOUT_MS)?;

        self.req.read_segment(&mut unit.get_node(), &mut self.segments.mixer_meter, TIMEOUT_MS)?;
        Ok(())
    }

    fn measure_elem(&mut self, _: &SndDice, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        if self.ctl.measure_elem(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_ctl.read_measured_elem(&self.segments.mixer_meter, elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[derive(Default, Debug)]
struct K8SpecificCtl(Vec<ElemId>);

impl K8SpecificCtl {
    const MIXER_ENABLE_NAME: &'static str = "mixer-enable";
    const AUX_IN_ENABLED_NAME: &'static str = "aux-input-enable";

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::MIXER_ENABLE_NAME, 0);
        card_cntr.add_bool_elems(&elem_id, 1, 1, true)
            .map(|mut elem_id_list| self.0.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::AUX_IN_ENABLED_NAME, 0);
        card_cntr.add_bool_elems(&elem_id, 1, 1, false)
            .map(|mut elem_id_list| self.0.append(&mut elem_id_list))?;

        Ok(())
    }

    fn read(
        &mut self,
        segments: &K8Segments,
        elem_id: &ElemId,
        elem_value: &mut ElemValue
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            Self::MIXER_ENABLE_NAME => {
                ElemValueAccessor::<bool>::set_val(elem_value, || {
                    Ok(segments.mixer_state.data.enabled)
                })
                .map(|_| true)
            }
            _ => self.read_notified_elem(segments, elem_id, elem_value),
        }
    }

    fn write(
        &mut self,
        unit: &mut SndDice,
        req: &mut FwReq,
        segments: &mut K8Segments,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            Self::MIXER_ENABLE_NAME => {
                ElemValueAccessor::<bool>::get_val(elem_value, |val| {
                    segments.mixer_state.data.enabled = val;
                    req.write_segment(&mut unit.get_node(), &mut segments.mixer_state, timeout_ms)
                })
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }

    fn read_notified_elem(
        &mut self,
        segments: &K8Segments,
        elem_id: &ElemId,
        elem_value: &mut ElemValue
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            Self::AUX_IN_ENABLED_NAME => {
                ElemValueAccessor::<bool>::set_val(elem_value, || {
                    Ok(segments.hw_state.data.aux_input_enabled)
                })
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }
}
