// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

use {
    super::*,
    alsa_ctl_tlv_codec::DbInterval,
    protocols::{latter::*, *},
    std::marker::PhantomData,
};

const LINE_INPUT_METER: &str = "meter:line-input";
const MIC_INPUT_METER: &str = "meter:mic-input";
const SPDIF_INPUT_METER: &str = "meter:spdif-input";
const ADAT_INPUT_METER: &str = "meter:adat-input";
const STREAM_INPUT_METER: &str = "meter:stream-input";
const LINE_OUTPUT_METER: &str = "meter:line-output";
const HP_OUTPUT_METER: &str = "meter:hp-output";
const SPDIF_OUTPUT_METER: &str = "meter:spdif-output";
const ADAT_OUTPUT_METER: &str = "meter:adat-output";

#[derive(Debug)]
pub struct LatterMeterCtl<T>(pub Vec<ElemId>, FfLatterMeterState, PhantomData<T>)
where
    T: RmeFfLatterMeterSpecification + RmeFfCacheableParamsOperation<FfLatterMeterState>;

impl<T: RmeFfLatterMeterSpecification> Default for LatterMeterCtl<T> {
    fn default() -> Self {
        Self(
            Default::default(),
            T::create_meter_state(),
            Default::default(),
        )
    }
}

impl<T> LatterMeterCtl<T>
where
    T: RmeFfLatterMeterSpecification + RmeFfCacheableParamsOperation<FfLatterMeterState>,
{
    const LEVEL_TLV: DbInterval = DbInterval {
        min: -9003,
        max: 600,
        linear: false,
        mute_avail: false,
    };

    pub fn cache(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let res = T::cache_wholly(req, node, &mut self.1, timeout_ms);
        debug!(params = ?self.1, ?res);
        res
    }

    pub fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        [
            (LINE_INPUT_METER, T::LINE_INPUT_COUNT),
            (MIC_INPUT_METER, T::MIC_INPUT_COUNT),
            (SPDIF_INPUT_METER, T::SPDIF_INPUT_COUNT),
            (ADAT_INPUT_METER, T::ADAT_INPUT_COUNT),
            (STREAM_INPUT_METER, T::STREAM_INPUT_COUNT),
            (LINE_OUTPUT_METER, T::LINE_OUTPUT_COUNT),
            (HP_OUTPUT_METER, T::HP_OUTPUT_COUNT),
            (SPDIF_OUTPUT_METER, T::SPDIF_OUTPUT_COUNT),
            (ADAT_OUTPUT_METER, T::ADAT_OUTPUT_COUNT),
        ]
        .iter()
        .try_for_each(|&(name, count)| {
            let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, name, 0);
            card_cntr
                .add_int_elems(
                    &elem_id,
                    1,
                    T::LEVEL_MIN,
                    T::LEVEL_MAX,
                    T::LEVEL_STEP,
                    count,
                    Some(&Vec::<u32>::from(&Self::LEVEL_TLV)),
                    false,
                )
                .map(|mut elem_id_list| self.0.append(&mut elem_id_list))
        })
    }

    pub fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            LINE_INPUT_METER => {
                elem_value.set_int(&self.1.line_inputs);
                Ok(true)
            }
            MIC_INPUT_METER => {
                elem_value.set_int(&self.1.mic_inputs);
                Ok(true)
            }
            SPDIF_INPUT_METER => {
                elem_value.set_int(&self.1.spdif_inputs);
                Ok(true)
            }
            ADAT_INPUT_METER => {
                elem_value.set_int(&self.1.adat_inputs);
                Ok(true)
            }
            STREAM_INPUT_METER => {
                elem_value.set_int(&self.1.stream_inputs);
                Ok(true)
            }
            LINE_OUTPUT_METER => {
                elem_value.set_int(&self.1.line_outputs);
                Ok(true)
            }
            HP_OUTPUT_METER => {
                elem_value.set_int(&self.1.hp_outputs);
                Ok(true)
            }
            SPDIF_OUTPUT_METER => {
                elem_value.set_int(&self.1.spdif_outputs);
                Ok(true)
            }
            ADAT_OUTPUT_METER => {
                elem_value.set_int(&self.1.adat_outputs);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

#[derive(Debug)]
pub struct LatterDspCtl<T>(FfLatterDspState, PhantomData<T>)
where
    T: RmeFfLatterDspSpecification
        + RmeFfLatterInputOperation
        + RmeFfLatterOutputOperation
        + RmeFfLatterMixerOperation
        + RmeFfLatterChStripOperation<FfLatterInputChStripState>
        + RmeFfLatterChStripOperation<FfLatterOutputChStripState>
        + RmeFfLatterFxOperation;

impl<T> Default for LatterDspCtl<T>
where
    T: RmeFfLatterDspSpecification
        + RmeFfLatterInputOperation
        + RmeFfLatterOutputOperation
        + RmeFfLatterMixerOperation
        + RmeFfLatterChStripOperation<FfLatterInputChStripState>
        + RmeFfLatterChStripOperation<FfLatterOutputChStripState>
        + RmeFfLatterFxOperation,
{
    fn default() -> Self {
        let mut state = T::create_dsp_state();
        state
            .output
            .vols
            .iter_mut()
            .for_each(|vol| *vol = T::PHYS_OUTPUT_VOL_MAX as i16);

        state.mixer.iter_mut().enumerate().for_each(|(i, mixer)| {
            mixer
                .stream_gains
                .iter_mut()
                .nth(i)
                .map(|gain| *gain = T::MIXER_INPUT_GAIN_ZERO as u16);
        });

        Self(state, Default::default())
    }
}

impl<T> LatterDspCtl<T>
where
    T: RmeFfLatterDspSpecification
        + RmeFfLatterInputOperation
        + RmeFfLatterOutputOperation
        + RmeFfLatterMixerOperation
        + RmeFfLatterChStripOperation<FfLatterInputChStripState>
        + RmeFfLatterChStripOperation<FfLatterOutputChStripState>
        + RmeFfLatterFxOperation,
{
    pub fn cache(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        self.cache_input(req, node, timeout_ms)?;
        self.cache_output(req, node, timeout_ms)?;
        self.cache_mixer(req, node, timeout_ms)?;
        FfLatterChStripCtlOperation::<T, FfLatterInputChStripState>::cache_ch_strip(
            self, req, node, timeout_ms,
        )?;
        FfLatterChStripCtlOperation::<T, FfLatterOutputChStripState>::cache_ch_strip(
            self, req, node, timeout_ms,
        )?;
        self.cache_fx(req, node, timeout_ms)?;
        Ok(())
    }

    pub fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.load_input(card_cntr)?;
        self.load_output(card_cntr)?;
        self.load_mixer(card_cntr)?;
        FfLatterChStripCtlOperation::<T, FfLatterInputChStripState>::load_ch_strip(
            self, card_cntr,
        )?;
        FfLatterChStripCtlOperation::<T, FfLatterOutputChStripState>::load_ch_strip(
            self, card_cntr,
        )?;
        self.load_fx(card_cntr)?;
        Ok(())
    }

    pub fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        if self.read_input(elem_id, elem_value)? {
            Ok(true)
        } else if self.read_output(elem_id, elem_value)? {
            Ok(true)
        } else if self.read_mixer(elem_id, elem_value)? {
            Ok(true)
        } else if FfLatterChStripCtlOperation::<T, FfLatterInputChStripState>::read_ch_strip(
            self, elem_id, elem_value,
        )? {
            Ok(true)
        } else if FfLatterChStripCtlOperation::<T, FfLatterOutputChStripState>::read_ch_strip(
            self, elem_id, elem_value,
        )? {
            Ok(true)
        } else if self.read_fx(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn write(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        if self.write_input(req, node, elem_id, elem_value, timeout_ms)? {
            Ok(true)
        } else if self.write_output(req, node, elem_id, elem_value, timeout_ms)? {
            Ok(true)
        } else if self.write_mixer(req, node, elem_id, elem_value, timeout_ms)? {
            Ok(true)
        } else if FfLatterChStripCtlOperation::<T, FfLatterInputChStripState>::write_ch_strip(
            self, req, node, elem_id, elem_value, timeout_ms,
        )? {
            Ok(true)
        } else if FfLatterChStripCtlOperation::<T, FfLatterOutputChStripState>::write_ch_strip(
            self, req, node, elem_id, elem_value, timeout_ms,
        )? {
            Ok(true)
        } else if self.write_fx(req, node, elem_id, elem_value, timeout_ms)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

const INPUT_STEREO_LINK_NAME: &str = "input:stereo-link";
const INPUT_LINE_GAIN_NAME: &str = "input:line-gain";
const INPUT_LINE_LEVEL_NAME: &str = "input:line-level";
const INPUT_MIC_POWER_NAME: &str = "input:mic-power";
const INPUT_MIC_INST_NAME: &str = "input:mic-instrument";
const INPUT_INVERT_PHASE_NAME: &str = "input:invert-phase";

impl<T: RmeFfLatterInputOperation> LatterDspCtl<T> {
    const INPUT_GAIN_TLV: DbInterval = DbInterval {
        min: 0,
        max: 1200,
        linear: false,
        mute_avail: false,
    };

    const INPUT_LINE_LEVELS: [LatterInNominalLevel; 2] = [
        LatterInNominalLevel::Low,
        LatterInNominalLevel::Professional,
    ];

    fn cache_input(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let res = T::init_input(req, node, &mut self.0, timeout_ms);
        debug!(params = ?self.0, ?res);
        res
    }

    fn load_input(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, INPUT_STEREO_LINK_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, T::PHYS_INPUT_COUNT / 2, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, INPUT_LINE_GAIN_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            1,
            T::PHYS_INPUT_GAIN_MIN,
            T::PHYS_INPUT_GAIN_MAX,
            T::PHYS_INPUT_GAIN_STEP,
            T::PHYS_INPUT_COUNT,
            Some(&Vec::<u32>::from(&Self::INPUT_GAIN_TLV)),
            true,
        )?;

        let labels: Vec<String> = Self::INPUT_LINE_LEVELS
            .iter()
            .map(|l| latter_line_in_nominal_level_to_string(l))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, INPUT_LINE_LEVEL_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, T::LINE_INPUT_COUNT, &labels, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, INPUT_MIC_POWER_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, T::MIC_INPUT_COUNT, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, INPUT_MIC_INST_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, T::MIC_INPUT_COUNT, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, INPUT_INVERT_PHASE_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, T::MIC_INPUT_COUNT, true)?;

        Ok(())
    }

    fn read_input(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            INPUT_STEREO_LINK_NAME => {
                elem_value.set_bool(&self.0.input.stereo_links);
                Ok(true)
            }
            INPUT_LINE_GAIN_NAME => {
                let vals: Vec<i32> = self
                    .0
                    .input
                    .line_gains
                    .iter()
                    .map(|&gain| gain as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            INPUT_LINE_LEVEL_NAME => {
                let vals: Vec<u32> = self
                    .0
                    .input
                    .line_levels
                    .iter()
                    .map(|level| {
                        let pos = Self::INPUT_LINE_LEVELS
                            .iter()
                            .position(|l| l.eq(level))
                            .unwrap();
                        pos as u32
                    })
                    .collect();
                elem_value.set_enum(&vals);
                Ok(true)
            }
            INPUT_MIC_POWER_NAME => {
                elem_value.set_bool(&self.0.input.mic_powers);
                Ok(true)
            }
            INPUT_MIC_INST_NAME => {
                elem_value.set_bool(&self.0.input.mic_insts);
                Ok(true)
            }
            INPUT_INVERT_PHASE_NAME => {
                elem_value.set_bool(&self.0.input.invert_phases);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write_input(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            INPUT_STEREO_LINK_NAME => {
                let mut params = self.0.input.clone();
                params
                    .stereo_links
                    .iter_mut()
                    .zip(elem_value.boolean())
                    .for_each(|(d, s)| *d = s);
                let res = T::write_input(req, node, &mut self.0, params, timeout_ms);
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            INPUT_LINE_GAIN_NAME => {
                let mut params = self.0.input.clone();
                params
                    .line_gains
                    .iter_mut()
                    .zip(elem_value.int())
                    .for_each(|(d, s)| *d = *s as i16);
                let res = T::write_input(req, node, &mut self.0, params, timeout_ms);
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            INPUT_LINE_LEVEL_NAME => {
                let mut params = self.0.input.clone();
                params
                    .line_levels
                    .iter_mut()
                    .zip(elem_value.enumerated())
                    .try_for_each(|(level, &val)| {
                        let pos = val as usize;
                        Self::INPUT_LINE_LEVELS
                            .iter()
                            .nth(pos)
                            .ok_or_else(|| {
                                let msg = format!("Invalid index of input nominal level: {}", pos);
                                Error::new(FileError::Inval, &msg)
                            })
                            .map(|&l| *level = l)
                    })?;
                let res = T::write_input(req, node, &mut self.0, params, timeout_ms);
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            INPUT_MIC_POWER_NAME => {
                let mut params = self.0.input.clone();
                params
                    .mic_powers
                    .iter_mut()
                    .zip(elem_value.boolean())
                    .for_each(|(d, s)| *d = s);
                let res = T::write_input(req, node, &mut self.0, params, timeout_ms);
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            INPUT_MIC_INST_NAME => {
                let mut params = self.0.input.clone();
                params
                    .mic_insts
                    .iter_mut()
                    .zip(elem_value.boolean())
                    .for_each(|(d, s)| *d = s);
                let res = T::write_input(req, node, &mut self.0, params, timeout_ms);
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            INPUT_INVERT_PHASE_NAME => {
                let mut params = self.0.input.clone();
                params
                    .invert_phases
                    .iter_mut()
                    .zip(elem_value.boolean())
                    .for_each(|(d, s)| *d = s);
                let res = T::write_input(req, node, &mut self.0, params, timeout_ms);
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

const VOL_NAME: &str = "output:volume";
const STEREO_BALANCE_NAME: &str = "output:stereo-balance";
const STEREO_LINK_NAME: &str = "output:stereo-link";
const INVERT_PHASE_NAME: &str = "output:invert-phase";
const LINE_LEVEL_NAME: &str = "output:line-level";

impl<T: RmeFfLatterOutputOperation> LatterDspCtl<T> {
    const VOL_TLV: DbInterval = DbInterval {
        min: -6500,
        max: 600,
        linear: false,
        mute_avail: false,
    };

    const OUTPUT_LINE_LEVELS: [LineOutNominalLevel; 3] = [
        LineOutNominalLevel::Consumer,
        LineOutNominalLevel::Professional,
        LineOutNominalLevel::High,
    ];

    fn cache_output(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let res = T::init_output(req, node, &mut self.0, timeout_ms);
        debug!(params = ?self.0, ?res);
        res
    }

    fn load_output(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, VOL_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            1,
            T::PHYS_OUTPUT_VOL_MIN,
            T::PHYS_OUTPUT_VOL_MAX,
            T::PHYS_OUTPUT_VOL_STEP,
            T::OUTPUT_COUNT,
            Some(&Vec::<u32>::from(&Self::VOL_TLV)),
            true,
        )?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, STEREO_BALANCE_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            1,
            T::PHYS_OUTPUT_BALANCE_MIN,
            T::PHYS_OUTPUT_BALANCE_MAX,
            T::PHYS_OUTPUT_BALANCE_STEP,
            T::OUTPUT_COUNT / 2,
            None,
            true,
        )?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, STEREO_LINK_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, T::OUTPUT_COUNT / 2, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, INVERT_PHASE_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, T::OUTPUT_COUNT, true)?;

        let labels: Vec<String> = Self::OUTPUT_LINE_LEVELS
            .iter()
            .map(|l| line_out_nominal_level_to_string(l))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, LINE_LEVEL_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, T::LINE_OUTPUT_COUNT, &labels, None, true)?;

        Ok(())
    }

    fn read_output(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            VOL_NAME => {
                let vals: Vec<i32> = self.0.output.vols.iter().map(|&vol| vol as i32).collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            STEREO_BALANCE_NAME => {
                let vals: Vec<i32> = self
                    .0
                    .output
                    .stereo_balance
                    .iter()
                    .map(|&balance| balance as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            STEREO_LINK_NAME => {
                elem_value.set_bool(&self.0.output.stereo_links);
                Ok(true)
            }
            INVERT_PHASE_NAME => {
                elem_value.set_bool(&self.0.output.invert_phases);
                Ok(true)
            }
            LINE_LEVEL_NAME => {
                let vals: Vec<u32> = self
                    .0
                    .output
                    .line_levels
                    .iter()
                    .map(|level| {
                        let pos = Self::OUTPUT_LINE_LEVELS
                            .iter()
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

    fn write_output(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            VOL_NAME => {
                let mut params = self.0.output.clone();
                params
                    .vols
                    .iter_mut()
                    .zip(elem_value.int())
                    .for_each(|(d, s)| *d = *s as i16);
                let res = T::write_output(req, node, &mut self.0, params, timeout_ms);
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            STEREO_BALANCE_NAME => {
                let mut params = self.0.output.clone();
                params
                    .stereo_balance
                    .iter_mut()
                    .zip(elem_value.int())
                    .for_each(|(d, s)| *d = *s as i16);
                let res = T::write_output(req, node, &mut self.0, params, timeout_ms);
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            STEREO_LINK_NAME => {
                let mut params = self.0.output.clone();
                params
                    .stereo_links
                    .iter_mut()
                    .zip(elem_value.boolean())
                    .for_each(|(d, s)| *d = s);
                let res = T::write_output(req, node, &mut self.0, params, timeout_ms);
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            INVERT_PHASE_NAME => {
                let mut params = self.0.output.clone();
                params
                    .invert_phases
                    .iter_mut()
                    .zip(elem_value.boolean())
                    .for_each(|(d, s)| *d = s);
                let res = T::write_output(req, node, &mut self.0, params, timeout_ms);
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            LINE_LEVEL_NAME => {
                let mut params = self.0.output.clone();
                params
                    .line_levels
                    .iter_mut()
                    .zip(elem_value.enumerated())
                    .try_for_each(|(level, &val)| {
                        let pos = val as usize;
                        Self::OUTPUT_LINE_LEVELS
                            .iter()
                            .nth(pos)
                            .ok_or_else(|| {
                                let msg =
                                    format!("Invalid for index of output nominal level: {}", pos);
                                Error::new(FileError::Inval, &msg)
                            })
                            .map(|&l| *level = l)
                    })?;
                let res = T::write_output(req, node, &mut self.0, params, timeout_ms);
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

const MIXER_LINE_SRC_GAIN_NAME: &str = "mixer:line-source-gain";
const MIXER_MIC_SRC_GAIN_NAME: &str = "mixer:mic-source-gain";
const MIXER_SPDIF_SRC_GAIN_NAME: &str = "mixer:spdif-source-gain";
const MIXER_ADAT_SRC_GAIN_NAME: &str = "mixer:adat-source-gain";
const MIXER_STREAM_SRC_GAIN_NAME: &str = "mixer:stream-source-gain";

impl<T: RmeFfLatterMixerOperation> LatterDspCtl<T> {
    const SRC_GAIN_TLV: DbInterval = DbInterval {
        min: -6500,
        max: 600,
        linear: false,
        mute_avail: false,
    };

    fn cache_mixer(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let res = T::init_mixers(req, node, &mut self.0, timeout_ms);
        debug!(params = ?self.0, ?res);
        res
    }

    fn load_mixer(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIXER_LINE_SRC_GAIN_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            T::OUTPUT_COUNT,
            T::MIXER_INPUT_GAIN_MIN,
            T::MIXER_INPUT_GAIN_MAX,
            T::MIXER_INPUT_GAIN_STEP,
            T::LINE_INPUT_COUNT,
            Some(&Vec::<u32>::from(&Self::SRC_GAIN_TLV)),
            true,
        )?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIXER_MIC_SRC_GAIN_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            T::OUTPUT_COUNT,
            T::MIXER_INPUT_GAIN_MIN,
            T::MIXER_INPUT_GAIN_MAX,
            T::MIXER_INPUT_GAIN_STEP,
            T::MIC_INPUT_COUNT,
            Some(&Vec::<u32>::from(&Self::SRC_GAIN_TLV)),
            true,
        )?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIXER_SPDIF_SRC_GAIN_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            T::OUTPUT_COUNT,
            T::MIXER_INPUT_GAIN_MIN,
            T::MIXER_INPUT_GAIN_MAX,
            T::MIXER_INPUT_GAIN_STEP,
            T::SPDIF_INPUT_COUNT,
            Some(&Vec::<u32>::from(&Self::SRC_GAIN_TLV)),
            true,
        )?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIXER_ADAT_SRC_GAIN_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            T::OUTPUT_COUNT,
            T::MIXER_INPUT_GAIN_MIN,
            T::MIXER_INPUT_GAIN_MAX,
            T::MIXER_INPUT_GAIN_STEP,
            T::ADAT_INPUT_COUNT,
            Some(&Vec::<u32>::from(&Self::SRC_GAIN_TLV)),
            true,
        )?;

        let elem_id =
            ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIXER_STREAM_SRC_GAIN_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            T::OUTPUT_COUNT,
            T::MIXER_INPUT_GAIN_MIN,
            T::MIXER_INPUT_GAIN_MAX,
            T::MIXER_INPUT_GAIN_STEP,
            T::STREAM_INPUT_COUNT,
            Some(&Vec::<u32>::from(&Self::SRC_GAIN_TLV)),
            true,
        )?;

        Ok(())
    }

    fn read_mixer(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            MIXER_LINE_SRC_GAIN_NAME => {
                let index = elem_id.index() as usize;
                let vals: Vec<i32> = self.0.mixer[index]
                    .line_gains
                    .iter()
                    .map(|&gain| gain as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            MIXER_MIC_SRC_GAIN_NAME => {
                let index = elem_id.index() as usize;
                let vals: Vec<i32> = self.0.mixer[index]
                    .mic_gains
                    .iter()
                    .map(|&gain| gain as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            MIXER_SPDIF_SRC_GAIN_NAME => {
                let index = elem_id.index() as usize;
                let vals: Vec<i32> = self.0.mixer[index]
                    .spdif_gains
                    .iter()
                    .map(|&gain| gain as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            MIXER_ADAT_SRC_GAIN_NAME => {
                let index = elem_id.index() as usize;
                let vals: Vec<i32> = self.0.mixer[index]
                    .adat_gains
                    .iter()
                    .map(|&gain| gain as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            MIXER_STREAM_SRC_GAIN_NAME => {
                let index = elem_id.index() as usize;
                let vals: Vec<i32> = self.0.mixer[index]
                    .stream_gains
                    .iter()
                    .map(|&gain| gain as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write_mixer(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            MIXER_LINE_SRC_GAIN_NAME => {
                let index = elem_id.index() as usize;
                let mut params = self.0.mixer[index].clone();
                params
                    .line_gains
                    .iter_mut()
                    .zip(elem_value.int())
                    .for_each(|(d, s)| *d = *s as u16);
                let res = T::write_mixer(req, node, &mut self.0, index, params, timeout_ms);
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            MIXER_MIC_SRC_GAIN_NAME => {
                let index = elem_id.index() as usize;
                let mut params = self.0.mixer[index].clone();
                params
                    .mic_gains
                    .iter_mut()
                    .zip(elem_value.int())
                    .for_each(|(d, s)| *d = *s as u16);
                let res = T::write_mixer(req, node, &mut self.0, index, params, timeout_ms);
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            MIXER_SPDIF_SRC_GAIN_NAME => {
                let index = elem_id.index() as usize;
                let mut params = self.0.mixer[index].clone();
                params
                    .spdif_gains
                    .iter_mut()
                    .zip(elem_value.int())
                    .for_each(|(d, s)| *d = *s as u16);
                let res = T::write_mixer(req, node, &mut self.0, index, params, timeout_ms);
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            MIXER_ADAT_SRC_GAIN_NAME => {
                let index = elem_id.index() as usize;
                let mut params = self.0.mixer[index].clone();
                params
                    .adat_gains
                    .iter_mut()
                    .zip(elem_value.int())
                    .for_each(|(d, s)| *d = *s as u16);
                let res = T::write_mixer(req, node, &mut self.0, index, params, timeout_ms);
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            MIXER_STREAM_SRC_GAIN_NAME => {
                let index = elem_id.index() as usize;
                let mut params = self.0.mixer[index].clone();
                params
                    .stream_gains
                    .iter_mut()
                    .zip(elem_value.int())
                    .for_each(|(d, s)| *d = *s as u16);
                let res = T::write_mixer(req, node, &mut self.0, index, params, timeout_ms);
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
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
    }
    .to_string()
}

fn eq_type_to_string(eq_type: &FfLatterChStripEqType) -> String {
    match eq_type {
        FfLatterChStripEqType::Peak => "Peak",
        FfLatterChStripEqType::Shelf => "Shelf",
        FfLatterChStripEqType::LowPass => "Lowpass",
    }
    .to_string()
}

pub trait FfLatterChStripCtlOperation<T: RmeFfLatterChStripOperation<U>, U> {
    const HPF_ACTIVATE_NAME: &'static str;
    const HPF_CUT_OFF_NAME: &'static str;
    const HPF_ROLL_OFF_NAME: &'static str;

    const EQ_ACTIVATE_NAME: &'static str;
    const EQ_LOW_TYPE_NAME: &'static str;
    const EQ_LOW_GAIN_NAME: &'static str;
    const EQ_LOW_FREQ_NAME: &'static str;
    const EQ_LOW_QUALITY_NAME: &'static str;
    const EQ_MIDDLE_GAIN_NAME: &'static str;
    const EQ_MIDDLE_FREQ_NAME: &'static str;
    const EQ_MIDDLE_QUALITY_NAME: &'static str;
    const EQ_HIGH_TYPE_NAME: &'static str;
    const EQ_HIGH_GAIN_NAME: &'static str;
    const EQ_HIGH_FREQ_NAME: &'static str;
    const EQ_HIGH_QUALITY_NAME: &'static str;

    const DYN_ACTIVATE_NAME: &'static str;
    const DYN_GAIN_NAME: &'static str;
    const DYN_ATTACK_NAME: &'static str;
    const DYN_RELEASE_NAME: &'static str;
    const DYN_COMP_THRESHOLD_NAME: &'static str;
    const DYN_COMP_RATIO_NAME: &'static str;
    const DYN_EX_THRESHOLD_NAME: &'static str;
    const DYN_EX_RATIO_NAME: &'static str;

    const AUTOLEVEL_ACTIVATE_NAME: &'static str;
    const AUTOLEVEL_MAX_GAIN_NAME: &'static str;
    const AUTOLEVEL_HEAD_ROOM_NAME: &'static str;
    const AUTOLEVEL_RISE_TIME_NAME: &'static str;

    const HPF_ROLL_OFF_LEVELS: [FfLatterHpfRollOffLevel; 4] = [
        FfLatterHpfRollOffLevel::L6,
        FfLatterHpfRollOffLevel::L12,
        FfLatterHpfRollOffLevel::L18,
        FfLatterHpfRollOffLevel::L24,
    ];

    const EQ_TYPES: [FfLatterChStripEqType; 3] = [
        FfLatterChStripEqType::Peak,
        FfLatterChStripEqType::Shelf,
        FfLatterChStripEqType::LowPass,
    ];

    fn state(&self) -> &FfLatterDspState;
    fn state_mut(&mut self) -> &mut FfLatterDspState;

    fn cache_ch_strip(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let res = T::init_ch_strip(req, node, self.state_mut(), timeout_ms);
        debug!(params = ?T::ch_strip(self.state()), ?res);
        res
    }

    fn load_ch_strip(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::HPF_ACTIVATE_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, T::CH_COUNT, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::HPF_CUT_OFF_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            1,
            T::HPF_CUT_OFF_MIN,
            T::HPF_CUT_OFF_MAX,
            T::HPF_CUT_OFF_STEP,
            T::CH_COUNT,
            None,
            true,
        )?;

        let labels: Vec<String> = Self::HPF_ROLL_OFF_LEVELS
            .iter()
            .map(|l| hpf_roll_off_level_to_string(l))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::HPF_ROLL_OFF_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, T::CH_COUNT, &labels, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::EQ_ACTIVATE_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, T::CH_COUNT, true)?;

        let labels: Vec<String> = Self::EQ_TYPES
            .iter()
            .map(|t| eq_type_to_string(t))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::EQ_LOW_TYPE_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, T::CH_COUNT, &labels, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::EQ_HIGH_TYPE_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, T::CH_COUNT, &labels, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::EQ_LOW_GAIN_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            1,
            T::EQ_GAIN_MIN,
            T::EQ_GAIN_MAX,
            T::EQ_GAIN_STEP,
            T::CH_COUNT,
            None,
            true,
        )?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::EQ_MIDDLE_GAIN_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            1,
            T::EQ_GAIN_MIN,
            T::EQ_GAIN_MAX,
            T::EQ_GAIN_STEP,
            T::CH_COUNT,
            None,
            true,
        )?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::EQ_HIGH_GAIN_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            1,
            T::EQ_GAIN_MIN,
            T::EQ_GAIN_MAX,
            T::EQ_GAIN_STEP,
            T::CH_COUNT,
            None,
            true,
        )?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::EQ_LOW_FREQ_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            1,
            T::EQ_FREQ_MIN,
            T::EQ_FREQ_MAX,
            T::EQ_FREQ_STEP,
            T::CH_COUNT,
            None,
            true,
        )?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::EQ_MIDDLE_FREQ_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            1,
            T::EQ_FREQ_MIN,
            T::EQ_FREQ_MAX,
            T::EQ_FREQ_STEP,
            T::CH_COUNT,
            None,
            true,
        )?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::EQ_HIGH_FREQ_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            1,
            T::EQ_FREQ_MIN,
            T::EQ_FREQ_MAX,
            T::EQ_FREQ_STEP,
            T::CH_COUNT,
            None,
            true,
        )?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::EQ_LOW_QUALITY_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            1,
            T::EQ_QUALITY_MIN,
            T::EQ_QUALITY_MAX,
            T::EQ_QUALITY_STEP,
            T::CH_COUNT,
            None,
            true,
        )?;

        let elem_id =
            ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::EQ_MIDDLE_QUALITY_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            1,
            T::EQ_QUALITY_MIN,
            T::EQ_QUALITY_MAX,
            T::EQ_QUALITY_STEP,
            T::CH_COUNT,
            None,
            true,
        )?;

        let elem_id =
            ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::EQ_HIGH_QUALITY_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            1,
            T::EQ_QUALITY_MIN,
            T::EQ_QUALITY_MAX,
            T::EQ_QUALITY_STEP,
            T::CH_COUNT,
            None,
            true,
        )?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::DYN_ACTIVATE_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, T::CH_COUNT, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::DYN_GAIN_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            1,
            T::DYN_GAIN_MIN,
            T::DYN_GAIN_MAX,
            T::DYN_GAIN_STEP,
            T::CH_COUNT,
            None,
            true,
        )?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::DYN_ATTACK_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            1,
            T::DYN_ATTACK_MIN,
            T::DYN_ATTACK_MAX,
            T::DYN_ATTACK_STEP,
            T::CH_COUNT,
            None,
            true,
        )?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::DYN_RELEASE_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            1,
            T::DYN_RELEASE_MIN,
            T::DYN_RELEASE_MAX,
            T::DYN_RELEASE_STEP,
            T::CH_COUNT,
            None,
            true,
        )?;

        let elem_id =
            ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::DYN_COMP_THRESHOLD_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            1,
            T::DYN_COMP_THRESHOLD_MIN,
            T::DYN_COMP_THRESHOLD_MAX,
            T::DYN_COMP_THRESHOLD_STEP,
            T::CH_COUNT,
            None,
            true,
        )?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::DYN_COMP_RATIO_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            1,
            T::DYN_RATIO_MIN,
            T::DYN_RATIO_MAX,
            T::DYN_RATIO_STEP,
            T::CH_COUNT,
            None,
            true,
        )?;

        let elem_id =
            ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::DYN_EX_THRESHOLD_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            1,
            T::DYN_EX_THRESHOLD_MIN,
            T::DYN_EX_THRESHOLD_MAX,
            T::DYN_EX_THRESHOLD_STEP,
            T::CH_COUNT,
            None,
            true,
        )?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::DYN_EX_RATIO_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            1,
            T::DYN_RATIO_MIN,
            T::DYN_RATIO_MAX,
            T::DYN_RATIO_STEP,
            T::CH_COUNT,
            None,
            true,
        )?;

        let elem_id =
            ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::AUTOLEVEL_ACTIVATE_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, T::CH_COUNT, true)?;

        let elem_id =
            ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::AUTOLEVEL_MAX_GAIN_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            1,
            T::AUTOLEVEL_MAX_GAIN_MIN,
            T::AUTOLEVEL_MAX_GAIN_MAX,
            T::AUTOLEVEL_MAX_GAIN_STEP,
            T::CH_COUNT,
            None,
            true,
        )?;

        let elem_id = ElemId::new_by_name(
            ElemIfaceType::Mixer,
            0,
            0,
            Self::AUTOLEVEL_HEAD_ROOM_NAME,
            0,
        );
        let _ = card_cntr.add_int_elems(
            &elem_id,
            1,
            T::AUTOLEVEL_HEAD_ROOM_MIN,
            T::AUTOLEVEL_HEAD_ROOM_MAX,
            T::AUTOLEVEL_HEAD_ROOM_STEP,
            T::CH_COUNT,
            None,
            true,
        )?;

        let elem_id = ElemId::new_by_name(
            ElemIfaceType::Mixer,
            0,
            0,
            Self::AUTOLEVEL_RISE_TIME_NAME,
            0,
        );
        let _ = card_cntr.add_int_elems(
            &elem_id,
            1,
            T::AUTOLEVEL_RISE_TIME_MIN,
            T::AUTOLEVEL_RISE_TIME_MAX,
            T::AUTOLEVEL_RISE_TIME_STEP,
            T::CH_COUNT,
            None,
            true,
        )?;

        Ok(())
    }

    fn read_ch_strip(
        &mut self,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        let n = elem_id.name();

        if n == Self::HPF_ACTIVATE_NAME {
            elem_value.set_bool(&T::ch_strip(self.state()).hpf.activates);
            Ok(true)
        } else if n == Self::HPF_CUT_OFF_NAME {
            let vals: Vec<i32> = T::ch_strip(self.state())
                .hpf
                .cut_offs
                .iter()
                .map(|&cut_off| cut_off as i32)
                .collect();
            elem_value.set_int(&vals);
            Ok(true)
        } else if n == Self::HPF_ROLL_OFF_NAME {
            let vals: Vec<u32> = T::ch_strip(self.state())
                .hpf
                .roll_offs
                .iter()
                .map(|roll_off| {
                    let pos = Self::HPF_ROLL_OFF_LEVELS
                        .iter()
                        .position(|l| l.eq(roll_off))
                        .unwrap();
                    pos as u32
                })
                .collect();
            elem_value.set_enum(&vals);
            Ok(true)
        } else if n == Self::EQ_ACTIVATE_NAME {
            elem_value.set_bool(&T::ch_strip(self.state()).eq.activates);
            Ok(true)
        } else if n == Self::EQ_LOW_TYPE_NAME {
            let vals: Vec<u32> = T::ch_strip(self.state())
                .eq
                .low_types
                .iter()
                .map(|eq_type| {
                    let pos = Self::EQ_TYPES.iter().position(|t| t.eq(eq_type)).unwrap();
                    pos as u32
                })
                .collect();
            elem_value.set_enum(&vals);
            Ok(true)
        } else if n == Self::EQ_HIGH_TYPE_NAME {
            let vals: Vec<u32> = T::ch_strip(self.state())
                .eq
                .high_types
                .iter()
                .map(|eq_type| {
                    let pos = Self::EQ_TYPES.iter().position(|t| t.eq(eq_type)).unwrap();
                    pos as u32
                })
                .collect();
            elem_value.set_enum(&vals);
            Ok(true)
        } else if n == Self::EQ_LOW_GAIN_NAME {
            let vals: Vec<i32> = T::ch_strip(self.state())
                .eq
                .low_gains
                .iter()
                .map(|&gain| gain as i32)
                .collect();
            elem_value.set_int(&vals);
            Ok(true)
        } else if n == Self::EQ_MIDDLE_GAIN_NAME {
            let vals: Vec<i32> = T::ch_strip(self.state())
                .eq
                .middle_gains
                .iter()
                .map(|&gain| gain as i32)
                .collect();
            elem_value.set_int(&vals);
            Ok(true)
        } else if n == Self::EQ_HIGH_GAIN_NAME {
            let vals: Vec<i32> = T::ch_strip(self.state())
                .eq
                .high_gains
                .iter()
                .map(|&gain| gain as i32)
                .collect();
            elem_value.set_int(&vals);
            Ok(true)
        } else if n == Self::EQ_LOW_FREQ_NAME {
            let vals: Vec<i32> = T::ch_strip(self.state())
                .eq
                .low_freqs
                .iter()
                .map(|&gain| gain as i32)
                .collect();
            elem_value.set_int(&vals);
            Ok(true)
        } else if n == Self::EQ_MIDDLE_FREQ_NAME {
            let vals: Vec<i32> = T::ch_strip(self.state())
                .eq
                .middle_freqs
                .iter()
                .map(|&gain| gain as i32)
                .collect();
            elem_value.set_int(&vals);
            Ok(true)
        } else if n == Self::EQ_HIGH_FREQ_NAME {
            let vals: Vec<i32> = T::ch_strip(self.state())
                .eq
                .high_freqs
                .iter()
                .map(|&gain| gain as i32)
                .collect();
            elem_value.set_int(&vals);
            Ok(true)
        } else if n == Self::EQ_LOW_QUALITY_NAME {
            let vals: Vec<i32> = T::ch_strip(self.state())
                .eq
                .low_qualities
                .iter()
                .map(|&gain| gain as i32)
                .collect();
            elem_value.set_int(&vals);
            Ok(true)
        } else if n == Self::EQ_MIDDLE_QUALITY_NAME {
            let vals: Vec<i32> = T::ch_strip(self.state())
                .eq
                .middle_qualities
                .iter()
                .map(|&gain| gain as i32)
                .collect();
            elem_value.set_int(&vals);
            Ok(true)
        } else if n == Self::EQ_HIGH_QUALITY_NAME {
            let vals: Vec<i32> = T::ch_strip(self.state())
                .eq
                .high_qualities
                .iter()
                .map(|&gain| gain as i32)
                .collect();
            elem_value.set_int(&vals);
            Ok(true)
        } else if n == Self::DYN_ACTIVATE_NAME {
            elem_value.set_bool(&T::ch_strip(self.state()).dynamics.activates);
            Ok(true)
        } else if n == Self::DYN_GAIN_NAME {
            let vals: Vec<i32> = T::ch_strip(self.state())
                .dynamics
                .gains
                .iter()
                .map(|&gain| gain as i32)
                .collect();
            elem_value.set_int(&vals);
            Ok(true)
        } else if n == Self::DYN_ATTACK_NAME {
            let vals: Vec<i32> = T::ch_strip(self.state())
                .dynamics
                .attacks
                .iter()
                .map(|&attack| attack as i32)
                .collect();
            elem_value.set_int(&vals);
            Ok(true)
        } else if n == Self::DYN_RELEASE_NAME {
            let vals: Vec<i32> = T::ch_strip(self.state())
                .dynamics
                .releases
                .iter()
                .map(|&release| release as i32)
                .collect();
            elem_value.set_int(&vals);
            Ok(true)
        } else if n == Self::DYN_COMP_THRESHOLD_NAME {
            let vals: Vec<i32> = T::ch_strip(self.state())
                .dynamics
                .compressor_thresholds
                .iter()
                .map(|&th| th as i32)
                .collect();
            elem_value.set_int(&vals);
            Ok(true)
        } else if n == Self::DYN_COMP_RATIO_NAME {
            let vals: Vec<i32> = T::ch_strip(self.state())
                .dynamics
                .compressor_ratios
                .iter()
                .map(|&ratio| ratio as i32)
                .collect();
            elem_value.set_int(&vals);
            Ok(true)
        } else if n == Self::DYN_EX_THRESHOLD_NAME {
            let vals: Vec<i32> = T::ch_strip(self.state())
                .dynamics
                .expander_thresholds
                .iter()
                .map(|&th| th as i32)
                .collect();
            elem_value.set_int(&vals);
            Ok(true)
        } else if n == Self::DYN_EX_RATIO_NAME {
            let vals: Vec<i32> = T::ch_strip(self.state())
                .dynamics
                .expander_ratios
                .iter()
                .map(|&ratio| ratio as i32)
                .collect();
            elem_value.set_int(&vals);
            Ok(true)
        } else if n == Self::AUTOLEVEL_ACTIVATE_NAME {
            let vals = T::ch_strip(self.state()).autolevel.activates.clone();
            elem_value.set_bool(&vals);
            Ok(true)
        } else if n == Self::AUTOLEVEL_MAX_GAIN_NAME {
            let vals: Vec<i32> = T::ch_strip(self.state())
                .autolevel
                .max_gains
                .iter()
                .map(|&gain| gain as i32)
                .collect();
            elem_value.set_int(&vals);
            Ok(true)
        } else if n == Self::AUTOLEVEL_HEAD_ROOM_NAME {
            let vals: Vec<i32> = T::ch_strip(self.state())
                .autolevel
                .headrooms
                .iter()
                .map(|&gain| gain as i32)
                .collect();
            elem_value.set_int(&vals);
            Ok(true)
        } else if n == Self::AUTOLEVEL_RISE_TIME_NAME {
            let vals: Vec<i32> = T::ch_strip(self.state())
                .autolevel
                .rise_times
                .iter()
                .map(|&gain| gain as i32)
                .collect();
            elem_value.set_int(&vals);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write_ch_strip(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        let n = elem_id.name();

        if n == Self::HPF_ACTIVATE_NAME {
            let mut params = T::ch_strip(self.state()).hpf.clone();
            params
                .activates
                .iter_mut()
                .zip(elem_value.boolean())
                .for_each(|(activate, val)| *activate = val);
            let res = T::write_ch_strip_hpf(req, node, self.state_mut(), params, timeout_ms);
            debug!(params = ?T::ch_strip(self.state()).hpf, ?res);
            res.map(|_| true)
        } else if n == Self::HPF_CUT_OFF_NAME {
            let mut params = T::ch_strip(self.state()).hpf.clone();
            params
                .cut_offs
                .iter_mut()
                .zip(elem_value.int())
                .for_each(|(dst, &val)| *dst = val as u16);
            let res = T::write_ch_strip_hpf(req, node, self.state_mut(), params, timeout_ms);
            debug!(params = ?T::ch_strip(self.state()).hpf, ?res);
            res.map(|_| true)
        } else if n == Self::HPF_ROLL_OFF_NAME {
            let mut params = T::ch_strip(self.state()).hpf.clone();
            params
                .roll_offs
                .iter_mut()
                .zip(elem_value.enumerated())
                .try_for_each(|(level, &val)| {
                    let pos = val as usize;
                    Self::HPF_ROLL_OFF_LEVELS
                        .iter()
                        .nth(pos)
                        .ok_or_else(|| {
                            let msg = format!("Invalid index of roll off levels: {}", pos);
                            Error::new(FileError::Inval, &msg)
                        })
                        .map(|&l| *level = l)
                })?;
            let res = T::write_ch_strip_hpf(req, node, self.state_mut(), params, timeout_ms);
            debug!(params = ?T::ch_strip(self.state()).hpf, ?res);
            res.map(|_| true)
        } else if n == Self::EQ_ACTIVATE_NAME {
            let mut params = T::ch_strip(self.state()).eq.clone();
            params
                .activates
                .iter_mut()
                .zip(elem_value.boolean())
                .for_each(|(activate, val)| *activate = val);
            let res = T::write_ch_strip_eq(req, node, self.state_mut(), params, timeout_ms);
            debug!(params = ?T::ch_strip(self.state()));
            res.map(|_| true)
        } else if n == Self::EQ_LOW_TYPE_NAME {
            let mut params = T::ch_strip(self.state()).eq.clone();
            params
                .low_types
                .iter_mut()
                .zip(elem_value.enumerated())
                .try_for_each(|(eq_type, &val)| {
                    let pos = val as usize;
                    Self::EQ_TYPES
                        .iter()
                        .nth(pos)
                        .ok_or_else(|| {
                            let msg = format!("Invalid index of equalizer types: {}", pos);
                            Error::new(FileError::Inval, &msg)
                        })
                        .map(|&t| *eq_type = t)
                })?;
            let res = T::write_ch_strip_eq(req, node, self.state_mut(), params, timeout_ms);
            debug!(params = ?T::ch_strip(self.state()).eq, ?res);
            res.map(|_| true)
        } else if n == Self::EQ_HIGH_TYPE_NAME {
            let mut params = T::ch_strip(self.state()).eq.clone();
            params
                .high_types
                .iter_mut()
                .zip(elem_value.enumerated())
                .try_for_each(|(eq_type, &val)| {
                    let pos = val as usize;
                    Self::EQ_TYPES
                        .iter()
                        .nth(pos)
                        .ok_or_else(|| {
                            let msg = format!("Invalid index of equalizer types: {}", pos);
                            Error::new(FileError::Inval, &msg)
                        })
                        .map(|&t| *eq_type = t)
                })?;
            let res = T::write_ch_strip_eq(req, node, self.state_mut(), params, timeout_ms);
            debug!(params = ?T::ch_strip(self.state()).eq, ?res);
            res.map(|_| true)
        } else if n == Self::EQ_LOW_GAIN_NAME {
            let mut params = T::ch_strip(self.state()).eq.clone();
            params
                .low_gains
                .iter_mut()
                .zip(elem_value.int())
                .for_each(|(gain, &val)| *gain = val as i16);
            let res = T::write_ch_strip_eq(req, node, self.state_mut(), params, timeout_ms);
            debug!(params = ?T::ch_strip(self.state()).eq, ?res);
            res.map(|_| true)
        } else if n == Self::EQ_MIDDLE_GAIN_NAME {
            let mut params = T::ch_strip(self.state()).eq.clone();
            params
                .middle_gains
                .iter_mut()
                .zip(elem_value.int())
                .for_each(|(gain, &val)| *gain = val as i16);
            let res = T::write_ch_strip_eq(req, node, self.state_mut(), params, timeout_ms);
            debug!(params = ?T::ch_strip(self.state()).eq, ?res);
            res.map(|_| true)
        } else if n == Self::EQ_HIGH_GAIN_NAME {
            let mut params = T::ch_strip(self.state()).eq.clone();
            params
                .high_gains
                .iter_mut()
                .zip(elem_value.int())
                .for_each(|(gain, &val)| *gain = val as i16);
            let res = T::write_ch_strip_eq(req, node, self.state_mut(), params, timeout_ms);
            debug!(params = ?T::ch_strip(self.state()).eq, ?res);
            res.map(|_| true)
        } else if n == Self::EQ_LOW_FREQ_NAME {
            let mut params = T::ch_strip(self.state()).eq.clone();
            params
                .low_freqs
                .iter_mut()
                .zip(elem_value.int())
                .for_each(|(freq, &val)| *freq = val as u16);
            let res = T::write_ch_strip_eq(req, node, self.state_mut(), params, timeout_ms);
            debug!(params = ?T::ch_strip(self.state()).eq, ?res);
            res.map(|_| true)
        } else if n == Self::EQ_MIDDLE_FREQ_NAME {
            let mut params = T::ch_strip(self.state()).eq.clone();
            params
                .middle_freqs
                .iter_mut()
                .zip(elem_value.int())
                .for_each(|(freq, &val)| *freq = val as u16);
            let res = T::write_ch_strip_eq(req, node, self.state_mut(), params, timeout_ms);
            debug!(params = ?T::ch_strip(self.state()).eq, ?res);
            res.map(|_| true)
        } else if n == Self::EQ_HIGH_FREQ_NAME {
            let mut params = T::ch_strip(self.state()).eq.clone();
            params
                .high_freqs
                .iter_mut()
                .zip(elem_value.int())
                .for_each(|(freq, &val)| *freq = val as u16);
            let res = T::write_ch_strip_eq(req, node, self.state_mut(), params, timeout_ms);
            debug!(params = ?T::ch_strip(self.state()).eq, ?res);
            res.map(|_| true)
        } else if n == Self::EQ_LOW_QUALITY_NAME {
            let mut params = T::ch_strip(self.state()).eq.clone();
            params
                .low_qualities
                .iter_mut()
                .zip(elem_value.int())
                .for_each(|(quality, &val)| *quality = val as u16);
            let res = T::write_ch_strip_eq(req, node, self.state_mut(), params, timeout_ms);
            debug!(params = ?T::ch_strip(self.state()).eq, ?res);
            res.map(|_| true)
        } else if n == Self::EQ_MIDDLE_QUALITY_NAME {
            let mut params = T::ch_strip(self.state()).eq.clone();
            params
                .middle_qualities
                .iter_mut()
                .zip(elem_value.int())
                .for_each(|(quality, &val)| *quality = val as u16);
            let res = T::write_ch_strip_eq(req, node, self.state_mut(), params, timeout_ms);
            debug!(params = ?T::ch_strip(self.state()).eq, ?res);
            res.map(|_| true)
        } else if n == Self::EQ_HIGH_QUALITY_NAME {
            let mut params = T::ch_strip(self.state()).eq.clone();
            params
                .high_qualities
                .iter_mut()
                .zip(elem_value.int())
                .for_each(|(quality, &val)| *quality = val as u16);
            let res = T::write_ch_strip_eq(req, node, self.state_mut(), params, timeout_ms);
            debug!(params = ?T::ch_strip(self.state()).eq, ?res);
            res.map(|_| true)
        } else if n == Self::DYN_ACTIVATE_NAME {
            let mut params = T::ch_strip(self.state()).dynamics.clone();
            params
                .activates
                .iter_mut()
                .zip(elem_value.boolean())
                .for_each(|(activate, val)| *activate = val);
            let res = T::write_ch_strip_dynamics(req, node, self.state_mut(), params, timeout_ms);
            debug!(params = ?T::ch_strip(self.state()).dynamics, ?res);
            res.map(|_| true)
        } else if n == Self::DYN_GAIN_NAME {
            let mut params = T::ch_strip(self.state()).dynamics.clone();
            params
                .gains
                .iter_mut()
                .zip(elem_value.int())
                .for_each(|(gain, &val)| *gain = val as i16);
            let res = T::write_ch_strip_dynamics(req, node, self.state_mut(), params, timeout_ms);
            debug!(params = ?T::ch_strip(self.state()).dynamics, ?res);
            res.map(|_| true)
        } else if n == Self::DYN_ATTACK_NAME {
            let mut params = T::ch_strip(self.state()).dynamics.clone();
            params
                .attacks
                .iter_mut()
                .zip(elem_value.int())
                .for_each(|(attack, &val)| *attack = val as u16);
            let res = T::write_ch_strip_dynamics(req, node, self.state_mut(), params, timeout_ms);
            debug!(params = ?T::ch_strip(self.state()).dynamics, ?res);
            res.map(|_| true)
        } else if n == Self::DYN_RELEASE_NAME {
            let mut params = T::ch_strip(self.state()).dynamics.clone();
            params
                .releases
                .iter_mut()
                .zip(elem_value.int())
                .for_each(|(release, &val)| *release = val as u16);
            let res = T::write_ch_strip_dynamics(req, node, self.state_mut(), params, timeout_ms);
            debug!(params = ?T::ch_strip(self.state()).dynamics, ?res);
            res.map(|_| true)
        } else if n == Self::DYN_COMP_THRESHOLD_NAME {
            let mut params = T::ch_strip(self.state()).dynamics.clone();
            params
                .compressor_thresholds
                .iter_mut()
                .zip(elem_value.int())
                .for_each(|(threshold, &val)| *threshold = val as i16);
            let res = T::write_ch_strip_dynamics(req, node, self.state_mut(), params, timeout_ms);
            debug!(params = ?T::ch_strip(self.state()).dynamics, ?res);
            res.map(|_| true)
        } else if n == Self::DYN_COMP_RATIO_NAME {
            let mut params = T::ch_strip(self.state()).dynamics.clone();
            params
                .compressor_ratios
                .iter_mut()
                .zip(elem_value.int())
                .for_each(|(ratio, &val)| *ratio = val as u16);
            let res = T::write_ch_strip_dynamics(req, node, self.state_mut(), params, timeout_ms);
            debug!(params = ?T::ch_strip(self.state()).dynamics, ?res);
            res.map(|_| true)
        } else if n == Self::DYN_EX_THRESHOLD_NAME {
            let mut params = T::ch_strip(self.state()).dynamics.clone();
            params
                .expander_thresholds
                .iter_mut()
                .zip(elem_value.int())
                .for_each(|(threshold, &val)| *threshold = val as i16);
            let res = T::write_ch_strip_dynamics(req, node, self.state_mut(), params, timeout_ms);
            debug!(params = ?T::ch_strip(self.state()).dynamics, ?res);
            res.map(|_| true)
        } else if n == Self::DYN_EX_RATIO_NAME {
            let mut params = T::ch_strip(self.state()).dynamics.clone();
            params
                .expander_ratios
                .iter_mut()
                .zip(elem_value.int())
                .for_each(|(ratio, &val)| *ratio = val as u16);
            let res = T::write_ch_strip_dynamics(req, node, self.state_mut(), params, timeout_ms);
            debug!(params = ?T::ch_strip(self.state()).dynamics, ?res);
            res.map(|_| true)
        } else if n == Self::AUTOLEVEL_ACTIVATE_NAME {
            let mut params = T::ch_strip(self.state()).autolevel.clone();
            params
                .activates
                .iter_mut()
                .zip(elem_value.boolean())
                .for_each(|(activate, val)| *activate = val);
            let res = T::write_ch_strip_autolevel(req, node, self.state_mut(), params, timeout_ms);
            debug!(params = ?T::ch_strip(self.state()).autolevel, ?res);
            res.map(|_| true)
        } else if n == Self::AUTOLEVEL_MAX_GAIN_NAME {
            let mut params = T::ch_strip(self.state()).autolevel.clone();
            params
                .max_gains
                .iter_mut()
                .zip(elem_value.int())
                .for_each(|(gain, &val)| *gain = val as u16);
            let res = T::write_ch_strip_autolevel(req, node, self.state_mut(), params, timeout_ms);
            debug!(params = ?T::ch_strip(self.state()).autolevel, ?res);
            res.map(|_| true)
        } else if n == Self::AUTOLEVEL_HEAD_ROOM_NAME {
            let mut params = T::ch_strip(self.state()).autolevel.clone();
            params
                .headrooms
                .iter_mut()
                .zip(elem_value.int())
                .for_each(|(headroom, &val)| *headroom = val as u16);
            let res = T::write_ch_strip_autolevel(req, node, self.state_mut(), params, timeout_ms);
            debug!(params = ?T::ch_strip(self.state()).autolevel, ?res);
            res.map(|_| true)
        } else if n == Self::AUTOLEVEL_RISE_TIME_NAME {
            let mut params = T::ch_strip(self.state()).autolevel.clone();
            params
                .rise_times
                .iter_mut()
                .zip(elem_value.int())
                .for_each(|(time, &val)| *time = val as u16);
            let res = T::write_ch_strip_autolevel(req, node, self.state_mut(), params, timeout_ms);
            debug!(params = ?T::ch_strip(self.state()).autolevel, ?res);
            res.map(|_| true)
        } else {
            Ok(false)
        }
    }
}

impl<T> FfLatterChStripCtlOperation<T, FfLatterInputChStripState> for LatterDspCtl<T>
where
    T: RmeFfLatterChStripOperation<FfLatterInputChStripState>,
{
    const HPF_ACTIVATE_NAME: &'static str = "input:hpf-activate";
    const HPF_CUT_OFF_NAME: &'static str = "input:hpf-cut-off";
    const HPF_ROLL_OFF_NAME: &'static str = "input:hpf-roll-off";

    const EQ_ACTIVATE_NAME: &'static str = "input:eq-activate";
    const EQ_LOW_TYPE_NAME: &'static str = "input:eq-low-type";
    const EQ_LOW_GAIN_NAME: &'static str = "input:eq-low-gain";
    const EQ_LOW_FREQ_NAME: &'static str = "input:eq-low-freq";
    const EQ_LOW_QUALITY_NAME: &'static str = "input:eq-low-quality";
    const EQ_MIDDLE_GAIN_NAME: &'static str = "input:eq-middle-gain";
    const EQ_MIDDLE_FREQ_NAME: &'static str = "input:eq-middle-freq";
    const EQ_MIDDLE_QUALITY_NAME: &'static str = "input:eq-middle-quality";
    const EQ_HIGH_TYPE_NAME: &'static str = "input:eq-high-type";
    const EQ_HIGH_GAIN_NAME: &'static str = "input:eq-high-gain";
    const EQ_HIGH_FREQ_NAME: &'static str = "input:eq-high-freq";
    const EQ_HIGH_QUALITY_NAME: &'static str = "input:eq-high-quality";

    const DYN_ACTIVATE_NAME: &'static str = "input:dyn-activate";
    const DYN_GAIN_NAME: &'static str = "input:dyn-gain";
    const DYN_ATTACK_NAME: &'static str = "input:dyn-attack";
    const DYN_RELEASE_NAME: &'static str = "input:dyn-release";
    const DYN_COMP_THRESHOLD_NAME: &'static str = "input:dyn-compressor-threshold";
    const DYN_COMP_RATIO_NAME: &'static str = "input:dyn-compressor-ratio";
    const DYN_EX_THRESHOLD_NAME: &'static str = "input:dyn-expander-threshold";
    const DYN_EX_RATIO_NAME: &'static str = "input:dyn-expander-ratio";

    const AUTOLEVEL_ACTIVATE_NAME: &'static str = "input:autolevel-activate";
    const AUTOLEVEL_MAX_GAIN_NAME: &'static str = "input:autolevel-max-gain";
    const AUTOLEVEL_HEAD_ROOM_NAME: &'static str = "input:autolevel-head-room";
    const AUTOLEVEL_RISE_TIME_NAME: &'static str = "input:autolevel-rise-time";

    fn state(&self) -> &FfLatterDspState {
        &self.0
    }

    fn state_mut(&mut self) -> &mut FfLatterDspState {
        &mut self.0
    }
}

impl<T> FfLatterChStripCtlOperation<T, FfLatterOutputChStripState> for LatterDspCtl<T>
where
    T: RmeFfLatterChStripOperation<FfLatterOutputChStripState>,
{
    const HPF_ACTIVATE_NAME: &'static str = "output:hpf-activate";
    const HPF_CUT_OFF_NAME: &'static str = "output:hpf-cut-off";
    const HPF_ROLL_OFF_NAME: &'static str = "output:hpf-roll-off";

    const EQ_ACTIVATE_NAME: &'static str = "output:eq-activate";
    const EQ_LOW_TYPE_NAME: &'static str = "output:eq-low-type";
    const EQ_LOW_GAIN_NAME: &'static str = "output:eq-low-gain";
    const EQ_LOW_FREQ_NAME: &'static str = "output:eq-low-freq";
    const EQ_LOW_QUALITY_NAME: &'static str = "output:eq-low-quality";
    const EQ_MIDDLE_GAIN_NAME: &'static str = "output:eq-middle-gain";
    const EQ_MIDDLE_FREQ_NAME: &'static str = "output:eq-middle-freq";
    const EQ_MIDDLE_QUALITY_NAME: &'static str = "output:eq-middle-quality";
    const EQ_HIGH_TYPE_NAME: &'static str = "output:eq-high-type";
    const EQ_HIGH_GAIN_NAME: &'static str = "output:eq-high-gain";
    const EQ_HIGH_FREQ_NAME: &'static str = "output:eq-high-freq";
    const EQ_HIGH_QUALITY_NAME: &'static str = "output:eq-high-quality";

    const DYN_ACTIVATE_NAME: &'static str = "output:dyn-activate";
    const DYN_GAIN_NAME: &'static str = "output:dyn-gain";
    const DYN_ATTACK_NAME: &'static str = "output:dyn-attack";
    const DYN_RELEASE_NAME: &'static str = "output:dyn-release";
    const DYN_COMP_THRESHOLD_NAME: &'static str = "output:dyn-compressor-threshold";
    const DYN_COMP_RATIO_NAME: &'static str = "output:dyn-compressor-ratio";
    const DYN_EX_THRESHOLD_NAME: &'static str = "output:dyn-expander-threshold";
    const DYN_EX_RATIO_NAME: &'static str = "output:dyn-expander-ratio";

    const AUTOLEVEL_ACTIVATE_NAME: &'static str = "output:autolevel-activate";
    const AUTOLEVEL_MAX_GAIN_NAME: &'static str = "output:autolevel-max-gain";
    const AUTOLEVEL_HEAD_ROOM_NAME: &'static str = "output:autolevel-head-room";
    const AUTOLEVEL_RISE_TIME_NAME: &'static str = "output:autolevel-rise-time";

    fn state(&self) -> &FfLatterDspState {
        &self.0
    }

    fn state_mut(&mut self) -> &mut FfLatterDspState {
        &mut self.0
    }
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
    }
    .to_string()
}

fn fx_echo_type_to_string(echo_type: &FfLatterFxEchoType) -> String {
    match echo_type {
        FfLatterFxEchoType::StereoEcho => "Stereo-echo",
        FfLatterFxEchoType::StereoCross => "Stereo-cross",
        FfLatterFxEchoType::PongEcho => "Pong-echo",
    }
    .to_string()
}

fn fx_echo_lpf_freq_to_string(lpf_freq: &FfLatterFxEchoLpfFreq) -> String {
    match lpf_freq {
        FfLatterFxEchoLpfFreq::Off => "Off",
        FfLatterFxEchoLpfFreq::H2000 => "2kHz",
        FfLatterFxEchoLpfFreq::H4000 => "4kHz",
        FfLatterFxEchoLpfFreq::H8000 => "8kHz",
        FfLatterFxEchoLpfFreq::H12000 => "12kHz",
        FfLatterFxEchoLpfFreq::H16000 => "16kHz",
    }
    .to_string()
}

const LINE_SRC_GAIN_NAME: &str = "fx:line-source-gain";
const MIC_SRC_GAIN_NAME: &str = "fx:mic-source-gain";
const SPDIF_SRC_GAIN_NAME: &str = "fx:spdif-source-gain";
const ADAT_SRC_GAIN_NAME: &str = "fx:adat-source-gain";
const STREAM_SRC_GAIN_NAME: &str = "fx:stream-source-gain";

const LINE_OUT_VOL_NAME: &str = "fx:line-output-volume";
const HP_OUT_VOL_NAME: &str = "fx:hp-output-volume";
const SPDIF_OUT_VOL_NAME: &str = "fx:spdif-output-volume";
const ADAT_OUT_VOL_NAME: &str = "fx:adat-output-volume";

const REVERB_ACTIVATE_NAME: &str = "fx:reverb-activate";
const REVERB_TYPE_NAME: &str = "fx:reverb-type";
const REVERB_PRE_DELAY_NAME: &str = "fx:reverb-pre-delay";
const REVERB_PRE_HPF_FREQ_NAME: &str = "fx:reverb-pre-hpf-freq";
const REVERB_ROOM_SCALE_NAME: &str = "fx:reverb-room-scale";
const REVERB_ATTACK_NAME: &str = "fx:reverb-attack";
const REVERB_HOLD_NAME: &str = "fx:reverb-hold";
const REVERB_RELEASE_NAME: &str = "fx:reverb-release";
const REVERB_POST_LPF_FREQ_NAME: &str = "fx:reverb-post-lpf-freq";
const REVERB_TIME_NAME: &str = "fx:reverb-time";
const REVERB_DAMPING_NAME: &str = "fx:reverb-damping";
const REVERB_SMOOTH_NAME: &str = "fx:reverb-smooth";
const REVERB_VOL_NAME: &str = "fx:reverb-volume";
const REVERB_STEREO_WIDTH_NAME: &str = "fx:reverb-stereo-width";

const ECHO_ACTIVATE_NAME: &str = "fx:echo-activate";
const ECHO_TYPE_NAME: &str = "fx:echo-type";
const ECHO_DELAY_NAME: &str = "fx:echo-delay";
const ECHO_FEEDBACK_NAME: &str = "fx:echo-feedback";
const ECHO_LPF_FREQ_NAME: &str = "fx:echo-lpf-freq";
const ECHO_VOL_NAME: &str = "fx:echo-volume";
const ECHO_STEREO_WIDTH_NAME: &str = "fx:echo-stereo-width";

impl<T: RmeFfLatterFxOperation> LatterDspCtl<T> {
    const PHYS_LEVEL_TLV: DbInterval = DbInterval {
        min: -6500,
        max: 000,
        linear: false,
        mute_avail: false,
    };
    const VIRT_LEVEL_TLV: DbInterval = DbInterval {
        min: -6500,
        max: 000,
        linear: false,
        mute_avail: false,
    };

    const REVERB_TYPES: [FfLatterFxReverbType; 15] = [
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

    const ECHO_TYPES: [FfLatterFxEchoType; 3] = [
        FfLatterFxEchoType::StereoEcho,
        FfLatterFxEchoType::StereoCross,
        FfLatterFxEchoType::PongEcho,
    ];

    const ECHO_LPF_FREQS: [FfLatterFxEchoLpfFreq; 6] = [
        FfLatterFxEchoLpfFreq::Off,
        FfLatterFxEchoLpfFreq::H2000,
        FfLatterFxEchoLpfFreq::H4000,
        FfLatterFxEchoLpfFreq::H8000,
        FfLatterFxEchoLpfFreq::H12000,
        FfLatterFxEchoLpfFreq::H16000,
    ];

    fn cache_fx(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let res = T::init_fx(req, node, &mut self.0, timeout_ms);
        debug!(params = ?self.0, ?res);
        res
    }

    fn load_fx(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        [
            (LINE_SRC_GAIN_NAME, T::LINE_INPUT_COUNT),
            (MIC_SRC_GAIN_NAME, T::MIC_INPUT_COUNT),
            (SPDIF_SRC_GAIN_NAME, T::SPDIF_INPUT_COUNT),
            (ADAT_SRC_GAIN_NAME, T::ADAT_INPUT_COUNT),
        ]
        .iter()
        .try_for_each(|&(name, count)| {
            let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
            card_cntr
                .add_int_elems(
                    &elem_id,
                    1,
                    T::FX_PHYS_LEVEL_MIN,
                    T::FX_PHYS_LEVEL_MAX,
                    T::FX_PHYS_LEVEL_STEP,
                    count,
                    Some(&Vec::<u32>::from(&Self::PHYS_LEVEL_TLV)),
                    true,
                )
                .map(|_| ())
        })?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, STREAM_SRC_GAIN_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            1,
            T::FX_VIRT_LEVEL_MIN,
            T::FX_VIRT_LEVEL_MAX,
            T::FX_VIRT_LEVEL_STEP,
            T::STREAM_INPUT_COUNT,
            Some(&Vec::<u32>::from(&Self::VIRT_LEVEL_TLV)),
            true,
        )?;

        [
            (LINE_OUT_VOL_NAME, T::LINE_OUTPUT_COUNT),
            (HP_OUT_VOL_NAME, T::HP_OUTPUT_COUNT),
            (SPDIF_OUT_VOL_NAME, T::SPDIF_OUTPUT_COUNT),
            (ADAT_OUT_VOL_NAME, T::ADAT_OUTPUT_COUNT),
        ]
        .iter()
        .try_for_each(|&(name, count)| {
            let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
            card_cntr
                .add_int_elems(
                    &elem_id,
                    1,
                    T::FX_PHYS_LEVEL_MIN,
                    T::FX_PHYS_LEVEL_MAX,
                    T::FX_PHYS_LEVEL_STEP,
                    count,
                    Some(&Vec::<u32>::from(&Self::PHYS_LEVEL_TLV)),
                    true,
                )
                .map(|_| ())
        })?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, REVERB_ACTIVATE_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        let labels: Vec<String> = Self::REVERB_TYPES
            .iter()
            .map(|t| fx_reverb_type_to_string(t))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, REVERB_TYPE_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, REVERB_PRE_DELAY_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            1,
            T::REVERB_PRE_DELAY_MIN,
            T::REVERB_PRE_DELAY_MAX,
            T::REVERB_PRE_DELAY_STEP,
            1,
            None,
            true,
        )?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, REVERB_PRE_HPF_FREQ_NAME, 0);
        let _ = card_cntr.add_int_elems(&elem_id, 1, 20, 500, 1, 1, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, REVERB_ROOM_SCALE_NAME, 0);
        let _ = card_cntr.add_int_elems(&elem_id, 1, 50, 300, 1, 1, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, REVERB_ATTACK_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            1,
            T::REVERB_ATTACK_MIN,
            T::REVERB_ATTACK_MAX,
            T::REVERB_ATTACK_STEP,
            1,
            None,
            true,
        );

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, REVERB_HOLD_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            1,
            T::REVERB_HOLD_MIN,
            T::REVERB_HOLD_MAX,
            T::REVERB_HOLD_STEP,
            1,
            None,
            true,
        );

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, REVERB_RELEASE_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            1,
            T::REVERB_RELEASE_MIN,
            T::REVERB_RELEASE_MAX,
            T::REVERB_RELEASE_STEP,
            1,
            None,
            true,
        );

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, REVERB_POST_LPF_FREQ_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            1,
            T::REVERB_POST_LPF_FREQ_MIN,
            T::REVERB_POST_LPF_FREQ_MAX,
            T::REVERB_POST_LPF_FREQ_STEP,
            1,
            None,
            true,
        );

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, REVERB_TIME_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            1,
            T::REVERB_TIME_MIN,
            T::REVERB_TIME_MAX,
            T::REVERB_TIME_STEP,
            1,
            None,
            true,
        );

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, REVERB_DAMPING_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            1,
            T::REVERB_DAMPING_MIN,
            T::REVERB_DAMPING_MAX,
            T::REVERB_DAMPING_STEP,
            1,
            None,
            true,
        );

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, REVERB_SMOOTH_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            1,
            T::REVERB_SMOOTH_MIN,
            T::REVERB_SMOOTH_MAX,
            T::REVERB_SMOOTH_STEP,
            1,
            None,
            true,
        );

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, REVERB_VOL_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            1,
            T::REVERB_VOL_MIN,
            T::REVERB_VOL_MAX,
            T::REVERB_VOL_STEP,
            1,
            None,
            true,
        );

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, REVERB_STEREO_WIDTH_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            1,
            T::REVERB_STEREO_WIDTH_MIN,
            T::REVERB_STEREO_WIDTH_MAX,
            T::REVERB_STEREO_WIDTH_STEP,
            1,
            None,
            true,
        );

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, ECHO_ACTIVATE_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        let labels: Vec<String> = Self::ECHO_TYPES
            .iter()
            .map(|t| fx_echo_type_to_string(t))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, ECHO_TYPE_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, ECHO_DELAY_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            1,
            T::ECHO_DELAY_MIN,
            T::ECHO_DELAY_MAX,
            T::ECHO_DELAY_STEP,
            1,
            None,
            true,
        );

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, ECHO_FEEDBACK_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            1,
            T::ECHO_FEEDBACK_MIN,
            T::ECHO_FEEDBACK_MAX,
            T::ECHO_FEEDBACK_STEP,
            1,
            None,
            true,
        );

        let labels: Vec<String> = Self::ECHO_LPF_FREQS
            .iter()
            .map(|t| fx_echo_lpf_freq_to_string(t))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, ECHO_LPF_FREQ_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, ECHO_VOL_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            1,
            T::ECHO_VOL_MIN,
            T::ECHO_VOL_MAX,
            T::ECHO_VOL_STEP,
            1,
            None,
            true,
        );

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, ECHO_STEREO_WIDTH_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            1,
            T::ECHO_STEREO_WIDTH_MIN,
            T::ECHO_STEREO_WIDTH_MAX,
            T::ECHO_STEREO_WIDTH_STEP,
            1,
            None,
            true,
        );

        Ok(())
    }

    fn read_fx(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            LINE_SRC_GAIN_NAME => {
                let vals: Vec<i32> = self
                    .0
                    .fx
                    .line_input_gains
                    .iter()
                    .map(|&gain| gain as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            MIC_SRC_GAIN_NAME => {
                let vals: Vec<i32> = self
                    .0
                    .fx
                    .mic_input_gains
                    .iter()
                    .map(|&gain| gain as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            SPDIF_SRC_GAIN_NAME => {
                let vals: Vec<i32> = self
                    .0
                    .fx
                    .spdif_input_gains
                    .iter()
                    .map(|&gain| gain as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            ADAT_SRC_GAIN_NAME => {
                let vals: Vec<i32> = self
                    .0
                    .fx
                    .adat_input_gains
                    .iter()
                    .map(|&gain| gain as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            STREAM_SRC_GAIN_NAME => {
                let vals: Vec<i32> = self
                    .0
                    .fx
                    .stream_input_gains
                    .iter()
                    .map(|&gain| gain as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            LINE_OUT_VOL_NAME => {
                let vals: Vec<i32> = self
                    .0
                    .fx
                    .line_output_vols
                    .iter()
                    .map(|&vol| vol as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            HP_OUT_VOL_NAME => {
                let vals: Vec<i32> = self
                    .0
                    .fx
                    .hp_output_vols
                    .iter()
                    .map(|&vol| vol as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            SPDIF_OUT_VOL_NAME => {
                let vals: Vec<i32> = self
                    .0
                    .fx
                    .spdif_output_vols
                    .iter()
                    .map(|&vol| vol as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            ADAT_OUT_VOL_NAME => {
                let vals: Vec<i32> = self
                    .0
                    .fx
                    .adat_output_vols
                    .iter()
                    .map(|&vol| vol as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            REVERB_ACTIVATE_NAME => {
                elem_value.set_bool(&[self.0.fx.reverb.activate]);
                Ok(true)
            }
            REVERB_TYPE_NAME => {
                let val = Self::REVERB_TYPES
                    .iter()
                    .position(|t| t.eq(&self.0.fx.reverb.reverb_type))
                    .unwrap();
                elem_value.set_enum(&[val as u32]);
                Ok(true)
            }
            REVERB_PRE_DELAY_NAME => {
                elem_value.set_int(&[self.0.fx.reverb.pre_delay as i32]);
                Ok(true)
            }
            REVERB_PRE_HPF_FREQ_NAME => {
                elem_value.set_int(&[self.0.fx.reverb.pre_hpf as i32]);
                Ok(true)
            }
            REVERB_ROOM_SCALE_NAME => {
                elem_value.set_int(&[self.0.fx.reverb.room_scale as i32]);
                Ok(true)
            }
            REVERB_ATTACK_NAME => {
                elem_value.set_int(&[self.0.fx.reverb.attack as i32]);
                Ok(true)
            }
            REVERB_HOLD_NAME => {
                elem_value.set_int(&[self.0.fx.reverb.hold as i32]);
                Ok(true)
            }
            REVERB_RELEASE_NAME => {
                elem_value.set_int(&[self.0.fx.reverb.release as i32]);
                Ok(true)
            }
            REVERB_POST_LPF_FREQ_NAME => {
                elem_value.set_int(&[self.0.fx.reverb.post_lpf as i32]);
                Ok(true)
            }
            REVERB_TIME_NAME => {
                elem_value.set_int(&[self.0.fx.reverb.time as i32]);
                Ok(true)
            }
            REVERB_DAMPING_NAME => {
                elem_value.set_int(&[self.0.fx.reverb.damping as i32]);
                Ok(true)
            }
            REVERB_SMOOTH_NAME => {
                elem_value.set_int(&[self.0.fx.reverb.smooth as i32]);
                Ok(true)
            }
            REVERB_VOL_NAME => {
                elem_value.set_int(&[self.0.fx.reverb.volume as i32]);
                Ok(true)
            }
            REVERB_STEREO_WIDTH_NAME => {
                elem_value.set_int(&[self.0.fx.reverb.stereo_width as i32]);
                Ok(true)
            }
            ECHO_ACTIVATE_NAME => {
                elem_value.set_bool(&[self.0.fx.echo.activate]);
                Ok(true)
            }
            ECHO_TYPE_NAME => {
                let pos = Self::ECHO_TYPES
                    .iter()
                    .position(|t| t.eq(&self.0.fx.echo.echo_type))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            ECHO_DELAY_NAME => {
                elem_value.set_int(&[self.0.fx.echo.delay as i32]);
                Ok(true)
            }
            ECHO_FEEDBACK_NAME => {
                elem_value.set_int(&[self.0.fx.echo.feedback as i32]);
                Ok(true)
            }
            ECHO_LPF_FREQ_NAME => {
                let pos = Self::ECHO_LPF_FREQS
                    .iter()
                    .position(|f| f.eq(&self.0.fx.echo.lpf))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            ECHO_VOL_NAME => {
                elem_value.set_int(&[self.0.fx.echo.volume as i32]);
                Ok(true)
            }
            ECHO_STEREO_WIDTH_NAME => {
                elem_value.set_int(&[self.0.fx.echo.stereo_width as i32]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write_fx(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            LINE_SRC_GAIN_NAME => {
                let mut params = self.0.fx.clone();
                params
                    .line_input_gains
                    .iter_mut()
                    .zip(elem_value.int())
                    .for_each(|(d, s)| *d = *s as i16);
                let res = T::write_fx_input_gains(req, node, &mut self.0, params, timeout_ms);
                debug!(params = ?self.0.fx, ?res);
                res.map(|_| true)
            }
            MIC_SRC_GAIN_NAME => {
                let mut params = self.0.fx.clone();
                params
                    .mic_input_gains
                    .iter_mut()
                    .zip(elem_value.int())
                    .for_each(|(d, s)| *d = *s as i16);
                let res = T::write_fx_input_gains(req, node, &mut self.0, params, timeout_ms);
                debug!(params = ?self.0.fx, ?res);
                res.map(|_| true)
            }
            SPDIF_SRC_GAIN_NAME => {
                let mut params = self.0.fx.clone();
                params
                    .spdif_input_gains
                    .iter_mut()
                    .zip(elem_value.int())
                    .for_each(|(d, s)| *d = *s as i16);
                let res = T::write_fx_input_gains(req, node, &mut self.0, params, timeout_ms);
                debug!(params = ?self.0.fx, ?res);
                res.map(|_| true)
            }
            ADAT_SRC_GAIN_NAME => {
                let mut params = self.0.fx.clone();
                params
                    .adat_input_gains
                    .iter_mut()
                    .zip(elem_value.int())
                    .for_each(|(d, s)| *d = *s as i16);
                let res = T::write_fx_input_gains(req, node, &mut self.0, params, timeout_ms);
                debug!(params = ?self.0.fx, ?res);
                res.map(|_| true)
            }
            STREAM_SRC_GAIN_NAME => {
                let mut params = self.0.fx.clone();
                params
                    .stream_input_gains
                    .iter_mut()
                    .zip(elem_value.int())
                    .for_each(|(d, s)| *d = *s as u16);
                let res = T::write_fx_input_gains(req, node, &mut self.0, params, timeout_ms);
                debug!(params = ?self.0.fx, ?res);
                res.map(|_| true)
            }
            LINE_OUT_VOL_NAME => {
                let mut params = self.0.fx.clone();
                params
                    .line_output_vols
                    .iter_mut()
                    .zip(elem_value.int())
                    .for_each(|(d, s)| *d = *s as i16);
                let res = T::write_fx_output_volumes(req, node, &mut self.0, params, timeout_ms);
                debug!(params = ?self.0.fx);
                res.map(|_| true)
            }
            HP_OUT_VOL_NAME => {
                let mut params = self.0.fx.clone();
                params
                    .hp_output_vols
                    .iter_mut()
                    .zip(elem_value.int())
                    .for_each(|(d, s)| *d = *s as i16);
                let res = T::write_fx_output_volumes(req, node, &mut self.0, params, timeout_ms);
                debug!(params = ?self.0.fx);
                res.map(|_| true)
            }
            SPDIF_OUT_VOL_NAME => {
                let mut params = self.0.fx.clone();
                params
                    .spdif_output_vols
                    .iter_mut()
                    .zip(elem_value.int())
                    .for_each(|(d, s)| *d = *s as i16);
                let res = T::write_fx_output_volumes(req, node, &mut self.0, params, timeout_ms);
                debug!(params = ?self.0.fx);
                res.map(|_| true)
            }
            ADAT_OUT_VOL_NAME => {
                let mut params = self.0.fx.clone();
                params
                    .adat_output_vols
                    .iter_mut()
                    .zip(elem_value.int())
                    .for_each(|(d, s)| *d = *s as i16);
                let res = T::write_fx_output_volumes(req, node, &mut self.0, params, timeout_ms);
                debug!(params = ?self.0.fx);
                res.map(|_| true)
            }
            REVERB_ACTIVATE_NAME => self
                .update_reverb(req, node, timeout_ms, |params| {
                    params.activate = elem_value.boolean()[0];
                    Ok(())
                })
                .map(|_| true),
            REVERB_TYPE_NAME => {
                let val = elem_value.enumerated()[0];
                let reverb_type = Self::REVERB_TYPES
                    .iter()
                    .nth(val as usize)
                    .ok_or_else(|| {
                        let msg = format!("Invalid index of type of reverb effect: {}", val);
                        Error::new(FileError::Inval, &msg)
                    })
                    .map(|&t| t)?;
                self.update_reverb(req, node, timeout_ms, |params| {
                    params.reverb_type = reverb_type;
                    Ok(())
                })
                .map(|_| true)
            }
            REVERB_PRE_DELAY_NAME => self
                .update_reverb(req, node, timeout_ms, |params| {
                    params.pre_delay = elem_value.int()[0] as u16;
                    Ok(())
                })
                .map(|_| true),
            REVERB_PRE_HPF_FREQ_NAME => self
                .update_reverb(req, node, timeout_ms, |params| {
                    params.pre_hpf = elem_value.int()[0] as u16;
                    Ok(())
                })
                .map(|_| true),
            REVERB_ROOM_SCALE_NAME => self
                .update_reverb(req, node, timeout_ms, |params| {
                    params.room_scale = elem_value.int()[0] as u16;
                    Ok(())
                })
                .map(|_| true),
            REVERB_ATTACK_NAME => self
                .update_reverb(req, node, timeout_ms, |params| {
                    params.attack = elem_value.int()[0] as u16;
                    Ok(())
                })
                .map(|_| true),
            REVERB_HOLD_NAME => self
                .update_reverb(req, node, timeout_ms, |params| {
                    params.hold = elem_value.int()[0] as u16;
                    Ok(())
                })
                .map(|_| true),
            REVERB_RELEASE_NAME => self
                .update_reverb(req, node, timeout_ms, |params| {
                    params.release = elem_value.int()[0] as u16;
                    Ok(())
                })
                .map(|_| true),
            REVERB_POST_LPF_FREQ_NAME => self
                .update_reverb(req, node, timeout_ms, |params| {
                    params.post_lpf = elem_value.int()[0] as u16;
                    Ok(())
                })
                .map(|_| true),
            REVERB_TIME_NAME => self
                .update_reverb(req, node, timeout_ms, |params| {
                    params.time = elem_value.int()[0] as u16;
                    Ok(())
                })
                .map(|_| true),
            REVERB_DAMPING_NAME => self
                .update_reverb(req, node, timeout_ms, |params| {
                    params.damping = elem_value.int()[0] as u16;
                    Ok(())
                })
                .map(|_| true),
            REVERB_SMOOTH_NAME => self
                .update_reverb(req, node, timeout_ms, |params| {
                    params.smooth = elem_value.int()[0] as u16;
                    Ok(())
                })
                .map(|_| true),
            REVERB_VOL_NAME => self
                .update_reverb(req, node, timeout_ms, |params| {
                    params.volume = elem_value.int()[0] as i16;
                    Ok(())
                })
                .map(|_| true),
            REVERB_STEREO_WIDTH_NAME => self
                .update_reverb(req, node, timeout_ms, |params| {
                    params.stereo_width = elem_value.int()[0] as u16;
                    Ok(())
                })
                .map(|_| true),
            ECHO_ACTIVATE_NAME => self
                .update_echo(req, node, timeout_ms, |params| {
                    params.activate = elem_value.boolean()[0];
                    Ok(())
                })
                .map(|_| true),
            ECHO_TYPE_NAME => {
                let val = elem_value.enumerated()[0];
                let echo_type = Self::ECHO_TYPES
                    .iter()
                    .nth(val as usize)
                    .ok_or_else(|| {
                        let msg = format!("Invalid index of type of echo effect: {}", val);
                        Error::new(FileError::Inval, &msg)
                    })
                    .map(|&t| t)?;
                self.update_echo(req, node, timeout_ms, |params| {
                    params.echo_type = echo_type;
                    Ok(())
                })?;
                Ok(true)
            }
            ECHO_DELAY_NAME => self
                .update_echo(req, node, timeout_ms, |params| {
                    params.delay = elem_value.int()[0] as u16;
                    Ok(())
                })
                .map(|_| true),
            ECHO_FEEDBACK_NAME => self
                .update_echo(req, node, timeout_ms, |params| {
                    params.feedback = elem_value.int()[0] as u16;
                    Ok(())
                })
                .map(|_| true),
            ECHO_LPF_FREQ_NAME => {
                let val = elem_value.enumerated()[0];
                let lpf = Self::ECHO_LPF_FREQS
                    .iter()
                    .nth(val as usize)
                    .ok_or_else(|| {
                        let msg = format!("Invalid index of type of echo HPF frequency: {}", val);
                        Error::new(FileError::Inval, &msg)
                    })
                    .map(|&t| t)?;
                self.update_echo(req, node, timeout_ms, |params| {
                    params.lpf = lpf;
                    Ok(())
                })?;
                Ok(true)
            }
            ECHO_VOL_NAME => self
                .update_echo(req, node, timeout_ms, |params| {
                    params.volume = elem_value.int()[0] as i16;
                    Ok(())
                })
                .map(|_| true),
            ECHO_STEREO_WIDTH_NAME => self
                .update_echo(req, node, timeout_ms, |params| {
                    params.stereo_width = elem_value.int()[0] as u16;
                    Ok(())
                })
                .map(|_| true),
            _ => Ok(false),
        }
    }

    fn update_reverb<F>(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        timeout_ms: u32,
        cb: F,
    ) -> Result<(), Error>
    where
        F: Fn(&mut FfLatterFxReverbState) -> Result<(), Error>,
    {
        let mut params = self.0.fx.reverb.clone();
        cb(&mut params)?;
        let res = T::write_fx_reverb(req, node, &mut self.0, &params, timeout_ms);
        debug!(?params, ?res);
        res
    }

    fn update_echo<F>(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        timeout_ms: u32,
        cb: F,
    ) -> Result<(), Error>
    where
        F: Fn(&mut FfLatterFxEchoState) -> Result<(), Error>,
    {
        let mut params = self.0.fx.echo.clone();
        cb(&mut params)?;
        let res = T::write_fx_echo(req, node, &mut self.0, &params, timeout_ms);
        debug!(?params, ?res);
        res
    }
}
