// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto
use glib::Error;

use alsactl::{ElemId, ElemIfaceType, ElemValue, ElemValueExt};

use hinawa::FwReq;
use hinawa::{SndDice, SndUnitExt};

use alsa_ctl_tlv_codec::items::DbInterval;

use core::card_cntr::*;
use core::elem_value_accessor::*;

use dice_protocols::tcat::{*, global_section::*};
use dice_protocols::lexicon::*;

use crate::common_ctl::*;

#[derive(Default)]
pub struct IonixModel{
    req: FwReq,
    sections: GeneralSections,
    ctl: CommonCtl,
    meter_ctl: MeterCtl,
    mixer_ctl: MixerCtl,
}

const TIMEOUT_MS: u32 = 20;

impl CtlModel<SndDice> for IonixModel {
    fn load(&mut self, unit: &mut SndDice, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let mut node = unit.get_node();

        self.sections = self.req.read_general_sections(&mut node, TIMEOUT_MS)?;
        let caps = self.req.read_clock_caps(&mut node, &self.sections, TIMEOUT_MS)?;
        let src_labels = self.req.read_clock_source_labels(&mut node, &self.sections, TIMEOUT_MS)?;
        self.ctl.load(card_cntr, &caps, &src_labels)?;

        self.meter_ctl.load(card_cntr)?;
        self.mixer_ctl.load(card_cntr)?;

        Ok(())
    }

    fn read(&mut self, unit: &mut SndDice, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        if self.ctl.read(unit, &mut self.req, &self.sections, elem_id, elem_value, TIMEOUT_MS)? {
            Ok(true)
        } else if self.mixer_ctl.read(unit, &mut self.req, elem_id, elem_value, TIMEOUT_MS)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(&mut self, unit: &mut SndDice, elem_id: &ElemId, old: &ElemValue, new: &ElemValue)
        -> Result<bool, Error>
    {
        if self.ctl.write(unit, &mut self.req, &self.sections, elem_id, old, new, TIMEOUT_MS)? {
            Ok(true)
        } else if self.mixer_ctl.write(unit, &mut self.req, elem_id, old, new, TIMEOUT_MS)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl NotifyModel<SndDice, u32> for IonixModel {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.ctl.notified_elem_list);
    }

    fn parse_notification(&mut self, unit: &mut SndDice, msg: &u32) -> Result<(), Error> {
        self.ctl.parse_notification(unit, &mut self.req, &self.sections, *msg, TIMEOUT_MS)
    }

    fn read_notified_elem(&mut self, _: &SndDice, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        self.ctl.read_notified_elem(elem_id, elem_value)
    }
}

impl MeasureModel<SndDice> for IonixModel {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.ctl.measured_elem_list);
        elem_id_list.extend_from_slice(&self.meter_ctl.measured_elem_list);
    }

    fn measure_states(&mut self, unit: &mut SndDice) -> Result<(), Error> {
        self.ctl.measure_states(unit, &mut self.req, &self.sections, TIMEOUT_MS)?;
        self.meter_ctl.measure_states(unit, &mut self.req, TIMEOUT_MS)?;
        Ok(())
    }

    fn measure_elem(&mut self, _: &SndDice, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        if self.ctl.measure_elem(elem_id, elem_value)? {
            Ok(true)
        } else if self.meter_ctl.read_measured_elem(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[derive(Default, Debug)]
struct MeterCtl{
    meters: IonixMeter,
    measured_elem_list: Vec<ElemId>,
}

impl MeterCtl {
    const SPDIF_INPUT_NAME: &'static str = "spdif-input-meter";
    const STREAM_INPUT_NAME: &'static str = "stream-input-meter";
    const ANALOG_INPUT_NAME: &'static str = "analog-input-meter";
    const BUS_OUTPUT_NAME: &'static str = "bus-output-meter";
    const MAIN_OUTPUT_NAME: &'static str = "main-output-meter";

    const LEVEL_MIN: i32 = 0;
    const LEVEL_MAX: i32 = 0x00000fff;
    const LEVEL_STEP: i32 = 1;
    const LEVEL_TLV: DbInterval = DbInterval{min: -6000, max: 0, linear: false, mute_avail: false};

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        [
            (Self::SPDIF_INPUT_NAME, IonixProtocol::SPDIF_INPUT_COUNT),
            (Self::STREAM_INPUT_NAME, IonixProtocol::STREAM_INPUT_COUNT),
            (Self::ANALOG_INPUT_NAME, IonixProtocol::ANALOG_INPUT_COUNT),
            (Self::BUS_OUTPUT_NAME, IonixProtocol::MIXER_BUS_COUNT),
            (Self::MAIN_OUTPUT_NAME, IonixProtocol::MIXER_MAIN_COUNT),
        ].iter()
            .try_for_each(|&(name, count)| {
                let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
                card_cntr.add_int_elems(&elem_id, 1, Self::LEVEL_MIN, Self::LEVEL_MAX, Self::LEVEL_STEP,
                                        count, Some(&Vec::<u32>::from(Self::LEVEL_TLV)), false)
                    .map(|mut elem_id_list| self.measured_elem_list.append(&mut elem_id_list))
            })?;

        Ok(())
    }

    fn measure_states(
        &mut self,
        unit: &mut SndDice,
        req: &mut FwReq,
        timeout_ms: u32
    ) -> Result<(), Error> {
        IonixProtocol::read_meters(
            req,
            &mut unit.get_node(),
            &mut self.meters,
            timeout_ms
        )
    }

    fn read_measured_elem(&self, elem_id: &ElemId, elem_value: &ElemValue) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            Self::SPDIF_INPUT_NAME => {
                elem_value.set_int(&self.meters.spdif_inputs);
                Ok(true)
            }
            Self::STREAM_INPUT_NAME => {
                elem_value.set_int(&self.meters.stream_inputs);
                Ok(true)
            }
            Self::ANALOG_INPUT_NAME => {
                elem_value.set_int(&self.meters.analog_inputs);
                Ok(true)
            }
            Self::BUS_OUTPUT_NAME => {
                elem_value.set_int(&self.meters.bus_outputs);
                Ok(true)
            }
            Self::MAIN_OUTPUT_NAME => {
                elem_value.set_int(&self.meters.main_outputs);
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
    const GAIN_TLV: DbInterval = DbInterval{min: -6000, max: 0, linear: false, mute_avail: false};

    const MIXER_SRCS: [MixerSrc;20] = [
        MixerSrc::Analog(0), MixerSrc::Analog(1), MixerSrc::Analog(2), MixerSrc::Analog(3),
        MixerSrc::Analog(4), MixerSrc::Analog(5), MixerSrc::Analog(6), MixerSrc::Analog(7),
        MixerSrc::Spdif(0), MixerSrc::Spdif(1),
        MixerSrc::Stream(0), MixerSrc::Stream(1), MixerSrc::Stream(2), MixerSrc::Stream(3),
        MixerSrc::Stream(4), MixerSrc::Stream(5), MixerSrc::Stream(6), MixerSrc::Stream(7),
        MixerSrc::Stream(8), MixerSrc::Stream(9),
    ];

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let labels: Vec<String> = Self::MIXER_SRCS.iter()
            .map(|s| mixer_src_to_string(s))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::BUS_SRC_GAIN_NAME, 0);
        let _ = card_cntr.add_int_elems(&elem_id, IonixProtocol::MIXER_BUS_COUNT,
                                        Self::GAIN_MIN, Self::GAIN_MAX, Self::GAIN_STEP, labels.len(),
                                        Some(&Vec::<u32>::from(Self::GAIN_TLV)), true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::MAIN_SRC_GAIN_NAME, 0);
        let _ = card_cntr.add_int_elems(&elem_id, IonixProtocol::MIXER_MAIN_COUNT,
                                        Self::GAIN_MIN, Self::GAIN_MAX, Self::GAIN_STEP, labels.len(),
                                        Some(&Vec::<u32>::from(Self::GAIN_TLV)), true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::REVERB_SRC_GAIN_NAME, 0);
        let _ = card_cntr.add_int_elems(&elem_id, IonixProtocol::MIXER_REVERB_COUNT,
                                        Self::GAIN_MIN, Self::GAIN_MAX, Self::GAIN_STEP, labels.len(),
                                        Some(&Vec::<u32>::from(Self::GAIN_TLV)), true)?;

        Ok(())
    }

    fn read(
        &self,
        unit: &mut SndDice,
        req: &mut FwReq,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
        timeout_ms: u32
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            Self::BUS_SRC_GAIN_NAME => {
                let mixer = elem_id.get_index() as usize;
                let mut node = unit.get_node();
                ElemValueAccessor::<i32>::set_vals(elem_value, Self::MIXER_SRCS.len(), |idx| {
                    IonixProtocol::read_mixer_bus_src_gain(
                        req,
                        &mut node,
                        mixer,
                        Self::MIXER_SRCS[idx],
                        timeout_ms
                    )
                        .map(|val| val as i32)
                })
                .map(|_| true)
            }
            Self::MAIN_SRC_GAIN_NAME => {
                let mixer = elem_id.get_index() as usize;
                let mut node = unit.get_node();
                ElemValueAccessor::<i32>::set_vals(elem_value, Self::MIXER_SRCS.len(), |idx| {
                    IonixProtocol::read_mixer_main_src_gain(
                        req,
                        &mut node,
                        mixer,
                        Self::MIXER_SRCS[idx],
                        timeout_ms
                    )
                        .map(|val| val as i32)
                })
                .map(|_| true)
            }
            Self::REVERB_SRC_GAIN_NAME => {
                let mixer = elem_id.get_index() as usize;
                let mut node = unit.get_node();
                ElemValueAccessor::<i32>::set_vals(elem_value, Self::MIXER_SRCS.len(), |idx| {
                    IonixProtocol::read_mixer_reverb_src_gain(
                        req,
                        &mut node,
                        mixer,
                        Self::MIXER_SRCS[idx],
                        timeout_ms
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
        unit: &mut SndDice,
        req: &mut FwReq,
        elem_id: &ElemId,
        old: &ElemValue,
        new: &ElemValue,
        timeout_ms: u32
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            Self::BUS_SRC_GAIN_NAME => {
                let mixer = elem_id.get_index() as usize;
                let mut node = unit.get_node();
                ElemValueAccessor::<i32>::get_vals(new, old, Self::MIXER_SRCS.len(), |idx, val| {
                    IonixProtocol::write_mixer_bus_src_gain(
                        req,
                        &mut node,
                        mixer,
                        Self::MIXER_SRCS[idx],
                        val as u32,
                        timeout_ms
                    )
                })
                .map(|_| true)
            }
            Self::MAIN_SRC_GAIN_NAME => {
                let mixer = elem_id.get_index() as usize;
                let mut node = unit.get_node();
                ElemValueAccessor::<i32>::get_vals(new, old, Self::MIXER_SRCS.len(), |idx, val| {
                    IonixProtocol::write_mixer_main_src_gain(
                        req,
                        &mut node,
                        mixer,
                        Self::MIXER_SRCS[idx],
                        val as u32,
                        timeout_ms
                    )
                })
                .map(|_| true)
            }
            Self::REVERB_SRC_GAIN_NAME => {
                let mixer = elem_id.get_index() as usize;
                let mut node = unit.get_node();
                ElemValueAccessor::<i32>::get_vals(new, old, Self::MIXER_SRCS.len(), |idx, val| {
                    IonixProtocol::write_mixer_reverb_src_gain(
                        req,
                        &mut node,
                        mixer,
                        Self::MIXER_SRCS[idx],
                        val as u32,
                        timeout_ms
                    )
                })
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }
}
