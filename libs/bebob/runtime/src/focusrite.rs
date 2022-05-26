// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

pub mod saffirepro10io_model;
pub mod saffirepro26io_model;

pub mod saffire_model;
pub mod saffirele_model;

use {
    super::{common_ctls::*, *},
    bebob_protocols::{
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
        match elem_id.get_name().as_str() {
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
        match elem_id.get_name().as_str() {
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
        match elem_id.get_name().as_str() {
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
        match elem_id.get_name().as_str() {
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

trait SaffireProioMeterCtlOperation<T: SaffireProioMeterOperation>:
    AsRef<SaffireProioMeterState> + AsMut<SaffireProioMeterState>
{
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
        T::read_state(req, &unit.1, self.as_mut(), timeout_ms)
    }

    fn read_state(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            MONITOR_KNOB_VALUE_NAME => {
                elem_value.set_int(&[self.as_ref().monitor_knob as i32]);
                Ok(true)
            }
            MUTE_LED_NAME => {
                elem_value.set_bool(&[self.as_ref().mute_led]);
                Ok(true)
            }
            DIM_LED_NAME => {
                elem_value.set_bool(&[self.as_ref().dim_led]);
                Ok(true)
            }
            EFFECTIVE_CLOCK_SRC_NAME => {
                let pos = T::SRC_LIST
                    .iter()
                    .position(|s| s.eq(&self.as_ref().effective_clk_srcs))
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

trait SaffireOutputCtlOperation<T: SaffireOutputOperation>:
    AsRef<SaffireOutputParameters> + AsMut<SaffireOutputParameters>
{
    const OUTPUT_LABELS: &'static [&'static str];

    fn load_params(
        &mut self,
        card_cntr: &mut CardCntr,
        unit: &(SndUnit, FwNode),
        req: &FwReq,
        timeout_ms: u32,
    ) -> Result<Vec<ElemId>, Error> {
        assert_eq!(
            Self::OUTPUT_LABELS.len(),
            T::OFFSETS.len(),
            "Programming error about labels for physical outputs",
        );

        *self.as_mut() = T::create_output_parameters();

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

        T::read_output_parameters(req, &unit.1, self.as_mut(), true, timeout_ms)?;

        Ok(measure_elem_id_list)
    }

    fn measure_params(
        &mut self,
        unit: &(SndUnit, FwNode),
        req: &FwReq,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        T::read_output_parameters(req, &unit.1, self.as_mut(), false, timeout_ms)
    }

    fn read_params(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            OUT_MUTE_NAME => {
                elem_value.set_bool(&self.as_ref().mutes);
                Ok(true)
            }
            OUT_VOL_NAME => {
                let vals: Vec<i32> = self.as_ref().vols.iter().map(|&val| val as i32).collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            OUT_HWCTL_NAME => {
                elem_value.set_bool(&self.as_ref().hwctls);
                Ok(true)
            }
            OUT_DIM_NAME => {
                elem_value.set_bool(&self.as_ref().dims);
                Ok(true)
            }
            OUT_PAD_NAME => {
                elem_value.set_bool(&self.as_ref().pads);
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
        match elem_id.get_name().as_str() {
            OUT_MUTE_NAME => {
                let mut vals = self.as_ref().mutes.clone();
                elem_value.get_bool(&mut vals);
                T::write_mutes(req, &unit.1, &vals, self.as_mut(), timeout_ms).map(|_| true)
            }
            OUT_VOL_NAME => {
                let mut vals = vec![Default::default(); self.as_ref().vols.len()];
                elem_value.get_int(&mut vals);
                let vols: Vec<u8> = vals.iter().map(|&vol| vol as u8).collect();
                T::write_vols(req, &unit.1, &vols, self.as_mut(), timeout_ms).map(|_| true)
            }
            OUT_HWCTL_NAME => {
                let mut vals = self.as_ref().hwctls.clone();
                elem_value.get_bool(&mut vals);
                T::write_hwctls(req, &unit.1, &vals, self.as_mut(), timeout_ms).map(|_| true)
            }
            OUT_DIM_NAME => {
                let mut vals = self.as_ref().dims.clone();
                elem_value.get_bool(&mut vals);
                T::write_dims(req, &unit.1, &vals, self.as_mut(), timeout_ms).map(|_| true)
            }
            OUT_PAD_NAME => {
                let mut vals = self.as_ref().pads.clone();
                elem_value.get_bool(&mut vals);
                T::write_pads(req, &unit.1, &vals, self.as_mut(), timeout_ms).map(|_| true)
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
        let name = elem_id.get_name();

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
        let name = &elem_id.get_name();

        if name.as_str() == Self::PHYS_INPUT_GAIN_NAME {
            if Self::MIXER_MODE != mixer_mode {
                Err(Error::new(
                    FileError::Inval,
                    "Not available at current mixer mode",
                ))
            } else {
                let mut vals = vec![0i32; T::PHYS_INPUT_COUNT];
                elem_value.get_int(&mut vals);
                let levels: Vec<i16> = vals.iter().fold(Vec::new(), |mut levels, &v| {
                    levels.push(v as i16);
                    levels
                });
                let index = elem_id.get_index() as usize;
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
                let mut vals = vec![0i32; T::REVERB_RETURN_COUNT];
                elem_value.get_int(&mut vals);
                let levels: Vec<i16> = vals.iter().fold(Vec::new(), |mut levels, &v| {
                    levels.push(v as i16);
                    levels
                });
                let index = elem_id.get_index() as usize;
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
                let mut vals = vec![0i32; T::STREAM_INPUT_COUNT];
                elem_value.get_int(&mut vals);
                let levels: Vec<i16> = vals.iter().fold(Vec::new(), |mut levels, &v| {
                    levels.push(v as i16);
                    levels
                });
                let index = elem_id.get_index() as usize;
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
    let index = elem_id.get_index() as usize;
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

trait SaffireThroughCtlOperation<T: SaffireThroughOperation> {
    fn load_params(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, MIDI_THROUGH_NAME, 0);
        card_cntr.add_bool_elems(&elem_id, 1, 1, false)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, AC3_THROUGH_NAME, 0);
        card_cntr.add_bool_elems(&elem_id, 1, 1, false)?;

        Ok(())
    }

    fn read_params(
        &self,
        unit: &(SndUnit, FwNode),
        req: &FwReq,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            MIDI_THROUGH_NAME => {
                let mut val = false;
                T::read_midi_through(req, &unit.1, &mut val, timeout_ms)?;
                elem_value.set_bool(&[val]);
                Ok(true)
            }
            AC3_THROUGH_NAME => {
                let mut val = false;
                T::read_ac3_through(req, &unit.1, &mut val, timeout_ms)?;
                elem_value.set_bool(&[val]);
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
        match elem_id.get_name().as_str() {
            MIDI_THROUGH_NAME => {
                let mut vals = [false];
                elem_value.get_bool(&mut vals);
                T::write_midi_through(req, &unit.1, vals[0], timeout_ms).map(|_| true)
            }
            AC3_THROUGH_NAME => {
                let mut vals = [false];
                elem_value.get_bool(&mut vals);
                T::write_ac3_through(req, &unit.1, vals[0], timeout_ms).map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

const PRO_MONITOR_ANALOG_INPUT_NAME: &str = "monitor:analog-input";
const PRO_MONITOR_SPDIF_INPUT_NAME: &str = "monitor:spdif-input";
const PRO_MONITOR_ADAT_INPUT_NAME: &str = "monitor:adat-input";

trait SaffireProioMonitorCtlOperation<T: SaffireProioMonitorProtocol>:
    AsRef<SaffireProioMonitorParameters> + AsMut<SaffireProioMonitorParameters>
{
    fn load_params(
        &mut self,
        card_cntr: &mut CardCntr,
        unit: &(SndUnit, FwNode),
        req: &FwReq,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        *self.as_mut() = T::create_params();

        let elem_id =
            ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, PRO_MONITOR_ANALOG_INPUT_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                self.as_ref().analog_inputs.len(),
                T::LEVEL_MIN as i32,
                T::LEVEL_MAX as i32,
                T::LEVEL_STEP as i32,
                self.as_ref().analog_inputs[0].len(),
                Some(&Into::<Vec<u32>>::into(LEVEL_TLV)),
                true,
            )
            .map(|_| ())?;

        let elem_id =
            ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, PRO_MONITOR_SPDIF_INPUT_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                self.as_ref().spdif_inputs.len(),
                T::LEVEL_MIN as i32,
                T::LEVEL_MAX as i32,
                T::LEVEL_STEP as i32,
                self.as_ref().spdif_inputs[0].len(),
                Some(&Into::<Vec<u32>>::into(LEVEL_TLV)),
                true,
            )
            .map(|_| ())?;

        if let Some(adat_inputs) = self.as_ref().adat_inputs {
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

        T::read_params(req, &unit.1, self.as_mut(), timeout_ms)
    }

    fn read_params(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            PRO_MONITOR_ANALOG_INPUT_NAME => {
                let idx = elem_id.get_index() as usize;
                let vals: Vec<i32> = self.as_ref().analog_inputs[idx]
                    .iter()
                    .map(|&val| val as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            PRO_MONITOR_SPDIF_INPUT_NAME => {
                let idx = elem_id.get_index() as usize;
                let vals: Vec<i32> = self.as_ref().spdif_inputs[idx]
                    .iter()
                    .map(|&val| val as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            PRO_MONITOR_ADAT_INPUT_NAME => {
                if let Some(adat_inputs) = self.as_ref().adat_inputs {
                    let idx = elem_id.get_index() as usize;
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
        match elem_id.get_name().as_str() {
            PRO_MONITOR_ANALOG_INPUT_NAME => {
                let idx = elem_id.get_index() as usize;
                let mut vals = vec![0; self.as_ref().analog_inputs[idx].len()];
                elem_value.get_int(&mut vals);
                let levels: Vec<i16> = vals.iter().map(|&level| level as i16).collect();
                T::write_analog_inputs(req, &unit.1, idx, &levels, self.as_mut(), timeout_ms)
                    .map(|_| true)
            }
            PRO_MONITOR_SPDIF_INPUT_NAME => {
                let idx = elem_id.get_index() as usize;
                let mut vals = vec![0; self.as_ref().spdif_inputs[idx].len()];
                elem_value.get_int(&mut vals);
                let levels: Vec<i16> = vals.iter().map(|&level| level as i16).collect();
                T::write_spdif_inputs(req, &unit.1, idx, &levels, self.as_mut(), timeout_ms)
                    .map(|_| true)
            }
            PRO_MONITOR_ADAT_INPUT_NAME => {
                if T::HAS_ADAT {
                    let mut vals = vec![0; 16];
                    elem_value.get_int(&mut vals);
                    let levels: Vec<i16> = vals.iter().map(|&level| level as i16).collect();
                    let idx = elem_id.get_index() as usize;
                    T::write_adat_inputs(req, &unit.1, idx, &levels, self.as_mut(), timeout_ms)
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
        match elem_id.get_name().as_str() {
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
        match elem_id.get_name().as_str() {
            PRO_MIXER_MONITOR_SRC_NAME => {
                let mut vals = vec![0; self.0.monitor_sources.len()];
                elem_value.get_int(&mut vals);
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
                let mut vals = vec![0; self.0.stream_source_pair0.len()];
                elem_value.get_int(&mut vals);
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
                let mut vals = vec![0; self.0.stream_sources.len()];
                elem_value.get_int(&mut vals);
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

trait SaffireProioSpecificCtlOperation<T: SaffireProioSpecificOperation>:
    AsRef<SaffireProioSpecificParameters> + AsMut<SaffireProioSpecificParameters>
{
    const STANDALONE_MODES: [SaffireProioStandaloneMode; 2] = [
        SaffireProioStandaloneMode::Mix,
        SaffireProioStandaloneMode::Track,
    ];

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

        *self.as_mut() = T::create_params();
        T::read_params(req, &unit.1, self.as_mut(), timeout_ms)
    }

    fn read_params(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            HEAD_ROOM_NAME => {
                elem_value.set_bool(&[self.as_ref().head_room]);
                Ok(true)
            }
            PHANTOM_POWERING_NAME => {
                elem_value.set_bool(&self.as_ref().phantom_powerings);
                Ok(true)
            }
            INSERT_SWAP_NAME => {
                elem_value.set_bool(&self.as_ref().insert_swaps);
                Ok(true)
            }
            STANDALONE_MODE_NAME => {
                let pos = Self::STANDALONE_MODES
                    .iter()
                    .position(|m| m.eq(&self.as_ref().standalone_mode))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            ADAT_ENABLE_NAME => {
                elem_value.set_bool(&[self.as_ref().adat_enabled]);
                Ok(true)
            }
            DIRECT_MONITORING_NAME => {
                elem_value.set_bool(&[self.as_ref().direct_monitoring]);
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
        match elem_id.get_name().as_str() {
            HEAD_ROOM_NAME => {
                let mut vals = [false];
                elem_value.get_bool(&mut vals);
                T::write_head_room(req, &unit.1, vals[0], self.as_mut(), timeout_ms).map(|_| true)
            }
            PHANTOM_POWERING_NAME => {
                let mut vals = self.as_ref().phantom_powerings.clone();
                elem_value.get_bool(&mut vals);
                T::write_phantom_powerings(req, &unit.1, &vals, self.as_mut(), timeout_ms)
                    .map(|_| true)
            }
            INSERT_SWAP_NAME => {
                let mut vals = self.as_ref().insert_swaps.clone();
                elem_value.get_bool(&mut vals);
                T::write_insert_swaps(req, &unit.1, &vals, self.as_mut(), timeout_ms).map(|_| true)
            }
            STANDALONE_MODE_NAME => {
                let mut vals = [0];
                elem_value.get_enum(&mut vals);
                let &mode = Self::STANDALONE_MODES
                    .iter()
                    .nth(vals[0] as usize)
                    .ok_or_else(|| {
                        let msg = format!("Invalid index of standalone mode: {}", vals[0]);
                        Error::new(FileError::Inval, &msg)
                    })?;
                T::write_standalone_mode(req, &unit.1, mode, self.as_mut(), timeout_ms)
                    .map(|_| true)
            }
            ADAT_ENABLE_NAME => {
                let mut vals = [false];
                elem_value.get_bool(&mut vals);
                T::write_adat_enable(req, &unit.1, vals[0], self.as_mut(), timeout_ms).map(|_| true)
            }
            DIRECT_MONITORING_NAME => {
                let mut vals = [false];
                elem_value.get_bool(&mut vals);
                T::write_direct_monitoring(req, &unit.1, vals[0], self.as_mut(), timeout_ms)
                    .map(|_| true)
            }
            _ => Ok(false),
        }
    }
}
