// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use {
    super::*,
    protocols::{maudio::special::*, *},
    std::marker::PhantomData,
};

pub type Fw1814Model = SpecialModel<Fw1814ClkProtocol>;
pub type ProjectMixModel = SpecialModel<ProjectMixClkProtocol>;

#[derive(Default, Debug)]
pub struct SpecialModel<T: MediaClockFrequencyOperation> {
    avc: BebobAvc,
    req: FwReq,
    clk_ctl: ClkCtl<T>,
    meter_ctl: MeterCtl,
    cache: MaudioSpecialStateCache,
    input_ctl: InputCtl,
    output_ctl: OutputCtl,
    aux_ctl: AuxCtl,
    mixer_ctl: MixerCtl,
}

const FCP_TIMEOUT_MS: u32 = 200;
const TIMEOUT_MS: u32 = 100;

#[derive(Default, Debug)]
struct ClkCtl<T: MediaClockFrequencyOperation>(Vec<ElemId>, MediaClockParameters, PhantomData<T>);

impl<T: MediaClockFrequencyOperation> MediaClkFreqCtlOperation<T> for ClkCtl<T> {
    fn state(&self) -> &MediaClockParameters {
        &self.1
    }

    fn state_mut(&mut self) -> &mut MediaClockParameters {
        &mut self.1
    }
}

#[derive(Default, Debug)]
struct MeterCtl(MaudioSpecialMeterState, Vec<ElemId>);

impl<T: MediaClockFrequencyOperation> SpecialModel<T> {
    pub fn cache(&mut self, unit: &mut (SndUnit, FwNode)) -> Result<(), Error> {
        self.avc.bind(&unit.1)?;

        self.meter_ctl.cache(&self.req, &unit.1, TIMEOUT_MS)?;
        self.input_ctl
            .cache(&self.req, &unit.1, &mut self.cache, TIMEOUT_MS)?;
        self.output_ctl
            .cache(&self.req, &unit.1, &mut self.cache, TIMEOUT_MS)?;
        self.aux_ctl
            .cache(&self.req, &unit.1, &mut self.cache, TIMEOUT_MS)?;
        self.mixer_ctl
            .cache(&self.req, &unit.1, &mut self.cache, TIMEOUT_MS)?;

        Ok(())
    }
}

impl<T: MediaClockFrequencyOperation> CtlModel<(SndUnit, FwNode)> for SpecialModel<T> {
    fn load(&mut self, _: &mut (SndUnit, FwNode), card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.clk_ctl
            .load_freq(card_cntr)
            .map(|mut elem_id_list| self.clk_ctl.0.append(&mut elem_id_list))?;

        self.meter_ctl.load_state(card_cntr)?;

        self.input_ctl.load_params(card_cntr)?;
        self.output_ctl.load_params(card_cntr)?;
        self.aux_ctl.load_params(card_cntr)?;
        self.mixer_ctl.load_params(card_cntr)?;

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
        } else if self.input_ctl.read_params(elem_id, elem_value)? {
            Ok(true)
        } else if self.output_ctl.read_params(elem_id, elem_value)? {
            Ok(true)
        } else if self.aux_ctl.read_params(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_ctl.read_params(elem_id, elem_value)? {
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
        } else if self.input_ctl.write_params(
            &mut self.cache,
            unit,
            &self.req,
            elem_id,
            new,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self.output_ctl.write_params(
            &mut self.cache,
            unit,
            &self.req,
            elem_id,
            new,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self.aux_ctl.write_params(
            &mut self.cache,
            unit,
            &self.req,
            elem_id,
            new,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self.mixer_ctl.write_params(
            &mut self.cache,
            unit,
            &self.req,
            elem_id,
            new,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl<T: MediaClockFrequencyOperation> MeasureModel<(SndUnit, FwNode)> for SpecialModel<T> {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.meter_ctl.1);
        elem_id_list.extend_from_slice(&self.output_ctl.1);
    }

    fn measure_states(&mut self, unit: &mut (SndUnit, FwNode)) -> Result<(), Error> {
        let switch = self.meter_ctl.0.switch;
        let prev_rotaries = self.meter_ctl.0.rotaries[..2].to_vec();

        self.meter_ctl.cache(&self.req, &unit.1, TIMEOUT_MS)?;

        if switch != self.meter_ctl.0.switch {
            let mut op = MaudioSpecialLedSwitch::new(self.meter_ctl.0.switch);
            self.avc.control(&AvcAddr::Unit, &mut op, FCP_TIMEOUT_MS)?;
        }

        // Compute in 32 bit storage.
        let val_min = MaudioSpecialOutputProtocol::VOLUME_MIN as i32;
        let val_max = MaudioSpecialOutputProtocol::VOLUME_MAX as i32;
        let range_min = MaudioSpecialMeterProtocol::ROTARY_MIN as i32;
        let range_max = MaudioSpecialMeterProtocol::ROTARY_MAX as i32;
        let delta_list: Vec<i32> = self.meter_ctl.0.rotaries[..2]
            .iter()
            .zip(prev_rotaries)
            .map(|(&curr, prev)| {
                ((curr as i32) - (prev as i32)) * (val_max - val_min) / (range_max - range_min)
            })
            .collect();

        let mut params = self.output_ctl.0.clone();
        params
            .headphone_volumes
            .iter_mut()
            .enumerate()
            .filter(|(i, _)| delta_list[i / 2] != 0)
            .for_each(|(i, vol)| {
                let delta = delta_list[i / 2];
                let mut val = *vol as i32;

                if delta < 0 {
                    if val > val_min - delta {
                        val += delta;
                    } else {
                        val = val_min;
                    }
                } else {
                    if val < val_max - delta {
                        val += delta;
                    } else {
                        val = val_max;
                    }
                }

                *vol = val as i16;
            });

        if params.headphone_volumes != self.output_ctl.0.headphone_volumes {
            MaudioSpecialOutputProtocol::partial_update(
                &self.req,
                &unit.1,
                &params,
                &mut self.cache,
                &mut self.output_ctl.0,
                TIMEOUT_MS,
            )?;
        }

        Ok(())
    }

    fn measure_elem(
        &mut self,
        _: &(SndUnit, FwNode),
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.meter_ctl.read_state(elem_id, elem_value)? {
            Ok(true)
        } else if self.output_ctl.read_params(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl<T: MediaClockFrequencyOperation> NotifyModel<(SndUnit, FwNode), bool> for SpecialModel<T> {
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

    fn read_notified_elem(
        &mut self,
        _: &(SndUnit, FwNode),
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        self.clk_ctl.read_freq(elem_id, elem_value)
    }
}

const ANALOG_INPUT_METER_NAME: &str = "meter:analog-input";
const SPDIF_INPUT_METER_NAME: &str = "meter:spdif-input";
const ADAT_INPUT_METER_NAME: &str = "meter:adat-input";
const ANALOG_OUTPUT_METER_NAME: &str = "meter:analog-output";
const SPDIF_OUTPUT_METER_NAME: &str = "meter:spdif-output";
const ADAT_OUTPUT_METER_NAME: &str = "meter:adat-output";
const HP_METER_NAME: &str = "meter:headhpone";
const AUX_OUT_METER_NAME: &str = "meter:aux-output";
const ROTARY_NAME: &str = "rotary";
const SWITCH_NAME: &str = "switch";
const SYNC_STATUS_NAME: &str = "sync-status";

const ANALOG_INPUT_LABELS: [&str; 8] = [
    "analog-input-1",
    "analog-input-2",
    "analog-input-3",
    "analog-input-4",
    "analog-input-5",
    "analog-input-6",
    "analog-input-7",
    "analog-input-8",
];

const SPDIF_INPUT_LABELS: [&str; 2] = ["spdif-input-1", "spdif-input-2"];

const ADAT_INPUT_LABELS: [&str; 8] = [
    "adat-input-1",
    "adat-input-2",
    "adat-input-3",
    "adat-input-4",
    "adat-input-5",
    "adat-input-6",
    "adat-input-7",
    "adat-input-8",
];

const ANALOG_OUTPUT_LABELS: [&str; 4] = [
    "analog-output-1",
    "analog-output-2",
    "analog-output-3",
    "analog-output-4",
];

const SPDIF_OUTPUT_LABELS: [&str; 2] = ["spdif-output-1", "spdif-input-2"];

const ADAT_OUTPUT_LABELS: [&str; 8] = [
    "adat-output-1",
    "adat-output-2",
    "adat-output-3",
    "adat-output-4",
    "adat-output-5",
    "adat-output-6",
    "adat-output-7",
    "adat-output-8",
];

const HEADPHONE_LABELS: [&'static str; 4] =
    ["headphone-1", "headphone-2", "headphone-3", "headphone-4"];

const AUX_OUTPUT_LABELS: [&'static str; 2] = ["aux-output-1", "aux-output-2"];

impl MeterCtl {
    const METER_TLV: DbInterval = DbInterval {
        min: -14400,
        max: 0,
        linear: false,
        mute_avail: false,
    };

    fn add_level_int_elems(
        card_cntr: &mut CardCntr,
        name: &str,
        labels: &[&str],
    ) -> Result<Vec<ElemId>, Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
        card_cntr.add_int_elems(
            &elem_id,
            1,
            MaudioSpecialMeterProtocol::LEVEL_MIN as i32,
            MaudioSpecialMeterProtocol::LEVEL_MAX as i32,
            MaudioSpecialMeterProtocol::LEVEL_STEP as i32,
            labels.len(),
            Some(&Into::<Vec<u32>>::into(Self::METER_TLV)),
            false,
        )
    }

    fn load_state(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        [
            (ANALOG_INPUT_METER_NAME, &ANALOG_INPUT_LABELS[..]),
            (SPDIF_INPUT_METER_NAME, &SPDIF_INPUT_LABELS[..]),
            (ADAT_INPUT_METER_NAME, &ADAT_INPUT_LABELS[..]),
            (ANALOG_OUTPUT_METER_NAME, &ANALOG_OUTPUT_LABELS[..]),
            (SPDIF_OUTPUT_METER_NAME, &SPDIF_OUTPUT_LABELS[..]),
            (ADAT_OUTPUT_METER_NAME, &ADAT_OUTPUT_LABELS[..]),
            (HP_METER_NAME, &HEADPHONE_LABELS[..]),
            (AUX_OUT_METER_NAME, &AUX_OUTPUT_LABELS[..]),
        ]
        .iter()
        .try_for_each(|(name, labels)| {
            Self::add_level_int_elems(card_cntr, name, labels)
                .map(|mut elem_id_list| self.1.append(&mut elem_id_list))
        })?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, ROTARY_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                MaudioSpecialMeterProtocol::ROTARY_MIN as i32,
                MaudioSpecialMeterProtocol::ROTARY_MAX as i32,
                MaudioSpecialMeterProtocol::ROTARY_STEP as i32,
                3,
                None,
                false,
            )
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, SWITCH_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, 1, 1, false)
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, SYNC_STATUS_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, 1, 1, false)
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        Ok(())
    }

    fn cache(&mut self, req: &FwReq, node: &FwNode, timeout_ms: u32) -> Result<(), Error> {
        MaudioSpecialMeterProtocol::cache(req, node, &mut self.0, timeout_ms)
    }

    fn read_state(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            ANALOG_INPUT_METER_NAME => {
                let vals: Vec<i32> = self.0.analog_inputs.iter().map(|&val| val as i32).collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            SPDIF_INPUT_METER_NAME => {
                let vals: Vec<i32> = self.0.spdif_inputs.iter().map(|&val| val as i32).collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            ADAT_INPUT_METER_NAME => {
                let vals: Vec<i32> = self.0.adat_inputs.iter().map(|&val| val as i32).collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            ANALOG_OUTPUT_METER_NAME => {
                let vals: Vec<i32> = self
                    .0
                    .analog_outputs
                    .iter()
                    .map(|&val| val as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            SPDIF_OUTPUT_METER_NAME => {
                let vals: Vec<i32> = self.0.spdif_outputs.iter().map(|&val| val as i32).collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            ADAT_OUTPUT_METER_NAME => {
                let vals: Vec<i32> = self.0.adat_outputs.iter().map(|&val| val as i32).collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            HP_METER_NAME => {
                let vals: Vec<i32> = self.0.headphone.iter().map(|&val| val as i32).collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            AUX_OUT_METER_NAME => {
                let vals: Vec<i32> = self.0.aux_outputs.iter().map(|&val| val as i32).collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            ROTARY_NAME => {
                let vals: Vec<i32> = self.0.rotaries.iter().map(|&val| val as i32).collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            SWITCH_NAME => {
                elem_value.set_bool(&[self.0.switch]);
                Ok(true)
            }
            SYNC_STATUS_NAME => {
                elem_value.set_bool(&[self.0.sync_status]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

#[derive(Default, Debug)]
struct InputCtl(MaudioSpecialInputParameters);

const STREAM_INPUT_GAIN_NAME: &str = "stream-input-gain";
const ANALOG_INPUT_GAIN_NAME: &str = "analog-input-gain";
const SPDIF_INPUT_GAIN_NAME: &str = "spdif-input-gain";
const ADAT_INPUT_GAIN_NAME: &str = "adat-input-gain";
const ANALOG_INPUT_BALANCE_NAME: &str = "analog-input-balance";
const SPDIF_INPUT_BALANCE_NAME: &str = "spdif-input-balance";
const ADAT_INPUT_BALANCE_NAME: &str = "adat-input-balance";

const STREAM_INPUT_LABELS: [&str; 4] = [
    "stream-input-1",
    "stream-input-2",
    "stream-input-3",
    "stream-input-4",
];

impl InputCtl {
    const GAIN_TLV: DbInterval = DbInterval {
        min: -12800,
        max: 0,
        linear: false,
        mute_avail: false,
    };

    fn add_input_gain_elem(
        card_cntr: &mut CardCntr,
        name: &str,
        labels: &[&str],
    ) -> Result<Vec<ElemId>, Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
        card_cntr.add_int_elems(
            &elem_id,
            1,
            MaudioSpecialInputProtocol::GAIN_MIN as i32,
            MaudioSpecialInputProtocol::GAIN_MAX as i32,
            MaudioSpecialInputProtocol::GAIN_STEP as i32,
            labels.len(),
            Some(&Into::<Vec<u32>>::into(Self::GAIN_TLV)),
            true,
        )
    }

    fn add_input_balance_elem(
        card_cntr: &mut CardCntr,
        name: &str,
        labels: &[&str],
    ) -> Result<Vec<ElemId>, Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
        card_cntr.add_int_elems(
            &elem_id,
            1,
            MaudioSpecialInputProtocol::BALANCE_MIN as i32,
            MaudioSpecialInputProtocol::BALANCE_MAX as i32,
            MaudioSpecialInputProtocol::BALANCE_STEP as i32,
            labels.len(),
            None,
            true,
        )
    }

    fn load_params(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        [
            (STREAM_INPUT_GAIN_NAME, &STREAM_INPUT_LABELS[..]),
            (ANALOG_INPUT_GAIN_NAME, &ANALOG_INPUT_LABELS[..]),
            (SPDIF_INPUT_GAIN_NAME, &SPDIF_INPUT_LABELS[..]),
            (ADAT_INPUT_GAIN_NAME, &ADAT_INPUT_LABELS[..]),
        ]
        .iter()
        .try_for_each(|(name, labels)| {
            Self::add_input_gain_elem(card_cntr, name, labels).map(|_| ())
        })?;

        [
            (ANALOG_INPUT_BALANCE_NAME, &ANALOG_INPUT_LABELS[..]),
            (SPDIF_INPUT_BALANCE_NAME, &SPDIF_INPUT_LABELS[..]),
            (ADAT_INPUT_BALANCE_NAME, &ADAT_INPUT_LABELS[..]),
        ]
        .iter()
        .try_for_each(|(name, labels)| {
            Self::add_input_balance_elem(card_cntr, name, labels).map(|_| ())
        })?;

        Ok(())
    }

    fn cache(
        &mut self,
        req: &FwReq,
        node: &FwNode,
        cache: &mut MaudioSpecialStateCache,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        MaudioSpecialInputProtocol::whole_update(req, node, &mut self.0, cache, timeout_ms)
    }

    fn read_int(elem_value: &mut ElemValue, gains: &[i16]) -> Result<bool, Error> {
        let vals: Vec<i32> = gains.iter().map(|&val| val as i32).collect();
        elem_value.set_int(&vals);
        Ok(true)
    }

    fn read_params(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            STREAM_INPUT_GAIN_NAME => Self::read_int(elem_value, &self.0.stream_gains),
            ANALOG_INPUT_GAIN_NAME => Self::read_int(elem_value, &self.0.analog_gains),
            SPDIF_INPUT_GAIN_NAME => Self::read_int(elem_value, &self.0.spdif_gains),
            ADAT_INPUT_GAIN_NAME => Self::read_int(elem_value, &self.0.adat_gains),
            ANALOG_INPUT_BALANCE_NAME => Self::read_int(elem_value, &self.0.analog_balances),
            SPDIF_INPUT_BALANCE_NAME => Self::read_int(elem_value, &self.0.spdif_balances),
            ADAT_INPUT_BALANCE_NAME => Self::read_int(elem_value, &self.0.adat_balances),
            _ => Ok(false),
        }
    }

    fn write_int<T>(
        curr: &mut MaudioSpecialInputParameters,
        elem_value: &ElemValue,
        count: usize,
        req: &FwReq,
        unit: &(SndUnit, FwNode),
        state: &mut MaudioSpecialStateCache,
        timeout_ms: u32,
        set: T,
    ) -> Result<bool, Error>
    where
        T: Fn(&mut MaudioSpecialInputParameters, &[i16]),
    {
        let mut params = curr.clone();
        let vals = &elem_value.int()[..count];
        let levels: Vec<i16> = vals.iter().map(|&val| val as i16).collect();
        set(&mut params, &levels);
        MaudioSpecialInputProtocol::partial_update(req, &unit.1, &params, state, curr, timeout_ms)
            .map(|_| true)
    }

    fn write_params(
        &mut self,
        state: &mut MaudioSpecialStateCache,
        unit: &(SndUnit, FwNode),
        req: &FwReq,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            STREAM_INPUT_GAIN_NAME => Self::write_int(
                &mut self.0,
                elem_value,
                STREAM_INPUT_LABELS.len(),
                req,
                unit,
                state,
                timeout_ms,
                |params, vals| params.stream_gains.copy_from_slice(&vals),
            ),
            ANALOG_INPUT_GAIN_NAME => Self::write_int(
                &mut self.0,
                elem_value,
                ANALOG_INPUT_LABELS.len(),
                req,
                unit,
                state,
                timeout_ms,
                |params, vals| params.analog_gains.copy_from_slice(&vals),
            ),
            SPDIF_INPUT_GAIN_NAME => Self::write_int(
                &mut self.0,
                elem_value,
                SPDIF_INPUT_LABELS.len(),
                req,
                unit,
                state,
                timeout_ms,
                |params, vals| params.spdif_gains.copy_from_slice(&vals),
            ),
            ADAT_INPUT_GAIN_NAME => Self::write_int(
                &mut self.0,
                elem_value,
                ADAT_INPUT_LABELS.len(),
                req,
                unit,
                state,
                timeout_ms,
                |params, vals| params.adat_gains.copy_from_slice(&vals),
            ),
            ANALOG_INPUT_BALANCE_NAME => Self::write_int(
                &mut self.0,
                elem_value,
                ANALOG_INPUT_LABELS.len(),
                req,
                unit,
                state,
                timeout_ms,
                |params, vals| params.analog_balances.copy_from_slice(&vals),
            ),
            SPDIF_INPUT_BALANCE_NAME => Self::write_int(
                &mut self.0,
                elem_value,
                SPDIF_INPUT_LABELS.len(),
                req,
                unit,
                state,
                timeout_ms,
                |params, vals| params.spdif_balances.copy_from_slice(&vals),
            ),
            ADAT_INPUT_BALANCE_NAME => Self::write_int(
                &mut self.0,
                elem_value,
                ADAT_INPUT_LABELS.len(),
                req,
                unit,
                state,
                timeout_ms,
                |params, vals| params.adat_balances.copy_from_slice(&vals),
            ),
            _ => Ok(false),
        }
    }
}

#[derive(Default, Debug)]
struct OutputCtl(MaudioSpecialOutputParameters, Vec<ElemId>);

const HP_VOL_NAME: &str = "headphone-volume";

const ANALOG_OUTPUT_PAIR_LABELS: [&'static str; 2] = ["analog-output-1/2", "analog-output-3/4"];
const HEADPHONE_PAIR_LABELS: [&'static str; 2] = ["headphone-1/2", "headphone-3/4"];

fn output_source_to_str(src: &OutputSource) -> &str {
    match src {
        OutputSource::MixerOutputPair => "mixer-output",
        OutputSource::AuxOutputPair0 => "aux-output-1/2",
    }
}

fn headphone_source_to_str(src: &HeadphoneSource) -> &str {
    match src {
        HeadphoneSource::MixerOutputPair0 => "mixer-output-1/2",
        HeadphoneSource::MixerOutputPair1 => "mixer-output-3/4",
        HeadphoneSource::AuxOutputPair0 => "aux-output-1/2",
    }
}

impl OutputCtl {
    const ANALOG_OUTPUT_PAIR_SOURCES: [OutputSource; 2] =
        [OutputSource::MixerOutputPair, OutputSource::AuxOutputPair0];

    const HEADPHONE_PAIR_SOURCES: [HeadphoneSource; 3] = [
        HeadphoneSource::MixerOutputPair0,
        HeadphoneSource::MixerOutputPair1,
        HeadphoneSource::AuxOutputPair0,
    ];

    const VOLUME_TLV: DbInterval = DbInterval {
        min: -12800,
        max: 0,
        linear: false,
        mute_avail: false,
    };

    fn add_volume_elem(
        card_cntr: &mut CardCntr,
        name: &str,
        labels: &[&str],
    ) -> Result<Vec<ElemId>, Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
        card_cntr.add_int_elems(
            &elem_id,
            1,
            MaudioSpecialOutputProtocol::VOLUME_MIN as i32,
            MaudioSpecialOutputProtocol::VOLUME_MAX as i32,
            MaudioSpecialOutputProtocol::VOLUME_STEP as i32,
            labels.len(),
            Some(&Into::<Vec<u32>>::into(Self::VOLUME_TLV)),
            true,
        )
    }

    fn add_enum_elem<T, F>(
        card_cntr: &mut CardCntr,
        name: &str,
        labels: &[&str],
        items: &[T],
        to_str: F,
    ) -> Result<Vec<ElemId>, Error>
    where
        F: Fn(&T) -> &str,
    {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
        let item_labels: Vec<&str> = items.iter().map(|src| to_str(src)).collect();
        card_cntr.add_enum_elems(&elem_id, 1, labels.len(), &item_labels, None, true)
    }

    fn load_params(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        Self::add_volume_elem(card_cntr, OUT_VOL_NAME, &ANALOG_OUTPUT_LABELS)?;
        Self::add_volume_elem(card_cntr, HP_VOL_NAME, &HEADPHONE_LABELS)
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        Self::add_enum_elem(
            card_cntr,
            OUT_SRC_NAME,
            &ANALOG_OUTPUT_PAIR_LABELS,
            &Self::ANALOG_OUTPUT_PAIR_SOURCES,
            output_source_to_str,
        )?;
        Self::add_enum_elem(
            card_cntr,
            HP_SRC_NAME,
            &HEADPHONE_PAIR_LABELS,
            &Self::HEADPHONE_PAIR_SOURCES,
            headphone_source_to_str,
        )?;

        Ok(())
    }

    fn cache(
        &mut self,
        req: &FwReq,
        node: &FwNode,
        cache: &mut MaudioSpecialStateCache,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        MaudioSpecialOutputProtocol::whole_update(req, node, &mut self.0, cache, timeout_ms)
    }

    fn read_int(elem_value: &mut ElemValue, gains: &[i16]) -> Result<bool, Error> {
        let vals: Vec<i32> = gains.iter().map(|&val| val as i32).collect();
        elem_value.set_int(&vals);
        Ok(true)
    }

    fn read_enum<T: Eq>(
        elem_value: &mut ElemValue,
        srcs: &[T],
        src_list: &[T],
    ) -> Result<bool, Error> {
        let vals: Vec<u32> = srcs
            .iter()
            .map(|s| {
                src_list
                    .iter()
                    .position(|entry| entry.eq(s))
                    .map(|pos| pos as u32)
                    .unwrap()
            })
            .collect();
        elem_value.set_enum(&vals);
        Ok(true)
    }

    fn read_params(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            OUT_VOL_NAME => Self::read_int(elem_value, &self.0.analog_volumes),
            OUT_SRC_NAME => Self::read_enum(
                elem_value,
                &self.0.analog_pair_sources,
                &Self::ANALOG_OUTPUT_PAIR_SOURCES,
            ),
            HP_VOL_NAME => Self::read_int(elem_value, &self.0.headphone_volumes),
            HP_SRC_NAME => Self::read_enum(
                elem_value,
                &self.0.headphone_pair_sources,
                &Self::HEADPHONE_PAIR_SOURCES,
            ),
            _ => Ok(false),
        }
    }

    fn write_int<T>(
        curr: &mut MaudioSpecialOutputParameters,
        elem_value: &ElemValue,
        labels: &[&str],
        req: &FwReq,
        unit: &(SndUnit, FwNode),
        state: &mut MaudioSpecialStateCache,
        timeout_ms: u32,
        set: T,
    ) -> Result<bool, Error>
    where
        T: Fn(&mut MaudioSpecialOutputParameters, &[i16]),
    {
        let mut params = curr.clone();
        let vals = &elem_value.int()[..labels.len()];
        let levels: Vec<i16> = vals.iter().map(|&val| val as i16).collect();
        set(&mut params, &levels);
        MaudioSpecialOutputProtocol::partial_update(req, &unit.1, &params, state, curr, timeout_ms)
            .map(|_| true)
    }

    fn write_enum<T, F>(
        curr: &mut MaudioSpecialOutputParameters,
        elem_value: &ElemValue,
        labels: &[&str],
        item_list: &[T],
        req: &FwReq,
        unit: &(SndUnit, FwNode),
        state: &mut MaudioSpecialStateCache,
        timeout_ms: u32,
        set: F,
    ) -> Result<bool, Error>
    where
        T: Eq + Copy,
        F: Fn(&mut MaudioSpecialOutputParameters, &[T]),
    {
        let mut params = curr.clone();
        let vals = &elem_value.enumerated()[..labels.len()];
        let mut srcs = Vec::with_capacity(vals.len());
        vals.iter().try_for_each(|&val| {
            item_list
                .iter()
                .nth(val as usize)
                .ok_or_else(|| {
                    let msg = format!("Invalid index: {}", val);
                    Error::new(FileError::Inval, &msg)
                })
                .map(|&src| srcs.push(src))
        })?;
        set(&mut params, &srcs);
        MaudioSpecialOutputProtocol::partial_update(req, &unit.1, &params, state, curr, timeout_ms)
            .map(|_| true)
    }

    fn write_params(
        &mut self,
        state: &mut MaudioSpecialStateCache,
        unit: &(SndUnit, FwNode),
        req: &FwReq,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            OUT_VOL_NAME => Self::write_int(
                &mut self.0,
                elem_value,
                &ANALOG_OUTPUT_LABELS[..],
                req,
                unit,
                state,
                timeout_ms,
                |params, vals| params.analog_volumes.copy_from_slice(vals),
            ),
            OUT_SRC_NAME => Self::write_enum(
                &mut self.0,
                elem_value,
                &ANALOG_OUTPUT_PAIR_LABELS[..],
                &Self::ANALOG_OUTPUT_PAIR_SOURCES,
                req,
                unit,
                state,
                timeout_ms,
                |params, vals| params.analog_pair_sources.copy_from_slice(vals),
            ),
            HP_VOL_NAME => Self::write_int(
                &mut self.0,
                elem_value,
                &HEADPHONE_LABELS[..],
                req,
                unit,
                state,
                timeout_ms,
                |params, srcs| params.headphone_volumes.copy_from_slice(srcs),
            ),
            HP_SRC_NAME => Self::write_enum(
                &mut self.0,
                elem_value,
                &HEADPHONE_PAIR_LABELS[..],
                &Self::HEADPHONE_PAIR_SOURCES,
                req,
                unit,
                state,
                timeout_ms,
                |params, srcs| params.headphone_pair_sources.copy_from_slice(srcs),
            ),
            _ => Ok(false),
        }
    }
}

#[derive(Default, Debug)]
struct AuxCtl(MaudioSpecialAuxParameters);

const AUX_OUT_VOL_NAME: &str = "aux-output-volume";
const AUX_STREAM_SRC_NAME: &str = "aux-stream-source";
const AUX_ANALOG_SRC_NAME: &str = "aux-analog-source";
const AUX_SPDIF_SRC_NAME: &str = "aux-spdif-source";
const AUX_ADAT_SRC_NAME: &str = "aux-adat-source";

impl AuxCtl {
    const VOL_TLV: DbInterval = DbInterval {
        min: -12800,
        max: 0,
        linear: false,
        mute_avail: false,
    };

    const GAIN_TLV: DbInterval = DbInterval {
        min: -12800,
        max: 0,
        linear: false,
        mute_avail: false,
    };

    fn add_input_int_elem(
        card_cntr: &mut CardCntr,
        name: &str,
        labels: &[&str],
    ) -> Result<Vec<ElemId>, Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
        card_cntr.add_int_elems(
            &elem_id,
            1,
            MaudioSpecialAuxProtocol::GAIN_MIN as i32,
            MaudioSpecialAuxProtocol::GAIN_MAX as i32,
            MaudioSpecialAuxProtocol::GAIN_STEP as i32,
            labels.len(),
            Some(&Into::<Vec<u32>>::into(Self::GAIN_TLV)),
            true,
        )
    }

    fn load_params(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, AUX_OUT_VOL_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            1,
            MaudioSpecialAuxProtocol::VOLUME_MIN as i32,
            MaudioSpecialAuxProtocol::VOLUME_MAX as i32,
            MaudioSpecialAuxProtocol::VOLUME_STEP as i32,
            AUX_OUTPUT_LABELS.len(),
            Some(&Into::<Vec<u32>>::into(Self::VOL_TLV)),
            true,
        )?;

        let _ = Self::add_input_int_elem(card_cntr, AUX_STREAM_SRC_NAME, &STREAM_INPUT_LABELS)?;
        let _ = Self::add_input_int_elem(card_cntr, AUX_ANALOG_SRC_NAME, &ANALOG_INPUT_LABELS)?;
        let _ = Self::add_input_int_elem(card_cntr, AUX_SPDIF_SRC_NAME, &SPDIF_INPUT_LABELS)?;
        let _ = Self::add_input_int_elem(card_cntr, AUX_ADAT_SRC_NAME, &ADAT_INPUT_LABELS)?;

        Ok(())
    }

    fn cache(
        &mut self,
        req: &FwReq,
        node: &FwNode,
        cache: &mut MaudioSpecialStateCache,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        MaudioSpecialAuxProtocol::whole_update(req, node, &mut self.0, cache, timeout_ms)
    }

    fn read_int(elem_value: &mut ElemValue, gains: &[i16]) -> Result<bool, Error> {
        let vals: Vec<i32> = gains.iter().map(|&val| val as i32).collect();
        elem_value.set_int(&vals);
        Ok(true)
    }

    fn read_params(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            AUX_OUT_VOL_NAME => Self::read_int(elem_value, &self.0.output_volumes),
            AUX_STREAM_SRC_NAME => Self::read_int(elem_value, &self.0.stream_gains),
            AUX_ANALOG_SRC_NAME => Self::read_int(elem_value, &self.0.analog_gains),
            AUX_SPDIF_SRC_NAME => Self::read_int(elem_value, &self.0.spdif_gains),
            AUX_ADAT_SRC_NAME => Self::read_int(elem_value, &self.0.adat_gains),
            _ => Ok(false),
        }
    }

    fn write_int<F>(
        curr: &mut MaudioSpecialAuxParameters,
        elem_value: &ElemValue,
        labels: &[&str],
        req: &FwReq,
        unit: &(SndUnit, FwNode),
        state: &mut MaudioSpecialStateCache,
        timeout_ms: u32,
        set: F,
    ) -> Result<bool, Error>
    where
        F: Fn(&mut MaudioSpecialAuxParameters, &[i16]),
    {
        let mut params = curr.clone();
        let vals = &elem_value.int()[..labels.len()];
        let levels: Vec<i16> = vals.iter().map(|&val| val as i16).collect();
        set(&mut params, &levels);
        MaudioSpecialAuxProtocol::partial_update(req, &unit.1, &params, state, curr, timeout_ms)
            .map(|_| true)
    }

    fn write_params(
        &mut self,
        state: &mut MaudioSpecialStateCache,
        unit: &(SndUnit, FwNode),
        req: &FwReq,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            AUX_OUT_VOL_NAME => Self::write_int(
                &mut self.0,
                elem_value,
                &AUX_OUTPUT_LABELS[..],
                req,
                unit,
                state,
                timeout_ms,
                |params, vals| params.output_volumes.copy_from_slice(&vals),
            ),
            AUX_STREAM_SRC_NAME => Self::write_int(
                &mut self.0,
                elem_value,
                &STREAM_INPUT_LABELS[..],
                req,
                unit,
                state,
                timeout_ms,
                |params, vals| params.stream_gains.copy_from_slice(&vals),
            ),
            AUX_ANALOG_SRC_NAME => Self::write_int(
                &mut self.0,
                elem_value,
                &ANALOG_INPUT_LABELS[..],
                req,
                unit,
                state,
                timeout_ms,
                |params, vals| params.analog_gains.copy_from_slice(&vals),
            ),
            AUX_SPDIF_SRC_NAME => Self::write_int(
                &mut self.0,
                elem_value,
                &SPDIF_INPUT_LABELS[..],
                req,
                unit,
                state,
                timeout_ms,
                |params, vals| params.spdif_gains.copy_from_slice(&vals),
            ),
            AUX_ADAT_SRC_NAME => Self::write_int(
                &mut self.0,
                elem_value,
                &ADAT_INPUT_LABELS[..],
                req,
                unit,
                state,
                timeout_ms,
                |params, vals| params.adat_gains.copy_from_slice(&vals),
            ),
            _ => Ok(false),
        }
    }
}

#[derive(Default, Debug)]
struct MixerCtl(MaudioSpecialMixerParameters);

const MIXER_ANALOG_SRC_NAME: &str = "mixer-analog-source";
const MIXER_SPDIF_SRC_NAME: &str = "mixer-spdif-source";
const MIXER_ADAT_SRC_NAME: &str = "mixer-adat-source";
const MIXER_STREAM_SRC_NAME: &str = "mixer-stream-source";

const MIXER_LABELS: [&str; 2] = ["mixer-1/2", "mixer-3/4"];

const ANALOG_INPUT_PAIR_LABELS: [&str; 4] = [
    "analog-input-1/2",
    "analog-input-3/4",
    "analog-input-5/6",
    "analog-input-7/8",
];

const SPDIF_INPUT_PAIR_LABELS: [&str; 1] = ["spdif-input-1/2"];

const ADAT_INPUT_PAIR_LABELS: [&str; 4] = [
    "adat-input-1/2",
    "adat-input-3/4",
    "adat-input-5/6",
    "adat-input-7/8",
];

const STREAM_INPUT_PAIR_LABELS: [&str; 2] = ["stream-input-1/2", "stream-input-3/4"];

impl MixerCtl {
    fn add_bool_elem(
        card_cntr: &mut CardCntr,
        name: &str,
        labels: &[&str],
    ) -> Result<Vec<ElemId>, Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
        card_cntr.add_bool_elems(&elem_id, MIXER_LABELS.len(), labels.len(), true)
    }

    fn load_params(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let _ = Self::add_bool_elem(card_cntr, MIXER_ANALOG_SRC_NAME, &ANALOG_INPUT_PAIR_LABELS)?;
        let _ = Self::add_bool_elem(card_cntr, MIXER_SPDIF_SRC_NAME, &SPDIF_INPUT_PAIR_LABELS)?;
        let _ = Self::add_bool_elem(card_cntr, MIXER_ADAT_SRC_NAME, &ADAT_INPUT_PAIR_LABELS)?;
        let _ = Self::add_bool_elem(card_cntr, MIXER_STREAM_SRC_NAME, &STREAM_INPUT_PAIR_LABELS)?;

        Ok(())
    }

    fn cache(
        &mut self,
        req: &FwReq,
        node: &FwNode,
        cache: &mut MaudioSpecialStateCache,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        MaudioSpecialMixerProtocol::whole_update(req, node, &mut self.0, cache, timeout_ms)
    }

    fn read_params(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            MIXER_ANALOG_SRC_NAME => {
                let index = elem_id.index() as usize;
                elem_value.set_bool(&self.0.analog_pairs[index]);
                Ok(true)
            }
            MIXER_SPDIF_SRC_NAME => {
                let index = elem_id.index() as usize;
                elem_value.set_bool(&[self.0.spdif_pairs[index]]);
                Ok(true)
            }
            MIXER_ADAT_SRC_NAME => {
                let index = elem_id.index() as usize;
                elem_value.set_bool(&self.0.adat_pairs[index]);
                Ok(true)
            }
            MIXER_STREAM_SRC_NAME => {
                let index = elem_id.index() as usize;
                elem_value.set_bool(&self.0.stream_pairs[index]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write_bool<T>(
        curr: &mut MaudioSpecialMixerParameters,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        labels: &[&str],
        req: &FwReq,
        unit: &(SndUnit, FwNode),
        state: &mut MaudioSpecialStateCache,
        timeout_ms: u32,
        set: T,
    ) -> Result<bool, Error>
    where
        T: Fn(&mut MaudioSpecialMixerParameters, usize, &[bool]),
    {
        let index = elem_id.index() as usize;
        let mut params = curr.clone();
        let vals = &elem_value.boolean()[..labels.len()];
        set(&mut params, index, &vals);
        MaudioSpecialMixerProtocol::partial_update(req, &unit.1, &params, state, curr, timeout_ms)
            .map(|_| true)
    }

    fn write_params(
        &mut self,
        state: &mut MaudioSpecialStateCache,
        unit: &(SndUnit, FwNode),
        req: &FwReq,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            MIXER_ANALOG_SRC_NAME => Self::write_bool(
                &mut self.0,
                elem_id,
                elem_value,
                &ANALOG_INPUT_PAIR_LABELS[..],
                req,
                unit,
                state,
                timeout_ms,
                |params, index, vals| params.analog_pairs[index].copy_from_slice(&vals),
            ),
            MIXER_SPDIF_SRC_NAME => Self::write_bool(
                &mut self.0,
                elem_id,
                elem_value,
                &SPDIF_INPUT_PAIR_LABELS[..],
                req,
                unit,
                state,
                timeout_ms,
                |params, index, vals| params.spdif_pairs[index] = vals[0],
            ),
            MIXER_ADAT_SRC_NAME => Self::write_bool(
                &mut self.0,
                elem_id,
                elem_value,
                &ADAT_INPUT_PAIR_LABELS[..],
                req,
                unit,
                state,
                timeout_ms,
                |params, index, vals| params.adat_pairs[index].copy_from_slice(&vals),
            ),
            MIXER_STREAM_SRC_NAME => Self::write_bool(
                &mut self.0,
                elem_id,
                elem_value,
                &STREAM_INPUT_PAIR_LABELS[..],
                req,
                unit,
                state,
                timeout_ms,
                |params, index, vals| params.stream_pairs[index].copy_from_slice(&vals),
            ),
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
        let mut card_cntr = CardCntr::default();
        let mut ctl = ClkCtl::<Fw1814ClkProtocol>::default();

        let error = ctl.load_freq(&mut card_cntr).unwrap_err();
        assert_eq!(error.kind::<CardError>(), Some(CardError::Failed));

        let mut ctl = ClkCtl::<ProjectMixClkProtocol>::default();

        let error = ctl.load_freq(&mut card_cntr).unwrap_err();
        assert_eq!(error.kind::<CardError>(), Some(CardError::Failed));
    }

    #[test]
    fn test_meter_label_count() {
        let meter_ctl = MeterCtl::default();
        assert_eq!(meter_ctl.0.analog_inputs.len(), ANALOG_INPUT_LABELS.len());
        assert_eq!(meter_ctl.0.spdif_inputs.len(), SPDIF_INPUT_LABELS.len());
        assert_eq!(meter_ctl.0.adat_inputs.len(), ADAT_INPUT_LABELS.len());
        assert_eq!(meter_ctl.0.analog_outputs.len(), ANALOG_OUTPUT_LABELS.len());
        assert_eq!(meter_ctl.0.spdif_outputs.len(), SPDIF_OUTPUT_LABELS.len());
        assert_eq!(meter_ctl.0.adat_outputs.len(), ADAT_OUTPUT_LABELS.len());
        assert_eq!(meter_ctl.0.headphone.len(), HEADPHONE_LABELS.len());
        assert_eq!(meter_ctl.0.aux_outputs.len(), AUX_OUTPUT_LABELS.len());
    }

    #[test]
    fn test_input_label_count() {
        let input_ctl = InputCtl::default();
        assert_eq!(input_ctl.0.stream_gains.len(), STREAM_INPUT_LABELS.len());
        assert_eq!(input_ctl.0.analog_gains.len(), ANALOG_INPUT_LABELS.len());
        assert_eq!(input_ctl.0.spdif_gains.len(), SPDIF_INPUT_LABELS.len());
        assert_eq!(input_ctl.0.adat_gains.len(), ADAT_INPUT_LABELS.len());
        assert_eq!(input_ctl.0.analog_balances.len(), ANALOG_INPUT_LABELS.len());
        assert_eq!(input_ctl.0.spdif_balances.len(), SPDIF_INPUT_LABELS.len());
        assert_eq!(input_ctl.0.adat_balances.len(), ADAT_INPUT_LABELS.len());
    }

    #[test]
    fn test_output_label_count() {
        let output_ctl = OutputCtl::default();
        assert_eq!(
            output_ctl.0.analog_volumes.len(),
            ANALOG_OUTPUT_LABELS.len()
        );
        assert_eq!(output_ctl.0.headphone_volumes.len(), HEADPHONE_LABELS.len());
        assert_eq!(
            output_ctl.0.analog_pair_sources.len(),
            ANALOG_OUTPUT_PAIR_LABELS.len()
        );
    }

    #[test]
    fn test_aux_label_count() {
        let aux_ctl = AuxCtl::default();
        assert_eq!(aux_ctl.0.output_volumes.len(), AUX_OUTPUT_LABELS.len());
        assert_eq!(aux_ctl.0.stream_gains.len(), STREAM_INPUT_LABELS.len());
        assert_eq!(aux_ctl.0.analog_gains.len(), ANALOG_INPUT_LABELS.len());
        assert_eq!(aux_ctl.0.spdif_gains.len(), SPDIF_INPUT_LABELS.len());
        assert_eq!(aux_ctl.0.adat_gains.len(), ADAT_INPUT_LABELS.len());
    }

    #[test]
    fn test_mixer_label_count() {
        let mixer_ctl = MixerCtl::default();
        assert_eq!(mixer_ctl.0.analog_pairs.len(), MIXER_LABELS.len());
        assert_eq!(
            mixer_ctl.0.analog_pairs[0].len(),
            ANALOG_INPUT_PAIR_LABELS.len()
        );
        assert_eq!(mixer_ctl.0.spdif_pairs.len(), MIXER_LABELS.len());
        assert_eq!(mixer_ctl.0.adat_pairs.len(), MIXER_LABELS.len());
        assert_eq!(
            mixer_ctl.0.adat_pairs[0].len(),
            ADAT_INPUT_PAIR_LABELS.len()
        );
        assert_eq!(mixer_ctl.0.stream_pairs.len(), MIXER_LABELS.len());
        assert_eq!(
            mixer_ctl.0.stream_pairs[0].len(),
            STREAM_INPUT_PAIR_LABELS.len()
        );
    }
}
