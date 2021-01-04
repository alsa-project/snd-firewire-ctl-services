// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use glib::Error;

use alsactl::{ElemId, ElemIfaceType, ElemValue};

use hinawa::{FwNode, SndDice, SndUnitExt};

use alsa_ctl_tlv_codec::items::DbInterval;

use dice_protocols::tcelectronic::{*, ch_strip::*};

use core::card_cntr::*;
use core::elem_value_accessor::*;

fn create_ch_strip_src_type_labels() -> Vec<String> {
    [
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
    ].iter()
        .map(|src_type| {
            match src_type {
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
            }.to_string()
        })
        .collect()
}

#[derive(Default, Debug)]
pub struct ChStripCtl {
    pub measured_elem_list: Vec<ElemId>,
    pub notified_elem_list: Vec<ElemId>,
}

impl<'a> ChStripCtl {
    const SRC_TYPE_NAME: &'a str = "ch-strip-source-type";
    const DEESSER_BYPASS_NAME: &'a str = "deesser-bypass";
    const EQ_BYPASS_NAME: &'a str = "equalizer-bypass";
    const LIMITTER_BYPASS_NAME: &'a str = "limitter-bypass";
    const BYPASS_NAME: &'a str = "ch-strip-bypass";

    const COMP_INPUT_GAIN_NAME: &'a str = "comp-input-gain";
    const COMP_MAKE_UP_GAIN: &'a str = "comp-make-up-gain";
    const COMP_FULL_BAND_ENABLE_NAME: &'a str = "comp-full-band-enable";
    const COMP_CTL_NAME: &'a str = "comp-control";
    const COMP_LEVEL_NAME: &'a str = "comp-level";

    const DEESSER_RATIO_NAME: &'a str = "deesser-ratio";

    const EQ_ENABLE_NAME: &'a str = "equalizer-enable";
    const EQ_BANDWIDTH_NAME: &'a str = "equalizer-bandwidth";
    const EQ_GAIN_NAME: &'a str = "equalizer-gain";
    const EQ_FREQ_NAME: &'a str = "equalizer-freq";

    const LIMITTER_THRESHOLD: &'a str = "limitter-threshold";

    const INPUT_METER_NAME: &'a str = "ch-strip-input-meter";
    const LIMIT_METER_NAME: &'a str = "ch-strip-limit-meter";
    const OUTPUT_METER_NAME: &'a str = "ch-strip-output-meter";
    const GAIN_METER_NAME: &'a str = "ch-strip-gain-meter";

    const COMP_GAIN_MIN: i32 = 0;
    const COMP_GAIN_MAX: i32 = 36;
    const COMP_GAIN_STEP: i32 = 1;
    const COMP_GAIN_TLV: DbInterval = DbInterval{min: -1800, max: 1800, linear: false, mute_avail: false};

    const COMP_CTL_MIN: i32 = 0;
    const COMP_CTL_MAX: i32 = 200;
    const COMP_CTL_STEP: i32 = 1;

    const COMP_LEVEL_MIN: i32 = 0;
    const COMP_LEVEL_MAX: i32 = 48;
    const COMP_LEVEL_STEP: i32 = 1;
    const COMP_LEVEL_TLV: DbInterval = DbInterval{min: -1800, max: 600, linear: false, mute_avail: false};

    const DEESSER_RATIO_MIN: i32 = 0;
    const DEESSER_RATIO_MAX: i32 = 10;
    const DEESSER_RATIO_STEP: i32 = 1;
    const DEESSER_RATIO_TLV: DbInterval = DbInterval{min: 0, max: 100, linear: false, mute_avail: false};

    const EQ_BANDWIDTH_MIN: i32 = 0;
    const EQ_BANDWIDTH_MAX: i32 = 39;
    const EQ_BANDWIDTH_STEP: i32 = 1;

    const EQ_GAIN_MIN: i32 = 0;
    const EQ_GAIN_MAX: i32 = 240;
    const EQ_GAIN_STEP: i32 = 1;
    const EQ_GAIN_TLV: DbInterval = DbInterval{min: -1200, max: 1200, linear: false, mute_avail: false};

    const EQ_FREQ_MIN: i32 = 0;
    const EQ_FREQ_MAX: i32 = 240;
    const EQ_FREQ_STEP: i32 = 1;

    const LIMITTER_THRESHOLD_MIN: i32 = 0;
    const LIMITTER_THRESHOLD_MAX: i32 = 72;
    const LIMITTER_THRESHOLD_STEP: i32 = 1;
    const LIMITTER_THRESHOLD_TLV: DbInterval = DbInterval{min: -1200, max: 0, linear: false, mute_avail: false};

    const LIMIT_METER_MIN: i32 = -12;
    const LIMIT_METER_MAX: i32 = 0;
    const LIMIT_METER_STEP: i32 = 1;
    const LIMIT_METER_TLV: DbInterval = DbInterval{min: -1200, max: 0, linear: false, mute_avail: false};

    const INOUT_METER_MIN: i32 = -72;
    const INOUT_METER_MAX: i32 = 0;
    const INOUT_METER_STEP: i32 = 1;
    const INOUT_METER_TLV: DbInterval = DbInterval{min: -7200, max: 0, linear: false, mute_avail: false};

    const GAIN_METER_MIN: i32 = -24;
    const GAIN_METER_MAX: i32 = 18;
    const GAIN_METER_STEP: i32 = 1;
    const GAIN_METER_TLV: DbInterval = DbInterval{min: -2400, max: 1800, linear: false, mute_avail: false};

    pub fn load<T, S, M>(&mut self, unit: &SndDice, proto: &T, state_segment: &mut TcKonnektSegment<S>,
                         meter_segment: &mut TcKonnektSegment<M>, timeout_ms: u32, card_cntr: &mut CardCntr)
        -> Result<(), Error>
        where T: TcKonnektSegmentProtocol<FwNode, S> + TcKonnektSegmentProtocol<FwNode, M>,
              S: TcKonnektSegmentData + AsRef<[ChStripState]>,
              TcKonnektSegment<S>: TcKonnektSegmentSpec,
              M: TcKonnektSegmentData + AsRef<[ChStripMeter]>,
              TcKonnektSegment<M>: TcKonnektSegmentSpec,
    {
        let node = unit.get_node();
        proto.read_segment(&node, state_segment, timeout_ms)?;
        proto.read_segment(&node, meter_segment, timeout_ms)?;

        let states = state_segment.data.as_ref();

        // Overall controls.
        let labels = create_ch_strip_src_type_labels();
        self.state_add_enum_elem(card_cntr, &states, Self::SRC_TYPE_NAME, 1, &labels, true)?;
        self.state_add_bool_elem(card_cntr, &states, Self::DEESSER_BYPASS_NAME, 1, true)?;
        self.state_add_bool_elem(card_cntr, &states, Self::EQ_BYPASS_NAME, 1, true)?;
        self.state_add_bool_elem(card_cntr, &states, Self::LIMITTER_BYPASS_NAME, 1, true)?;
        self.state_add_bool_elem(card_cntr, &states, Self::BYPASS_NAME, 1, true)?;

        // Controls for compressor part.
        self.state_add_int_elem(card_cntr, &states, Self::COMP_INPUT_GAIN_NAME, 1,
                                Self::COMP_GAIN_MIN, Self::COMP_GAIN_MAX, Self::COMP_GAIN_STEP,
                                Some(&Into::<Vec<u32>>::into(Self::COMP_GAIN_TLV)), true)?;
        self.state_add_int_elem(card_cntr, &states, Self::COMP_MAKE_UP_GAIN, 1,
                                Self::COMP_GAIN_MIN, Self::COMP_GAIN_MAX, Self::COMP_GAIN_STEP,
                                Some(&Into::<Vec<u32>>::into(Self::COMP_GAIN_TLV)), true)?;
        self.state_add_bool_elem(card_cntr, &states, Self::COMP_FULL_BAND_ENABLE_NAME, 1, true)?;
        self.state_add_int_elem(card_cntr, &states, Self::COMP_CTL_NAME, 3,
                                Self::COMP_CTL_MIN, Self::COMP_CTL_MAX, Self::COMP_CTL_STEP,
                                None, true)?;
        self.state_add_int_elem(card_cntr, &states, Self::COMP_LEVEL_NAME, 3,
                                Self::COMP_LEVEL_MIN, Self::COMP_LEVEL_MAX, Self::COMP_LEVEL_STEP,
                                Some(&Into::<Vec<u32>>::into(Self::COMP_LEVEL_TLV)), true)?;

        // Controls for deesser part.
        self.state_add_int_elem(card_cntr, &states, Self::DEESSER_RATIO_NAME, 1,
                                Self::DEESSER_RATIO_MIN, Self::DEESSER_RATIO_MAX, Self::DEESSER_RATIO_STEP,
                                Some(&Into::<Vec<u32>>::into(Self::DEESSER_RATIO_TLV)), true)?;

        // Controls for equalizer part.
        self.state_add_bool_elem(card_cntr, &states, Self::EQ_ENABLE_NAME, 4, true)?;
        self.state_add_int_elem(card_cntr, &states, Self::EQ_BANDWIDTH_NAME, 4,
                                Self::EQ_BANDWIDTH_MIN, Self::EQ_BANDWIDTH_MAX, Self::EQ_BANDWIDTH_STEP,
                                None, true)?;
        self.state_add_int_elem(card_cntr, &states, Self::EQ_GAIN_NAME, 4,
                                Self::EQ_GAIN_MIN, Self::EQ_GAIN_MAX, Self::EQ_GAIN_STEP,
                                Some(&Into::<Vec<u32>>::into(Self::EQ_GAIN_TLV)), true)?;
        self.state_add_int_elem(card_cntr, &states, Self::EQ_FREQ_NAME, 4,
                                Self::EQ_FREQ_MIN, Self::EQ_FREQ_MAX, Self::EQ_FREQ_STEP,
                                None, true)?;

        // Controls for limitter part.
        self.state_add_int_elem(card_cntr, &states, Self::LIMITTER_THRESHOLD, 1,
                        Self::LIMITTER_THRESHOLD_MIN, Self::LIMITTER_THRESHOLD_MAX, Self::LIMITTER_THRESHOLD_STEP,
                        Some(&Into::<Vec<u32>>::into(Self::LIMITTER_THRESHOLD_TLV)), true)?;

        // Controls for meter segment.
        let meters = meter_segment.data.as_ref();
        self.meter_add_int_elem(card_cntr, &meters, Self::INPUT_METER_NAME, 1,
                                Self::INOUT_METER_MIN, Self::INOUT_METER_MAX, Self::INOUT_METER_STEP,
                                Some(&Into::<Vec<u32>>::into(Self::INOUT_METER_TLV)), false)?;

        self.meter_add_int_elem(card_cntr, &meters, Self::LIMIT_METER_NAME, 1,
                                Self::LIMIT_METER_MIN, Self::LIMIT_METER_MAX, Self::LIMIT_METER_STEP,
                                Some(&Into::<Vec<u32>>::into(Self::LIMIT_METER_TLV)), false)?;

        self.meter_add_int_elem(card_cntr, &meters, Self::OUTPUT_METER_NAME, 1,
                                Self::INOUT_METER_MIN, Self::INOUT_METER_MAX, Self::INOUT_METER_STEP,
                                Some(&Into::<Vec<u32>>::into(Self::INOUT_METER_TLV)), false)?;

        self.meter_add_int_elem(card_cntr, &meters, Self::GAIN_METER_NAME, 1,
                                Self::GAIN_METER_MIN, Self::GAIN_METER_MAX, Self::GAIN_METER_STEP,
                                Some(&Into::<Vec<u32>>::into(Self::GAIN_METER_TLV)), false)?;

        Ok(())
    }

    fn state_add_int_elem(&mut self, card_cntr: &mut CardCntr, states: &[ChStripState], name: &str,
                          count: usize, min: i32, max: i32, step: i32, tlv: Option<&[u32]>, unlock: bool)
        -> Result<(), Error>
    {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
        card_cntr.add_int_elems(&elem_id, count, min, max, step, states.len(), tlv, unlock)
            .map(|mut elem_id_list| self.notified_elem_list.append(&mut elem_id_list))
    }

    fn state_add_enum_elem<T: AsRef<str>>(&mut self, card_cntr: &mut CardCntr, states: &[ChStripState],
                                          name: &str, count: usize, labels: &[T], locked: bool)
        -> Result<(), Error>
    {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
        card_cntr.add_enum_elems(&elem_id, count, states.len(), labels, None, locked)
            .map(|mut elem_id_list| self.notified_elem_list.append(&mut elem_id_list))
    }

    fn state_add_bool_elem(&mut self, card_cntr: &mut CardCntr, states: &[ChStripState], name: &str,
                           count: usize, unlock: bool)
        -> Result<(), Error>
    {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
        card_cntr.add_bool_elems(&elem_id, count, states.len(), unlock)
            .map(|mut elem_id_list| self.notified_elem_list.append(&mut elem_id_list))
    }

    fn meter_add_int_elem(&mut self, card_cntr: &mut CardCntr, meters: &[ChStripMeter], name: &str,
                          count: usize, min: i32, max: i32, step: i32, tlv: Option<&[u32]>, unlock: bool)
        -> Result<(), Error>
    {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
        card_cntr.add_int_elems(&elem_id, count, min, max, step, meters.len(), tlv, unlock)
            .map(|mut elem_id_list| self.measured_elem_list.append(&mut elem_id_list))
    }

    pub fn read<S, M>(&self, state_segment: &TcKonnektSegment<S>, meter_segment: &TcKonnektSegment<M>,
                      elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
        where S: TcKonnektSegmentData + AsRef<[ChStripState]>,
              TcKonnektSegment<S>: TcKonnektSegmentSpec,
              M: TcKonnektSegmentData + AsRef<[ChStripMeter]>,
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
        where S: TcKonnektSegmentData + AsRef<[ChStripState]>,
              TcKonnektSegment<S>: TcKonnektSegmentSpec,
              F: Fn(&ChStripState) -> T,
              T: Default + Copy + Eq,
              ElemValue: ElemValueAccessor<T>,
    {
        let states = segment.data.as_ref();
        ElemValueAccessor::<T>::set_vals(elem_value, states.len(), |idx| {
            Ok(cb(&states[idx]))
        })
        .map(|_| true)
    }

    fn meter_read_elem<M, T, F>(&self, segment: &TcKonnektSegment<M>, elem_value: &ElemValue, cb: F)
        -> Result<bool, Error>
        where M: TcKonnektSegmentData + AsRef<[ChStripMeter]>,
              TcKonnektSegment<M>: TcKonnektSegmentSpec,
              F: Fn(&ChStripMeter) -> T,
              T: Default + Copy + Eq,
              ElemValue: ElemValueAccessor<T>,
    {
        let meters = segment.data.as_ref();
        ElemValueAccessor::<T>::set_vals(elem_value, meters.len(), |idx| {
            Ok(cb(&meters[idx]))
        })
        .map(|_| true)
    }

    pub fn write<T, S>(&mut self, unit: &SndDice, proto: &T, segment: &mut TcKonnektSegment<S>,
                       elem_id: &ElemId, old: &ElemValue, new: &ElemValue, timeout_ms: u32)
        -> Result<bool, Error>
        where T: TcKonnektSegmentProtocol<FwNode, S>,
              S: TcKonnektSegmentData + AsMut<[ChStripState]>,
              TcKonnektSegment<S>: TcKonnektSegmentSpec,
    {
        match elem_id.get_name().as_str() {
            Self::SRC_TYPE_NAME => {
                self.state_write_elem(unit, proto, segment, old, new, timeout_ms,
                                      |state, val: u32| state.src_type = ChStripSrcType::from(val))
            }
            Self::DEESSER_BYPASS_NAME => {
                self.state_write_elem(unit, proto, segment, old, new, timeout_ms,
                                      |state, val: bool| state.deesser.bypass = val)
            }
            Self::EQ_BYPASS_NAME => {
                self.state_write_elem(unit, proto,  segment,old, new, timeout_ms,
                                      |state, val: bool| state.eq_bypass = val)
            }
            Self::LIMITTER_BYPASS_NAME => {
                self.state_write_elem(unit, proto,  segment,old, new, timeout_ms,
                                      |state, val: bool| state.limitter_bypass = val)
            }
            Self::BYPASS_NAME => {
                self.state_write_elem(unit, proto,  segment,old, new, timeout_ms,
                                      |state, val: bool| state.bypass = val)
            }
            Self::COMP_INPUT_GAIN_NAME => {
                self.state_write_elem(unit, proto,  segment,old, new, timeout_ms,
                                      |state, val: i32| state.comp.input_gain = val as u32)
            }
            Self::COMP_MAKE_UP_GAIN => {
                self.state_write_elem(unit, proto, segment, old, new, timeout_ms,
                                      |state, val: i32| state.comp.make_up_gain = val as u32)
            }
            Self::COMP_FULL_BAND_ENABLE_NAME => {
                self.state_write_elem(unit, proto, segment, old, new, timeout_ms,
                                      |state, val: bool| state.comp.full_band_enabled = val)
            }
            Self::COMP_CTL_NAME => {
                let idx = elem_id.get_index() as usize;
                self.state_write_elem(unit, proto, segment, old, new, timeout_ms,
                                      |state, val: i32| state.comp.ctl[idx] = val as u32)
            }
            Self::COMP_LEVEL_NAME => {
                let idx = elem_id.get_index() as usize;
                self.state_write_elem(unit, proto, segment, old, new, timeout_ms,
                                      |state, val: i32| state.comp.level[idx] = val as u32)
            }
            Self::DEESSER_RATIO_NAME => {
                self.state_write_elem(unit, proto, segment, old, new, timeout_ms,
                                      |state, val: i32| state.deesser.ratio = val as u32)
            }
            Self::EQ_ENABLE_NAME => {
                let idx = elem_id.get_index() as usize;
                self.state_write_elem(unit, proto, segment, old, new, timeout_ms,
                                      |state, val: bool| state.eq[idx].enabled = val)
            }
            Self::EQ_BANDWIDTH_NAME => {
                let idx = elem_id.get_index() as usize;
                self.state_write_elem(unit, proto, segment, old, new, timeout_ms,
                                      |state, val: i32| state.eq[idx].bandwidth = val as u32)
            }
            Self::EQ_GAIN_NAME => {
                let idx = elem_id.get_index() as usize;
                self.state_write_elem(unit, proto, segment, old, new, timeout_ms,
                                      |state, val: i32| state.eq[idx].gain = val as u32)
            }
            Self::EQ_FREQ_NAME => {
                let idx = elem_id.get_index() as usize;
                self.state_write_elem(unit, proto, segment, old, new, timeout_ms,
                                      |state, val: i32| state.eq[idx].freq = val as u32)
            }
            Self::LIMITTER_THRESHOLD => {
                self.state_write_elem(unit, proto, segment, old, new, timeout_ms,
                                      |state, val: i32| state.limitter.threshold = val as u32)
            }
            _ => Ok(false),
        }
    }

    fn state_write_elem<T, S, U, F>(&mut self, unit: &SndDice, proto: &T, segment: &mut TcKonnektSegment<S>,
                                    old: &ElemValue, new: &ElemValue, timeout_ms: u32, cb: F)
        -> Result<bool, Error>
        where T: TcKonnektSegmentProtocol<FwNode, S>,
              S: TcKonnektSegmentData + AsMut<[ChStripState]>,
              TcKonnektSegment<S>: TcKonnektSegmentSpec,
              F: Fn(&mut ChStripState, U) -> (),
              U: Default + Copy + Eq,
              ElemValue: ElemValueAccessor<U>,
    {
        let states = segment.data.as_mut();
        ElemValueAccessor::<U>::get_vals(new, old, states.len(), |idx, val| {
            cb(&mut states[idx], val);
            Ok(())
        })
        .and_then(|_| proto.write_segment(&unit.get_node(), segment, timeout_ms))
        .map(|_| true)
    }

    pub fn read_notified_elem<S>(&self, segment: &TcKonnektSegment<S>, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
        where S: TcKonnektSegmentData + AsRef<[ChStripState]>,
              TcKonnektSegment<S>: TcKonnektSegmentSpec,
    {
        match elem_id.get_name().as_str() {
            Self::SRC_TYPE_NAME => {
                self.state_read_elem(segment, elem_value, |state| u32::from(state.src_type))
                    .map(|_| true)
            }
            Self::DEESSER_BYPASS_NAME => {
                self.state_read_elem(segment, elem_value, |state| state.deesser.bypass)
            }
            Self::EQ_BYPASS_NAME => {
                self.state_read_elem(segment, elem_value, |state| state.eq_bypass)
            }
            Self::LIMITTER_BYPASS_NAME => {
                self.state_read_elem(segment, elem_value, |state| state.limitter_bypass)
            }
            Self::BYPASS_NAME => {
                self.state_read_elem(segment, elem_value, |state| state.bypass)
            }
            Self::COMP_INPUT_GAIN_NAME => {
                self.state_read_elem(segment, elem_value, |state| state.comp.input_gain as i32)
            }
            Self::COMP_MAKE_UP_GAIN => {
                self.state_read_elem(segment, elem_value, |state| state.comp.make_up_gain as i32)
            }
            Self::COMP_FULL_BAND_ENABLE_NAME => {
                self.state_read_elem(segment, elem_value, |state| state.comp.full_band_enabled)
            }
            Self::COMP_CTL_NAME => {
                let idx = elem_id.get_index() as usize;
                self.state_read_elem(segment, elem_value, |entry| entry.comp.ctl[idx] as i32)
            }
            Self::COMP_LEVEL_NAME => {
                let idx = elem_id.get_index() as usize;
                self.state_read_elem(segment, elem_value, |entry| entry.comp.level[idx] as i32)
            }
            Self::DEESSER_RATIO_NAME => {
                self.state_read_elem(segment, elem_value, |state| state.deesser.ratio as i32)
            }
            Self::EQ_ENABLE_NAME => {
                let idx = elem_id.get_index() as usize;
                self.state_read_elem(segment, elem_value, |state| state.eq[idx].enabled)
            }
            Self::EQ_BANDWIDTH_NAME => {
                let idx = elem_id.get_index() as usize;
                self.state_read_elem(segment, elem_value, |state| state.eq[idx].bandwidth as i32)
            }
            Self::EQ_GAIN_NAME => {
                let idx = elem_id.get_index() as usize;
                self.state_read_elem(segment, elem_value, |state| state.eq[idx].gain as i32)
            }
            Self::EQ_FREQ_NAME => {
                let idx = elem_id.get_index() as usize;
                self.state_read_elem(segment, elem_value, |state| state.eq[idx].freq as i32)
            }
            Self::LIMITTER_THRESHOLD => {
                self.state_read_elem(segment, elem_value, |state| state.limitter.threshold as i32)
            }
            _ => Ok(false),
        }
    }

    pub fn measure_states<T, S, M>(&mut self, unit: &SndDice, proto: &T, state_segment: &TcKonnektSegment<S>,
                                   meter_segment: &mut TcKonnektSegment<M>, timeout_ms: u32)
        -> Result<(), Error>
        where T: TcKonnektSegmentProtocol<FwNode, S> + TcKonnektSegmentProtocol<FwNode, M>,
              S: TcKonnektSegmentData + AsRef<[ChStripState]>,
              TcKonnektSegment<S>: TcKonnektSegmentSpec,
              M: TcKonnektSegmentData,
              TcKonnektSegment<M>: TcKonnektSegmentSpec,
    {
        if state_segment.data.as_ref().iter().find(|s| s.bypass).is_none() {
            proto.read_segment(&unit.get_node(), meter_segment, timeout_ms)
        } else {
            Ok(())
        }
    }

    pub fn read_measured_elem<M>(&self, segment: &TcKonnektSegment<M>, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
        where M: TcKonnektSegmentData + AsRef<[ChStripMeter]>,
              TcKonnektSegment<M>: TcKonnektSegmentSpec,
    {
        match elem_id.get_name().as_str() {
            Self::INPUT_METER_NAME => {
                self.meter_read_elem(segment, elem_value, |meter| meter.input)
            }
            Self::LIMIT_METER_NAME => {
                self.meter_read_elem(segment, elem_value, |meter| meter.limit)
            }
            Self::OUTPUT_METER_NAME => {
                self.meter_read_elem(segment, elem_value, |meter| meter.output)
            }
            Self::GAIN_METER_NAME => {
                let idx = match elem_id.get_index() {
                    2 => 2,
                    1 => 1,
                    _ => 0,
                };
                self.meter_read_elem(segment, elem_value, |meter| meter.gains[idx])
            }
            _ => Ok(false),
        }
    }
}
