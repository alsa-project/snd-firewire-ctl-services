// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use {
    super::*,
    protocols::alesis::{mixer::*, *},
    std::marker::PhantomData,
};

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
        + IofwMixerOperation
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

impl<T> IofwModel<T>
where
    T: IofwMeterSpecification
        + AlesisParametersOperation<IofwMeterParams>
        + IofwOutputSpecification
        + AlesisParametersOperation<IofwOutputParams>
        + AlesisMutableParametersOperation<IofwOutputParams>
        + IofwMixerOperation
        + TcatNotifiedSectionOperation<GlobalParameters>
        + TcatFluctuatedSectionOperation<GlobalParameters>
        + TcatMutableSectionOperation<GlobalParameters>
        + TcatNotifiedSectionOperation<TxStreamFormatParameters>
        + TcatNotifiedSectionOperation<RxStreamFormatParameters>
        + TcatSectionOperation<ExtendedSyncParameters>,
{
    pub fn cache(&mut self, unit: &mut (SndDice, FwNode)) -> Result<(), Error> {
        T::read_general_sections(&self.req, &unit.1, &mut self.sections, TIMEOUT_MS)?;

        self.common_ctl
            .whole_cache(&self.req, &unit.1, &mut self.sections, TIMEOUT_MS)?;

        self.meter_ctl.cache(&self.req, &unit.1, TIMEOUT_MS)?;
        self.output_ctl.cache(&self.req, &unit.1, TIMEOUT_MS)?;

        Ok(())
    }
}

impl<T> CtlModel<(SndDice, FwNode)> for IofwModel<T>
where
    T: IofwMeterSpecification
        + AlesisParametersOperation<IofwMeterParams>
        + IofwOutputSpecification
        + AlesisParametersOperation<IofwOutputParams>
        + AlesisMutableParametersOperation<IofwOutputParams>
        + IofwMixerOperation
        + TcatNotifiedSectionOperation<GlobalParameters>
        + TcatFluctuatedSectionOperation<GlobalParameters>
        + TcatMutableSectionOperation<GlobalParameters>
        + TcatNotifiedSectionOperation<TxStreamFormatParameters>
        + TcatNotifiedSectionOperation<RxStreamFormatParameters>
        + TcatSectionOperation<ExtendedSyncParameters>,
{
    fn load(
        &mut self,
        unit: &mut (SndDice, FwNode),
        card_cntr: &mut CardCntr,
    ) -> Result<(), Error> {
        self.common_ctl.load(card_cntr, &self.sections).map(
            |(measured_elem_id_list, notified_elem_id_list)| {
                self.common_ctl.0 = measured_elem_id_list;
                self.common_ctl.1 = notified_elem_id_list;
            },
        )?;

        self.meter_ctl.load(card_cntr)?;
        self.mixer_ctl
            .load(card_cntr, unit, &mut self.req, TIMEOUT_MS)
            .map(|mut elem_id_list| self.mixer_ctl.1.append(&mut elem_id_list))?;
        self.output_ctl.load(card_cntr)?;

        Ok(())
    }

    fn read(
        &mut self,
        _: &mut (SndDice, FwNode),
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.common_ctl.read(&self.sections, elem_id, elem_value)? {
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
            .mixer_ctl
            .write(unit, &mut self.req, elem_id, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .output_ctl
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
        + IofwMixerOperation
        + TcatNotifiedSectionOperation<GlobalParameters>
        + TcatFluctuatedSectionOperation<GlobalParameters>
        + TcatMutableSectionOperation<GlobalParameters>
        + TcatNotifiedSectionOperation<TxStreamFormatParameters>
        + TcatNotifiedSectionOperation<RxStreamFormatParameters>
        + TcatSectionOperation<ExtendedSyncParameters>,
{
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.common_ctl.1);
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
        self.common_ctl.read(&self.sections, elem_id, elem_value)
    }
}

impl<T> MeasureModel<(SndDice, FwNode)> for IofwModel<T>
where
    T: IofwMeterSpecification
        + AlesisParametersOperation<IofwMeterParams>
        + IofwOutputSpecification
        + AlesisParametersOperation<IofwOutputParams>
        + AlesisMutableParametersOperation<IofwOutputParams>
        + IofwMixerOperation
        + TcatNotifiedSectionOperation<GlobalParameters>
        + TcatFluctuatedSectionOperation<GlobalParameters>
        + TcatMutableSectionOperation<GlobalParameters>
        + TcatNotifiedSectionOperation<TxStreamFormatParameters>
        + TcatNotifiedSectionOperation<RxStreamFormatParameters>
        + TcatSectionOperation<ExtendedSyncParameters>,
{
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.common_ctl.0);
        elem_id_list.extend_from_slice(&self.meter_ctl.1);
    }

    fn measure_states(&mut self, unit: &mut (SndDice, FwNode)) -> Result<(), Error> {
        self.common_ctl
            .measure(&self.req, &unit.1, &mut self.sections, TIMEOUT_MS)?;
        self.meter_ctl.cache(&self.req, &unit.1, TIMEOUT_MS)?;
        self.mixer_ctl
            .measure_states(unit, &mut self.req, TIMEOUT_MS)?;

        Ok(())
    }

    fn measure_elem(
        &mut self,
        _: &(SndDice, FwNode),
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.common_ctl.read(&self.sections, elem_id, elem_value)? {
            Ok(true)
        } else if self.meter_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_ctl.read_measured_elem(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[derive(Default, Debug)]
pub struct CommonCtl<T>(Vec<ElemId>, Vec<ElemId>, PhantomData<T>)
where
    T: IofwMeterSpecification
        + AlesisParametersOperation<IofwMeterParams>
        + IofwOutputSpecification
        + AlesisParametersOperation<IofwOutputParams>
        + AlesisMutableParametersOperation<IofwOutputParams>
        + IofwMixerOperation
        + TcatNotifiedSectionOperation<GlobalParameters>
        + TcatFluctuatedSectionOperation<GlobalParameters>
        + TcatMutableSectionOperation<GlobalParameters>
        + TcatNotifiedSectionOperation<TxStreamFormatParameters>
        + TcatNotifiedSectionOperation<RxStreamFormatParameters>
        + TcatSectionOperation<ExtendedSyncParameters>;

impl<T> CommonCtlOperation<T> for CommonCtl<T> where
    T: IofwMeterSpecification
        + AlesisParametersOperation<IofwMeterParams>
        + IofwOutputSpecification
        + AlesisParametersOperation<IofwOutputParams>
        + AlesisMutableParametersOperation<IofwOutputParams>
        + IofwMixerOperation
        + TcatNotifiedSectionOperation<GlobalParameters>
        + TcatFluctuatedSectionOperation<GlobalParameters>
        + TcatMutableSectionOperation<GlobalParameters>
        + TcatNotifiedSectionOperation<TxStreamFormatParameters>
        + TcatNotifiedSectionOperation<RxStreamFormatParameters>
        + TcatSectionOperation<ExtendedSyncParameters>
{
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
        T::cache_whole_params(req, node, &mut self.0, timeout_ms)
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
        T::cache_whole_params(req, node, &mut self.0, timeout_ms)
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
                T::update_partial_params(req, node, &params, &mut self.0, timeout_ms).map(|_| true)
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
                T::update_partial_params(req, node, &params, &mut self.0, timeout_ms).map(|_| true)
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
                T::update_partial_params(req, node, &params, &mut self.0, timeout_ms).map(|_| true)
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
                T::update_partial_params(req, node, &params, &mut self.0, timeout_ms).map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

#[derive(Default, Debug)]
struct MixerCtl<T>(IofwMixerState, Vec<ElemId>, PhantomData<T>)
where
    T: IofwMixerOperation;

impl<T> MixerCtlOperation<T> for MixerCtl<T>
where
    T: IofwMixerOperation,
{
    fn state(&self) -> &IofwMixerState {
        &self.0
    }

    fn state_mut(&mut self) -> &mut IofwMixerState {
        &mut self.0
    }
}

const INPUT_GAIN_NAME: &str = "monitor-input-gain";
const INPUT_MUTE_NAME: &str = "monitor-input-mute";

const STREAM_GAIN_NAME: &str = "mixer-stream-gain";

const OUTPUT_VOL_NAME: &str = "monitor-output-volume";
const OUTPUT_MUTE_NAME: &str = "monitor-output-mute";

const MIX_BLEND_KNOB_NAME: &str = "mix-blend-knob";
const MAIN_LEVEL_KNOB_NAME: &str = "main-level-knob";

pub trait MixerCtlOperation<T: IofwMixerOperation> {
    fn state(&self) -> &IofwMixerState;
    fn state_mut(&mut self) -> &mut IofwMixerState;

    const LEVEL_MIN: i32 = 0;
    const LEVEL_MAX: i32 = 0x007fff00;
    const LEVEL_STEP: i32 = 0x100;
    const LEVEL_TLV: DbInterval = DbInterval {
        min: -9000,
        max: 0,
        linear: false,
        mute_avail: false,
    };

    const KNOB_MIN: i32 = 0;
    const KNOB_MAX: i32 = 0x100;
    const KNOB_STEP: i32 = 1;

    fn load(
        &mut self,
        card_cntr: &mut CardCntr,
        unit: &mut (SndDice, FwNode),
        req: &mut FwReq,
        timeout_ms: u32,
    ) -> Result<Vec<ElemId>, Error> {
        let mut state = T::create_mixer_state();
        T::read_mixer_src_gains(req, &mut unit.1, &mut state, timeout_ms)?;
        T::read_mixer_src_mutes(req, &mut unit.1, &mut state, timeout_ms)?;
        T::read_mixer_out_vols(req, &mut unit.1, &mut state, timeout_ms)?;
        T::read_mixer_out_mutes(req, &mut unit.1, &mut state, timeout_ms)?;
        *self.state_mut() = state;

        let mut measured_elem_id_list = Vec::new();

        let count = T::ANALOG_INPUT_COUNT + T::DIGITAL_A_INPUT_COUNT + T::DIGITAL_B_INPUT_COUNT;
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, INPUT_GAIN_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            T::MIXER_COUNT,
            Self::LEVEL_MIN,
            Self::LEVEL_MAX,
            Self::LEVEL_STEP,
            count,
            Some(&Into::<Vec<u32>>::into(Self::LEVEL_TLV)),
            true,
        )?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, INPUT_MUTE_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, T::MIXER_PAIR_COUNT, count, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, STREAM_GAIN_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            T::MIXER_COUNT,
            Self::LEVEL_MIN,
            Self::LEVEL_MAX,
            Self::LEVEL_STEP,
            T::STREAM_INPUT_COUNT,
            Some(&Into::<Vec<u32>>::into(Self::LEVEL_TLV)),
            true,
        )?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, OUTPUT_VOL_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                Self::LEVEL_MIN,
                Self::LEVEL_MAX,
                Self::LEVEL_STEP,
                count,
                Some(&Into::<Vec<u32>>::into(Self::LEVEL_TLV)),
                true,
            )
            .map(|mut elem_id_list| measured_elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, OUTPUT_MUTE_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, T::MIXER_COUNT, true)?;

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
            .map(|mut elem_id_list| measured_elem_id_list.append(&mut elem_id_list))?;

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
            .map(|mut elem_id_list| measured_elem_id_list.append(&mut elem_id_list))?;

        Ok(measured_elem_id_list)
    }

    fn read(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            INPUT_GAIN_NAME => {
                let mixer = elem_id.index() as usize;
                let gains = &self.state().gains[mixer];
                let mut vals = Vec::new();
                vals.extend_from_slice(&gains.analog_inputs);
                vals.extend_from_slice(&gains.digital_a_inputs);
                vals.extend_from_slice(&gains.digital_b_inputs);
                elem_value.set_int(&vals);
                Ok(true)
            }
            INPUT_MUTE_NAME => {
                let mixer = elem_id.index() as usize;
                let mutes = &self.state().mutes[mixer];
                let mut vals = Vec::new();
                vals.extend_from_slice(&mutes.analog_inputs);
                vals.extend_from_slice(&mutes.digital_a_inputs);
                vals.extend_from_slice(&mutes.digital_b_inputs);
                elem_value.set_bool(&vals);
                Ok(true)
            }
            STREAM_GAIN_NAME => {
                let mixer = elem_id.index() as usize;
                let gains = &self.state().gains[mixer];
                let mut vals = Vec::new();
                vals.extend_from_slice(&gains.stream_inputs);
                elem_value.set_int(&vals);
                Ok(true)
            }
            OUTPUT_MUTE_NAME => {
                elem_value.set_bool(&self.state().out_mutes);
                Ok(true)
            }
            _ => self.read_measured_elem(elem_id, elem_value),
        }
    }

    fn write(
        &mut self,
        unit: &mut (SndDice, FwNode),
        req: &mut FwReq,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            INPUT_GAIN_NAME => {
                let mixer = elem_id.index() as usize;
                let mut gains = self.state().gains[mixer].clone();

                let analog_input_count = gains.analog_inputs.len();
                let digital_a_input_count = gains.digital_a_inputs.len();
                let digital_b_input_count = gains.digital_b_inputs.len();
                let vals = &elem_value.int()
                    [..(analog_input_count + digital_a_input_count + digital_b_input_count)];

                let analog_inputs = &vals[..analog_input_count];
                let digital_a_inputs =
                    &vals[analog_input_count..(analog_input_count + digital_a_input_count)];
                let digital_b_inputs = &vals[(analog_input_count + digital_a_input_count)..];

                gains.analog_inputs.copy_from_slice(&analog_inputs);
                gains.digital_a_inputs.copy_from_slice(&digital_a_inputs);
                gains.digital_b_inputs.copy_from_slice(&digital_b_inputs);

                T::write_mixer_src_gains(
                    req,
                    &mut unit.1,
                    mixer,
                    &gains,
                    self.state_mut(),
                    timeout_ms,
                )
                .map(|_| true)
            }
            INPUT_MUTE_NAME => {
                let mixer = elem_id.index() as usize;
                let mut mutes = self.state().mutes[mixer].clone();

                let analog_input_count = mutes.analog_inputs.len();
                let digital_a_input_count = mutes.digital_a_inputs.len();
                let digital_b_input_count = mutes.digital_b_inputs.len();
                let vals = &elem_value.boolean()
                    [..(analog_input_count + digital_a_input_count + digital_b_input_count)];

                let analog_inputs = &vals[..analog_input_count];
                let digital_a_inputs =
                    &vals[analog_input_count..(analog_input_count + digital_a_input_count)];
                let digital_b_inputs = &vals[(analog_input_count + digital_a_input_count)..];

                mutes.analog_inputs.copy_from_slice(&analog_inputs);
                mutes.digital_a_inputs.copy_from_slice(&digital_a_inputs);
                mutes.digital_b_inputs.copy_from_slice(&digital_b_inputs);

                T::write_mixer_src_mutes(
                    req,
                    &mut unit.1,
                    mixer,
                    &mutes,
                    self.state_mut(),
                    timeout_ms,
                )
                .map(|_| true)
            }
            STREAM_GAIN_NAME => {
                let mixer = elem_id.index() as usize;
                let mut gains = self.state().gains[mixer].clone();

                gains
                    .stream_inputs
                    .iter_mut()
                    .zip(elem_value.int())
                    .for_each(|(d, s)| *d = *s);

                T::write_mixer_src_gains(
                    req,
                    &mut unit.1,
                    mixer,
                    &gains,
                    self.state_mut(),
                    timeout_ms,
                )
                .map(|_| true)
            }
            OUTPUT_VOL_NAME => {
                let vals = &elem_value.int()[..self.state().out_vols.len()];
                T::write_mixer_out_vols(req, &mut unit.1, &vals, self.state_mut(), timeout_ms)
                    .map(|_| true)
            }
            OUTPUT_MUTE_NAME => {
                let vals = &elem_value.boolean()[..self.state().out_mutes.len()];
                T::write_mixer_out_mutes(req, &mut unit.1, &vals, self.state_mut(), timeout_ms)
                    .map(|_| true)
            }
            _ => Ok(false),
        }
    }

    fn measure_states(
        &mut self,
        unit: &mut (SndDice, FwNode),
        req: &mut FwReq,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let old = self.state().knobs.mix_blend as i32;
        T::read_knob_state(req, &mut unit.1, self.state_mut(), timeout_ms)?;

        let new = self.state().knobs.mix_blend as i32;
        if new != old {
            // NOTE: The calculation is done within 32 bit storage without overflow.
            let val = Self::LEVEL_MAX * new / Self::KNOB_MAX;
            let mut new = self.state().out_vols.clone();
            new[0] = val;
            new[1] = val;
            T::write_mixer_out_vols(req, &mut unit.1, &new, self.state_mut(), timeout_ms)?;
        }

        Ok(())
    }

    fn read_measured_elem(&self, elem_id: &ElemId, elem_value: &ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            OUTPUT_VOL_NAME => {
                elem_value.set_int(&self.state().out_vols);
                Ok(true)
            }
            MIX_BLEND_KNOB_NAME => {
                elem_value.set_int(&[self.state().knobs.mix_blend as i32]);
                Ok(true)
            }
            MAIN_LEVEL_KNOB_NAME => {
                elem_value.set_int(&[self.state().knobs.main_level as i32]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

pub fn detect_io26fw_model(node: &FwNode) -> Result<bool, Error> {
    let req = FwReq::default();
    let mut sections = GeneralSections::default();
    Io14fwProtocol::read_general_sections(&req, node, &mut sections, TIMEOUT_MS)?;

    Io14fwProtocol::whole_cache(&req, node, &mut sections.global, TIMEOUT_MS)?;
    let config = &sections.global.params.clock_config;

    match config.rate {
        ClockRate::R32000 | ClockRate::R44100 | ClockRate::R48000 | ClockRate::AnyLow => {
            Io14fwProtocol::whole_cache(&req, node, &mut sections.tx_stream_format, TIMEOUT_MS)?;
            let entries = &sections.tx_stream_format.params.0;
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
            Io14fwProtocol::whole_cache(&req, node, &mut sections.tx_stream_format, TIMEOUT_MS)?;
            let entries = &sections.tx_stream_format.params.0;
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
            let nickname = &sections.global.params.nickname;
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
