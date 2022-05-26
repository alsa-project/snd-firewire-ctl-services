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
        ChStripSrcType::Reserved(_) => "reserved",
    }
}

pub trait ChStripCtlOperation<S, T, U>
where
    S: TcKonnektSegmentData,
    T: TcKonnektSegmentData,
    TcKonnektSegment<S>: TcKonnektSegmentSpec + TcKonnektNotifiedSegmentSpec,
    TcKonnektSegment<T>: TcKonnektSegmentSpec,
    U: SegmentOperation<S> + SegmentOperation<T>,
{
    fn states_segment(&self) -> &TcKonnektSegment<S>;
    fn states_segment_mut(&mut self) -> &mut TcKonnektSegment<S>;

    fn meters_segment_mut(&mut self) -> &mut TcKonnektSegment<T>;

    fn states(&self) -> &[ChStripState];
    fn states_mut(&mut self) -> &mut [ChStripState];

    fn meters(&self) -> &[ChStripMeter];

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

    fn load(
        &mut self,
        card_cntr: &mut CardCntr,
        unit: &mut (SndDice, FwNode),
        req: &mut FwReq,
        timeout_ms: u32,
    ) -> Result<(Vec<ElemId>, Vec<ElemId>), Error> {
        U::read_segment(req, &mut unit.1, self.states_segment_mut(), timeout_ms)?;
        U::read_segment(req, &mut unit.1, self.meters_segment_mut(), timeout_ms)?;

        let channels = self.states().len();
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

        // Controls for meter segment.
        let channels = self.meters().len();
        let mut measured_elem_id_list = Vec::new();
        meter_add_int_elem(
            card_cntr,
            &mut measured_elem_id_list,
            channels,
            INPUT_METER_NAME,
            1,
            Self::INOUT_METER_MIN,
            Self::INOUT_METER_MAX,
            Self::INOUT_METER_STEP,
            Some(&Into::<Vec<u32>>::into(Self::INOUT_METER_TLV)),
            false,
        )?;

        meter_add_int_elem(
            card_cntr,
            &mut measured_elem_id_list,
            channels,
            LIMIT_METER_NAME,
            1,
            Self::LIMIT_METER_MIN,
            Self::LIMIT_METER_MAX,
            Self::LIMIT_METER_STEP,
            Some(&Into::<Vec<u32>>::into(Self::LIMIT_METER_TLV)),
            false,
        )?;

        meter_add_int_elem(
            card_cntr,
            &mut measured_elem_id_list,
            channels,
            OUTPUT_METER_NAME,
            1,
            Self::INOUT_METER_MIN,
            Self::INOUT_METER_MAX,
            Self::INOUT_METER_STEP,
            Some(&Into::<Vec<u32>>::into(Self::INOUT_METER_TLV)),
            false,
        )?;

        meter_add_int_elem(
            card_cntr,
            &mut measured_elem_id_list,
            channels,
            GAIN_METER_NAME,
            1,
            Self::GAIN_METER_MIN,
            Self::GAIN_METER_MAX,
            Self::GAIN_METER_STEP,
            Some(&Into::<Vec<u32>>::into(Self::GAIN_METER_TLV)),
            false,
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
        unit: &mut (SndDice, FwNode),
        req: &mut FwReq,
        elem_id: &ElemId,
        old: &ElemValue,
        new: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            SRC_TYPE_NAME => {
                self.state_write_elem(unit, req, old, new, timeout_ms, |state, val: u32| {
                    state.src_type = Self::SRC_TYPES
                        .iter()
                        .nth(val as usize)
                        .map(|&t| t)
                        .unwrap_or_default()
                })
            }
            DEESSER_BYPASS_NAME => {
                self.state_write_elem(unit, req, old, new, timeout_ms, |state, val: bool| {
                    state.deesser.bypass = val
                })
            }
            EQ_BYPASS_NAME => {
                self.state_write_elem(unit, req, old, new, timeout_ms, |state, val: bool| {
                    state.eq_bypass = val
                })
            }
            LIMITTER_BYPASS_NAME => {
                self.state_write_elem(unit, req, old, new, timeout_ms, |state, val: bool| {
                    state.limitter_bypass = val
                })
            }
            BYPASS_NAME => {
                self.state_write_elem(unit, req, old, new, timeout_ms, |state, val: bool| {
                    state.bypass = val
                })
            }
            COMP_INPUT_GAIN_NAME => {
                self.state_write_elem(unit, req, old, new, timeout_ms, |state, val: i32| {
                    state.comp.input_gain = val as u32
                })
            }
            COMP_MAKE_UP_GAIN_NAME => {
                self.state_write_elem(unit, req, old, new, timeout_ms, |state, val: i32| {
                    state.comp.make_up_gain = val as u32
                })
            }
            COMP_FULL_BAND_ENABLE_NAME => {
                self.state_write_elem(unit, req, old, new, timeout_ms, |state, val: bool| {
                    state.comp.full_band_enabled = val
                })
            }
            COMP_CTL_NAME => {
                let idx = elem_id.get_index() as usize;
                self.state_write_elem(unit, req, old, new, timeout_ms, |state, val: i32| {
                    state.comp.ctl[idx] = val as u32
                })
            }
            COMP_LEVEL_NAME => {
                let idx = elem_id.get_index() as usize;
                self.state_write_elem(unit, req, old, new, timeout_ms, |state, val: i32| {
                    state.comp.level[idx] = val as u32
                })
            }
            DEESSER_RATIO_NAME => {
                self.state_write_elem(unit, req, old, new, timeout_ms, |state, val: i32| {
                    state.deesser.ratio = val as u32
                })
            }
            EQ_ENABLE_NAME => {
                let idx = elem_id.get_index() as usize;
                self.state_write_elem(unit, req, old, new, timeout_ms, |state, val: bool| {
                    state.eq[idx].enabled = val
                })
            }
            EQ_BANDWIDTH_NAME => {
                let idx = elem_id.get_index() as usize;
                self.state_write_elem(unit, req, old, new, timeout_ms, |state, val: i32| {
                    state.eq[idx].bandwidth = val as u32
                })
            }
            EQ_GAIN_NAME => {
                let idx = elem_id.get_index() as usize;
                self.state_write_elem(unit, req, old, new, timeout_ms, |state, val: i32| {
                    state.eq[idx].gain = val as u32
                })
            }
            EQ_FREQ_NAME => {
                let idx = elem_id.get_index() as usize;
                self.state_write_elem(unit, req, old, new, timeout_ms, |state, val: i32| {
                    state.eq[idx].freq = val as u32
                })
            }
            LIMITTER_THRESHOLD_NAME => {
                self.state_write_elem(unit, req, old, new, timeout_ms, |state, val: i32| {
                    state.limitter.threshold = val as u32
                })
            }
            _ => Ok(false),
        }
    }

    fn state_read_elem<V, F>(&self, elem_value: &ElemValue, cb: F) -> Result<bool, Error>
    where
        F: Fn(&ChStripState) -> V,
        V: Default + Copy + Eq,
        ElemValue: ElemValueAccessor<V>,
    {
        let states = self.states();
        ElemValueAccessor::<V>::set_vals(elem_value, states.len(), |idx| Ok(cb(&states[idx])))
            .map(|_| true)
    }

    fn meter_read_elem<V, F>(&self, elem_value: &ElemValue, cb: F) -> Result<bool, Error>
    where
        F: Fn(&ChStripMeter) -> V,
        V: Default + Copy + Eq,
        ElemValue: ElemValueAccessor<V>,
    {
        let meters = self.meters();
        ElemValueAccessor::<V>::set_vals(elem_value, meters.len(), |idx| Ok(cb(&meters[idx])))
            .map(|_| true)
    }

    fn state_write_elem<V, F>(
        &mut self,
        unit: &mut (SndDice, FwNode),
        req: &mut FwReq,
        old: &ElemValue,
        new: &ElemValue,
        timeout_ms: u32,
        cb: F,
    ) -> Result<bool, Error>
    where
        F: Fn(&mut ChStripState, V) -> (),
        V: Default + Copy + Eq,
        ElemValue: ElemValueAccessor<V>,
    {
        let states = self.states_mut();
        ElemValueAccessor::<V>::get_vals(new, old, states.len(), |idx, val| {
            cb(&mut states[idx], val);
            Ok(())
        })?;
        U::write_segment(req, &mut unit.1, &mut self.states_segment_mut(), timeout_ms).map(|_| true)
    }

    fn read_notified_elem(
        &self,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            SRC_TYPE_NAME => self
                .state_read_elem(elem_value, |state| {
                    Self::SRC_TYPES
                        .iter()
                        .position(|t| state.src_type.eq(t))
                        .unwrap() as u32
                })
                .map(|_| true),
            DEESSER_BYPASS_NAME => self.state_read_elem(elem_value, |state| state.deesser.bypass),
            EQ_BYPASS_NAME => self.state_read_elem(elem_value, |state| state.eq_bypass),
            LIMITTER_BYPASS_NAME => self.state_read_elem(elem_value, |state| state.limitter_bypass),
            BYPASS_NAME => self.state_read_elem(elem_value, |state| state.bypass),
            COMP_INPUT_GAIN_NAME => {
                self.state_read_elem(elem_value, |state| state.comp.input_gain as i32)
            }
            COMP_MAKE_UP_GAIN_NAME => {
                self.state_read_elem(elem_value, |state| state.comp.make_up_gain as i32)
            }
            COMP_FULL_BAND_ENABLE_NAME => {
                self.state_read_elem(elem_value, |state| state.comp.full_band_enabled)
            }
            COMP_CTL_NAME => {
                let idx = elem_id.get_index() as usize;
                self.state_read_elem(elem_value, |entry| entry.comp.ctl[idx] as i32)
            }
            COMP_LEVEL_NAME => {
                let idx = elem_id.get_index() as usize;
                self.state_read_elem(elem_value, |entry| entry.comp.level[idx] as i32)
            }
            DEESSER_RATIO_NAME => {
                self.state_read_elem(elem_value, |state| state.deesser.ratio as i32)
            }
            EQ_ENABLE_NAME => {
                let idx = elem_id.get_index() as usize;
                self.state_read_elem(elem_value, |state| state.eq[idx].enabled)
            }
            EQ_BANDWIDTH_NAME => {
                let idx = elem_id.get_index() as usize;
                self.state_read_elem(elem_value, |state| state.eq[idx].bandwidth as i32)
            }
            EQ_GAIN_NAME => {
                let idx = elem_id.get_index() as usize;
                self.state_read_elem(elem_value, |state| state.eq[idx].gain as i32)
            }
            EQ_FREQ_NAME => {
                let idx = elem_id.get_index() as usize;
                self.state_read_elem(elem_value, |state| state.eq[idx].freq as i32)
            }
            LIMITTER_THRESHOLD_NAME => {
                self.state_read_elem(elem_value, |state| state.limitter.threshold as i32)
            }
            _ => Ok(false),
        }
    }

    fn read_measured_elem(
        &self,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            INPUT_METER_NAME => self.meter_read_elem(elem_value, |meter| meter.input),
            LIMIT_METER_NAME => self.meter_read_elem(elem_value, |meter| meter.limit),
            OUTPUT_METER_NAME => self.meter_read_elem(elem_value, |meter| meter.output),
            GAIN_METER_NAME => {
                let idx = match elem_id.get_index() {
                    2 => 2,
                    1 => 1,
                    _ => 0,
                };
                self.meter_read_elem(elem_value, |meter| meter.gains[idx])
            }
            _ => Ok(false),
        }
    }

    fn measure_states(
        &mut self,
        unit: &mut (SndDice, FwNode),
        req: &mut FwReq,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        if self.states().iter().find(|s| s.bypass).is_none() {
            U::read_segment(req, &mut unit.1, self.meters_segment_mut(), timeout_ms)
        } else {
            Ok(())
        }
    }

    fn parse_notification(
        &mut self,
        unit: &mut (SndDice, FwNode),
        req: &mut FwReq,
        msg: u32,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        if self.states_segment().has_segment_change(msg) {
            U::read_segment(req, &mut unit.1, self.states_segment_mut(), timeout_ms)
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

fn meter_add_int_elem(
    card_cntr: &mut CardCntr,
    measured_elem_id_list: &mut Vec<ElemId>,
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
        .map(|mut elem_id_list| measured_elem_id_list.append(&mut elem_id_list))
}
