// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::{Error, FileError};

use alsactl::{ElemId, ElemIfaceType, ElemValue, ElemValueExtManual};

use hinawa::FwReq;
use hinawa::{SndDice, SndUnitExt};

use alsa_ctl_tlv_codec::items::DbInterval;

use core::card_cntr::*;
use core::elem_value_accessor::*;

use dice_protocols::tcat::{*, global_section::*};
use dice_protocols::tcelectronic::*;
use dice_protocols::tcelectronic::studio::*;

use crate::common_ctl::*;
use super::ch_strip_ctl::*;
use super::reverb_ctl::*;
use super::fw_led_ctl::*;

#[derive(Default)]
pub struct Studiok48Model{
    proto: Studiok48Proto,
    sections: GeneralSections,
    segments: StudioSegments,
    ctl: CommonCtl,
    ch_strip_ctl: ChStripCtl,
    reverb_ctl: ReverbCtl,
    hw_state_ctl: HwStateCtl,
    phys_out_ctl: PhysOutCtl,
}

const TIMEOUT_MS: u32 = 20;

impl CtlModel<SndDice> for Studiok48Model {
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
        self.proto.read_segment(&node, &mut self.segments.phys_out, TIMEOUT_MS)?;

        self.hw_state_ctl.load(card_cntr)?;
        self.phys_out_ctl.load(card_cntr)?;

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
        } else if self.hw_state_ctl.read(&self.segments, elem_id, elem_value)? {
            Ok(true)
        } else if self.phys_out_ctl.read(&self.segments, elem_id, elem_value)? {
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
        } else if self.hw_state_ctl.write(unit, &self.proto, &mut self.segments, elem_id,
                                          new, TIMEOUT_MS)? {
            Ok(true)
        } else if self.phys_out_ctl.write(unit, &self.proto, &mut self.segments, elem_id, old, new,
                                          TIMEOUT_MS)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl NotifyModel<SndDice, u32> for Studiok48Model {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.ctl.notified_elem_list);
        elem_id_list.extend_from_slice(&self.ch_strip_ctl.notified_elem_list);
        elem_id_list.extend_from_slice(&self.reverb_ctl.notified_elem_list);
        elem_id_list.extend_from_slice(&self.hw_state_ctl.notified_elem_list);
        elem_id_list.extend_from_slice(&self.phys_out_ctl.0);
    }

    fn parse_notification(&mut self, unit: &SndDice, msg: &u32) -> Result<(), Error> {
        self.ctl.parse_notification(unit, &self.proto, &self.sections, *msg, TIMEOUT_MS)?;

        let node = unit.get_node();
        self.proto.parse_notification(&node, &mut self.segments.ch_strip_state, TIMEOUT_MS, *msg)?;
        self.proto.parse_notification(&node, &mut self.segments.reverb_state, TIMEOUT_MS, *msg)?;
        self.proto.parse_notification(&node, &mut self.segments.hw_state, TIMEOUT_MS, *msg)?;
        self.proto.parse_notification(&node, &mut self.segments.phys_out, TIMEOUT_MS, msg)?;

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
        } else if self.hw_state_ctl.read(&self.segments, elem_id, elem_value)? {
            Ok(true)
        } else if self.phys_out_ctl.read(&self.segments, elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl MeasureModel<hinawa::SndDice> for Studiok48Model {
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
struct Studiok48Proto(FwReq);

impl AsRef<FwReq> for Studiok48Proto {
    fn as_ref(&self) -> &FwReq {
        &self.0
    }
}

fn analog_jack_state_to_string(state: &StudioAnalogJackState) -> String {
    match state {
        StudioAnalogJackState::FrontSelected => "front-selected",
        StudioAnalogJackState::FrontInserted => "front-inserted",
        StudioAnalogJackState::RearSelected => "rear-selected",
        StudioAnalogJackState::RearInserted => "rear-inserted",
    }.to_string()
}

#[derive(Default, Debug)]
struct HwStateCtl {
    notified_elem_list: Vec<ElemId>,
    fw_led_ctl: FwLedCtl,
}

impl<'a> HwStateCtl {
    // TODO: For Jack detection in ALSA applications.
    const ANALOG_JACK_STATE_NAME: &'a str = "analog-jack-state";
    const HP_JACK_STATE_NAME: &'a str = "headphone-jack-state";
    const VALID_MASTER_LEVEL_NAME: &'a str = "valid-master-level";

    const ANALOG_JACK_STATES: &'a [StudioAnalogJackState] = &[
        StudioAnalogJackState::FrontSelected,
        StudioAnalogJackState::FrontInserted,
        StudioAnalogJackState::RearSelected,
        StudioAnalogJackState::RearInserted,
    ];

    fn load(&mut self, card_cntr: &mut CardCntr)
        -> Result<(), Error>
    {
        let labels = Self::ANALOG_JACK_STATES.iter()
            .map(|s| analog_jack_state_to_string(s))
            .collect::<Vec<_>>();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::ANALOG_JACK_STATE_NAME, 0);
        card_cntr.add_enum_elems(&elem_id, 1, STUDIO_ANALOG_JACK_STATE_COUNT, &labels, None, false)
            .map(|mut elem_id_list| self.notified_elem_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::HP_JACK_STATE_NAME, 0);
        card_cntr.add_bool_elems(&elem_id, 1, 2, false)
            .map(|mut elem_id_list| self.notified_elem_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::VALID_MASTER_LEVEL_NAME, 0);
        card_cntr.add_bool_elems(&elem_id, 1, 1, false)
            .map(|mut elem_id_list| self.notified_elem_list.append(&mut elem_id_list))?;

        self.fw_led_ctl.load(card_cntr)
            .map(|_| self.notified_elem_list.extend_from_slice(&self.fw_led_ctl.0))?;

        Ok(())
    }

    fn read(&mut self, segments: &StudioSegments, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        if self.fw_led_ctl.read(&segments.hw_state, elem_id, elem_value)? {
            Ok(true)
        } else if self.read_notified_elem(&segments, elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(&mut self, unit: &SndDice, proto: &Studiok48Proto, segments: &mut StudioSegments,
                 elem_id: &ElemId, elem_value: &ElemValue, timeout_ms: u32)
        -> Result<bool, Error>
    {
        self.fw_led_ctl.write(unit, proto, &mut segments.hw_state, elem_id, elem_value, timeout_ms)
    }

    fn read_notified_elem(&mut self, segments: &StudioSegments, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::ANALOG_JACK_STATE_NAME => {
                let analog_jack_states = &segments.hw_state.data.analog_jack_states;
                ElemValueAccessor::<u32>::set_vals(elem_value, analog_jack_states.len(), |idx| {
                    let pos = Self::ANALOG_JACK_STATES.iter()
                        .position(|s| s.eq(&analog_jack_states[idx]))
                        .expect("Programming error");
                    Ok(pos as u32)
                })
                .map(|_| true)
            }
            Self::HP_JACK_STATE_NAME => {
                elem_value.set_bool(&segments.hw_state.data.hp_state);
                Ok(true)
            }
            Self::VALID_MASTER_LEVEL_NAME => {
                elem_value.set_bool(&[segments.hw_state.data.valid_master_level]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

fn src_pair_entry_to_string(entry: &SrcEntry) -> String {
    match entry {
        SrcEntry::Unused => "Unused".to_string(),
        SrcEntry::Analog(ch) => format!("Analog-{}", ch + 1),
        SrcEntry::Spdif(ch) => format!("S/PDIF-{}", ch + 1),
        SrcEntry::Adat(ch) => format!("ADAT-{}", ch + 1),
        SrcEntry::StreamA(ch) => format!("Stream-A-{}", ch + 1),
        SrcEntry::StreamB(ch) => format!("Stream-B-{}", ch + 1),
        SrcEntry::Mixer(ch) => {
            if *ch < 2 {
                format!("Mixer-{}", ch + 1)
            } else if *ch < 6 {
                format!("Aux-{}", ch - 1)
            } else {
                format!("Reverb-{}", ch - 5)
            }
        }
    }
}

#[derive(Default, Debug)]
struct PhysOutCtl(pub Vec<ElemId>);

impl<'a> PhysOutCtl {
    const MASTER_OUT_DIM_NAME: &'a str = "master-out-dim";
    const MASTER_OUT_VOL_NAME: &'a str = "master-out-volume";
    const MASTER_OUT_DIM_VOL_NAME: &'a str = "master-out-dim-volume";

    const PHYS_OUT_STEREO_LINK_NAME: &'a str = "phys-out-stereo-link";
    const PHYS_OUT_SRC_NAME: &'a str = "phys-out-source";
    const PHYS_OUT_LEVEL_NAME: &'a str = "phys-out-level";
    const PHYS_OUT_SPKR_TRIM_NAME: &'a str = "phys-out-speaker-trim";
    const PHYS_OUT_SPKR_DELAY_NAME: &'a str = "phys-out-speaker-delay";

    const PHYS_OUT_SRCS: &'a [SrcEntry] = &[
        SrcEntry::Unused,
        SrcEntry::Analog(0), SrcEntry::Analog(1), SrcEntry::Analog(2), SrcEntry::Analog(3),
        SrcEntry::Analog(4), SrcEntry::Analog(5), SrcEntry::Analog(6), SrcEntry::Analog(7),
        SrcEntry::Analog(8), SrcEntry::Analog(9), SrcEntry::Analog(10), SrcEntry::Analog(11),
        SrcEntry::Spdif(0), SrcEntry::Spdif(1),
        SrcEntry::Adat(0), SrcEntry::Adat(1), SrcEntry::Adat(2), SrcEntry::Adat(3),
        SrcEntry::Adat(4), SrcEntry::Adat(5), SrcEntry::Adat(6), SrcEntry::Adat(7),
        SrcEntry::StreamA(0), SrcEntry::StreamA(1), SrcEntry::StreamA(2), SrcEntry::StreamA(3),
        SrcEntry::StreamA(4), SrcEntry::StreamA(5), SrcEntry::StreamA(6), SrcEntry::StreamA(7),
        SrcEntry::StreamA(8), SrcEntry::StreamA(9), SrcEntry::StreamA(10), SrcEntry::StreamA(11),
        SrcEntry::StreamA(12), SrcEntry::StreamA(13), SrcEntry::StreamA(14), SrcEntry::StreamA(15),
        SrcEntry::StreamB(0), SrcEntry::StreamB(1), SrcEntry::StreamB(2), SrcEntry::StreamB(3),
        SrcEntry::StreamB(4), SrcEntry::StreamB(5), SrcEntry::StreamB(6), SrcEntry::StreamB(7),
        SrcEntry::StreamB(8), SrcEntry::StreamB(9), SrcEntry::StreamB(10), SrcEntry::StreamB(11),
        SrcEntry::Mixer(0), SrcEntry::Mixer(1),
        SrcEntry::Mixer(2), SrcEntry::Mixer(3), SrcEntry::Mixer(4), SrcEntry::Mixer(5),
        SrcEntry::Mixer(6), SrcEntry::Mixer(7),
    ];

    const VOL_MIN: i32 = -1000;
    const VOL_MAX: i32 = 0;
    const VOL_STEP: i32 = 1;
    const VOL_TLV: DbInterval = DbInterval{min: -7200, max: 0, linear: false, mute_avail: false};

    const PHYS_OUT_LEVEL_LABELS: &'a [&'a str] = &[
        "Muted",
        "Line",
        "Speaker",
    ];

    const TRIM_MIN: i32 = -20;
    const TRIM_MAX: i32 = 0;
    const TRIM_STEP: i32 = 1;

    const DELAY_MIN: i32 = 0;
    const DELAY_MAX: i32 = 30;
    const DELAY_STEP: i32 = 1;

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        // For master output.
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::MASTER_OUT_DIM_NAME, 0);
        card_cntr.add_bool_elems(&elem_id, 1, 1, true)
            .map(|mut elem_id_list| self.0.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::MASTER_OUT_VOL_NAME, 0);
        card_cntr.add_int_elems(&elem_id, 1, Self::VOL_MIN, Self::VOL_MAX, Self::VOL_STEP,
                                1, Some(&Into::<Vec<u32>>::into(Self::VOL_TLV)), true)
            .map(|mut elem_id_list| self.0.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::MASTER_OUT_DIM_VOL_NAME, 0);
        card_cntr.add_int_elems(&elem_id, 1, Self::VOL_MIN, Self::VOL_MAX, Self::VOL_STEP,
                                1, Some(&Into::<Vec<u32>>::into(Self::VOL_TLV)), true)
            .map(|mut elem_id_list| self.0.append(&mut elem_id_list))?;

        // For source of output pair.
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::PHYS_OUT_STEREO_LINK_NAME, 0);
        card_cntr.add_bool_elems(&elem_id, 1, STUDIO_PHYS_OUT_PAIR_COUNT, true)
            .map(|mut elem_id_list| self.0.append(&mut elem_id_list))?;

        let labels: Vec<String> = Self::PHYS_OUT_SRCS.iter()
            .map(|src| src_pair_entry_to_string(src))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::PHYS_OUT_SRC_NAME, 0);
        card_cntr.add_enum_elems(&elem_id, 1, STUDIO_PHYS_OUT_PAIR_COUNT * 2, &labels, None, true)
            .map(|mut elem_id_list| self.0.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::PHYS_OUT_LEVEL_NAME, 0);
        card_cntr.add_enum_elems(&elem_id, 1, STUDIO_PHYS_OUT_PAIR_COUNT * 2, Self::PHYS_OUT_LEVEL_LABELS,
                                 None, true)
            .map(|mut elem_id_list| self.0.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::PHYS_OUT_SPKR_TRIM_NAME, 0);
        card_cntr.add_int_elems(&elem_id, 1, Self::TRIM_MIN, Self::TRIM_MAX, Self::TRIM_STEP,
                                STUDIO_PHYS_OUT_PAIR_COUNT * 2, None, true)
            .map(|mut elem_id_list| self.0.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::PHYS_OUT_SPKR_DELAY_NAME, 0);
        card_cntr.add_int_elems(&elem_id, 1, Self::DELAY_MIN, Self::DELAY_MAX, Self::DELAY_STEP,
                                STUDIO_PHYS_OUT_PAIR_COUNT * 2, None, true)
            .map(|mut elem_id_list| self.0.append(&mut elem_id_list))?;

        Ok(())
    }

    fn read(&mut self, segments: &StudioSegments, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::MASTER_OUT_DIM_NAME => {
                ElemValueAccessor::<bool>::set_val(elem_value, || {
                    Ok(segments.phys_out.data.master_out.dim_enabled)
                })
                .map(|_| true)
            }
            Self::MASTER_OUT_VOL_NAME => {
                ElemValueAccessor::<i32>::set_val(elem_value, || {
                    Ok(segments.phys_out.data.master_out.vol)
                })
                .map(|_| true)
            }
            Self::MASTER_OUT_DIM_VOL_NAME => {
                ElemValueAccessor::<i32>::set_val(elem_value, || {
                    Ok(segments.phys_out.data.master_out.dim_vol)
                })
                .map(|_| true)
            }
            Self::PHYS_OUT_STEREO_LINK_NAME => {
                ElemValueAccessor::<bool>::set_vals(elem_value, STUDIO_PHYS_OUT_PAIR_COUNT, |idx| {
                    Ok(segments.phys_out.data.out_pair_srcs[idx].stereo_link)
                })
                .map(|_| true)
            }
            Self::PHYS_OUT_SRC_NAME => {
                Self::read_out_src_param(segments, elem_value, |param| {
                     let pos = Self::PHYS_OUT_SRCS.iter()
                        .position(|s| s.eq(&param.src))
                         .expect("Programming error");
                     Ok(pos as u32)
                 })
                .map(|_| true)
            }
            Self::PHYS_OUT_LEVEL_NAME => {
                 ElemValueAccessor::<u32>::set_vals(elem_value, STUDIO_PHYS_OUT_PAIR_COUNT, |idx| {
                    let val = if segments.phys_out.data.muted[idx] {
                        0
                    } else if !segments.phys_out.data.spkr_assigns[idx] {
                         1
                     } else {
                        2
                     };
                     Ok(val)
                 })
                 .map(|_| true)
            }
            Self::PHYS_OUT_SPKR_TRIM_NAME => {
                Self::read_out_src_param(segments, elem_value, |param| {
                    Ok(param.vol)
                })
            }
            Self::PHYS_OUT_SPKR_DELAY_NAME => {
                Self::read_out_src_param(segments, elem_value, |param| {
                    Ok(param.delay)
                })
            }
            _ => Ok(false),
        }
    }

    fn read_out_src_param<T, F>(segments: &StudioSegments, elem_value: &mut ElemValue, cb: F)
        -> Result<bool, Error>
        where F: Fn(&PhysOutSrcParam) -> Result<T, Error>,
              T: Default + Copy + Eq,
              ElemValue: ElemValueAccessor<T>,
    {
        ElemValueAccessor::<T>::set_vals(elem_value, STUDIO_PHYS_OUT_PAIR_COUNT * 2, |idx| {
            let i = idx / 2;
            let ch = idx % 2;
            let param = if ch == 0 {
                &segments.phys_out.data.out_pair_srcs[i].left
            } else {
                &segments.phys_out.data.out_pair_srcs[i].right
            };
            cb(param)
        })
        .map(|_| true)
    }

    fn write(&mut self, unit: &SndDice, proto: &Studiok48Proto, segments: &mut StudioSegments,
             elem_id: &ElemId, old: &ElemValue, new: &ElemValue, timeout_ms: u32)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::MASTER_OUT_DIM_NAME => {
                ElemValueAccessor::<bool>::get_val(new, |val| {
                    segments.phys_out.data.master_out.dim_enabled = val;
                    Ok(())
                })
                .and_then(|_| proto.write_segment(&unit.get_node(), &mut segments.phys_out, timeout_ms))
                .map(|_| true)
            }
            Self::MASTER_OUT_VOL_NAME => {
                ElemValueAccessor::<i32>::get_val(new, |val| {
                    segments.phys_out.data.master_out.vol = val;
                    Ok(())
                })
                .and_then(|_| proto.write_segment(&unit.get_node(), &mut segments.phys_out, timeout_ms))
                .map(|_| true)
            }
            Self::MASTER_OUT_DIM_VOL_NAME => {
                ElemValueAccessor::<i32>::get_val(new, |val| {
                    segments.phys_out.data.master_out.dim_vol = val;
                    Ok(())
                })
                .and_then(|_| proto.write_segment(&unit.get_node(), &mut segments.phys_out, timeout_ms))
                .map(|_| true)
            }
            Self::PHYS_OUT_STEREO_LINK_NAME => {
                ElemValueAccessor::<bool>::get_vals(new, old, STUDIO_PHYS_OUT_PAIR_COUNT, |idx, val| {
                    segments.phys_out.data.out_pair_srcs[idx].stereo_link = val;
                    Ok(())
                })
                .and_then(|_| proto.write_segment(&unit.get_node(), &mut segments.phys_out, timeout_ms))
                .map(|_| true)
            }
            Self::PHYS_OUT_SRC_NAME => {
                Self::write_out_src_param(unit, proto, segments, new, old, timeout_ms, |param, val: u32| {
                     Self::PHYS_OUT_SRCS.iter()
                         .nth(val as usize)
                         .ok_or_else(|| {
                             let msg = format!("Invalid value for index of source of output: {}", val);
                             Error::new(FileError::Inval, &msg)
                         })
                        .map(|&s| param.src = s)
                 })
            }
            Self::PHYS_OUT_LEVEL_NAME => {
                 ElemValueAccessor::<u32>::get_vals(new, old, STUDIO_PHYS_OUT_PAIR_COUNT, |idx, val| {
                    if val == 0 {
                        segments.phys_out.data.muted[idx] = true;
                    } else {
                        segments.phys_out.data.muted[idx] = false;
                        if val == 1 {
                            segments.phys_out.data.spkr_assigns[idx] = false;
                        } else {
                            segments.phys_out.data.spkr_assigns[idx] = true;
                        }
                    }
                    Ok(())
                })
                .and_then(|_| proto.write_segment(&unit.get_node(), &mut segments.phys_out, timeout_ms))
                .map(|_| true)
            }
            Self::PHYS_OUT_SPKR_TRIM_NAME => {
                Self::write_out_src_param(unit, proto, segments, new, old, timeout_ms, |param, val| {
                    param.vol = val;
                    Ok(())
                })
                .and_then(|_| proto.write_segment(&unit.get_node(), &mut segments.phys_out, timeout_ms))
                .map(|_| true)
            }
            Self::PHYS_OUT_SPKR_DELAY_NAME => {
                Self::write_out_src_param(unit, proto, segments, new, old, timeout_ms, |param, val| {
                    param.delay = val;
                    Ok(())
                })
                .and_then(|_| proto.write_segment(&unit.get_node(), &mut segments.phys_out, timeout_ms))
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }

    fn write_out_src_param<T, F>(unit: &SndDice, proto: &Studiok48Proto, segments: &mut StudioSegments,
                                 new: &ElemValue, old: &ElemValue, timeout_ms: u32, cb: F)
        -> Result<bool, Error>
        where F: Fn(&mut PhysOutSrcParam, T) -> Result<(), Error>,
              T: Default + Copy + Eq,
              ElemValue: ElemValueAccessor<T>,
    {
        ElemValueAccessor::<T>::get_vals(new, old, STUDIO_PHYS_OUT_PAIR_COUNT * 2, |idx, val| {
            let i = idx / 2;
            let ch = idx % 2;
            let param = if ch == 0 {
                &mut segments.phys_out.data.out_pair_srcs[i].left
            } else {
                &mut segments.phys_out.data.out_pair_srcs[i].right
            };
            cb(param, val)
        })
        .and_then(|_| proto.write_segment(&unit.get_node(), &mut segments.phys_out, timeout_ms))
        .map(|_| true)
    }
}
