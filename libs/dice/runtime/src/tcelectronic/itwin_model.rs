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
use dice_protocols::tcelectronic::shell::itwin::*;

use crate::common_ctl::*;
use super::ch_strip_ctl::*;
use super::reverb_ctl::*;
use super::shell_ctl::*;

#[derive(Default)]
pub struct ItwinModel{
    proto: ItwinProto,
    sections: GeneralSections,
    segments: ItwinSegments,
    ctl: CommonCtl,
    ch_strip_ctl: ChStripCtl,
    reverb_ctl: ReverbCtl,
    hw_state_ctl: HwStateCtl,
    mixer_ctl: ShellMixerCtl,
    mixer_stream_src_pair_ctl: MixerStreamSrcPairCtl,
    standalone_ctl: ShellStandaloneCtl,
    knob_ctl: ShellKnobCtl,
    specific_ctl: ItwinSpecificCtl,
}

const TIMEOUT_MS: u32 = 20;

impl CtlModel<SndDice> for ItwinModel {
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
        self.mixer_stream_src_pair_ctl.load(&mut self.segments.config, card_cntr)?;
        self.standalone_ctl.load(&self.segments.config, card_cntr)?;
        self.knob_ctl.load(&self.segments.knob, card_cntr)?;
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
        } else if self.mixer_stream_src_pair_ctl.read(&self.segments.config, elem_id, elem_value)? {
            Ok(true)
        } else if self.standalone_ctl.read(&self.segments.config, elem_id, elem_value)? {
            Ok(true)
        } else if self.knob_ctl.read(&self.segments.knob, elem_id, elem_value)? {
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
        } else if self.hw_state_ctl.write(unit, &self.proto, &mut self.segments.hw_state, elem_id,
                                          new, TIMEOUT_MS)? {
            Ok(true)
        } else if self.mixer_ctl.write(unit, &self.proto, &mut self.segments.mixer_state, elem_id, old, new,
                                       TIMEOUT_MS)? {
            Ok(true)
        } else if self.mixer_stream_src_pair_ctl.write(unit, &self.proto, &mut self.segments.config, elem_id,
                                                       new, TIMEOUT_MS)? {
            Ok(true)
        } else if self.standalone_ctl.write(unit, &self.proto, &mut self.segments.config, elem_id, new,
                                            TIMEOUT_MS)? {
            Ok(true)
        } else if self.knob_ctl.write(unit, &self.proto, &mut self.segments.knob, elem_id, new,
                                      TIMEOUT_MS)? {
            Ok(true)
        } else if self.specific_ctl.write(unit, &self.proto, &mut self.segments, elem_id, old, new,
                                          TIMEOUT_MS)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl NotifyModel<SndDice, u32> for ItwinModel {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.ctl.notified_elem_list);
        elem_id_list.extend_from_slice(&self.ch_strip_ctl.notified_elem_list);
        elem_id_list.extend_from_slice(&self.reverb_ctl.notified_elem_list);
        elem_id_list.extend_from_slice(&self.hw_state_ctl.notified_elem_list);
        elem_id_list.extend_from_slice(&self.mixer_ctl.notified_elem_list);
        elem_id_list.extend_from_slice(&self.knob_ctl.notified_elem_list);
        elem_id_list.extend_from_slice(&self.specific_ctl.0);
    }

    fn parse_notification(&mut self, unit: &mut SndDice, msg: &u32) -> Result<(), Error> {
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
        } else if self.knob_ctl.read(&self.segments.knob, elem_id, elem_value)? {
            Ok(true)
        } else if self.specific_ctl.read_notified_elem(&self.segments, elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl MeasureModel<hinawa::SndDice> for ItwinModel {
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
struct ItwinProto(FwReq);

impl AsRef<FwReq> for ItwinProto {
    fn as_ref(&self) -> &FwReq {
        &self.0
    }
}

fn itwin_phys_out_src_to_string(src: &ItwinOutputPairSrc) -> String {
    match src {
        ItwinOutputPairSrc::MixerOut01 => "Mixer-out-1/2",
        ItwinOutputPairSrc::Analog01 => "Analog-1/2",
        ItwinOutputPairSrc::Analog23 => "Analog-3/4",
        ItwinOutputPairSrc::Spdif01 => "S/PDIF-1/2",
        ItwinOutputPairSrc::Adat01 => "ADAT-1/2",
        ItwinOutputPairSrc::Adat23 => "ADAT-3/4",
        ItwinOutputPairSrc::Adat45 => "ADAT-5/6",
        ItwinOutputPairSrc::Adat67 => "ADAT-7/8",
        ItwinOutputPairSrc::Stream01 => "Stream-1/2",
        ItwinOutputPairSrc::Stream23 => "Stream-3/4",
        ItwinOutputPairSrc::Stream45 => "Stream-5/6",
        ItwinOutputPairSrc::Stream67 => "Stream-7/8",
        ItwinOutputPairSrc::Stream89 => "Stream-9/10",
        ItwinOutputPairSrc::Stream1011 => "Stream-11/12",
        ItwinOutputPairSrc::Stream1213 => "Stream-13/14",
        ItwinOutputPairSrc::MixerSend01 => "Mixer-send-1/2",
    }.to_string()
}

fn listening_mode_to_string(mode: &ListeningMode) -> String {
    match mode {
        ListeningMode::Monaural => "Monaural",
        ListeningMode::Stereo => "Stereo",
        ListeningMode::Side => "Side",
    }.to_string()
}

#[derive(Default, Debug)]
struct ItwinSpecificCtl(Vec<ElemId>);

impl<'a> ItwinSpecificCtl {
    const CLK_RECOVERY_NAME: &'a str = "clock-recovery";
    const OUT_SRC_NAME: &'a str = "output-source";
    const MIXER_ENABLE_NAME: &'a str = "mixer-enable";
    const LISTENING_MODE_NAME: &'a str = "listening-mode";

    const OUT_SRCS: [ItwinOutputPairSrc;16] = [
        ItwinOutputPairSrc::MixerOut01,
        ItwinOutputPairSrc::Analog01,
        ItwinOutputPairSrc::Analog23,
        ItwinOutputPairSrc::Spdif01,
        ItwinOutputPairSrc::Adat01,
        ItwinOutputPairSrc::Adat23,
        ItwinOutputPairSrc::Adat45,
        ItwinOutputPairSrc::Adat67,
        ItwinOutputPairSrc::Stream01,
        ItwinOutputPairSrc::Stream23,
        ItwinOutputPairSrc::Stream45,
        ItwinOutputPairSrc::Stream67,
        ItwinOutputPairSrc::Stream89,
        ItwinOutputPairSrc::Stream1011,
        ItwinOutputPairSrc::Stream1213,
        ItwinOutputPairSrc::MixerSend01,
    ];

    const LISTENING_MODES: [ListeningMode;3] = [
        ListeningMode::Monaural,
        ListeningMode::Stereo,
        ListeningMode::Side,
    ];

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::CLK_RECOVERY_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        let labels: Vec<String> = Self::OUT_SRCS.iter()
            .map(|s| itwin_phys_out_src_to_string(s))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::OUT_SRC_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, ITWIN_PHYS_OUT_PAIR_COUNT, &labels, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::MIXER_ENABLE_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        let labels: Vec<String> = Self::LISTENING_MODES.iter()
            .map(|m| listening_mode_to_string(m))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::LISTENING_MODE_NAME, 0);
        card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)
            .map(|mut elem_id_list| self.0.append(&mut elem_id_list))?;

        Ok(())
    }

    fn read(&mut self, segments: &ItwinSegments, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::CLK_RECOVERY_NAME => {
                ElemValueAccessor::<bool>::set_val(elem_value, || {
                    Ok(segments.knob.data.clock_recovery)
                })
                .map(|_| true)
            }
            Self::OUT_SRC_NAME => {
                ElemValueAccessor::<u32>::set_vals(elem_value, ITWIN_PHYS_OUT_PAIR_COUNT, |idx| {
                    let pos = Self::OUT_SRCS.iter()
                        .position(|&s| s == segments.config.data.output_pair_src[idx])
                        .expect("Programming error...");
                    Ok(pos as u32)
                })
                .map(|_| true)
            }
            Self::MIXER_ENABLE_NAME => {
                ElemValueAccessor::<bool>::set_val(elem_value, || {
                    Ok(segments.mixer_state.data.enabled)
                })
                .map(|_| true)
            }
            _ => self.read_notified_elem(segments, elem_id, elem_value),
        }
    }

    fn write(&mut self, unit: &SndDice, proto: &ItwinProto, segments: &mut ItwinSegments, elem_id: &ElemId,
             old: &ElemValue, new: &ElemValue, timeout_ms: u32)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::CLK_RECOVERY_NAME => {
                ElemValueAccessor::<bool>::get_val(new, |val| {
                    segments.knob.data.clock_recovery = val;
                    proto.write_segment(&unit.get_node(), &mut segments.knob, timeout_ms)
                })
                .map(|_| true)
            }
            Self::OUT_SRC_NAME => {
                let mut count = 0;
                ElemValueAccessor::<u32>::get_vals(new, old, ITWIN_PHYS_OUT_PAIR_COUNT, |idx, val| {
                    Self::OUT_SRCS.iter()
                        .nth(val as usize)
                        .ok_or_else(|| {
                            let msg = format!("Invalid value for index of output source: {}", val);
                            Error::new(FileError::Inval, &msg)
                        })
                        .and_then(|&s| {
                            segments.config.data.output_pair_src[idx] = s;
                            count += 1;
                            Ok(())
                        })
                })
                .and_then(|_| {
                    if count > 0 {
                        proto.write_segment(&unit.get_node(), &mut segments.config, timeout_ms)?;
                    }
                    Ok(true)
                })
            }
            Self::MIXER_ENABLE_NAME => {
                ElemValueAccessor::<bool>::get_val(new, |val| {
                    segments.mixer_state.data.enabled = val;
                    proto.write_segment(&unit.get_node(), &mut segments.mixer_state, timeout_ms)
                })
                .map(|_| true)
            }
            Self::LISTENING_MODE_NAME => {
                ElemValueAccessor::<u32>::get_val(new, |val| {
                    Self::LISTENING_MODES.iter()
                        .nth(val as usize)
                        .ok_or_else(|| {
                            let msg = format!("Invalid value for index of listening mode: {}", val);
                            Error::new(FileError::Inval, &msg)
                        })
                        .and_then(|&m| {
                            segments.hw_state.data.listening_mode = m;
                            proto.write_segment(&unit.get_node(), &mut segments.hw_state, timeout_ms)
                        })
                })
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }

    fn read_notified_elem(&mut self, segments: &ItwinSegments, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::LISTENING_MODE_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let pos = Self::LISTENING_MODES.iter()
                        .position(|&m| m == segments.hw_state.data.listening_mode)
                        .expect("Programming error...");
                    Ok(pos as u32)
                })
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }
}
