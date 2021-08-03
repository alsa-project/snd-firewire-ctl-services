// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

use glib::Error;

use hinawa::FwFcpExt;
use hinawa::{SndUnit, SndUnitExt};

use alsactl::{ElemId, ElemValue, ElemValueExt, ElemValueExtManual};

use core::card_cntr::*;

use bebob_protocols::{*, focusrite::saffire::*};

use crate::{common_ctls::*, model::{IN_METER_NAME, OUT_METER_NAME}};
use super::*;

#[derive(Default)]
pub struct SaffireLeModel {
    req: FwReq,
    avc: BebobAvc,
    clk_ctl: ClkCtl,
    meter_ctl: MeterCtl,
}

// NOTE: At 88.2/96.0 kHz, AV/C transaction takes more time than 44.1/48.0 kHz.
const FCP_TIMEOUT_MS: u32 = 200;
const TIMEOUT_MS: u32 = 100;

#[derive(Default)]
struct ClkCtl(Vec<ElemId>);

impl MediaClkFreqCtlOperation<SaffireLeClkProtocol> for ClkCtl {}

impl SamplingClkSrcCtlOperation<SaffireLeClkProtocol> for ClkCtl {
    const SRC_LABELS: &'static [&'static str] = &["Internal", "S/PDIF"];
}

#[derive(Default)]
struct MeterCtl(Vec<ElemId>, SaffireLeMeter);

impl CtlModel<SndUnit> for SaffireLeModel {
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

        self.meter_ctl.load_meter(card_cntr, unit, &self.req, FCP_TIMEOUT_MS)
            .map(|mut elem_id_list| self.meter_ctl.0.append(&mut elem_id_list))?;

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
        } else {
            Ok(false)
        }
    }
}

impl NotifyModel<SndUnit, bool> for SaffireLeModel {
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

impl MeasureModel<SndUnit> for SaffireLeModel {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.meter_ctl.0);
    }

    fn measure_states(&mut self, unit: &mut SndUnit) -> Result<(), Error> {
        self.meter_ctl.measure_meter(unit, &self.req, TIMEOUT_MS)
    }

    fn measure_elem(&mut self, _: &SndUnit, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        self.meter_ctl.read_meter(elem_id, elem_value)
    }
}

const METER_STREAM_INPUT_NAME: &str = "stream-input-meter";
const METER_DIG_INPUT_DETECT_NAME: &str = "digital-input-detection";

impl MeterCtl {
    const PHYS_INPUT_LABELS: &'static [&'static str] = &[
        "analog-input-1", "analog-input-2", "analog-input-3", "analog-input-4",
        "digital-input-1", "digital-input-2",

    ];
    const STREAM_INPUT_LABELS: &'static [&'static str] = &[
        "stream-input-1/2", "stream-input-3/4", "stream-input-5/6", "stream-input-7/8",
    ];
    const PHYS_OUTPUT_LABELS: &'static [&'static str] = &[
        "analog-output-1", "analog-output-2", "analog-output-3", "analog-output-4",
        "analog-output-5", "analog-output-6",
        "digital-output-1", "digital-output-2",
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
                SaffireLeMeterProtocol::LEVEL_MIN,
                SaffireLeMeterProtocol::LEVEL_MAX,
                SaffireLeMeterProtocol::LEVEL_STEP,
                Self::PHYS_INPUT_LABELS.len(),
                None,
                false,
            )
            .map(|mut elem_id_list| measured_elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(
            ElemIfaceType::Mixer,
            0,
            0,
            METER_STREAM_INPUT_NAME,
            0,
        );
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                SaffireLeMeterProtocol::LEVEL_MIN,
                SaffireLeMeterProtocol::LEVEL_MAX,
                SaffireLeMeterProtocol::LEVEL_STEP,
                Self::STREAM_INPUT_LABELS.len(),
                None,
                false,
            )
            .map(|mut elem_id_list| measured_elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, OUT_METER_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                SaffireLeMeterProtocol::LEVEL_MIN,
                SaffireLeMeterProtocol::LEVEL_MAX,
                SaffireLeMeterProtocol::LEVEL_STEP,
                Self::PHYS_OUTPUT_LABELS.len(),
                None,
                false,
            )
            .map(|mut elem_id_list| measured_elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(
            ElemIfaceType::Card,
            0,
            0,
            METER_DIG_INPUT_DETECT_NAME,
            0,
        );
        card_cntr
            .add_bool_elems(&elem_id, 1, 1, false)
            .map(|mut elem_id_list| measured_elem_id_list.append(&mut elem_id_list))?;

        SaffireLeMeterProtocol::read_meter(req, &unit.get_node(), &mut self.1, timeout_ms)?;

        Ok(measured_elem_id_list)
    }

    fn measure_meter(&mut self, unit: &SndUnit, req: &FwReq, timeout_ms: u32) -> Result<(), Error> {
        SaffireLeMeterProtocol::read_meter(req, &unit.get_node(), &mut self.1, timeout_ms)
    }

    fn read_meter(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            IN_METER_NAME => {
                elem_value.set_int(&self.1.phys_inputs);
                Ok(true)
            }
            METER_STREAM_INPUT_NAME => {
                elem_value.set_int(&self.1.stream_inputs);
                Ok(true)
            }
            OUT_METER_NAME => {
                elem_value.set_int(&self.1.phys_outputs);
                Ok(true)
            }
            METER_DIG_INPUT_DETECT_NAME => {
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
}
