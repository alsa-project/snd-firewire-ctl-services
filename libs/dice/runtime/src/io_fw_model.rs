// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::{Error, FileError};

use alsactl::{ElemId, ElemIfaceType, ElemValue, ElemValueExt, ElemValueExtManual};

use hinawa::{FwNode, FwReq, SndDice, SndUnitExt};

use alsa_ctl_tlv_codec::items::DbInterval;

use core::card_cntr::*;
use core::elem_value_accessor::*;

use dice_protocols::tcat::{*, global_section::*, tx_stream_format_section::*};
use dice_protocols::alesis::{meter::*, mixer::*, output::*};

use crate::common_ctl::*;

#[derive(Default)]
pub struct IoFwModel{
    req: FwReq,
    sections: GeneralSections,
    ctl: CommonCtl,
    state: AlesisIoFwState,
    meter_ctl: MeterCtl,
    mixer_ctl: MixerCtl,
    out_ctl: OutCtl,
}

const TIMEOUT_MS: u32 = 20;

impl CtlModel<SndDice> for IoFwModel {
    fn load(&mut self, unit: &mut SndDice, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let mut node = unit.get_node();

        self.sections = self.req.read_general_sections(&mut node, TIMEOUT_MS)?;
        let caps = self.req.read_clock_caps(&mut node, &self.sections, TIMEOUT_MS)?;
        let src_labels = self.req.read_clock_source_labels(&mut node, &self.sections, TIMEOUT_MS)?;
        self.ctl.load(card_cntr, &caps, &src_labels)?;

        self.state = AlesisIoFwState::new(&mut node, &mut self.req, &self.sections, TIMEOUT_MS)?;
        self.meter_ctl.load(card_cntr, unit, &mut self.req, &mut self.state, TIMEOUT_MS)?;
        self.mixer_ctl.load(card_cntr, unit, &mut self.req, &mut self.state, TIMEOUT_MS)?;
        self.out_ctl.load(card_cntr, &mut self.state)?;

        Ok(())
    }

    fn read(&mut self, unit: &mut SndDice, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        if self.ctl.read(unit, &mut self.req, &self.sections, elem_id, elem_value, TIMEOUT_MS)? {
            Ok(true)
        } else if self.mixer_ctl.read(&self.state, elem_id, elem_value)? {
            Ok(true)
        } else if self.out_ctl.read(unit, &mut self.req, &self.state, elem_id, elem_value, TIMEOUT_MS)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(&mut self, unit: &mut SndDice, elem_id: &ElemId, old: &ElemValue, new: &ElemValue)
        -> Result<bool, Error>
    {
        if self.ctl.write(unit, &mut self.req, &self.sections, elem_id, old, new, TIMEOUT_MS)? {
            Ok(true)
        } else if self.mixer_ctl.write(unit, &mut self.req, &mut self.state, elem_id, new, TIMEOUT_MS)? {
            Ok(true)
        } else if self.out_ctl.write(unit, &mut self.req, &self.state, elem_id, new, TIMEOUT_MS)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl NotifyModel<SndDice, u32> for IoFwModel {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.ctl.notified_elem_list);
    }

    fn parse_notification(&mut self, unit: &mut SndDice, msg: &u32) -> Result<(), Error> {
        self.ctl.parse_notification(unit, &mut self.req, &self.sections, *msg, TIMEOUT_MS)
    }

    fn read_notified_elem(&mut self, _: &SndDice, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        self.ctl.read_notified_elem(elem_id, elem_value)
    }
}

impl MeasureModel<SndDice> for IoFwModel {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.ctl.measured_elem_list);
        elem_id_list.extend_from_slice(&self.meter_ctl.0);
        elem_id_list.extend_from_slice(&self.mixer_ctl.0);
    }

    fn measure_states(&mut self, unit: &mut SndDice) -> Result<(), Error> {
        self.ctl.measure_states(unit, &mut self.req, &self.sections, TIMEOUT_MS)?;
        self.meter_ctl.measure_states(unit, &mut self.req, &mut self.state, TIMEOUT_MS)?;
        self.mixer_ctl.measure_states(unit, &mut self.req, &mut self.state, TIMEOUT_MS)?;
        Ok(())
    }

    fn measure_elem(&mut self, _: &SndDice, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        if self.ctl.measure_elem(elem_id, elem_value)? {
            Ok(true)
        } else if self.meter_ctl.read_measured_elem(&self.state, elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_ctl.read_measured_elem(&self.state, elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[derive(Debug)]
enum AlesisIoFwState{
    Io14(IoFwState<Io14Meter, Io14MixerState>),
    Io26(IoFwState<Io26Meter, Io26MixerState>),
}

impl Default for AlesisIoFwState {
    fn default() -> Self {
        Self::Io14(Default::default())
    }
}

impl AlesisIoFwState {
    fn new(
        node: &mut FwNode,
        req: &mut FwReq,
        sections: &GeneralSections,
        timeout_ms: u32
    ) -> Result<Self, Error> {
        let config = req.read_clock_config(node, sections, timeout_ms)?;
        match config.rate {
            ClockRate::R32000 |
            ClockRate::R44100 |
            ClockRate::R48000 |
            ClockRate::AnyLow => {
                let entries = req.read_tx_stream_format_entries(node, sections, timeout_ms)?;
                if entries.len() == 2 && entries[0].pcm == 10 && entries[1].pcm == 16 {
                    Ok(Self::Io26(Default::default()))
                } else if entries.len() == 2 && entries[0].pcm == 6 && entries[1].pcm == 8 {
                    Ok(Self::Io14(Default::default()))
                } else {
                    Err(Error::new(FileError::Nxio, "Unexpected combination of stream format."))
                }
            }
            ClockRate::R88200 |
            ClockRate::R96000 |
            ClockRate::AnyMid => {
                let entries = req.read_tx_stream_format_entries(node, sections, timeout_ms)?;
                if entries.len() == 2 && entries[0].pcm == 10 && entries[1].pcm == 4 {
                    Ok(Self::Io26(Default::default()))
                } else if entries.len() == 2 && entries[0].pcm == 6 && entries[1].pcm == 4 {
                    Ok(Self::Io14(Default::default()))
                } else {
                    Err(Error::new(FileError::Nxio, "Unexpected combination of stream format."))
                }
            }
            ClockRate::R176400 |
            ClockRate::R192000 |
            ClockRate::AnyHigh => {
                let nickname = req.read_nickname(node, sections, timeout_ms)?;
                match nickname.as_str() {
                    "iO 26" => Ok(Self::Io26(Default::default())),
                    "iO 14" => Ok(Self::Io14(Default::default())),
                    _ => Err(Error::new(FileError::Nxio, "Fail to detect type of iO model due to changed nickname")),
                }
            }
            _ => Err(Error::new(FileError::Nxio, "Unexpected value of rate of sampling clock.")),
        }
    }
}

impl AsRef<IoMeter> for AlesisIoFwState {
    fn as_ref(&self) -> &IoMeter {
        match self {
            Self::Io14(s) => s.meter.as_ref(),
            Self::Io26(s) => s.meter.as_ref(),
        }
    }
}

impl AsMut<IoMeter> for AlesisIoFwState {
    fn as_mut(&mut self) -> &mut IoMeter {
        match self {
            Self::Io14(s) => s.meter.as_mut(),
            Self::Io26(s) => s.meter.as_mut(),
        }
    }
}

impl AsRef<IoMixerState> for AlesisIoFwState {
    fn as_ref(&self) -> &IoMixerState {
        match self {
            Self::Io14(s) => s.mixer.as_ref(),
            Self::Io26(s) => s.mixer.as_ref(),
        }
    }
}

impl AsMut<IoMixerState> for AlesisIoFwState {
    fn as_mut(&mut self) -> &mut IoMixerState {
        match self {
            Self::Io14(s) => s.mixer.as_mut(),
            Self::Io26(s) => s.mixer.as_mut(),
        }
    }
}

#[derive(Default, Debug)]
struct IoFwState<M, S>
    where M: Default + AsRef<IoMeter> + AsMut<IoMeter>,
          S: Default + AsRef<IoMixerState> + AsMut<IoMixerState>,
{
    meter: M,
    mixer: S,
    out: OutCtl,
}

#[derive(Default, Debug)]
struct MeterCtl(Vec<ElemId>);

impl MeterCtl {
    const ANALOG_INPUT_METER_NAME: &'static str = "analog-input-meters";
    const DIGITAL_A_INPUT_METER_NAME: &'static str = "digital-a-input-meters";
    const DIGITAL_B_INPUT_METER_NAME: &'static str = "digital-b-input-meters";
    const MIXER_OUT_METER_NAME: &'static str = "mixer-output-meters";

    const LEVEL_MIN: i32 = 0;
    const LEVEL_MAX: i32 = 0x007fff00;
    const LEVEL_STEP: i32 = 0x100;
    const LEVEL_TLV: DbInterval = DbInterval{min: -9000, max: 0, linear: false, mute_avail: false};

    fn load(
        &mut self,
        card_cntr: &mut CardCntr,
        unit: &mut SndDice,
        req: &mut FwReq,
        state: &mut AlesisIoFwState,
        timeout_ms: u32
    ) -> Result<(), Error> {
        req.read_meter(&mut unit.get_node(), state, timeout_ms)?;

        let m = AsRef::<IoMeter>::as_ref(state);

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::ANALOG_INPUT_METER_NAME, 0);
        card_cntr.add_int_elems(&elem_id, 1, Self::LEVEL_MIN, Self::LEVEL_MAX, Self::LEVEL_STEP,
                                m.analog_inputs.len(), Some(&Vec::<u32>::from(Self::LEVEL_TLV)), false)
            .map(|mut elem_id_list| self.0.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::DIGITAL_A_INPUT_METER_NAME, 0);
        card_cntr.add_int_elems(&elem_id, 1, Self::LEVEL_MIN, Self::LEVEL_MAX, Self::LEVEL_STEP,
                                m.digital_a_inputs.len(), Some(&Vec::<u32>::from(Self::LEVEL_TLV)), false)
            .map(|mut elem_id_list| self.0.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::DIGITAL_B_INPUT_METER_NAME, 0);
        card_cntr.add_int_elems(&elem_id, 1, Self::LEVEL_MIN, Self::LEVEL_MAX, Self::LEVEL_STEP,
                                m.digital_b_inputs.len(), Some(&Vec::<u32>::from(Self::LEVEL_TLV)), false)
            .map(|mut elem_id_list| self.0.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::MIXER_OUT_METER_NAME, 0);
        card_cntr.add_int_elems(&elem_id, 1, Self::LEVEL_MIN, Self::LEVEL_MAX, Self::LEVEL_STEP,
                                m.mixer_outputs.len(), Some(&Vec::<u32>::from(Self::LEVEL_TLV)), false)
            .map(|mut elem_id_list| self.0.append(&mut elem_id_list))?;

        Ok(())
    }

    fn measure_states(
        &mut self,
        unit: &mut SndDice,
        req: &mut FwReq,
        state: &mut AlesisIoFwState,
        timeout_ms: u32
    ) -> Result<(), Error> {
        req.read_meter(&mut unit.get_node(), state, timeout_ms)
    }

    fn read_measured_elem(
        &self,
        state: &AlesisIoFwState,
        elem_id: &ElemId,
        elem_value: &mut ElemValue
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            Self::ANALOG_INPUT_METER_NAME => {
                let m = AsRef::<IoMeter>::as_ref(state);
                elem_value.set_int(&m.analog_inputs);
                Ok(true)
            }
            Self::DIGITAL_A_INPUT_METER_NAME => {
                let m = AsRef::<IoMeter>::as_ref(state);
                elem_value.set_int(&m.digital_a_inputs);
                Ok(true)
            }
            Self::DIGITAL_B_INPUT_METER_NAME => {
                let m = AsRef::<IoMeter>::as_ref(state);
                elem_value.set_int(&m.digital_b_inputs);
                Ok(true)
            }
            Self::MIXER_OUT_METER_NAME => {
                let m = AsRef::<IoMeter>::as_ref(state);
                elem_value.set_int(&m.mixer_outputs);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

#[derive(Default, Debug)]
struct MixerCtl(Vec<ElemId>);

impl MixerCtl {
    const INPUT_GAIN_NAME: &'static str = "monitor-input-gain";
    const INPUT_MUTE_NAME: &'static str = "monitor-input-mute";

    const STREAM_GAIN_NAME: &'static str = "mixer-stream-gain";

    const OUTPUT_VOL_NAME: &'static str = "monitor-output-volume";
    const OUTPUT_MUTE_NAME: &'static str = "monitor-output-mute";

    const MIX_BLEND_KNOB_NAME: &'static str = "mix-blend-knob";
    const MAIN_LEVEL_KNOB_NAME: &'static str = "main-level-knob";

    const LEVEL_MIN: i32 = 0;
    const LEVEL_MAX: i32 = 0x007fff00;
    const LEVEL_STEP: i32 = 0x100;
    const LEVEL_TLV: DbInterval = DbInterval{min: -9000, max: 0, linear: false, mute_avail: false};

    const KNOB_MIN: i32 = 0;
    const KNOB_MAX: i32 = 0x100;
    const KNOB_STEP: i32 = 1;

    fn load(
        &mut self,
        card_cntr: &mut CardCntr,
        unit: &mut SndDice,
        req: &mut FwReq,
        state: &mut AlesisIoFwState,
        timeout_ms: u32
    ) -> Result<(), Error> {
        let mut node = unit.get_node();
        req.read_mixer_src_gains(&mut node, state, timeout_ms)?;
        req.read_mixer_src_mutes(&mut node, state, timeout_ms)?;
        req.read_mixer_out_vols(&mut node, state, timeout_ms)?;
        req.read_mixer_out_mutes(&mut node, state, timeout_ms)?;

        let s = AsRef::<IoMixerState>::as_ref(state);

        let count = s.gains[0].analog_inputs.len() + s.gains[0].digital_a_inputs.len() +
                    s.gains[0].digital_b_inputs.len();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::INPUT_GAIN_NAME, 0);
        let _ = card_cntr.add_int_elems(&elem_id, s.gains.len(),
                                        Self::LEVEL_MIN, Self::LEVEL_MAX, Self::LEVEL_STEP, count,
                                        Some(&Into::<Vec<u32>>::into(Self::LEVEL_TLV)), true)?;

        let count = s.mutes[0].analog_inputs.len() + s.mutes[0].digital_a_inputs.len() +
                    s.mutes[0].digital_b_inputs.len();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::INPUT_MUTE_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, s.mutes.len(), count, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::STREAM_GAIN_NAME, 0);
        let _ = card_cntr.add_int_elems(&elem_id, s.gains.len(),
                                        Self::LEVEL_MIN, Self::LEVEL_MAX, Self::LEVEL_STEP,
                                        s.gains[0].stream_inputs.len(),
                                        Some(&Into::<Vec<u32>>::into(Self::LEVEL_TLV)), true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::OUTPUT_VOL_NAME, 0);
        card_cntr.add_int_elems(&elem_id, 1, Self::LEVEL_MIN, Self::LEVEL_MAX, Self::LEVEL_STEP,
                                s.out_vols.len(), Some(&Into::<Vec<u32>>::into(Self::LEVEL_TLV)), true)
            .map(|mut elem_id_list| self.0.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::OUTPUT_MUTE_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, s.out_mutes.len(), true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::MIX_BLEND_KNOB_NAME, 0);
        card_cntr.add_int_elems(&elem_id, 1, Self::KNOB_MIN, Self::KNOB_MAX, Self::KNOB_STEP, 1, None, false)
            .map(|mut elem_id_list| self.0.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::MAIN_LEVEL_KNOB_NAME, 0);
        card_cntr.add_int_elems(&elem_id, 1, Self::KNOB_MIN, Self::KNOB_MAX, Self::KNOB_STEP, 1, None, false)
            .map(|mut elem_id_list| self.0.append(&mut elem_id_list))?;

        Ok(())
    }

    fn read(
        &self,
        state: &AlesisIoFwState,
        elem_id: &ElemId,
        elem_value: &mut ElemValue
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            Self::INPUT_GAIN_NAME => {
                let mixer = elem_id.get_index() as usize;
                let gains = &AsRef::<IoMixerState>::as_ref(state).gains[mixer];
                let mut vals = Vec::new();
                vals.extend_from_slice(&gains.analog_inputs);
                vals.extend_from_slice(&gains.digital_a_inputs);
                vals.extend_from_slice(&gains.digital_b_inputs);
                elem_value.set_int(&vals);
                Ok(true)
            }
            Self::INPUT_MUTE_NAME => {
                let mixer = elem_id.get_index() as usize;
                let mutes = &AsRef::<IoMixerState>::as_ref(state).mutes[mixer];
                let mut vals = Vec::new();
                vals.extend_from_slice(&mutes.analog_inputs);
                vals.extend_from_slice(&mutes.digital_a_inputs);
                vals.extend_from_slice(&mutes.digital_b_inputs);
                elem_value.set_bool(&vals);
                Ok(true)
            }
            Self::STREAM_GAIN_NAME => {
                let mixer = elem_id.get_index() as usize;
                let gains = &AsRef::<IoMixerState>::as_ref(state).gains[mixer];
                let mut vals = Vec::new();
                vals.extend_from_slice(&gains.stream_inputs);
                elem_value.set_int(&vals);
                Ok(true)
            }
            Self::OUTPUT_MUTE_NAME => {
                let s = AsRef::<IoMixerState>::as_ref(state);
                elem_value.set_bool(&s.out_mutes);
                Ok(true)
            }
            _ => self.read_measured_elem(state, elem_id, elem_value),
        }
    }

    fn write(
        &mut self,
        unit: &mut SndDice,
        req: &mut FwReq,
        state: &mut AlesisIoFwState,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            Self::INPUT_GAIN_NAME => {
                let mixer = elem_id.get_index() as usize;
                let mut gains = AsRef::<IoMixerState>::as_ref(state).gains[mixer].clone();

                let analog_input_count = gains.analog_inputs.len();
                let digital_a_input_count = gains.digital_a_inputs.len();
                let digital_b_input_count = gains.digital_b_inputs.len();
                let mut vals = vec![0; analog_input_count + digital_a_input_count + digital_b_input_count];
                elem_value.get_int(&mut vals);

                let analog_inputs = &vals[..analog_input_count];
                let digital_a_inputs = &vals[analog_input_count..(analog_input_count + digital_a_input_count)];
                let digital_b_inputs = &vals[(analog_input_count + digital_a_input_count)..];

                gains.analog_inputs.copy_from_slice(&analog_inputs);
                gains.digital_a_inputs.copy_from_slice(&digital_a_inputs);
                gains.digital_b_inputs.copy_from_slice(&digital_b_inputs);

                req.write_mixer_src_gains(&mut unit.get_node(), state, mixer, &gains, timeout_ms)?;

                Ok(true)
            }
            Self::INPUT_MUTE_NAME => {
                let mixer = elem_id.get_index() as usize;
                let mut mutes = AsRef::<IoMixerState>::as_ref(state).mutes[mixer].clone();

                let analog_input_count = mutes.analog_inputs.len();
                let digital_a_input_count = mutes.digital_a_inputs.len();
                let digital_b_input_count = mutes.digital_b_inputs.len();
                let mut vals = vec![false; analog_input_count + digital_a_input_count + digital_b_input_count];
                elem_value.get_bool(&mut vals);

                let analog_inputs = &vals[..analog_input_count];
                let digital_a_inputs = &vals[analog_input_count..(analog_input_count + digital_a_input_count)];
                let digital_b_inputs = &vals[(analog_input_count + digital_a_input_count)..];

                mutes.analog_inputs.copy_from_slice(&analog_inputs);
                mutes.digital_a_inputs.copy_from_slice(&digital_a_inputs);
                mutes.digital_b_inputs.copy_from_slice(&digital_b_inputs);

                req.write_mixer_src_mutes(&mut unit.get_node(), state, mixer, &mutes, timeout_ms)?;

                Ok(true)
            }
            Self::STREAM_GAIN_NAME => {
                let mixer = elem_id.get_index() as usize;
                let mut gains = AsRef::<IoMixerState>::as_ref(state).gains[mixer].clone();

                elem_value.get_int(&mut gains.stream_inputs);

                req.write_mixer_src_gains(&mut unit.get_node(), state, mixer, &gains, timeout_ms)?;

                Ok(true)
            }
            Self::OUTPUT_VOL_NAME => {
                let s = AsRef::<IoMixerState>::as_ref(state);
                let mut vals = s.out_vols.clone();
                elem_value.get_int(&mut vals);
                req.write_mixer_out_vols(&mut unit.get_node(), state, &vals, timeout_ms)?;
                Ok(true)
            }
            Self::OUTPUT_MUTE_NAME => {
                let s = AsRef::<IoMixerState>::as_ref(state);
                let mut vals = s.out_mutes.clone();
                elem_value.get_bool(&mut vals);
                req.write_mixer_out_mutes(&mut unit.get_node(), state, &vals, timeout_ms)?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn measure_states(
        &mut self,
        unit: &mut SndDice,
        req: &mut FwReq,
        state: &mut AlesisIoFwState,
        timeout_ms: u32
    ) -> Result<(), Error> {
        let mut node = unit.get_node();

        let old = AsRef::<IoMixerState>::as_ref(state).knobs.mix_blend as i32;
        req.read_knob_state(&mut node, state, timeout_ms)?;

        let new = AsRef::<IoMixerState>::as_ref(state).knobs.mix_blend as i32;
        if new != old {
            // NOTE: The calculation is done within 32 bit storage without overflow.
            let val = Self::LEVEL_MAX * new / Self::KNOB_MAX;
            let mut new = AsRef::<IoMixerState>::as_ref(state).out_vols.clone();
            new[0] = val;
            new[1] = val;
            req.write_mixer_out_vols(&mut node, state, &new, timeout_ms)?;
        }

        Ok(())
    }

    fn read_measured_elem(
        &self,
        state: &AlesisIoFwState,
        elem_id: &ElemId,
        elem_value: &ElemValue
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            Self::OUTPUT_VOL_NAME => {
                let s = AsRef::<IoMixerState>::as_ref(state);
                elem_value.set_int(&s.out_vols);
                Ok(true)
            }
            Self::MIX_BLEND_KNOB_NAME => {
                let s = AsRef::<IoMixerState>::as_ref(state);
                elem_value.set_int(&[s.knobs.mix_blend as i32]);
                Ok(true)
            }
            Self::MAIN_LEVEL_KNOB_NAME => {
                let s = AsRef::<IoMixerState>::as_ref(state);
                elem_value.set_int(&[s.knobs.main_level as i32]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

fn nominal_signal_level_to_str(level: &NominalSignalLevel) -> &'static str {
    match level {
        NominalSignalLevel::Consumer => "-10dBV",
        NominalSignalLevel::Professional => "+4dBu",
    }
}

fn digital_b_67_src_to_str(src: &DigitalB67Src) -> &'static str {
    match src {
        DigitalB67Src::Spdif12 => "S/PDIF-input-1/2",
        DigitalB67Src::Adat67 => "ADAT-input-7/8",
    }
}

fn mixer_out_pair_to_str(pair: &MixerOutPair) -> &'static str {
    match pair {
        MixerOutPair::Mixer01 => "Mixer-output-1/2",
        MixerOutPair::Mixer23 => "Mixer-output-3/4",
        MixerOutPair::Mixer45 => "Mixer-output-5/6",
        MixerOutPair::Mixer67 => "Mixer-output-7/8",
    }
}

#[derive(Default, Debug)]
struct OutCtl;

impl OutCtl {
    const OUT_LEVEL_NAME: &'static str = "output-level";
    const DIGITAL_B_67_SRC_NAME: &'static str = "monitor-digital-b-7/8-source";
    const SPDIF_OUT_SRC_NAME: &'static str = "S/PDIF-1/2-output-source";
    const HP23_SRC_NAME: &'static str = "Headphone-3/4-output-source";

    const OUT_LEVELS: [NominalSignalLevel; 2] = [
        NominalSignalLevel::Consumer,
        NominalSignalLevel::Professional,
    ];

    const DIGITAL_B_67_SRCS: [DigitalB67Src; 2] = [
        DigitalB67Src::Spdif12,
        DigitalB67Src::Adat67,
    ];

    const MIXER_OUT_PAIRS: [MixerOutPair; 4] = [
        MixerOutPair::Mixer01,
        MixerOutPair::Mixer23,
        MixerOutPair::Mixer45,
        MixerOutPair::Mixer67,
    ];

    fn load(&mut self, card_cntr: &mut CardCntr, state: &AlesisIoFwState) -> Result<(), Error> {
        let m = AsRef::<IoMeter>::as_ref(state);
        let labels: Vec<&str> = Self::OUT_LEVELS.iter()
            .map(|l| nominal_signal_level_to_str(l))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::OUT_LEVEL_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, m.analog_inputs.len(), &labels, None, true)?;

        if m.digital_b_inputs.len() == 8 {
            let labels: Vec<&str> = Self::DIGITAL_B_67_SRCS.iter()
                .map(|s| digital_b_67_src_to_str(s))
                .collect();
            let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::DIGITAL_B_67_SRC_NAME, 0);
            let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;
        }

        let labels: Vec<&str> = Self::MIXER_OUT_PAIRS.iter()
            .map(|p| mixer_out_pair_to_str(p))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::SPDIF_OUT_SRC_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::HP23_SRC_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        Ok(())
    }

    fn read(
        &self,
        unit: &mut SndDice,
        req: &mut FwReq,
        state: &AlesisIoFwState,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
        timeout_ms: u32
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            Self::OUT_LEVEL_NAME => {
                let m = &AsRef::<IoMeter>::as_ref(state);
                let mut levels = vec![NominalSignalLevel::default();m.analog_inputs.len()];
                req.read_out_levels(&mut unit.get_node(), &mut levels, timeout_ms)?;
                let vals: Vec<u32> = levels.iter()
                    .map(|level| Self::OUT_LEVELS.iter().position(|l| l.eq(level)).unwrap() as u32)
                    .collect();
                elem_value.set_enum(&vals);
                Ok(true)
            }
            Self::DIGITAL_B_67_SRC_NAME => {
                let mut src = DigitalB67Src::default();
                req.read_mixer_digital_b_67_src(&mut unit.get_node(), &mut src, timeout_ms)?;
                let pos = Self::DIGITAL_B_67_SRCS.iter().position(|s| s.eq(&src)).unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            Self::SPDIF_OUT_SRC_NAME => {
                let mut pair = MixerOutPair::default();
                req.read_spdif_out_src(&mut unit.get_node(), &mut pair, timeout_ms)?;
                let pos = Self::MIXER_OUT_PAIRS.iter().position(|p| p.eq(&pair)).unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            Self::HP23_SRC_NAME => {
                let mut pair = MixerOutPair::default();
                req.read_hp23_out_src(&mut unit.get_node(), &mut pair, timeout_ms)?;
                let pos = Self::MIXER_OUT_PAIRS.iter().position(|p| p.eq(&pair)).unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write(
        &self,
        unit: &mut SndDice,
        req: &mut FwReq,
        state: &AlesisIoFwState,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            Self::OUT_LEVEL_NAME => {
                let m = &AsRef::<IoMeter>::as_ref(state);
                let mut vals = vec![0; m.analog_inputs.len()];
                elem_value.get_enum(&mut vals);
                let levels: Vec<NominalSignalLevel> = vals.iter()
                    .map(|v| NominalSignalLevel::from(*v))
                    .collect();
                req.write_out_levels(&mut unit.get_node(), &levels, timeout_ms)
                    .map(|_| true)
            }
            Self::DIGITAL_B_67_SRC_NAME => {
                ElemValueAccessor::<u32>::get_val(elem_value, |val| {
                    let src = Self::DIGITAL_B_67_SRCS.iter().nth(val as usize).ok_or_else(|| {
                        let msg = format!("Invalid value for index of source of digital B 7/8: {}",
                                          val);
                        Error::new(FileError::Inval, &msg)
                    })?;
                    req.write_mixer_digital_b_67_src(&mut unit.get_node(), src, timeout_ms)
                })
                .map(|_| true)
            }
            Self::SPDIF_OUT_SRC_NAME => {
                ElemValueAccessor::<u32>::get_val(elem_value, |val| {
                    let src = Self::MIXER_OUT_PAIRS.iter().nth(val as usize).ok_or_else(|| {
                        let msg = format!("Invalid value for index of pair of mixer output: {}",
                                          val);
                        Error::new(FileError::Inval, &msg)
                    })?;
                    req.write_spdif_out_src(&mut unit.get_node(), src, timeout_ms)
                })
                .map(|_| true)
            }
            Self::HP23_SRC_NAME => {
                ElemValueAccessor::<u32>::get_val(elem_value, |val| {
                    let src = Self::MIXER_OUT_PAIRS.iter().nth(val as usize).ok_or_else(|| {
                        let msg = format!("Invalid value for index of pair of mixer output: {}",
                                          val);
                        Error::new(FileError::Inval, &msg)
                    })?;
                    req.write_hp23_out_src(&mut unit.get_node(), src, timeout_ms)
                })
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }
}
