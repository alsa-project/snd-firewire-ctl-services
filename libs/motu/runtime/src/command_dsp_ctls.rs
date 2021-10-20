// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

use glib::{Error, FileError};

use hinawa::FwReq;
use hinawa::{SndMotu, SndUnitExt};

use alsactl::{ElemId, ElemIfaceType, ElemValue, ElemValueExt, ElemValueExtManual};

//use alsa_ctl_tlv_codec::items::DbInterval;

use core::card_cntr::*;
use core::elem_value_accessor::*;

use motu_protocols::command_dsp::*;

use crate::*;

const REVERB_ENABLE: &str = "reverb-enable";
const REVERB_SPLIT_POINT_NAME: &str = "reverb-split-point";
const REVERB_PRE_DELAY_NAME: &str = "reverb-pre-delay";
const REVERB_SHELF_FILTER_FREQ_NAME: &str = "reverb-shelf-filter-frequency";
const REVERB_SHELF_FILTER_ATTR_NAME: &str = "reverb-shelf-filter-attenuation";
const REVERB_DECAY_TIME_NAME: &str = "reverb-decay-time";
const REVERB_FREQ_TIME_NAME: &str = "reverb-frequency-time";
const REVERB_FREQ_CROSSOVER_NAME: &str = "reverb-frequency-crossover";
const REVERB_WIDTH_NAME: &str = "reverb-width";
const REVERB_REFLECTION_MODE_NAME: &str = "reverb-reflection-mode";
const REVERB_REFLECTION_SIZE_NAME: &str = "reverb-reflection-size";
const REVERB_REFLECTION_LEVEL_NAME: &str = "reverb-reflection-level";

fn reverb_split_point_to_str(point: &SplitPoint) -> &'static str {
    match point {
        SplitPoint::Output => "output",
        SplitPoint::Mixer => "mixer",
        SplitPoint::Reserved(_) => "reserved",
    }
}

fn reverb_room_shape_to_str(shape: &RoomShape) -> &'static str {
    match shape {
        RoomShape::A => "A",
        RoomShape::B => "B",
        RoomShape::C => "C",
        RoomShape::D => "D",
        RoomShape::E => "E",
        RoomShape::Reserved(_) => "reserved",
    }
}

pub trait CommandDspReverbCtlOperation<T: CommandDspReverbOperation> {
    fn state(&self) -> &CommandDspReverbState;
    fn state_mut(&mut self) -> &mut CommandDspReverbState;

    const SPLIT_POINTS: [SplitPoint; 2] = [
        SplitPoint::Output,
        SplitPoint::Mixer,
    ];

    const ROOM_SHAPES: [RoomShape; 5] = [
        RoomShape::A,
        RoomShape::B,
        RoomShape::C,
        RoomShape::D,
        RoomShape::E,
    ];

    const F32_CONVERT_SCALE: f32 = 1000000.0;

    fn load(
        &mut self,
        card_cntr: &mut CardCntr,
    ) -> Result<Vec<ElemId>, Error> {
        let mut notified_elem_id_list = Vec::new();

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, REVERB_ENABLE, 0);
        card_cntr.add_bool_elems(&elem_id, 1, 1, true)
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        let labels: Vec<&str> = Self::SPLIT_POINTS
            .iter()
            .map(|p| reverb_split_point_to_str(p))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, REVERB_SPLIT_POINT_NAME, 0);
        card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, REVERB_DECAY_TIME_NAME, 0);
        card_cntr.add_int_elems(
            &elem_id,
            1,
            T::DECAY_TIME_MIN,
            T::DECAY_TIME_MAX,
            T::DECAY_TIME_STEP,
            1,
            None,
            true,
        )
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, REVERB_PRE_DELAY_NAME, 0);
        card_cntr.add_int_elems(
            &elem_id,
            1,
            T::PRE_DELAY_MIN,
            T::PRE_DELAY_MAX,
            T::PRE_DELAY_STEP,
            1,
            None,
            true,
        )
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, REVERB_SHELF_FILTER_FREQ_NAME, 0);
        card_cntr.add_int_elems(
            &elem_id,
            1,
            T::SHELF_FILTER_FREQ_MIN,
            T::SHELF_FILTER_FREQ_MAX,
            T::SHELF_FILTER_FREQ_STEP,
            1,
            None,
            true,
        )
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, REVERB_SHELF_FILTER_ATTR_NAME, 0);
        card_cntr.add_int_elems(
            &elem_id,
            1,
            T::SHELF_FILTER_ATTR_MIN,
            T::SHELF_FILTER_ATTR_MAX,
            T::SHELF_FILTER_ATTR_STEP,
            1,
            None,
            true,
        )
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, REVERB_FREQ_TIME_NAME, 0);
        card_cntr.add_int_elems(
            &elem_id,
            1,
            T::FREQ_TIME_MIN,
            T::FREQ_TIME_MAX,
            T::FREQ_TIME_STEP,
            3,
            None,
            true,
        )
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, REVERB_FREQ_CROSSOVER_NAME, 0);
        card_cntr.add_int_elems(
            &elem_id,
            1,
            T::FREQ_CROSSOVER_MIN,
            T::FREQ_CROSSOVER_MAX,
            T::FREQ_CROSSOVER_STEP,
            2,
            None,
            true,
        )
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, REVERB_WIDTH_NAME, 0);
        card_cntr.add_int_elems(
            &elem_id,
            1,
            (T::WIDTH_MIN * Self::F32_CONVERT_SCALE) as i32,
            (T::WIDTH_MAX * Self::F32_CONVERT_SCALE) as i32,
            1,
            1,
            None,
            true,
        )
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        let labels: Vec<&str> = Self::ROOM_SHAPES
            .iter()
            .map(|p| reverb_room_shape_to_str(p))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, REVERB_REFLECTION_MODE_NAME, 0);
        card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, REVERB_REFLECTION_SIZE_NAME, 0);
        card_cntr.add_int_elems(
            &elem_id,
            1,
            T::REFLECTION_SIZE_MIN,
            T::REFLECTION_SIZE_MAX,
            T::REFLECTION_SIZE_STEP,
            1,
            None,
            true,
        )
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, REVERB_REFLECTION_LEVEL_NAME, 0);
        card_cntr.add_int_elems(
            &elem_id,
            1,
            (T::REFLECTION_LEVEL_MIN * Self::F32_CONVERT_SCALE) as i32,
            (T::REFLECTION_LEVEL_MAX * Self::F32_CONVERT_SCALE) as i32,
            1,
            1,
            None,
            true,
        )
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        Ok(notified_elem_id_list)
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            REVERB_ENABLE => {
                elem_value.set_bool(&[self.state().enable]);
                Ok(true)
            }
            REVERB_SPLIT_POINT_NAME => {
                let pos = Self::SPLIT_POINTS
                    .iter()
                    .position(|p| self.state().split_point.eq(p))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            REVERB_PRE_DELAY_NAME => {
                elem_value.set_int(&[self.state().pre_delay]);
                Ok(true)
            }
            REVERB_SHELF_FILTER_FREQ_NAME => {
                elem_value.set_int(&[self.state().shelf_filter_freq]);
                Ok(true)
            }
            REVERB_SHELF_FILTER_ATTR_NAME => {
                elem_value.set_int(&[self.state().shelf_filter_attenuation]);
                Ok(true)
            }
            REVERB_DECAY_TIME_NAME => {
                elem_value.set_int(&[self.state().decay_time]);
                Ok(true)
            }
            REVERB_FREQ_TIME_NAME => {
                elem_value.set_int(&self.state().freq_time);
                Ok(true)
            }
            REVERB_FREQ_CROSSOVER_NAME => {
                elem_value.set_int(&self.state().freq_crossover);
                Ok(true)
            }
            REVERB_WIDTH_NAME => {
                let val = (self.state().width * Self::F32_CONVERT_SCALE) as i32;
                elem_value.set_int(&[val]);
                Ok(true)
            }
            REVERB_REFLECTION_MODE_NAME => {
                let pos = Self::ROOM_SHAPES
                    .iter()
                    .position(|m| self.state().reflection_mode.eq(m))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            REVERB_REFLECTION_SIZE_NAME => {
                elem_value.set_int(&[self.state().reflection_size]);
                Ok(true)
            }
            REVERB_REFLECTION_LEVEL_NAME => {
                let val = (self.state().reflection_level * Self::F32_CONVERT_SCALE) as i32;
                elem_value.set_int(&[val]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write(
        &mut self,
        sequence_number: &mut u8,
        unit: &mut SndMotu,
        req: &mut FwReq,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            REVERB_ENABLE => {
                let mut vals = [false];
                elem_value.get_bool(&mut vals);
                self.write_state(sequence_number, unit, req, timeout_ms, |state| {
                    state.enable = vals[0];
                    Ok(())
                })
            }
            REVERB_SPLIT_POINT_NAME => {
                let mut vals = [0];
                elem_value.get_enum(&mut vals);
                let &split_point = Self::SPLIT_POINTS
                    .iter()
                    .nth(vals[0] as usize)
                    .ok_or_else(|| {
                        let msg = format!("Invalid index for split points: {}", vals[0]);
                        Error::new(FileError::Inval, &msg)
                    })?;
                self.write_state(sequence_number, unit, req, timeout_ms, |state| {
                    state.split_point = split_point;
                    Ok(())
                })
            }
            REVERB_PRE_DELAY_NAME => {
                let mut vals = [0];
                elem_value.get_int(&mut vals);
                self.write_state(sequence_number, unit, req, timeout_ms, |state| {
                    state.pre_delay = vals[0];
                    Ok(())
                })
            }
            REVERB_SHELF_FILTER_FREQ_NAME => {
                let mut vals = [0];
                elem_value.get_int(&mut vals);
                self.write_state(sequence_number, unit, req, timeout_ms, |state| {
                    state.shelf_filter_freq = vals[0];
                    Ok(())
                })
            }
            REVERB_SHELF_FILTER_ATTR_NAME => {
                let mut vals = [0];
                elem_value.get_int(&mut vals);
                self.write_state(sequence_number, unit, req, timeout_ms, |state| {
                    state.shelf_filter_attenuation = vals[0];
                    Ok(())
                })
            }
            REVERB_DECAY_TIME_NAME => {
                let mut vals = [0];
                elem_value.get_int(&mut vals);
                self.write_state(sequence_number, unit, req, timeout_ms, |state| {
                    state.decay_time = vals[0];
                    Ok(())
                })
            }
            REVERB_FREQ_TIME_NAME => {
                let mut vals = vec![0; T::FREQ_TIME_COUNT];
                elem_value.get_int(&mut vals);
                self.write_state(sequence_number, unit, req, timeout_ms, |state| {
                    state.freq_time.copy_from_slice(&vals);
                    Ok(())
                })
            }
            REVERB_FREQ_CROSSOVER_NAME => {
                let mut vals = vec![0; T::FREQ_CROSSOVER_COUNT];
                elem_value.get_int(&mut vals);
                self.write_state(sequence_number, unit, req, timeout_ms, |state| {
                    state.freq_crossover.copy_from_slice(&vals);
                    Ok(())
                })
            }
            REVERB_WIDTH_NAME => {
                let mut vals = [0];
                elem_value.get_int(&mut vals);
                let val = (vals[0] as f32) / Self::F32_CONVERT_SCALE;
                self.write_state(sequence_number, unit, req, timeout_ms, |state| {
                    state.width = val;
                    Ok(())
                })
            }
            REVERB_REFLECTION_MODE_NAME => {
                let mut vals = [0];
                elem_value.get_enum(&mut vals);
                let &mode = Self::ROOM_SHAPES
                    .iter()
                    .nth(vals[0] as usize)
                    .ok_or_else(|| {
                        let msg = format!("Invalid index for reflection modes: {}", vals[0]);
                        Error::new(FileError::Inval, &msg)
                    })?;
                self.write_state(sequence_number, unit, req, timeout_ms, |state| {
                    state.reflection_mode = mode;
                    Ok(())
                })
            }
            REVERB_REFLECTION_SIZE_NAME => {
                let mut vals = [0];
                elem_value.get_int(&mut vals);
                self.write_state(sequence_number, unit, req, timeout_ms, |state| {
                    state.reflection_size = vals[0];
                    Ok(())
                })
            }
            REVERB_REFLECTION_LEVEL_NAME => {
                let mut vals = [0];
                elem_value.get_int(&mut vals);
                let val = (vals[0] as f32) / Self::F32_CONVERT_SCALE;
                self.write_state(sequence_number, unit, req, timeout_ms, |state| {
                    state.reflection_level = val;
                    Ok(())
                })
            }
            _ => Ok(false),
        }
    }

    fn write_state<F>(
        &mut self,
        sequence_number: &mut u8,
        unit: &mut SndMotu,
        req: &mut FwReq,
        timeout_ms: u32,
        func: F,
    ) -> Result<bool, Error>
        where F: Fn(&mut CommandDspReverbState) -> Result<(), Error>,
    {
        let mut state = self.state().clone();
        func(&mut state)?;
        T::write_reverb_state(
            req,
            &mut unit.get_node(),
            sequence_number,
            state,
            self.state_mut(),
            timeout_ms
        )
            .map(|_| true)
    }

    fn parse_commands(&mut self, cmds: &[DspCmd]) {
        T::parse_reverb_commands(self.state_mut(), cmds);
    }
}

const MAIN_VOLUME_NAME: &str = "main-volume";

pub trait CommandDspMonitorCtlOperation<T: CommandDspMonitorOperation> {
    fn state(&self) -> &CommandDspMonitorState;
    fn state_mut(&mut self) -> &mut CommandDspMonitorState;

    fn load(
        &mut self,
        card_cntr: &mut CardCntr,
    ) -> Result<Vec<ElemId>, Error> {
        let mut notified_elem_id_list = Vec::new();

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MAIN_VOLUME_NAME, 0);
        card_cntr.add_int_elems(
            &elem_id,
            1,
            T::VOLUME_MIN,
            T::VOLUME_MAX,
            T::VOLUME_STEP,
            1,
            None,
            true,
        )
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        Ok(notified_elem_id_list)
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            MAIN_VOLUME_NAME => {
                elem_value.set_int(&[self.state().main_volume]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write(
        &mut self,
        sequence_number: &mut u8,
        unit: &mut SndMotu,
        req: &mut FwReq,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            MAIN_VOLUME_NAME => {
                let mut vals = [0];
                elem_value.get_int(&mut vals);
                let mut state =self.state().clone();
                state.main_volume = vals[0];
                T::write_monitor_state(
                    req,
                    &mut unit.get_node(),
                    sequence_number,
                    state,
                    self.state_mut(),
                    timeout_ms
                )
                    .map(|_| true)
            }
            _ => Ok(false),
        }
    }

    fn parse_commands(&mut self, cmds: &[DspCmd]) {
        T::parse_monitor_commands(self.state_mut(), cmds);
    }
}

fn mixer_source_stereo_pair_mode_to_str(mode: &SourceStereoPairMode) -> &'static str {
    match mode {
        SourceStereoPairMode::Width => "width",
        SourceStereoPairMode::LrBalance => "left-right-balance",
        SourceStereoPairMode::Reserved(_) => "reserved",
    }
}

const MIXER_OUTPUT_DESTINATION_NAME: &str = "mixer-output-destination";
const MIXER_OUTPUT_MUTE_NAME: &str = "mixer-output-mute";
const MIXER_OUTPUT_VOLUME_NAME: &str = "mixer-output-volume";
const MIXER_REVERB_SEND_NAME: &str = "mixer-reverb-send";
const MIXER_REVERB_RETURN_NAME: &str = "mixer-reverb-return";

const MIXER_SOURCE_MUTE_NAME: &str = "mixer-soruce-mute";
const MIXER_SOURCE_SOLO_NAME: &str = "mixer-source-solo";
const MIXER_SOURCE_GAIN_NAME: &str = "mixer-source-gain";
const MIXER_SOURCE_PAN_NAME: &str = "mixer-source-pan";
const MIXER_SOURCE_STEREO_PAIR_MODE_NAME: &str = "mixer-source-stereo-mode";
const MIXER_SOURCE_STEREO_BALANCE_NAME: &str = "mixer-source-stereo-balance";
const MIXER_SOURCE_STEREO_WIDTH_NAME: &str = "mixer-source-stereo-width";

pub trait CommandDspMixerCtlOperation<T: CommandDspMixerOperation> {
    fn state(&self) -> &CommandDspMixerState;
    fn state_mut(&mut self) -> &mut CommandDspMixerState;

    const SOURCE_STEREO_PAIR_MODES: [SourceStereoPairMode; 2] = [
        SourceStereoPairMode::Width,
        SourceStereoPairMode::LrBalance,
    ];

    fn load(
        &mut self,
        card_cntr: &mut CardCntr,
    ) -> Result<Vec<ElemId>, Error> {
        let state = T::create_mixer_state();
        *self.state_mut() = state;

        let mut notified_elem_id_list = Vec::new();

        let labels: Vec<&str> = T::OUTPUT_PORTS
            .iter()
            .map(|p| target_port_to_str(p))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIXER_OUTPUT_DESTINATION_NAME, 0);
        card_cntr.add_enum_elems(&elem_id, 1, T::MIXER_COUNT, &labels, None, true)
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIXER_OUTPUT_MUTE_NAME, 0);
        card_cntr.add_bool_elems(&elem_id, 1, T::MIXER_COUNT, true)
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIXER_OUTPUT_VOLUME_NAME, 0);
        card_cntr.add_int_elems(
            &elem_id,
            1,
            T::OUTPUT_VOLUME_MIN,
            T::OUTPUT_VOLUME_MAX,
            T::OUTPUT_VOLUME_STEP,
            T::MIXER_COUNT,
            None,
            true,
        )
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIXER_REVERB_SEND_NAME, 0);
        card_cntr.add_int_elems(
            &elem_id,
            1,
            T::OUTPUT_VOLUME_MIN,
            T::OUTPUT_VOLUME_MAX,
            T::OUTPUT_VOLUME_STEP,
            T::MIXER_COUNT,
            None,
            true,
        )
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIXER_REVERB_RETURN_NAME, 0);
        card_cntr.add_int_elems(
            &elem_id,
            1,
            T::OUTPUT_VOLUME_MIN,
            T::OUTPUT_VOLUME_MAX,
            T::OUTPUT_VOLUME_STEP,
            T::MIXER_COUNT,
            None,
            true,
        )
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIXER_SOURCE_MUTE_NAME, 0);
        card_cntr.add_bool_elems(&elem_id, T::MIXER_COUNT, T::SOURCE_PORTS.len(), true)
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIXER_SOURCE_SOLO_NAME, 0);
        card_cntr.add_bool_elems(&elem_id, T::MIXER_COUNT, T::SOURCE_PORTS.len(), true)
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIXER_SOURCE_GAIN_NAME, 0);
        card_cntr.add_int_elems(
            &elem_id,
            T::MIXER_COUNT,
            T::SOURCE_GAIN_MIN,
            T::SOURCE_GAIN_MAX,
            T::SOURCE_GAIN_STEP,
            T::SOURCE_PORTS.len(),
            None,
            true,
        )
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIXER_SOURCE_PAN_NAME, 0);
        card_cntr.add_int_elems(
            &elem_id,
            T::MIXER_COUNT,
            T::SOURCE_PAN_MIN,
            T::SOURCE_PAN_MAX,
            T::SOURCE_PAN_STEP,
            T::SOURCE_PORTS.len(),
            None,
            true,
        )
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        let labels: Vec<&str> = Self::SOURCE_STEREO_PAIR_MODES
            .iter()
            .map(|p| mixer_source_stereo_pair_mode_to_str(p))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIXER_SOURCE_STEREO_PAIR_MODE_NAME, 0);
        card_cntr.add_enum_elems(
            &elem_id,
            T::MIXER_COUNT,
            T::SOURCE_PORTS.len(),
            &labels,
            None,
            true
        )
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIXER_SOURCE_STEREO_BALANCE_NAME, 0);
        card_cntr.add_int_elems(
            &elem_id,
            T::MIXER_COUNT,
            T::SOURCE_PAN_MIN,
            T::SOURCE_PAN_MAX,
            T::SOURCE_PAN_STEP,
            T::SOURCE_PORTS.len(),
            None,
            true,
        )
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIXER_SOURCE_STEREO_WIDTH_NAME, 0);
        card_cntr.add_int_elems(
            &elem_id,
            T::MIXER_COUNT,
            T::SOURCE_PAN_MIN,
            T::SOURCE_PAN_MAX,
            T::SOURCE_PAN_STEP,
            T::SOURCE_PORTS.len(),
            None,
            true,
        )
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        Ok(notified_elem_id_list)
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            MIXER_OUTPUT_DESTINATION_NAME => {
                ElemValueAccessor::<u32>::set_vals(elem_value, T::MIXER_COUNT, |idx| {
                    let pos = T::OUTPUT_PORTS
                        .iter()
                        .position(|p| self.state().output_assign[idx].eq(p))
                        .unwrap();
                    Ok(pos as u32)
                })
                    .map(|_| true)
            }
            MIXER_OUTPUT_MUTE_NAME => {
                elem_value.set_bool(&self.state().output_mute);
                Ok(true)
            }
            MIXER_OUTPUT_VOLUME_NAME => {
                elem_value.set_int(&self.state().output_volume);
                Ok(true)
            }
            MIXER_REVERB_SEND_NAME => {
                elem_value.set_int(&self.state().reverb_send);
                Ok(true)
            }
            MIXER_REVERB_RETURN_NAME => {
                elem_value.set_int(&self.state().reverb_return);
                Ok(true)
            }
            MIXER_SOURCE_MUTE_NAME => {
                let mixer = elem_id.get_index() as usize;
                elem_value.set_bool(&self.state().source[mixer].mute);
                Ok(true)
            }
            MIXER_SOURCE_SOLO_NAME => {
                let mixer = elem_id.get_index() as usize;
                elem_value.set_bool(&self.state().source[mixer].solo);
                Ok(true)
            }
            MIXER_SOURCE_PAN_NAME => {
                let mixer = elem_id.get_index() as usize;
                elem_value.set_int(&self.state().source[mixer].pan);
                Ok(true)
            }
            MIXER_SOURCE_GAIN_NAME => {
                let mixer = elem_id.get_index() as usize;
                elem_value.set_int(&self.state().source[mixer].gain);
                Ok(true)
            }
            MIXER_SOURCE_STEREO_PAIR_MODE_NAME => {
                let mixer = elem_id.get_index() as usize;
                ElemValueAccessor::<u32>::set_vals(elem_value, T::SOURCE_PORTS.len(), |idx| {
                    let pos = Self::SOURCE_STEREO_PAIR_MODES
                        .iter()
                        .position(|m| self.state().source[mixer].stereo_mode[idx].eq(m))
                        .unwrap();
                    Ok(pos as u32)
                })
                    .map(|_| true)
            }
            MIXER_SOURCE_STEREO_BALANCE_NAME => {
                let mixer = elem_id.get_index() as usize;
                elem_value.set_int(&self.state().source[mixer].stereo_balance);
                Ok(true)
            }
            MIXER_SOURCE_STEREO_WIDTH_NAME => {
                let mixer = elem_id.get_index() as usize;
                elem_value.set_int(&self.state().source[mixer].stereo_width);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write(
        &mut self,
        sequence_number: &mut u8,
        unit: &mut SndMotu,
        req: &mut FwReq,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            MIXER_OUTPUT_DESTINATION_NAME => {
                let mut vals = vec![0; T::MIXER_COUNT];
                elem_value.get_enum(&mut vals);
                let mut dsts = Vec::new();
                vals
                    .iter()
                    .try_for_each(|&val| {
                        T::OUTPUT_PORTS
                            .iter()
                            .nth(val as usize)
                            .ok_or_else(|| {
                                let msg = format!("Invalid index of output destinations: {}", val);
                                Error::new(FileError::Inval, &msg)
                            })
                            .map(|&p| dsts.push(p))
                    })?;
                self.write_state(sequence_number, unit, req, timeout_ms, |state| {
                    state.output_assign.copy_from_slice(&dsts);
                    Ok(())
                })
            }
            MIXER_OUTPUT_MUTE_NAME => {
                let mut vals = vec![false; T::MIXER_COUNT];
                elem_value.get_bool(&mut vals);
                self.write_state(sequence_number, unit, req, timeout_ms, |state| {
                    state.output_mute.copy_from_slice(&vals);
                    Ok(())
                })
            }
            MIXER_OUTPUT_VOLUME_NAME => {
                let mut vals = vec![0; T::MIXER_COUNT];
                elem_value.get_int(&mut vals);
                self.write_state(sequence_number, unit, req, timeout_ms, |state| {
                    state.output_volume.copy_from_slice(&vals);
                    Ok(())
                })
            }
            MIXER_REVERB_SEND_NAME => {
                let mut vals = vec![0; T::MIXER_COUNT];
                elem_value.get_int(&mut vals);
                self.write_state(sequence_number, unit, req, timeout_ms, |state| {
                    state.reverb_send.copy_from_slice(&vals);
                    Ok(())
                })
            }
            MIXER_REVERB_RETURN_NAME => {
                let mut vals = vec![0; T::MIXER_COUNT];
                elem_value.get_int(&mut vals);
                self.write_state(sequence_number, unit, req, timeout_ms, |state| {
                    state.reverb_return.copy_from_slice(&vals);
                    Ok(())
                })
            }
            MIXER_SOURCE_MUTE_NAME => {
                let mut vals = vec![false; T::SOURCE_PORTS.len()];
                elem_value.get_bool(&mut vals);
                let mixer = elem_id.get_index() as usize;
                self.write_state(sequence_number, unit, req, timeout_ms, |state| {
                    state.source[mixer].mute.copy_from_slice(&vals);
                    Ok(())
                })
            }
            MIXER_SOURCE_SOLO_NAME => {
                let mut vals = vec![false; T::SOURCE_PORTS.len()];
                elem_value.get_bool(&mut vals);
                let mixer = elem_id.get_index() as usize;
                self.write_state(sequence_number, unit, req, timeout_ms, |state| {
                    state.source[mixer].solo.copy_from_slice(&vals);
                    Ok(())
                })
            }
            MIXER_SOURCE_PAN_NAME => {
                let mut vals = vec![0; T::SOURCE_PORTS.len()];
                elem_value.get_int(&mut vals);
                let mixer = elem_id.get_index() as usize;
                self.write_state(sequence_number, unit, req, timeout_ms, |state| {
                    state.source[mixer].pan.copy_from_slice(&vals);
                    Ok(())
                })
            }
            MIXER_SOURCE_GAIN_NAME => {
                let mut vals = vec![0; T::SOURCE_PORTS.len()];
                elem_value.get_int(&mut vals);
                let mixer = elem_id.get_index() as usize;
                self.write_state(sequence_number, unit, req, timeout_ms, |state| {
                    state.source[mixer].gain.copy_from_slice(&vals);
                    Ok(())
                })
            }
            MIXER_SOURCE_STEREO_PAIR_MODE_NAME => {
                let mut vals = vec![0; T::SOURCE_PORTS.len()];
                elem_value.get_enum(&mut vals);
                let mut stereo_modes = Vec::new();
                vals
                    .iter()
                    .try_for_each(|&val| {
                        Self::SOURCE_STEREO_PAIR_MODES
                            .iter()
                            .nth(val as usize)
                            .ok_or_else(|| {
                                let msg = format!("Invalid index of stereo pair modes: {}", val);
                                Error::new(FileError::Inval, &msg)
                            })
                            .map(|&mode| stereo_modes.push(mode))
                    })?;
                let mixer = elem_id.get_index() as usize;
                self.write_state(sequence_number, unit, req, timeout_ms, |state| {
                    state.source[mixer].stereo_mode.copy_from_slice(&stereo_modes);
                    Ok(())
                })
            }
            MIXER_SOURCE_STEREO_BALANCE_NAME => {
                let mut vals = vec![0; T::SOURCE_PORTS.len()];
                elem_value.get_int(&mut vals);
                let mixer = elem_id.get_index() as usize;
                self.write_state(sequence_number, unit, req, timeout_ms, |state| {
                    state.source[mixer].stereo_balance.copy_from_slice(&vals);
                    Ok(())
                })
            }
            MIXER_SOURCE_STEREO_WIDTH_NAME=> {
                let mut vals = vec![0; T::SOURCE_PORTS.len()];
                elem_value.get_int(&mut vals);
                let mixer = elem_id.get_index() as usize;
                self.write_state(sequence_number, unit, req, timeout_ms, |state| {
                    state.source[mixer].stereo_width.copy_from_slice(&vals);
                    Ok(())
                })
            }
            _ => Ok(false),
        }
    }

    fn parse_commands(&mut self, cmds: &[DspCmd]) {
        T::parse_mixer_commands(self.state_mut(), cmds);
    }

    fn write_state<F>(
        &mut self,
        sequence_number: &mut u8,
        unit: &mut SndMotu,
        req: &mut FwReq,
        timeout_ms: u32,
        func: F,
    ) -> Result<bool, Error>
        where F: Fn(&mut CommandDspMixerState) -> Result<(), Error>,
    {
        let mut state = self.state().clone();
        func(&mut state)?;
        T::write_mixer_state(
            req,
            &mut unit.get_node(),
            sequence_number,
            state,
            self.state_mut(),
            timeout_ms
        )
            .map(|_| true)
    }
}

fn roll_off_level_to_str(level: &RollOffLevel) -> &'static str {
    match level {
        RollOffLevel::L6 => "6dB-per-octave",
        RollOffLevel::L12 => "12dB-per-octave",
        RollOffLevel::L18 => "18dB-per-octave",
        RollOffLevel::L24 => "24dB-per-octave",
        RollOffLevel::L30 => "30dB-per-octave",
        RollOffLevel::L36 => "36dB-per-octave",
        RollOffLevel::Reserved(_) => "reserved",
    }
}

fn filter_type_5_to_str(filter_type: &FilterType5) -> &'static str {
    match filter_type {
        FilterType5::T1 => "I",
        FilterType5::T2 => "II",
        FilterType5::T3 => "III",
        FilterType5::T4 => "IV",
        FilterType5::Shelf => "shelf",
        FilterType5::Reserved(_) => "reserved",
    }
}

fn filter_type_4_to_str(filter_type: &FilterType4) -> &'static str {
    match filter_type {
        FilterType4::T1 => "I",
        FilterType4::T2 => "II",
        FilterType4::T3 => "III",
        FilterType4::T4 => "IV",
        FilterType4::Reserved(_) => "reserved",
    }
}

// TODO: better trait parameters to distinguish input and output.
pub trait CommandDspEqualizerCtlOperation<T: CommandDspOperation, U: Default> {
    const CH_COUNT: usize;

    fn state(&self) -> &CommandDspEqualizerState;

    const ENABLE_NAME: &'static str;

    const HPF_ENABLE_NAME: &'static str;
    const HPF_SLOPE_NAME: &'static str;
    const HPF_FREQ_NAME: &'static str;

    const LPF_ENABLE_NAME: &'static str;
    const LPF_SLOPE_NAME: &'static str;
    const LPF_FREQ_NAME: &'static str;

    const LF_ENABLE_NAME: &'static str;
    const LF_TYPE_NAME: &'static str;
    const LF_FREQ_NAME: &'static str;
    const LF_GAIN_NAME: &'static str;
    const LF_WIDTH_NAME: &'static str;

    const LMF_ENABLE_NAME: &'static str;
    const LMF_TYPE_NAME: &'static str;
    const LMF_FREQ_NAME: &'static str;
    const LMF_GAIN_NAME: &'static str;
    const LMF_WIDTH_NAME: &'static str;

    const MF_ENABLE_NAME: &'static str;
    const MF_TYPE_NAME: &'static str;
    const MF_FREQ_NAME: &'static str;
    const MF_GAIN_NAME: &'static str;
    const MF_WIDTH_NAME: &'static str;

    const HMF_ENABLE_NAME: &'static str;
    const HMF_TYPE_NAME: &'static str;
    const HMF_FREQ_NAME: &'static str;
    const HMF_GAIN_NAME: &'static str;
    const HMF_WIDTH_NAME: &'static str;

    const HF_ENABLE_NAME: &'static str;
    const HF_TYPE_NAME: &'static str;
    const HF_FREQ_NAME: &'static str;
    const HF_GAIN_NAME: &'static str;
    const HF_WIDTH_NAME: &'static str;

    const ROLL_OFF_LEVELS: [RollOffLevel; 6] = [
        RollOffLevel::L6,
        RollOffLevel::L12,
        RollOffLevel::L18,
        RollOffLevel::L24,
        RollOffLevel::L30,
        RollOffLevel::L36,
    ];

    const FILTER_TYPE_5: [FilterType5; 5] = [
        FilterType5::T1,
        FilterType5::T2,
        FilterType5::T3,
        FilterType5::T4,
        FilterType5::Shelf,
    ];

    const FILTER_TYPE_4: [FilterType4; 4] = [
        FilterType4::T1,
        FilterType4::T2,
        FilterType4::T3,
        FilterType4::T4,
    ];

    const LEVEL_DETECT_MODES: [LevelDetectMode; 2] = [
        LevelDetectMode::Peak,
        LevelDetectMode::Rms,
    ];

    const LEVELER_MODES: [LevelerMode; 2] = [
        LevelerMode::Compress,
        LevelerMode::Limit,
    ];

    fn load_equalizer(
        &mut self,
        card_cntr: &mut CardCntr,
    ) -> Result<Vec<ElemId>, Error> {
        let mut notified_elem_id_list = Vec::new();

        // Enable.
        [
            Self::ENABLE_NAME,
            Self::HPF_ENABLE_NAME,
            Self::LPF_ENABLE_NAME,
            Self::LF_ENABLE_NAME,
            Self::LMF_ENABLE_NAME,
            Self::MF_ENABLE_NAME,
            Self::HMF_ENABLE_NAME,
            Self::HF_ENABLE_NAME,
        ]
            .iter()
            .try_for_each(|name| {
                let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
                card_cntr.add_bool_elems(&elem_id, 1, Self::CH_COUNT, true)
                    .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))
            })?;

        // Roll over level.
        let labels: Vec<&str> = Self::ROLL_OFF_LEVELS
            .iter()
            .map(|level| roll_off_level_to_str(level))
            .collect();
        [
            Self::HPF_SLOPE_NAME,
            Self::LPF_SLOPE_NAME,
        ]
            .iter()
            .try_for_each(|name| {
                let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
                card_cntr.add_enum_elems(&elem_id, 1, Self::CH_COUNT, &labels, None, true)
                    .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))
            })?;

        // Filter type 5.
        let labels: Vec<&str> = Self::FILTER_TYPE_5
            .iter()
            .map(|filter_type| filter_type_5_to_str(filter_type))
            .collect();
        [
            Self::LF_TYPE_NAME,
            Self::HF_TYPE_NAME,
        ]
            .iter()
            .try_for_each(|name| {
                let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
                card_cntr.add_enum_elems(&elem_id, 1, Self::CH_COUNT, &labels, None, true)
                    .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))
            })?;

        // Filter type 4.
        let labels: Vec<&str> = Self::FILTER_TYPE_4
            .iter()
            .map(|filter_type| filter_type_4_to_str(filter_type))
            .collect();
        [
            Self::LMF_TYPE_NAME,
            Self::MF_TYPE_NAME,
            Self::HMF_TYPE_NAME,
        ]
            .iter()
            .try_for_each(|name| {
                let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
                card_cntr.add_enum_elems(&elem_id, 1, Self::CH_COUNT, &labels, None, true)
                    .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))
            })?;

        // Frequency.
        [
            Self::HPF_FREQ_NAME,
            Self::LPF_FREQ_NAME,
            Self::LF_FREQ_NAME,
            Self::LMF_FREQ_NAME,
            Self::MF_FREQ_NAME,
            Self::HMF_FREQ_NAME,
            Self::HF_FREQ_NAME,
        ]
            .iter()
            .try_for_each(|name| {
                let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
                card_cntr
                    .add_int_elems(
                        &elem_id,
                        1,
                        EqualizerParameter::FREQ_MIN,
                        EqualizerParameter::FREQ_MAX,
                        EqualizerParameter::FREQ_STEP,
                        Self::CH_COUNT,
                        None,
                        true,
                    )
                        .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))
            })?;

        // Gain.
        [
            Self::LF_GAIN_NAME,
            Self::LMF_GAIN_NAME,
            Self::MF_GAIN_NAME,
            Self::HMF_GAIN_NAME,
            Self::HF_GAIN_NAME,
        ]
            .iter()
            .try_for_each(|name| {
                let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
                card_cntr
                    .add_int_elems(
                        &elem_id,
                        1,
                        EqualizerParameter::GAIN_MIN,
                        EqualizerParameter::GAIN_MAX,
                        EqualizerParameter::GAIN_STEP,
                        Self::CH_COUNT,
                        None,
                        true,
                    )
                        .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))
            })?;

        // Width.
        [
            Self::LF_WIDTH_NAME,
            Self::LMF_WIDTH_NAME,
            Self::MF_WIDTH_NAME,
            Self::HMF_WIDTH_NAME,
            Self::HF_WIDTH_NAME,
        ]
            .iter()
            .try_for_each(|name| {
                let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
                card_cntr
                    .add_int_elems(
                        &elem_id,
                        1,
                        EqualizerParameter::WIDTH_MIN,
                        EqualizerParameter::WIDTH_MAX,
                        EqualizerParameter::WIDTH_STEP,
                        Self::CH_COUNT,
                        None,
                        true,
                    )
                        .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))
            })?;

        Ok(notified_elem_id_list)
    }

    fn read_bool_values(
        elem_value: &mut ElemValue,
        vals: &[bool],
    ) -> Result<bool, Error> {
        assert_eq!(vals.len(), Self::CH_COUNT);

        elem_value.set_bool(vals);
        Ok(true)
    }

    fn read_int_values(
        elem_value: &mut ElemValue,
        vals: &[i32],
    ) -> Result<bool, Error> {
        assert_eq!(vals.len(), Self::CH_COUNT);

        elem_value.set_int(vals);
        Ok(true)
    }

    fn read_roll_off_level(
        elem_value: &mut ElemValue,
        levels: &[RollOffLevel]
    ) -> Result<bool, Error> {
        assert_eq!(levels.len(), Self::CH_COUNT);

        ElemValueAccessor::<u32>::set_vals(elem_value, Self::CH_COUNT, |idx| {
            let pos = Self::ROLL_OFF_LEVELS
                .iter()
                .position(|l| levels[idx].eq(l))
                .unwrap();
            Ok(pos as u32)
        })
            .map(|_| true)
    }

    fn read_filter_type_5(
        elem_value: &mut ElemValue,
        filter_types: &[FilterType5]
    ) -> Result<bool, Error> {
        assert_eq!(filter_types.len(), Self::CH_COUNT);

        ElemValueAccessor::<u32>::set_vals(elem_value, Self::CH_COUNT, |idx| {
            let pos = Self::FILTER_TYPE_5
                .iter()
                .position(|f| filter_types[idx].eq(f))
                .unwrap();
            Ok(pos as u32)
        })
            .map(|_| true)
    }

    fn read_filter_type_4(
        elem_value: &mut ElemValue,
        filter_types: &[FilterType4]
    ) -> Result<bool, Error> {
        assert_eq!(filter_types.len(), Self::CH_COUNT);

        ElemValueAccessor::<u32>::set_vals(elem_value, Self::CH_COUNT, |idx| {
            let pos = Self::FILTER_TYPE_4
                .iter()
                .position(|f| filter_types[idx].eq(f))
                .unwrap();
            Ok(pos as u32)
        })
            .map(|_| true)
    }

    fn read_equalizer(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        let name = elem_id.get_name();

        if name == Self::ENABLE_NAME {
            Self::read_bool_values(elem_value, &self.state().enable)
        } else if name == Self::HPF_ENABLE_NAME {
            Self::read_bool_values(elem_value, &self.state().hpf_enable)
        } else if name == Self::HPF_SLOPE_NAME {
            Self::read_roll_off_level(elem_value, &self.state().hpf_slope)
        } else if name == Self::HPF_FREQ_NAME {
            Self::read_int_values(elem_value, &self.state().hpf_freq)
        } else if name == Self::LPF_ENABLE_NAME {
            Self::read_bool_values(elem_value, &self.state().lpf_enable)
        } else if name == Self::LPF_SLOPE_NAME {
            Self::read_roll_off_level(elem_value, &self.state().lpf_slope)
        } else if name == Self::LPF_FREQ_NAME {
            Self::read_int_values(elem_value, &self.state().lpf_freq)
        } else if name == Self::LF_ENABLE_NAME {
            Self::read_bool_values(elem_value, &self.state().lf_enable)
        } else if name == Self::LF_TYPE_NAME {
            Self::read_filter_type_5(elem_value, &self.state().lf_type)
        } else if name == Self::LF_FREQ_NAME {
            Self::read_int_values(elem_value, &self.state().lf_freq)
        } else if name == Self::LF_GAIN_NAME {
            Self::read_int_values(elem_value, &self.state().lf_gain)
        } else if name == Self::LF_WIDTH_NAME {
            Self::read_int_values(elem_value, &self.state().lf_width)
        } else if name == Self::LMF_ENABLE_NAME {
            Self::read_bool_values(elem_value, &self.state().lmf_enable)
        } else if name == Self::LMF_TYPE_NAME {
            Self::read_filter_type_4(elem_value, &self.state().lmf_type)
        } else if name == Self::LMF_FREQ_NAME {
            Self::read_int_values(elem_value, &self.state().lmf_freq)
        } else if name == Self::LMF_GAIN_NAME {
            Self::read_int_values(elem_value, &self.state().lmf_gain)
        } else if name == Self::LMF_WIDTH_NAME {
            Self::read_int_values(elem_value, &self.state().lmf_width)
        } else if name == Self::MF_ENABLE_NAME {
            Self::read_bool_values(elem_value, &self.state().mf_enable)
        } else if name == Self::MF_TYPE_NAME {
            Self::read_filter_type_4(elem_value, &self.state().mf_type)
        } else if name == Self::MF_FREQ_NAME {
            Self::read_int_values(elem_value, &self.state().mf_freq)
        } else if name == Self::MF_GAIN_NAME {
            Self::read_int_values(elem_value, &self.state().mf_gain)
        } else if name == Self::MF_WIDTH_NAME {
            Self::read_int_values(elem_value, &self.state().mf_width)
        } else if name == Self::HMF_ENABLE_NAME {
            Self::read_bool_values(elem_value, &self.state().hmf_enable)
        } else if name == Self::HMF_TYPE_NAME {
            Self::read_filter_type_4(elem_value, &self.state().hmf_type)
        } else if name == Self::HMF_FREQ_NAME {
            Self::read_int_values(elem_value, &self.state().hmf_freq)
        } else if name == Self::HMF_GAIN_NAME {
            Self::read_int_values(elem_value, &self.state().hmf_gain)
        } else if name == Self::HMF_WIDTH_NAME {
            Self::read_int_values(elem_value, &self.state().hmf_width)
        } else if name == Self::HF_ENABLE_NAME {
            Self::read_bool_values(elem_value, &self.state().hf_enable)
        } else if name == Self::HF_TYPE_NAME {
            Self::read_filter_type_5(elem_value, &self.state().hf_type)
        } else if name == Self::HF_FREQ_NAME {
            Self::read_int_values(elem_value, &self.state().hf_freq)
        } else if name == Self::HF_GAIN_NAME {
            Self::read_int_values(elem_value, &self.state().hf_gain)
        } else if name == Self::HF_WIDTH_NAME {
            Self::read_int_values(elem_value, &self.state().hf_width)
        } else {
            Ok(false)
        }
    }

    fn write_bool_values<F>(
        &mut self,
        sequence_number: &mut u8,
        unit: &mut SndMotu,
        req: &mut FwReq,
        elem_value: &ElemValue,
        timeout_ms: u32,
        func: F,
    ) -> Result<bool, Error>
        where F: Fn(&mut CommandDspEqualizerState, &[bool]),
    {
        let mut vals = vec![false; Self::CH_COUNT];
        elem_value.get_bool(&mut vals);
        self.write_equalizer_state(sequence_number, unit, req, timeout_ms, |state| {
            func(state, &vals);
            Ok(())
        })
    }

    fn write_int_values<F>(
        &mut self,
        sequence_number: &mut u8,
        unit: &mut SndMotu,
        req: &mut FwReq,
        elem_value: &ElemValue,
        timeout_ms: u32,
        func: F,
    ) -> Result<bool, Error>
        where F: Fn(&mut CommandDspEqualizerState, &[i32]),
    {
        let mut vals = vec![0; Self::CH_COUNT];
        elem_value.get_int(&mut vals);
        self.write_equalizer_state(sequence_number, unit, req, timeout_ms, |state| {
            func(state, &vals);
            Ok(())
        })
    }

    fn write_roll_off_level<F>(
        &mut self,
        sequence_number: &mut u8,
        unit: &mut SndMotu,
        req: &mut FwReq,
        elem_value: &ElemValue,
        timeout_ms: u32,
        func: F,
    ) -> Result<bool, Error>
        where F: Fn(&mut CommandDspEqualizerState, &[RollOffLevel]),
    {
        let mut vals = vec![0; Self::CH_COUNT];
        elem_value.get_enum(&mut vals);
        let mut levels = Vec::new();
        vals
            .iter()
            .try_for_each(|&val| {
                Self::ROLL_OFF_LEVELS
                    .iter()
                    .nth(val as usize)
                    .ok_or_else(|| {
                        let msg = format!("Invalid index of roll off levels: {}", val);
                        Error::new(FileError::Inval, &msg)
                    })
                    .map(|&l| levels.push(l))
            })?;
        self.write_equalizer_state(sequence_number, unit, req, timeout_ms, |state| {
            func(state, &levels);
            Ok(())
        })
    }

    fn write_filter_type_5<F>(
        &mut self,
        sequence_number: &mut u8,
        unit: &mut SndMotu,
        req: &mut FwReq,
        elem_value: &ElemValue,
        timeout_ms: u32,
        func: F,
    ) -> Result<bool, Error>
        where F: Fn(&mut CommandDspEqualizerState, &[FilterType5]),
    {
        let mut vals = vec![0; Self::CH_COUNT];
        elem_value.get_enum(&mut vals);
        let mut filter_types = Vec::new();
        vals
            .iter()
            .try_for_each(|&val| {
                Self::FILTER_TYPE_5
                    .iter()
                    .nth(val as usize)
                    .ok_or_else(|| {
                        let msg = format!("Invalid index of filter type 5: {}", val);
                        Error::new(FileError::Inval, &msg)
                    })
                    .map(|&filter_type| filter_types.push(filter_type))
            })?;
        self.write_equalizer_state(sequence_number, unit, req, timeout_ms, |state| {
            func(state, &filter_types);
            Ok(())
        })
    }

    fn write_filter_type_4<F>(
        &mut self,
        sequence_number: &mut u8,
        unit: &mut SndMotu,
        req: &mut FwReq,
        elem_value: &ElemValue,
        timeout_ms: u32,
        func: F,
    ) -> Result<bool, Error>
        where F: Fn(&mut CommandDspEqualizerState, &[FilterType4]),
    {
        let mut vals = vec![0; Self::CH_COUNT];
        elem_value.get_enum(&mut vals);
        let mut filter_types = Vec::new();
        vals
            .iter()
            .try_for_each(|&val| {
                Self::FILTER_TYPE_4
                    .iter()
                    .nth(val as usize)
                    .ok_or_else(|| {
                        let msg = format!("Invalid index of filter type 4: {}", val);
                        Error::new(FileError::Inval, &msg)
                    })
                    .map(|&filter_type| filter_types.push(filter_type))
            })?;
        self.write_equalizer_state(sequence_number, unit, req, timeout_ms, |state| {
            func(state, &filter_types);
            Ok(())
        })
    }

    fn write_equalizer(
        &mut self,
        sequence_number: &mut u8,
        unit: &mut SndMotu,
        req: &mut FwReq,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        let name = elem_id.get_name();

        if name == Self::ENABLE_NAME {
            self.write_bool_values(sequence_number, unit, req, elem_value, timeout_ms, |state, vals| {
                state.enable.copy_from_slice(vals);
            })
        } else if name == Self::HPF_ENABLE_NAME {
            self.write_bool_values(sequence_number, unit, req, elem_value, timeout_ms, |state, vals| {
                state.hpf_enable.copy_from_slice(vals);
            })
        } else if name == Self::HPF_SLOPE_NAME {
            self.write_roll_off_level(sequence_number, unit, req, elem_value, timeout_ms, |state, vals| {
                state.hpf_slope.copy_from_slice(vals)
            })
        } else if name == Self::HPF_FREQ_NAME {
            self.write_int_values(sequence_number, unit, req, elem_value, timeout_ms, |state, vals| {
                state.hpf_freq.copy_from_slice(vals);
            })
        } else if name == Self::LPF_ENABLE_NAME {
            self.write_bool_values(sequence_number, unit, req, elem_value, timeout_ms, |state, vals| {
                state.lpf_enable.copy_from_slice(vals);
            })
        } else if name == Self::LPF_SLOPE_NAME {
            self.write_roll_off_level(sequence_number, unit, req, elem_value, timeout_ms, |state, vals| {
                state.lpf_slope.copy_from_slice(vals)
            })
        } else if name == Self::LPF_FREQ_NAME {
            self.write_int_values(sequence_number, unit, req, elem_value, timeout_ms, |state, vals| {
                state.lpf_freq.copy_from_slice(vals);
            })
        } else if name == Self::LF_ENABLE_NAME {
            self.write_bool_values(sequence_number, unit, req, elem_value, timeout_ms, |state, vals| {
                state.lf_enable.copy_from_slice(vals);
            })
        } else if name == Self::LF_TYPE_NAME {
            self.write_filter_type_5(sequence_number, unit, req, elem_value, timeout_ms, |state, vals| {
                state.lf_type.copy_from_slice(vals)
            })
        } else if name == Self::LF_FREQ_NAME {
            self.write_int_values(sequence_number, unit, req, elem_value, timeout_ms, |state, vals| {
                state.lf_freq.copy_from_slice(vals);
            })
        } else if name == Self::LF_GAIN_NAME {
            self.write_int_values(sequence_number, unit, req, elem_value, timeout_ms, |state, vals| {
                state.lf_gain.copy_from_slice(vals);
            })
        } else if name == Self::LF_WIDTH_NAME {
            self.write_int_values(sequence_number, unit, req, elem_value, timeout_ms, |state, vals| {
                state.lf_width.copy_from_slice(vals);
            })
        } else if name == Self::LMF_ENABLE_NAME {
            self.write_bool_values(sequence_number, unit, req, elem_value, timeout_ms, |state, vals| {
                state.lmf_enable.copy_from_slice(vals);
            })
        } else if name == Self::LMF_TYPE_NAME {
            self.write_filter_type_4(sequence_number, unit, req, elem_value, timeout_ms, |state, vals| {
                state.lmf_type.copy_from_slice(vals)
            })
        } else if name == Self::LMF_FREQ_NAME {
            self.write_int_values(sequence_number, unit, req, elem_value, timeout_ms, |state, vals| {
                state.lmf_freq.copy_from_slice(vals);
            })
        } else if name == Self::LMF_GAIN_NAME {
            self.write_int_values(sequence_number, unit, req, elem_value, timeout_ms, |state, vals| {
                state.lmf_gain.copy_from_slice(vals);
            })
        } else if name == Self::LMF_WIDTH_NAME {
            self.write_int_values(sequence_number, unit, req, elem_value, timeout_ms, |state, vals| {
                state.lmf_width.copy_from_slice(vals);
            })
        } else if name == Self::MF_ENABLE_NAME {
            self.write_bool_values(sequence_number, unit, req, elem_value, timeout_ms, |state, vals| {
                state.mf_enable.copy_from_slice(vals);
            })
        } else if name == Self::MF_TYPE_NAME {
            self.write_filter_type_4(sequence_number, unit, req, elem_value, timeout_ms, |state, vals| {
                state.mf_type.copy_from_slice(vals)
            })
        } else if name == Self::MF_FREQ_NAME {
            self.write_int_values(sequence_number, unit, req, elem_value, timeout_ms, |state, vals| {
                state.mf_freq.copy_from_slice(vals);
            })
        } else if name == Self::MF_GAIN_NAME {
            self.write_int_values(sequence_number, unit, req, elem_value, timeout_ms, |state, vals| {
                state.mf_gain.copy_from_slice(vals);
            })
        } else if name == Self::MF_WIDTH_NAME {
            self.write_int_values(sequence_number, unit, req, elem_value, timeout_ms, |state, vals| {
                state.mf_width.copy_from_slice(vals);
            })
        } else if name == Self::HMF_ENABLE_NAME {
            self.write_bool_values(sequence_number, unit, req, elem_value, timeout_ms, |state, vals| {
                state.hmf_enable.copy_from_slice(vals);
            })
        } else if name == Self::HMF_TYPE_NAME {
            self.write_filter_type_4(sequence_number, unit, req, elem_value, timeout_ms, |state, vals| {
                state.hmf_type.copy_from_slice(vals)
            })
        } else if name == Self::HMF_FREQ_NAME {
            self.write_int_values(sequence_number, unit, req, elem_value, timeout_ms, |state, vals| {
                state.hmf_freq.copy_from_slice(vals);
            })
        } else if name == Self::HMF_GAIN_NAME {
            self.write_int_values(sequence_number, unit, req, elem_value, timeout_ms, |state, vals| {
                state.hmf_gain.copy_from_slice(vals);
            })
        } else if name == Self::HMF_WIDTH_NAME {
            self.write_int_values(sequence_number, unit, req, elem_value, timeout_ms, |state, vals| {
                state.hmf_width.copy_from_slice(vals);
            })
        } else if name == Self::HF_ENABLE_NAME {
            self.write_bool_values(sequence_number, unit, req, elem_value, timeout_ms, |state, vals| {
                state.hf_enable.copy_from_slice(vals);
            })
        } else if name == Self::HF_TYPE_NAME {
            self.write_filter_type_5(sequence_number, unit, req, elem_value, timeout_ms, |state, vals| {
                state.hf_type.copy_from_slice(vals)
            })
        } else if name == Self::HF_FREQ_NAME {
            self.write_int_values(sequence_number, unit, req, elem_value, timeout_ms, |state, vals| {
                state.hf_freq.copy_from_slice(vals);
            })
        } else if name == Self::HF_GAIN_NAME {
            self.write_int_values(sequence_number, unit, req, elem_value, timeout_ms, |state, vals| {
                state.hf_gain.copy_from_slice(vals);
            })
        } else if name == Self::HF_WIDTH_NAME {
            self.write_int_values(sequence_number, unit, req, elem_value, timeout_ms, |state, vals| {
                state.hf_width.copy_from_slice(vals);
            })
        } else {
            Ok(false)
        }
    }

    fn write_equalizer_state<F>(
        &mut self,
        sequence_number: &mut u8,
        unit: &mut SndMotu,
        req: &mut FwReq,
        timeout_ms: u32,
        func: F,
    ) -> Result<bool, Error>
        where F: Fn(&mut CommandDspEqualizerState) -> Result<(), Error>;
}

fn level_detect_mode_to_str(mode: &LevelDetectMode) -> &'static str {
    match mode {
        LevelDetectMode::Peak => "peak",
        LevelDetectMode::Rms => "rms",
        LevelDetectMode::Reserved(_) => "reserved",
    }
}

fn leveler_mode_to_str(mode: &LevelerMode) -> &'static str {
    match mode {
        LevelerMode::Compress => "compress",
        LevelerMode::Limit => "limit",
        LevelerMode::Reserved(_) => "reserved",
    }
}

// TODO: better trait parameters to distinguish input and output.
pub trait CommandDspDynamicsCtlOperation<T: CommandDspOperation, U: Default> {
    const CH_COUNT: usize;

    fn state(&self) -> &CommandDspDynamicsState;

    const ENABLE_NAME: &'static str = "input-dynamics-enable";

    const COMP_ENABLE_NAME: &'static str;
    const COMP_DETECT_MODE_NAME: &'static str;
    const COMP_THRESHOLD_NAME: &'static str;
    const COMP_RATIO_NAME: &'static str;
    const COMP_ATTACK_NAME: &'static str;
    const COMP_RELEASE_NAME: &'static str;
    const COMP_GAIN_NAME: &'static str;

    const LEVELER_ENABLE_NAME: &'static str;
    const LEVELER_MODE_NAME: &'static str;
    const LEVELER_MAKEUP_NAME: &'static str;
    const LEVELER_REDUCE_NAME: &'static str;

    const LEVEL_DETECT_MODES: [LevelDetectMode; 2] = [
        LevelDetectMode::Peak,
        LevelDetectMode::Rms,
    ];

    const LEVELER_MODES: [LevelerMode; 2] = [
        LevelerMode::Compress,
        LevelerMode::Limit,
    ];

    fn load_dynamics(
        &mut self,
        card_cntr: &mut CardCntr,
    ) -> Result<Vec<ElemId>, Error> {
        let mut notified_elem_id_list = Vec::new();

        // Enable.
        [
            Self::ENABLE_NAME,
            Self::COMP_ENABLE_NAME,
            Self::LEVELER_ENABLE_NAME,
        ]
            .iter()
            .try_for_each(|name| {
                let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
                card_cntr.add_bool_elems(&elem_id, 1, Self::CH_COUNT, true)
                    .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))
            })?;

        let labels: Vec<&str> = Self::LEVEL_DETECT_MODES
            .iter()
            .map(|m| level_detect_mode_to_str(m))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::COMP_DETECT_MODE_NAME, 0);
        card_cntr.add_enum_elems(&elem_id, 1, Self::CH_COUNT, &labels, None, true)
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::COMP_THRESHOLD_NAME, 0);
        card_cntr.add_int_elems(
            &elem_id,
            1,
            DynamicsParameter::THRESHOLD_MIN,
            DynamicsParameter::THRESHOLD_MAX,
            DynamicsParameter::THRESHOLD_STEP,
            Self::CH_COUNT,
            None,
            true,
        )
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::COMP_RATIO_NAME, 0);
        card_cntr.add_int_elems(
            &elem_id,
            1,
            DynamicsParameter::RATIO_MIN,
            DynamicsParameter::RATIO_MAX,
            DynamicsParameter::RATIO_STEP,
            Self::CH_COUNT,
            None,
            true,
        )
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::COMP_ATTACK_NAME, 0);
        card_cntr.add_int_elems(
            &elem_id,
            1,
            DynamicsParameter::ATTACK_MIN,
            DynamicsParameter::ATTACK_MAX,
            DynamicsParameter::ATTACK_STEP,
            Self::CH_COUNT,
            None,
            true,
        )
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::COMP_RELEASE_NAME, 0);
        card_cntr.add_int_elems(
            &elem_id,
            1,
            DynamicsParameter::RELEASE_MIN,
            DynamicsParameter::RELEASE_MAX,
            DynamicsParameter::RELEASE_STEP,
            Self::CH_COUNT,
            None,
            true,
        )
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::COMP_GAIN_NAME, 0);
        card_cntr.add_int_elems(
            &elem_id,
            1,
            DynamicsParameter::GAIN_MIN,
            DynamicsParameter::GAIN_MAX,
            DynamicsParameter::GAIN_STEP,
            Self::CH_COUNT,
            None,
            true,
        )
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        let labels: Vec<&str> = Self::LEVELER_MODES
            .iter()
            .map(|m| leveler_mode_to_str(m))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::LEVELER_MODE_NAME, 0);
        card_cntr.add_enum_elems(&elem_id, 1, Self::CH_COUNT, &labels, None, true)
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::LEVELER_MAKEUP_NAME, 0);
        card_cntr.add_int_elems(
            &elem_id,
            1,
            DynamicsParameter::PERCENTAGE_MIN,
            DynamicsParameter::PERCENTAGE_MAX,
            DynamicsParameter::PERCENTAGE_STEP,
            Self::CH_COUNT,
            None,
            true,
        )
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::LEVELER_REDUCE_NAME, 0);
        card_cntr.add_int_elems(
            &elem_id,
            1,
            DynamicsParameter::PERCENTAGE_MIN,
            DynamicsParameter::PERCENTAGE_MAX,
            DynamicsParameter::PERCENTAGE_STEP,
            Self::CH_COUNT,
            None,
            true,
        )
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        Ok(notified_elem_id_list)
    }

    fn read_bool_values(
        elem_value: &mut ElemValue,
        vals: &[bool],
    ) -> Result<bool, Error> {
        assert_eq!(vals.len(), Self::CH_COUNT);

        elem_value.set_bool(vals);
        Ok(true)
    }

    fn read_int_values(
        elem_value: &mut ElemValue,
        vals: &[i32],
    ) -> Result<bool, Error> {
        assert_eq!(vals.len(), Self::CH_COUNT);

        elem_value.set_int(vals);
        Ok(true)
    }

    fn read_level_detect_mode(
        elem_value: &mut ElemValue,
        modes: &[LevelDetectMode],
    ) -> Result<bool, Error> {
        assert_eq!(modes.len(), Self::CH_COUNT);

        ElemValueAccessor::<u32>::set_vals(elem_value, Self::CH_COUNT, |idx| {
            let pos = Self::LEVEL_DETECT_MODES
                .iter()
                .position(|m| modes[idx].eq(m))
                .unwrap();
            Ok(pos as u32)
        })
            .map(|_| true)
    }

    fn read_leveler_mode(
        elem_value: &mut ElemValue,
        modes: &[LevelerMode],
    ) -> Result<bool, Error> {
        assert_eq!(modes.len(), Self::CH_COUNT);

        ElemValueAccessor::<u32>::set_vals(elem_value, Self::CH_COUNT, |idx| {
            let pos = Self::LEVELER_MODES
                .iter()
                .position(|m| modes[idx].eq(m))
                .unwrap();
            Ok(pos as u32)
        })
            .map(|_| true)
    }

    fn read_dynamics(
        &mut self,
        elem_id: &ElemId,
        elem_value: &mut ElemValue
    ) -> Result<bool, Error> {
        let name = elem_id.get_name();

        if name == Self::ENABLE_NAME {
            Self::read_bool_values(elem_value, &self.state().enable)
        } else if name == Self::COMP_ENABLE_NAME {
            Self::read_bool_values(elem_value, &self.state().comp_enable)
        } else if name == Self::COMP_DETECT_MODE_NAME {
            Self::read_level_detect_mode(elem_value, &self.state().comp_detect_mode)
        } else if name == Self::COMP_THRESHOLD_NAME {
            Self::read_int_values(elem_value, &self.state().comp_threshold)
        } else if name == Self::COMP_RATIO_NAME {
            Self::read_int_values(elem_value, &self.state().comp_ratio)
        } else if name == Self::COMP_ATTACK_NAME {
            Self::read_int_values(elem_value, &self.state().comp_attack)
        } else if name == Self::COMP_RELEASE_NAME {
            Self::read_int_values(elem_value, &self.state().comp_release)
        } else if name == Self::COMP_GAIN_NAME {
            Self::read_int_values(elem_value, &self.state().comp_trim)
        } else if name == Self::LEVELER_ENABLE_NAME {
            Self::read_bool_values(elem_value, &self.state().leveler_enable)
        } else if name == Self::LEVELER_MODE_NAME {
            Self::read_leveler_mode(elem_value, &self.state().leveler_mode)
        } else if name == Self::LEVELER_MAKEUP_NAME {
            Self::read_int_values(elem_value, &self.state().leveler_makeup)
        } else if name == Self::LEVELER_REDUCE_NAME {
            Self::read_int_values(elem_value, &self.state().leveler_reduce)
        } else {
            Ok(false)
        }
    }

    fn write_bool_values<F>(
        &mut self,
        sequence_number: &mut u8,
        unit: &mut SndMotu,
        req: &mut FwReq,
        elem_value: &ElemValue,
        timeout_ms: u32,
        func: F,
    ) -> Result<bool, Error>
        where F: Fn(&mut CommandDspDynamicsState, &[bool]),
    {
        let mut vals = vec![false; Self::CH_COUNT];
        elem_value.get_bool(&mut vals);
        self.write_dynamics_state(sequence_number, unit, req, timeout_ms, |state| {
            func(state, &vals);
            Ok(())
        })
    }

    fn write_int_values<F>(
        &mut self,
        sequence_number: &mut u8,
        unit: &mut SndMotu,
        req: &mut FwReq,
        elem_value: &ElemValue,
        timeout_ms: u32,
        func: F,
    ) -> Result<bool, Error>
        where F: Fn(&mut CommandDspDynamicsState, &[i32]),
    {
        let mut vals = vec![0; Self::CH_COUNT];
        elem_value.get_int(&mut vals);
        self.write_dynamics_state(sequence_number, unit, req, timeout_ms, |state| {
            func(state, &vals);
            Ok(())
        })
    }

    fn write_level_detect_mode<F>(
        &mut self,
        sequence_number: &mut u8,
        unit: &mut SndMotu,
        req: &mut FwReq,
        elem_value: &ElemValue,
        timeout_ms: u32,
        func: F,
    ) -> Result<bool, Error>
        where F: Fn(&mut CommandDspDynamicsState, &[LevelDetectMode]),
    {
        let mut vals = vec![0; Self::CH_COUNT];
        elem_value.get_enum(&mut vals);
        let mut modes = Vec::new();
        vals
            .iter()
            .try_for_each(|&val| {
                Self::LEVEL_DETECT_MODES
                    .iter()
                    .nth(val as usize)
                    .ok_or_else(|| {
                        let msg = format!("Invalid index of level detect modes: {}", val);
                        Error::new(FileError::Inval, &msg)
                    })
                    .map(|&mode| modes.push(mode))
            })?;
        self.write_dynamics_state(sequence_number, unit, req, timeout_ms, |state| {
            func(state, &modes);
            Ok(())
        })
    }

    fn write_leveler_mode<F>(
        &mut self,
        sequence_number: &mut u8,
        unit: &mut SndMotu,
        req: &mut FwReq,
        elem_value: &ElemValue,
        timeout_ms: u32,
        func: F,
    ) -> Result<bool, Error>
        where F: Fn(&mut CommandDspDynamicsState, &[LevelerMode]),
    {
        let mut vals = vec![0; Self::CH_COUNT];
        elem_value.get_enum(&mut vals);
        let mut modes = Vec::new();
        vals
            .iter()
            .try_for_each(|&val| {
                Self::LEVELER_MODES
                    .iter()
                    .nth(val as usize)
                    .ok_or_else(|| {
                        let msg = format!("Invalid index of leveler  modes: {}", val);
                        Error::new(FileError::Inval, &msg)
                    })
                    .map(|&mode| modes.push(mode))
            })?;
        self.write_dynamics_state(sequence_number, unit, req, timeout_ms, |state| {
            func(state, &modes);
            Ok(())
        })
    }

    fn write_dynamics(
        &mut self,
        sequence_number: &mut u8,
        unit: &mut SndMotu,
        req: &mut FwReq,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        let name = elem_id.get_name();

        if name == Self::ENABLE_NAME {
            self.write_bool_values(sequence_number, unit, req, elem_value, timeout_ms, |state, vals| {
                state.enable.copy_from_slice(vals)
            })
        } else if name == Self::COMP_ENABLE_NAME {
            self.write_bool_values(sequence_number, unit, req, elem_value, timeout_ms, |state, vals| {
                state.comp_enable.copy_from_slice(vals)
            })
        } else if name == Self::COMP_DETECT_MODE_NAME {
            self.write_level_detect_mode(sequence_number, unit, req, elem_value, timeout_ms, |state, vals| {
                state.comp_detect_mode.copy_from_slice(vals)
            })
        } else if name == Self::COMP_THRESHOLD_NAME {
            self.write_int_values(sequence_number, unit, req, elem_value, timeout_ms, |state, vals| {
                state.comp_threshold.copy_from_slice(vals)
            })
        } else if name == Self::COMP_RATIO_NAME {
            self.write_int_values(sequence_number, unit, req, elem_value, timeout_ms, |state, vals| {
                state.comp_ratio.copy_from_slice(vals)
            })
        } else if name == Self::COMP_ATTACK_NAME {
            self.write_int_values(sequence_number, unit, req, elem_value, timeout_ms, |state, vals| {
                state.comp_attack.copy_from_slice(vals)
            })
        } else if name == Self::COMP_RELEASE_NAME {
            self.write_int_values(sequence_number, unit, req, elem_value, timeout_ms, |state, vals| {
                state.comp_release.copy_from_slice(vals)
            })
        } else if name == Self::COMP_GAIN_NAME {
            self.write_int_values(sequence_number, unit, req, elem_value, timeout_ms, |state, vals| {
                state.comp_trim.copy_from_slice(vals)
            })
        } else if name == Self::LEVELER_ENABLE_NAME {
            self.write_bool_values(sequence_number, unit, req, elem_value, timeout_ms, |state, vals| {
                state.leveler_enable.copy_from_slice(vals)
            })
        } else if name == Self::LEVELER_MODE_NAME {
            self.write_leveler_mode(sequence_number, unit, req, elem_value, timeout_ms, |state, vals| {
                state.leveler_mode.copy_from_slice(vals)
            })
        } else if name == Self::LEVELER_MAKEUP_NAME {
            self.write_int_values(sequence_number, unit, req, elem_value, timeout_ms, |state, vals| {
                state.leveler_makeup.copy_from_slice(vals)
            })
        } else if name == Self::LEVELER_REDUCE_NAME {
            self.write_int_values(sequence_number, unit, req, elem_value, timeout_ms, |state, vals| {
                state.leveler_reduce.copy_from_slice(vals)
            })
        } else {
            Ok(false)
        }
    }

    fn write_dynamics_state<F>(
        &mut self,
        sequence_number: &mut u8,
        unit: &mut SndMotu,
        req: &mut FwReq,
        timeout_ms: u32,
        func: F,
    ) -> Result<bool, Error>
        where F: Fn(&mut CommandDspDynamicsState) -> Result<(), Error>;
}

fn input_stereo_pair_mode_to_string(mode: &InputStereoPairMode) -> &'static str {
    match mode {
        InputStereoPairMode::LeftRight => "left-right",
        InputStereoPairMode::MonauralStereo => "monaural-stereo",
        InputStereoPairMode::Reserved(_) => "reverved",
    }
}

const INPUT_PHASE_NAME: &str = "input-phase";
const INPUT_PAIR_NAME: &str = "input-pair";
const INPUT_GAIN_NAME: &str = "input-gain";
const INPUT_SWAP_NAME: &str = "input-swap";
const INPUT_STEREO_MODE_NAME: &str = "input-stereo-mode";
const INPUT_WIDTH_NAME: &str = "input-width";

const INPUT_REVERB_SEND_NAME: &str = "input-reverb-send";
const INPUT_REVERB_BALANCE_NAME: &str = "input-reverb-balance";

const MIC_PAD_NAME: &str = "mic-pad";
const MIC_PHANTOM_NAME: &str = "mic-phantom";
const MIC_LIMITTER_NAME: &str = "mic-limitter";
const MIC_LOOKAHEAD_NAME: &str = "mic-lookahead";
const MIC_SOFT_CLIP_NAME: &str = "mic-soft-clip";

pub trait CommandDspInputCtlOperation<T: CommandDspInputOperation> {
    fn state(&self) -> &CommandDspInputState;
    fn state_mut(&mut self) -> &mut CommandDspInputState;

    const STEREO_PAIR_MODES: [InputStereoPairMode; 2] = [
        InputStereoPairMode::LeftRight,
        InputStereoPairMode::MonauralStereo,
    ];

    fn load(
        &mut self,
        card_cntr: &mut CardCntr,
    ) -> Result<Vec<ElemId>, Error> {
        let state = T::create_input_state();
        *self.state_mut() = state;

        let mut notified_elem_id_list = Vec::new();

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, INPUT_PHASE_NAME, 0);
        card_cntr.add_bool_elems(&elem_id, 1, T::INPUT_PORTS.len(), true)
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, INPUT_PAIR_NAME, 0);
        card_cntr.add_bool_elems(&elem_id, 1, T::INPUT_PORTS.len(), true)
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, INPUT_GAIN_NAME, 0);
        card_cntr.add_int_elems(
            &elem_id,
            1,
            T::GAIN_MIN,
            T::GAIN_MAX,
            T::GAIN_STEP,
            T::INPUT_PORTS.len(),
            None,
            true,
        )
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, INPUT_SWAP_NAME, 0);
        card_cntr.add_bool_elems(&elem_id, 1, T::INPUT_PORTS.len(), true)
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        let labels: Vec<&str> = Self::STEREO_PAIR_MODES
            .iter()
            .map(|m| input_stereo_pair_mode_to_string(m))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, INPUT_STEREO_MODE_NAME, 0);
        card_cntr.add_enum_elems(&elem_id, 1, T::INPUT_PORTS.len(), &labels, None, true)
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, INPUT_WIDTH_NAME, 0);
        card_cntr.add_int_elems(
            &elem_id,
            1,
            T::WIDTH_MIN,
            T::WIDTH_MAX,
            T::WIDTH_STEP,
            T::INPUT_PORTS.len(),
            None,
            true,
        )
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, INPUT_REVERB_SEND_NAME, 0);
        card_cntr.add_int_elems(
            &elem_id,
            1,
            T::GAIN_MIN,
            T::GAIN_MAX,
            T::GAIN_STEP,
            T::INPUT_PORTS.len(),
            None,
            true,
        )
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, INPUT_REVERB_BALANCE_NAME, 0);
        card_cntr.add_int_elems(
            &elem_id,
            1,
            T::BALANCE_MIN,
            T::BALANCE_MAX,
            T::BALANCE_STEP,
            T::INPUT_PORTS.len(),
            None,
            true,
        )
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        if T::MIC_COUNT > 0 {
            let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIC_PAD_NAME, 0);
            card_cntr.add_bool_elems(&elem_id, 1, T::MIC_COUNT, true)
                .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

            let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIC_PHANTOM_NAME, 0);
            card_cntr.add_bool_elems(&elem_id, 1, T::MIC_COUNT, true)
                .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

            let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIC_LIMITTER_NAME, 0);
            card_cntr.add_bool_elems(&elem_id, 1, T::MIC_COUNT, true)
                .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

            let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIC_LOOKAHEAD_NAME, 0);
            card_cntr.add_bool_elems(&elem_id, 1, T::MIC_COUNT, true)
                .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

            let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIC_SOFT_CLIP_NAME, 0);
            card_cntr.add_bool_elems(&elem_id, 1, T::MIC_COUNT, true)
                .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;
        }

        Ok(notified_elem_id_list)
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            INPUT_PHASE_NAME => {
                elem_value.set_bool(&self.state().phase);
                Ok(true)
            }
            INPUT_PAIR_NAME => {
                elem_value.set_bool(&self.state().pair);
                Ok(true)
            }
            INPUT_GAIN_NAME => {
                elem_value.set_int(&self.state().gain);
                Ok(true)
            }
            INPUT_SWAP_NAME => {
                elem_value.set_bool(&self.state().swap);
                Ok(true)
            }
            INPUT_STEREO_MODE_NAME => {
                ElemValueAccessor::<u32>::set_vals(elem_value, T::INPUT_PORTS.len(), |idx| {
                    let pos = Self::STEREO_PAIR_MODES
                        .iter()
                        .position(|p| self.state().stereo_mode[idx].eq(p))
                        .unwrap();
                    Ok(pos as u32)
                })
                    .map(|_| true)
            }
            INPUT_WIDTH_NAME => {
                elem_value.set_int(&self.state().width);
                Ok(true)
            }
            INPUT_REVERB_SEND_NAME => {
                elem_value.set_int(&self.state().reverb_send);
                Ok(true)
            }
            INPUT_REVERB_BALANCE_NAME => {
                elem_value.set_int(&self.state().reverb_balance);
                Ok(true)
            }
            MIC_PAD_NAME => {
                elem_value.set_bool(&self.state().pad);
                Ok(true)
            }
            MIC_PHANTOM_NAME => {
                elem_value.set_bool(&self.state().phantom);
                Ok(true)
            }
            MIC_LIMITTER_NAME => {
                elem_value.set_bool(&self.state().limitter);
                Ok(true)
            }
            MIC_LOOKAHEAD_NAME => {
                elem_value.set_bool(&self.state().lookahead);
                Ok(true)
            }
            MIC_SOFT_CLIP_NAME => {
                elem_value.set_bool(&self.state().soft_clip);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write(
        &mut self,
        sequence_number: &mut u8,
        unit: &mut SndMotu,
        req: &mut FwReq,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            INPUT_PHASE_NAME => {
                let mut vals = vec![false; T::INPUT_PORTS.len()];
                elem_value.get_bool(&mut vals);
                self.write_state(sequence_number, unit, req, timeout_ms, |state| {
                    state.phase.copy_from_slice(&vals);
                    Ok(())
                })
            }
            INPUT_PAIR_NAME => {
                let mut vals = vec![false; T::INPUT_PORTS.len()];
                elem_value.get_bool(&mut vals);
                self.write_state(sequence_number, unit, req, timeout_ms, |state| {
                    state.pair.copy_from_slice(&vals);
                    Ok(())
                })
            }
            INPUT_GAIN_NAME => {
                let mut vals = vec![0; T::INPUT_PORTS.len()];
                elem_value.get_int(&mut vals);
                self.write_state(sequence_number, unit, req, timeout_ms, |state| {
                    state.gain.copy_from_slice(&vals);
                    Ok(())
                })
            }
            INPUT_SWAP_NAME => {
                let mut vals = vec![false; T::INPUT_PORTS.len()];
                elem_value.get_bool(&mut vals);
                self.write_state(sequence_number, unit, req, timeout_ms, |state| {
                    state.swap.copy_from_slice(&vals);
                    Ok(())
                })
            }
            INPUT_STEREO_MODE_NAME => {
                let mut vals = vec![0; T::INPUT_PORTS.len()];
                elem_value.get_enum(&mut vals);
                let mut modes = Vec::new();
                vals
                    .iter()
                    .try_for_each(|&val| {
                        Self::STEREO_PAIR_MODES
                            .iter()
                            .nth(val as usize)
                            .ok_or_else(|| {
                                let msg = format!("Invalid index of stereo pair modes: {}", val);
                                Error::new(FileError::Inval, &msg)
                            })
                            .map(|&m| modes.push(m))
                    })?;
                self.write_state(sequence_number, unit, req, timeout_ms, |state| {
                    state.stereo_mode.copy_from_slice(&modes);
                    Ok(())
                })
            }
            INPUT_WIDTH_NAME => {
                let mut vals = vec![0; T::INPUT_PORTS.len()];
                elem_value.get_int(&mut vals);
                self.write_state(sequence_number, unit, req, timeout_ms, |state| {
                    state.width.copy_from_slice(&vals);
                    Ok(())
                })
            }
            INPUT_REVERB_SEND_NAME => {
                let mut vals = vec![0; T::INPUT_PORTS.len()];
                elem_value.get_int(&mut vals);
                self.write_state(sequence_number, unit, req, timeout_ms, |state| {
                    state.reverb_send.copy_from_slice(&vals);
                    Ok(())
                })
            }
            INPUT_REVERB_BALANCE_NAME => {
                let mut vals = vec![0; T::INPUT_PORTS.len()];
                elem_value.get_int(&mut vals);
                self.write_state(sequence_number, unit, req, timeout_ms, |state| {
                    state.reverb_balance.copy_from_slice(&vals);
                    Ok(())
                })
            }
            MIC_PAD_NAME => {
                let mut vals = vec![false; T::MIC_COUNT];
                elem_value.get_bool(&mut vals);
                self.write_state(sequence_number, unit, req, timeout_ms, |state| {
                    state.pad.copy_from_slice(&vals);
                    Ok(())
                })
            }
            MIC_PHANTOM_NAME => {
                let mut vals = vec![false; T::MIC_COUNT];
                elem_value.get_bool(&mut vals);
                self.write_state(sequence_number, unit, req, timeout_ms, |state| {
                    state.phantom.copy_from_slice(&vals);
                    Ok(())
                })
            }
            MIC_LIMITTER_NAME => {
                let mut vals = vec![false; T::MIC_COUNT];
                elem_value.get_bool(&mut vals);
                self.write_state(sequence_number, unit, req, timeout_ms, |state| {
                    state.limitter.copy_from_slice(&vals);
                    Ok(())
                })
            }
            MIC_LOOKAHEAD_NAME => {
                let mut vals = vec![false; T::MIC_COUNT];
                elem_value.get_bool(&mut vals);
                self.write_state(sequence_number, unit, req, timeout_ms, |state| {
                    state.lookahead.copy_from_slice(&vals);
                    Ok(())
                })
            }
            MIC_SOFT_CLIP_NAME => {
                let mut vals = vec![false; T::MIC_COUNT];
                elem_value.get_bool(&mut vals);
                self.write_state(sequence_number, unit, req, timeout_ms, |state| {
                    state.soft_clip.copy_from_slice(&vals);
                    Ok(())
                })
            }
            _ => Ok(false),
        }
    }

    fn parse_commands(&mut self, cmds: &[DspCmd]) {
        T::parse_input_commands(self.state_mut(), cmds);
    }

    fn write_state<F>(
        &mut self,
        sequence_number: &mut u8,
        unit: &mut SndMotu,
        req: &mut FwReq,
        timeout_ms: u32,
        func: F,
    ) -> Result<bool, Error>
        where F: Fn(&mut CommandDspInputState) -> Result<(), Error>,
    {
        let mut state = self.state().clone();
        func(&mut state)?;
        T::write_input_state(
            req,
            &mut unit.get_node(),
            sequence_number,
            state,
            self.state_mut(),
            timeout_ms
        )
            .map(|_| true)
    }
}

impl<O, T> CommandDspEqualizerCtlOperation<T, CommandDspInputState> for O
    where
        O: CommandDspInputCtlOperation<T>,
        T: CommandDspInputOperation,
{
    const CH_COUNT: usize = T::INPUT_PORTS.len();

    const ENABLE_NAME: &'static str = "input-equalizer-enable";

    const HPF_ENABLE_NAME: &'static str = "input-equalizer-hpf-enable";
    const HPF_SLOPE_NAME: &'static str = "input-equalizer-hpf-slope";
    const HPF_FREQ_NAME: &'static str = "input-equalizer-hpf-frequency";

    const LPF_ENABLE_NAME: &'static str = "input-equalizer-lpf-enable";
    const LPF_SLOPE_NAME: &'static str = "input-equalizer-lpf-slope";
    const LPF_FREQ_NAME: &'static str = "input-equalizer-lpf-frequency";

    const LF_ENABLE_NAME: &'static str = "input-equalizer-lf-enable";
    const LF_TYPE_NAME: &'static str = "input-equalizer-lf-type";
    const LF_FREQ_NAME: &'static str = "input-equalizer-lf-frequency";
    const LF_GAIN_NAME: &'static str = "input-equalizer-lf-gain";
    const LF_WIDTH_NAME: &'static str = "input-equalizer-lf-width";

    const LMF_ENABLE_NAME: &'static str = "input-equalizer-lmf-enable";
    const LMF_TYPE_NAME: &'static str = "input-equalizer-lmf-type";
    const LMF_FREQ_NAME: &'static str = "input-equalizer-lmf-frequency";
    const LMF_GAIN_NAME: &'static str = "input-equalizer-lmf-gain";
    const LMF_WIDTH_NAME: &'static str = "input-equalizer-lmf-width";

    const MF_ENABLE_NAME: &'static str = "input-equalizer-mf-enable";
    const MF_TYPE_NAME: &'static str = "input-equalizer-mf-type";
    const MF_FREQ_NAME: &'static str = "input-equalizer-mf-frequency";
    const MF_GAIN_NAME: &'static str = "input-equalizer-mf-gain";
    const MF_WIDTH_NAME: &'static str = "input-equalizer-mf-width";

    const HMF_ENABLE_NAME: &'static str = "input-equalizer-hmf-enable";
    const HMF_TYPE_NAME: &'static str = "input-equalizer-hmf-type";
    const HMF_FREQ_NAME: &'static str = "input-equalizer-hmf-frequency";
    const HMF_GAIN_NAME: &'static str = "input-equalizer-hmf-gain";
    const HMF_WIDTH_NAME: &'static str = "input-equalizer-hmf-width";

    const HF_ENABLE_NAME: &'static str = "input-equalizer-hf-enable";
    const HF_TYPE_NAME: &'static str = "input-equalizer-hf-type";
    const HF_FREQ_NAME: &'static str = "input-equalizer-hf-frequency";
    const HF_GAIN_NAME: &'static str = "input-equalizer-hf-gain";
    const HF_WIDTH_NAME: &'static str = "input-equalizer-hf-width";

    fn state(&self) -> &CommandDspEqualizerState {
        &self.state().equalizer
    }

    fn write_equalizer_state<F>(
        &mut self,
        sequence_number: &mut u8,
        unit: &mut SndMotu,
        req: &mut FwReq,
        timeout_ms: u32,
        func: F,
    ) -> Result<bool, Error>
        where F: Fn(&mut CommandDspEqualizerState) -> Result<(), Error>,
    {
        let mut state = self.state().clone();
        func(&mut state.equalizer)?;
        T::write_input_state(
            req,
            &mut unit.get_node(),
            sequence_number,
            state,
            self.state_mut(),
            timeout_ms
        )
            .map(|_| true)
    }
}

impl<O, T> CommandDspDynamicsCtlOperation<T, CommandDspInputState> for O
    where
        O: CommandDspInputCtlOperation<T>,
        T: CommandDspInputOperation,
{
    const CH_COUNT: usize = T::INPUT_PORTS.len();

    const ENABLE_NAME: &'static str = "input-dynamics-enable";

    const COMP_ENABLE_NAME: &'static str = "input-dynamics-compressor-enable";
    const COMP_DETECT_MODE_NAME: &'static str = "input-dynamics-compressor-detect";
    const COMP_THRESHOLD_NAME: &'static str = "input-dynamics-compressor-threshold";
    const COMP_RATIO_NAME: &'static str = "input-dynamics-compressor-ratio";
    const COMP_ATTACK_NAME: &'static str = "input-dynamics-compressor-attack";
    const COMP_RELEASE_NAME: &'static str = "input-dynamics-compressor-release";
    const COMP_GAIN_NAME: &'static str = "input-dynamics-compressor-gain";

    const LEVELER_ENABLE_NAME: &'static str = "input-dynamics-leveler-enable";
    const LEVELER_MODE_NAME: &'static str = "input-dynamics-leveler-mode";
    const LEVELER_MAKEUP_NAME: &'static str = "input-dynamics-leveler-makeup";
    const LEVELER_REDUCE_NAME: &'static str = "input-dynamics-leveler-reduce";

    fn state(&self) -> &CommandDspDynamicsState {
        &self.state().dynamics
    }

    fn write_dynamics_state<F>(
        &mut self,
        sequence_number: &mut u8,
        unit: &mut SndMotu,
        req: &mut FwReq,
        timeout_ms: u32,
        func: F,
    ) -> Result<bool, Error>
        where F: Fn(&mut CommandDspDynamicsState) -> Result<(), Error>,
    {
        let mut state = self.state().clone();
        func(&mut state.dynamics)?;
        T::write_input_state(
            req,
            &mut unit.get_node(),
            sequence_number,
            state,
            self.state_mut(),
            timeout_ms
        )
            .map(|_| true)
    }
}

const OUTPUT_REVERB_SEND_NAME: &str = "output-reverb-send";
const OUTPUT_REVERB_RETURN_NAME: &str = "output-reverb-return";
const OUTPUT_MASTER_MONITOR_NAME: &str = "output-master-monitor";
const OUTPUT_MASTER_TALKBACK_NAME: &str = "output-master-talkback";
const OUTPUT_MASTER_LISTENBACK_NAME: &str = "output-master-listenback";

pub trait CommandDspOutputCtlOperation<T: CommandDspOutputOperation> {
    fn state(&self) -> &CommandDspOutputState;
    fn state_mut(&mut self) -> &mut CommandDspOutputState;

    fn load(
        &mut self,
        card_cntr: &mut CardCntr,
    ) -> Result<Vec<ElemId>, Error> {
        let state = T::create_output_state();
        *self.state_mut() = state;

        let mut notified_elem_id_list = Vec::new();

        [
            (OUTPUT_REVERB_SEND_NAME, T::GAIN_MIN, T::GAIN_MAX, T::GAIN_STEP),
            (OUTPUT_REVERB_RETURN_NAME, T::VOLUME_MIN, T::VOLUME_MAX, T::VOLUME_STEP),
        ]
            .iter()
            .try_for_each(|&(name, min, max, step)| {
                let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
                card_cntr.add_int_elems(
                    &elem_id,
                    1,
                    min,
                    max,
                    step,
                    T::OUTPUT_PORTS.len(),
                    None,
                    true,
                )
                    .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))
            })?;

        [
            OUTPUT_MASTER_MONITOR_NAME,
            OUTPUT_MASTER_TALKBACK_NAME,
            OUTPUT_MASTER_LISTENBACK_NAME,
        ]
            .iter()
            .try_for_each(|name| {
                let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
                card_cntr.add_bool_elems(&elem_id, 1, T::OUTPUT_PORTS.len(), true)
                    .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))
            })?;

        Ok(notified_elem_id_list)
    }

    fn read_bool_values(elem_value: &mut ElemValue, vals: &[bool]) -> Result<bool, Error> {
        elem_value.set_bool(&vals);
        Ok(true)
    }

    fn read_int_values(elem_value: &mut ElemValue, vals: &[i32]) -> Result<bool, Error> {
        elem_value.set_int(&vals);
        Ok(true)
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            OUTPUT_REVERB_SEND_NAME => {
                Self::read_int_values(elem_value, &self.state().reverb_send)
            }
            OUTPUT_REVERB_RETURN_NAME => {
                Self::read_int_values(elem_value, &self.state().reverb_return)
            }
            OUTPUT_MASTER_MONITOR_NAME => {
                Self::read_bool_values(elem_value, &self.state().master_monitor)
            }
            OUTPUT_MASTER_TALKBACK_NAME => {
                Self::read_bool_values(elem_value, &self.state().master_talkback)
            }
            OUTPUT_MASTER_LISTENBACK_NAME => {
                Self::read_bool_values(elem_value, &self.state().master_listenback)
            }
            _ => Ok(false),
        }
    }

    fn write_bool_values<F>(
        &mut self,
        sequence_number: &mut u8,
        unit: &mut SndMotu,
        req: &mut FwReq,
        elem_value: &ElemValue,
        timeout_ms: u32,
        func: F,
    ) -> Result<bool, Error>
        where F: Fn(&mut CommandDspOutputState, &[bool]),
    {
        let mut vals = vec![false; T::OUTPUT_PORTS.len()];
        elem_value.get_bool(&mut vals);
        self.write_state(sequence_number, unit, req, timeout_ms, |state| {
            func(state, &vals);
            Ok(())
        })
    }

    fn write_int_values<F>(
        &mut self,
        sequence_number: &mut u8,
        unit: &mut SndMotu,
        req: &mut FwReq,
        elem_value: &ElemValue,
        timeout_ms: u32,
        func: F,
    ) -> Result<bool, Error>
        where F: Fn(&mut CommandDspOutputState, &[i32]),
    {
        let mut vals = vec![0; T::OUTPUT_PORTS.len()];
        elem_value.get_int(&mut vals);
        self.write_state(sequence_number, unit, req, timeout_ms, |state| {
            func(state, &vals);
            Ok(())
        })
    }

    fn write(
        &mut self,
        sequence_number: &mut u8,
        unit: &mut SndMotu,
        req: &mut FwReq,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            OUTPUT_REVERB_SEND_NAME => {
                self.write_int_values(sequence_number, unit, req, elem_value, timeout_ms, |state, vals| {
                    state.reverb_send.copy_from_slice(&vals);
                })
            }
            OUTPUT_REVERB_RETURN_NAME => {
                self.write_int_values(sequence_number, unit, req, elem_value, timeout_ms, |state, vals| {
                    state.reverb_return.copy_from_slice(&vals);
                })
            }
            OUTPUT_MASTER_MONITOR_NAME => {
                self.write_bool_values(sequence_number, unit, req, elem_value, timeout_ms, |state, vals| {
                    state.master_monitor.copy_from_slice(&vals);
                })
            }
            OUTPUT_MASTER_TALKBACK_NAME => {
                self.write_bool_values(sequence_number, unit, req, elem_value, timeout_ms, |state, vals| {
                    state.master_talkback.copy_from_slice(&vals);
                })
            }
            OUTPUT_MASTER_LISTENBACK_NAME => {
                self.write_bool_values(sequence_number, unit, req, elem_value, timeout_ms, |state, vals| {
                    state.master_listenback.copy_from_slice(&vals);
                })
            }
            _ => Ok(false),
        }
    }

    fn parse_commands(&mut self, cmds: &[DspCmd]) {
        T::parse_output_commands(self.state_mut(), cmds);
    }

    fn write_state<F>(
        &mut self,
        sequence_number: &mut u8,
        unit: &mut SndMotu,
        req: &mut FwReq,
        timeout_ms: u32,
        func: F,
    ) -> Result<bool, Error>
        where F: Fn(&mut CommandDspOutputState) -> Result<(), Error>,
    {
        let mut state = self.state().clone();
        func(&mut state)?;
        T::write_output_state(
            req,
            &mut unit.get_node(),
            sequence_number,
            state,
            self.state_mut(),
            timeout_ms
        )
            .map(|_| true)
    }
}

impl<O, T> CommandDspEqualizerCtlOperation<T, CommandDspOutputState> for O
    where
        O: CommandDspOutputCtlOperation<T>,
        T: CommandDspOutputOperation,
{
    const CH_COUNT: usize = T::OUTPUT_PORTS.len();

    const ENABLE_NAME: &'static str = "output-equalizer-enable";

    const HPF_ENABLE_NAME: &'static str = "output-equalizer-hpf-enable";
    const HPF_SLOPE_NAME: &'static str = "output-equalizer-hpf-slope";
    const HPF_FREQ_NAME: &'static str = "output-equalizer-hpf-frequency";

    const LPF_ENABLE_NAME: &'static str = "output-equalizer-lpf-enable";
    const LPF_SLOPE_NAME: &'static str = "output-equalizer-lpf-slope";
    const LPF_FREQ_NAME: &'static str = "output-equalizer-lpf-frequency";

    const LF_ENABLE_NAME: &'static str = "output-equalizer-lf-enable";
    const LF_TYPE_NAME: &'static str = "output-equalizer-lf-type";
    const LF_FREQ_NAME: &'static str = "output-equalizer-lf-frequency";
    const LF_GAIN_NAME: &'static str = "output-equalizer-lf-gain";
    const LF_WIDTH_NAME: &'static str = "output-equalizer-lf-width";

    const LMF_ENABLE_NAME: &'static str = "output-equalizer-lmf-enable";
    const LMF_TYPE_NAME: &'static str = "output-equalizer-lmf-type";
    const LMF_FREQ_NAME: &'static str = "output-equalizer-lmf-frequency";
    const LMF_GAIN_NAME: &'static str = "output-equalizer-lmf-gain";
    const LMF_WIDTH_NAME: &'static str = "output-equalizer-lmf-width";

    const MF_ENABLE_NAME: &'static str = "output-equalizer-mf-enable";
    const MF_TYPE_NAME: &'static str = "output-equalizer-mf-type";
    const MF_FREQ_NAME: &'static str = "output-equalizer-mf-frequency";
    const MF_GAIN_NAME: &'static str = "output-equalizer-mf-gain";
    const MF_WIDTH_NAME: &'static str = "output-equalizer-mf-width";

    const HMF_ENABLE_NAME: &'static str = "output-equalizer-hmf-enable";
    const HMF_TYPE_NAME: &'static str = "output-equalizer-hmf-type";
    const HMF_FREQ_NAME: &'static str = "output-equalizer-hmf-frequency";
    const HMF_GAIN_NAME: &'static str = "output-equalizer-hmf-gain";
    const HMF_WIDTH_NAME: &'static str = "output-equalizer-hmf-width";

    const HF_ENABLE_NAME: &'static str = "output-equalizer-hf-enable";
    const HF_TYPE_NAME: &'static str = "output-equalizer-hf-type";
    const HF_FREQ_NAME: &'static str = "output-equalizer-hf-frequency";
    const HF_GAIN_NAME: &'static str = "output-equalizer-hf-gain";
    const HF_WIDTH_NAME: &'static str = "output-equalizer-hf-width";

    fn state(&self) -> &CommandDspEqualizerState {
        &self.state().equalizer
    }

    fn write_equalizer_state<F>(
        &mut self,
        sequence_number: &mut u8,
        unit: &mut SndMotu,
        req: &mut FwReq,
        timeout_ms: u32,
        func: F,
    ) -> Result<bool, Error>
        where F: Fn(&mut CommandDspEqualizerState) -> Result<(), Error>,
    {
        let mut state = self.state().clone();
        func(&mut state.equalizer)?;
        T::write_output_state(
            req,
            &mut unit.get_node(),
            sequence_number,
            state,
            self.state_mut(),
            timeout_ms
        )
            .map(|_| true)
    }
}

impl<O, T> CommandDspDynamicsCtlOperation<T, CommandDspOutputState> for O
    where
        O: CommandDspOutputCtlOperation<T>,
        T: CommandDspOutputOperation,
{
    const CH_COUNT: usize = T::OUTPUT_PORTS.len();

    const ENABLE_NAME: &'static str = "output-dynamics-enable";

    const COMP_ENABLE_NAME: &'static str = "output-dynamics-compressor-enable";
    const COMP_DETECT_MODE_NAME: &'static str = "output-dynamics-compressor-detect";
    const COMP_THRESHOLD_NAME: &'static str = "output-dynamics-compressor-threshold";
    const COMP_RATIO_NAME: &'static str = "output-dynamics-compressor-ratio";
    const COMP_ATTACK_NAME: &'static str = "output-dynamics-compressor-attack";
    const COMP_RELEASE_NAME: &'static str = "output-dynamics-compressor-release";
    const COMP_GAIN_NAME: &'static str = "output-dynamics-compressor-gain";

    const LEVELER_ENABLE_NAME: &'static str = "output-dynamics-leveler-enable";
    const LEVELER_MODE_NAME: &'static str = "output-dynamics-leveler-mode";
    const LEVELER_MAKEUP_NAME: &'static str = "output-dynamics-leveler-makeup";
    const LEVELER_REDUCE_NAME: &'static str = "output-dynamics-leveler-reduce";

    fn state(&self) -> &CommandDspDynamicsState {
        &self.state().dynamics
    }

    fn write_dynamics_state<F>(
        &mut self,
        sequence_number: &mut u8,
        unit: &mut SndMotu,
        req: &mut FwReq,
        timeout_ms: u32,
        func: F,
    ) -> Result<bool, Error>
        where F: Fn(&mut CommandDspDynamicsState) -> Result<(), Error>,
    {
        let mut state = self.state().clone();
        func(&mut state.dynamics)?;
        T::write_output_state(
            req,
            &mut unit.get_node(),
            sequence_number,
            state,
            self.state_mut(),
            timeout_ms
        )
            .map(|_| true)
    }
}

const RESOURCE_USAGE_NAME: &str = "resource-usage";

pub trait CommandDspResourcebCtlOperation {
    fn state(&self) -> &u32;
    fn state_mut(&mut self) -> &mut u32;

    fn load(
        &mut self,
        card_cntr: &mut CardCntr,
    ) -> Result<Vec<ElemId>, Error> {
        let mut notified_elem_id_list = Vec::new();

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, RESOURCE_USAGE_NAME, 0);
        card_cntr.add_int_elems(
            &elem_id,
            1,
            0x00000000,
            0x42c80000,
            0x01,
            1,
            None,
            false,
        )
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        Ok(notified_elem_id_list)
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            RESOURCE_USAGE_NAME => {
                let val = *self.state() as i32;
                elem_value.set_int(&[val]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn parse_commands(&mut self, cmds: &[DspCmd]) {
        cmds
            .iter()
            .for_each(|cmd| {
                if let DspCmd::Resource(c) = cmd {
                    match c {
                        // TODO: flag?
                        ResourceCmd::Usage(usage, _) => *self.state_mut() = *usage,
                        _ => (),
                    }
                }
            });
    }
}
