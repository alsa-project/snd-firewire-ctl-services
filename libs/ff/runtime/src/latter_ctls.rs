// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto
use glib::{Error, FileError};

use alsactl::{ElemId, ElemIfaceType, ElemValue, ElemValueExt, ElemValueExtManual};
use hinawa::{FwNode, SndUnit, SndUnitExt};

use alsa_ctl_tlv_codec::items::DbInterval;

use core::card_cntr::*;

use ff_protocols::{*, latter::*};

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
    output_ctl: FfLatterOutputCtl,
    mixer_ctl: FfLatterMixerCtl,
    input_ch_strip_ctl: FfLatterInputChStripCtl,
    output_ch_strip_ctl: FfLatterOutputChStripCtl,
    fx_ctl: FfLatterFxCtl,
}

impl<'a, V> FfLatterDspCtl<V>
    where V: RmeFfLatterDspSpec + AsRef<FfLatterDspState> + AsMut<FfLatterDspState>,
{
    pub fn load<U>(&mut self, unit: &SndUnit, proto: &U, timeout_ms: u32, card_cntr: &mut CardCntr)
        -> Result<(), Error>
        where U: RmeFfLatterDspProtocol<FwNode, V> + RmeFfLatterInputProtocol<FwNode, V> +
                 RmeFfLatterOutputProtocol<FwNode, V> + RmeFfLatterMixerProtocol<FwNode, V> +
                 RmeFfLatterChStripProtocol<FwNode, V, FfLatterInputChStripState> +
                 RmeFfLatterChStripProtocol<FwNode, V, FfLatterOutputChStripState> +
                 RmeFfLatterFxProtocol<FwNode, V>,
    {
        self.input_ctl.load(unit, proto, &mut self.state, timeout_ms, card_cntr)?;
        self.output_ctl.load(unit, proto, &mut self.state, timeout_ms, card_cntr)?;
        self.mixer_ctl.load(unit, proto, &mut self.state, timeout_ms, card_cntr)?;
        self.input_ch_strip_ctl.load(unit, proto, &mut self.state.as_mut().input_ch_strip, timeout_ms, card_cntr)?;
        self.output_ch_strip_ctl.load(unit, proto, &mut self.state.as_mut().output_ch_strip, timeout_ms, card_cntr)?;
        self.fx_ctl.load(unit, proto, &mut self.state, timeout_ms, card_cntr)?;
        Ok(())
    }

    pub fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        if self.input_ctl.read(&self.state, elem_id, elem_value)? {
            Ok(true)
        } else if self.output_ctl.read(&self.state, elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_ctl.read(&self.state, elem_id, elem_value)? {
            Ok(true)
        } else if self.input_ch_strip_ctl.read(&self.state.as_ref().input_ch_strip, elem_id, elem_value)? {
            Ok(true)
        } else if self.output_ch_strip_ctl.read(&self.state.as_ref().output_ch_strip, elem_id, elem_value)? {
            Ok(true)
        } else if self.fx_ctl.read(&self.state, elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn write<U>(&mut self, unit: &SndUnit, proto: &U, elem_id: &ElemId, elem_value: &ElemValue,
                    timeout_ms: u32)
        -> Result<bool, Error>
        where U: RmeFfLatterDspProtocol<FwNode, V> + RmeFfLatterInputProtocol<FwNode, V> +
                 RmeFfLatterOutputProtocol<FwNode, V> + RmeFfLatterMixerProtocol<FwNode, V> +
                 RmeFfLatterChStripProtocol<FwNode, V, FfLatterInputChStripState> +
                 RmeFfLatterChStripProtocol<FwNode, V, FfLatterOutputChStripState> +
                 RmeFfLatterFxProtocol<FwNode, V>,
    {
        if self.input_ctl.write(unit, proto, &mut self.state, elem_id, elem_value, timeout_ms)? {
            Ok(true)
        } else if self.output_ctl.write(unit, proto, &mut self.state, elem_id, elem_value, timeout_ms)? {
            Ok(true)
        } else if self.mixer_ctl.write(unit, proto, &mut self.state, elem_id, elem_value, timeout_ms)? {
            Ok(true)
        } else if self.input_ch_strip_ctl.write(unit, proto, &mut self.state.as_mut().input_ch_strip,
                                                elem_id, elem_value, timeout_ms)? {
            Ok(true)
        } else if self.output_ch_strip_ctl.write(unit, proto, &mut self.state.as_mut().output_ch_strip,
                                                 elem_id, elem_value, timeout_ms)? {
            Ok(true)
        } else if self.fx_ctl.write(unit, proto, &mut self.state, elem_id, elem_value, timeout_ms)? {
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

#[derive(Default, Debug)]
struct FfLatterOutputCtl;

impl<'a> FfLatterOutputCtl {
    const VOL_NAME: &'a str = "output:volume";
    const STEREO_BALANCE_NAME: &'a str = "output:stereo-balance";
    const STEREO_LINK_NAME: &'a str = "output:stereo-link";
    const INVERT_PHASE_NAME: &'a str = "output:invert-phase";
    const LINE_LEVEL_NAME: &'a str = "output:line-level";

    const VOL_MIN: i32 = -650;
    const VOL_MAX: i32 = 60;
    const VOL_STEP: i32 = 1;
    const VOL_TLV: DbInterval = DbInterval{min: -6500, max: 600, linear: false, mute_avail: false};

    const BALANCE_MIN: i32 = -100;
    const BALANCE_MAX: i32 = 100;
    const BALANCE_STEP: i32 = 1;

    const LINE_LEVELS: [LineOutNominalLevel;3] = [
        LineOutNominalLevel::Consumer,
        LineOutNominalLevel::Professional,
        LineOutNominalLevel::High,
    ];

    fn load<U, V>(&mut self, unit: &SndUnit, proto: &U, state: &mut V, timeout_ms: u32,
                  card_cntr: &mut CardCntr)
        -> Result<(), Error>
        where U: RmeFfLatterOutputProtocol<FwNode, V>,
              V: RmeFfLatterDspSpec + AsRef<FfLatterDspState> + AsMut<FfLatterDspState>,
    {
        state.as_mut().output.vols.iter_mut()
            .for_each(|vol| *vol = Self::VOL_MAX as i16);
        proto.init_output(&unit.get_node(), state, timeout_ms)?;

        let s = &state.as_ref().output;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::VOL_NAME, 0);
        let _ = card_cntr.add_int_elems(&elem_id, 1, Self::VOL_MIN, Self::VOL_MAX, Self::VOL_STEP,
                                        s.vols.len(), Some(&Vec::<u32>::from(&Self::VOL_TLV)),
                                        true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::STEREO_BALANCE_NAME, 0);
        let _ = card_cntr.add_int_elems(&elem_id, 1, Self::BALANCE_MIN, Self::BALANCE_MAX, Self::BALANCE_STEP,
                                        s.stereo_balance.len(), None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::STEREO_LINK_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, s.stereo_links.len(), true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::INVERT_PHASE_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, s.invert_phases.len(), true)?;

        let labels: Vec<String> = Self::LINE_LEVELS.iter()
            .map(|l| line_out_nominal_level_to_string(l))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::LINE_LEVEL_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, s.line_levels.len(), &labels, None, true)?;

        Ok(())
    }

    fn read<V>(&mut self, state: &V, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
        where V: RmeFfLatterDspSpec + AsRef<FfLatterDspState>,
    {
        match elem_id.get_name().as_str() {
            Self::VOL_NAME => {
                let vals: Vec<i32> = state.as_ref().output.vols.iter()
                    .map(|&vol| vol as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            Self::STEREO_BALANCE_NAME => {
                let vals: Vec<i32> = state.as_ref().output.stereo_balance.iter()
                    .map(|&balance| balance as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            Self::STEREO_LINK_NAME => {
                elem_value.set_bool(&state.as_ref().output.stereo_links);
                Ok(true)
            }
            Self::INVERT_PHASE_NAME => {
                elem_value.set_bool(&state.as_ref().output.invert_phases);
                Ok(true)
            }
            Self::LINE_LEVEL_NAME => {
                let vals: Vec<u32> = state.as_ref().output.line_levels.iter()
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
            _ => Ok(false),
        }
    }

    fn write<U, V>(&mut self, unit: &SndUnit, proto: &U, state: &mut V, elem_id: &ElemId,
                   elem_value: &ElemValue, timeout_ms: u32)
        -> Result<bool, Error>
        where U: RmeFfLatterOutputProtocol<FwNode, V>,
              V: RmeFfLatterDspSpec + AsRef<FfLatterDspState> + AsMut<FfLatterDspState>,
    {
        match elem_id.get_name().as_str() {
            Self::VOL_NAME => {
                let mut s = state.as_ref().output.clone();
                let mut vals = vec![0;s.vols.len()];
                elem_value.get_int(&mut vals);
                s.vols.iter_mut()
                    .zip(vals.iter())
                    .for_each(|(d, s)| *d = *s as i16);
                proto.write_output(&unit.get_node(), state, s, timeout_ms)
                    .map(|_| true)
            }
            Self::STEREO_BALANCE_NAME  => {
                let mut s = state.as_ref().output.clone();
                let mut vals = vec![0;s.stereo_balance.len()];
                elem_value.get_int(&mut vals);
                s.stereo_balance.iter_mut()
                    .zip(vals.iter())
                    .for_each(|(d, s)| *d = *s as i16);
                proto.write_output(&unit.get_node(), state, s, timeout_ms)
                    .map(|_| true)
            }
            Self::STEREO_LINK_NAME => {
                let mut s = state.as_ref().output.clone();
                elem_value.get_bool(&mut s.stereo_links);
                proto.write_output(&unit.get_node(), state, s, timeout_ms)
                    .map(|_| true)
            }
            Self::INVERT_PHASE_NAME => {
                let mut s = state.as_ref().output.clone();
                elem_value.get_bool(&mut s.invert_phases);
                proto.write_output(&unit.get_node(), state, s, timeout_ms)
                    .map(|_| true)
            }
            Self::LINE_LEVEL_NAME => {
                let mut s = state.as_ref().output.clone();
                let mut vals = vec![0;s.line_levels.len()];
                elem_value.get_enum(&mut vals);
                vals.iter()
                    .enumerate()
                    .try_for_each(|(i, &pos)| {
                        Self::LINE_LEVELS.iter()
                            .nth(pos as usize)
                            .ok_or_else(|| {
                                let msg = format!("Invalid value for index of output nominal level: {}", pos);
                                Error::new(FileError::Inval, &msg)
                            })
                            .map(|&l| s.line_levels[i] = l) 
                    })?;
                proto.write_output(&unit.get_node(), state, s, timeout_ms)
                    .map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

#[derive(Default, Debug)]
struct FfLatterMixerCtl;

impl<'a> FfLatterMixerCtl {
    const LINE_SRC_GAIN_NAME: &'a str = "mixer:line-source-gain";
    const MIC_SRC_GAIN_NAME: &'a str = "mixer:mic-source-gain";
    const SPDIF_SRC_GAIN_NAME: &'a str = "mixer:spdif-source-gain";
    const ADAT_SRC_GAIN_NAME: &'a str = "mixer:adat-source-gain";
    const STREAM_SRC_GAIN_NAME: &'a str = "mixer:stream-source-gain";

    const GAIN_MIN: i32 = 0x0000 as i32;
    const GAIN_ZERO: i32 = 0x9000 as i32;
    const GAIN_MAX: i32 = 0xa000 as i32;
    const GAIN_STEP: i32 = 1;
    const GAIN_TLV: DbInterval = DbInterval{min: -6500, max: 600, linear: false, mute_avail: false};

    fn load<U, V>(&mut self, unit: &SndUnit, proto: &U, state: &mut V, timeout_ms: u32,
                  card_cntr: &mut CardCntr)
        -> Result<(), Error>
        where U: RmeFfLatterMixerProtocol<FwNode, V>,
              V: RmeFfLatterDspSpec + AsRef<FfLatterDspState> + AsMut<FfLatterDspState>,
    {
        state.as_mut().mixer.iter_mut()
            .enumerate()
            .for_each(|(i, mixer)| {
                mixer.stream_gains.iter_mut()
                    .nth(i)
                    .map(|gain| *gain = Self::GAIN_ZERO as u16);
            });

        proto.init_mixers(&unit.get_node(), state, timeout_ms)?;

        let s = &state.as_ref().mixer;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::LINE_SRC_GAIN_NAME, 0);
        let _ = card_cntr.add_int_elems(&elem_id, s.len(), Self::GAIN_MIN, Self::GAIN_MAX, Self::GAIN_STEP,
                                        s[0].line_gains.len(), Some(&Vec::<u32>::from(&Self::GAIN_TLV)),
                                        true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::MIC_SRC_GAIN_NAME, 0);
        let _ = card_cntr.add_int_elems(&elem_id, s.len(), Self::GAIN_MIN, Self::GAIN_MAX, Self::GAIN_STEP,
                                        s[0].mic_gains.len(), Some(&Vec::<u32>::from(&Self::GAIN_TLV)),
                                        true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::SPDIF_SRC_GAIN_NAME, 0);
        let _ = card_cntr.add_int_elems(&elem_id, s.len(), Self::GAIN_MIN, Self::GAIN_MAX, Self::GAIN_STEP,
                                        s[0].spdif_gains.len(), Some(&Vec::<u32>::from(&Self::GAIN_TLV)),
                                        true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::ADAT_SRC_GAIN_NAME, 0);
        let _ = card_cntr.add_int_elems(&elem_id, s.len(), Self::GAIN_MIN, Self::GAIN_MAX, Self::GAIN_STEP,
                                        s[0].adat_gains.len(), Some(&Vec::<u32>::from(&Self::GAIN_TLV)),
                                        true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::STREAM_SRC_GAIN_NAME, 0);
        let _ = card_cntr.add_int_elems(&elem_id, s.len(), Self::GAIN_MIN, Self::GAIN_MAX, Self::GAIN_STEP,
                                        s[0].stream_gains.len(), Some(&Vec::<u32>::from(&Self::GAIN_TLV)),
                                        true)?;

        Ok(())
    }

    fn read<V>(&mut self, state: &V, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
        where V: RmeFfLatterDspSpec + AsRef<FfLatterDspState>,
    {
        match elem_id.get_name().as_str() {
            Self::LINE_SRC_GAIN_NAME => {
                let index = elem_id.get_index() as usize;
                let vals: Vec<i32> = state.as_ref().mixer[index].line_gains.iter()
                    .map(|&gain| gain as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            Self::MIC_SRC_GAIN_NAME => {
                let index = elem_id.get_index() as usize;
                let vals: Vec<i32> = state.as_ref().mixer[index].mic_gains.iter()
                    .map(|&gain| gain as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            Self::SPDIF_SRC_GAIN_NAME => {
                let index = elem_id.get_index() as usize;
                let vals: Vec<i32> = state.as_ref().mixer[index].spdif_gains.iter()
                    .map(|&gain| gain as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            Self::ADAT_SRC_GAIN_NAME => {
                let index = elem_id.get_index() as usize;
                let vals: Vec<i32> = state.as_ref().mixer[index].adat_gains.iter()
                    .map(|&gain| gain as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            Self::STREAM_SRC_GAIN_NAME => {
                let index = elem_id.get_index() as usize;
                let vals: Vec<i32> = state.as_ref().mixer[index].stream_gains.iter()
                    .map(|&gain| gain as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write<U, V>(&mut self, unit: &SndUnit, proto: &U, state: &mut V, elem_id: &ElemId,
                   new: &ElemValue, timeout_ms: u32)
        -> Result<bool, Error>
        where U: RmeFfLatterMixerProtocol<FwNode, V>,
              V: RmeFfLatterDspSpec + AsRef<FfLatterDspState> + AsMut<FfLatterDspState>,
    {
        match elem_id.get_name().as_str() {
            Self::LINE_SRC_GAIN_NAME => {
                let index = elem_id.get_index() as usize;
                let mut s = state.as_ref().mixer[index].clone();
                let mut vals = vec![0;s.line_gains.len()];
                new.get_int(&mut vals);
                s.line_gains.iter_mut()
                    .zip(vals.iter())
                    .for_each(|(d, s)| *d = *s as u16);
                proto.write_mixer(&unit.get_node(), state, index, s, timeout_ms)
                    .map(|_| true)
            }
            Self::MIC_SRC_GAIN_NAME => {
                let index = elem_id.get_index() as usize;
                let mut s = state.as_ref().mixer[index].clone();
                let mut vals = vec![0;s.mic_gains.len()];
                new.get_int(&mut vals);
                s.mic_gains.iter_mut()
                    .zip(vals.iter())
                    .for_each(|(d, s)| *d = *s as u16);
                proto.write_mixer(&unit.get_node(), state, index, s, timeout_ms)
                    .map(|_| true)
            }
            Self::SPDIF_SRC_GAIN_NAME => {
                let index = elem_id.get_index() as usize;
                let mut s = state.as_ref().mixer[index].clone();
                let mut vals = vec![0;s.spdif_gains.len()];
                new.get_int(&mut vals);
                s.spdif_gains.iter_mut()
                    .zip(vals.iter())
                    .for_each(|(d, s)| *d = *s as u16);
                proto.write_mixer(&unit.get_node(), state, index, s, timeout_ms)
                    .map(|_| true)
            }
            Self::ADAT_SRC_GAIN_NAME => {
                let index = elem_id.get_index() as usize;
                let mut s = state.as_ref().mixer[index].clone();
                let mut vals = vec![0;s.adat_gains.len()];
                new.get_int(&mut vals);
                s.adat_gains.iter_mut()
                    .zip(vals.iter())
                    .for_each(|(d, s)| *d = *s as u16);
                proto.write_mixer(&unit.get_node(), state, index, s, timeout_ms)
                    .map(|_| true)
            }
            Self::STREAM_SRC_GAIN_NAME => {
                let index = elem_id.get_index() as usize;
                let mut s = state.as_ref().mixer[index].clone();
                let mut vals = vec![0;s.stream_gains.len()];
                new.get_int(&mut vals);
                s.stream_gains.iter_mut()
                    .zip(vals.iter())
                    .for_each(|(d, s)| *d = *s as u16);
                proto.write_mixer(&unit.get_node(), state, index, s, timeout_ms)
                    .map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

fn hpf_roll_off_level_to_string(level: &FfLatterHpfRollOffLevel) -> String {
    match level {
        FfLatterHpfRollOffLevel::L6 => "6dB/octave",
        FfLatterHpfRollOffLevel::L12 => "12dB/octave",
        FfLatterHpfRollOffLevel::L18 => "18dB/octave",
        FfLatterHpfRollOffLevel::L24 => "24dB/octave",
    }.to_string()
}

fn eq_type_to_string(eq_type: &FfLatterChStripEqType) -> String {
    match eq_type {
        FfLatterChStripEqType::Peak => "Peak",
        FfLatterChStripEqType::Shelf => "Shelf",
        FfLatterChStripEqType::LowPass => "Lowpass",
    }.to_string()
}

trait RmeFfLatterChStripCtl<'a, T>
    where T: AsMut<FfLatterChStripState> + AsRef<FfLatterChStripState>,
{
    const HPF_ACTIVATE_NAME: &'a str;
    const HPF_CUT_OFF_NAME: &'a str;
    const HPF_ROLL_OFF_NAME: &'a str;

    const EQ_ACTIVATE_NAME: &'a str;
    const EQ_LOW_TYPE_NAME: &'a str;
    const EQ_LOW_GAIN_NAME: &'a str;
    const EQ_LOW_FREQ_NAME: &'a str;
    const EQ_LOW_QUALITY_NAME: &'a str;
    const EQ_MIDDLE_GAIN_NAME: &'a str;
    const EQ_MIDDLE_FREQ_NAME: &'a str;
    const EQ_MIDDLE_QUALITY_NAME: &'a str;
    const EQ_HIGH_TYPE_NAME: &'a str;
    const EQ_HIGH_GAIN_NAME: &'a str;
    const EQ_HIGH_FREQ_NAME: &'a str;
    const EQ_HIGH_QUALITY_NAME: &'a str;

    const DYN_ACTIVATE_NAME: &'a str;
    const DYN_GAIN_NAME: &'a str;
    const DYN_ATTACK_NAME: &'a str;
    const DYN_RELEASE_NAME: &'a str;
    const DYN_COMP_THRESHOLD_NAME: &'a str;
    const DYN_COMP_RATIO_NAME: &'a str;
    const DYN_EX_THRESHOLD_NAME: &'a str;
    const DYN_EX_RATIO_NAME: &'a str;

    const AUTOLEVEL_ACTIVATE_NAME: &'a str;
    const AUTOLEVEL_MAX_GAIN_NAME: &'a str;
    const AUTOLEVEL_HEAD_ROOM_NAME: &'a str;
    const AUTOLEVEL_RISE_TIME_NAME: &'a str;

    const HPF_ROLL_OFF_LEVELS: [FfLatterHpfRollOffLevel;4] = [
        FfLatterHpfRollOffLevel::L6,
        FfLatterHpfRollOffLevel::L12,
        FfLatterHpfRollOffLevel::L18,
        FfLatterHpfRollOffLevel::L24,
    ];

    const EQ_TYPES: [FfLatterChStripEqType;3] = [
        FfLatterChStripEqType::Peak,
        FfLatterChStripEqType::Shelf,
        FfLatterChStripEqType::LowPass,
    ];

    fn load<U, V>(&mut self, unit: &SndUnit, proto: &U, state: &mut T, timeout_ms: u32,
                  card_cntr: &mut CardCntr)
        -> Result<(), Error>
        where U: RmeFfLatterChStripProtocol<FwNode, V, T>,
              V: RmeFfLatterDspSpec + AsMut<FfLatterDspState> + AsRef<FfLatterDspState>,
    {
        proto.init_ch_strip(&unit.get_node(), state, timeout_ms)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::HPF_ACTIVATE_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, state.as_ref().hpf.activates.len(), true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::HPF_CUT_OFF_NAME, 0);
        let _ = card_cntr.add_int_elems(&elem_id, 1, 20, 500, 1, state.as_ref().hpf.cut_offs.len(),
                                        None, true)?;

        let labels: Vec<String> = Self::HPF_ROLL_OFF_LEVELS.iter()
            .map(|l| hpf_roll_off_level_to_string(l))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::HPF_ROLL_OFF_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, state.as_ref().hpf.roll_offs.len(), &labels,
                                         None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::EQ_ACTIVATE_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, state.as_ref().eq.activates.len(), true)?;

        let labels: Vec<String> = Self::EQ_TYPES.iter()
            .map(|t| eq_type_to_string(t))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::EQ_LOW_TYPE_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, state.as_ref().eq.low_types.len(), &labels,
                                         None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::EQ_HIGH_TYPE_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, state.as_ref().eq.high_types.len(), &labels,
                                         None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::EQ_LOW_GAIN_NAME, 0);
        let _ = card_cntr.add_int_elems(&elem_id, 1, -20, 20, 1, state.as_ref().eq.low_gains.len(),
                                        None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::EQ_MIDDLE_GAIN_NAME, 0);
        let _ = card_cntr.add_int_elems(&elem_id, 1, -20, 20, 1, state.as_ref().eq.middle_gains.len(),
                                        None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::EQ_HIGH_GAIN_NAME, 0);
        let _ = card_cntr.add_int_elems(&elem_id, 1, -20, 20, 1, state.as_ref().eq.high_gains.len(),
                                        None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::EQ_LOW_FREQ_NAME, 0);
        let _ = card_cntr.add_int_elems(&elem_id, 1, 20, 20000, 1, state.as_ref().eq.low_freqs.len(),
                                        None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::EQ_MIDDLE_FREQ_NAME, 0);
        let _ = card_cntr.add_int_elems(&elem_id, 1, 20, 20000, 1, state.as_ref().eq.middle_freqs.len(),
                                        None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::EQ_HIGH_FREQ_NAME, 0);
        let _ = card_cntr.add_int_elems(&elem_id, 1, 20, 20000, 1, state.as_ref().eq.high_freqs.len(),
                                        None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::EQ_LOW_QUALITY_NAME, 0);
        let _ = card_cntr.add_int_elems(&elem_id, 1, 7, 50, 1, state.as_ref().eq.low_qualities.len(),
                                        None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::EQ_MIDDLE_QUALITY_NAME, 0);
        let _ = card_cntr.add_int_elems(&elem_id, 1, 7, 50, 1, state.as_ref().eq.middle_qualities.len(),
                                        None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::EQ_HIGH_QUALITY_NAME, 0);
        let _ = card_cntr.add_int_elems(&elem_id, 1, 7, 50, 1, state.as_ref().eq.high_qualities.len(),
                                        None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::DYN_ACTIVATE_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, state.as_ref().dynamics.activates.len(), true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::DYN_GAIN_NAME, 0);
        let _ = card_cntr.add_int_elems(&elem_id, 1, -300, 300, 1,
                                        state.as_ref().dynamics.gains.len(), None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::DYN_ATTACK_NAME, 0);
        let _ = card_cntr.add_int_elems(&elem_id, 1, 0, 200, 1,
                                        state.as_ref().dynamics.attacks.len(), None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::DYN_RELEASE_NAME, 0);
        let _ = card_cntr.add_int_elems(&elem_id, 1, 100, 999, 1,
                                        state.as_ref().dynamics.releases.len(), None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::DYN_COMP_THRESHOLD_NAME, 0);
        let _ = card_cntr.add_int_elems(&elem_id, 1, -600, 0, 1,
                                        state.as_ref().dynamics.compressor_thresholds.len(), None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::DYN_COMP_RATIO_NAME, 0);
        let _ = card_cntr.add_int_elems(&elem_id, 1, 10, 100, 1,
                                        state.as_ref().dynamics.compressor_ratios.len(), None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::DYN_EX_THRESHOLD_NAME, 0);
        let _ = card_cntr.add_int_elems(&elem_id, 1, -990, -200, 1,
                                        state.as_ref().dynamics.expander_thresholds.len(), None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::DYN_EX_RATIO_NAME, 0);
        let _ = card_cntr.add_int_elems(&elem_id, 1, 10, 100, 1,
                                        state.as_ref().dynamics.expander_ratios.len(), None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::AUTOLEVEL_ACTIVATE_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, state.as_ref().autolevel.activates.len(), true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::AUTOLEVEL_MAX_GAIN_NAME, 0);
        let _ = card_cntr.add_int_elems(&elem_id, 1, 0, 180, 1, state.as_ref().autolevel.max_gains.len(),
                                        None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::AUTOLEVEL_HEAD_ROOM_NAME, 0);
        let _ = card_cntr.add_int_elems(&elem_id, 1, 30, 120, 1, state.as_ref().autolevel.headrooms.len(),
                                        None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::AUTOLEVEL_RISE_TIME_NAME, 0);
        let _ = card_cntr.add_int_elems(&elem_id, 1, 1, 99, 1, state.as_ref().autolevel.rise_times.len(),
                                        None, true)?;
        Ok(())
    }

    fn read(&mut self, state: &T, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        let n = elem_id.get_name();

        if n == Self::HPF_ACTIVATE_NAME {
           elem_value.set_bool(&state.as_ref().hpf.activates);
           Ok(true)
        } else if n == Self::HPF_CUT_OFF_NAME {
            let vals: Vec<i32> = state.as_ref().hpf.cut_offs.iter()
                .map(|&cut_off| cut_off as i32)
                .collect();
            elem_value.set_int(&vals);
            Ok(true)
        } else if n == Self::HPF_ROLL_OFF_NAME {
            let vals: Vec<u32> = state.as_ref().hpf.roll_offs.iter()
                .map(|roll_off| {
                    let pos = Self::HPF_ROLL_OFF_LEVELS.iter()
                        .position(|l| l.eq(roll_off))
                        .unwrap();
                    pos as u32
                })
                .collect();
            elem_value.set_enum(&vals);
            Ok(true)
        } else if n == Self::EQ_ACTIVATE_NAME {
            elem_value.set_bool(&state.as_ref().eq.activates);
            Ok(true)
        } else if n == Self::EQ_LOW_TYPE_NAME {
            let vals: Vec<u32> = state.as_ref().eq.low_types.iter()
                .map(|eq_type| {
                    let pos = Self::EQ_TYPES.iter()
                        .position(|t| t.eq(eq_type))
                        .unwrap();
                    pos as u32
                })
                .collect();
            elem_value.set_enum(&vals);
            Ok(true)
        } else if n == Self::EQ_HIGH_TYPE_NAME {
            let vals: Vec<u32> = state.as_ref().eq.high_types.iter()
                .map(|eq_type| {
                    let pos = Self::EQ_TYPES.iter()
                        .position(|t| t.eq(eq_type))
                        .unwrap();
                    pos as u32
                })
                .collect();
            elem_value.set_enum(&vals);
            Ok(true)
        } else if n == Self::EQ_LOW_GAIN_NAME {
            let vals: Vec<i32> = state.as_ref().eq.low_gains.iter()
                .map(|&gain| gain as i32)
                .collect();
            elem_value.set_int(&vals);
            Ok(true)
        } else if n == Self::EQ_MIDDLE_GAIN_NAME {
            let vals: Vec<i32> = state.as_ref().eq.middle_gains.iter()
                .map(|&gain| gain as i32)
                .collect();
            elem_value.set_int(&vals);
            Ok(true)
        } else if n == Self::EQ_HIGH_GAIN_NAME {
            let vals: Vec<i32> = state.as_ref().eq.high_gains.iter()
                .map(|&gain| gain as i32)
                .collect();
            elem_value.set_int(&vals);
            Ok(true)
        } else if n == Self::EQ_LOW_FREQ_NAME {
            let vals: Vec<i32> = state.as_ref().eq.low_freqs.iter()
                .map(|&gain| gain as i32)
                .collect();
            elem_value.set_int(&vals);
            Ok(true)
        } else if n == Self::EQ_MIDDLE_FREQ_NAME {
            let vals: Vec<i32> = state.as_ref().eq.middle_freqs.iter()
                .map(|&gain| gain as i32)
                .collect();
            elem_value.set_int(&vals);
            Ok(true)
        } else if n == Self::EQ_HIGH_FREQ_NAME {
            let vals: Vec<i32> = state.as_ref().eq.high_freqs.iter()
                .map(|&gain| gain as i32)
                .collect();
            elem_value.set_int(&vals);
            Ok(true)
        } else if n == Self::EQ_LOW_QUALITY_NAME {
            let vals: Vec<i32> = state.as_ref().eq.low_qualities.iter()
                .map(|&gain| gain as i32)
                .collect();
            elem_value.set_int(&vals);
            Ok(true)
        } else if n == Self::EQ_MIDDLE_QUALITY_NAME {
            let vals: Vec<i32> = state.as_ref().eq.middle_qualities.iter()
                .map(|&gain| gain as i32)
                .collect();
            elem_value.set_int(&vals);
            Ok(true)
        } else if n == Self::EQ_HIGH_QUALITY_NAME {
            let vals: Vec<i32> = state.as_ref().eq.high_qualities.iter()
                .map(|&gain| gain as i32)
                .collect();
            elem_value.set_int(&vals);
            Ok(true)
        } else if n == Self::DYN_ACTIVATE_NAME {
            elem_value.set_bool(&state.as_ref().dynamics.activates);
            Ok(true)
        } else if n == Self::DYN_GAIN_NAME {
            let vals: Vec<i32> = state.as_ref().dynamics.gains.iter()
                .map(|&gain| gain as i32)
                .collect();
            elem_value.set_int(&vals);
            Ok(true)
        } else if n == Self::DYN_ATTACK_NAME {
            let vals: Vec<i32> = state.as_ref().dynamics.attacks.iter()
                .map(|&attack| attack as i32)
                .collect();
            elem_value.set_int(&vals);
            Ok(true)
        } else if n == Self::DYN_RELEASE_NAME {
            let vals: Vec<i32> = state.as_ref().dynamics.releases.iter()
                .map(|&release| release as i32)
                .collect();
            elem_value.set_int(&vals);
            Ok(true)
        } else if n == Self::DYN_COMP_THRESHOLD_NAME {
            let vals: Vec<i32> = state.as_ref().dynamics.compressor_thresholds.iter()
                .map(|&th| th as i32)
                .collect();
            elem_value.set_int(&vals);
            Ok(true)
        } else if n == Self::DYN_COMP_RATIO_NAME {
            let vals: Vec<i32> = state.as_ref().dynamics.compressor_ratios.iter()
                .map(|&ratio| ratio as i32)
                .collect();
            elem_value.set_int(&vals);
            Ok(true)
        } else if n == Self::DYN_EX_THRESHOLD_NAME {
            let vals: Vec<i32> = state.as_ref().dynamics.expander_thresholds.iter()
                .map(|&th| th as i32)
                .collect();
            elem_value.set_int(&vals);
            Ok(true)
        } else if n == Self::DYN_EX_RATIO_NAME {
            let vals: Vec<i32> = state.as_ref().dynamics.expander_ratios.iter()
                .map(|&ratio| ratio as i32)
                .collect();
            elem_value.set_int(&vals);
            Ok(true)
        } else if n == Self::AUTOLEVEL_ACTIVATE_NAME {
            let vals = state.as_ref().autolevel.activates.clone();
            elem_value.set_bool(&vals);
            Ok(true)
        } else if n == Self::AUTOLEVEL_MAX_GAIN_NAME {
            let vals: Vec<i32> = state.as_ref().autolevel.max_gains.iter()
                .map(|&gain| gain as i32)
                .collect();
            elem_value.set_int(&vals);
            Ok(true)
        } else if n == Self::AUTOLEVEL_HEAD_ROOM_NAME {
            let vals: Vec<i32> = state.as_ref().autolevel.headrooms.iter()
                .map(|&gain| gain as i32)
                .collect();
            elem_value.set_int(&vals);
            Ok(true)
        } else if n == Self::AUTOLEVEL_RISE_TIME_NAME {
            let vals: Vec<i32> = state.as_ref().autolevel.rise_times.iter()
                .map(|&gain| gain as i32)
                .collect();
            elem_value.set_int(&vals);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write<U, V>(&mut self, unit: &SndUnit, proto: &U, state: &mut T, elem_id: &ElemId,
                   elem_value: &ElemValue, timeout_ms: u32)
        -> Result<bool, Error>
        where U: RmeFfLatterChStripProtocol<FwNode, V, T>,
              V: RmeFfLatterDspSpec + AsMut<FfLatterDspState> + AsRef<FfLatterDspState>,
    {
        let n = elem_id.get_name();

        if n == Self::HPF_ACTIVATE_NAME {
            let mut vals = state.as_ref().hpf.activates.clone();
            elem_value.get_bool(&mut vals);
            Self::update_hpf(unit, proto, state, timeout_ms, |s| Ok(s.activates.copy_from_slice(&vals)))
                .map(|_| true)
        } else if n == Self::HPF_CUT_OFF_NAME {
            let mut vals = vec![0;state.as_ref().hpf.cut_offs.len()];
            elem_value.get_int(&mut vals);
            let cut_offs: Vec<u16> = vals.iter()
                .map(|&val| val as u16)
                .collect();
            Self::update_hpf(unit, proto, state, timeout_ms, |s| Ok(s.cut_offs.copy_from_slice(&cut_offs)))
                .map(|_| true)
        } else if n == Self::HPF_ROLL_OFF_NAME {
            let mut vals = vec![0;state.as_ref().hpf.roll_offs.len()];
            elem_value.get_enum(&mut vals);
            let mut roll_offs = Vec::new();
            vals.iter()
                .try_for_each(|&pos| {
                    Self::HPF_ROLL_OFF_LEVELS.iter()
                        .nth(pos as usize)
                        .ok_or_else(|| {
                            let msg = format!("Invalid value for index of roll off levels: {}", pos);
                            Error::new(FileError::Inval, &msg)
                        })
                        .map(|&l| roll_offs.push(l))
                })?;
            Self::update_hpf(unit, proto, state, timeout_ms, |s| Ok(s.roll_offs.copy_from_slice(&roll_offs)))
                .map(|_| true)
        } else if n == Self::EQ_ACTIVATE_NAME {
            let mut activates = state.as_ref().eq.activates.clone();
            elem_value.get_bool(&mut activates);
            Self::update_eq(unit, proto, state, timeout_ms, |s| Ok(s.activates.copy_from_slice(&activates)))
                .map(|_| true)
        } else if n == Self::EQ_LOW_TYPE_NAME {
            let mut vals = vec![0;state.as_ref().eq.low_types.len()];
            elem_value.get_enum(&mut vals);
            let mut eq_types = Vec::new();
            vals.iter()
                .try_for_each(|&pos| {
                    Self::EQ_TYPES.iter()
                        .nth(pos as usize)
                        .ok_or_else(|| {
                            let msg = format!("Invalid value for index of equalizer types: {}", pos);
                            Error::new(FileError::Inval, &msg)
                        })
                        .map(|&t| eq_types.push(t))
                })?;
            Self::update_eq(unit, proto, state, timeout_ms, |s| Ok(s.low_types.copy_from_slice(&eq_types)))
                .map(|_| true)
        } else if n == Self::EQ_HIGH_TYPE_NAME {
            let mut vals = vec![0;state.as_ref().eq.high_types.len()];
            elem_value.get_enum(&mut vals);
            let mut eq_types = Vec::new();
            vals.iter()
                .try_for_each(|&pos| {
                    Self::EQ_TYPES.iter()
                        .nth(pos as usize)
                        .ok_or_else(|| {
                            let msg = format!("Invalid value for index of equalizer types: {}", pos);
                            Error::new(FileError::Inval, &msg)
                        })
                        .map(|&t| eq_types.push(t))
                })?;
            Self::update_eq(unit, proto, state, timeout_ms, |s| Ok(s.high_types.copy_from_slice(&eq_types)))
                .map(|_| true)
        } else if n == Self::EQ_LOW_GAIN_NAME {
            let mut vals = vec![0;state.as_ref().eq.low_gains.len()];
            elem_value.get_int(&mut vals);
            let gains: Vec<i16> = vals.iter()
                .map(|&val| val as i16)
                .collect();
            Self::update_eq(unit, proto, state, timeout_ms, |s| Ok(s.low_gains.copy_from_slice(&gains)))
                .map(|_| true)
        } else if n == Self::EQ_MIDDLE_GAIN_NAME {
            let mut vals = vec![0;state.as_ref().eq.middle_gains.len()];
            elem_value.get_int(&mut vals);
            let gains: Vec<i16> = vals.iter()
                .map(|&val| val as i16)
                .collect();
            Self::update_eq(unit, proto, state, timeout_ms, |s| Ok(s.middle_gains.copy_from_slice(&gains)))
                .map(|_| true)
        } else if n == Self::EQ_HIGH_GAIN_NAME {
            let mut vals = vec![0;state.as_ref().eq.high_gains.len()];
            elem_value.get_int(&mut vals);
            let gains: Vec<i16> = vals.iter()
                .map(|&val| val as i16)
                .collect();
            Self::update_eq(unit, proto, state, timeout_ms, |s| Ok(s.high_gains.copy_from_slice(&gains)))
                .map(|_| true)
        } else if n == Self::EQ_LOW_FREQ_NAME {
            let mut vals = vec![0;state.as_ref().eq.low_freqs.len()];
            elem_value.get_int(&mut vals);
            let freqs: Vec<u16> = vals.iter()
                .map(|&val| val as u16)
                .collect();
            Self::update_eq(unit, proto, state, timeout_ms, |s| Ok(s.low_freqs.copy_from_slice(&freqs)))
                .map(|_| true)
        } else if n == Self::EQ_MIDDLE_FREQ_NAME {
            let mut vals = vec![0;state.as_ref().eq.middle_freqs.len()];
            elem_value.get_int(&mut vals);
            let freqs: Vec<u16> = vals.iter()
                .map(|&val| val as u16)
                .collect();
            Self::update_eq(unit, proto, state, timeout_ms, |s| Ok(s.middle_freqs.copy_from_slice(&freqs)))
                .map(|_| true)
        } else if n == Self::EQ_HIGH_FREQ_NAME {
            let mut vals = vec![0;state.as_ref().eq.high_freqs.len()];
            elem_value.get_int(&mut vals);
            let freqs: Vec<u16> = vals.iter()
                .map(|&val| val as u16)
                .collect();
            Self::update_eq(unit, proto, state, timeout_ms, |s| Ok(s.high_freqs.copy_from_slice(&freqs)))
                .map(|_| true)
        } else if n == Self::EQ_LOW_QUALITY_NAME {
            let mut vals = vec![0;state.as_ref().eq.low_qualities.len()];
            elem_value.get_int(&mut vals);
            let freqs: Vec<u16> = vals.iter()
                .map(|&val| val as u16)
                .collect();
            Self::update_eq(unit, proto, state, timeout_ms, |s| Ok(s.low_qualities.copy_from_slice(&freqs)))
                .map(|_| true)
        } else if n == Self::EQ_MIDDLE_QUALITY_NAME {
            let mut vals = vec![0;state.as_ref().eq.middle_qualities.len()];
            elem_value.get_int(&mut vals);
            let freqs: Vec<u16> = vals.iter()
                .map(|&val| val as u16)
                .collect();
            Self::update_eq(unit, proto, state, timeout_ms, |s| Ok(s.middle_qualities.copy_from_slice(&freqs)))
                .map(|_| true)
        } else if n == Self::EQ_HIGH_QUALITY_NAME {
            let mut vals = vec![0;state.as_ref().eq.high_qualities.len()];
            elem_value.get_int(&mut vals);
            let freqs: Vec<u16> = vals.iter()
                .map(|&val| val as u16)
                .collect();
            Self::update_eq(unit, proto, state, timeout_ms, |s| Ok(s.high_qualities.copy_from_slice(&freqs)))
                .map(|_| true)
        } else if n == Self::DYN_ACTIVATE_NAME {
            let mut activates = state.as_ref().dynamics.activates.clone();
            elem_value.get_bool(&mut activates);
            Self::update_dynamics(unit, proto, state, timeout_ms, |s| Ok(s.activates.copy_from_slice(&activates)))
                .map(|_| true)
        } else if n == Self::DYN_GAIN_NAME {
            let mut vals = vec![0;state.as_ref().dynamics.gains.len()];
            elem_value.get_int(&mut vals);
            let gains: Vec<i16> = vals.iter()
                .map(|&val| val as i16)
                .collect();
            Self::update_dynamics(unit, proto, state, timeout_ms, |s| Ok(s.gains.copy_from_slice(&gains)))
                .map(|_| true)
        } else if n == Self::DYN_ATTACK_NAME {
            let mut vals = vec![0;state.as_ref().dynamics.attacks.len()];
            elem_value.get_int(&mut vals);
            let attacks: Vec<u16> = vals.iter()
                .map(|&val| val as u16)
                .collect();
            Self::update_dynamics(unit, proto, state, timeout_ms, |s| Ok(s.attacks.copy_from_slice(&attacks)))
                .map(|_| true)
        } else if n == Self::DYN_RELEASE_NAME {
            let mut vals = vec![0;state.as_ref().dynamics.releases.len()];
            elem_value.get_int(&mut vals);
            let release: Vec<u16> = vals.iter()
                .map(|&val| val as u16)
                .collect();
            Self::update_dynamics(unit, proto, state, timeout_ms, |s| Ok(s.releases.copy_from_slice(&release)))
                .map(|_| true)
        } else if n == Self::DYN_COMP_THRESHOLD_NAME {
            let mut vals = vec![0;state.as_ref().dynamics.compressor_thresholds.len()];
            elem_value.get_int(&mut vals);
            let ths: Vec<i16> = vals.iter()
                .map(|&val| val as i16)
                .collect();
            Self::update_dynamics(unit, proto, state, timeout_ms, |s| Ok(s.compressor_thresholds.copy_from_slice(&ths)))
                .map(|_| true)
        } else if n == Self::DYN_COMP_RATIO_NAME {
            let mut vals = vec![0;state.as_ref().dynamics.compressor_ratios.len()];
            elem_value.get_int(&mut vals);
            let ratios: Vec<u16> = vals.iter()
                .map(|&val| val as u16)
                .collect();
            Self::update_dynamics(unit, proto, state, timeout_ms, |s| Ok(s.compressor_ratios.copy_from_slice(&ratios)))
                .map(|_| true)
        } else if n == Self::DYN_EX_THRESHOLD_NAME {
            let mut vals = vec![0;state.as_ref().dynamics.expander_thresholds.len()];
            elem_value.get_int(&mut vals);
            let ths: Vec<i16> = vals.iter()
                .map(|&val| val as i16)
                .collect();
            Self::update_dynamics(unit, proto, state, timeout_ms, |s| Ok(s.expander_thresholds.copy_from_slice(&ths)))
                .map(|_| true)
        } else if n == Self::DYN_EX_RATIO_NAME {
            let mut vals = vec![0;state.as_ref().dynamics.compressor_ratios.len()];
            elem_value.get_int(&mut vals);
            let ratios: Vec<u16> = vals.iter()
                .map(|&val| val as u16)
                .collect();
            Self::update_dynamics(unit, proto, state, timeout_ms, |s| Ok(s.expander_ratios.copy_from_slice(&ratios)))
                .map(|_| true)
        } else if n == Self::AUTOLEVEL_ACTIVATE_NAME {
            let mut activates = state.as_ref().autolevel.activates.clone();
            elem_value.get_bool(&mut activates);
            Self::update_autolevel(unit, proto, state, timeout_ms, |s| Ok(s.activates.copy_from_slice(&activates)))
                .map(|_| true)
        } else if n == Self::AUTOLEVEL_MAX_GAIN_NAME {
            let mut vals = vec![0;state.as_ref().autolevel.max_gains.len()];
            elem_value.get_int(&mut vals);
            let gains: Vec<u16> = vals.iter()
                .map(|&val| val as u16)
                .collect();
            Self::update_autolevel(unit, proto, state, timeout_ms, |s| Ok(s.max_gains.copy_from_slice(&gains)))
                .map(|_| true)
        } else if n == Self::AUTOLEVEL_HEAD_ROOM_NAME {
            let mut vals = vec![0;state.as_ref().autolevel.headrooms.len()];
            elem_value.get_int(&mut vals);
            let rooms: Vec<u16> = vals.iter()
                .map(|&val| val as u16)
                .collect();
            Self::update_autolevel(unit, proto, state, timeout_ms, |s| Ok(s.headrooms.copy_from_slice(&rooms)))
                .map(|_| true)
        } else if n == Self::AUTOLEVEL_RISE_TIME_NAME {
            let mut vals = vec![0;state.as_ref().autolevel.rise_times.len()];
            elem_value.get_int(&mut vals);
            let times: Vec<u16> = vals.iter()
                .map(|&val| val as u16)
                .collect();
            Self::update_autolevel(unit, proto, state, timeout_ms, |s| Ok(s.rise_times.copy_from_slice(&times)))
                .map(|_| true)
        } else {
            Ok(false)
        }
    }

    fn update_hpf<U, V, F>(unit: &SndUnit, proto: &U, state: &mut T, timeout_ms: u32, cb: F)
        -> Result<(), Error>
        where U: RmeFfLatterChStripProtocol<FwNode, V, T>,
              V: RmeFfLatterDspSpec + AsMut<FfLatterDspState> + AsRef<FfLatterDspState>,
              F: Fn(&mut FfLatterHpfState) -> Result<(), Error>,
    {
        let mut s = state.as_ref().hpf.clone();
        cb(&mut s)?;
        proto.write_ch_strip_hpf(&unit.get_node(), state, s, timeout_ms)
    }

    fn update_eq<U, V, F>(unit: &SndUnit, proto: &U, state: &mut T, timeout_ms: u32, cb: F)
        -> Result<(), Error>
        where U: RmeFfLatterChStripProtocol<FwNode, V, T>,
              V: RmeFfLatterDspSpec + AsMut<FfLatterDspState> + AsRef<FfLatterDspState>,
              F: Fn(&mut FfLatterEqState) -> Result<(), Error>,
    {
        let mut s = state.as_ref().eq.clone();
        cb(&mut s)?;
        proto.write_ch_strip_eq(&unit.get_node(), state, s, timeout_ms)
    }

    fn update_dynamics<U, V, F>(unit: &SndUnit, proto: &U, state: &mut T, timeout_ms: u32, cb: F)
        -> Result<(), Error>
        where U: RmeFfLatterChStripProtocol<FwNode, V, T>,
              V: RmeFfLatterDspSpec + AsMut<FfLatterDspState> + AsRef<FfLatterDspState>,
              F: Fn(&mut FfLatterDynState) -> Result<(), Error>,
    {
        let mut s = state.as_ref().dynamics.clone();
        cb(&mut s)?;
        proto.write_ch_strip_dynamics(&unit.get_node(), state, s, timeout_ms)
    }

    fn update_autolevel<U, V, F>(unit: &SndUnit, proto: &U, state: &mut T, timeout_ms: u32, cb: F)
        -> Result<(), Error>
        where U: RmeFfLatterChStripProtocol<FwNode, V, T>,
              V: RmeFfLatterDspSpec + AsMut<FfLatterDspState> + AsRef<FfLatterDspState>,
              F: Fn(&mut FfLatterAutolevelState) -> Result<(), Error>,
    {
        let mut s = state.as_ref().autolevel.clone();
        cb(&mut s)?;
        proto.write_ch_strip_autolevel(&unit.get_node(), state, s, timeout_ms)
    }
}

#[derive(Default, Debug)]
struct FfLatterInputChStripCtl;

impl<'a> RmeFfLatterChStripCtl<'a, FfLatterInputChStripState> for FfLatterInputChStripCtl {
    const HPF_ACTIVATE_NAME: &'a str = "input:hpf-activate";
    const HPF_CUT_OFF_NAME: &'a str = "input:hpf-cut-off";
    const HPF_ROLL_OFF_NAME: &'a str = "input:hpf-roll-off";

    const EQ_ACTIVATE_NAME: &'a str = "input:eq-activate";
    const EQ_LOW_TYPE_NAME: &'a str = "input:eq-low-type";
    const EQ_LOW_GAIN_NAME: &'a str = "input:eq-low-gain";
    const EQ_LOW_FREQ_NAME: &'a str = "input:eq-low-freq";
    const EQ_LOW_QUALITY_NAME: &'a str = "input:eq-low-quality";
    const EQ_MIDDLE_GAIN_NAME: &'a str = "input:eq-middle-gain";
    const EQ_MIDDLE_FREQ_NAME: &'a str = "input:eq-middle-freq";
    const EQ_MIDDLE_QUALITY_NAME: &'a str = "input:eq-middle-quality";
    const EQ_HIGH_TYPE_NAME: &'a str = "input:eq-high-type";
    const EQ_HIGH_GAIN_NAME: &'a str = "input:eq-high-gain";
    const EQ_HIGH_FREQ_NAME: &'a str = "input:eq-high-freq";
    const EQ_HIGH_QUALITY_NAME: &'a str = "input:eq-high-quality";

    const DYN_ACTIVATE_NAME: &'a str = "input:dyn-activate";
    const DYN_GAIN_NAME: &'a str = "input:dyn-gain";
    const DYN_ATTACK_NAME: &'a str = "input:dyn-attack";
    const DYN_RELEASE_NAME: &'a str = "input:dyn-release";
    const DYN_COMP_THRESHOLD_NAME: &'a str = "input:dyn-compressor-threshold";
    const DYN_COMP_RATIO_NAME: &'a str = "input:dyn-compressor-ratio";
    const DYN_EX_THRESHOLD_NAME: &'a str = "input:dyn-expander-threshold";
    const DYN_EX_RATIO_NAME: &'a str =  "input:dyn-expander-ratio";

    const AUTOLEVEL_ACTIVATE_NAME: &'a str = "input:autolevel-activate";
    const AUTOLEVEL_MAX_GAIN_NAME: &'a str = "input:autolevel-max-gain";
    const AUTOLEVEL_HEAD_ROOM_NAME: &'a str = "input:autolevel-head-room";
    const AUTOLEVEL_RISE_TIME_NAME: &'a str = "input:autolevel-rise-time";
}

#[derive(Default, Debug)]
struct FfLatterOutputChStripCtl;

impl<'a> RmeFfLatterChStripCtl<'a, FfLatterOutputChStripState> for FfLatterOutputChStripCtl {
    const HPF_ACTIVATE_NAME: &'a str = "output:hpf-activate";
    const HPF_CUT_OFF_NAME: &'a str = "output:hpf-cut-off";
    const HPF_ROLL_OFF_NAME: &'a str = "output:hpf-roll-off";

    const EQ_ACTIVATE_NAME: &'a str = "output:eq-activate";
    const EQ_LOW_TYPE_NAME: &'a str = "output:eq-low-type";
    const EQ_LOW_GAIN_NAME: &'a str = "output:eq-low-gain";
    const EQ_LOW_FREQ_NAME: &'a str = "output:eq-low-freq";
    const EQ_LOW_QUALITY_NAME: &'a str = "output:eq-low-quality";
    const EQ_MIDDLE_GAIN_NAME: &'a str = "output:eq-middle-gain";
    const EQ_MIDDLE_FREQ_NAME: &'a str = "output:eq-middle-freq";
    const EQ_MIDDLE_QUALITY_NAME: &'a str = "output:eq-middle-quality";
    const EQ_HIGH_TYPE_NAME: &'a str = "output:eq-high-type";
    const EQ_HIGH_GAIN_NAME: &'a str = "output:eq-high-gain";
    const EQ_HIGH_FREQ_NAME: &'a str = "output:eq-high-freq";
    const EQ_HIGH_QUALITY_NAME: &'a str = "output:eq-high-quality";

    const DYN_ACTIVATE_NAME: &'a str = "output:dyn-activate";
    const DYN_GAIN_NAME: &'a str = "output:dyn-gain";
    const DYN_ATTACK_NAME: &'a str = "output:dyn-attack";
    const DYN_RELEASE_NAME: &'a str = "output:dyn-release";
    const DYN_COMP_THRESHOLD_NAME: &'a str = "output:dyn-compressor-threshold";
    const DYN_COMP_RATIO_NAME: &'a str = "output:dyn-compressor-ratio";
    const DYN_EX_THRESHOLD_NAME: &'a str = "output:dyn-expander-threshold";
    const DYN_EX_RATIO_NAME: &'a str =  "output:dyn-expander-ratio";

    const AUTOLEVEL_ACTIVATE_NAME: &'a str = "output:autolevel-activate";
    const AUTOLEVEL_MAX_GAIN_NAME: &'a str = "output:autolevel-max-gain";
    const AUTOLEVEL_HEAD_ROOM_NAME: &'a str = "output:autolevel-head-room";
    const AUTOLEVEL_RISE_TIME_NAME: &'a str = "output:autolevel-rise-time";
}

fn fx_reverb_type_to_string(reverb_type: &FfLatterFxReverbType) -> String {
    match reverb_type {
        FfLatterFxReverbType::SmallRoom => "Small-room",
        FfLatterFxReverbType::MediumRoom => "Medium-room",
        FfLatterFxReverbType::LargeRoom => "Large-room",
        FfLatterFxReverbType::Walls => "Walls",
        FfLatterFxReverbType::Shorty => "Shorty",
        FfLatterFxReverbType::Attack => "Attack",
        FfLatterFxReverbType::Swagger => "Swagger",
        FfLatterFxReverbType::OldSchool => "Old-school",
        FfLatterFxReverbType::Echoistic => "Echoistic",
        FfLatterFxReverbType::EightPlusNine => "8-plus-9",
        FfLatterFxReverbType::GrandWide => "Grand-wide",
        FfLatterFxReverbType::Thicker => "Thicker",
        FfLatterFxReverbType::Envelope => "Envelope",
        FfLatterFxReverbType::Gated => "Gated",
        FfLatterFxReverbType::Space => "Space",
    }.to_string()
}

fn fx_echo_type_to_string(echo_type: &FfLatterFxEchoType) -> String {
    match echo_type {
        FfLatterFxEchoType::StereoEcho => "Stereo-echo",
        FfLatterFxEchoType::StereoCross => "Stereo-cross",
        FfLatterFxEchoType::PongEcho => "Pong-echo",
    }.to_string()
}

fn fx_echo_lpf_freq_to_string(lpf_freq: &FfLatterFxEchoLpfFreq) -> String {
    match lpf_freq {
        FfLatterFxEchoLpfFreq::Off => "Off",
        FfLatterFxEchoLpfFreq::H2000 => "2kHz",
        FfLatterFxEchoLpfFreq::H4000 => "4kHz",
        FfLatterFxEchoLpfFreq::H8000 => "8kHz",
        FfLatterFxEchoLpfFreq::H12000 => "12kHz",
        FfLatterFxEchoLpfFreq::H16000 => "16kHz",
    }.to_string()
}

#[derive(Default, Debug)]
struct FfLatterFxCtl;

impl<'a> FfLatterFxCtl {
    const LINE_SRC_GAIN_NAME: &'a str = "fx:line-source-gain";
    const MIC_SRC_GAIN_NAME: &'a str = "fx:mic-source-gain";
    const SPDIF_SRC_GAIN_NAME: &'a str = "fx:spdif-source-gain";
    const ADAT_SRC_GAIN_NAME: &'a str = "fx:adat-source-gain";
    const STREAM_SRC_GAIN_NAME: &'a str = "fx:stream-source-gain";

    const LINE_OUT_VOL_NAME: &'a str = "fx:line-output-volume";
    const HP_OUT_VOL_NAME: &'a str = "fx:hp-output-volume";
    const SPDIF_OUT_VOL_NAME: &'a str = "fx:spdif-output-volume";
    const ADAT_OUT_VOL_NAME: &'a str = "fx:adat-output-volume";

    const REVERB_ACTIVATE_NAME: &'a str = "fx:reverb-activate";
    const REVERB_TYPE_NAME: &'a str = "fx:reverb-type";
    const REVERB_PRE_DELAY_NAME: &'a str = "fx:reverb-pre-delay";
    const REVERB_PRE_HPF_FREQ_NAME: &'a str = "fx:reverb-pre-hpf-freq";
    const REVERB_ROOM_SCALE_NAME: &'a str = "fx:reverb-room-scale";
    const REVERB_ATTACK_NAME: &'a str = "fx:reverb-attack";
    const REVERB_HOLD_NAME: &'a str = "fx:reverb-hold";
    const REVERB_RELEASE_NAME: &'a str = "fx:reverb-release";
    const REVERB_POST_LPF_FREQ_NAME: &'a str = "fx:reverb-post-lpf-freq";
    const REVERB_TIME_NAME: &'a str = "fx:reverb-time";
    const REVERB_DAMPING_NAME: &'a str = "fx:reverb-damping";
    const REVERB_SMOOTH_NAME: &'a str = "fx:reverb-smooth";
    const REVERB_VOL_NAME: &'a str = "fx:reverb-volume";
    const REVERB_STEREO_WIDTH_NAME: &'a str = "fx:reverb-stereo-width";

    const ECHO_ACTIVATE_NAME: &'a str = "fx:echo-activate";
    const ECHO_TYPE_NAME: &'a str = "fx:echo-type";
    const ECHO_DELAY_NAME: &'a str = "fx:echo-delay";
    const ECHO_FEEDBACK_NAME: &'a str = "fx:echo-feedback";
    const ECHO_LPF_FREQ_NAME: &'a str = "fx:echo-lpf-freq";
    const ECHO_VOL_NAME: &'a str = "fx:echo-volume";
    const ECHO_STEREO_WIDTH_NAME: &'a str = "fx:echo-stereo-width";

    const PHYS_LEVEL_MIN: i32 = -650;
    const PHYS_LEVEL_MAX: i32 = 0;
    const PHYS_LEVEL_STEP: i32 = 1;
    const PHYS_LEVEL_TLV: DbInterval = DbInterval{min: -6500, max: 000, linear: false, mute_avail: false};

    const VIRT_LEVEL_MIN: i32 = 0;
    const VIRT_LEVEL_MAX: i32 = 35676;
    const VIRT_LEVEL_STEP: i32 = 1;
    const VIRT_LEVEL_TLV: DbInterval = DbInterval{min: -6500, max: 000, linear: false, mute_avail: false};

    const REVERB_TYPES: [FfLatterFxReverbType;15] = [
        FfLatterFxReverbType::SmallRoom,
        FfLatterFxReverbType::MediumRoom,
        FfLatterFxReverbType::LargeRoom,
        FfLatterFxReverbType::Walls,
        FfLatterFxReverbType::Shorty,
        FfLatterFxReverbType::Attack,
        FfLatterFxReverbType::Swagger,
        FfLatterFxReverbType::OldSchool,
        FfLatterFxReverbType::Echoistic,
        FfLatterFxReverbType::EightPlusNine,
        FfLatterFxReverbType::GrandWide,
        FfLatterFxReverbType::Thicker,
        FfLatterFxReverbType::Envelope,
        FfLatterFxReverbType::Gated,
        FfLatterFxReverbType::Space,
    ];

    const ECHO_TYPES: [FfLatterFxEchoType;3] = [
        FfLatterFxEchoType::StereoEcho,
        FfLatterFxEchoType::StereoCross,
        FfLatterFxEchoType::PongEcho,
    ];

    const ECHO_LPF_FREQS: [FfLatterFxEchoLpfFreq;6] = [
        FfLatterFxEchoLpfFreq::Off,
        FfLatterFxEchoLpfFreq::H2000,
        FfLatterFxEchoLpfFreq::H4000,
        FfLatterFxEchoLpfFreq::H8000,
        FfLatterFxEchoLpfFreq::H12000,
        FfLatterFxEchoLpfFreq::H16000,
    ];

    fn load<U, V>(&mut self, unit: &SndUnit, proto: &U, state: &mut V, timeout_ms: u32,
                  card_cntr: &mut CardCntr)
        -> Result<(), Error>
        where U: RmeFfLatterFxProtocol<FwNode, V>,
              V: RmeFfLatterDspSpec + AsRef<FfLatterDspState> + AsMut<FfLatterDspState>,
    {
        proto.init_fx(&unit.get_node(), state, timeout_ms)?;

        let s = state.as_ref();

        [
            (Self::LINE_SRC_GAIN_NAME, &s.fx.line_input_gains),
            (Self::MIC_SRC_GAIN_NAME, &s.fx.mic_input_gains),
            (Self::SPDIF_SRC_GAIN_NAME, &s.fx.spdif_input_gains),
            (Self::ADAT_SRC_GAIN_NAME, &s.fx.adat_input_gains),
        ].iter()
            .try_for_each(|(name, gains)| {
                let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
                card_cntr.add_int_elems(&elem_id, 1,
                                        Self::PHYS_LEVEL_MIN, Self::PHYS_LEVEL_MAX, Self::PHYS_LEVEL_STEP,
                                        gains.len(), Some(&Vec::<u32>::from(&Self::PHYS_LEVEL_TLV)), true)
                    .map(|_| ())
            })?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::STREAM_SRC_GAIN_NAME, 0);
        let _ = card_cntr.add_int_elems(&elem_id, 1,
                                        Self::VIRT_LEVEL_MIN, Self::VIRT_LEVEL_MAX, Self::VIRT_LEVEL_STEP,
                                        s.fx.stream_input_gains.len(),
                                        Some(&Vec::<u32>::from(&Self::VIRT_LEVEL_TLV)), true)?;

        [
            (Self::LINE_OUT_VOL_NAME, &s.fx.line_output_vols),
            (Self::HP_OUT_VOL_NAME, &s.fx.hp_output_vols),
            (Self::SPDIF_OUT_VOL_NAME, &s.fx.spdif_output_vols),
            (Self::ADAT_OUT_VOL_NAME, &s.fx.adat_output_vols),
        ].iter()
            .try_for_each(|(name, vols)| {
                let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
                card_cntr.add_int_elems(&elem_id, 1,
                                        Self::PHYS_LEVEL_MIN, Self::PHYS_LEVEL_MAX, Self::PHYS_LEVEL_STEP,
                                        vols.len(), Some(&Vec::<u32>::from(&Self::PHYS_LEVEL_TLV)), true)
                    .map(|_| ())
            })?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::REVERB_ACTIVATE_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        let labels: Vec<String> = Self::REVERB_TYPES.iter()
            .map(|t| fx_reverb_type_to_string(t))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::REVERB_TYPE_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::REVERB_PRE_DELAY_NAME, 0);
        let _ = card_cntr.add_int_elems(&elem_id, 1, 0, 999, 1, 1, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::REVERB_PRE_HPF_FREQ_NAME, 0);
        let _ = card_cntr.add_int_elems(&elem_id, 1, 20, 500, 1, 1, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::REVERB_ROOM_SCALE_NAME, 0);
        let _ = card_cntr.add_int_elems(&elem_id, 1, 50, 300, 1, 1, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::REVERB_ATTACK_NAME, 0);
        let _ = card_cntr.add_int_elems(&elem_id, 1, 5, 400, 1, 1, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::REVERB_HOLD_NAME, 0);
        let _ = card_cntr.add_int_elems(&elem_id, 1, 5, 400, 1, 1, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::REVERB_RELEASE_NAME, 0);
        let _ = card_cntr.add_int_elems(&elem_id, 1, 5, 500, 1, 1, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::REVERB_POST_LPF_FREQ_NAME, 0);
        let _ = card_cntr.add_int_elems(&elem_id, 1, 200, 20000, 1, 1, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::REVERB_TIME_NAME, 0);
        let _ = card_cntr.add_int_elems(&elem_id, 1, 1, 49, 1, 1, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::REVERB_DAMPING_NAME, 0);
        let _ = card_cntr.add_int_elems(&elem_id, 1, 2000, 20000, 1, 1, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::REVERB_SMOOTH_NAME, 0);
        let _ = card_cntr.add_int_elems(&elem_id, 1, 0, 100, 1, 1, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::REVERB_VOL_NAME, 0);
        let _ = card_cntr.add_int_elems(&elem_id, 1, -650, 60, 1, 1, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::REVERB_STEREO_WIDTH_NAME, 0);
        let _ = card_cntr.add_int_elems(&elem_id, 1, 0, 100, 1, 1, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::ECHO_ACTIVATE_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        let labels: Vec<String> = Self::ECHO_TYPES.iter()
            .map(|t| fx_echo_type_to_string(t))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::ECHO_TYPE_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::ECHO_DELAY_NAME, 0);
        let _ = card_cntr.add_int_elems(&elem_id, 1, 0, 100, 1, 1, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::ECHO_FEEDBACK_NAME, 0);
        let _ = card_cntr.add_int_elems(&elem_id, 1, 0, 100, 1, 1, None, true)?;

        let labels: Vec<String> = Self::ECHO_LPF_FREQS.iter()
            .map(|t| fx_echo_lpf_freq_to_string(t))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::ECHO_LPF_FREQ_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::ECHO_VOL_NAME, 0);
        let _ = card_cntr.add_int_elems(&elem_id, 1, -650, 0, 1, 1, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::ECHO_STEREO_WIDTH_NAME, 0);
        let _ = card_cntr.add_int_elems(&elem_id, 1, 0, 100, 1, 1, None, true)?;

        Ok(())
    }

    fn read<V>(&mut self, state: &V, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
        where V: RmeFfLatterDspSpec + AsRef<FfLatterDspState>,
    {
        match elem_id.get_name().as_str() {
            Self::LINE_SRC_GAIN_NAME => {
                let vals: Vec<i32> = state.as_ref().fx.line_input_gains.iter()
                    .map(|&gain| gain as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            Self::MIC_SRC_GAIN_NAME => {
                let vals: Vec<i32> = state.as_ref().fx.mic_input_gains.iter()
                    .map(|&gain| gain as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            Self::SPDIF_SRC_GAIN_NAME => {
                let vals: Vec<i32> = state.as_ref().fx.spdif_input_gains.iter()
                    .map(|&gain| gain as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            Self::ADAT_SRC_GAIN_NAME => {
                let vals: Vec<i32> = state.as_ref().fx.adat_input_gains.iter()
                    .map(|&gain| gain as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            Self::STREAM_SRC_GAIN_NAME => {
                let vals: Vec<i32> = state.as_ref().fx.stream_input_gains.iter()
                    .map(|&gain| gain as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            Self::LINE_OUT_VOL_NAME => {
                let vals: Vec<i32> = state.as_ref().fx.line_output_vols.iter()
                    .map(|&vol| vol as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            Self::HP_OUT_VOL_NAME => {
                let vals: Vec<i32> = state.as_ref().fx.hp_output_vols.iter()
                    .map(|&vol| vol as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            Self::SPDIF_OUT_VOL_NAME => {
                let vals: Vec<i32> = state.as_ref().fx.spdif_output_vols.iter()
                    .map(|&vol| vol as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            Self::ADAT_OUT_VOL_NAME => {
                let vals: Vec<i32> = state.as_ref().fx.adat_output_vols.iter()
                    .map(|&vol| vol as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            Self::REVERB_ACTIVATE_NAME => {
                elem_value.set_bool(&[state.as_ref().fx.reverb.activate]);
                Ok(true)
            }
            Self::REVERB_TYPE_NAME => {
                let val = Self::REVERB_TYPES.iter()
                    .position(|t| t.eq(&state.as_ref().fx.reverb.reverb_type))
                    .unwrap();
                elem_value.set_enum(&[val as u32]);
                Ok(true)
            }
            Self::REVERB_PRE_DELAY_NAME => {
                elem_value.set_int(&[state.as_ref().fx.reverb.pre_delay as i32]);
                Ok(true)
            }
            Self::REVERB_PRE_HPF_FREQ_NAME => {
                elem_value.set_int(&[state.as_ref().fx.reverb.pre_hpf as i32]);
                Ok(true)
            }
            Self::REVERB_ROOM_SCALE_NAME => {
                elem_value.set_int(&[state.as_ref().fx.reverb.room_scale as i32]);
                Ok(true)
            }
            Self::REVERB_ATTACK_NAME => {
                elem_value.set_int(&[state.as_ref().fx.reverb.attack as i32]);
                Ok(true)
            }
            Self::REVERB_HOLD_NAME => {
                elem_value.set_int(&[state.as_ref().fx.reverb.hold as i32]);
                Ok(true)
            }
            Self::REVERB_RELEASE_NAME => {
                elem_value.set_int(&[state.as_ref().fx.reverb.release as i32]);
                Ok(true)
            }
            Self::REVERB_POST_LPF_FREQ_NAME => {
                elem_value.set_int(&[state.as_ref().fx.reverb.post_lpf as i32]);
                Ok(true)
            }
            Self::REVERB_TIME_NAME => {
                elem_value.set_int(&[state.as_ref().fx.reverb.time as i32]);
                Ok(true)
            }
            Self::REVERB_DAMPING_NAME => {
                elem_value.set_int(&[state.as_ref().fx.reverb.damping as i32]);
                Ok(true)
            }
            Self::REVERB_SMOOTH_NAME => {
                elem_value.set_int(&[state.as_ref().fx.reverb.smooth as i32]);
                Ok(true)
            }
            Self::REVERB_VOL_NAME => {
                elem_value.set_int(&[state.as_ref().fx.reverb.volume as i32]);
                Ok(true)
            }
            Self::REVERB_STEREO_WIDTH_NAME => {
                elem_value.set_int(&[state.as_ref().fx.reverb.stereo_width as i32]);
                Ok(true)
            }
            Self::ECHO_ACTIVATE_NAME => {
                elem_value.set_bool(&[state.as_ref().fx.echo.activate]);
                Ok(true)
            }
            Self::ECHO_TYPE_NAME => {
                let pos = Self::ECHO_TYPES.iter()
                    .position(|t| t.eq(&state.as_ref().fx.echo.echo_type))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            Self::ECHO_DELAY_NAME => {
                elem_value.set_int(&[state.as_ref().fx.echo.delay as i32]);
                Ok(true)
            }
            Self::ECHO_FEEDBACK_NAME => {
                elem_value.set_int(&[state.as_ref().fx.echo.feedback as i32]);
                Ok(true)
            }
            Self::ECHO_LPF_FREQ_NAME => {
                let pos = Self::ECHO_LPF_FREQS.iter()
                    .position(|f| f.eq(&state.as_ref().fx.echo.lpf))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            Self::ECHO_VOL_NAME => {
                elem_value.set_int(&[state.as_ref().fx.echo.volume as i32]);
                Ok(true)
            }
            Self::ECHO_STEREO_WIDTH_NAME => {
                elem_value.set_int(&[state.as_ref().fx.echo.stereo_width as i32]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write<U, V>(&mut self, unit: &SndUnit, proto: &U, state: &mut V, elem_id: &ElemId,
                   elem_value: &ElemValue, timeout_ms: u32)
        -> Result<bool, Error>
        where U: RmeFfLatterFxProtocol<FwNode, V>,
              V: RmeFfLatterDspSpec + AsRef<FfLatterDspState> + AsMut<FfLatterDspState>,
    {
        match elem_id.get_name().as_str() {
            Self::LINE_SRC_GAIN_NAME => {
                let mut s = state.as_ref().fx.clone();
                let mut vals = vec![0;s.line_input_gains.len()];
                elem_value.get_int(&mut vals);
                s.line_input_gains.iter_mut()
                    .zip(vals.iter())
                    .for_each(|(d, s)| *d = *s as i16);
                proto.write_fx_input_gains(&unit.get_node(), state, s, timeout_ms)
                    .map(|_| true)
            }
            Self::MIC_SRC_GAIN_NAME => {
                let mut s = state.as_ref().fx.clone();
                let mut vals = vec![0;s.mic_input_gains.len()];
                elem_value.get_int(&mut vals);
                s.mic_input_gains.iter_mut()
                    .zip(vals.iter())
                    .for_each(|(d, s)| *d = *s as i16);
                proto.write_fx_input_gains(&unit.get_node(), state, s, timeout_ms)
                    .map(|_| true)
            }
            Self::SPDIF_SRC_GAIN_NAME => {
                let mut s = state.as_ref().fx.clone();
                let mut vals = vec![0;s.spdif_input_gains.len()];
                elem_value.get_int(&mut vals);
                s.spdif_input_gains.iter_mut()
                    .zip(vals.iter())
                    .for_each(|(d, s)| *d = *s as i16);
                proto.write_fx_input_gains(&unit.get_node(), state, s, timeout_ms)
                    .map(|_| true)
            }
            Self::ADAT_SRC_GAIN_NAME => {
                let mut s = state.as_ref().fx.clone();
                let mut vals = vec![0;s.adat_input_gains.len()];
                elem_value.get_int(&mut vals);
                s.adat_input_gains.iter_mut()
                    .zip(vals.iter())
                    .for_each(|(d, s)| *d = *s as i16);
                proto.write_fx_input_gains(&unit.get_node(), state, s, timeout_ms)
                    .map(|_| true)
            }
            Self::STREAM_SRC_GAIN_NAME => {
                let mut s = state.as_ref().fx.clone();
                let mut vals = vec![0;s.stream_input_gains.len()];
                elem_value.get_int(&mut vals);
                s.stream_input_gains.iter_mut()
                    .zip(vals.iter())
                    .for_each(|(d, s)| *d = *s as u16);
                proto.write_fx_input_gains(&unit.get_node(), state, s, timeout_ms)
                    .map(|_| true)
            }
            Self::LINE_OUT_VOL_NAME => {
                let mut s = state.as_ref().fx.clone();
                let mut vals = vec![0;s.line_output_vols.len()];
                elem_value.get_int(&mut vals);
                s.line_output_vols.iter_mut()
                    .zip(vals.iter())
                    .for_each(|(d, s)| *d = *s as i16);
                proto.write_fx_output_volumes(&unit.get_node(), state, s, timeout_ms)
                    .map(|_| true)
            }
            Self::HP_OUT_VOL_NAME => {
                let mut s = state.as_ref().fx.clone();
                let mut vals = vec![0;s.hp_output_vols.len()];
                elem_value.get_int(&mut vals);
                s.hp_output_vols.iter_mut()
                    .zip(vals.iter())
                    .for_each(|(d, s)| *d = *s as i16);
                proto.write_fx_output_volumes(&unit.get_node(), state, s, timeout_ms)
                    .map(|_| true)
            }
            Self::SPDIF_OUT_VOL_NAME => {
                let mut s = state.as_ref().fx.clone();
                let mut vals = vec![0;s.spdif_output_vols.len()];
                elem_value.get_int(&mut vals);
                s.spdif_output_vols.iter_mut()
                    .zip(vals.iter())
                    .for_each(|(d, s)| *d = *s as i16);
                proto.write_fx_output_volumes(&unit.get_node(), state, s, timeout_ms)
                    .map(|_| true)
            }
            Self::ADAT_OUT_VOL_NAME => {
                let mut s = state.as_ref().fx.clone();
                let mut vals = vec![0;s.adat_output_vols.len()];
                elem_value.get_int(&mut vals);
                s.adat_output_vols.iter_mut()
                    .zip(vals.iter())
                    .for_each(|(d, s)| *d = *s as i16);
                proto.write_fx_output_volumes(&unit.get_node(), state, s, timeout_ms)
                    .map(|_| true)
            }
            Self::REVERB_ACTIVATE_NAME => {
                let mut vals = [false];
                elem_value.get_bool(&mut vals);
                Self::update_reverb(unit, proto, state, timeout_ms, |s| {
                    s.activate = vals[0];
                    Ok(())
                })
                .map(|_| true)
            }
            Self::REVERB_TYPE_NAME => {
                let mut vals = [0];
                elem_value.get_enum(&mut vals);
                let reverb_type = Self::REVERB_TYPES.iter()
                    .nth(vals[0] as usize)
                    .ok_or_else(|| {
                        let msg = format!("Invalid value for index of type of reverb effect: {}", vals[0]);
                        Error::new(FileError::Inval, &msg)
                    })
                    .map(|&t| t)?;
                Self::update_reverb(unit, proto, state, timeout_ms, |s| {
                    s.reverb_type = reverb_type;
                    Ok(())
                })
                .map(|_| true)
            }
            Self::REVERB_PRE_DELAY_NAME => {
                let mut vals = [0];
                elem_value.get_int(&mut vals);
                Self::update_reverb(unit, proto, state, timeout_ms, |s| {
                    s.pre_delay = vals[0] as u16;
                    Ok(())
                })
                .map(|_| true)
            }
            Self::REVERB_PRE_HPF_FREQ_NAME => {
                let mut vals = [0];
                elem_value.get_int(&mut vals);
                Self::update_reverb(unit, proto, state, timeout_ms, |s| {
                    s.pre_hpf = vals[0] as u16;
                    Ok(())
                })
                .map(|_| true)
            }
            Self::REVERB_ROOM_SCALE_NAME => {
                let mut vals = [0];
                elem_value.get_int(&mut vals);
                Self::update_reverb(unit, proto, state, timeout_ms, |s| {
                    s.room_scale = vals[0] as u16;
                    Ok(())
                })
                .map(|_| true)
            }
            Self::REVERB_ATTACK_NAME => {
                let mut vals = [0];
                elem_value.get_int(&mut vals);
                Self::update_reverb(unit, proto, state, timeout_ms, |s| {
                    s.attack = vals[0] as u16;
                    Ok(())
                })
                .map(|_| true)
            }
            Self::REVERB_HOLD_NAME => {
                let mut vals = [0];
                elem_value.get_int(&mut vals);
                Self::update_reverb(unit, proto, state, timeout_ms, |s| {
                    s.hold = vals[0] as u16;
                    Ok(())
                })
                .map(|_| true)
            }
            Self::REVERB_RELEASE_NAME => {
                let mut vals = [0];
                elem_value.get_int(&mut vals);
                Self::update_reverb(unit, proto, state, timeout_ms, |s| {
                    s.release = vals[0] as u16;
                    Ok(())
                })
                .map(|_| true)
            }
            Self::REVERB_POST_LPF_FREQ_NAME => {
                let mut vals = [0];
                elem_value.get_int(&mut vals);
                Self::update_reverb(unit, proto, state, timeout_ms, |s| {
                    s.post_lpf = vals[0] as u16;
                    Ok(())
                })
                .map(|_| true)
            }
            Self::REVERB_TIME_NAME => {
                let mut vals = [0];
                elem_value.get_int(&mut vals);
                Self::update_reverb(unit, proto, state, timeout_ms, |s| {
                    s.time = vals[0] as u16;
                    Ok(())
                })
                .map(|_| true)
            }
            Self::REVERB_DAMPING_NAME => {
                let mut vals = [0];
                elem_value.get_int(&mut vals);
                Self::update_reverb(unit, proto, state, timeout_ms, |s| {
                    s.damping = vals[0] as u16;
                    Ok(())
                })
                .map(|_| true)
            }
            Self::REVERB_SMOOTH_NAME => {
                let mut vals = [0];
                elem_value.get_int(&mut vals);
                Self::update_reverb(unit, proto, state, timeout_ms, |s| {
                    s.smooth = vals[0] as u16;
                    Ok(())
                })
                .map(|_| true)
            }
            Self::REVERB_VOL_NAME => {
                let mut vals = [0];
                elem_value.get_int(&mut vals);
                Self::update_reverb(unit, proto, state, timeout_ms, |s| {
                    s.volume = vals[0] as i16;
                    Ok(())
                })
                .map(|_| true)
            }
            Self::REVERB_STEREO_WIDTH_NAME => {
                let mut vals = [0];
                elem_value.get_int(&mut vals);
                Self::update_reverb(unit, proto, state, timeout_ms, |s| {
                    s.stereo_width = vals[0] as u16;
                    Ok(())
                })
                .map(|_| true)
            }
            Self::ECHO_ACTIVATE_NAME => {
                let mut vals = [false];
                elem_value.get_bool(&mut vals);
                Self::update_echo(unit, proto, state, timeout_ms, |s| {
                    s.activate = vals[0];
                    Ok(())
                })
                .map(|_| true)
            }
            Self::ECHO_TYPE_NAME => {
                let mut vals = [0];
                elem_value.get_enum(&mut vals);
                let echo_type = Self::ECHO_TYPES.iter()
                    .nth(vals[0] as usize)
                    .ok_or_else(|| {
                        let msg = format!("Invalid value for index of type of echo effect: {}", vals[0]);
                        Error::new(FileError::Inval, &msg)
                    })
                    .map(|&t| t)?;
                Self::update_echo(unit, proto, state, timeout_ms, |s| {
                    s.echo_type = echo_type;
                    Ok(())
                })
                .map(|_| true)
            }
            Self::ECHO_DELAY_NAME => {
                let mut vals = [0];
                elem_value.get_int(&mut vals);
                Self::update_echo(unit, proto, state, timeout_ms, |s| {
                    s.delay = vals[0] as u16;
                    Ok(())
                })
                .map(|_| true)
            }
            Self::ECHO_FEEDBACK_NAME => {
                let mut vals = [0];
                elem_value.get_int(&mut vals);
                Self::update_echo(unit, proto, state, timeout_ms, |s| {
                    s.feedback = vals[0] as u16;
                    Ok(())
                })
                .map(|_| true)
            }
            Self::ECHO_LPF_FREQ_NAME => {
                let mut vals = [0];
                elem_value.get_enum(&mut vals);
                let lpf = Self::ECHO_LPF_FREQS.iter()
                    .nth(vals[0] as usize)
                    .ok_or_else(|| {
                        let msg = format!("Invalid value for index of type of echo HPF frequency: {}", vals[0]);
                        Error::new(FileError::Inval, &msg)
                    })
                    .map(|&t| t)?;
                Self::update_echo(unit, proto, state, timeout_ms, |s| {
                    s.lpf = lpf;
                    Ok(())
                })
                .map(|_| true)
            }
            Self::ECHO_VOL_NAME => {
                let mut vals = [0];
                elem_value.get_int(&mut vals);
                Self::update_echo(unit, proto, state, timeout_ms, |s| {
                    s.volume = vals[0] as i16;
                    Ok(())
                })
                .map(|_| true)
            }
            Self::ECHO_STEREO_WIDTH_NAME => {
                let mut vals = [0];
                elem_value.get_int(&mut vals);
                Self::update_echo(unit, proto, state, timeout_ms, |s| {
                    s.stereo_width = vals[0] as u16;
                    Ok(())
                })
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }

    fn update_reverb<U, V, F>(unit: &SndUnit, proto: &U, state: &mut V, timeout_ms: u32, cb: F)
        -> Result<(), Error>
        where U: RmeFfLatterFxProtocol<FwNode, V>,
              V: RmeFfLatterDspSpec + AsRef<FfLatterDspState> + AsMut<FfLatterDspState>,
              F: Fn(&mut FfLatterFxReverbState) -> Result<(), Error>,
    {
        let mut s = state.as_ref().fx.reverb.clone();
        cb(&mut s)?;
        proto.write_fx_reverb(&unit.get_node(), state, &s, timeout_ms)
    }

    fn update_echo<U, V, F>(unit: &SndUnit, proto: &U, state: &mut V, timeout_ms: u32, cb: F)
        -> Result<(), Error>
        where U: RmeFfLatterFxProtocol<FwNode, V>,
              V: RmeFfLatterDspSpec + AsRef<FfLatterDspState> + AsMut<FfLatterDspState>,
              F: Fn(&mut FfLatterFxEchoState) -> Result<(), Error>,
    {
        let mut s = state.as_ref().fx.echo.clone();
        cb(&mut s)?;
        proto.write_fx_echo(&unit.get_node(), state, &s, timeout_ms)
    }
}
