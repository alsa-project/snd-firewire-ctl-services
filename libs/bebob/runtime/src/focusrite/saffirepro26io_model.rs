// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

use glib::Error;

use hinawa::{FwFcpExt, FwReq};
use hinawa::{SndUnit, SndUnitExt};

use alsactl::{ElemId, ElemValue};

use core::card_cntr::*;

use bebob_protocols::{focusrite::saffireproio::*, *};

use super::*;

#[derive(Default)]
pub struct SaffirePro26ioModel {
    req: FwReq,
    avc: BebobAvc,
    clk_ctl: ClkCtl,
    meter_ctl: MeterCtl,
    out_ctl: OutputCtl,
    through_ctl: ThroughCtl,
    monitor_ctl: MonitorCtl,
    mixer_ctl: SaffireProioMixerCtl,
    specific_ctl: SpecificCtl,
}

const TIMEOUT_MS: u32 = 50;

#[derive(Default)]
struct ClkCtl(Vec<ElemId>);

impl SaffireProMediaClkFreqCtlOperation<SaffirePro26ioClkProtocol> for ClkCtl {}

impl SaffireProSamplingClkSrcCtlOperation<SaffirePro26ioClkProtocol> for ClkCtl {}

#[derive(Default)]
struct MeterCtl(Vec<ElemId>, SaffireProioMeterState);

impl AsRef<SaffireProioMeterState> for MeterCtl {
    fn as_ref(&self) -> &SaffireProioMeterState {
        &self.1
    }
}

impl AsMut<SaffireProioMeterState> for MeterCtl {
    fn as_mut(&mut self) -> &mut SaffireProioMeterState {
        &mut self.1
    }
}

impl SaffireProioMeterCtlOperation<SaffirePro26ioMeterProtocol> for MeterCtl {}

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
impl SaffireOutputCtlOperation<SaffireProioOutputProtocol> for OutputCtl {
    const OUTPUT_LABELS: &'static [&'static str] = &[
        "analog-output-1/2",
        "analog-output-3/4",
        "analog-output-5/6",
        "analog-output-7/8",
    ];
}

#[derive(Default)]
struct ThroughCtl;

impl SaffireThroughCtlOperation<SaffireProioThroughProtocol> for ThroughCtl {}

#[derive(Default)]
struct MonitorCtl(SaffireProioMonitorParameters);

impl AsRef<SaffireProioMonitorParameters> for MonitorCtl {
    fn as_ref(&self) -> &SaffireProioMonitorParameters {
        &self.0
    }
}

impl AsMut<SaffireProioMonitorParameters> for MonitorCtl {
    fn as_mut(&mut self) -> &mut SaffireProioMonitorParameters {
        &mut self.0
    }
}

impl SaffireProioMonitorCtlOperation<SaffirePro26ioMonitorProtocol> for MonitorCtl {}

#[derive(Default)]
struct SpecificCtl(SaffireProioSpecificParameters);

impl AsRef<SaffireProioSpecificParameters> for SpecificCtl {
    fn as_ref(&self) -> &SaffireProioSpecificParameters {
        &self.0
    }
}

impl AsMut<SaffireProioSpecificParameters> for SpecificCtl {
    fn as_mut(&mut self) -> &mut SaffireProioSpecificParameters {
        &mut self.0
    }
}

impl SaffireProioSpecificCtlOperation<SaffirePro26ioSpecificProtocol> for SpecificCtl {}

impl CtlModel<SndUnit> for SaffirePro26ioModel {
    fn load(&mut self, unit: &mut SndUnit, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.avc.as_ref().bind(&unit.get_node())?;

        self.clk_ctl
            .load_freq(card_cntr)
            .map(|mut elem_id_list| self.clk_ctl.0.append(&mut elem_id_list))?;

        self.clk_ctl
            .load_src(card_cntr)
            .map(|mut elem_id_list| self.clk_ctl.0.append(&mut elem_id_list))?;

        self.meter_ctl
            .load_state(card_cntr, unit, &self.req, TIMEOUT_MS)
            .map(|mut elem_id_list| self.meter_ctl.0.append(&mut elem_id_list))?;

        self.out_ctl
            .load_params(card_cntr, unit, &self.req, TIMEOUT_MS)
            .map(|mut elem_id_list| self.out_ctl.0.append(&mut elem_id_list))?;

        self.through_ctl.load_params(card_cntr)?;

        self.monitor_ctl
            .load_params(card_cntr, unit, &self.req, TIMEOUT_MS)?;

        self.mixer_ctl
            .load_params(card_cntr, unit, &self.req, TIMEOUT_MS)?;

        self.specific_ctl
            .load_params(card_cntr, unit, &self.req, TIMEOUT_MS)?;

        Ok(())
    }

    fn read(
        &mut self,
        unit: &mut SndUnit,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self
            .clk_ctl
            .read_freq(unit, &self.req, elem_id, elem_value, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .clk_ctl
            .read_src(unit, &self.req, elem_id, elem_value, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self.meter_ctl.read_state(elem_id, elem_value)? {
            Ok(true)
        } else if self.out_ctl.read_params(elem_id, elem_value)? {
            Ok(true)
        } else if self
            .through_ctl
            .read_params(unit, &self.req, elem_id, elem_value, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self.monitor_ctl.read_params(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_ctl.read_params(elem_id, elem_value)? {
            Ok(true)
        } else if self.specific_ctl.read_params(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(
        &mut self,
        unit: &mut SndUnit,
        elem_id: &ElemId,
        _: &ElemValue,
        new: &ElemValue,
    ) -> Result<bool, Error> {
        if self
            .clk_ctl
            .write_freq(unit, &self.req, elem_id, new, TIMEOUT_MS * 3)?
        {
            Ok(true)
        } else if self
            .clk_ctl
            .write_src(unit, &self.req, elem_id, new, TIMEOUT_MS * 3)?
        {
            Ok(true)
        } else if self
            .out_ctl
            .write_params(unit, &self.req, elem_id, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .through_ctl
            .write_params(unit, &self.req, elem_id, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .monitor_ctl
            .write_params(unit, &self.req, elem_id, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .mixer_ctl
            .write_params(unit, &self.req, elem_id, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .specific_ctl
            .write_params(unit, &self.req, elem_id, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl NotifyModel<SndUnit, bool> for SaffirePro26ioModel {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.clk_ctl.0);
    }

    fn parse_notification(&mut self, _: &mut SndUnit, _: &bool) -> Result<(), Error> {
        Ok(())
    }

    fn read_notified_elem(
        &mut self,
        unit: &SndUnit,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        self.clk_ctl
            .read_freq(unit, &self.req, elem_id, elem_value, TIMEOUT_MS)
    }
}

impl MeasureModel<SndUnit> for SaffirePro26ioModel {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.meter_ctl.0);
    }

    fn measure_states(&mut self, unit: &mut SndUnit) -> Result<(), Error> {
        self.meter_ctl.measure_state(unit, &self.req, TIMEOUT_MS)
    }

    fn measure_elem(
        &mut self,
        _: &SndUnit,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        self.meter_ctl.read_state(elem_id, elem_value)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use alsactl::CardError;

    #[test]
    fn test_output_params_definition() {
        let mut card_cntr = CardCntr::new();
        let mut ctl = OutputCtl::default();
        let unit = SndUnit::default();
        let req = FwReq::default();

        let error = ctl
            .load_params(&mut card_cntr, &unit, &req, TIMEOUT_MS)
            .unwrap_err();
        assert_eq!(error.kind::<CardError>(), Some(CardError::Failed));
    }
}
