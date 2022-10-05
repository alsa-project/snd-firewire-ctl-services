// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use super::*;

const SRC_TYPE_NAME: &str = "ch-strip-source-type";
const DEESSER_BYPASS_NAME: &str = "deesser-bypass";
const EQ_BYPASS_NAME: &str = "equalizer-bypass";
const LIMITTER_BYPASS_NAME: &str = "limitter-bypass";
const BYPASS_NAME: &str = "ch-strip-bypass";

const COMP_INPUT_GAIN_NAME: &str = "comp-input-gain";
const COMP_MAKE_UP_GAIN_NAME: &str = "comp-make-up-gain";
const COMP_FULL_BAND_ENABLE_NAME: &str = "comp-full-band-enable";
const COMP_CTL_NAME: &str = "comp-control";
const COMP_LEVEL_NAME: &str = "comp-level";

const DEESSER_RATIO_NAME: &str = "deesser-ratio";

const EQ_ENABLE_NAME: &str = "equalizer-enable";
const EQ_BANDWIDTH_NAME: &str = "equalizer-bandwidth";
const EQ_GAIN_NAME: &str = "equalizer-gain";
const EQ_FREQ_NAME: &str = "equalizer-freq";

const LIMITTER_THRESHOLD_NAME: &str = "limitter-threshold";

const INPUT_METER_NAME: &str = "ch-strip-input-meter";
const LIMIT_METER_NAME: &str = "ch-strip-limit-meter";
const OUTPUT_METER_NAME: &str = "ch-strip-output-meter";
const GAIN_METER_NAME: &str = "ch-strip-gain-meter";

fn src_type_to_str(t: &ChStripSrcType) -> &'static str {
    match t {
        ChStripSrcType::FemaleVocal => "Female-vocal",
        ChStripSrcType::MaleVocal => "Male-vocal",
        ChStripSrcType::Guitar => "Guitar",
        ChStripSrcType::Piano => "Piano",
        ChStripSrcType::Speak => "Speak",
        ChStripSrcType::Choir => "Choir",
        ChStripSrcType::Horns => "Horns",
        ChStripSrcType::Bass => "Bass",
        ChStripSrcType::Kick => "Kick",
        ChStripSrcType::Snare => "Snare",
        ChStripSrcType::MixRock => "Mix-rock",
        ChStripSrcType::MixSoft => "Mix-soft",
        ChStripSrcType::Percussion => "Percussion",
        ChStripSrcType::Kit => "Kit",
        ChStripSrcType::MixAcoustic => "Mix-acoustic",
        ChStripSrcType::MixPurist => "Mix-purist",
        ChStripSrcType::House => "House",
        ChStripSrcType::Trance => "Trance",
        ChStripSrcType::Chill => "Chill",
        ChStripSrcType::HipHop => "Hip-hop",
        ChStripSrcType::DrumAndBass => "Drum'n'bass",
        ChStripSrcType::ElectroTechno => "Electro-techno",
    }
}

pub trait ChStripStateCtlOperation<S, T>
where
    S: Clone,
    T: TcKonnektSegmentOperation<S>
        + TcKonnektMutableSegmentOperation<S>
        + TcKonnektNotifiedSegmentOperation<S>,
{
    fn segment(&self) -> &TcKonnektSegment<S>;
    fn segment_mut(&mut self) -> &mut TcKonnektSegment<S>;

    fn states(params: &S) -> &[ChStripState];
    fn states_mut(params: &mut S) -> &mut [ChStripState];

    const COMP_GAIN_MIN: i32 = 0;
    const COMP_GAIN_MAX: i32 = 36;
    const COMP_GAIN_STEP: i32 = 1;
    const COMP_GAIN_TLV: DbInterval = DbInterval {
        min: -1800,
        max: 1800,
        linear: false,
        mute_avail: false,
    };

    const COMP_CTL_MIN: i32 = 0;
    const COMP_CTL_MAX: i32 = 200;
    const COMP_CTL_STEP: i32 = 1;

    const COMP_LEVEL_MIN: i32 = 0;
    const COMP_LEVEL_MAX: i32 = 48;
    const COMP_LEVEL_STEP: i32 = 1;
    const COMP_LEVEL_TLV: DbInterval = DbInterval {
        min: -1800,
        max: 600,
        linear: false,
        mute_avail: false,
    };

    const DEESSER_RATIO_MIN: i32 = 0;
    const DEESSER_RATIO_MAX: i32 = 10;
    const DEESSER_RATIO_STEP: i32 = 1;
    const DEESSER_RATIO_TLV: DbInterval = DbInterval {
        min: 0,
        max: 100,
        linear: false,
        mute_avail: false,
    };

    const EQ_BANDWIDTH_MIN: i32 = 0;
    const EQ_BANDWIDTH_MAX: i32 = 39;
    const EQ_BANDWIDTH_STEP: i32 = 1;

    const EQ_GAIN_MIN: i32 = 0;
    const EQ_GAIN_MAX: i32 = 240;
    const EQ_GAIN_STEP: i32 = 1;
    const EQ_GAIN_TLV: DbInterval = DbInterval {
        min: -1200,
        max: 1200,
        linear: false,
        mute_avail: false,
    };

    const EQ_FREQ_MIN: i32 = 0;
    const EQ_FREQ_MAX: i32 = 240;
    const EQ_FREQ_STEP: i32 = 1;

    const LIMITTER_THRESHOLD_MIN: i32 = 0;
    const LIMITTER_THRESHOLD_MAX: i32 = 72;
    const LIMITTER_THRESHOLD_STEP: i32 = 1;
    const LIMITTER_THRESHOLD_TLV: DbInterval = DbInterval {
        min: -1200,
        max: 0,
        linear: false,
        mute_avail: false,
    };

    const LIMIT_METER_MIN: i32 = -12;
    const LIMIT_METER_MAX: i32 = 0;
    const LIMIT_METER_STEP: i32 = 1;
    const LIMIT_METER_TLV: DbInterval = DbInterval {
        min: -1200,
        max: 0,
        linear: false,
        mute_avail: false,
    };

    const INOUT_METER_MIN: i32 = -72;
    const INOUT_METER_MAX: i32 = 0;
    const INOUT_METER_STEP: i32 = 1;
    const INOUT_METER_TLV: DbInterval = DbInterval {
        min: -7200,
        max: 0,
        linear: false,
        mute_avail: false,
    };

    const GAIN_METER_MIN: i32 = -24;
    const GAIN_METER_MAX: i32 = 18;
    const GAIN_METER_STEP: i32 = 1;
    const GAIN_METER_TLV: DbInterval = DbInterval {
        min: -2400,
        max: 1800,
        linear: false,
        mute_avail: false,
    };

    const SRC_TYPES: [ChStripSrcType; 22] = [
        ChStripSrcType::FemaleVocal,
        ChStripSrcType::MaleVocal,
        ChStripSrcType::Guitar,
        ChStripSrcType::Piano,
        ChStripSrcType::Speak,
        ChStripSrcType::Choir,
        ChStripSrcType::Horns,
        ChStripSrcType::Bass,
        ChStripSrcType::Kick,
        ChStripSrcType::Snare,
        ChStripSrcType::MixRock,
        ChStripSrcType::MixSoft,
        ChStripSrcType::Percussion,
        ChStripSrcType::Kit,
        ChStripSrcType::MixAcoustic,
        ChStripSrcType::MixPurist,
        ChStripSrcType::House,
        ChStripSrcType::Trance,
        ChStripSrcType::Chill,
        ChStripSrcType::HipHop,
        ChStripSrcType::DrumAndBass,
        ChStripSrcType::ElectroTechno,
    ];

    fn are_bypassed(&self) -> bool {
        Self::states(&self.segment().data)
            .iter()
            .find(|s| s.bypass)
            .is_some()
    }

    fn cache(&mut self, req: &FwReq, node: &FwNode, timeout_ms: u32) -> Result<(), Error> {
        T::cache_whole_segment(req, node, self.segment_mut(), timeout_ms)
    }

    fn load(&self, card_cntr: &mut CardCntr) -> Result<Vec<ElemId>, Error> {
        let channels = Self::states(&self.segment().data).len();
        let mut notified_elem_id_list = Vec::new();

        // Overall controls.
        let labels: Vec<&str> = Self::SRC_TYPES.iter().map(|t| src_type_to_str(t)).collect();
        state_add_enum_elem(
            card_cntr,
            &mut notified_elem_id_list,
            channels,
            SRC_TYPE_NAME,
            1,
            &labels,
            true,
        )?;
        state_add_bool_elem(
            card_cntr,
            &mut notified_elem_id_list,
            channels,
            DEESSER_BYPASS_NAME,
            1,
            true,
        )?;
        state_add_bool_elem(
            card_cntr,
            &mut notified_elem_id_list,
            channels,
            EQ_BYPASS_NAME,
            1,
            true,
        )?;
        state_add_bool_elem(
            card_cntr,
            &mut notified_elem_id_list,
            channels,
            LIMITTER_BYPASS_NAME,
            1,
            true,
        )?;
        state_add_bool_elem(
            card_cntr,
            &mut notified_elem_id_list,
            channels,
            BYPASS_NAME,
            1,
            true,
        )?;

        // Controls for compressor part.
        state_add_int_elem(
            card_cntr,
            &mut notified_elem_id_list,
            channels,
            COMP_INPUT_GAIN_NAME,
            1,
            Self::COMP_GAIN_MIN,
            Self::COMP_GAIN_MAX,
            Self::COMP_GAIN_STEP,
            Some(&Into::<Vec<u32>>::into(Self::COMP_GAIN_TLV)),
            true,
        )?;
        state_add_int_elem(
            card_cntr,
            &mut notified_elem_id_list,
            channels,
            COMP_MAKE_UP_GAIN_NAME,
            1,
            Self::COMP_GAIN_MIN,
            Self::COMP_GAIN_MAX,
            Self::COMP_GAIN_STEP,
            Some(&Into::<Vec<u32>>::into(Self::COMP_GAIN_TLV)),
            true,
        )?;
        state_add_bool_elem(
            card_cntr,
            &mut notified_elem_id_list,
            channels,
            COMP_FULL_BAND_ENABLE_NAME,
            1,
            true,
        )?;
        state_add_int_elem(
            card_cntr,
            &mut notified_elem_id_list,
            channels,
            COMP_CTL_NAME,
            3,
            Self::COMP_CTL_MIN,
            Self::COMP_CTL_MAX,
            Self::COMP_CTL_STEP,
            None,
            true,
        )?;
        state_add_int_elem(
            card_cntr,
            &mut notified_elem_id_list,
            channels,
            COMP_LEVEL_NAME,
            3,
            Self::COMP_LEVEL_MIN,
            Self::COMP_LEVEL_MAX,
            Self::COMP_LEVEL_STEP,
            Some(&Into::<Vec<u32>>::into(Self::COMP_LEVEL_TLV)),
            true,
        )?;

        // Controls for deesser part.
        state_add_int_elem(
            card_cntr,
            &mut notified_elem_id_list,
            channels,
            DEESSER_RATIO_NAME,
            1,
            Self::DEESSER_RATIO_MIN,
            Self::DEESSER_RATIO_MAX,
            Self::DEESSER_RATIO_STEP,
            Some(&Into::<Vec<u32>>::into(Self::DEESSER_RATIO_TLV)),
            true,
        )?;

        // Controls for equalizer part.
        state_add_bool_elem(
            card_cntr,
            &mut notified_elem_id_list,
            channels,
            EQ_ENABLE_NAME,
            4,
            true,
        )?;
        state_add_int_elem(
            card_cntr,
            &mut notified_elem_id_list,
            channels,
            EQ_BANDWIDTH_NAME,
            4,
            Self::EQ_BANDWIDTH_MIN,
            Self::EQ_BANDWIDTH_MAX,
            Self::EQ_BANDWIDTH_STEP,
            None,
            true,
        )?;
        state_add_int_elem(
            card_cntr,
            &mut notified_elem_id_list,
            channels,
            EQ_GAIN_NAME,
            4,
            Self::EQ_GAIN_MIN,
            Self::EQ_GAIN_MAX,
            Self::EQ_GAIN_STEP,
            Some(&Into::<Vec<u32>>::into(Self::EQ_GAIN_TLV)),
            true,
        )?;
        state_add_int_elem(
            card_cntr,
            &mut notified_elem_id_list,
            channels,
            EQ_FREQ_NAME,
            4,
            Self::EQ_FREQ_MIN,
            Self::EQ_FREQ_MAX,
            Self::EQ_FREQ_STEP,
            None,
            true,
        )?;

        // Controls for limitter part.
        state_add_int_elem(
            card_cntr,
            &mut notified_elem_id_list,
            channels,
            LIMITTER_THRESHOLD_NAME,
            1,
            Self::LIMITTER_THRESHOLD_MIN,
            Self::LIMITTER_THRESHOLD_MAX,
            Self::LIMITTER_THRESHOLD_STEP,
            Some(&Into::<Vec<u32>>::into(Self::LIMITTER_THRESHOLD_TLV)),
            true,
        )?;

        Ok(notified_elem_id_list)
    }

    fn read(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            SRC_TYPE_NAME => {
                let params = &self.segment().data;
                let states = Self::states(&params);
                let vals: Vec<u32> = states
                    .iter()
                    .map(|state| {
                        let pos = Self::SRC_TYPES
                            .iter()
                            .position(|t| state.src_type.eq(t))
                            .unwrap();
                        pos as u32
                    })
                    .collect();
                elem_value.set_enum(&vals);
                Ok(true)
            }
            DEESSER_BYPASS_NAME => {
                let params = &self.segment().data;
                let states = Self::states(&params);
                let vals: Vec<bool> = states.iter().map(|state| state.deesser.bypass).collect();
                elem_value.set_bool(&vals);
                Ok(true)
            }
            EQ_BYPASS_NAME => {
                let params = &self.segment().data;
                let states = Self::states(&params);
                let vals: Vec<bool> = states.iter().map(|state| state.eq_bypass).collect();
                elem_value.set_bool(&vals);
                Ok(true)
            }
            LIMITTER_BYPASS_NAME => {
                let params = &self.segment().data;
                let states = Self::states(&params);
                let vals: Vec<bool> = states.iter().map(|state| state.limitter_bypass).collect();
                elem_value.set_bool(&vals);
                Ok(true)
            }
            BYPASS_NAME => {
                let params = &self.segment().data;
                let states = Self::states(&params);
                let vals: Vec<bool> = states.iter().map(|state| state.bypass).collect();
                elem_value.set_bool(&vals);
                Ok(true)
            }
            COMP_INPUT_GAIN_NAME => {
                let params = &self.segment().data;
                let states = Self::states(&params);
                let vals: Vec<i32> = states
                    .iter()
                    .map(|state| state.comp.input_gain as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            COMP_MAKE_UP_GAIN_NAME => {
                let params = &self.segment().data;
                let states = Self::states(&params);
                let vals: Vec<i32> = states
                    .iter()
                    .map(|state| state.comp.make_up_gain as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            COMP_FULL_BAND_ENABLE_NAME => {
                let params = &self.segment().data;
                let states = Self::states(&params);
                let vals: Vec<bool> = states
                    .iter()
                    .map(|state| state.comp.full_band_enabled)
                    .collect();
                elem_value.set_bool(&vals);
                Ok(true)
            }
            COMP_CTL_NAME => {
                let params = &self.segment().data;
                let states = Self::states(&params);
                let idx = elem_id.index() as usize;
                let vals: Vec<i32> = states
                    .iter()
                    .map(|state| state.comp.ctl[idx] as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            COMP_LEVEL_NAME => {
                let params = &self.segment().data;
                let states = Self::states(&params);
                let idx = elem_id.index() as usize;
                let vals: Vec<i32> = states
                    .iter()
                    .map(|state| state.comp.level[idx] as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            DEESSER_RATIO_NAME => {
                let params = &self.segment().data;
                let states = Self::states(&params);
                let vals: Vec<i32> = states
                    .iter()
                    .map(|state| state.deesser.ratio as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            EQ_ENABLE_NAME => {
                let params = &self.segment().data;
                let states = Self::states(&params);
                let idx = elem_id.index() as usize;
                let vals: Vec<bool> = states.iter().map(|state| state.eq[idx].enabled).collect();
                elem_value.set_bool(&vals);
                Ok(true)
            }
            EQ_BANDWIDTH_NAME => {
                let params = &self.segment().data;
                let states = Self::states(&params);
                let idx = elem_id.index() as usize;
                let vals: Vec<i32> = states
                    .iter()
                    .map(|state| state.eq[idx].bandwidth as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            EQ_GAIN_NAME => {
                let params = &self.segment().data;
                let states = Self::states(&params);
                let idx = elem_id.index() as usize;
                let vals: Vec<i32> = states
                    .iter()
                    .map(|state| state.eq[idx].gain as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            EQ_FREQ_NAME => {
                let params = &self.segment().data;
                let states = Self::states(&params);
                let idx = elem_id.index() as usize;
                let vals: Vec<i32> = states
                    .iter()
                    .map(|state| state.eq[idx].freq as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            LIMITTER_THRESHOLD_NAME => {
                let params = &self.segment().data;
                let states = Self::states(&params);
                let vals: Vec<i32> = states
                    .iter()
                    .map(|state| state.limitter.threshold as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            _ => Ok(false),
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
            SRC_TYPE_NAME => {
                let mut params = self.segment().data.clone();
                let states = Self::states_mut(&mut params);
                states
                    .iter_mut()
                    .zip(elem_value.enumerated())
                    .try_for_each(|(state, &val)| {
                        let pos = val as usize;
                        Self::SRC_TYPES
                            .iter()
                            .nth(val as usize)
                            .ok_or_else(|| {
                                let msg = format!("Source type not found for position {}", pos);
                                Error::new(FileError::Inval, &msg)
                            })
                            .map(|&t| state.src_type = t)
                    })?;
                T::update_partial_segment(req, node, &params, self.segment_mut(), timeout_ms)
                    .map(|_| true)
            }
            DEESSER_BYPASS_NAME => {
                let mut params = self.segment().data.clone();
                let states = Self::states_mut(&mut params);
                states
                    .iter_mut()
                    .zip(elem_value.boolean())
                    .for_each(|(state, val)| state.deesser.bypass = val);
                T::update_partial_segment(req, node, &params, self.segment_mut(), timeout_ms)
                    .map(|_| true)
            }
            EQ_BYPASS_NAME => {
                let mut params = self.segment().data.clone();
                let states = Self::states_mut(&mut params);
                states
                    .iter_mut()
                    .zip(elem_value.boolean())
                    .for_each(|(state, val)| state.eq_bypass = val);
                T::update_partial_segment(req, node, &params, self.segment_mut(), timeout_ms)
                    .map(|_| true)
            }
            LIMITTER_BYPASS_NAME => {
                let mut params = self.segment().data.clone();
                let states = Self::states_mut(&mut params);
                states
                    .iter_mut()
                    .zip(elem_value.boolean())
                    .for_each(|(state, val)| state.limitter_bypass = val);
                T::update_partial_segment(req, node, &params, self.segment_mut(), timeout_ms)
                    .map(|_| true)
            }
            BYPASS_NAME => {
                let mut params = self.segment().data.clone();
                let states = Self::states_mut(&mut params);
                states
                    .iter_mut()
                    .zip(elem_value.boolean())
                    .for_each(|(state, val)| state.bypass = val);
                T::update_partial_segment(req, node, &params, self.segment_mut(), timeout_ms)
                    .map(|_| true)
            }
            COMP_INPUT_GAIN_NAME => {
                let mut params = self.segment().data.clone();
                let states = Self::states_mut(&mut params);
                states
                    .iter_mut()
                    .zip(elem_value.int())
                    .for_each(|(state, &val)| state.comp.input_gain = val as u32);
                T::update_partial_segment(req, node, &params, self.segment_mut(), timeout_ms)
                    .map(|_| true)
            }
            COMP_MAKE_UP_GAIN_NAME => {
                let mut params = self.segment().data.clone();
                let states = Self::states_mut(&mut params);
                states
                    .iter_mut()
                    .zip(elem_value.int())
                    .for_each(|(state, &val)| state.comp.make_up_gain = val as u32);
                T::update_partial_segment(req, node, &params, self.segment_mut(), timeout_ms)
                    .map(|_| true)
            }
            COMP_FULL_BAND_ENABLE_NAME => {
                let mut params = self.segment().data.clone();
                let states = Self::states_mut(&mut params);
                states
                    .iter_mut()
                    .zip(elem_value.boolean())
                    .for_each(|(state, val)| state.comp.full_band_enabled = val);
                T::update_partial_segment(req, node, &params, self.segment_mut(), timeout_ms)
                    .map(|_| true)
            }
            COMP_CTL_NAME => {
                let mut params = self.segment().data.clone();
                let states = Self::states_mut(&mut params);
                let idx = elem_id.index() as usize;
                states
                    .iter_mut()
                    .zip(elem_value.int())
                    .for_each(|(state, &val)| state.comp.ctl[idx] = val as u32);
                T::update_partial_segment(req, node, &params, self.segment_mut(), timeout_ms)
                    .map(|_| true)
            }
            COMP_LEVEL_NAME => {
                let mut params = self.segment().data.clone();
                let states = Self::states_mut(&mut params);
                let idx = elem_id.index() as usize;
                states
                    .iter_mut()
                    .zip(elem_value.int())
                    .for_each(|(state, &val)| state.comp.level[idx] = val as u32);
                T::update_partial_segment(req, node, &params, self.segment_mut(), timeout_ms)
                    .map(|_| true)
            }
            DEESSER_RATIO_NAME => {
                let mut params = self.segment().data.clone();
                let states = Self::states_mut(&mut params);
                states
                    .iter_mut()
                    .zip(elem_value.int())
                    .for_each(|(state, &val)| state.deesser.ratio = val as u32);
                T::update_partial_segment(req, node, &params, self.segment_mut(), timeout_ms)
                    .map(|_| true)
            }
            EQ_ENABLE_NAME => {
                let mut params = self.segment().data.clone();
                let states = Self::states_mut(&mut params);
                let idx = elem_id.index() as usize;
                states
                    .iter_mut()
                    .zip(elem_value.boolean())
                    .for_each(|(state, val)| state.eq[idx].enabled = val);
                T::update_partial_segment(req, node, &params, self.segment_mut(), timeout_ms)
                    .map(|_| true)
            }
            EQ_BANDWIDTH_NAME => {
                let mut params = self.segment().data.clone();
                let states = Self::states_mut(&mut params);
                let idx = elem_id.index() as usize;
                states
                    .iter_mut()
                    .zip(elem_value.int())
                    .for_each(|(state, &val)| state.eq[idx].bandwidth = val as u32);
                T::update_partial_segment(req, node, &params, self.segment_mut(), timeout_ms)
                    .map(|_| true)
            }
            EQ_GAIN_NAME => {
                let mut params = self.segment().data.clone();
                let states = Self::states_mut(&mut params);
                let idx = elem_id.index() as usize;
                states
                    .iter_mut()
                    .zip(elem_value.int())
                    .for_each(|(state, &val)| state.eq[idx].gain = val as u32);
                T::update_partial_segment(req, node, &params, self.segment_mut(), timeout_ms)
                    .map(|_| true)
            }
            EQ_FREQ_NAME => {
                let mut params = self.segment().data.clone();
                let states = Self::states_mut(&mut params);
                let idx = elem_id.index() as usize;
                states
                    .iter_mut()
                    .zip(elem_value.int())
                    .for_each(|(state, &val)| state.eq[idx].freq = val as u32);
                T::update_partial_segment(req, node, &params, self.segment_mut(), timeout_ms)
                    .map(|_| true)
            }
            LIMITTER_THRESHOLD_NAME => {
                let mut params = self.segment().data.clone();
                let states = Self::states_mut(&mut params);
                states
                    .iter_mut()
                    .zip(elem_value.int())
                    .for_each(|(state, &val)| state.limitter.threshold = val as u32);
                T::update_partial_segment(req, node, &params, self.segment_mut(), timeout_ms)
                    .map(|_| true)
            }
            _ => Ok(false),
        }
    }

    fn parse_notification(
        &mut self,
        req: &FwReq,
        node: &FwNode,
        msg: u32,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        if T::is_notified_segment(self.segment(), msg) {
            T::cache_whole_segment(req, node, self.segment_mut(), timeout_ms)
        } else {
            Ok(())
        }
    }
}

fn state_add_int_elem(
    card_cntr: &mut CardCntr,
    notified_elem_id_list: &mut Vec<ElemId>,
    channels: usize,
    name: &str,
    count: usize,
    min: i32,
    max: i32,
    step: i32,
    tlv: Option<&[u32]>,
    unlock: bool,
) -> Result<(), Error> {
    let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
    card_cntr
        .add_int_elems(&elem_id, count, min, max, step, channels, tlv, unlock)
        .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))
}

fn state_add_enum_elem<T: AsRef<str>>(
    card_cntr: &mut CardCntr,
    notified_elem_id_list: &mut Vec<ElemId>,
    channels: usize,
    name: &str,
    count: usize,
    labels: &[T],
    locked: bool,
) -> Result<(), Error> {
    let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
    card_cntr
        .add_enum_elems(&elem_id, count, channels, labels, None, locked)
        .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))
}

fn state_add_bool_elem(
    card_cntr: &mut CardCntr,
    notified_elem_id_list: &mut Vec<ElemId>,
    channels: usize,
    name: &str,
    count: usize,
    unlock: bool,
) -> Result<(), Error> {
    let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
    card_cntr
        .add_bool_elems(&elem_id, count, channels, unlock)
        .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))
}

const LIMIT_METER_MIN: i32 = -12;
const LIMIT_METER_MAX: i32 = 0;
const LIMIT_METER_STEP: i32 = 1;
const LIMIT_METER_TLV: DbInterval = DbInterval {
    min: -1200,
    max: 0,
    linear: false,
    mute_avail: false,
};

const INOUT_METER_MIN: i32 = -72;
const INOUT_METER_MAX: i32 = 0;
const INOUT_METER_STEP: i32 = 1;
const INOUT_METER_TLV: DbInterval = DbInterval {
    min: -7200,
    max: 0,
    linear: false,
    mute_avail: false,
};

const GAIN_METER_MIN: i32 = -24;
const GAIN_METER_MAX: i32 = 18;
const GAIN_METER_STEP: i32 = 1;
const GAIN_METER_TLV: DbInterval = DbInterval {
    min: -2400,
    max: 1800,
    linear: false,
    mute_avail: false,
};

pub trait ChStripMeterCtlOperation<S, T>
where
    T: TcKonnektSegmentOperation<S>,
{
    fn meters(&self) -> &[ChStripMeter];

    fn segment(&self) -> &TcKonnektSegment<S>;
    fn segment_mut(&mut self) -> &mut TcKonnektSegment<S>;

    fn cache(&mut self, req: &FwReq, node: &FwNode, timeout_ms: u32) -> Result<(), Error> {
        T::cache_whole_segment(req, node, self.segment_mut(), timeout_ms)
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<Vec<ElemId>, Error> {
        let mut measured_elem_id_list = Vec::new();

        let count = self.meters().len();

        [
            (
                INPUT_METER_NAME,
                INOUT_METER_MIN,
                INOUT_METER_MAX,
                INOUT_METER_STEP,
                INOUT_METER_TLV,
            ),
            (
                LIMIT_METER_NAME,
                LIMIT_METER_MIN,
                LIMIT_METER_MAX,
                LIMIT_METER_STEP,
                LIMIT_METER_TLV,
            ),
            (
                OUTPUT_METER_NAME,
                INOUT_METER_MIN,
                INOUT_METER_MAX,
                INOUT_METER_STEP,
                INOUT_METER_TLV,
            ),
            (
                GAIN_METER_NAME,
                GAIN_METER_MIN,
                GAIN_METER_MAX,
                GAIN_METER_STEP,
                GAIN_METER_TLV,
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
                    count,
                    Some(&Into::<Vec<u32>>::into(tlv)),
                    false,
                )
                .map(|mut elem_id_list| measured_elem_id_list.append(&mut elem_id_list))
        })
        .map(|_| measured_elem_id_list)
    }

    fn read(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            INPUT_METER_NAME => {
                let meters = self.meters();
                let vals: Vec<i32> = meters.iter().map(|meter| meter.input).collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            LIMIT_METER_NAME => {
                let meters = self.meters();
                let vals: Vec<i32> = meters.iter().map(|meter| meter.limit).collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            OUTPUT_METER_NAME => {
                let meters = self.meters();
                let vals: Vec<i32> = meters.iter().map(|meter| meter.output).collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            GAIN_METER_NAME => {
                let meters = self.meters();
                let idx = elem_id.index() as usize;
                let vals: Vec<i32> = meters.iter().map(|meter| meter.gains[idx]).collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
