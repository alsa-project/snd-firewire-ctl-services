// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use alsactl::{ElemId, ElemValue};

use hinawa::FwReq;
use hinawa::{SndDice, SndUnitExt};

use core::card_cntr::*;

use dice_protocols::tcat::{*, global_section::*};
use dice_protocols::tcelectronic::*;
use dice_protocols::tcelectronic::shell::klive::*;

use crate::common_ctl::*;
use super::ch_strip_ctl::*;
use super::reverb_ctl::*;
use super::shell_ctl::*;

#[derive(Default)]
pub struct KliveModel{
    proto: KliveProto,
    sections: GeneralSections,
    segments: KliveSegments,
    ctl: CommonCtl,
    ch_strip_ctl: ChStripCtl,
    reverb_ctl: ReverbCtl,
    hw_state_ctl: HwStateCtl,
}

const TIMEOUT_MS: u32 = 20;

impl CtlModel<SndDice> for KliveModel {
    fn load(&mut self, unit: &SndDice, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let node = unit.get_node();

        self.sections = self.proto.read_general_sections(&node, TIMEOUT_MS)?;
        let caps = self.proto.read_clock_caps(&node, &self.sections, TIMEOUT_MS)?;
        let src_labels = self.proto.read_clock_source_labels(&node, &self.sections, TIMEOUT_MS)?;
        self.ctl.load(card_cntr, &caps, &src_labels)?;

        self.ch_strip_ctl.load(unit, &self.proto, &mut self.segments.ch_strip_state,
                               &mut self.segments.ch_strip_meter, TIMEOUT_MS, card_cntr)?;
        self.reverb_ctl.load(unit, &self.proto, &mut self.segments.reverb_state, &mut self.segments.reverb_meter,
                             TIMEOUT_MS, card_cntr)?;

        let node = unit.get_node();
        self.proto.read_segment(&node, &mut self.segments.hw_state, TIMEOUT_MS)?;

        self.hw_state_ctl.load(card_cntr)?;

        Ok(())
    }

    fn read(&mut self, unit: &SndDice, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        if self.ctl.read(unit, &self.proto, &self.sections, elem_id, elem_value, TIMEOUT_MS)? {
            Ok(true)
        } else if self.ch_strip_ctl.read(&self.segments.ch_strip_state, &self.segments.ch_strip_meter,
                                         elem_id, elem_value)? {
            Ok(true)
        } else if self.reverb_ctl.read(&self.segments.reverb_state, &self.segments.reverb_meter,
                                       elem_id, elem_value)? {
            Ok(true)
        } else if self.hw_state_ctl.read(&self.segments.hw_state, elem_id, elem_value)? {
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
        } else if self.ch_strip_ctl.write(unit, &self.proto, &mut self.segments.ch_strip_state, elem_id,
                                          old, new, TIMEOUT_MS)? {
            Ok(true)
        } else if self.reverb_ctl.write(unit, &self.proto, &mut self.segments.reverb_state, elem_id,
                                        new, TIMEOUT_MS)? {
            Ok(true)
        } else if self.hw_state_ctl.write(unit, &self.proto, &mut self.segments.hw_state, elem_id,
                                          new, TIMEOUT_MS)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl NotifyModel<SndDice, u32> for KliveModel {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.ctl.notified_elem_list);
        elem_id_list.extend_from_slice(&self.ch_strip_ctl.notified_elem_list);
        elem_id_list.extend_from_slice(&self.reverb_ctl.notified_elem_list);
        elem_id_list.extend_from_slice(&self.hw_state_ctl.notified_elem_list);
    }

    fn parse_notification(&mut self, unit: &SndDice, msg: &u32) -> Result<(), Error> {
        self.ctl.parse_notification(unit, &self.proto, &self.sections, *msg, TIMEOUT_MS)?;

        let node = unit.get_node();
        self.proto.parse_notification(&node, &mut self.segments.ch_strip_state, TIMEOUT_MS, *msg)?;
        self.proto.parse_notification(&node, &mut self.segments.reverb_state, TIMEOUT_MS, *msg)?;
        self.proto.parse_notification(&node, &mut self.segments.hw_state, TIMEOUT_MS, *msg)?;
        Ok(())
    }

    fn read_notified_elem(&mut self, _: &SndDice, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        if self.ctl.read_notified_elem(elem_id, elem_value)? {
            Ok(true)
        } else if self.ch_strip_ctl.read_notified_elem(&self.segments.ch_strip_state, elem_id, elem_value)? {
            Ok(true)
        } else if self.reverb_ctl.read_notified_elem(&self.segments.reverb_state, elem_id, elem_value)? {
            Ok(true)
        } else if self.hw_state_ctl.read(&self.segments.hw_state, elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl MeasureModel<hinawa::SndDice> for KliveModel {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.ctl.measured_elem_list);
        elem_id_list.extend_from_slice(&self.ch_strip_ctl.measured_elem_list);
        elem_id_list.extend_from_slice(&self.reverb_ctl.measured_elem_list);
    }

    fn measure_states(&mut self, unit: &SndDice) -> Result<(), Error> {
        self.ctl.measure_states(unit, &self.proto, &self.sections, TIMEOUT_MS)?;
        self.ch_strip_ctl.measure_states(unit, &self.proto, &self.segments.ch_strip_state,
                                         &mut self.segments.ch_strip_meter, TIMEOUT_MS)?;
        self.reverb_ctl.measure_states(unit, &self.proto, &self.segments.reverb_state,
                                       &mut self.segments.reverb_meter, TIMEOUT_MS)?;
        Ok(())
    }

    fn measure_elem(&mut self, _: &SndDice, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        if self.ctl.measure_elem(elem_id, elem_value)? {
            Ok(true)
        } else if self.ch_strip_ctl.read_measured_elem(&self.segments.ch_strip_meter, elem_id, elem_value)? {
            Ok(true)
        } else if self.reverb_ctl.read_measured_elem(&self.segments.reverb_meter, elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[derive(Default, Debug)]
struct KliveProto(FwReq);

impl AsRef<FwReq> for KliveProto {
    fn as_ref(&self) -> &FwReq {
        &self.0
    }
}
