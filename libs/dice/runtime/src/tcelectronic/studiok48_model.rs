// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::{Error, FileError};

use alsactl::{ElemId, ElemIfaceType, ElemValue, ElemValueExt, ElemValueExtManual};

use hinawa::FwReq;
use hinawa::{SndDice, SndUnitExt};

use alsa_ctl_tlv_codec::items::DbInterval;

use core::card_cntr::*;
use core::elem_value_accessor::*;

use dice_protocols::tcat::{*, global_section::*};
use dice_protocols::tcelectronic::*;
use dice_protocols::tcelectronic::studio::*;

use crate::common_ctl::*;
use super::ch_strip_ctl::*;
use super::reverb_ctl::*;
use super::fw_led_ctl::*;
use super::standalone_ctl::*;
use super::midi_send_ctl::*;
use super::prog_ctl::*;

#[derive(Default)]
pub struct Studiok48Model{
    req: FwReq,
    sections: GeneralSections,
    segments: StudioSegments,
    ctl: CommonCtl,
    ch_strip_ctl: ChStripCtl,
    reverb_ctl: ReverbCtl,
    hw_state_ctl: HwStateCtl,
    phys_out_ctl: PhysOutCtl,
    mixer_ctl: MixerCtl,
    config_ctl: ConfigCtl,
    remote_ctl: RemoteCtl,
    lineout_ctl: LineoutCtl,
}

const TIMEOUT_MS: u32 = 20;

impl CtlModel<SndDice> for Studiok48Model {
    fn load(&mut self, unit: &mut SndDice, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let mut node = unit.get_node();

        self.sections = self.req.read_general_sections(&mut node, TIMEOUT_MS)?;
        let caps = self.req.read_clock_caps(&mut node, &self.sections, TIMEOUT_MS)?;
        let src_labels = self.req.read_clock_source_labels(&mut node, &self.sections, TIMEOUT_MS)?;
        self.ctl.load(card_cntr, &caps, &src_labels)?;

        self.ch_strip_ctl.load(unit, &mut self.req, &mut self.segments.ch_strip_state,
                               &mut self.segments.ch_strip_meter, TIMEOUT_MS, card_cntr)?;
        self.reverb_ctl.load(unit, &mut self.req, &mut self.segments.reverb_state, &mut self.segments.reverb_meter,
                             TIMEOUT_MS, card_cntr)?;

        self.req.read_segment(&mut node, &mut self.segments.hw_state, TIMEOUT_MS)?;
        self.req.read_segment(&mut node, &mut self.segments.phys_out, TIMEOUT_MS)?;
        self.req.read_segment(&mut node, &mut self.segments.mixer_state, TIMEOUT_MS)?;
        self.req.read_segment(&mut node, &mut self.segments.config, TIMEOUT_MS)?;
        self.req.read_segment(&mut node, &mut self.segments.remote, TIMEOUT_MS)?;
        self.req.read_segment(&mut node, &mut self.segments.out_level, TIMEOUT_MS)?;

        self.hw_state_ctl.load(card_cntr)?;
        self.phys_out_ctl.load(card_cntr)?;
        self.mixer_ctl.load(&self.segments, card_cntr)?;
        self.config_ctl.load(card_cntr)?;
        self.remote_ctl.load(card_cntr)?;
        self.lineout_ctl.load(card_cntr)?;

        Ok(())
    }

    fn read(&mut self, unit: &mut SndDice, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        if self.ctl.read(unit, &mut self.req, &self.sections, elem_id, elem_value, TIMEOUT_MS)? {
            Ok(true)
        } else if self.ch_strip_ctl.read(&self.segments.ch_strip_state, &self.segments.ch_strip_meter,
                                         elem_id, elem_value)? {
            Ok(true)
        } else if self.reverb_ctl.read(&self.segments.reverb_state, &self.segments.reverb_meter,
                                       elem_id, elem_value)? {
            Ok(true)
        } else if self.hw_state_ctl.read(&self.segments, elem_id, elem_value)? {
            Ok(true)
        } else if self.phys_out_ctl.read(&self.segments, elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_ctl.read(&self.segments, elem_id, elem_value)? {
            Ok(true)
        } else if self.config_ctl.read(&self.segments, elem_id, elem_value)? {
            Ok(true)
        } else if self.remote_ctl.read(&self.segments, elem_id, elem_value)? {
            Ok(true)
        } else if self.lineout_ctl.read(&self.segments, elem_id, elem_value)? {
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
        } else if self.ch_strip_ctl.write(unit, &mut self.req, &mut self.segments.ch_strip_state, elem_id,
                                          old, new, TIMEOUT_MS)? {
            Ok(true)
        } else if self.reverb_ctl.write(unit, &mut self.req, &mut self.segments.reverb_state, elem_id,
                                        new, TIMEOUT_MS)? {
            Ok(true)
        } else if self.hw_state_ctl.write(unit, &mut self.req, &mut self.segments, elem_id,
                                          new, TIMEOUT_MS)? {
            Ok(true)
        } else if self.phys_out_ctl.write(unit, &mut self.req, &mut self.segments, elem_id, old, new,
                                          TIMEOUT_MS)? {
            Ok(true)
        } else if self.mixer_ctl.write(unit, &mut self.req, &mut self.segments, elem_id, old, new,
                                       TIMEOUT_MS)? {
            Ok(true)
        } else if self.config_ctl.write(unit, &mut self.req, &mut self.segments, elem_id, new, TIMEOUT_MS)? {
            Ok(true)
        } else if self.remote_ctl.write(unit, &mut self.req, &mut self.segments, elem_id, old, new, TIMEOUT_MS)? {
            Ok(true)
        } else if self.lineout_ctl.write(unit, &mut self.req, &mut self.segments, elem_id, new, TIMEOUT_MS)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl NotifyModel<SndDice, u32> for Studiok48Model {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.ctl.notified_elem_list);
        elem_id_list.extend_from_slice(&self.ch_strip_ctl.notified_elem_list);
        elem_id_list.extend_from_slice(&self.reverb_ctl.notified_elem_list);
        elem_id_list.extend_from_slice(&self.hw_state_ctl.notified_elem_list);
        elem_id_list.extend_from_slice(&self.phys_out_ctl.0);
        elem_id_list.extend_from_slice(&self.mixer_ctl.notified_elem_list);
        elem_id_list.extend_from_slice(&self.remote_ctl.notified_elem_list);
    }

    fn parse_notification(&mut self, unit: &mut SndDice, msg: &u32) -> Result<(), Error> {
        self.ctl.parse_notification(unit, &mut self.req, &self.sections, *msg, TIMEOUT_MS)?;

        let mut node = unit.get_node();
        self.req.parse_notification(&mut node, &mut self.segments.ch_strip_state, TIMEOUT_MS, *msg)?;
        self.req.parse_notification(&mut node, &mut self.segments.reverb_state, TIMEOUT_MS, *msg)?;
        self.req.parse_notification(&mut node, &mut self.segments.hw_state, TIMEOUT_MS, *msg)?;
        self.req.parse_notification(&mut node, &mut self.segments.phys_out, TIMEOUT_MS, msg)?;
        self.req.parse_notification(&mut node, &mut self.segments.mixer_state, TIMEOUT_MS, msg)?;
        self.req.parse_notification(&mut node, &mut self.segments.remote, TIMEOUT_MS, msg)?;

        Ok(())
    }

    fn read_notified_elem(&mut self, _: &SndDice, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        if self.ctl.read_notified_elem(elem_id, elem_value)? {
            Ok(true)
        } else if self.ch_strip_ctl.read_notified_elem(&self.segments.ch_strip_state, elem_id, elem_value)? {
            Ok(true)
        } else if self.reverb_ctl.read_notified_elem(&self.segments.reverb_state, elem_id, elem_value)? {
            Ok(true)
        } else if self.hw_state_ctl.read(&self.segments, elem_id, elem_value)? {
            Ok(true)
        } else if self.phys_out_ctl.read(&self.segments, elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_ctl.read(&self.segments, elem_id, elem_value)? {
            Ok(true)
        } else if self.remote_ctl.read(&self.segments, elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl MeasureModel<SndDice> for Studiok48Model {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.ctl.measured_elem_list);
        elem_id_list.extend_from_slice(&self.ch_strip_ctl.measured_elem_list);
        elem_id_list.extend_from_slice(&self.reverb_ctl.measured_elem_list);
        elem_id_list.extend_from_slice(&self.mixer_ctl.measured_elem_list);
    }

    fn measure_states(&mut self, unit: &mut SndDice) -> Result<(), Error> {
        self.ctl.measure_states(unit, &mut self.req, &self.sections, TIMEOUT_MS)?;
        self.ch_strip_ctl.measure_states(unit, &mut self.req, &self.segments.ch_strip_state,
                                         &mut self.segments.ch_strip_meter, TIMEOUT_MS)?;
        self.reverb_ctl.measure_states(unit, &mut self.req, &self.segments.reverb_state,
                                       &mut self.segments.reverb_meter, TIMEOUT_MS)?;

        let mut node = unit.get_node();
        self.req.read_segment(&mut node, &mut self.segments.mixer_meter, TIMEOUT_MS)?;

        Ok(())
    }

    fn measure_elem(&mut self, _: &SndDice, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        if self.ctl.measure_elem(elem_id, elem_value)? {
            Ok(true)
        } else if self.ch_strip_ctl.read_measured_elem(&self.segments.ch_strip_meter, elem_id, elem_value)? {
            Ok(true)
        } else if self.reverb_ctl.read_measured_elem(&self.segments.reverb_meter, elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_ctl.read_measured_elem(&self.segments, elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

fn analog_jack_state_to_str(state: &StudioAnalogJackState) -> &'static str {
    match state {
        StudioAnalogJackState::FrontSelected => "front-selected",
        StudioAnalogJackState::FrontInserted => "front-inserted",
        StudioAnalogJackState::RearSelected => "rear-selected",
        StudioAnalogJackState::RearInserted => "rear-inserted",
    }
}

#[derive(Default, Debug)]
struct HwStateCtl {
    notified_elem_list: Vec<ElemId>,
    fw_led_ctl: FwLedCtl,
}

// TODO: For Jack detection in ALSA applications.
const ANALOG_JACK_STATE_NAME: &str = "analog-jack-state";
const HP_JACK_STATE_NAME: &str = "headphone-jack-state";
const VALID_MASTER_LEVEL_NAME: &str = "valid-master-level";

impl HwStateCtl {
    const ANALOG_JACK_STATES: [StudioAnalogJackState; 4] = [
        StudioAnalogJackState::FrontSelected,
        StudioAnalogJackState::FrontInserted,
        StudioAnalogJackState::RearSelected,
        StudioAnalogJackState::RearInserted,
    ];

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let labels = Self::ANALOG_JACK_STATES.iter()
            .map(|s| analog_jack_state_to_str(s))
            .collect::<Vec<_>>();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, ANALOG_JACK_STATE_NAME, 0);
        card_cntr.add_enum_elems(&elem_id, 1, STUDIO_ANALOG_JACK_STATE_COUNT, &labels, None, false)
            .map(|mut elem_id_list| self.notified_elem_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, HP_JACK_STATE_NAME, 0);
        card_cntr.add_bool_elems(&elem_id, 1, 2, false)
            .map(|mut elem_id_list| self.notified_elem_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, VALID_MASTER_LEVEL_NAME, 0);
        card_cntr.add_bool_elems(&elem_id, 1, 1, false)
            .map(|mut elem_id_list| self.notified_elem_list.append(&mut elem_id_list))?;

        self.fw_led_ctl.load(card_cntr)
            .map(|_| self.notified_elem_list.extend_from_slice(&self.fw_led_ctl.0))?;

        Ok(())
    }

    fn read(
        &mut self,
        segments: &StudioSegments,
        elem_id: &ElemId,
        elem_value: &mut ElemValue
    ) -> Result<bool, Error> {
        if self.fw_led_ctl.read(&segments.hw_state, elem_id, elem_value)? {
            Ok(true)
        } else if self.read_notified_elem(&segments, elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(
        &mut self,
        unit: &mut SndDice,
        req: &mut FwReq,
        segments: &mut StudioSegments,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32
    ) -> Result<bool, Error> {
        self.fw_led_ctl.write(unit, req, &mut segments.hw_state, elem_id, elem_value, timeout_ms)
    }

    fn read_notified_elem(
        &mut self,
        segments: &StudioSegments,
        elem_id: &ElemId,
        elem_value: &mut ElemValue
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            ANALOG_JACK_STATE_NAME => {
                let analog_jack_states = &segments.hw_state.data.analog_jack_states;
                ElemValueAccessor::<u32>::set_vals(elem_value, analog_jack_states.len(), |idx| {
                    let pos = Self::ANALOG_JACK_STATES.iter()
                        .position(|s| s.eq(&analog_jack_states[idx]))
                        .expect("Programming error");
                    Ok(pos as u32)
                })
                .map(|_| true)
            }
            HP_JACK_STATE_NAME => {
                elem_value.set_bool(&segments.hw_state.data.hp_state);
                Ok(true)
            }
            VALID_MASTER_LEVEL_NAME => {
                elem_value.set_bool(&[segments.hw_state.data.valid_master_level]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

fn src_pair_entry_to_string(entry: &SrcEntry) -> String {
    match entry {
        SrcEntry::Unused => "Unused".to_string(),
        SrcEntry::Analog(ch) => format!("Analog-{}", ch + 1),
        SrcEntry::Spdif(ch) => format!("S/PDIF-{}", ch + 1),
        SrcEntry::Adat(ch) => format!("ADAT-{}", ch + 1),
        SrcEntry::StreamA(ch) => format!("Stream-A-{}", ch + 1),
        SrcEntry::StreamB(ch) => format!("Stream-B-{}", ch + 1),
        SrcEntry::Mixer(ch) => {
            if *ch < 2 {
                format!("Mixer-{}", ch + 1)
            } else if *ch < 6 {
                format!("Aux-{}", ch - 1)
            } else {
                format!("Reverb-{}", ch - 5)
            }
        }
    }
}

fn cross_over_freq_to_string(freq: &CrossOverFreq) -> String {
    match freq {
        CrossOverFreq::F50 => "50Hz".to_string(),
        CrossOverFreq::F80 => "80Hz".to_string(),
        CrossOverFreq::F95 => "95Hz".to_string(),
        CrossOverFreq::F110 => "110Hz".to_string(),
        CrossOverFreq::F115 => "115Hz".to_string(),
        CrossOverFreq::F120 => "120Hz".to_string(),
        CrossOverFreq::Reserved(val) => format!("Reserved({})", val),
    }
}

fn high_pass_freq_to_string(freq: &HighPassFreq) -> String {
    match freq {
        HighPassFreq::Off => "Off".to_string(),
        HighPassFreq::Above12 => "12HzAbove".to_string(),
        HighPassFreq::Above24 => "24HzAbove".to_string(),
        HighPassFreq::Reserved(val) => format!("Reserved({})", val),
    }
}

fn low_pass_freq_to_string(freq: &LowPassFreq) -> String {
    match freq {
        LowPassFreq::Below12 => "12HzBelow".to_string(),
        LowPassFreq::Below24 => "24HzBelow".to_string(),
        LowPassFreq::Reserved(val) => format!("Reserved({})", val),
    }
}

#[derive(Default, Debug)]
struct PhysOutCtl(pub Vec<ElemId>);

const MASTER_OUT_DIM_NAME: &str = "master-out-dim";
const MASTER_OUT_VOL_NAME: &str = "master-out-volume";
const MASTER_OUT_DIM_VOL_NAME: &str = "master-out-dim-volume";

const OUT_STEREO_LINK_NAME: &str = "output-stereo-link";
const OUT_MUTE_NAME: &str = "output-mute";
const OUT_SRC_NAME: &str = "output-source";

const OUT_GRP_SELECT_NAME: &str = "output-group:select";
const OUT_GRP_SRC_ENABLE_NAME: &str = "output-group:source-enable";
const OUT_GRP_SRC_TRIM_NAME: &str = "output-group:source-trim";
const OUT_GRP_SRC_DELAY_NAME: &str = "output-group:source-delay";
const OUT_GRP_SRC_ASSIGN_NAME: &str = "output-group:source-assign";
const OUT_GRP_BASS_MANAGEMENT_NAME: &str = "output-group:bass-management";
const OUT_GRP_MAIN_CROSS_OVER_FREQ_NAME: &str = "output-group:main-cross-over-frequency";
const OUT_GRP_MAIN_LEVEL_TO_SUB_NAME: &str = "output-group:main-level-to-sub";
const OUT_GRP_SUB_LEVEL_TO_SUB_NAME: &str = "output-group:sub-level-to-sub";
const OUT_GRP_MAIN_FILTER_FOR_MAIN_NAME: &str = "output-group:main-filter-for-main";
const OUT_GRP_MAIN_FILTER_FOR_SUB_NAME: &str = "output-group:main-filter-for-sub";

impl PhysOutCtl {
    const PHYS_OUT_SRCS: [SrcEntry;59] = [
        SrcEntry::Unused,
        SrcEntry::Analog(0), SrcEntry::Analog(1), SrcEntry::Analog(2), SrcEntry::Analog(3),
        SrcEntry::Analog(4), SrcEntry::Analog(5), SrcEntry::Analog(6), SrcEntry::Analog(7),
        SrcEntry::Analog(8), SrcEntry::Analog(9), SrcEntry::Analog(10), SrcEntry::Analog(11),
        SrcEntry::Spdif(0), SrcEntry::Spdif(1),
        SrcEntry::Adat(0), SrcEntry::Adat(1), SrcEntry::Adat(2), SrcEntry::Adat(3),
        SrcEntry::Adat(4), SrcEntry::Adat(5), SrcEntry::Adat(6), SrcEntry::Adat(7),
        SrcEntry::StreamA(0), SrcEntry::StreamA(1), SrcEntry::StreamA(2), SrcEntry::StreamA(3),
        SrcEntry::StreamA(4), SrcEntry::StreamA(5), SrcEntry::StreamA(6), SrcEntry::StreamA(7),
        SrcEntry::StreamA(8), SrcEntry::StreamA(9), SrcEntry::StreamA(10), SrcEntry::StreamA(11),
        SrcEntry::StreamA(12), SrcEntry::StreamA(13), SrcEntry::StreamA(14), SrcEntry::StreamA(15),
        SrcEntry::StreamB(0), SrcEntry::StreamB(1), SrcEntry::StreamB(2), SrcEntry::StreamB(3),
        SrcEntry::StreamB(4), SrcEntry::StreamB(5), SrcEntry::StreamB(6), SrcEntry::StreamB(7),
        SrcEntry::StreamB(8), SrcEntry::StreamB(9), SrcEntry::StreamB(10), SrcEntry::StreamB(11),
        SrcEntry::Mixer(0), SrcEntry::Mixer(1),
        SrcEntry::Mixer(2), SrcEntry::Mixer(3), SrcEntry::Mixer(4), SrcEntry::Mixer(5),
        SrcEntry::Mixer(6), SrcEntry::Mixer(7),
    ];

    const VOL_MIN: i32 = -1000;
    const VOL_MAX: i32 = 0;
    const VOL_STEP: i32 = 1;
    const VOL_TLV: DbInterval = DbInterval{min: -7200, max: 0, linear: false, mute_avail: false};

    const OUT_GRPS: [&'static str; 3] = ["Group-A", "Group-B", "Group-C"];

    const CROSS_OVER_FREQS: [CrossOverFreq; 6] = [
        CrossOverFreq::F50,
        CrossOverFreq::F80,
        CrossOverFreq::F95,
        CrossOverFreq::F110,
        CrossOverFreq::F115,
        CrossOverFreq::F120,
    ];

    const HIGH_PASS_FREQS: [HighPassFreq; 3] = [
        HighPassFreq::Off,
        HighPassFreq::Above12,
        HighPassFreq::Above24,
    ];

    const LOW_PASS_FREQS: [LowPassFreq; 2] = [
        LowPassFreq::Below12,
        LowPassFreq::Below24,
    ];

    const TRIM_MIN: i32 = -20;
    const TRIM_MAX: i32 = 0;
    const TRIM_STEP: i32 = 1;

    const DELAY_MIN: i32 = 0;
    const DELAY_MAX: i32 = 30;
    const DELAY_STEP: i32 = 1;

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        // For master output.
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MASTER_OUT_DIM_NAME, 0);
        card_cntr.add_bool_elems(&elem_id, 1, 1, true)
            .map(|mut elem_id_list| self.0.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MASTER_OUT_VOL_NAME, 0);
        card_cntr.add_int_elems(&elem_id, 1, Self::VOL_MIN, Self::VOL_MAX, Self::VOL_STEP,
                                1, Some(&Into::<Vec<u32>>::into(Self::VOL_TLV)), true)
            .map(|mut elem_id_list| self.0.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MASTER_OUT_DIM_VOL_NAME, 0);
        card_cntr.add_int_elems(&elem_id, 1, Self::VOL_MIN, Self::VOL_MAX, Self::VOL_STEP,
                                1, Some(&Into::<Vec<u32>>::into(Self::VOL_TLV)), true)
            .map(|mut elem_id_list| self.0.append(&mut elem_id_list))?;

        // For source of output pair.
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, OUT_STEREO_LINK_NAME, 0);
        card_cntr.add_bool_elems(&elem_id, 1, STUDIO_PHYS_OUT_PAIR_COUNT, true)
            .map(|mut elem_id_list| self.0.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, OUT_MUTE_NAME, 0);
        card_cntr.add_bool_elems(&elem_id, 1, STUDIO_PHYS_OUT_PAIR_COUNT * 2, true)
            .map(|mut elem_id_list| self.0.append(&mut elem_id_list))?;

        let labels: Vec<String> = Self::PHYS_OUT_SRCS.iter()
            .map(|src| src_pair_entry_to_string(src))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, OUT_SRC_NAME, 0);
        card_cntr.add_enum_elems(&elem_id, 1, STUDIO_PHYS_OUT_PAIR_COUNT * 2, &labels, None, true)
            .map(|mut elem_id_list| self.0.append(&mut elem_id_list))?;

        // For output group.
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, OUT_GRP_SELECT_NAME, 0);
        card_cntr.add_enum_elems(&elem_id, 1, 1, &Self::OUT_GRPS, None, true)
            .map(|mut elem_id_list| self.0.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, OUT_GRP_SRC_ENABLE_NAME, 0);
        card_cntr.add_bool_elems(&elem_id, 1, STUDIO_PHYS_OUT_PAIR_COUNT * 2, true)
            .map(|mut elem_id_list| self.0.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, OUT_GRP_SRC_TRIM_NAME, 0);
        card_cntr.add_int_elems(&elem_id, 1, Self::TRIM_MIN, Self::TRIM_MAX, Self::TRIM_STEP,
                                STUDIO_PHYS_OUT_PAIR_COUNT * 2, None, true)
            .map(|mut elem_id_list| self.0.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, OUT_GRP_SRC_DELAY_NAME, 0);
        card_cntr.add_int_elems(&elem_id, 1, Self::DELAY_MIN, Self::DELAY_MAX, Self::DELAY_STEP,
                                STUDIO_PHYS_OUT_PAIR_COUNT * 2, None, true)
            .map(|mut elem_id_list| self.0.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, OUT_GRP_SRC_ASSIGN_NAME, 0);
        card_cntr.add_bool_elems(&elem_id, STUDIO_OUTPUT_GROUP_COUNT, STUDIO_PHYS_OUT_PAIR_COUNT * 2, true)
            .map(|mut elem_id_list| self.0.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, OUT_GRP_BASS_MANAGEMENT_NAME, 0);
        card_cntr.add_bool_elems(&elem_id, 1, STUDIO_OUTPUT_GROUP_COUNT, true)
            .map(|mut elem_id_list| self.0.append(&mut elem_id_list))?;

        let labels: Vec<String> = Self::CROSS_OVER_FREQS.iter()
            .map(|src| cross_over_freq_to_string(src))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, OUT_GRP_MAIN_CROSS_OVER_FREQ_NAME, 0);
        card_cntr.add_enum_elems(&elem_id, 1, STUDIO_OUTPUT_GROUP_COUNT, &labels, None, true)
            .map(|mut elem_id_list| self.0.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, OUT_GRP_MAIN_LEVEL_TO_SUB_NAME, 0);
        card_cntr.add_int_elems(&elem_id, 1, Self::VOL_MIN, Self::VOL_MAX, Self::VOL_STEP,
                                STUDIO_OUTPUT_GROUP_COUNT, Some(&Into::<Vec<u32>>::into(Self::VOL_TLV)), true)
            .map(|mut elem_id_list| self.0.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, OUT_GRP_SUB_LEVEL_TO_SUB_NAME, 0);
        card_cntr.add_int_elems(&elem_id, 1, Self::VOL_MIN, Self::VOL_MAX, Self::VOL_STEP,
                                STUDIO_OUTPUT_GROUP_COUNT, Some(&Into::<Vec<u32>>::into(Self::VOL_TLV)), true)
            .map(|mut elem_id_list| self.0.append(&mut elem_id_list))?;

        let labels: Vec<String> = Self::HIGH_PASS_FREQS.iter()
            .map(|src| high_pass_freq_to_string(src))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, OUT_GRP_MAIN_FILTER_FOR_MAIN_NAME, 0);
        card_cntr.add_enum_elems(&elem_id, 1, STUDIO_OUTPUT_GROUP_COUNT, &labels, None, true)
            .map(|mut elem_id_list| self.0.append(&mut elem_id_list))?;

        let labels: Vec<String> = Self::LOW_PASS_FREQS.iter()
            .map(|src| low_pass_freq_to_string(src))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, OUT_GRP_MAIN_FILTER_FOR_SUB_NAME, 0);
        card_cntr.add_enum_elems(&elem_id, 1, STUDIO_OUTPUT_GROUP_COUNT, &labels, None, true)
            .map(|mut elem_id_list| self.0.append(&mut elem_id_list))?;

        Ok(())
    }

    fn read(
        &mut self,
        segments: &StudioSegments,
        elem_id: &ElemId,
        elem_value: &mut ElemValue
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            MASTER_OUT_DIM_NAME => {
                ElemValueAccessor::<bool>::set_val(elem_value, || {
                    Ok(segments.phys_out.data.master_out.dim_enabled)
                })
                .map(|_| true)
            }
            MASTER_OUT_VOL_NAME => {
                ElemValueAccessor::<i32>::set_val(elem_value, || {
                    Ok(segments.phys_out.data.master_out.vol)
                })
                .map(|_| true)
            }
            MASTER_OUT_DIM_VOL_NAME => {
                ElemValueAccessor::<i32>::set_val(elem_value, || {
                    Ok(segments.phys_out.data.master_out.dim_vol)
                })
                .map(|_| true)
            }
            OUT_STEREO_LINK_NAME => {
                ElemValueAccessor::<bool>::set_vals(elem_value, STUDIO_PHYS_OUT_PAIR_COUNT, |idx| {
                    Ok(segments.phys_out.data.out_pair_srcs[idx].stereo_link)
                })
                .map(|_| true)
            }
            OUT_MUTE_NAME => {
                elem_value.set_bool(&segments.phys_out.data.out_mutes);
                Ok(true)
            }
            OUT_SRC_NAME => {
                Self::read_out_src_param(segments, elem_value, |param| {
                     let pos = Self::PHYS_OUT_SRCS.iter()
                        .position(|s| s.eq(&param.src))
                         .expect("Programming error");
                     Ok(pos as u32)
                 })
                .map(|_| true)
            }
            OUT_GRP_SELECT_NAME => {
                elem_value.set_enum(&[segments.phys_out.data.selected_out_grp as u32]);
                Ok(true)
            }
            OUT_GRP_SRC_ENABLE_NAME => {
                elem_value.set_bool(&segments.phys_out.data.out_assign_to_grp);
                Ok(true)
            }
            OUT_GRP_SRC_TRIM_NAME => {
                Self::read_out_src_param(segments, elem_value, |param| {
                    Ok(param.vol)
                })
            }
            OUT_GRP_SRC_DELAY_NAME => {
                Self::read_out_src_param(segments, elem_value, |param| {
                    Ok(param.delay)
                })
            }
            OUT_GRP_SRC_ASSIGN_NAME => {
                let index = elem_id.get_index() as usize;
                elem_value.set_bool(&segments.phys_out.data.out_grps[index].assigned_phys_outs);
                Ok(true)
            }
            OUT_GRP_BASS_MANAGEMENT_NAME => {
                ElemValueAccessor::<bool>::set_vals(elem_value, STUDIO_OUTPUT_GROUP_COUNT, |idx| {
                    Ok(segments.phys_out.data.out_grps[idx].bass_management)
                })
                .map(|_| true)
            }
            OUT_GRP_MAIN_CROSS_OVER_FREQ_NAME => {
                ElemValueAccessor::<u32>::set_vals(elem_value, STUDIO_OUTPUT_GROUP_COUNT, |idx| {
                    let pos = Self::CROSS_OVER_FREQS.iter()
                        .position(|freq| freq.eq(&segments.phys_out.data.out_grps[idx].main_cross_over_freq))
                        .unwrap();
                    Ok(pos as u32)
                })
                .map(|_| true)
            }
            OUT_GRP_MAIN_LEVEL_TO_SUB_NAME => {
                ElemValueAccessor::<i32>::set_vals(elem_value, STUDIO_OUTPUT_GROUP_COUNT, |idx| {
                    Ok(segments.phys_out.data.out_grps[idx].main_level_to_sub)
                })
                .map(|_| true)
            }
            OUT_GRP_SUB_LEVEL_TO_SUB_NAME => {
                ElemValueAccessor::<i32>::set_vals(elem_value, STUDIO_OUTPUT_GROUP_COUNT, |idx| {
                    Ok(segments.phys_out.data.out_grps[idx].sub_level_to_sub)
                })
                .map(|_| true)
            }
            OUT_GRP_MAIN_FILTER_FOR_MAIN_NAME => {
                ElemValueAccessor::<u32>::set_vals(elem_value, STUDIO_OUTPUT_GROUP_COUNT, |idx| {
                    let pos = Self::HIGH_PASS_FREQS.iter()
                        .position(|freq| freq.eq(&segments.phys_out.data.out_grps[idx].main_filter_for_main))
                        .unwrap();
                    Ok(pos as u32)
                })
                .map(|_| true)
            }
            OUT_GRP_MAIN_FILTER_FOR_SUB_NAME => {
                ElemValueAccessor::<u32>::set_vals(elem_value, STUDIO_OUTPUT_GROUP_COUNT, |idx| {
                    let pos = Self::LOW_PASS_FREQS.iter()
                        .position(|freq| freq.eq(&segments.phys_out.data.out_grps[idx].main_filter_for_sub))
                        .unwrap();
                    Ok(pos as u32)
                })
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }

    fn read_out_src_param<T, F>(
        segments: &StudioSegments,
        elem_value: &mut ElemValue,
        cb: F
    ) -> Result<bool, Error>
        where F: Fn(&PhysOutSrcParam) -> Result<T, Error>,
              T: Default + Copy + Eq,
              ElemValue: ElemValueAccessor<T>,
    {
        ElemValueAccessor::<T>::set_vals(elem_value, STUDIO_PHYS_OUT_PAIR_COUNT * 2, |idx| {
            let i = idx / 2;
            let ch = idx % 2;
            let param = if ch == 0 {
                &segments.phys_out.data.out_pair_srcs[i].left
            } else {
                &segments.phys_out.data.out_pair_srcs[i].right
            };
            cb(param)
        })
        .map(|_| true)
    }

    fn write(
        &mut self,
        unit: &mut SndDice,
        req: &mut FwReq,
        segments: &mut StudioSegments,
        elem_id: &ElemId,
        old: &ElemValue,
        new: &ElemValue,
        timeout_ms: u32
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            MASTER_OUT_DIM_NAME => {
                ElemValueAccessor::<bool>::get_val(new, |val| {
                    segments.phys_out.data.master_out.dim_enabled = val;
                    Ok(())
                })
                .and_then(|_| req.write_segment(&mut unit.get_node(), &mut segments.phys_out, timeout_ms))
                .map(|_| true)
            }
            MASTER_OUT_VOL_NAME => {
                ElemValueAccessor::<i32>::get_val(new, |val| {
                    segments.phys_out.data.master_out.vol = val;
                    Ok(())
                })
                .and_then(|_| req.write_segment(&mut unit.get_node(), &mut segments.phys_out, timeout_ms))
                .map(|_| true)
            }
            MASTER_OUT_DIM_VOL_NAME => {
                ElemValueAccessor::<i32>::get_val(new, |val| {
                    segments.phys_out.data.master_out.dim_vol = val;
                    Ok(())
                })
                .and_then(|_| req.write_segment(&mut unit.get_node(), &mut segments.phys_out, timeout_ms))
                .map(|_| true)
            }
            OUT_STEREO_LINK_NAME => {
                ElemValueAccessor::<bool>::get_vals(new, old, STUDIO_PHYS_OUT_PAIR_COUNT, |idx, val| {
                    segments.phys_out.data.out_pair_srcs[idx].stereo_link = val;
                    Ok(())
                })
                .and_then(|_| req.write_segment(&mut unit.get_node(), &mut segments.phys_out, timeout_ms))
                .map(|_| true)
            }
            OUT_MUTE_NAME => {
                new.get_bool(&mut segments.phys_out.data.out_mutes);
                req.write_segment(&mut unit.get_node(), &mut segments.phys_out, timeout_ms)
                    .map(|_| true)
            }
            OUT_SRC_NAME => {
                Self::write_out_src_param(unit, req, segments, new, old, timeout_ms, |param, val: u32| {
                     Self::PHYS_OUT_SRCS.iter()
                         .nth(val as usize)
                         .ok_or_else(|| {
                             let msg = format!("Invalid value for index of source of output: {}", val);
                             Error::new(FileError::Inval, &msg)
                         })
                        .map(|&s| param.src = s)
                 })
            }
            OUT_GRP_SELECT_NAME => {
                let mut vals = [0];
                new.get_enum(&mut vals);
                segments.phys_out.data.selected_out_grp = vals[0] as usize;
                req.write_segment(&mut unit.get_node(), &mut segments.phys_out, timeout_ms)
                    .map(|_| true)
            }
            OUT_GRP_SRC_ENABLE_NAME => {
                new.get_bool(&mut segments.phys_out.data.out_assign_to_grp);
                req.write_segment(&mut unit.get_node(), &mut segments.phys_out, timeout_ms)
                    .map(|_| true)
            }
            OUT_GRP_SRC_TRIM_NAME => {
                Self::write_out_src_param(unit, req, segments, new, old, timeout_ms, |param, val| {
                    param.vol = val;
                    Ok(())
                })
                .and_then(|_| req.write_segment(&mut unit.get_node(), &mut segments.phys_out, timeout_ms))
                .map(|_| true)
            }
            OUT_GRP_SRC_DELAY_NAME => {
                Self::write_out_src_param(unit, req, segments, new, old, timeout_ms, |param, val| {
                    param.delay = val;
                    Ok(())
                })
                .and_then(|_| req.write_segment(&mut unit.get_node(), &mut segments.phys_out, timeout_ms))
                .map(|_| true)
            }
            OUT_GRP_SRC_ASSIGN_NAME => {
                let mut vals = [false;STUDIO_PHYS_OUT_PAIR_COUNT * 2];
                new.get_bool(&mut vals);
                let count = vals.iter().filter(|&v| *v).count();
                if count > STUDIO_MAX_SURROUND_CHANNELS {
                    let msg = format!("Maximum {} channels are supported for surround channels, but {} given",
                                      STUDIO_MAX_SURROUND_CHANNELS, count);
                    Err(Error::new(FileError::Inval, &msg))
                } else {
                    let index = elem_id.get_index() as usize;
                    segments.phys_out.data.out_grps[index].assigned_phys_outs.copy_from_slice(&vals);
                    req.write_segment(&mut unit.get_node(), &mut segments.phys_out, timeout_ms)
                        .map(|_| true)
                }
            }
            OUT_GRP_BASS_MANAGEMENT_NAME => {
                ElemValueAccessor::<bool>::get_vals(new, old, STUDIO_OUTPUT_GROUP_COUNT, |idx, val| {
                    segments.phys_out.data.out_grps[idx].bass_management = val;
                    Ok(())
                })
                .and_then(|_| req.write_segment(&mut unit.get_node(), &mut segments.phys_out, timeout_ms))
                .map(|_| true)
            }
            OUT_GRP_MAIN_CROSS_OVER_FREQ_NAME => {
                ElemValueAccessor::<u32>::get_vals(new, old, STUDIO_OUTPUT_GROUP_COUNT, |idx, val| {
                    Self::CROSS_OVER_FREQS.iter()
                        .nth(val as usize)
                        .ok_or_else(|| {
                            let msg = format!("Invalid value for index of cross over frequency: {}", val);
                            Error::new(FileError::Inval, &msg)
                        })
                        .map(|&freq| segments.phys_out.data.out_grps[idx].main_cross_over_freq = freq)
                })
                .and_then(|_| req.write_segment(&mut unit.get_node(), &mut segments.phys_out, timeout_ms))
                .map(|_| true)
            }
            OUT_GRP_MAIN_LEVEL_TO_SUB_NAME => {
                ElemValueAccessor::<i32>::get_vals(new, old, STUDIO_OUTPUT_GROUP_COUNT, |idx, val| {
                    segments.phys_out.data.out_grps[idx].main_level_to_sub = val;
                    Ok(())
                })
                .and_then(|_| req.write_segment(&mut unit.get_node(), &mut segments.phys_out, timeout_ms))
                .map(|_| true)
            }
            OUT_GRP_SUB_LEVEL_TO_SUB_NAME => {
                ElemValueAccessor::<i32>::get_vals(new, old, STUDIO_OUTPUT_GROUP_COUNT, |idx, val| {
                    segments.phys_out.data.out_grps[idx].sub_level_to_sub = val;
                    Ok(())
                })
                .and_then(|_| req.write_segment(&mut unit.get_node(), &mut segments.phys_out, timeout_ms))
                .map(|_| true)
            }
            OUT_GRP_MAIN_FILTER_FOR_MAIN_NAME => {
                ElemValueAccessor::<u32>::get_vals(new, old, STUDIO_OUTPUT_GROUP_COUNT, |idx, val| {
                    Self::HIGH_PASS_FREQS.iter()
                        .nth(val as usize)
                        .ok_or_else(|| {
                            let msg = format!("Invalid value for index of high pass frequency: {}", val);
                            Error::new(FileError::Inval, &msg)
                        })
                        .map(|&freq| segments.phys_out.data.out_grps[idx].main_filter_for_main = freq)
                })
                .and_then(|_| req.write_segment(&mut unit.get_node(), &mut segments.phys_out, timeout_ms))
                .map(|_| true)
            }
            OUT_GRP_MAIN_FILTER_FOR_SUB_NAME => {
                ElemValueAccessor::<u32>::get_vals(new, old, STUDIO_OUTPUT_GROUP_COUNT, |idx, val| {
                    Self::LOW_PASS_FREQS.iter()
                        .nth(val as usize)
                        .ok_or_else(|| {
                            let msg = format!("Invalid value for index of low pass frequency: {}", val);
                            Error::new(FileError::Inval, &msg)
                        })
                        .map(|&freq| segments.phys_out.data.out_grps[idx].main_filter_for_sub = freq)
                })
                .and_then(|_| req.write_segment(&mut unit.get_node(), &mut segments.phys_out, timeout_ms))
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }

    fn write_out_src_param<T, F>(
        unit: &mut SndDice,
        req: &mut FwReq,
        segments: &mut StudioSegments,
        new: &ElemValue,
        old: &ElemValue,
        timeout_ms: u32,
        cb: F
    ) -> Result<bool, Error>
        where F: Fn(&mut PhysOutSrcParam, T) -> Result<(), Error>,
              T: Default + Copy + Eq,
              ElemValue: ElemValueAccessor<T>,
    {
        ElemValueAccessor::<T>::get_vals(new, old, STUDIO_PHYS_OUT_PAIR_COUNT * 2, |idx, val| {
            let i = idx / 2;
            let ch = idx % 2;
            let param = if ch == 0 {
                &mut segments.phys_out.data.out_pair_srcs[i].left
            } else {
                &mut segments.phys_out.data.out_pair_srcs[i].right
            };
            cb(param, val)
        })
        .and_then(|_| req.write_segment(&mut unit.get_node(), &mut segments.phys_out, timeout_ms))
        .map(|_| true)
    }
}

fn src_pair_mode_to_str(entry: &MonitorSrcPairMode) -> &'static str {
    match entry {
        MonitorSrcPairMode::Inactive => "Inactive",
        MonitorSrcPairMode::Active => "Active",
        MonitorSrcPairMode::Fixed => "Fixed",
    }
}

#[derive(Default, Debug)]
pub struct MixerCtl{
    notified_elem_list: Vec<ElemId>,
    measured_elem_list: Vec<ElemId>,
}

const SRC_PAIR_MODE_NAME: &str = "mixer-input-mode";
const SRC_ENTRY_NAME: &str = "mixer-input-source";
const SRC_STEREO_LINK_NAME: &str = "mixer-input-stereo-link";
const SRC_GAIN_NAME: &str = "mixer-input-gain";
const SRC_PAN_NAME: &str = "mixer-input-pan";
const REVERB_SRC_GAIN_NAME: &str = "reverb-input-gain";
const AUX01_SRC_GAIN_NAME: &str = "aux-1/2-input-gain";
const AUX23_SRC_GAIN_NAME: &str = "aux-3/4-input-gain";
const SRC_MUTE_NAME: &str = "mixer-input-mute";

const OUT_DIM_NAME: &str = "mixer-output-dim";
const OUT_VOL_NAME: &str = "mixer-output-volume";
const OUT_DIM_VOL_NAME: &str = "mixer-output-dim-volume";
const REVERB_RETURN_MUTE_NAME: &str = "reverb-return-mute";
const REVERB_RETURN_GAIN_NAME: &str = "reverb-return-gain";

const POST_FADER_NAME: &str = "mixer-post-fader";

const CH_STRIP_AS_PLUGIN_NAME: &str = "channel-strip-as-plugin";
const CH_STRIP_SRC_NAME: &str = "channel-strip-source";
const CH_STRIP_23_AT_MID_RATE: &str = "channel-strip-3/4-at-mid-rate";

const MIXER_ENABLE_NAME: &str = "mixer-direct-monitoring";

const MIXER_INPUT_METER_NAME: &str = "mixer-input-meter";
const MIXER_OUTPUT_METER_NAME: &str = "mixer-output-meter";
const AUX_OUTPUT_METER_NAME: &str = "aux-output-meter";

impl MixerCtl {
    const SRC_PAIR_MODES: [MonitorSrcPairMode; 3] = [
        MonitorSrcPairMode::Inactive,
        MonitorSrcPairMode::Active,
        MonitorSrcPairMode::Fixed,
    ];

    const SRC_PAIR_ENTRIES: [SrcEntry; 51] = [
        SrcEntry::Unused,
        SrcEntry::Analog(0), SrcEntry::Analog(1), SrcEntry::Analog(2), SrcEntry::Analog(3),
        SrcEntry::Analog(4), SrcEntry::Analog(5), SrcEntry::Analog(6), SrcEntry::Analog(7),
        SrcEntry::Analog(8), SrcEntry::Analog(9), SrcEntry::Analog(10), SrcEntry::Analog(11),
        SrcEntry::Spdif(0), SrcEntry::Spdif(1),
        SrcEntry::Adat(0), SrcEntry::Adat(1), SrcEntry::Adat(2), SrcEntry::Adat(3),
        SrcEntry::Adat(4), SrcEntry::Adat(5), SrcEntry::Adat(6), SrcEntry::Adat(7),
        SrcEntry::StreamA(0), SrcEntry::StreamA(1), SrcEntry::StreamA(2), SrcEntry::StreamA(3),
        SrcEntry::StreamA(4), SrcEntry::StreamA(5), SrcEntry::StreamA(6), SrcEntry::StreamA(7),
        SrcEntry::StreamA(8), SrcEntry::StreamA(9), SrcEntry::StreamA(10), SrcEntry::StreamA(11),
        SrcEntry::StreamA(12), SrcEntry::StreamA(13), SrcEntry::StreamA(14), SrcEntry::StreamA(15),
        SrcEntry::StreamB(0), SrcEntry::StreamB(1), SrcEntry::StreamB(2), SrcEntry::StreamB(3),
        SrcEntry::StreamB(4), SrcEntry::StreamB(5), SrcEntry::StreamB(6), SrcEntry::StreamB(7),
        SrcEntry::StreamB(8), SrcEntry::StreamB(9), SrcEntry::StreamB(10), SrcEntry::StreamB(11),
    ];

    const OUT_LABELS: [&'static str; 3] = ["Main-1/2", "Aux-1/2", "Aux-3/4"];
    const SEND_TARGET_LABELS: [&'static str; 3] = ["Reverb-1/2", "Aux-1/2", "Aux-3/4"];

    const LEVEL_MIN: i32 = -1000;
    const LEVEL_MAX: i32 = 0;
    const LEVEL_STEP: i32 = 1;
    const LEVEL_TLV: DbInterval = DbInterval{min: -7200, max: 0, linear: false, mute_avail: false};

    const PAN_MIN: i32 = -50;
    const PAN_MAX: i32 = 50;
    const PAN_STEP: i32 = 1;

    fn load(&mut self, segments: &StudioSegments, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let state = &segments.mixer_state.data;
        let labels: Vec<String> = (0..state.src_pairs.len())
            .map(|i| format!("Mixer-source-{}/{}", i + 1, i + 2))
            .collect();
        let item_labels: Vec<&str> = Self::SRC_PAIR_MODES.iter()
            .map(|m| src_pair_mode_to_str(m))
            .collect();
        self.state_add_elem_enum(card_cntr, SRC_PAIR_MODE_NAME, 1, labels.len(), &item_labels)?;
        self.state_add_elem_bool(card_cntr, SRC_STEREO_LINK_NAME, 1, labels.len())?;

        let labels: Vec<String> = (0..(state.src_pairs.len() * 2))
            .map(|i| format!("Mixer-source-{}", i + 1))
            .collect();
        let item_labels: Vec<String> = Self::SRC_PAIR_ENTRIES.iter()
            .map(|s| src_pair_entry_to_string(s))
            .collect();
        self.state_add_elem_enum(card_cntr, SRC_ENTRY_NAME, 1, labels.len(), &item_labels)?;
        self.state_add_elem_level(card_cntr, SRC_GAIN_NAME, 1, labels.len())?;
        self.state_add_elem_pan(card_cntr, SRC_PAN_NAME, 1, labels.len())?;
        self.state_add_elem_level(card_cntr, REVERB_SRC_GAIN_NAME, 1, labels.len())?;
        self.state_add_elem_level(card_cntr, AUX01_SRC_GAIN_NAME, 1, labels.len())?;
        self.state_add_elem_level(card_cntr, AUX23_SRC_GAIN_NAME, 1, labels.len())?;
        self.state_add_elem_bool(card_cntr, SRC_MUTE_NAME, 1, labels.len())?;

        let labels = &Self::OUT_LABELS;
        self.state_add_elem_bool(card_cntr, REVERB_RETURN_MUTE_NAME, 1, labels.len())?;
        self.state_add_elem_level(card_cntr, REVERB_RETURN_GAIN_NAME, 1, labels.len())?;
        self.state_add_elem_bool(card_cntr, OUT_DIM_NAME, 1, labels.len())?;
        self.state_add_elem_level(card_cntr, OUT_VOL_NAME, 1, labels.len())?;
        self.state_add_elem_level(card_cntr, OUT_DIM_VOL_NAME, 1, labels.len())?;

        let labels = &Self::SEND_TARGET_LABELS;
        self.state_add_elem_bool(card_cntr, POST_FADER_NAME, 1, labels.len())?;

        let labels: Vec<String> = (0..2)
            .map(|i| format!("Channel-strip-{}/{}", i + 1, i + 2))
            .collect();
        self.state_add_elem_bool(card_cntr, CH_STRIP_AS_PLUGIN_NAME, 1, labels.len())?;
        let labels: Vec<String> = (0..4)
            .map(|i| format!("Channel-strip-{}", i))
            .collect();
        self.state_add_elem_enum(card_cntr, CH_STRIP_SRC_NAME, 1, labels.len(), &item_labels)?;
        self.state_add_elem_bool(card_cntr, CH_STRIP_23_AT_MID_RATE, 1, 1)?;

        self.state_add_elem_bool(card_cntr, MIXER_ENABLE_NAME, 1, 1)?;

        // For metering.
        let meter = &segments.mixer_meter.data;
        let labels: Vec<String> = (0..meter.src_inputs.len())
            .map(|i| format!("mixer-input-{}", i))
            .collect();
        self.meter_add_elem_level(card_cntr, MIXER_INPUT_METER_NAME, labels.len())?;

        let labels: Vec<String> = (0..meter.mixer_outputs.len())
            .map(|i| format!("mixer-output-{}", i))
            .collect();
        self.meter_add_elem_level(card_cntr, MIXER_OUTPUT_METER_NAME, labels.len())?;

        let labels: Vec<String> = (0..meter.mixer_outputs.len())
            .map(|i| format!("aux-output-{}", i))
            .collect();
        self.meter_add_elem_level(card_cntr, AUX_OUTPUT_METER_NAME, labels.len())?;

        Ok(())
    }

    fn state_add_elem_enum<T: AsRef<str>>(
        &mut self,
        card_cntr: &mut CardCntr,
        name: &str,
        count: usize,
        value_count: usize,
        labels: &[T]
    ) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
        card_cntr.add_enum_elems(&elem_id, count, value_count, &labels, None, true)
            .map(|mut elem_id_list| self.notified_elem_list.append(&mut elem_id_list))
    }

    fn state_add_elem_bool(
        &mut self,
        card_cntr: &mut CardCntr,
        name: &str,
        count: usize,
        value_count: usize
    ) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
        card_cntr.add_bool_elems(&elem_id, count, value_count, true)
            .map(|mut elem_id_list| self.notified_elem_list.append(&mut elem_id_list))
    }

    fn state_add_elem_level(
        &mut self,
        card_cntr: &mut CardCntr,
        name: &str,
        count: usize,
        value_count: usize
    ) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
        card_cntr.add_int_elems(&elem_id, count, Self::LEVEL_MIN, Self::LEVEL_MAX, Self::LEVEL_STEP,
                                value_count, Some(&Into::<Vec<u32>>::into(Self::LEVEL_TLV)), true)
            .map(|mut elem_id_list| self.notified_elem_list.append(&mut elem_id_list))
    }

    fn state_add_elem_pan(
        &mut self,
        card_cntr: &mut CardCntr,
        name: &str,
        count: usize,
        value_count: usize
    ) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
        card_cntr.add_int_elems(&elem_id, count, Self::PAN_MIN, Self::PAN_MAX, Self::PAN_STEP, value_count,
                                None, true)
            .map(|mut elem_id_list| self.notified_elem_list.append(&mut elem_id_list))
    }

    fn meter_add_elem_level(
        &mut self,
        card_cntr: &mut CardCntr,
        name: &str,
        value_count: usize
    ) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
        card_cntr.add_int_elems(&elem_id, 1, Self::LEVEL_MIN, Self::LEVEL_MAX, Self::LEVEL_STEP,
                                value_count, Some(&Into::<Vec<u32>>::into(Self::LEVEL_TLV)), false)
            .map(|mut elem_id_list| self.measured_elem_list.append(&mut elem_id_list))
    }

    fn read(
        &mut self,
        segments: &StudioSegments,
        elem_id: &ElemId,
        elem_value: &mut ElemValue
    ) -> Result<bool, Error> {
        if self.read_notified_elem(segments, elem_id, elem_value)? {
            Ok(true)
        } else if self.read_measured_elem(segments, elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn state_read_src_param<T, F>(
        segments: &StudioSegments,
        elem_value: &ElemValue,
        cb: F
    ) -> Result<bool, Error>
        where T: Default + Copy + Eq,
              F: Fn(&MonitorSrcParam) -> Result<T, Error>,
              ElemValue: ElemValueAccessor<T>,
    {
        let count = segments.mixer_state.data.src_pairs.len() * 2;
        ElemValueAccessor::<T>::set_vals(elem_value, count, |idx| {
            let i = idx / 2;
            let ch = idx % 2;
            let param = if ch == 0 {
                &segments.mixer_state.data.src_pairs[i].left
            } else {
                &segments.mixer_state.data.src_pairs[i].right
            };
            cb(param)
        })
        .map(|_| true)
    }

    fn state_read_out_pair<T, F>(
        segments: &StudioSegments,
        elem_value: &ElemValue,
        cb: F
    ) -> Result<bool, Error>
        where T: Copy + Default + Eq + PartialEq,
              F: Fn(&OutPair) -> Result<T, Error>,
              ElemValue: ElemValueAccessor<T>,
    {
        ElemValueAccessor::<T>::set_vals(elem_value, Self::OUT_LABELS.len(), |idx| {
            cb(&segments.mixer_state.data.mixer_out[idx])
        })
        .map(|_| true)
    }

    fn write(
        &mut self,
        unit: &mut SndDice,
        req: &mut FwReq,
        segments: &mut StudioSegments,
        elem_id: &ElemId,
        old: &ElemValue,
        new: &ElemValue,
        timeout_ms: u32
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            SRC_PAIR_MODE_NAME => {
                let state = &mut segments.mixer_state.data;
                ElemValueAccessor::<u32>::get_vals(new, old, state.src_pairs.len(), |idx, val| {
                    if let Some(m) = Self::SRC_PAIR_MODES.iter().nth(val as usize) {
                        if state.src_pairs[idx].mode != MonitorSrcPairMode::Fixed {
                            if *m != MonitorSrcPairMode::Fixed {
                                state.src_pairs[idx].mode = *m;
                                Ok(())
                            } else {
                                let msg = format!("The fixed mode is not newly available: {}", idx);
                                Err(Error::new(FileError::Inval, &msg))
                            }
                        } else {
                            let msg = format!("The source of mixer is immutable: {}", idx);
                            Err(Error::new(FileError::Inval, &msg))
                        }
                    } else {
                        let msg = format!("Invalid value for index of mixer source: {}", val);
                        Err(Error::new(FileError::Inval, &msg))
                    }
                })
                .and_then(|_| req.write_segment(&mut unit.get_node(), &mut segments.mixer_state, timeout_ms))
                .map(|_| true)
            }
            SRC_ENTRY_NAME => {
                Self::state_write_src_param(unit, req, segments, new, old, timeout_ms, |param, val: u32| {
                    Self::SRC_PAIR_ENTRIES.iter()
                        .nth(val as usize)
                        .ok_or_else(|| {
                            let msg = format!("Invalid value for index of mixer source: {}", val);
                            Error::new(FileError::Inval, &msg)
                        })
                        .map(|&s| param.src = s)
                })
            }
            SRC_STEREO_LINK_NAME => {
                let pair_count = segments.mixer_state.data.src_pairs.len();
                ElemValueAccessor::<bool>::get_vals(new, old, pair_count, |idx, val| {
                    segments.mixer_state.data.src_pairs[idx].stereo_link = val;
                    Ok(())
                })
                .and_then(|_| req.write_segment(&mut unit.get_node(), &mut segments.mixer_state, timeout_ms))
                .map(|_| true)
            }
            SRC_GAIN_NAME => {
                Self::state_write_src_param(unit, req, segments, new, old, timeout_ms, |param, val| {
                    param.gain_to_main = val;
                    Ok(())
                })
            }
            SRC_PAN_NAME => {
                Self::state_write_src_param(unit, req, segments, new, old, timeout_ms, |param, val| {
                    param.pan_to_main = val;
                    Ok(())
                })
            }
            REVERB_SRC_GAIN_NAME => {
                Self::state_write_src_param(unit, req, segments, new, old, timeout_ms, |param, val| {
                    param.gain_to_reverb = val;
                    Ok(())
                })
            }
            AUX01_SRC_GAIN_NAME => {
                Self::state_write_src_param(unit, req, segments, new, old, timeout_ms, |param, val| {
                    param.gain_to_aux0 = val;
                    Ok(())
                })
            }
            AUX23_SRC_GAIN_NAME => {
                Self::state_write_src_param(unit, req, segments, new, old, timeout_ms, |param, val| {
                    param.gain_to_aux1 = val;
                    Ok(())
                })
            }
            SRC_MUTE_NAME => {
                new.get_bool(&mut segments.mixer_state.data.mutes);
                req.write_segment(&mut unit.get_node(), &mut segments.mixer_state, timeout_ms)
                    .map(|_| true)
            }
            OUT_DIM_NAME => {
                Self::state_write_out_pair(unit, req, segments, new, old, timeout_ms, |pair, val| {
                    pair.dim_enabled = val;
                    Ok(())
                })
            }
            OUT_VOL_NAME=> {
                Self::state_write_out_pair(unit, req, segments, new, old, timeout_ms, |pair, val| {
                    pair.vol = val;
                    Ok(())
                })
            }
            OUT_DIM_VOL_NAME => {
                Self::state_write_out_pair(unit, req, segments, new, old, timeout_ms, |pair, val| {
                    pair.dim_vol = val;
                    Ok(())
                })
            }
            REVERB_RETURN_MUTE_NAME => {
                new.get_bool(&mut segments.mixer_state.data.reverb_return_mute);
                req.write_segment(&mut unit.get_node(), &mut segments.mixer_state, timeout_ms)
                    .map(|_| true)
            }
            REVERB_RETURN_GAIN_NAME => {
                new.get_int(&mut segments.mixer_state.data.reverb_return_gain);
                req.write_segment(&mut unit.get_node(), &mut segments.mixer_state, timeout_ms)
                    .map(|_| true)
            }
            CH_STRIP_AS_PLUGIN_NAME => {
                new.get_bool(&mut segments.mixer_state.data.ch_strip_as_plugin);
                req.write_segment(&mut unit.get_node(), &mut segments.mixer_state, timeout_ms)
                    .map(|_| true)
            }
            CH_STRIP_SRC_NAME => {
                let count = segments.mixer_state.data.ch_strip_src.len();
                ElemValueAccessor::<u32>::get_vals(new, old, count, |idx, val| {
                    Self::SRC_PAIR_ENTRIES.iter()
                        .nth(val as usize)
                        .ok_or_else(|| {
                            let msg = format!("Invalid value for index of ch strip source: {}", val);
                            Error::new(FileError::Inval, &msg)
                        })
                        .map(|&s| segments.mixer_state.data.ch_strip_src[idx] = s)
                })
                .and_then(|_| req.write_segment(&mut unit.get_node(), &mut segments.mixer_state, timeout_ms))
                .map(|_| true)
            }
            CH_STRIP_23_AT_MID_RATE => {
                ElemValueAccessor::<bool>::get_val(new, |val| {
                    segments.mixer_state.data.ch_strip_23_at_mid_rate = val;
                    Ok(())
                })
                .and_then(|_| req.write_segment(&mut unit.get_node(), &mut segments.mixer_state, timeout_ms))
                .map(|_| true)
            }
            POST_FADER_NAME => {
                new.get_bool(&mut segments.mixer_state.data.post_fader);
                req.write_segment(&mut unit.get_node(), &mut segments.mixer_state, timeout_ms)
                    .map(|_| true)
            }
            MIXER_ENABLE_NAME => {
                ElemValueAccessor::<bool>::get_val(new, |val| {
                    segments.mixer_state.data.enabled = val;
                    Ok(())
                })
                .and_then(|_| req.write_segment(&mut unit.get_node(), &mut segments.mixer_state, timeout_ms))
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }

    fn state_write_src_param<T, F>(
        unit: &mut SndDice,
        req: &mut FwReq,
        segments: &mut StudioSegments,
        new: &ElemValue,
        old: &ElemValue,
        timeout_ms: u32,
        cb: F
    ) -> Result<bool, Error>
        where T: Default + Copy + Eq,
              F: Fn(&mut MonitorSrcParam, T) -> Result<(), Error>,
              ElemValue: ElemValueAccessor<T>,
    {
        let count = segments.mixer_state.data.src_pairs.len() * 2;
        ElemValueAccessor::<T>::get_vals(new, old, count, |idx, val| {
            let i = idx / 2;
            let ch = idx % 2;
            let param = if ch == 0 {
                &mut segments.mixer_state.data.src_pairs[i].left
            } else {
                &mut segments.mixer_state.data.src_pairs[i].right
            };
            cb(param, val)
        })
        .and_then(|_| req.write_segment(&mut unit.get_node(), &mut segments.mixer_state, timeout_ms))
        .map(|_| true)
    }

    fn state_write_out_pair<T, F>(
        unit: &mut SndDice,
        req: &mut FwReq,
        segments: &mut StudioSegments,
        new: &ElemValue,
        old: &ElemValue,
        timeout_ms: u32,
        cb: F
    ) -> Result<bool, Error>
        where T: Default + Copy + Eq,
              F: Fn(&mut OutPair, T) -> Result<(), Error>,
              ElemValue: ElemValueAccessor<T>,
    {
        ElemValueAccessor::<T>::get_vals(new, old, Self::OUT_LABELS.len(), |idx, val| {
            cb(&mut segments.mixer_state.data.mixer_out[idx], val)
        })
        .and_then(|_| req.write_segment(&mut unit.get_node(), &mut segments.mixer_state, timeout_ms))
        .map(|_| true)
    }

    fn read_notified_elem(
        &self,
        segments: &StudioSegments,
        elem_id: &ElemId,
        elem_value: &mut ElemValue
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            SRC_PAIR_MODE_NAME => {
                let pair_count = segments.mixer_state.data.src_pairs.len();
                ElemValueAccessor::<u32>::set_vals(elem_value, pair_count, |idx| {
                    let pos = Self::SRC_PAIR_MODES.iter()
                        .position(|m| m.eq(&segments.mixer_state.data.src_pairs[idx].mode))
                        .expect("Programming error...");
                    Ok(pos as u32)
                })
                .map(|_| true)
            }
            SRC_STEREO_LINK_NAME => {
                let pair_count = segments.mixer_state.data.src_pairs.len();
                ElemValueAccessor::<bool>::set_vals(elem_value, pair_count, |idx| {
                    Ok(segments.mixer_state.data.src_pairs[idx].stereo_link)
                })
                .map(|_| true)
            }
            SRC_ENTRY_NAME => {
                Self::state_read_src_param(segments, elem_value, |param| {
                    let pos = Self::SRC_PAIR_ENTRIES.iter()
                        .position(|m| m.eq(&param.src))
                        .expect("Programming error...");
                    Ok(pos as u32)
                })
            }
            SRC_GAIN_NAME => {
                Self::state_read_src_param(segments, elem_value, |param| Ok(param.gain_to_main))
            }
            SRC_PAN_NAME => {
                Self::state_read_src_param(segments, elem_value, |param| Ok(param.pan_to_main))
            }
            REVERB_SRC_GAIN_NAME => {
                Self::state_read_src_param(segments, elem_value, |param| Ok(param.gain_to_reverb))
            }
            AUX01_SRC_GAIN_NAME => {
                Self::state_read_src_param(segments, elem_value, |param| Ok(param.gain_to_aux0))
            }
            AUX23_SRC_GAIN_NAME => {
                Self::state_read_src_param(segments, elem_value, |param| Ok(param.gain_to_aux1))
            }
            SRC_MUTE_NAME => {
                elem_value.set_bool(&segments.mixer_state.data.mutes);
                Ok(true)
            }
            OUT_DIM_NAME => {
                Self::state_read_out_pair(segments, elem_value, |pair| Ok(pair.dim_enabled))
            }
            OUT_VOL_NAME => {
                Self::state_read_out_pair(segments, elem_value, |pair| Ok(pair.vol))
            }
            OUT_DIM_VOL_NAME => {
                Self::state_read_out_pair(segments, elem_value, |pair| Ok(pair.dim_vol))
            }
            REVERB_RETURN_MUTE_NAME => {
                elem_value.set_bool(&segments.mixer_state.data.reverb_return_mute);
                Ok(true)
            }
            REVERB_RETURN_GAIN_NAME => {
                elem_value.set_int(&segments.mixer_state.data.reverb_return_gain);
                Ok(true)
            }
            POST_FADER_NAME => {
                elem_value.set_bool(&segments.mixer_state.data.post_fader);
                Ok(true)
            }
            CH_STRIP_AS_PLUGIN_NAME => {
                elem_value.set_bool(&segments.mixer_state.data.ch_strip_as_plugin);
                Ok(true)
            }
            CH_STRIP_SRC_NAME => {
                let count = segments.mixer_state.data.ch_strip_src.len();
                ElemValueAccessor::<u32>::set_vals(elem_value, count, |idx| {
                    let pos = Self::SRC_PAIR_ENTRIES.iter()
                        .position(|s| s.eq(&segments.mixer_state.data.ch_strip_src[idx]))
                        .expect("Programming error...");
                    Ok(pos as u32)
                })
                .map(|_| true)
            }
            CH_STRIP_23_AT_MID_RATE => {
                elem_value.set_bool(&[segments.mixer_state.data.ch_strip_23_at_mid_rate]);
                Ok(true)
            }
            MIXER_ENABLE_NAME => {
                elem_value.set_bool(&[segments.mixer_state.data.enabled]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn read_measured_elem(
        &self,
        segments: &StudioSegments,
        elem_id: &ElemId,
        elem_value: &mut ElemValue
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            MIXER_INPUT_METER_NAME => {
                elem_value.set_int(&segments.mixer_meter.data.src_inputs);
                Ok(true)
            }
            MIXER_OUTPUT_METER_NAME => {
                elem_value.set_int(&segments.mixer_meter.data.mixer_outputs);
                Ok(true)
            }
            AUX_OUTPUT_METER_NAME => {
                elem_value.set_int(&segments.mixer_meter.data.aux_outputs);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

fn opt_iface_mode_to_str(mode: &OptIfaceMode) -> &'static str {
    match mode {
        OptIfaceMode::Adat => "ADAT",
        OptIfaceMode::Spdif => "S/PDIF",
    }
}

fn standalone_clk_src_to_str(src: &StudioStandaloneClkSrc) -> &'static str {
    match src {
        StudioStandaloneClkSrc::Adat => "ADAT",
        StudioStandaloneClkSrc::SpdifOnOpt01 => "S/PDIF-opt-1/2",
        StudioStandaloneClkSrc::SpdifOnOpt23 => "S/PDIF-opt-3/4",
        StudioStandaloneClkSrc::SpdifOnCoax => "S/PDIF-coax",
        StudioStandaloneClkSrc::WordClock => "Word-clock",
        StudioStandaloneClkSrc::Internal => "Internal",
    }
}

#[derive(Default, Debug)]
pub struct ConfigCtl{
    standalone_rate: TcKonnektStandaloneCtl,
    midi_send: MidiSendCtl,
}

const OPT_IFACE_MODE_NAME: &str = "opt-iface-mode";
const STANDALONE_CLK_SRC_NAME: &str = "standalone-clock-source";
const CLOCK_RECOVERY_NAME: &str = "clock-recovery";

impl ConfigCtl {
    const OPT_IFACE_MODES: [OptIfaceMode; 2] = [OptIfaceMode::Adat, OptIfaceMode::Spdif];

    const STANDALONE_CLK_SRCS: [StudioStandaloneClkSrc; 6] = [
        StudioStandaloneClkSrc::Adat,
        StudioStandaloneClkSrc::SpdifOnOpt01,
        StudioStandaloneClkSrc::SpdifOnOpt23,
        StudioStandaloneClkSrc::SpdifOnCoax,
        StudioStandaloneClkSrc::WordClock,
        StudioStandaloneClkSrc::Internal,
    ];

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let labels: Vec<&str> = Self::OPT_IFACE_MODES.iter()
            .map(|m| opt_iface_mode_to_str(m))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, OPT_IFACE_MODE_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let labels: Vec<&str> = Self::STANDALONE_CLK_SRCS.iter()
            .map(|r| standalone_clk_src_to_str(r))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, STANDALONE_CLK_SRC_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        self.standalone_rate.load(card_cntr)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, CLOCK_RECOVERY_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        self.midi_send.load(card_cntr)?;

        Ok(())
    }

    fn read(
        &mut self,
        segments: &StudioSegments,
        elem_id: &ElemId,
        elem_value: &mut ElemValue
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            OPT_IFACE_MODE_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let pos = Self::OPT_IFACE_MODES.iter()
                        .position(|m| m.eq(&segments.config.data.opt_iface_mode))
                        .expect("Programming error...");
                    Ok(pos as u32)
                })
                .map(|_| true)
            }
            STANDALONE_CLK_SRC_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let pos = Self::STANDALONE_CLK_SRCS.iter()
                        .position(|s| s.eq(&segments.config.data.standalone_src))
                        .expect("Programming error...");
                    Ok(pos as u32)
                })
                .map(|_| true)
            }
            CLOCK_RECOVERY_NAME => {
                ElemValueAccessor::<bool>::set_val(elem_value, || Ok(segments.config.data.clock_recovery))
                .map(|_| true)
            }
            _ => {
                if self.standalone_rate.read(&segments.config, elem_id, elem_value)? {
                    Ok(true)
                } else if self.midi_send.read(&segments.config, elem_id, elem_value)? {
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
        segments: &mut StudioSegments,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            OPT_IFACE_MODE_NAME => {
                ElemValueAccessor::<u32>::get_val(elem_value, |val| {
                    Self::OPT_IFACE_MODES.iter()
                        .nth(val as usize)
                        .ok_or_else(|| {
                            let msg = format!("Invalid value for index of standalone clock source: {}", val);
                            Error::new(FileError::Inval, &msg)
                        })
                        .map(|&m| segments.config.data.opt_iface_mode = m)
                })
                .and_then(|_| req.write_segment(&mut unit.get_node(), &mut segments.config, timeout_ms))
                .map(|_| true)
            }
            STANDALONE_CLK_SRC_NAME => {
                ElemValueAccessor::<u32>::get_val(elem_value, |val| {
                    Self::STANDALONE_CLK_SRCS.iter()
                        .nth(val as usize)
                        .ok_or_else(|| {
                            let msg = format!("Invalid value for index of standalone clock source: {}", val);
                            Error::new(FileError::Inval, &msg)
                        })
                        .map(|&s| segments.config.data.standalone_src = s)
                })
                .and_then(|_| req.write_segment(&mut unit.get_node(), &mut segments.config, timeout_ms))
                .map(|_| true)
            }
            CLOCK_RECOVERY_NAME => {
                ElemValueAccessor::<bool>::get_val(elem_value, |val| {
                    segments.config.data.clock_recovery = val;
                    Ok(())
                })
                .and_then(|_| req.write_segment(&mut unit.get_node(), &mut segments.config, timeout_ms))
                .map(|_| true)
            }
            _ => {
                if self.standalone_rate.write(unit, req, &mut segments.config, elem_id, elem_value,
                                              timeout_ms)? {
                    Ok(true)
                } else if self.midi_send.write(unit, req, &mut segments.config, elem_id, elem_value,
                                               timeout_ms)? {
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
        }
    }
}

fn effect_button_mode_to_str(mode: &RemoteEffectButtonMode) -> &'static str {
    match mode {
        RemoteEffectButtonMode::Reverb => "Reverb",
        RemoteEffectButtonMode::Midi => "Midi",
    }
}

fn knob_push_mode_to_str(mode: &KnobPushMode) -> &'static str {
    match mode {
        KnobPushMode::Pan => "Pan",
        KnobPushMode::GainToReverb => "Reverb",
        KnobPushMode::GainToAux0 => "Aux-1/2",
        KnobPushMode::GainToAux1 => "Aux-3/4",
    }
}

#[derive(Default, Debug)]
pub struct RemoteCtl{
    notified_elem_list: Vec<ElemId>,
    prog_ctl: TcKonnektProgramCtl,
}

const USER_ASSIGN_NAME: &str = "remote-user-assign";
const EFFECT_BUTTON_MODE_NAME: &str = "remote-effect-button-mode";
const FALLBACK_TO_MASTER_ENABLE_NAME: &str = "remote-fallback-to-master-enable";
const FALLBACK_TO_MASTER_DURATION_NAME: &str = "remote-fallback-to-master-duration";
const KNOB_PUSH_MODE_NAME: &str = "remote-knob-push-mode";

impl RemoteCtl {
    const EFFECT_BUTTON_MODES: [RemoteEffectButtonMode; 2] = [
        RemoteEffectButtonMode::Reverb,
        RemoteEffectButtonMode::Midi,
    ];

    // NOTE: by milisecond.
    const DURATION_MIN: i32 = 10;
    const DURATION_MAX: i32 = 1000;
    const DURATION_STEP: i32 = 1;

    const KNOB_PUSH_MODES: [KnobPushMode; 4] = [
        KnobPushMode::Pan,
        KnobPushMode::GainToReverb,
        KnobPushMode::GainToAux0,
        KnobPushMode::GainToAux1,
    ];

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.prog_ctl.load(card_cntr)?;
        self.notified_elem_list.extend_from_slice(&self.prog_ctl.0);

        let labels: Vec<String> = MixerCtl::SRC_PAIR_ENTRIES.iter()
            .map(|src| src_pair_entry_to_string(src))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, USER_ASSIGN_NAME, 0);
        card_cntr.add_enum_elems(&elem_id, 1, STUDIO_REMOTE_USER_ASSIGN_COUNT, &labels, None, true)
            .map(|mut elem_id_list| self.notified_elem_list.append(&mut elem_id_list))?;

        let labels: Vec<&str> = Self::EFFECT_BUTTON_MODES.iter()
            .map(|m| effect_button_mode_to_str(m))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, EFFECT_BUTTON_MODE_NAME, 0);
        card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)
            .map(|mut elem_id_list| self.notified_elem_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, FALLBACK_TO_MASTER_ENABLE_NAME, 0);
        card_cntr.add_bool_elems(&elem_id, 1, 1, true)
            .map(|mut elem_id_list| self.notified_elem_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, FALLBACK_TO_MASTER_DURATION_NAME, 0);
        card_cntr.add_int_elems(&elem_id, 1, Self::DURATION_MIN, Self::DURATION_MAX, Self::DURATION_STEP,
                                1, None, true)
            .map(|mut elem_id_list| self.notified_elem_list.append(&mut elem_id_list))?;

        let labels: Vec<&str> = Self::KNOB_PUSH_MODES.iter()
            .map(|m| knob_push_mode_to_str(m))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, KNOB_PUSH_MODE_NAME, 0);
        card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)
            .map(|mut elem_id_list| self.notified_elem_list.append(&mut elem_id_list))?;

        Ok(())
    }

    fn read(
        &mut self,
        segments: &StudioSegments,
        elem_id: &ElemId,
        elem_value: &mut ElemValue
    ) -> Result<bool, Error> {
        self.read_notified_elem(segments, elem_id, elem_value)
    }

    fn write(
        &mut self,
        unit: &mut SndDice,
        req: &mut FwReq,
        segments: &mut StudioSegments,
        elem_id: &ElemId,
        old: &ElemValue,
        new: &ElemValue,
        timeout_ms: u32
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            USER_ASSIGN_NAME => {
                ElemValueAccessor::<u32>::get_vals(new, old, STUDIO_REMOTE_USER_ASSIGN_COUNT, |idx, val| {
                    MixerCtl::SRC_PAIR_ENTRIES.iter()
                        .nth(val as usize)
                        .ok_or_else(|| {
                            let msg = format!("Invalid value for index of source of user assignment: {}", val);
                            Error::new(FileError::Inval, &msg)
                        })
                        .map(|&s| segments.remote.data.user_assigns[idx] = s)
                })
                .and_then(|_| req.write_segment(&mut unit.get_node(), &mut segments.remote, timeout_ms))
                .map(|_| true)
            }
            EFFECT_BUTTON_MODE_NAME => {
                ElemValueAccessor::<u32>::get_val(new, |val| {
                    Self::EFFECT_BUTTON_MODES.iter()
                        .nth(val as usize)
                        .ok_or_else(|| {
                            let msg = format!("Invalid value for index of source of user assignment: {}", val);
                            Error::new(FileError::Inval, &msg)
                        })
                        .map(|&m| segments.remote.data.effect_button_mode = m)
                })
                .and_then(|_| req.write_segment(&mut unit.get_node(), &mut segments.remote, timeout_ms))
                .map(|_| true)
            }
            FALLBACK_TO_MASTER_ENABLE_NAME => {
                ElemValueAccessor::<bool>::get_val(new, |val| {
                    segments.remote.data.fallback_to_master_enable = val;
                    Ok(())
                })
                .and_then(|_| req.write_segment(&mut unit.get_node(), &mut segments.remote, timeout_ms))
                .map(|_| true)
            }
            FALLBACK_TO_MASTER_DURATION_NAME => {
                ElemValueAccessor::<i32>::get_val(new, |val| {
                    segments.remote.data.fallback_to_master_duration = val as u32;
                    Ok(())
                })
                .and_then(|_| req.write_segment(&mut unit.get_node(), &mut segments.remote, timeout_ms))
                .map(|_| true)
            }
            KNOB_PUSH_MODE_NAME => {
                ElemValueAccessor::<u32>::get_val(new, |val| {
                    Self::KNOB_PUSH_MODES.iter()
                        .nth(val as usize)
                        .ok_or_else(|| {
                            let msg = format!("Invalid value for index of source of user assignment: {}", val);
                            Error::new(FileError::Inval, &msg)
                        })
                        .map(|&m| segments.remote.data.knob_push_mode = m)
                })
                .and_then(|_| req.write_segment(&mut unit.get_node(), &mut segments.remote, timeout_ms))
                .map(|_| true)
            }
            _ => self.prog_ctl.write(unit, req, &mut segments.remote, elem_id, new, timeout_ms)
        }
    }

    fn read_notified_elem(
        &mut self,
        segments: &StudioSegments,
        elem_id: &ElemId,
        elem_value: &mut ElemValue
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            USER_ASSIGN_NAME => {
                ElemValueAccessor::<u32>::set_vals(elem_value, STUDIO_REMOTE_USER_ASSIGN_COUNT, |idx| {
                    let pos = MixerCtl::SRC_PAIR_ENTRIES.iter()
                        .position(|p| p.eq(&segments.remote.data.user_assigns[idx]))
                        .expect("Programming error...");
                    Ok(pos as u32)
                })
                .map(|_| true)
            }
            EFFECT_BUTTON_MODE_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let pos = Self::EFFECT_BUTTON_MODES.iter()
                        .position(|m| m.eq(&segments.remote.data.effect_button_mode))
                        .expect("Programming error");
                    Ok(pos as u32)
                })
                .map(|_| true)
            }
            FALLBACK_TO_MASTER_ENABLE_NAME => {
                ElemValueAccessor::<bool>::set_val(elem_value, || {
                    Ok(segments.remote.data.fallback_to_master_enable)
                })
                .map(|_| true)
            }
            FALLBACK_TO_MASTER_DURATION_NAME => {
                ElemValueAccessor::<i32>::set_val(elem_value, || {
                    Ok(segments.remote.data.fallback_to_master_duration as i32)
                })
                .map(|_| true)
            }
            KNOB_PUSH_MODE_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let pos = Self::KNOB_PUSH_MODES.iter()
                        .position(|m| m.eq(&segments.remote.data.knob_push_mode))
                        .expect("Programming error");
                    Ok(pos as u32)
                })
                .map(|_| true)
            }
            _ => self.prog_ctl.read(&segments.remote, elem_id, elem_value),
        }
    }
}

fn nominal_signal_level_to_str(level: &NominalSignalLevel) -> &'static str {
    match level {
        NominalSignalLevel::Professional => "+4dBu",
        NominalSignalLevel::Consumer => "-10dBV",
    }
}

#[derive(Default, Debug)]
pub struct LineoutCtl;

const LINE_OUT_45_LEVEL_NAME: &str = "line-out-5/6-level";
const LINE_OUT_67_LEVEL_NAME: &str = "line-out-7/8-level";
const LINE_OUT_89_LEVEL_NAME: &str = "line-out-9/10-level";
const LINE_OUT_1011_LEVEL_NAME: &str = "line-out-11/12-level";

impl LineoutCtl {
    const NOMINAL_SIGNAL_LEVELS: [NominalSignalLevel; 2] = [
        NominalSignalLevel::Professional,
        NominalSignalLevel::Consumer,
    ];

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let labels: Vec<&str> = Self::NOMINAL_SIGNAL_LEVELS.iter()
            .map(|m| nominal_signal_level_to_str(m))
            .collect();

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, LINE_OUT_45_LEVEL_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, LINE_OUT_67_LEVEL_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, LINE_OUT_89_LEVEL_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, LINE_OUT_1011_LEVEL_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        Ok(())
    }

    fn read(
        &mut self,
        segments: &StudioSegments,
        elem_id: &ElemId,
        elem_value: &mut ElemValue
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            LINE_OUT_45_LEVEL_NAME => {
                Self::read_as_index(elem_value, segments.out_level.data.line_45)
            }
            LINE_OUT_67_LEVEL_NAME => {
                Self::read_as_index(elem_value, segments.out_level.data.line_67)
            }
            LINE_OUT_89_LEVEL_NAME => {
                Self::read_as_index(elem_value, segments.out_level.data.line_89)
            }
            LINE_OUT_1011_LEVEL_NAME => {
                Self::read_as_index(elem_value, segments.out_level.data.line_1011)
            }
            _ => Ok(false),
        }
    }

    fn read_as_index(
        elem_value:&mut ElemValue,
        level: NominalSignalLevel
    ) -> Result<bool, Error> {
        ElemValueAccessor::<u32>::set_val(elem_value, || {
            let pos = Self::NOMINAL_SIGNAL_LEVELS.iter()
                .position(|l| l.eq(&level))
                .expect("Programming error.");
            Ok(pos as u32)
        })
        .map(|_| true)
    }

    fn write(
        &mut self,
        unit: &mut SndDice,
        req: &mut FwReq,
        segments: &mut StudioSegments,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            LINE_OUT_45_LEVEL_NAME => {
                Self::write_as_index(unit, req, segments, elem_value, timeout_ms, |data, level| {
                    data.line_45 = level
                })
            }
            LINE_OUT_67_LEVEL_NAME => {
                Self::write_as_index(unit, req, segments, elem_value, timeout_ms, |data, level| {
                    data.line_67 = level
                })
            }
            LINE_OUT_89_LEVEL_NAME => {
                Self::write_as_index(unit, req, segments, elem_value, timeout_ms, |data, level| {
                    data.line_89 = level
                })
            }
            LINE_OUT_1011_LEVEL_NAME => {
                Self::write_as_index(unit, req, segments, elem_value, timeout_ms, |data, level| {
                    data.line_1011 = level
                })
            }
            _ => Ok(false),
        }
    }

    fn write_as_index<F>(
        unit: &mut SndDice,
        req: &mut FwReq,
        segments: &mut StudioSegments,
        elem_value: &ElemValue,
        timeout_ms: u32,
        cb: F
    ) -> Result<bool, Error>
        where F: Fn(&mut StudioLineOutLevel, NominalSignalLevel),
    {
        ElemValueAccessor::<u32>::get_val(elem_value, |val| {
            Self::NOMINAL_SIGNAL_LEVELS.iter()
                .nth(val as usize)
                .ok_or_else(|| {
                    let msg = format!("Invalid value for index of nominal level: {}", val);
                    Error::new(FileError::Inval, &msg)
                })
                .map(|&l| cb(&mut segments.out_level.data, l))
        })
        .and_then(|_| req.write_segment(&mut unit.get_node(), &mut segments.out_level, timeout_ms))
        .map(|_| true)
    }
}
