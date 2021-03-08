// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto
use glib::{Error, FileError};

use alsactl::{ElemId, ElemIfaceType, ElemValue, ElemValueExt, ElemValueExtManual};
use hinawa::{FwNode, SndUnit, SndUnitExt};

use alsa_ctl_tlv_codec::items::DbInterval;

use core::card_cntr::*;

use ff_protocols::latter::*;

use super::model::*;

#[derive(Default, Debug)]
pub struct FfLatterMeterCtl<V>
    where V: RmeFfLatterMeterSpec + AsRef<FfLatterMeterState> + AsMut<FfLatterMeterState>,
{
    state: V,
    measured_elem_list: Vec<ElemId>,
}

impl<'a, V> FfLatterMeterCtl<V>
    where V: RmeFfLatterMeterSpec + AsRef<FfLatterMeterState> + AsMut<FfLatterMeterState>,
{
    const LINE_INPUT_METER: &'a str = "meter:line-input";
    const MIC_INPUT_METER: &'a str = "meter:mic-input";
    const SPDIF_INPUT_METER: &'a str = "meter:spdif-input";
    const ADAT_INPUT_METER: &'a str = "meter:adat-input";
    const STREAM_INPUT_METER: &'a str = "meter:stream-input";
    const LINE_OUTPUT_METER: &'a str = "meter:line-output";
    const HP_OUTPUT_METER: &'a str = "meter:hp-output";
    const SPDIF_OUTPUT_METER: &'a str = "meter:spdif-output";
    const ADAT_OUTPUT_METER: &'a str = "meter:adat-output";

    const LEVEL_MIN: i32 = 0x0;
    const LEVEL_MAX: i32 = 0x07fffff0;
    const LEVEL_STEP: i32 = 0x10;
    const LEVEL_TLV: DbInterval = DbInterval{min: -9003, max: 600, linear: false, mute_avail: false};

    pub fn load<U>(&mut self, unit: &SndUnit, proto: &U, timeout_ms: u32, card_cntr: &mut CardCntr)
        -> Result<(), Error>
        where U: RmeFfLatterMeterProtocol<FwNode, V>,
    {
        proto.read_meter(&unit.get_node(), &mut self.state, timeout_ms)?;

        [
            (Self::LINE_INPUT_METER, V::LINE_INPUT_COUNT),
            (Self::MIC_INPUT_METER, V::MIC_INPUT_COUNT),
            (Self::SPDIF_INPUT_METER, V::SPDIF_INPUT_COUNT),
            (Self::ADAT_INPUT_METER, V::ADAT_INPUT_COUNT),
            (Self::STREAM_INPUT_METER, V::STREAM_INPUT_COUNT),
            (Self::LINE_OUTPUT_METER, V::LINE_OUTPUT_COUNT),
            (Self::HP_OUTPUT_METER, V::HP_OUTPUT_COUNT),
            (Self::SPDIF_OUTPUT_METER, V::SPDIF_OUTPUT_COUNT),
            (Self::ADAT_OUTPUT_METER, V::ADAT_OUTPUT_COUNT),
        ].iter()
            .try_for_each(|(name, count)| {
                let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, name, 0);
                card_cntr.add_int_elems(&elem_id, 1, Self::LEVEL_MIN, Self::LEVEL_MAX, Self::LEVEL_STEP, *count,
                                        Some(&Vec::<u32>::from(&Self::LEVEL_TLV)), false)
                    .map(|mut elem_id_list| self.measured_elem_list.append(&mut elem_id_list))
            })?;

        Ok(())
    }

    pub fn get_measured_elem_list(&self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.measured_elem_list);
    }

    pub fn measure_states<U>(&mut self, unit: &SndUnit, proto: &U, timeout_ms: u32)
        -> Result<(), Error>
        where U: RmeFfLatterMeterProtocol<FwNode, V>,
    {
        proto.read_meter(&unit.get_node(), &mut self.state, timeout_ms)
    }

    pub fn read_measured_elem(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::LINE_INPUT_METER => {
                elem_value.set_int(&self.state.as_ref().line_inputs);
                Ok(true)
            }
            Self::MIC_INPUT_METER => {
                elem_value.set_int(&self.state.as_ref().mic_inputs);
                Ok(true)
            }
            Self::SPDIF_INPUT_METER => {
                elem_value.set_int(&self.state.as_ref().spdif_inputs);
                Ok(true)
            }
            Self::ADAT_INPUT_METER => {
                elem_value.set_int(&self.state.as_ref().adat_inputs);
                Ok(true)
            }
            Self::STREAM_INPUT_METER => {
                elem_value.set_int(&self.state.as_ref().stream_inputs);
                Ok(true)
            }
            Self::LINE_OUTPUT_METER => {
                elem_value.set_int(&self.state.as_ref().line_outputs);
                Ok(true)
            }
            Self::HP_OUTPUT_METER => {
                elem_value.set_int(&self.state.as_ref().hp_outputs);
                Ok(true)
            }
            Self::SPDIF_OUTPUT_METER => {
                elem_value.set_int(&self.state.as_ref().spdif_outputs);
                Ok(true)
            }
            Self::ADAT_OUTPUT_METER => {
                elem_value.set_int(&self.state.as_ref().adat_outputs);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

#[derive(Default, Debug)]
pub struct FfLatterDspCtl<V>
    where V: RmeFfLatterDspSpec + AsRef<FfLatterDspState> + AsMut<FfLatterDspState>,
{
    state: V,
    input_ctl: FfLatterInputCtl,
}

impl<'a, V> FfLatterDspCtl<V>
    where V: RmeFfLatterDspSpec + AsRef<FfLatterDspState> + AsMut<FfLatterDspState>,
{
    pub fn load<U>(&mut self, unit: &SndUnit, proto: &U, timeout_ms: u32, card_cntr: &mut CardCntr)
        -> Result<(), Error>
        where U: RmeFfLatterDspProtocol<FwNode, V> + RmeFfLatterInputProtocol<FwNode, V> +
    {
        self.input_ctl.load(unit, proto, &mut self.state, timeout_ms, card_cntr)?;
        Ok(())
    }

    pub fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        if self.input_ctl.read(&self.state, elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn write<U>(&mut self, unit: &SndUnit, proto: &U, elem_id: &ElemId, elem_value: &ElemValue,
                    timeout_ms: u32)
        -> Result<bool, Error>
        where U: RmeFfLatterDspProtocol<FwNode, V> + RmeFfLatterInputProtocol<FwNode, V> +
    {
        if self.input_ctl.write(unit, proto, &mut self.state, elem_id, elem_value, timeout_ms)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[derive(Default, Debug)]
struct FfLatterInputCtl;

impl<'a> FfLatterInputCtl {
    const STEREO_LINK_NAME: &'a str = "input:stereo-link";
    const LINE_GAIN_NAME: &'a str = "input:line-gain";
    const LINE_LEVEL_NAME: &'a str = "input:line-level";
    const MIC_POWER_NAME: &'a str = "input:mic-power";
    const MIC_INST_NAME: &'a str = "input:mic-instrument";
    const INVERT_PHASE_NAME: &'a str = "input:invert-phase";

    const GAIN_MIN: i32 = 0;
    const GAIN_MAX: i32 = 120;
    const GAIN_STEP: i32 = 1;
    const GAIN_TLV: DbInterval = DbInterval{min: 0, max: 1200, linear: false, mute_avail: false};

    const LINE_LEVELS: [LatterInNominalLevel;2] = [
        LatterInNominalLevel::Low,
        LatterInNominalLevel::Professional,
    ];

    fn load<U, V>(&mut self, unit: &SndUnit, proto: &U, state: &mut V, timeout_ms: u32,
                  card_cntr: &mut CardCntr)
        -> Result<(), Error>
        where U: RmeFfLatterInputProtocol<FwNode, V>,
              V: RmeFfLatterDspSpec + AsRef<FfLatterDspState> + AsMut<FfLatterDspState>,
    {
        proto.init_input(&unit.get_node(), state, timeout_ms)?;

        let s = &state.as_ref().input;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::STEREO_LINK_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, s.stereo_links.len(), true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::LINE_GAIN_NAME, 0);
        let _ = card_cntr.add_int_elems(&elem_id, 1, Self::GAIN_MIN, Self::GAIN_MAX, Self::GAIN_STEP,
                                        s.line_gains.len(), Some(&Vec::<u32>::from(&Self::GAIN_TLV)),
                                        true)?;

        let labels: Vec<String> = Self::LINE_LEVELS.iter()
            .map(|l| latter_line_in_nominal_level_to_string(l))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::LINE_LEVEL_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, s.line_levels.len(), &labels, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::MIC_POWER_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, s.mic_powers.len(), true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::MIC_INST_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, s.mic_insts.len(), true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::INVERT_PHASE_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, s.invert_phases.len(), true)?;

        Ok(())
    }

    fn read<V>(&mut self, state: &V, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
        where V: RmeFfLatterDspSpec + AsRef<FfLatterDspState>,
    {
        match elem_id.get_name().as_str() {
            Self::STEREO_LINK_NAME => {
                elem_value.set_bool(&state.as_ref().input.stereo_links);
                Ok(true)
            }
            Self::LINE_GAIN_NAME => {
                let vals: Vec<i32> = state.as_ref().input.line_gains.iter()
                    .map(|&gain| gain as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            Self::LINE_LEVEL_NAME => {
                let vals: Vec<u32> = state.as_ref().input.line_levels.iter()
                    .map(|level| {
                        let pos = Self::LINE_LEVELS.iter()
                            .position(|l| l.eq(level))
                            .unwrap();
                        pos as u32
                    })
                    .collect();
                elem_value.set_enum(&vals);
                Ok(true)
            }
            Self::MIC_POWER_NAME => {
                elem_value.set_bool(&state.as_ref().input.mic_powers);
                Ok(true)
            }
            Self::MIC_INST_NAME => {
                elem_value.set_bool(&state.as_ref().input.mic_insts);
                Ok(true)
            }
            Self::INVERT_PHASE_NAME => {
                elem_value.set_bool(&state.as_ref().input.invert_phases);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write<U, V>(&mut self, unit: &SndUnit, proto: &U, state: &mut V, elem_id: &ElemId,
                   elem_value: &ElemValue, timeout_ms: u32)
        -> Result<bool, Error>
        where U: RmeFfLatterInputProtocol<FwNode, V>,
              V: RmeFfLatterDspSpec + AsRef<FfLatterDspState> + AsMut<FfLatterDspState>,
    {
        match elem_id.get_name().as_str() {
            Self::STEREO_LINK_NAME => {
                let mut s = state.as_ref().input.clone();
                elem_value.get_bool(&mut s.stereo_links);
                proto.write_input(&unit.get_node(), state, s, timeout_ms)
                    .map(|_| true)
            }
            Self::LINE_GAIN_NAME => {
                let mut s = state.as_ref().input.clone();
                let mut vals = vec![0;s.line_gains.len()];
                elem_value.get_int(&mut vals);
                s.line_gains.iter_mut()
                    .zip(vals.iter())
                    .for_each(|(d, s)| *d = *s as i16);
                proto.write_input(&unit.get_node(), state, s, timeout_ms)
                    .map(|_| true)
            }
            Self::LINE_LEVEL_NAME => {
                let mut s = state.as_mut().input.clone();
                let mut vals = vec![0;s.line_levels.len()];
                elem_value.get_enum(&mut vals);
                vals.iter()
                    .enumerate()
                    .try_for_each(|(i, &pos)| {
                        Self::LINE_LEVELS.iter()
                            .nth(pos as usize)
                            .ok_or_else(|| {
                                let msg = format!("Invalid value for index of input nominal level: {}", pos);
                                Error::new(FileError::Inval, &msg)
                            })
                            .map(|&l| s.line_levels[i] = l)
                    })?;
                proto.write_input(&unit.get_node(), state, s, timeout_ms)
                    .map(|_| true)
            }
            Self::MIC_POWER_NAME => {
                let mut s = state.as_ref().input.clone();
                elem_value.get_bool(&mut s.mic_powers);
                proto.write_input(&unit.get_node(), state, s, timeout_ms)
                    .map(|_| true)
            }
            Self::MIC_INST_NAME => {
                let mut s = state.as_ref().input.clone();
                elem_value.get_bool(&mut s.mic_insts);
                proto.write_input(&unit.get_node(), state, s, timeout_ms)
                    .map(|_| true)
            }
            Self::INVERT_PHASE_NAME => {
                let mut s = state.as_ref().input.clone();
                elem_value.get_bool(&mut s.invert_phases);
                proto.write_input(&unit.get_node(), state, s, timeout_ms)
                    .map(|_| true)
            }
            _ => Ok(false),
        }
    }
}
