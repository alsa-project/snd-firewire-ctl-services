// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2022 Takashi Sakamoto

use {super::*, std::marker::PhantomData};

pub type SaffirePro10ioModel = SaffireProIoModel<
    SaffirePro10ioClkProtocol,
    SaffirePro10ioMeterProtocol,
    SaffirePro10ioMonitorProtocol,
    SaffirePro10ioSpecificProtocol,
>;
pub type SaffirePro26ioModel = SaffireProIoModel<
    SaffirePro26ioClkProtocol,
    SaffirePro26ioMeterProtocol,
    SaffirePro26ioMonitorProtocol,
    SaffirePro26ioSpecificProtocol,
>;

#[derive(Default, Debug)]
pub struct SaffireProIoModel<C, M, O, S>
where
    C: SaffireProioMediaClockFrequencyOperation + SaffireProioSamplingClockSourceOperation,
    M: SaffireProioMeterOperation,
    O: SaffireProioMonitorProtocol,
    S: SaffireProioSpecificOperation,
{
    req: FwReq,
    avc: BebobAvc,
    clk_ctl: ClkCtl<C>,
    meter_ctl: MeterCtl<M>,
    out_ctl: OutputCtl,
    through_ctl: ThroughCtl,
    monitor_ctl: MonitorCtl<O>,
    mixer_ctl: SaffireProioMixerCtl,
    specific_ctl: SpecificCtl<S>,
}

const TIMEOUT_MS: u32 = 50;

#[derive(Default, Debug)]
struct ClkCtl<C>(Vec<ElemId>, PhantomData<C>)
where
    C: SaffireProioMediaClockFrequencyOperation + SaffireProioSamplingClockSourceOperation;

impl<C> SaffireProMediaClkFreqCtlOperation<C> for ClkCtl<C> where
    C: SaffireProioMediaClockFrequencyOperation + SaffireProioSamplingClockSourceOperation
{
}

impl<C> SaffireProSamplingClkSrcCtlOperation<C> for ClkCtl<C> where
    C: SaffireProioMediaClockFrequencyOperation + SaffireProioSamplingClockSourceOperation
{
}

#[derive(Default, Debug)]
struct MeterCtl<M>(SaffireProioMeterState, Vec<ElemId>, PhantomData<M>)
where
    M: SaffireProioMeterOperation;

impl<M> SaffireProioMeterCtlOperation<M> for MeterCtl<M>
where
    M: SaffireProioMeterOperation,
{
    fn state(&self) -> &SaffireProioMeterState {
        &self.0
    }

    fn state_mut(&mut self) -> &mut SaffireProioMeterState {
        &mut self.0
    }
}

#[derive(Default, Debug)]
struct OutputCtl(SaffireOutputParameters, Vec<ElemId>);

impl SaffireOutputCtlOperation<SaffireProioOutputProtocol> for OutputCtl {
    const OUTPUT_LABELS: &'static [&'static str] = &[
        "analog-output-1/2",
        "analog-output-3/4",
        "analog-output-5/6",
        "analog-output-7/8",
    ];

    fn state(&self) -> &SaffireOutputParameters {
        &self.0
    }

    fn state_mut(&mut self) -> &mut SaffireOutputParameters {
        &mut self.0
    }
}

#[derive(Default, Debug)]
struct ThroughCtl;

impl SaffireThroughCtlOperation<SaffireProioThroughProtocol> for ThroughCtl {}

#[derive(Default, Debug)]
struct MonitorCtl<O>(SaffireProioMonitorParameters, PhantomData<O>)
where
    O: SaffireProioMonitorProtocol;

impl<O> SaffireProioMonitorCtlOperation<O> for MonitorCtl<O>
where
    O: SaffireProioMonitorProtocol,
{
    fn state(&self) -> &SaffireProioMonitorParameters {
        &self.0
    }

    fn state_mut(&mut self) -> &mut SaffireProioMonitorParameters {
        &mut self.0
    }
}

#[derive(Default, Debug)]
struct SpecificCtl<S>(SaffireProioSpecificParameters, PhantomData<S>)
where
    S: SaffireProioSpecificOperation;

impl<S> SaffireProioSpecificCtlOperation<S> for SpecificCtl<S>
where
    S: SaffireProioSpecificOperation,
{
    fn state(&self) -> &SaffireProioSpecificParameters {
        &self.0
    }

    fn state_mut(&mut self) -> &mut SaffireProioSpecificParameters {
        &mut self.0
    }
}

impl<C, M, O, S> CtlModel<(SndUnit, FwNode)> for SaffireProIoModel<C, M, O, S>
where
    C: SaffireProioMediaClockFrequencyOperation + SaffireProioSamplingClockSourceOperation,
    M: SaffireProioMeterOperation,
    O: SaffireProioMonitorProtocol,
    S: SaffireProioSpecificOperation,
{
    fn load(
        &mut self,
        unit: &mut (SndUnit, FwNode),
        card_cntr: &mut CardCntr,
    ) -> Result<(), Error> {
        self.avc.bind(&unit.1)?;

        self.clk_ctl
            .load_freq(card_cntr)
            .map(|mut elem_id_list| self.clk_ctl.0.append(&mut elem_id_list))?;

        self.clk_ctl
            .load_src(card_cntr)
            .map(|mut elem_id_list| self.clk_ctl.0.append(&mut elem_id_list))?;

        self.meter_ctl
            .load_state(card_cntr, unit, &self.req, TIMEOUT_MS)
            .map(|mut elem_id_list| self.meter_ctl.1.append(&mut elem_id_list))?;

        self.out_ctl
            .load_params(card_cntr, unit, &self.req, TIMEOUT_MS)
            .map(|mut elem_id_list| self.out_ctl.1.append(&mut elem_id_list))?;

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
        unit: &mut (SndUnit, FwNode),
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
        unit: &mut (SndUnit, FwNode),
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

impl<C, M, O, S> NotifyModel<(SndUnit, FwNode), bool> for SaffireProIoModel<C, M, O, S>
where
    C: SaffireProioMediaClockFrequencyOperation + SaffireProioSamplingClockSourceOperation,
    M: SaffireProioMeterOperation,
    O: SaffireProioMonitorProtocol,
    S: SaffireProioSpecificOperation,
{
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.clk_ctl.0);
    }

    fn parse_notification(&mut self, _: &mut (SndUnit, FwNode), _: &bool) -> Result<(), Error> {
        Ok(())
    }

    fn read_notified_elem(
        &mut self,
        unit: &(SndUnit, FwNode),
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        self.clk_ctl
            .read_freq(unit, &self.req, elem_id, elem_value, TIMEOUT_MS)
    }
}

impl<C, M, O, S> MeasureModel<(SndUnit, FwNode)> for SaffireProIoModel<C, M, O, S>
where
    C: SaffireProioMediaClockFrequencyOperation + SaffireProioSamplingClockSourceOperation,
    M: SaffireProioMeterOperation,
    O: SaffireProioMonitorProtocol,
    S: SaffireProioSpecificOperation,
{
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.meter_ctl.1);
    }

    fn measure_states(&mut self, unit: &mut (SndUnit, FwNode)) -> Result<(), Error> {
        self.meter_ctl.measure_state(unit, &self.req, TIMEOUT_MS)
    }

    fn measure_elem(
        &mut self,
        _: &(SndUnit, FwNode),
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
        let mut card_cntr = CardCntr::default();
        let mut ctl = OutputCtl::default();
        let unit = SndUnit::default();
        let node = FwNode::default();
        let req = FwReq::default();

        let error = ctl
            .load_params(&mut card_cntr, &(unit, node), &req, TIMEOUT_MS)
            .unwrap_err();
        assert_eq!(error.kind::<CardError>(), Some(CardError::Failed));
    }
}
