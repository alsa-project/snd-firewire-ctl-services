// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

use glib::Error;

use hinawa::FwFcpExt;
use hinawa::{SndUnit, SndUnitExt};

use alsactl::{ElemId, ElemIfaceType, ElemValue, ElemValueExt};

use core::card_cntr::*;
use core::elem_value_accessor::ElemValueAccessor;

use alsa_ctl_tlv_codec::items::DbInterval;

use bebob_protocols::{*, apogee::ensemble::*};

use crate::model::{IN_METER_NAME, OUT_METER_NAME};

use crate::common_ctls::*;
use super::apogee_ctls::{HwCtl, DisplayCtl, OpticalCtl, InputCtl, OutputCtl, MixerCtl, RouteCtl, ResamplerCtl};

const FCP_TIMEOUT_MS: u32 = 100;

pub struct EnsembleModel {
    avc: BebobAvc,
    clk_ctl: ClkCtl,
    meter_ctl: MeterCtl,
    hw_ctls: HwCtl,
    display_ctls: DisplayCtl,
    opt_iface_ctls: OpticalCtl,
    input_ctls: InputCtl,
    out_ctls: OutputCtl,
    mixer_ctls: MixerCtl,
    route_ctls: RouteCtl,
    resampler_ctls: ResamplerCtl,
}

#[derive(Default)]
struct ClkCtl(Vec<ElemId>);

impl MediaClkFreqCtlOperation<EnsembleClkProtocol> for ClkCtl {}

impl SamplingClkSrcCtlOperation<EnsembleClkProtocol> for ClkCtl {
    const SRC_LABELS: &'static [&'static str] = &[
        "Internal",
        "S/PDIF-coax",
        "Optical",
        "Word Clock",
    ];
}

impl Default for EnsembleModel {
    fn default() -> Self {
        Self {
            avc: Default::default(),
            clk_ctl: Default::default(),
            meter_ctl: Default::default(),
            hw_ctls: HwCtl::new(),
            display_ctls: DisplayCtl::new(),
            opt_iface_ctls: OpticalCtl::new(),
            input_ctls: InputCtl::new(),
            out_ctls: OutputCtl::new(),
            mixer_ctls: MixerCtl::new(),
            route_ctls: RouteCtl::new(),
            resampler_ctls: ResamplerCtl::new(),
        }
    }
}

impl CtlModel<SndUnit> for EnsembleModel {
    fn load(&mut self, unit: &mut SndUnit, card_cntr: &mut CardCntr)
        -> Result<(), Error>
    {
        self.avc.as_ref().bind(&unit.get_node())?;

        self.clk_ctl.load_freq(card_cntr)
            .map(|mut elem_id_list| self.clk_ctl.0.append(&mut elem_id_list))?;

        self.clk_ctl.load_src(card_cntr)
            .map(|mut elem_id_list| self.clk_ctl.0.append(&mut elem_id_list))?;

        self.meter_ctl.load_state(card_cntr, &mut self.avc, FCP_TIMEOUT_MS)?;

        self.hw_ctls.load(&self.avc, card_cntr, FCP_TIMEOUT_MS)?;
        self.display_ctls.load(&self.avc, card_cntr, FCP_TIMEOUT_MS)?;
        self.opt_iface_ctls.load(&self.avc, card_cntr, FCP_TIMEOUT_MS)?;
        self.input_ctls.load(&self.avc, card_cntr, FCP_TIMEOUT_MS)?;
        self.out_ctls.load(&self.avc, card_cntr, FCP_TIMEOUT_MS)?;
        self.mixer_ctls.load(&self.avc, card_cntr, FCP_TIMEOUT_MS)?;
        self.route_ctls.load(&self.avc, card_cntr, FCP_TIMEOUT_MS)?;
        self.resampler_ctls.load(&self.avc, card_cntr, FCP_TIMEOUT_MS)?;

        Ok(())
    }

    fn read(&mut self, _: &mut SndUnit, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        if self.clk_ctl.read_freq(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.clk_ctl.read_src(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.meter_ctl.read_state(elem_id, elem_value)? {
            Ok(true)
        } else if self.hw_ctls.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.display_ctls.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.opt_iface_ctls.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.input_ctls.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.out_ctls.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_ctls.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.route_ctls.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.resampler_ctls.read(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(&mut self, unit: &mut SndUnit, elem_id: &ElemId, old: &ElemValue, new: &ElemValue)
        -> Result<bool, Error>
    {
        if self.clk_ctl.write_freq(unit, &self.avc, elem_id, old, new, FCP_TIMEOUT_MS * 3)? {
            Ok(true)
        } else if self.clk_ctl.write_src(unit, &self.avc, elem_id, old, new, FCP_TIMEOUT_MS * 3)? {
            Ok(true)
        } else if self.hw_ctls.write(unit, &self.avc, elem_id, old, new, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.display_ctls.write(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.opt_iface_ctls.write(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.input_ctls.write(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.out_ctls.write(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.mixer_ctls.write(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.route_ctls.write(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.resampler_ctls.write(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else {
            Ok(true)
        }
    }
}

impl MeasureModel<SndUnit> for EnsembleModel {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.meter_ctl.1);
    }

    fn measure_states(&mut self, _: &mut SndUnit) -> Result<(), Error> {
        self.meter_ctl.measure_state(&mut self.avc, FCP_TIMEOUT_MS)
    }

    fn measure_elem(&mut self, _: &SndUnit, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        self.meter_ctl.read_state(elem_id, elem_value)
    }
}

impl NotifyModel<SndUnit, bool> for EnsembleModel {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.clk_ctl.0);
    }

    fn parse_notification(&mut self, _: &mut SndUnit, _: &bool) -> Result<(), Error> {
        Ok(())
    }

    fn read_notified_elem(&mut self, _: &SndUnit, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        self.clk_ctl.read_freq(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)
    }
}

#[derive(Default)]
struct MeterCtl(EnsembleMeter, Vec<ElemId>);

const KNOB_IN_TARGET_NAME: &str = "knob-input-target";
const KNOB_OUT_TARGET_NAME: &str = "knob-output-target";

fn knob_input_target_to_str(target: &KnobInputTarget) -> &str {
    match target {
        KnobInputTarget::Mic0 => "mic-1",
        KnobInputTarget::Mic1 => "mic-2",
        KnobInputTarget::Mic2 => "mic-3",
        KnobInputTarget::Mic3 => "mic-4",
    }
}

fn knob_output_target_to_str(target: &KnobOutputTarget) -> &str {
    match target {
        KnobOutputTarget::AnalogOutputPair0 => "main",
        KnobOutputTarget::HeadphonePair0 => "headphone-1/2",
        KnobOutputTarget::HeadphonePair1 => "headphone-3/4",
    }
}

impl MeterCtl {
    const METER_TLV: DbInterval = DbInterval{min: -4800, max: 0, linear: false, mute_avail: false};

    const IN_METER_LABELS: [&'static str; 18] = [
        "analog-input-1", "analog-input-2", "analog-input-3", "analog-input-4",
        "analog-input-5", "analog-input-6", "analog-input-7", "analog-input-8",
        "spdif-input-1", "spdif-input-2",
        "adat-input-1", "adat-input-2", "adat-input-3", "adat-input-4",
        "adat-input-5", "adat-input-6", "adat-input-7", "adat-input-8",
    ];

    const OUT_METER_LABELS: [&'static str; 16] = [
        "analog-output-1", "analog-output-2", "analog-output-3", "analog-output-4",
        "analog-output-5", "analog-output-6", "analog-output-7", "analog-output-8",
        "spdif-output-1", "spdif-output-2",
        "adat-output-1", "adat-output-2", "adat-output-3", "adat-output-4",
        "adat-output-5", "adat-output-6",
        //"adat-output-7", "adat-output-8",
    ];

    const KNOB_INPUT_TARGETS: [KnobInputTarget; 4] = [
        KnobInputTarget::Mic0,
        KnobInputTarget::Mic1,
        KnobInputTarget::Mic2,
        KnobInputTarget::Mic3,
    ];

    const KNOB_OUTPUT_TARGETS: [KnobOutputTarget; 3] = [
        KnobOutputTarget::AnalogOutputPair0,
        KnobOutputTarget::HeadphonePair0,
        KnobOutputTarget::HeadphonePair1,
    ];

    const LEVEL_MIN: i32 = EnsembleMeterProtocol::LEVEL_MIN as i32;
    const LEVEL_MAX: i32 = EnsembleMeterProtocol::LEVEL_MAX as i32;
    const LEVEL_STEP: i32 = EnsembleMeterProtocol::LEVEL_STEP as i32;

    fn load_state(
        &mut self,
        card_cntr: &mut CardCntr,
        avc: &mut BebobAvc,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let labels: Vec<&str> = Self::KNOB_INPUT_TARGETS.iter()
            .map(|t| knob_input_target_to_str(t))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, KNOB_IN_TARGET_NAME, 0);
        card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        let labels: Vec<&str> = Self::KNOB_OUTPUT_TARGETS.iter()
            .map(|t| knob_output_target_to_str(t))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, KNOB_OUT_TARGET_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, true)
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, IN_METER_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                Self::LEVEL_MIN,
                Self::LEVEL_MAX,
                Self::LEVEL_STEP,
                Self::IN_METER_LABELS.len(),
                Some(&Into::<Vec<u32>>::into(Self::METER_TLV)),
                false,
            )
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, OUT_METER_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                Self::LEVEL_MIN,
                Self::LEVEL_MAX,
                Self::LEVEL_STEP,
                Self::OUT_METER_LABELS.len(),
                Some(&Into::<Vec<u32>>::into(Self::METER_TLV)),
                false,
            )
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        self.measure_state(avc, timeout_ms)
    }

    fn measure_state(&mut self, avc: &mut BebobAvc, timeout_ms: u32) -> Result<(), Error> {
        EnsembleMeterProtocol::measure_meter(avc, &mut self.0, timeout_ms)
    }

    fn read_state(
        &self,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            KNOB_IN_TARGET_NAME => {
                let idx = Self::KNOB_INPUT_TARGETS.iter()
                    .position(|t| t.eq(&self.0.knob_input_target))
                    .unwrap();
                elem_value.set_enum(&[idx as u32]);
                Ok(true)
            }
            KNOB_OUT_TARGET_NAME => {
                let idx = Self::KNOB_OUTPUT_TARGETS.iter()
                    .position(|t| t.eq(&self.0.knob_output_target))
                    .unwrap();
                elem_value.set_enum(&[idx as u32]);
                Ok(true)
            }
            IN_METER_NAME => {
                ElemValueAccessor::<i32>::set_vals(elem_value, 18, |idx| {
                    Ok(self.0.phys_inputs[idx] as i32)
                })?;
                Ok(true)
            }
            OUT_METER_NAME => {
                ElemValueAccessor::<i32>::set_vals(elem_value, 16, |idx| {
                    Ok(self.0.phys_outputs[idx] as i32)
                })?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use alsactl::CardError;

    #[test]
    fn test_clk_ctl_definition() {
        let mut card_cntr = CardCntr::new();
        let mut ctl = ClkCtl::default();

        let error = ctl.load_freq(&mut card_cntr).unwrap_err();
        assert_eq!(error.kind::<CardError>(), Some(CardError::Failed));

        let error = ctl.load_src(&mut card_cntr).unwrap_err();
        assert_eq!(error.kind::<CardError>(), Some(CardError::Failed));
    }
}
