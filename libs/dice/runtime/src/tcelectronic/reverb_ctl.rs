// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use alsactl::{ElemId, ElemIfaceType, ElemValue, ElemValueExt};

use hinawa::{FwNode, SndDice, SndUnitExt};

use alsa_ctl_tlv_codec::items::DbInterval;

use dice_protocols::tcelectronic::{*, reverb::*};

use core::card_cntr::*;
use core::elem_value_accessor::*;

fn create_reverb_algorithm_labels() -> Vec<String> {
    [
        ReverbAlgorithm::Live1,
        ReverbAlgorithm::Hall,
        ReverbAlgorithm::Plate,
        ReverbAlgorithm::Club,
        ReverbAlgorithm::ConcertHall,
        ReverbAlgorithm::Cathedral,
        ReverbAlgorithm::Church,
        ReverbAlgorithm::Room,
        ReverbAlgorithm::SmallRoom,
        ReverbAlgorithm::Box,
        ReverbAlgorithm::Ambient,
        ReverbAlgorithm::Live2,
        ReverbAlgorithm::Live3,
        ReverbAlgorithm::Spring,
    ].iter()
        .map(|algorithm| {
            match algorithm {
                ReverbAlgorithm::Live1 => "Live1",
                ReverbAlgorithm::Hall => "Hall",
                ReverbAlgorithm::Plate => "Plate",
                ReverbAlgorithm::Club => "Club",
                ReverbAlgorithm::ConcertHall => "Concert-hall",
                ReverbAlgorithm::Cathedral => "Cathedral",
                ReverbAlgorithm::Church => "Church",
                ReverbAlgorithm::Room => "Room",
                ReverbAlgorithm::SmallRoom => "Small-room",
                ReverbAlgorithm::Box => "Box",
                ReverbAlgorithm::Ambient => "Ambient",
                ReverbAlgorithm::Live2 => "Live2",
                ReverbAlgorithm::Live3 => "Live3",
                ReverbAlgorithm::Spring => "Spring",
                ReverbAlgorithm::Reserved(_) => "Reserved",
            }.to_string()
        })
        .collect()
}

#[derive(Default, Debug)]
pub struct ReverbCtl {
    pub notified_elem_list: Vec<ElemId>,
    pub measured_elem_list: Vec<ElemId>,
}

impl ReverbCtl {
    const REVERB_INPUT_LEVEL_NAME: &'static str = "reverb-input-level";
    const REVERB_BYPASS_NAME: &'static str = "reverb-bypass";
    const REVERB_KILL_WET: &'static str = "reverb-kill-wet";
    const REVERB_KILL_DRY: &'static str = "reverb-kill-dry";
    const REVERB_OUTPUT_LEVEL_NAME: &'static str = "reverb-output-level";
    const REVERB_TIME_DECAY_NAME: &'static str = "reverb-time-decay";
    const REVERB_TIME_PRE_DECAY_NAME: &'static str = "reverb-time-pre-decay";
    const REVERB_COLOR_LOW_NAME: &'static str = "reverb-color-low";
    const REVERB_COLOR_HIGH_NAME: &'static str = "reverb-color-high";
    const REVERB_COLOR_HIGH_FACTOR_NAME: &'static str = "reverb-color-high-factor";
    const REVERB_MOD_RATE_NAME: &'static str = "reverb-mod-rate";
    const REVERB_MOD_DEPTH_NAME: &'static str = "reverb-mod-depth";
    const REVERB_LEVEL_EARLY_NAME: &'static str = "reverb-level-early";
    const REVERB_LEVEL_REVERB_NAME: &'static str = "reverb-level-reverb";
    const REVERB_LEVEL_DRY_NAME: &'static str = "reverb-level-dry";
    const REVERB_ALGORITHM_NAME: &'static str = "reverb-algorithm";
    const REVERB_OUTPUT_METER_NAME: &'static str = "reverb-output-meter";
    const REVERB_INPUT_METER_NAME: &'static str = "reverb-input-meter";

    const INPUT_LEVEL_MIN: i32 = -240;
    const INPUT_LEVEL_MAX: i32 = 0;
    const INPUT_LEVEL_STEP: i32 = 1;
    const INPUT_LEVEL_TLV: DbInterval = DbInterval{min: -2400, max: 0, linear: false, mute_avail: false};

    const OUTPUT_LEVEL_MIN: i32 = -240;
    const OUTPUT_LEVEL_MAX: i32 = 120;
    const OUTPUT_LEVEL_STEP: i32 = 1;
    const OUTPUT_LEVEL_TLV: DbInterval = DbInterval{min: -2400, max: 1200, linear: false, mute_avail: false};

    const DECAY_MIN: i32 = 1;
    const DECAY_MAX: i32 = 290;
    const DECAY_STEP: i32 = 1;

    const PRE_DECAY_MIN: i32 = 0;
    const PRE_DECAY_MAX: i32 = 100;
    const PRE_DECAY_STEP: i32 = 1;

    const COLOR_MIN: i32 = -50;
    const COLOR_MAX: i32 = 50;
    const COLOR_STEP: i32 = 1;

    const FACTOR_MIN: i32 = -25;
    const FACTOR_MAX: i32 = 25;
    const FACTOR_STEP: i32 = 1;

    const LEVEL_MIN: i32 = -48;
    const LEVEL_MAX: i32 = 0;
    const LEVEL_STEP: i32 = 1;

    const METER_OUTPUT_MIN: i32 = -1000;
    const METER_OUTPUT_MAX: i32 = 500;
    const METER_OUTPUT_STEP: i32 = 1;
    const METER_OUTPUT_TLV: DbInterval = DbInterval{min: -2400, max: 1200, linear: false, mute_avail: false};

    const METER_INPUT_MIN: i32 = -1000;
    const METER_INPUT_MAX: i32 = 0;
    const METER_INPUT_STEP: i32 = 1;
    const METER_INPUT_TLV:  DbInterval = DbInterval{min: -2400, max: 0, linear: false, mute_avail: false};

    pub fn load<T, S, M>(&mut self, unit: &SndDice, proto: &T, state_segment: &mut TcKonnektSegment<S>,
                         meter_segment: &mut TcKonnektSegment<M>, timeout_ms: u32, card_cntr: &mut CardCntr)
        -> Result<(), Error>
        where T: TcKonnektSegmentProtocol<FwNode, S> + TcKonnektSegmentProtocol<FwNode, M>,
              S: TcKonnektSegmentData + AsRef<ReverbState>,
              TcKonnektSegment<S>: TcKonnektSegmentSpec,
              M: TcKonnektSegmentData + AsRef<ReverbMeter>,
              TcKonnektSegment<M>: TcKonnektSegmentSpec,
    {
        let node = unit.get_node();
        proto.read_segment(&node, state_segment, timeout_ms)?;
        proto.read_segment(&node, meter_segment, timeout_ms)?;

        self.state_add_int_elem(card_cntr, Self::REVERB_INPUT_LEVEL_NAME,
                                Self::INPUT_LEVEL_MIN, Self::INPUT_LEVEL_MAX, Self::INPUT_LEVEL_STEP,
                                1, Some(&Into::<Vec<u32>>::into(Self::INPUT_LEVEL_TLV)), true)?;

        self.state_add_bool_elem(card_cntr, Self::REVERB_BYPASS_NAME, 1, true)?;
        self.state_add_bool_elem(card_cntr, Self::REVERB_KILL_WET, 1, true)?;
        self.state_add_bool_elem(card_cntr, Self::REVERB_KILL_DRY, 1, true)?;

        self.state_add_int_elem(card_cntr, Self::REVERB_OUTPUT_LEVEL_NAME,
                                Self::OUTPUT_LEVEL_MIN, Self::OUTPUT_LEVEL_MAX, Self::OUTPUT_LEVEL_STEP,
                                1, Some(&Into::<Vec<u32>>::into(Self::OUTPUT_LEVEL_TLV)), true)?;

        self.state_add_int_elem(card_cntr, Self::REVERB_TIME_DECAY_NAME,
                                Self::DECAY_MIN, Self::DECAY_MAX, Self::DECAY_STEP, 1, None, true)?;
        self.state_add_int_elem(card_cntr, Self::REVERB_TIME_PRE_DECAY_NAME,
                                Self::PRE_DECAY_MIN, Self::PRE_DECAY_MAX, Self::PRE_DECAY_STEP,
                                1, None, true)?;

        self.state_add_int_elem(card_cntr, Self::REVERB_COLOR_LOW_NAME,
                                Self::COLOR_MIN, Self::COLOR_MAX, Self::COLOR_STEP,
                                1, None, true)?;
        self.state_add_int_elem(card_cntr, Self::REVERB_COLOR_HIGH_NAME,
                                Self::COLOR_MIN, Self::COLOR_MAX, Self::COLOR_STEP,
                                1, None, true)?;
        self.state_add_int_elem(card_cntr, Self::REVERB_COLOR_HIGH_FACTOR_NAME,
                                Self::FACTOR_MIN, Self::FACTOR_MAX, Self::FACTOR_STEP,
                                1, None, true)?;

        self.state_add_int_elem(card_cntr, Self::REVERB_MOD_RATE_NAME,
                                Self::FACTOR_MIN, Self::FACTOR_MAX, Self::FACTOR_STEP,
                                1, None, true)?;
        self.state_add_int_elem(card_cntr, Self::REVERB_MOD_DEPTH_NAME,
                                Self::FACTOR_MIN, Self::FACTOR_MAX, Self::FACTOR_STEP,
                                1, None, true)?;

        self.state_add_int_elem(card_cntr, Self::REVERB_LEVEL_EARLY_NAME,
                                Self::LEVEL_MIN, Self::LEVEL_MAX, Self::LEVEL_STEP,
                                1, None, true)?;
        self.state_add_int_elem(card_cntr, Self::REVERB_LEVEL_REVERB_NAME,
                                Self::LEVEL_MIN, Self::LEVEL_MAX, Self::LEVEL_STEP,
                                1, None, true)?;
        self.state_add_int_elem(card_cntr, Self::REVERB_LEVEL_DRY_NAME,
                          Self::LEVEL_MIN, Self::LEVEL_MAX, Self::LEVEL_STEP,
                          1, None, true)?;

        let labels = create_reverb_algorithm_labels();
        self.state_add_enum_elem(card_cntr, Self::REVERB_ALGORITHM_NAME, 1, &labels, true)?;

        self.meter_add_int_elem(card_cntr, Self::REVERB_OUTPUT_METER_NAME,
                                Self::METER_OUTPUT_MIN, Self::METER_OUTPUT_MAX, Self::METER_OUTPUT_STEP,
                                2, Some(&Into::<Vec<u32>>::into(Self::METER_OUTPUT_TLV)), false)?;

        self.meter_add_int_elem(card_cntr, Self::REVERB_INPUT_METER_NAME,
                                Self::METER_INPUT_MIN, Self::METER_INPUT_MAX, Self::METER_INPUT_STEP,
                                2, Some(&Into::<Vec<u32>>::into(Self::METER_INPUT_TLV)), true)?;

        Ok(())
    }

    fn state_add_int_elem(&mut self, card_cntr: &mut CardCntr, name: &str, min: i32, max: i32, step: i32,
                    value_count: usize, tlv: Option<&[u32]>, locked: bool)
        -> Result<(), Error>
    {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
        card_cntr.add_int_elems(&elem_id, 1, min, max, step, value_count, tlv, locked)
            .map(|mut elem_id_list| self.notified_elem_list.append(&mut elem_id_list))
    }

    fn state_add_enum_elem<T: AsRef<str>>(&mut self, card_cntr: &mut CardCntr, name: &str, value_count: usize,
                                    labels: &[T], locked: bool)
        -> Result<(), Error>
    {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
        card_cntr.add_enum_elems(&elem_id, 1, value_count, labels, None, locked)
            .map(|mut elem_id_list| self.notified_elem_list.append(&mut elem_id_list))
    }

    fn state_add_bool_elem(&mut self, card_cntr: &mut CardCntr, name: &str, value_count: usize, locked: bool)
        -> Result<(), Error>
    {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
        card_cntr.add_bool_elems(&elem_id, 1, value_count, locked)
            .map(|mut elem_id_list| self.notified_elem_list.append(&mut elem_id_list))
    }

    fn meter_add_int_elem(&mut self, card_cntr: &mut CardCntr, name: &str, min: i32, max: i32, step: i32,
                          value_count: usize, tlv: Option<&[u32]>, locked: bool)
        -> Result<(), Error>
    {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
        card_cntr.add_int_elems(&elem_id, 1, min, max, step, value_count, tlv, locked)
            .map(|mut elem_id_list| self.measured_elem_list.append(&mut elem_id_list))
    }

    pub fn read<S, M>(&self, state_segment: &TcKonnektSegment<S>, meter_segment: &TcKonnektSegment<M>,
                      elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
        where S: TcKonnektSegmentData + AsRef<ReverbState>,
              TcKonnektSegment<S>: TcKonnektSegmentSpec,
              M: TcKonnektSegmentData + AsRef<ReverbMeter>,
              TcKonnektSegment<M>: TcKonnektSegmentSpec,
    {
        if self.read_notified_elem(state_segment, elem_id, elem_value)? {
            Ok(true)
        } else if self.read_measured_elem(meter_segment, elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn state_read_elem<S, T, F>(&self, segment: &TcKonnektSegment<S>, elem_value: &ElemValue, cb: F)
        -> Result<bool, Error>
        where F: Fn(&ReverbState) -> T,
              S: TcKonnektSegmentData + AsRef<ReverbState>,
              T: Default + Copy + Eq,
              ElemValue: ElemValueAccessor<T>,
    {
        ElemValueAccessor::<T>::set_val(elem_value, || {
            Ok(cb(&segment.data.as_ref()))
        })
        .map(|_| true)
    }

    pub fn write<T, S>(&mut self, unit: &SndDice, proto: &T, segment: &mut TcKonnektSegment<S>,
                    elem_id: &ElemId, elem_value: &ElemValue, timeout_ms: u32)
        -> Result<bool, Error>
        where T: TcKonnektSegmentProtocol<FwNode, S>,
              S: TcKonnektSegmentData + AsMut<ReverbState>,
              TcKonnektSegment<S>: TcKonnektSegmentSpec,
    {
        match elem_id.get_name().as_str() {
            Self::REVERB_INPUT_LEVEL_NAME => {
                self.state_write_elem(unit, proto, segment, elem_value, timeout_ms,
                                          |state, val| state.input_level = val)
            }
            Self::REVERB_BYPASS_NAME => {
                self.state_write_elem(unit, proto, segment, elem_value, timeout_ms,
                                           |state, val| state.bypass = val)
            }
            Self::REVERB_KILL_WET => {
                self.state_write_elem(unit, proto, segment, elem_value, timeout_ms,
                                           |state, val| state.kill_wet = val)
            }
            Self::REVERB_KILL_DRY => {
                self.state_write_elem(unit, proto, segment, elem_value, timeout_ms,
                                           |state, val| state.kill_dry = val)
            }
            Self::REVERB_OUTPUT_LEVEL_NAME => {
                self.state_write_elem(unit, proto, segment, elem_value, timeout_ms,
                                          |state, val| state.output_level = val)
            }
            Self::REVERB_TIME_DECAY_NAME => {
                self.state_write_elem(unit, proto, segment, elem_value, timeout_ms,
                                          |state, val| state.time_decay = val)
            }
            Self::REVERB_TIME_PRE_DECAY_NAME => {
                self.state_write_elem(unit, proto, segment, elem_value, timeout_ms,
                                          |state, val| state.time_pre_decay = val)
            }
            Self::REVERB_COLOR_LOW_NAME => {
                self.state_write_elem(unit, proto, segment, elem_value, timeout_ms,
                                          |state, val| state.color_low = val)
            }
            Self::REVERB_COLOR_HIGH_NAME => {
                self.state_write_elem(unit, proto, segment, elem_value, timeout_ms,
                                          |state, val| state.color_high = val)
            }
            Self::REVERB_COLOR_HIGH_FACTOR_NAME => {
                self.state_write_elem(unit, proto, segment, elem_value, timeout_ms,
                                          |state, val| state.color_high_factor = val)
            }
            Self::REVERB_MOD_RATE_NAME => {
                self.state_write_elem(unit, proto, segment, elem_value, timeout_ms,
                                          |state, val| state.mod_rate = val)
            }
            Self::REVERB_MOD_DEPTH_NAME => {
                self.state_write_elem(unit, proto, segment, elem_value, timeout_ms,
                                          |state, val| state.mod_depth = val)
            }
            Self::REVERB_LEVEL_EARLY_NAME => {
                self.state_write_elem(unit, proto, segment, elem_value, timeout_ms,
                                          |state, val| state.level_early = val)
            }
            Self::REVERB_LEVEL_REVERB_NAME => {
                self.state_write_elem(unit, proto, segment, elem_value, timeout_ms,
                                          |state, val| state.level_reverb = val)
            }
            Self::REVERB_LEVEL_DRY_NAME => {
                self.state_write_elem(unit, proto, segment, elem_value, timeout_ms,
                                          |state, val| state.level_dry = val)
            }
            Self::REVERB_ALGORITHM_NAME => {
                self.state_write_elem(unit, proto, segment, elem_value, timeout_ms,
                                           |state, val: u32| state.algorithm = ReverbAlgorithm::from(val))
            }
            _ => Ok(false),
        }
    }

    fn state_write_elem<T, S, U, F>(&mut self, unit: &SndDice, proto: &T, segment: &mut TcKonnektSegment<S>,
                                    elem_value: &ElemValue, timeout_ms: u32, cb: F)
        -> Result<bool, Error>
        where T: TcKonnektSegmentProtocol<FwNode, S>,
              S: TcKonnektSegmentData + AsMut<ReverbState>,
              TcKonnektSegment<S>: TcKonnektSegmentSpec,
              F: Fn(&mut ReverbState, U) -> (),
              U: Default + Copy + Eq,
              ElemValue: ElemValueAccessor<U>,
    {
        ElemValueAccessor::<U>::get_val(elem_value, |val| {
            cb(&mut segment.data.as_mut(), val);
            Ok(())
        })
        .and_then(|_| proto.write_segment(&unit.get_node(), segment, timeout_ms))
        .map(|_| true)
    }

    pub fn read_notified_elem<S>(&self, segment: &TcKonnektSegment<S>, elem_id: &ElemId,
                                 elem_value: &mut ElemValue)
        -> Result<bool, Error>
        where S: TcKonnektSegmentData + AsRef<ReverbState>,
              TcKonnektSegment<S>: TcKonnektSegmentSpec,
    {
        match elem_id.get_name().as_str() {
            Self::REVERB_INPUT_LEVEL_NAME => {
                self.state_read_elem(segment, elem_value, |state| state.input_level)
            }
            Self::REVERB_BYPASS_NAME => {
                self.state_read_elem(segment, elem_value, |state| state.bypass)
            }
            Self::REVERB_KILL_WET => {
                self.state_read_elem(segment, elem_value, |state| state.kill_wet)
            }
            Self::REVERB_KILL_DRY => {
                self.state_read_elem(segment, elem_value, |state| state.kill_dry)
            }
            Self::REVERB_OUTPUT_LEVEL_NAME => {
                self.state_read_elem(segment, elem_value, |state| state.output_level)
            }
            Self::REVERB_TIME_DECAY_NAME => {
                self.state_read_elem(segment, elem_value, |state| state.time_decay)
            }
            Self::REVERB_TIME_PRE_DECAY_NAME => {
                self.state_read_elem(segment, elem_value, |state| state.time_pre_decay)
            }
            Self::REVERB_COLOR_LOW_NAME => {
                self.state_read_elem(segment, elem_value, |state| state.color_low)
            }
            Self::REVERB_COLOR_HIGH_NAME => {
                self.state_read_elem(segment, elem_value, |state| state.color_high)
            }
            Self::REVERB_COLOR_HIGH_FACTOR_NAME => {
                self.state_read_elem(segment, elem_value, |state| state.color_high_factor)
            }
            Self::REVERB_MOD_RATE_NAME => {
                self.state_read_elem(segment, elem_value, |state| state.mod_rate)
            }
            Self::REVERB_MOD_DEPTH_NAME => {
                self.state_read_elem(segment, elem_value, |state| state.mod_depth)
            }
            Self::REVERB_LEVEL_EARLY_NAME => {
                self.state_read_elem(segment, elem_value, |state| state.level_early)
            }
            Self::REVERB_LEVEL_REVERB_NAME => {
                self.state_read_elem(segment, elem_value, |state| state.level_reverb)
            }
            Self::REVERB_LEVEL_DRY_NAME => {
                self.state_read_elem(segment, elem_value, |state| state.level_dry)
            }
            Self::REVERB_ALGORITHM_NAME => {
                self.state_read_elem(segment, elem_value, |state| u32::from(state.algorithm))
            }
            _ => Ok(false),
        }
    }

    pub fn measure_states<T, S, M>(&mut self, unit: &SndDice, proto: &T, state_segment: &TcKonnektSegment<S>,
                                   meter_segment: &mut TcKonnektSegment<M>, timeout_ms: u32)
        -> Result<(), Error>
        where T: TcKonnektSegmentProtocol<FwNode, M>,
              S: TcKonnektSegmentData + AsRef<ReverbState> + AsMut<ReverbState>,
              TcKonnektSegment<S>: TcKonnektSegmentSpec,
              M: TcKonnektSegmentData + AsRef<ReverbMeter> + AsMut<ReverbMeter>,
              TcKonnektSegment<M>: TcKonnektSegmentSpec,
    {
        if !state_segment.data.as_ref().bypass {
            proto.read_segment(&unit.get_node(), meter_segment, timeout_ms)
        } else {
            Ok(())
        }
    }

    pub fn read_measured_elem<M>(&self, segment: &TcKonnektSegment<M>, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
        where M: TcKonnektSegmentData + AsRef<ReverbMeter>,
    {
        match elem_id.get_name().as_str() {
            Self::REVERB_OUTPUT_METER_NAME => {
                let meter = segment.data.as_ref();
                elem_value.set_int(&meter.outputs);
                Ok(true)
            }
            Self::REVERB_INPUT_METER_NAME => {
                let meter = segment.data.as_ref();
                elem_value.set_int(&meter.inputs);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
