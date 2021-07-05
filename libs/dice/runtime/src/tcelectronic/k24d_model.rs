// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::{Error, FileError};

use alsactl::{ElemId, ElemIfaceType, ElemValue};

use hinawa::FwReq;
use hinawa::{SndDice, SndUnitExt};

use core::card_cntr::*;
use core::elem_value_accessor::*;

use dice_protocols::tcat::{*, global_section::*};
use dice_protocols::tcelectronic::*;
use dice_protocols::tcelectronic::shell::k24d::*;

use crate::common_ctl::*;
use super::ch_strip_ctl::*;
use super::reverb_ctl::*;
use super::shell_ctl::*;
use super::prog_ctl::*;

#[derive(Default)]
pub struct K24dModel{
    proto: K24dProto,
    sections: GeneralSections,
    segments: K24dSegments,
    ctl: CommonCtl,
    ch_strip_ctl: ChStripCtl,
    reverb_ctl: ReverbCtl,
    hw_state_ctl: HwStateCtl,
    mixer_ctl: ShellMixerCtl,
    reverb_return_ctl: ShellReverbReturnCtl,
    standalone_ctl: ShellStandaloneCtl,
    coax_iface_ctl: ShellCoaxIfaceCtl,
    opt_iface_ctl: ShellOptIfaceCtl,
    knob_ctl: ShellKnobCtl,
    knob2_ctl: ShellKnob2Ctl,
    prog_ctl: TcKonnektProgramCtl,
    specific_ctl: K24dSpecificCtl,
}

const TIMEOUT_MS: u32 = 20;

impl CtlModel<SndDice> for K24dModel {
    fn load(&mut self, unit: &mut SndDice, card_cntr: &mut CardCntr) -> Result<(), Error> {
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
        self.proto.read_segment(&node, &mut self.segments.mixer_state, TIMEOUT_MS)?;
        self.proto.read_segment(&node, &mut self.segments.config, TIMEOUT_MS)?;
        self.proto.read_segment(&node, &mut self.segments.knob, TIMEOUT_MS)?;

        self.hw_state_ctl.load(card_cntr)?;
        self.mixer_ctl.load(&self.segments.mixer_state, &self.segments.mixer_meter, card_cntr)?;
        self.reverb_return_ctl.load(card_cntr)?;
        self.standalone_ctl.load(&self.segments.config, card_cntr)?;
        self.coax_iface_ctl.load(card_cntr)?;
        self.opt_iface_ctl.load(card_cntr)?;
        self.knob_ctl.load(&self.segments.knob, card_cntr)?;
        self.knob2_ctl.load(&self.segments.knob, card_cntr)?;
        self.prog_ctl.load(card_cntr)?;
        self.specific_ctl.load(card_cntr)?;

        Ok(())
    }

    fn read(&mut self, unit: &mut SndDice, elem_id: &ElemId, elem_value: &mut ElemValue)
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
        } else if self.mixer_ctl.read(&self.segments.mixer_state, &self.segments.mixer_meter, elem_id,
                                      elem_value)? {
            Ok(true)
        } else if self.reverb_return_ctl.read(&self.segments.mixer_state, elem_id, elem_value)? {
            Ok(true)
        } else if self.standalone_ctl.read(&self.segments.config, elem_id, elem_value)? {
            Ok(true)
        } else if self.coax_iface_ctl.read(&self.segments.config, elem_id, elem_value)? {
            Ok(true)
        } else if self.opt_iface_ctl.read(&self.segments.config, elem_id, elem_value)? {
            Ok(true)
        } else if self.knob_ctl.read(&self.segments.knob, elem_id, elem_value)? {
            Ok(true)
        } else if self.knob2_ctl.read(&self.segments.knob, elem_id, elem_value)? {
            Ok(true)
        } else if self.prog_ctl.read(&self.segments.knob, elem_id, elem_value)? {
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
        if self.ctl.write(unit, &self.proto, &self.sections, elem_id, old, new, TIMEOUT_MS)? {
            Ok(true)
        } else if self.ch_strip_ctl.write(unit, &self.proto, &mut self.segments.ch_strip_state, elem_id,
                                          old, new, TIMEOUT_MS)? {
            Ok(true)
        } else if self.reverb_ctl.write(unit, &self.proto, &mut self.segments.reverb_state, elem_id,
                                        new, TIMEOUT_MS)? {
            Ok(true)
        } else if self.hw_state_ctl.write(unit, &self.proto, &mut self.segments.hw_state, elem_id, new, TIMEOUT_MS)? {
            Ok(true)
        } else if self.mixer_ctl.write(unit, &self.proto, &mut self.segments.mixer_state, elem_id,
                                       old, new, TIMEOUT_MS)? {
            Ok(true)
        } else if self.reverb_return_ctl.write(unit, &self.proto, &mut self.segments.mixer_state, elem_id, new,
                                               TIMEOUT_MS)? {
            Ok(true)
        } else if self.standalone_ctl.write(unit, &self.proto, &mut self.segments.config, elem_id, new,
                                            TIMEOUT_MS)? {
            Ok(true)
        } else if self.coax_iface_ctl.write(unit, &self.proto, &mut self.segments.config, elem_id, new,
                                            TIMEOUT_MS)? {
            Ok(true)
        } else if self.opt_iface_ctl.write(unit, &self.proto, &mut self.segments.config, elem_id, new,
                                           TIMEOUT_MS)? {
            Ok(true)
        } else if self.knob_ctl.write(unit, &self.proto, &mut self.segments.knob, elem_id, new,
                                      TIMEOUT_MS)? {
            Ok(true)
        } else if self.knob2_ctl.write(unit, &self.proto, &mut self.segments.knob, elem_id, new,
                                       TIMEOUT_MS)? {
            Ok(true)
        } else if self.prog_ctl.write(unit, &self.proto, &mut self.segments.knob, elem_id, new,
                                      TIMEOUT_MS)? {
            Ok(true)
        } else if self.specific_ctl.write(unit, &self.proto, &mut self.segments, elem_id, new,
                                          TIMEOUT_MS)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl NotifyModel<SndDice, u32> for K24dModel {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.ctl.notified_elem_list);
        elem_id_list.extend_from_slice(&self.ch_strip_ctl.notified_elem_list);
        elem_id_list.extend_from_slice(&self.reverb_ctl.notified_elem_list);
        elem_id_list.extend_from_slice(&self.hw_state_ctl.notified_elem_list);
        elem_id_list.extend_from_slice(&self.mixer_ctl.notified_elem_list);
        elem_id_list.extend_from_slice(&self.reverb_return_ctl.0);
        elem_id_list.extend_from_slice(&self.knob_ctl.notified_elem_list);
        elem_id_list.extend_from_slice(&self.prog_ctl.0);
    }

    fn parse_notification(&mut self, unit: &SndDice, msg: &u32) -> Result<(), Error> {
        self.ctl.parse_notification(unit, &self.proto, &self.sections, *msg, TIMEOUT_MS)?;

        let node = unit.get_node();
        self.proto.parse_notification(&node, &mut self.segments.ch_strip_state, TIMEOUT_MS, *msg)?;
        self.proto.parse_notification(&node, &mut self.segments.reverb_state, TIMEOUT_MS, *msg)?;
        self.proto.parse_notification(&node, &mut self.segments.hw_state, TIMEOUT_MS, *msg)?;
        self.proto.parse_notification(&node, &mut self.segments.mixer_state, TIMEOUT_MS, *msg)?;
        self.proto.parse_notification(&node, &mut self.segments.config, TIMEOUT_MS, *msg)?;
        self.proto.parse_notification(&node, &mut self.segments.knob, TIMEOUT_MS, *msg)?;
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
        } else if self.mixer_ctl.read_notified_elem(&self.segments.mixer_state, elem_id, elem_value)? {
            Ok(true)
        } else if self.reverb_return_ctl.read_notified_elem(&self.segments.mixer_state, elem_id, elem_value)? {
            Ok(true)
        } else if self.knob_ctl.read(&self.segments.knob, elem_id, elem_value)? {
            Ok(true)
        } else if self.prog_ctl.read(&self.segments.knob, elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl MeasureModel<hinawa::SndDice> for K24dModel {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.ctl.measured_elem_list);
        elem_id_list.extend_from_slice(&self.ch_strip_ctl.measured_elem_list);
        elem_id_list.extend_from_slice(&self.reverb_ctl.measured_elem_list);
        elem_id_list.extend_from_slice(&self.mixer_ctl.measured_elem_list);
    }

    fn measure_states(&mut self, unit: &mut SndDice) -> Result<(), Error> {
        self.ctl.measure_states(unit, &self.proto, &self.sections, TIMEOUT_MS)?;
        self.ch_strip_ctl.measure_states(unit, &self.proto, &self.segments.ch_strip_state,
                                         &mut self.segments.ch_strip_meter, TIMEOUT_MS)?;
        self.reverb_ctl.measure_states(unit, &self.proto, &self.segments.reverb_state,
                                       &mut self.segments.reverb_meter, TIMEOUT_MS)?;
        self.proto.read_segment(&unit.get_node(), &mut self.segments.mixer_meter, TIMEOUT_MS)?;
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
        } else if self.mixer_ctl.read_measured_elem(&self.segments.mixer_meter, elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
#[derive(Default, Debug)]
struct K24dProto(FwReq);

impl AsRef<FwReq> for K24dProto {
    fn as_ref(&self) -> &FwReq {
        &self.0
    }
}

#[derive(Default, Debug)]
struct K24dSpecificCtl(Vec<ElemId>);

impl<'a> K24dSpecificCtl {
    const OUT_23_SRC_NAME: &'a str = "output-3/4-source";
    const USE_CH_STRIP_AS_PLUGIN_NAME: &'a str = "use-channel-strip-as-plugin";
    const USE_REVERB_AT_MID_RATE: &'a str = "use-reverb-at-mid-rate";
    const MIXER_ENABLE_NAME: &'a str = "mixer-enable";

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let labels: Vec<String> = PHYS_OUT_SRCS.iter()
            .map(|s| phys_out_src_to_string(s))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::OUT_23_SRC_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::USE_CH_STRIP_AS_PLUGIN_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::USE_REVERB_AT_MID_RATE, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::MIXER_ENABLE_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        Ok(())
    }

    fn read(&mut self, segments: &K24dSegments, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::OUT_23_SRC_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let pos = PHYS_OUT_SRCS.iter()
                        .position(|s| s.eq(&segments.config.data.out_23_src))
                        .expect("Programming error...");
                    Ok(pos as u32)
                })
                .map(|_| true)
            }
            Self::USE_CH_STRIP_AS_PLUGIN_NAME => {
                ElemValueAccessor::<bool>::set_val(elem_value, || {
                    Ok(segments.mixer_state.data.use_ch_strip_as_plugin)
                })
                .map(|_| true)
            }
            Self::USE_REVERB_AT_MID_RATE => {
                ElemValueAccessor::<bool>::set_val(elem_value, || {
                    Ok(segments.mixer_state.data.use_reverb_at_mid_rate)
                })
                .map(|_| true)
            }
            Self::MIXER_ENABLE_NAME => {
                ElemValueAccessor::<bool>::set_val(elem_value, || {
                    Ok(segments.mixer_state.data.enabled)
                })
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }

    fn write(&mut self, unit: &SndDice, proto: &K24dProto, segments: &mut K24dSegments, elem_id: &ElemId,
             elem_value: &ElemValue, timeout_ms: u32)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::OUT_23_SRC_NAME => {
                ElemValueAccessor::<u32>::get_val(elem_value, |val| {
                    PHYS_OUT_SRCS.iter()
                        .nth(val as usize)
                        .ok_or_else(|| {
                            let msg = format!("Invalid value for index of output source: {}", val);
                            Error::new(FileError::Inval, &msg)
                        })
                        .and_then(|&s| {
                            segments.config.data.out_23_src = s;
                            proto.write_segment(&unit.get_node(), &mut segments.config, timeout_ms)
                        })
                })
                .map(|_| true)
            }
            Self::USE_CH_STRIP_AS_PLUGIN_NAME => {
                ElemValueAccessor::<bool>::get_val(elem_value, |val| {
                    segments.mixer_state.data.use_ch_strip_as_plugin = val;
                    proto.write_segment(&unit.get_node(), &mut segments.mixer_state, timeout_ms)
                })
                .map(|_| true)
            }
            Self::USE_REVERB_AT_MID_RATE => {
                ElemValueAccessor::<bool>::get_val(elem_value, |val| {
                    segments.mixer_state.data.use_reverb_at_mid_rate = val;
                    proto.write_segment(&unit.get_node(), &mut segments.mixer_state, timeout_ms)
                })
                .map(|_| true)
            }
            Self::MIXER_ENABLE_NAME => {
                ElemValueAccessor::<bool>::get_val(elem_value, |val| {
                    segments.mixer_state.data.enabled = val;
                    proto.write_segment(&unit.get_node(), &mut segments.mixer_state, timeout_ms)
                })
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }
}
