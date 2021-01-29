// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto
use glib::{Error, FileError};

use alsactl::{ElemId, ElemIfaceType, ElemValue, ElemValueExt, ElemValueExtManual};

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
    out_ctl: OutputCtl,
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
        self.out_ctl.load(card_cntr, unit, &self.proto, TIMEOUT_MS)?;

        Ok(())
    }

    fn read(&mut self, unit: &SndDice, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        if self.ctl.read(unit, &self.proto, &self.sections, elem_id, elem_value, TIMEOUT_MS)? {
            Ok(true)
        } else if self.out_ctl.read(unit, &self.proto, elem_id, elem_value, TIMEOUT_MS)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(&mut self, unit: &SndDice, elem_id: &ElemId, old: &ElemValue, new: &ElemValue)
        -> Result<bool, Error>
    {
        if self.ctl.write(unit, &self.proto, &self.sections, elem_id, old, new, TIMEOUT_MS)? {
            Ok(true)
        } else if self.out_ctl.write(unit, &self.proto, elem_id, new, TIMEOUT_MS)? {
            Ok(true)
        } else {
            Ok(false)
        }
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

#[derive(Default, Debug)]
struct OutputCtl{
    states: OutputState,
}

fn output_src_to_string(src: &OutputSrc) -> String {
    match src {
        OutputSrc::Analog(ch) => format!("Analog-{}", ch + 1),
        OutputSrc::Adat0(ch) => format!("ADAT-A-{}", ch + 1),
        OutputSrc::Spdif(ch) => format!("S/PDIF-{}", ch + 1),
        OutputSrc::Stream(ch) => format!("Stream-{}", ch + 1),
        OutputSrc::StreamAdat1(ch) => format!("Stream-{}/ADAT-B-{}", ch + 11, ch + 1),
        OutputSrc::MixerOut(ch) => format!("Mixer-{}", ch + 1),
        OutputSrc::Reserved(val) => format!("Reserved({})", val),
    }
}

impl<'a> OutputCtl {
    const SRC_NAME: &'a str = "output-source";
    const VOL_NAME: &'a str = "output-volume";
    const MUTE_NAME: &'a str = "output-mute";
    const LINK_NAME: &'a str = "output-link";
    const TERMINATE_BNC_NAME: &'a str = "terminate-bnc";

    const VOL_MIN: i32 = 0;
    const VOL_MAX: i32 = 0xff;
    const VOL_STEP: i32 = 1;
    const VOL_TLV: DbInterval = DbInterval{min: -9600, max: 0, linear: false, mute_avail: false};

    const SRCS: [OutputSrc;54] = [
        OutputSrc::Analog(0), OutputSrc::Analog(1), OutputSrc::Analog(2), OutputSrc::Analog(3),
        OutputSrc::Analog(4), OutputSrc::Analog(5), OutputSrc::Analog(6), OutputSrc::Analog(7),
        OutputSrc::Adat0(0), OutputSrc::Adat0(1), OutputSrc::Adat0(2), OutputSrc::Adat0(3),
        OutputSrc::Adat0(4), OutputSrc::Adat0(5), OutputSrc::Adat0(6), OutputSrc::Adat0(7),
        OutputSrc::Spdif(0), OutputSrc::Spdif(1),
        OutputSrc::Stream(0), OutputSrc::Stream(1), OutputSrc::Stream(2), OutputSrc::Stream(3),
        OutputSrc::Stream(4), OutputSrc::Stream(5), OutputSrc::Stream(6), OutputSrc::Stream(7),
        OutputSrc::Stream(8), OutputSrc::Stream(9),
        OutputSrc::StreamAdat1(0), OutputSrc::StreamAdat1(1),
        OutputSrc::StreamAdat1(2), OutputSrc::StreamAdat1(3),
        OutputSrc::StreamAdat1(4), OutputSrc::StreamAdat1(5),
        OutputSrc::StreamAdat1(6), OutputSrc::StreamAdat1(7),
        OutputSrc::MixerOut(0), OutputSrc::MixerOut(1), OutputSrc::MixerOut(2), OutputSrc::MixerOut(3),
        OutputSrc::MixerOut(4), OutputSrc::MixerOut(5), OutputSrc::MixerOut(6), OutputSrc::MixerOut(7),
        OutputSrc::MixerOut(8), OutputSrc::MixerOut(9), OutputSrc::MixerOut(10), OutputSrc::MixerOut(11),
        OutputSrc::MixerOut(12), OutputSrc::MixerOut(13), OutputSrc::MixerOut(14), OutputSrc::MixerOut(15),
        OutputSrc::MixerOut(16), OutputSrc::MixerOut(17),
    ];

    fn load(&mut self, card_cntr: &mut CardCntr, unit: &SndDice, proto: &FStudioProto, timeout_ms: u32)
        -> Result<(), Error>
    {
        proto.read_output_states(&unit.get_node(), &mut self.states, timeout_ms)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::VOL_NAME, 0);
        card_cntr.add_int_elems(&elem_id, 1, Self::VOL_MIN, Self::VOL_MAX, Self::VOL_STEP,
                                self.states.vols.len(), Some(&Into::<Vec<u32>>::into(Self::VOL_TLV)), true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::MUTE_NAME, 0);
        card_cntr.add_bool_elems(&elem_id, 1, self.states.mutes.len(), true)?;

        let labels: Vec<String> = Self::SRCS.iter()
            .map(|s| output_src_to_string(s))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::SRC_NAME, 0);
        card_cntr.add_enum_elems(&elem_id, 1, self.states.srcs.len(), &labels, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::LINK_NAME, 0);
        card_cntr.add_bool_elems(&elem_id, 1, self.states.links.len(), true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::TERMINATE_BNC_NAME, 0);
        card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        Ok(())
    }

    fn read(&mut self, unit: &SndDice, proto: &FStudioProto, elem_id: &ElemId, elem_value: &mut ElemValue,
            timeout_ms: u32)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::VOL_NAME => {
                let vals: Vec<i32> = self.states.vols.iter()
                    .map(|&vol| vol as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            Self::MUTE_NAME => {
                elem_value.set_bool(&self.states.mutes);
                Ok(true)
            }
            Self::SRC_NAME => {
                let vals: Vec<u32> = self.states.srcs.iter()
                    .map(|src| {
                        let pos = Self::SRCS.iter()
                            .position(|s| s.eq(src))
                            .unwrap();
                        pos as u32
                    })
                    .collect();
                elem_value.set_enum(&vals);
                Ok(true)
            }
            Self::LINK_NAME => {
                elem_value.set_bool(&self.states.links);
                Ok(true)
            }
            Self::TERMINATE_BNC_NAME => {
                proto.read_bnc_terminate(&unit.get_node(), timeout_ms)
                    .map(|terminate| {
                        elem_value.set_bool(&[terminate]);
                        true
                    })
            }
            _ => Ok(false),
        }
    }

    fn write(&mut self, unit: &SndDice, proto: &FStudioProto, elem_id: &ElemId, elem_value: &ElemValue,
             timeout_ms: u32)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::VOL_NAME => {
                let mut vals = vec![0;self.states.vols.len()];
                elem_value.get_int(&mut vals);
                let vols: Vec<u8> = vals.iter()
                    .map(|&val| val as u8)
                    .collect();
                proto.write_output_vols(&unit.get_node(), &mut self.states, &vols, timeout_ms)
                    .map(|_| true)
            }
            Self::MUTE_NAME => {
                let mut vals = self.states.mutes.clone();
                elem_value.get_bool(&mut vals);
                proto.write_output_mute(&unit.get_node(), &mut self.states, &vals, timeout_ms)
                    .map(|_| true)
            }
            Self::SRC_NAME => {
                let mut vals = vec![0;self.states.srcs.len()];
                elem_value.get_enum(&mut vals);

                let mut srcs = self.states.srcs.clone();
                vals.iter()
                    .enumerate()
                    .try_for_each(|(i, &val)| {
                        Self::SRCS.iter()
                            .nth(val as usize)
                            .ok_or_else(|| {
                                let msg = format!("Invalid value for index of output source: {}", val);
                                Error::new(FileError::Inval, &msg)
                            })
                            .map(|&src| srcs[i] = src)
                    })?;
                proto.write_output_src(&unit.get_node(), &mut self.states, &srcs, timeout_ms)
                    .map(|_| true)
            }
            Self::LINK_NAME => {
                let mut vals = self.states.links.clone();
                elem_value.get_bool(&mut vals);
                proto.write_output_link(&unit.get_node(), &mut self.states, &vals, timeout_ms)
                    .map(|_| true)
            }
            Self::TERMINATE_BNC_NAME => {
                let mut vals = [false];
                elem_value.get_bool(&mut vals);
                proto.write_bnc_terminalte(&unit.get_node(), vals[0], timeout_ms)
                    .map(|_| true)
            }
            _ => Ok(false),
        }
    }
}
