// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

use {super::*, protocols::lexicon::*};

#[derive(Default)]
pub struct IonixModel {
    req: FwReq,
    sections: GeneralSections,
    common_ctl: CommonCtl,
    meter_ctl: MeterCtl,
    mixer_ctl: MixerCtl,
}

const TIMEOUT_MS: u32 = 20;

impl IonixModel {
    pub fn cache(&mut self, unit: &mut (SndDice, FwNode)) -> Result<(), Error> {
        IonixProtocol::read_general_sections(&self.req, &unit.1, &mut self.sections, TIMEOUT_MS)?;

        self.common_ctl
            .whole_cache(&self.req, &unit.1, &mut self.sections, TIMEOUT_MS)?;
        self.meter_ctl.cache(&self.req, &unit.1, TIMEOUT_MS)?;

        Ok(())
    }
}

impl CtlModel<(SndDice, FwNode)> for IonixModel {
    fn load(&mut self, _: &mut (SndDice, FwNode), card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.common_ctl.load(card_cntr, &self.sections).map(
            |(measured_elem_id_list, notified_elem_id_list)| {
                self.common_ctl.0 = measured_elem_id_list;
                self.common_ctl.1 = notified_elem_id_list;
            },
        )?;

        self.meter_ctl.load(card_cntr)?;
        self.mixer_ctl.load(card_cntr)?;

        Ok(())
    }

    fn read(
        &mut self,
        unit: &mut (SndDice, FwNode),
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.common_ctl.read(&self.sections, elem_id, elem_value)? {
            Ok(true)
        } else if self
            .mixer_ctl
            .read(unit, &mut self.req, elem_id, elem_value, TIMEOUT_MS)?
        {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(
        &mut self,
        unit: &mut (SndDice, FwNode),
        elem_id: &ElemId,
        old: &ElemValue,
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
            .write(unit, &mut self.req, elem_id, old, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl NotifyModel<(SndDice, FwNode), u32> for IonixModel {
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

impl MeasureModel<(SndDice, FwNode)> for IonixModel {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.common_ctl.0);
        elem_id_list.extend_from_slice(&self.meter_ctl.1);
    }

    fn measure_states(&mut self, unit: &mut (SndDice, FwNode)) -> Result<(), Error> {
        self.common_ctl
            .measure(&self.req, &unit.1, &mut self.sections, TIMEOUT_MS)?;
        self.meter_ctl.cache(&self.req, &unit.1, TIMEOUT_MS)?;
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
        } else if self.meter_ctl.read_measured_elem(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[derive(Default, Debug)]
struct CommonCtl(Vec<ElemId>, Vec<ElemId>);

impl CommonCtlOperation<IonixProtocol> for CommonCtl {}

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
        IonixProtocol::cache_whole_params(req, node, &mut self.0, timeout_ms)
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
                elem_value.set_int(&self.0.spdif_inputs);
                Ok(true)
            }
            Self::STREAM_INPUT_NAME => {
                elem_value.set_int(&self.0.stream_inputs);
                Ok(true)
            }
            Self::ANALOG_INPUT_NAME => {
                elem_value.set_int(&self.0.analog_inputs);
                Ok(true)
            }
            Self::BUS_OUTPUT_NAME => {
                elem_value.set_int(&self.0.bus_outputs);
                Ok(true)
            }
            Self::MAIN_OUTPUT_NAME => {
                elem_value.set_int(&self.0.main_outputs);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

fn mixer_src_to_string(src: &MixerSrc) -> String {
    match src {
        MixerSrc::Stream(ch) => format!("Stream-{}", ch + 1),
        MixerSrc::Spdif(ch) => format!("S/PDIF-{}", ch + 1),
        MixerSrc::Analog(ch) => format!("Analog-{}", ch + 1),
    }
}

#[derive(Default, Debug)]
struct MixerCtl;

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

    const MIXER_SRCS: [MixerSrc; 20] = [
        MixerSrc::Analog(0),
        MixerSrc::Analog(1),
        MixerSrc::Analog(2),
        MixerSrc::Analog(3),
        MixerSrc::Analog(4),
        MixerSrc::Analog(5),
        MixerSrc::Analog(6),
        MixerSrc::Analog(7),
        MixerSrc::Spdif(0),
        MixerSrc::Spdif(1),
        MixerSrc::Stream(0),
        MixerSrc::Stream(1),
        MixerSrc::Stream(2),
        MixerSrc::Stream(3),
        MixerSrc::Stream(4),
        MixerSrc::Stream(5),
        MixerSrc::Stream(6),
        MixerSrc::Stream(7),
        MixerSrc::Stream(8),
        MixerSrc::Stream(9),
    ];

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let labels: Vec<String> = Self::MIXER_SRCS
            .iter()
            .map(|s| mixer_src_to_string(s))
            .collect();
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

    fn read(
        &self,
        unit: &mut (SndDice, FwNode),
        req: &mut FwReq,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            Self::BUS_SRC_GAIN_NAME => {
                let mixer = elem_id.index() as usize;
                ElemValueAccessor::<i32>::set_vals(elem_value, Self::MIXER_SRCS.len(), |idx| {
                    IonixProtocol::read_mixer_bus_src_gain(
                        req,
                        &mut unit.1,
                        mixer,
                        Self::MIXER_SRCS[idx],
                        timeout_ms,
                    )
                    .map(|val| val as i32)
                })
                .map(|_| true)
            }
            Self::MAIN_SRC_GAIN_NAME => {
                let mixer = elem_id.index() as usize;
                ElemValueAccessor::<i32>::set_vals(elem_value, Self::MIXER_SRCS.len(), |idx| {
                    IonixProtocol::read_mixer_main_src_gain(
                        req,
                        &mut unit.1,
                        mixer,
                        Self::MIXER_SRCS[idx],
                        timeout_ms,
                    )
                    .map(|val| val as i32)
                })
                .map(|_| true)
            }
            Self::REVERB_SRC_GAIN_NAME => {
                let mixer = elem_id.index() as usize;
                ElemValueAccessor::<i32>::set_vals(elem_value, Self::MIXER_SRCS.len(), |idx| {
                    IonixProtocol::read_mixer_reverb_src_gain(
                        req,
                        &mut unit.1,
                        mixer,
                        Self::MIXER_SRCS[idx],
                        timeout_ms,
                    )
                    .map(|val| val as i32)
                })
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }

    fn write(
        &mut self,
        unit: &mut (SndDice, FwNode),
        req: &mut FwReq,
        elem_id: &ElemId,
        old: &ElemValue,
        new: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            Self::BUS_SRC_GAIN_NAME => {
                let mixer = elem_id.index() as usize;
                ElemValueAccessor::<i32>::get_vals(new, old, Self::MIXER_SRCS.len(), |idx, val| {
                    IonixProtocol::write_mixer_bus_src_gain(
                        req,
                        &mut unit.1,
                        mixer,
                        Self::MIXER_SRCS[idx],
                        val as u32,
                        timeout_ms,
                    )
                })
                .map(|_| true)
            }
            Self::MAIN_SRC_GAIN_NAME => {
                let mixer = elem_id.index() as usize;
                ElemValueAccessor::<i32>::get_vals(new, old, Self::MIXER_SRCS.len(), |idx, val| {
                    IonixProtocol::write_mixer_main_src_gain(
                        req,
                        &mut unit.1,
                        mixer,
                        Self::MIXER_SRCS[idx],
                        val as u32,
                        timeout_ms,
                    )
                })
                .map(|_| true)
            }
            Self::REVERB_SRC_GAIN_NAME => {
                let mixer = elem_id.index() as usize;
                ElemValueAccessor::<i32>::get_vals(new, old, Self::MIXER_SRCS.len(), |idx, val| {
                    IonixProtocol::write_mixer_reverb_src_gain(
                        req,
                        &mut unit.1,
                        mixer,
                        Self::MIXER_SRCS[idx],
                        val as u32,
                        timeout_ms,
                    )
                })
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }
}
