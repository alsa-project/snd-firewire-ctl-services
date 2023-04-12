// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

use {super::*, protocols::lexicon::*};

#[derive(Default)]
pub struct IonixModel {
    req: FwReq,
    sections: GeneralSections,
    common_ctl: CommonCtl<IonixProtocol>,
    meter_ctl: MeterCtl,
    mixer_ctl: MixerCtl,
}

const TIMEOUT_MS: u32 = 20;

impl CtlModel<(SndDice, FwNode)> for IonixModel {
    fn cache(&mut self, unit: &mut (SndDice, FwNode)) -> Result<(), Error> {
        IonixProtocol::read_general_sections(&self.req, &unit.1, &mut self.sections, TIMEOUT_MS)?;

        self.common_ctl
            .cache_whole_params(&self.req, &unit.1, &mut self.sections, TIMEOUT_MS)?;
        self.meter_ctl.cache(&self.req, &unit.1, TIMEOUT_MS)?;
        self.mixer_ctl.cache(&self.req, &unit.1, TIMEOUT_MS)?;

        Ok(())
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.common_ctl.load(card_cntr)?;

        self.meter_ctl.load(card_cntr)?;
        self.mixer_ctl.load(card_cntr)?;

        Ok(())
    }

    fn read(
        &mut self,
        _: &mut (SndDice, FwNode),
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.common_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(
        &mut self,
        unit: &mut (SndDice, FwNode),
        elem_id: &ElemId,
        _: &ElemValue,
        new: &ElemValue,
    ) -> Result<bool, Error> {
        if self.common_ctl.write(
            &unit.0,
            &self.req,
            &unit.1,
            &mut self.sections,
            elem_id,
            new,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self
            .mixer_ctl
            .write(&self.req, &unit.1, elem_id, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl NotifyModel<(SndDice, FwNode), u32> for IonixModel {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.common_ctl.notified_elem_id_list);
    }

    fn parse_notification(&mut self, unit: &mut (SndDice, FwNode), msg: &u32) -> Result<(), Error> {
        self.common_ctl
            .parse_notification(&self.req, &unit.1, &mut self.sections, *msg, TIMEOUT_MS)
    }

    fn read_notified_elem(
        &mut self,
        _: &(SndDice, FwNode),
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        self.common_ctl.read(elem_id, elem_value)
    }
}

impl MeasureModel<(SndDice, FwNode)> for IonixModel {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.common_ctl.measured_elem_id_list);
        elem_id_list.extend_from_slice(&self.meter_ctl.1);
    }

    fn measure_states(&mut self, unit: &mut (SndDice, FwNode)) -> Result<(), Error> {
        self.common_ctl
            .cache_partial_params(&self.req, &unit.1, &mut self.sections, TIMEOUT_MS)?;
        self.meter_ctl.cache(&self.req, &unit.1, TIMEOUT_MS)?;
        Ok(())
    }

    fn measure_elem(
        &mut self,
        _: &(SndDice, FwNode),
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.common_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.meter_ctl.read_measured_elem(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[derive(Default, Debug)]
struct MeterCtl(IonixMeter, Vec<ElemId>);

impl MeterCtl {
    const SPDIF_INPUT_NAME: &'static str = "spdif-input-meter";
    const STREAM_INPUT_NAME: &'static str = "stream-input-meter";
    const ANALOG_INPUT_NAME: &'static str = "analog-input-meter";
    const BUS_OUTPUT_NAME: &'static str = "bus-output-meter";
    const MAIN_OUTPUT_NAME: &'static str = "main-output-meter";

    const LEVEL_MIN: i32 = 0;
    const LEVEL_MAX: i32 = 0x00000fff;
    const LEVEL_STEP: i32 = 1;
    const LEVEL_TLV: DbInterval = DbInterval {
        min: -6000,
        max: 0,
        linear: false,
        mute_avail: false,
    };

    fn cache(&mut self, req: &FwReq, node: &FwNode, timeout_ms: u32) -> Result<(), Error> {
        let res = IonixProtocol::cache_whole_params(req, node, &mut self.0, timeout_ms);
        debug!(params = ?self.0, ?res);
        res
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        [
            (Self::SPDIF_INPUT_NAME, IonixProtocol::SPDIF_INPUT_COUNT),
            (Self::STREAM_INPUT_NAME, IonixProtocol::STREAM_INPUT_COUNT),
            (Self::ANALOG_INPUT_NAME, IonixProtocol::ANALOG_INPUT_COUNT),
            (Self::BUS_OUTPUT_NAME, IonixProtocol::MIXER_BUS_COUNT),
            (Self::MAIN_OUTPUT_NAME, IonixProtocol::MIXER_MAIN_COUNT),
        ]
        .iter()
        .try_for_each(|&(name, count)| {
            let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
            card_cntr
                .add_int_elems(
                    &elem_id,
                    1,
                    Self::LEVEL_MIN,
                    Self::LEVEL_MAX,
                    Self::LEVEL_STEP,
                    count,
                    Some(&Vec::<u32>::from(Self::LEVEL_TLV)),
                    false,
                )
                .map(|mut elem_id_list| self.1.append(&mut elem_id_list))
        })
    }

    fn read_measured_elem(&self, elem_id: &ElemId, elem_value: &ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            Self::SPDIF_INPUT_NAME => {
                let vals: Vec<i32> = self.0.spdif_inputs.iter().map(|&val| val as i32).collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            Self::STREAM_INPUT_NAME => {
                let vals: Vec<i32> = self.0.stream_inputs.iter().map(|&val| val as i32).collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            Self::ANALOG_INPUT_NAME => {
                let vals: Vec<i32> = self.0.analog_inputs.iter().map(|&val| val as i32).collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            Self::BUS_OUTPUT_NAME => {
                let vals: Vec<i32> = self.0.bus_outputs.iter().map(|&val| val as i32).collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            Self::MAIN_OUTPUT_NAME => {
                let vals: Vec<i32> = self.0.main_outputs.iter().map(|&val| val as i32).collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

#[derive(Default, Debug)]
struct MixerCtl(IonixMixerParameters);

impl MixerCtl {
    const BUS_SRC_GAIN_NAME: &'static str = "mixer-bus-input-gain";
    const MAIN_SRC_GAIN_NAME: &'static str = "mixer-main-input-gain";
    const REVERB_SRC_GAIN_NAME: &'static str = "mixer-reverb-input-gain";

    const GAIN_MIN: i32 = 0;
    const GAIN_MAX: i32 = 0x00007fff;
    const GAIN_STEP: i32 = 1;
    const GAIN_TLV: DbInterval = DbInterval {
        min: -6000,
        max: 0,
        linear: false,
        mute_avail: false,
    };

    fn cache(&mut self, req: &FwReq, node: &FwNode, timeout_ms: u32) -> Result<(), Error> {
        let res = IonixProtocol::cache_whole_params(req, node, &mut self.0, timeout_ms);
        debug!(params = ?self.0, ?res);
        res
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let mut labels = Vec::new();
        [
            ("analog-input", IonixProtocol::ANALOG_INPUT_COUNT),
            ("spdif-input", IonixProtocol::SPDIF_INPUT_COUNT),
            ("stream-input", IonixProtocol::STREAM_INPUT_COUNT),
        ]
        .iter()
        .for_each(|(label, count)| {
            (0..*count).for_each(|i| labels.push(format!("{}-{}", label, i)));
        });

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::BUS_SRC_GAIN_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            IonixProtocol::MIXER_BUS_COUNT,
            Self::GAIN_MIN,
            Self::GAIN_MAX,
            Self::GAIN_STEP,
            labels.len(),
            Some(&Vec::<u32>::from(Self::GAIN_TLV)),
            true,
        )?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::MAIN_SRC_GAIN_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            IonixProtocol::MIXER_MAIN_COUNT,
            Self::GAIN_MIN,
            Self::GAIN_MAX,
            Self::GAIN_STEP,
            labels.len(),
            Some(&Vec::<u32>::from(Self::GAIN_TLV)),
            true,
        )?;

        let elem_id =
            ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::REVERB_SRC_GAIN_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            IonixProtocol::MIXER_REVERB_COUNT,
            Self::GAIN_MIN,
            Self::GAIN_MAX,
            Self::GAIN_STEP,
            labels.len(),
            Some(&Vec::<u32>::from(Self::GAIN_TLV)),
            true,
        )?;

        Ok(())
    }

    fn read(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            Self::BUS_SRC_GAIN_NAME => {
                let mixer = elem_id.index() as usize;
                let srcs = self.0.bus_sources.iter().nth(mixer).ok_or_else(|| {
                    let msg = format!("Mixer not found for position {}", mixer);
                    Error::new(FileError::Inval, &msg)
                })?;
                let gains: Vec<i32> = srcs
                    .analog_inputs
                    .iter()
                    .chain(srcs.spdif_inputs.iter())
                    .chain(srcs.stream_inputs.iter())
                    .map(|&gain| gain as i32)
                    .collect();
                elem_value.set_int(&gains);
                Ok(true)
            }
            Self::MAIN_SRC_GAIN_NAME => {
                let mixer = elem_id.index() as usize;
                let srcs = self.0.main_sources.iter().nth(mixer).ok_or_else(|| {
                    let msg = format!("Mixer not found for position {}", mixer);
                    Error::new(FileError::Inval, &msg)
                })?;
                let gains: Vec<i32> = srcs
                    .analog_inputs
                    .iter()
                    .chain(srcs.spdif_inputs.iter())
                    .chain(srcs.stream_inputs.iter())
                    .map(|&gain| gain as i32)
                    .collect();
                elem_value.set_int(&gains);
                Ok(true)
            }
            Self::REVERB_SRC_GAIN_NAME => {
                let mixer = elem_id.index() as usize;
                let srcs = self.0.reverb_sources.iter().nth(mixer).ok_or_else(|| {
                    let msg = format!("Mixer not found for position {}", mixer);
                    Error::new(FileError::Inval, &msg)
                })?;
                let gains: Vec<i32> = srcs
                    .analog_inputs
                    .iter()
                    .chain(srcs.spdif_inputs.iter())
                    .chain(srcs.stream_inputs.iter())
                    .map(|&gain| gain as i32)
                    .collect();
                elem_value.set_int(&gains);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write(
        &mut self,
        req: &FwReq,
        node: &FwNode,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            Self::BUS_SRC_GAIN_NAME => {
                let mixer = elem_id.index() as usize;
                let mut params = self.0.clone();
                let srcs = params.bus_sources.iter_mut().nth(mixer).ok_or_else(|| {
                    let msg = format!("Mixer not found for position {}", mixer);
                    Error::new(FileError::Inval, &msg)
                })?;
                srcs.analog_inputs
                    .iter_mut()
                    .chain(srcs.spdif_inputs.iter_mut())
                    .chain(srcs.stream_inputs.iter_mut())
                    .zip(elem_value.int())
                    .for_each(|(gain, &val)| *gain = val as i16);
                let res = IonixProtocol::update_partial_parameters(
                    req,
                    node,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            Self::MAIN_SRC_GAIN_NAME => {
                let mixer = elem_id.index() as usize;
                let mut params = self.0.clone();
                let srcs = params.main_sources.iter_mut().nth(mixer).ok_or_else(|| {
                    let msg = format!("Mixer not found for position {}", mixer);
                    Error::new(FileError::Inval, &msg)
                })?;
                srcs.analog_inputs
                    .iter_mut()
                    .chain(srcs.spdif_inputs.iter_mut())
                    .chain(srcs.stream_inputs.iter_mut())
                    .zip(elem_value.int())
                    .for_each(|(gain, &val)| *gain = val as i16);
                let res = IonixProtocol::update_partial_parameters(
                    req,
                    node,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            Self::REVERB_SRC_GAIN_NAME => {
                let mixer = elem_id.index() as usize;
                let mut params = self.0.clone();
                let srcs = params.reverb_sources.iter_mut().nth(mixer).ok_or_else(|| {
                    let msg = format!("Mixer not found for position {}", mixer);
                    Error::new(FileError::Inval, &msg)
                })?;
                srcs.analog_inputs
                    .iter_mut()
                    .chain(srcs.spdif_inputs.iter_mut())
                    .chain(srcs.stream_inputs.iter_mut())
                    .zip(elem_value.int())
                    .for_each(|(gain, &val)| *gain = val as i16);
                let res = IonixProtocol::update_partial_parameters(
                    req,
                    node,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            _ => Ok(false),
        }
    }
}
