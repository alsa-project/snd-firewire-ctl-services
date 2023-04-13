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
    C: SaffireProioMediaClockSpecification + SaffireProioSamplingClockSpecification,
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
struct ClkCtl<C>(
    MediaClockParameters,
    SamplingClockParameters,
    Vec<ElemId>,
    PhantomData<C>,
)
where
    C: SaffireProioMediaClockSpecification + SaffireProioSamplingClockSpecification;

impl<C> SaffireProMediaClkFreqCtlOperation<C> for ClkCtl<C>
where
    C: SaffireProioMediaClockSpecification + SaffireProioSamplingClockSpecification,
{
    fn state(&self) -> &MediaClockParameters {
        &self.0
    }

    fn state_mut(&mut self) -> &mut MediaClockParameters {
        &mut self.0
    }
}

impl<C> SaffireProSamplingClkSrcCtlOperation<C> for ClkCtl<C>
where
    C: SaffireProioMediaClockSpecification + SaffireProioSamplingClockSpecification,
{
    fn state(&self) -> &SamplingClockParameters {
        &self.1
    }

    fn state_mut(&mut self) -> &mut SamplingClockParameters {
        &mut self.1
    }
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

#[derive(Debug)]
struct OutputCtl(SaffireOutputParameters, Vec<ElemId>);

impl Default for OutputCtl {
    fn default() -> Self {
        Self(
            SaffireProioOutputProtocol::create_output_parameters(),
            Default::default(),
        )
    }
}

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
struct ThroughCtl(SaffireThroughParameters);

impl SaffireThroughCtlOperation<SaffireProioThroughProtocol> for ThroughCtl {
    fn state(&self) -> &SaffireThroughParameters {
        &self.0
    }

    fn state_mut(&mut self) -> &mut SaffireThroughParameters {
        &mut self.0
    }
}

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

#[derive(Debug)]
struct SpecificCtl<S>(SaffireProioSpecificParameters, PhantomData<S>)
where
    S: SaffireProioSpecificOperation;

impl<S> Default for SpecificCtl<S>
where
    S: SaffireProioSpecificOperation,
{
    fn default() -> Self {
        Self(S::create_params(), Default::default())
    }
}

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
    C: SaffireProioMediaClockSpecification + SaffireProioSamplingClockSpecification,
    M: SaffireProioMeterOperation,
    O: SaffireProioMonitorProtocol,
    S: SaffireProioSpecificOperation,
{
    fn cache(&mut self, unit: &mut (SndUnit, FwNode)) -> Result<(), Error> {
        self.avc.bind(&unit.1)?;

        self.clk_ctl.cache_freq(&self.req, &unit.1, TIMEOUT_MS)?;
        self.clk_ctl.cache_src(&self.req, &unit.1, TIMEOUT_MS)?;
        self.meter_ctl.cache(&self.req, &unit.1, TIMEOUT_MS)?;
        self.out_ctl.cache(&self.req, &unit.1, TIMEOUT_MS)?;
        self.through_ctl.cache(&self.req, &unit.1, TIMEOUT_MS)?;
        self.monitor_ctl.cache(&self.req, &unit.1, TIMEOUT_MS)?;
        self.mixer_ctl.cache(&self.req, &unit.1, TIMEOUT_MS)?;
        self.specific_ctl.cache(&self.req, &unit.1, TIMEOUT_MS)?;

        Ok(())
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.clk_ctl
            .load_freq(card_cntr)
            .map(|mut elem_id_list| self.clk_ctl.2.append(&mut elem_id_list))?;

        self.clk_ctl
            .load_src(card_cntr)
            .map(|mut elem_id_list| self.clk_ctl.2.append(&mut elem_id_list))?;

        self.meter_ctl
            .load_state(card_cntr)
            .map(|mut elem_id_list| self.meter_ctl.1.append(&mut elem_id_list))?;

        self.out_ctl
            .load_params(card_cntr)
            .map(|mut elem_id_list| self.out_ctl.1.append(&mut elem_id_list))?;

        self.through_ctl.load_params(card_cntr)?;

        self.monitor_ctl.load_params(card_cntr)?;

        self.mixer_ctl.load_params(card_cntr)?;

        self.specific_ctl.load_params(card_cntr)?;

        Ok(())
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        if self.clk_ctl.read_freq(elem_id, elem_value)? {
            Ok(true)
        } else if self.clk_ctl.read_src(elem_id, elem_value)? {
            Ok(true)
        } else if self.meter_ctl.read_state(elem_id, elem_value)? {
            Ok(true)
        } else if self.out_ctl.read_params(elem_id, elem_value)? {
            Ok(true)
        } else if self.through_ctl.read_params(elem_id, elem_value)? {
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
            .write_params(&self.req, &unit.1, elem_id, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .through_ctl
            .write_params(&self.req, &unit.1, elem_id, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .monitor_ctl
            .write_params(&self.req, &unit.1, elem_id, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .mixer_ctl
            .write_params(&self.req, &unit.1, elem_id, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .specific_ctl
            .write_params(&self.req, &unit.1, elem_id, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl<C, M, O, S> NotifyModel<(SndUnit, FwNode), bool> for SaffireProIoModel<C, M, O, S>
where
    C: SaffireProioMediaClockSpecification + SaffireProioSamplingClockSpecification,
    M: SaffireProioMeterOperation,
    O: SaffireProioMonitorProtocol,
    S: SaffireProioSpecificOperation,
{
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.clk_ctl.2);
    }

    fn parse_notification(
        &mut self,
        (_, node): &mut (SndUnit, FwNode),
        &locked: &bool,
    ) -> Result<(), Error> {
        if locked {
            self.clk_ctl.cache_src(&self.req, node, TIMEOUT_MS)?;
            C::cache(&self.req, node, &mut self.clk_ctl.0, TIMEOUT_MS)?;
        }
        Ok(())
    }
}

impl<C, M, O, S> MeasureModel<(SndUnit, FwNode)> for SaffireProIoModel<C, M, O, S>
where
    C: SaffireProioMediaClockSpecification + SaffireProioSamplingClockSpecification,
    M: SaffireProioMeterOperation,
    O: SaffireProioMonitorProtocol,
    S: SaffireProioSpecificOperation,
{
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.meter_ctl.1);
    }

    fn measure_states(&mut self, unit: &mut (SndUnit, FwNode)) -> Result<(), Error> {
        self.meter_ctl.cache(&self.req, &unit.1, TIMEOUT_MS)
    }
}

trait SaffireProMediaClkFreqCtlOperation<T: SaffireProioMediaClockSpecification> {
    fn state(&self) -> &MediaClockParameters;
    fn state_mut(&mut self) -> &mut MediaClockParameters;

    fn load_freq(&mut self, card_cntr: &mut CardCntr) -> Result<Vec<ElemId>, Error> {
        let labels: Vec<String> = T::FREQ_LIST.iter().map(|&r| r.to_string()).collect();

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, CLK_RATE_NAME, 0);
        card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)
    }

    fn cache_freq(&mut self, req: &FwReq, node: &FwNode, timeout_ms: u32) -> Result<(), Error> {
        let res = T::cache(req, node, self.state_mut(), timeout_ms);
        debug!(params = ?self.state(), ?res);
        res
    }

    fn read_freq(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            CLK_RATE_NAME => {
                elem_value.set_enum(&[self.state().freq_idx as u32]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write_freq(
        &mut self,
        unit: &mut (SndUnit, FwNode),
        req: &FwReq,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            CLK_RATE_NAME => {
                unit.0.lock()?;
                let mut params = self.state().clone();
                params.freq_idx = elem_value.enumerated()[0] as usize;
                let res =
                    T::update(req, &unit.1, &params, self.state_mut(), timeout_ms).map(|_| true);
                let _ = unit.0.unlock();
                debug!(params = ?self.state(), ?res);
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

trait SaffireProSamplingClkSrcCtlOperation<T: SaffireProioSamplingClockSpecification> {
    fn state(&self) -> &SamplingClockParameters;
    fn state_mut(&mut self) -> &mut SamplingClockParameters;

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

    fn cache_src(&mut self, req: &FwReq, node: &FwNode, timeout_ms: u32) -> Result<(), Error> {
        let res = T::cache(req, node, self.state_mut(), timeout_ms);
        debug!(params = ?self.state(), ?res);
        res
    }

    fn read_src(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            CLK_SRC_NAME => {
                elem_value.set_enum(&[self.state().src_idx as u32]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write_src(
        &mut self,
        unit: &mut (SndUnit, FwNode),
        req: &FwReq,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            CLK_SRC_NAME => {
                unit.0.lock()?;
                let mut params = self.state().clone();
                params.src_idx = elem_value.enumerated()[0] as usize;
                let res =
                    T::update(req, &unit.1, &params, self.state_mut(), timeout_ms).map(|_| true);
                let _ = unit.0.unlock();
                debug!(params = ?self.state(), ?res);
                res.map(|_| true)
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

    fn load_state(&mut self, card_cntr: &mut CardCntr) -> Result<Vec<ElemId>, Error> {
        let mut measured_elem_id_list = Vec::new();

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, MONITOR_KNOB_VALUE_NAME, 0);
        card_cntr
            .add_int_elems(&elem_id, 1, 0, u8::MAX as i32, 1, 1, None, false)
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

        Ok(measured_elem_id_list)
    }

    fn cache(&mut self, req: &FwReq, node: &FwNode, timeout_ms: u32) -> Result<(), Error> {
        let res = T::cache(req, node, self.state_mut(), timeout_ms);
        debug!(params = ?self.state(), ?res);
        res
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

const PRO_MONITOR_ANALOG_INPUT_NAME: &str = "monitor:analog-input";
const PRO_MONITOR_SPDIF_INPUT_NAME: &str = "monitor:spdif-input";
const PRO_MONITOR_ADAT_INPUT_NAME: &str = "monitor:adat-input";

trait SaffireProioMonitorCtlOperation<T: SaffireProioMonitorProtocol> {
    fn state(&self) -> &SaffireProioMonitorParameters;
    fn state_mut(&mut self) -> &mut SaffireProioMonitorParameters;

    fn load_params(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
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

        Ok(())
    }

    fn cache(&mut self, req: &FwReq, node: &FwNode, timeout_ms: u32) -> Result<(), Error> {
        let res = T::cache(req, node, self.state_mut(), timeout_ms);
        debug!(params = ?self.state(), ?res);
        res
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
        req: &FwReq,
        node: &FwNode,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            PRO_MONITOR_ANALOG_INPUT_NAME => {
                let idx = elem_id.index() as usize;
                let mut params = self.state().clone();
                let analog_inputs = &mut params.analog_inputs[idx];
                let vals = &elem_value.int()[..analog_inputs.len()];
                analog_inputs
                    .iter_mut()
                    .zip(vals)
                    .for_each(|(gain, &val)| *gain = val as i16);
                let res = T::update(req, node, &params, self.state_mut(), timeout_ms);
                debug!(params = ?self.state(), ?res);
                res.map(|_| true)
            }
            PRO_MONITOR_SPDIF_INPUT_NAME => {
                let idx = elem_id.index() as usize;
                let mut params = self.state().clone();
                let spdif_inputs = &mut params.spdif_inputs[idx];
                let vals = &elem_value.int()[..spdif_inputs.len()];
                params.spdif_inputs[idx]
                    .iter_mut()
                    .zip(vals)
                    .for_each(|(gain, &val)| *gain = val as i16);
                let res = T::update(req, node, &params, self.state_mut(), timeout_ms);
                debug!(params = ?self.state(), ?res);
                res.map(|_| true)
            }
            PRO_MONITOR_ADAT_INPUT_NAME => {
                let idx = elem_id.index() as usize;
                let mut params = self.state().clone();
                if let Some(adat_input_list) = &mut params.adat_inputs {
                    let adat_inputs = &mut adat_input_list[idx];
                    let vals = &elem_value.int()[..adat_inputs.len()];
                    adat_inputs
                        .iter_mut()
                        .zip(vals)
                        .for_each(|(gain, &val)| *gain = val as i16);
                }
                let res = T::update(req, node, &params, self.state_mut(), timeout_ms);
                debug!(params = ?self.state(), ?res);
                res.map(|_| true)
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
    fn load_params(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
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

        Ok(())
    }

    fn cache(&mut self, req: &FwReq, node: &FwNode, timeout_ms: u32) -> Result<(), Error> {
        let res = SaffireProioMixerProtocol::cache(req, node, &mut self.0, timeout_ms);
        debug!(params = ?self.0, ?res);
        res
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
        req: &FwReq,
        node: &FwNode,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            PRO_MIXER_MONITOR_SRC_NAME => {
                let mut params = self.0.clone();
                let monitor_sources = &mut params.monitor_sources;
                let vals = &elem_value.int()[..monitor_sources.len()];
                monitor_sources
                    .iter_mut()
                    .zip(vals)
                    .for_each(|(level, &val)| *level = val as i16);
                let res =
                    SaffireProioMixerProtocol::update(req, node, &params, &mut self.0, timeout_ms);
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            PRO_MIXER_STREAM_SRC_PAIR_0_NAME => {
                let mut params = self.0.clone();
                let stream_source_pair0 = &mut params.stream_source_pair0;
                let vals = &elem_value.int()[..stream_source_pair0.len()];
                stream_source_pair0
                    .iter_mut()
                    .zip(vals)
                    .for_each(|(level, &val)| *level = val as i16);
                let res =
                    SaffireProioMixerProtocol::update(req, node, &params, &mut self.0, timeout_ms);
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            PRO_MIXER_STREAM_SRC_NAME => {
                let mut params = self.0.clone();
                let stream_sources = &mut params.stream_sources;
                let vals = &elem_value.int()[..stream_sources.len()];
                stream_sources
                    .iter_mut()
                    .zip(vals)
                    .for_each(|(level, &val)| *level = val as i16);
                let res =
                    SaffireProioMixerProtocol::update(req, node, &params, &mut self.0, timeout_ms);
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
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

    fn load_params(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
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

        Ok(())
    }

    fn cache(&mut self, req: &FwReq, node: &FwNode, timeout_ms: u32) -> Result<(), Error> {
        let res = T::cache(req, node, self.state_mut(), timeout_ms);
        debug!(params = ?self.state(), ?res);
        res
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
        req: &FwReq,
        node: &FwNode,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            HEAD_ROOM_NAME => {
                let mut params = self.state().clone();
                params.head_room = elem_value.boolean()[0];
                let res = T::update(req, node, &params, self.state_mut(), timeout_ms);
                debug!(params = ?self.state(), ?res);
                res.map(|_| true)
            }
            PHANTOM_POWERING_NAME => {
                let mut params = self.state().clone();
                let phantom_powerings = &mut params.phantom_powerings;
                let vals = &elem_value.boolean()[..phantom_powerings.len()];
                phantom_powerings.copy_from_slice(&vals);
                let res = T::update(req, node, &params, self.state_mut(), timeout_ms);
                debug!(params = ?self.state(), ?res);
                res.map(|_| true)
            }
            INSERT_SWAP_NAME => {
                let mut params = self.state().clone();
                let insert_swaps = &mut params.insert_swaps;
                let vals = &elem_value.boolean()[..insert_swaps.len()];
                insert_swaps.copy_from_slice(&vals);
                let res = T::update(req, node, &params, self.state_mut(), timeout_ms);
                debug!(params = ?self.state(), ?res);
                res.map(|_| true)
            }
            STANDALONE_MODE_NAME => {
                let mut params = self.state().clone();
                let val = elem_value.enumerated()[0];
                params.standalone_mode = Self::STANDALONE_MODES
                    .iter()
                    .nth(val as usize)
                    .ok_or_else(|| {
                        let msg = format!("Invalid index of standalone mode: {}", val);
                        Error::new(FileError::Inval, &msg)
                    })
                    .copied()?;
                let res = T::update(req, node, &params, self.state_mut(), timeout_ms);
                debug!(params = ?self.state(), ?res);
                res.map(|_| true)
            }
            ADAT_ENABLE_NAME => {
                let mut params = self.state().clone();
                params.adat_enabled = elem_value.boolean()[0];
                let res = T::update(req, node, &params, self.state_mut(), timeout_ms);
                debug!(params = ?self.state(), ?res);
                res.map(|_| true)
            }
            DIRECT_MONITORING_NAME => {
                let mut params = self.state().clone();
                params.direct_monitoring = elem_value.boolean()[0];
                let res = T::update(req, node, &params, self.state_mut(), timeout_ms);
                debug!(params = ?self.state(), ?res);
                res.map(|_| true)
            }
            _ => Ok(false),
        }
    }
}
