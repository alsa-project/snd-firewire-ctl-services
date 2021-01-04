// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use alsactl::{ElemId, ElemIfaceType, ElemValue, ElemValueExt};

use hinawa::FwReq;
use hinawa::{SndDice, SndUnitExt};

use core::card_cntr::*;

use alsa_ctl_tlv_codec::items::DbInterval;

use dice_protocols::tcat::{*, global_section::*};
use dice_protocols::tcelectronic::{*, desktop::*};

use crate::common_ctl::*;

#[derive(Default)]
pub struct Desktopk6Model{
    proto: Desktopk6Proto,
    sections: GeneralSections,
    segments: DesktopSegments,
    ctl: CommonCtl,
    meter_ctl: MeterCtl,
}

const TIMEOUT_MS: u32 = 20;

impl CtlModel<SndDice> for Desktopk6Model {
    fn load(&mut self, unit: &SndDice, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let node = unit.get_node();

        self.sections = self.proto.read_general_sections(&node, TIMEOUT_MS)?;
        let caps = self.proto.read_clock_caps(&node, &self.sections, TIMEOUT_MS)?;
        let src_labels = self.proto.read_clock_source_labels(&node, &self.sections, TIMEOUT_MS)?;
        self.ctl.load(card_cntr, &caps, &src_labels)?;

        self.proto.read_segment(&node, &mut self.segments.meter, TIMEOUT_MS)?;

        self.meter_ctl.load(&self.segments, card_cntr)?;

        Ok(())
    }

    fn read(&mut self, unit: &SndDice, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        if self.ctl.read(unit, &self.proto, &self.sections, elem_id, elem_value, TIMEOUT_MS)? {
            Ok(true)
        } else if self.meter_ctl.read(&self.segments, elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(&mut self, unit: &SndDice, elem_id: &ElemId, old: &ElemValue, new: &ElemValue)
        -> Result<bool, Error>
    {
        self.ctl.write(unit, &self.proto, &self.sections, elem_id, old, new, TIMEOUT_MS)
    }
}

impl NotifyModel<SndDice, u32> for Desktopk6Model {
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

impl MeasureModel<hinawa::SndDice> for Desktopk6Model {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.ctl.measured_elem_list);
        elem_id_list.extend_from_slice(&self.meter_ctl.0);
    }

    fn measure_states(&mut self, unit: &SndDice) -> Result<(), Error> {
        self.ctl.measure_states(unit, &self.proto, &self.sections, TIMEOUT_MS)?;

        self.proto.read_segment(&unit.get_node(), &mut self.segments.meter, TIMEOUT_MS)?;

        Ok(())
    }

    fn measure_elem(&mut self, _: &SndDice, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        if self.ctl.measure_elem(elem_id, elem_value)? {
            Ok(true)
        } else if self.meter_ctl.read(&self.segments, elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}


#[derive(Default, Debug)]
struct Desktopk6Proto(FwReq);

impl AsRef<FwReq> for Desktopk6Proto {
    fn as_ref(&self) -> &FwReq {
        &self.0
    }
}

#[derive(Default, Debug)]
pub struct MeterCtl(Vec<ElemId>);

impl<'a> MeterCtl {
    const ANALOG_IN_NAME: &'a str = "analog-input-meters";
    const MIXER_OUT_NAME: &'a str = "mixer-output-meters";
    const STREAM_IN_NAME: &'a str = "stream-input-meters";

    const METER_MIN: i32 = -1000;
    const METER_MAX: i32 = 0;
    const METER_STEP: i32 = 1;
    const METER_TLV: DbInterval = DbInterval{min: -9400, max: 0, linear: false, mute_avail: false};

    fn load(&mut self, segments: &DesktopSegments, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let labels = (0..segments.meter.data.analog_inputs.len())
            .map(|i| format!("Analog-input-{}", i))
            .collect::<Vec<_>>();
        self.add_meter_elem(card_cntr, Self::ANALOG_IN_NAME, &labels)?;

        let labels = (0..segments.meter.data.mixer_outputs.len())
            .map(|i| format!("Mixer-output-{}", i))
            .collect::<Vec<_>>();
        self.add_meter_elem(card_cntr, Self::MIXER_OUT_NAME, &labels)?;

        let labels = (0..segments.meter.data.stream_inputs.len())
            .map(|i| format!("Stream-input-{}", i))
            .collect::<Vec<_>>();
        self.add_meter_elem(card_cntr, Self::STREAM_IN_NAME, &labels)?;

        Ok(())
    }

    fn add_meter_elem<T: AsRef<str>>(&mut self, card_cntr: &mut CardCntr, name: &str, labels: &[T])
        -> Result<(), Error>
    {
        let elem_id = alsactl::ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
        card_cntr.add_int_elems(&elem_id, 1, Self::METER_MIN, Self::METER_MAX, Self::METER_STEP,
                                labels.len(), Some(&Into::<Vec<u32>>::into(Self::METER_TLV)), false)
            .map(|mut elem_id_list| self.0.append(&mut elem_id_list))
    }

    fn read(&self, segments: &DesktopSegments, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::ANALOG_IN_NAME => {
                elem_value.set_int(&segments.meter.data.analog_inputs);
                Ok(true)
            }
            Self::MIXER_OUT_NAME => {
                elem_value.set_int(&segments.meter.data.mixer_outputs);
                Ok(true)
            }
            Self::STREAM_IN_NAME => {
                elem_value.set_int(&segments.meter.data.stream_inputs);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
