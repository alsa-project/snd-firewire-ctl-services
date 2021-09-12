// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::{Error, FileError};

use alsactl::{ElemId, ElemIfaceType, ElemValue, ElemValueExt, ElemValueExtManual};

use hinawa::{FwReq, SndDice, SndUnitExt};

use alsa_ctl_tlv_codec::items::DbInterval;

use core::card_cntr::*;
use core::elem_value_accessor::*;

use dice_protocols::tcat::{*, global_section::*, tx_stream_format_section::*};
use dice_protocols::alesis::{meter::*, mixer::*, output::*, *};

use crate::common_ctl::*;

#[derive(Default)]
pub struct IoFwModel{
    req: FwReq,
    sections: GeneralSections,
    common_ctl: CommonCtl,
    io_fw_ctls: Option<IofwCtls>,
}

const TIMEOUT_MS: u32 = 20;

impl CtlModel<SndDice> for IoFwModel {
    fn load(&mut self, unit: &mut SndDice, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let mut node = unit.get_node();

        self.sections = GeneralProtocol::read_general_sections(
            &mut self.req,
            &mut node,
            TIMEOUT_MS
        )?;
        let caps = GlobalSectionProtocol::read_clock_caps(
            &mut self.req,
            &mut node,
            &self.sections,
            TIMEOUT_MS
        )?;
        let src_labels = GlobalSectionProtocol::read_clock_source_labels(
            &mut self.req,
            &mut node,
            &self.sections,
            TIMEOUT_MS
        )?;
        self.common_ctl.load(card_cntr, &caps, &src_labels)?;

        IofwCtls::new(unit, &mut self.req, &self.sections, TIMEOUT_MS).and_then(|mut ctls| {
            ctls.load(card_cntr, unit, &mut self.req, TIMEOUT_MS)
                .map(|_| self.io_fw_ctls = Some(ctls))
        })?;

        Ok(())
    }

    fn read(&mut self, unit: &mut SndDice, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        if self.common_ctl.read(unit, &mut self.req, &self.sections, elem_id, elem_value, TIMEOUT_MS)? {
            Ok(true)
        } else if let Some(ctls) = &mut self.io_fw_ctls {
            ctls.read(unit, &mut self.req, elem_id, elem_value, TIMEOUT_MS)
        } else {
          Ok(false)
        }
    }

    fn write(&mut self, unit: &mut SndDice, elem_id: &ElemId, old: &ElemValue, new: &ElemValue)
        -> Result<bool, Error>
    {
        if self.common_ctl.write(unit, &mut self.req, &self.sections, elem_id, old, new, TIMEOUT_MS)? {
            Ok(true)
        } else if let Some(ctls) = &mut self.io_fw_ctls {
            ctls.write(unit, &mut self.req, elem_id, new, TIMEOUT_MS)
        } else {
          Ok(false)
        }
    }
}

impl NotifyModel<SndDice, u32> for IoFwModel {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.common_ctl.notified_elem_list);
    }

    fn parse_notification(&mut self, unit: &mut SndDice, msg: &u32) -> Result<(), Error> {
        self.common_ctl.parse_notification(unit, &mut self.req, &self.sections, *msg, TIMEOUT_MS)
    }

    fn read_notified_elem(&mut self, _: &SndDice, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        self.common_ctl.read_notified_elem(elem_id, elem_value)
    }
}

impl MeasureModel<SndDice> for IoFwModel {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.common_ctl.measured_elem_list);
        if let Some(ctls) = &self.io_fw_ctls {
            ctls.get_measure_elem_list(elem_id_list);
        }
    }

    fn measure_states(&mut self, unit: &mut SndDice) -> Result<(), Error> {
        self.common_ctl.measure_states(unit, &mut self.req, &self.sections, TIMEOUT_MS)?;
        
        if let Some(ctls) = &mut self.io_fw_ctls {
            ctls.measure_states(unit, &mut self.req, TIMEOUT_MS)?;
        }

        Ok(())
    }

    fn measure_elem(&mut self, _: &SndDice, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        if self.common_ctl.measure_elem(elem_id, elem_value)? {
            Ok(true)
        } else if let Some(ctls) = &mut self.io_fw_ctls {
            ctls.read_measured_elem(elem_id, elem_value)
        } else {
            Ok(false)
        }
    }
}

enum IofwCtls {
    Io14(Io14fwMeterCtl, Io14fwMixerCtl, Io14fwOutputCtl),
    Io26(Io26fwMeterCtl, Io26fwMixerCtl, Io26fwOutputCtl),
}

impl IofwCtls {
    fn new(
        unit: &mut SndDice,
        req: &mut FwReq,
        sections: &GeneralSections,
        timeout_ms: u32
    ) -> Result<Self, Error> {
        let mut node = unit.get_node();
        let config = GlobalSectionProtocol::read_clock_config(
            req,
            &mut node,
            sections,
            timeout_ms
        )?;
        match config.rate {
            ClockRate::R32000 |
            ClockRate::R44100 |
            ClockRate::R48000 |
            ClockRate::AnyLow => {
                let entries = TxStreamFormatSectionProtocol::read_entries(
                    req,
                    &mut node,
                    sections,
                    timeout_ms
                )?;
                if entries.len() == 2 && entries[0].pcm == 10 && entries[1].pcm == 16 {
                    Ok(Self::Io26(Default::default(), Default::default(), Default::default()))
                } else if entries.len() == 2 && entries[0].pcm == 6 && entries[1].pcm == 8 {
                    Ok(Self::Io14(Default::default(), Default::default(), Default::default()))
                } else {
                    Err(Error::new(FileError::Nxio, "Unexpected combination of stream format."))
                }
            }
            ClockRate::R88200 |
            ClockRate::R96000 |
            ClockRate::AnyMid => {
                let entries = TxStreamFormatSectionProtocol::read_entries(
                    req,
                    &mut node,
                    sections,
                    timeout_ms
                )?;
                if entries.len() == 2 && entries[0].pcm == 10 && entries[1].pcm == 4 {
                    Ok(Self::Io26(Default::default(), Default::default(), Default::default()))
                } else if entries.len() == 2 && entries[0].pcm == 6 && entries[1].pcm == 4 {
                    Ok(Self::Io14(Default::default(), Default::default(), Default::default()))
                } else {
                    Err(Error::new(FileError::Nxio, "Unexpected combination of stream format."))
                }
            }
            ClockRate::R176400 |
            ClockRate::R192000 |
            ClockRate::AnyHigh => {
                let nickname = GlobalSectionProtocol::read_nickname(
                    req,
                    &mut node,
                    sections,
                    timeout_ms
                )?;
                match nickname.as_str() {
                    "iO 26" => {
                        Ok(Self::Io26(Default::default(), Default::default(), Default::default()))
                    }
                    "iO 14" => {
                        Ok(Self::Io14(Default::default(), Default::default(), Default::default()))
                    }
                    _ => {
                        let msg = "Fail to detect type of iO model due to changed nickname";
                        Err(Error::new(FileError::Nxio, &msg))
                    }
                }
            }
            _ => Err(Error::new(FileError::Nxio, "Unexpected value of rate of sampling clock.")),
        }
    }


    fn load(
        &mut self,
        card_cntr: &mut CardCntr,
        unit: &mut SndDice,
        req: &mut FwReq,
        timeout_ms: u32
    ) -> Result<(), Error> {
        match self {
            Self::Io14(meter_ctl, mixer_ctl, output_ctl) => {
                meter_ctl.load(card_cntr, unit, req, timeout_ms)
                    .map(|mut elem_id_list| meter_ctl.1.append(&mut elem_id_list))?;
                mixer_ctl.load(card_cntr, unit, req, timeout_ms)
                    .map(|mut elem_id_list| mixer_ctl.1.append(&mut elem_id_list))?;
                output_ctl.load(card_cntr)
            }
            Self::Io26(meter_ctl, mixer_ctl, output_ctl) => {
                meter_ctl.load(card_cntr, unit, req, timeout_ms)
                    .map(|mut elem_id_list| meter_ctl.1.append(&mut elem_id_list))?;
                mixer_ctl.load(card_cntr, unit, req, timeout_ms)
                    .map(|mut elem_id_list| mixer_ctl.1.append(&mut elem_id_list))?;
                output_ctl.load(card_cntr)
            }
        }
    }

    fn read(
        &self,
        unit: &mut SndDice,
        req: &mut FwReq,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
        timeout_ms: u32
    ) -> Result<bool, Error> {
        match self {
            Self::Io14(_, mixer_ctl, output_ctl) => {
                if mixer_ctl.read(elem_id, elem_value)? {
                    Ok(true)
                } else if output_ctl.read(unit, req, elem_id, elem_value, timeout_ms)? {
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            Self::Io26(_, mixer_ctl, output_ctl) => {
                if mixer_ctl.read(elem_id, elem_value)? {
                    Ok(true)
                } else if output_ctl.read(unit, req, elem_id, elem_value, timeout_ms)? {
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
        }
    }

    fn write(
        &mut self,
        unit: &mut SndDice,
        req: &mut FwReq,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32
    ) -> Result<bool, Error> {
        match self {
            Self::Io14(_, mixer_ctl, output_ctl) => {
                if mixer_ctl.write(unit, req, elem_id, elem_value, timeout_ms)? {
                    Ok(true)
                } else if output_ctl.write(unit, req, elem_id, elem_value, timeout_ms)? {
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            Self::Io26(_, mixer_ctl, output_ctl) => {
                if mixer_ctl.write(unit, req, elem_id, elem_value, timeout_ms)? {
                    Ok(true)
                } else if output_ctl.write(unit, req, elem_id, elem_value, timeout_ms)? {
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
        }
    }

    fn get_measure_elem_list(&self, elem_id_list: &mut Vec<ElemId>) {
        match self {
            Self::Io14(meter_ctl, mixer_ctl, _) => {
                elem_id_list.extend_from_slice(&meter_ctl.1);
                elem_id_list.extend_from_slice(&mixer_ctl.1);
            }
            Self::Io26(meter_ctl, mixer_ctl, _) => {
                elem_id_list.extend_from_slice(&meter_ctl.1);
                elem_id_list.extend_from_slice(&mixer_ctl.1);
            }
        }
    }

    fn measure_states(
        &mut self,
        unit: &mut SndDice,
        req: &mut FwReq,
        timeout_ms: u32
    ) -> Result<(), Error> {
        match self {
            Self::Io14(meter_ctl, mixer_ctl, _) => {
                meter_ctl.measure_states(unit, req, timeout_ms)?;
                mixer_ctl.measure_states(unit, req, timeout_ms)
            }
            Self::Io26(meter_ctl, mixer_ctl, _) => {
                meter_ctl.measure_states(unit, req, timeout_ms)?;
                mixer_ctl.measure_states(unit, req, timeout_ms)
            }
        }
    }

    fn read_measured_elem(
        &self,
        elem_id: &ElemId,
        elem_value: &mut ElemValue
    ) -> Result<bool, Error> {
        match self {
            Self::Io14(meter_ctl, mixer_ctl, _) => {
                if meter_ctl.read_measured_elem(elem_id, elem_value)? {
                    Ok(true)
                } else if mixer_ctl.read_measured_elem(elem_id, elem_value)? {
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            Self::Io26(meter_ctl, mixer_ctl, _) => {
                if meter_ctl.read_measured_elem(elem_id, elem_value)? {
                    Ok(true)
                } else if mixer_ctl.read_measured_elem(elem_id, elem_value)? {
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
        }
    }
}

#[derive(Default)]
struct Io14fwMeterCtl(IofwMeterState, Vec<ElemId>);

impl MeterCtlOperation<Io14fwProtocol> for Io14fwMeterCtl {
    fn meter(&self) -> &IofwMeterState {
        &self.0
    }

    fn meter_mut(&mut self) -> &mut IofwMeterState {
        &mut self.0
    }
}

#[derive(Default)]
struct Io26fwMeterCtl(IofwMeterState, Vec<ElemId>);

impl MeterCtlOperation<Io26fwProtocol> for Io26fwMeterCtl {
    fn meter(&self) -> &IofwMeterState {
        &self.0
    }

    fn meter_mut(&mut self) -> &mut IofwMeterState {
        &mut self.0
    }
}

const ANALOG_INPUT_METER_NAME: &str = "analog-input-meters";
const DIGITAL_A_INPUT_METER_NAME: &str = "digital-a-input-meters";
const DIGITAL_B_INPUT_METER_NAME: &str = "digital-b-input-meters";
const MIXER_OUT_METER_NAME: &str = "mixer-output-meters";

trait MeterCtlOperation<T: IofwMeterOperation> {
    const LEVEL_TLV: DbInterval = DbInterval{min: -9000, max: 0, linear: false, mute_avail: false};

    fn meter(&self) -> &IofwMeterState;
    fn meter_mut(&mut self) -> &mut IofwMeterState;

    fn load(
        &mut self,
        card_cntr: &mut CardCntr,
        unit: &mut SndDice,
        req: &mut FwReq,
        timeout_ms: u32
    ) -> Result<Vec<ElemId>, Error> {
        let mut state = T::create_meter_state();
        T::read_meter(req, &mut unit.get_node(), &mut state, timeout_ms)?;
        *self.meter_mut() = state;

        let mut measured_elem_id_list = Vec::new();

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, ANALOG_INPUT_METER_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                T::LEVEL_MIN,
                T::LEVEL_MAX,
                T::LEVEL_STEP,
                T::ANALOG_INPUT_COUNT,
                Some(&Vec::<u32>::from(Self::LEVEL_TLV)),
                false
            )
                .map(|mut elem_id_list| measured_elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, DIGITAL_A_INPUT_METER_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                T::LEVEL_MIN,
                T::LEVEL_MAX,
                T::LEVEL_STEP,
                T::DIGITAL_A_INPUT_COUNT,
                Some(&Vec::<u32>::from(Self::LEVEL_TLV)),
                false
            )
                .map(|mut elem_id_list| measured_elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, DIGITAL_B_INPUT_METER_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                T::LEVEL_MIN,
                T::LEVEL_MAX,
                T::LEVEL_STEP,
                T::DIGITAL_B_INPUT_COUNT,
                Some(&Vec::<u32>::from(Self::LEVEL_TLV)),
                false
            )
                .map(|mut elem_id_list| measured_elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIXER_OUT_METER_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                T::LEVEL_MIN,
                T::LEVEL_MAX,
                T::LEVEL_STEP,
                T::MIXER_COUNT,
                Some(&Vec::<u32>::from(Self::LEVEL_TLV)),
                false
            )
                .map(|mut elem_id_list| measured_elem_id_list.append(&mut elem_id_list))?;

        Ok(measured_elem_id_list)
    }

    fn measure_states(
        &mut self,
        unit: &mut SndDice,
        req: &mut FwReq,
        timeout_ms: u32
    ) -> Result<(), Error> {
        T::read_meter(req, &mut unit.get_node(), self.meter_mut(), timeout_ms)
    }

    fn read_measured_elem(
        &self,
        elem_id: &ElemId,
        elem_value: &mut ElemValue
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            ANALOG_INPUT_METER_NAME => {
                elem_value.set_int(&self.meter().analog_inputs);
                Ok(true)
            }
            DIGITAL_A_INPUT_METER_NAME => {
                elem_value.set_int(&self.meter().digital_a_inputs);
                Ok(true)
            }
            DIGITAL_B_INPUT_METER_NAME => {
                elem_value.set_int(&self.meter().digital_b_inputs);
                Ok(true)
            }
            MIXER_OUT_METER_NAME => {
                elem_value.set_int(&self.meter().mixer_outputs);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

#[derive(Default)]
struct Io14fwMixerCtl(IofwMixerState, Vec<ElemId>);

impl MixerCtlOperation<Io14fwProtocol> for Io14fwMixerCtl {
    fn state(&self) -> &IofwMixerState {
        &self.0
    }

    fn state_mut(&mut self) -> &mut IofwMixerState {
        &mut self.0
    }
}

#[derive(Default)]
struct Io26fwMixerCtl(IofwMixerState, Vec<ElemId>);

impl MixerCtlOperation<Io26fwProtocol> for Io26fwMixerCtl {
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

trait MixerCtlOperation<T: IofwMixerOperation> {
    fn state(&self) -> &IofwMixerState;
    fn state_mut(&mut self) -> &mut IofwMixerState;

    const LEVEL_MIN: i32 = 0;
    const LEVEL_MAX: i32 = 0x007fff00;
    const LEVEL_STEP: i32 = 0x100;
    const LEVEL_TLV: DbInterval = DbInterval{min: -9000, max: 0, linear: false, mute_avail: false};

    const KNOB_MIN: i32 = 0;
    const KNOB_MAX: i32 = 0x100;
    const KNOB_STEP: i32 = 1;

    fn load(
        &mut self,
        card_cntr: &mut CardCntr,
        unit: &mut SndDice,
        req: &mut FwReq,
        timeout_ms: u32
    ) -> Result<Vec<ElemId>, Error> {
        let mut state = T::create_mixer_state();
        let mut node = unit.get_node();
        T::read_mixer_src_gains(req, &mut node, &mut state, timeout_ms)?;
        T::read_mixer_src_mutes(req, &mut node, &mut state, timeout_ms)?;
        T::read_mixer_out_vols(req, &mut node, &mut state, timeout_ms)?;
        T::read_mixer_out_mutes(req, &mut node, &mut state, timeout_ms)?;
        *self.state_mut() = state;

        let mut measured_elem_id_list = Vec::new();

        let count = T::ANALOG_INPUT_COUNT + T::DIGITAL_A_INPUT_COUNT + T::DIGITAL_B_INPUT_COUNT;
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, INPUT_GAIN_NAME, 0);
        let _ = card_cntr
            .add_int_elems(
                &elem_id,
                T::MIXER_COUNT,
                Self::LEVEL_MIN,
                Self::LEVEL_MAX,
                Self::LEVEL_STEP,
                count,
                Some(&Into::<Vec<u32>>::into(Self::LEVEL_TLV)),
                true
            )?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, INPUT_MUTE_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, T::MIXER_PAIR_COUNT, count, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, STREAM_GAIN_NAME, 0);
        let _ = card_cntr
            .add_int_elems(
                &elem_id,
                T::MIXER_COUNT,
                Self::LEVEL_MIN,
                Self::LEVEL_MAX,
                Self::LEVEL_STEP,
                T::STREAM_INPUT_COUNT,
                Some(&Into::<Vec<u32>>::into(Self::LEVEL_TLV)),
                true
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
                true
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
                false
            )
                .map(|mut elem_id_list| measured_elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, MAIN_LEVEL_KNOB_NAME, 0);
        card_cntr.add_int_elems(
            &elem_id,
            1,
            Self::KNOB_MIN,
            Self::KNOB_MAX,
            Self::KNOB_STEP,
            1,
            None,
            false
        )
            .map(|mut elem_id_list| measured_elem_id_list.append(&mut elem_id_list))?;

        Ok(measured_elem_id_list)
    }

    fn read(
        &self,
        elem_id: &ElemId,
        elem_value: &mut ElemValue
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            INPUT_GAIN_NAME => {
                let mixer = elem_id.get_index() as usize;
                let gains = &self.state().gains[mixer];
                let mut vals = Vec::new();
                vals.extend_from_slice(&gains.analog_inputs);
                vals.extend_from_slice(&gains.digital_a_inputs);
                vals.extend_from_slice(&gains.digital_b_inputs);
                elem_value.set_int(&vals);
                Ok(true)
            }
            INPUT_MUTE_NAME => {
                let mixer = elem_id.get_index() as usize;
                let mutes = &self.state().mutes[mixer];
                let mut vals = Vec::new();
                vals.extend_from_slice(&mutes.analog_inputs);
                vals.extend_from_slice(&mutes.digital_a_inputs);
                vals.extend_from_slice(&mutes.digital_b_inputs);
                elem_value.set_bool(&vals);
                Ok(true)
            }
            STREAM_GAIN_NAME => {
                let mixer = elem_id.get_index() as usize;
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
        unit: &mut SndDice,
        req: &mut FwReq,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            INPUT_GAIN_NAME => {
                let mixer = elem_id.get_index() as usize;
                let mut gains = self.state().gains[mixer].clone();

                let analog_input_count = gains.analog_inputs.len();
                let digital_a_input_count = gains.digital_a_inputs.len();
                let digital_b_input_count = gains.digital_b_inputs.len();
                let mut vals = vec![0; analog_input_count + digital_a_input_count + digital_b_input_count];
                elem_value.get_int(&mut vals);

                let analog_inputs = &vals[..analog_input_count];
                let digital_a_inputs = &vals[analog_input_count..(analog_input_count + digital_a_input_count)];
                let digital_b_inputs = &vals[(analog_input_count + digital_a_input_count)..];

                gains.analog_inputs.copy_from_slice(&analog_inputs);
                gains.digital_a_inputs.copy_from_slice(&digital_a_inputs);
                gains.digital_b_inputs.copy_from_slice(&digital_b_inputs);

                T::write_mixer_src_gains(
                    req,
                    &mut unit.get_node(),
                    mixer,
                    &gains,
                    self.state_mut(),
                    timeout_ms
                )
                    .map(|_| true)
            }
            INPUT_MUTE_NAME => {
                let mixer = elem_id.get_index() as usize;
                let mut mutes = self.state().mutes[mixer].clone();

                let analog_input_count = mutes.analog_inputs.len();
                let digital_a_input_count = mutes.digital_a_inputs.len();
                let digital_b_input_count = mutes.digital_b_inputs.len();
                let mut vals = vec![false; analog_input_count + digital_a_input_count + digital_b_input_count];
                elem_value.get_bool(&mut vals);

                let analog_inputs = &vals[..analog_input_count];
                let digital_a_inputs = &vals[analog_input_count..(analog_input_count + digital_a_input_count)];
                let digital_b_inputs = &vals[(analog_input_count + digital_a_input_count)..];

                mutes.analog_inputs.copy_from_slice(&analog_inputs);
                mutes.digital_a_inputs.copy_from_slice(&digital_a_inputs);
                mutes.digital_b_inputs.copy_from_slice(&digital_b_inputs);

                T::write_mixer_src_mutes(
                    req,
                    &mut unit.get_node(),
                    mixer,
                    &mutes,
                    self.state_mut(),
                    timeout_ms
                )
                    .map(|_| true)
            }
            STREAM_GAIN_NAME => {
                let mixer = elem_id.get_index() as usize;
                let mut gains = self.state().gains[mixer].clone();

                elem_value.get_int(&mut gains.stream_inputs);

                T::write_mixer_src_gains(
                    req,
                    &mut unit.get_node(),
                    mixer,
                    &gains,
                    self.state_mut(),
                    timeout_ms
                )
                    .map(|_| true)
            }
            OUTPUT_VOL_NAME => {
                let mut vals = self.state().out_vols.clone();
                elem_value.get_int(&mut vals);
                T::write_mixer_out_vols(
                    req,
                    &mut unit.get_node(),
                    &vals,
                    self.state_mut(),
                    timeout_ms
                )
                    .map(|_| true)
            }
            OUTPUT_MUTE_NAME => {
                let mut vals = self.state().out_mutes.clone();
                elem_value.get_bool(&mut vals);
                T::write_mixer_out_mutes(
                    req,
                    &mut unit.get_node(),
                    &vals,
                    self.state_mut(),
                    timeout_ms
                )
                    .map(|_| true)
            }
            _ => Ok(false),
        }
    }

    fn measure_states(
        &mut self,
        unit: &mut SndDice,
        req: &mut FwReq,
        timeout_ms: u32
    ) -> Result<(), Error> {
        let mut node = unit.get_node();

        let old = self.state().knobs.mix_blend as i32;
        T::read_knob_state(req, &mut node, self.state_mut(), timeout_ms)?;

        let new = self.state().knobs.mix_blend as i32;
        if new != old {
            // NOTE: The calculation is done within 32 bit storage without overflow.
            let val = Self::LEVEL_MAX * new / Self::KNOB_MAX;
            let mut new = self.state().out_vols.clone();
            new[0] = val;
            new[1] = val;
            T::write_mixer_out_vols(req, &mut node, &new, self.state_mut(), timeout_ms)?;
        }

        Ok(())
    }

    fn read_measured_elem(
        &self,
        elem_id: &ElemId,
        elem_value: &ElemValue
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
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

#[derive(Default)]
struct Io14fwOutputCtl;

impl OutputCtlOperation<Io14fwProtocol> for Io14fwOutputCtl {}

#[derive(Default)]
struct Io26fwOutputCtl;

impl OutputCtlOperation<Io26fwProtocol> for Io26fwOutputCtl {}

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

trait OutputCtlOperation<T: IofwOutputOperation> {
    const OUT_LEVELS: [NominalSignalLevel; 2] = [
        NominalSignalLevel::Consumer,
        NominalSignalLevel::Professional,
    ];

    const DIGITAL_B_67_SRCS: [DigitalB67Src; 2] = [
        DigitalB67Src::Spdif12,
        DigitalB67Src::Adat67,
    ];

    const MIXER_OUT_PAIRS: [MixerOutPair; 4] = [
        MixerOutPair::Mixer01,
        MixerOutPair::Mixer23,
        MixerOutPair::Mixer45,
        MixerOutPair::Mixer67,
    ];

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let labels: Vec<&str> = Self::OUT_LEVELS.iter()
            .map(|l| nominal_signal_level_to_str(l))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, OUT_LEVEL_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, T::ANALOG_OUTPUT_COUNT, &labels, None, true)?;

        if T::HAS_OPT_IFACE_B {
            let labels: Vec<&str> = Self::DIGITAL_B_67_SRCS.iter()
                .map(|s| digital_b_67_src_to_str(s))
                .collect();
            let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, DIGITAL_B_67_SRC_NAME, 0);
            let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;
        }

        let labels: Vec<&str> = Self::MIXER_OUT_PAIRS.iter()
            .map(|p| mixer_out_pair_to_str(p))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, SPDIF_OUT_SRC_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, HP23_SRC_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

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
            OUT_LEVEL_NAME => {
                let mut levels = vec![NominalSignalLevel::default(); T::ANALOG_OUTPUT_COUNT];
                T::read_out_levels(req, &mut unit.get_node(), &mut levels, timeout_ms)?;
                let vals: Vec<u32> = levels.iter()
                    .map(|level| Self::OUT_LEVELS.iter().position(|l| l.eq(level)).unwrap() as u32)
                    .collect();
                elem_value.set_enum(&vals);
                Ok(true)
            }
            DIGITAL_B_67_SRC_NAME => {
                let mut src = DigitalB67Src::default();
                T::read_mixer_digital_b_67_src(req, &mut unit.get_node(), &mut src, timeout_ms)?;
                let pos = Self::DIGITAL_B_67_SRCS.iter().position(|s| s.eq(&src)).unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            SPDIF_OUT_SRC_NAME => {
                let mut pair = MixerOutPair::default();
                T::read_spdif_out_src(req, &mut unit.get_node(), &mut pair, timeout_ms)?;
                let pos = Self::MIXER_OUT_PAIRS.iter().position(|p| p.eq(&pair)).unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            HP23_SRC_NAME => {
                let mut pair = MixerOutPair::default();
                T::read_hp23_out_src(req, &mut unit.get_node(), &mut pair, timeout_ms)?;
                let pos = Self::MIXER_OUT_PAIRS.iter().position(|p| p.eq(&pair)).unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write(
        &self,
        unit: &mut SndDice,
        req: &mut FwReq,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            OUT_LEVEL_NAME => {
                let mut vals = vec![0; T::ANALOG_OUTPUT_COUNT];
                elem_value.get_enum(&mut vals);
                let levels: Vec<NominalSignalLevel> = vals.iter()
                    .map(|v| NominalSignalLevel::from(*v))
                    .collect();
                T::write_out_levels(req, &mut unit.get_node(), &levels, timeout_ms)
                    .map(|_| true)
            }
            DIGITAL_B_67_SRC_NAME => {
                ElemValueAccessor::<u32>::get_val(elem_value, |val| {
                    let src = Self::DIGITAL_B_67_SRCS.iter().nth(val as usize).ok_or_else(|| {
                        let msg = format!("Invalid index of source of digital B 7/8: {}",
                                          val);
                        Error::new(FileError::Inval, &msg)
                    })?;
                    T::write_mixer_digital_b_67_src(req, &mut unit.get_node(), src, timeout_ms)
                })
                    .map(|_| true)
            }
            SPDIF_OUT_SRC_NAME => {
                ElemValueAccessor::<u32>::get_val(elem_value, |val| {
                    let src = Self::MIXER_OUT_PAIRS.iter().nth(val as usize).ok_or_else(|| {
                        let msg = format!("Invalid index of pair of mixer output: {}",
                                          val);
                        Error::new(FileError::Inval, &msg)
                    })?;
                    T::write_spdif_out_src(req, &mut unit.get_node(), src, timeout_ms)
                })
                    .map(|_| true)
            }
            HP23_SRC_NAME => {
                ElemValueAccessor::<u32>::get_val(elem_value, |val| {
                    let src = Self::MIXER_OUT_PAIRS.iter().nth(val as usize).ok_or_else(|| {
                        let msg = format!("Invalid index of pair of mixer output: {}",
                                          val);
                        Error::new(FileError::Inval, &msg)
                    })?;
                    T::write_hp23_out_src(req, &mut unit.get_node(), src, timeout_ms)
                })
                    .map(|_| true)
            }
            _ => Ok(false),
        }
    }
}
