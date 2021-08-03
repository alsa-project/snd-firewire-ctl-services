// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

pub mod saffirepro26io_model;
pub mod saffirepro10io_model;

pub mod saffire_model;
pub mod saffirele_model;

use glib::{Error, FileError};

use hinawa::FwReq;
use hinawa::{SndUnit, SndUnitExt};

use alsactl::{ElemId, ElemIfaceType, ElemValue, ElemValueExt, ElemValueExtManual};

use alsa_ctl_tlv_codec::items::DbInterval;

use core::card_cntr::CardCntr;
use core::elem_value_accessor::ElemValueAccessor;

use bebob_protocols::focusrite::{*, saffire::*, saffireproio::*};

use crate::model::{CLK_RATE_NAME, CLK_SRC_NAME};

trait SaffireProMediaClkFreqCtlOperation<T: SaffireProioMediaClockFrequencyOperation> {
    fn load_freq(&mut self, card_cntr: &mut CardCntr) -> Result<Vec<ElemId>, Error> {
        let labels: Vec<String> = T::FREQ_LIST.iter().map(|&r| r.to_string()).collect();

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, CLK_RATE_NAME, 0);
        card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)
    }

    fn read_freq(
        &self,
        unit: &SndUnit,
        req: &FwReq,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            CLK_RATE_NAME => ElemValueAccessor::<u32>::set_val(elem_value, || {
                T::read_clk_freq(req, &unit.get_node(), timeout_ms).map(|idx| idx as u32)
            })
            .map(|_| true),
            _ => Ok(false),
        }
    }

    fn write_freq(
        &self,
        unit: &mut SndUnit,
        req: &FwReq,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            CLK_RATE_NAME => {
                unit.lock()?;
                let res = ElemValueAccessor::<u32>::get_val(elem_value, |val| {
                    T::write_clk_freq(req, &unit.get_node(), val as usize, timeout_ms)
                })
                .map(|_| true);
                let _ = unit.unlock();
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

        let labels: Vec<&str> = T::SRC_LIST.iter()
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
        unit: &mut SndUnit,
        req: &FwReq,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            CLK_SRC_NAME => ElemValueAccessor::<u32>::set_val(elem_value, || {
                T::read_clk_src(req, &unit.get_node(), timeout_ms).map(|idx| idx as u32)
            })
            .map(|_| true),
            _ => Ok(false),
        }
    }

    fn write_src(
        &self,
        unit: &mut SndUnit,
        req: &FwReq,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            CLK_SRC_NAME => {
                unit.lock()?;
                let res = ElemValueAccessor::<u32>::get_val(elem_value, |val| {
                    T::write_clk_src(req, &unit.get_node(), val as usize, timeout_ms)
                })
                .map(|_| true);
                let _ = unit.unlock();
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
        unit: &SndUnit,
        req: &FwReq,
        timeout_ms: u32,
    ) -> Result<Vec<ElemId>, Error> {
        let mut measured_elem_id_list = Vec::new();

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MONITOR_KNOB_VALUE_NAME, 0);
        card_cntr.add_int_elems(
            &elem_id,
            1,
            0,
            u8::MIN as i32,
            1,
            1,
            None,
        false)
            .map(|mut elem_id_list| measured_elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, MUTE_LED_NAME, 0);
        card_cntr.add_bool_elems(&elem_id, 1, 1, false)
            .map(|mut elem_id_list| measured_elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, DIM_LED_NAME, 0);
        card_cntr.add_bool_elems(&elem_id, 1, 1, false)
            .map(|mut elem_id_list| measured_elem_id_list.append(&mut elem_id_list))?;

        let labels: Vec<&str> = T::SRC_LIST.iter()
            .map(|s| sampling_clk_src_to_str(s))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, CLK_SRC_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, false)
            .map(|mut elem_id| measured_elem_id_list.append(&mut elem_id))?;

        self.measure_state(unit, req, timeout_ms)?;

        Ok(measured_elem_id_list)
    }

    fn measure_state(
        &mut self,
        unit: &SndUnit,
        req: &FwReq,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        T::read_state(req, &unit.get_node(), self.as_mut(), timeout_ms)
    }

    fn read_state(
        &self,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
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
                let pos = T::SRC_LIST.iter()
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
        unit: &SndUnit,
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

        self.measure_params(unit, req, timeout_ms)?;

        Ok(measure_elem_id_list)
    }

    fn measure_params(&mut self, unit: &SndUnit, req: &FwReq, timeout_ms: u32) -> Result<(), Error> {
        T::read_output_parameters(req, &unit.get_node(), self.as_mut(), timeout_ms)
    }

    fn read_params(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            OUT_MUTE_NAME => {
                elem_value.set_bool(&self.as_ref().mutes);
                Ok(true)
            }
            OUT_VOL_NAME => {
                let vals: Vec<i32> = self.as_ref().vols.iter()
                    .map(|&val| val as i32)
                    .collect();
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
        unit: &SndUnit,
        req: &FwReq,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            OUT_MUTE_NAME => {
                let mut vals = self.as_ref().mutes.clone();
                elem_value.get_bool(&mut vals);
                T::write_mutes(req, &unit.get_node(), &vals, self.as_mut(), timeout_ms)
                    .map(|_| true)
            }
            OUT_VOL_NAME => {
                let mut vals = vec![Default::default(); self.as_ref().vols.len()];
                elem_value.get_int(&mut vals);
                let vols: Vec<u8> = vals.iter().map(|&vol| vol as u8).collect();
                T::write_vols(req, &unit.get_node(), &vols, self.as_mut(), timeout_ms)
                    .map(|_| true)
            }
            OUT_HWCTL_NAME => {
                let mut vals = self.as_ref().hwctls.clone();
                elem_value.get_bool(&mut vals);
                T::write_hwctls(req, &unit.get_node(), &vals, self.as_mut(), timeout_ms)
                    .map(|_| true)
            }
            OUT_DIM_NAME => {
                let mut vals = self.as_ref().dims.clone();
                elem_value.get_bool(&mut vals);
                T::write_dims(req, &unit.get_node(), &vals, self.as_mut(), timeout_ms)
                    .map(|_| true)
            }
            OUT_PAD_NAME => {
                let mut vals = self.as_ref().pads.clone();
                elem_value.get_bool(&mut vals);
                T::write_pads(req, &unit.get_node(), &vals, self.as_mut(), timeout_ms)
                    .map(|_| true)
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
        unit: &SndUnit,
        req: &FwReq,
        timeout_ms: u32,
    ) -> Result<Vec<ElemId>, Error> {
        *self.as_mut() = T::create_mixer_state();

        let mut measured_elem_id_list = Vec::new();

        let elem_id = ElemId::new_by_name(
            ElemIfaceType::Mixer,
            0,
            0,
            Self::PHYS_INPUT_GAIN_NAME,
            0,
        );
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

        let elem_id = ElemId::new_by_name(
            ElemIfaceType::Mixer,
            0,
            0,
            Self::REVERB_RETURN_GAIN_NAME,
            0,
        );
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

        let elem_id = ElemId::new_by_name(
            ElemIfaceType::Mixer,
            0,
            0,
            Self::STREAM_SRC_GAIN_NAME,
            0,
        );
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
            T::read_mixer_state(req, &unit.get_node(), self.as_mut(), timeout_ms)?;
        }

        Ok(measured_elem_id_list)
    }

    fn write_state(&mut self, unit: &SndUnit, req: &FwReq, timeout_ms: u32) -> Result<(), Error> {
        T::write_mixer_state(req, &unit.get_node(), self.as_mut(), timeout_ms)
    }

    fn read_src_levels(
        &self,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
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
        unit: &SndUnit,
        req: &FwReq,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        let name = &elem_id.get_name();

        if name.as_str() == Self::PHYS_INPUT_GAIN_NAME {
            if Self::MIXER_MODE != mixer_mode {
                Err(Error::new(FileError::Inval, "Not available at current mixer mode"))
            } else {
                let mut vals = vec![0i32; T::PHYS_INPUT_COUNT];
                elem_value.get_int(&mut vals);
                let levels: Vec<i16> = vals.iter().fold(Vec::new(), |mut levels, &v| {
                    levels.push(v as i16);
                    levels
                });
                let index = elem_id.get_index() as usize;
                T::write_phys_inputs(req, &unit.get_node(), index, &levels, self.as_mut(),
                                     timeout_ms)
                    .map(|_| true)
            }
        } else if name.as_str() == Self::REVERB_RETURN_GAIN_NAME {
            if Self::MIXER_MODE != mixer_mode {
                Err(Error::new(FileError::Inval, "Not available at current mixer mode"))
            } else {
                let mut vals = vec![0i32; T::REVERB_RETURN_COUNT];
                elem_value.get_int(&mut vals);
                let levels: Vec<i16> = vals.iter().fold(Vec::new(), |mut levels, &v| {
                    levels.push(v as i16);
                    levels
                });
                let index = elem_id.get_index() as usize;
                T::write_reverb_returns(req, &unit.get_node(), index, &levels, self.as_mut(),
                                        timeout_ms)
                    .map(|_| true)
            }
        } else if name.as_str() == Self::STREAM_SRC_GAIN_NAME {
            if Self::MIXER_MODE != mixer_mode {
                Err(Error::new(FileError::Inval, "Not available at current mixer mode"))
            } else {
                let mut vals = vec![0i32; T::STREAM_INPUT_COUNT];
                elem_value.get_int(&mut vals);
                let levels: Vec<i16> = vals.iter().fold(Vec::new(), |mut levels, &v| {
                    levels.push(v as i16);
                    levels
                });
                let index = elem_id.get_index() as usize;
                T::write_stream_inputs(req, &unit.get_node(), index, &levels, self.as_mut(),
                                       timeout_ms)
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
    levels_list.iter()
        .nth(index)
        .ok_or_else(|| {
            let msg = format!("Invalid index of source level list {}", index);
            Error::new(FileError::Inval, &msg)
        })
        .map(|levels| {
            let vals: Vec<i32> = levels.iter()
                .fold(Vec::new(), |mut vals, &level| {
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
        let elem_id = ElemId::new_by_name(
            ElemIfaceType::Card,
            0,
            0,
            MIDI_THROUGH_NAME,
            0,
        );
        card_cntr
            .add_bool_elems(&elem_id, 1, 1, false)?;

        let elem_id = ElemId::new_by_name(
            ElemIfaceType::Card,
            0,
            0,
            AC3_THROUGH_NAME,
            0,
        );
        card_cntr
            .add_bool_elems(&elem_id, 1, 1, false)?;

        Ok(())
    }

    fn read_params(
        &self,
        unit: &SndUnit,
        req: &FwReq,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            MIDI_THROUGH_NAME => {
                let mut val = false;
                T::read_midi_through(req, &unit.get_node(), &mut val, timeout_ms)?;
                elem_value.set_bool(&[val]);
                Ok(true)
            }
            AC3_THROUGH_NAME => {
                let mut val = false;
                T::read_ac3_through(req, &unit.get_node(), &mut val, timeout_ms)?;
                elem_value.set_bool(&[val]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write_params(
        &mut self,
        unit: &SndUnit,
        req: &FwReq,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            MIDI_THROUGH_NAME => {
                let mut vals = [false];
                elem_value.get_bool(&mut vals);
                T::write_midi_through(req, &unit.get_node(), vals[0], timeout_ms)
                    .map(|_| true)
            }
            AC3_THROUGH_NAME => {
                let mut vals = [false];
                elem_value.get_bool(&mut vals);
                T::write_ac3_through(req, &unit.get_node(), vals[0], timeout_ms)
                    .map(|_| true)
            }
            _ => Ok(false),
        }
    }
}
