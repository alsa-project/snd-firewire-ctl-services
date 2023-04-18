// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use super::*;

#[derive(Default, Debug)]
pub struct ChStripStateCtl<T, U>
where
    T: TcKonnektSegmentOperation<U>
        + TcKonnektMutableSegmentOperation<U>
        + TcKonnektNotifiedSegmentOperation<U>,
    TcKonnektSegment<U>: Default,
    U: Debug + Clone + AsRef<[ChStripState]> + AsMut<[ChStripState]>,
{
    pub elem_id_list: Vec<ElemId>,
    segment: TcKonnektSegment<U>,
    _phantom: PhantomData<T>,
}

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

const SRC_TYPES: &[ChStripSrcType] = &[
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

impl<T, U> ChStripStateCtl<T, U>
where
    T: TcKonnektSegmentOperation<U>
        + TcKonnektMutableSegmentOperation<U>
        + TcKonnektNotifiedSegmentOperation<U>,
    TcKonnektSegment<U>: Default,
    U: Debug + Clone + AsRef<[ChStripState]> + AsMut<[ChStripState]>,
{
    pub fn are_bypassed(&self) -> bool {
        self.segment
            .data
            .as_ref()
            .iter()
            .find(|s| s.bypass)
            .is_some()
    }

    pub fn cache(&mut self, req: &FwReq, node: &FwNode, timeout_ms: u32) -> Result<(), Error> {
        let res = T::cache_whole_segment(req, node, &mut self.segment, timeout_ms);
        debug!(params = ?self.segment.data, ?res);
        res
    }

    pub fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let channels = self.segment.data.as_ref().len();

        // For overall.
        let labels: Vec<&str> = SRC_TYPES.iter().map(|t| src_type_to_str(t)).collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, SRC_TYPE_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, channels, &labels, None, true)
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        [
            DEESSER_BYPASS_NAME,
            EQ_BYPASS_NAME,
            LIMITTER_BYPASS_NAME,
            BYPASS_NAME,
        ]
        .iter()
        .try_for_each(|name| {
            let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
            card_cntr
                .add_bool_elems(&elem_id, 1, channels, true)
                .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))
        })?;

        // For compressors.
        [COMP_INPUT_GAIN_NAME, COMP_MAKE_UP_GAIN_NAME]
            .iter()
            .try_for_each(|name| {
                let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
                card_cntr
                    .add_int_elems(
                        &elem_id,
                        1,
                        COMP_GAIN_MIN,
                        COMP_GAIN_MAX,
                        COMP_GAIN_STEP,
                        channels,
                        Some(&Into::<Vec<u32>>::into(COMP_GAIN_TLV)),
                        true,
                    )
                    .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))
            })?;

        let elem_id =
            ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, COMP_FULL_BAND_ENABLE_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, 1, channels, true)
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, COMP_CTL_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                3,
                COMP_CTL_MIN,
                COMP_CTL_MAX,
                COMP_CTL_STEP,
                channels,
                None,
                true,
            )
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, COMP_LEVEL_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                3,
                COMP_LEVEL_MIN,
                COMP_LEVEL_MAX,
                COMP_LEVEL_STEP,
                channels,
                Some(&Into::<Vec<u32>>::into(COMP_LEVEL_TLV)),
                true,
            )
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        // For deessers.
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, DEESSER_RATIO_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                DEESSER_RATIO_MIN,
                DEESSER_RATIO_MAX,
                DEESSER_RATIO_STEP,
                channels,
                Some(&Into::<Vec<u32>>::into(DEESSER_RATIO_TLV)),
                true,
            )
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        // For equalizers.
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, EQ_ENABLE_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, 1, channels, true)
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, EQ_BANDWIDTH_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                4,
                EQ_BANDWIDTH_MIN,
                EQ_BANDWIDTH_MAX,
                EQ_BANDWIDTH_STEP,
                channels,
                None,
                true,
            )
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, EQ_GAIN_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                4,
                EQ_GAIN_MIN,
                EQ_GAIN_MAX,
                EQ_GAIN_STEP,
                channels,
                Some(&Into::<Vec<u32>>::into(EQ_GAIN_TLV)),
                true,
            )
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, EQ_FREQ_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                4,
                EQ_FREQ_MIN,
                EQ_FREQ_MAX,
                EQ_FREQ_STEP,
                channels,
                None,
                true,
            )
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        // For limitters.
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, LIMITTER_THRESHOLD_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                LIMITTER_THRESHOLD_MIN,
                LIMITTER_THRESHOLD_MAX,
                LIMITTER_THRESHOLD_STEP,
                channels,
                Some(&Into::<Vec<u32>>::into(LIMITTER_THRESHOLD_TLV)),
                true,
            )
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        Ok(())
    }

    pub fn read(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            SRC_TYPE_NAME => {
                let params = self.segment.data.as_ref();
                let vals: Vec<u32> = params
                    .iter()
                    .map(|param| {
                        let pos = SRC_TYPES.iter().position(|t| param.src_type.eq(t)).unwrap();
                        pos as u32
                    })
                    .collect();
                elem_value.set_enum(&vals);
                Ok(true)
            }
            DEESSER_BYPASS_NAME => {
                let params = self.segment.data.as_ref();
                let vals: Vec<bool> = params.iter().map(|param| param.deesser.bypass).collect();
                elem_value.set_bool(&vals);
                Ok(true)
            }
            EQ_BYPASS_NAME => {
                let params = self.segment.data.as_ref();
                let vals: Vec<bool> = params.iter().map(|param| param.eq_bypass).collect();
                elem_value.set_bool(&vals);
                Ok(true)
            }
            LIMITTER_BYPASS_NAME => {
                let params = self.segment.data.as_ref();
                let vals: Vec<bool> = params.iter().map(|param| param.limitter_bypass).collect();
                elem_value.set_bool(&vals);
                Ok(true)
            }
            BYPASS_NAME => {
                let params = self.segment.data.as_ref();
                let vals: Vec<bool> = params.iter().map(|param| param.bypass).collect();
                elem_value.set_bool(&vals);
                Ok(true)
            }
            COMP_INPUT_GAIN_NAME => {
                let params = self.segment.data.as_ref();
                let vals: Vec<i32> = params
                    .iter()
                    .map(|param| param.comp.input_gain as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            COMP_MAKE_UP_GAIN_NAME => {
                let params = self.segment.data.as_ref();
                let vals: Vec<i32> = params
                    .iter()
                    .map(|param| param.comp.make_up_gain as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            COMP_FULL_BAND_ENABLE_NAME => {
                let params = self.segment.data.as_ref();
                let vals: Vec<bool> = params
                    .iter()
                    .map(|param| param.comp.full_band_enabled)
                    .collect();
                elem_value.set_bool(&vals);
                Ok(true)
            }
            COMP_CTL_NAME => {
                let params = self.segment.data.as_ref();
                let idx = elem_id.index() as usize;
                let vals: Vec<i32> = params
                    .iter()
                    .map(|params| params.comp.ctl[idx] as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            COMP_LEVEL_NAME => {
                let params = self.segment.data.as_ref();
                let idx = elem_id.index() as usize;
                let vals: Vec<i32> = params
                    .iter()
                    .map(|param| param.comp.level[idx] as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            DEESSER_RATIO_NAME => {
                let params = self.segment.data.as_ref();
                let vals: Vec<i32> = params
                    .iter()
                    .map(|param| param.deesser.ratio as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            EQ_ENABLE_NAME => {
                let params = self.segment.data.as_ref();
                let idx = elem_id.index() as usize;
                let vals: Vec<bool> = params.iter().map(|param| param.eq[idx].enabled).collect();
                elem_value.set_bool(&vals);
                Ok(true)
            }
            EQ_BANDWIDTH_NAME => {
                let params = self.segment.data.as_ref();
                let idx = elem_id.index() as usize;
                let vals: Vec<i32> = params
                    .iter()
                    .map(|param| param.eq[idx].bandwidth as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            EQ_GAIN_NAME => {
                let params = self.segment.data.as_ref();
                let idx = elem_id.index() as usize;
                let vals: Vec<i32> = params
                    .iter()
                    .map(|param| param.eq[idx].gain as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            EQ_FREQ_NAME => {
                let params = self.segment.data.as_ref();
                let idx = elem_id.index() as usize;
                let vals: Vec<i32> = params
                    .iter()
                    .map(|param| param.eq[idx].freq as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            LIMITTER_THRESHOLD_NAME => {
                let params = self.segment.data.as_ref();
                let vals: Vec<i32> = params
                    .iter()
                    .map(|param| param.limitter.threshold as i32)
                    .collect();
                elem_value.set_int(&vals);
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
            SRC_TYPE_NAME => {
                let mut data = self.segment.data.clone();
                let params = data.as_mut();
                params
                    .iter_mut()
                    .zip(elem_value.enumerated())
                    .try_for_each(|(param, &val)| {
                        let pos = val as usize;
                        SRC_TYPES
                            .iter()
                            .nth(pos)
                            .ok_or_else(|| {
                                let msg = format!("Source type not found for position {}", pos);
                                Error::new(FileError::Inval, &msg)
                            })
                            .map(|&t| param.src_type = t)
                    })?;
                let res =
                    T::update_partial_segment(req, node, &data, &mut self.segment, timeout_ms);
                debug!(params = ?self.segment.data, ?res);
                res.map(|_| true)
            }
            DEESSER_BYPASS_NAME => {
                let mut data = self.segment.data.clone();
                let params = data.as_mut();
                params
                    .iter_mut()
                    .zip(elem_value.boolean())
                    .for_each(|(param, val)| param.deesser.bypass = val);
                let res =
                    T::update_partial_segment(req, node, &data, &mut self.segment, timeout_ms);
                debug!(params = ?self.segment.data, ?res);
                res.map(|_| true)
            }
            EQ_BYPASS_NAME => {
                let mut data = self.segment.data.clone();
                let params = data.as_mut();
                params
                    .iter_mut()
                    .zip(elem_value.boolean())
                    .for_each(|(param, val)| param.eq_bypass = val);
                let res =
                    T::update_partial_segment(req, node, &data, &mut self.segment, timeout_ms);
                debug!(params = ?self.segment.data, ?res);
                res.map(|_| true)
            }
            LIMITTER_BYPASS_NAME => {
                let mut data = self.segment.data.clone();
                let params = data.as_mut();
                params
                    .iter_mut()
                    .zip(elem_value.boolean())
                    .for_each(|(param, val)| param.limitter_bypass = val);
                let res =
                    T::update_partial_segment(req, node, &data, &mut self.segment, timeout_ms);
                debug!(params = ?self.segment.data, ?res);
                res.map(|_| true)
            }
            BYPASS_NAME => {
                let mut data = self.segment.data.clone();
                let params = data.as_mut();
                params
                    .iter_mut()
                    .zip(elem_value.boolean())
                    .for_each(|(param, val)| param.bypass = val);
                let res =
                    T::update_partial_segment(req, node, &data, &mut self.segment, timeout_ms);
                debug!(params = ?self.segment.data, ?res);
                res.map(|_| true)
            }
            COMP_INPUT_GAIN_NAME => {
                let mut data = self.segment.data.clone();
                let params = data.as_mut();
                params
                    .iter_mut()
                    .zip(elem_value.int())
                    .for_each(|(params, &val)| params.comp.input_gain = val as u32);
                let res =
                    T::update_partial_segment(req, node, &data, &mut self.segment, timeout_ms);
                debug!(params = ?self.segment.data, ?res);
                res.map(|_| true)
            }
            COMP_MAKE_UP_GAIN_NAME => {
                let mut data = self.segment.data.clone();
                let params = data.as_mut();
                params
                    .iter_mut()
                    .zip(elem_value.int())
                    .for_each(|(params, &val)| params.comp.make_up_gain = val as u32);
                let res =
                    T::update_partial_segment(req, node, &data, &mut self.segment, timeout_ms);
                debug!(params = ?self.segment.data, ?res);
                res.map(|_| true)
            }
            COMP_FULL_BAND_ENABLE_NAME => {
                let mut data = self.segment.data.clone();
                let params = data.as_mut();
                params
                    .iter_mut()
                    .zip(elem_value.boolean())
                    .for_each(|(params, val)| params.comp.full_band_enabled = val);
                let res =
                    T::update_partial_segment(req, node, &data, &mut self.segment, timeout_ms);
                debug!(params = ?self.segment.data, ?res);
                res.map(|_| true)
            }
            COMP_CTL_NAME => {
                let idx = elem_id.index() as usize;
                let mut data = self.segment.data.clone();
                let params = data.as_mut();
                params
                    .iter_mut()
                    .zip(elem_value.int())
                    .for_each(|(params, &val)| params.comp.ctl[idx] = val as u32);
                let res =
                    T::update_partial_segment(req, node, &data, &mut self.segment, timeout_ms);
                debug!(params = ?self.segment.data, ?res);
                res.map(|_| true)
            }
            COMP_LEVEL_NAME => {
                let idx = elem_id.index() as usize;
                let mut data = self.segment.data.clone();
                let params = data.as_mut();
                params
                    .iter_mut()
                    .zip(elem_value.int())
                    .for_each(|(params, &val)| params.comp.level[idx] = val as u32);
                let res =
                    T::update_partial_segment(req, node, &data, &mut self.segment, timeout_ms);
                debug!(params = ?self.segment.data, ?res);
                res.map(|_| true)
            }
            DEESSER_RATIO_NAME => {
                let mut data = self.segment.data.clone();
                let params = data.as_mut();
                params
                    .iter_mut()
                    .zip(elem_value.int())
                    .for_each(|(params, &val)| params.deesser.ratio = val as u32);
                let res =
                    T::update_partial_segment(req, node, &data, &mut self.segment, timeout_ms);
                debug!(params = ?self.segment.data, ?res);
                res.map(|_| true)
            }
            EQ_ENABLE_NAME => {
                let idx = elem_id.index() as usize;
                let mut data = self.segment.data.clone();
                let params = data.as_mut();
                params
                    .iter_mut()
                    .zip(elem_value.boolean())
                    .for_each(|(param, val)| param.eq[idx].enabled = val);
                let res =
                    T::update_partial_segment(req, node, &data, &mut self.segment, timeout_ms);
                debug!(params = ?self.segment.data, ?res);
                res.map(|_| true)
            }
            EQ_BANDWIDTH_NAME => {
                let idx = elem_id.index() as usize;
                let mut data = self.segment.data.clone();
                let params = data.as_mut();
                params
                    .iter_mut()
                    .zip(elem_value.int())
                    .for_each(|(param, &val)| param.eq[idx].bandwidth = val as u32);
                let res =
                    T::update_partial_segment(req, node, &data, &mut self.segment, timeout_ms);
                debug!(params = ?self.segment.data, ?res);
                res.map(|_| true)
            }
            EQ_GAIN_NAME => {
                let idx = elem_id.index() as usize;
                let mut data = self.segment.data.clone();
                let params = data.as_mut();
                params
                    .iter_mut()
                    .zip(elem_value.int())
                    .for_each(|(param, &val)| param.eq[idx].gain = val as u32);
                let res =
                    T::update_partial_segment(req, node, &data, &mut self.segment, timeout_ms);
                debug!(params = ?self.segment.data, ?res);
                res.map(|_| true)
            }
            EQ_FREQ_NAME => {
                let idx = elem_id.index() as usize;
                let mut data = self.segment.data.clone();
                let params = data.as_mut();
                params
                    .iter_mut()
                    .zip(elem_value.int())
                    .for_each(|(param, &val)| param.eq[idx].freq = val as u32);
                let res =
                    T::update_partial_segment(req, node, &data, &mut self.segment, timeout_ms);
                debug!(params = ?self.segment.data, ?res);
                res.map(|_| true)
            }
            LIMITTER_THRESHOLD_NAME => {
                let mut data = self.segment.data.clone();
                let params = data.as_mut();
                params
                    .iter_mut()
                    .zip(elem_value.int())
                    .for_each(|(param, &val)| param.limitter.threshold = val as u32);
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
pub struct ChStripMeterCtl<T, U>
where
    T: TcKonnektSegmentOperation<U>,
    TcKonnektSegment<U>: Default,
    U: Debug + Clone + AsRef<[ChStripMeter]> + AsMut<[ChStripMeter]>,
{
    pub elem_id_list: Vec<ElemId>,
    segment: TcKonnektSegment<U>,
    _phantom: PhantomData<T>,
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

impl<T, U> ChStripMeterCtl<T, U>
where
    T: TcKonnektSegmentOperation<U>,
    TcKonnektSegment<U>: Default,
    U: Debug + Clone + AsRef<[ChStripMeter]> + AsMut<[ChStripMeter]>,
{
    pub fn cache(&mut self, req: &FwReq, node: &FwNode, timeout_ms: u32) -> Result<(), Error> {
        let res = T::cache_whole_segment(req, node, &mut self.segment, timeout_ms);
        debug!(params = ?self.segment.data, ?res);
        res
    }

    pub fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let count = self.segment.data.as_ref().len();

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
                .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))
        })
    }

    pub fn read(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            INPUT_METER_NAME => {
                let params = self.segment.data.as_ref();
                let vals: Vec<i32> = params.iter().map(|param| param.input).collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            LIMIT_METER_NAME => {
                let params = self.segment.data.as_ref();
                let vals: Vec<i32> = params.iter().map(|param| param.limit).collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            OUTPUT_METER_NAME => {
                let params = self.segment.data.as_ref();
                let vals: Vec<i32> = params.iter().map(|param| param.output).collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            GAIN_METER_NAME => {
                let idx = elem_id.index() as usize;
                let params = self.segment.data.as_ref();
                let vals: Vec<i32> = params.iter().map(|param| param.gains[idx]).collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
