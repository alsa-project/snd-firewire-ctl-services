// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

pub(crate) use super::command_dsp_runtime::*;

const F32_CONVERT_SCALE: f32 = 1000000.0;

fn f32_to_i32(val: f32) -> Result<i32, Error> {
    if val == f32::INFINITY
        || val == f32::NEG_INFINITY
        || val >= (i32::MAX / F32_CONVERT_SCALE as i32) as f32
        || val <= (i32::MIN / F32_CONVERT_SCALE as i32) as f32
    {
        let msg = format!(
            "{}f32 multiplied by {}f32 can not be casted to i32",
            val, F32_CONVERT_SCALE
        );
        Err(Error::new(FileError::Inval, &msg))
    } else {
        Ok((val * F32_CONVERT_SCALE) as i32)
    }
}

fn f32_from_i32(val: i32) -> Result<f32, Error> {
    if val >= (f32::MAX * F32_CONVERT_SCALE) as i32 || val <= (f32::MIN * F32_CONVERT_SCALE) as i32
    {
        let msg = format!(
            "{}i32 divided by {}i32 can not be casted to f32",
            val, F32_CONVERT_SCALE as i32,
        );
        Err(Error::new(FileError::Inval, &msg))
    } else {
        Ok((val as f32) / F32_CONVERT_SCALE)
    }
}

fn read_f32_to_i32_value(dst: &mut ElemValue, src: &f32) -> Result<(), Error> {
    f32_to_i32(*src).map(|val| dst.set_int(&[val]))
}

fn write_f32_from_i32_value(dst: &mut f32, src: &ElemValue) -> Result<(), Error> {
    f32_from_i32(src.int()[0]).map(|val| *dst = val)
}

fn read_f32_to_i32_values(dst: &mut ElemValue, src: &[f32]) -> Result<(), Error> {
    let mut vals = Vec::new();
    src.iter()
        .try_for_each(|&val| f32_to_i32(val).map(|val| vals.push(val)))
        .map(|_| dst.set_int(&vals))
}

fn write_f32_from_i32_values(dst: &mut [f32], src: &ElemValue) -> Result<(), Error> {
    dst.iter_mut()
        .zip(src.int())
        .try_for_each(|(d, &val)| f32_from_i32(val).map(|v| *d = v))
}

fn u32_to_i32(val: u32) -> Result<i32, Error> {
    if val >= i32::MAX as u32 {
        let msg = format!("{}u32 can not be casted to i32", val,);
        Err(Error::new(FileError::Inval, &msg))
    } else {
        Ok(val as i32)
    }
}

fn u32_from_i32(val: i32) -> Result<u32, Error> {
    if val < 0 {
        let msg = format!("{}i32 can not be casted to u32", val,);
        Err(Error::new(FileError::Inval, &msg))
    } else {
        Ok(val as u32)
    }
}

fn read_u32_to_i32_value(dst: &mut ElemValue, src: &u32) -> Result<(), Error> {
    u32_to_i32(*src).map(|val| dst.set_int(&[val]))
}

fn write_u32_from_i32_value(dst: &mut u32, src: &ElemValue) -> Result<(), Error> {
    u32_from_i32(src.int()[0]).map(|val| *dst = val)
}

fn read_u32_to_i32_values(dst: &mut ElemValue, src: &[u32]) -> Result<(), Error> {
    let mut vals = Vec::new();
    src.iter()
        .try_for_each(|&val| u32_to_i32(val).map(|v| vals.push(v)))
        .map(|_| dst.set_int(&vals))
}

fn write_u32_from_i32_values(dst: &mut [u32], src: &ElemValue) -> Result<(), Error> {
    dst.iter_mut()
        .zip(src.int())
        .try_for_each(|(d, &val)| u32_from_i32(val).map(|v| *d = v))
}

fn read_bool_value(dst: &mut ElemValue, src: &bool) {
    dst.set_bool(&[*src]);
}

fn write_bool_value(dst: &mut bool, src: &ElemValue) {
    *dst = src.boolean()[0];
}

fn read_bool_values(dst: &mut ElemValue, src: &[bool]) {
    dst.set_bool(src);
}

fn write_bool_values(dst: &mut [bool], src: &ElemValue) {
    let vals = &src.boolean()[..dst.len()];
    dst.copy_from_slice(vals);
}

fn read_i32_value(dst: &mut ElemValue, src: &i32) {
    dst.set_int(&[*src]);
}

fn write_i32_value(dst: &mut i32, src: &ElemValue) {
    *dst = src.int()[0];
}

fn read_i32_values(dst: &mut ElemValue, src: &[i32]) {
    dst.set_int(src);
}

fn write_i32_values(dst: &mut [i32], src: &ElemValue) {
    let vals = &src.int()[..dst.len()];
    dst.copy_from_slice(vals);
}

fn read_enum_value<T: Eq>(dst: &mut ElemValue, src: &T, table: &[T]) {
    let pos = table.iter().position(|e| src.eq(e)).unwrap();
    dst.set_enum(&[pos as u32]);
}

fn write_enum_value<T: Copy + Eq>(dst: &mut T, src: &ElemValue, table: &[T]) -> Result<(), Error> {
    let pos = src.enumerated()[0] as usize;
    table
        .iter()
        .nth(pos)
        .ok_or_else(|| {
            let msg = format!("Invalid index of enumeration: {}", pos);
            Error::new(FileError::Inval, &msg)
        })
        .map(|&e| *dst = e)
}

fn read_enum_values<T: Eq>(dst: &mut ElemValue, src: &[T], table: &[T]) {
    let vals: Vec<u32> = src
        .iter()
        .map(|enumeration| {
            let pos = table.iter().position(|e| enumeration.eq(e)).unwrap();
            pos as u32
        })
        .collect();
    dst.set_enum(&vals);
}

fn write_enum_values<T: Copy + Eq>(
    dst: &mut [T],
    src: &ElemValue,
    table: &[T],
) -> Result<(), Error> {
    dst.iter_mut()
        .zip(src.enumerated())
        .try_for_each(|(enumeration, &val)| {
            let pos = val as usize;
            table
                .iter()
                .nth(pos)
                .ok_or_else(|| {
                    let msg = format!("Invalid index of enumeration: {}", pos);
                    Error::new(FileError::Inval, &msg)
                })
                .map(|&e| *enumeration = e)
        })
}

#[derive(Default, Debug)]
pub(crate) struct CommandDspReverbCtl<T>
where
    T: MotuCommandDspReverbSpecification
        + MotuCommandDspParametersOperation<CommandDspReverbState>
        + MotuCommandDspUpdatableParamsOperation<CommandDspReverbState>,
{
    pub elem_id_list: Vec<ElemId>,
    params: CommandDspReverbState,
    _phantom: PhantomData<T>,
}

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

impl<T> CommandDspReverbCtl<T>
where
    T: MotuCommandDspReverbSpecification
        + MotuCommandDspParametersOperation<CommandDspReverbState>
        + MotuCommandDspUpdatableParamsOperation<CommandDspReverbState>,
{
    const SPLIT_POINTS: [SplitPoint; 2] = [SplitPoint::Output, SplitPoint::Mixer];

    const ROOM_SHAPES: [RoomShape; 5] = [
        RoomShape::A,
        RoomShape::B,
        RoomShape::C,
        RoomShape::D,
        RoomShape::E,
    ];

    pub(crate) fn parse_commands(&mut self, cmds: &[DspCmd]) {
        for cmd in cmds {
            let _ = T::parse_command(&mut self.params, cmd);
        }
    }

    pub(crate) fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, REVERB_ENABLE, 0);
        card_cntr
            .add_bool_elems(&elem_id, 1, 1, true)
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let labels: Vec<&str> = Self::SPLIT_POINTS
            .iter()
            .map(|p| reverb_split_point_to_str(p))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, REVERB_SPLIT_POINT_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, true)
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, REVERB_DECAY_TIME_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                u32_to_i32(T::DECAY_TIME_MIN)?,
                u32_to_i32(T::DECAY_TIME_MAX)?,
                u32_to_i32(T::DECAY_TIME_STEP)?,
                1,
                None,
                true,
            )
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, REVERB_PRE_DELAY_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                u32_to_i32(T::PRE_DELAY_MIN)?,
                u32_to_i32(T::PRE_DELAY_MAX)?,
                u32_to_i32(T::PRE_DELAY_STEP)?,
                1,
                None,
                true,
            )
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id =
            ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, REVERB_SHELF_FILTER_FREQ_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                u32_to_i32(T::SHELF_FILTER_FREQ_MIN)?,
                u32_to_i32(T::SHELF_FILTER_FREQ_MAX)?,
                u32_to_i32(T::SHELF_FILTER_FREQ_STEP)?,
                1,
                None,
                true,
            )
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id =
            ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, REVERB_SHELF_FILTER_ATTR_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                T::SHELF_FILTER_ATTR_MIN,
                T::SHELF_FILTER_ATTR_MAX,
                T::SHELF_FILTER_ATTR_STEP,
                1,
                None,
                true,
            )
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, REVERB_FREQ_TIME_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                u32_to_i32(T::FREQ_TIME_MIN)?,
                u32_to_i32(T::FREQ_TIME_MAX)?,
                u32_to_i32(T::FREQ_TIME_STEP)?,
                3,
                None,
                true,
            )
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id =
            ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, REVERB_FREQ_CROSSOVER_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                u32_to_i32(T::FREQ_CROSSOVER_MIN)?,
                u32_to_i32(T::FREQ_CROSSOVER_MAX)?,
                u32_to_i32(T::FREQ_CROSSOVER_STEP)?,
                2,
                None,
                true,
            )
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, REVERB_WIDTH_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                f32_to_i32(T::WIDTH_MIN)?,
                f32_to_i32(T::WIDTH_MAX)?,
                1,
                1,
                None,
                true,
            )
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let labels: Vec<&str> = Self::ROOM_SHAPES
            .iter()
            .map(|p| reverb_room_shape_to_str(p))
            .collect();
        let elem_id =
            ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, REVERB_REFLECTION_MODE_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, true)
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id =
            ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, REVERB_REFLECTION_SIZE_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                u32_to_i32(T::REFLECTION_SIZE_MIN)?,
                u32_to_i32(T::REFLECTION_SIZE_MAX)?,
                u32_to_i32(T::REFLECTION_SIZE_STEP)?,
                1,
                None,
                true,
            )
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id =
            ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, REVERB_REFLECTION_LEVEL_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                f32_to_i32(T::REFLECTION_LEVEL_MIN)?,
                f32_to_i32(T::REFLECTION_LEVEL_MAX)?,
                1,
                1,
                None,
                true,
            )
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        Ok(())
    }

    pub(crate) fn read(
        &mut self,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            REVERB_ENABLE => {
                read_bool_value(elem_value, &self.params.enable);
                Ok(true)
            }
            REVERB_SPLIT_POINT_NAME => {
                read_enum_value(elem_value, &self.params.split_point, &Self::SPLIT_POINTS);
                Ok(true)
            }
            REVERB_PRE_DELAY_NAME => {
                read_u32_to_i32_value(elem_value, &self.params.pre_delay)?;
                Ok(true)
            }
            REVERB_SHELF_FILTER_FREQ_NAME => {
                read_u32_to_i32_value(elem_value, &self.params.shelf_filter_freq)?;
                Ok(true)
            }
            REVERB_SHELF_FILTER_ATTR_NAME => {
                read_i32_value(elem_value, &self.params.shelf_filter_attenuation);
                Ok(true)
            }
            REVERB_DECAY_TIME_NAME => {
                read_u32_to_i32_value(elem_value, &self.params.decay_time)?;
                Ok(true)
            }
            REVERB_FREQ_TIME_NAME => {
                read_u32_to_i32_values(elem_value, &self.params.freq_time)?;
                Ok(true)
            }
            REVERB_FREQ_CROSSOVER_NAME => {
                read_u32_to_i32_values(elem_value, &self.params.freq_crossover)?;
                Ok(true)
            }
            REVERB_WIDTH_NAME => {
                read_f32_to_i32_value(elem_value, &self.params.width)?;
                Ok(true)
            }
            REVERB_REFLECTION_MODE_NAME => {
                read_enum_value(elem_value, &self.params.reflection_mode, &Self::ROOM_SHAPES);
                Ok(true)
            }
            REVERB_REFLECTION_SIZE_NAME => {
                read_u32_to_i32_value(elem_value, &self.params.reflection_size)?;
                Ok(true)
            }
            REVERB_REFLECTION_LEVEL_NAME => {
                read_f32_to_i32_value(elem_value, &self.params.reflection_level)?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub(crate) fn write(
        &mut self,
        sequence_number: &mut u8,
        req: &mut FwReq,
        node: &mut FwNode,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            REVERB_ENABLE => {
                let mut params = self.params.clone();
                write_bool_value(&mut params.enable, elem_value);
                let res = T::update_partially(
                    req,
                    node,
                    sequence_number,
                    &mut self.params,
                    params,
                    timeout_ms,
                );
                res.map(|_| true)
            }
            REVERB_SPLIT_POINT_NAME => {
                let mut params = self.params.clone();
                write_enum_value(&mut params.split_point, elem_value, &Self::SPLIT_POINTS)?;
                let res = T::update_partially(
                    req,
                    node,
                    sequence_number,
                    &mut self.params,
                    params,
                    timeout_ms,
                );
                res.map(|_| true)
            }
            REVERB_PRE_DELAY_NAME => {
                let mut params = self.params.clone();
                write_u32_from_i32_value(&mut params.pre_delay, elem_value)?;
                let res = T::update_partially(
                    req,
                    node,
                    sequence_number,
                    &mut self.params,
                    params,
                    timeout_ms,
                );
                res.map(|_| true)
            }
            REVERB_SHELF_FILTER_FREQ_NAME => {
                let mut params = self.params.clone();
                write_u32_from_i32_value(&mut params.shelf_filter_freq, elem_value)?;
                let res = T::update_partially(
                    req,
                    node,
                    sequence_number,
                    &mut self.params,
                    params,
                    timeout_ms,
                );
                res.map(|_| true)
            }
            REVERB_SHELF_FILTER_ATTR_NAME => {
                let mut params = self.params.clone();
                write_i32_value(&mut params.shelf_filter_attenuation, elem_value);
                let res = T::update_partially(
                    req,
                    node,
                    sequence_number,
                    &mut self.params,
                    params,
                    timeout_ms,
                );
                res.map(|_| true)
            }
            REVERB_DECAY_TIME_NAME => {
                let mut params = self.params.clone();
                write_u32_from_i32_value(&mut params.decay_time, elem_value)?;
                let res = T::update_partially(
                    req,
                    node,
                    sequence_number,
                    &mut self.params,
                    params,
                    timeout_ms,
                );
                res.map(|_| true)
            }
            REVERB_FREQ_TIME_NAME => {
                let mut params = self.params.clone();
                write_u32_from_i32_values(&mut params.freq_time, elem_value)?;
                let res = T::update_partially(
                    req,
                    node,
                    sequence_number,
                    &mut self.params,
                    params,
                    timeout_ms,
                );
                res.map(|_| true)
            }
            REVERB_FREQ_CROSSOVER_NAME => {
                let mut params = self.params.clone();
                write_u32_from_i32_values(&mut params.freq_crossover, elem_value)?;
                let res = T::update_partially(
                    req,
                    node,
                    sequence_number,
                    &mut self.params,
                    params,
                    timeout_ms,
                );
                res.map(|_| true)
            }
            REVERB_WIDTH_NAME => {
                let mut params = self.params.clone();
                write_f32_from_i32_value(&mut params.width, elem_value)?;
                let res = T::update_partially(
                    req,
                    node,
                    sequence_number,
                    &mut self.params,
                    params,
                    timeout_ms,
                );
                res.map(|_| true)
            }
            REVERB_REFLECTION_MODE_NAME => {
                let mut params = self.params.clone();
                write_enum_value(&mut params.reflection_mode, elem_value, &Self::ROOM_SHAPES)?;
                let res = T::update_partially(
                    req,
                    node,
                    sequence_number,
                    &mut self.params,
                    params,
                    timeout_ms,
                );
                res.map(|_| true)
            }
            REVERB_REFLECTION_SIZE_NAME => {
                let mut params = self.params.clone();
                write_u32_from_i32_value(&mut params.reflection_size, elem_value)?;
                let res = T::update_partially(
                    req,
                    node,
                    sequence_number,
                    &mut self.params,
                    params,
                    timeout_ms,
                );
                res.map(|_| true)
            }
            REVERB_REFLECTION_LEVEL_NAME => {
                let mut params = self.params.clone();
                write_f32_from_i32_value(&mut params.reflection_level, elem_value)?;
                let res = T::update_partially(
                    req,
                    node,
                    sequence_number,
                    &mut self.params,
                    params,
                    timeout_ms,
                );
                res.map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

#[derive(Default, Debug)]
pub(crate) struct CommandDspMonitorCtl<T>
where
    T: MotuCommandDspMonitorSpecification
        + MotuCommandDspParametersOperation<CommandDspMonitorState>
        + MotuCommandDspUpdatableParamsOperation<CommandDspMonitorState>,
{
    pub elem_id_list: Vec<ElemId>,
    params: CommandDspMonitorState,
    _phantom: PhantomData<T>,
}

const MAIN_VOLUME_NAME: &str = "main-volume";
const TALKBACK_ENABLE_NAME: &str = "talkback-enable";
const LISTENBACK_ENABLE_NAME: &str = "listenback-enable";
const TALKBACK_VOLUME_NAME: &str = "talkback-volume";
const LISTENBACK_VOLUME_NAME: &str = "listenback-volume";

impl<T> CommandDspMonitorCtl<T>
where
    T: MotuCommandDspMonitorSpecification
        + MotuCommandDspParametersOperation<CommandDspMonitorState>
        + MotuCommandDspUpdatableParamsOperation<CommandDspMonitorState>,
{
    pub(crate) fn parse_commands(&mut self, cmds: &[DspCmd]) {
        for cmd in cmds {
            let _ = T::parse_command(&mut self.params, cmd);
        }
    }

    pub(crate) fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        [
            MAIN_VOLUME_NAME,
            TALKBACK_VOLUME_NAME,
            LISTENBACK_VOLUME_NAME,
        ]
        .iter()
        .try_for_each(|name| {
            let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
            card_cntr
                .add_int_elems(
                    &elem_id,
                    1,
                    f32_to_i32(T::VOLUME_MIN)?,
                    f32_to_i32(T::VOLUME_MAX)?,
                    1,
                    1,
                    None,
                    true,
                )
                .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))
        })?;

        [TALKBACK_ENABLE_NAME, LISTENBACK_ENABLE_NAME]
            .iter()
            .try_for_each(|name| {
                let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
                card_cntr
                    .add_bool_elems(&elem_id, 1, 1, true)
                    .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))
            })?;

        Ok(())
    }

    pub(crate) fn read(
        &mut self,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            MAIN_VOLUME_NAME => {
                read_f32_to_i32_value(elem_value, &self.params.main_volume)?;
                Ok(true)
            }
            TALKBACK_ENABLE_NAME => {
                read_bool_value(elem_value, &self.params.talkback_enable);
                Ok(true)
            }
            LISTENBACK_ENABLE_NAME => {
                read_bool_value(elem_value, &self.params.listenback_enable);
                Ok(true)
            }
            TALKBACK_VOLUME_NAME => {
                read_f32_to_i32_value(elem_value, &self.params.talkback_volume)?;
                Ok(true)
            }
            LISTENBACK_VOLUME_NAME => {
                read_f32_to_i32_value(elem_value, &self.params.listenback_volume)?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub(crate) fn write(
        &mut self,
        sequence_number: &mut u8,
        req: &mut FwReq,
        node: &mut FwNode,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            MAIN_VOLUME_NAME => {
                let mut params = self.params.clone();
                write_f32_from_i32_value(&mut params.main_volume, elem_value)?;
                let res = T::update_partially(
                    req,
                    node,
                    sequence_number,
                    &mut self.params,
                    params,
                    timeout_ms,
                );
                res.map(|_| true)
            }
            TALKBACK_ENABLE_NAME => {
                let mut params = self.params.clone();
                write_bool_value(&mut params.talkback_enable, elem_value);
                let res = T::update_partially(
                    req,
                    node,
                    sequence_number,
                    &mut self.params,
                    params,
                    timeout_ms,
                );
                res.map(|_| true)
            }
            LISTENBACK_ENABLE_NAME => {
                let mut params = self.params.clone();
                write_bool_value(&mut params.listenback_enable, elem_value);
                let res = T::update_partially(
                    req,
                    node,
                    sequence_number,
                    &mut self.params,
                    params,
                    timeout_ms,
                );
                res.map(|_| true)
            }
            TALKBACK_VOLUME_NAME => {
                let mut params = self.params.clone();
                write_f32_from_i32_value(&mut params.talkback_volume, elem_value)?;
                let res = T::update_partially(
                    req,
                    node,
                    sequence_number,
                    &mut self.params,
                    params,
                    timeout_ms,
                );
                res.map(|_| true)
            }
            LISTENBACK_VOLUME_NAME => {
                let mut params = self.params.clone();
                write_f32_from_i32_value(&mut params.listenback_volume, elem_value)?;
                let res = T::update_partially(
                    req,
                    node,
                    sequence_number,
                    &mut self.params,
                    params,
                    timeout_ms,
                );
                res.map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

#[derive(Debug)]
pub(crate) struct CommandDspMixerCtl<T>
where
    T: MotuCommandDspMixerSpecification
        + MotuCommandDspParametersOperation<CommandDspMixerState>
        + MotuCommandDspUpdatableParamsOperation<CommandDspMixerState>,
{
    pub elem_id_list: Vec<ElemId>,
    params: CommandDspMixerState,
    _phantom: PhantomData<T>,
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

impl<T> Default for CommandDspMixerCtl<T>
where
    T: MotuCommandDspMixerSpecification
        + MotuCommandDspParametersOperation<CommandDspMixerState>
        + MotuCommandDspUpdatableParamsOperation<CommandDspMixerState>,
{
    fn default() -> Self {
        Self {
            elem_id_list: Default::default(),
            params: T::create_mixer_state(),
            _phantom: Default::default(),
        }
    }
}

impl<T> CommandDspMixerCtl<T>
where
    T: MotuCommandDspMixerSpecification
        + MotuCommandDspParametersOperation<CommandDspMixerState>
        + MotuCommandDspUpdatableParamsOperation<CommandDspMixerState>,
{
    const SOURCE_STEREO_PAIR_MODES: [SourceStereoPairMode; 2] =
        [SourceStereoPairMode::Width, SourceStereoPairMode::LrBalance];

    pub(crate) fn parse_commands(&mut self, cmds: &[DspCmd]) {
        for cmd in cmds {
            let _ = T::parse_command(&mut self.params, cmd);
        }
    }

    pub(crate) fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let labels: Vec<String> = T::OUTPUT_PORTS
            .iter()
            .map(|p| target_port_to_string(p))
            .collect();
        let elem_id =
            ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIXER_OUTPUT_DESTINATION_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, T::MIXER_COUNT, &labels, None, true)
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIXER_OUTPUT_MUTE_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, 1, T::MIXER_COUNT, true)
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIXER_OUTPUT_VOLUME_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                f32_to_i32(T::OUTPUT_VOLUME_MIN)?,
                f32_to_i32(T::OUTPUT_VOLUME_MAX)?,
                1,
                T::MIXER_COUNT,
                None,
                true,
            )
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIXER_REVERB_SEND_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                f32_to_i32(T::OUTPUT_VOLUME_MIN)?,
                f32_to_i32(T::OUTPUT_VOLUME_MAX)?,
                1,
                T::MIXER_COUNT,
                None,
                true,
            )
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIXER_REVERB_RETURN_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                f32_to_i32(T::OUTPUT_VOLUME_MIN)?,
                f32_to_i32(T::OUTPUT_VOLUME_MAX)?,
                1,
                T::MIXER_COUNT,
                None,
                true,
            )
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIXER_SOURCE_MUTE_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, T::MIXER_COUNT, T::SOURCE_PORTS.len(), true)
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIXER_SOURCE_SOLO_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, T::MIXER_COUNT, T::SOURCE_PORTS.len(), true)
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIXER_SOURCE_GAIN_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                T::MIXER_COUNT,
                f32_to_i32(T::SOURCE_GAIN_MIN)?,
                f32_to_i32(T::SOURCE_GAIN_MAX)?,
                1,
                T::SOURCE_PORTS.len(),
                None,
                true,
            )
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIXER_SOURCE_PAN_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                T::MIXER_COUNT,
                f32_to_i32(T::SOURCE_PAN_MIN)?,
                f32_to_i32(T::SOURCE_PAN_MAX)?,
                1,
                T::SOURCE_PORTS.len(),
                None,
                true,
            )
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let labels: Vec<&str> = Self::SOURCE_STEREO_PAIR_MODES
            .iter()
            .map(|p| mixer_source_stereo_pair_mode_to_str(p))
            .collect();
        let elem_id = ElemId::new_by_name(
            ElemIfaceType::Mixer,
            0,
            0,
            MIXER_SOURCE_STEREO_PAIR_MODE_NAME,
            0,
        );
        card_cntr
            .add_enum_elems(
                &elem_id,
                T::MIXER_COUNT,
                T::SOURCE_PORTS.len(),
                &labels,
                None,
                true,
            )
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(
            ElemIfaceType::Mixer,
            0,
            0,
            MIXER_SOURCE_STEREO_BALANCE_NAME,
            0,
        );
        card_cntr
            .add_int_elems(
                &elem_id,
                T::MIXER_COUNT,
                f32_to_i32(T::SOURCE_PAN_MIN)?,
                f32_to_i32(T::SOURCE_PAN_MAX)?,
                1,
                T::SOURCE_PORTS.len(),
                None,
                true,
            )
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(
            ElemIfaceType::Mixer,
            0,
            0,
            MIXER_SOURCE_STEREO_WIDTH_NAME,
            0,
        );
        card_cntr
            .add_int_elems(
                &elem_id,
                T::MIXER_COUNT,
                f32_to_i32(T::SOURCE_PAN_MIN)?,
                f32_to_i32(T::SOURCE_PAN_MAX)?,
                1,
                T::SOURCE_PORTS.len(),
                None,
                true,
            )
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        Ok(())
    }

    pub(crate) fn read(
        &mut self,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            MIXER_OUTPUT_DESTINATION_NAME => {
                read_enum_values(elem_value, &self.params.output_assign, T::OUTPUT_PORTS);
                Ok(true)
            }
            MIXER_OUTPUT_MUTE_NAME => {
                read_bool_values(elem_value, &self.params.output_mute);
                Ok(true)
            }
            MIXER_OUTPUT_VOLUME_NAME => {
                read_f32_to_i32_values(elem_value, &self.params.output_volume)?;
                Ok(true)
            }
            MIXER_REVERB_SEND_NAME => {
                read_f32_to_i32_values(elem_value, &self.params.reverb_send)?;
                Ok(true)
            }
            MIXER_REVERB_RETURN_NAME => {
                read_f32_to_i32_values(elem_value, &self.params.reverb_return)?;
                Ok(true)
            }
            MIXER_SOURCE_MUTE_NAME => {
                let mixer = elem_id.index() as usize;
                let src = self.params.source.iter().nth(mixer).ok_or_else(|| {
                    let msg = format!("Invalid index for mixer source: {}", mixer);
                    Error::new(FileError::Inval, &msg)
                })?;
                read_bool_values(elem_value, &src.mute);
                Ok(true)
            }
            MIXER_SOURCE_SOLO_NAME => {
                let mixer = elem_id.index() as usize;
                let src = self.params.source.iter().nth(mixer).ok_or_else(|| {
                    let msg = format!("Invalid index for mixer source: {}", mixer);
                    Error::new(FileError::Inval, &msg)
                })?;
                read_bool_values(elem_value, &src.solo);
                Ok(true)
            }
            MIXER_SOURCE_PAN_NAME => {
                let mixer = elem_id.index() as usize;
                let src = self.params.source.iter().nth(mixer).ok_or_else(|| {
                    let msg = format!("Invalid index for mixer source: {}", mixer);
                    Error::new(FileError::Inval, &msg)
                })?;
                read_f32_to_i32_values(elem_value, &src.pan)?;
                Ok(true)
            }
            MIXER_SOURCE_GAIN_NAME => {
                let mixer = elem_id.index() as usize;
                let src = self.params.source.iter().nth(mixer).ok_or_else(|| {
                    let msg = format!("Invalid index for mixer source: {}", mixer);
                    Error::new(FileError::Inval, &msg)
                })?;
                read_f32_to_i32_values(elem_value, &src.gain)?;
                Ok(true)
            }
            MIXER_SOURCE_STEREO_PAIR_MODE_NAME => {
                let mixer = elem_id.index() as usize;
                let src = self.params.source.iter().nth(mixer).ok_or_else(|| {
                    let msg = format!("Invalid index for mixer source: {}", mixer);
                    Error::new(FileError::Inval, &msg)
                })?;
                read_enum_values(
                    elem_value,
                    &src.stereo_mode,
                    &Self::SOURCE_STEREO_PAIR_MODES,
                );
                Ok(true)
            }
            MIXER_SOURCE_STEREO_BALANCE_NAME => {
                let mixer = elem_id.index() as usize;
                let src = self.params.source.iter().nth(mixer).ok_or_else(|| {
                    let msg = format!("Invalid index for mixer source: {}", mixer);
                    Error::new(FileError::Inval, &msg)
                })?;
                read_f32_to_i32_values(elem_value, &src.stereo_balance)?;
                Ok(true)
            }
            MIXER_SOURCE_STEREO_WIDTH_NAME => {
                let mixer = elem_id.index() as usize;
                let src = self.params.source.iter().nth(mixer).ok_or_else(|| {
                    let msg = format!("Invalid index for mixer source: {}", mixer);
                    Error::new(FileError::Inval, &msg)
                })?;
                read_f32_to_i32_values(elem_value, &src.stereo_width)?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub(crate) fn write(
        &mut self,
        sequence_number: &mut u8,
        req: &mut FwReq,
        node: &mut FwNode,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            MIXER_OUTPUT_DESTINATION_NAME => {
                let mut params = self.params.clone();
                write_enum_values(&mut params.output_assign, elem_value, T::OUTPUT_PORTS)?;
                let res = T::update_partially(
                    req,
                    node,
                    sequence_number,
                    &mut self.params,
                    params,
                    timeout_ms,
                );
                res.map(|_| true)
            }
            MIXER_OUTPUT_MUTE_NAME => {
                let mut params = self.params.clone();
                write_bool_values(&mut params.output_mute, elem_value);
                let res = T::update_partially(
                    req,
                    node,
                    sequence_number,
                    &mut self.params,
                    params,
                    timeout_ms,
                );
                res.map(|_| true)
            }
            MIXER_OUTPUT_VOLUME_NAME => {
                let mut params = self.params.clone();
                write_f32_from_i32_values(&mut params.output_volume, elem_value)?;
                let res = T::update_partially(
                    req,
                    node,
                    sequence_number,
                    &mut self.params,
                    params,
                    timeout_ms,
                );
                res.map(|_| true)
            }
            MIXER_REVERB_SEND_NAME => {
                let mut params = self.params.clone();
                write_f32_from_i32_values(&mut params.reverb_send, elem_value)?;
                let res = T::update_partially(
                    req,
                    node,
                    sequence_number,
                    &mut self.params,
                    params,
                    timeout_ms,
                );
                res.map(|_| true)
            }
            MIXER_REVERB_RETURN_NAME => {
                let mut params = self.params.clone();
                write_f32_from_i32_values(&mut params.reverb_return, elem_value)?;
                let res = T::update_partially(
                    req,
                    node,
                    sequence_number,
                    &mut self.params,
                    params,
                    timeout_ms,
                );
                res.map(|_| true)
            }
            MIXER_SOURCE_MUTE_NAME => {
                let mut params = self.params.clone();
                let mixer = elem_id.index() as usize;
                let src = params.source.iter_mut().nth(mixer).ok_or_else(|| {
                    let msg = format!("Invalid index for mixer source: {}", mixer);
                    Error::new(FileError::Inval, &msg)
                })?;
                write_bool_values(&mut src.mute, elem_value);
                let res = T::update_partially(
                    req,
                    node,
                    sequence_number,
                    &mut self.params,
                    params,
                    timeout_ms,
                );
                res.map(|_| true)
            }
            MIXER_SOURCE_SOLO_NAME => {
                let mut params = self.params.clone();
                let mixer = elem_id.index() as usize;
                let src = params.source.iter_mut().nth(mixer).ok_or_else(|| {
                    let msg = format!("Invalid index for mixer source: {}", mixer);
                    Error::new(FileError::Inval, &msg)
                })?;
                write_bool_values(&mut src.solo, elem_value);
                let res = T::update_partially(
                    req,
                    node,
                    sequence_number,
                    &mut self.params,
                    params,
                    timeout_ms,
                );
                res.map(|_| true)
            }
            MIXER_SOURCE_PAN_NAME => {
                let mut params = self.params.clone();
                let mixer = elem_id.index() as usize;
                let src = params.source.iter_mut().nth(mixer).ok_or_else(|| {
                    let msg = format!("Invalid index for mixer source: {}", mixer);
                    Error::new(FileError::Inval, &msg)
                })?;
                write_f32_from_i32_values(&mut src.pan, elem_value)?;
                let res = T::update_partially(
                    req,
                    node,
                    sequence_number,
                    &mut self.params,
                    params,
                    timeout_ms,
                );
                res.map(|_| true)
            }
            MIXER_SOURCE_GAIN_NAME => {
                let mut params = self.params.clone();
                let mixer = elem_id.index() as usize;
                let src = params.source.iter_mut().nth(mixer).ok_or_else(|| {
                    let msg = format!("Invalid index for mixer source: {}", mixer);
                    Error::new(FileError::Inval, &msg)
                })?;
                write_f32_from_i32_values(&mut src.gain, elem_value)?;
                let res = T::update_partially(
                    req,
                    node,
                    sequence_number,
                    &mut self.params,
                    params,
                    timeout_ms,
                );
                res.map(|_| true)
            }
            MIXER_SOURCE_STEREO_PAIR_MODE_NAME => {
                let mut params = self.params.clone();
                let mixer = elem_id.index() as usize;
                let src = params.source.iter_mut().nth(mixer).ok_or_else(|| {
                    let msg = format!("Invalid index for mixer source: {}", mixer);
                    Error::new(FileError::Inval, &msg)
                })?;
                write_enum_values(
                    &mut src.stereo_mode,
                    elem_value,
                    &Self::SOURCE_STEREO_PAIR_MODES,
                )?;
                let res = T::update_partially(
                    req,
                    node,
                    sequence_number,
                    &mut self.params,
                    params,
                    timeout_ms,
                );
                res.map(|_| true)
            }
            MIXER_SOURCE_STEREO_BALANCE_NAME => {
                let mut params = self.params.clone();
                let mixer = elem_id.index() as usize;
                let src = params.source.iter_mut().nth(mixer).ok_or_else(|| {
                    let msg = format!("Invalid index for mixer source: {}", mixer);
                    Error::new(FileError::Inval, &msg)
                })?;
                write_f32_from_i32_values(&mut src.stereo_balance, elem_value)?;
                let res = T::update_partially(
                    req,
                    node,
                    sequence_number,
                    &mut self.params,
                    params,
                    timeout_ms,
                );
                res.map(|_| true)
            }
            MIXER_SOURCE_STEREO_WIDTH_NAME => {
                let mut params = self.params.clone();
                let mixer = elem_id.index() as usize;
                let src = params.source.iter_mut().nth(mixer).ok_or_else(|| {
                    let msg = format!("Invalid index for mixer source: {}", mixer);
                    Error::new(FileError::Inval, &msg)
                })?;
                write_f32_from_i32_values(&mut src.stereo_width, elem_value)?;
                let res = T::update_partially(
                    req,
                    node,
                    sequence_number,
                    &mut self.params,
                    params,
                    timeout_ms,
                );
                res.map(|_| true)
            }
            _ => Ok(false),
        }
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
pub trait CommandDspEqualizerCtlOperation<T, U>
where
    T: MotuCommandDspEqualizerSpecification + MotuCommandDspUpdatableParamsOperation<U>,
    U: Clone + AsRef<CommandDspEqualizerState> + AsMut<CommandDspEqualizerState>,
{
    const CH_COUNT: usize;

    fn params(&self) -> &U;
    fn params_mut(&mut self) -> &mut U;

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

    fn load_equalizer(&mut self, card_cntr: &mut CardCntr) -> Result<Vec<ElemId>, Error> {
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
            card_cntr
                .add_bool_elems(&elem_id, 1, Self::CH_COUNT, true)
                .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))
        })?;

        // Roll over level.
        let labels: Vec<&str> = Self::ROLL_OFF_LEVELS
            .iter()
            .map(|level| roll_off_level_to_str(level))
            .collect();
        [Self::HPF_SLOPE_NAME, Self::LPF_SLOPE_NAME]
            .iter()
            .try_for_each(|name| {
                let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
                card_cntr
                    .add_enum_elems(&elem_id, 1, Self::CH_COUNT, &labels, None, true)
                    .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))
            })?;

        // Filter type 5.
        let labels: Vec<&str> = Self::FILTER_TYPE_5
            .iter()
            .map(|filter_type| filter_type_5_to_str(filter_type))
            .collect();
        [Self::LF_TYPE_NAME, Self::HF_TYPE_NAME]
            .iter()
            .try_for_each(|name| {
                let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
                card_cntr
                    .add_enum_elems(&elem_id, 1, Self::CH_COUNT, &labels, None, true)
                    .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))
            })?;

        // Filter type 4.
        let labels: Vec<&str> = Self::FILTER_TYPE_4
            .iter()
            .map(|filter_type| filter_type_4_to_str(filter_type))
            .collect();
        [Self::LMF_TYPE_NAME, Self::MF_TYPE_NAME, Self::HMF_TYPE_NAME]
            .iter()
            .try_for_each(|name| {
                let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
                card_cntr
                    .add_enum_elems(&elem_id, 1, Self::CH_COUNT, &labels, None, true)
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
                    u32_to_i32(T::EQUALIZER_FREQ_MIN)?,
                    u32_to_i32(T::EQUALIZER_FREQ_MAX)?,
                    u32_to_i32(T::EQUALIZER_FREQ_STEP)?,
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
                    f32_to_i32(T::EQUALIZER_GAIN_MIN)?,
                    f32_to_i32(T::EQUALIZER_GAIN_MAX)?,
                    1,
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
                    f32_to_i32(T::EQUALIZER_WIDTH_MIN)?,
                    f32_to_i32(T::EQUALIZER_WIDTH_MAX)?,
                    1,
                    Self::CH_COUNT,
                    None,
                    true,
                )
                .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))
        })?;

        Ok(notified_elem_id_list)
    }

    fn read_equalizer(
        &mut self,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        let name = elem_id.name();

        if name == Self::ENABLE_NAME {
            let equalizer = self.params().as_ref();
            read_bool_values(elem_value, &equalizer.enable);
            Ok(true)
        } else if name == Self::HPF_ENABLE_NAME {
            let equalizer = self.params().as_ref();
            read_bool_values(elem_value, &equalizer.hpf_enable);
            Ok(true)
        } else if name == Self::HPF_SLOPE_NAME {
            let equalizer = self.params().as_ref();
            read_enum_values(elem_value, &equalizer.hpf_slope, &Self::ROLL_OFF_LEVELS);
            Ok(true)
        } else if name == Self::HPF_FREQ_NAME {
            let equalizer = self.params().as_ref();
            read_u32_to_i32_values(elem_value, &equalizer.hpf_freq)?;
            Ok(true)
        } else if name == Self::LPF_ENABLE_NAME {
            let equalizer = self.params().as_ref();
            read_bool_values(elem_value, &equalizer.lpf_enable);
            Ok(true)
        } else if name == Self::LPF_SLOPE_NAME {
            let equalizer = self.params().as_ref();
            read_enum_values(elem_value, &equalizer.lpf_slope, &Self::ROLL_OFF_LEVELS);
            Ok(true)
        } else if name == Self::LPF_FREQ_NAME {
            let equalizer = self.params().as_ref();
            read_u32_to_i32_values(elem_value, &equalizer.lpf_freq)?;
            Ok(true)
        } else if name == Self::LF_ENABLE_NAME {
            let equalizer = self.params().as_ref();
            read_bool_values(elem_value, &equalizer.lf_enable);
            Ok(true)
        } else if name == Self::LF_TYPE_NAME {
            let equalizer = self.params().as_ref();
            read_enum_values(elem_value, &equalizer.lf_type, &Self::FILTER_TYPE_5);
            Ok(true)
        } else if name == Self::LF_FREQ_NAME {
            let equalizer = self.params().as_ref();
            read_u32_to_i32_values(elem_value, &equalizer.lf_freq)?;
            Ok(true)
        } else if name == Self::LF_GAIN_NAME {
            let equalizer = self.params().as_ref();
            read_f32_to_i32_values(elem_value, &equalizer.lf_gain)?;
            Ok(true)
        } else if name == Self::LF_WIDTH_NAME {
            let equalizer = self.params().as_ref();
            read_f32_to_i32_values(elem_value, &equalizer.lf_width)?;
            Ok(true)
        } else if name == Self::LMF_ENABLE_NAME {
            let equalizer = self.params().as_ref();
            read_bool_values(elem_value, &equalizer.lmf_enable);
            Ok(true)
        } else if name == Self::LMF_TYPE_NAME {
            let equalizer = self.params().as_ref();
            read_enum_values(elem_value, &equalizer.lmf_type, &Self::FILTER_TYPE_4);
            Ok(true)
        } else if name == Self::LMF_FREQ_NAME {
            let equalizer = self.params().as_ref();
            read_u32_to_i32_values(elem_value, &equalizer.lmf_freq)?;
            Ok(true)
        } else if name == Self::LMF_GAIN_NAME {
            let equalizer = self.params().as_ref();
            read_f32_to_i32_values(elem_value, &equalizer.lmf_gain)?;
            Ok(true)
        } else if name == Self::LMF_WIDTH_NAME {
            let equalizer = self.params().as_ref();
            read_f32_to_i32_values(elem_value, &equalizer.lmf_width)?;
            Ok(true)
        } else if name == Self::MF_ENABLE_NAME {
            let equalizer = self.params().as_ref();
            read_bool_values(elem_value, &equalizer.mf_enable);
            Ok(true)
        } else if name == Self::MF_TYPE_NAME {
            let equalizer = self.params().as_ref();
            read_enum_values(elem_value, &equalizer.mf_type, &Self::FILTER_TYPE_4);
            Ok(true)
        } else if name == Self::MF_FREQ_NAME {
            let equalizer = self.params().as_ref();
            read_u32_to_i32_values(elem_value, &equalizer.mf_freq)?;
            Ok(true)
        } else if name == Self::MF_GAIN_NAME {
            let equalizer = self.params().as_ref();
            read_f32_to_i32_values(elem_value, &equalizer.mf_gain)?;
            Ok(true)
        } else if name == Self::MF_WIDTH_NAME {
            let equalizer = self.params().as_ref();
            read_f32_to_i32_values(elem_value, &equalizer.mf_width)?;
            Ok(true)
        } else if name == Self::HMF_ENABLE_NAME {
            let equalizer = self.params().as_ref();
            read_bool_values(elem_value, &equalizer.hmf_enable);
            Ok(true)
        } else if name == Self::HMF_TYPE_NAME {
            let equalizer = self.params().as_ref();
            read_enum_values(elem_value, &equalizer.hmf_type, &Self::FILTER_TYPE_4);
            Ok(true)
        } else if name == Self::HMF_FREQ_NAME {
            let equalizer = self.params().as_ref();
            read_u32_to_i32_values(elem_value, &equalizer.hmf_freq)?;
            Ok(true)
        } else if name == Self::HMF_GAIN_NAME {
            let equalizer = self.params().as_ref();
            read_f32_to_i32_values(elem_value, &equalizer.hmf_gain)?;
            Ok(true)
        } else if name == Self::HMF_WIDTH_NAME {
            let equalizer = self.params().as_ref();
            read_f32_to_i32_values(elem_value, &equalizer.hmf_width)?;
            Ok(true)
        } else if name == Self::HF_ENABLE_NAME {
            let equalizer = self.params().as_ref();
            read_bool_values(elem_value, &equalizer.hf_enable);
            Ok(true)
        } else if name == Self::HF_TYPE_NAME {
            let equalizer = self.params().as_ref();
            read_enum_values(elem_value, &equalizer.hf_type, &Self::FILTER_TYPE_5);
            Ok(true)
        } else if name == Self::HF_FREQ_NAME {
            let equalizer = self.params().as_ref();
            read_u32_to_i32_values(elem_value, &equalizer.hf_freq)?;
            Ok(true)
        } else if name == Self::HF_GAIN_NAME {
            let equalizer = self.params().as_ref();
            read_f32_to_i32_values(elem_value, &equalizer.hf_gain)?;
            Ok(true)
        } else if name == Self::HF_WIDTH_NAME {
            let equalizer = self.params().as_ref();
            read_f32_to_i32_values(elem_value, &equalizer.hf_width)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write_equalizer(
        &mut self,
        sequence_number: &mut u8,
        req: &mut FwReq,
        node: &mut FwNode,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        let name = elem_id.name();

        if name == Self::ENABLE_NAME {
            let mut params = self.params().clone();
            let equalizer = &mut params.as_mut();
            write_bool_values(&mut equalizer.enable, elem_value);
            let res = T::update_partially(
                req,
                node,
                sequence_number,
                self.params_mut(),
                params,
                timeout_ms,
            );
            res.map(|_| true)
        } else if name == Self::HPF_ENABLE_NAME {
            let mut params = self.params().clone();
            let equalizer = &mut params.as_mut();
            write_bool_values(&mut equalizer.hpf_enable, elem_value);
            let res = T::update_partially(
                req,
                node,
                sequence_number,
                self.params_mut(),
                params,
                timeout_ms,
            );
            res.map(|_| true)
        } else if name == Self::HPF_SLOPE_NAME {
            let mut params = self.params().clone();
            let equalizer = &mut params.as_mut();
            write_enum_values(&mut equalizer.hpf_slope, elem_value, &Self::ROLL_OFF_LEVELS)?;
            let res = T::update_partially(
                req,
                node,
                sequence_number,
                self.params_mut(),
                params,
                timeout_ms,
            );
            res.map(|_| true)
        } else if name == Self::HPF_FREQ_NAME {
            let mut params = self.params().clone();
            let equalizer = &mut params.as_mut();
            write_u32_from_i32_values(&mut equalizer.hpf_freq, elem_value)?;
            let res = T::update_partially(
                req,
                node,
                sequence_number,
                self.params_mut(),
                params,
                timeout_ms,
            );
            res.map(|_| true)
        } else if name == Self::LPF_ENABLE_NAME {
            let mut params = self.params().clone();
            let equalizer = &mut params.as_mut();
            write_bool_values(&mut equalizer.lpf_enable, elem_value);
            let res = T::update_partially(
                req,
                node,
                sequence_number,
                self.params_mut(),
                params,
                timeout_ms,
            );
            res.map(|_| true)
        } else if name == Self::LPF_SLOPE_NAME {
            let mut params = self.params().clone();
            let equalizer = &mut params.as_mut();
            write_enum_values(&mut equalizer.lpf_slope, elem_value, &Self::ROLL_OFF_LEVELS)?;
            let res = T::update_partially(
                req,
                node,
                sequence_number,
                self.params_mut(),
                params,
                timeout_ms,
            );
            res.map(|_| true)
        } else if name == Self::LPF_FREQ_NAME {
            let mut params = self.params().clone();
            let equalizer = &mut params.as_mut();
            write_u32_from_i32_values(&mut equalizer.lpf_freq, elem_value)?;
            let res = T::update_partially(
                req,
                node,
                sequence_number,
                self.params_mut(),
                params,
                timeout_ms,
            );
            res.map(|_| true)
        } else if name == Self::LF_ENABLE_NAME {
            let mut params = self.params().clone();
            let equalizer = &mut params.as_mut();
            write_bool_values(&mut equalizer.lpf_enable, elem_value);
            let res = T::update_partially(
                req,
                node,
                sequence_number,
                self.params_mut(),
                params,
                timeout_ms,
            );
            res.map(|_| true)
        } else if name == Self::LF_TYPE_NAME {
            let mut params = self.params().clone();
            let equalizer = &mut params.as_mut();
            write_enum_values(&mut equalizer.lf_type, elem_value, &Self::FILTER_TYPE_5)?;
            let res = T::update_partially(
                req,
                node,
                sequence_number,
                self.params_mut(),
                params,
                timeout_ms,
            );
            res.map(|_| true)
        } else if name == Self::LF_FREQ_NAME {
            let mut params = self.params().clone();
            let equalizer = &mut params.as_mut();
            write_u32_from_i32_values(&mut equalizer.lf_freq, elem_value)?;
            let res = T::update_partially(
                req,
                node,
                sequence_number,
                self.params_mut(),
                params,
                timeout_ms,
            );
            res.map(|_| true)
        } else if name == Self::LF_GAIN_NAME {
            let mut params = self.params().clone();
            let equalizer = &mut params.as_mut();
            write_f32_from_i32_values(&mut equalizer.lf_gain, elem_value)?;
            let res = T::update_partially(
                req,
                node,
                sequence_number,
                self.params_mut(),
                params,
                timeout_ms,
            );
            res.map(|_| true)
        } else if name == Self::LF_WIDTH_NAME {
            let mut params = self.params().clone();
            let equalizer = &mut params.as_mut();
            write_f32_from_i32_values(&mut equalizer.lf_width, elem_value)?;
            let res = T::update_partially(
                req,
                node,
                sequence_number,
                self.params_mut(),
                params,
                timeout_ms,
            );
            res.map(|_| true)
        } else if name == Self::LMF_ENABLE_NAME {
            let mut params = self.params().clone();
            let equalizer = &mut params.as_mut();
            write_bool_values(&mut equalizer.lmf_enable, elem_value);
            let res = T::update_partially(
                req,
                node,
                sequence_number,
                self.params_mut(),
                params,
                timeout_ms,
            );
            res.map(|_| true)
        } else if name == Self::LMF_TYPE_NAME {
            let mut params = self.params().clone();
            let equalizer = &mut params.as_mut();
            write_enum_values(&mut equalizer.lmf_type, elem_value, &Self::FILTER_TYPE_4)?;
            let res = T::update_partially(
                req,
                node,
                sequence_number,
                self.params_mut(),
                params,
                timeout_ms,
            );
            res.map(|_| true)
        } else if name == Self::LMF_FREQ_NAME {
            let mut params = self.params().clone();
            let equalizer = &mut params.as_mut();
            write_u32_from_i32_values(&mut equalizer.lmf_freq, elem_value)?;
            let res = T::update_partially(
                req,
                node,
                sequence_number,
                self.params_mut(),
                params,
                timeout_ms,
            );
            res.map(|_| true)
        } else if name == Self::LMF_GAIN_NAME {
            let mut params = self.params().clone();
            let equalizer = &mut params.as_mut();
            write_f32_from_i32_values(&mut equalizer.lmf_gain, elem_value)?;
            let res = T::update_partially(
                req,
                node,
                sequence_number,
                self.params_mut(),
                params,
                timeout_ms,
            );
            res.map(|_| true)
        } else if name == Self::LMF_WIDTH_NAME {
            let mut params = self.params().clone();
            let equalizer = &mut params.as_mut();
            write_f32_from_i32_values(&mut equalizer.lmf_width, elem_value)?;
            let res = T::update_partially(
                req,
                node,
                sequence_number,
                self.params_mut(),
                params,
                timeout_ms,
            );
            res.map(|_| true)
        } else if name == Self::MF_ENABLE_NAME {
            let mut params = self.params().clone();
            let equalizer = &mut params.as_mut();
            write_bool_values(&mut equalizer.mf_enable, elem_value);
            let res = T::update_partially(
                req,
                node,
                sequence_number,
                self.params_mut(),
                params,
                timeout_ms,
            );
            res.map(|_| true)
        } else if name == Self::MF_TYPE_NAME {
            let mut params = self.params().clone();
            let equalizer = &mut params.as_mut();
            write_enum_values(&mut equalizer.mf_type, elem_value, &Self::FILTER_TYPE_4)?;
            let res = T::update_partially(
                req,
                node,
                sequence_number,
                self.params_mut(),
                params,
                timeout_ms,
            );
            res.map(|_| true)
        } else if name == Self::MF_FREQ_NAME {
            let mut params = self.params().clone();
            let equalizer = &mut params.as_mut();
            write_u32_from_i32_values(&mut equalizer.mf_freq, elem_value)?;
            let res = T::update_partially(
                req,
                node,
                sequence_number,
                self.params_mut(),
                params,
                timeout_ms,
            );
            res.map(|_| true)
        } else if name == Self::MF_GAIN_NAME {
            let mut params = self.params().clone();
            let equalizer = &mut params.as_mut();
            write_f32_from_i32_values(&mut equalizer.mf_gain, elem_value)?;
            let res = T::update_partially(
                req,
                node,
                sequence_number,
                self.params_mut(),
                params,
                timeout_ms,
            );
            res.map(|_| true)
        } else if name == Self::MF_WIDTH_NAME {
            let mut params = self.params().clone();
            let equalizer = &mut params.as_mut();
            write_f32_from_i32_values(&mut equalizer.mf_width, elem_value)?;
            let res = T::update_partially(
                req,
                node,
                sequence_number,
                self.params_mut(),
                params,
                timeout_ms,
            );
            res.map(|_| true)
        } else if name == Self::HMF_ENABLE_NAME {
            let mut params = self.params().clone();
            let equalizer = &mut params.as_mut();
            write_bool_values(&mut equalizer.hmf_enable, elem_value);
            let res = T::update_partially(
                req,
                node,
                sequence_number,
                self.params_mut(),
                params,
                timeout_ms,
            );
            res.map(|_| true)
        } else if name == Self::HMF_TYPE_NAME {
            let mut params = self.params().clone();
            let equalizer = &mut params.as_mut();
            write_enum_values(&mut equalizer.hmf_type, elem_value, &Self::FILTER_TYPE_4)?;
            let res = T::update_partially(
                req,
                node,
                sequence_number,
                self.params_mut(),
                params,
                timeout_ms,
            );
            res.map(|_| true)
        } else if name == Self::HMF_FREQ_NAME {
            let mut params = self.params().clone();
            let equalizer = &mut params.as_mut();
            write_u32_from_i32_values(&mut equalizer.hmf_freq, elem_value)?;
            let res = T::update_partially(
                req,
                node,
                sequence_number,
                self.params_mut(),
                params,
                timeout_ms,
            );
            res.map(|_| true)
        } else if name == Self::HMF_GAIN_NAME {
            let mut params = self.params().clone();
            let equalizer = &mut params.as_mut();
            write_f32_from_i32_values(&mut equalizer.hmf_gain, elem_value)?;
            let res = T::update_partially(
                req,
                node,
                sequence_number,
                self.params_mut(),
                params,
                timeout_ms,
            );
            res.map(|_| true)
        } else if name == Self::HMF_WIDTH_NAME {
            let mut params = self.params().clone();
            let equalizer = &mut params.as_mut();
            write_f32_from_i32_values(&mut equalizer.hmf_width, elem_value)?;
            let res = T::update_partially(
                req,
                node,
                sequence_number,
                self.params_mut(),
                params,
                timeout_ms,
            );
            res.map(|_| true)
        } else if name == Self::HF_ENABLE_NAME {
            let mut params = self.params().clone();
            let equalizer = &mut params.as_mut();
            write_bool_values(&mut equalizer.hf_enable, elem_value);
            let res = T::update_partially(
                req,
                node,
                sequence_number,
                self.params_mut(),
                params,
                timeout_ms,
            );
            res.map(|_| true)
        } else if name == Self::HF_TYPE_NAME {
            let mut params = self.params().clone();
            let equalizer = &mut params.as_mut();
            write_enum_values(&mut equalizer.hf_type, elem_value, &Self::FILTER_TYPE_5)?;
            let res = T::update_partially(
                req,
                node,
                sequence_number,
                self.params_mut(),
                params,
                timeout_ms,
            );
            res.map(|_| true)
        } else if name == Self::HF_FREQ_NAME {
            let mut params = self.params().clone();
            let equalizer = &mut params.as_mut();
            write_u32_from_i32_values(&mut equalizer.hf_freq, elem_value)?;
            let res = T::update_partially(
                req,
                node,
                sequence_number,
                self.params_mut(),
                params,
                timeout_ms,
            );
            res.map(|_| true)
        } else if name == Self::HF_GAIN_NAME {
            let mut params = self.params().clone();
            let equalizer = &mut params.as_mut();
            write_f32_from_i32_values(&mut equalizer.hf_gain, elem_value)?;
            let res = T::update_partially(
                req,
                node,
                sequence_number,
                self.params_mut(),
                params,
                timeout_ms,
            );
            res.map(|_| true)
        } else if name == Self::HF_WIDTH_NAME {
            let mut params = self.params().clone();
            let equalizer = &mut params.as_mut();
            write_f32_from_i32_values(&mut equalizer.hf_width, elem_value)?;
            let res = T::update_partially(
                req,
                node,
                sequence_number,
                self.params_mut(),
                params,
                timeout_ms,
            );
            res.map(|_| true)
        } else {
            Ok(false)
        }
    }
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
pub trait CommandDspDynamicsCtlOperation<T, U>
where
    T: MotuCommandDspDynamicsSpecification + MotuCommandDspUpdatableParamsOperation<U>,
    U: Clone + AsRef<CommandDspDynamicsState> + AsMut<CommandDspDynamicsState>,
{
    const CH_COUNT: usize;

    fn params(&self) -> &U;
    fn params_mut(&mut self) -> &mut U;

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

    const LEVEL_DETECT_MODES: [LevelDetectMode; 2] = [LevelDetectMode::Peak, LevelDetectMode::Rms];

    const LEVELER_MODES: [LevelerMode; 2] = [LevelerMode::Compress, LevelerMode::Limit];

    fn load_dynamics(&mut self, card_cntr: &mut CardCntr) -> Result<Vec<ElemId>, Error> {
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
            card_cntr
                .add_bool_elems(&elem_id, 1, Self::CH_COUNT, true)
                .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))
        })?;

        let labels: Vec<&str> = Self::LEVEL_DETECT_MODES
            .iter()
            .map(|m| level_detect_mode_to_str(m))
            .collect();
        let elem_id =
            ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::COMP_DETECT_MODE_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, Self::CH_COUNT, &labels, None, true)
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::COMP_THRESHOLD_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                T::COMP_THRESHOLD_MIN,
                T::COMP_THRESHOLD_MAX,
                T::COMP_THRESHOLD_STEP,
                Self::CH_COUNT,
                None,
                true,
            )
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::COMP_RATIO_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                f32_to_i32(T::COMP_RATIO_MIN)?,
                f32_to_i32(T::COMP_RATIO_MAX)?,
                1,
                Self::CH_COUNT,
                None,
                true,
            )
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::COMP_ATTACK_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                T::COMP_ATTACK_MIN as i32,
                T::COMP_ATTACK_MAX as i32,
                T::COMP_ATTACK_STEP as i32,
                Self::CH_COUNT,
                None,
                true,
            )
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::COMP_RELEASE_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                T::COMP_RELEASE_MIN as i32,
                T::COMP_RELEASE_MAX as i32,
                T::COMP_RELEASE_STEP as i32,
                Self::CH_COUNT,
                None,
                true,
            )
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::COMP_GAIN_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                f32_to_i32(T::COMP_GAIN_MIN)?,
                f32_to_i32(T::COMP_GAIN_MAX)?,
                1,
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
        card_cntr
            .add_enum_elems(&elem_id, 1, Self::CH_COUNT, &labels, None, true)
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::LEVELER_MAKEUP_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                u32_to_i32(T::LEVELER_PERCENTAGE_MIN)?,
                u32_to_i32(T::LEVELER_PERCENTAGE_MAX)?,
                u32_to_i32(T::LEVELER_PERCENTAGE_STEP)?,
                Self::CH_COUNT,
                None,
                true,
            )
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::LEVELER_REDUCE_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                T::LEVELER_PERCENTAGE_MIN as i32,
                T::LEVELER_PERCENTAGE_MAX as i32,
                T::LEVELER_PERCENTAGE_STEP as i32,
                Self::CH_COUNT,
                None,
                true,
            )
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        Ok(notified_elem_id_list)
    }

    fn read_dynamics(
        &mut self,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        let name = elem_id.name();

        if name == Self::ENABLE_NAME {
            let dynamics = self.params().as_ref();
            read_bool_values(elem_value, &dynamics.enable);
            Ok(true)
        } else if name == Self::COMP_ENABLE_NAME {
            let dynamics = self.params().as_ref();
            read_bool_values(elem_value, &dynamics.comp_enable);
            Ok(true)
        } else if name == Self::COMP_DETECT_MODE_NAME {
            let dynamics = self.params().as_ref();
            read_enum_values(
                elem_value,
                &dynamics.comp_detect_mode,
                &Self::LEVEL_DETECT_MODES,
            );
            Ok(true)
        } else if name == Self::COMP_THRESHOLD_NAME {
            let dynamics = self.params().as_ref();
            read_i32_values(elem_value, &dynamics.comp_threshold);
            Ok(true)
        } else if name == Self::COMP_RATIO_NAME {
            let dynamics = self.params().as_ref();
            read_f32_to_i32_values(elem_value, &dynamics.comp_ratio)?;
            Ok(true)
        } else if name == Self::COMP_ATTACK_NAME {
            let dynamics = self.params().as_ref();
            read_u32_to_i32_values(elem_value, &dynamics.comp_attack)?;
            Ok(true)
        } else if name == Self::COMP_RELEASE_NAME {
            let dynamics = self.params().as_ref();
            read_u32_to_i32_values(elem_value, &dynamics.comp_release)?;
            Ok(true)
        } else if name == Self::COMP_GAIN_NAME {
            let dynamics = self.params().as_ref();
            read_f32_to_i32_values(elem_value, &dynamics.comp_gain)?;
            Ok(true)
        } else if name == Self::LEVELER_ENABLE_NAME {
            let dynamics = self.params().as_ref();
            read_bool_values(elem_value, &dynamics.leveler_enable);
            Ok(true)
        } else if name == Self::LEVELER_MODE_NAME {
            let dynamics = self.params().as_ref();
            read_enum_values(elem_value, &dynamics.leveler_mode, &Self::LEVELER_MODES);
            Ok(true)
        } else if name == Self::LEVELER_MAKEUP_NAME {
            let dynamics = self.params().as_ref();
            read_u32_to_i32_values(elem_value, &dynamics.leveler_makeup)?;
            Ok(true)
        } else if name == Self::LEVELER_REDUCE_NAME {
            let dynamics = self.params().as_ref();
            read_u32_to_i32_values(elem_value, &dynamics.leveler_reduce)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write_dynamics(
        &mut self,
        sequence_number: &mut u8,
        req: &mut FwReq,
        node: &mut FwNode,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        let name = elem_id.name();

        if name == Self::ENABLE_NAME {
            let mut params = self.params().clone();
            let dynamics = params.as_mut();
            write_bool_values(&mut dynamics.enable, elem_value);
            let res = T::update_partially(
                req,
                node,
                sequence_number,
                self.params_mut(),
                params,
                timeout_ms,
            );
            res.map(|_| true)
        } else if name == Self::COMP_ENABLE_NAME {
            let mut params = self.params().clone();
            let dynamics = params.as_mut();
            write_bool_values(&mut dynamics.comp_enable, elem_value);
            let res = T::update_partially(
                req,
                node,
                sequence_number,
                self.params_mut(),
                params,
                timeout_ms,
            );
            res.map(|_| true)
        } else if name == Self::COMP_DETECT_MODE_NAME {
            let mut params = self.params().clone();
            let dynamics = params.as_mut();
            write_enum_values(
                &mut dynamics.comp_detect_mode,
                elem_value,
                &Self::LEVEL_DETECT_MODES,
            )?;
            let res = T::update_partially(
                req,
                node,
                sequence_number,
                self.params_mut(),
                params,
                timeout_ms,
            );
            res.map(|_| true)
        } else if name == Self::COMP_THRESHOLD_NAME {
            let mut params = self.params().clone();
            let dynamics = params.as_mut();
            write_i32_values(&mut dynamics.comp_threshold, elem_value);
            let res = T::update_partially(
                req,
                node,
                sequence_number,
                self.params_mut(),
                params,
                timeout_ms,
            );
            res.map(|_| true)
        } else if name == Self::COMP_RATIO_NAME {
            let mut params = self.params().clone();
            let dynamics = params.as_mut();
            write_f32_from_i32_values(&mut dynamics.comp_ratio, elem_value)?;
            let res = T::update_partially(
                req,
                node,
                sequence_number,
                self.params_mut(),
                params,
                timeout_ms,
            );
            res.map(|_| true)
        } else if name == Self::COMP_ATTACK_NAME {
            let mut params = self.params().clone();
            let dynamics = params.as_mut();
            write_u32_from_i32_values(&mut dynamics.comp_attack, elem_value)?;
            let res = T::update_partially(
                req,
                node,
                sequence_number,
                self.params_mut(),
                params,
                timeout_ms,
            );
            res.map(|_| true)
        } else if name == Self::COMP_RELEASE_NAME {
            let mut params = self.params().clone();
            let dynamics = params.as_mut();
            write_u32_from_i32_values(&mut dynamics.comp_release, elem_value)?;
            let res = T::update_partially(
                req,
                node,
                sequence_number,
                self.params_mut(),
                params,
                timeout_ms,
            );
            res.map(|_| true)
        } else if name == Self::COMP_GAIN_NAME {
            let mut params = self.params().clone();
            let dynamics = params.as_mut();
            write_f32_from_i32_values(&mut dynamics.comp_gain, elem_value)?;
            let res = T::update_partially(
                req,
                node,
                sequence_number,
                self.params_mut(),
                params,
                timeout_ms,
            );
            res.map(|_| true)
        } else if name == Self::LEVELER_ENABLE_NAME {
            let mut params = self.params().clone();
            let dynamics = params.as_mut();
            write_bool_values(&mut dynamics.leveler_enable, elem_value);
            let res = T::update_partially(
                req,
                node,
                sequence_number,
                self.params_mut(),
                params,
                timeout_ms,
            );
            res.map(|_| true)
        } else if name == Self::LEVELER_MODE_NAME {
            let mut params = self.params().clone();
            let dynamics = params.as_mut();
            write_enum_values(&mut dynamics.leveler_mode, elem_value, &Self::LEVELER_MODES)?;
            let res = T::update_partially(
                req,
                node,
                sequence_number,
                self.params_mut(),
                params,
                timeout_ms,
            );
            res.map(|_| true)
        } else if name == Self::LEVELER_MAKEUP_NAME {
            let mut params = self.params().clone();
            let dynamics = params.as_mut();
            write_u32_from_i32_values(&mut dynamics.leveler_makeup, elem_value)?;
            let res = T::update_partially(
                req,
                node,
                sequence_number,
                self.params_mut(),
                params,
                timeout_ms,
            );
            res.map(|_| true)
        } else if name == Self::LEVELER_REDUCE_NAME {
            let mut params = self.params().clone();
            let dynamics = params.as_mut();
            write_u32_from_i32_values(&mut dynamics.leveler_reduce, elem_value)?;
            let res = T::update_partially(
                req,
                node,
                sequence_number,
                self.params_mut(),
                params,
                timeout_ms,
            );
            res.map(|_| true)
        } else {
            Ok(false)
        }
    }
}

fn input_stereo_pair_mode_to_string(mode: &InputStereoPairMode) -> &'static str {
    match mode {
        InputStereoPairMode::LeftRight => "left-right",
        InputStereoPairMode::MonauralStereo => "monaural-stereo",
        InputStereoPairMode::Reserved(_) => "reverved",
    }
}

#[derive(Debug)]
pub(crate) struct CommandDspInputCtl<T>
where
    T: MotuCommandDspInputSpecification
        + MotuCommandDspEqualizerSpecification
        + MotuCommandDspDynamicsSpecification
        + MotuCommandDspParametersOperation<CommandDspInputState>
        + MotuCommandDspUpdatableParamsOperation<CommandDspInputState>,
{
    pub elem_id_list: Vec<ElemId>,
    params: CommandDspInputState,
    _phantom: PhantomData<T>,
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

impl<T> Default for CommandDspInputCtl<T>
where
    T: MotuCommandDspInputSpecification
        + MotuCommandDspEqualizerSpecification
        + MotuCommandDspDynamicsSpecification
        + MotuCommandDspParametersOperation<CommandDspInputState>
        + MotuCommandDspUpdatableParamsOperation<CommandDspInputState>,
{
    fn default() -> Self {
        Self {
            elem_id_list: Default::default(),
            params: T::create_input_state(),
            _phantom: Default::default(),
        }
    }
}

impl<T> CommandDspInputCtl<T>
where
    T: MotuCommandDspInputSpecification
        + MotuCommandDspEqualizerSpecification
        + MotuCommandDspDynamicsSpecification
        + MotuCommandDspParametersOperation<CommandDspInputState>
        + MotuCommandDspUpdatableParamsOperation<CommandDspInputState>,
{
    const STEREO_PAIR_MODES: [InputStereoPairMode; 2] = [
        InputStereoPairMode::LeftRight,
        InputStereoPairMode::MonauralStereo,
    ];

    pub(crate) fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, INPUT_PHASE_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, 1, T::INPUT_PORTS.len(), true)
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, INPUT_PAIR_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, 1, T::INPUT_PORTS.len(), true)
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, INPUT_GAIN_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                T::INPUT_GAIN_MIN,
                T::INPUT_GAIN_MAX,
                T::INPUT_GAIN_STEP,
                T::INPUT_PORTS.len(),
                None,
                true,
            )
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, INPUT_SWAP_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, 1, T::INPUT_PORTS.len(), true)
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let labels: Vec<&str> = Self::STEREO_PAIR_MODES
            .iter()
            .map(|m| input_stereo_pair_mode_to_string(m))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, INPUT_STEREO_MODE_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, T::INPUT_PORTS.len(), &labels, None, true)
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, INPUT_WIDTH_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                f32_to_i32(T::INPUT_WIDTH_MIN)?,
                f32_to_i32(T::INPUT_WIDTH_MAX)?,
                1,
                T::INPUT_PORTS.len(),
                None,
                true,
            )
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, INPUT_REVERB_SEND_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                f32_to_i32(T::INPUT_REVERB_GAIN_MIN)?,
                f32_to_i32(T::INPUT_REVERB_GAIN_MAX)?,
                1,
                T::INPUT_PORTS.len(),
                None,
                true,
            )
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, INPUT_REVERB_BALANCE_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                f32_to_i32(T::INPUT_REVERB_BALANCE_MIN)?,
                f32_to_i32(T::INPUT_REVERB_BALANCE_MAX)?,
                1,
                T::INPUT_PORTS.len(),
                None,
                true,
            )
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        if T::MIC_COUNT > 0 {
            let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIC_PAD_NAME, 0);
            card_cntr
                .add_bool_elems(&elem_id, 1, T::MIC_COUNT, true)
                .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

            let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIC_PHANTOM_NAME, 0);
            card_cntr
                .add_bool_elems(&elem_id, 1, T::MIC_COUNT, true)
                .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

            let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIC_LIMITTER_NAME, 0);
            card_cntr
                .add_bool_elems(&elem_id, 1, T::MIC_COUNT, true)
                .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

            let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIC_LOOKAHEAD_NAME, 0);
            card_cntr
                .add_bool_elems(&elem_id, 1, T::MIC_COUNT, true)
                .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

            let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIC_SOFT_CLIP_NAME, 0);
            card_cntr
                .add_bool_elems(&elem_id, 1, T::MIC_COUNT, true)
                .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;
        }

        Ok(())
    }

    pub(crate) fn read(
        &mut self,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            INPUT_PHASE_NAME => {
                read_bool_values(elem_value, &self.params.phase);
                Ok(true)
            }
            INPUT_PAIR_NAME => {
                read_bool_values(elem_value, &self.params.pair);
                Ok(true)
            }
            INPUT_GAIN_NAME => {
                read_i32_values(elem_value, &self.params.gain);
                Ok(true)
            }
            INPUT_SWAP_NAME => {
                read_bool_values(elem_value, &self.params.swap);
                Ok(true)
            }
            INPUT_STEREO_MODE_NAME => {
                read_enum_values(
                    elem_value,
                    &self.params.stereo_mode,
                    &Self::STEREO_PAIR_MODES,
                );
                Ok(true)
            }
            INPUT_WIDTH_NAME => {
                read_f32_to_i32_values(elem_value, &self.params.width)?;
                Ok(true)
            }
            INPUT_REVERB_SEND_NAME => {
                read_f32_to_i32_values(elem_value, &self.params.reverb_send)?;
                Ok(true)
            }
            INPUT_REVERB_BALANCE_NAME => {
                read_f32_to_i32_values(elem_value, &self.params.reverb_balance)?;
                Ok(true)
            }
            MIC_PAD_NAME => {
                read_bool_values(elem_value, &self.params.pad);
                Ok(true)
            }
            MIC_PHANTOM_NAME => {
                read_bool_values(elem_value, &self.params.phantom);
                Ok(true)
            }
            MIC_LIMITTER_NAME => {
                read_bool_values(elem_value, &self.params.limitter);
                Ok(true)
            }
            MIC_LOOKAHEAD_NAME => {
                read_bool_values(elem_value, &self.params.lookahead);
                Ok(true)
            }
            MIC_SOFT_CLIP_NAME => {
                read_bool_values(elem_value, &self.params.soft_clip);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub(crate) fn write(
        &mut self,
        sequence_number: &mut u8,
        req: &mut FwReq,
        node: &mut FwNode,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            INPUT_PHASE_NAME => {
                let mut params = self.params.clone();
                write_bool_values(&mut params.phase, elem_value);
                let res = T::update_partially(
                    req,
                    node,
                    sequence_number,
                    &mut self.params,
                    params,
                    timeout_ms,
                );
                res.map(|_| true)
            }
            INPUT_PAIR_NAME => {
                let mut params = self.params.clone();
                write_bool_values(&mut params.pair, elem_value);
                let res = T::update_partially(
                    req,
                    node,
                    sequence_number,
                    &mut self.params,
                    params,
                    timeout_ms,
                );
                res.map(|_| true)
            }
            INPUT_GAIN_NAME => {
                let mut params = self.params.clone();
                write_i32_values(&mut params.gain, elem_value);
                let res = T::update_partially(
                    req,
                    node,
                    sequence_number,
                    &mut self.params,
                    params,
                    timeout_ms,
                );
                res.map(|_| true)
            }
            INPUT_SWAP_NAME => {
                let mut params = self.params.clone();
                write_bool_values(&mut params.swap, elem_value);
                let res = T::update_partially(
                    req,
                    node,
                    sequence_number,
                    &mut self.params,
                    params,
                    timeout_ms,
                );
                res.map(|_| true)
            }
            INPUT_STEREO_MODE_NAME => {
                let mut params = self.params.clone();
                write_enum_values(
                    &mut params.stereo_mode,
                    elem_value,
                    &Self::STEREO_PAIR_MODES,
                )?;
                let res = T::update_partially(
                    req,
                    node,
                    sequence_number,
                    &mut self.params,
                    params,
                    timeout_ms,
                );
                res.map(|_| true)
            }
            INPUT_WIDTH_NAME => {
                let mut params = self.params.clone();
                write_f32_from_i32_values(&mut params.width, elem_value)?;
                let res = T::update_partially(
                    req,
                    node,
                    sequence_number,
                    &mut self.params,
                    params,
                    timeout_ms,
                );
                res.map(|_| true)
            }
            INPUT_REVERB_SEND_NAME => {
                let mut params = self.params.clone();
                write_f32_from_i32_values(&mut params.reverb_send, elem_value)?;
                let res = T::update_partially(
                    req,
                    node,
                    sequence_number,
                    &mut self.params,
                    params,
                    timeout_ms,
                );
                res.map(|_| true)
            }
            INPUT_REVERB_BALANCE_NAME => {
                let mut params = self.params.clone();
                write_f32_from_i32_values(&mut params.reverb_balance, elem_value)?;
                let res = T::update_partially(
                    req,
                    node,
                    sequence_number,
                    &mut self.params,
                    params,
                    timeout_ms,
                );
                res.map(|_| true)
            }
            MIC_PAD_NAME => {
                let mut params = self.params.clone();
                write_bool_values(&mut params.pad, elem_value);
                let res = T::update_partially(
                    req,
                    node,
                    sequence_number,
                    &mut self.params,
                    params,
                    timeout_ms,
                );
                res.map(|_| true)
            }
            MIC_PHANTOM_NAME => {
                let mut params = self.params.clone();
                write_bool_values(&mut params.phantom, elem_value);
                let res = T::update_partially(
                    req,
                    node,
                    sequence_number,
                    &mut self.params,
                    params,
                    timeout_ms,
                );
                res.map(|_| true)
            }
            MIC_LIMITTER_NAME => {
                let mut params = self.params.clone();
                write_bool_values(&mut params.limitter, elem_value);
                let res = T::update_partially(
                    req,
                    node,
                    sequence_number,
                    &mut self.params,
                    params,
                    timeout_ms,
                );
                res.map(|_| true)
            }
            MIC_LOOKAHEAD_NAME => {
                let mut params = self.params.clone();
                write_bool_values(&mut params.lookahead, elem_value);
                let res = T::update_partially(
                    req,
                    node,
                    sequence_number,
                    &mut self.params,
                    params,
                    timeout_ms,
                );
                res.map(|_| true)
            }
            MIC_SOFT_CLIP_NAME => {
                let mut params = self.params.clone();
                write_bool_values(&mut params.soft_clip, elem_value);
                let res = T::update_partially(
                    req,
                    node,
                    sequence_number,
                    &mut self.params,
                    params,
                    timeout_ms,
                );
                res.map(|_| true)
            }
            _ => Ok(false),
        }
    }

    pub(crate) fn parse_commands(&mut self, cmds: &[DspCmd]) {
        for cmd in cmds {
            let _ = T::parse_command(&mut self.params, cmd);
        }
    }
}

impl<T> CommandDspEqualizerCtlOperation<T, CommandDspInputState> for CommandDspInputCtl<T>
where
    T: MotuCommandDspInputSpecification
        + MotuCommandDspEqualizerSpecification
        + MotuCommandDspDynamicsSpecification
        + MotuCommandDspParametersOperation<CommandDspInputState>
        + MotuCommandDspUpdatableParamsOperation<CommandDspInputState>,
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

    fn params(&self) -> &CommandDspInputState {
        &self.params
    }

    fn params_mut(&mut self) -> &mut CommandDspInputState {
        &mut self.params
    }
}

impl<T> CommandDspDynamicsCtlOperation<T, CommandDspInputState> for CommandDspInputCtl<T>
where
    T: MotuCommandDspInputSpecification
        + MotuCommandDspEqualizerSpecification
        + MotuCommandDspDynamicsSpecification
        + MotuCommandDspParametersOperation<CommandDspInputState>
        + MotuCommandDspUpdatableParamsOperation<CommandDspInputState>,
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

    fn params(&self) -> &CommandDspInputState {
        &self.params
    }

    fn params_mut(&mut self) -> &mut CommandDspInputState {
        &mut self.params
    }
}

#[derive(Debug)]
pub(crate) struct CommandDspOutputCtl<T>
where
    T: MotuCommandDspOutputSpecification
        + MotuCommandDspEqualizerSpecification
        + MotuCommandDspDynamicsSpecification
        + MotuCommandDspParametersOperation<CommandDspOutputState>
        + MotuCommandDspUpdatableParamsOperation<CommandDspOutputState>,
{
    pub elem_id_list: Vec<ElemId>,
    params: CommandDspOutputState,
    _phantom: PhantomData<T>,
}

const OUTPUT_REVERB_SEND_NAME: &str = "output-reverb-send";
const OUTPUT_REVERB_RETURN_NAME: &str = "output-reverb-return";
const OUTPUT_MASTER_MONITOR_NAME: &str = "output-master-monitor";
const OUTPUT_MASTER_TALKBACK_NAME: &str = "output-master-talkback";
const OUTPUT_MASTER_LISTENBACK_NAME: &str = "output-master-listenback";

impl<T> Default for CommandDspOutputCtl<T>
where
    T: MotuCommandDspOutputSpecification
        + MotuCommandDspEqualizerSpecification
        + MotuCommandDspDynamicsSpecification
        + MotuCommandDspParametersOperation<CommandDspOutputState>
        + MotuCommandDspUpdatableParamsOperation<CommandDspOutputState>,
{
    fn default() -> Self {
        Self {
            elem_id_list: Default::default(),
            params: T::create_output_state(),
            _phantom: Default::default(),
        }
    }
}

impl<T> CommandDspOutputCtl<T>
where
    T: MotuCommandDspOutputSpecification
        + MotuCommandDspEqualizerSpecification
        + MotuCommandDspDynamicsSpecification
        + MotuCommandDspParametersOperation<CommandDspOutputState>
        + MotuCommandDspUpdatableParamsOperation<CommandDspOutputState>,
{
    pub(crate) fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        [
            (
                OUTPUT_REVERB_SEND_NAME,
                T::OUTPUT_GAIN_MIN,
                T::OUTPUT_GAIN_MAX,
            ),
            (
                OUTPUT_REVERB_RETURN_NAME,
                T::OUTPUT_VOLUME_MIN,
                T::OUTPUT_VOLUME_MAX,
            ),
        ]
        .iter()
        .try_for_each(|&(name, min, max)| {
            let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
            card_cntr
                .add_int_elems(
                    &elem_id,
                    1,
                    f32_to_i32(min)?,
                    f32_to_i32(max)?,
                    1,
                    T::OUTPUT_PORTS.len(),
                    None,
                    true,
                )
                .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))
        })?;

        [
            OUTPUT_MASTER_MONITOR_NAME,
            OUTPUT_MASTER_TALKBACK_NAME,
            OUTPUT_MASTER_LISTENBACK_NAME,
        ]
        .iter()
        .try_for_each(|name| {
            let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
            card_cntr
                .add_bool_elems(&elem_id, 1, T::OUTPUT_PORTS.len(), true)
                .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))
        })?;

        Ok(())
    }

    pub(crate) fn read(
        &mut self,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            OUTPUT_REVERB_SEND_NAME => {
                read_f32_to_i32_values(elem_value, &self.params.reverb_send)?;
                Ok(true)
            }
            OUTPUT_REVERB_RETURN_NAME => {
                read_f32_to_i32_values(elem_value, &self.params.reverb_return)?;
                Ok(true)
            }
            OUTPUT_MASTER_MONITOR_NAME => {
                read_bool_values(elem_value, &self.params.master_monitor);
                Ok(true)
            }
            OUTPUT_MASTER_TALKBACK_NAME => {
                read_bool_values(elem_value, &self.params.master_talkback);
                Ok(true)
            }
            OUTPUT_MASTER_LISTENBACK_NAME => {
                read_bool_values(elem_value, &self.params.master_listenback);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub(crate) fn write(
        &mut self,
        sequence_number: &mut u8,
        req: &mut FwReq,
        node: &mut FwNode,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            OUTPUT_REVERB_SEND_NAME => {
                let mut params = self.params.clone();
                write_f32_from_i32_values(&mut params.reverb_send, elem_value)?;
                let res = T::update_partially(
                    req,
                    node,
                    sequence_number,
                    &mut self.params,
                    params,
                    timeout_ms,
                );
                res.map(|_| true)
            }
            OUTPUT_REVERB_RETURN_NAME => {
                let mut params = self.params.clone();
                write_f32_from_i32_values(&mut params.reverb_return, elem_value)?;
                let res = T::update_partially(
                    req,
                    node,
                    sequence_number,
                    &mut self.params,
                    params,
                    timeout_ms,
                );
                res.map(|_| true)
            }
            OUTPUT_MASTER_MONITOR_NAME => {
                let mut params = self.params.clone();
                write_bool_values(&mut params.master_monitor, elem_value);
                let res = T::update_partially(
                    req,
                    node,
                    sequence_number,
                    &mut self.params,
                    params,
                    timeout_ms,
                );
                res.map(|_| true)
            }
            OUTPUT_MASTER_TALKBACK_NAME => {
                let mut params = self.params.clone();
                write_bool_values(&mut params.master_talkback, elem_value);
                let res = T::update_partially(
                    req,
                    node,
                    sequence_number,
                    &mut self.params,
                    params,
                    timeout_ms,
                );
                res.map(|_| true)
            }
            OUTPUT_MASTER_LISTENBACK_NAME => {
                let mut params = self.params.clone();
                write_bool_values(&mut params.master_listenback, elem_value);
                let res = T::update_partially(
                    req,
                    node,
                    sequence_number,
                    &mut self.params,
                    params,
                    timeout_ms,
                );
                res.map(|_| true)
            }
            _ => Ok(false),
        }
    }

    pub(crate) fn parse_commands(&mut self, cmds: &[DspCmd]) {
        for cmd in cmds {
            let _ = T::parse_command(&mut self.params, cmd);
        }
    }
}

impl<T> CommandDspEqualizerCtlOperation<T, CommandDspOutputState> for CommandDspOutputCtl<T>
where
    T: MotuCommandDspOutputSpecification
        + MotuCommandDspEqualizerSpecification
        + MotuCommandDspDynamicsSpecification
        + MotuCommandDspParametersOperation<CommandDspOutputState>
        + MotuCommandDspUpdatableParamsOperation<CommandDspOutputState>,
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

    fn params(&self) -> &CommandDspOutputState {
        &self.params
    }

    fn params_mut(&mut self) -> &mut CommandDspOutputState {
        &mut self.params
    }
}

impl<T> CommandDspDynamicsCtlOperation<T, CommandDspOutputState> for CommandDspOutputCtl<T>
where
    T: MotuCommandDspOutputSpecification
        + MotuCommandDspEqualizerSpecification
        + MotuCommandDspDynamicsSpecification
        + MotuCommandDspParametersOperation<CommandDspOutputState>
        + MotuCommandDspUpdatableParamsOperation<CommandDspOutputState>,
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

    fn params(&self) -> &CommandDspOutputState {
        &self.params
    }

    fn params_mut(&mut self) -> &mut CommandDspOutputState {
        &mut self.params
    }
}

#[derive(Default, Debug)]
pub(crate) struct CommandDspResourceCtl {
    pub elem_id_list: Vec<ElemId>,
    state: u32,
}

const RESOURCE_USAGE_NAME: &str = "resource-usage";

impl CommandDspResourceCtl {
    pub(crate) fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, RESOURCE_USAGE_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                f32_to_i32(ResourceCmd::USAGE_MIN)?,
                f32_to_i32(ResourceCmd::USAGE_MAX)?,
                1,
                1,
                None,
                false,
            )
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))
    }

    pub(crate) fn read(
        &mut self,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            RESOURCE_USAGE_NAME => {
                read_u32_to_i32_value(elem_value, &self.state)?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub(crate) fn parse_commands(&mut self, cmds: &[DspCmd]) {
        cmds.iter().for_each(|cmd| {
            if let DspCmd::Resource(c) = cmd {
                match c {
                    // TODO: flag?
                    ResourceCmd::Usage(usage, _) => {
                        let val = f32_to_i32(*usage).unwrap();
                        self.state = val as u32;
                    }
                    _ => (),
                }
            }
        });
    }
}

#[derive(Debug)]
pub(crate) struct CommandDspMeterCtl<T>
where
    T: MotuCommandDspMeterSpecification
        + MotuCommandDspImageOperation<CommandDspMeterState, [f32; 400]>,
{
    pub elem_id_list: Vec<ElemId>,
    params: CommandDspMeterState,
    image: [f32; 400],
    _phantom: PhantomData<T>,
}

const INPUT_METER_NAME: &str = "input-meter";
const OUTPUT_METER_NAME: &str = "output-meter";

impl<T> Default for CommandDspMeterCtl<T>
where
    T: MotuCommandDspMeterSpecification
        + MotuCommandDspImageOperation<CommandDspMeterState, [f32; 400]>,
{
    fn default() -> Self {
        Self {
            elem_id_list: Default::default(),
            params: T::create_meter_state(),
            image: [0.0; 400],
            _phantom: Default::default(),
        }
    }
}

impl<T> CommandDspMeterCtl<T>
where
    T: MotuCommandDspMeterSpecification
        + MotuCommandDspImageOperation<CommandDspMeterState, [f32; 400]>,
{
    pub(crate) fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, INPUT_METER_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                f32_to_i32(T::LEVEL_MIN)?,
                f32_to_i32(T::LEVEL_MAX)?,
                1,
                T::INPUT_PORTS.len(),
                None,
                false,
            )
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, OUTPUT_METER_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                f32_to_i32(T::LEVEL_MIN)?,
                f32_to_i32(T::LEVEL_MAX)?,
                1,
                T::OUTPUT_PORTS.len(),
                None,
                false,
            )
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        Ok(())
    }

    pub(crate) fn read(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            INPUT_METER_NAME => {
                read_f32_to_i32_values(elem_value, &self.params.inputs)?;
                Ok(true)
            }
            OUTPUT_METER_NAME => {
                read_f32_to_i32_values(elem_value, &self.params.outputs)?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub(crate) fn read_dsp_meter(&mut self, unit: &mut SndMotu) -> Result<(), Error> {
        unit.read_float_meter(&mut self.image)?;
        T::parse_image(&mut self.params, &self.image);
        Ok(())
    }
}
