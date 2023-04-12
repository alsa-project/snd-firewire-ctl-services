// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

use super::*;

#[derive(Default, Debug)]
pub struct SaffireLeModel {
    req: FwReq,
    avc: BebobAvc,
    clk_ctl: ClkCtl,
    meter_ctl: MeterCtl,
    out_ctl: OutputCtl,
    specific_ctl: SpecificCtl,
    mixer_low_rate_ctl: MixerLowRateCtl,
    mixer_middle_rate_ctl: MixerMiddleRateCtl,
    through_ctl: ThroughCtl,
}

// NOTE: At 88.2/96.0 kHz, AV/C transaction takes more time than 44.1/48.0 kHz.
const FCP_TIMEOUT_MS: u32 = 200;
const TIMEOUT_MS: u32 = 100;

#[derive(Default, Debug)]
struct ClkCtl(Vec<ElemId>, MediaClockParameters, SamplingClockParameters);

impl MediaClkFreqCtlOperation<SaffireLeClkProtocol> for ClkCtl {
    fn state(&self) -> &MediaClockParameters {
        &self.1
    }

    fn state_mut(&mut self) -> &mut MediaClockParameters {
        &mut self.1
    }
}

impl SamplingClkSrcCtlOperation<SaffireLeClkProtocol> for ClkCtl {
    const SRC_LABELS: &'static [&'static str] = &["Internal", "S/PDIF"];

    fn state(&self) -> &SamplingClockParameters {
        &self.2
    }

    fn state_mut(&mut self) -> &mut SamplingClockParameters {
        &mut self.2
    }
}

#[derive(Default, Debug)]
struct MeterCtl(Vec<ElemId>, SaffireLeMeter);

#[derive(Debug)]
struct OutputCtl(Vec<ElemId>, SaffireOutputParameters);

impl Default for OutputCtl {
    fn default() -> Self {
        Self(
            Default::default(),
            SaffireLeOutputProtocol::create_output_parameters(),
        )
    }
}

impl SaffireOutputCtlOperation<SaffireLeOutputProtocol> for OutputCtl {
    const OUTPUT_LABELS: &'static [&'static str] = &[
        "analog-output-1/2",
        "analog-output-3/4",
        "analog-output-5/6",
    ];

    fn state(&self) -> &SaffireOutputParameters {
        &self.1
    }

    fn state_mut(&mut self) -> &mut SaffireOutputParameters {
        &mut self.1
    }
}

#[derive(Default, Debug)]
struct SpecificCtl(SaffireLeSpecificParameters);

#[derive(Default, Debug)]
struct MixerLowRateCtl(SaffireLeMixerLowRateState);

#[derive(Default, Debug)]
struct MixerMiddleRateCtl(SaffireLeMixerMiddleRateState);

#[derive(Default, Debug)]
struct ThroughCtl(SaffireThroughParameters);

impl SaffireThroughCtlOperation<SaffireLeThroughProtocol> for ThroughCtl {
    fn state(&self) -> &SaffireThroughParameters {
        &self.0
    }

    fn state_mut(&mut self) -> &mut SaffireThroughParameters {
        &mut self.0
    }
}

impl CtlModel<(SndUnit, FwNode)> for SaffireLeModel {
    fn cache(&mut self, unit: &mut (SndUnit, FwNode)) -> Result<(), Error> {
        self.avc.bind(&unit.1)?;

        self.clk_ctl.cache_freq(&self.avc, FCP_TIMEOUT_MS)?;
        self.clk_ctl.cache_src(&self.avc, FCP_TIMEOUT_MS)?;
        self.meter_ctl.cache(&self.req, &unit.1, TIMEOUT_MS)?;
        self.out_ctl.cache(&self.req, &unit.1, TIMEOUT_MS)?;
        self.specific_ctl.cache(&self.req, &unit.1, TIMEOUT_MS)?;
        self.mixer_low_rate_ctl
            .cache(&self.req, &unit.1, TIMEOUT_MS)?;
        self.mixer_middle_rate_ctl
            .cache(&self.req, &unit.1, TIMEOUT_MS)?;
        self.through_ctl.cache(&self.req, &unit.1, TIMEOUT_MS)?;

        Ok(())
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.clk_ctl
            .load_freq(card_cntr)
            .map(|mut elem_id_list| self.clk_ctl.0.append(&mut elem_id_list))?;

        self.clk_ctl
            .load_src(card_cntr)
            .map(|mut elem_id_list| self.clk_ctl.0.append(&mut elem_id_list))?;

        self.meter_ctl
            .load_meter(card_cntr)
            .map(|mut elem_id_list| self.meter_ctl.0.append(&mut elem_id_list))?;

        self.out_ctl
            .load_params(card_cntr)
            .map(|mut elem_id_list| self.out_ctl.0.append(&mut elem_id_list))?;

        self.specific_ctl.load_params(card_cntr)?;

        self.mixer_low_rate_ctl.load_src_gains(card_cntr)?;
        self.mixer_middle_rate_ctl.load_src_gains(card_cntr)?;

        self.through_ctl.load_params(card_cntr)?;

        Ok(())
    }

    fn read(
        &mut self,
        _: &mut (SndUnit, FwNode),
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.clk_ctl.read_freq(elem_id, elem_value)? {
            Ok(true)
        } else if self.clk_ctl.read_src(elem_id, elem_value)? {
            Ok(true)
        } else if self.meter_ctl.read_meter(elem_id, elem_value)? {
            Ok(true)
        } else if self.out_ctl.read_params(elem_id, elem_value)? {
            Ok(true)
        } else if self.specific_ctl.read_params(elem_id, elem_value)? {
            Ok(true)
        } else if self
            .mixer_low_rate_ctl
            .read_src_gains(elem_id, elem_value)?
        {
            Ok(true)
        } else if self
            .mixer_middle_rate_ctl
            .read_src_gains(elem_id, elem_value)?
        {
            Ok(true)
        } else if self.through_ctl.read_params(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(
        &mut self,
        unit: &mut (SndUnit, FwNode),
        elem_id: &ElemId,
        old: &ElemValue,
        new: &ElemValue,
    ) -> Result<bool, Error> {
        if self.clk_ctl.write_freq(
            &mut unit.0,
            &self.avc,
            elem_id,
            old,
            new,
            FCP_TIMEOUT_MS * 3,
        )? {
            Ok(true)
        } else if self.clk_ctl.write_src(
            &mut unit.0,
            &self.avc,
            elem_id,
            old,
            new,
            FCP_TIMEOUT_MS * 3,
        )? {
            Ok(true)
        } else if self
            .out_ctl
            .write_params(&self.req, &unit.1, elem_id, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .specific_ctl
            .write_params(&self.req, &unit.1, elem_id, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .mixer_low_rate_ctl
            .write_src_gains(&self.req, &unit.1, elem_id, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .mixer_middle_rate_ctl
            .write_src_gains(&self.req, &unit.1, elem_id, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .through_ctl
            .write_params(&self.req, &unit.1, elem_id, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl NotifyModel<(SndUnit, FwNode), bool> for SaffireLeModel {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.clk_ctl.0);
    }

    fn parse_notification(
        &mut self,
        _: &mut (SndUnit, FwNode),
        &locked: &bool,
    ) -> Result<(), Error> {
        if locked {
            self.clk_ctl.cache_freq(&self.avc, FCP_TIMEOUT_MS)?;
        }
        Ok(())
    }
}

impl MeasureModel<(SndUnit, FwNode)> for SaffireLeModel {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.meter_ctl.0);
        elem_id_list.extend_from_slice(&self.out_ctl.0);
    }

    fn measure_states(&mut self, unit: &mut (SndUnit, FwNode)) -> Result<(), Error> {
        self.meter_ctl.cache(&self.req, &unit.1, TIMEOUT_MS)?;
        self.out_ctl.cache(&self.req, &unit.1, TIMEOUT_MS)?;
        Ok(())
    }

    fn measure_elem(
        &mut self,
        _: &(SndUnit, FwNode),
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.meter_ctl.read_meter(elem_id, elem_value)? {
            Ok(true)
        } else if self.out_ctl.read_params(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

const METER_STREAM_INPUT_NAME: &str = "stream-input-meter";
const METER_DIG_INPUT_DETECT_NAME: &str = "digital-input-detection";

impl MeterCtl {
    const PHYS_INPUT_LABELS: &'static [&'static str] = &[
        "analog-input-1",
        "analog-input-2",
        "analog-input-3",
        "analog-input-4",
        "digital-input-1",
        "digital-input-2",
    ];
    const STREAM_INPUT_LABELS: &'static [&'static str] = &[
        "stream-input-1/2",
        "stream-input-3/4",
        "stream-input-5/6",
        "stream-input-7/8",
    ];
    const PHYS_OUTPUT_LABELS: &'static [&'static str] = &[
        "analog-output-1",
        "analog-output-2",
        "analog-output-3",
        "analog-output-4",
        "analog-output-5",
        "analog-output-6",
        "digital-output-1",
        "digital-output-2",
    ];

    fn load_meter(&mut self, card_cntr: &mut CardCntr) -> Result<Vec<ElemId>, Error> {
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

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, METER_STREAM_INPUT_NAME, 0);
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

        let elem_id =
            ElemId::new_by_name(ElemIfaceType::Card, 0, 0, METER_DIG_INPUT_DETECT_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, 1, 1, false)
            .map(|mut elem_id_list| measured_elem_id_list.append(&mut elem_id_list))?;

        Ok(measured_elem_id_list)
    }

    fn cache(&mut self, req: &FwReq, node: &FwNode, timeout_ms: u32) -> Result<(), Error> {
        SaffireLeMeterProtocol::cache(req, node, &mut self.1, timeout_ms)
    }

    fn read_meter(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
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

const ANALOG_INPUT_2_3_HIGH_GAIN: &str = "analog-input-2/3-high-gain";

impl SpecificCtl {
    fn load_params(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, ANALOG_INPUT_2_3_HIGH_GAIN, 0);
        card_cntr.add_bool_elems(&elem_id, 1, 2, false).map(|_| ())
    }

    fn cache(&mut self, req: &FwReq, node: &FwNode, timeout_ms: u32) -> Result<(), Error> {
        SaffireLeSpecificProtocol::cache(req, node, &mut self.0, timeout_ms)
    }

    fn read_params(&self, elem_id: &ElemId, elem_value: &ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            ANALOG_INPUT_2_3_HIGH_GAIN => {
                elem_value.set_bool(&self.0.analog_input_2_3_high_gains);
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
            ANALOG_INPUT_2_3_HIGH_GAIN => {
                let mut params = self.0.clone();
                let states = &mut params.analog_input_2_3_high_gains;
                let vals = &elem_value.boolean()[..states.len()];
                states.copy_from_slice(vals);
                SaffireLeSpecificProtocol::update(req, node, &params, &mut self.0, timeout_ms)
                    .map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

const PHYS_SRC_GAIN_NAME: &str = "mixer:low:phys-source-gain";
const STREAM_SRC_GAIN_NAME: &str = "mixer:low:stream-source-gain";

impl MixerLowRateCtl {
    fn load_src_gains(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, PHYS_SRC_GAIN_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                self.0.phys_src_gains.len(),
                SaffireLeMixerLowRateProtocol::LEVEL_MIN as i32,
                SaffireLeMixerLowRateProtocol::LEVEL_MAX as i32,
                SaffireLeMixerLowRateProtocol::LEVEL_STEP as i32,
                self.0.phys_src_gains[0].len(),
                Some(&Into::<Vec<u32>>::into(LEVEL_TLV)),
                true,
            )
            .map(|_| ())?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, STREAM_SRC_GAIN_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                self.0.stream_src_gains.len(),
                SaffireLeMixerLowRateProtocol::LEVEL_MIN as i32,
                SaffireLeMixerLowRateProtocol::LEVEL_MAX as i32,
                SaffireLeMixerLowRateProtocol::LEVEL_STEP as i32,
                self.0.stream_src_gains[0].len(),
                Some(&Into::<Vec<u32>>::into(LEVEL_TLV)),
                true,
            )
            .map(|_| ())?;

        Ok(())
    }

    fn cache(&mut self, req: &FwReq, node: &FwNode, timeout_ms: u32) -> Result<(), Error> {
        SaffireLeMixerLowRateProtocol::cache(req, node, &mut self.0, timeout_ms)
    }

    fn read_src_gains(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            PHYS_SRC_GAIN_NAME => read_mixer_src_gains(elem_value, elem_id, &self.0.phys_src_gains),
            STREAM_SRC_GAIN_NAME => {
                read_mixer_src_gains(elem_value, elem_id, &self.0.stream_src_gains)
            }
            _ => Ok(false),
        }
    }

    fn write_src_gains(
        &mut self,
        req: &FwReq,
        node: &FwNode,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            PHYS_SRC_GAIN_NAME => {
                let index = elem_id.index() as usize;
                let mut params = self.0.clone();
                let levels = &mut params.phys_src_gains[index];
                let vals = &elem_value.int()[..levels.len()];
                levels
                    .iter_mut()
                    .zip(vals)
                    .for_each(|(level, &val)| *level = val as i16);
                SaffireLeMixerLowRateProtocol::update(req, node, &params, &mut self.0, timeout_ms)
                    .map(|_| true)
            }
            STREAM_SRC_GAIN_NAME => {
                let index = elem_id.index() as usize;
                let mut params = self.0.clone();
                let levels = &mut params.stream_src_gains[index];
                let vals = &elem_value.int()[..levels.len()];
                levels
                    .iter_mut()
                    .zip(vals)
                    .for_each(|(level, &val)| *level = val as i16);
                SaffireLeMixerLowRateProtocol::update(req, node, &params, &mut self.0, timeout_ms)
                    .map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

const MIDDLE_MONITOR_PHYS_SRC_GAIN_NAME: &str = "monitor:middle:phys-source-gain";
const MIDDLE_MONITOR_SRC_GAIN_NAME: &str = "mixer:middle:monitor-source-gain";
const MIDDLE_STREAM_SRC_GAIN_NAME: &str = "mixer:middle:stream-source-gain";

impl MixerMiddleRateCtl {
    fn load_src_gains(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(
            ElemIfaceType::Mixer,
            0,
            0,
            MIDDLE_MONITOR_PHYS_SRC_GAIN_NAME,
            0,
        );
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                SaffireLeMixerMiddleRateProtocol::LEVEL_MIN as i32,
                SaffireLeMixerMiddleRateProtocol::LEVEL_MAX as i32,
                SaffireLeMixerMiddleRateProtocol::LEVEL_STEP as i32,
                self.0.monitor_src_phys_input_gains.len(),
                Some(&Into::<Vec<u32>>::into(LEVEL_TLV)),
                true,
            )
            .map(|_| ())?;

        let elem_id =
            ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIDDLE_MONITOR_SRC_GAIN_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                self.0.monitor_out_src_pair_gains.len(),
                SaffireLeMixerMiddleRateProtocol::LEVEL_MIN as i32,
                SaffireLeMixerMiddleRateProtocol::LEVEL_MAX as i32,
                SaffireLeMixerMiddleRateProtocol::LEVEL_STEP as i32,
                self.0.monitor_out_src_pair_gains[0].len(),
                Some(&Into::<Vec<u32>>::into(LEVEL_TLV)),
                true,
            )
            .map(|_| ())?;

        let elem_id =
            ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIDDLE_STREAM_SRC_GAIN_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                self.0.stream_src_pair_gains.len(),
                SaffireLeMixerMiddleRateProtocol::LEVEL_MIN as i32,
                SaffireLeMixerMiddleRateProtocol::LEVEL_MAX as i32,
                SaffireLeMixerMiddleRateProtocol::LEVEL_STEP as i32,
                self.0.stream_src_pair_gains[0].len(),
                Some(&Into::<Vec<u32>>::into(LEVEL_TLV)),
                true,
            )
            .map(|_| ())?;

        Ok(())
    }

    fn cache(&mut self, req: &FwReq, node: &FwNode, timeout_ms: u32) -> Result<(), Error> {
        SaffireLeMixerMiddleRateProtocol::cache(req, node, &mut self.0, timeout_ms)
    }

    fn read_src_gains(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            MIDDLE_MONITOR_PHYS_SRC_GAIN_NAME => {
                let gains: Vec<i32> = self
                    .0
                    .monitor_src_phys_input_gains
                    .iter()
                    .map(|&val| val as i32)
                    .collect();
                elem_value.set_int(&gains);
                Ok(true)
            }
            MIDDLE_MONITOR_SRC_GAIN_NAME => {
                read_mixer_src_gains(elem_value, elem_id, &self.0.monitor_out_src_pair_gains)
            }
            MIDDLE_STREAM_SRC_GAIN_NAME => {
                read_mixer_src_gains(elem_value, elem_id, &self.0.stream_src_pair_gains)
            }
            _ => Ok(false),
        }
    }

    fn write_src_gains(
        &mut self,
        req: &FwReq,
        node: &FwNode,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            MIDDLE_MONITOR_PHYS_SRC_GAIN_NAME => {
                let mut params = self.0.clone();
                let levels = &mut params.monitor_src_phys_input_gains;
                let vals = &elem_value.int()[..levels.len()];
                levels
                    .iter_mut()
                    .zip(vals)
                    .for_each(|(level, &val)| *level = val as i16);
                SaffireLeMixerMiddleRateProtocol::update(
                    req,
                    node,
                    &params,
                    &mut self.0,
                    timeout_ms,
                )
                .map(|_| true)
            }
            MIDDLE_MONITOR_SRC_GAIN_NAME => {
                let index = elem_id.index() as usize;
                let mut params = self.0.clone();
                let levels = &mut params.monitor_out_src_pair_gains[index];
                let vals = &elem_value.int()[..levels.len()];
                levels
                    .iter_mut()
                    .zip(vals)
                    .for_each(|(level, &val)| *level = val as i16);
                SaffireLeMixerMiddleRateProtocol::update(
                    req,
                    node,
                    &params,
                    &mut self.0,
                    timeout_ms,
                )
                .map(|_| true)
            }
            MIDDLE_STREAM_SRC_GAIN_NAME => {
                let index = elem_id.index() as usize;
                let mut params = self.0.clone();
                let levels = &mut params.stream_src_pair_gains[index];
                let vals = &elem_value.int()[..levels.len()];
                levels
                    .iter_mut()
                    .zip(vals)
                    .for_each(|(level, &val)| *level = val as i16);
                SaffireLeMixerMiddleRateProtocol::update(
                    req,
                    node,
                    &params,
                    &mut self.0,
                    timeout_ms,
                )
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

fn read_mixer_src_gains<T>(
    elem_value: &mut ElemValue,
    elem_id: &ElemId,
    levels_list: &[T],
) -> Result<bool, Error>
where
    T: AsRef<[i16]>,
{
    let index = elem_id.index() as usize;
    levels_list
        .iter()
        .nth(index)
        .ok_or_else(|| {
            let msg = format!("Invalid index of source level list {}", index);
            Error::new(FileError::Inval, &msg)
        })
        .map(|levels| {
            let vals: Vec<i32> = levels.as_ref().iter().fold(Vec::new(), |mut vals, &level| {
                vals.push(level as i32);
                vals
            });
            elem_value.set_int(&vals);
            true
        })
}

#[cfg(test)]
mod test {
    use super::*;
    use alsactl::CardError;

    #[test]
    fn test_clk_ctl_definition() {
        let mut card_cntr = CardCntr::default();
        let mut ctl = ClkCtl::default();

        let error = ctl.load_freq(&mut card_cntr).unwrap_err();
        assert_eq!(error.kind::<CardError>(), Some(CardError::Failed));

        let error = ctl.load_src(&mut card_cntr).unwrap_err();
        assert_eq!(error.kind::<CardError>(), Some(CardError::Failed));
    }
}
