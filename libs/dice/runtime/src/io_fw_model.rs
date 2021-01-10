// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::{Error, FileError};

use alsactl::{ElemId, ElemIfaceType, ElemValue, ElemValueExt};

use hinawa::{FwNode, FwReq, SndDice, SndUnitExt};

use alsa_ctl_tlv_codec::items::DbInterval;

use core::card_cntr::*;

use dice_protocols::tcat::{*, global_section::*, tx_stream_format_section::*};
use dice_protocols::alesis::meter::*;

use crate::common_ctl::*;

#[derive(Default)]
pub struct IoFwModel{
    proto: FwReq,
    sections: GeneralSections,
    ctl: CommonCtl,
    state: AlesisIoFwState,
    meter_ctl: MeterCtl,
}

const TIMEOUT_MS: u32 = 20;

impl CtlModel<SndDice> for IoFwModel {
    fn load(&mut self, unit: &SndDice, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let node = unit.get_node();

        self.sections = self.proto.read_general_sections(&node, TIMEOUT_MS)?;
        let caps = self.proto.read_clock_caps(&node, &self.sections, TIMEOUT_MS)?;
        let src_labels = self.proto.read_clock_source_labels(&node, &self.sections, TIMEOUT_MS)?;
        self.ctl.load(card_cntr, &caps, &src_labels)?;

        self.state = AlesisIoFwState::new(&node, &self.proto, &self.sections, TIMEOUT_MS)?;
        self.meter_ctl.load(card_cntr, unit, &self.proto, &mut self.state, TIMEOUT_MS)?;

        Ok(())
    }

    fn read(&mut self, unit: &SndDice, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        self.ctl.read(unit, &self.proto, &self.sections, elem_id, elem_value, TIMEOUT_MS)
    }

    fn write(&mut self, unit: &SndDice, elem_id: &ElemId, old: &ElemValue, new: &ElemValue)
        -> Result<bool, Error>
    {
        self.ctl.write(unit, &self.proto, &self.sections, elem_id, old, new, TIMEOUT_MS)
    }
}

impl NotifyModel<SndDice, u32> for IoFwModel {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.ctl.notified_elem_list);
    }

    fn parse_notification(&mut self, unit: &SndDice, msg: &u32) -> Result<(), Error> {
        self.ctl.parse_notification(unit, &self.proto, &self.sections, *msg, TIMEOUT_MS)
    }

    fn read_notified_elem(&mut self, _: &SndDice, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        self.ctl.read_notified_elem(elem_id, elem_value)
    }
}

impl MeasureModel<hinawa::SndDice> for IoFwModel {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.ctl.measured_elem_list);
        elem_id_list.extend_from_slice(&self.meter_ctl.0);
    }

    fn measure_states(&mut self, unit: &SndDice) -> Result<(), Error> {
        self.ctl.measure_states(unit, &self.proto, &self.sections, TIMEOUT_MS)?;
        self.meter_ctl.measure_states(unit, &self.proto, &mut self.state, TIMEOUT_MS)?;
        Ok(())
    }

    fn measure_elem(&mut self, _: &SndDice, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        if self.ctl.measure_elem(elem_id, elem_value)? {
            Ok(true)
        } else if self.meter_ctl.read_measured_elem(&self.state, elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[derive(Debug)]
enum AlesisIoFwState{
    Io14(IoFwState<Io14Meter>),
    Io26(IoFwState<Io26Meter>),
}

impl Default for AlesisIoFwState {
    fn default() -> Self {
        Self::Io14(Default::default())
    }
}

impl AlesisIoFwState {
    fn new(node: &FwNode, proto: &FwReq, sections: &GeneralSections, timeout_ms: u32)
        -> Result<Self, Error>
    {
        let config = proto.read_clock_config(node, sections, timeout_ms)?;
        match config.rate {
            ClockRate::R32000 |
            ClockRate::R44100 |
            ClockRate::R48000 |
            ClockRate::AnyLow => {
                let entries = proto.read_tx_stream_format_entries(node, sections, timeout_ms)?;
                if entries.len() == 2 && entries[0].pcm == 10 && entries[1].pcm == 16 {
                    Ok(Self::Io26(Default::default()))
                } else if entries.len() == 2 && entries[0].pcm == 6 && entries[1].pcm == 8 {
                    Ok(Self::Io14(Default::default()))
                } else {
                    Err(Error::new(FileError::Nxio, "Unexpected combination of stream format."))
                }
            }
            ClockRate::R88200 |
            ClockRate::R96000 |
            ClockRate::AnyMid => {
                let entries = proto.read_tx_stream_format_entries(node, sections, timeout_ms)?;
                if entries.len() == 2 && entries[0].pcm == 10 && entries[1].pcm == 4 {
                    Ok(Self::Io26(Default::default()))
                } else if entries.len() == 2 && entries[0].pcm == 6 && entries[1].pcm == 4 {
                    Ok(Self::Io14(Default::default()))
                } else {
                    Err(Error::new(FileError::Nxio, "Unexpected combination of stream format."))
                }
            }
            ClockRate::R176400 |
            ClockRate::R192000 |
            ClockRate::AnyHigh => {
                let nickname = proto.read_nickname(node, sections, timeout_ms)?;
                match nickname.as_str() {
                    "iO 26" => Ok(Self::Io26(Default::default())),
                    "iO 14" => Ok(Self::Io14(Default::default())),
                    _ => Err(Error::new(FileError::Nxio, "Fail to detect type of iO model due to changed nickname")),
                }
            }
            _ => Err(Error::new(FileError::Nxio, "Unexpected value of rate of sampling clock.")),
        }
    }
}

impl AsRef<IoMeter> for AlesisIoFwState {
    fn as_ref(&self) -> &IoMeter {
        match self {
            Self::Io14(s) => s.meter.as_ref(),
            Self::Io26(s) => s.meter.as_ref(),
        }
    }
}

impl AsMut<IoMeter> for AlesisIoFwState {
    fn as_mut(&mut self) -> &mut IoMeter {
        match self {
            Self::Io14(s) => s.meter.as_mut(),
            Self::Io26(s) => s.meter.as_mut(),
        }
    }
}

#[derive(Default, Debug)]
struct IoFwState<M>
    where M: Default + AsRef<IoMeter> + AsMut<IoMeter>,
{
    meter: M,
}

#[derive(Default, Debug)]
struct MeterCtl(Vec<ElemId>);

impl<'a> MeterCtl {
    const ANALOG_INPUT_METER_NAME: &'a str = "analog-input-meters";
    const DIGITAL_A_INPUT_METER_NAME: &'a str = "digital-a-input-meters";
    const DIGITAL_B_INPUT_METER_NAME: &'a str = "digital-b-input-meters";
    const MIXER_OUT_METER_NAME: &'a str = "mixer-output-meters";

    const LEVEL_MIN: i32 = 0;
    const LEVEL_MAX: i32 = 0x007fff00;
    const LEVEL_STEP: i32 = 0x100;
    const LEVEL_TLV: DbInterval = DbInterval{min: -9000, max: 0, linear: false, mute_avail: false};

    fn load(&mut self, card_cntr: &mut CardCntr, unit: &SndDice, proto: &FwReq, state: &mut AlesisIoFwState,
            timeout_ms: u32)
        -> Result<(), Error>
    {
        proto.read_meter(&unit.get_node(), state, timeout_ms)?;

        let m = AsRef::<IoMeter>::as_ref(state);

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::ANALOG_INPUT_METER_NAME, 0);
        card_cntr.add_int_elems(&elem_id, 1, Self::LEVEL_MIN, Self::LEVEL_MAX, Self::LEVEL_STEP,
                                m.analog_inputs.len(), Some(&Vec::<u32>::from(Self::LEVEL_TLV)), false)
            .map(|mut elem_id_list| self.0.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::DIGITAL_A_INPUT_METER_NAME, 0);
        card_cntr.add_int_elems(&elem_id, 1, Self::LEVEL_MIN, Self::LEVEL_MAX, Self::LEVEL_STEP,
                                m.digital_a_inputs.len(), Some(&Vec::<u32>::from(Self::LEVEL_TLV)), false)
            .map(|mut elem_id_list| self.0.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::DIGITAL_B_INPUT_METER_NAME, 0);
        card_cntr.add_int_elems(&elem_id, 1, Self::LEVEL_MIN, Self::LEVEL_MAX, Self::LEVEL_STEP,
                                m.digital_b_inputs.len(), Some(&Vec::<u32>::from(Self::LEVEL_TLV)), false)
            .map(|mut elem_id_list| self.0.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::MIXER_OUT_METER_NAME, 0);
        card_cntr.add_int_elems(&elem_id, 1, Self::LEVEL_MIN, Self::LEVEL_MAX, Self::LEVEL_STEP,
                                m.mixer_outputs.len(), Some(&Vec::<u32>::from(Self::LEVEL_TLV)), false)
            .map(|mut elem_id_list| self.0.append(&mut elem_id_list))?;

        Ok(())
    }

    fn measure_states(&mut self, unit: &SndDice, proto: &FwReq, state: &mut AlesisIoFwState, timeout_ms: u32)
        -> Result<(), Error>
    {
        proto.read_meter(&unit.get_node(), state, timeout_ms)
    }

    fn read_measured_elem(&self, state: &AlesisIoFwState, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::ANALOG_INPUT_METER_NAME => {
                let m = AsRef::<IoMeter>::as_ref(state);
                elem_value.set_int(&m.analog_inputs);
                Ok(true)
            }
            Self::DIGITAL_A_INPUT_METER_NAME => {
                let m = AsRef::<IoMeter>::as_ref(state);
                elem_value.set_int(&m.digital_a_inputs);
                Ok(true)
            }
            Self::DIGITAL_B_INPUT_METER_NAME => {
                let m = AsRef::<IoMeter>::as_ref(state);
                elem_value.set_int(&m.digital_b_inputs);
                Ok(true)
            }
            Self::MIXER_OUT_METER_NAME => {
                let m = AsRef::<IoMeter>::as_ref(state);
                elem_value.set_int(&m.mixer_outputs);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
