// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use super::*;

const REVERB_INPUT_LEVEL_NAME: &str = "reverb-input-level";
const REVERB_BYPASS_NAME: &str = "reverb-bypass";
const REVERB_KILL_WET: &str = "reverb-kill-wet";
const REVERB_KILL_DRY: &str = "reverb-kill-dry";
const REVERB_OUTPUT_LEVEL_NAME: &str = "reverb-output-level";
const REVERB_TIME_DECAY_NAME: &str = "reverb-time-decay";
const REVERB_TIME_PRE_DECAY_NAME: &str = "reverb-time-pre-decay";
const REVERB_COLOR_LOW_NAME: &str = "reverb-color-low";
const REVERB_COLOR_HIGH_NAME: &str = "reverb-color-high";
const REVERB_COLOR_HIGH_FACTOR_NAME: &str = "reverb-color-high-factor";
const REVERB_MOD_RATE_NAME: &str = "reverb-mod-rate";
const REVERB_MOD_DEPTH_NAME: &str = "reverb-mod-depth";
const REVERB_LEVEL_EARLY_NAME: &str = "reverb-level-early";
const REVERB_LEVEL_REVERB_NAME: &str = "reverb-level-reverb";
const REVERB_LEVEL_DRY_NAME: &str = "reverb-level-dry";
const REVERB_ALGORITHM_NAME: &str = "reverb-algorithm";
const REVERB_OUTPUT_METER_NAME: &str = "reverb-output-meter";
const REVERB_INPUT_METER_NAME: &str = "reverb-input-meter";

fn reverb_algorithm_to_str(algo: &ReverbAlgorithm) -> &'static str {
    match algo {
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
    }
}

pub trait ReverbCtlOperation<S, T, U>
where
    S: Clone,
    U: TcKonnektSegmentOperation<S>
        + TcKonnektSegmentOperation<T>
        + TcKonnektMutableSegmentOperation<S>
        + TcKonnektNotifiedSegmentOperation<S>,
{
    fn state_segment(&self) -> &TcKonnektSegment<S>;
    fn state_segment_mut(&mut self) -> &mut TcKonnektSegment<S>;

    fn meter_segment(&self) -> &TcKonnektSegment<T>;
    fn meter_segment_mut(&mut self) -> &mut TcKonnektSegment<T>;

    fn state(params: &S) -> &ReverbState;
    fn state_mut(params: &mut S) -> &mut ReverbState;

    fn meter(parms: &T) -> &ReverbMeter;

    const INPUT_LEVEL_MIN: i32 = -240;
    const INPUT_LEVEL_MAX: i32 = 0;
    const INPUT_LEVEL_STEP: i32 = 1;
    const INPUT_LEVEL_TLV: DbInterval = DbInterval {
        min: -2400,
        max: 0,
        linear: false,
        mute_avail: false,
    };

    const OUTPUT_LEVEL_MIN: i32 = -240;
    const OUTPUT_LEVEL_MAX: i32 = 120;
    const OUTPUT_LEVEL_STEP: i32 = 1;
    const OUTPUT_LEVEL_TLV: DbInterval = DbInterval {
        min: -2400,
        max: 1200,
        linear: false,
        mute_avail: false,
    };

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
    const METER_OUTPUT_TLV: DbInterval = DbInterval {
        min: -2400,
        max: 1200,
        linear: false,
        mute_avail: false,
    };

    const METER_INPUT_MIN: i32 = -1000;
    const METER_INPUT_MAX: i32 = 0;
    const METER_INPUT_STEP: i32 = 1;
    const METER_INPUT_TLV: DbInterval = DbInterval {
        min: -2400,
        max: 0,
        linear: false,
        mute_avail: false,
    };

    const ALGORITHMS: [ReverbAlgorithm; 14] = [
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
    ];

    fn cache(&mut self, req: &FwReq, node: &FwNode, timeout_ms: u32) -> Result<(), Error> {
        U::cache_whole_segment(req, node, self.state_segment_mut(), timeout_ms)?;
        U::cache_whole_segment(req, node, self.meter_segment_mut(), timeout_ms)?;
        Ok(())
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(Vec<ElemId>, Vec<ElemId>), Error> {
        let mut notified_elem_id_list = Vec::new();

        state_add_int_elem(
            card_cntr,
            &mut notified_elem_id_list,
            REVERB_INPUT_LEVEL_NAME,
            Self::INPUT_LEVEL_MIN,
            Self::INPUT_LEVEL_MAX,
            Self::INPUT_LEVEL_STEP,
            1,
            Some(&Into::<Vec<u32>>::into(Self::INPUT_LEVEL_TLV)),
            true,
        )?;

        state_add_bool_elem(
            card_cntr,
            &mut notified_elem_id_list,
            REVERB_BYPASS_NAME,
            1,
            true,
        )?;
        state_add_bool_elem(
            card_cntr,
            &mut notified_elem_id_list,
            REVERB_KILL_WET,
            1,
            true,
        )?;
        state_add_bool_elem(
            card_cntr,
            &mut notified_elem_id_list,
            REVERB_KILL_DRY,
            1,
            true,
        )?;

        state_add_int_elem(
            card_cntr,
            &mut notified_elem_id_list,
            REVERB_OUTPUT_LEVEL_NAME,
            Self::OUTPUT_LEVEL_MIN,
            Self::OUTPUT_LEVEL_MAX,
            Self::OUTPUT_LEVEL_STEP,
            1,
            Some(&Into::<Vec<u32>>::into(Self::OUTPUT_LEVEL_TLV)),
            true,
        )?;

        state_add_int_elem(
            card_cntr,
            &mut notified_elem_id_list,
            REVERB_TIME_DECAY_NAME,
            Self::DECAY_MIN,
            Self::DECAY_MAX,
            Self::DECAY_STEP,
            1,
            None,
            true,
        )?;
        state_add_int_elem(
            card_cntr,
            &mut notified_elem_id_list,
            REVERB_TIME_PRE_DECAY_NAME,
            Self::PRE_DECAY_MIN,
            Self::PRE_DECAY_MAX,
            Self::PRE_DECAY_STEP,
            1,
            None,
            true,
        )?;

        state_add_int_elem(
            card_cntr,
            &mut notified_elem_id_list,
            REVERB_COLOR_LOW_NAME,
            Self::COLOR_MIN,
            Self::COLOR_MAX,
            Self::COLOR_STEP,
            1,
            None,
            true,
        )?;
        state_add_int_elem(
            card_cntr,
            &mut notified_elem_id_list,
            REVERB_COLOR_HIGH_NAME,
            Self::COLOR_MIN,
            Self::COLOR_MAX,
            Self::COLOR_STEP,
            1,
            None,
            true,
        )?;
        state_add_int_elem(
            card_cntr,
            &mut notified_elem_id_list,
            REVERB_COLOR_HIGH_FACTOR_NAME,
            Self::FACTOR_MIN,
            Self::FACTOR_MAX,
            Self::FACTOR_STEP,
            1,
            None,
            true,
        )?;

        state_add_int_elem(
            card_cntr,
            &mut notified_elem_id_list,
            REVERB_MOD_RATE_NAME,
            Self::FACTOR_MIN,
            Self::FACTOR_MAX,
            Self::FACTOR_STEP,
            1,
            None,
            true,
        )?;
        state_add_int_elem(
            card_cntr,
            &mut notified_elem_id_list,
            REVERB_MOD_DEPTH_NAME,
            Self::FACTOR_MIN,
            Self::FACTOR_MAX,
            Self::FACTOR_STEP,
            1,
            None,
            true,
        )?;

        state_add_int_elem(
            card_cntr,
            &mut notified_elem_id_list,
            REVERB_LEVEL_EARLY_NAME,
            Self::LEVEL_MIN,
            Self::LEVEL_MAX,
            Self::LEVEL_STEP,
            1,
            None,
            true,
        )?;
        state_add_int_elem(
            card_cntr,
            &mut notified_elem_id_list,
            REVERB_LEVEL_REVERB_NAME,
            Self::LEVEL_MIN,
            Self::LEVEL_MAX,
            Self::LEVEL_STEP,
            1,
            None,
            true,
        )?;
        state_add_int_elem(
            card_cntr,
            &mut notified_elem_id_list,
            REVERB_LEVEL_DRY_NAME,
            Self::LEVEL_MIN,
            Self::LEVEL_MAX,
            Self::LEVEL_STEP,
            1,
            None,
            true,
        )?;

        let labels: Vec<&str> = Self::ALGORITHMS
            .iter()
            .map(|algo| reverb_algorithm_to_str(algo))
            .collect();
        state_add_enum_elem(
            card_cntr,
            &mut notified_elem_id_list,
            REVERB_ALGORITHM_NAME,
            1,
            &labels,
            true,
        )?;

        let mut measured_elem_id_list = Vec::new();

        meter_add_int_elem(
            card_cntr,
            &mut measured_elem_id_list,
            REVERB_OUTPUT_METER_NAME,
            Self::METER_OUTPUT_MIN,
            Self::METER_OUTPUT_MAX,
            Self::METER_OUTPUT_STEP,
            2,
            Some(&Into::<Vec<u32>>::into(Self::METER_OUTPUT_TLV)),
            false,
        )?;

        meter_add_int_elem(
            card_cntr,
            &mut measured_elem_id_list,
            REVERB_INPUT_METER_NAME,
            Self::METER_INPUT_MIN,
            Self::METER_INPUT_MAX,
            Self::METER_INPUT_STEP,
            2,
            Some(&Into::<Vec<u32>>::into(Self::METER_INPUT_TLV)),
            true,
        )?;

        Ok((notified_elem_id_list, measured_elem_id_list))
    }

    fn read(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        if self.read_notified_elem(elem_id, elem_value)? {
            Ok(true)
        } else if self.read_measured_elem(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(
        &mut self,
        req: &FwReq,
        node: &FwNode,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            REVERB_INPUT_LEVEL_NAME => {
                let mut params = self.state_segment().data.clone();
                let mut state = Self::state_mut(&mut params);
                state.input_level = elem_value.int()[0];
                U::update_partial_segment(req, node, &params, self.state_segment_mut(), timeout_ms)
                    .map(|_| true)
            }
            REVERB_BYPASS_NAME => {
                let mut params = self.state_segment().data.clone();
                let mut state = Self::state_mut(&mut params);
                state.bypass = elem_value.boolean()[0];
                U::update_partial_segment(req, node, &params, self.state_segment_mut(), timeout_ms)
                    .map(|_| true)
            }
            REVERB_KILL_WET => {
                let mut params = self.state_segment().data.clone();
                let mut state = Self::state_mut(&mut params);
                state.kill_wet = elem_value.boolean()[0];
                U::update_partial_segment(req, node, &params, self.state_segment_mut(), timeout_ms)
                    .map(|_| true)
            }
            REVERB_KILL_DRY => {
                let mut params = self.state_segment().data.clone();
                let mut state = Self::state_mut(&mut params);
                state.kill_dry = elem_value.boolean()[0];
                U::update_partial_segment(req, node, &params, self.state_segment_mut(), timeout_ms)
                    .map(|_| true)
            }
            REVERB_OUTPUT_LEVEL_NAME => {
                let mut params = self.state_segment().data.clone();
                let mut state = Self::state_mut(&mut params);
                state.output_level = elem_value.int()[0];
                U::update_partial_segment(req, node, &params, self.state_segment_mut(), timeout_ms)
                    .map(|_| true)
            }
            REVERB_TIME_DECAY_NAME => {
                let mut params = self.state_segment().data.clone();
                let mut state = Self::state_mut(&mut params);
                state.time_decay = elem_value.int()[0];
                U::update_partial_segment(req, node, &params, self.state_segment_mut(), timeout_ms)
                    .map(|_| true)
            }
            REVERB_TIME_PRE_DECAY_NAME => {
                let mut params = self.state_segment().data.clone();
                let mut state = Self::state_mut(&mut params);
                state.time_pre_decay = elem_value.int()[0];
                U::update_partial_segment(req, node, &params, self.state_segment_mut(), timeout_ms)
                    .map(|_| true)
            }
            REVERB_COLOR_LOW_NAME => {
                let mut params = self.state_segment().data.clone();
                let mut state = Self::state_mut(&mut params);
                state.color_low = elem_value.int()[0];
                U::update_partial_segment(req, node, &params, self.state_segment_mut(), timeout_ms)
                    .map(|_| true)
            }
            REVERB_COLOR_HIGH_NAME => {
                let mut params = self.state_segment().data.clone();
                let mut state = Self::state_mut(&mut params);
                state.color_high = elem_value.int()[0];
                U::update_partial_segment(req, node, &params, self.state_segment_mut(), timeout_ms)
                    .map(|_| true)
            }
            REVERB_COLOR_HIGH_FACTOR_NAME => {
                let mut params = self.state_segment().data.clone();
                let mut state = Self::state_mut(&mut params);
                state.color_high_factor = elem_value.int()[0];
                U::update_partial_segment(req, node, &params, self.state_segment_mut(), timeout_ms)
                    .map(|_| true)
            }
            REVERB_MOD_RATE_NAME => {
                let mut params = self.state_segment().data.clone();
                let mut state = Self::state_mut(&mut params);
                state.mod_rate = elem_value.int()[0];
                U::update_partial_segment(req, node, &params, self.state_segment_mut(), timeout_ms)
                    .map(|_| true)
            }
            REVERB_MOD_DEPTH_NAME => {
                let mut params = self.state_segment().data.clone();
                let mut state = Self::state_mut(&mut params);
                state.mod_depth = elem_value.int()[0];
                U::update_partial_segment(req, node, &params, self.state_segment_mut(), timeout_ms)
                    .map(|_| true)
            }
            REVERB_LEVEL_EARLY_NAME => {
                let mut params = self.state_segment().data.clone();
                let mut state = Self::state_mut(&mut params);
                state.level_early = elem_value.int()[0];
                U::update_partial_segment(req, node, &params, self.state_segment_mut(), timeout_ms)
                    .map(|_| true)
            }
            REVERB_LEVEL_REVERB_NAME => {
                let mut params = self.state_segment().data.clone();
                let mut state = Self::state_mut(&mut params);
                state.level_reverb = elem_value.int()[0];
                U::update_partial_segment(req, node, &params, self.state_segment_mut(), timeout_ms)
                    .map(|_| true)
            }
            REVERB_LEVEL_DRY_NAME => {
                let mut params = self.state_segment().data.clone();
                let mut state = Self::state_mut(&mut params);
                state.level_dry = elem_value.int()[0];
                U::update_partial_segment(req, node, &params, self.state_segment_mut(), timeout_ms)
                    .map(|_| true)
            }
            REVERB_ALGORITHM_NAME => {
                let mut params = self.state_segment().data.clone();
                let mut state = Self::state_mut(&mut params);
                let pos = elem_value.enumerated()[0] as usize;
                state.algorithm = Self::ALGORITHMS
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| {
                        let msg = format!("reverb algorithm not found for position {}", pos);
                        Error::new(FileError::Inval, &msg)
                    })
                    .copied()?;
                U::update_partial_segment(req, node, &params, self.state_segment_mut(), timeout_ms)
                    .map(|_| true)
            }
            _ => Ok(false),
        }
    }

    fn read_notified_elem(
        &self,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            REVERB_INPUT_LEVEL_NAME => {
                let params = &self.state_segment().data;
                let state = Self::state(&params);
                elem_value.set_int(&[state.input_level]);
                Ok(true)
            }
            REVERB_BYPASS_NAME => {
                let params = &self.state_segment().data;
                let state = Self::state(&params);
                elem_value.set_bool(&[state.bypass]);
                Ok(true)
            }
            REVERB_KILL_WET => {
                let params = &self.state_segment().data;
                let state = Self::state(&params);
                elem_value.set_bool(&[state.kill_wet]);
                Ok(true)
            }
            REVERB_KILL_DRY => {
                let params = &self.state_segment().data;
                let state = Self::state(&params);
                elem_value.set_bool(&[state.kill_dry]);
                Ok(true)
            }
            REVERB_OUTPUT_LEVEL_NAME => {
                let params = &self.state_segment().data;
                let state = Self::state(&params);
                elem_value.set_int(&[state.output_level]);
                Ok(true)
            }
            REVERB_TIME_DECAY_NAME => {
                let params = &self.state_segment().data;
                let state = Self::state(&params);
                elem_value.set_int(&[state.time_decay]);
                Ok(true)
            }
            REVERB_TIME_PRE_DECAY_NAME => {
                let params = &self.state_segment().data;
                let state = Self::state(&params);
                elem_value.set_int(&[state.time_pre_decay]);
                Ok(true)
            }
            REVERB_COLOR_LOW_NAME => {
                let params = &self.state_segment().data;
                let state = Self::state(&params);
                elem_value.set_int(&[state.color_low]);
                Ok(true)
            }
            REVERB_COLOR_HIGH_NAME => {
                let params = &self.state_segment().data;
                let state = Self::state(&params);
                elem_value.set_int(&[state.color_high]);
                Ok(true)
            }
            REVERB_COLOR_HIGH_FACTOR_NAME => {
                let params = &self.state_segment().data;
                let state = Self::state(&params);
                elem_value.set_int(&[state.color_high_factor]);
                Ok(true)
            }
            REVERB_MOD_RATE_NAME => {
                let params = &self.state_segment().data;
                let state = Self::state(&params);
                elem_value.set_int(&[state.mod_rate]);
                Ok(true)
            }
            REVERB_MOD_DEPTH_NAME => {
                let params = &self.state_segment().data;
                let state = Self::state(&params);
                elem_value.set_int(&[state.mod_depth]);
                Ok(true)
            }
            REVERB_LEVEL_EARLY_NAME => {
                let params = &self.state_segment().data;
                let state = Self::state(&params);
                elem_value.set_int(&[state.level_early]);
                Ok(true)
            }
            REVERB_LEVEL_REVERB_NAME => {
                let params = &self.state_segment().data;
                let state = Self::state(&params);
                elem_value.set_int(&[state.level_reverb]);
                Ok(true)
            }
            REVERB_LEVEL_DRY_NAME => {
                let params = &self.state_segment().data;
                let state = Self::state(&params);
                elem_value.set_int(&[state.level_dry]);
                Ok(true)
            }
            REVERB_ALGORITHM_NAME => {
                let params = &self.state_segment().data;
                let state = Self::state(&params);
                let pos = Self::ALGORITHMS
                    .iter()
                    .position(|algo| state.algorithm.eq(algo))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn read_measured_elem(
        &self,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            REVERB_OUTPUT_METER_NAME => {
                let params = &self.meter_segment().data;
                let meter = Self::meter(&params);
                elem_value.set_int(&meter.outputs);
                Ok(true)
            }
            REVERB_INPUT_METER_NAME => {
                let params = &self.meter_segment().data;
                let meter = Self::meter(&params);
                elem_value.set_int(&meter.inputs);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn measure_states(&mut self, req: &FwReq, node: &FwNode, timeout_ms: u32) -> Result<(), Error> {
        if !Self::state(&self.state_segment().data).bypass {
            U::cache_whole_segment(req, node, self.meter_segment_mut(), timeout_ms)
        } else {
            Ok(())
        }
    }

    fn parse_notification(
        &mut self,
        req: &FwReq,
        node: &FwNode,
        msg: u32,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        if U::is_notified_segment(self.state_segment(), msg) {
            U::cache_whole_segment(req, node, self.state_segment_mut(), timeout_ms)
        } else {
            Ok(())
        }
    }
}

fn state_add_int_elem(
    card_cntr: &mut CardCntr,
    notified_elem_id_list: &mut Vec<ElemId>,
    name: &str,
    min: i32,
    max: i32,
    step: i32,
    value_count: usize,
    tlv: Option<&[u32]>,
    locked: bool,
) -> Result<(), Error> {
    let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
    card_cntr
        .add_int_elems(&elem_id, 1, min, max, step, value_count, tlv, locked)
        .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))
}

fn state_add_enum_elem<T: AsRef<str>>(
    card_cntr: &mut CardCntr,
    notified_elem_id_list: &mut Vec<ElemId>,
    name: &str,
    value_count: usize,
    labels: &[T],
    locked: bool,
) -> Result<(), Error> {
    let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
    card_cntr
        .add_enum_elems(&elem_id, 1, value_count, labels, None, locked)
        .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))
}

fn state_add_bool_elem(
    card_cntr: &mut CardCntr,
    notified_elem_id_list: &mut Vec<ElemId>,
    name: &str,
    value_count: usize,
    locked: bool,
) -> Result<(), Error> {
    let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
    card_cntr
        .add_bool_elems(&elem_id, 1, value_count, locked)
        .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))
}

fn meter_add_int_elem(
    card_cntr: &mut CardCntr,
    measured_elem_id_list: &mut Vec<ElemId>,
    name: &str,
    min: i32,
    max: i32,
    step: i32,
    value_count: usize,
    tlv: Option<&[u32]>,
    locked: bool,
) -> Result<(), Error> {
    let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
    card_cntr
        .add_int_elems(&elem_id, 1, min, max, step, value_count, tlv, locked)
        .map(|mut elem_id_list| measured_elem_id_list.append(&mut elem_id_list))
}
