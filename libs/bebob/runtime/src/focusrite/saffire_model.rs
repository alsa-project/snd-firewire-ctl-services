// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

use glib::Error;

use hinawa::{FwFcpExt, FwReq};
use hinawa::{SndUnit, SndUnitExt};

use alsactl::{ElemId, ElemIfaceType, ElemValue, ElemValueExt, ElemValueExtManual};

use core::card_cntr::*;

use bebob_protocols::{*, focusrite::{*, saffire::*}};

use crate::common_ctls::*;
use crate::model::IN_METER_NAME;
use super::*;

#[derive(Default)]
pub struct SaffireModel {
    req: FwReq,
    avc: BebobAvc,
    clk_ctl: ClkCtl,
    meter_ctl: MeterCtl,
    out_ctl: OutputCtl,
}

const FCP_TIMEOUT_MS: u32 = 100;
const TIMEOUT_MS: u32 = 50;

#[derive(Default)]
struct ClkCtl(Vec<ElemId>);

impl MediaClkFreqCtlOperation<SaffireClkProtocol> for ClkCtl {}

impl SamplingClkSrcCtlOperation<SaffireClkProtocol> for ClkCtl {
    const SRC_LABELS: &'static [&'static str] = &["Internal", "S/PDIF"];
}

#[derive(Default)]
struct MeterCtl(Vec<ElemId>, SaffireMeter);

#[derive(Default)]
struct OutputCtl(Vec<ElemId>, SaffireOutputParameters);

impl AsRef<SaffireOutputParameters> for OutputCtl {
    fn as_ref(&self) -> &SaffireOutputParameters {
        &self.1
    }
}

impl AsMut<SaffireOutputParameters> for OutputCtl {
    fn as_mut(&mut self) -> &mut SaffireOutputParameters {
        &mut self.1
    }
}

impl SaffireOutputCtlOperation<SaffireOutputProtocol> for OutputCtl {
    const OUTPUT_LABELS: &'static [&'static str] = &[
        "analog-output-1/2", "analog-output-3/4", "analog-output-5/6", "analog-output-7/8",
        "digital-output-1/2",
    ];
}

impl CtlModel<SndUnit> for SaffireModel {
    fn load(
        &mut self,
        unit: &mut SndUnit,
        card_cntr: &mut CardCntr,
    ) -> Result<(), Error> {
        self.avc.as_ref().bind(&unit.get_node())?;

        self.clk_ctl.load_freq(card_cntr)
            .map(|mut elem_id_list| self.clk_ctl.0.append(&mut elem_id_list))?;

        self.clk_ctl.load_src(card_cntr)
            .map(|mut elem_id_list| self.clk_ctl.0.append(&mut elem_id_list))?;

        self.meter_ctl.load_meter(card_cntr, unit, &self.req, TIMEOUT_MS)
            .map(|mut elem_id_list| self.meter_ctl.0.append(&mut elem_id_list))?;

        self.out_ctl.load_params(card_cntr, unit, &self.req, TIMEOUT_MS)
            .map(|mut elem_id_list| self.out_ctl.0.append(&mut elem_id_list))?;

        Ok(())
    }

    fn read(
        &mut self,
        _: &mut SndUnit,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.clk_ctl.read_freq(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.clk_ctl.read_src(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.meter_ctl.read_meter(elem_id, elem_value)? {
            Ok(true)
        } else if self.out_ctl.read_params(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(
        &mut self,
        unit: &mut SndUnit,
        elem_id: &ElemId,
        old: &ElemValue,
        new: &ElemValue,
    ) -> Result<bool, Error> {
        if self.clk_ctl.write_freq(unit, &self.avc, elem_id, old, new, FCP_TIMEOUT_MS * 3)? {
            Ok(true)
        } else if self.clk_ctl.write_src(unit, &self.avc, elem_id, old, new, FCP_TIMEOUT_MS * 3)? {
            Ok(true)
        } else if self.out_ctl.write_params(unit, &self.req, elem_id, new, TIMEOUT_MS)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl NotifyModel<SndUnit, bool> for SaffireModel {
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

impl MeasureModel<SndUnit> for SaffireModel {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.meter_ctl.0);
        elem_id_list.extend_from_slice(&self.out_ctl.0);
    }

    fn measure_states(&mut self, unit: &mut SndUnit) -> Result<(), Error> {
        self.meter_ctl.measure_meter(unit, &self.req, TIMEOUT_MS)?;
        self.out_ctl.measure_params(unit, &self.req, TIMEOUT_MS)?;
        Ok(())
    }

    fn measure_elem(&mut self, _: &SndUnit, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        if self.meter_ctl.read_meter(elem_id, elem_value)? {
            Ok(true)
        } else if self.out_ctl.read_params(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl MeterCtl {
    const METER_DIG_INPUT_DETECT_NAME: &'static str = "digital-input-detection";

    const PHYS_INPUT_LABELS: &'static [&'static str] = &[
        "analog-input-1", "analog-input-2", "analog-input-3", "analog-input-4",

    ];

    fn load_meter(
        &mut self,
        card_cntr: &mut CardCntr,
        unit: &SndUnit,
        req: &FwReq,
        timeout_ms: u32,
    ) -> Result<Vec<ElemId>, Error> {

        let mut measured_elem_id_list = Vec::new();

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, IN_METER_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                SaffireMeterProtocol::LEVEL_MIN as i32,
                SaffireMeterProtocol::LEVEL_MAX as i32,
                SaffireMeterProtocol::LEVEL_STEP as i32,
                Self::PHYS_INPUT_LABELS.len(),
                None,
                false,
            )
            .map(|mut elem_id_list| measured_elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(
            ElemIfaceType::Card,
            0,
            0,
            Self::METER_DIG_INPUT_DETECT_NAME,
            0,
        );
        card_cntr
            .add_bool_elems(&elem_id, 1, 1, false)
            .map(|mut elem_id_list| measured_elem_id_list.append(&mut elem_id_list))?;

        SaffireMeterProtocol::read_meter(req, &unit.get_node(), &mut self.1, timeout_ms)?;

        Ok(measured_elem_id_list)
    }

    fn measure_meter(&mut self, unit: &SndUnit, req: &FwReq, timeout_ms: u32) -> Result<(), Error> {
        SaffireMeterProtocol::read_meter(req, &unit.get_node(), &mut self.1, timeout_ms)
    }

    fn read_meter(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            IN_METER_NAME => {
                elem_value.set_int(&self.1.phys_inputs);
                Ok(true)
            }
            Self::METER_DIG_INPUT_DETECT_NAME => {
                elem_value.set_bool(&[self.1.dig_input_detect]);
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

    #[test]
    fn test_output_params_definition() {
        let mut card_cntr = CardCntr::new();
        let mut ctl = OutputCtl::default();
        let unit = SndUnit::default();
        let req = FwReq::default();

        let error = ctl.load_params(&mut card_cntr, &unit, &req, TIMEOUT_MS).unwrap_err();
        assert_eq!(error.kind::<CardError>(), Some(CardError::Failed));
    }
}
