// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto
use glib::Error;

use alsactl::{ElemId, ElemIfaceType, ElemValue, ElemValueExt};

use hinawa::{SndDice, SndUnitExt};

use alsa_ctl_tlv_codec::items::DbInterval;

use core::card_cntr::*;

use dice_protocols::tcat::*;
use dice_protocols::tcat::global_section::*;
use dice_protocols::presonus::fstudio::*;

use crate::common_ctl::*;

#[derive(Default)]
pub struct FStudioModel{
    proto: FStudioProto,
    sections: GeneralSections,
    ctl: CommonCtl,
    meter_ctl: MeterCtl,
}

const TIMEOUT_MS: u32 = 20;

// MEMO: the device returns 'SPDIF\ADAT\Word Clock\Unused\Unused\Unused\Unused\Internal\\'.
const AVAIL_CLK_SRC_LABELS: [&str;13] = [
    "S/PDIF",
    "Unused",
    "Unused",
    "Unused",
    "Unused",
    "ADAT",
    "Unused",
    "WordClock",
    "Unused",
    "Unused",
    "Unused",
    "Unused",
    "Internal",
];

impl CtlModel<SndDice> for FStudioModel {
    fn load(&mut self, unit: &SndDice, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let node = unit.get_node();

        self.sections = self.proto.read_general_sections(&node, TIMEOUT_MS)?;
        let caps = self.proto.read_clock_caps(&node, &self.sections, TIMEOUT_MS)?;
        let entries: Vec<_> = AVAIL_CLK_SRC_LABELS.iter()
            .map(|l| l.to_string())
            .collect();
        let src_labels = ClockSourceLabels{entries};
        self.ctl.load(card_cntr, &caps, &src_labels)?;

        self.meter_ctl.load(card_cntr, unit, &self.proto, TIMEOUT_MS)?;

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

impl NotifyModel<SndDice, u32> for FStudioModel {
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

impl MeasureModel<hinawa::SndDice> for FStudioModel {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.ctl.measured_elem_list);
        elem_id_list.extend_from_slice(&self.meter_ctl.measured_elem_list);
    }

    fn measure_states(&mut self, unit: &SndDice) -> Result<(), Error> {
        self.ctl.measure_states(unit, &self.proto, &self.sections, TIMEOUT_MS)?;
        self.meter_ctl.measure_states(unit, &self.proto, TIMEOUT_MS)?;
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
    meter: FStudioMeter,
    measured_elem_list: Vec<ElemId>,
}

impl<'a> MeterCtl {
    const ANALOG_INPUT_NAME: &'a str = "analog-input-meter";
    const STREAM_INPUT_NAME: &'a str = "stream-input-meter";
    const MIXER_OUTPUT_NAME: &'a str = "mixer-output-meter";

    const LEVEL_MIN: i32 = 0x00;
    const LEVEL_MAX: i32 = 0xff;
    const LEVEL_STEP: i32 = 1;
    const LEVEL_TLV: DbInterval = DbInterval{min: -9600, max: 0, linear: false, mute_avail: false};

    fn load(&mut self, card_cntr: &mut CardCntr, unit: &SndDice, proto: &FStudioProto, timeout_ms: u32)
        -> Result<(), Error>
    {
        proto.read_meter(&unit.get_node(), &mut self.meter, timeout_ms)?;

        [
            (Self::ANALOG_INPUT_NAME, self.meter.analog_inputs.len()),
            (Self::STREAM_INPUT_NAME, self.meter.stream_inputs.len()),
            (Self::MIXER_OUTPUT_NAME, self.meter.mixer_outputs.len()),
        ].iter()
            .try_for_each(|&(name, count)| {
                let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, name, 0);
                card_cntr.add_int_elems(&elem_id, 1, Self::LEVEL_MIN, Self::LEVEL_MAX, Self::LEVEL_STEP,
                                        count, Some(&Into::<Vec<u32>>::into(Self::LEVEL_TLV)), false)
                    .map(|mut elem_id_list| self.measured_elem_list.append(&mut elem_id_list))
            })?;

        Ok(())
    }

    fn measure_states(&mut self, unit: &SndDice, proto: &FStudioProto, timeout_ms: u32)
        -> Result<(), Error>
    {
        proto.read_meter(&unit.get_node(), &mut self.meter, timeout_ms)
    }

    fn read_measured_elem(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::ANALOG_INPUT_NAME => {
                let vals: Vec<i32> = self.meter.analog_inputs.iter()
                    .map(|&l| l as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            Self::STREAM_INPUT_NAME => {
                let vals: Vec<i32> = self.meter.stream_inputs.iter()
                    .map(|&l| l as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            Self::MIXER_OUTPUT_NAME => {
                let vals: Vec<i32> = self.meter.mixer_outputs.iter()
                    .map(|&l| l as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
