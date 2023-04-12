// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use {super::*, protocols::alesis::*, std::marker::PhantomData};

const TIMEOUT_MS: u32 = 20;

pub type Io14fwModel = IofwModel<Io14fwProtocol>;
pub type Io26fwModel = IofwModel<Io26fwProtocol>;

#[derive(Default)]
pub struct IofwModel<T>
where
    T: IofwMeterSpecification
        + AlesisParametersOperation<IofwMeterParams>
        + IofwOutputSpecification
        + AlesisParametersOperation<IofwOutputParams>
        + AlesisMutableParametersOperation<IofwOutputParams>
        + AlesisParametersOperation<IofwOutputParams>
        + AlesisMutableParametersOperation<IofwOutputParams>
        + IofwMixerSpecification
        + AlesisParametersOperation<IofwMixerParams>
        + AlesisMutableParametersOperation<IofwMixerParams>
        + TcatNotifiedSectionOperation<GlobalParameters>
        + TcatFluctuatedSectionOperation<GlobalParameters>
        + TcatMutableSectionOperation<GlobalParameters>
        + TcatNotifiedSectionOperation<TxStreamFormatParameters>
        + TcatNotifiedSectionOperation<RxStreamFormatParameters>
        + TcatSectionOperation<ExtendedSyncParameters>,
{
    req: FwReq,
    sections: GeneralSections,
    common_ctl: CommonCtl<T>,
    meter_ctl: MeterCtl<T>,
    mixer_ctl: MixerCtl<T>,
    output_ctl: OutputCtl<T>,
}

impl<T> CtlModel<(SndDice, FwNode)> for IofwModel<T>
where
    T: IofwMeterSpecification
        + AlesisParametersOperation<IofwMeterParams>
        + IofwOutputSpecification
        + AlesisParametersOperation<IofwOutputParams>
        + AlesisMutableParametersOperation<IofwOutputParams>
        + AlesisParametersOperation<IofwOutputParams>
        + AlesisMutableParametersOperation<IofwOutputParams>
        + IofwMixerSpecification
        + AlesisParametersOperation<IofwMixerParams>
        + AlesisMutableParametersOperation<IofwMixerParams>
        + TcatNotifiedSectionOperation<GlobalParameters>
        + TcatFluctuatedSectionOperation<GlobalParameters>
        + TcatMutableSectionOperation<GlobalParameters>
        + TcatNotifiedSectionOperation<TxStreamFormatParameters>
        + TcatNotifiedSectionOperation<RxStreamFormatParameters>
        + TcatSectionOperation<ExtendedSyncParameters>,
{
    fn cache(&mut self, unit: &mut (SndDice, FwNode)) -> Result<(), Error> {
        T::read_general_sections(&self.req, &unit.1, &mut self.sections, TIMEOUT_MS)?;

        self.common_ctl
            .cache_whole_params(&self.req, &unit.1, &mut self.sections, TIMEOUT_MS)?;

        self.meter_ctl.cache(&self.req, &unit.1, TIMEOUT_MS)?;
        self.output_ctl.cache(&self.req, &unit.1, TIMEOUT_MS)?;
        self.mixer_ctl.whole_cache(&self.req, &unit.1, TIMEOUT_MS)?;

        Ok(())
    }

    fn load(&mut self, _: &mut (SndDice, FwNode), card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.common_ctl.load(card_cntr)?;

        self.meter_ctl.load(card_cntr)?;
        self.output_ctl.load(card_cntr)?;
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
        } else if self.meter_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.output_ctl.read(elem_id, elem_value)? {
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
            .output_ctl
            .write(&self.req, &unit.1, elem_id, new, TIMEOUT_MS)?
        {
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

impl<T> NotifyModel<(SndDice, FwNode), u32> for IofwModel<T>
where
    T: IofwMeterSpecification
        + AlesisParametersOperation<IofwMeterParams>
        + IofwOutputSpecification
        + AlesisParametersOperation<IofwOutputParams>
        + AlesisMutableParametersOperation<IofwOutputParams>
        + AlesisParametersOperation<IofwOutputParams>
        + AlesisMutableParametersOperation<IofwOutputParams>
        + IofwMixerSpecification
        + AlesisParametersOperation<IofwMixerParams>
        + AlesisMutableParametersOperation<IofwMixerParams>
        + TcatNotifiedSectionOperation<GlobalParameters>
        + TcatFluctuatedSectionOperation<GlobalParameters>
        + TcatMutableSectionOperation<GlobalParameters>
        + TcatNotifiedSectionOperation<TxStreamFormatParameters>
        + TcatNotifiedSectionOperation<RxStreamFormatParameters>
        + TcatSectionOperation<ExtendedSyncParameters>,
{
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

impl<T> MeasureModel<(SndDice, FwNode)> for IofwModel<T>
where
    T: IofwMeterSpecification
        + AlesisParametersOperation<IofwMeterParams>
        + IofwOutputSpecification
        + AlesisParametersOperation<IofwOutputParams>
        + AlesisMutableParametersOperation<IofwOutputParams>
        + AlesisParametersOperation<IofwOutputParams>
        + AlesisMutableParametersOperation<IofwOutputParams>
        + IofwMixerSpecification
        + AlesisParametersOperation<IofwMixerParams>
        + AlesisMutableParametersOperation<IofwMixerParams>
        + TcatNotifiedSectionOperation<GlobalParameters>
        + TcatFluctuatedSectionOperation<GlobalParameters>
        + TcatMutableSectionOperation<GlobalParameters>
        + TcatNotifiedSectionOperation<TxStreamFormatParameters>
        + TcatNotifiedSectionOperation<RxStreamFormatParameters>
        + TcatSectionOperation<ExtendedSyncParameters>,
{
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.common_ctl.measured_elem_id_list);
        elem_id_list.extend_from_slice(&self.meter_ctl.1);
        elem_id_list.extend_from_slice(&self.mixer_ctl.1);
    }

    fn measure_states(&mut self, unit: &mut (SndDice, FwNode)) -> Result<(), Error> {
        self.common_ctl
            .cache_partial_params(&self.req, &unit.1, &mut self.sections, TIMEOUT_MS)?;
        self.meter_ctl.cache(&self.req, &unit.1, TIMEOUT_MS)?;
        self.mixer_ctl
            .partial_cache(&self.req, &unit.1, TIMEOUT_MS)?;
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
        } else if self.meter_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[derive(Debug)]
struct MeterCtl<T>(IofwMeterParams, Vec<ElemId>, PhantomData<T>)
where
    T: IofwMeterSpecification + AlesisParametersOperation<IofwMeterParams>;

impl<T> Default for MeterCtl<T>
where
    T: IofwMeterSpecification + AlesisParametersOperation<IofwMeterParams>,
{
    fn default() -> Self {
        Self(
            T::create_meter_params(),
            Default::default(),
            Default::default(),
        )
    }
}

const ANALOG_INPUT_METER_NAME: &str = "analog-input-meters";
const DIGITAL_A_INPUT_METER_NAME: &str = "digital-a-input-meters";
const DIGITAL_B_INPUT_METER_NAME: &str = "digital-b-input-meters";
const MIXER_OUT_METER_NAME: &str = "mixer-output-meters";

impl<T> MeterCtl<T>
where
    T: IofwMeterSpecification + AlesisParametersOperation<IofwMeterParams>,
{
    const LEVEL_MIN: i32 = T::LEVEL_MIN as i32;
    const LEVEL_MAX: i32 = T::LEVEL_MAX as i32;
    const LEVEL_STEP: i32 = 1;
    const LEVEL_TLV: DbInterval = DbInterval {
        min: -9000,
        max: 0,
        linear: false,
        mute_avail: false,
    };

    fn cache(&mut self, req: &FwReq, node: &FwNode, timeout_ms: u32) -> Result<(), Error> {
        let res = T::cache_whole_params(req, node, &mut self.0, timeout_ms);
        debug!(params = ?self.0, ?res);
        res
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, ANALOG_INPUT_METER_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                Self::LEVEL_MIN,
                Self::LEVEL_MAX,
                Self::LEVEL_STEP,
                T::ANALOG_INPUT_COUNT,
                Some(&Vec::<u32>::from(Self::LEVEL_TLV)),
                false,
            )
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        let elem_id =
            ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, DIGITAL_A_INPUT_METER_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                Self::LEVEL_MIN,
                Self::LEVEL_MAX,
                Self::LEVEL_STEP,
                T::DIGITAL_A_INPUT_COUNT,
                Some(&Vec::<u32>::from(Self::LEVEL_TLV)),
                false,
            )
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        let elem_id =
            ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, DIGITAL_B_INPUT_METER_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                Self::LEVEL_MIN,
                Self::LEVEL_MAX,
                Self::LEVEL_STEP,
                T::DIGITAL_B_INPUT_COUNT,
                Some(&Vec::<u32>::from(Self::LEVEL_TLV)),
                false,
            )
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIXER_OUT_METER_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                Self::LEVEL_MIN,
                Self::LEVEL_MAX,
                Self::LEVEL_STEP,
                T::MIXER_OUTPUT_COUNT,
                Some(&Vec::<u32>::from(Self::LEVEL_TLV)),
                false,
            )
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        Ok(())
    }

    fn read(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            ANALOG_INPUT_METER_NAME => {
                let params = &self.0;
                let vals: Vec<i32> = params.analog_inputs.iter().map(|&val| val as i32).collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            DIGITAL_A_INPUT_METER_NAME => {
                let params = &self.0;
                let vals: Vec<i32> = params
                    .digital_a_inputs
                    .iter()
                    .map(|&val| val as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            DIGITAL_B_INPUT_METER_NAME => {
                let params = &self.0;
                let vals: Vec<i32> = params
                    .digital_b_inputs
                    .iter()
                    .map(|&val| val as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            MIXER_OUT_METER_NAME => {
                let params = &self.0;
                let vals: Vec<i32> = params.mixer_outputs.iter().map(|&val| val as i32).collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

#[derive(Debug)]
struct OutputCtl<T>(IofwOutputParams, PhantomData<T>)
where
    T: IofwOutputSpecification
        + AlesisParametersOperation<IofwOutputParams>
        + AlesisMutableParametersOperation<IofwOutputParams>;

impl<T> Default for OutputCtl<T>
where
    T: IofwOutputSpecification
        + AlesisParametersOperation<IofwOutputParams>
        + AlesisMutableParametersOperation<IofwOutputParams>,
{
    fn default() -> Self {
        Self(T::create_output_params(), Default::default())
    }
}

fn nominal_signal_level_to_str(level: &NominalSignalLevel) -> &'static str {
    match level {
        NominalSignalLevel::Consumer => "-10dBV",
        NominalSignalLevel::Professional => "+4dBu",
    }
}

fn digital_b_67_src_to_str(src: &DigitalB67Src) -> &'static str {
    match src {
        DigitalB67Src::Spdif12 => "S/PDIF-input-1/2",
        DigitalB67Src::Adat67 => "ADAT-input-7/8",
    }
}

fn mixer_out_pair_to_str(pair: &MixerOutPair) -> &'static str {
    match pair {
        MixerOutPair::Mixer01 => "Mixer-output-1/2",
        MixerOutPair::Mixer23 => "Mixer-output-3/4",
        MixerOutPair::Mixer45 => "Mixer-output-5/6",
        MixerOutPair::Mixer67 => "Mixer-output-7/8",
    }
}

const OUT_LEVEL_NAME: &str = "output-level";
const DIGITAL_B_67_SRC_NAME: &str = "monitor-digital-b-7/8-source";
const SPDIF_OUT_SRC_NAME: &str = "S/PDIF-1/2-output-source";
const HP23_SRC_NAME: &str = "Headphone-3/4-output-source";

impl<T> OutputCtl<T>
where
    T: IofwOutputSpecification
        + AlesisParametersOperation<IofwOutputParams>
        + AlesisMutableParametersOperation<IofwOutputParams>,
{
    const OUT_LEVELS: [NominalSignalLevel; 2] = [
        NominalSignalLevel::Consumer,
        NominalSignalLevel::Professional,
    ];

    const DIGITAL_B_67_SRCS: [DigitalB67Src; 2] = [DigitalB67Src::Spdif12, DigitalB67Src::Adat67];

    const MIXER_OUT_PAIRS: [MixerOutPair; 4] = [
        MixerOutPair::Mixer01,
        MixerOutPair::Mixer23,
        MixerOutPair::Mixer45,
        MixerOutPair::Mixer67,
    ];

    fn cache(&mut self, req: &FwReq, node: &FwNode, timeout_ms: u32) -> Result<(), Error> {
        let res = T::cache_whole_params(req, node, &mut self.0, timeout_ms);
        debug!(params = ?self.0, ?res);
        res
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let labels: Vec<&str> = Self::OUT_LEVELS
            .iter()
            .map(|l| nominal_signal_level_to_str(l))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, OUT_LEVEL_NAME, 0);
        let _ =
            card_cntr.add_enum_elems(&elem_id, 1, T::ANALOG_OUTPUT_COUNT, &labels, None, true)?;

        if T::HAS_OPT_IFACE_B {
            let labels: Vec<&str> = Self::DIGITAL_B_67_SRCS
                .iter()
                .map(|s| digital_b_67_src_to_str(s))
                .collect();
            let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, DIGITAL_B_67_SRC_NAME, 0);
            let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;
        }

        let labels: Vec<&str> = Self::MIXER_OUT_PAIRS
            .iter()
            .map(|p| mixer_out_pair_to_str(p))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, SPDIF_OUT_SRC_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, HP23_SRC_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        Ok(())
    }

    fn read(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            OUT_LEVEL_NAME => {
                let params = &self.0;
                let vals: Vec<u32> = params
                    .nominal_levels
                    .iter()
                    .map(|level| {
                        let pos = Self::OUT_LEVELS.iter().position(|l| level.eq(&l)).unwrap();
                        pos as u32
                    })
                    .collect();
                elem_value.set_enum(&vals);
                Ok(true)
            }
            DIGITAL_B_67_SRC_NAME => {
                let params = &self.0;
                let pos = Self::DIGITAL_B_67_SRCS
                    .iter()
                    .position(|s| params.digital_67_src.eq(&s))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            SPDIF_OUT_SRC_NAME => {
                let params = &self.0;
                let pos = Self::MIXER_OUT_PAIRS
                    .iter()
                    .position(|p| params.spdif_out_src.eq(&p))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            HP23_SRC_NAME => {
                let params = &self.0;
                let pos = Self::MIXER_OUT_PAIRS
                    .iter()
                    .position(|p| params.headphone2_3_out_src.eq(&p))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
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
            OUT_LEVEL_NAME => {
                let mut params = self.0.clone();
                params
                    .nominal_levels
                    .iter_mut()
                    .zip(elem_value.enumerated())
                    .try_for_each(|(level, &val)| {
                        let pos = val as usize;
                        Self::OUT_LEVELS
                            .iter()
                            .nth(pos)
                            .ok_or_else(|| {
                                let msg =
                                    format!("Nominal output level not found for position {}", pos);
                                Error::new(FileError::Inval, &msg)
                            })
                            .map(|&l| *level = l)
                    })?;
                let res = T::update_partial_params(req, node, &params, &mut self.0, timeout_ms);
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            DIGITAL_B_67_SRC_NAME => {
                let mut params = self.0.clone();
                let pos = elem_value.enumerated()[0] as usize;
                Self::DIGITAL_B_67_SRCS
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| {
                        let msg =
                            format!("Digital B 7/8 output source not found for position {}", pos);
                        Error::new(FileError::Inval, &msg)
                    })
                    .map(|&s| params.digital_67_src = s)?;
                let res = T::update_partial_params(req, node, &params, &mut self.0, timeout_ms);
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            .map(|_| true),
            SPDIF_OUT_SRC_NAME => {
                let mut params = self.0.clone();
                let pos = elem_value.enumerated()[0] as usize;
                Self::MIXER_OUT_PAIRS
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| {
                        let msg = format!("S/PDIF output source not found for position {}", pos);
                        Error::new(FileError::Inval, &msg)
                    })
                    .map(|&s| params.spdif_out_src = s)?;
                let res = T::update_partial_params(req, node, &params, &mut self.0, timeout_ms);
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            HP23_SRC_NAME => {
                let mut params = self.0.clone();
                let pos = elem_value.enumerated()[0] as usize;
                Self::MIXER_OUT_PAIRS
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| {
                        let msg = format!("Headphone 3/4 source not found for postiion: {}", pos);
                        Error::new(FileError::Inval, &msg)
                    })
                    .map(|&s| params.headphone2_3_out_src = s)?;
                let res = T::update_partial_params(req, node, &params, &mut self.0, timeout_ms);
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

#[derive(Debug)]
struct MixerCtl<T>(IofwMixerParams, Vec<ElemId>, PhantomData<T>)
where
    T: IofwMixerSpecification
        + AlesisParametersOperation<IofwMixerParams>
        + AlesisMutableParametersOperation<IofwMixerParams>;

impl<T> Default for MixerCtl<T>
where
    T: IofwMixerSpecification
        + AlesisParametersOperation<IofwMixerParams>
        + AlesisMutableParametersOperation<IofwMixerParams>,
{
    fn default() -> Self {
        Self(
            T::create_mixer_params(),
            Default::default(),
            Default::default(),
        )
    }
}

const INPUT_GAIN_NAME: &str = "monitor-input-gain";
const INPUT_MUTE_NAME: &str = "monitor-input-mute";

const STREAM_GAIN_NAME: &str = "mixer-stream-gain";

const OUTPUT_VOL_NAME: &str = "monitor-output-volume";
const OUTPUT_MUTE_NAME: &str = "monitor-output-mute";

const MIX_BLEND_KNOB_NAME: &str = "mix-blend-knob";
const MAIN_LEVEL_KNOB_NAME: &str = "main-level-knob";

impl<T> MixerCtl<T>
where
    T: IofwMixerSpecification
        + AlesisParametersOperation<IofwMixerParams>
        + AlesisMutableParametersOperation<IofwMixerParams>,
{
    const GAIN_MIN: i32 = T::GAIN_MIN;
    const GAIN_MAX: i32 = T::GAIN_MAX;
    const GAIN_STEP: i32 = 1;
    const GAIN_TLV: DbInterval = DbInterval {
        min: -6000,
        max: 0,
        linear: false,
        mute_avail: false,
    };

    const VOLUME_MIN: i32 = T::VOLUME_MIN as i32;
    const VOLUME_MAX: i32 = T::VOLUME_MAX as i32;
    const VOLUME_STEP: i32 = 1;
    const VOLUME_TLV: DbInterval = DbInterval {
        min: -6000,
        max: 0,
        linear: false,
        mute_avail: false,
    };

    const KNOB_MIN: i32 = T::VOLUME_MIN as i32;
    const KNOB_MAX: i32 = T::VOLUME_MAX as i32;
    const KNOB_STEP: i32 = 1;

    const MONITOR_INPUT_COUNT: usize = (T::ANALOG_INPUT_PAIR_COUNT
        + T::DIGITAL_A_INPUT_PAIR_COUNT
        + T::DIGITAL_B_INPUT_PAIR_COUNT)
        * 2;
    const STREAM_INPUT_COUNT: usize = T::STREAM_INPUT_PAIR_COUNT * 2;
    const MIXER_OUTPUT_COUNT: usize = T::MIXER_OUTPUT_PAIR_COUNT * 2;

    fn whole_cache(&mut self, req: &FwReq, node: &FwNode, timeout_ms: u32) -> Result<(), Error> {
        let res = T::cache_whole_params(req, node, &mut self.0, timeout_ms);
        debug!(params = ?self.0, ?res);
        res
    }

    fn partial_cache(&mut self, req: &FwReq, node: &FwNode, timeout_ms: u32) -> Result<(), Error> {
        let old = self.0.blend_knob;
        T::cache_partial_params(req, node, &mut self.0, timeout_ms)?;
        let new = self.0.blend_knob;

        if old != new {
            let mut params = self.0.clone();
            params.mixer_pairs[0].monitor_pair.output_volumes[0] = new;
            params.mixer_pairs[0].monitor_pair.output_volumes[1] = new;
            let res = T::update_partial_params(req, node, &params, &mut self.0, timeout_ms);
            debug!(params = ?self.0, ?res);
            res?;
        }

        Ok(())
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, INPUT_GAIN_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            Self::MIXER_OUTPUT_COUNT,
            Self::GAIN_MIN,
            Self::GAIN_MAX,
            Self::GAIN_STEP,
            Self::MONITOR_INPUT_COUNT,
            Some(&Into::<Vec<u32>>::into(Self::GAIN_TLV)),
            true,
        )?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, INPUT_MUTE_NAME, 0);
        let _ = card_cntr.add_bool_elems(
            &elem_id,
            T::MIXER_OUTPUT_PAIR_COUNT,
            Self::MONITOR_INPUT_COUNT,
            true,
        )?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, STREAM_GAIN_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            Self::MIXER_OUTPUT_COUNT,
            Self::GAIN_MIN,
            Self::GAIN_MAX,
            Self::GAIN_STEP,
            Self::STREAM_INPUT_COUNT,
            Some(&Into::<Vec<u32>>::into(Self::GAIN_TLV)),
            true,
        )?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, OUTPUT_VOL_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                Self::VOLUME_MIN,
                Self::VOLUME_MAX,
                Self::VOLUME_STEP,
                Self::MIXER_OUTPUT_COUNT,
                Some(&Into::<Vec<u32>>::into(Self::VOLUME_TLV)),
                true,
            )
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, OUTPUT_MUTE_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, Self::MIXER_OUTPUT_COUNT, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, MIX_BLEND_KNOB_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                Self::KNOB_MIN,
                Self::KNOB_MAX,
                Self::KNOB_STEP,
                1,
                None,
                false,
            )
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, MAIN_LEVEL_KNOB_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                Self::KNOB_MIN,
                Self::KNOB_MAX,
                Self::KNOB_STEP,
                1,
                None,
                false,
            )
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        Ok(())
    }

    fn read(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            INPUT_GAIN_NAME => {
                let params = &self.0;
                let mixer = elem_id.index() as usize;
                let srcs = &params.mixer_pairs[mixer / 2];
                let vals: Vec<i32> = srcs
                    .monitor_pair
                    .analog_input_pairs
                    .iter()
                    .chain(srcs.monitor_pair.digital_a_input_pairs.iter())
                    .chain(srcs.monitor_pair.digital_b_input_pairs.iter())
                    .flat_map(|pair| {
                        if mixer % 2 == 0 {
                            pair.gain_to_left.iter()
                        } else {
                            pair.gain_to_right.iter()
                        }
                    })
                    .copied()
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            INPUT_MUTE_NAME => {
                let params = &self.0;
                let mixer_pair_index = elem_id.index() as usize;
                let srcs = &params.mixer_pairs[mixer_pair_index];
                let vals: Vec<bool> = srcs
                    .monitor_pair
                    .analog_input_pairs
                    .iter()
                    .chain(srcs.monitor_pair.digital_a_input_pairs.iter())
                    .chain(srcs.monitor_pair.digital_b_input_pairs.iter())
                    .flat_map(|pair| pair.mutes.iter())
                    .copied()
                    .collect();
                elem_value.set_bool(&vals);
                Ok(true)
            }
            STREAM_GAIN_NAME => {
                let params = &self.0;
                let mixer = elem_id.index() as usize;
                let srcs = &params.mixer_pairs[mixer / 2];
                let vals = if mixer % 2 > 0 {
                    &srcs.stream_inputs_to_right[..]
                } else {
                    &srcs.stream_inputs_to_left[..]
                };
                elem_value.set_int(&vals);
                Ok(true)
            }
            OUTPUT_MUTE_NAME => {
                let params = &self.0;
                let vals: Vec<bool> = params
                    .mixer_pairs
                    .iter()
                    .flat_map(|mixer| mixer.monitor_pair.output_mutes.iter())
                    .copied()
                    .collect();
                elem_value.set_bool(&vals);
                Ok(true)
            }
            OUTPUT_VOL_NAME => {
                let params = &self.0;
                let vals: Vec<i32> = params
                    .mixer_pairs
                    .iter()
                    .flat_map(|mixer| mixer.monitor_pair.output_volumes.iter())
                    .map(|&vol| vol as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            MAIN_LEVEL_KNOB_NAME => {
                let params = &self.0;
                elem_value.set_int(&[params.master_knob as i32]);
                Ok(true)
            }
            MIX_BLEND_KNOB_NAME => {
                let params = &self.0;
                elem_value.set_int(&[params.blend_knob as i32]);
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
            INPUT_GAIN_NAME => {
                let mut params = self.0.clone();
                let mixer = elem_id.index() as usize;
                let srcs = &mut params.mixer_pairs[mixer / 2];
                srcs.monitor_pair
                    .analog_input_pairs
                    .iter_mut()
                    .chain(srcs.monitor_pair.digital_a_input_pairs.iter_mut())
                    .chain(srcs.monitor_pair.digital_b_input_pairs.iter_mut())
                    .flat_map(|pair| {
                        if mixer % 2 == 0 {
                            pair.gain_to_left.iter_mut()
                        } else {
                            pair.gain_to_right.iter_mut()
                        }
                    })
                    .zip(elem_value.int())
                    .for_each(|(gain, &val)| *gain = val);
                let res = T::update_partial_params(req, node, &params, &mut self.0, timeout_ms);
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            INPUT_MUTE_NAME => {
                let mut params = self.0.clone();
                let mixer = elem_id.index() as usize;
                let srcs = &mut params.mixer_pairs[mixer / 2];
                srcs.monitor_pair
                    .analog_input_pairs
                    .iter_mut()
                    .chain(srcs.monitor_pair.digital_a_input_pairs.iter_mut())
                    .chain(srcs.monitor_pair.digital_b_input_pairs.iter_mut())
                    .flat_map(|pair| pair.mutes.iter_mut())
                    .zip(elem_value.boolean())
                    .for_each(|(mute, val)| *mute = val);
                T::update_partial_params(req, node, &params, &mut self.0, timeout_ms).map(|_| true)
            }
            STREAM_GAIN_NAME => {
                let mut params = self.0.clone();
                let mixer = elem_id.index() as usize;
                let srcs = &mut params.mixer_pairs[mixer / 2];
                let gains = if mixer % 2 > 0 {
                    &mut srcs.stream_inputs_to_right[..]
                } else {
                    &mut srcs.stream_inputs_to_left[..]
                };
                gains
                    .iter_mut()
                    .zip(elem_value.int())
                    .for_each(|(gain, &val)| *gain = val);
                let res = T::update_partial_params(req, node, &params, &mut self.0, timeout_ms);
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            OUTPUT_VOL_NAME => {
                let mut params = self.0.clone();
                params
                    .mixer_pairs
                    .iter_mut()
                    .flat_map(|mixer_pair| mixer_pair.monitor_pair.output_volumes.iter_mut())
                    .zip(elem_value.int())
                    .for_each(|(vol, &val)| *vol = val as u32);
                let res = T::update_partial_params(req, node, &params, &mut self.0, timeout_ms);
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            OUTPUT_MUTE_NAME => {
                let mut params = self.0.clone();
                params
                    .mixer_pairs
                    .iter_mut()
                    .flat_map(|srcs| srcs.monitor_pair.output_mutes.iter_mut())
                    .zip(elem_value.boolean())
                    .for_each(|(mute, val)| *mute = val);
                let res = T::update_partial_params(req, node, &params, &mut self.0, timeout_ms);
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

pub fn detect_io26fw_model(node: &FwNode) -> Result<bool, Error> {
    let req = FwReq::default();
    let mut sections = GeneralSections::default();
    Io14fwProtocol::read_general_sections(&req, node, &mut sections, TIMEOUT_MS)?;

    let mut global_params = GlobalParameters::default();
    Io14fwProtocol::whole_cache(&req, node, &sections.global, &mut global_params, TIMEOUT_MS)?;
    let config = &global_params.clock_config;

    match config.rate {
        ClockRate::R32000 | ClockRate::R44100 | ClockRate::R48000 | ClockRate::AnyLow => {
            let mut params = TxStreamFormatParameters::default();
            Io14fwProtocol::whole_cache(
                &req,
                node,
                &sections.tx_stream_format,
                &mut params,
                TIMEOUT_MS,
            )?;
            let entries = &params.0;
            if entries.len() == 2 && entries[0].pcm == 10 && entries[1].pcm == 16 {
                Ok(true)
            } else if entries.len() == 2 && entries[0].pcm == 6 && entries[1].pcm == 8 {
                Ok(false)
            } else {
                Err(Error::new(
                    FileError::Nxio,
                    "Unexpected combination of stream format.",
                ))
            }
        }
        ClockRate::R88200 | ClockRate::R96000 | ClockRate::AnyMid => {
            let mut params = TxStreamFormatParameters::default();
            Io14fwProtocol::whole_cache(
                &req,
                node,
                &sections.tx_stream_format,
                &mut params,
                TIMEOUT_MS,
            )?;
            let entries = &params.0;
            if entries.len() == 2 && entries[0].pcm == 10 && entries[1].pcm == 4 {
                Ok(true)
            } else if entries.len() == 2 && entries[0].pcm == 6 && entries[1].pcm == 4 {
                Ok(false)
            } else {
                Err(Error::new(
                    FileError::Nxio,
                    "Unexpected combination of stream format.",
                ))
            }
        }
        ClockRate::R176400 | ClockRate::R192000 | ClockRate::AnyHigh => {
            let nickname = &global_params.nickname;
            match nickname.as_str() {
                "iO 26" => Ok(true),
                "iO 14" => Ok(false),
                _ => {
                    let msg = "Fail to detect type of iO model due to changed nickname";
                    Err(Error::new(FileError::Nxio, &msg))
                }
            }
        }
        _ => Err(Error::new(
            FileError::Nxio,
            "Unexpected value of rate of sampling clock.",
        )),
    }
}
