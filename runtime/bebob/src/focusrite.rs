// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

pub mod saffireproio_model;

pub mod saffire_model;
pub mod saffirele_model;

use {
    super::{common_ctls::*, *},
    protocols::{
        focusrite::{saffire::*, saffireproio::*, *},
        *,
    },
};

trait SaffireProMediaClkFreqCtlOperation<T: SaffireProioMediaClockFrequencyOperation> {
    fn load_freq(&mut self, card_cntr: &mut CardCntr) -> Result<Vec<ElemId>, Error> {
        let labels: Vec<String> = T::FREQ_LIST.iter().map(|&r| r.to_string()).collect();

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, CLK_RATE_NAME, 0);
        card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)
    }

    fn read_freq(
        &self,
        unit: &(SndUnit, FwNode),
        req: &FwReq,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            CLK_RATE_NAME => ElemValueAccessor::<u32>::set_val(elem_value, || {
                T::read_clk_freq(req, &unit.1, timeout_ms).map(|idx| idx as u32)
            })
            .map(|_| true),
            _ => Ok(false),
        }
    }

    fn write_freq(
        &self,
        unit: &mut (SndUnit, FwNode),
        req: &FwReq,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            CLK_RATE_NAME => {
                unit.0.lock()?;
                let res = ElemValueAccessor::<u32>::get_val(elem_value, |val| {
                    T::write_clk_freq(req, &unit.1, val as usize, timeout_ms)
                })
                .map(|_| true);
                let _ = unit.0.unlock();
                res
            }
            _ => Ok(false),
        }
    }
}

fn sampling_clk_src_to_str(src: &SaffireProioSamplingClockSource) -> &str {
    match src {
        SaffireProioSamplingClockSource::Internal => "Internal",
        SaffireProioSamplingClockSource::Spdif => "S/PDIF",
        SaffireProioSamplingClockSource::Adat0 => "ADAT-A",
        SaffireProioSamplingClockSource::Adat1 => "ADAT-B",
        SaffireProioSamplingClockSource::WordClock => "Word-clock",
    }
}

trait SaffireProSamplingClkSrcCtlOperation<T: SaffireProioSamplingClockSourceOperation> {
    fn load_src(&mut self, card_cntr: &mut CardCntr) -> Result<Vec<ElemId>, Error> {
        let mut elem_id_list = Vec::new();

        let labels: Vec<&str> = T::SRC_LIST
            .iter()
            .map(|s| sampling_clk_src_to_str(s))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, CLK_SRC_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, true)
            .map(|mut elem_id| elem_id_list.append(&mut elem_id))?;

        Ok(elem_id_list)
    }

    fn read_src(
        &self,
        unit: &mut (SndUnit, FwNode),
        req: &FwReq,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            CLK_SRC_NAME => ElemValueAccessor::<u32>::set_val(elem_value, || {
                T::read_clk_src(req, &unit.1, timeout_ms).map(|idx| idx as u32)
            })
            .map(|_| true),
            _ => Ok(false),
        }
    }

    fn write_src(
        &self,
        unit: &mut (SndUnit, FwNode),
        req: &FwReq,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            CLK_SRC_NAME => {
                unit.0.lock()?;
                let res = ElemValueAccessor::<u32>::get_val(elem_value, |val| {
                    T::write_clk_src(req, &unit.1, val as usize, timeout_ms)
                })
                .map(|_| true);
                let _ = unit.0.unlock();
                res
            }
            _ => Ok(false),
        }
    }
}

const MONITOR_KNOB_VALUE_NAME: &str = "monitor-knob-value";
const MUTE_LED_NAME: &str = "mute-led";
const DIM_LED_NAME: &str = "dim-led";
const EFFECTIVE_CLOCK_SRC_NAME: &str = "effective-clock-source";

trait SaffireProioMeterCtlOperation<T: SaffireProioMeterOperation> {
    fn state(&self) -> &SaffireProioMeterState;
    fn state_mut(&mut self) -> &mut SaffireProioMeterState;

    fn load_state(
        &mut self,
        card_cntr: &mut CardCntr,
        unit: &(SndUnit, FwNode),
        req: &FwReq,
        timeout_ms: u32,
    ) -> Result<Vec<ElemId>, Error> {
        let mut measured_elem_id_list = Vec::new();

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, MONITOR_KNOB_VALUE_NAME, 0);
        card_cntr
            .add_int_elems(&elem_id, 1, 0, u8::MIN as i32, 1, 1, None, false)
            .map(|mut elem_id_list| measured_elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, MUTE_LED_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, 1, 1, false)
            .map(|mut elem_id_list| measured_elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, DIM_LED_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, 1, 1, false)
            .map(|mut elem_id_list| measured_elem_id_list.append(&mut elem_id_list))?;

        let labels: Vec<&str> = T::SRC_LIST
            .iter()
            .map(|s| sampling_clk_src_to_str(s))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, EFFECTIVE_CLOCK_SRC_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, false)
            .map(|mut elem_id| measured_elem_id_list.append(&mut elem_id))?;

        self.measure_state(unit, req, timeout_ms)?;

        Ok(measured_elem_id_list)
    }

    fn measure_state(
        &mut self,
        unit: &(SndUnit, FwNode),
        req: &FwReq,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        T::read_state(req, &unit.1, self.state_mut(), timeout_ms)
    }

    fn read_state(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            MONITOR_KNOB_VALUE_NAME => {
                elem_value.set_int(&[self.state().monitor_knob as i32]);
                Ok(true)
            }
            MUTE_LED_NAME => {
                elem_value.set_bool(&[self.state().mute_led]);
                Ok(true)
            }
            DIM_LED_NAME => {
                elem_value.set_bool(&[self.state().dim_led]);
                Ok(true)
            }
            EFFECTIVE_CLOCK_SRC_NAME => {
                let pos = T::SRC_LIST
                    .iter()
                    .position(|s| s.eq(&self.state().effective_clk_srcs))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

const OUT_MUTE_NAME: &str = "phys-output-mute";
const OUT_VOL_NAME: &str = "phys-output-volume";
const OUT_HWCTL_NAME: &str = "phys-output-hwctl";
const OUT_DIM_NAME: &str = "phys-output-dim";
const OUT_PAD_NAME: &str = "phys-output-pad";

const LEVEL_TLV: DbInterval = DbInterval {
    min: -9600,
    max: 0,
    linear: false,
    mute_avail: false,
};

trait SaffireOutputCtlOperation<
    T: SaffireParametersOperation<SaffireOutputParameters> + SaffireOutputSpecification,
>
{
    const OUTPUT_LABELS: &'static [&'static str];

    fn state(&self) -> &SaffireOutputParameters;
    fn state_mut(&mut self) -> &mut SaffireOutputParameters;

    fn load_params(&mut self, card_cntr: &mut CardCntr) -> Result<Vec<ElemId>, Error> {
        assert_eq!(
            Self::OUTPUT_LABELS.len(),
            T::OFFSETS.len(),
            "Programming error about labels for physical outputs",
        );

        let mut measure_elem_id_list = Vec::new();

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, OUT_MUTE_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, 1, T::MUTE_COUNT, true)
            .map(|mut elem_id_list| measure_elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, OUT_VOL_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                T::LEVEL_MIN as i32,
                T::LEVEL_MAX as i32,
                T::LEVEL_STEP as i32,
                T::VOL_COUNT,
                Some(&Into::<Vec<u32>>::into(LEVEL_TLV)),
                true,
            )
            .map(|mut elem_id_list| measure_elem_id_list.append(&mut elem_id_list))?;

        if T::HWCTL_COUNT > 0 {
            let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, OUT_HWCTL_NAME, 0);
            card_cntr
                .add_bool_elems(&elem_id, 1, T::HWCTL_COUNT, true)
                .map(|mut elem_id_list| measure_elem_id_list.append(&mut elem_id_list))?;
        }

        if T::DIM_COUNT > 0 {
            let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, OUT_DIM_NAME, 0);
            card_cntr
                .add_bool_elems(&elem_id, 1, T::DIM_COUNT, true)
                .map(|mut elem_id_list| measure_elem_id_list.append(&mut elem_id_list))?;
        }

        if T::PAD_COUNT > 0 {
            let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, OUT_PAD_NAME, 0);
            card_cntr
                .add_bool_elems(&elem_id, 1, T::PAD_COUNT, true)
                .map(|mut elem_id_list| measure_elem_id_list.append(&mut elem_id_list))?;
        }

        Ok(measure_elem_id_list)
    }

    fn measure_params(&mut self, req: &FwReq, node: &FwNode, timeout_ms: u32) -> Result<(), Error> {
        T::cache(req, node, self.state_mut(), timeout_ms)
    }

    fn read_params(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            OUT_MUTE_NAME => {
                elem_value.set_bool(&self.state().mutes);
                Ok(true)
            }
            OUT_VOL_NAME => {
                let vals: Vec<i32> = self.state().vols.iter().map(|&val| val as i32).collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            OUT_HWCTL_NAME => {
                elem_value.set_bool(&self.state().hwctls);
                Ok(true)
            }
            OUT_DIM_NAME => {
                elem_value.set_bool(&self.state().dims);
                Ok(true)
            }
            OUT_PAD_NAME => {
                elem_value.set_bool(&self.state().pads);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write_params(
        &mut self,
        req: &FwReq,
        node: &FwNode,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            OUT_MUTE_NAME => {
                let mut params = self.state().clone();
                params
                    .mutes
                    .copy_from_slice(&elem_value.boolean()[..T::MUTE_COUNT]);
                T::update(req, node, &params, self.state_mut(), timeout_ms).map(|_| true)
            }
            OUT_VOL_NAME => {
                let mut params = self.state().clone();
                params
                    .vols
                    .iter_mut()
                    .zip(&elem_value.int()[..T::VOL_COUNT])
                    .for_each(|(vol, val)| *vol = *val as u8);
                T::update(req, node, &params, self.state_mut(), timeout_ms).map(|_| true)
            }
            OUT_HWCTL_NAME => {
                let mut params = self.state().clone();
                params
                    .hwctls
                    .copy_from_slice(&elem_value.boolean()[..T::HWCTL_COUNT]);
                T::update(req, node, &params, self.state_mut(), timeout_ms).map(|_| true)
            }
            OUT_DIM_NAME => {
                let mut params = self.state().clone();
                params
                    .dims
                    .copy_from_slice(&elem_value.boolean()[..T::DIM_COUNT]);
                T::update(req, node, &params, self.state_mut(), timeout_ms).map(|_| true)
            }
            OUT_PAD_NAME => {
                let mut params = self.state().clone();
                params
                    .pads
                    .copy_from_slice(&elem_value.boolean()[..T::PAD_COUNT]);
                T::update(req, node, &params, self.state_mut(), timeout_ms).map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

trait SaffireMixerCtlOperation<T: SaffireMixerOperation>:
    AsRef<SaffireMixerState> + AsMut<SaffireMixerState>
{
    const PHYS_INPUT_GAIN_NAME: &'static str;
    const REVERB_RETURN_GAIN_NAME: &'static str;
    const STREAM_SRC_GAIN_NAME: &'static str;

    const MIXER_MODE: SaffireMixerMode;

    fn load_src_levels(
        &mut self,
        card_cntr: &mut CardCntr,
        mixer_mode: SaffireMixerMode,
        unit: &(SndUnit, FwNode),
        req: &FwReq,
        timeout_ms: u32,
    ) -> Result<Vec<ElemId>, Error> {
        *self.as_mut() = T::create_mixer_state();

        let mut measured_elem_id_list = Vec::new();

        let elem_id =
            ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::PHYS_INPUT_GAIN_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                T::OUTPUT_PAIR_COUNT,
                T::LEVEL_MIN as i32,
                T::LEVEL_MAX as i32,
                T::LEVEL_STEP as i32,
                T::PHYS_INPUT_COUNT,
                Some(&Into::<Vec<u32>>::into(LEVEL_TLV)),
                true,
            )
            .map(|mut elem_id_list| measured_elem_id_list.append(&mut elem_id_list))?;

        let elem_id =
            ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::REVERB_RETURN_GAIN_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                T::OUTPUT_PAIR_COUNT,
                T::LEVEL_MIN as i32,
                T::LEVEL_MAX as i32,
                T::LEVEL_STEP as i32,
                T::REVERB_RETURN_COUNT,
                Some(&Into::<Vec<u32>>::into(LEVEL_TLV)),
                true,
            )
            .map(|mut elem_id_list| measured_elem_id_list.append(&mut elem_id_list))?;

        let elem_id =
            ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::STREAM_SRC_GAIN_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                T::OUTPUT_PAIR_COUNT,
                T::LEVEL_MIN as i32,
                T::LEVEL_MAX as i32,
                T::LEVEL_STEP as i32,
                T::STREAM_INPUT_COUNT,
                Some(&Into::<Vec<u32>>::into(LEVEL_TLV)),
                true,
            )
            .map(|mut elem_id_list| measured_elem_id_list.append(&mut elem_id_list))?;

        if Self::MIXER_MODE == mixer_mode {
            T::read_mixer_state(req, &unit.1, self.as_mut(), timeout_ms)?;
        }

        Ok(measured_elem_id_list)
    }

    fn write_state(
        &mut self,
        unit: &(SndUnit, FwNode),
        req: &FwReq,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        T::write_mixer_state(req, &unit.1, self.as_mut(), timeout_ms)
    }

    fn read_src_levels(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        let name = elem_id.name();

        if name.as_str() == Self::PHYS_INPUT_GAIN_NAME {
            read_mixer_src_levels(elem_value, elem_id, &self.as_ref().phys_inputs)
        } else if name.as_str() == Self::REVERB_RETURN_GAIN_NAME {
            read_mixer_src_levels(elem_value, elem_id, &self.as_ref().reverb_returns)
        } else if name.as_str() == Self::STREAM_SRC_GAIN_NAME {
            read_mixer_src_levels(elem_value, elem_id, &self.as_ref().stream_inputs)
        } else {
            Ok(false)
        }
    }

    fn write_src_levels(
        &mut self,
        mixer_mode: SaffireMixerMode,
        unit: &(SndUnit, FwNode),
        req: &FwReq,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        let name = &elem_id.name();

        if name.as_str() == Self::PHYS_INPUT_GAIN_NAME {
            if Self::MIXER_MODE != mixer_mode {
                Err(Error::new(
                    FileError::Inval,
                    "Not available at current mixer mode",
                ))
            } else {
                let vals = &elem_value.int()[..T::PHYS_INPUT_COUNT];
                let levels: Vec<i16> = vals.iter().fold(Vec::new(), |mut levels, &v| {
                    levels.push(v as i16);
                    levels
                });
                let index = elem_id.index() as usize;
                T::write_phys_inputs(req, &unit.1, index, &levels, self.as_mut(), timeout_ms)
                    .map(|_| true)
            }
        } else if name.as_str() == Self::REVERB_RETURN_GAIN_NAME {
            if Self::MIXER_MODE != mixer_mode {
                Err(Error::new(
                    FileError::Inval,
                    "Not available at current mixer mode",
                ))
            } else {
                let vals = &elem_value.int()[..T::REVERB_RETURN_COUNT];
                let levels: Vec<i16> = vals.iter().fold(Vec::new(), |mut levels, &v| {
                    levels.push(v as i16);
                    levels
                });
                let index = elem_id.index() as usize;
                T::write_reverb_returns(req, &unit.1, index, &levels, self.as_mut(), timeout_ms)
                    .map(|_| true)
            }
        } else if name.as_str() == Self::STREAM_SRC_GAIN_NAME {
            if Self::MIXER_MODE != mixer_mode {
                Err(Error::new(
                    FileError::Inval,
                    "Not available at current mixer mode",
                ))
            } else {
                let vals = &elem_value.int()[..T::STREAM_INPUT_COUNT];
                let levels: Vec<i16> = vals.iter().fold(Vec::new(), |mut levels, &v| {
                    levels.push(v as i16);
                    levels
                });
                let index = elem_id.index() as usize;
                T::write_stream_inputs(req, &unit.1, index, &levels, self.as_mut(), timeout_ms)
                    .map(|_| true)
            }
        } else {
            Ok(false)
        }
    }
}

fn read_mixer_src_levels(
    elem_value: &mut ElemValue,
    elem_id: &ElemId,
    levels_list: &[Vec<i16>],
) -> Result<bool, Error> {
    let index = elem_id.index() as usize;
    levels_list
        .iter()
        .nth(index)
        .ok_or_else(|| {
            let msg = format!("Invalid index of source level list {}", index);
            Error::new(FileError::Inval, &msg)
        })
        .map(|levels| {
            let vals: Vec<i32> = levels.iter().fold(Vec::new(), |mut vals, &level| {
                vals.push(level as i32);
                vals
            });
            elem_value.set_int(&vals);
            true
        })
}

const MIDI_THROUGH_NAME: &str = "MIDI-through";
const AC3_THROUGH_NAME: &str = "AC3-through";

trait SaffireThroughCtlOperation<T: SaffireParametersOperation<SaffireThroughParameters>> {
    fn state(&self) -> &SaffireThroughParameters;
    fn state_mut(&mut self) -> &mut SaffireThroughParameters;

    fn load_params(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, MIDI_THROUGH_NAME, 0);
        card_cntr.add_bool_elems(&elem_id, 1, 1, false)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, AC3_THROUGH_NAME, 0);
        card_cntr.add_bool_elems(&elem_id, 1, 1, false)?;

        Ok(())
    }

    fn read_params(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            MIDI_THROUGH_NAME => {
                elem_value.set_bool(&[self.state().midi]);
                Ok(true)
            }
            AC3_THROUGH_NAME => {
                elem_value.set_bool(&[self.state().ac3]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write_params(
        &mut self,
        req: &FwReq,
        node: &FwNode,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            MIDI_THROUGH_NAME => {
                let mut params = self.state().clone();
                params.midi = elem_value.boolean()[0];
                T::update(req, node, &params, self.state_mut(), timeout_ms).map(|_| true)
            }
            AC3_THROUGH_NAME => {
                let mut params = self.state().clone();
                params.ac3 = elem_value.boolean()[0];
                T::update(req, node, &params, self.state_mut(), timeout_ms).map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

const PRO_MONITOR_ANALOG_INPUT_NAME: &str = "monitor:analog-input";
const PRO_MONITOR_SPDIF_INPUT_NAME: &str = "monitor:spdif-input";
const PRO_MONITOR_ADAT_INPUT_NAME: &str = "monitor:adat-input";

trait SaffireProioMonitorCtlOperation<T: SaffireProioMonitorProtocol> {
    fn state(&self) -> &SaffireProioMonitorParameters;
    fn state_mut(&mut self) -> &mut SaffireProioMonitorParameters;

    fn load_params(
        &mut self,
        card_cntr: &mut CardCntr,
        unit: &(SndUnit, FwNode),
        req: &FwReq,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        *self.state_mut() = T::create_params();

        let elem_id =
            ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, PRO_MONITOR_ANALOG_INPUT_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                self.state().analog_inputs.len(),
                T::LEVEL_MIN as i32,
                T::LEVEL_MAX as i32,
                T::LEVEL_STEP as i32,
                self.state().analog_inputs[0].len(),
                Some(&Into::<Vec<u32>>::into(LEVEL_TLV)),
                true,
            )
            .map(|_| ())?;

        let elem_id =
            ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, PRO_MONITOR_SPDIF_INPUT_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                self.state().spdif_inputs.len(),
                T::LEVEL_MIN as i32,
                T::LEVEL_MAX as i32,
                T::LEVEL_STEP as i32,
                self.state().spdif_inputs[0].len(),
                Some(&Into::<Vec<u32>>::into(LEVEL_TLV)),
                true,
            )
            .map(|_| ())?;

        if let Some(adat_inputs) = self.state().adat_inputs {
            let elem_id =
                ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, PRO_MONITOR_ADAT_INPUT_NAME, 0);
            card_cntr
                .add_int_elems(
                    &elem_id,
                    adat_inputs.len(),
                    T::LEVEL_MIN as i32,
                    T::LEVEL_MAX as i32,
                    T::LEVEL_STEP as i32,
                    adat_inputs[0].len(),
                    Some(&Into::<Vec<u32>>::into(LEVEL_TLV)),
                    true,
                )
                .map(|_| ())?;
        }

        T::read_params(req, &unit.1, self.state_mut(), timeout_ms)
    }

    fn read_params(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            PRO_MONITOR_ANALOG_INPUT_NAME => {
                let idx = elem_id.index() as usize;
                let vals: Vec<i32> = self.state().analog_inputs[idx]
                    .iter()
                    .map(|&val| val as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            PRO_MONITOR_SPDIF_INPUT_NAME => {
                let idx = elem_id.index() as usize;
                let vals: Vec<i32> = self.state().spdif_inputs[idx]
                    .iter()
                    .map(|&val| val as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            PRO_MONITOR_ADAT_INPUT_NAME => {
                if let Some(adat_inputs) = self.state().adat_inputs {
                    let idx = elem_id.index() as usize;
                    let vals: Vec<i32> = adat_inputs[idx].iter().map(|&val| val as i32).collect();
                    elem_value.set_int(&vals);
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            _ => Ok(false),
        }
    }

    fn write_params(
        &mut self,
        unit: &(SndUnit, FwNode),
        req: &FwReq,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            PRO_MONITOR_ANALOG_INPUT_NAME => {
                let idx = elem_id.index() as usize;
                let vals = &elem_value.int()[..self.state().analog_inputs[idx].len()];
                let levels: Vec<i16> = vals.iter().map(|&level| level as i16).collect();
                T::write_analog_inputs(req, &unit.1, idx, &levels, self.state_mut(), timeout_ms)
                    .map(|_| true)
            }
            PRO_MONITOR_SPDIF_INPUT_NAME => {
                let idx = elem_id.index() as usize;
                let vals = &elem_value.int()[..self.state().spdif_inputs[idx].len()];
                let levels: Vec<i16> = vals.iter().map(|&level| level as i16).collect();
                T::write_spdif_inputs(req, &unit.1, idx, &levels, self.state_mut(), timeout_ms)
                    .map(|_| true)
            }
            PRO_MONITOR_ADAT_INPUT_NAME => {
                if T::HAS_ADAT {
                    let vals = &elem_value.int()[..16];
                    let levels: Vec<i16> = vals.iter().map(|&level| level as i16).collect();
                    let idx = elem_id.index() as usize;
                    T::write_adat_inputs(req, &unit.1, idx, &levels, self.state_mut(), timeout_ms)
                        .map(|_| true)
                } else {
                    Ok(false)
                }
            }
            _ => Ok(false),
        }
    }
}

const PRO_MIXER_MONITOR_SRC_NAME: &str = "mixer:monitor-source";
const PRO_MIXER_STREAM_SRC_PAIR_0_NAME: &str = "mixer:stream-source-1/2";
const PRO_MIXER_STREAM_SRC_NAME: &str = "mixer:stream-source";

#[derive(Default, Debug)]
struct SaffireProioMixerCtl(SaffireProioMixerParameters);

impl SaffireProioMixerCtl {
    fn load_params(
        &mut self,
        card_cntr: &mut CardCntr,
        unit: &(SndUnit, FwNode),
        req: &FwReq,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let elem_id =
            ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, PRO_MIXER_MONITOR_SRC_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                SaffireProioMixerProtocol::LEVEL_MIN as i32,
                SaffireProioMixerProtocol::LEVEL_MAX as i32,
                SaffireProioMixerProtocol::LEVEL_STEP as i32,
                self.0.monitor_sources.len(),
                Some(&Into::<Vec<u32>>::into(LEVEL_TLV)),
                true,
            )
            .map(|_| ())?;

        let elem_id = ElemId::new_by_name(
            ElemIfaceType::Mixer,
            0,
            0,
            PRO_MIXER_STREAM_SRC_PAIR_0_NAME,
            0,
        );
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                SaffireProioMixerProtocol::LEVEL_MIN as i32,
                SaffireProioMixerProtocol::LEVEL_MAX as i32,
                SaffireProioMixerProtocol::LEVEL_STEP as i32,
                self.0.stream_source_pair0.len(),
                Some(&Into::<Vec<u32>>::into(LEVEL_TLV)),
                true,
            )
            .map(|_| ())?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, PRO_MIXER_STREAM_SRC_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                SaffireProioMixerProtocol::LEVEL_MIN as i32,
                SaffireProioMixerProtocol::LEVEL_MAX as i32,
                SaffireProioMixerProtocol::LEVEL_STEP as i32,
                self.0.stream_sources.len(),
                Some(&Into::<Vec<u32>>::into(LEVEL_TLV)),
                true,
            )
            .map(|_| ())?;

        SaffireProioMixerProtocol::read_params(req, &unit.1, &mut self.0, timeout_ms)
    }

    fn read_params(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            PRO_MIXER_MONITOR_SRC_NAME => {
                let vals: Vec<i32> = self
                    .0
                    .monitor_sources
                    .iter()
                    .map(|&val| val as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            PRO_MIXER_STREAM_SRC_PAIR_0_NAME => {
                let vals: Vec<i32> = self
                    .0
                    .stream_source_pair0
                    .iter()
                    .map(|&val| val as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            PRO_MIXER_STREAM_SRC_NAME => {
                let vals: Vec<i32> = self
                    .0
                    .stream_sources
                    .iter()
                    .map(|&val| val as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write_params(
        &mut self,
        unit: &(SndUnit, FwNode),
        req: &FwReq,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            PRO_MIXER_MONITOR_SRC_NAME => {
                let vals = &elem_value.int()[..self.0.monitor_sources.len()];
                let levels: Vec<i16> = vals.iter().map(|&level| level as i16).collect();
                SaffireProioMixerProtocol::write_monitor_sources(
                    req,
                    &unit.1,
                    &levels,
                    &mut self.0,
                    timeout_ms,
                )
                .map(|_| true)
            }
            PRO_MIXER_STREAM_SRC_PAIR_0_NAME => {
                let vals = &elem_value.int()[..self.0.stream_source_pair0.len()];
                let levels: Vec<i16> = vals.iter().map(|&level| level as i16).collect();
                SaffireProioMixerProtocol::write_stream_source_pair0(
                    req,
                    &unit.1,
                    &levels,
                    &mut self.0,
                    timeout_ms,
                )
                .map(|_| true)
            }
            PRO_MIXER_STREAM_SRC_NAME => {
                let vals = &elem_value.int()[..self.0.stream_sources.len()];
                let levels: Vec<i16> = vals.iter().map(|&level| level as i16).collect();
                SaffireProioMixerProtocol::write_stream_sources(
                    req,
                    &unit.1,
                    &levels,
                    &mut self.0,
                    timeout_ms,
                )
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

const HEAD_ROOM_NAME: &str = "head-room";
const PHANTOM_POWERING_NAME: &str = "phantom-powering";
const INSERT_SWAP_NAME: &str = "insert-swap";
const STANDALONE_MODE_NAME: &str = "standalone-mode";
const ADAT_ENABLE_NAME: &str = "adat-enable";
const DIRECT_MONITORING_NAME: &str = "direct-monitoring";

fn standalone_mode_to_str(mode: &SaffireProioStandaloneMode) -> &str {
    match mode {
        SaffireProioStandaloneMode::Mix => "mix",
        SaffireProioStandaloneMode::Track => "track",
    }
}

trait SaffireProioSpecificCtlOperation<T: SaffireProioSpecificOperation> {
    const STANDALONE_MODES: [SaffireProioStandaloneMode; 2] = [
        SaffireProioStandaloneMode::Mix,
        SaffireProioStandaloneMode::Track,
    ];

    fn state(&self) -> &SaffireProioSpecificParameters;
    fn state_mut(&mut self) -> &mut SaffireProioSpecificParameters;

    fn load_params(
        &mut self,
        card_cntr: &mut CardCntr,
        unit: &(SndUnit, FwNode),
        req: &FwReq,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, HEAD_ROOM_NAME, 0);
        card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        if T::PHANTOM_POWERING_COUNT > 0 {
            let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, PHANTOM_POWERING_NAME, 0);
            card_cntr.add_bool_elems(&elem_id, 1, 2, true)?;
        }

        if T::INSERT_SWAP_COUNT > 0 {
            let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, INSERT_SWAP_NAME, 0);
            card_cntr.add_bool_elems(&elem_id, 1, 2, true)?;
        }

        let labels: Vec<&str> = Self::STANDALONE_MODES
            .iter()
            .map(|m| standalone_mode_to_str(m))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, STANDALONE_MODE_NAME, 0);
        card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, ADAT_ENABLE_NAME, 0);
        card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, DIRECT_MONITORING_NAME, 0);
        card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        *self.state_mut() = T::create_params();
        T::read_params(req, &unit.1, self.state_mut(), timeout_ms)
    }

    fn read_params(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            HEAD_ROOM_NAME => {
                elem_value.set_bool(&[self.state().head_room]);
                Ok(true)
            }
            PHANTOM_POWERING_NAME => {
                elem_value.set_bool(&self.state().phantom_powerings);
                Ok(true)
            }
            INSERT_SWAP_NAME => {
                elem_value.set_bool(&self.state().insert_swaps);
                Ok(true)
            }
            STANDALONE_MODE_NAME => {
                let pos = Self::STANDALONE_MODES
                    .iter()
                    .position(|m| m.eq(&self.state().standalone_mode))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            ADAT_ENABLE_NAME => {
                elem_value.set_bool(&[self.state().adat_enabled]);
                Ok(true)
            }
            DIRECT_MONITORING_NAME => {
                elem_value.set_bool(&[self.state().direct_monitoring]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write_params(
        &mut self,
        unit: &(SndUnit, FwNode),
        req: &FwReq,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            HEAD_ROOM_NAME => {
                let val = elem_value.boolean()[0];
                T::write_head_room(req, &unit.1, val, self.state_mut(), timeout_ms).map(|_| true)
            }
            PHANTOM_POWERING_NAME => {
                let vals = &elem_value.boolean()[..self.state().phantom_powerings.len()];
                T::write_phantom_powerings(req, &unit.1, &vals, self.state_mut(), timeout_ms)
                    .map(|_| true)
            }
            INSERT_SWAP_NAME => {
                let vals = &elem_value.boolean()[..self.state().insert_swaps.len()];
                T::write_insert_swaps(req, &unit.1, &vals, self.state_mut(), timeout_ms)
                    .map(|_| true)
            }
            STANDALONE_MODE_NAME => {
                let val = elem_value.enumerated()[0];
                let &mode = Self::STANDALONE_MODES
                    .iter()
                    .nth(val as usize)
                    .ok_or_else(|| {
                        let msg = format!("Invalid index of standalone mode: {}", val);
                        Error::new(FileError::Inval, &msg)
                    })?;
                T::write_standalone_mode(req, &unit.1, mode, self.state_mut(), timeout_ms)
                    .map(|_| true)
            }
            ADAT_ENABLE_NAME => {
                let val = elem_value.boolean()[0];
                T::write_adat_enable(req, &unit.1, val, self.state_mut(), timeout_ms).map(|_| true)
            }
            DIRECT_MONITORING_NAME => {
                let val = elem_value.boolean()[0];
                T::write_direct_monitoring(req, &unit.1, val, self.state_mut(), timeout_ms)
                    .map(|_| true)
            }
            _ => Ok(false),
        }
    }
}
