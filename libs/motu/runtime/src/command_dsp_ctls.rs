// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

use glib::{Error, FileError};

use hinawa::FwReq;
use hinawa::{SndMotu, SndUnitExt};

use alsactl::{ElemId, ElemIfaceType, ElemValue, ElemValueExt, ElemValueExtManual};

//use alsa_ctl_tlv_codec::items::DbInterval;

use core::card_cntr::*;
//use core::elem_value_accessor::*;

use motu_protocols::command_dsp::*;

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
