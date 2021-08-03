// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

pub mod saffirepro26io_model;
pub mod saffirepro10io_model;

pub mod saffire_model;
pub mod saffirele_model;

use glib::Error;

use hinawa::FwReq;
use hinawa::{SndUnit, SndUnitExt};

use alsactl::{ElemId, ElemIfaceType, ElemValue, ElemValueExt, ElemValueExtManual};

use alsa_ctl_tlv_codec::items::DbInterval;

use core::card_cntr::CardCntr;
use core::elem_value_accessor::ElemValueAccessor;

use bebob_protocols::focusrite::{*, saffireproio::*};

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
