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
            T::WIDTH_MIN,
            T::WIDTH_MAX,
            T::WIDTH_STEP,
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
            T::REFLECTION_LEVEL_MIN,
            T::REFLECTION_LEVEL_MAX,
            T::REFLECTION_LEVEL_STEP,
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
                elem_value.set_int(&[self.state().width]);
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
                elem_value.set_int(&[self.state().reflection_level]);
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
                self.write_state(sequence_number, unit, req, timeout_ms, |state| {
                    state.width = vals[0];
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
                self.write_state(sequence_number, unit, req, timeout_ms, |state| {
                    state.reflection_level = vals[0];
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
