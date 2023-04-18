// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use super::*;

#[derive(Default, Debug)]
pub struct ReverbStateCtl<T, U>
where
    T: TcKonnektSegmentOperation<U>
        + TcKonnektMutableSegmentOperation<U>
        + TcKonnektNotifiedSegmentOperation<U>,
    TcKonnektSegment<U>: Default,
    U: Debug + Clone + AsRef<ReverbState> + AsMut<ReverbState>,
{
    pub elem_id_list: Vec<ElemId>,
    segment: TcKonnektSegment<U>,
    _phantom: PhantomData<T>,
}

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

const ALGORITHMS: &[ReverbAlgorithm] = &[
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

fn reverb_algorithm_to_str(algo: &ReverbAlgorithm) -> &str {
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

impl<T, U> ReverbStateCtl<T, U>
where
    T: TcKonnektSegmentOperation<U>
        + TcKonnektMutableSegmentOperation<U>
        + TcKonnektNotifiedSegmentOperation<U>,
    TcKonnektSegment<U>: Default,
    U: Debug + Clone + AsRef<ReverbState> + AsMut<ReverbState>,
{
    pub fn is_bypassed(&self) -> bool {
        self.segment.data.as_ref().bypass
    }

    pub fn cache(&mut self, req: &FwReq, node: &FwNode, timeout_ms: u32) -> Result<(), Error> {
        let res = T::cache_whole_segment(req, node, &mut self.segment, timeout_ms);
        debug!(params = ?self.segment.data, ?res);
        res
    }

    pub fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, REVERB_INPUT_LEVEL_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                INPUT_LEVEL_MIN,
                INPUT_LEVEL_MAX,
                INPUT_LEVEL_STEP,
                1,
                Some(&Into::<Vec<u32>>::into(INPUT_LEVEL_TLV)),
                true,
            )
            .map(|mut list| self.elem_id_list.append(&mut list))?;

        [REVERB_BYPASS_NAME, REVERB_KILL_WET, REVERB_KILL_DRY]
            .iter()
            .try_for_each(|name| {
                let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
                card_cntr
                    .add_bool_elems(&elem_id, 1, 1, true)
                    .map(|mut list| self.elem_id_list.append(&mut list))
            })?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, REVERB_TIME_DECAY_NAME, 0);
        card_cntr
            .add_int_elems(&elem_id, 1, DECAY_MIN, DECAY_MAX, DECAY_STEP, 1, None, true)
            .map(|mut list| self.elem_id_list.append(&mut list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, REVERB_OUTPUT_LEVEL_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                OUTPUT_LEVEL_MIN,
                OUTPUT_LEVEL_MAX,
                OUTPUT_LEVEL_STEP,
                1,
                Some(&Into::<Vec<u32>>::into(OUTPUT_LEVEL_TLV)),
                true,
            )
            .map(|mut list| self.elem_id_list.append(&mut list))?;

        let elem_id =
            ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, REVERB_TIME_PRE_DECAY_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                PRE_DECAY_MIN,
                PRE_DECAY_MAX,
                PRE_DECAY_STEP,
                1,
                None,
                true,
            )
            .map(|mut list| self.elem_id_list.append(&mut list))?;

        [REVERB_COLOR_LOW_NAME, REVERB_COLOR_HIGH_NAME]
            .iter()
            .try_for_each(|name| {
                let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
                card_cntr
                    .add_int_elems(&elem_id, 1, COLOR_MIN, COLOR_MAX, COLOR_STEP, 1, None, true)
                    .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))
            })?;

        [REVERB_MOD_RATE_NAME, REVERB_MOD_DEPTH_NAME]
            .iter()
            .try_for_each(|name| {
                let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
                card_cntr
                    .add_int_elems(
                        &elem_id,
                        1,
                        FACTOR_MIN,
                        FACTOR_MAX,
                        FACTOR_STEP,
                        1,
                        None,
                        true,
                    )
                    .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))
            })?;

        [
            REVERB_LEVEL_EARLY_NAME,
            REVERB_LEVEL_REVERB_NAME,
            REVERB_LEVEL_DRY_NAME,
        ]
        .iter()
        .try_for_each(|name| {
            let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
            card_cntr
                .add_int_elems(&elem_id, 1, LEVEL_MIN, LEVEL_MAX, LEVEL_STEP, 1, None, true)
                .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))
        })?;

        let labels: Vec<&str> = ALGORITHMS
            .iter()
            .map(|algo| reverb_algorithm_to_str(algo))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, REVERB_ALGORITHM_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, true)
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        Ok(())
    }

    pub fn read(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            REVERB_INPUT_LEVEL_NAME => {
                let params = &self.segment.data.as_ref();
                elem_value.set_int(&[params.input_level]);
                Ok(true)
            }
            REVERB_BYPASS_NAME => {
                let params = &self.segment.data.as_ref();
                elem_value.set_bool(&[params.bypass]);
                Ok(true)
            }
            REVERB_KILL_WET => {
                let params = &self.segment.data.as_ref();
                elem_value.set_bool(&[params.kill_wet]);
                Ok(true)
            }
            REVERB_KILL_DRY => {
                let params = &self.segment.data.as_ref();
                elem_value.set_bool(&[params.kill_dry]);
                Ok(true)
            }
            REVERB_OUTPUT_LEVEL_NAME => {
                let params = &self.segment.data.as_ref();
                elem_value.set_int(&[params.output_level]);
                Ok(true)
            }
            REVERB_TIME_DECAY_NAME => {
                let params = &self.segment.data.as_ref();
                elem_value.set_int(&[params.time_decay]);
                Ok(true)
            }
            REVERB_TIME_PRE_DECAY_NAME => {
                let params = &self.segment.data.as_ref();
                elem_value.set_int(&[params.time_pre_decay]);
                Ok(true)
            }
            REVERB_COLOR_LOW_NAME => {
                let params = &self.segment.data.as_ref();
                elem_value.set_int(&[params.color_low]);
                Ok(true)
            }
            REVERB_COLOR_HIGH_NAME => {
                let params = &self.segment.data.as_ref();
                elem_value.set_int(&[params.color_high]);
                Ok(true)
            }
            REVERB_COLOR_HIGH_FACTOR_NAME => {
                let params = &self.segment.data.as_ref();
                elem_value.set_int(&[params.color_high_factor]);
                Ok(true)
            }
            REVERB_MOD_RATE_NAME => {
                let params = &self.segment.data.as_ref();
                elem_value.set_int(&[params.mod_rate]);
                Ok(true)
            }
            REVERB_MOD_DEPTH_NAME => {
                let params = &self.segment.data.as_ref();
                elem_value.set_int(&[params.mod_depth]);
                Ok(true)
            }
            REVERB_LEVEL_EARLY_NAME => {
                let params = &self.segment.data.as_ref();
                elem_value.set_int(&[params.level_early]);
                Ok(true)
            }
            REVERB_LEVEL_REVERB_NAME => {
                let params = &self.segment.data.as_ref();
                elem_value.set_int(&[params.level_reverb]);
                Ok(true)
            }
            REVERB_LEVEL_DRY_NAME => {
                let params = &self.segment.data.as_ref();
                elem_value.set_int(&[params.level_dry]);
                Ok(true)
            }
            REVERB_ALGORITHM_NAME => {
                let params = &self.segment.data.as_ref();
                let pos = ALGORITHMS
                    .iter()
                    .position(|algo| params.algorithm.eq(algo))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub fn write(
        &mut self,
        req: &FwReq,
        node: &FwNode,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            REVERB_INPUT_LEVEL_NAME => {
                let mut data = self.segment.data.clone();
                let params = data.as_mut();
                params.input_level = elem_value.int()[0];
                let res =
                    T::update_partial_segment(req, node, &data, &mut self.segment, timeout_ms);
                debug!(params = ?self.segment.data, ?res);
                res.map(|_| true)
            }
            REVERB_BYPASS_NAME => {
                let mut data = self.segment.data.clone();
                let params = data.as_mut();
                params.bypass = elem_value.boolean()[0];
                let res =
                    T::update_partial_segment(req, node, &data, &mut self.segment, timeout_ms);
                debug!(params = ?self.segment.data, ?res);
                res.map(|_| true)
            }
            REVERB_KILL_WET => {
                let mut data = self.segment.data.clone();
                let params = data.as_mut();
                params.kill_wet = elem_value.boolean()[0];
                let res =
                    T::update_partial_segment(req, node, &data, &mut self.segment, timeout_ms);
                debug!(params = ?self.segment.data, ?res);
                res.map(|_| true)
            }
            REVERB_KILL_DRY => {
                let mut data = self.segment.data.clone();
                let params = data.as_mut();
                params.kill_dry = elem_value.boolean()[0];
                let res =
                    T::update_partial_segment(req, node, &data, &mut self.segment, timeout_ms);
                debug!(params = ?self.segment.data, ?res);
                res.map(|_| true)
            }
            REVERB_OUTPUT_LEVEL_NAME => {
                let mut data = self.segment.data.clone();
                let params = data.as_mut();
                params.output_level = elem_value.int()[0];
                let res =
                    T::update_partial_segment(req, node, &data, &mut self.segment, timeout_ms);
                debug!(params = ?self.segment.data, ?res);
                res.map(|_| true)
            }
            REVERB_TIME_DECAY_NAME => {
                let mut data = self.segment.data.clone();
                let params = data.as_mut();
                params.time_decay = elem_value.int()[0];
                let res =
                    T::update_partial_segment(req, node, &data, &mut self.segment, timeout_ms);
                debug!(params = ?self.segment.data, ?res);
                res.map(|_| true)
            }
            REVERB_TIME_PRE_DECAY_NAME => {
                let mut data = self.segment.data.clone();
                let params = data.as_mut();
                params.time_pre_decay = elem_value.int()[0];
                let res =
                    T::update_partial_segment(req, node, &data, &mut self.segment, timeout_ms);
                debug!(params = ?self.segment.data, ?res);
                res.map(|_| true)
            }
            REVERB_COLOR_LOW_NAME => {
                let mut data = self.segment.data.clone();
                let params = data.as_mut();
                params.color_low = elem_value.int()[0];
                let res =
                    T::update_partial_segment(req, node, &data, &mut self.segment, timeout_ms);
                debug!(params = ?self.segment.data, ?res);
                res.map(|_| true)
            }
            REVERB_COLOR_HIGH_NAME => {
                let mut data = self.segment.data.clone();
                let params = data.as_mut();
                params.color_high = elem_value.int()[0];
                let res =
                    T::update_partial_segment(req, node, &data, &mut self.segment, timeout_ms);
                debug!(params = ?self.segment.data, ?res);
                res.map(|_| true)
            }
            REVERB_COLOR_HIGH_FACTOR_NAME => {
                let mut data = self.segment.data.clone();
                let params = data.as_mut();
                params.color_high_factor = elem_value.int()[0];
                let res =
                    T::update_partial_segment(req, node, &data, &mut self.segment, timeout_ms);
                debug!(params = ?self.segment.data, ?res);
                res.map(|_| true)
            }
            REVERB_MOD_RATE_NAME => {
                let mut data = self.segment.data.clone();
                let params = data.as_mut();
                params.mod_rate = elem_value.int()[0];
                let res =
                    T::update_partial_segment(req, node, &data, &mut self.segment, timeout_ms);
                debug!(params = ?self.segment.data, ?res);
                res.map(|_| true)
            }
            REVERB_MOD_DEPTH_NAME => {
                let mut data = self.segment.data.clone();
                let params = data.as_mut();
                params.mod_depth = elem_value.int()[0];
                let res =
                    T::update_partial_segment(req, node, &data, &mut self.segment, timeout_ms);
                debug!(params = ?self.segment.data, ?res);
                res.map(|_| true)
            }
            REVERB_LEVEL_EARLY_NAME => {
                let mut data = self.segment.data.clone();
                let params = data.as_mut();
                params.level_early = elem_value.int()[0];
                let res =
                    T::update_partial_segment(req, node, &data, &mut self.segment, timeout_ms);
                debug!(params = ?self.segment.data, ?res);
                res.map(|_| true)
            }
            REVERB_LEVEL_REVERB_NAME => {
                let mut data = self.segment.data.clone();
                let params = data.as_mut();
                params.level_reverb = elem_value.int()[0];
                let res =
                    T::update_partial_segment(req, node, &data, &mut self.segment, timeout_ms);
                debug!(params = ?self.segment.data, ?res);
                res.map(|_| true)
            }
            REVERB_LEVEL_DRY_NAME => {
                let mut data = self.segment.data.clone();
                let params = data.as_mut();
                params.level_dry = elem_value.int()[0];
                let res =
                    T::update_partial_segment(req, node, &data, &mut self.segment, timeout_ms);
                debug!(params = ?self.segment.data, ?res);
                res.map(|_| true)
            }
            REVERB_ALGORITHM_NAME => {
                let mut data = self.segment.data.clone();
                let params = data.as_mut();
                let pos = elem_value.enumerated()[0] as usize;
                ALGORITHMS
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| {
                        let msg = format!("reverb algorithm not found for position {}", pos);
                        Error::new(FileError::Inval, &msg)
                    })
                    .map(|&algo| params.algorithm = algo)?;
                let res =
                    T::update_partial_segment(req, node, &data, &mut self.segment, timeout_ms);
                debug!(params = ?self.segment.data, ?res);
                res.map(|_| true)
            }
            _ => Ok(false),
        }
    }

    pub fn parse_notification(
        &mut self,
        req: &FwReq,
        node: &FwNode,
        msg: u32,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        if T::is_notified_segment(&self.segment, msg) {
            let res = T::cache_whole_segment(req, node, &mut self.segment, timeout_ms);
            debug!(params = ?self.segment.data, ?res);
            res
        } else {
            Ok(())
        }
    }
}

#[derive(Default, Debug)]
pub struct ReverbMeterCtl<T, U>
where
    T: TcKonnektSegmentOperation<U>,
    TcKonnektSegment<U>: Default,
    U: Debug + Clone + AsRef<ReverbMeter> + AsMut<ReverbMeter>,
{
    pub elem_id_list: Vec<ElemId>,
    segment: TcKonnektSegment<U>,
    _phantom: PhantomData<T>,
}

const REVERB_OUTPUT_METER_NAME: &str = "reverb-output-meter";
const REVERB_INPUT_METER_NAME: &str = "reverb-input-meter";

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

impl<T, U> ReverbMeterCtl<T, U>
where
    T: TcKonnektSegmentOperation<U>,
    TcKonnektSegment<U>: Default,
    U: Debug + Clone + AsRef<ReverbMeter> + AsMut<ReverbMeter>,
{
    pub fn cache(&mut self, req: &FwReq, node: &FwNode, timeout_ms: u32) -> Result<(), Error> {
        let res = T::cache_whole_segment(req, node, &mut self.segment, timeout_ms);
        debug!(params = ?self.segment.data, ?res);
        res
    }

    pub fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        [
            (
                REVERB_OUTPUT_METER_NAME,
                METER_OUTPUT_MIN,
                METER_OUTPUT_MAX,
                METER_OUTPUT_STEP,
                METER_OUTPUT_TLV,
            ),
            (
                REVERB_INPUT_METER_NAME,
                METER_INPUT_MIN,
                METER_INPUT_MAX,
                METER_INPUT_STEP,
                METER_INPUT_TLV,
            ),
        ]
        .iter()
        .try_for_each(|&(name, min, max, step, tlv)| {
            let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
            card_cntr
                .add_int_elems(
                    &elem_id,
                    1,
                    min,
                    max,
                    step,
                    2,
                    Some(&Into::<Vec<u32>>::into(tlv)),
                    false,
                )
                .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))
        })
    }

    pub fn read(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            REVERB_OUTPUT_METER_NAME => {
                let params = self.segment.data.as_ref();
                elem_value.set_int(&params.outputs);
                Ok(true)
            }
            REVERB_INPUT_METER_NAME => {
                let params = self.segment.data.as_ref();
                elem_value.set_int(&params.inputs);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
